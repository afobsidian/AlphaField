//! # AlphaField Backtest
//!
//! Event-driven backtesting engine

pub mod error;
pub mod portfolio;
pub mod exchange;
pub mod metrics;
pub mod engine;
pub mod strategy;

pub use error::BacktestError;
pub use portfolio::Portfolio;
pub use exchange::{ExchangeSimulator, SlippageModel};
pub use metrics::PerformanceMetrics;
pub use engine::BacktestEngine;
pub use strategy::{Strategy, StrategyCombiner, OrderRequest, OrderSide, OrderType};
