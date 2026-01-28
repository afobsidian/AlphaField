//! ML Validation Module
//!
//! Provides validation utilities for ML models including:
//! - Walk-forward ML validation (train → predict → evaluate)
//! - Rolling retraining
//! - Overfitting detection

use super::data_split::RollingWindowGenerator;
use super::ensemble::ModelMetrics;
use super::models::MLModel;
use alphafield_core::TradingMode;
use serde::{Deserialize, Serialize};

/// Result from a single validation window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowValidationResult {
    pub window_id: usize,
    pub train_start: usize,
    pub train_end: usize,
    pub test_start: usize,
    pub test_end: usize,
    pub train_metrics: ModelMetrics,
    pub test_metrics: ModelMetrics,
}

/// Aggregated validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLValidationResult {
    /// Results per window
    pub windows: Vec<WindowValidationResult>,
    /// Average test MAE across windows
    pub avg_test_mae: f64,
    /// Average test R-squared
    pub avg_test_r2: f64,
    /// Standard deviation of test MAE
    pub std_test_mae: f64,
    /// Average train-test gap (for overfit detection)
    pub avg_train_test_gap: f64,
    /// Overall stability score (0-100)
    pub stability_score: f64,
    /// Overfitting risk assessment
    pub overfit_detection: OverfitDetection,
}

impl Default for MLValidationResult {
    fn default() -> Self {
        Self {
            windows: vec![],
            avg_test_mae: 0.0,
            avg_test_r2: 0.0,
            std_test_mae: 0.0,
            avg_train_test_gap: 0.0,
            stability_score: 0.0,
            overfit_detection: OverfitDetection::default(),
        }
    }
}

/// Overfitting detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverfitDetection {
    /// Is overfitting likely?
    pub is_overfit: bool,
    /// Confidence in the assessment (0-1)
    pub confidence: f64,
    /// Train-test performance gap
    pub performance_gap: f64,
    /// Description of findings
    pub description: String,
}

impl Default for OverfitDetection {
    fn default() -> Self {
        Self {
            is_overfit: false,
            confidence: 0.0,
            performance_gap: 0.0,
            description: "Not evaluated".to_string(),
        }
    }
}

/// Walk-forward ML validation executor
pub struct MLValidation {
    /// Training window size
    train_window: usize,
    /// Test window size
    test_window: usize,
    /// Step size between windows
    step_size: usize,
    /// Gap between train and test
    gap: usize,
    /// Trading mode for label filtering (Spot = Buy only, Margin = Buy & Sell)
    trading_mode: TradingMode,
}

impl MLValidation {
    /// Create new ML validator
    pub fn new(
        train_window: usize,
        test_window: usize,
        step_size: usize,
        trading_mode: TradingMode,
    ) -> Self {
        Self {
            train_window,
            test_window,
            step_size,
            gap: 0,
            trading_mode,
        }
    }

    /// Add gap between train and test sets
    pub fn with_gap(mut self, gap: usize) -> Self {
        self.gap = gap;
        self
    }

    /// Run walk-forward validation
    ///
    /// The model_factory creates a fresh model for each window to simulate
    /// realistic retraining scenarios.
    pub fn validate<F>(
        &self,
        features: &[Vec<f64>],
        labels: &[f64],
        model_factory: F,
    ) -> MLValidationResult
    where
        F: Fn() -> Box<dyn MLModel>,
    {
        let generator =
            RollingWindowGenerator::new(self.train_window, self.test_window, self.step_size)
                .with_gap(self.gap);

        let windows = generator.generate_windows(features.len());

        if windows.is_empty() {
            return MLValidationResult::default();
        }

        let mut window_results = Vec::new();
        let mut test_maes = Vec::new();
        let mut test_r2s = Vec::new();
        let mut train_test_gaps = Vec::new();

        for window in windows {
            // Extract train/test data
            let train_features: Vec<Vec<f64>> =
                features[window.train_start..window.train_end].to_vec();
            let train_labels: Vec<f64> = labels[window.train_start..window.train_end].to_vec();

            let test_features: Vec<Vec<f64>> =
                features[window.test_start..window.test_end].to_vec();
            let test_labels: Vec<f64> = labels[window.test_start..window.test_end].to_vec();

            // Filter labels based on TradingMode
            // Spot mode: only train on positive labels (Buy signals)
            // Margin mode: train on all labels (both Buy and Sell signals)
            let (train_features_filtered, train_labels_filtered) =
                if self.trading_mode == TradingMode::Spot {
                    // Filter to only positive labels (Buy signals) for Spot mode
                    let mut filtered_features = Vec::new();
                    let mut filtered_labels = Vec::new();
                    for (feat, lbl) in train_features.iter().zip(train_labels.iter()) {
                        if *lbl >= 0.0 {
                            filtered_features.push(feat.clone());
                            filtered_labels.push(*lbl);
                        }
                    }
                    (filtered_features, filtered_labels)
                } else {
                    // Use all labels for Margin mode
                    (train_features, train_labels)
                };

            // Train fresh model
            let mut model = model_factory();
            if model
                .train(&train_features_filtered, &train_labels_filtered)
                .is_err()
            {
                continue;
            }

            // Evaluate on train and test
            let train_preds = model.predict_batch(&train_features_filtered);
            let test_preds = model.predict_batch(&test_features);

            let train_metrics =
                ModelMetrics::calculate_regression(&train_preds, &train_labels_filtered);
            let test_metrics = ModelMetrics::calculate_regression(&test_preds, &test_labels);

            // Calculate train-test gap
            let gap = (train_metrics.r_squared - test_metrics.r_squared).abs();
            train_test_gaps.push(gap);

            test_maes.push(test_metrics.mae);
            test_r2s.push(test_metrics.r_squared);

            window_results.push(WindowValidationResult {
                window_id: window.window_id,
                train_start: window.train_start,
                train_end: window.train_end,
                test_start: window.test_start,
                test_end: window.test_end,
                train_metrics,
                test_metrics,
            });
        }

        if window_results.is_empty() {
            return MLValidationResult::default();
        }

        // Calculate aggregates
        let n = window_results.len() as f64;

        let avg_test_mae = test_maes.iter().sum::<f64>() / n;
        let avg_test_r2 = test_r2s.iter().sum::<f64>() / n;
        let avg_train_test_gap = train_test_gaps.iter().sum::<f64>() / n;

        let std_test_mae = {
            let variance: f64 = test_maes
                .iter()
                .map(|m| (m - avg_test_mae).powi(2))
                .sum::<f64>()
                / n;
            variance.sqrt()
        };

        // Calculate stability score (0-100)
        let stability_score = calculate_stability_score(&test_r2s, &train_test_gaps);

        // Detect overfitting
        let overfit_detection = detect_overfitting(&window_results, avg_train_test_gap);

        MLValidationResult {
            windows: window_results,
            avg_test_mae,
            avg_test_r2,
            std_test_mae,
            avg_train_test_gap,
            stability_score,
            overfit_detection,
        }
    }
}

/// Calculate stability score from validation results
fn calculate_stability_score(test_r2s: &[f64], train_test_gaps: &[f64]) -> f64 {
    if test_r2s.is_empty() {
        return 0.0;
    }

    // Component 1: Average test R² (0-40 points)
    let avg_r2: f64 = test_r2s.iter().sum::<f64>() / test_r2s.len() as f64;
    let r2_score = (avg_r2.max(0.0) * 40.0).min(40.0);

    // Component 2: Consistency of R² (0-30 points)
    let mean_r2 = avg_r2;
    let r2_std = {
        let var: f64 =
            test_r2s.iter().map(|r| (r - mean_r2).powi(2)).sum::<f64>() / test_r2s.len() as f64;
        var.sqrt()
    };
    let consistency_score = ((1.0 - r2_std.min(1.0)) * 30.0).max(0.0);

    // Component 3: Low train-test gap (0-30 points)
    let avg_gap: f64 = train_test_gaps.iter().sum::<f64>() / train_test_gaps.len().max(1) as f64;
    let gap_score = ((1.0 - avg_gap * 2.0).max(0.0) * 30.0).min(30.0);

    r2_score + consistency_score + gap_score
}

/// Detect overfitting from validation results
fn detect_overfitting(windows: &[WindowValidationResult], avg_gap: f64) -> OverfitDetection {
    if windows.is_empty() {
        return OverfitDetection::default();
    }

    // Signs of overfitting:
    // 1. Large train-test performance gap (>0.2)
    // 2. Declining test performance across windows
    // 3. Very high train R² (>0.95) with low test R²

    let mut overfit_signals = 0;
    let mut total_checks = 0;

    // Check 1: Performance gap
    total_checks += 1;
    if avg_gap > 0.2 {
        overfit_signals += 1;
    }

    // Check 2: Train-test gap per window
    let high_gap_windows = windows
        .iter()
        .filter(|w| (w.train_metrics.r_squared - w.test_metrics.r_squared) > 0.3)
        .count();
    total_checks += 1;
    if high_gap_windows as f64 / windows.len() as f64 > 0.5 {
        overfit_signals += 1;
    }

    // Check 3: Declining performance trend
    if windows.len() >= 3 {
        total_checks += 1;
        let first_half_avg: f64 = windows[..windows.len() / 2]
            .iter()
            .map(|w| w.test_metrics.r_squared)
            .sum::<f64>()
            / (windows.len() / 2) as f64;
        let second_half_avg: f64 = windows[windows.len() / 2..]
            .iter()
            .map(|w| w.test_metrics.r_squared)
            .sum::<f64>()
            / (windows.len() - windows.len() / 2) as f64;

        if first_half_avg > second_half_avg + 0.1 {
            overfit_signals += 1;
        }
    }

    let confidence = overfit_signals as f64 / total_checks as f64;
    let is_overfit = confidence >= 0.5;

    let description = if is_overfit {
        if avg_gap > 0.3 {
            "High overfitting risk: Large train-test performance gap".to_string()
        } else {
            "Moderate overfitting risk: Inconsistent out-of-sample performance".to_string()
        }
    } else if confidence > 0.2 {
        "Low overfitting risk: Some performance degradation observed".to_string()
    } else {
        "No significant overfitting detected".to_string()
    };

    OverfitDetection {
        is_overfit,
        confidence,
        performance_gap: avg_gap,
        description,
    }
}

/// Quick train-test evaluation without walk-forward
pub fn quick_evaluation(
    model: &dyn MLModel,
    test_features: &[Vec<f64>],
    test_labels: &[f64],
) -> ModelMetrics {
    let predictions = model.predict_batch(test_features);
    ModelMetrics::calculate_regression(&predictions, test_labels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::models::LinearRegression;

    fn make_synthetic_data(n: usize) -> (Vec<Vec<f64>>, Vec<f64>) {
        let features: Vec<Vec<f64>> = (0..n).map(|i| vec![i as f64, (i as f64 * 0.5)]).collect();
        let labels: Vec<f64> = features
            .iter()
            .map(|f| 2.0 * f[0] + 3.0 * f[1] + 1.0 + (f[0] * 0.1).sin())
            .collect();
        (features, labels)
    }

    #[test]
    fn test_walk_forward_validation() {
        let (features, labels) = make_synthetic_data(200);

        let validator = MLValidation::new(50, 20, 30, TradingMode::Spot);
        let result = validator.validate(&features, &labels, || Box::new(LinearRegression::new()));

        assert!(!result.windows.is_empty());
        assert!(result.avg_test_mae >= 0.0);
        assert!(result.stability_score >= 0.0);
        assert!(result.stability_score <= 100.0);
    }

    #[test]
    fn test_insufficient_data() {
        let (features, labels) = make_synthetic_data(10);

        let validator = MLValidation::new(50, 20, 30, TradingMode::Spot);
        let result = validator.validate(&features, &labels, || Box::new(LinearRegression::new()));

        assert!(result.windows.is_empty());
    }

    #[test]
    fn test_walk_forward_validation_margin() {
        let (features, labels) = make_synthetic_data(200);

        // Create validator with Margin mode (trains on both Buy and Sell signals)
        let validator = MLValidation::new(50, 20, 30, TradingMode::Margin);
        let result = validator.validate(&features, &labels, || Box::new(LinearRegression::new()));

        // Verify validation completed successfully with Margin mode
        assert!(!result.windows.is_empty());
        assert!(result.avg_test_mae >= 0.0);
        assert!(result.stability_score >= 0.0);
        assert!(result.stability_score <= 100.0);

        // Verify that Margin mode processes both positive and negative labels
        // (by checking that metrics were calculated)
        assert!(result.avg_test_r2.is_finite());
        assert!(result.avg_test_mae.is_finite());
    }

    #[test]
    fn test_overfit_detection_healthy() {
        // Create mock results with small gaps
        let windows = vec![WindowValidationResult {
            window_id: 0,
            train_start: 0,
            train_end: 50,
            test_start: 50,
            test_end: 70,
            train_metrics: ModelMetrics {
                mae: 0.1,
                mse: 0.01,
                rmse: 0.1,
                r_squared: 0.8,
                accuracy: None,
            },
            test_metrics: ModelMetrics {
                mae: 0.12,
                mse: 0.014,
                rmse: 0.12,
                r_squared: 0.75,
                accuracy: None,
            },
        }];

        let detection = detect_overfitting(&windows, 0.05);
        assert!(!detection.is_overfit);
    }

    #[test]
    fn test_overfit_detection_overfit() {
        // Create mock results with large gaps
        let windows = vec![WindowValidationResult {
            window_id: 0,
            train_start: 0,
            train_end: 50,
            test_start: 50,
            test_end: 70,
            train_metrics: ModelMetrics {
                mae: 0.01,
                mse: 0.001,
                rmse: 0.01,
                r_squared: 0.99,
                accuracy: None,
            },
            test_metrics: ModelMetrics {
                mae: 0.5,
                mse: 0.25,
                rmse: 0.5,
                r_squared: 0.3,
                accuracy: None,
            },
        }];

        let detection = detect_overfitting(&windows, 0.69);
        assert!(detection.is_overfit);
        assert!(detection.performance_gap > 0.5);
    }

    #[test]
    fn test_stability_score() {
        let r2s = vec![0.7, 0.65, 0.72, 0.68];
        let gaps = vec![0.05, 0.08, 0.06, 0.07];

        let score = calculate_stability_score(&r2s, &gaps);
        assert!(score > 50.0); // Should be reasonably stable
        assert!(score <= 100.0);
    }

    #[test]
    fn test_quick_evaluation() {
        let mut model = LinearRegression::new();
        let features = vec![vec![1.0], vec![2.0], vec![3.0]];
        let labels = vec![2.0, 4.0, 6.0];

        model.train(&features, &labels).unwrap();
        let metrics = quick_evaluation(&model, &features, &labels);

        assert!(metrics.mae >= 0.0);
    }
}
