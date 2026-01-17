//! Keltner Channel Mean Reversion Strategy
//!
//! This strategy uses Keltner channels (EMA + ATR) to identify price extremes
//! and trade the reversion, with volume confirmation.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Atr, Ema, Indicator};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Configuration for Keltner Channel Reversion strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeltnerReversionConfig {
    /// Period for EMA calculation (middle band)
    pub ema_period: usize,
    /// Period for ATR calculation
    pub atr_period: usize,
    /// ATR multiplier for upper/lower bands
    pub atr_multiplier: f64,
    /// Volume multiplier for entry confirmation
    pub volume_multiplier: f64,
    /// Stop loss percentage
    pub stop_loss: f64,
}

impl KeltnerReversionConfig {
    pub fn new(ema_period: usize, atr_period: usize, atr_multiplier: f64) -> Self {
        Self {
            ema_period,
            atr_period,
            atr_multiplier,
            volume_multiplier: 1.5,
            stop_loss: 3.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            ema_period: 20,
            atr_period: 10,
            atr_multiplier: 2.0,
            volume_multiplier: 1.5,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.ema_period == 0 {
            return Err("EMA period must be greater than 0".to_string());
        }
        if self.atr_period == 0 {
            return Err("ATR period must be greater than 0".to_string());
        }
        if self.atr_multiplier <= 0.0 {
            return Err("ATR multiplier must be positive".to_string());
        }
        if self.volume_multiplier <= 0.0 {
            return Err("Volume multiplier must be positive".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be greater than 0".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for KeltnerReversionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "KeltnerReversion(ema={}, atr={}, mult={:.1}, vol={:.1}x)",
            self.ema_period, self.atr_period, self.atr_multiplier, self.volume_multiplier
        )
    }
}

/// Keltner Channel Mean Reversion Strategy
pub struct KeltnerReversionStrategy {
    config: KeltnerReversionConfig,
    ema: Ema,
    atr: Atr,
    volumes: VecDeque<f64>,
    last_position: SignalType,
    entry_price: Option<f64>,
}

impl Default for KeltnerReversionStrategy {
    fn default() -> Self {
        // Default: 20-period EMA, 10-period ATR, 2.0x multiplier, 1.5x volume multiplier, 3% SL
        Self::from_config(KeltnerReversionConfig::default_config())
    }
}

impl KeltnerReversionStrategy {
    pub fn new(ema_period: usize, atr_period: usize, atr_multiplier: f64) -> Self {
        let config = KeltnerReversionConfig::new(ema_period, atr_period, atr_multiplier);
        Self::from_config(config)
    }

    pub fn from_config(config: KeltnerReversionConfig) -> Self {
        config.validate().expect("Invalid KeltnerReversionConfig");

        Self {
            ema: Ema::new(config.ema_period),
            atr: Atr::new(config.atr_period),
            volumes: VecDeque::with_capacity(config.ema_period),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
        }
    }

    pub fn config(&self) -> &KeltnerReversionConfig {
        &self.config
    }

    /// Calculate average volume
    fn average_volume(&self) -> Option<f64> {
        if self.volumes.is_empty() {
            return None;
        }
        let sum: f64 = self.volumes.iter().sum();
        Some(sum / self.volumes.len() as f64)
    }
}

impl MetadataStrategy for KeltnerReversionStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Keltner Channel Mean Reversion".to_string(),
            category: StrategyCategory::MeanReversion,
            sub_type: Some("keltner_reversion".to_string()),
            description: format!(
                "Keltner channel mean reversion strategy with EMA period {}, ATR period {}, multiplier {:.1}.
                Requires volume >= {:.1}x average for entry confirmation.
                Buys when price touches lower channel (EMA - {}*ATR) with high volume,
                exits at middle band (EMA) or upper channel.",
                self.config.ema_period, self.config.atr_period, self.config.atr_multiplier,
                self.config.volume_multiplier, self.config.atr_multiplier
            ),
            hypothesis_path: "hypotheses/mean_reversion/keltner_reversion.md".to_string(),
            required_indicators: vec!["EMA".to_string(), "ATR".to_string(), "Volume".to_string()],
            expected_regimes: vec![MarketRegime::Sideways, MarketRegime::Ranging],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.25,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::MeanReversion
    }
}

impl Strategy for KeltnerReversionStrategy {
    fn name(&self) -> &str {
        "Keltner Channel Mean Reversion"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let middle = self.ema.update(bar.close)?;
        let atr_value = self.atr.update(bar)?;
        let price = bar.close;

        // Track volume
        self.volumes.push_back(bar.volume);
        if self.volumes.len() > self.config.ema_period {
            self.volumes.pop_front();
        }

        let avg_volume = self.average_volume()?;

        // Calculate Keltner bands
        let upper_band = middle + (self.config.atr_multiplier * atr_value);
        let lower_band = middle - (self.config.atr_multiplier * atr_value);

        // Check volume confirmation
        let volume_confirmed = bar.volume >= self.config.volume_multiplier * avg_volume;

        // EXIT LOGIC
        if self.last_position == SignalType::Buy {
            if let Some(entry) = self.entry_price {
                let profit_pct = (price - entry) / entry * 100.0;

                // Exit condition 1: Stop loss
                if profit_pct <= -self.config.stop_loss {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("Stop Loss Exit: {:.1}% loss", profit_pct)),
                    }]);
                }

                // Exit condition 2: Price reaches middle band (EMA)
                if price >= middle {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Keltner Middle Exit: Price {:.2} >= EMA {:.2}",
                            price, middle
                        )),
                    }]);
                }

                // Exit condition 3: Price reaches upper band (profit target)
                if price >= upper_band {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Keltner Upper Exit: Price {:.2} >= Upper {:.2}",
                            price, upper_band
                        )),
                    }]);
                }
            }
        }

        // ENTRY LOGIC - Price touches lower band AND high volume
        if self.last_position != SignalType::Buy && price <= lower_band && volume_confirmed {
            self.last_position = SignalType::Buy;
            self.entry_price = Some(price);
            let distance = (middle - price) / middle;
            let strength = distance.clamp(0.3, 1.0);

            return Some(vec![Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Buy,
                strength,
                metadata: Some(format!(
                    "Keltner Lower Entry: Price {:.2} <= Lower {:.2}, Volume {:.0} ({:.1}x avg)",
                    price,
                    lower_band,
                    bar.volume,
                    bar.volume / avg_volume
                )),
            }]);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keltner_reversion_creation() {
        let strategy = KeltnerReversionStrategy::new(20, 10, 2.0);
        assert_eq!(strategy.name(), "Keltner Channel Mean Reversion");
    }

    #[test]
    fn test_config_validation() {
        let config = KeltnerReversionConfig::default_config();
        assert!(config.validate().is_ok());

        let invalid_config = KeltnerReversionConfig {
            ema_period: 0,
            ..KeltnerReversionConfig::default_config()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_average_volume() {
        let mut strategy = KeltnerReversionStrategy::new(20, 10, 2.0);
        strategy.volumes.push_back(1000.0);
        strategy.volumes.push_back(2000.0);
        strategy.volumes.push_back(3000.0);

        assert_eq!(strategy.average_volume(), Some(2000.0));
    }
}
