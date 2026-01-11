//! Mean Reversion Strategies Module
//!
//! This module contains mean reversion trading strategies that aim to profit from
//! price reversions to statistical averages. Mean reversion strategies typically
//! enter when price deviates significantly from its mean and exit when price
//! returns to the mean.
//!
//! # Strategies Included
//! - **Bollinger Bands**: Bollinger Band reversion with RSI confirmation
//! - **RSI Reversion**: Pure RSI-based mean reversion
//! - **Statistical Arbitrage**: Z-score based pairs trading (adapted for spot)
//! - **Stochastic Reversion**: Stochastic oscillator mean reversion
//! - **Keltner Channel**: Keltner channel reversion with volume confirmation
//! - **Price Channel**: Donchian channel reversion
//! - **Z-Score Reversion**: Statistical z-score reversion

pub mod bollinger_bands;
pub mod rsi_reversion;
pub mod stat_arb;
pub mod stoch_reversion;
pub mod keltner_reversion;
pub mod price_channel;
pub mod zscore_reversion;

// Re-export strategies for convenience
pub use bollinger_bands::BollingerBandsStrategy;
pub use rsi_reversion::RSIReversionStrategy;
pub use stat_arb::StatArbStrategy;
pub use stoch_reversion::StochReversionStrategy;
pub use keltner_reversion::KeltnerReversionStrategy;
pub use price_channel::PriceChannelStrategy;
pub use zscore_reversion::ZScoreReversionStrategy;
