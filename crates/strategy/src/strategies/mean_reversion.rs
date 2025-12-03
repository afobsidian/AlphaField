//! Mean Reversion Strategy using Bollinger Bands
//!
//! This strategy uses Bollinger Bands to identify price extremes
//! and trade the reversion to the mean.

use crate::indicators::BollingerBands;
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for Mean Reversion strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeanReversionConfig {
    /// Period for Bollinger Bands calculation
    pub period: usize,
    /// Number of standard deviations
    pub num_std_dev: f64,
}

impl MeanReversionConfig {
    pub fn new(period: usize, num_std_dev: f64) -> Self {
        Self {
            period,
            num_std_dev,
        }
    }

    pub fn default_config() -> Self {
        Self {
            period: 20,
            num_std_dev: 2.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.period == 0 {
            return Err("Period must be greater than 0".to_string());
        }
        if self.num_std_dev <= 0.0 {
            return Err("Standard deviations must be positive".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for MeanReversionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MeanReversion(period={}, std_dev={:.1})",
            self.period, self.num_std_dev
        )
    }
}

/// Mean Reversion Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: Price touches or crosses below lower Bollinger Band
/// - **Sell Signal**: Price touches or crosses above upper Bollinger Band
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::MeanReversionStrategy;
///
/// let strategy = MeanReversionStrategy::new(20, 2.0);
/// ```
pub struct MeanReversionStrategy {
    config: MeanReversionConfig,
    bb: BollingerBands,
    last_position: SignalType,
}

impl MeanReversionStrategy {
    /// Creates a new Mean Reversion strategy
    ///
    /// # Arguments
    /// * `period` - Bollinger Bands period
    /// * `num_std_dev` - Number of standard deviations for bands
    pub fn new(period: usize, num_std_dev: f64) -> Self {
        let config = MeanReversionConfig::new(period, num_std_dev);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: MeanReversionConfig) -> Self {
        config.validate().expect("Invalid MeanReversionConfig");

        Self {
            bb: BollingerBands::new(config.period, config.num_std_dev),
            config,
            last_position: SignalType::Hold,
        }
    }

    pub fn config(&self) -> &MeanReversionConfig {
        &self.config
    }
}

impl Strategy for MeanReversionStrategy {
    fn name(&self) -> &str {
        "Bollinger Bands Mean Reversion"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Signal> {
        let (upper, middle, lower) = self.bb.update(bar.close)?;

        let price = bar.close;

        // Buy when price touches lower band (oversold)
        if price <= lower && self.last_position != SignalType::Buy {
            self.last_position = SignalType::Buy;
            let distance = (middle - price) / middle;
            return Some(Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Buy,
                strength: distance.min(1.0),
                metadata: Some(format!(
                    "BB Lower Band Touch: Price {:.2} <= Lower {:.2}",
                    price, lower
                )),
            });
        }

        // Sell when price touches upper band (overbought)
        if price >= upper && self.last_position != SignalType::Sell {
            self.last_position = SignalType::Sell;
            let distance = (price - middle) / middle;
            return Some(Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Sell,
                strength: distance.min(1.0),
                metadata: Some(format!(
                    "BB Upper Band Touch: Price {:.2} >= Upper {:.2}",
                    price, upper
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
    fn test_mean_reversion_creation() {
        let strategy = MeanReversionStrategy::new(20, 2.0);
        assert_eq!(strategy.name(), "Bollinger Bands Mean Reversion");
    }

    #[test]
    fn test_config_validation() {
        let config = MeanReversionConfig::new(20, 2.0);
        assert!(config.validate().is_ok());

        let invalid_config = MeanReversionConfig::new(0, 2.0);
        assert!(invalid_config.validate().is_err());
    }
}
