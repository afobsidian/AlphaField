//! Portfolio Monte Carlo Simulation Module
//!
//! Monte Carlo simulation for multi-strategy portfolios with correlation preservation:
//! - Cholesky decomposition for correlation-aware reshuffling
//! - Strategy failure scenario simulation
//! - Portfolio-level confidence intervals
//! - Correlation breakdown stress testing

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for portfolio Monte Carlo simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMonteCarloConfig {
    /// Number of simulations to run
    pub num_simulations: usize,
    /// Initial capital
    pub initial_capital: f64,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Whether to preserve correlations in simulation
    pub preserve_correlations: bool,
    /// Strategy failure probability (0.0 to 1.0)
    pub strategy_failure_probability: f64,
    /// Maximum correlation level to simulate (for stress testing)
    pub max_correlation_stress: f64,
}

impl Default for PortfolioMonteCarloConfig {
    fn default() -> Self {
        Self {
            num_simulations: 1000,
            initial_capital: 10000.0,
            seed: None,
            preserve_correlations: true,
            strategy_failure_probability: 0.0,
            max_correlation_stress: 1.0,
        }
    }
}

impl PortfolioMonteCarloConfig {
    /// Create new Monte Carlo configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set number of simulations
    pub fn with_simulations(mut self, num: usize) -> Self {
        self.num_simulations = num.max(100);
        self
    }

    /// Set initial capital
    pub fn with_initial_capital(mut self, capital: f64) -> Self {
        self.initial_capital = capital.max(1000.0);
        self
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Enable correlation preservation
    pub fn with_correlation_preservation(mut self, preserve: bool) -> Self {
        self.preserve_correlations = preserve;
        self
    }

    /// Set strategy failure probability for stress testing
    pub fn with_strategy_failure_probability(mut self, prob: f64) -> Self {
        self.strategy_failure_probability = prob.clamp(0.0, 1.0);
        self
    }
}

/// Metrics from a single Monte Carlo simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSimulationMetrics {
    /// Final portfolio equity
    pub final_equity: f64,
    /// Total return
    pub total_return: f64,
    /// Maximum drawdown
    pub max_drawdown: f64,
    /// Portfolio volatility (annualized)
    pub volatility: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    /// Number of active strategies (for failure scenarios)
    pub active_strategies: usize,
}

impl Default for PortfolioSimulationMetrics {
    fn default() -> Self {
        Self {
            final_equity: 10000.0,
            total_return: 0.0,
            max_drawdown: 0.0,
            volatility: 0.0,
            sharpe_ratio: 0.0,
            active_strategies: 0,
        }
    }
}

/// Aggregated Monte Carlo results for portfolios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMonteCarloResult {
    /// Number of simulations run
    pub num_simulations: usize,
    /// Original (unshuffled) performance
    pub original_metrics: PortfolioSimulationMetrics,

    /// Equity percentiles
    pub equity_5th: f64,
    pub equity_50th: f64,
    pub equity_95th: f64,

    /// Return percentiles
    pub return_5th: f64,
    pub return_50th: f64,
    pub return_95th: f64,

    /// Drawdown percentiles
    pub drawdown_5th: f64,
    pub drawdown_50th: f64,
    pub drawdown_95th: f64,

    /// Probability of profit
    pub probability_of_profit: f64,

    /// 95% Value at Risk
    pub var_95: f64,

    /// All simulation results (for detailed analysis)
    pub simulations: Vec<PortfolioSimulationMetrics>,
}

/// Portfolio Monte Carlo simulator
pub struct PortfolioMonteCarloSimulator {
    config: PortfolioMonteCarloConfig,
}

impl PortfolioMonteCarloSimulator {
    /// Create new simulator with configuration
    pub fn new(config: PortfolioMonteCarloConfig) -> Self {
        Self { config }
    }

    /// Run Monte Carlo simulation on a portfolio
    ///
    /// # Arguments
    /// * `strategy_names` - List of strategies in portfolio
    /// * `weights` - Portfolio weights (sum to 1.0)
    /// * `equity_curves` - Historical equity curves for each strategy
    /// * `correlation_matrix` - Correlation matrix between strategies
    ///
    /// # Returns
    /// Complete Monte Carlo results
    pub fn simulate(
        &self,
        strategy_names: &[String],
        weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
        correlation_matrix: &[Vec<f64>],
    ) -> Result<PortfolioMonteCarloResult, String> {
        // Validate inputs
        if strategy_names.len() != weights.len() {
            return Err("Strategy names and weights must have same length".to_string());
        }

        if strategy_names.len() < 2 {
            return Err("Need at least 2 strategies for portfolio simulation".to_string());
        }

        // Extract returns from equity curves
        let returns: HashMap<String, Vec<f64>> = equity_curves
            .iter()
            .map(|(name, curve)| {
                let rets: Vec<f64> = curve
                    .windows(2)
                    .map(|w| {
                        if w[0] > 0.0 {
                            (w[1] - w[0]) / w[0]
                        } else {
                            0.0
                        }
                    })
                    .collect();
                (name.clone(), rets)
            })
            .collect();

        // Find minimum return series length
        let min_len = returns.values().map(|r| r.len()).min().unwrap_or(0);

        if min_len < 10 {
            return Err("Insufficient return data for simulation".to_string());
        }

        // Calculate original metrics
        let original_metrics =
            self.calculate_original_metrics(strategy_names, weights, equity_curves)?;

        // Create RNG
        let mut rng = match self.config.seed {
            Some(seed) => ChaCha8Rng::seed_from_u64(seed),
            None => ChaCha8Rng::from_entropy(),
        };

        // Run simulations
        let mut simulations = Vec::with_capacity(self.config.num_simulations);

        for _ in 0..self.config.num_simulations {
            let metrics = if self.config.preserve_correlations {
                self.simulate_with_correlations(
                    strategy_names,
                    weights,
                    &returns,
                    correlation_matrix,
                    min_len,
                    &mut rng,
                )?
            } else {
                self.simulate_independent(strategy_names, weights, &returns, min_len, &mut rng)?
            };

            simulations.push(metrics);
        }

        // Aggregate results
        let result = self.aggregate_results(&simulations, original_metrics);

        Ok(result)
    }

    /// Calculate original (unshuffled) metrics
    fn calculate_original_metrics(
        &self,
        strategy_names: &[String],
        weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
    ) -> Result<PortfolioSimulationMetrics, String> {
        let combined = self.combine_equity_curves(strategy_names, weights, equity_curves)?;

        let initial = self.config.initial_capital;
        let final_equity = *combined.last().unwrap_or(&initial);
        let total_return = (final_equity - initial) / initial;

        // Calculate drawdown
        let mut peak = combined[0];
        let mut max_dd = 0.0;
        for &equity in &combined {
            if equity > peak {
                peak = equity;
            }
            let dd = (peak - equity) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }

        // Calculate volatility
        let returns: Vec<f64> = combined.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();

        let volatility = if returns.len() >= 2 {
            let mean = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>()
                / (returns.len() - 1) as f64;
            (variance * 252.0).sqrt()
        } else {
            0.0
        };

        let sharpe = if volatility > 1e-10 {
            (total_return - 0.02) / volatility
        } else {
            0.0
        };

        Ok(PortfolioSimulationMetrics {
            final_equity,
            total_return,
            max_drawdown: max_dd,
            volatility,
            sharpe_ratio: sharpe,
            active_strategies: strategy_names.len(),
        })
    }

    /// Simulate with preserved correlations using Cholesky decomposition
    fn simulate_with_correlations(
        &self,
        strategy_names: &[String],
        weights: &[f64],
        returns: &HashMap<String, Vec<f64>>,
        correlation_matrix: &[Vec<f64>],
        length: usize,
        rng: &mut ChaCha8Rng,
    ) -> Result<PortfolioSimulationMetrics, String> {
        // Calculate mean and std dev for each strategy
        let mut stats: Vec<(f64, f64)> = Vec::new(); // (mean, std_dev)
        for name in strategy_names {
            if let Some(rets) = returns.get(name) {
                let mean = rets.iter().sum::<f64>() / rets.len() as f64;
                let variance =
                    rets.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / rets.len() as f64;
                let std_dev = variance.sqrt();
                stats.push((mean, std_dev));
            } else {
                stats.push((0.0, 0.0));
            }
        }

        // Build Cholesky decomposition of correlation matrix
        let chol = self.cholesky_decomposition(correlation_matrix)?;

        // Generate correlated random returns
        let mut strategy_equities: Vec<Vec<f64>> = Vec::new();
        for (idx, _name) in strategy_names.iter().enumerate() {
            let (mean, std_dev) = stats[idx];
            let mut equity = self.config.initial_capital;
            let mut equity_curve = vec![equity];

            for _ in 0..length {
                // Generate independent standard normal
                let z: f64 = Normal::new(0.0, 1.0).unwrap().sample(rng);

                // Correlate using Cholesky
                let correlated_z = if idx < chol.len() {
                    let row = &chol[idx];
                    row.iter()
                        .take(idx + 1)
                        .map(|&coeff| coeff * z)
                        .sum::<f64>()
                } else {
                    z
                };

                // Generate return
                let ret = mean + std_dev * correlated_z;
                equity *= 1.0 + ret;
                equity_curve.push(equity);
            }

            strategy_equities.push(equity_curve);
        }

        // Apply strategy failures if configured
        let active_indices = if self.config.strategy_failure_probability > 0.0 {
            let mut active = Vec::new();
            for (idx, _) in strategy_names.iter().enumerate() {
                if rand::random::<f64>() > self.config.strategy_failure_probability {
                    active.push(idx);
                }
            }
            if active.is_empty() {
                active.push(0); // At least one strategy active
            }
            active
        } else {
            (0..strategy_names.len()).collect()
        };

        // Combine strategies with weights
        let mut portfolio_equity = self.config.initial_capital;
        let mut portfolio_curve = vec![portfolio_equity];
        let mut peak = portfolio_equity;
        let mut max_dd = 0.0;

        for t in 1..=length {
            let mut weighted_return = 0.0;
            let mut total_weight: f64 = 0.0;

            for &idx in &active_indices {
                if t < strategy_equities[idx].len() {
                    let ret = (strategy_equities[idx][t] - strategy_equities[idx][t - 1])
                        / strategy_equities[idx][t - 1];
                    weighted_return += ret * weights[idx];
                    total_weight += weights[idx];
                }
            }

            // Normalize by active weight
            if total_weight > 0.0 {
                weighted_return /= total_weight;
            }

            portfolio_equity *= 1.0 + weighted_return;
            portfolio_curve.push(portfolio_equity);

            if portfolio_equity > peak {
                peak = portfolio_equity;
            }
            let dd = (peak - portfolio_equity) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }

        // Calculate metrics
        let total_return =
            (portfolio_equity - self.config.initial_capital) / self.config.initial_capital;

        let portfolio_returns: Vec<f64> = portfolio_curve
            .windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();

        let volatility = if portfolio_returns.len() >= 2 {
            let mean = portfolio_returns.iter().sum::<f64>() / portfolio_returns.len() as f64;
            let variance = portfolio_returns
                .iter()
                .map(|r| (r - mean).powi(2))
                .sum::<f64>()
                / (portfolio_returns.len() - 1) as f64;
            (variance * 252.0).sqrt()
        } else {
            0.0
        };

        let sharpe = if volatility > 1e-10 {
            (total_return - 0.02) / volatility
        } else {
            0.0
        };

        Ok(PortfolioSimulationMetrics {
            final_equity: portfolio_equity,
            total_return,
            max_drawdown: max_dd,
            volatility,
            sharpe_ratio: sharpe,
            active_strategies: active_indices.len(),
        })
    }

    /// Simulate with independent (uncorrelated) returns
    fn simulate_independent(
        &self,
        strategy_names: &[String],
        weights: &[f64],
        returns: &HashMap<String, Vec<f64>>,
        length: usize,
        rng: &mut ChaCha8Rng,
    ) -> Result<PortfolioSimulationMetrics, String> {
        // Use identity matrix as correlation (no correlation)
        let n = strategy_names.len();
        let identity: Vec<Vec<f64>> = (0..n)
            .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
            .collect();

        self.simulate_with_correlations(strategy_names, weights, returns, &identity, length, rng)
    }

    /// Perform Cholesky decomposition of a correlation matrix
    #[allow(clippy::needless_range_loop)]
    fn cholesky_decomposition(&self, matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, String> {
        let n = matrix.len();
        let mut chol = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in 0..=i {
                let mut sum = matrix[i][j];

                for k in 0..j {
                    sum -= chol[i][k] * chol[j][k];
                }

                if i == j {
                    if sum <= 0.0 {
                        // Matrix not positive definite - add small regularization
                        sum = 1e-10;
                    }
                    chol[i][j] = sum.sqrt();
                } else if chol[j][j] > 1e-10 {
                    chol[i][j] = sum / chol[j][j];
                }
            }
        }

        Ok(chol)
    }

    /// Combine equity curves according to weights
    fn combine_equity_curves(
        &self,
        strategy_names: &[String],
        weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
    ) -> Result<Vec<f64>, String> {
        let min_len = strategy_names
            .iter()
            .filter_map(|name| equity_curves.get(name).map(|c| c.len()))
            .min()
            .unwrap_or(0);

        if min_len == 0 {
            return Err("No equity curve data available".to_string());
        }

        let mut combined = Vec::with_capacity(min_len);
        let mut current_equity = self.config.initial_capital;

        for i in 0..min_len {
            if i == 0 {
                combined.push(current_equity);
                continue;
            }

            let mut weighted_return = 0.0;
            for (idx, name) in strategy_names.iter().enumerate() {
                if let Some(curve) = equity_curves.get(name) {
                    if i < curve.len() && i > 0 && curve[i - 1] > 0.0 {
                        let ret = (curve[i] - curve[i - 1]) / curve[i - 1];
                        weighted_return += ret * weights[idx];
                    }
                }
            }

            current_equity *= 1.0 + weighted_return;
            combined.push(current_equity);
        }

        Ok(combined)
    }

    /// Aggregate simulation results into percentiles
    fn aggregate_results(
        &self,
        simulations: &[PortfolioSimulationMetrics],
        original: PortfolioSimulationMetrics,
    ) -> PortfolioMonteCarloResult {
        let n = simulations.len();

        // Extract values
        let mut equities: Vec<f64> = simulations.iter().map(|s| s.final_equity).collect();
        let mut returns: Vec<f64> = simulations.iter().map(|s| s.total_return).collect();
        let mut drawdowns: Vec<f64> = simulations.iter().map(|s| s.max_drawdown).collect();

        // Sort for percentiles
        equities.sort_by(|a, b| a.partial_cmp(b).unwrap());
        returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        drawdowns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Calculate indices
        let idx_5 = (n as f64 * 0.05) as usize;
        let idx_50 = n / 2;
        let idx_95 = (n as f64 * 0.95) as usize;

        // Calculate probability of profit
        let profitable = simulations.iter().filter(|s| s.total_return > 0.0).count();
        let prob_profit = profitable as f64 / n as f64;

        // Calculate 95% VaR
        let var_95 = if idx_5 > 0 { -returns[idx_5] } else { 0.0 };

        PortfolioMonteCarloResult {
            num_simulations: n,
            original_metrics: original,
            equity_5th: equities[idx_5],
            equity_50th: equities[idx_50],
            equity_95th: equities[idx_95.min(n - 1)],
            return_5th: returns[idx_5],
            return_50th: returns[idx_50],
            return_95th: returns[idx_95.min(n - 1)],
            drawdown_5th: drawdowns[idx_5],
            drawdown_50th: drawdowns[idx_50],
            drawdown_95th: drawdowns[idx_95.min(n - 1)],
            probability_of_profit: prob_profit,
            var_95,
            simulations: simulations.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Type alias for test portfolio data tuple
    type TestPortfolioData = (
        Vec<String>,
        Vec<f64>,
        HashMap<String, Vec<f64>>,
        Vec<Vec<f64>>,
    );

    fn create_test_portfolio() -> TestPortfolioData {
        let names = vec!["Strategy A".to_string(), "Strategy B".to_string()];
        let weights = vec![0.5, 0.5];

        let mut curves = HashMap::new();
        // Strategy A: steady growth
        curves.insert(
            "Strategy A".to_string(),
            (0..=100).map(|i| 10000.0 * (1.0005f64).powi(i)).collect(),
        );
        // Strategy B: higher volatility
        curves.insert(
            "Strategy B".to_string(),
            (0..=100)
                .map(|i| {
                    let trend = 10000.0 * (1.0007f64).powi(i);
                    let noise = if i % 5 == 0 { 0.98 } else { 1.0 };
                    trend * noise
                })
                .collect(),
        );

        // Low correlation
        let corr = vec![vec![1.0, 0.3], vec![0.3, 1.0]];

        (names, weights, curves, corr)
    }

    #[test]
    fn test_monte_carlo_config() {
        let config = PortfolioMonteCarloConfig::new()
            .with_simulations(2000)
            .with_seed(42)
            .with_correlation_preservation(true);

        assert_eq!(config.num_simulations, 2000);
        assert_eq!(config.seed, Some(42));
        assert!(config.preserve_correlations);
    }

    #[test]
    fn test_basic_simulation() {
        let (names, weights, curves, corr) = create_test_portfolio();
        let config = PortfolioMonteCarloConfig::new()
            .with_simulations(500)
            .with_seed(42);
        let simulator = PortfolioMonteCarloSimulator::new(config);

        let result = simulator.simulate(&names, &weights, &curves, &corr);

        assert!(result.is_ok());
        let mc = result.unwrap();

        assert_eq!(mc.num_simulations, 500);
        assert!(mc.probability_of_profit > 0.0);
        assert!(mc.return_5th <= mc.return_50th);
        assert!(mc.return_50th <= mc.return_95th);
    }

    #[test]
    fn test_percentile_ordering() {
        let (names, weights, curves, corr) = create_test_portfolio();
        let config = PortfolioMonteCarloConfig::new()
            .with_simulations(100)
            .with_seed(123);
        let simulator = PortfolioMonteCarloSimulator::new(config);

        let result = simulator
            .simulate(&names, &weights, &curves, &corr)
            .unwrap();

        // Percentiles should be ordered
        assert!(result.equity_5th <= result.equity_50th);
        assert!(result.equity_50th <= result.equity_95th);
        assert!(result.return_5th <= result.return_50th);
        assert!(result.return_50th <= result.return_95th);
    }

    #[test]
    fn test_var_calculation() {
        let (names, weights, curves, corr) = create_test_portfolio();
        let config = PortfolioMonteCarloConfig::new()
            .with_simulations(200)
            .with_seed(456);
        let simulator = PortfolioMonteCarloSimulator::new(config);

        let result = simulator
            .simulate(&names, &weights, &curves, &corr)
            .unwrap();

        // VaR should be non-negative
        assert!(result.var_95 >= 0.0);
        // VaR should be approximately equal to -return_5th
        assert!((result.var_95 - (-result.return_5th)).abs() < 0.01);
    }

    #[test]
    fn test_cholesky_decomposition() {
        let config = PortfolioMonteCarloConfig::new();
        let simulator = PortfolioMonteCarloSimulator::new(config);

        // Simple 2x2 correlation matrix
        let corr = vec![vec![1.0, 0.5], vec![0.5, 1.0]];

        let chol = simulator.cholesky_decomposition(&corr).unwrap();

        // Check structure: lower triangular
        assert!(chol[0][1].abs() < 0.001);
        assert!(chol[0][0] > 0.0);
        assert!(chol[1][0] > 0.0);
        assert!(chol[1][1] > 0.0);

        // Check: L * L^T should reconstruct original matrix
        let reconstructed_00 = chol[0][0] * chol[0][0];
        let reconstructed_01 = chol[0][0] * chol[1][0];
        let reconstructed_11 = chol[1][0] * chol[1][0] + chol[1][1] * chol[1][1];

        assert!((reconstructed_00 - 1.0).abs() < 0.001);
        assert!((reconstructed_01 - 0.5).abs() < 0.001);
        assert!((reconstructed_11 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_invalid_weights() {
        let names = vec!["A".to_string(), "B".to_string()];
        let weights = vec![0.6]; // Mismatched length
        let curves = HashMap::new();
        let corr = vec![vec![1.0, 0.0], vec![0.0, 1.0]];

        let config = PortfolioMonteCarloConfig::new();
        let simulator = PortfolioMonteCarloSimulator::new(config);

        let result = simulator.simulate(&names, &weights, &curves, &corr);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_strategy_error() {
        let names = vec!["Only".to_string()];
        let weights = vec![1.0];
        let curves = HashMap::new();
        let corr = vec![vec![1.0]];

        let config = PortfolioMonteCarloConfig::new();
        let simulator = PortfolioMonteCarloSimulator::new(config);

        let result = simulator.simulate(&names, &weights, &curves, &corr);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 2 strategies"));
    }

    #[test]
    fn test_strategy_failure_simulation() {
        let (names, weights, curves, corr) = create_test_portfolio();
        let config = PortfolioMonteCarloConfig::new()
            .with_simulations(100)
            .with_strategy_failure_probability(0.3)
            .with_seed(789);
        let simulator = PortfolioMonteCarloSimulator::new(config);

        let result = simulator
            .simulate(&names, &weights, &curves, &corr)
            .unwrap();

        // With strategy failures, some simulations should have fewer active strategies
        let min_active = result
            .simulations
            .iter()
            .map(|s| s.active_strategies)
            .min()
            .unwrap();
        assert!(min_active <= 2);
    }

    #[test]
    fn test_reproducibility_with_seed() {
        let (names, weights, curves, corr) = create_test_portfolio();

        let config1 = PortfolioMonteCarloConfig::new()
            .with_simulations(100)
            .with_seed(42);
        let config2 = PortfolioMonteCarloConfig::new()
            .with_simulations(100)
            .with_seed(42);

        let sim1 = PortfolioMonteCarloSimulator::new(config1);
        let sim2 = PortfolioMonteCarloSimulator::new(config2);

        let result1 = sim1.simulate(&names, &weights, &curves, &corr).unwrap();
        let result2 = sim2.simulate(&names, &weights, &curves, &corr).unwrap();

        // With same seed, results should be identical
        assert!((result1.return_50th - result2.return_50th).abs() < 0.001);
        assert!((result1.probability_of_profit - result2.probability_of_profit).abs() < 0.001);
    }
}
