//! Portfolio Walk-Forward Analysis Module
//!
//! Implements rolling window analysis for multi-strategy portfolios:
//! - Re-optimize portfolio weights at each window step
//! - Track portfolio metrics consistency over time
//! - Detect performance degradation or regime changes
//! - Measure out-of-sample performance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::portfolio_optimization::{
    OptimizationConfig, OptimizationObjective, PortfolioOptimizer,
};

/// Configuration for portfolio walk-forward analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioWalkForwardConfig {
    /// Window size (number of data points in each period)
    pub window_size: usize,
    /// Step size (how much to advance each window)
    pub step_size: usize,
    /// Initial capital for each window
    pub initial_capital: f64,
    /// Optimization objective
    pub optimization_objective: OptimizationObjective,
    /// Risk-free rate for Sharpe calculation
    pub risk_free_rate: f64,
    /// Minimum data points required for optimization
    pub min_optimization_points: usize,
}

impl Default for PortfolioWalkForwardConfig {
    fn default() -> Self {
        Self {
            window_size: 60, // 60 periods (e.g., days)
            step_size: 20,   // Advance 20 periods each step
            initial_capital: 10000.0,
            optimization_objective: OptimizationObjective::MaximizeSharpe,
            risk_free_rate: 0.02,
            min_optimization_points: 30,
        }
    }
}

impl PortfolioWalkForwardConfig {
    /// Create new walk-forward configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set window size
    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size.max(20);
        self
    }

    /// Set step size
    pub fn with_step_size(mut self, size: usize) -> Self {
        self.step_size = size.max(5);
        self
    }

    /// Set initial capital
    pub fn with_initial_capital(mut self, capital: f64) -> Self {
        self.initial_capital = capital.max(1000.0);
        self
    }

    /// Set optimization objective
    pub fn with_objective(mut self, objective: OptimizationObjective) -> Self {
        self.optimization_objective = objective;
        self
    }

    /// Set risk-free rate
    pub fn with_risk_free_rate(mut self, rate: f64) -> Self {
        self.risk_free_rate = rate.clamp(0.0, 0.5);
        self
    }
}

/// Results from a single walk-forward window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardWindowResult {
    /// Window index (0, 1, 2, ...)
    pub window_index: usize,
    /// Start index in the data
    pub start_index: usize,
    /// End index in the data
    pub end_index: usize,
    /// Optimal weights determined in training period
    pub optimized_weights: HashMap<String, f64>,
    /// In-sample (training) performance metrics
    pub in_sample_return: f64,
    pub in_sample_volatility: f64,
    pub in_sample_sharpe: f64,
    pub in_sample_drawdown: f64,
    /// Out-of-sample (test) performance metrics
    pub out_of_sample_return: f64,
    pub out_of_sample_volatility: f64,
    pub out_of_sample_sharpe: f64,
    pub out_of_sample_drawdown: f64,
    /// Weight turnover from previous window
    pub weight_turnover: f64,
    /// Optimization convergence status
    pub optimization_converged: bool,
}

/// Complete walk-forward analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioWalkForwardResult {
    /// Individual window results
    pub window_results: Vec<WalkForwardWindowResult>,
    /// Average in-sample Sharpe ratio
    pub average_in_sample_sharpe: f64,
    /// Average out-of-sample Sharpe ratio
    pub average_out_of_sample_sharpe: f64,
    /// Average in-sample return
    pub average_in_sample_return: f64,
    /// Average out-of-sample return
    pub average_out_of_sample_return: f64,
    /// Average in-sample drawdown
    pub average_in_sample_drawdown: f64,
    /// Average out-of-sample drawdown
    pub average_out_of_sample_drawdown: f64,
    /// Average drawdown (combined in and out of sample)
    pub average_drawdown: f64,
    /// Consistency score (0.0 to 1.0, higher is better)
    pub consistency_score: f64,
    /// Average weight turnover (lower is better)
    pub average_weight_turnover: f64,
    /// Number of windows where optimization converged
    pub converged_windows: usize,
    /// Total number of windows
    pub total_windows: usize,
}

/// Portfolio walk-forward analyzer
pub struct PortfolioWalkForwardAnalyzer {
    config: PortfolioWalkForwardConfig,
}

impl PortfolioWalkForwardAnalyzer {
    /// Create new analyzer with configuration
    pub fn new(config: PortfolioWalkForwardConfig) -> Self {
        Self { config }
    }

    /// Run walk-forward analysis on a portfolio
    ///
    /// # Arguments
    /// * `strategy_names` - List of strategies in portfolio
    /// * `equity_curves` - Historical equity curves for each strategy
    /// * `optimizer` - Portfolio optimizer to use
    ///
    /// # Returns
    /// Complete walk-forward analysis results
    pub fn analyze<O: PortfolioOptimizer>(
        &self,
        strategy_names: &[String],
        equity_curves: &HashMap<String, Vec<f64>>,
        optimizer: &O,
    ) -> Result<PortfolioWalkForwardResult, String> {
        // Validate data availability
        if strategy_names.len() < 2 {
            return Err("Need at least 2 strategies for walk-forward analysis".to_string());
        }

        // Check all strategies have data
        for name in strategy_names {
            if !equity_curves.contains_key(name) {
                return Err(format!("Missing equity curve for strategy: {}", name));
            }
        }

        // Find minimum curve length
        let min_len = strategy_names
            .iter()
            .filter_map(|name| equity_curves.get(name).map(|c| c.len()))
            .min()
            .unwrap_or(0);

        if min_len < self.config.window_size * 2 {
            return Err(format!(
                "Insufficient data: need at least {} points, have {}",
                self.config.window_size * 2,
                min_len
            ));
        }

        let mut window_results = Vec::new();
        let mut previous_weights: Option<HashMap<String, f64>> = None;

        // Calculate number of windows
        let num_windows = (min_len - self.config.window_size) / self.config.step_size;

        for window_idx in 0..=num_windows {
            let train_start = window_idx * self.config.step_size;
            let train_end = train_start + self.config.window_size;
            let test_end = (train_end + self.config.step_size).min(min_len);

            if train_end >= min_len || test_end > min_len {
                break;
            }

            // Extract training period equity curves
            let train_curves: HashMap<String, Vec<f64>> = strategy_names
                .iter()
                .filter_map(|name| {
                    equity_curves.get(name).map(|curve| {
                        let train_data = curve[train_start..train_end].to_vec();
                        (name.clone(), train_data)
                    })
                })
                .collect();

            // Optimize weights on training data
            let opt_config = OptimizationConfig::new()
                .with_objective(self.config.optimization_objective)
                .with_risk_free_rate(self.config.risk_free_rate)
                .with_initial_capital(self.config.initial_capital);

            let opt_result = optimizer.optimize(strategy_names, &train_curves, &opt_config);

            let (optimized_weights, converged) = match opt_result {
                Ok(result) => (result.allocations, result.converged),
                Err(_) => {
                    // Use equal weights if optimization fails
                    let equal_weight = 1.0 / strategy_names.len() as f64;
                    let mut fallback_weights = HashMap::new();
                    for name in strategy_names {
                        fallback_weights.insert(name.clone(), equal_weight);
                    }
                    (fallback_weights, false)
                }
            };

            // Calculate in-sample performance
            let in_sample_metrics = self.calculate_window_metrics(
                strategy_names,
                &optimized_weights,
                equity_curves,
                train_start,
                train_end,
            )?;

            // Calculate out-of-sample performance
            let out_sample_metrics = self.calculate_window_metrics(
                strategy_names,
                &optimized_weights,
                equity_curves,
                train_end,
                test_end,
            )?;

            // Calculate weight turnover
            let weight_turnover = if let Some(ref prev) = previous_weights {
                self.calculate_weight_turnover(&optimized_weights, prev)
            } else {
                0.0
            };

            window_results.push(WalkForwardWindowResult {
                window_index: window_idx,
                start_index: train_start,
                end_index: test_end,
                optimized_weights: optimized_weights.clone(),
                in_sample_return: in_sample_metrics.0,
                in_sample_volatility: in_sample_metrics.1,
                in_sample_sharpe: in_sample_metrics.2,
                in_sample_drawdown: in_sample_metrics.3,
                out_of_sample_return: out_sample_metrics.0,
                out_of_sample_volatility: out_sample_metrics.1,
                out_of_sample_sharpe: out_sample_metrics.2,
                out_of_sample_drawdown: out_sample_metrics.3,
                weight_turnover,
                optimization_converged: converged,
            });

            previous_weights = Some(optimized_weights);
        }

        if window_results.is_empty() {
            return Err("No valid windows could be created".to_string());
        }

        // Aggregate results
        let aggregated = self.aggregate_results(&window_results);

        Ok(aggregated)
    }

    /// Calculate metrics for a specific window
    fn calculate_window_metrics(
        &self,
        strategy_names: &[String],
        weights: &HashMap<String, f64>,
        equity_curves: &HashMap<String, Vec<f64>>,
        start: usize,
        end: usize,
    ) -> Result<(f64, f64, f64, f64), String> {
        // Combine curves for this window
        let mut portfolio_equity = vec![self.config.initial_capital];
        let mut current_equity = self.config.initial_capital;

        for i in (start + 1)..end {
            let mut weighted_return = 0.0;

            for name in strategy_names {
                if let Some(curve) = equity_curves.get(name) {
                    if i < curve.len() && i > 0 && curve[i - 1] > 0.0 {
                        let ret = (curve[i] - curve[i - 1]) / curve[i - 1];
                        let weight = weights.get(name).copied().unwrap_or(0.0);
                        weighted_return += ret * weight;
                    }
                }
            }

            current_equity *= 1.0 + weighted_return;
            portfolio_equity.push(current_equity);
        }

        if portfolio_equity.len() < 2 {
            return Ok((0.0, 0.0, 0.0, 0.0));
        }

        // Calculate return
        let initial = portfolio_equity[0];
        let final_val = portfolio_equity[portfolio_equity.len() - 1];
        let total_return = (final_val - initial) / initial;

        // Calculate volatility
        let returns: Vec<f64> = portfolio_equity
            .windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();

        let volatility = if returns.len() >= 2 {
            let mean = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>()
                / (returns.len() - 1) as f64;
            (variance * 252.0).sqrt() // Annualize assuming daily data
        } else {
            0.0
        };

        // Calculate Sharpe ratio
        let sharpe = if volatility > 1e-10 {
            (total_return - self.config.risk_free_rate) / volatility
        } else {
            0.0
        };

        // Calculate max drawdown
        let mut peak = portfolio_equity[0];
        let mut max_dd = 0.0;
        for &equity in &portfolio_equity {
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }

        Ok((total_return, volatility, sharpe, max_dd))
    }

    /// Calculate weight turnover between two weight configurations
    fn calculate_weight_turnover(
        &self,
        current: &HashMap<String, f64>,
        previous: &HashMap<String, f64>,
    ) -> f64 {
        // Calculate sum of absolute differences divided by 2
        let all_strategies: std::collections::HashSet<String> =
            current.keys().chain(previous.keys()).cloned().collect();

        let mut total_turnover = 0.0;
        for strategy in all_strategies {
            let current_weight = current.get(&strategy).copied().unwrap_or(0.0);
            let previous_weight = previous.get(&strategy).copied().unwrap_or(0.0);
            total_turnover += (current_weight - previous_weight).abs();
        }

        total_turnover / 2.0
    }

    /// Aggregate results across all windows
    fn aggregate_results(
        &self,
        window_results: &[WalkForwardWindowResult],
    ) -> PortfolioWalkForwardResult {
        let n = window_results.len() as f64;

        // Calculate averages
        let avg_in_sharpe: f64 = window_results
            .iter()
            .map(|w| w.in_sample_sharpe)
            .sum::<f64>()
            / n;
        let avg_out_sharpe: f64 = window_results
            .iter()
            .map(|w| w.out_of_sample_sharpe)
            .sum::<f64>()
            / n;
        let avg_in_return: f64 = window_results
            .iter()
            .map(|w| w.in_sample_return)
            .sum::<f64>()
            / n;
        let avg_out_return: f64 = window_results
            .iter()
            .map(|w| w.out_of_sample_return)
            .sum::<f64>()
            / n;
        let avg_in_dd: f64 = window_results
            .iter()
            .map(|w| w.in_sample_drawdown)
            .sum::<f64>()
            / n;
        let avg_out_dd: f64 = window_results
            .iter()
            .map(|w| w.out_of_sample_drawdown)
            .sum::<f64>()
            / n;
        let avg_dd = (avg_in_dd + avg_out_dd) / 2.0;
        let avg_turnover: f64 = window_results
            .iter()
            .map(|w| w.weight_turnover)
            .sum::<f64>()
            / n;

        // Calculate consistency score
        // Based on how well out-of-sample performance matches in-sample
        let sharpe_differences: Vec<f64> = window_results
            .iter()
            .map(|w| (w.in_sample_sharpe - w.out_of_sample_sharpe).abs())
            .collect();
        let avg_sharpe_diff = sharpe_differences.iter().sum::<f64>() / n;
        let consistency_score = (1.0 - avg_sharpe_diff).max(0.0);

        // Count converged windows
        let converged = window_results
            .iter()
            .filter(|w| w.optimization_converged)
            .count();

        PortfolioWalkForwardResult {
            window_results: window_results.to_vec(),
            average_in_sample_sharpe: avg_in_sharpe,
            average_out_of_sample_sharpe: avg_out_sharpe,
            average_in_sample_return: avg_in_return,
            average_out_of_sample_return: avg_out_return,
            average_in_sample_drawdown: avg_in_dd,
            average_out_of_sample_drawdown: avg_out_dd,
            average_drawdown: avg_dd,
            consistency_score,
            average_weight_turnover: avg_turnover,
            converged_windows: converged,
            total_windows: window_results.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portfolio_optimization::algorithms::mean_variance::MeanVarianceOptimizer;

    fn create_test_data() -> (Vec<String>, HashMap<String, Vec<f64>>) {
        let names = vec!["Strategy A".to_string(), "Strategy B".to_string()];

        let mut curves = HashMap::new();
        // Generate longer curves for walk-forward (need at least 120 points for 2 windows of 60)
        curves.insert(
            "Strategy A".to_string(),
            (0..=150).map(|i| 10000.0 * (1.0005f64).powi(i)).collect(),
        );
        curves.insert(
            "Strategy B".to_string(),
            (0..=150)
                .map(|i| {
                    let trend = 10000.0 * (1.0007f64).powi(i);
                    let noise = if i % 5 == 0 { 0.98 } else { 1.0 };
                    trend * noise
                })
                .collect(),
        );

        (names, curves)
    }

    #[test]
    fn test_walk_forward_config() {
        let config = PortfolioWalkForwardConfig::new()
            .with_window_size(30)
            .with_step_size(10)
            .with_objective(OptimizationObjective::MinimizeVolatility);

        assert_eq!(config.window_size, 30);
        assert_eq!(config.step_size, 10);
        assert_eq!(
            config.optimization_objective,
            OptimizationObjective::MinimizeVolatility
        );
    }

    #[test]
    fn test_walk_forward_analysis() {
        let (names, curves) = create_test_data();
        let config = PortfolioWalkForwardConfig::new()
            .with_window_size(40)
            .with_step_size(20);
        let analyzer = PortfolioWalkForwardAnalyzer::new(config);
        let optimizer = MeanVarianceOptimizer::new();

        let result = analyzer.analyze(&names, &curves, &optimizer);

        // Should succeed with enough data
        assert!(
            result.is_ok(),
            "Walk-forward should succeed: {:?}",
            result.err()
        );

        let wf = result.unwrap();
        assert!(!wf.window_results.is_empty());
        assert!(wf.total_windows > 0);
    }

    #[test]
    fn test_insufficient_data_error() {
        let names = vec!["A".to_string(), "B".to_string()];
        let mut curves = HashMap::new();
        curves.insert("A".to_string(), vec![10000.0; 50]); // Too short
        curves.insert("B".to_string(), vec![10000.0; 50]);

        let config = PortfolioWalkForwardConfig::new().with_window_size(60); // Needs 120 minimum
        let analyzer = PortfolioWalkForwardAnalyzer::new(config);
        let optimizer = MeanVarianceOptimizer::new();

        let result = analyzer.analyze(&names, &curves, &optimizer);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient data"));
    }

    #[test]
    fn test_single_strategy_error() {
        let names = vec!["Only".to_string()];
        let curves = HashMap::new();
        let config = PortfolioWalkForwardConfig::new();
        let analyzer = PortfolioWalkForwardAnalyzer::new(config);
        let optimizer = MeanVarianceOptimizer::new();

        let result = analyzer.analyze(&names, &curves, &optimizer);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 2 strategies"));
    }

    #[test]
    fn test_aggregation_calculations() {
        let windows = vec![
            WalkForwardWindowResult {
                window_index: 0,
                start_index: 0,
                end_index: 60,
                optimized_weights: HashMap::new(),
                in_sample_return: 0.05,
                in_sample_volatility: 0.10,
                in_sample_sharpe: 0.50,
                in_sample_drawdown: 0.05,
                out_of_sample_return: 0.04,
                out_of_sample_volatility: 0.11,
                out_of_sample_sharpe: 0.36,
                out_of_sample_drawdown: 0.06,
                weight_turnover: 0.0,
                optimization_converged: true,
            },
            WalkForwardWindowResult {
                window_index: 1,
                start_index: 20,
                end_index: 80,
                optimized_weights: HashMap::new(),
                in_sample_return: 0.06,
                in_sample_volatility: 0.09,
                in_sample_sharpe: 0.67,
                in_sample_drawdown: 0.04,
                out_of_sample_return: 0.05,
                out_of_sample_volatility: 0.10,
                out_of_sample_sharpe: 0.50,
                out_of_sample_drawdown: 0.05,
                weight_turnover: 0.15,
                optimization_converged: true,
            },
        ];

        let config = PortfolioWalkForwardConfig::new();
        let analyzer = PortfolioWalkForwardAnalyzer::new(config);
        let result = analyzer.aggregate_results(&windows);

        // Averages should be calculated correctly
        assert_eq!(result.average_in_sample_return, 0.055); // (0.05 + 0.06) / 2
        assert_eq!(result.average_out_of_sample_return, 0.045); // (0.04 + 0.05) / 2
        assert_eq!(result.average_weight_turnover, 0.075); // (0.0 + 0.15) / 2
        assert_eq!(result.total_windows, 2);
        assert_eq!(result.converged_windows, 2);

        // Consistency score should be calculated
        // |0.50 - 0.36| + |0.67 - 0.50| = 0.14 + 0.17 = 0.31 / 2 = 0.155
        // Score = 1 - 0.155 = 0.845
        assert!((result.consistency_score - 0.845).abs() < 0.01);
    }

    #[test]
    fn test_weight_turnover_calculation() {
        let config = PortfolioWalkForwardConfig::new();
        let analyzer = PortfolioWalkForwardAnalyzer::new(config);

        let prev = {
            let mut m = HashMap::new();
            m.insert("A".to_string(), 0.5);
            m.insert("B".to_string(), 0.5);
            m
        };

        let curr = {
            let mut m = HashMap::new();
            m.insert("A".to_string(), 0.6);
            m.insert("B".to_string(), 0.4);
            m
        };

        let turnover = analyzer.calculate_weight_turnover(&curr, &prev);
        // |0.6 - 0.5| + |0.4 - 0.5| = 0.1 + 0.1 = 0.2 / 2 = 0.1
        assert!((turnover - 0.10).abs() < 0.001);
    }

    #[test]
    fn test_window_metrics_calculation() {
        let names = vec!["A".to_string(), "B".to_string()];
        let mut curves = HashMap::new();
        curves.insert("A".to_string(), vec![10000.0, 10100.0, 10200.0, 10300.0]);
        curves.insert("B".to_string(), vec![10000.0, 10050.0, 10100.0, 10150.0]);

        let weights = {
            let mut m = HashMap::new();
            m.insert("A".to_string(), 0.5);
            m.insert("B".to_string(), 0.5);
            m
        };

        let config = PortfolioWalkForwardConfig::new();
        let analyzer = PortfolioWalkForwardAnalyzer::new(config);

        let metrics = analyzer
            .calculate_window_metrics(&names, &weights, &curves, 0, 4)
            .unwrap();

        // Combined return should be positive
        assert!(metrics.0 > 0.0);
        assert!(metrics.1 >= 0.0); // Volatility
        assert!(metrics.2 >= 0.0); // Sharpe
        assert!(metrics.3 >= 0.0); // Drawdown
    }
}
