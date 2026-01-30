//! Portfolio Optimization Module
//!
//! Provides portfolio construction and optimization capabilities including
//! mean-variance optimization, risk parity, and other weight allocation strategies.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod algorithms;
mod constraints;
mod objectives;
mod optimizer;

pub use algorithms::inverse_volatility::{InverseVolatilityOptimizer, InverseVolatilityWeights};
pub use algorithms::mean_variance::{MeanVarianceOptimizer, MeanVarianceResult};
pub use algorithms::risk_parity::{RiskParityOptimizer, RiskParityResult};
pub use constraints::{PortfolioConstraint, WeightConstraint};
pub use objectives::{OptimizationObjective, PortfolioObjective};
pub use optimizer::{OptimizationConfig, PortfolioOptimizer, StrategyAllocation};

/// Represents a multi-strategy portfolio with allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiStrategyPortfolio {
    /// Portfolio name/identifier
    pub name: String,
    /// Strategy allocations (strategy_name -> weight)
    pub allocations: HashMap<String, f64>,
    /// Historical equity curves for each strategy
    pub equity_curves: HashMap<String, Vec<f64>>,
    /// Strategy metadata
    pub strategy_metadata: HashMap<String, StrategyMetadata>,
}

impl MultiStrategyPortfolio {
    /// Create a new empty portfolio
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            allocations: HashMap::new(),
            equity_curves: HashMap::new(),
            strategy_metadata: HashMap::new(),
        }
    }

    /// Add a strategy to the portfolio
    pub fn add_strategy(
        &mut self,
        name: impl Into<String>,
        equity_curve: Vec<f64>,
        metadata: StrategyMetadata,
    ) {
        let name = name.into();
        self.equity_curves.insert(name.clone(), equity_curve);
        self.strategy_metadata.insert(name, metadata);
    }

    /// Set allocation weight for a strategy
    pub fn set_allocation(&mut self, strategy_name: &str, weight: f64) {
        self.allocations.insert(strategy_name.to_string(), weight);
    }

    /// Get the combined equity curve based on current allocations
    pub fn combined_equity_curve(&self, initial_capital: f64) -> Vec<f64> {
        if self.allocations.is_empty() || self.equity_curves.is_empty() {
            return vec![initial_capital];
        }

        // Find the minimum length across all equity curves
        let min_len = self
            .equity_curves
            .values()
            .map(|v| v.len())
            .min()
            .unwrap_or(0);

        if min_len == 0 {
            return vec![initial_capital];
        }

        let mut combined = Vec::with_capacity(min_len);
        let mut current_equity = initial_capital;

        for i in 0..min_len {
            if i == 0 {
                combined.push(initial_capital);
                continue;
            }

            // Calculate weighted return across all strategies
            let mut weighted_return = 0.0;
            for (strategy_name, weight) in &self.allocations {
                if let Some(equity) = self.equity_curves.get(strategy_name) {
                    if i < equity.len() && i > 0 {
                        let prev = equity[i - 1];
                        let curr = equity[i];
                        if prev > 0.0 {
                            let strategy_return = (curr - prev) / prev;
                            weighted_return += strategy_return * weight;
                        }
                    }
                }
            }

            current_equity *= 1.0 + weighted_return;
            combined.push(current_equity);
        }

        combined
    }

    /// Validate that allocations sum to approximately 1.0
    pub fn validate_weights(&self, tolerance: f64) -> Result<(), String> {
        let total: f64 = self.allocations.values().sum();
        if (total - 1.0).abs() > tolerance {
            return Err(format!(
                "Portfolio weights must sum to 1.0, got {:.4}",
                total
            ));
        }

        // Check all weights are non-negative
        for (name, weight) in &self.allocations {
            if *weight < 0.0 {
                return Err(format!(
                    "Strategy '{}' has negative weight: {:.4}",
                    name, weight
                ));
            }
        }

        Ok(())
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize_weights(&mut self) {
        let total: f64 = self.allocations.values().sum();
        if total > 0.0 {
            for weight in self.allocations.values_mut() {
                *weight /= total;
            }
        }
    }
}

/// Metadata for a strategy in a portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetadata {
    /// Strategy name
    pub name: String,
    /// Asset class or sector
    pub asset_class: Option<String>,
    /// Trading timeframe
    pub timeframe: Option<String>,
    /// Expected return (annualized)
    pub expected_return: Option<f64>,
    /// Volatility (annualized)
    pub volatility: Option<f64>,
    /// Maximum drawdown
    pub max_drawdown: Option<f64>,
    /// Sharpe ratio
    pub sharpe_ratio: Option<f64>,
}

impl StrategyMetadata {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            asset_class: None,
            timeframe: None,
            expected_return: None,
            volatility: None,
            max_drawdown: None,
            sharpe_ratio: None,
        }
    }

    pub fn with_asset_class(mut self, asset_class: impl Into<String>) -> Self {
        self.asset_class = Some(asset_class.into());
        self
    }

    pub fn with_timeframe(mut self, timeframe: impl Into<String>) -> Self {
        self.timeframe = Some(timeframe.into());
        self
    }

    pub fn with_expected_return(mut self, ret: f64) -> Self {
        self.expected_return = Some(ret);
        self
    }

    pub fn with_volatility(mut self, vol: f64) -> Self {
        self.volatility = Some(vol);
        self
    }

    pub fn with_max_drawdown(mut self, dd: f64) -> Self {
        self.max_drawdown = Some(dd);
        self
    }

    pub fn with_sharpe_ratio(mut self, sharpe: f64) -> Self {
        self.sharpe_ratio = Some(sharpe);
        self
    }
}

/// Portfolio optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Optimal allocations
    pub allocations: HashMap<String, f64>,
    /// Expected portfolio return
    pub expected_return: f64,
    /// Expected portfolio volatility
    pub expected_volatility: f64,
    /// Expected Sharpe ratio
    pub expected_sharpe: f64,
    /// Diversification ratio
    pub diversification_ratio: f64,
    /// Optimization objective used
    pub objective: OptimizationObjective,
    /// Number of iterations performed
    pub iterations: usize,
    /// Whether optimization converged
    pub converged: bool,
    /// Optimization status message
    pub status_message: String,
}

impl Default for OptimizationResult {
    fn default() -> Self {
        Self {
            allocations: HashMap::new(),
            expected_return: 0.0,
            expected_volatility: 0.0,
            expected_sharpe: 0.0,
            diversification_ratio: 0.0,
            objective: OptimizationObjective::MaximizeSharpe,
            iterations: 0,
            converged: false,
            status_message: "Not optimized".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_creation() {
        let portfolio = MultiStrategyPortfolio::new("Test Portfolio");
        assert_eq!(portfolio.name, "Test Portfolio");
        assert!(portfolio.allocations.is_empty());
    }

    #[test]
    fn test_add_strategy() {
        let mut portfolio = MultiStrategyPortfolio::new("Test");
        let equity_curve = vec![100.0, 105.0, 110.0, 108.0, 115.0];
        let metadata = StrategyMetadata::new("Strategy A");

        portfolio.add_strategy("Strategy A", equity_curve, metadata);
        assert!(portfolio.equity_curves.contains_key("Strategy A"));
        assert!(portfolio.strategy_metadata.contains_key("Strategy A"));
    }

    #[test]
    fn test_set_allocation() {
        let mut portfolio = MultiStrategyPortfolio::new("Test");
        portfolio.set_allocation("Strategy A", 0.6);
        portfolio.set_allocation("Strategy B", 0.4);

        assert_eq!(portfolio.allocations.get("Strategy A"), Some(&0.6));
        assert_eq!(portfolio.allocations.get("Strategy B"), Some(&0.4));
    }

    #[test]
    fn test_validate_weights_pass() {
        let mut portfolio = MultiStrategyPortfolio::new("Test");
        portfolio.set_allocation("A", 0.5);
        portfolio.set_allocation("B", 0.5);

        assert!(portfolio.validate_weights(0.01).is_ok());
    }

    #[test]
    fn test_validate_weights_fail_sum() {
        let mut portfolio = MultiStrategyPortfolio::new("Test");
        portfolio.set_allocation("A", 0.6);
        portfolio.set_allocation("B", 0.3);

        assert!(portfolio.validate_weights(0.01).is_err());
    }

    #[test]
    fn test_validate_weights_fail_negative() {
        let mut portfolio = MultiStrategyPortfolio::new("Test");
        portfolio.set_allocation("A", 1.2);
        portfolio.set_allocation("B", -0.2);

        assert!(portfolio.validate_weights(0.01).is_err());
    }

    #[test]
    fn test_normalize_weights() {
        let mut portfolio = MultiStrategyPortfolio::new("Test");
        portfolio.set_allocation("A", 0.4);
        portfolio.set_allocation("B", 0.6);

        // Double all weights
        for weight in portfolio.allocations.values_mut() {
            *weight *= 2.0;
        }

        portfolio.normalize_weights();

        let total: f64 = portfolio.allocations.values().sum();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_combined_equity_curve() {
        let mut portfolio = MultiStrategyPortfolio::new("Test");
        
        // Strategy A: goes from 100 to 110 (10% return)
        let equity_a = vec![100.0, 102.0, 105.0, 108.0, 110.0];
        // Strategy B: goes from 100 to 105 (5% return)
        let equity_b = vec![100.0, 101.0, 103.0, 104.0, 105.0];

        portfolio.add_strategy("A", equity_a, StrategyMetadata::new("A"));
        portfolio.add_strategy("B", equity_b, StrategyMetadata::new("B"));
        portfolio.set_allocation("A", 0.5);
        portfolio.set_allocation("B", 0.5);

        let combined = portfolio.combined_equity_curve(10000.0);

        assert_eq!(combined.len(), 5);
        assert_eq!(combined[0], 10000.0);
        // Combined should be somewhere between the two individual curves
        assert!(combined[4] > 10000.0);
    }

    #[test]
    fn test_strategy_metadata_builder() {
        let metadata = StrategyMetadata::new("Test Strategy")
            .with_asset_class("Crypto")
            .with_timeframe("1H")
            .with_expected_return(0.25)
            .with_volatility(0.30)
            .with_sharpe_ratio(1.2);

        assert_eq!(metadata.name, "Test Strategy");
        assert_eq!(metadata.asset_class, Some("Crypto".to_string()));
        assert_eq!(metadata.timeframe, Some("1H".to_string()));
        assert_eq!(metadata.expected_return, Some(0.25));
        assert_eq!(metadata.volatility, Some(0.30));
        assert_eq!(metadata.sharpe_ratio, Some(1.2));
    }
}
