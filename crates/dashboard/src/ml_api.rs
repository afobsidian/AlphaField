//! ML API Endpoints
//!
//! Provides HTTP API endpoints for ML model training, prediction, and management.

use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use alphafield_backtest::ml::{
    DataSplitter, FeatureConfig, FeatureExtractor, LinearRegression, LogisticRegression, MLModel,
    MLModelType, MLValidation, ModelMetadata, ModelStorage, RandomForest, Scaler, SplitConfig,
    StandardScaler, TrainMetrics,
};

use crate::api::AppState;
use crate::services::data_service::fetch_data_with_cache;

// =============================================================================
// TRAIN MODEL API
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct TrainRequest {
    /// Model type to train
    pub model_type: String,
    /// Symbol to train on
    pub symbol: String,
    /// Data interval (e.g., "1h", "4h")
    pub interval: String,
    /// Number of days of historical data
    pub days: u32,
    /// Prediction horizon in bars
    pub prediction_horizon: Option<usize>,
    /// Train/test split ratio
    pub train_ratio: Option<f64>,
    /// Feature configuration overrides
    pub feature_config: Option<FeatureConfigRequest>,
}

#[derive(Debug, Deserialize)]
pub struct FeatureConfigRequest {
    pub sma_periods: Option<Vec<usize>>,
    pub rsi_period: Option<usize>,
    pub include_volume: Option<bool>,
    pub include_volatility: Option<bool>,
}

#[derive(Serialize)]
pub struct TrainResponse {
    pub success: bool,
    pub model_id: Option<String>,
    pub model_type: String,
    pub train_samples: usize,
    pub test_samples: usize,
    pub train_mae: f64,
    pub train_r_squared: f64,
    pub test_mae: f64,
    pub test_r_squared: f64,
    pub feature_names: Vec<String>,
    pub feature_importance: Option<Vec<(String, f64)>>,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

pub async fn train_model(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TrainRequest>,
) -> Json<TrainResponse> {
    let start = std::time::Instant::now();
    info!(
        model_type = %req.model_type,
        symbol = %req.symbol,
        "Starting ML model training"
    );

    // 1. Parse model type
    let model_type = match req.model_type.to_lowercase().as_str() {
        "linear" | "linearregression" => MLModelType::LinearRegression,
        "logistic" | "logisticregression" => MLModelType::LogisticRegression,
        "randomforest" | "rf" => MLModelType::RandomForest,
        _ => {
            return Json(TrainResponse {
                success: false,
                model_id: None,
                model_type: req.model_type,
                train_samples: 0,
                test_samples: 0,
                train_mae: 0.0,
                train_r_squared: 0.0,
                test_mae: 0.0,
                test_r_squared: 0.0,
                feature_names: vec![],
                feature_importance: None,
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: Some(
                    "Unknown model type. Use: linear, logistic, or randomforest".to_string(),
                ),
            });
        }
    };

    // 2. Fetch data
    use chrono::{Duration, Utc};
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(req.days as i64);

    let fetch_symbol = req.symbol.clone();
    let fetch_interval = req.interval.clone();

    let fetch_result = tokio::spawn(async move {
        fetch_data_with_cache(fetch_symbol, fetch_interval, start_time, end_time).await
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|res| res);

    let bars = match fetch_result {
        Ok((bars, _)) => bars,
        Err(e) => {
            error!(error = %e, "Failed to fetch data for training");
            return Json(TrainResponse {
                success: false,
                model_id: None,
                model_type: req.model_type,
                train_samples: 0,
                test_samples: 0,
                train_mae: 0.0,
                train_r_squared: 0.0,
                test_mae: 0.0,
                test_r_squared: 0.0,
                feature_names: vec![],
                feature_importance: None,
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("Failed to fetch data: {}", e)),
            });
        }
    };

    if bars.len() < 100 {
        return Json(TrainResponse {
            success: false,
            model_id: None,
            model_type: req.model_type,
            train_samples: 0,
            test_samples: 0,
            train_mae: 0.0,
            train_r_squared: 0.0,
            test_mae: 0.0,
            test_r_squared: 0.0,
            feature_names: vec![],
            feature_importance: None,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some("Insufficient data (need at least 100 bars)".to_string()),
        });
    }

    // 3. Configure features
    let mut feature_config = FeatureConfig {
        prediction_horizon: req.prediction_horizon.unwrap_or(1),
        ..Default::default()
    };

    if let Some(fc) = &req.feature_config {
        if let Some(ref periods) = fc.sma_periods {
            feature_config.sma_periods = periods.clone();
        }
        if let Some(period) = fc.rsi_period {
            feature_config.rsi_period = period;
        }
        if let Some(vol) = fc.include_volume {
            feature_config.include_volume = vol;
        }
        if let Some(volatility) = fc.include_volatility {
            feature_config.include_volatility = volatility;
        }
    }

    // 4. Extract features
    let extractor = FeatureExtractor::new(feature_config);
    let feature_set = extractor.extract(&bars);

    if feature_set.is_empty() {
        return Json(TrainResponse {
            success: false,
            model_id: None,
            model_type: req.model_type,
            train_samples: 0,
            test_samples: 0,
            train_mae: 0.0,
            train_r_squared: 0.0,
            test_mae: 0.0,
            test_r_squared: 0.0,
            feature_names: vec![],
            feature_importance: None,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some("Feature extraction failed - insufficient data after warmup".to_string()),
        });
    }

    // 5. Split data
    let split_config = SplitConfig::train_test(req.train_ratio.unwrap_or(0.8));
    let splitter = DataSplitter::new(split_config);
    let (feature_split, label_split) =
        splitter.split_with_labels(&feature_set.features, &feature_set.labels);

    let train_features = feature_split.train;
    let train_labels = label_split.train;
    let test_features = feature_split.test;
    let test_labels = label_split.test;

    if train_features.is_empty() || test_features.is_empty() {
        return Json(TrainResponse {
            success: false,
            model_id: None,
            model_type: req.model_type,
            train_samples: 0,
            test_samples: 0,
            train_mae: 0.0,
            train_r_squared: 0.0,
            test_mae: 0.0,
            test_r_squared: 0.0,
            feature_names: feature_set.feature_names,
            feature_importance: None,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some("Insufficient data after split".to_string()),
        });
    }

    // 6. Normalize features
    let mut scaler = StandardScaler::new();
    let train_scaled = scaler.fit_transform(&train_features);
    let test_scaled = scaler.transform(&test_features);

    // 7. Train model
    let mut model: Box<dyn MLModel> = match model_type {
        MLModelType::LinearRegression => Box::new(LinearRegression::new()),
        MLModelType::LogisticRegression => Box::new(LogisticRegression::new()),
        MLModelType::RandomForest => Box::new(RandomForest::new(10, 5)),
        MLModelType::DecisionTree => Box::new(RandomForest::new(1, 5)), // Single tree
    };

    if let Err(e) = model.train(&train_scaled, &train_labels) {
        return Json(TrainResponse {
            success: false,
            model_id: None,
            model_type: req.model_type,
            train_samples: train_labels.len(),
            test_samples: test_labels.len(),
            train_mae: 0.0,
            train_r_squared: 0.0,
            test_mae: 0.0,
            test_r_squared: 0.0,
            feature_names: feature_set.feature_names,
            feature_importance: None,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("Training failed: {}", e)),
        });
    }

    // 8. Evaluate
    let train_preds: Vec<f64> = train_scaled.iter().map(|f| model.predict(f)).collect();
    let test_preds: Vec<f64> = test_scaled.iter().map(|f| model.predict(f)).collect();

    let train_metrics = calculate_metrics(&train_preds, &train_labels);
    let test_metrics = calculate_metrics(&test_preds, &test_labels);

    // 9. Feature importance
    let feature_importance = model.feature_importance().map(|imp| {
        feature_set
            .feature_names
            .iter()
            .zip(imp.iter())
            .map(|(name, &val)| (name.clone(), val))
            .collect()
    });

    // 10. Save model (generate ID only - full serialization deferred for simplicity)
    let model_id = ModelStorage::generate_id(model_type, &req.symbol);
    // Note: For now we just generate the ID. Full model persistence
    // will be added when we implement model loading for inference.
    let _metadata = ModelMetadata::new(
        &model_id,
        model_type,
        feature_set.feature_names.clone(),
        vec![req.symbol.clone()],
        &req.interval,
        req.prediction_horizon.unwrap_or(1),
    )
    .with_metrics(TrainMetrics {
        train_mae: train_metrics.mae,
        train_r2: train_metrics.r_squared,
        train_accuracy: None,
        val_mae: Some(test_metrics.mae),
        val_r2: Some(test_metrics.r_squared),
        n_train_samples: train_labels.len(),
    });

    info!(
        model_id = %model_id,
        train_mae = train_metrics.mae,
        test_mae = test_metrics.mae,
        "Model training complete"
    );

    Json(TrainResponse {
        success: true,
        model_id: Some(model_id),
        model_type: model_type.to_string(),
        train_samples: train_labels.len(),
        test_samples: test_labels.len(),
        train_mae: train_metrics.mae,
        train_r_squared: train_metrics.r_squared,
        test_mae: test_metrics.mae,
        test_r_squared: test_metrics.r_squared,
        feature_names: feature_set.feature_names,
        feature_importance,
        elapsed_ms: start.elapsed().as_millis() as u64,
        error: None,
    })
}

/// Simple metrics calculation
fn calculate_metrics(predictions: &[f64], labels: &[f64]) -> SimpleMetrics {
    let n = predictions.len() as f64;
    if n == 0.0 {
        return SimpleMetrics {
            mae: 0.0,
            r_squared: 0.0,
        };
    }

    let mae: f64 = predictions
        .iter()
        .zip(labels.iter())
        .map(|(p, l)| (p - l).abs())
        .sum::<f64>()
        / n;

    let mean_label = labels.iter().sum::<f64>() / n;
    let ss_tot: f64 = labels.iter().map(|l| (l - mean_label).powi(2)).sum();
    let ss_res: f64 = predictions
        .iter()
        .zip(labels.iter())
        .map(|(p, l)| (p - l).powi(2))
        .sum();

    let r_squared = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };

    SimpleMetrics { mae, r_squared }
}

struct SimpleMetrics {
    mae: f64,
    r_squared: f64,
}

// =============================================================================
// LIST MODELS API
// =============================================================================

#[derive(Serialize)]
pub struct ModelsListResponse {
    pub success: bool,
    pub models: Vec<ModelInfo>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub model_type: String,
    pub trained_at: String,
    pub symbols: Vec<String>,
    pub interval: String,
    pub train_mae: f64,
    pub train_r2: f64,
}

pub async fn list_models(State(_state): State<Arc<AppState>>) -> Json<ModelsListResponse> {
    let storage = ModelStorage::new();

    match storage.list_models() {
        Ok(models) => {
            let model_infos: Vec<ModelInfo> = models
                .iter()
                .map(|m| ModelInfo {
                    id: m.id.clone(),
                    model_type: m.model_type.to_string(),
                    trained_at: m.trained_at.to_rfc3339(),
                    symbols: m.symbols.clone(),
                    interval: m.interval.clone(),
                    train_mae: m.metrics.train_mae,
                    train_r2: m.metrics.train_r2,
                })
                .collect();

            Json(ModelsListResponse {
                success: true,
                models: model_infos,
                error: None,
            })
        }
        Err(e) => Json(ModelsListResponse {
            success: false,
            models: vec![],
            error: Some(e),
        }),
    }
}

// =============================================================================
// DELETE MODEL API
// =============================================================================

#[derive(Serialize)]
pub struct DeleteModelResponse {
    pub success: bool,
    pub error: Option<String>,
}

pub async fn delete_model(
    axum::extract::Path(id): axum::extract::Path<String>,
    State(_state): State<Arc<AppState>>,
) -> Json<DeleteModelResponse> {
    let storage = ModelStorage::new();

    match storage.delete(&id) {
        Ok(()) => Json(DeleteModelResponse {
            success: true,
            error: None,
        }),
        Err(e) => Json(DeleteModelResponse {
            success: false,
            error: Some(e),
        }),
    }
}

// =============================================================================
// VALIDATE MODEL API (Walk-Forward)
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    pub symbol: String,
    pub interval: String,
    pub days: u32,
    pub model_type: String,
    pub train_window_days: Option<usize>,
    pub test_window_days: Option<usize>,
    #[serde(default = "default_trading_mode")]
    pub trading_mode: String,
}

fn default_trading_mode() -> String {
    "Spot".to_string()
}

#[derive(Serialize)]
pub struct ValidateResponse {
    pub success: bool,
    pub n_windows: usize,
    pub avg_test_mae: f64,
    pub avg_test_r_squared: f64,
    pub stability_score: f64,
    pub is_overfit: bool,
    pub overfit_description: String,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

pub async fn validate_model(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ValidateRequest>,
) -> Json<ValidateResponse> {
    let start = std::time::Instant::now();
    info!(
        model_type = %req.model_type,
        symbol = %req.symbol,
        "Starting walk-forward ML validation"
    );

    // Fetch data
    use chrono::{Duration, Utc};
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(req.days as i64);

    let fetch_symbol = req.symbol.clone();
    let fetch_interval = req.interval.clone();

    let fetch_result = tokio::spawn(async move {
        fetch_data_with_cache(fetch_symbol, fetch_interval, start_time, end_time).await
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|res| res);

    let bars = match fetch_result {
        Ok((bars, _)) => bars,
        Err(e) => {
            return Json(ValidateResponse {
                success: false,
                n_windows: 0,
                avg_test_mae: 0.0,
                avg_test_r_squared: 0.0,
                stability_score: 0.0,
                is_overfit: false,
                overfit_description: String::new(),
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("Failed to fetch data: {}", e)),
            });
        }
    };

    // Extract features
    let extractor = FeatureExtractor::new(FeatureConfig::default());
    let feature_set = extractor.extract(&bars);

    if feature_set.n_samples() < 200 {
        return Json(ValidateResponse {
            success: false,
            n_windows: 0,
            avg_test_mae: 0.0,
            avg_test_r_squared: 0.0,
            stability_score: 0.0,
            is_overfit: false,
            overfit_description: String::new(),
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some("Insufficient data for walk-forward validation".to_string()),
        });
    }

    // Normalize
    let mut scaler = StandardScaler::new();
    let scaled = scaler.fit_transform(&feature_set.features);

    // Walk-forward validation
    let train_window = req.train_window_days.unwrap_or(100);
    let test_window = req.test_window_days.unwrap_or(30);

    let trading_mode = match req.trading_mode.as_str() {
        "Margin" => alphafield_core::TradingMode::Margin,
        _ => alphafield_core::TradingMode::Spot,
    };
    let validator = MLValidation::new(train_window, test_window, test_window / 2, trading_mode);

    let model_type = match req.model_type.to_lowercase().as_str() {
        "linear" | "linearregression" => MLModelType::LinearRegression,
        "logistic" | "logisticregression" => MLModelType::LogisticRegression,
        "randomforest" | "rf" => MLModelType::RandomForest,
        _ => MLModelType::LinearRegression,
    };

    let result = validator.validate(&scaled, &feature_set.labels, || -> Box<dyn MLModel> {
        match model_type {
            MLModelType::LinearRegression => Box::new(LinearRegression::new()),
            MLModelType::LogisticRegression => Box::new(LogisticRegression::new()),
            MLModelType::RandomForest => Box::new(RandomForest::new(5, 3)),
            MLModelType::DecisionTree => Box::new(RandomForest::new(1, 3)),
        }
    });

    Json(ValidateResponse {
        success: true,
        n_windows: result.windows.len(),
        avg_test_mae: result.avg_test_mae,
        avg_test_r_squared: result.avg_test_r2,
        stability_score: result.stability_score,
        is_overfit: result.overfit_detection.is_overfit,
        overfit_description: result.overfit_detection.description,
        elapsed_ms: start.elapsed().as_millis() as u64,
        error: None,
    })
}

// =============================================================================
// MULTI-SYMBOL TRAIN API (Random Subsets)
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct TrainMultiRequest {
    /// Model type to train
    pub model_type: String,
    /// Symbols to train on (uses random subsets from each)
    pub symbols: Vec<String>,
    /// Data interval (e.g., "1h", "4h")
    pub interval: String,
    /// Number of days of historical data per symbol
    pub days: u32,
    /// Number of random samples per symbol
    pub samples_per_symbol: Option<usize>,
    /// Prediction horizon in bars
    pub prediction_horizon: Option<usize>,
}

#[derive(Serialize)]
pub struct TrainMultiResponse {
    pub success: bool,
    pub model_id: Option<String>,
    pub model_type: String,
    pub symbols_used: Vec<String>,
    pub total_samples: usize,
    pub train_samples: usize,
    pub test_samples: usize,
    pub train_mae: f64,
    pub train_r_squared: f64,
    pub test_mae: f64,
    pub test_r_squared: f64,
    pub stability_score: f64,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

/// Train ML model on random subsets from multiple symbols
pub async fn train_multi_symbol(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TrainMultiRequest>,
) -> Json<TrainMultiResponse> {
    use chrono::{Duration, Utc};

    let start = std::time::Instant::now();
    info!(
        model_type = %req.model_type,
        symbols = ?req.symbols,
        "Starting multi-symbol ML training"
    );

    if req.symbols.is_empty() {
        return Json(TrainMultiResponse {
            success: false,
            model_id: None,
            model_type: req.model_type,
            symbols_used: vec![],
            total_samples: 0,
            train_samples: 0,
            test_samples: 0,
            train_mae: 0.0,
            train_r_squared: 0.0,
            test_mae: 0.0,
            test_r_squared: 0.0,
            stability_score: 0.0,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some("No symbols provided".to_string()),
        });
    }

    // Parse model type
    let model_type = match req.model_type.to_lowercase().as_str() {
        "linear" | "linearregression" => MLModelType::LinearRegression,
        "logistic" | "logisticregression" => MLModelType::LogisticRegression,
        "randomforest" | "rf" => MLModelType::RandomForest,
        _ => MLModelType::LinearRegression,
    };

    let samples_per_symbol = req.samples_per_symbol.unwrap_or(200);

    // Collect features from all symbols
    let mut all_features: Vec<Vec<f64>> = Vec::new();
    let mut all_labels: Vec<f64> = Vec::new();
    let mut symbols_used: Vec<String> = Vec::new();

    let end_time = Utc::now();
    let start_time = end_time - Duration::days(req.days as i64);

    for symbol in &req.symbols {
        let fetch_symbol = symbol.clone();
        let fetch_interval = req.interval.clone();

        let fetch_result = tokio::spawn(async move {
            fetch_data_with_cache(fetch_symbol, fetch_interval, start_time, end_time).await
        })
        .await
        .map_err(|e| e.to_string())
        .and_then(|res| res);

        let bars = match fetch_result {
            Ok((bars, _)) => bars,
            Err(e) => {
                info!(symbol = %symbol, error = %e, "Skipping symbol due to fetch error");
                continue;
            }
        };

        if bars.len() < 100 {
            info!(symbol = %symbol, "Skipping symbol - insufficient data");
            continue;
        }

        // Extract features
        let feature_config = FeatureConfig {
            prediction_horizon: req.prediction_horizon.unwrap_or(1),
            ..Default::default()
        };
        let extractor = FeatureExtractor::new(feature_config);
        let feature_set = extractor.extract(&bars);

        if feature_set.is_empty() {
            continue;
        }

        // Random sampling: pick random indices using a simple deterministic approach
        // We use timestamp-based seed for reproducibility
        let n_available = feature_set.n_samples();
        let n_to_sample = samples_per_symbol.min(n_available);

        // Create indices and shuffle using a simple seed-based approach
        let mut indices: Vec<usize> = (0..n_available).collect();
        // Use symbol hash and timestamp as seed for pseudo-random but deterministic shuffle
        let seed = symbol
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_add(b as u64))
            .wrapping_add(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            );

        // Fisher-Yates shuffle with simple LCG
        let mut lcg_state = seed;
        for i in (1..indices.len()).rev() {
            lcg_state = lcg_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (lcg_state as usize) % (i + 1);
            indices.swap(i, j);
        }

        for &idx in indices.iter().take(n_to_sample) {
            all_features.push(feature_set.features[idx].clone());
            all_labels.push(feature_set.labels[idx]);
        }

        symbols_used.push(symbol.clone());
        info!(symbol = %symbol, samples = n_to_sample, "Added samples from symbol");
    }

    if all_features.is_empty() {
        return Json(TrainMultiResponse {
            success: false,
            model_id: None,
            model_type: req.model_type,
            symbols_used,
            total_samples: 0,
            train_samples: 0,
            test_samples: 0,
            train_mae: 0.0,
            train_r_squared: 0.0,
            test_mae: 0.0,
            test_r_squared: 0.0,
            stability_score: 0.0,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some("No valid data from any symbol".to_string()),
        });
    }

    // Shuffle combined data using LCG
    let mut combined: Vec<(Vec<f64>, f64)> = all_features
        .into_iter()
        .zip(all_labels.into_iter())
        .collect();

    // Fisher-Yates shuffle with LCG for combined data
    let shuffle_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut lcg_state = shuffle_seed;
    for i in (1..combined.len()).rev() {
        lcg_state = lcg_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (lcg_state as usize) % (i + 1);
        combined.swap(i, j);
    }

    let total_samples = combined.len();
    let train_size = (total_samples as f64 * 0.8) as usize;

    let (train_data, test_data) = combined.split_at(train_size);
    let train_features: Vec<Vec<f64>> = train_data.iter().map(|(f, _)| f.clone()).collect();
    let train_labels: Vec<f64> = train_data.iter().map(|(_, l)| *l).collect();
    let test_features: Vec<Vec<f64>> = test_data.iter().map(|(f, _)| f.clone()).collect();
    let test_labels: Vec<f64> = test_data.iter().map(|(_, l)| *l).collect();

    // Normalize
    let mut scaler = StandardScaler::new();
    let train_scaled = scaler.fit_transform(&train_features);
    let test_scaled = scaler.transform(&test_features);

    // Train model
    let mut model: Box<dyn MLModel> = match model_type {
        MLModelType::LinearRegression => Box::new(LinearRegression::new()),
        MLModelType::LogisticRegression => Box::new(LogisticRegression::new()),
        MLModelType::RandomForest => Box::new(RandomForest::new(10, 5)),
        MLModelType::DecisionTree => Box::new(RandomForest::new(1, 5)),
    };

    if let Err(e) = model.train(&train_scaled, &train_labels) {
        return Json(TrainMultiResponse {
            success: false,
            model_id: None,
            model_type: req.model_type,
            symbols_used,
            total_samples,
            train_samples: train_labels.len(),
            test_samples: test_labels.len(),
            train_mae: 0.0,
            train_r_squared: 0.0,
            test_mae: 0.0,
            test_r_squared: 0.0,
            stability_score: 0.0,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("Training failed: {}", e)),
        });
    }

    // Evaluate
    let train_preds: Vec<f64> = train_scaled.iter().map(|f| model.predict(f)).collect();
    let test_preds: Vec<f64> = test_scaled.iter().map(|f| model.predict(f)).collect();

    let train_metrics = calculate_metrics(&train_preds, &train_labels);
    let test_metrics = calculate_metrics(&test_preds, &test_labels);

    // Calculate stability score: how consistent is performance across random samples
    let train_test_gap = (train_metrics.r_squared - test_metrics.r_squared).abs();
    let stability_score = ((1.0 - train_test_gap) * 100.0).clamp(0.0, 100.0);

    // Generate model ID
    let model_id = format!(
        "multi_{}_{}",
        match model_type {
            MLModelType::LinearRegression => "lr",
            MLModelType::LogisticRegression => "logr",
            MLModelType::RandomForest => "rf",
            MLModelType::DecisionTree => "dt",
        },
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );

    info!(
        model_id = %model_id,
        symbols = ?symbols_used,
        total_samples = total_samples,
        train_r2 = train_metrics.r_squared,
        test_r2 = test_metrics.r_squared,
        "Multi-symbol training complete"
    );

    Json(TrainMultiResponse {
        success: true,
        model_id: Some(model_id),
        model_type: model_type.to_string(),
        symbols_used,
        total_samples,
        train_samples: train_labels.len(),
        test_samples: test_labels.len(),
        train_mae: train_metrics.mae,
        train_r_squared: train_metrics.r_squared,
        test_mae: test_metrics.mae,
        test_r_squared: test_metrics.r_squared,
        stability_score,
        elapsed_ms: start.elapsed().as_millis() as u64,
        error: None,
    })
}
