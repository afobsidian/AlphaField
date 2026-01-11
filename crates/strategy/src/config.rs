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
    /// Minimum separation between MAs for valid signal (e.g., 1.0 for 1%)
    pub min_separation: f64,
    /// Enable RSI overbought filter
    pub rsi_filter_enabled: bool,
    /// RSI period (used if rsi_filter_enabled is true)
    pub rsi_period: Option<usize>,
    /// RSI threshold for overbought filter (entries blocked if RSI > threshold)
    pub rsi_threshold: Option<f64>,
    /// Enable ADX trending filter
    pub adx_filter_enabled: bool,
    /// ADX period (used if adx_filter_enabled is true)
    pub adx_period: Option<usize>,
    /// ADX threshold for trending filter (entries only if ADX >= threshold)
    pub adx_threshold: Option<f64>,
    /// ATR period for trailing stop calculation
    pub atr_period: Option<usize>,
    /// ATR multiplier for trailing stop (e.g., 2.0 means 2 * ATR)
    pub atr_multiplier: Option<f64>,
    /// Trailing stop percentage after take profit (e.g., 2.0 for 2%)
    pub trailing_stop: Option<f64>,
    /// Volume minimum multiplier (entries only if volume >= avg * multiplier)
    pub volume_min_multiplier: Option<f64>,
}

impl GoldenCrossConfig {
    /// Creates a new Golden Cross configuration
    ///
    /// # Arguments
    /// * `fast_period` - Period for fast SMA (typically 10-50)
    /// * `slow_period` - Period for slow SMA (typically 50-200)
    /// * `take_profit` - Take profit percentage
    /// * `stop_loss` - Stop loss percentage
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
            min_separation: 1.0,
            rsi_filter_enabled: false,
            rsi_period: None,
            rsi_threshold: None,
            adx_filter_enabled: false,
            adx_period: None,
            adx_threshold: None,
            atr_period: Some(14),
            atr_multiplier: Some(2.0),
            trailing_stop: Some(2.0),
            volume_min_multiplier: None,
        }
    }

    /// Creates a default configuration (50/200 - classic golden cross)
    pub fn default_config() -> Self {
        Self {
            fast_period: 50,
            slow_period: 200,
            take_profit: 5.0,
            stop_loss: 5.0,
            min_separation: 1.0,
            rsi_filter_enabled: false,
            rsi_period: None,
            rsi_threshold: None,
            adx_filter_enabled: false,
            adx_period: None,
            adx_threshold: None,
            atr_period: Some(14),
            atr_multiplier: Some(2.0),
            trailing_stop: Some(2.0),
            volume_min_multiplier: None,
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
        if self.min_separation < 0.0 {
            return Err("Min separation must be non-negative".to_string());
        }
        if self.rsi_filter_enabled {
            if self.rsi_period.is_none() || self.rsi_period == Some(0) {
                return Err("RSI period must be specified when RSI filter is enabled".to_string());
            }
            if self.rsi_threshold.is_none() {
                return Err(
                    "RSI threshold must be specified when RSI filter is enabled".to_string()
                );
            }
            let threshold = self.rsi_threshold.unwrap();
            if threshold <= 0.0 || threshold >= 100.0 {
                return Err("RSI threshold must be between 0 and 100".to_string());
            }
        }
        if self.adx_filter_enabled {
            if self.adx_period.is_none() || self.adx_period == Some(0) {
                return Err("ADX period must be specified when ADX filter is enabled".to_string());
            }
            if self.adx_threshold.is_none() {
                return Err(
                    "ADX threshold must be specified when ADX filter is enabled".to_string()
                );
            }
            let threshold = self.adx_threshold.unwrap();
            if threshold <= 0.0 {
                return Err("ADX threshold must be positive".to_string());
            }
        }
        if let Some(atr_period) = self.atr_period {
            if atr_period == 0 {
                return Err("ATR period must be greater than 0".to_string());
            }
        }
        if let Some(atr_multiplier) = self.atr_multiplier {
            if atr_multiplier <= 0.0 {
                return Err("ATR multiplier must be positive".to_string());
            }
        }
        if let Some(trailing_stop) = self.trailing_stop {
            if trailing_stop <= 0.0 {
                return Err("Trailing stop must be positive".to_string());
            }
        }
        if let Some(vol_mult) = self.volume_min_multiplier {
            if vol_mult <= 0.0 {
                return Err("Volume multiplier must be positive".to_string());
            }
        }
        Ok(())
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Failed to serialize config: {}", e))
    }
}

/// Configuration for Adaptive MA strategy (Kaufman's KAMA)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveMAConfig {
    /// Fast period for efficiency ratio calculation
    pub fast_period: usize,
    /// Slow period for efficiency ratio calculation
    pub slow_period: usize,
    /// Period for price change calculation
    pub price_period: usize,
    /// Take profit percentage
    pub take_profit: f64,
    /// Stop loss percentage
    pub stop_loss: f64,
    /// Trailing stop percentage after take profit
    pub trailing_stop: Option<f64>,
    /// Enable RSI filter for overbought/oversold conditions
    pub rsi_filter_enabled: bool,
    /// RSI period (used if rsi_filter_enabled is true)
    pub rsi_period: Option<usize>,
    /// RSI threshold for entry filter
    pub rsi_threshold: Option<f64>,
    /// Enable ADX filter for trend strength
    pub adx_filter_enabled: bool,
    /// ADX period (used if adx_filter_enabled is true)
    pub adx_period: Option<usize>,
    /// ADX threshold for trend strength
    pub adx_threshold: Option<f64>,
    /// ATR period for volatility calculation
    pub atr_period: usize,
    /// ATR multiplier for stop loss calculation
    pub atr_multiplier: Option<f64>,
    /// Volume confirmation multiplier
    pub volume_min_multiplier: Option<f64>,
}

impl AdaptiveMAConfig {
    /// Creates a new Adaptive MA configuration
    ///
    /// # Arguments
    /// * `fast_period` - Fast period for efficiency ratio
    /// * `slow_period` - Slow period for efficiency ratio
    /// * `price_period` - Period for price change calculation
    /// * `take_profit` - Take profit percentage
    /// * `stop_loss` - Stop loss percentage
    ///
    /// # Example
    /// ```
    /// use alphafield_strategy::config::{AdaptiveMAConfig, StrategyConfig};
    /// let config = AdaptiveMAConfig::new(10, 2, 30, 5.0, 3.0);
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        price_period: usize,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            fast_period,
            slow_period,
            price_period,
            take_profit,
            stop_loss,
            trailing_stop: Some(2.0),
            rsi_filter_enabled: false,
            rsi_period: None,
            rsi_threshold: None,
            adx_filter_enabled: false,
            adx_period: None,
            adx_threshold: None,
            atr_period: 14,
            atr_multiplier: Some(2.0),
            volume_min_multiplier: Some(1.2),
        }
    }

    /// Creates a default configuration
    pub fn default_config() -> Self {
        Self {
            fast_period: 10,
            slow_period: 30,
            price_period: 10,
            take_profit: 5.0,
            stop_loss: 3.0,
            trailing_stop: Some(2.0),
            rsi_filter_enabled: false,
            rsi_period: None,
            rsi_threshold: None,
            adx_filter_enabled: false,
            adx_period: None,
            adx_threshold: None,
            atr_period: 14,
            atr_multiplier: Some(2.0),
            volume_min_multiplier: Some(1.2),
        }
    }
}

impl fmt::Display for AdaptiveMAConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Adaptive MA(fast={}, slow={}, price={}, TP={:.1}%, SL={:.1}%)",
            self.fast_period, self.slow_period, self.price_period, self.take_profit, self.stop_loss
        )
    }
}

impl StrategyConfig for AdaptiveMAConfig {
    fn strategy_name(&self) -> &str {
        "Adaptive MA"
    }

    fn validate(&self) -> Result<(), String> {
        if self.fast_period == 0 {
            return Err("Fast period must be greater than 0".to_string());
        }
        if self.slow_period == 0 {
            return Err("Slow period must be greater than 0".to_string());
        }
        if self.price_period == 0 {
            return Err("Price period must be greater than 0".to_string());
        }
        if self.take_profit <= 0.0 {
            return Err("Take profit must be positive".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be positive".to_string());
        }
        if let Some(trailing) = self.trailing_stop {
            if trailing <= 0.0 {
                return Err("Trailing stop must be positive".to_string());
            }
        }
        if self.rsi_filter_enabled {
            if self.rsi_period.is_none() {
                return Err("RSI period must be specified when RSI filter is enabled".to_string());
            }
            if self.rsi_threshold.is_none() {
                return Err(
                    "RSI threshold must be specified when RSI filter is enabled".to_string()
                );
            }
        }
        if self.adx_filter_enabled {
            if self.adx_period.is_none() {
                return Err("ADX period must be specified when ADX filter is enabled".to_string());
            }
            if self.adx_threshold.is_none() {
                return Err(
                    "ADX threshold must be specified when ADX filter is enabled".to_string()
                );
            }
        }
        if self.atr_period == 0 {
            return Err("ATR period must be greater than 0".to_string());
        }
        if let Some(atr_mult) = self.atr_multiplier {
            if atr_mult <= 0.0 {
                return Err("ATR multiplier must be positive".to_string());
            }
        }
        if let Some(vol_mult) = self.volume_min_multiplier {
            if vol_mult <= 0.0 {
                return Err("Volume multiplier must be positive".to_string());
            }
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
            "GoldenCross(fast={}, slow={}, tp={:.1}%, sl={:.1}%, sep={:.1}%)",
            self.fast_period,
            self.slow_period,
            self.take_profit,
            self.stop_loss,
            self.min_separation
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

/// Configuration for Breakout strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakoutConfig {
    /// Lookback period for calculating high/low
    pub lookback_period: usize,
    /// Volume multiplier for confirmation (e.g., 1.5 means 1.5x average volume)
    pub volume_multiplier: f64,
    /// Minimum ATR as percentage of price
    pub min_atr_pct: f64,
    /// ATR multiplier for initial stop loss
    pub atr_stop_multiplier: f64,
    /// Trailing stop percentage after take profit
    pub trailing_stop_pct: f64,
    /// First take profit level (percentage)
    pub tp1_pct: f64,
    /// Percentage of position to close at TP1
    pub tp1_close_pct: f64,
    /// Second take profit level (percentage)
    pub tp2_pct: f64,
    /// Percentage of position to close at TP2
    pub tp2_close_pct: f64,
    /// Third take profit level (percentage)
    pub tp3_pct: f64,
    /// Maximum days to hold position
    pub max_days_in_position: usize,
    /// Enable RSI overbought filter
    pub rsi_filter_enabled: bool,
    /// RSI period (used if rsi_filter_enabled is true)
    pub rsi_period: Option<usize>,
    /// RSI threshold for overbought filter
    pub rsi_threshold: Option<f64>,
    /// Enable ADX trending filter
    pub adx_filter_enabled: bool,
    /// ADX period (used if adx_filter_enabled is true)
    pub adx_period: Option<usize>,
    /// ADX threshold for trending filter
    pub adx_threshold: Option<f64>,
    /// ATR period for volatility calculation
    pub atr_period: usize,
}

impl BreakoutConfig {
    /// Creates a new Breakout configuration
    ///
    /// # Arguments
    /// * `lookback_period` - Period for high/low calculation (typically 20)
    /// * `volume_multiplier` - Volume confirmation multiplier (typically 1.5)
    /// * `min_atr_pct` - Minimum ATR as % of price (typically 1.0)
    /// * `atr_stop_multiplier` - ATR multiplier for stop (typically 3.0)
    /// * `trailing_stop_pct` - Trailing stop % after TP (typically 2.5)
    ///
    /// # Example
    /// ```
    /// use alphafield_strategy::config::{BreakoutConfig, StrategyConfig};
    /// let config = BreakoutConfig::new(20, 1.5, 1.0, 3.0, 2.5);
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn new(
        lookback_period: usize,
        volume_multiplier: f64,
        min_atr_pct: f64,
        atr_stop_multiplier: f64,
        trailing_stop_pct: f64,
    ) -> Self {
        Self {
            lookback_period,
            volume_multiplier,
            min_atr_pct,
            atr_stop_multiplier,
            trailing_stop_pct,
            tp1_pct: 5.0,
            tp1_close_pct: 40.0,
            tp2_pct: 10.0,
            tp2_close_pct: 30.0,
            tp3_pct: 20.0,
            max_days_in_position: 40,
            rsi_filter_enabled: false,
            rsi_period: None,
            rsi_threshold: None,
            adx_filter_enabled: false,
            adx_period: None,
            adx_threshold: None,
            atr_period: 14,
        }
    }

    /// Creates a default configuration
    pub fn default_config() -> Self {
        Self {
            lookback_period: 20,
            volume_multiplier: 1.5,
            min_atr_pct: 1.0,
            atr_stop_multiplier: 3.0,
            trailing_stop_pct: 2.5,
            tp1_pct: 5.0,
            tp1_close_pct: 40.0,
            tp2_pct: 10.0,
            tp2_close_pct: 30.0,
            tp3_pct: 20.0,
            max_days_in_position: 40,
            rsi_filter_enabled: false,
            rsi_period: None,
            rsi_threshold: None,
            adx_filter_enabled: false,
            adx_period: None,
            adx_threshold: None,
            atr_period: 14,
        }
    }
}

impl StrategyConfig for BreakoutConfig {
    fn strategy_name(&self) -> &str {
        "Breakout"
    }

    fn validate(&self) -> Result<(), String> {
        if self.lookback_period == 0 {
            return Err("Lookback period must be greater than 0".to_string());
        }
        if self.lookback_period > 200 {
            return Err("Lookback period must be less than 200".to_string());
        }
        if self.volume_multiplier <= 0.0 {
            return Err("Volume multiplier must be positive".to_string());
        }
        if self.min_atr_pct < 0.0 {
            return Err("Minimum ATR percent must be non-negative".to_string());
        }
        if self.atr_stop_multiplier <= 0.0 {
            return Err("ATR stop multiplier must be positive".to_string());
        }
        if self.trailing_stop_pct <= 0.0 {
            return Err("Trailing stop percent must be positive".to_string());
        }
        if self.tp1_pct <= 0.0 {
            return Err("TP1 percent must be positive".to_string());
        }
        if self.tp1_close_pct <= 0.0 || self.tp1_close_pct > 100.0 {
            return Err("TP1 close percent must be between 0 and 100".to_string());
        }
        if self.tp2_pct <= self.tp1_pct {
            return Err("TP2 must be greater than TP1".to_string());
        }
        if self.tp2_close_pct <= 0.0 || self.tp2_close_pct > 100.0 {
            return Err("TP2 close percent must be between 0 and 100".to_string());
        }
        if self.tp3_pct <= self.tp2_pct {
            return Err("TP3 must be greater than TP2".to_string());
        }
        if self.max_days_in_position == 0 {
            return Err("Max days in position must be greater than 0".to_string());
        }
        if self.rsi_filter_enabled {
            if self.rsi_period.is_none() || self.rsi_period == Some(0) {
                return Err("RSI period must be specified when RSI filter is enabled".to_string());
            }
            if self.rsi_threshold.is_none() {
                return Err(
                    "RSI threshold must be specified when RSI filter is enabled".to_string()
                );
            }
            let threshold = self.rsi_threshold.unwrap();
            if threshold <= 0.0 || threshold >= 100.0 {
                return Err("RSI threshold must be between 0 and 100".to_string());
            }
        }
        if self.adx_filter_enabled {
            if self.adx_period.is_none() || self.adx_period == Some(0) {
                return Err("ADX period must be specified when ADX filter is enabled".to_string());
            }
            if self.adx_threshold.is_none() {
                return Err(
                    "ADX threshold must be specified when ADX filter is enabled".to_string()
                );
            }
            let threshold = self.adx_threshold.unwrap();
            if threshold <= 0.0 {
                return Err("ADX threshold must be positive".to_string());
            }
        }
        if self.atr_period == 0 {
            return Err("ATR period must be greater than 0".to_string());
        }
        Ok(())
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Failed to serialize config: {}", e))
    }
}

impl fmt::Display for BreakoutConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Breakout(lookback={}, vol_mult={:.1}, atr_stop_mult={:.1}, tp={:.1}/{:.1}/{:.1}%)",
            self.lookback_period,
            self.volume_multiplier,
            self.atr_stop_multiplier,
            self.tp1_pct,
            self.tp2_pct,
            self.tp3_pct
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

    #[test]
    fn test_breakout_config_valid() {
        let config = BreakoutConfig::new(20, 1.5, 1.0, 3.0, 2.5);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_breakout_config_invalid_lookback() {
        let config = BreakoutConfig::new(0, 1.5, 1.0, 3.0, 2.5);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_breakout_config_invalid_tp_order() {
        let mut config = BreakoutConfig::new(20, 1.5, 1.0, 3.0, 2.5);
        config.tp2_pct = 3.0; // Less than TP1
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_macrossover_config_valid() {
        let config = MACrossoverConfig::new(10, 30, 2.0, 5.0, 3.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_macrossover_config_invalid_periods() {
        let config = MACrossoverConfig::new(30, 10, 2.0, 5.0, 3.0); // Invalid: fast > slow
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_macrossover_config_with_filters() {
        let mut config = MACrossoverConfig::new(10, 30, 2.0, 5.0, 3.0);
        config.rsi_filter_enabled = true;
        config.rsi_period = Some(14);
        config.rsi_threshold = Some(70.0);
        config.adx_filter_enabled = true;
        config.adx_period = Some(14);
        config.adx_threshold = Some(25.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_adaptive_ma_config_valid() {
        let config = AdaptiveMAConfig::new(10, 2, 30, 5.0, 3.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_adaptive_ma_config_invalid_periods() {
        let config = AdaptiveMAConfig::new(0, 2, 30, 5.0, 3.0); // Invalid: fast_period = 0
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_triple_ma_config_valid() {
        let config = TripleMAConfig::new(5, 15, 30, 5.0, 3.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_triple_ma_config_invalid_order() {
        let config = TripleMAConfig::new(30, 15, 5, 5.0, 3.0); // Invalid: not in ascending order
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_macd_trend_config_valid() {
        let config = MacdTrendConfig::new(12, 26, 9, 5.0, 3.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_macd_trend_config_invalid_signal() {
        let config = MacdTrendConfig::new(12, 26, 0, 5.0, 3.0); // Invalid: signal_period = 0
        assert!(config.validate().is_err());
    }
}

/// Configuration for MACD Trend strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacdTrendConfig {
    /// Fast EMA period for MACD calculation
    pub fast_period: usize,
    /// Slow EMA period for MACD calculation
    pub slow_period: usize,
    /// Signal line period for MACD
    pub signal_period: usize,
    /// Take profit percentage
    pub take_profit: f64,
    /// Stop loss percentage
    pub stop_loss: f64,
    /// Trailing stop percentage after take profit
    pub trailing_stop: Option<f64>,
    /// Enable RSI filter for overbought/oversold conditions
    pub rsi_filter_enabled: bool,
    /// RSI period (used if rsi_filter_enabled is true)
    pub rsi_period: Option<usize>,
    /// RSI threshold for entry filter
    pub rsi_threshold: Option<f64>,
    /// Enable ADX filter for trend strength
    pub adx_filter_enabled: bool,
    /// ADX period (used if adx_filter_enabled is true)
    pub adx_period: Option<usize>,
    /// ADX threshold for trend strength
    pub adx_threshold: Option<f64>,
    /// ATR period for volatility calculation
    pub atr_period: usize,
    /// ATR multiplier for stop loss calculation
    pub atr_multiplier: Option<f64>,
    /// Volume confirmation multiplier
    pub volume_min_multiplier: Option<f64>,
    /// Enable histogram confirmation
    pub histogram_confirmation_enabled: bool,
    /// Minimum histogram value for confirmation
    pub min_histogram_value: Option<f64>,
}

impl MacdTrendConfig {
    /// Creates a new MACD Trend configuration
    ///
    /// # Arguments
    /// * `fast_period` - Fast EMA period for MACD
    /// * `slow_period` - Slow EMA period for MACD
    /// * `signal_period` - Signal line period for MACD
    /// * `take_profit` - Take profit percentage
    /// * `stop_loss` - Stop loss percentage
    ///
    /// # Example
    /// ```
    /// use alphafield_strategy::config::{MacdTrendConfig, StrategyConfig};
    /// let config = MacdTrendConfig::new(12, 26, 9, 5.0, 3.0);
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            fast_period,
            slow_period,
            signal_period,
            take_profit,
            stop_loss,
            trailing_stop: Some(2.0),
            rsi_filter_enabled: false,
            rsi_period: None,
            rsi_threshold: None,
            adx_filter_enabled: false,
            adx_period: None,
            adx_threshold: None,
            atr_period: 14,
            atr_multiplier: Some(2.0),
            volume_min_multiplier: Some(1.2),
            histogram_confirmation_enabled: true,
            min_histogram_value: Some(0.1),
        }
    }

    /// Creates a default configuration (classic MACD settings)
    pub fn default_config() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            take_profit: 5.0,
            stop_loss: 3.0,
            trailing_stop: Some(2.0),
            rsi_filter_enabled: false,
            rsi_period: None,
            rsi_threshold: None,
            adx_filter_enabled: false,
            adx_period: None,
            adx_threshold: None,
            atr_period: 14,
            atr_multiplier: Some(2.0),
            volume_min_multiplier: Some(1.2),
            histogram_confirmation_enabled: true,
            min_histogram_value: Some(0.1),
        }
    }
}

impl fmt::Display for MacdTrendConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MACD Trend(fast={}, slow={}, signal={}, TP={:.1}%, SL={:.1}%)",
            self.fast_period,
            self.slow_period,
            self.signal_period,
            self.take_profit,
            self.stop_loss
        )
    }
}

impl StrategyConfig for MacdTrendConfig {
    fn strategy_name(&self) -> &str {
        "MACD Trend"
    }

    fn validate(&self) -> Result<(), String> {
        if self.fast_period == 0 {
            return Err("Fast period must be greater than 0".to_string());
        }
        if self.slow_period == 0 {
            return Err("Slow period must be greater than 0".to_string());
        }
        if self.signal_period == 0 {
            return Err("Signal period must be greater than 0".to_string());
        }
        if self.fast_period >= self.slow_period {
            return Err("Fast period must be less than slow period".to_string());
        }
        if self.take_profit <= 0.0 {
            return Err("Take profit must be positive".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be positive".to_string());
        }
        if let Some(trailing) = self.trailing_stop {
            if trailing <= 0.0 {
                return Err("Trailing stop must be positive".to_string());
            }
        }
        if self.rsi_filter_enabled {
            if self.rsi_period.is_none() {
                return Err("RSI period must be specified when RSI filter is enabled".to_string());
            }
            if self.rsi_threshold.is_none() {
                return Err(
                    "RSI threshold must be specified when RSI filter is enabled".to_string()
                );
            }
        }
        if self.adx_filter_enabled {
            if self.adx_period.is_none() {
                return Err("ADX period must be specified when ADX filter is enabled".to_string());
            }
            if self.adx_threshold.is_none() {
                return Err(
                    "ADX threshold must be specified when ADX filter is enabled".to_string()
                );
            }
        }
        if self.atr_period == 0 {
            return Err("ATR period must be greater than 0".to_string());
        }
        if let Some(atr_mult) = self.atr_multiplier {
            if atr_mult <= 0.0 {
                return Err("ATR multiplier must be positive".to_string());
            }
        }
        if let Some(vol_mult) = self.volume_min_multiplier {
            if vol_mult <= 0.0 {
                return Err("Volume multiplier must be positive".to_string());
            }
        }
        if let Some(min_hist) = self.min_histogram_value {
            if min_hist < 0.0 {
                return Err("Minimum histogram value must be non-negative".to_string());
            }
        }
        Ok(())
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Failed to serialize config: {}", e))
    }
}

/// Configuration for Triple MA strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripleMAConfig {
    /// Fast moving average period
    pub fast_period: usize,
    /// Medium moving average period
    pub medium_period: usize,
    /// Slow moving average period
    pub slow_period: usize,
    /// Take profit percentage
    pub take_profit: f64,
    /// Stop loss percentage
    pub stop_loss: f64,
    /// Trailing stop percentage after take profit
    pub trailing_stop: Option<f64>,
    /// Enable RSI filter for overbought/oversold conditions
    pub rsi_filter_enabled: bool,
    /// RSI period (used if rsi_filter_enabled is true)
    pub rsi_period: Option<usize>,
    /// RSI threshold for entry filter
    pub rsi_threshold: Option<f64>,
    /// Enable ADX filter for trend strength
    pub adx_filter_enabled: bool,
    /// ADX period (used if adx_filter_enabled is true)
    pub adx_period: Option<usize>,
    /// ADX threshold for trend strength
    pub adx_threshold: Option<f64>,
    /// ATR period for volatility calculation
    pub atr_period: usize,
    /// ATR multiplier for stop loss calculation
    pub atr_multiplier: Option<f64>,
    /// Volume confirmation multiplier
    pub volume_min_multiplier: Option<f64>,
    /// MA type: "SMA" or "EMA"
    pub ma_type: String,
}

impl TripleMAConfig {
    /// Creates a new Triple MA configuration
    ///
    /// # Arguments
    /// * `fast_period` - Period for fast moving average
    /// * `medium_period` - Period for medium moving average
    /// * `slow_period` - Period for slow moving average
    /// * `take_profit` - Take profit percentage
    /// * `stop_loss` - Stop loss percentage
    ///
    /// # Example
    /// ```
    /// use alphafield_strategy::config::{TripleMAConfig, StrategyConfig};
    /// let config = TripleMAConfig::new(5, 15, 30, 5.0, 3.0);
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn new(
        fast_period: usize,
        medium_period: usize,
        slow_period: usize,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            fast_period,
            medium_period,
            slow_period,
            take_profit,
            stop_loss,
            trailing_stop: Some(2.0),
            rsi_filter_enabled: false,
            rsi_period: None,
            rsi_threshold: None,
            adx_filter_enabled: false,
            adx_period: None,
            adx_threshold: None,
            atr_period: 14,
            atr_multiplier: Some(2.0),
            volume_min_multiplier: Some(1.2),
            ma_type: "SMA".to_string(),
        }
    }

    /// Creates a default configuration
    pub fn default_config() -> Self {
        Self {
            fast_period: 5,
            medium_period: 15,
            slow_period: 30,
            take_profit: 5.0,
            stop_loss: 3.0,
            trailing_stop: Some(2.0),
            rsi_filter_enabled: false,
            rsi_period: None,
            rsi_threshold: None,
            adx_filter_enabled: false,
            adx_period: None,
            adx_threshold: None,
            atr_period: 14,
            atr_multiplier: Some(2.0),
            volume_min_multiplier: Some(1.2),
            ma_type: "SMA".to_string(),
        }
    }
}

impl fmt::Display for TripleMAConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Triple MA(fast={}, med={}, slow={}, TP={:.1}%, SL={:.1}%)",
            self.fast_period,
            self.medium_period,
            self.slow_period,
            self.take_profit,
            self.stop_loss
        )
    }
}

impl StrategyConfig for TripleMAConfig {
    fn strategy_name(&self) -> &str {
        "Triple MA"
    }

    fn validate(&self) -> Result<(), String> {
        if self.fast_period == 0 {
            return Err("Fast period must be greater than 0".to_string());
        }
        if self.medium_period == 0 {
            return Err("Medium period must be greater than 0".to_string());
        }
        if self.slow_period == 0 {
            return Err("Slow period must be greater than 0".to_string());
        }
        if self.fast_period >= self.medium_period {
            return Err("Fast period must be less than medium period".to_string());
        }
        if self.medium_period >= self.slow_period {
            return Err("Medium period must be less than slow period".to_string());
        }
        if self.take_profit <= 0.0 {
            return Err("Take profit must be positive".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be positive".to_string());
        }
        if let Some(trailing) = self.trailing_stop {
            if trailing <= 0.0 {
                return Err("Trailing stop must be positive".to_string());
            }
        }
        if self.rsi_filter_enabled {
            if self.rsi_period.is_none() {
                return Err("RSI period must be specified when RSI filter is enabled".to_string());
            }
            if self.rsi_threshold.is_none() {
                return Err(
                    "RSI threshold must be specified when RSI filter is enabled".to_string()
                );
            }
        }
        if self.adx_filter_enabled {
            if self.adx_period.is_none() {
                return Err("ADX period must be specified when ADX filter is enabled".to_string());
            }
            if self.adx_threshold.is_none() {
                return Err(
                    "ADX threshold must be specified when ADX filter is enabled".to_string()
                );
            }
        }
        if self.atr_period == 0 {
            return Err("ATR period must be greater than 0".to_string());
        }
        if let Some(atr_mult) = self.atr_multiplier {
            if atr_mult <= 0.0 {
                return Err("ATR multiplier must be positive".to_string());
            }
        }
        if let Some(vol_mult) = self.volume_min_multiplier {
            if vol_mult <= 0.0 {
                return Err("Volume multiplier must be positive".to_string());
            }
        }
        if self.ma_type != "SMA" && self.ma_type != "EMA" {
            return Err("MA type must be either 'SMA' or 'EMA'".to_string());
        }
        Ok(())
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Failed to serialize config: {}", e))
    }
}

/// Configuration for MA Crossover strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACrossoverConfig {
    /// Fast moving average period
    pub fast_period: usize,
    /// Slow moving average period
    pub slow_period: usize,
    /// Minimum separation percentage between MAs for entry
    pub min_separation: f64,
    /// Take profit percentage
    pub take_profit: f64,
    /// Stop loss percentage
    pub stop_loss: f64,
    /// Trailing stop percentage after take profit
    pub trailing_stop: Option<f64>,
    /// Enable RSI filter for overbought/oversold conditions
    pub rsi_filter_enabled: bool,
    /// RSI period (used if rsi_filter_enabled is true)
    pub rsi_period: Option<usize>,
    /// RSI threshold for entry filter
    pub rsi_threshold: Option<f64>,
    /// Enable ADX filter for trend strength
    pub adx_filter_enabled: bool,
    /// ADX period (used if adx_filter_enabled is true)
    pub adx_period: Option<usize>,
    /// ADX threshold for trend strength
    pub adx_threshold: Option<f64>,
    /// ATR period for volatility calculation
    pub atr_period: usize,
    /// ATR multiplier for stop loss calculation
    pub atr_multiplier: Option<f64>,
    /// Volume confirmation multiplier
    pub volume_min_multiplier: Option<f64>,
    /// MA type: "SMA" or "EMA"
    pub ma_type: String,
}

impl MACrossoverConfig {
    /// Creates a new MA Crossover configuration
    ///
    /// # Arguments
    /// * `fast_period` - Period for fast moving average
    /// * `slow_period` - Period for slow moving average
    /// * `min_separation` - Minimum separation percentage between MAs
    /// * `take_profit` - Take profit percentage
    /// * `stop_loss` - Stop loss percentage
    ///
    /// # Example
    /// ```
    /// use alphafield_strategy::config::{MACrossoverConfig, StrategyConfig};
    /// let config = MACrossoverConfig::new(10, 30, 2.0, 5.0, 3.0);
    /// assert!(config.validate().is_ok());
    /// ```
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        min_separation: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            fast_period,
            slow_period,
            min_separation,
            take_profit,
            stop_loss,
            trailing_stop: Some(2.0),
            rsi_filter_enabled: false,
            rsi_period: None,
            rsi_threshold: None,
            adx_filter_enabled: false,
            adx_period: None,
            adx_threshold: None,
            atr_period: 14,
            atr_multiplier: Some(2.0),
            volume_min_multiplier: Some(1.2),
            ma_type: "SMA".to_string(),
        }
    }

    /// Creates a default configuration
    pub fn default_config() -> Self {
        Self {
            fast_period: 10,
            slow_period: 30,
            min_separation: 2.0,
            take_profit: 5.0,
            stop_loss: 3.0,
            trailing_stop: Some(2.0),
            rsi_filter_enabled: false,
            rsi_period: None,
            rsi_threshold: None,
            adx_filter_enabled: false,
            adx_period: None,
            adx_threshold: None,
            atr_period: 14,
            atr_multiplier: Some(2.0),
            volume_min_multiplier: Some(1.2),
            ma_type: "SMA".to_string(),
        }
    }
}

impl fmt::Display for MACrossoverConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MA Crossover(fast={}, slow={}, sep={:.1}%, TP={:.1}%, SL={:.1}%)",
            self.fast_period,
            self.slow_period,
            self.min_separation,
            self.take_profit,
            self.stop_loss
        )
    }
}

impl StrategyConfig for MACrossoverConfig {
    fn strategy_name(&self) -> &str {
        "MA Crossover"
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
        if self.min_separation < 0.0 {
            return Err("Minimum separation must be non-negative".to_string());
        }
        if self.take_profit <= 0.0 {
            return Err("Take profit must be positive".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be positive".to_string());
        }
        if let Some(trailing) = self.trailing_stop {
            if trailing <= 0.0 {
                return Err("Trailing stop must be positive".to_string());
            }
        }
        if self.rsi_filter_enabled {
            if self.rsi_period.is_none() {
                return Err("RSI period must be specified when RSI filter is enabled".to_string());
            }
            if self.rsi_threshold.is_none() {
                return Err(
                    "RSI threshold must be specified when RSI filter is enabled".to_string()
                );
            }
        }
        if self.adx_filter_enabled {
            if self.adx_period.is_none() {
                return Err("ADX period must be specified when ADX filter is enabled".to_string());
            }
            if self.adx_threshold.is_none() {
                return Err(
                    "ADX threshold must be specified when ADX filter is enabled".to_string()
                );
            }
        }
        if self.atr_period == 0 {
            return Err("ATR period must be greater than 0".to_string());
        }
        if let Some(atr_mult) = self.atr_multiplier {
            if atr_mult <= 0.0 {
                return Err("ATR multiplier must be positive".to_string());
            }
        }
        if let Some(vol_mult) = self.volume_min_multiplier {
            if vol_mult <= 0.0 {
                return Err("Volume multiplier must be positive".to_string());
            }
        }
        if self.ma_type != "SMA" && self.ma_type != "EMA" {
            return Err("MA type must be either 'SMA' or 'EMA'".to_string());
        }
        Ok(())
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Failed to serialize config: {}", e))
    }
}
