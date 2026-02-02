//! Kelly Criterion Position Sizing
//!
//! Implements the Kelly Criterion for optimal position sizing.
//! The Kelly formula maximizes expected logarithmic wealth.
//!
//! Formula: f* = (bp - q) / b
//! Where:
//! - f* = optimal fraction of capital to bet
//! - b = odds received on win (avg_win / |avg_loss|)
//! - p = probability of win
//! - q = probability of loss (1 - p)

use serde::{Deserialize, Serialize};

use super::{PositionSizing, TradeStatistics};

/// Kelly fraction variant
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum KellyFraction {
    /// Full Kelly (aggressive, can have large drawdowns)
    Full,
    /// Half Kelly (conservative, still good growth)
    #[default]
    Half,
    /// Quarter Kelly (very conservative)
    Quarter,
    /// Custom fraction (e.g., 0.3 for 30% of full Kelly)
    Custom(f64),
}

impl KellyFraction {
    /// Get the multiplier for this fraction
    pub fn multiplier(&self) -> f64 {
        match self {
            KellyFraction::Full => 1.0,
            KellyFraction::Half => 0.5,
            KellyFraction::Quarter => 0.25,
            KellyFraction::Custom(f) => f.clamp(0.01, 2.0),
        }
    }
}

/// Kelly Criterion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KellyResult {
    /// Full Kelly fraction (before applying fraction multiplier)
    pub full_kelly_fraction: f64,
    /// Applied Kelly fraction (after applying fraction multiplier)
    pub applied_fraction: f64,
    /// Recommended position size (units/contracts)
    pub position_size: f64,
    /// Fraction of capital to allocate
    pub capital_fraction: f64,
    /// Expected growth rate per trade
    pub expected_growth: f64,
    /// Probability of ruin (approximate)
    pub ruin_probability: f64,
    /// Kelly fraction type used
    pub fraction_type: KellyFraction,
    /// Win rate used in calculation
    pub win_rate: f64,
    /// Payoff ratio used in calculation
    pub payoff_ratio: f64,
}

/// Kelly Criterion Position Sizing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KellyCriterion {
    /// Kelly fraction to use (conservative variants available)
    pub fraction: KellyFraction,
    /// Maximum position size as fraction of capital
    pub max_capital_fraction: f64,
    /// Minimum edge required (Kelly > 0)
    pub require_edge: bool,
}

impl Default for KellyCriterion {
    fn default() -> Self {
        Self {
            fraction: KellyFraction::Half,
            max_capital_fraction: 0.5,
            require_edge: true,
        }
    }
}

impl KellyCriterion {
    /// Create a new Kelly Criterion sizer with default settings (Half Kelly)
    pub fn new() -> Self {
        Self::default()
    }

    /// Use full Kelly (aggressive)
    pub fn full() -> Self {
        Self {
            fraction: KellyFraction::Full,
            max_capital_fraction: 0.5,
            require_edge: true,
        }
    }

    /// Use half Kelly (conservative, recommended)
    pub fn half() -> Self {
        Self {
            fraction: KellyFraction::Half,
            max_capital_fraction: 0.5,
            require_edge: true,
        }
    }

    /// Use quarter Kelly (very conservative)
    pub fn quarter() -> Self {
        Self {
            fraction: KellyFraction::Quarter,
            max_capital_fraction: 0.5,
            require_edge: true,
        }
    }

    /// Set custom fraction
    pub fn with_fraction(mut self, fraction: KellyFraction) -> Self {
        self.fraction = fraction;
        self
    }

    /// Set maximum capital fraction
    pub fn with_max_capital_fraction(mut self, max: f64) -> Self {
        self.max_capital_fraction = max.clamp(0.01, 1.0);
        self
    }

    /// Set whether to require positive edge
    pub fn with_require_edge(mut self, require: bool) -> Self {
        self.require_edge = require;
        self
    }

    /// Calculate Kelly Criterion from trade statistics
    pub fn calculate_from_stats(
        &self,
        capital: f64,
        current_price: f64,
        stats: &TradeStatistics,
    ) -> Result<KellyResult, String> {
        stats.validate()?;
        self.calculate(
            capital,
            stats.win_rate,
            stats.avg_win_return,
            stats.avg_loss_return,
            current_price,
        )
    }

    /// Calculate Kelly Criterion position size
    ///
    /// # Arguments
    /// * `capital` - Available capital
    /// * `win_rate` - Probability of winning (p)
    /// * `avg_win` - Average win return (as decimal)
    /// * `avg_loss` - Average loss return (as decimal, negative)
    /// * `current_price` - Current market price per unit
    ///
    /// # Returns
    /// Kelly result with optimal position size
    pub fn calculate(
        &self,
        capital: f64,
        win_rate: f64,
        avg_win: f64,
        avg_loss: f64,
        current_price: f64,
    ) -> Result<KellyResult, String> {
        // Validate inputs
        if capital <= 0.0 {
            return Err("Capital must be positive".to_string());
        }

        if current_price <= 0.0 {
            return Err("Current price must be positive".to_string());
        }

        if win_rate <= 0.0 || win_rate >= 1.0 {
            return Err("Win rate must be between 0 and 1".to_string());
        }

        if avg_win <= 0.0 {
            return Err("Average win must be positive".to_string());
        }

        if avg_loss >= 0.0 {
            return Err("Average loss must be negative".to_string());
        }

        // Calculate payoff ratio (b)
        let payoff_ratio = avg_win / avg_loss.abs();

        // Calculate full Kelly fraction: f* = (bp - q) / b
        // Where q = 1 - p (probability of loss)
        let loss_rate = 1.0 - win_rate;
        let full_kelly = (payoff_ratio * win_rate - loss_rate) / payoff_ratio;

        // Apply Kelly fraction multiplier
        let kelly_multiplier = self.fraction.multiplier();
        let applied_kelly = full_kelly * kelly_multiplier;

        // Check edge requirement
        if self.require_edge && full_kelly <= 0.0 {
            return Err(format!(
                "No positive edge: full Kelly = {:.4}. Strategy has negative expected value.",
                full_kelly
            ));
        }

        // Apply maximum capital constraint
        let final_kelly =
            applied_kelly.clamp(-self.max_capital_fraction, self.max_capital_fraction);

        // Calculate position size
        let capital_allocation = capital * final_kelly;
        let position_size = capital_allocation / current_price;

        // Calculate expected growth rate
        // G(f) = p * ln(1 + b*f) + q * ln(1 - f)
        let growth_rate = if final_kelly > 0.0 {
            win_rate * (1.0 + payoff_ratio * final_kelly).ln()
                + loss_rate * (1.0 - final_kelly).ln()
        } else {
            0.0
        };

        // Approximate probability of ruin (simplified)
        // This is a rough approximation assuming continuous trading
        let ruin_prob = if final_kelly > 0.0 && growth_rate > 0.0 {
            (-2.0 * growth_rate).exp()
        } else {
            1.0
        };

        Ok(KellyResult {
            full_kelly_fraction: full_kelly,
            applied_fraction: final_kelly,
            position_size: position_size.max(0.0),
            capital_fraction: final_kelly.max(0.0),
            expected_growth: growth_rate,
            ruin_probability: ruin_prob.clamp(0.0, 1.0),
            fraction_type: self.fraction,
            win_rate,
            payoff_ratio,
        })
    }

    /// Calculate Kelly Criterion for multiple simultaneous bets
    ///
    /// This is useful when trading multiple uncorrelated strategies
    pub fn calculate_simultaneous(
        &self,
        opportunities: &[KellyOpportunity],
        total_capital: f64,
    ) -> Result<Vec<KellyResult>, String> {
        let mut results = Vec::new();

        for opp in opportunities {
            let result = self.calculate(
                total_capital,
                opp.win_rate,
                opp.avg_win,
                opp.avg_loss,
                opp.price,
            )?;
            results.push(result);
        }

        Ok(results)
    }
}

impl PositionSizing for KellyCriterion {
    fn calculate_position(
        &self,
        capital: f64,
        win_rate: f64,
        avg_win: f64,
        avg_loss: f64,
        current_price: f64,
    ) -> Result<f64, String> {
        let result = self.calculate(capital, win_rate, avg_win, avg_loss, current_price)?;
        Ok(result.position_size)
    }

    fn description(&self) -> &'static str {
        "Kelly Criterion optimal position sizing"
    }
}

/// Opportunity for Kelly calculation
#[derive(Debug, Clone)]
pub struct KellyOpportunity {
    /// Win rate for this opportunity
    pub win_rate: f64,
    /// Average win
    pub avg_win: f64,
    /// Average loss
    pub avg_loss: f64,
    /// Current price
    pub price: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kelly_calculation() {
        let kelly = KellyCriterion::half();

        // Classic example: 60% win rate, 2:1 payoff
        // Full Kelly = (2*0.6 - 0.4) / 2 = 0.4 (40%)
        // Half Kelly = 20%
        let result = kelly.calculate(10000.0, 0.6, 0.10, -0.05, 100.0).unwrap();

        assert!(result.full_kelly_fraction > 0.0);
        assert_eq!(result.fraction_type, KellyFraction::Half);
        assert!(result.applied_fraction > 0.0);
        assert!(result.position_size > 0.0);
    }

    #[test]
    fn test_kelly_classic_example() {
        let kelly = KellyCriterion::full();

        // Classic coin toss: 60% heads, win $2 on heads, lose $1 on tails
        // Full Kelly = (2*0.6 - 0.4) / 2 = 0.4
        let result = kelly.calculate(10000.0, 0.6, 2.0, -1.0, 1.0).unwrap();

        assert!((result.full_kelly_fraction - 0.4).abs() < 0.001);
        assert!((result.applied_fraction - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_kelly_no_edge() {
        let kelly = KellyCriterion::full().with_require_edge(true);

        // 50% win rate, 1:1 payoff = no edge
        let result = kelly.calculate(10000.0, 0.5, 0.10, -0.10, 100.0);

        assert!(result.is_err());
    }

    #[test]
    fn test_kelly_fractions() {
        let full = KellyCriterion::full();
        let half = KellyCriterion::half();
        let quarter = KellyCriterion::quarter();

        let win_rate = 0.6;
        let avg_win = 0.10;
        let avg_loss = -0.05;
        let capital = 10000.0;
        let price = 100.0;

        let r_full = full
            .calculate(capital, win_rate, avg_win, avg_loss, price)
            .unwrap();
        let r_half = half
            .calculate(capital, win_rate, avg_win, avg_loss, price)
            .unwrap();
        let r_quarter = quarter
            .calculate(capital, win_rate, avg_win, avg_loss, price)
            .unwrap();

        // Half Kelly should be approximately half of full Kelly
        assert!((r_half.applied_fraction - r_full.applied_fraction * 0.5).abs() < 0.01);
        assert!((r_quarter.applied_fraction - r_full.applied_fraction * 0.25).abs() < 0.01);
    }

    #[test]
    fn test_kelly_max_capital() {
        let kelly = KellyCriterion::full().with_max_capital_fraction(0.1);

        // Even with good edge, should be capped at 10%
        let result = kelly.calculate(10000.0, 0.7, 0.20, -0.05, 100.0).unwrap();

        assert!(result.applied_fraction <= 0.1);
    }

    #[test]
    fn test_kelly_from_trade_stats() {
        let returns = vec![
            0.05, -0.03, 0.04, -0.02, 0.06, -0.04, 0.03, -0.01, 0.05, -0.02, 0.04, -0.03, 0.05,
            -0.02, 0.03,
        ];

        let stats = TradeStatistics::from_returns(&returns).unwrap();
        let kelly = KellyCriterion::half();

        let result = kelly.calculate_from_stats(10000.0, 100.0, &stats).unwrap();

        assert!(result.position_size >= 0.0);
        assert!(result.win_rate > 0.0);
        assert!(result.payoff_ratio > 0.0);
    }

    #[test]
    fn test_kelly_expected_growth() {
        let kelly = KellyCriterion::full();

        // Good edge should have positive expected growth
        let result = kelly.calculate(10000.0, 0.6, 0.10, -0.05, 100.0).unwrap();

        assert!(result.expected_growth > 0.0);
    }

    #[test]
    fn test_invalid_capital() {
        let kelly = KellyCriterion::half();
        let result = kelly.calculate(0.0, 0.6, 0.10, -0.05, 100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_price() {
        let kelly = KellyCriterion::half();
        let result = kelly.calculate(10000.0, 0.6, 0.10, -0.05, 0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_kelly_fraction_multipliers() {
        assert_eq!(KellyFraction::Full.multiplier(), 1.0);
        assert_eq!(KellyFraction::Half.multiplier(), 0.5);
        assert_eq!(KellyFraction::Quarter.multiplier(), 0.25);
        assert_eq!(KellyFraction::Custom(0.3).multiplier(), 0.3);
    }
}
