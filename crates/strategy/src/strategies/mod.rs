//! Modular strategy implementations
//!
//! This module provides a collection of trading strategies organized
//! into separate submodules for better maintainability and testing.

pub mod golden_cross;
pub mod mean_reversion;
pub mod momentum;
pub mod multi_indicator;
pub mod rsi;
pub mod sentiment;
pub mod trend_following;
pub mod volatility;

// Re-export strategies for convenience
pub use golden_cross::GoldenCrossStrategy;
pub use rsi::RsiStrategy;

// Re-export trend following strategies
pub use trend_following::*;

// Re-export mean reversion strategies
pub use mean_reversion::*;

// Re-export momentum strategies
pub use momentum::*;

// Re-export volatility strategies
pub use volatility::*;

// Re-export multi_indicator strategies
pub use multi_indicator::*;

// Re-export sentiment strategies
pub use sentiment::*;
