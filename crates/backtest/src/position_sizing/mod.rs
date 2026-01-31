//! Position Sizing Module
//!
//! Provides position sizing strategies including Kelly Criterion,
//! fractional Kelly variants, and volatility-adjusted sizing.

use serde::{Deserialize, Serialize};

pub mod kelly;
pub mod volatility_adjusted;

pub use kelly::{KellyCriterion, KellyFraction, KellyResult};
pub use volatility_adjusted::{VolatilityAdjustedSizing, VolatilitySizingConfig};

/// Base trait for position sizing strategies
pub trait PositionSizing {
    /// Calculate position size based on available capital and trade statistics
    ///
    /// # Arguments
    /// * `capital` - Available capital
    /// * `win_rate` - Historical win rate (0.0 to 1.0)
    /// * `avg_win` - Average winning trade (positive value)
    /// * `avg_loss` - Average losing trade (negative value)
    /// * `current_price` - Current market price
    ///
    /// # Returns
    /// Optimal position size (number of units/contracts)
    fn calculate_position(
        &self,
        capital: f64,
        win_rate: f64,
        avg_win: f64,
        avg_loss: f64,
        current_price: f64,
    ) -> Result<f64, String>;

    /// Get a description of the sizing method
    fn description(&self) -> &'static str;
}

/// Trade statistics used for position sizing calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeStatistics {
    /// Total number of trades
    pub total_trades: usize,
    /// Number of winning trades
    pub winning_trades: usize,
    /// Number of losing trades
    pub losing_trades: usize,
    /// Average winning trade return (as decimal, e.g., 0.05 for 5%)
    pub avg_win_return: f64,
    /// Average losing trade return (as decimal, e.g., -0.03 for -3%)
    pub avg_loss_return: f64,
    /// Maximum winning trade return
    pub max_win_return: f64,
    /// Maximum losing trade return (most negative)
    pub max_loss_return: f64,
    /// Win rate (0.0 to 1.0)
    pub win_rate: f64,
    /// Payoff ratio (avg_win / |avg_loss|)
    pub payoff_ratio: f64,
    /// Expectancy per trade
    pub expectancy: f64,
}

impl TradeStatistics {
    /// Calculate trade statistics from a series of trade returns
    ///
    /// # Arguments
    /// * `returns` - Vector of trade returns (positive for wins, negative for losses)
    pub fn from_returns(returns: &[f64]) -> Result<Self, String> {
        if returns.len() < 10 {
            return Err("Need at least 10 trades for reliable statistics".to_string());
        }

        let total_trades = returns.len();
        let winning_trades = returns.iter().filter(|&&r| r > 0.0).count();
        let losing_trades = returns.iter().filter(|&&r| r < 0.0).count();

        if winning_trades == 0 || losing_trades == 0 {
            return Err("Need both winning and losing trades".to_string());
        }

        let wins: Vec<f64> = returns.iter().filter(|&&r| r > 0.0).copied().collect();
        let losses: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).copied().collect();

        let avg_win_return = wins.iter().sum::<f64>() / wins.len() as f64;
        let avg_loss_return = losses.iter().sum::<f64>() / losses.len() as f64;

        let max_win_return = wins.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let max_loss_return = losses.iter().copied().fold(f64::INFINITY, f64::min);

        let win_rate = winning_trades as f64 / total_trades as f64;
        let payoff_ratio = if avg_loss_return.abs() > 1e-10 {
            avg_win_return / avg_loss_return.abs()
        } else {
            0.0
        };

        let expectancy = win_rate * avg_win_return + (1.0 - win_rate) * avg_loss_return;

        Ok(Self {
            total_trades,
            winning_trades,
            losing_trades,
            avg_win_return,
            avg_loss_return,
            max_win_return,
            max_loss_return,
            win_rate,
            payoff_ratio,
            expectancy,
        })
    }

    /// Validate that statistics are reasonable for position sizing
    pub fn validate(&self) -> Result<(), String> {
        if self.total_trades < 10 {
            return Err("Insufficient trade history (< 10 trades)".to_string());
        }

        if self.win_rate <= 0.0 || self.win_rate >= 1.0 {
            return Err("Invalid win rate".to_string());
        }

        if self.avg_win_return <= 0.0 {
            return Err("Average win must be positive".to_string());
        }

        if self.avg_loss_return >= 0.0 {
            return Err("Average loss must be negative".to_string());
        }

        Ok(())
    }
}

/// Position sizing result with risk metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizingResult {
    /// Recommended position size (units/contracts)
    pub position_size: f64,
    /// Position size as fraction of capital
    pub capital_fraction: f64,
    /// Dollar amount at risk
    pub dollar_risk: f64,
    /// Risk as percentage of capital
    pub risk_percentage: f64,
    /// Expected value of position
    pub expected_value: f64,
    /// Confidence interval (lower, upper) for position size
    pub confidence_interval: Option<(f64, f64)>,
}

/// Fixed fractional position sizing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedFractionalSizing {
    /// Fraction of capital to risk per trade (e.g., 0.02 for 2%)
    pub risk_fraction: f64,
    /// Maximum position size as fraction of capital
    pub max_position_fraction: f64,
}

impl Default for FixedFractionalSizing {
    fn default() -> Self {
        Self {
            risk_fraction: 0.02,
            max_position_fraction: 1.0,
        }
    }
}

impl FixedFractionalSizing {
    pub fn new(risk_fraction: f64) -> Self {
        Self {
            risk_fraction: risk_fraction.clamp(0.001, 0.5),
            max_position_fraction: 1.0,
        }
    }

    pub fn with_max_position_fraction(mut self, fraction: f64) -> Self {
        self.max_position_fraction = fraction.clamp(0.01, 10.0);
        self
    }
}

impl PositionSizing for FixedFractionalSizing {
    fn calculate_position(
        &self,
        capital: f64,
        _win_rate: f64,
        _avg_win: f64,
        avg_loss: f64,
        current_price: f64,
    ) -> Result<f64, String> {
        if capital <= 0.0 {
            return Err("Capital must be positive".to_string());
        }

        if current_price <= 0.0 {
            return Err("Current price must be positive".to_string());
        }

        if avg_loss >= 0.0 {
            return Err("Average loss must be negative for risk calculation".to_string());
        }

        // Calculate dollar risk amount
        let dollar_risk = capital * self.risk_fraction;

        // Calculate risk per unit (assuming avg_loss is a percentage)
        let risk_per_unit = current_price * avg_loss.abs();

        if risk_per_unit < 1e-10 {
            return Err("Risk per unit is too small".to_string());
        }

        // Calculate position size
        let position_size = dollar_risk / risk_per_unit;

        // Apply maximum position constraint
        let max_position = (capital * self.max_position_fraction) / current_price;
        let final_size = position_size.min(max_position);

        Ok(final_size)
    }

    fn description(&self) -> &'static str {
        "Fixed fractional position sizing (e.g., risk 2% per trade)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_statistics_from_returns() {
        let returns = vec![
            0.05, -0.03, 0.04, -0.02, 0.06, -0.04, 0.03, -0.01, 0.05, -0.02,
        ];

        let stats = TradeStatistics::from_returns(&returns).unwrap();

        assert_eq!(stats.total_trades, 10);
        assert_eq!(stats.winning_trades, 5);
        assert_eq!(stats.losing_trades, 5);
        assert!(stats.win_rate > 0.4 && stats.win_rate < 0.6);
        assert!(stats.avg_win_return > 0.0);
        assert!(stats.avg_loss_return < 0.0);
        assert!(stats.payoff_ratio > 1.0);
    }

    #[test]
    fn test_trade_statistics_validation() {
        let returns = vec![
            0.05, -0.03, 0.04, -0.02, 0.06, -0.04, 0.03, -0.01, 0.05, -0.02,
        ];

        let stats = TradeStatistics::from_returns(&returns).unwrap();
        assert!(stats.validate().is_ok());
    }

    #[test]
    fn test_trade_statistics_insufficient_trades() {
        let returns = vec![0.05, -0.03];
        let result = TradeStatistics::from_returns(&returns);
        assert!(result.is_err());
    }

    #[test]
    fn test_fixed_fractional_sizing() {
        let sizing = FixedFractionalSizing::new(0.02); // Risk 2% per trade

        // Capital: $10,000, avg loss: -3%, price: $100
        let position = sizing
            .calculate_position(10000.0, 0.6, 0.05, -0.03, 100.0)
            .unwrap();

        // Dollar risk = $10,000 * 0.02 = $200
        // Risk per unit = $100 * 0.03 = $3
        // Position size = $200 / $3 = 66.67 units
        let dollar_risk = 10000.0 * 0.02;
        let risk_per_unit = 100.0 * 0.03;
        let expected = dollar_risk / risk_per_unit;

        assert!((position - expected).abs() < 0.01);
    }

    #[test]
    fn test_fixed_fractional_max_position() {
        let sizing = FixedFractionalSizing::new(0.50) // Risk 50% per trade (extreme)
            .with_max_position_fraction(0.5); // But max 50% of capital

        let position = sizing
            .calculate_position(10000.0, 0.6, 0.05, -0.03, 100.0)
            .unwrap();

        // Max position = $10,000 * 0.5 / $100 = 50 units
        assert!(position <= 50.0);
    }

    #[test]
    fn test_fixed_fractional_invalid_capital() {
        let sizing = FixedFractionalSizing::new(0.02);
        let result = sizing.calculate_position(0.0, 0.6, 0.05, -0.03, 100.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_fixed_fractional_invalid_price() {
        let sizing = FixedFractionalSizing::new(0.02);
        let result = sizing.calculate_position(10000.0, 0.6, 0.05, -0.03, 0.0);
        assert!(result.is_err());
    }
}
