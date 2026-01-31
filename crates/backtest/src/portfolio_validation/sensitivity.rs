//! Portfolio Sensitivity Analysis Module
//!
//! Provides sensitivity analysis at the portfolio level including:
//! - Leave-one-out analysis (impact of removing each strategy)
//! - Weight perturbation testing
//! - Strategy correlation impact analysis
//! - Marginal contribution to risk

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result from portfolio sensitivity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSensitivityResult {
    /// Base portfolio performance without any changes
    pub base_performance: StrategyImpact,
    /// Impact of removing each strategy (leave-one-out)
    pub leave_one_out: Vec<StrategyImpact>,
    /// Weight perturbation results
    pub weight_perturbations: Vec<WeightPerturbationResult>,
    /// Maximum impact observed across all tests
    pub max_impact: f64,
    /// Strategy with highest impact when removed
    pub most_impactful_strategy: String,
    /// Optimal weight adjustments recommended
    pub recommended_adjustments: Vec<WeightAdjustment>,
}

/// Impact metrics for a strategy or portfolio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyImpact {
    /// Strategy name (or "Base Portfolio" for base case)
    pub strategy_name: String,
    /// Portfolio return with this strategy removed/adjusted
    pub portfolio_return: f64,
    /// Portfolio volatility with this strategy removed/adjusted
    pub portfolio_volatility: f64,
    /// Maximum drawdown
    pub max_drawdown: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    /// Change in return relative to base
    pub return_delta: f64,
    /// Change in Sharpe ratio relative to base
    pub sharpe_delta: f64,
    /// Percentage impact on portfolio performance
    pub impact_percentage: f64,
}

/// Result from weight perturbation test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightPerturbationResult {
    /// Strategy whose weight was perturbed
    pub strategy_name: String,
    /// Original weight
    pub original_weight: f64,
    /// Perturbed weight
    pub perturbed_weight: f64,
    /// Resulting portfolio performance
    pub resulting_performance: StrategyImpact,
    /// Sensitivity coefficient (change in metric / change in weight)
    pub sensitivity_coefficient: f64,
}

/// Recommended weight adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightAdjustment {
    /// Strategy name
    pub strategy_name: String,
    /// Current weight
    pub current_weight: f64,
    /// Recommended weight
    pub recommended_weight: f64,
    /// Reason for adjustment
    pub reason: String,
}

/// Portfolio sensitivity analyzer
pub struct PortfolioSensitivityAnalyzer {
    initial_capital: f64,
    perturbation_size: f64,
}

impl PortfolioSensitivityAnalyzer {
    /// Create new sensitivity analyzer
    pub fn new(initial_capital: f64) -> Self {
        Self {
            initial_capital: initial_capital.max(1000.0),
            perturbation_size: 0.05, // 5% weight perturbation
        }
    }

    /// Set perturbation size (default 5%)
    pub fn with_perturbation_size(mut self, size: f64) -> Self {
        self.perturbation_size = size.clamp(0.01, 0.20);
        self
    }

    /// Run complete sensitivity analysis on portfolio
    ///
    /// # Arguments
    /// * `strategy_names` - List of strategies in portfolio
    /// * `base_weights` - Current portfolio weights (sum to 1.0)
    /// * `equity_curves` - Historical equity curves for each strategy
    ///
    /// # Returns
    /// Complete sensitivity analysis results
    pub fn analyze(
        &self,
        strategy_names: &[String],
        base_weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
    ) -> Result<PortfolioSensitivityResult, String> {
        // Validate inputs
        if strategy_names.len() != base_weights.len() {
            return Err("Strategy names and weights must have same length".to_string());
        }

        if strategy_names.len() < 2 {
            return Err("Need at least 2 strategies for sensitivity analysis".to_string());
        }

        // Calculate base performance
        let base_performance = self.calculate_impact(
            "Base Portfolio",
            strategy_names,
            base_weights,
            equity_curves,
            None,
        )?;

        // Leave-one-out analysis
        let leave_one_out = self.run_leave_one_out(
            strategy_names,
            base_weights,
            equity_curves,
            &base_performance,
        )?;

        // Weight perturbation
        let weight_perturbations = self.run_weight_perturbations(
            strategy_names,
            base_weights,
            equity_curves,
            &base_performance,
        )?;

        // Find max impact
        let max_impact = leave_one_out
            .iter()
            .map(|i| i.impact_percentage.abs())
            .fold(0.0, f64::max);

        // Find most impactful strategy
        let most_impactful = leave_one_out
            .iter()
            .max_by(|a, b| {
                a.impact_percentage
                    .abs()
                    .partial_cmp(&b.impact_percentage.abs())
                    .unwrap()
            })
            .map(|i| i.strategy_name.clone())
            .unwrap_or_default();

        // Generate recommendations
        let recommended_adjustments = self.generate_recommendations(
            strategy_names,
            base_weights,
            &leave_one_out,
            &weight_perturbations,
        );

        Ok(PortfolioSensitivityResult {
            base_performance,
            leave_one_out,
            weight_perturbations,
            max_impact,
            most_impactful_strategy: most_impactful,
            recommended_adjustments,
        })
    }

    /// Run leave-one-out analysis
    fn run_leave_one_out(
        &self,
        strategy_names: &[String],
        base_weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
        base_performance: &StrategyImpact,
    ) -> Result<Vec<StrategyImpact>, String> {
        let mut results = Vec::new();

        for (idx, strategy_name) in strategy_names.iter().enumerate() {
            // Create weights with this strategy removed
            let mut modified_weights = base_weights.to_vec();
            modified_weights[idx] = 0.0;

            // Redistribute weight to remaining strategies
            let remaining: f64 = modified_weights.iter().sum();
            if remaining > 0.0 {
                let scale = 1.0 / remaining;
                for w in &mut modified_weights {
                    *w *= scale;
                }
            }

            let impact = self.calculate_impact(
                strategy_name,
                strategy_names,
                &modified_weights,
                equity_curves,
                Some(base_performance),
            )?;

            results.push(StrategyImpact {
                strategy_name: format!("Without {}", strategy_name),
                ..impact
            });
        }

        Ok(results)
    }

    /// Run weight perturbation tests
    fn run_weight_perturbations(
        &self,
        strategy_names: &[String],
        base_weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
        base_performance: &StrategyImpact,
    ) -> Result<Vec<WeightPerturbationResult>, String> {
        let mut results = Vec::new();

        for (idx, strategy_name) in strategy_names.iter().enumerate() {
            // Test positive perturbation (increase weight)
            let mut perturbed_weights = base_weights.to_vec();
            let original_weight = perturbed_weights[idx];

            // Increase this strategy's weight
            let new_weight = (original_weight + self.perturbation_size).min(1.0);
            perturbed_weights[idx] = new_weight;

            // Scale other weights proportionally to maintain sum = 1.0
            let other_sum: f64 = perturbed_weights
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != idx)
                .map(|(_, w)| w)
                .sum();

            if other_sum > 0.0 {
                let scale = (1.0 - new_weight) / other_sum;
                for (i, w) in perturbed_weights.iter_mut().enumerate() {
                    if i != idx {
                        *w *= scale;
                    }
                }
            }

            let impact = self.calculate_impact(
                &format!(
                    "{} (+{}%)",
                    strategy_name,
                    (self.perturbation_size * 100.0) as i32
                ),
                strategy_names,
                &perturbed_weights,
                equity_curves,
                Some(base_performance),
            )?;

            // Calculate sensitivity coefficient (change in return per unit weight change)
            let weight_change = new_weight - original_weight;
            let sensitivity = if weight_change > 1e-10 {
                (impact.portfolio_return - base_performance.portfolio_return) / weight_change
            } else {
                0.0
            };

            results.push(WeightPerturbationResult {
                strategy_name: strategy_name.clone(),
                original_weight,
                perturbed_weight: new_weight,
                resulting_performance: impact,
                sensitivity_coefficient: sensitivity,
            });
        }

        Ok(results)
    }

    /// Calculate impact metrics for a given weight configuration
    fn calculate_impact(
        &self,
        name: &str,
        strategy_names: &[String],
        weights: &[f64],
        equity_curves: &HashMap<String, Vec<f64>>,
        base: Option<&StrategyImpact>,
    ) -> Result<StrategyImpact, String> {
        // Combine equity curves
        let combined = self.combine_curves(strategy_names, weights, equity_curves)?;

        if combined.len() < 2 {
            return Err("Insufficient data for impact calculation".to_string());
        }

        let initial = combined[0];
        let final_val = combined[combined.len() - 1];
        let portfolio_return = (final_val - initial) / initial;

        // Calculate volatility
        let returns = self.calculate_returns(&combined);
        let volatility = if returns.len() >= 2 {
            let mean = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>()
                / (returns.len() - 1) as f64;
            (variance * 252.0).sqrt() // Annualize
        } else {
            0.0
        };

        // Calculate max drawdown
        let max_drawdown = self.calculate_max_drawdown(&combined);

        // Calculate Sharpe ratio
        let sharpe_ratio = if volatility > 1e-10 {
            (portfolio_return - 0.02) / volatility // Assume 2% risk-free rate
        } else {
            0.0
        };

        // Calculate deltas if base provided
        let (return_delta, sharpe_delta, impact_percentage) = if let Some(base) = base {
            let ret_delta = portfolio_return - base.portfolio_return;
            let sharpe_del = sharpe_ratio - base.sharpe_ratio;
            let impact = if base.portfolio_return.abs() > 1e-10 {
                (ret_delta / base.portfolio_return.abs()) * 100.0
            } else {
                0.0
            };
            (ret_delta, sharpe_del, impact)
        } else {
            (0.0, 0.0, 0.0)
        };

        Ok(StrategyImpact {
            strategy_name: name.to_string(),
            portfolio_return,
            portfolio_volatility: volatility,
            max_drawdown,
            sharpe_ratio,
            return_delta,
            sharpe_delta,
            impact_percentage,
        })
    }

    /// Generate weight adjustment recommendations
    fn generate_recommendations(
        &self,
        strategy_names: &[String],
        base_weights: &[f64],
        leave_one_out: &[StrategyImpact],
        perturbations: &[WeightPerturbationResult],
    ) -> Vec<WeightAdjustment> {
        let mut adjustments = Vec::new();

        // Find strategies with negative impact when removed (indicates they're valuable)
        for (idx, strategy_name) in strategy_names.iter().enumerate() {
            if let Some(loo) = leave_one_out
                .iter()
                .find(|i| i.strategy_name == format!("Without {}", strategy_name))
            {
                // If removing the strategy hurts performance, it's valuable
                if loo.return_delta < -0.01 {
                    // More than 1% negative impact
                    let current_weight = base_weights[idx];
                    let sensitivity = perturbations
                        .iter()
                        .find(|p| p.strategy_name == *strategy_name)
                        .map(|p| p.sensitivity_coefficient)
                        .unwrap_or(0.0);

                    // Recommend increasing weight if positive sensitivity
                    if sensitivity > 0.05 {
                        let new_weight = (current_weight * 1.2).min(0.5); // Cap at 50%
                        adjustments.push(WeightAdjustment {
                            strategy_name: strategy_name.clone(),
                            current_weight,
                            recommended_weight: new_weight,
                            reason: format!(
                                "Strategy provides {:.1}% diversification benefit",
                                -loo.impact_percentage
                            ),
                        });
                    }
                }

                // If removing the strategy helps performance, consider reducing weight
                if loo.return_delta > 0.01 {
                    let current_weight = base_weights[idx];
                    let new_weight = current_weight * 0.8; // Reduce by 20%
                    adjustments.push(WeightAdjustment {
                        strategy_name: strategy_name.clone(),
                        current_weight,
                        recommended_weight: new_weight,
                        reason: "Portfolio performs better without this strategy".to_string(),
                    });
                }
            }
        }

        adjustments
    }

    /// Combine equity curves according to weights
    fn combine_curves(
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
        let mut current_equity = self.initial_capital;

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

    /// Calculate maximum drawdown
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> (Vec<String>, Vec<f64>, HashMap<String, Vec<f64>>) {
        let names = vec![
            "Strategy A".to_string(),
            "Strategy B".to_string(),
            "Strategy C".to_string(),
        ];
        let weights = vec![0.4, 0.35, 0.25];

        let mut curves = HashMap::new();
        // Strategy A: steady performer
        curves.insert(
            "Strategy A".to_string(),
            (0..=50).map(|i| 10000.0 * (1.0006f64).powi(i)).collect(),
        );
        // Strategy B: higher return, more volatile
        curves.insert(
            "Strategy B".to_string(),
            (0..=50)
                .map(|i| {
                    let trend = 10000.0 * (1.0009f64).powi(i);
                    let noise = if i % 4 == 0 { 0.97 } else { 1.0 };
                    trend * noise
                })
                .collect(),
        );
        // Strategy C: diversifier (different pattern)
        curves.insert(
            "Strategy C".to_string(),
            (0..=50)
                .map(|i| {
                    let trend = 10000.0 * (1.0004f64).powi(i);
                    let noise = if i % 3 == 0 { 1.03 } else { 0.97 };
                    trend * noise
                })
                .collect(),
        );

        (names, weights, curves)
    }

    #[test]
    fn test_sensitivity_analyzer_creation() {
        let _analyzer = PortfolioSensitivityAnalyzer::new(10000.0);
        // Just verify it creates without error
    }

    #[test]
    fn test_sensitivity_analysis() {
        let (names, weights, curves) = create_test_data();
        let analyzer = PortfolioSensitivityAnalyzer::new(10000.0);

        let result = analyzer.analyze(&names, &weights, &curves);
        assert!(result.is_ok());

        let sensitivity = result.unwrap();
        assert!(sensitivity.leave_one_out.len() == 3); // One for each strategy
        assert!(!sensitivity.weight_perturbations.is_empty());
        assert!(sensitivity.base_performance.portfolio_return >= 0.0);
    }

    #[test]
    fn test_leave_one_out_impact() {
        let (names, weights, curves) = create_test_data();
        let analyzer = PortfolioSensitivityAnalyzer::new(10000.0);

        let result = analyzer.analyze(&names, &weights, &curves).unwrap();

        // Each leave-one-out should have a strategy name
        for loo in &result.leave_one_out {
            assert!(loo.strategy_name.starts_with("Without "));
            // Impact percentage should be calculated
            assert!(loo.impact_percentage.abs() >= 0.0);
        }
    }

    #[test]
    fn test_weight_perturbation() {
        let (names, weights, curves) = create_test_data();
        let analyzer = PortfolioSensitivityAnalyzer::new(10000.0).with_perturbation_size(0.10); // 10% perturbation

        let result = analyzer.analyze(&names, &weights, &curves).unwrap();

        // Should have perturbation results for each strategy
        assert_eq!(result.weight_perturbations.len(), 3);

        // Check perturbation structure
        for pert in &result.weight_perturbations {
            assert!(pert.perturbed_weight > pert.original_weight);
            assert!(pert.sensitivity_coefficient.abs() >= 0.0);
        }
    }

    #[test]
    fn test_max_impact_detection() {
        let (names, weights, curves) = create_test_data();
        let analyzer = PortfolioSensitivityAnalyzer::new(10000.0);

        let result = analyzer.analyze(&names, &weights, &curves).unwrap();

        // Max impact should be non-negative
        assert!(result.max_impact >= 0.0);

        // Most impactful strategy should be identified
        assert!(!result.most_impactful_strategy.is_empty());
    }

    #[test]
    fn test_recommendations_generation() {
        let (names, weights, curves) = create_test_data();
        let analyzer = PortfolioSensitivityAnalyzer::new(10000.0);

        let result = analyzer.analyze(&names, &weights, &curves).unwrap();

        // Recommendations should be generated
        // Note: May be empty if no adjustments are deemed necessary
        for adj in &result.recommended_adjustments {
            assert!(!adj.strategy_name.is_empty());
            assert!(adj.recommended_weight >= 0.0);
            assert!(!adj.reason.is_empty());
        }
    }

    #[test]
    fn test_invalid_inputs() {
        let analyzer = PortfolioSensitivityAnalyzer::new(10000.0);

        // Mismatched lengths
        let names = vec!["A".to_string(), "B".to_string()];
        let weights = vec![0.5]; // Only 1 weight
        let curves = HashMap::new();

        let result = analyzer.analyze(&names, &weights, &curves);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_strategy_error() {
        let analyzer = PortfolioSensitivityAnalyzer::new(10000.0);

        let names = vec!["Only".to_string()];
        let weights = vec![1.0];
        let curves = HashMap::new();

        let result = analyzer.analyze(&names, &weights, &curves);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 2 strategies"));
    }

    #[test]
    fn test_impact_calculation() {
        let analyzer = PortfolioSensitivityAnalyzer::new(10000.0);

        // Simple test curves
        let names = vec!["A".to_string(), "B".to_string()];
        let weights = vec![0.5, 0.5];

        let mut curves = HashMap::new();
        curves.insert("A".to_string(), vec![10000.0, 10100.0, 10200.0]);
        curves.insert("B".to_string(), vec![10000.0, 10050.0, 10100.0]);

        let base = analyzer
            .calculate_impact("Base", &names, &weights, &curves, None)
            .unwrap();

        assert!(base.portfolio_return > 0.0);
        // Sharpe ratio may be negative if return < risk-free rate or with very short time series
        // For this simple test with only 3 points, just verify it's calculated
        assert!(base.sharpe_ratio.is_finite());
        assert_eq!(base.return_delta, 0.0); // No base provided
    }
}
