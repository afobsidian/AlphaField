//! Momentum Strategy using EMA and MACD
//!
//! This strategy combines multiple indicators to identify strong momentum trends.

use crate::indicators::{Ema, Indicator, Macd};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for Momentum strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumConfig {
    /// EMA period for trend confirmation
    pub ema_period: usize,
    /// MACD fast period
    pub macd_fast: usize,
    /// MACD slow period
    pub macd_slow: usize,
    /// MACD signal period
    pub macd_signal: usize,
}

impl MomentumConfig {
    pub fn new(ema_period: usize, macd_fast: usize, macd_slow: usize, macd_signal: usize) -> Self {
        Self {
            ema_period,
            macd_fast,
            macd_slow,
            macd_signal,
        }
    }

    pub fn default_config() -> Self {
        Self {
            ema_period: 50,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.ema_period == 0 {
            return Err("EMA period must be greater than 0".to_string());
        }
        if self.macd_fast >= self.macd_slow {
            return Err("MACD fast period must be less than slow period".to_string());
        }
        if self.macd_signal == 0 {
            return Err("MACD signal period must be greater than 0".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for MomentumConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Momentum(ema={}, macd={}/{}/{})",
            self.ema_period, self.macd_fast, self.macd_slow, self.macd_signal
        )
    }
}

/// Momentum Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: Price > EMA AND MACD crosses above signal line
/// - **Sell Signal**: Price < EMA AND MACD crosses below signal line
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::MomentumStrategy;
///
/// let strategy = MomentumStrategy::new(50, 12, 26, 9);
/// ```
pub struct MomentumStrategy {
    config: MomentumConfig,
    ema: Ema,
    macd: Macd,
    last_position: SignalType,
}

impl MomentumStrategy {
    /// Creates a new Momentum strategy
    pub fn new(ema_period: usize, macd_fast: usize, macd_slow: usize, macd_signal: usize) -> Self {
        let config = MomentumConfig::new(ema_period, macd_fast, macd_slow, macd_signal);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: MomentumConfig) -> Self {
        config.validate().expect("Invalid MomentumConfig");

        Self {
            ema: Ema::new(config.ema_period),
            macd: Macd::new(config.macd_fast, config.macd_slow, config.macd_signal),
            config,
            last_position: SignalType::Hold,
        }
    }

    pub fn config(&self) -> &MomentumConfig {
        &self.config
    }
}

impl Strategy for MomentumStrategy {
    fn name(&self) -> &str {
        "EMA-MACD Momentum"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Signal> {
        let ema_val = self.ema.update(bar.close)?;
        let (macd_line, signal_line, _histogram) = self.macd.update(bar.close)?;

        let price = bar.close;

        // Bullish: Price above EMA and MACD crosses above signal
        if price > ema_val && macd_line > signal_line && self.last_position != SignalType::Buy {
            self.last_position = SignalType::Buy;
            return Some(Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Buy,
                strength: ((macd_line - signal_line) / macd_line.abs()).abs().min(1.0),
                metadata: Some(format!(
                    "Bullish Momentum: Price {:.2} > EMA {:.2}, MACD {:.4} > Signal {:.4}",
                    price, ema_val, macd_line, signal_line
                )),
            });
        }

        // Bearish: Price below EMA and MACD crosses below signal
        if price < ema_val && macd_line < signal_line && self.last_position != SignalType::Sell {
            self.last_position = SignalType::Sell;
            return Some(Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Sell,
                strength: ((signal_line - macd_line) / signal_line.abs())
                    .abs()
                    .min(1.0),
                metadata: Some(format!(
                    "Bearish Momentum: Price {:.2} < EMA {:.2}, MACD {:.4} < Signal {:.4}",
                    price, ema_val, macd_line, signal_line
                )),
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_momentum_creation() {
        let strategy = MomentumStrategy::new(50, 12, 26, 9);
        assert_eq!(strategy.name(), "EMA-MACD Momentum");
    }

    #[test]
    fn test_config_validation() {
        let config = MomentumConfig::new(50, 12, 26, 9);
        assert!(config.validate().is_ok());

        let invalid_config = MomentumConfig::new(50, 26, 12, 9); // fast > slow
        assert!(invalid_config.validate().is_err());
    }
}
