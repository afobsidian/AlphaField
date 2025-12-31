//! # AlphaField Backtest
//!
//! Event-driven backtesting engine with advanced analytics

pub mod adapter;
pub mod asset_sentiment;
pub mod benchmark;
pub mod correlation;
pub mod drawdown;
pub mod engine;
pub mod error;
pub mod exchange;
pub mod metrics;
pub mod monte_carlo;
pub mod optimizer;
pub mod portfolio;
pub mod rolling_stats;
pub mod sensitivity;
pub mod strategy;
pub mod trade;
pub mod walk_forward;

pub use adapter::StrategyAdapter;
pub use asset_sentiment::{
    AssetSentiment, AssetSentimentCalculator, AssetSentimentClassification, AssetSentimentSummary,
    MomentumZone, RsiZone,
};
pub use benchmark::{BenchmarkComparison, BenchmarkConfig, BenchmarkType};
pub use correlation::{
    CorrelationAnalyzer, CorrelationConfig, CorrelationMatrix, CorrelationResult,
};
pub use drawdown::{DrawdownAnalysis, DrawdownPeriod, DrawdownPoint};
pub use engine::BacktestEngine;
pub use error::BacktestError;
pub use exchange::{ExchangeSimulator, SlippageModel};
pub use metrics::PerformanceMetrics;
pub use monte_carlo::{MonteCarloConfig, MonteCarloResult, MonteCarloSimulator};
pub use optimizer::{
    get_strategy_bounds, OptimizationResult, ParamBounds, ParamSweepResult, ParameterOptimizer,
};
pub use portfolio::Portfolio;
pub use rolling_stats::{MonthlyReturn, RollingStats};
pub use sensitivity::{ParameterRange, SensitivityAnalyzer, SensitivityConfig, SensitivityResult};
pub use strategy::{OrderRequest, OrderSide, OrderType, Strategy, StrategyCombiner};
pub use trade::{Trade, TradeSide, TradeStats};
pub use walk_forward::{WalkForwardAnalyzer, WalkForwardConfig, WalkForwardResult};
