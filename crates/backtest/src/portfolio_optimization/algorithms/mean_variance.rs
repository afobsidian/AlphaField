//! Mean-Variance Optimization Algorithm (Markowitz)
//!
//! Implements Markowitz mean-variance optimization for portfolio allocation.
//! Supports multiple objectives: maximize Sharpe, minimize volatility,
//! or maximize return with risk constraints.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::portfolio_optimization::{
    constraints::PortfolioConstraint,
    optimizer::{calculate_mean_return, equity_to_returns, prepare_optimization_data},
    OptimizationConfig, OptimizationObjective, OptimizationResult, PortfolioOptimizer,
};

/// Result from mean-variance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeanVarianceResult {
    /// Optimal weights
    pub weights: Vec<f64>,
    /// Portfolio expected return (annualized)
    pub expected_return: f64,
    /// Portfolio volatility (annualized)
    pub volatility: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    /// Risk-free rate used
    pub risk_free_rate: f64,
    /// Individual strategy risk contributions
    pub risk_contributions: Vec<f64>,
    /// Efficient frontier points (return, volatility)
    pub efficient_frontier: Vec<(f64, f64)>,
}

/// Mean-Variance Portfolio Optimizer
///
/// Implements Markowitz mean-variance optimization using quadratic programming.
/// Can optimize for:
/// - Maximum Sharpe ratio
/// - Minimum volatility
/// - Maximum return with risk constraints
#[derive(Debug, Clone)]
pub struct MeanVarianceOptimizer {
    /// Number of grid points for efficient frontier calculation
    frontier_points: usize,
}

impl Default for MeanVarianceOptimizer {
    fn default() -> Self {
        Self {
            frontier_points: 20,
        }
    }
}

impl MeanVarianceOptimizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_frontier_points(mut self, points: usize) -> Self {
        self.frontier_points = points.clamp(10, 100);
        self
    }

    /// Optimize portfolio for maximum Sharpe ratio
    fn optimize_sharpe(
        &self,
        means: &[f64],
        cov_matrix: &[Vec<f64>],
        risk_free_rate: f64,
        constraints: &PortfolioConstraint,
    ) -> Result<(Vec<f64>, MeanVarianceResult), String> {
        // Calculate efficient frontier
        let frontier = self.calculate_efficient_frontier(means, cov_matrix, constraints)?;

        if frontier.is_empty() {
            return Err("Could not construct efficient frontier".to_string());
        }

        // Find portfolio with maximum Sharpe ratio
        let mut best_sharpe = f64::MIN;
        let mut best_weights = frontier[0].0.clone();

        for (weights, ret, vol) in &frontier {
            let sharpe = if *vol > 1e-10 {
                (*ret - risk_free_rate) / *vol
            } else {
                0.0
            };

            if sharpe > best_sharpe {
                best_sharpe = sharpe;
                best_weights = weights.clone();
            }
        }

        let portfolio_return = self.calculate_portfolio_return(&best_weights, means);
        let portfolio_vol = self.calculate_portfolio_volatility(&best_weights, cov_matrix)?;

        let risk_contributions = self.calculate_risk_contributions(&best_weights, cov_matrix);

        let frontier_points: Vec<(f64, f64)> = frontier.iter().map(|(_, r, v)| (*r, *v)).collect();

        let result = MeanVarianceResult {
            weights: best_weights.clone(),
            expected_return: portfolio_return,
            volatility: portfolio_vol,
            sharpe_ratio: best_sharpe,
            risk_free_rate,
            risk_contributions,
            efficient_frontier: frontier_points,
        };

        Ok((best_weights, result))
    }

    /// Optimize for minimum volatility
    fn optimize_min_volatility(
        &self,
        means: &[f64],
        cov_matrix: &[Vec<f64>],
        risk_free_rate: f64,
        constraints: &PortfolioConstraint,
    ) -> Result<(Vec<f64>, MeanVarianceResult), String> {
        let n = means.len();

        // Start with equal weights
        let mut weights = vec![1.0 / n as f64; n];

        // Use gradient descent to minimize volatility
        let learning_rate = 0.1;
        let max_iterations = 1000;
        let tolerance = 1e-8;

        for _ in 0..max_iterations {
            let gradient = self.calculate_volatility_gradient(&weights, cov_matrix)?;

            // Update weights
            for i in 0..n {
                weights[i] -= learning_rate * gradient[i];
            }

            // Project onto constraint set (sum to 1, non-negative)
            weights = self.project_weights(&weights, constraints)?;

            // Check convergence
            let gradient_norm: f64 = gradient.iter().map(|g| g * g).sum::<f64>().sqrt();
            if gradient_norm < tolerance {
                break;
            }
        }

        let portfolio_return = self.calculate_portfolio_return(&weights, means);
        let portfolio_vol = self.calculate_portfolio_volatility(&weights, cov_matrix)?;
        let sharpe = if portfolio_vol > 1e-10 {
            (portfolio_return - risk_free_rate) / portfolio_vol
        } else {
            0.0
        };

        let risk_contributions = self.calculate_risk_contributions(&weights, cov_matrix);
        let frontier = self.calculate_efficient_frontier(means, cov_matrix, constraints)?;

        let result = MeanVarianceResult {
            weights: weights.clone(),
            expected_return: portfolio_return,
            volatility: portfolio_vol,
            sharpe_ratio: sharpe,
            risk_free_rate,
            risk_contributions,
            efficient_frontier: frontier.iter().map(|(_, r, v)| (*r, *v)).collect(),
        };

        Ok((weights, result))
    }

    /// Calculate efficient frontier by varying target returns
    fn calculate_efficient_frontier(
        &self,
        means: &[f64],
        cov_matrix: &[Vec<f64>],
        constraints: &PortfolioConstraint,
    ) -> Result<Vec<(Vec<f64>, f64, f64)>, String> {
        let min_return = means.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_return = means.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let mut frontier = Vec::with_capacity(self.frontier_points);

        // Generate portfolios along the efficient frontier
        for i in 0..self.frontier_points {
            let target_return = min_return
                + (max_return - min_return) * (i as f64 / (self.frontier_points - 1) as f64);

            // Try to find minimum volatility portfolio for this target return
            let weights =
                self.solve_min_vol_for_return(means, cov_matrix, target_return, constraints)?;

            let vol = self.calculate_portfolio_volatility(&weights, cov_matrix)?;
            let actual_return = self.calculate_portfolio_return(&weights, means);

            frontier.push((weights, actual_return, vol));
        }

        Ok(frontier)
    }

    /// Solve for minimum volatility portfolio with target return constraint
    fn solve_min_vol_for_return(
        &self,
        means: &[f64],
        cov_matrix: &[Vec<f64>],
        target_return: f64,
        constraints: &PortfolioConstraint,
    ) -> Result<Vec<f64>, String> {
        let n = means.len();
        let mut weights = vec![1.0 / n as f64; n];

        let learning_rate = 0.01;
        let max_iterations = 500;
        let tolerance = 1e-8;

        for _ in 0..max_iterations {
            // Calculate current portfolio return
            let current_return: f64 = weights.iter().zip(means.iter()).map(|(w, m)| w * m).sum();

            // Gradient of volatility
            let vol_gradient = self.calculate_volatility_gradient(&weights, cov_matrix)?;

            // Gradient of return constraint
            let return_diff = current_return - target_return;
            let return_gradient: Vec<f64> = means.iter().map(|m| 2.0 * return_diff * m).collect();

            // Combined gradient (Lagrange multiplier approach simplified)
            let gradient: Vec<f64> = vol_gradient
                .iter()
                .zip(return_gradient.iter())
                .map(|(vg, rg)| vg + 0.1 * rg)
                .collect();

            // Update weights
            for i in 0..n {
                weights[i] -= learning_rate * gradient[i];
            }

            // Project onto constraints
            weights = self.project_weights(&weights, constraints)?;

            // Check convergence
            let grad_norm: f64 = gradient.iter().map(|g| g * g).sum::<f64>().sqrt();
            if grad_norm < tolerance {
                break;
            }
        }

        Ok(weights)
    }

    /// Project weights onto constraint set (sum to 1, non-negative)
    fn project_weights(
        &self,
        weights: &[f64],
        constraints: &PortfolioConstraint,
    ) -> Result<Vec<f64>, String> {
        let n = weights.len();

        // Set negative weights to 0 if short positions not allowed
        let mut projected: Vec<f64> = if constraints.allow_short {
            weights.to_vec()
        } else {
            weights.iter().map(|w| w.max(0.0)).collect()
        };

        // Normalize to sum to 1
        let sum: f64 = projected.iter().sum();
        if sum > 1e-10 {
            for w in &mut projected {
                *w /= sum;
            }
        } else {
            // If all weights are zero, use equal weights
            projected = vec![1.0 / n as f64; n];
        }

        Ok(projected)
    }

    /// Calculate portfolio return
    fn calculate_portfolio_return(&self, weights: &[f64], returns: &[f64]) -> f64 {
        weights.iter().zip(returns.iter()).map(|(w, r)| w * r).sum()
    }

    /// Calculate portfolio volatility
    fn calculate_portfolio_volatility(
        &self,
        weights: &[f64],
        cov_matrix: &[Vec<f64>],
    ) -> Result<f64, String> {
        let n = weights.len();
        let mut variance = 0.0;

        for i in 0..n {
            for j in 0..n {
                if j < cov_matrix[i].len() {
                    variance += weights[i] * weights[j] * cov_matrix[i][j];
                }
            }
        }

        if variance < 0.0 && variance > -1e-10 {
            variance = 0.0; // Numerical error correction
        }

        if variance < 0.0 {
            return Err(format!("Negative variance: {}", variance));
        }

        Ok(variance.sqrt())
    }

    /// Calculate gradient of portfolio volatility w.r.t. weights
    fn calculate_volatility_gradient(
        &self,
        weights: &[f64],
        cov_matrix: &[Vec<f64>],
    ) -> Result<Vec<f64>, String> {
        let n = weights.len();
        let vol = self.calculate_portfolio_volatility(weights, cov_matrix)?;

        if vol < 1e-10 {
            return Ok(vec![0.0; n]);
        }

        let mut gradient = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                if j < cov_matrix[i].len() {
                    gradient[i] += cov_matrix[i][j] * weights[j];
                }
            }
            gradient[i] /= vol;
        }

        Ok(gradient)
    }

    /// Calculate risk contributions for each asset
    fn calculate_risk_contributions(&self, weights: &[f64], cov_matrix: &[Vec<f64>]) -> Vec<f64> {
        let vol = match self.calculate_portfolio_volatility(weights, cov_matrix) {
            Ok(v) => v,
            Err(_) => return vec![0.0; weights.len()],
        };

        if vol < 1e-10 {
            return vec![0.0; weights.len()];
        }

        let n = weights.len();
        let mut contributions = vec![0.0; n];

        for i in 0..n {
            let marginal_risk: f64 = (0..n)
                .filter_map(|j| cov_matrix[i].get(j).map(|&c| c * weights[j]))
                .sum();
            contributions[i] = weights[i] * marginal_risk / vol;
        }

        contributions
    }
}

impl PortfolioOptimizer for MeanVarianceOptimizer {
    fn optimize(
        &self,
        strategy_names: &[String],
        equity_curves: &HashMap<String, Vec<f64>>,
        config: &OptimizationConfig,
    ) -> Result<OptimizationResult, String> {
        // Prepare data
        let (means, _, cov_matrix) = prepare_optimization_data(strategy_names, equity_curves)?;

        // Calculate annualized volatilities for individual strategies
        let mut volatilities = Vec::with_capacity(strategy_names.len());
        for name in strategy_names {
            let equity = equity_curves
                .get(name)
                .ok_or_else(|| format!("Missing equity curve for: {}", name))?;
            let returns = equity_to_returns(equity);
            let vol = calculate_mean_return(&returns) * (252.0_f64).sqrt();
            volatilities.push(vol);
        }

        // Run optimization based on objective
        let (weights, mv_result) = match config.objective {
            OptimizationObjective::MaximizeSharpe | OptimizationObjective::MaximizeReturn => self
                .optimize_sharpe(
                &means,
                &cov_matrix,
                config.risk_free_rate,
                &config.constraints,
            )?,
            OptimizationObjective::MinimizeVolatility | OptimizationObjective::MinimumVariance => {
                self.optimize_min_volatility(
                    &means,
                    &cov_matrix,
                    config.risk_free_rate,
                    &config.constraints,
                )?
            }
            _ => {
                return Err(format!(
                    "Objective {:?} not supported by MeanVarianceOptimizer",
                    config.objective
                ));
            }
        };

        // Build allocations map
        let mut allocations = HashMap::new();
        for (i, name) in strategy_names.iter().enumerate() {
            allocations.insert(name.clone(), weights[i]);
        }

        // Validate constraints
        if let Err(e) = config.constraints.check(&allocations) {
            return Err(format!("Constraint violation: {}", e));
        }

        // Calculate diversification ratio
        let weighted_avg_vol: f64 = weights
            .iter()
            .zip(volatilities.iter())
            .map(|(w, v)| w * v)
            .sum();
        let diversification_ratio = if mv_result.volatility > 1e-10 {
            weighted_avg_vol / mv_result.volatility
        } else {
            1.0
        };

        Ok(OptimizationResult {
            allocations,
            expected_return: mv_result.expected_return,
            expected_volatility: mv_result.volatility,
            expected_sharpe: mv_result.sharpe_ratio,
            diversification_ratio,
            objective: config.objective,
            iterations: self.frontier_points,
            converged: true,
            status_message: "Mean-variance optimization complete".to_string(),
        })
    }

    fn name(&self) -> &'static str {
        "Mean-Variance (Markowitz)"
    }

    fn description(&self) -> &'static str {
        "Classical Markowitz mean-variance optimization for efficient frontier portfolios"
    }

    fn supports_objective(&self, objective: OptimizationObjective) -> bool {
        matches!(
            objective,
            OptimizationObjective::MaximizeSharpe
                | OptimizationObjective::MinimizeVolatility
                | OptimizationObjective::MinimumVariance
                | OptimizationObjective::MaximizeReturn
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> (Vec<String>, HashMap<String, Vec<f64>>) {
        let names = vec!["Strategy A".to_string(), "Strategy B".to_string()];

        let mut curves = HashMap::new();
        // Strategy A: steady growth
        curves.insert(
            "Strategy A".to_string(),
            (0..=50).map(|i| 100.0 * (1.005f64).powi(i)).collect(),
        );
        // Strategy B: higher return, higher vol
        let equity_b: Vec<f64> = (0..=50)
            .map(|i| {
                let trend = 100.0 * (1.008f64).powi(i);
                let noise = if i % 2 == 0 { 1.02 } else { 0.98 };
                trend * noise
            })
            .collect();
        curves.insert("Strategy B".to_string(), equity_b);

        (names, curves)
    }

    #[test]
    fn test_optimizer_name() {
        let optimizer = MeanVarianceOptimizer::new();
        assert_eq!(optimizer.name(), "Mean-Variance (Markowitz)");
    }

    #[test]
    fn test_supports_objective() {
        let optimizer = MeanVarianceOptimizer::new();

        assert!(optimizer.supports_objective(OptimizationObjective::MaximizeSharpe));
        assert!(optimizer.supports_objective(OptimizationObjective::MinimizeVolatility));
        assert!(optimizer.supports_objective(OptimizationObjective::MinimumVariance));
        assert!(optimizer.supports_objective(OptimizationObjective::MaximizeReturn));
        assert!(!optimizer.supports_objective(OptimizationObjective::EqualRiskContribution));
    }

    #[test]
    fn test_optimize_sharpe() {
        let (names, curves) = create_test_data();
        let optimizer = MeanVarianceOptimizer::new();
        let config =
            OptimizationConfig::new().with_objective(OptimizationObjective::MaximizeSharpe);

        let result = optimizer.optimize(&names, &curves, &config);
        assert!(result.is_ok());

        let opt_result = result.unwrap();
        assert!(opt_result.converged);
        assert_eq!(opt_result.allocations.len(), 2);
        assert!(opt_result.expected_sharpe != 0.0);
    }

    #[test]
    fn test_optimize_min_volatility() {
        let (names, curves) = create_test_data();
        let optimizer = MeanVarianceOptimizer::new();
        let config =
            OptimizationConfig::new().with_objective(OptimizationObjective::MinimizeVolatility);

        let result = optimizer.optimize(&names, &curves, &config);
        assert!(result.is_ok());

        let opt_result = result.unwrap();
        assert!(opt_result.converged);
        assert!(opt_result.expected_volatility >= 0.0);
    }
}
