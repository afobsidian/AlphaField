//! Portfolio Optimizer Trait and Configuration
//!
//! Defines the core optimizer interface that all portfolio optimization
//! algorithms must implement.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::constraints::PortfolioConstraint;
use super::objectives::OptimizationObjective;
use super::{OptimizationResult, StrategyMetadata};

/// Type alias for mean returns vector
type MeanReturns = Vec<f64>;
/// Type alias for return series matrix (strategies x time periods)
type ReturnSeries = Vec<Vec<f64>>;
/// Type alias for covariance matrix
type CovarianceMatrix = Vec<Vec<f64>>;

/// Configuration for portfolio optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Optimization objective
    pub objective: OptimizationObjective,
    /// Risk-free rate (for Sharpe calculation)
    pub risk_free_rate: f64,
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Convergence tolerance
    pub tolerance: f64,
    /// Random seed (for stochastic optimizers)
    pub random_seed: Option<u64>,
    /// Initial capital for equity curve calculations
    pub initial_capital: f64,
    /// Portfolio constraints
    pub constraints: PortfolioConstraint,
    /// Optimization method-specific parameters
    pub method_params: HashMap<String, f64>,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            objective: OptimizationObjective::MaximizeSharpe,
            risk_free_rate: 0.02,
            max_iterations: 1000,
            tolerance: 1e-6,
            random_seed: None,
            initial_capital: 10000.0,
            constraints: PortfolioConstraint::new(),
            method_params: HashMap::new(),
        }
    }
}

impl OptimizationConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the optimization objective
    pub fn with_objective(mut self, objective: OptimizationObjective) -> Self {
        self.objective = objective;
        self
    }

    /// Set the risk-free rate
    pub fn with_risk_free_rate(mut self, rate: f64) -> Self {
        self.risk_free_rate = rate.clamp(0.0, 0.5);
        self
    }

    /// Set the maximum number of iterations
    pub fn with_max_iterations(mut self, iterations: usize) -> Self {
        self.max_iterations = iterations.max(100);
        self
    }

    /// Set the convergence tolerance
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance.clamp(1e-10, 1.0);
        self
    }

    /// Set a random seed for reproducibility
    pub fn with_random_seed(mut self, seed: u64) -> Self {
        self.random_seed = Some(seed);
        self
    }

    /// Set initial capital
    pub fn with_initial_capital(mut self, capital: f64) -> Self {
        self.initial_capital = capital.max(100.0);
        self
    }

    /// Set portfolio constraints
    pub fn with_constraints(mut self, constraints: PortfolioConstraint) -> Self {
        self.constraints = constraints;
        self
    }

    /// Add a method-specific parameter
    pub fn with_param(mut self, key: impl Into<String>, value: f64) -> Self {
        self.method_params.insert(key.into(), value);
        self
    }
}

/// Allocation for a single strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAllocation {
    /// Strategy name
    pub name: String,
    /// Allocation weight (0.0 to 1.0)
    pub weight: f64,
    /// Risk contribution (for risk parity)
    pub risk_contribution: Option<f64>,
    /// Marginal contribution to portfolio return
    pub marginal_return: Option<f64>,
    /// Strategy metadata
    pub metadata: Option<StrategyMetadata>,
}

impl StrategyAllocation {
    pub fn new(name: impl Into<String>, weight: f64) -> Self {
        Self {
            name: name.into(),
            weight: weight.clamp(0.0, 1.0),
            risk_contribution: None,
            marginal_return: None,
            metadata: None,
        }
    }

    pub fn with_risk_contribution(mut self, rc: f64) -> Self {
        self.risk_contribution = Some(rc);
        self
    }

    pub fn with_metadata(mut self, metadata: StrategyMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Core trait for portfolio optimizers
pub trait PortfolioOptimizer {
    /// Optimize a portfolio given historical data
    ///
    /// # Arguments
    /// * `strategy_names` - List of strategy identifiers
    /// * `equity_curves` - Historical equity curves for each strategy
    /// * `config` - Optimization configuration
    ///
    /// # Returns
    /// Optimization result with optimal allocations
    fn optimize(
        &self,
        strategy_names: &[String],
        equity_curves: &HashMap<String, Vec<f64>>,
        config: &OptimizationConfig,
    ) -> Result<OptimizationResult, String>;

    /// Get the optimizer name
    fn name(&self) -> &'static str;

    /// Get the optimizer description
    fn description(&self) -> &'static str;

    /// Check if this optimizer supports a given objective
    fn supports_objective(&self, objective: OptimizationObjective) -> bool;
}

/// Helper function to calculate returns from equity curve
pub fn equity_to_returns(equity: &[f64]) -> Vec<f64> {
    if equity.len() < 2 {
        return Vec::new();
    }

    equity
        .windows(2)
        .map(|w| {
            if w[0] != 0.0 {
                (w[1] - w[0]) / w[0]
            } else {
                0.0
            }
        })
        .collect()
}

/// Helper function to calculate mean return
pub fn calculate_mean_return(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    returns.iter().sum::<f64>() / returns.len() as f64
}

/// Helper function to calculate volatility (standard deviation)
pub fn calculate_volatility(returns: &[f64]) -> f64 {
    if returns.len() < 2 {
        return 0.0;
    }

    let mean = calculate_mean_return(returns);
    let variance =
        returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;

    variance.sqrt()
}

/// Helper function to calculate covariance matrix from return series
pub fn calculate_covariance_matrix(return_series: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, String> {
    if return_series.len() < 2 {
        return Err("Need at least 2 return series".to_string());
    }

    // Find minimum length
    let min_len = return_series.iter().map(|s| s.len()).min().unwrap_or(0);
    if min_len < 2 {
        return Err("Insufficient data points".to_string());
    }

    let n = return_series.len();
    let mut cov_matrix = vec![vec![0.0; n]; n];

    // Calculate means for each series
    let means: Vec<f64> = return_series
        .iter()
        .map(|s| calculate_mean_return(&s[..min_len]))
        .collect();

    // Calculate covariance matrix
    for i in 0..n {
        for j in 0..n {
            let cov = calculate_pairwise_covariance(
                &return_series[i][..min_len],
                &return_series[j][..min_len],
                means[i],
                means[j],
            );
            cov_matrix[i][j] = cov;
        }
    }

    Ok(cov_matrix)
}

/// Calculate pairwise covariance
fn calculate_pairwise_covariance(x: &[f64], y: &[f64], mean_x: f64, mean_y: f64) -> f64 {
    if x.len() != y.len() || x.len() < 2 {
        return 0.0;
    }

    let n = x.len();
    let sum: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
        .sum();

    sum / (n - 1) as f64
}

/// Helper function to prepare optimization data from equity curves
pub fn prepare_optimization_data(
    strategy_names: &[String],
    equity_curves: &HashMap<String, Vec<f64>>,
) -> Result<(MeanReturns, ReturnSeries, CovarianceMatrix), String> {
    if strategy_names.len() < 2 {
        return Err("Need at least 2 strategies for portfolio optimization".to_string());
    }

    // Collect returns for each strategy
    let mut return_series: Vec<Vec<f64>> = Vec::new();
    let mut means: Vec<f64> = Vec::new();

    for name in strategy_names {
        let equity = equity_curves
            .get(name)
            .ok_or_else(|| format!("Missing equity curve for strategy: {}", name))?;

        let returns = equity_to_returns(equity);
        if returns.len() < 2 {
            return Err(format!(
                "Insufficient data points for strategy: {} (need at least 2)",
                name
            ));
        }

        // Annualize returns (assuming daily data)
        let mean_return = calculate_mean_return(&returns) * 252.0;

        means.push(mean_return);
        return_series.push(returns);
    }

    // Calculate covariance matrix (daily units)
    let mut cov_matrix = calculate_covariance_matrix(&return_series)?;

    // Annualize covariance matrix to match annualized mean returns (×252)
    for row in cov_matrix.iter_mut() {
        for value in row.iter_mut() {
            *value *= 252.0;
        }
    }

    Ok((means, return_series, cov_matrix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_config_builder() {
        let config = OptimizationConfig::new()
            .with_objective(OptimizationObjective::MaximizeSharpe)
            .with_risk_free_rate(0.03)
            .with_max_iterations(500)
            .with_tolerance(1e-5)
            .with_random_seed(42);

        assert_eq!(config.objective, OptimizationObjective::MaximizeSharpe);
        assert_eq!(config.risk_free_rate, 0.03);
        assert_eq!(config.max_iterations, 500);
        assert_eq!(config.tolerance, 1e-5);
        assert_eq!(config.random_seed, Some(42));
    }

    #[test]
    fn test_strategy_allocation() {
        let allocation = StrategyAllocation::new("Test Strategy", 0.3).with_risk_contribution(0.05);

        assert_eq!(allocation.name, "Test Strategy");
        assert_eq!(allocation.weight, 0.3);
        assert_eq!(allocation.risk_contribution, Some(0.05));
    }

    #[test]
    fn test_equity_to_returns() {
        let equity = vec![100.0, 110.0, 121.0, 110.0];
        let returns = equity_to_returns(&equity);

        assert_eq!(returns.len(), 3);
        assert!((returns[0] - 0.10).abs() < 0.001); // 10% return
        assert!((returns[1] - 0.10).abs() < 0.001); // 10% return
        assert!((returns[2] - (-0.0909)).abs() < 0.01); // -9.09% return
    }

    #[test]
    fn test_calculate_mean_return() {
        let returns = vec![0.01, 0.02, -0.01, 0.03];
        let mean = calculate_mean_return(&returns);

        assert!((mean - 0.0125).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_volatility() {
        let returns = vec![0.01, 0.02, -0.01, 0.03];
        let vol = calculate_volatility(&returns);

        assert!(vol > 0.0);
        assert!(vol < 0.1);
    }

    #[test]
    fn test_calculate_covariance_matrix() {
        // Two strategies with different returns
        let returns_a = vec![0.01, 0.02, -0.01, 0.03];
        let returns_b = vec![0.005, 0.015, -0.005, 0.025];
        let series = vec![returns_a, returns_b];

        let cov_matrix = calculate_covariance_matrix(&series).unwrap();

        assert_eq!(cov_matrix.len(), 2);
        assert_eq!(cov_matrix[0].len(), 2);
        assert!(cov_matrix[0][0] > 0.0); // Variance should be positive
        assert!(cov_matrix[1][1] > 0.0);
    }

    #[test]
    fn test_covariance_matrix_error_too_few() {
        let series = vec![vec![0.01, 0.02]];
        let result = calculate_covariance_matrix(&series);
        assert!(result.is_err());
    }

    #[test]
    fn test_prepare_optimization_data() {
        let mut equity_curves = HashMap::new();
        equity_curves.insert("Strategy A".to_string(), vec![100.0, 110.0, 121.0, 110.0]);
        equity_curves.insert("Strategy B".to_string(), vec![100.0, 105.0, 110.0, 115.0]);

        let strategy_names = vec!["Strategy A".to_string(), "Strategy B".to_string()];

        let result = prepare_optimization_data(&strategy_names, &equity_curves);

        assert!(result.is_ok());
        let (means, _, cov) = result.unwrap();
        assert_eq!(means.len(), 2);
        assert_eq!(cov.len(), 2);
    }

    #[test]
    fn test_prepare_data_missing_strategy() {
        let equity_curves = HashMap::new();
        let strategy_names = vec!["Missing Strategy".to_string()];

        let result = prepare_optimization_data(&strategy_names, &equity_curves);
        assert!(result.is_err());
    }
}
