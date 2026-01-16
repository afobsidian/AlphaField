//! Multi-Indicator Strategies
//!
//! This module contains strategies that combine multiple indicators
//! in sophisticated ways: hybrid approaches, adaptive weighting,
//! ensemble methods, regime-switching, confidence-based sizing,
//! and ML-enhanced feature extraction.

pub mod adaptive_combo;
pub mod confidence_weighted;
pub mod ensemble_weighted;
pub mod macd_rsi_combo;
pub mod ml_enhanced;
pub mod regime_switching;
pub mod trend_mean_rev;

// Re-export strategies for convenience
pub use adaptive_combo::AdaptiveComboStrategy;
pub use confidence_weighted::ConfidenceWeightedStrategy;
pub use ensemble_weighted::EnsembleWeightedStrategy;
pub use macd_rsi_combo::MACDRSIComboStrategy;
pub use ml_enhanced::{MLEnhancedConfig, MLEnhancedStrategy};
pub use regime_switching::RegimeSwitchingStrategy;
pub use trend_mean_rev::TrendMeanRevStrategy;
