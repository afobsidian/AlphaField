//! Modular strategy implementations
//!
//! This module provides a collection of trading strategies organized
//! into separate submodules for better maintainability and testing.

pub mod golden_cross;
pub mod mean_reversion;
pub mod momentum;
pub mod rsi;
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
