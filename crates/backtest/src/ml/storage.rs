//! Model Storage Module
//!
//! Provides serialization and persistence for trained ML models.

use super::models::{
    DecisionTree, LinearRegression, LogisticRegression, MLModelType, RandomForest,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Training metrics stored with model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainMetrics {
    /// Mean Absolute Error on training data
    pub train_mae: f64,
    /// R-squared on training data
    pub train_r2: f64,
    /// Accuracy (for classifiers)
    pub train_accuracy: Option<f64>,
    /// Validation MAE (if validation set used)
    pub val_mae: Option<f64>,
    /// Validation R-squared
    pub val_r2: Option<f64>,
    /// Number of training samples
    pub n_train_samples: usize,
}

impl Default for TrainMetrics {
    fn default() -> Self {
        Self {
            train_mae: 0.0,
            train_r2: 0.0,
            train_accuracy: None,
            val_mae: None,
            val_r2: None,
            n_train_samples: 0,
        }
    }
}

/// Metadata stored with each model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Unique model identifier
    pub id: String,
    /// Model type
    pub model_type: MLModelType,
    /// When the model was trained
    pub trained_at: DateTime<Utc>,
    /// Feature names used for training
    pub feature_names: Vec<String>,
    /// Symbol(s) the model was trained on
    pub symbols: Vec<String>,
    /// Training interval (e.g., "1h", "4h")
    pub interval: String,
    /// Prediction horizon in bars
    pub prediction_horizon: usize,
    /// Training metrics
    pub metrics: TrainMetrics,
    /// Additional configuration used
    pub config: HashMap<String, String>,
}

impl ModelMetadata {
    /// Create new metadata
    pub fn new(
        id: &str,
        model_type: MLModelType,
        feature_names: Vec<String>,
        symbols: Vec<String>,
        interval: &str,
        prediction_horizon: usize,
    ) -> Self {
        Self {
            id: id.to_string(),
            model_type,
            trained_at: Utc::now(),
            feature_names,
            symbols,
            interval: interval.to_string(),
            prediction_horizon,
            metrics: TrainMetrics::default(),
            config: HashMap::new(),
        }
    }

    /// Update training metrics
    pub fn with_metrics(mut self, metrics: TrainMetrics) -> Self {
        self.metrics = metrics;
        self
    }
}

/// Serialized model container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedModel {
    pub metadata: ModelMetadata,
    pub model_data: ModelData,
}

/// Model-specific data for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelData {
    LinearRegression(LinearRegression),
    LogisticRegression(LogisticRegression),
    DecisionTree(DecisionTree),
    RandomForest(RandomForest),
}

/// Model storage manager
pub struct ModelStorage {
    base_path: PathBuf,
}

impl ModelStorage {
    /// Create storage with default path
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("./models"),
        }
    }

    /// Create storage with custom path
    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            base_path: path.as_ref().to_path_buf(),
        }
    }

    /// Ensure storage directory exists
    fn ensure_dir(&self) -> Result<(), String> {
        fs::create_dir_all(&self.base_path)
            .map_err(|e| format!("Failed to create models directory: {}", e))
    }

    /// Get model file path
    fn model_path(&self, id: &str) -> PathBuf {
        self.base_path.join(format!("{}.json", id))
    }

    /// Save a linear regression model
    pub fn save_linear_regression(
        &self,
        model: &LinearRegression,
        metadata: ModelMetadata,
    ) -> Result<String, String> {
        self.ensure_dir()?;

        let serialized = SerializedModel {
            metadata: metadata.clone(),
            model_data: ModelData::LinearRegression(model.clone()),
        };

        let json = serde_json::to_string_pretty(&serialized)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let path = self.model_path(&metadata.id);
        fs::write(&path, json).map_err(|e| format!("Failed to write model: {}", e))?;

        Ok(metadata.id)
    }

    /// Save a logistic regression model
    pub fn save_logistic_regression(
        &self,
        model: &LogisticRegression,
        metadata: ModelMetadata,
    ) -> Result<String, String> {
        self.ensure_dir()?;

        let serialized = SerializedModel {
            metadata: metadata.clone(),
            model_data: ModelData::LogisticRegression(model.clone()),
        };

        let json = serde_json::to_string_pretty(&serialized)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let path = self.model_path(&metadata.id);
        fs::write(&path, json).map_err(|e| format!("Failed to write model: {}", e))?;

        Ok(metadata.id)
    }

    /// Save a decision tree model
    pub fn save_decision_tree(
        &self,
        model: &DecisionTree,
        metadata: ModelMetadata,
    ) -> Result<String, String> {
        self.ensure_dir()?;

        let serialized = SerializedModel {
            metadata: metadata.clone(),
            model_data: ModelData::DecisionTree(model.clone()),
        };

        let json = serde_json::to_string_pretty(&serialized)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let path = self.model_path(&metadata.id);
        fs::write(&path, json).map_err(|e| format!("Failed to write model: {}", e))?;

        Ok(metadata.id)
    }

    /// Save a random forest model
    pub fn save_random_forest(
        &self,
        model: &RandomForest,
        metadata: ModelMetadata,
    ) -> Result<String, String> {
        self.ensure_dir()?;

        let serialized = SerializedModel {
            metadata: metadata.clone(),
            model_data: ModelData::RandomForest(model.clone()),
        };

        let json = serde_json::to_string_pretty(&serialized)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        let path = self.model_path(&metadata.id);
        fs::write(&path, json).map_err(|e| format!("Failed to write model: {}", e))?;

        Ok(metadata.id)
    }

    /// Load a serialized model
    pub fn load(&self, id: &str) -> Result<SerializedModel, String> {
        let path = self.model_path(id);

        let json =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read model {}: {}", id, e))?;

        serde_json::from_str(&json).map_err(|e| format!("Failed to deserialize model: {}", e))
    }

    /// List all saved models
    pub fn list_models(&self) -> Result<Vec<ModelMetadata>, String> {
        self.ensure_dir()?;

        let mut models = Vec::new();

        let entries = fs::read_dir(&self.base_path)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(model) = serde_json::from_str::<SerializedModel>(&content) {
                        models.push(model.metadata);
                    }
                }
            }
        }

        // Sort by training date (newest first)
        models.sort_by(|a, b| b.trained_at.cmp(&a.trained_at));

        Ok(models)
    }

    /// Delete a model
    pub fn delete(&self, id: &str) -> Result<(), String> {
        let path = self.model_path(id);

        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("Failed to delete model: {}", e))
        } else {
            Err(format!("Model {} not found", id))
        }
    }

    /// Check if model exists
    pub fn exists(&self, id: &str) -> bool {
        self.model_path(id).exists()
    }

    /// Generate unique model ID
    pub fn generate_id(model_type: MLModelType, symbol: &str) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let type_str = match model_type {
            MLModelType::LinearRegression => "lr",
            MLModelType::LogisticRegression => "logr",
            MLModelType::DecisionTree => "dt",
            MLModelType::RandomForest => "rf",
        };
        format!(
            "{}_{}_{}_{}",
            type_str,
            symbol.to_lowercase(),
            timestamp,
            rand_suffix()
        )
    }
}

impl Default for ModelStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate random suffix for IDs
fn rand_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{:04x}", nanos % 0xFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::models::MLModel;
    use std::env;

    fn temp_storage() -> ModelStorage {
        let temp_dir = env::temp_dir().join("alphafield_test_models");
        ModelStorage::with_path(temp_dir)
    }

    #[test]
    fn test_generate_id() {
        let id = ModelStorage::generate_id(MLModelType::RandomForest, "BTCUSDT");
        assert!(id.starts_with("rf_btcusdt_"));
    }

    #[test]
    fn test_metadata_creation() {
        let meta = ModelMetadata::new(
            "test_model",
            MLModelType::LinearRegression,
            vec!["feature1".to_string()],
            vec!["BTCUSDT".to_string()],
            "1h",
            1,
        );

        assert_eq!(meta.id, "test_model");
        assert_eq!(meta.model_type, MLModelType::LinearRegression);
        assert_eq!(meta.prediction_horizon, 1);
    }

    #[test]
    fn test_save_load_linear_regression() {
        let storage = temp_storage();

        // Create and train a model
        let mut model = LinearRegression::new();
        let features = vec![vec![1.0, 2.0], vec![2.0, 4.0]];
        let labels = vec![5.0, 10.0];
        model.train(&features, &labels).unwrap();

        let id = "test_lr_model";
        let metadata = ModelMetadata::new(
            id,
            MLModelType::LinearRegression,
            vec!["x1".to_string(), "x2".to_string()],
            vec!["TEST".to_string()],
            "1h",
            1,
        );

        // Save
        let saved_id = storage.save_linear_regression(&model, metadata).unwrap();
        assert_eq!(saved_id, id);

        // Load
        let loaded = storage.load(id).unwrap();
        assert_eq!(loaded.metadata.id, id);

        if let ModelData::LinearRegression(loaded_model) = loaded.model_data {
            assert!(loaded_model.is_trained());
        } else {
            panic!("Wrong model type loaded");
        }

        // Cleanup
        let _ = storage.delete(id);
    }

    #[test]
    fn test_list_models() {
        let storage = temp_storage();

        // Should return empty list or existing models
        let result = storage.list_models();
        assert!(result.is_ok());
    }

    #[test]
    fn test_train_metrics_default() {
        let metrics = TrainMetrics::default();
        assert_eq!(metrics.train_mae, 0.0);
        assert_eq!(metrics.n_train_samples, 0);
    }
}
