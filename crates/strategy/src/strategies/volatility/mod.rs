//! Volatility-based trading strategies
//!
//! This module contains volatility-focused strategies that adapt to market volatility
//! conditions. These strategies are designed to:
//! - Capture volatility expansions (breakouts)
//! - Trade volatility squeezes (low vol to high vol transitions)
//! - Use volatility for dynamic position sizing and risk management
//! - Adapt strategies based on volatility regimes
//! - Trade mean reversion during extreme volatility (contrarian)

pub mod atr_breakout;
pub mod atr_trailing;
pub mod garch_strategy;
pub mod vix_style;
pub mod vol_regime;
pub mod vol_sizing;
pub mod vol_squeeze;

// Re-export strategies for convenience
pub use atr_breakout::ATRBreakoutStrategy;
pub use atr_trailing::ATRTrailingStrategy;
pub use garch_strategy::GARCHStrategy;
pub use vix_style::VIXStyleStrategy;
pub use vol_regime::VolRegimeStrategy;
pub use vol_sizing::VolSizingStrategy;
pub use vol_squeeze::VolSqueezeStrategy;
