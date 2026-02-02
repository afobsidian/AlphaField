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
pub mod journal;
pub mod metrics;
pub mod ml;
pub mod monte_carlo;
pub mod multi_strategy_engine;
pub mod optimization_workflow;
pub mod optimizer;
pub mod portfolio;
pub mod portfolio_optimization;
pub mod portfolio_validation;
pub mod position_sizing;
pub mod reports;
pub mod rolling_stats;
pub mod sensitivity;
pub mod strategy;
pub mod tax;
pub mod trade;
pub mod validation;
pub mod walk_forward;

pub use journal::{JournalEntry, TradeJournal};
pub use reports::{PerformanceReport, PeriodSummary, ReportPeriod, StrategyBreakdown};
pub use tax::{CostBasisMethod, TaxCalculator, TaxLot, TaxSummary, TaxableEvent};

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
pub use multi_strategy_engine::{
    MultiStrategyBacktestEngine, MultiStrategyBacktestResult, MultiStrategyConfig,
    RebalancingEvent, StrategyPerformance,
};
pub use optimization_workflow::{
    OptimizationWorkflow, ParameterDispersion, WorkflowConfig, WorkflowResult,
};
pub use optimizer::{
    get_strategy_bounds, OptimizationResult as ParamOptimizationResult, ParamBounds,
    ParamSweepResult, ParameterOptimizer,
};
pub use portfolio::Portfolio;
pub use portfolio_optimization::{
    algorithms::inverse_volatility::InverseVolatilityOptimizer,
    algorithms::mean_variance::MeanVarianceOptimizer,
    algorithms::risk_parity::RiskParityOptimizer,
    algorithms::{optimize_with_multiple_methods, OptimizerFactory},
    constraints::{PortfolioConstraint, WeightConstraint},
    objectives::{
        DiversificationObjective, OptimizationObjective, PortfolioObjective, ReturnObjective,
        RiskParityObjective, SharpeObjective, VolatilityObjective,
    },
    optimizer::{
        prepare_optimization_data, OptimizationConfig, PortfolioOptimizer, StrategyAllocation,
    },
    MultiStrategyPortfolio, OptimizationResult, StrategyMetadata,
};
pub use portfolio_validation::{
    monte_carlo::{
        PortfolioMonteCarloConfig, PortfolioMonteCarloResult, PortfolioMonteCarloSimulator,
    },
    sensitivity::{
        PortfolioSensitivityAnalyzer, PortfolioSensitivityResult, StrategyImpact, WeightAdjustment,
    },
    stress_test::{
        StressScenario, StressTestConfig, StressTestResult, StressTester, TailRiskMetrics,
    },
    walk_forward::{
        PortfolioWalkForwardAnalyzer, PortfolioWalkForwardConfig, PortfolioWalkForwardResult,
        WalkForwardWindowResult,
    },
    PortfolioValidationReport, PortfolioValidator,
};
pub use position_sizing::{
    kelly::KellyOpportunity, FixedFractionalSizing, KellyCriterion, KellyFraction, KellyResult,
    PositionSizing, SizingResult, TradeStatistics, VolatilityAdjustedSizing,
    VolatilitySizingConfig,
};
pub use rolling_stats::{MonthlyReturn, RollingStats};
pub use sensitivity::{ParameterRange, SensitivityAnalyzer, SensitivityConfig, SensitivityResult};
pub use strategy::{BuyAndHold, OrderRequest, OrderSide, OrderType, Strategy, StrategyCombiner};
pub use trade::{Trade, TradeSide, TradeStats};
pub use validation::{
    BacktestResult, DeploymentRecommendation, MarketRegime, Recommendations, RegimeAnalysisResult,
    StrategyValidator, ValidationConfig, ValidationReport, ValidationThresholds, ValidationVerdict,
};
pub use walk_forward::{WalkForwardAnalyzer, WalkForwardConfig, WalkForwardResult};
