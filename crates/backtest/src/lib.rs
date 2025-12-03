//! # AlphaField Backtest
//!
//! Event-driven backtesting engine

pub mod adapter;
pub mod engine;
pub mod error;
pub mod exchange;
pub mod metrics;
pub mod portfolio;
pub mod strategy;

pub use adapter::StrategyAdapter;
pub use engine::BacktestEngine;
pub use error::BacktestError;
pub use exchange::{ExchangeSimulator, SlippageModel};
pub use metrics::PerformanceMetrics;
pub use portfolio::Portfolio;
pub use strategy::{OrderRequest, OrderSide, OrderType, Strategy, StrategyCombiner};
