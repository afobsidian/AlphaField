//! Portfolio Optimization Algorithms
//!
//! This module contains implementations of various portfolio optimization
//! algorithms including mean-variance, risk parity, and inverse volatility.

pub mod inverse_volatility;
pub mod mean_variance;
pub mod risk_parity;

use std::collections::HashMap;

use crate::portfolio_optimization::{
    optimizer::{OptimizationConfig, PortfolioOptimizer},
    OptimizationObjective, OptimizationResult,
};

/// Factory for creating portfolio optimizers
pub struct OptimizerFactory;

impl OptimizerFactory {
    /// Create an optimizer for a given objective
    ///
    /// # Arguments
    /// * `objective` - The optimization objective
    ///
    /// # Returns
    /// A boxed optimizer that implements the PortfolioOptimizer trait
    pub fn create_optimizer(objective: OptimizationObjective) -> Box<dyn PortfolioOptimizer> {
        match objective {
            OptimizationObjective::MaximizeSharpe
            | OptimizationObjective::MinimizeVolatility
            | OptimizationObjective::MinimumVariance
            | OptimizationObjective::MaximizeReturn => {
                Box::new(mean_variance::MeanVarianceOptimizer::new())
            }
            OptimizationObjective::EqualRiskContribution => {
                Box::new(risk_parity::RiskParityOptimizer::new())
            }
            OptimizationObjective::MaximizeDiversification => {
                // For diversification, we use mean-variance optimizer
                // as it can find well-diversified portfolios
                Box::new(mean_variance::MeanVarianceOptimizer::new())
            }
        }
    }

    /// Get all available optimizers
    pub fn all_optimizers() -> Vec<Box<dyn PortfolioOptimizer>> {
        vec![
            Box::new(mean_variance::MeanVarianceOptimizer::new()),
            Box::new(risk_parity::RiskParityOptimizer::new()),
            Box::new(inverse_volatility::InverseVolatilityOptimizer::new()),
        ]
    }

    /// Get optimizer that best supports the given objective
    pub fn best_optimizer_for(objective: OptimizationObjective) -> Box<dyn PortfolioOptimizer> {
        let optimizers = Self::all_optimizers();

        for optimizer in optimizers {
            if optimizer.supports_objective(objective) {
                return optimizer;
            }
        }

        // Fallback to mean-variance
        Box::new(mean_variance::MeanVarianceOptimizer::new())
    }
}

/// Run optimization with multiple algorithms and return the best result
pub fn optimize_with_multiple_methods(
    strategy_names: &[String],
    equity_curves: &HashMap<String, Vec<f64>>,
    config: &OptimizationConfig,
) -> Result<(OptimizationObjective, OptimizationResult), String> {
    let objectives = vec![
        OptimizationObjective::MaximizeSharpe,
        OptimizationObjective::MinimizeVolatility,
        OptimizationObjective::EqualRiskContribution,
    ];

    let mut best_result: Option<(OptimizationObjective, OptimizationResult)> = None;
    let mut best_score = f64::MIN;

    for objective in objectives {
        let optimizer = OptimizerFactory::best_optimizer_for(objective);

        let mut obj_config = config.clone();
        obj_config.objective = objective;

        match optimizer.optimize(strategy_names, equity_curves, &obj_config) {
            Ok(result) => {
                // Score based on objective
                let score = match objective {
                    OptimizationObjective::MaximizeSharpe => result.expected_sharpe,
                    OptimizationObjective::MinimizeVolatility => -result.expected_volatility,
                    OptimizationObjective::EqualRiskContribution => result.diversification_ratio,
                    _ => result.expected_sharpe,
                };

                if score > best_score {
                    best_score = score;
                    best_result = Some((objective, result));
                }
            }
            Err(_) => continue,
        }
    }

    best_result.ok_or_else(|| "All optimization methods failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_data() -> (Vec<String>, HashMap<String, Vec<f64>>) {
        let names = vec!["Strategy A".to_string(), "Strategy B".to_string()];

        let mut curves = HashMap::new();
        curves.insert(
            "Strategy A".to_string(),
            (0..=50).map(|i| 100.0 * (1.005f64).powi(i)).collect(),
        );
        curves.insert(
            "Strategy B".to_string(),
            (0..=50).map(|i| 100.0 * (1.004f64).powi(i)).collect(),
        );

        (names, curves)
    }

    #[test]
    fn test_optimizer_factory_create() {
        let optimizer = OptimizerFactory::create_optimizer(OptimizationObjective::MaximizeSharpe);
        assert!(optimizer.supports_objective(OptimizationObjective::MaximizeSharpe));

        let optimizer =
            OptimizerFactory::create_optimizer(OptimizationObjective::EqualRiskContribution);
        assert!(optimizer.supports_objective(OptimizationObjective::EqualRiskContribution));
    }

    #[test]
    fn test_best_optimizer_for() {
        let optimizer = OptimizerFactory::best_optimizer_for(OptimizationObjective::MaximizeSharpe);
        assert!(optimizer.supports_objective(OptimizationObjective::MaximizeSharpe));

        let optimizer =
            OptimizerFactory::best_optimizer_for(OptimizationObjective::MinimizeVolatility);
        assert!(optimizer.supports_objective(OptimizationObjective::MinimizeVolatility));
    }

    #[test]
    fn test_all_optimizers() {
        let optimizers = OptimizerFactory::all_optimizers();
        assert!(!optimizers.is_empty());
    }

    #[test]
    fn test_optimize_with_multiple_methods() {
        let (names, curves) = create_test_data();
        let config = OptimizationConfig::default();

        let result = optimize_with_multiple_methods(&names, &curves, &config);
        assert!(result.is_ok());

        let (objective, opt_result) = result.unwrap();
        assert!(!opt_result.allocations.is_empty());
        assert!(matches!(
            objective,
            OptimizationObjective::MaximizeSharpe
                | OptimizationObjective::MinimizeVolatility
                | OptimizationObjective::EqualRiskContribution
        ));
    }
}
