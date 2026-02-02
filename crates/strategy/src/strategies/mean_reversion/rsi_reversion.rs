//! RSI Mean Reversion Strategy
//!
//! This strategy uses the Relative Strength Index (RSI) to identify oversold/overbought
//! conditions and trade the reversion to the mean.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Indicator, Rsi, Sma};
use alphafield_core::{Bar, PositionState, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for RSI Reversion strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIReversionConfig {
    /// RSI period
    pub rsi_period: usize,
    /// Oversold threshold (entry)
    pub oversold_threshold: f64,
    /// Overbought threshold (exit)
    pub overbought_threshold: f64,
    /// Exit at neutral RSI (default 50)
    pub exit_threshold: f64,
    /// Use trend filter (don't trade against strong trends)
    pub trend_filter: bool,
    /// Trend filter period (200 SMA)
    pub trend_period: usize,
    /// Stop loss percentage
    pub stop_loss: f64,
}

impl RSIReversionConfig {
    pub fn new(rsi_period: usize, oversold: f64, overbought: f64) -> Self {
        Self {
            rsi_period,
            oversold_threshold: oversold,
            overbought_threshold: overbought,
            exit_threshold: 50.0,
            trend_filter: true,
            trend_period: 200,
            stop_loss: 5.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            rsi_period: 14,
            oversold_threshold: 30.0,
            overbought_threshold: 70.0,
            exit_threshold: 50.0,
            trend_filter: true,
            trend_period: 200,
            stop_loss: 5.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.rsi_period == 0 {
            return Err("RSI period must be greater than 0".to_string());
        }
        if self.oversold_threshold <= 0.0 || self.oversold_threshold >= 100.0 {
            return Err("Oversold threshold must be between 0 and 100".to_string());
        }
        if self.overbought_threshold <= 0.0 || self.overbought_threshold >= 100.0 {
            return Err("Overbought threshold must be between 0 and 100".to_string());
        }
        if self.exit_threshold <= self.oversold_threshold
            || self.exit_threshold >= self.overbought_threshold
        {
            return Err("Exit threshold must be between oversold and overbought".to_string());
        }
        if self.trend_period == 0 {
            return Err("Trend period must be greater than 0".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be greater than 0".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RsiState {
    Neutral,
    Overbought,
    Oversold,
}

impl fmt::Display for RSIReversionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RSIReversion(period={}, oversold={:.0}, overbought={:.0}, filter={})",
            self.rsi_period, self.oversold_threshold, self.overbought_threshold, self.trend_filter
        )
    }
}

/// RSI Mean Reversion Strategy
pub struct RSIReversionStrategy {
    config: RSIReversionConfig,
    rsi: Rsi,
    trend_sma: Option<Sma>,
    position: PositionState,
    long_entry_price: Option<f64>,
    short_entry_price: Option<f64>,
    rsi_state: RsiState,
}

impl RSIReversionStrategy {
    pub fn new(rsi_period: usize, oversold: f64, overbought: f64) -> Self {
        let config = RSIReversionConfig::new(rsi_period, oversold, overbought);
        Self::from_config(config)
    }

    pub fn from_config(config: RSIReversionConfig) -> Self {
        config.validate().expect("Invalid RSIReversionConfig");

        let trend_sma = if config.trend_filter {
            Some(Sma::new(config.trend_period))
        } else {
            None
        };

        Self {
            rsi: Rsi::new(config.rsi_period),
            trend_sma,
            config,
            position: PositionState::Flat,
            long_entry_price: None,
            short_entry_price: None,
            rsi_state: RsiState::Neutral,
        }
    }

    pub fn config(&self) -> &RSIReversionConfig {
        &self.config
    }
}

impl Default for RSIReversionStrategy {
    fn default() -> Self {
        // Default: 14-period RSI, 30/70 thresholds, 200-period SMA trend filter, 5% SL
        Self::from_config(RSIReversionConfig::default_config())
    }
}

impl MetadataStrategy for RSIReversionStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "RSI Mean Reversion".to_string(),
            category: StrategyCategory::MeanReversion,
            sub_type: Some("rsi_reversion".to_string()),
            description: format!(
                "RSI-based mean reversion strategy with period {}, oversold {:.0}, overbought {:.0}. \
                {} {:.0}-period SMA trend filter. \
                Long: Buys on oversold RSI (not in strong downtrend), sells on neutral/overbought. \
                Short: Sells on overbought RSI, buys on neutral/oversold.",
                self.config.rsi_period, self.config.oversold_threshold, self.config.overbought_threshold,
                if self.config.trend_filter { "Uses" } else { "No" }, self.config.trend_period
            ),
            hypothesis_path: "hypotheses/mean_reversion/rsi_reversion.md".to_string(),
            required_indicators: vec!["RSI".to_string(), "SMA".to_string()],
            expected_regimes: vec![MarketRegime::Sideways, MarketRegime::Ranging],
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

impl Strategy for RSIReversionStrategy {
    fn name(&self) -> &str {
        "RSI Mean Reversion"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let rsi_value = self.rsi.update(bar.close)?;
        let price = bar.close;

        // Update trend filter if enabled
        let trend_allowed = if let Some(ref mut sma) = self.trend_sma {
            if let Some(sma_value) = sma.update(price) {
                price > sma_value // Only trade if price above 200 SMA
            } else {
                true // Not enough data for trend filter - allow trading
            }
        } else {
            true // Trend filter disabled
        };

        // Update RSI state
        let current_state = if rsi_value >= self.config.overbought_threshold {
            RsiState::Overbought
        } else if rsi_value <= self.config.oversold_threshold {
            RsiState::Oversold
        } else {
            RsiState::Neutral
        };
        self.rsi_state = current_state;

        // === LONG POSITION EXIT LOGIC ===
        if self.position == PositionState::Long {
            if let Some(entry) = self.long_entry_price {
                let profit_pct = (price - entry) / entry * 100.0;

                // Long Exit 1: Stop loss
                if profit_pct <= -self.config.stop_loss {
                    self.position = PositionState::Flat;
                    self.long_entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("Long Stop Loss: {:.1}% loss", profit_pct)),
                    }]);
                }

                // Long Exit 2: RSI overbought
                if rsi_value >= self.config.overbought_threshold {
                    self.position = PositionState::Flat;
                    self.long_entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("RSI Overbought Long Exit: {:.1}", rsi_value)),
                    }]);
                }

                // Long Exit 3: RSI returns to neutral
                if rsi_value >= self.config.exit_threshold {
                    self.position = PositionState::Flat;
                    self.long_entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 0.8,
                        metadata: Some(format!("RSI Neutral Long Exit: {:.1}", rsi_value)),
                    }]);
                }
            }
        }

        // === SHORT POSITION EXIT LOGIC ===
        if self.position == PositionState::Short {
            if let Some(entry) = self.short_entry_price {
                // For shorts: profit when price drops
                let profit_pct = (entry - price) / entry * 100.0;

                // Short Exit 1: Stop loss
                if profit_pct <= -self.config.stop_loss {
                    self.position = PositionState::Flat;
                    self.short_entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: 1.0,
                        metadata: Some(format!("Short Stop Loss: {:.1}% loss", profit_pct)),
                    }]);
                }

                // Short Exit 2: RSI oversold
                if rsi_value <= self.config.oversold_threshold {
                    self.position = PositionState::Flat;
                    self.short_entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: 1.0,
                        metadata: Some(format!("RSI Oversold Short Exit: {:.1}", rsi_value)),
                    }]);
                }

                // Short Exit 3: RSI returns to neutral
                if rsi_value <= self.config.exit_threshold {
                    self.position = PositionState::Flat;
                    self.short_entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: 0.8,
                        metadata: Some(format!("RSI Neutral Short Exit: {:.1}", rsi_value)),
                    }]);
                }
            }
        }

        // === ENTRY LOGIC (only when flat) ===
        if self.position == PositionState::Flat {
            // Long Entry: RSI oversold and trend filter passed
            if rsi_value <= self.config.oversold_threshold && trend_allowed {
                self.position = PositionState::Long;
                self.long_entry_price = Some(price);
                let strength =
                    (self.config.oversold_threshold - rsi_value) / self.config.oversold_threshold;

                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Buy,
                    strength: strength.clamp(0.3, 1.0),
                    metadata: Some(format!(
                        "RSI Long Entry: Oversold {:.1} (trend filter: {})",
                        rsi_value, trend_allowed
                    )),
                }]);
            }

            // Short Entry: RSI overbought (no trend filter requirement for shorts)
            if rsi_value >= self.config.overbought_threshold {
                self.position = PositionState::Short;
                self.short_entry_price = Some(price);
                let strength = (rsi_value - self.config.overbought_threshold)
                    / (100.0 - self.config.overbought_threshold);

                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: strength.clamp(0.3, 1.0),
                    metadata: Some(format!("RSI Short Entry: Overbought {:.1}", rsi_value)),
                }]);
            }
        }

        None
    }

    fn reset(&mut self) {
        self.rsi.reset();
        if let Some(ref mut sma) = self.trend_sma {
            sma.reset();
        }
        self.position = PositionState::Flat;
        self.long_entry_price = None;
        self.short_entry_price = None;
        self.rsi_state = RsiState::Neutral;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_reversion_creation() {
        let strategy = RSIReversionStrategy::new(14, 30.0, 70.0);
        assert_eq!(strategy.name(), "RSI Mean Reversion");
    }

    #[test]
    fn test_config_validation() {
        let config = RSIReversionConfig::default_config();
        assert!(config.validate().is_ok());

        let invalid_config = RSIReversionConfig {
            rsi_period: 0,
            ..RSIReversionConfig::default_config()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_trend_filter_enabled() {
        let config = RSIReversionConfig::default_config();
        let strategy = RSIReversionStrategy::from_config(config);
        assert!(strategy.trend_sma.is_some());
    }

    #[test]
    fn test_trend_filter_disabled() {
        let mut config = RSIReversionConfig::default_config();
        config.trend_filter = false;
        let strategy = RSIReversionStrategy::from_config(config);
        assert!(strategy.trend_sma.is_none());
    }
}
