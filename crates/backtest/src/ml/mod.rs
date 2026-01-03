//! Machine Learning Module for AlphaField
//!
//! This module provides ML-based trading models including:
//! - Feature engineering from OHLCV data
//! - Data normalization and preprocessing
//! - ML model implementations (regression, classification, ensemble)
//! - Model storage and persistence
//! - Walk-forward ML validation
//! - ML-based trading strategies

pub mod data_split;
pub mod ensemble;
pub mod features;
pub mod models;
pub mod normalization;
pub mod storage;
pub mod strategy;
pub mod validation;

// Re-export commonly used types
pub use data_split::{DataSplit, DataSplitter, SplitConfig};
pub use features::{FeatureConfig, FeatureExtractor, FeatureSet};
pub use models::{
    DecisionTree, LinearRegression, LogisticRegression, MLModel, MLModelType, RandomForest,
};
pub use normalization::{MinMaxScaler, Scaler, StandardScaler};
pub use storage::{ModelMetadata, ModelStorage, TrainMetrics};
pub use strategy::{MLStrategy, MLStrategyConfig};
pub use validation::{MLValidation, MLValidationResult, OverfitDetection};
