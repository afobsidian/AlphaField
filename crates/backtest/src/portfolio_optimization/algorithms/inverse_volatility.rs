//! Inverse Volatility Weighting Algorithm
//!
//! Simple portfolio optimization based on inverse volatility weighting.
//! Strategies with lower volatility receive higher weights.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::portfolio_optimization::{
    optimizer::{calculate_volatility, equity_to_returns, prepare_optimization_data},
    OptimizationConfig, OptimizationObjective, OptimizationResult, PortfolioOptimizer,
};

/// Result from inverse volatility optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InverseVolatilityWeights {
    /// Strategy names
    pub strategies: Vec<String>,
    /// Weights for each strategy
    pub weights: Vec<f64>,
    /// Volatility for each strategy
    pub volatilities: Vec<f64>,
    /// Portfolio volatility
    pub portfolio_volatility: f64,
}

/// Inverse Volatility Portfolio Optimizer
///
/// This optimizer allocates weights inversely proportional to each
/// strategy's volatility. Lower volatility strategies receive higher weights.
#[derive(Debug, Clone, Default)]
pub struct InverseVolatilityOptimizer;

impl InverseVolatilityOptimizer {
    pub fn new() -> Self {
        Self
    }

    /// Calculate inverse volatility weights
    fn calculate_weights(&self, volatilities: &[f64]) -> Vec<f64> {
        let inverse_vols: Vec<f64> = volatilities
            .iter()
            .map(|v| {
                if *v < 1e-10 {
                    1e10 // Very high weight for zero volatility
                } else {
                    1.0 / v
                }
            })
            .collect();

        let total: f64 = inverse_vols.iter().sum();
        if total < 1e-10 {
            // Equal weight fallback
            let n = volatilities.len();
            vec![1.0 / n as f64; n]
        } else {
            inverse_vols.iter().map(|iv| iv / total).collect()
        }
    }

    /// Calculate portfolio volatility given weights and covariance matrix
    fn calculate_portfolio_volatility(&self, weights: &[f64], cov_matrix: &[Vec<f64>]) -> f64 {
        if weights.is_empty() || cov_matrix.is_empty() {
            return 0.0;
        }

        let mut variance = 0.0;
        let n = weights.len();

        for i in 0..n {
            for j in 0..n {
                if j < cov_matrix[i].len() {
                    variance += weights[i] * weights[j] * cov_matrix[i][j];
                }
            }
        }

        variance.sqrt()
    }
}

impl PortfolioOptimizer for InverseVolatilityOptimizer {
    fn optimize(
        &self,
        strategy_names: &[String],
        equity_curves: &HashMap<String, Vec<f64>>,
        config: &OptimizationConfig,
    ) -> Result<OptimizationResult, String> {
        if strategy_names.len() < 2 {
            return Err("Need at least 2 strategies for portfolio optimization".to_string());
        }

        // Prepare optimization data
        let (means, _return_series, cov_matrix) =
            prepare_optimization_data(strategy_names, equity_curves)?;

        // Calculate individual volatilities (annualized)
        let mut volatilities = Vec::with_capacity(strategy_names.len());
        for name in strategy_names {
            let equity = equity_curves
                .get(name)
                .ok_or_else(|| format!("Missing equity curve for: {}", name))?;
            let returns = equity_to_returns(equity);
            let vol = calculate_volatility(&returns) * (252.0_f64).sqrt(); // Annualize
            volatilities.push(vol);
        }

        // Calculate inverse volatility weights
        let weights = self.calculate_weights(&volatilities);

        // Build allocations map
        let mut allocations = HashMap::new();
        for (i, name) in strategy_names.iter().enumerate() {
            allocations.insert(name.clone(), weights[i]);
        }

        // Validate constraints
        if let Err(e) = config.constraints.check(&allocations) {
            return Err(format!("Constraint violation: {}", e));
        }

        // Calculate portfolio statistics
        let portfolio_return: f64 = weights.iter().zip(means.iter()).map(|(w, r)| w * r).sum();
        let portfolio_vol = self.calculate_portfolio_volatility(&weights, &cov_matrix);
        let portfolio_vol_annual = portfolio_vol * (252.0_f64).sqrt();

        let sharpe = if portfolio_vol_annual > 1e-10 {
            (portfolio_return - config.risk_free_rate) / portfolio_vol_annual
        } else {
            0.0
        };

        // Calculate diversification ratio
        let weighted_avg_vol: f64 = weights
            .iter()
            .zip(volatilities.iter())
            .map(|(w, v)| w * v)
            .sum();
        let diversification_ratio = if portfolio_vol_annual > 1e-10 {
            weighted_avg_vol / portfolio_vol_annual
        } else {
            1.0
        };

        Ok(OptimizationResult {
            allocations,
            expected_return: portfolio_return,
            expected_volatility: portfolio_vol_annual,
            expected_sharpe: sharpe,
            diversification_ratio,
            objective: OptimizationObjective::MinimizeVolatility,
            iterations: 1,
            converged: true,
            status_message: "Inverse volatility optimization complete".to_string(),
        })
    }

    fn name(&self) -> &'static str {
        "Inverse Volatility"
    }

    fn description(&self) -> &'static str {
        "Allocates weights inversely proportional to strategy volatility"
    }

    fn supports_objective(&self, objective: OptimizationObjective) -> bool {
        matches!(
            objective,
            OptimizationObjective::MinimizeVolatility | OptimizationObjective::MinimumVariance
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> (Vec<String>, HashMap<String, Vec<f64>>) {
        let names = vec![
            "Low Vol Strategy".to_string(),
            "High Vol Strategy".to_string(),
        ];

        let mut curves = HashMap::new();
        // Low vol: steady 1% daily returns
        curves.insert(
            "Low Vol Strategy".to_string(),
            (0..=30).map(|i| 100.0 * (1.01f64).powi(i)).collect(),
        );
        // High vol: more erratic 2% average but with swings
        let high_vol_equity: Vec<f64> = vec![
            100.0, 106.0, 103.0, 110.0, 105.0, 112.0, 108.0, 115.0, 110.0, 118.0, 113.0, 120.0,
            115.0, 122.0, 117.0, 125.0, 120.0, 127.0, 122.0, 130.0, 125.0, 132.0, 127.0, 135.0,
            130.0, 138.0, 132.0, 140.0, 135.0, 143.0, 138.0,
        ];
        curves.insert("High Vol Strategy".to_string(), high_vol_equity);

        (names, curves)
    }

    #[test]
    fn test_inverse_volatility_optimizer_name() {
        let optimizer = InverseVolatilityOptimizer::new();
        assert_eq!(optimizer.name(), "Inverse Volatility");
    }

    #[test]
    fn test_calculate_weights() {
        let optimizer = InverseVolatilityOptimizer::new();

        // Low vol = 0.10, High vol = 0.30
        let vols = vec![0.10, 0.30];
        let weights = optimizer.calculate_weights(&vols);

        // Low vol should get higher weight
        // Inverse: 10.0 and 3.333... -> normalized: 0.75 and 0.25
        assert!(weights[0] > weights[1]);
        assert!((weights[0] + weights[1] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_optimization() {
        let (names, curves) = create_test_data();
        let optimizer = InverseVolatilityOptimizer::new();
        let config = OptimizationConfig::default();

        let result = optimizer.optimize(&names, &curves, &config);
        assert!(result.is_ok());

        let opt_result = result.unwrap();
        assert!(opt_result.converged);
        assert_eq!(opt_result.allocations.len(), 2);

        // Low vol strategy should get higher allocation
        let low_vol_weight = opt_result.allocations.get("Low Vol Strategy").unwrap();
        let high_vol_weight = opt_result.allocations.get("High Vol Strategy").unwrap();
        assert!(low_vol_weight > high_vol_weight);

        // Weights should sum to 1.0
        let total: f64 = opt_result.allocations.values().sum();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_too_few_strategies() {
        let optimizer = InverseVolatilityOptimizer::new();
        let names = vec!["Only One".to_string()];
        let curves = HashMap::new();
        let config = OptimizationConfig::default();

        let result = optimizer.optimize(&names, &curves, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_supports_objective() {
        let optimizer = InverseVolatilityOptimizer::new();

        assert!(optimizer.supports_objective(OptimizationObjective::MinimizeVolatility));
        assert!(optimizer.supports_objective(OptimizationObjective::MinimumVariance));
        assert!(!optimizer.supports_objective(OptimizationObjective::MaximizeSharpe));
        assert!(!optimizer.supports_objective(OptimizationObjective::MaximizeReturn));
    }
}
