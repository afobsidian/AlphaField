//! Modular strategy implementations
//!
//! This module provides a collection of trading strategies organized
//! into separate submodules for better maintainability and testing.

pub mod golden_cross;
pub mod mean_reversion;
pub mod momentum;
pub mod rsi;
pub mod trend_following;

// Re-export strategies for convenience
pub use golden_cross::GoldenCrossStrategy;
pub use mean_reversion::MeanReversionStrategy;
pub use momentum::MomentumStrategy;
pub use rsi::RsiStrategy;
pub use trend_following::GoldenCrossStrategy as TrendGoldenCrossStrategy;
