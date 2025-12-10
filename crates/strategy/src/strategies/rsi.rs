//! RSI Mean Reversion Strategy
//!
//! A mean-reversion strategy that uses the Relative Strength Index (RSI)
//! to identify oversold and overbought conditions.

use crate::config::{RsiConfig, StrategyConfig};
use crate::indicators::{Indicator, Rsi};
use alphafield_core::{Bar, Signal, SignalType, Strategy};

/// RSI Mean Reversion Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: RSI falls below lower bound (oversold)
/// - **Sell Signal**: RSI rises above upper bound (overbought)
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::RsiStrategy;
/// use alphafield_strategy::config::RsiConfig;
///
/// let config = RsiConfig::new(14, 30.0, 70.0);
/// let strategy = RsiStrategy::from_config(config);
/// ```
pub struct RsiStrategy {
    config: RsiConfig,
    rsi: Rsi,
    position: SignalType, // Track current position to avoid spamming signals
    entry_price: Option<f64>,  // Track entry price for exit logic
}

impl RsiStrategy {
    /// Creates a new RSI strategy with specified parameters
    ///
    /// # Arguments
    /// * `period` - RSI calculation period
    /// * `lower_bound` - Oversold threshold
    /// * `upper_bound` - Overbought threshold
    pub fn new(period: usize, lower_bound: f64, upper_bound: f64) -> Self {
        let config = RsiConfig::new(period, lower_bound, upper_bound);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    ///
    /// # Panics
    /// Panics if the configuration is invalid
    pub fn from_config(config: RsiConfig) -> Self {
        config.validate().expect("Invalid RsiConfig");

        Self {
            rsi: Rsi::new(config.period),
            config,
            position: SignalType::Hold,
            entry_price: None,
        }
    }

    /// Returns the current configuration
    pub fn config(&self) -> &RsiConfig {
        &self.config
    }
}

impl Strategy for RsiStrategy {
    fn name(&self) -> &str {
        self.config.strategy_name()
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let rsi_val = self.rsi.update(bar.close)?;
        let price = bar.close;

        // EXIT LOGIC FIRST
        if self.position == SignalType::Buy {
            if let Some(entry) = self.entry_price {
                // Exit 1: RSI returns to neutral zone (40-60)
                if rsi_val >= 40.0 && rsi_val <= 60.0 {
                    self.position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("RSI Neutral Exit: {:.2}", rsi_val)),
                    }]);
                }
                
                // Exit 2: RSI overbought (take profit)
                if rsi_val > self.config.upper_bound {
                    self.position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("RSI Overbought Exit: {:.2}", rsi_val)),
                    }]);
                }
                
                // Exit 3: Take profit at 3%
                let profit_pct = (price - entry) / entry * 100.0;
                if profit_pct >= 3.0 {
                    self.position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("Take Profit: {:.1}%", profit_pct)),
                    }]);
                }
                
                // Exit 4: Stop loss at 5%
                if profit_pct <= -5.0 {
                    self.position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("Stop Loss: {:.1}%", profit_pct)),
                    }]);
                }
            }
        }

        // ENTRY LOGIC - only when not in position
        if rsi_val < self.config.lower_bound && self.position != SignalType::Buy {
            self.position = SignalType::Buy;
            self.entry_price = Some(price);
            return Some(vec![Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Buy,
                strength: (self.config.lower_bound - rsi_val) / self.config.lower_bound,
                metadata: Some(format!("RSI Oversold Entry: {:.2}", rsi_val)),
            }]);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_strategy_creation() {
        let strategy = RsiStrategy::new(14, 30.0, 70.0);
        assert_eq!(strategy.name(), "RSI Mean Reversion");
    }

    #[test]
    fn test_rsi_strategy_from_config() {
        let config = RsiConfig::new(10, 25.0, 75.0);
        let strategy = RsiStrategy::from_config(config);
        assert_eq!(strategy.config().period, 10);
        assert_eq!(strategy.config().lower_bound, 25.0);
        assert_eq!(strategy.config().upper_bound, 75.0);
    }

    #[test]
    #[should_panic(expected = "Invalid RsiConfig")]
    fn test_rsi_strategy_invalid_config() {
        let config = RsiConfig::new(14, 80.0, 70.0); // Invalid: lower > upper
        RsiStrategy::from_config(config);
    }
}
