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
/// - **Buy Signal**: RSI crosses below lower bound (oversold)
/// - **Sell Signal**: RSI crosses above upper bound (overbought) or TP/SL
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::RsiStrategy;
/// use alphafield_strategy::config::RsiConfig;
///
/// let config = RsiConfig::new(14, 30.0, 70.0, 3.0, 5.0);
/// let strategy = RsiStrategy::from_config(config);
/// ```
pub struct RsiStrategy {
    config: RsiConfig,
    rsi: Rsi,
    position: SignalType,     // Track current position to avoid spamming signals
    entry_price: Option<f64>, // Track entry price for exit logic
    last_rsi: Option<f64>,    // Track previous RSI for crossover detection
}

impl RsiStrategy {
    /// Creates a new RSI strategy with specified parameters
    ///
    /// # Arguments
    /// * `period` - RSI calculation period
    /// * `lower_bound` - Oversold threshold
    /// * `upper_bound` - Overbought threshold
    /// * `upper_bound` - Overbought threshold
    pub fn new(period: usize, lower_bound: f64, upper_bound: f64) -> Self {
        let config = RsiConfig::new(period, lower_bound, upper_bound, 3.0, 5.0);
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
            last_rsi: None,
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
        let prev_rsi = self.last_rsi;

        // Update state for next bar
        self.last_rsi = Some(rsi_val);

        // EXIT LOGIC FIRST (only when in position)
        if self.position == SignalType::Buy {
            if let Some(entry) = self.entry_price {
                let profit_pct = (price - entry) / entry * 100.0;

                // Exit 1: Take profit (price-based)
                if profit_pct >= self.config.take_profit {
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

                // Exit 2: Stop loss (price-based)
                if profit_pct <= -self.config.stop_loss {
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

                // Exit 3: RSI crosses above upper bound (overbought crossover)
                if let Some(prev) = prev_rsi {
                    if prev <= self.config.upper_bound && rsi_val > self.config.upper_bound {
                        self.position = SignalType::Hold;
                        self.entry_price = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 1.0,
                            metadata: Some(format!(
                                "RSI Overbought Crossover Exit: {:.2}",
                                rsi_val
                            )),
                        }]);
                    }
                }
            }
        }

        // ENTRY LOGIC - RSI crosses below lower bound (oversold crossover)
        if self.position != SignalType::Buy {
            if let Some(prev) = prev_rsi {
                if prev >= self.config.lower_bound && rsi_val < self.config.lower_bound {
                    self.position = SignalType::Buy;
                    self.entry_price = Some(price);
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: (self.config.lower_bound - rsi_val) / self.config.lower_bound,
                        metadata: Some(format!("RSI Oversold Crossover Entry: {:.2}", rsi_val)),
                    }]);
                }
            }
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
        let config = RsiConfig::new(10, 25.0, 75.0, 3.0, 5.0);
        let strategy = RsiStrategy::from_config(config);
        assert_eq!(strategy.config().period, 10);
        assert_eq!(strategy.config().lower_bound, 25.0);
        assert_eq!(strategy.config().upper_bound, 75.0);
    }

    #[test]
    #[should_panic(expected = "Invalid RsiConfig")]
    fn test_rsi_strategy_invalid_config() {
        let config = RsiConfig::new(14, 80.0, 70.0, 3.0, 5.0); // Invalid: lower > upper
        RsiStrategy::from_config(config);
    }
}
