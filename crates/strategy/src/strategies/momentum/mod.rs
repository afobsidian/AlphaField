//! Momentum-based trading strategies
//!
//! This module contains momentum strategies that identify and follow strong price trends.
//! Momentum strategies typically perform best in trending markets with clear directional movement.

pub mod adx_trend;
pub mod macd_strategy;
pub mod momentum_factor;
pub mod multi_tf_momentum;
pub mod roc_strategy;
pub mod rsi_momentum;
pub mod volume_momentum;

// Re-export strategies for convenience
pub use adx_trend::AdxTrendStrategy;
pub use macd_strategy::MomentumStrategy as MACDStrategy; // Rename for clarity
pub use momentum_factor::MomentumFactorStrategy;
pub use multi_tf_momentum::MultiTfMomentumStrategy;
pub use roc_strategy::RocStrategy;
pub use rsi_momentum::RsiMomentumStrategy;
pub use volume_momentum::VolumeMomentumStrategy;
