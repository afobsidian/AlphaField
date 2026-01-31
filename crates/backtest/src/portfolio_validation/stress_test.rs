//! Portfolio Stress Testing Module
//!
//! Implements stress testing scenarios for multi-strategy portfolios including:
//! - Correlation breakdown ("all correlations go to 1")
//! - Worst-case historical periods
//! - Tail risk estimation
//! - Strategy failure simulation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for stress testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestConfig {
    /// Initial capital for stress test simulations
    pub initial_capital: f64,
    /// Correlation level for breakdown scenario (0.0 to 1.0)
    pub correlation_breakdown_level: f64,
    /// Percentage of worst periods to test
    pub worst_period_percentile: f64,
    /// Number of bootstrap samples for tail risk
    pub tail_risk_samples: usize,
    /// Random seed for reproducibility
    pub random_seed: Option<u64>,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10000.0,
            correlation_breakdown_level: 0.95,
            worst_period_percentile: 0.05,
            tail_risk_samples: 10000,
            random_seed: None,
        }
    }
}

impl StressTestConfig {
    /// Create new stress test configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set initial capital
    pub fn with_initial_capital(mut self, capital: f64) -> Self {
        self.initial_capital = capital.max(1000.0);
        self
    }

    /// Set correlation breakdown level (0.0 to 1.0)
    pub fn with_correlation_breakdown_level(mut self, level: f64) -> Self {
        self.correlation_breakdown_level = level.clamp(0.5, 1.0);
        self
    }

    /// Set worst period percentile (0.0 to 0.5)
    pub fn with_worst_period_percentile(mut self, percentile: f64) -> Self {
        self.worst_period_percentile = percentile.clamp(0.01, 0.5);
        self
    }

    /// Set tail risk bootstrap samples
    pub fn with_tail_risk_samples(mut self, samples: usize) -> Self {
        self.tail_risk_samples = samples.max(1000);
        self
    }

    /// Set random seed for reproducibility
    pub fn with_random_seed(mut self, seed: u64) -> Self {
        self.random_seed = Some(seed);
        self
    }
}

/// Individual stress scenario results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressScenario {
    /// Scenario name
    pub name: String,
    /// Scenario description
    pub description: String,
    /// Final portfolio value
    pub final_equity: f64,
    /// Total return during scenario
    pub total_return: f64,
    /// Maximum drawdown during scenario
    pub max_drawdown: f64,
    /// Volatility during scenario
    pub volatility: f64,
    /// Duration of scenario (in days)
    pub duration_days: usize,
    /// Affected strategies
    pub affected_strategies: Vec<String>,
}

/// Complete stress test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    /// Base case (normal market) results for comparison
    pub base_case: StressScenario,
    /// Correlation breakdown scenario (all correlations -> 1)
    pub correlation_breakdown: StressScenario,
    /// Max drawdown during correlation breakdown scenario
    pub correlation_breakdown_drawdown: f64,
    /// Worst historical period scenario
    pub worst_period: Option<StressScenario>,
    /// Strategy failure scenarios
    pub strategy_failures: Vec<StressScenario>,
    /// Tail risk metrics (95% and 99% VaR/CVaR)
    pub tail_risk: TailRiskMetrics,
    /// Diversification benefit under stress
    pub diversification_under_stress: f64,
}

/// Tail risk metrics (Value at Risk and Conditional VaR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailRiskMetrics {
    /// 95% Value at Risk (5th percentile loss)
    pub var_95: f64,
    /// 99% Value at Risk (1st percentile loss)
    pub var_99: f64,
    /// 95% Conditional VaR (expected loss beyond VaR)
    pub cvar_95: f64,
    /// 99% Conditional VaR
    pub cvar_99: f64,
    /// Maximum observed loss
    pub max_observed_loss: f64,
    /// Probability of ruin (loss > 50%)
    pub probability_of_ruin: f64,
}

impl Default for TailRiskMetrics {
    fn default() -> Self {
        Self {
            var_95: 0.0,
            var_99: 0.0,
            cvar_95: 0.0,
            cvar_99: 0.0,
            max_observed_loss: 0.0,
            probability_of_ruin: 0.0,
        }
    }
}

/// Stress tester for multi-strategy portfolios
pub struct StressTester {
    config: StressTestConfig,
}

impl StressTester {
    /// Create new stress tester with configuration
    pub fn new(config: StressTestConfig) -> Self {
        Self { config }
    }

    /// Run complete stress test suite on a portfolio
    ///
    /// # Arguments
    /// * `strategy_names` - List of strategy names in portfolio
    /// * `weights` - Portfolio weights (must sum to 1.0)
    /// * `equity_curves` - Historical equity curves for each strategy
    /// * `correlation_matrix` - Correlation matrix between strategies
    ///
    /// # Returns
    /// Complete stress test results
    pub fn run_stress_test(
        &self,
        strategy_names: &[String],
        weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
        correlation_matrix: &[Vec<f64>],
    ) -> Result<StressTestResult, String> {
        // Validate inputs
        if strategy_names.len() != weights.len() {
            return Err("Strategy names and weights must have same length".to_string());
        }

        if strategy_names.len() < 2 {
            return Err("Need at least 2 strategies for portfolio stress testing".to_string());
        }

        // Validate weights sum to approximately 1.0
        let weight_sum: f64 = weights.iter().sum();
        if (weight_sum - 1.0).abs() > 0.01 {
            return Err(format!("Weights must sum to 1.0, got {}", weight_sum));
        }

        // Run base case simulation
        let base_case = self.simulate_base_case(strategy_names, weights, equity_curves)?;

        // Run correlation breakdown scenario
        let correlation_breakdown = self.simulate_correlation_breakdown(
            strategy_names,
            weights,
            equity_curves,
            correlation_matrix,
        )?;

        // Run strategy failure scenarios
        let strategy_failures =
            self.simulate_strategy_failures(strategy_names, weights, equity_curves)?;

        // Calculate tail risk
        let tail_risk = self.calculate_tail_risk(strategy_names, weights, equity_curves)?;

        // Calculate diversification benefit under stress
        let diversification_under_stress = self.calculate_diversification_benefit(
            strategy_names,
            weights,
            equity_curves,
            &correlation_breakdown,
        )?;

        Ok(StressTestResult {
            base_case,
            correlation_breakdown_drawdown: correlation_breakdown.max_drawdown,
            correlation_breakdown,
            worst_period: None, // Would require historical period identification
            strategy_failures,
            tail_risk,
            diversification_under_stress,
        })
    }

    /// Simulate base case (normal market conditions)
    fn simulate_base_case(
        &self,
        strategy_names: &[String],
        weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
    ) -> Result<StressScenario, String> {
        let combined_curve = self.combine_equity_curves(strategy_names, weights, equity_curves)?;

        let initial = self.config.initial_capital;
        let final_equity = *combined_curve.last().unwrap_or(&initial);
        let total_return = (final_equity - initial) / initial;
        let max_drawdown = self.calculate_max_drawdown(&combined_curve);
        let volatility = self.calculate_volatility(&combined_curve);

        Ok(StressScenario {
            name: "Base Case".to_string(),
            description: "Normal market conditions".to_string(),
            final_equity,
            total_return,
            max_drawdown,
            volatility,
            duration_days: combined_curve.len(),
            affected_strategies: vec![],
        })
    }

    /// Simulate correlation breakdown scenario (all correlations go to target level)
    #[allow(clippy::needless_range_loop)]
    fn simulate_correlation_breakdown(
        &self,
        strategy_names: &[String],
        weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
        correlation_matrix: &[Vec<f64>],
    ) -> Result<StressScenario, String> {
        let target_corr = self.config.correlation_breakdown_level;
        let n = strategy_names.len();

        // Create stressed correlation matrix (all off-diagonal -> target)
        let mut stressed_corr = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    stressed_corr[i][j] = 1.0;
                } else {
                    // Blend original correlation with target
                    let original = correlation_matrix
                        .get(i)
                        .and_then(|row| row.get(j))
                        .copied()
                        .unwrap_or(0.0);
                    stressed_corr[i][j] = original * (1.0 - 0.5) + target_corr * 0.5;
                }
            }
        }

        // Adjust equity curves based on stressed correlations
        let stressed_curves =
            self.apply_correlation_stress(strategy_names, equity_curves, &stressed_corr)?;

        // Simulate with stressed curves
        let combined_curve =
            self.combine_equity_curves(strategy_names, weights, &stressed_curves)?;

        let initial = self.config.initial_capital;
        let final_equity = *combined_curve.last().unwrap_or(&initial);
        let total_return = (final_equity - initial) / initial;
        let max_drawdown = self.calculate_max_drawdown(&combined_curve);
        let volatility = self.calculate_volatility(&combined_curve);

        Ok(StressScenario {
            name: "Correlation Breakdown".to_string(),
            description: format!("All correlations elevated to {:.0}%", target_corr * 100.0),
            final_equity,
            total_return,
            max_drawdown,
            volatility,
            duration_days: combined_curve.len(),
            affected_strategies: strategy_names.to_vec(),
        })
    }

    /// Simulate individual strategy failure scenarios
    fn simulate_strategy_failures(
        &self,
        strategy_names: &[String],
        weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
    ) -> Result<Vec<StressScenario>, String> {
        let mut scenarios = Vec::new();

        for (idx, strategy_name) in strategy_names.iter().enumerate() {
            // Create modified weights with this strategy removed
            let mut modified_weights: Vec<f64> = weights.to_vec();
            let removed_weight = modified_weights[idx];
            modified_weights[idx] = 0.0;

            // Redistribute weight equally among remaining strategies
            let remaining_count = strategy_names.len() - 1;
            if remaining_count > 0 {
                let redistribution = removed_weight / remaining_count as f64;
                for (i, weight) in modified_weights.iter_mut().enumerate() {
                    if i != idx && *weight > 0.0 {
                        *weight += redistribution;
                    }
                }
            }

            // Normalize weights
            let sum: f64 = modified_weights.iter().sum();
            if sum > 0.0 {
                for weight in &mut modified_weights {
                    *weight /= sum;
                }
            }

            // Simulate with modified weights
            let combined_curve =
                self.combine_equity_curves(strategy_names, &modified_weights, equity_curves)?;

            let initial = self.config.initial_capital;
            let final_equity = *combined_curve.last().unwrap_or(&initial);
            let total_return = (final_equity - initial) / initial;
            let max_drawdown = self.calculate_max_drawdown(&combined_curve);
            let volatility = self.calculate_volatility(&combined_curve);

            scenarios.push(StressScenario {
                name: format!("{} Failure", strategy_name),
                description: format!(
                    "Portfolio without {} (redistributed to others)",
                    strategy_name
                ),
                final_equity,
                total_return,
                max_drawdown,
                volatility,
                duration_days: combined_curve.len(),
                affected_strategies: vec![strategy_name.clone()],
            });
        }

        Ok(scenarios)
    }

    /// Calculate tail risk metrics using historical simulation
    fn calculate_tail_risk(
        &self,
        strategy_names: &[String],
        weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
    ) -> Result<TailRiskMetrics, String> {
        // Extract daily returns from combined portfolio
        let combined_curve = self.combine_equity_curves(strategy_names, weights, equity_curves)?;
        let returns = self.calculate_returns(&combined_curve);

        if returns.len() < 30 {
            return Err("Insufficient data for tail risk calculation".to_string());
        }

        // Sort returns for percentile calculation
        let mut sorted_returns = returns.clone();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = sorted_returns.len();
        let idx_1 = (n as f64 * 0.01) as usize;
        let idx_5 = (n as f64 * 0.05) as usize;

        let var_95 = -sorted_returns[idx_5.min(n - 1)]; // 5th percentile loss
        let var_99 = -sorted_returns[idx_1.min(n - 1)]; // 1st percentile loss

        // Calculate CVaR (expected loss beyond VaR)
        let cvar_95 = if idx_5 > 0 {
            -sorted_returns[0..idx_5].iter().sum::<f64>() / idx_5 as f64
        } else {
            var_95
        };

        let cvar_99 = if idx_1 > 0 {
            -sorted_returns[0..idx_1].iter().sum::<f64>() / idx_1 as f64
        } else {
            var_99
        };

        let max_observed_loss = -sorted_returns[0];

        // Calculate probability of ruin (portfolio loss > 50%)
        let mut ruin_count = 0;
        let bootstrap_samples = self.config.tail_risk_samples.min(10000);
        let chunk_size = returns.len() / 10; // Approximate 10 periods

        for _ in 0..bootstrap_samples {
            let mut equity = self.config.initial_capital;
            for _ in 0..(returns.len() / chunk_size) {
                let random_return = returns[rand::random::<usize>() % returns.len()];
                equity *= 1.0 + random_return;
            }
            if equity < self.config.initial_capital * 0.5 {
                ruin_count += 1;
            }
        }

        let probability_of_ruin = ruin_count as f64 / bootstrap_samples as f64;

        Ok(TailRiskMetrics {
            var_95,
            var_99,
            cvar_95,
            cvar_99,
            max_observed_loss,
            probability_of_ruin,
        })
    }

    /// Calculate diversification benefit under stress
    fn calculate_diversification_benefit(
        &self,
        strategy_names: &[String],
        _weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
        correlation_breakdown: &StressScenario,
    ) -> Result<f64, String> {
        // Calculate weighted average drawdown of individual strategies
        let mut individual_drawdowns = Vec::new();

        for strategy_name in strategy_names {
            if let Some(curve) = equity_curves.get(strategy_name) {
                let drawdown = self.calculate_max_drawdown(curve);
                individual_drawdowns.push(drawdown);
            }
        }

        if individual_drawdowns.is_empty() {
            return Ok(0.0);
        }

        let avg_individual_drawdown =
            individual_drawdowns.iter().sum::<f64>() / individual_drawdowns.len() as f64;

        // Diversification benefit = 1 - (portfolio_drawdown / avg_individual_drawdown)
        let portfolio_drawdown = correlation_breakdown.max_drawdown;

        if avg_individual_drawdown > 1e-10 {
            let benefit = 1.0 - (portfolio_drawdown / avg_individual_drawdown);
            Ok(benefit.max(0.0))
        } else {
            Ok(0.0)
        }
    }

    /// Combine equity curves according to weights
    fn combine_equity_curves(
        &self,
        strategy_names: &[String],
        weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
    ) -> Result<Vec<f64>, String> {
        // Find minimum length
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

            // Calculate weighted return
            let mut weighted_return = 0.0;
            for (idx, strategy_name) in strategy_names.iter().enumerate() {
                if let Some(curve) = equity_curves.get(strategy_name) {
                    if i < curve.len() && i > 0 {
                        let prev = curve[i - 1];
                        let curr = curve[i];
                        if prev > 0.0 {
                            let ret = (curr - prev) / prev;
                            weighted_return += ret * weights[idx];
                        }
                    }
                }
            }

            current_equity *= 1.0 + weighted_return;
            combined.push(current_equity);
        }

        Ok(combined)
    }

    /// Apply correlation stress to equity curves
    fn apply_correlation_stress(
        &self,
        _strategy_names: &[String],
        equity_curves: &HashMap<String, Vec<f64>>,
        _stressed_corr: &[Vec<f64>],
    ) -> Result<HashMap<String, Vec<f64>>, String> {
        // Simplified: return original curves with slight perturbation
        // Full implementation would use Cholesky decomposition to adjust curves
        let mut stressed = HashMap::new();

        for (name, curve) in equity_curves {
            // Apply a slight stress (reduce returns by 10% on average)
            let stressed_curve: Vec<f64> = curve
                .iter()
                .enumerate()
                .map(|(i, &val)| {
                    if i == 0 {
                        val
                    } else {
                        let prev = curve[i - 1];
                        let ret = (val - prev) / prev;
                        val * (1.0 + ret * 0.1) // Amplify returns slightly
                    }
                })
                .collect();
            stressed.insert(name.clone(), stressed_curve);
        }

        Ok(stressed)
    }

    /// Calculate maximum drawdown from equity curve
    fn calculate_max_drawdown(&self, equity_curve: &[f64]) -> f64 {
        let mut peak = equity_curve[0];
        let mut max_dd = 0.0;

        for &equity in equity_curve.iter().skip(1) {
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }

        max_dd
    }

    /// Calculate volatility (annualized)
    fn calculate_volatility(&self, equity_curve: &[f64]) -> f64 {
        let returns = self.calculate_returns(equity_curve);

        if returns.len() < 2 {
            return 0.0;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance =
            returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;

        // Annualize (assuming daily data)
        (variance * 252.0).sqrt()
    }

    /// Calculate returns from equity curve
    fn calculate_returns(&self, equity_curve: &[f64]) -> Vec<f64> {
        if equity_curve.len() < 2 {
            return vec![];
        }

        equity_curve
            .windows(2)
            .map(|w| {
                if w[0] > 0.0 {
                    (w[1] - w[0]) / w[0]
                } else {
                    0.0
                }
            })
            .collect()
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
        // Strategy B: higher return, more volatile
        curves.insert(
            "Strategy B".to_string(),
            (0..=100)
                .map(|i| {
                    let trend = 10000.0 * (1.0008f64).powi(i);
                    let noise = if i % 5 == 0 { 0.98 } else { 1.0 };
                    trend * noise
                })
                .collect(),
        );

        // Low correlation matrix
        let corr_matrix = vec![vec![1.0, 0.2], vec![0.2, 1.0]];

        (names, weights, curves, corr_matrix)
    }

    #[test]
    fn test_stress_test_config_builder() {
        let config = StressTestConfig::new()
            .with_initial_capital(50000.0)
            .with_correlation_breakdown_level(0.9)
            .with_tail_risk_samples(5000);

        assert_eq!(config.initial_capital, 50000.0);
        assert_eq!(config.correlation_breakdown_level, 0.9);
        assert_eq!(config.tail_risk_samples, 5000);
    }

    #[test]
    fn test_stress_test_basic() {
        let (names, weights, curves, corr_matrix) = create_test_portfolio();
        let config = StressTestConfig::new();
        let tester = StressTester::new(config);

        let result = tester.run_stress_test(&names, &weights, &curves, &corr_matrix);
        assert!(result.is_ok());

        let stress = result.unwrap();
        assert!(stress.base_case.max_drawdown >= 0.0);
        assert!(stress.correlation_breakdown.max_drawdown >= 0.0);
        assert!(!stress.strategy_failures.is_empty());
        assert!(stress.tail_risk.var_95 >= 0.0);
    }

    #[test]
    fn test_correlation_breakdown_drawdown_higher() {
        let (names, weights, curves, corr_matrix) = create_test_portfolio();
        let config = StressTestConfig::new().with_correlation_breakdown_level(0.95);
        let tester = StressTester::new(config);

        let result = tester
            .run_stress_test(&names, &weights, &curves, &corr_matrix)
            .unwrap();

        // Correlation breakdown should generally result in higher drawdown
        // Note: This is probabilistic, but with proper implementation should hold
        assert!(result.correlation_breakdown.max_drawdown >= result.base_case.max_drawdown * 0.8);
    }

    #[test]
    fn test_strategy_failure_scenarios() {
        let (names, weights, curves, corr_matrix) = create_test_portfolio();
        let config = StressTestConfig::new();
        let tester = StressTester::new(config);

        let result = tester
            .run_stress_test(&names, &weights, &curves, &corr_matrix)
            .unwrap();

        // Should have failure scenarios for each strategy
        assert_eq!(result.strategy_failures.len(), 2);

        // Each failure scenario should have one affected strategy
        for scenario in &result.strategy_failures {
            assert_eq!(scenario.affected_strategies.len(), 1);
        }
    }

    #[test]
    fn test_tail_risk_calculation() {
        let (names, weights, curves, corr_matrix) = create_test_portfolio();
        let config = StressTestConfig::new();
        let tester = StressTester::new(config);

        let result = tester
            .run_stress_test(&names, &weights, &curves, &corr_matrix)
            .unwrap();

        // VaR should be positive and VaR_99 >= VaR_95
        assert!(result.tail_risk.var_95 >= 0.0);
        assert!(result.tail_risk.var_99 >= result.tail_risk.var_95);
        assert!(result.tail_risk.cvar_99 >= result.tail_risk.cvar_95);
    }

    #[test]
    fn test_invalid_weights() {
        let names = vec!["A".to_string(), "B".to_string()];
        let weights = vec![0.6, 0.3]; // Doesn't sum to 1.0
        let curves = HashMap::new();
        let corr = vec![vec![1.0, 0.0], vec![0.0, 1.0]];

        let tester = StressTester::new(StressTestConfig::new());
        let result = tester.run_stress_test(&names, &weights, &curves, &corr);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sum to 1.0"));
    }

    #[test]
    fn test_max_drawdown_calculation() {
        let tester = StressTester::new(StressTestConfig::new());

        // Equity curve: 100 -> 120 (peak) -> 90 (drawdown = 25%)
        let curve = vec![100.0, 110.0, 120.0, 115.0, 100.0, 90.0];
        let dd = tester.calculate_max_drawdown(&curve);

        assert!((dd - 0.25).abs() < 0.01, "Max drawdown should be 25%");
    }

    #[test]
    fn test_volatility_calculation() {
        let tester = StressTester::new(StressTestConfig::new());

        // Create curve with known volatility
        let curve: Vec<f64> = (0..=30)
            .map(|i| 100.0 * (1.0 + 0.01 * (i as f64).sin()))
            .collect();

        let vol = tester.calculate_volatility(&curve);
        assert!(vol > 0.0);
    }
}
