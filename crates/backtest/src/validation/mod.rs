//! # Automated Strategy Validation Framework
//!
//! Comprehensive validation system for trading strategies including
//! backtesting, walk-forward analysis, Monte Carlo simulation, and
//! regime-based performance analysis.

pub mod regime;
pub mod regime_testing;
pub mod robustness;
pub mod scoring;
pub mod statistical_significance;
pub mod temporal;
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
    /// Phase 13: Maximum complexity parameters (for complexity penalty)
    pub max_parameters: usize,
    /// Phase 13: Maximum indicators (for complexity penalty)
    pub max_indicators: usize,
    /// Phase 13: Maximum branches (for complexity penalty)
    pub max_branches: usize,
    /// Phase 13: Noise levels for perturbation testing (as decimal, e.g., 0.01 for 1%)
    pub perturbation_noise_levels: Vec<f64>,
    /// Phase 13: Window size as fraction of total trades (0.0-1.0)
    pub rolling_window_fraction: f64,
    /// Phase 13: Expanding window step as fraction of total trades (0.0-1.0)
    pub expanding_window_step_fraction: f64,
    /// Phase 13: Maximum iterations for bootstrap/permutation tests
    pub max_statistical_iterations: usize,
    /// Phase 13: Enable early stopping for large datasets
    pub enable_early_stopping: bool,
    /// Phase 13: Timeout in seconds for statistical tests (0 = no timeout)
    pub statistical_timeout_seconds: Option<u64>,
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
            // Phase 13 defaults
            max_parameters: 10,
            max_indicators: 5,
            max_branches: 20,
            perturbation_noise_levels: vec![0.01, 0.02, 0.05],
            rolling_window_fraction: 0.5,
            expanding_window_step_fraction: 0.2,
            max_statistical_iterations: 1000,
            enable_early_stopping: true,
            statistical_timeout_seconds: Some(30),
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

    /// Long/Short trade breakdown (top-level for API convenience)
    pub long_trades_count: usize,
    pub short_trades_count: usize,
    pub long_win_rate: f64,
    pub short_win_rate: f64,

    /// Component results
    pub backtest: BacktestResult,
    pub walk_forward: WalkForwardResult,
    pub monte_carlo: MonteCarloResult,
    pub regime_analysis: RegimeAnalysisResult,

    /// Phase 13 Advanced Validation
    pub statistical_significance: Option<statistical_significance::StatisticalSignificanceResult>,
    pub robustness: Option<robustness::RobustnessResult>,
    pub temporal_validation: Option<temporal::TemporalValidationResult>,
    pub regime_testing: Option<regime_testing::RegimeTestingResult>,

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
    /// Long/Short breakdown metrics
    pub long_trades_count: usize,
    pub short_trades_count: usize,
    pub long_win_rate: f64,
    pub short_win_rate: f64,
    pub avg_long_profit: f64,
    pub avg_short_profit: f64,
    pub total_long_profit: f64,
    pub total_short_profit: f64,
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
    // Phase 13 components
    pub statistical_significance: Option<statistical_significance::StatisticalSignificanceResult>,
    pub robustness: Option<robustness::RobustnessResult>,
    pub temporal_validation: Option<temporal::TemporalValidationResult>,
    pub regime_testing: Option<regime_testing::RegimeTestingResult>,
}

impl ValidationComponents {
    /// Run all Phase 13 advanced validations based on available data
    ///
    /// This method automatically runs statistical significance, regime testing,
    /// temporal validation, and robustness analysis if sufficient data is available.
    ///
    /// # Returns
    /// Updated ValidationComponents with Phase 13 results populated
    pub fn run_phase_13_validations(mut self) -> Self {
        let risk_free_rate = self.config.risk_free_rate;

        // Statistical Significance Validation
        if !self.backtest.trades.is_empty() {
            let stat_sig_result = statistical_significance::validate_statistical_significance(
                &self.backtest.trades,
                &self.backtest.metrics,
                risk_free_rate,
                None,
                None,
                self.config.statistical_timeout_seconds,
            );
            self.statistical_significance = Some(stat_sig_result.ok().unwrap_or_default());
        }

        // Regime Testing
        if !self.backtest.trades.is_empty() {
            let regime_test_result = regime_testing::validate_regime_testing(
                &self.backtest.trades,
                &self.backtest.metrics,
                risk_free_rate,
            );
            self.regime_testing = Some(regime_test_result);
        }

        // Temporal Validation
        if self.backtest.trades.len() > 252 {
            let temporal_result = temporal::validate_temporal(
                &self.backtest.trades,
                &self.backtest.metrics,
                risk_free_rate,
                None, // paper_sharpe
                None, // live_sharpe
                self.config.expanding_window_step_fraction,
                self.config.rolling_window_fraction,
            );
            self.temporal_validation = Some(temporal_result);
        }

        // Robustness Validation
        let strategy_params = robustness::StrategyParams {
            n_parameters: self.config.max_parameters,
            n_indicators: self.config.max_indicators,
            n_branches: self.config.max_branches,
            timeframe_results: vec![],
            cross_asset_results: vec![],
        };
        let robustness_result = robustness::validate_robustness(
            &self.backtest.trades,
            &self.backtest.metrics,
            risk_free_rate,
            &strategy_params,
            &self.config.perturbation_noise_levels,
            self.config.max_parameters,
            self.config.max_indicators,
            self.config.max_branches,
        );
        self.robustness = Some(robustness_result);

        self
    }
}

/// Shared helper: Calculate Sharpe ratio from returns
///
/// This function is used across multiple validation modules to ensure
/// consistent Sharpe ratio calculations.
///
/// # Arguments
/// * `returns` - Vector of returns (e.g., from trades)
/// * `risk_free_rate` - Annual risk-free rate (default: 0.02)
///
/// # Returns
/// * Sharpe ratio (annualized), or 0.0 if insufficient data
///
/// # Notes
/// - Annualizes assuming daily returns (252 trading days per year)
/// - Returns 0.0 for empty returns or zero standard deviation
pub fn calculate_sharpe_from_returns(returns: &[f64], risk_free_rate: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;

    let mut variance = 0.0;
    for &r in returns {
        variance += (r - mean_return).powi(2);
    }
    variance /= (returns.len() - 1) as f64;

    let std_dev = variance.sqrt();

    if std_dev < 1e-10 {
        0.0
    } else {
        // Annualize (assuming daily returns)
        let annual_mean = mean_return * 252.0;
        let annual_std = std_dev * (252.0_f64).sqrt();
        (annual_mean - risk_free_rate) / annual_std
    }
}

// Re-export regime module types
pub use regime::{MarketRegime, RegimeAnalysisResult, RegimeAnalyzer, RegimePerformance};
pub use scoring::{RecommendationsGenerator, ScoreCalculator, ScoreWeights};

// Re-export Phase 13 advanced validation modules
pub use regime_testing::{validate_regime_testing, RegimeTestingResult};
pub use robustness::{validate_robustness, RobustnessResult, StrategyParams};
pub use statistical_significance::{
    validate_statistical_significance, StatisticalSignificanceResult,
};
pub use temporal::{validate_temporal, TemporalValidationResult};

// Re-export main validator
pub use validator::StrategyValidator;
