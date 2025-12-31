//! Strategy configuration traits and utilities
//!
//! This module provides a common interface for strategy configuration,
//! enabling easy parameter management and serialization.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Trait that all strategy configurations must implement
///
/// This enables uniform handling of strategy parameters across different
/// strategy types, making it easier to serialize, display, and manage
/// configurations.
pub trait StrategyConfig: fmt::Debug + fmt::Display + Send + Sync {
    /// Returns the name of the strategy
    fn strategy_name(&self) -> &str;

    /// Validates the configuration parameters
    ///
    /// # Returns
    /// `Ok(())` if configuration is valid, `Err` with description otherwise
    fn validate(&self) -> Result<(), String>;

    /// Returns a JSON representation of the configuration
    fn to_json(&self) -> Result<String, String>;
}

/// Configuration for Golden Cross strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenCrossConfig {
    /// Fast moving average period
    pub fast_period: usize,
    /// Slow moving average period
    pub slow_period: usize,
    /// Take Profit percentage (e.g., 5.0 for 5%)
    pub take_profit: f64,
    /// Stop Loss percentage (e.g., 5.0 for 5%)
    pub stop_loss: f64,
}

impl GoldenCrossConfig {
    /// Creates a new Golden Cross configuration
    ///
    /// # Arguments
    /// * `fast_period` - Period for fast SMA (typically 10-50)
    /// * `slow_period` - Period for slow SMA (typically 50-200)
    ///
    /// # Example
    /// ```
    /// use alphafield_strategy::config::{GoldenCrossConfig, StrategyConfig};
    /// let config = GoldenCrossConfig::new(10, 30, 5.0, 5.0);
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn new(fast_period: usize, slow_period: usize, take_profit: f64, stop_loss: f64) -> Self {
        Self {
            fast_period,
            slow_period,
            take_profit,
            stop_loss,
        }
    }

    /// Creates a default configuration (50/200 - classic golden cross)
    pub fn default_config() -> Self {
        Self {
            fast_period: 50,
            slow_period: 200,
            take_profit: 5.0,
            stop_loss: 5.0,
        }
    }
}

impl StrategyConfig for GoldenCrossConfig {
    fn strategy_name(&self) -> &str {
        "Golden Cross"
    }

    fn validate(&self) -> Result<(), String> {
        if self.fast_period == 0 {
            return Err("Fast period must be greater than 0".to_string());
        }
        if self.slow_period == 0 {
            return Err("Slow period must be greater than 0".to_string());
        }
        if self.fast_period >= self.slow_period {
            return Err("Fast period must be less than slow period".to_string());
        }
        if self.take_profit <= 0.0 {
            return Err("Take profit must be greater than 0".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be greater than 0".to_string());
        }
        Ok(())
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Failed to serialize config: {}", e))
    }
}

impl fmt::Display for GoldenCrossConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GoldenCross(fast={}, slow={}, tp={:.1}%, sl={:.1}%)",
            self.fast_period, self.slow_period, self.take_profit, self.stop_loss
        )
    }
}

/// Configuration for RSI strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsiConfig {
    /// RSI calculation period
    pub period: usize,
    /// Lower threshold for oversold condition
    pub lower_bound: f64,
    /// Upper threshold for overbought condition
    pub upper_bound: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl RsiConfig {
    /// Creates a new RSI configuration
    ///
    /// # Arguments
    /// * `period` - RSI calculation period (typically 14)
    /// * `lower_bound` - Oversold threshold (typically 30)
    /// * `upper_bound` - Overbought threshold (typically 70)
    ///
    /// # Example
    /// ```
    /// use alphafield_strategy::config::{RsiConfig, StrategyConfig};
    /// let config = RsiConfig::new(14, 30.0, 70.0, 3.0, 5.0);
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn new(
        period: usize,
        lower_bound: f64,
        upper_bound: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            period,
            lower_bound,
            upper_bound,
            take_profit,
            stop_loss,
        }
    }

    /// Creates a default configuration (14, 30, 70)
    pub fn default_config() -> Self {
        Self {
            period: 14,
            lower_bound: 30.0,
            upper_bound: 70.0,
            take_profit: 3.0,
            stop_loss: 5.0,
        }
    }
}

impl StrategyConfig for RsiConfig {
    fn strategy_name(&self) -> &str {
        "RSI Mean Reversion"
    }

    fn validate(&self) -> Result<(), String> {
        if self.period == 0 {
            return Err("Period must be greater than 0".to_string());
        }
        if self.lower_bound <= 0.0 || self.lower_bound >= 100.0 {
            return Err("Lower bound must be between 0 and 100".to_string());
        }
        if self.upper_bound <= 0.0 || self.upper_bound >= 100.0 {
            return Err("Upper bound must be between 0 and 100".to_string());
        }
        if self.lower_bound >= self.upper_bound {
            return Err("Lower bound must be less than upper bound".to_string());
        }
        if self.take_profit <= 0.0 {
            return Err("Take profit must be greater than 0".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be greater than 0".to_string());
        }
        Ok(())
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Failed to serialize config: {}", e))
    }
}

impl fmt::Display for RsiConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RSI(period={}, lower={:.1}, upper={:.1}, tp={:.1}%, sl={:.1}%)",
            self.period, self.lower_bound, self.upper_bound, self.take_profit, self.stop_loss
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_cross_config_valid() {
        let config = GoldenCrossConfig::new(10, 30, 5.0, 5.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_golden_cross_config_invalid_order() {
        let config = GoldenCrossConfig::new(50, 20, 5.0, 5.0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_rsi_config_valid() {
        let config = RsiConfig::new(14, 30.0, 70.0, 3.0, 5.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_rsi_config_invalid_bounds() {
        let config = RsiConfig::new(14, 80.0, 70.0, 3.0, 5.0);
        assert!(config.validate().is_err());
    }
}
