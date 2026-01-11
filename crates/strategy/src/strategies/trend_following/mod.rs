//! Trend Following Strategies Module
//!
//! This module contains trend-following trading strategies that aim to capture
//! sustained market movements. Trend-following strategies typically enter when
//! a trend is established and exit when the trend reverses.
//!
//! # Strategies Included
//! - **Golden Cross**: Classic SMA crossover strategy (50/200)
//! - **Breakout**: Price breakout from recent highs/lows
//! - **MA Crossover**: Generic moving average crossover with configurable periods
//! - **Adaptive MA**: Kaufman's Adaptive Moving Average (KAMA)
//! - **Triple MA**: Three moving average alignment system
//! - **MACD Trend**: MACD-based trend following
//! - **Parabolic SAR**: Parabolic SAR trailing stop strategy

pub mod adaptive_ma;
pub mod breakout;
pub mod golden_cross;
pub mod ma_crossover;
pub mod macd_trend;
pub mod parabolic_sar;
pub mod triple_ma;

// Re-export strategies for convenience
pub use adaptive_ma::AdaptiveMAStrategy;
pub use breakout::BreakoutStrategy;
pub use golden_cross::GoldenCrossStrategy;
pub use ma_crossover::MACrossoverStrategy;
pub use macd_trend::MacdTrendStrategy;
pub use parabolic_sar::ParabolicSARStrategy;
pub use triple_ma::TripleMAStrategy;
