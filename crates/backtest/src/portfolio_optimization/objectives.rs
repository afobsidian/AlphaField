//! Portfolio Optimization Objectives
//!
//! Defines various objective functions for portfolio optimization,
//! such as maximizing Sharpe ratio, minimizing volatility, or maximizing diversification.

use serde::{Deserialize, Serialize};

/// High-level optimization objective
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum OptimizationObjective {
    /// Maximize Sharpe ratio (risk-adjusted return)
    #[default]
    MaximizeSharpe,
    /// Minimize portfolio volatility
    MinimizeVolatility,
    /// Maximize expected return
    MaximizeReturn,
    /// Maximize diversification ratio
    MaximizeDiversification,
    /// Equal risk contribution (Risk Parity)
    EqualRiskContribution,
    /// Minimum variance portfolio
    MinimumVariance,
}

impl OptimizationObjective {
    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            OptimizationObjective::MaximizeSharpe => "Maximize Sharpe Ratio",
            OptimizationObjective::MinimizeVolatility => "Minimize Volatility",
            OptimizationObjective::MaximizeReturn => "Maximize Expected Return",
            OptimizationObjective::MaximizeDiversification => "Maximize Diversification Ratio",
            OptimizationObjective::EqualRiskContribution => "Equal Risk Contribution (Risk Parity)",
            OptimizationObjective::MinimumVariance => "Minimum Variance Portfolio",
        }
    }

    /// Whether this objective seeks to maximize (true) or minimize (false)
    pub fn is_maximization(&self) -> bool {
        match self {
            OptimizationObjective::MaximizeSharpe => true,
            OptimizationObjective::MinimizeVolatility => false,
            OptimizationObjective::MaximizeReturn => true,
            OptimizationObjective::MaximizeDiversification => true,
            OptimizationObjective::EqualRiskContribution => true,
            OptimizationObjective::MinimumVariance => false,
        }
    }
}

/// A portfolio objective function that can be evaluated
pub trait PortfolioObjective {
    /// Evaluate the objective function given returns, volatilities, and correlations
    ///
    /// # Arguments
    /// * `weights` - Portfolio weights for each strategy
    /// * `returns` - Expected returns for each strategy
    /// * `cov_matrix` - Covariance matrix (strategies x strategies)
    ///
    /// # Returns
    /// The objective value (higher is better for maximization objectives)
    fn evaluate(
        &self,
        weights: &[f64],
        returns: &[f64],
        cov_matrix: &[Vec<f64>],
    ) -> Result<f64, String>;

    /// Get the objective type
    fn objective_type(&self) -> OptimizationObjective;

    /// Whether higher values are better
    fn is_maximization(&self) -> bool {
        self.objective_type().is_maximization()
    }
}

/// Sharpe ratio objective
#[derive(Debug, Clone)]
pub struct SharpeObjective {
    risk_free_rate: f64,
}

impl SharpeObjective {
    pub fn new(risk_free_rate: f64) -> Self {
        Self { risk_free_rate }
    }
}

impl Default for SharpeObjective {
    fn default() -> Self {
        Self {
            risk_free_rate: 0.02,
        }
    }
}

impl PortfolioObjective for SharpeObjective {
    fn evaluate(
        &self,
        weights: &[f64],
        returns: &[f64],
        cov_matrix: &[Vec<f64>],
    ) -> Result<f64, String> {
        if weights.is_empty() || returns.is_empty() {
            return Err("Empty weights or returns".to_string());
        }

        // Calculate portfolio return
        let portfolio_return: f64 = weights.iter().zip(returns.iter()).map(|(w, r)| w * r).sum();

        // Calculate portfolio variance: w^T * Σ * w
        let portfolio_variance = calculate_portfolio_variance(weights, cov_matrix)?;
        let portfolio_volatility = portfolio_variance.sqrt();

        if portfolio_volatility < 1e-10 {
            return Ok(0.0); // Avoid division by zero
        }

        let sharpe = (portfolio_return - self.risk_free_rate) / portfolio_volatility;
        Ok(sharpe)
    }

    fn objective_type(&self) -> OptimizationObjective {
        OptimizationObjective::MaximizeSharpe
    }
}

/// Volatility minimization objective
#[derive(Debug, Clone, Default)]
pub struct VolatilityObjective;

impl PortfolioObjective for VolatilityObjective {
    fn evaluate(
        &self,
        weights: &[f64],
        _returns: &[f64],
        cov_matrix: &[Vec<f64>],
    ) -> Result<f64, String> {
        let variance = calculate_portfolio_variance(weights, cov_matrix)?;
        let volatility = variance.sqrt();
        Ok(-volatility) // Negative because we minimize, but interface maximizes
    }

    fn objective_type(&self) -> OptimizationObjective {
        OptimizationObjective::MinimizeVolatility
    }
}

/// Return maximization objective (ignores risk)
#[derive(Debug, Clone, Default)]
pub struct ReturnObjective;

impl PortfolioObjective for ReturnObjective {
    fn evaluate(
        &self,
        weights: &[f64],
        returns: &[f64],
        _cov_matrix: &[Vec<f64>],
    ) -> Result<f64, String> {
        let portfolio_return: f64 = weights.iter().zip(returns.iter()).map(|(w, r)| w * r).sum();
        Ok(portfolio_return)
    }

    fn objective_type(&self) -> OptimizationObjective {
        OptimizationObjective::MaximizeReturn
    }
}

/// Diversification ratio objective
/// Diversification Ratio = (weighted average volatility) / portfolio volatility
/// Higher is better (above 1.0 means diversification benefits)
#[derive(Debug, Clone, Default)]
pub struct DiversificationObjective;

impl PortfolioObjective for DiversificationObjective {
    fn evaluate(
        &self,
        weights: &[f64],
        _returns: &[f64],
        cov_matrix: &[Vec<f64>],
    ) -> Result<f64, String> {
        if weights.is_empty() || cov_matrix.is_empty() {
            return Err("Empty weights or covariance matrix".to_string());
        }

        // Calculate individual volatilities (standard deviations)
        let volatilities: Vec<f64> = (0..weights.len())
            .map(|i| cov_matrix[i][i].sqrt())
            .collect();

        // Weighted average volatility
        let weighted_avg_vol: f64 = weights
            .iter()
            .zip(volatilities.iter())
            .map(|(w, v)| w * v)
            .sum();

        // Portfolio volatility
        let portfolio_var = calculate_portfolio_variance(weights, cov_matrix)?;
        let portfolio_vol = portfolio_var.sqrt();

        if portfolio_vol < 1e-10 {
            return Ok(1.0); // No diversification possible
        }

        let diversification_ratio = weighted_avg_vol / portfolio_vol;
        Ok(diversification_ratio)
    }

    fn objective_type(&self) -> OptimizationObjective {
        OptimizationObjective::MaximizeDiversification
    }
}

/// Risk Parity objective - equal risk contribution from each asset
/// Minimizes variance of risk contributions
#[derive(Debug, Clone, Default)]
pub struct RiskParityObjective;

impl RiskParityObjective {
    /// Calculate risk contribution for each asset
    fn calculate_risk_contributions(
        &self,
        weights: &[f64],
        cov_matrix: &[Vec<f64>],
    ) -> Result<Vec<f64>, String> {
        let portfolio_var = calculate_portfolio_variance(weights, cov_matrix)?;
        let portfolio_vol = portfolio_var.sqrt();

        if portfolio_vol < 1e-10 {
            return Ok(vec![0.0; weights.len()]);
        }

        let mut contributions = Vec::with_capacity(weights.len());

        for i in 0..weights.len() {
            // RC_i = w_i * (Σw)_i / portfolio_vol
            let marginal_contribution: f64 = (0..weights.len())
                .map(|j| cov_matrix[i][j] * weights[j])
                .sum();

            let risk_contribution = weights[i] * marginal_contribution / portfolio_vol;
            contributions.push(risk_contribution);
        }

        Ok(contributions)
    }
}

impl PortfolioObjective for RiskParityObjective {
    fn evaluate(
        &self,
        weights: &[f64],
        _returns: &[f64],
        cov_matrix: &[Vec<f64>],
    ) -> Result<f64, String> {
        let contributions = self.calculate_risk_contributions(weights, cov_matrix)?;

        if contributions.is_empty() {
            return Ok(0.0);
        }

        // Target: equal risk contribution = portfolio_vol / n
        let portfolio_var = calculate_portfolio_variance(weights, cov_matrix)?;
        let portfolio_vol = portfolio_var.sqrt();
        let target_contribution = portfolio_vol / weights.len() as f64;

        // Minimize sum of squared deviations from target
        let objective: f64 = contributions
            .iter()
            .map(|rc| (rc - target_contribution).powi(2))
            .sum();

        // Return negative because we want to minimize this
        Ok(-objective)
    }

    fn objective_type(&self) -> OptimizationObjective {
        OptimizationObjective::EqualRiskContribution
    }
}

/// Helper function to calculate portfolio variance
fn calculate_portfolio_variance(weights: &[f64], cov_matrix: &[Vec<f64>]) -> Result<f64, String> {
    if weights.is_empty() {
        return Err("Empty weights".to_string());
    }

    if cov_matrix.is_empty() {
        return Err("Empty covariance matrix".to_string());
    }

    let n = weights.len();

    if cov_matrix.len() != n {
        return Err(format!(
            "Covariance matrix size mismatch: {}x{} vs weights length {}",
            cov_matrix.len(),
            cov_matrix[0].len(),
            n
        ));
    }

    // Portfolio variance = w^T * Σ * w
    let mut variance = 0.0;
    for i in 0..n {
        for j in 0..n {
            if j >= cov_matrix[i].len() {
                return Err(format!(
                    "Covariance matrix row {} has insufficient columns",
                    i
                ));
            }
            variance += weights[i] * weights[j] * cov_matrix[i][j];
        }
    }

    Ok(variance)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> (Vec<f64>, Vec<f64>, Vec<Vec<f64>>) {
        // Two assets
        let weights = vec![0.5, 0.5];
        let returns = vec![0.10, 0.15]; // 10% and 15% annual returns
        let cov_matrix = vec![
            vec![0.04, 0.01], // Asset 1 variance = 0.04 (20% vol), covariance = 0.01
            vec![0.01, 0.09], // Asset 2 variance = 0.09 (30% vol)
        ];
        (weights, returns, cov_matrix)
    }

    #[test]
    fn test_sharpe_objective() {
        let (weights, returns, cov_matrix) = create_test_data();
        let objective = SharpeObjective::new(0.02);

        let result = objective.evaluate(&weights, &returns, &cov_matrix).unwrap();

        // Portfolio return = 0.5 * 0.10 + 0.5 * 0.15 = 0.125
        // Portfolio variance = w^T * Σ * w
        // = 0.5^2 * 0.04 + 2 * 0.5 * 0.5 * 0.01 + 0.5^2 * 0.09
        // = 0.25 * 0.04 + 0.5 * 0.01 + 0.25 * 0.09
        // = 0.01 + 0.005 + 0.0225 = 0.0375
        // Portfolio vol = sqrt(0.0375) ≈ 0.1936
        // Sharpe = (0.125 - 0.02) / 0.1936 ≈ 0.542

        assert!(result > 0.0);
        assert!(result < 1.0);
    }

    #[test]
    fn test_volatility_objective() {
        let (weights, returns, cov_matrix) = create_test_data();
        let objective = VolatilityObjective;

        let result = objective.evaluate(&weights, &returns, &cov_matrix).unwrap();

        // Should return negative volatility
        assert!(result < 0.0);
        assert!(result.abs() > 0.0 && result.abs() < 0.5);
    }

    #[test]
    fn test_return_objective() {
        let (weights, returns, _cov_matrix) = create_test_data();
        let objective = ReturnObjective;

        let result = objective.evaluate(&weights, &returns, &[]).unwrap();

        // Expected: 0.5 * 0.10 + 0.5 * 0.15 = 0.125
        assert!((result - 0.125).abs() < 0.001);
    }

    #[test]
    fn test_diversification_objective() {
        let (weights, returns, cov_matrix) = create_test_data();
        let objective = DiversificationObjective;

        let result = objective.evaluate(&weights, &returns, &cov_matrix).unwrap();

        // Diversification ratio should be > 1.0 when assets are not perfectly correlated
        assert!(result > 1.0);
    }

    #[test]
    fn test_risk_parity_objective() {
        let (weights, returns, cov_matrix) = create_test_data();
        let objective = RiskParityObjective;

        let result = objective.evaluate(&weights, &returns, &cov_matrix).unwrap();

        // Should be negative (we minimize variance of risk contributions)
        assert!(result <= 0.0);
    }

    #[test]
    fn test_portfolio_variance_calculation() {
        let weights = vec![0.5, 0.5];
        let cov_matrix = vec![vec![0.04, 0.01], vec![0.01, 0.09]];

        let variance = calculate_portfolio_variance(&weights, &cov_matrix).unwrap();

        // Expected: 0.25 * 0.04 + 0.5 * 0.01 + 0.25 * 0.09 = 0.0375
        assert!((variance - 0.0375).abs() < 0.0001);
    }

    #[test]
    fn test_empty_weights_error() {
        let cov_matrix = vec![vec![0.04]];
        let result = calculate_portfolio_variance(&[], &cov_matrix);
        assert!(result.is_err());
    }

    #[test]
    fn test_optimization_objective_descriptions() {
        let objectives = vec![
            OptimizationObjective::MaximizeSharpe,
            OptimizationObjective::MinimizeVolatility,
            OptimizationObjective::MaximizeReturn,
            OptimizationObjective::MaximizeDiversification,
            OptimizationObjective::EqualRiskContribution,
            OptimizationObjective::MinimumVariance,
        ];

        for obj in objectives {
            let desc = obj.description();
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn test_is_maximization() {
        assert!(OptimizationObjective::MaximizeSharpe.is_maximization());
        assert!(!OptimizationObjective::MinimizeVolatility.is_maximization());
        assert!(OptimizationObjective::MaximizeReturn.is_maximization());
        assert!(OptimizationObjective::MaximizeDiversification.is_maximization());
        assert!(OptimizationObjective::EqualRiskContribution.is_maximization());
        assert!(!OptimizationObjective::MinimumVariance.is_maximization());
    }
}
