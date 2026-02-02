//! Price Channel (Donchian Channel) Mean Reversion Strategy
//!
//! This strategy uses Donchian channels (highest high / lowest low over N periods)
//! to identify price extremes and trade the reversion.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use alphafield_core::{Bar, PositionState, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Configuration for Price Channel Reversion strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceChannelConfig {
    /// Lookback period for highest/lowest calculation
    pub lookback_period: usize,
    /// Exit at middle of channel (as percentage: 50 = midpoint)
    pub exit_percent: f64,
    /// Stop loss percentage
    pub stop_loss: f64,
}

impl PriceChannelConfig {
    pub fn new(lookback_period: usize) -> Self {
        Self {
            lookback_period,
            exit_percent: 50.0,
            stop_loss: 3.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            lookback_period: 20,
            exit_percent: 50.0,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.lookback_period == 0 {
            return Err("Lookback period must be greater than 0".to_string());
        }
        if self.exit_percent <= 0.0 || self.exit_percent > 100.0 {
            return Err("Exit percent must be between 0 and 100".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be greater than 0".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for PriceChannelConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PriceChannel(period={}, exit={:.0}%, sl={:.1}%)",
            self.lookback_period, self.exit_percent, self.stop_loss
        )
    }
}

/// Price Channel Mean Reversion Strategy
pub struct PriceChannelStrategy {
    config: PriceChannelConfig,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    position: PositionState,
    long_entry_price: Option<f64>,
    short_entry_price: Option<f64>,
}

impl Default for PriceChannelStrategy {
    fn default() -> Self {
        // Default: 20-period lookback, 50% exit, 3% SL
        Self::from_config(PriceChannelConfig::default_config())
    }
}

impl PriceChannelStrategy {
    pub fn new(lookback_period: usize) -> Self {
        let config = PriceChannelConfig::new(lookback_period);
        Self::from_config(config)
    }

    pub fn from_config(config: PriceChannelConfig) -> Self {
        config.validate().expect("Invalid PriceChannelConfig");

        Self {
            highs: VecDeque::with_capacity(config.lookback_period),
            lows: VecDeque::with_capacity(config.lookback_period),
            config,
            position: PositionState::Flat,
            long_entry_price: None,
            short_entry_price: None,
        }
    }

    pub fn config(&self) -> &PriceChannelConfig {
        &self.config
    }

    /// Calculate the highest high over the lookback period
    fn highest_high(&self) -> Option<f64> {
        if self.highs.is_empty() {
            return None;
        }
        self.highs
            .iter()
            .copied()
            .fold(None, |max, val| Some(max.map_or(val, |m: f64| m.max(val))))
    }

    /// Calculate the lowest low over the lookback period
    fn lowest_low(&self) -> Option<f64> {
        if self.lows.is_empty() {
            return None;
        }
        self.lows
            .iter()
            .copied()
            .fold(None, |min, val| Some(min.map_or(val, |m: f64| m.min(val))))
    }

    /// Calculate exit level based on exit_percent
    fn exit_level(&self) -> Option<f64> {
        let high = self.highest_high()?;
        let low = self.lowest_low()?;
        let range = high - low;
        Some(low + (range * self.config.exit_percent / 100.0))
    }
}

impl MetadataStrategy for PriceChannelStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Price Channel Mean Reversion".to_string(),
            category: StrategyCategory::MeanReversion,
            sub_type: Some("price_channel".to_string()),
            description: format!(
                "Donchian price channel mean reversion strategy with {}-period lookback. \
                Long: Buys when price touches lowest low, exits at {:.0}% of channel range. \
                Short: Sells when price touches highest high, exits at {:.0}% of channel range. \
                Uses {:.1}% stop loss.",
                self.config.lookback_period,
                self.config.exit_percent,
                self.config.exit_percent,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/mean_reversion/price_channel.md".to_string(),
            required_indicators: vec!["HighestHigh".to_string(), "LowestLow".to_string()],
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

impl Strategy for PriceChannelStrategy {
    fn name(&self) -> &str {
        "Price Channel Mean Reversion"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;

        // Add bar to window
        self.highs.push_back(bar.high);
        self.lows.push_back(bar.low);

        if self.highs.len() > self.config.lookback_period {
            self.highs.pop_front();
            self.lows.pop_front();
        }

        // Need full lookback period
        if self.highs.len() < self.config.lookback_period {
            return None;
        }

        let highest_high = self.highest_high()?;
        let lowest_low = self.lowest_low()?;
        let exit_level = self.exit_level()?;

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

                // Long Exit 2: Price reaches channel middle/exit level
                if price >= exit_level {
                    self.position = PositionState::Flat;
                    self.long_entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Channel Long Exit: Price {:.2} >= Exit Level {:.2}",
                            price, exit_level
                        )),
                    }]);
                }

                // Long Exit 3: Price reaches highest high (profit target)
                if price >= highest_high {
                    self.position = PositionState::Flat;
                    self.long_entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Upper Channel Long Exit: Price {:.2} >= Highest High {:.2}",
                            price, highest_high
                        )),
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

                // Short Exit 2: Price reaches channel middle/exit level
                // For shorts, we exit when price reaches the exit level (which is below entry for shorts)
                let channel_range = highest_high - lowest_low;
                let short_exit_level =
                    highest_high - (channel_range * self.config.exit_percent / 100.0);
                if price <= short_exit_level {
                    self.position = PositionState::Flat;
                    self.short_entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Channel Short Exit: Price {:.2} <= Exit Level {:.2}",
                            price, short_exit_level
                        )),
                    }]);
                }

                // Short Exit 3: Price reaches lowest low (profit target)
                if price <= lowest_low {
                    self.position = PositionState::Flat;
                    self.short_entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Lower Channel Short Exit: Price {:.2} <= Lowest Low {:.2}",
                            price, lowest_low
                        )),
                    }]);
                }
            }
        }

        // === ENTRY LOGIC (only when flat) ===
        if self.position == PositionState::Flat {
            // Long Entry: Price touches lowest low
            if price <= lowest_low {
                self.position = PositionState::Long;
                self.long_entry_price = Some(price);
                let channel_range = highest_high - lowest_low;
                let distance_from_low = (price - lowest_low) / channel_range;
                let strength = (1.0 - distance_from_low).clamp(0.3, 1.0);

                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Buy,
                    strength,
                    metadata: Some(format!(
                        "Lower Channel Long Entry: Price {:.2} <= Lowest Low {:.2}",
                        price, lowest_low
                    )),
                }]);
            }

            // Short Entry: Price touches highest high
            if price >= highest_high {
                self.position = PositionState::Short;
                self.short_entry_price = Some(price);
                let channel_range = highest_high - lowest_low;
                let distance_from_high = (highest_high - price) / channel_range;
                let strength = (1.0 - distance_from_high).clamp(0.3, 1.0);

                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength,
                    metadata: Some(format!(
                        "Upper Channel Short Entry: Price {:.2} >= Highest High {:.2}",
                        price, highest_high
                    )),
                }]);
            }
        }

        None
    }

    fn reset(&mut self) {
        self.highs.clear();
        self.lows.clear();
        self.position = PositionState::Flat;
        self.long_entry_price = None;
        self.short_entry_price = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_channel_creation() {
        let strategy = PriceChannelStrategy::new(20);
        assert_eq!(strategy.name(), "Price Channel Mean Reversion");
    }

    #[test]
    fn test_config_validation() {
        let config = PriceChannelConfig::default_config();
        assert!(config.validate().is_ok());

        let invalid_config = PriceChannelConfig {
            lookback_period: 0,
            ..PriceChannelConfig::default_config()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_highest_lowest() {
        let mut strategy = PriceChannelStrategy::new(3);
        strategy.highs.push_back(10.0);
        strategy.highs.push_back(20.0);
        strategy.highs.push_back(15.0);
        strategy.lows.push_back(5.0);
        strategy.lows.push_back(10.0);
        strategy.lows.push_back(7.0);

        assert_eq!(strategy.highest_high(), Some(20.0));
        assert_eq!(strategy.lowest_low(), Some(5.0));
    }

    #[test]
    fn test_exit_level() {
        let mut strategy = PriceChannelStrategy::new(3);
        strategy.highs.push_back(10.0);
        strategy.highs.push_back(20.0);
        strategy.highs.push_back(15.0);
        strategy.lows.push_back(0.0);
        strategy.lows.push_back(10.0);
        strategy.lows.push_back(5.0);

        // Highest = 20, Lowest = 0, Range = 20, Exit at 50% = 0 + 20*0.5 = 10
        assert_eq!(strategy.exit_level(), Some(10.0));
    }
}
