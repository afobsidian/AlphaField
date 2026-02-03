//! ATR Breakout Strategy
//!
//! This strategy uses Average True Range (ATR) to calculate dynamic breakout
//! levels that adapt to market volatility. Entry signals occur when price breaks
//! above a previous high plus a multiple of ATR, capturing volatility expansions.
//! The strategy exits when price breaks below a previous low minus ATR, or on
//! take profit / stop loss levels.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Atr, Indicator, Sma};
use alphafield_core::{Bar, PositionState, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Configuration for ATR Breakout strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ATRBreakoutConfig {
    /// Period for ATR calculation
    pub atr_period: usize,
    /// ATR multiplier for breakout calculation
    pub atr_multiplier: f64,
    /// Lookback period for high/low calculation
    pub lookback_period: usize,
    /// Optional: Fast MA for trend filter (set to 0 to disable)
    pub trend_ma_period: usize,
    /// Volume multiplier for entry confirmation
    pub volume_multiplier: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl ATRBreakoutConfig {
    pub fn new(atr_period: usize, atr_multiplier: f64, lookback_period: usize) -> Self {
        Self {
            atr_period,
            atr_multiplier,
            lookback_period,
            trend_ma_period: 50,
            volume_multiplier: 1.5,
            take_profit: 8.0,
            stop_loss: 4.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            atr_period: 14,
            atr_multiplier: 1.5,
            lookback_period: 20,
            trend_ma_period: 50,
            volume_multiplier: 1.5,
            take_profit: 8.0,
            stop_loss: 4.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.atr_period == 0 {
            return Err("ATR period must be greater than 0".to_string());
        }
        if self.atr_multiplier <= 0.0 {
            return Err("ATR multiplier must be positive".to_string());
        }
        if self.lookback_period == 0 {
            return Err("Lookback period must be greater than 0".to_string());
        }
        if self.volume_multiplier <= 0.0 {
            return Err("Volume multiplier must be positive".to_string());
        }
        if self.take_profit <= 0.0 {
            return Err("Take profit must be greater than 0".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be greater than 0".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for ATRBreakoutConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ATRBreakout(atr_period={}, multiplier={:.1}, lookback={}, trend_ma={}, tp={:.1}%, sl={:.1}%)",
            self.atr_period,
            self.atr_multiplier,
            self.lookback_period,
            self.trend_ma_period,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// ATR Breakout Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: Price breaks above (lookback high + ATR * multiplier) with volume confirmation
/// - **Sell Signal**: Price breaks below (lookback low - ATR * multiplier), take profit, or stop loss
/// - **Trend Filter**: Optional - only enter if price is above trend MA
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::volatility::ATRBreakoutStrategy;
///
/// let strategy = ATRBreakoutStrategy::new(14, 1.5, 20);
/// ```
pub struct ATRBreakoutStrategy {
    config: ATRBreakoutConfig,
    atr: Atr,
    trend_ma: Option<Sma>,
    price_history: VecDeque<f64>,
    volume_history: VecDeque<f64>,
    position: PositionState,
    long_entry_price: Option<f64>,
    short_entry_price: Option<f64>,
}

impl ATRBreakoutStrategy {
    /// Creates a new ATR Breakout strategy
    ///
    /// # Arguments
    /// * `atr_period` - ATR calculation period
    /// * `atr_multiplier` - Multiplier for ATR in breakout calculation
    /// * `lookback_period` - Period for high/low calculation
    pub fn new(atr_period: usize, atr_multiplier: f64, lookback_period: usize) -> Self {
        let config = ATRBreakoutConfig::new(atr_period, atr_multiplier, lookback_period);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: ATRBreakoutConfig) -> Self {
        config.validate().expect("Invalid ATRBreakoutConfig");

        let trend_ma = if config.trend_ma_period > 0 {
            Some(Sma::new(config.trend_ma_period))
        } else {
            None
        };

        Self {
            atr: Atr::new(config.atr_period),
            trend_ma,
            price_history: VecDeque::with_capacity(config.lookback_period),
            volume_history: VecDeque::with_capacity(20),
            config,
            position: PositionState::Flat,
            long_entry_price: None,
            short_entry_price: None,
        }
    }

    pub fn config(&self) -> &ATRBreakoutConfig {
        &self.config
    }

    /// Reset all position-related state
    fn reset_state(&mut self) {
        self.position = PositionState::Flat;
        self.long_entry_price = None;
        self.short_entry_price = None;
    }

    /// Calculate upper breakout level
    /// Get the highest price in the lookback period
    fn get_lookback_high(&self) -> Option<f64> {
        if self.price_history.len() >= self.config.lookback_period {
            Some(
                *self
                    .price_history
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            )
        } else {
            None
        }
    }

    /// Get the lowest price in the lookback period
    fn get_lookback_low(&self) -> Option<f64> {
        if self.price_history.len() >= self.config.lookback_period {
            Some(
                *self
                    .price_history
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            )
        } else {
            None
        }
    }

    /// Calculate average volume
    fn average_volume(&self) -> Option<f64> {
        if self.volume_history.is_empty() {
            return None;
        }
        let sum: f64 = self.volume_history.iter().sum();
        Some(sum / self.volume_history.len() as f64)
    }

    /// Check if volume confirms the signal
    fn check_volume_confirmation(&self, current_volume: f64) -> bool {
        if let Some(avg_vol) = self.average_volume() {
            return current_volume >= avg_vol * self.config.volume_multiplier;
        }
        true // Not enough volume history
    }

    /// Check if trend filter allows entry
    fn check_trend_filter(&self, price: f64) -> bool {
        if let Some(ref ma) = self.trend_ma {
            if let Some(ma_value) = ma.value() {
                return price > ma_value;
            }
        }
        true // No trend filter configured
    }
}

impl Default for ATRBreakoutStrategy {
    fn default() -> Self {
        // Default: 14-period ATR, 1.5x multiplier, 20-period lookback, 50-period trend MA, 1.5x volume multiplier, 8% TP, 4% SL
        Self::from_config(ATRBreakoutConfig::default_config())
    }
}

impl MetadataStrategy for ATRBreakoutStrategy {
    fn metadata(&self) -> StrategyMetadata {
        let mut required_indicators = vec!["ATR".to_string(), "Price".to_string()];
        if self.trend_ma.is_some() {
            required_indicators.push("SMA".to_string());
        }

        let trend_filter_desc = if self.trend_ma.is_some() {
            format!(
                " with {} period SMA trend filter",
                self.config.trend_ma_period
            )
        } else {
            String::new()
        };

        StrategyMetadata {
            name: "ATR Breakout".to_string(),
            category: StrategyCategory::VolatilityBased,
            sub_type: Some("atr_breakout".to_string()),
            description: format!(
                "ATR-based breakout strategy using {} period ATR with {:.1}x multiplier. \
                Enters when price breaks above ({}-bar high + ATR*{:.1}) or breaks below ({}-bar low - ATR*{:.1}). \
                Requires {:.1}x volume confirmation{}. \
                Uses {:.1}% TP and {:.1}% SL.",
                self.config.atr_period,
                self.config.atr_multiplier,
                self.config.lookback_period,
                self.config.atr_multiplier,
                self.config.lookback_period,
                self.config.atr_multiplier,
                self.config.volume_multiplier,
                trend_filter_desc,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/volatility/atr_breakout.md".to_string(),
            required_indicators,
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Trending, MarketRegime::Ranging, MarketRegime::Bear],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.25,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::VolatilityBased
    }
}

impl Strategy for ATRBreakoutStrategy {
    fn name(&self) -> &str {
        "ATR Breakout"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;

        // Update indicators
        let atr_value = self.atr.update(bar)?;
        if let Some(ref mut ma) = self.trend_ma {
            ma.update(price);
        }

        // Track price history
        self.price_history.push_back(price);
        if self.price_history.len() > self.config.lookback_period {
            self.price_history.pop_front();
        }

        // Track volume history
        self.volume_history.push_back(bar.volume);
        if self.volume_history.len() > 20 {
            self.volume_history.pop_front();
        }

        // ENTRY LOGIC (only when in Flat position)
        if self.position == PositionState::Flat {
            if let (Some(lookback_high), Some(lookback_low)) =
                (self.get_lookback_high(), self.get_lookback_low())
            {
                let volume_confirmed = self.check_volume_confirmation(bar.volume);
                let trend_ok = self.check_trend_filter(price);

                // Calculate breakout levels
                let buy_level = lookback_high + (atr_value * self.config.atr_multiplier);
                let sell_level = lookback_low - (atr_value * self.config.atr_multiplier);

                // === LONG ENTRY: Buy breakout ===
                if price > buy_level && volume_confirmed && trend_ok {
                    self.position = PositionState::Long;
                    self.long_entry_price = Some(price);

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: 1.0,
                        metadata: Some(format!(
                            "ATR Buy Breakout: Price {:.2} > Buy Level {:.2} (High {:.2} + ATR*{:.1}), Volume: {:.0}",
                            price, buy_level, lookback_high, self.config.atr_multiplier, bar.volume
                        )),
                    }]);
                }

                // === SHORT ENTRY: Sell breakout ===
                if price < sell_level && volume_confirmed && trend_ok {
                    self.position = PositionState::Short;
                    self.short_entry_price = Some(price);

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!(
                            "ATR Sell Breakout: Price {:.2} < Sell Level {:.2} (Low {:.2} - ATR*{:.1}), Volume: {:.0}",
                            price, sell_level, lookback_low, self.config.atr_multiplier, bar.volume
                        )),
                    }]);
                }
            }
        }

        // EXIT LOGIC (only when in position)
        if self.position != PositionState::Flat {
            let mut signals = Vec::new();

            // === LONG POSITION EXIT LOGIC ===
            if self.position == PositionState::Long {
                if let Some(entry) = self.long_entry_price {
                    let profit_pct = (price - entry) / entry * 100.0;

                    // Take Profit
                    if profit_pct >= self.config.take_profit {
                        self.reset_state();
                        signals.push(Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 1.0,
                            metadata: Some(format!("Take Profit: {:.1}% profit", profit_pct)),
                        });
                        return Some(signals);
                    }

                    // Stop Loss
                    if profit_pct <= -self.config.stop_loss {
                        self.reset_state();
                        signals.push(Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 1.0,
                            metadata: Some(format!("Stop Loss: {:.1}% loss", profit_pct)),
                        });
                        return Some(signals);
                    }
                }
            }
            // === SHORT POSITION EXIT LOGIC ===
            else if self.position == PositionState::Short {
                if let Some(entry) = self.short_entry_price {
                    let profit_pct = (entry - price) / entry * 100.0;

                    // Take Profit
                    if profit_pct >= self.config.take_profit {
                        self.reset_state();
                        signals.push(Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Buy,
                            strength: 1.0,
                            metadata: Some(format!("Take Profit: {:.1}% profit", profit_pct)),
                        });
                        return Some(signals);
                    }

                    // Stop Loss
                    if profit_pct <= -self.config.stop_loss {
                        self.reset_state();
                        signals.push(Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Buy,
                            strength: 1.0,
                            metadata: Some(format!("Stop Loss: {:.1}% loss", profit_pct)),
                        });
                        return Some(signals);
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[allow(dead_code)]
    fn create_test_bar(
        timestamp: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Bar {
        Bar {
            timestamp: Utc.timestamp_opt(timestamp, 0).unwrap(),
            open,
            high,
            low,
            close,
            volume,
        }
    }

    #[test]
    fn test_atr_breakout_creation() {
        let strategy = ATRBreakoutStrategy::new(14, 1.5, 20);
        assert_eq!(strategy.config().atr_period, 14);
        assert_eq!(strategy.config().atr_multiplier, 1.5);
        assert_eq!(strategy.config().lookback_period, 20);
    }

    #[test]
    fn test_atr_breakout_config_valid() {
        let config = ATRBreakoutConfig::new(14, 1.5, 20);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_atr_breakout_invalid_config() {
        let config = ATRBreakoutConfig {
            atr_period: 0,
            atr_multiplier: 1.5,
            lookback_period: 20,
            trend_ma_period: 50,
            volume_multiplier: 1.5,
            take_profit: 8.0,
            stop_loss: 4.0,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_atr_breakout_from_config() {
        let config = ATRBreakoutConfig::new(21, 2.0, 25);
        let strategy = ATRBreakoutStrategy::from_config(config);
        assert_eq!(strategy.config().atr_period, 21);
        assert_eq!(strategy.config().atr_multiplier, 2.0);
    }

    #[test]
    fn test_atr_breakout_metadata() {
        let strategy = ATRBreakoutStrategy::new(14, 1.5, 20);
        let metadata = strategy.metadata();
        assert_eq!(metadata.name, "ATR Breakout");
        assert_eq!(metadata.category, StrategyCategory::VolatilityBased);
    }

    #[test]
    fn test_get_lookback_high_low() {
        let mut strategy = ATRBreakoutStrategy::new(14, 1.5, 5);

        // Add prices: 100, 105, 102, 108, 95
        for price in &[100.0, 105.0, 102.0, 108.0, 95.0] {
            strategy.price_history.push_back(*price);
        }

        assert_eq!(strategy.get_lookback_high(), Some(108.0));
        assert_eq!(strategy.get_lookback_low(), Some(95.0));
    }

    #[test]
    fn test_check_volume_confirmation() {
        let mut strategy = ATRBreakoutStrategy::new(14, 1.5, 20);

        // Not enough history
        assert!(strategy.check_volume_confirmation(1000.0));

        // Build up volume history
        for _ in 0..21 {
            strategy.volume_history.push_back(1000.0);
        }

        // Volume above average (1.5x multiplier)
        assert!(strategy.check_volume_confirmation(1600.0));

        // Volume below average
        assert!(!strategy.check_volume_confirmation(1400.0));
    }

    #[test]
    fn test_check_trend_filter() {
        let mut strategy = ATRBreakoutStrategy::from_config(ATRBreakoutConfig {
            atr_period: 14,
            atr_multiplier: 1.5,
            lookback_period: 20,
            trend_ma_period: 50,
            volume_multiplier: 1.5,
            take_profit: 8.0,
            stop_loss: 4.0,
        });

        // No trend MA
        let strategy_no_trend = ATRBreakoutStrategy::from_config(ATRBreakoutConfig {
            atr_period: 14,
            atr_multiplier: 1.5,
            lookback_period: 20,
            trend_ma_period: 0,
            volume_multiplier: 1.5,
            take_profit: 8.0,
            stop_loss: 4.0,
        });

        assert!(strategy_no_trend.check_trend_filter(100.0));

        // Build up MA history
        for _ in 0..51 {
            strategy.trend_ma.as_mut().unwrap().update(100.0);
        }

        // Price above MA
        assert!(strategy.check_trend_filter(101.0));

        // Price below MA
        assert!(!strategy.check_trend_filter(99.0));
    }

    #[test]
    fn test_atr_breakout_new_instance_clean_state() {
        let strategy = ATRBreakoutStrategy::new(14, 1.5, 20);
        assert_eq!(strategy.position, PositionState::Flat);
        assert!(strategy.long_entry_price.is_none());
        assert!(strategy.short_entry_price.is_none());
    }
}
