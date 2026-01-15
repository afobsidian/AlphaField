//! # Automated Strategy Validation Framework
//!
//! Comprehensive validation system for trading strategies including
//! backtesting, walk-forward analysis, Monte Carlo simulation, and
//! regime-based performance analysis.

pub mod regime;
pub mod scoring;
pub mod validator;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::metrics::PerformanceMetrics;
use crate::monte_carlo::MonteCarloResult;
use crate::trade::Trade;
use crate::walk_forward::{WalkForwardConfig, WalkForwardResult};

/// Configuration for strategy validation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Data source (database connection string)
    pub data_source: String,
    /// Symbol to validate against
    pub symbol: String,
    /// Timeframe/interval
    pub interval: String,
    /// Walk-forward configuration
    pub walk_forward: WalkForwardConfig,
    /// Risk-free rate for Sharpe calculation
    pub risk_free_rate: f64,
    /// Pass/fail thresholds
    pub thresholds: ValidationThresholds,
    /// Initial capital for backtesting
    pub initial_capital: f64,
    /// Fee rate for simulated trading
    pub fee_rate: f64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            data_source: String::new(),
            symbol: String::new(),
            interval: String::new(),
            walk_forward: WalkForwardConfig::default(),
            risk_free_rate: 0.02,
            thresholds: ValidationThresholds::default(),
            initial_capital: 10000.0,
            fee_rate: 0.001,
        }
    }
}

/// Thresholds for pass/fail verdicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationThresholds {
    /// Minimum Sharpe ratio to pass
    pub min_sharpe: f64,
    /// Maximum acceptable drawdown (as percentage, e.g., 0.30 for 30%)
    pub max_drawdown: f64,
    /// Minimum walk-forward win rate (percentage of profitable windows)
    pub min_win_rate: f64,
    /// Minimum Monte Carlo positive return probability
    pub min_positive_probability: f64,
}

impl Default for ValidationThresholds {
    fn default() -> Self {
        Self {
            min_sharpe: 1.0,
            max_drawdown: 0.30,
            min_win_rate: 0.60,
            min_positive_probability: 0.70,
        }
    }
}

/// Test period information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPeriod {
    /// Start date
    pub start: DateTime<Utc>,
    /// End date
    pub end: DateTime<Utc>,
    /// Total number of bars
    pub total_bars: usize,
}

/// Comprehensive validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Strategy name
    pub strategy_name: String,
    /// Validation timestamp
    pub validated_at: DateTime<Utc>,
    /// Data period tested
    pub test_period: TestPeriod,

    /// Overall scores
    pub overall_score: f64, // 0-100
    pub grade: char, // A-F
    pub verdict: ValidationVerdict,

    /// Component results
    pub backtest: BacktestResult,
    pub walk_forward: WalkForwardResult,
    pub monte_carlo: MonteCarloResult,
    pub regime_analysis: RegimeAnalysisResult,

    /// Risk assessment
    pub risk_assessment: RiskAssessment,

    /// Recommendations
    pub recommendations: Recommendations,
}

/// Validation verdict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationVerdict {
    /// Strategy passes all criteria
    Pass,
    /// Strategy fails key criteria
    Fail,
    /// Strategy shows promise but needs optimization
    NeedsOptimization,
}

/// Backtest result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    /// Performance metrics from backtest
    pub metrics: PerformanceMetrics,
    /// Number of trades executed
    pub total_trades: usize,
    /// Win rate
    pub win_rate: f64,
    /// Profit factor
    pub profit_factor: f64,
    /// Completed trades from backtest
    pub trades: Vec<Trade>,
}

/// Risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Expected maximum drawdown
    pub expected_max_drawdown: f64,
    /// Actual maximum drawdown
    pub actual_max_drawdown: f64,
    /// Tail risk (5th percentile return)
    pub tail_risk: f64,
    /// Exposure profile (average position size as % of capital)
    pub avg_exposure: f64,
    /// Leverage check (should be 1.0 for spot-only)
    pub leverage: f64,
    /// Risk rating
    pub risk_rating: RiskRating,
}

/// Risk rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskRating {
    Low,
    Medium,
    High,
    Extreme,
}

/// Recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendations {
    /// Strategy strengths
    pub strengths: Vec<String>,
    /// Strategy weaknesses
    pub weaknesses: Vec<String>,
    /// Improvement suggestions
    pub improvements: Vec<String>,
    /// Deployment recommendation
    pub deployment: DeploymentRecommendation,
}

/// Deployment recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentRecommendation {
    /// Ready for deployment
    Deploy { confidence: f64 },
    /// Optimize parameters then re-validate
    OptimizeThenValidate { params: Vec<String> },
    /// Reject strategy
    Reject { reason: String },
}

/// Validation components used for scoring
#[derive(Debug, Clone)]
pub struct ValidationComponents {
    pub backtest: BacktestResult,
    pub walk_forward: WalkForwardResult,
    pub monte_carlo: MonteCarloResult,
    pub regime: RegimeAnalysisResult,
    pub config: ValidationConfig,
}

// Re-export regime module types
pub use regime::{MarketRegime, RegimeAnalysisResult, RegimeAnalyzer, RegimePerformance};
pub use scoring::{RecommendationsGenerator, ScoreCalculator, ScoreWeights};
pub use validator::StrategyValidator;
