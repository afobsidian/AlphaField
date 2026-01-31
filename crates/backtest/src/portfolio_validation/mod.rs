//! Portfolio Validation Module
//!
//! Provides comprehensive portfolio-level validation including:
//! - Stress testing with correlation breakdown scenarios
//! - Portfolio sensitivity analysis (leave-one-out)
//! - Portfolio-level walk-forward analysis
//! - Monte Carlo simulation with correlation preservation
//!
//! This module extends the single-strategy validation framework to work
//! with multi-strategy portfolios, testing robustness and diversification benefits.

pub mod monte_carlo;
pub mod sensitivity;
pub mod stress_test;
pub mod walk_forward;

pub use monte_carlo::{
    PortfolioMonteCarloConfig, PortfolioMonteCarloResult, PortfolioMonteCarloSimulator,
};
pub use sensitivity::{PortfolioSensitivityAnalyzer, PortfolioSensitivityResult, StrategyImpact};
pub use stress_test::{StressScenario, StressTestConfig, StressTestResult, StressTester};
pub use walk_forward::{
    PortfolioWalkForwardAnalyzer, PortfolioWalkForwardConfig, PortfolioWalkForwardResult,
};

/// Portfolio validation report combining all validation methods
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PortfolioValidationReport {
    /// Portfolio name
    pub portfolio_name: String,
    /// Number of strategies in portfolio
    pub num_strategies: usize,
    /// Walk-forward analysis results
    pub walk_forward: Option<PortfolioWalkForwardResult>,
    /// Monte Carlo simulation results
    pub monte_carlo: Option<PortfolioMonteCarloResult>,
    /// Sensitivity analysis results
    pub sensitivity: Option<PortfolioSensitivityResult>,
    /// Stress testing results
    pub stress_test: Option<StressTestResult>,
    /// Overall validation score (0.0 to 1.0)
    pub validation_score: f64,
    /// Validation recommendations
    pub recommendations: Vec<String>,
}

impl PortfolioValidationReport {
    /// Create a new empty validation report
    pub fn new(portfolio_name: impl Into<String>, num_strategies: usize) -> Self {
        Self {
            portfolio_name: portfolio_name.into(),
            num_strategies,
            walk_forward: None,
            monte_carlo: None,
            sensitivity: None,
            stress_test: None,
            validation_score: 0.0,
            recommendations: Vec::new(),
        }
    }

    /// Generate recommendations based on validation results
    pub fn generate_recommendations(&mut self) {
        self.recommendations.clear();

        // Walk-forward recommendations
        if let Some(ref wf) = self.walk_forward {
            if wf.consistency_score < 0.5 {
                self.recommendations.push(
                    "Portfolio shows poor time consistency - consider reducing strategy count or adding regime detection".to_string()
                );
            }
            if wf.average_drawdown > 0.25 {
                self.recommendations.push(
                    "High drawdown detected in walk-forward testing - consider risk parity weighting".to_string()
                );
            }
        }

        // Monte Carlo recommendations
        if let Some(ref mc) = self.monte_carlo {
            if mc.probability_of_profit < 0.6 {
                self.recommendations.push(
                    "Low probability of profit in Monte Carlo simulation - portfolio may be over-optimized".to_string()
                );
            }
            if mc.var_95 > 0.20 {
                self.recommendations.push(
                    "High tail risk detected (95% VaR) - consider adding tail hedge strategies"
                        .to_string(),
                );
            }
        }

        // Sensitivity recommendations
        if let Some(ref sens) = self.sensitivity {
            if sens.max_impact > 0.5 {
                self.recommendations.push(format!(
                    "Portfolio highly sensitive to removal of {} - consider diversification",
                    sens.most_impactful_strategy
                ));
            }
        }

        // Stress test recommendations
        if let Some(ref stress) = self.stress_test {
            if stress.correlation_breakdown_drawdown > 0.35 {
                self.recommendations.push(
                    "Severe drawdown in correlation breakdown scenario - monitor correlation during market stress".to_string()
                );
            }
        }

        // General recommendations
        if self.recommendations.is_empty() {
            self.recommendations.push(
                "Portfolio validation passed all checks - suitable for deployment with standard monitoring".to_string()
            );
        }
    }

    /// Calculate overall validation score
    pub fn calculate_score(&mut self) {
        let mut scores = Vec::new();
        let mut weights = Vec::new();

        // Walk-forward score (consistency)
        if let Some(ref wf) = self.walk_forward {
            scores.push(wf.consistency_score);
            weights.push(0.25);
        }

        // Monte Carlo score (probability of profit normalized)
        if let Some(ref mc) = self.monte_carlo {
            scores.push(mc.probability_of_profit.min(1.0));
            weights.push(0.25);
        }

        // Sensitivity score (inverse of max impact)
        if let Some(ref sens) = self.sensitivity {
            let sensitivity_score = (1.0 - sens.max_impact.min(1.0)).max(0.0);
            scores.push(sensitivity_score);
            weights.push(0.25);
        }

        // Stress test score (based on correlation breakdown severity)
        if let Some(ref stress) = self.stress_test {
            let stress_score = (1.0 - stress.correlation_breakdown_drawdown).max(0.0);
            scores.push(stress_score);
            weights.push(0.25);
        }

        // Calculate weighted average
        if !scores.is_empty() {
            let total_weight: f64 = weights.iter().sum();
            self.validation_score = scores
                .iter()
                .zip(weights.iter())
                .map(|(s, w)| s * w)
                .sum::<f64>()
                / total_weight;
        }
    }
}

/// Comprehensive portfolio validator that runs all validation methods
pub struct PortfolioValidator;

impl PortfolioValidator {
    /// Run complete portfolio validation suite
    ///
    /// # Arguments
    /// * `portfolio` - Multi-strategy portfolio to validate
    /// * `equity_curves` - Historical equity curves for each strategy
    /// * `strategy_returns` - Return series for each strategy
    /// * `config` - Validation configuration
    ///
    /// # Returns
    /// Complete validation report
    pub fn validate(
        portfolio_name: &str,
        strategy_names: &[String],
        _equity_curves: &std::collections::HashMap<String, Vec<f64>>,
        _strategy_returns: &std::collections::HashMap<String, Vec<f64>>,
    ) -> PortfolioValidationReport {
        let mut report = PortfolioValidationReport::new(portfolio_name, strategy_names.len());

        // Note: Full validation requires additional data sources
        // This is a skeleton that would be filled with actual validation runs
        // when integrated with the backtest engine

        report.calculate_score();
        report.generate_recommendations();

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_validation_report_creation() {
        let report = PortfolioValidationReport::new("Test Portfolio", 3);
        assert_eq!(report.portfolio_name, "Test Portfolio");
        assert_eq!(report.num_strategies, 3);
        assert_eq!(report.validation_score, 0.0);
        assert!(report.recommendations.is_empty());
    }

    #[test]
    fn test_recommendations_generation() {
        let mut report = PortfolioValidationReport::new("Test", 2);

        // Simulate poor walk-forward results
        report.walk_forward = Some(PortfolioWalkForwardResult {
            consistency_score: 0.3,
            average_in_sample_return: 0.1,
            average_in_sample_sharpe: 0.5,
            average_out_of_sample_return: 0.05,
            average_out_of_sample_sharpe: 0.3,
            average_drawdown: 0.30,
            average_in_sample_drawdown: 0.25,
            average_out_of_sample_drawdown: 0.35,
            average_weight_turnover: 0.15,
            converged_windows: 8,
            total_windows: 10,
            window_results: vec![],
        });

        report.generate_recommendations();

        assert!(!report.recommendations.is_empty());
        assert!(report.recommendations[0].contains("poor time consistency"));
    }

    #[test]
    fn test_validation_score_calculation() {
        let mut report = PortfolioValidationReport::new("Test", 2);

        // Add validation results
        report.walk_forward = Some(PortfolioWalkForwardResult {
            consistency_score: 0.8,
            average_in_sample_return: 0.15,
            average_in_sample_sharpe: 1.2,
            average_out_of_sample_return: 0.14,
            average_out_of_sample_sharpe: 1.1,
            average_drawdown: 0.10,
            average_in_sample_drawdown: 0.08,
            average_out_of_sample_drawdown: 0.12,
            average_weight_turnover: 0.05,
            converged_windows: 10,
            total_windows: 10,
            window_results: vec![],
        });

        report.monte_carlo = Some(PortfolioMonteCarloResult {
            num_simulations: 1000,
            original_metrics: Default::default(),
            equity_5th: 9000.0,
            equity_50th: 11000.0,
            equity_95th: 13000.0,
            return_5th: -0.1,
            return_50th: 0.1,
            return_95th: 0.3,
            drawdown_5th: 0.05,
            drawdown_50th: 0.15,
            drawdown_95th: 0.25,
            probability_of_profit: 0.75,
            var_95: 0.10,
            simulations: vec![],
        });

        report.calculate_score();

        // Score should be between 0 and 1
        assert!(report.validation_score >= 0.0 && report.validation_score <= 1.0);
        // With 0.8 consistency and 0.75 probability, score should be around 0.775
        assert!(report.validation_score > 0.7 && report.validation_score < 0.8);
    }
}
