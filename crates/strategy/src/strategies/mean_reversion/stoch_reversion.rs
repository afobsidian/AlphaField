//! Stochastic Oscillator Mean Reversion Strategy
//!
//! This strategy uses the Stochastic oscillator to identify oversold/overbought
//! conditions and trade the reversion.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::Stochastic;
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for Stochastic Reversion strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochReversionConfig {
    /// Period for %K calculation
    pub k_period: usize,
    /// Period for %D (SMA of %K) calculation
    pub d_period: usize,
    /// Smoothing period
    pub smooth_period: usize,
    /// Oversold threshold
    pub oversold: f64,
    /// Overbought threshold
    pub overbought: f64,
    /// Stop loss percentage
    pub stop_loss: f64,
}

impl StochReversionConfig {
    pub fn new(k_period: usize, d_period: usize) -> Self {
        Self {
            k_period,
            d_period,
            smooth_period: 3,
            oversold: 20.0,
            overbought: 80.0,
            stop_loss: 5.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            k_period: 14,
            d_period: 3,
            smooth_period: 3,
            oversold: 20.0,
            overbought: 80.0,
            stop_loss: 5.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.k_period == 0 {
            return Err("%K period must be greater than 0".to_string());
        }
        if self.d_period == 0 {
            return Err("%D period must be greater than 0".to_string());
        }
        if self.smooth_period == 0 {
            return Err("Smooth period must be greater than 0".to_string());
        }
        if self.oversold <= 0.0 || self.oversold >= 100.0 {
            return Err("Oversold threshold must be between 0 and 100".to_string());
        }
        if self.overbought <= 0.0 || self.overbought >= 100.0 {
            return Err("Overbought threshold must be between 0 and 100".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be greater than 0".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for StochReversionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StochReversion(k={}, d={}, smooth={}, oversold={:.0}, overbought={:.0})",
            self.k_period, self.d_period, self.smooth_period, self.oversold, self.overbought
        )
    }
}

/// Stochastic Mean Reversion Strategy
pub struct StochReversionStrategy {
    config: StochReversionConfig,
    stoch: Stochastic,
    last_position: SignalType,
    entry_price: Option<f64>,
    last_k: Option<f64>,
    last_d: Option<f64>,
}

impl StochReversionStrategy {
    pub fn new(k_period: usize, d_period: usize) -> Self {
        let config = StochReversionConfig::new(k_period, d_period);
        Self::from_config(config)
    }

    pub fn from_config(config: StochReversionConfig) -> Self {
        config.validate().expect("Invalid StochReversionConfig");

        Self {
            stoch: Stochastic::new(config.k_period, config.d_period, config.smooth_period),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            last_k: None,
            last_d: None,
        }
    }

    pub fn config(&self) -> &StochReversionConfig {
        &self.config
    }
}

impl MetadataStrategy for StochReversionStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Stochastic Mean Reversion".to_string(),
            category: StrategyCategory::MeanReversion,
            sub_type: Some("stochastic_reversion".to_string()),
            description: format!(
                "Stochastic oscillator mean reversion strategy with %K period {}, %D period {}, smooth period {}. 
                Oversold {:.0}, overbought {:.0}. Buys when %K drops below oversold, 
                sells when %K rises above overbought or crosses below %D.",
                self.config.k_period, self.config.d_period, self.config.smooth_period,
                self.config.oversold, self.config.overbought
            ),
            hypothesis_path: "hypotheses/mean_reversion/stoch_reversion.md".to_string(),
            required_indicators: vec!["Stochastic".to_string()],
            expected_regimes: vec![MarketRegime::Sideways, MarketRegime::Ranging, MarketRegime::LowVolatility],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.20,
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

impl Strategy for StochReversionStrategy {
    fn name(&self) -> &str {
        "Stochastic Mean Reversion"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let (k_value, d_value) = self.stoch.update(bar)?;
        let price = bar.close;

        // Save previous values for crossover detection
        let prev_k = self.last_k;
        let prev_d = self.last_d;

        self.last_k = Some(k_value);
        self.last_d = Some(d_value);

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

                // Exit condition 2: %K overbought
                if k_value >= self.config.overbought {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("Stoch Overbought Exit: %K = {:.1}", k_value)),
                    }]);
                }

                // Exit condition 3: %K crosses below %D (bearish crossover)
                if let (Some(pk), Some(pd)) = (prev_k, prev_d) {
                    if pk >= pd && k_value < d_value {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 0.9,
                            metadata: Some(format!(
                                "Stoch Bearish Crossover Exit: %K={:.1} crossed below %D={:.1}",
                                k_value, d_value
                            )),
                        }]);
                    }
                }
            }
        }

        // ENTRY LOGIC - %K oversold and optionally crosses above %D
        if self.last_position != SignalType::Buy && k_value <= self.config.oversold {
            // Optional: Confirm with bullish crossover (K crosses above D)
            let crossover_confirmed = if let (Some(pk), Some(pd)) = (prev_k, prev_d) {
                pk <= pd && k_value > d_value
            } else {
                false
            };

            if crossover_confirmed || k_value <= self.config.oversold {
                self.last_position = SignalType::Buy;
                self.entry_price = Some(price);
                let strength = (self.config.oversold - k_value) / self.config.oversold;
                let signal_strength = if crossover_confirmed { 1.0 } else { strength.min(0.9).max(0.3) };
                
                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Buy,
                    strength: signal_strength,
                    metadata: Some(format!(
                        "Stoch Oversold Entry: %K={:.1}, %D={:.1}, Crossover={}",
                        k_value, d_value, crossover_confirmed
                    )),
                }]);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_bar(high: f64, low: f64, close: f64) -> Bar {
        Bar {
            timestamp: Utc::now(),
            symbol: "BTC".to_string(),
            open: (high + low) / 2.0,
            high,
            low,
            close,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_stoch_reversion_creation() {
        let strategy = StochReversionStrategy::new(14, 3);
        assert_eq!(strategy.name(), "Stochastic Mean Reversion");
    }

    #[test]
    fn test_config_validation() {
        let config = StochReversionConfig::default_config();
        assert!(config.validate().is_ok());

        let invalid_config = StochReversionConfig {
            k_period: 0,
            ..StochReversionConfig::default_config()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_stochastic_parameters() {
        let config = StochReversionConfig::default_config();
        assert_eq!(config.k_period, 14);
        assert_eq!(config.d_period, 3);
        assert_eq!(config.smooth_period, 3);
        assert_eq!(config.oversold, 20.0);
        assert_eq!(config.overbought, 80.0);
    }
}
