//! Breakout Strategy
//!
//! A trend-following strategy that generates buy signals when price breaks
//! above recent highs with sufficient volume confirmation, and sell signals
//! when price breaks below recent lows or hits stop loss/take profit levels.
//!
//! Enhanced with multiple filters: volume confirmation, ATR-based stops,
//! optional RSI overbought filter, and optional ADX trending filter.

use crate::config::{BreakoutConfig, StrategyConfig};
use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Adx, Atr, Indicator, Rsi, Sma};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use std::collections::VecDeque;

/// Breakout Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: Price breaks above lookback high with volume confirmation
/// - **Sell Signal**: Price breaks below lookback low, hits stop loss, or take profit
/// - **Filters**: Volume confirmation, ATR-based stops, optional RSI, optional ADX
///
/// # Example
/// ```
/// use alphafield_strategy::trend_following::BreakoutStrategy;
/// use alphafield_strategy::config::BreakoutConfig;
///
/// let config = BreakoutConfig::new(20, 1.5, 1.0, 3.0, 2.5);
/// let strategy = BreakoutStrategy::from_config(config);
/// ```
pub struct BreakoutStrategy {
    config: BreakoutConfig,
    volume_sma: Sma,
    atr: Atr,
    rsi: Option<Rsi>,
    adx: Option<Adx>,
    price_history: VecDeque<f64>,
    volume_history: VecDeque<f64>,
    entry_price: Option<f64>,
    entry_date: Option<chrono::DateTime<chrono::Utc>>,
    highest_since_entry: Option<f64>,
    lowest_since_entry: Option<f64>,
    breakout_level: Option<f64>,
    stop_loss_level: Option<f64>,
    trailing_stop_level: Option<f64>,
    position_size: f64,
    days_in_position: usize,
    tp1_hit: bool,
    tp2_hit: bool,
    tp3_hit: bool,
}

impl Default for BreakoutStrategy {
    fn default() -> Self {
        // Default: 20-period lookback, 1.5x volume multiplier, 1% min ATR, 3x ATR stop, 2.5% trailing stop
        Self::from_config(BreakoutConfig::default_config())
    }
}

impl BreakoutStrategy {
    /// Creates a new Breakout strategy with specified lookback period
    ///
    /// # Arguments
    /// * `lookback_period` - Period for high/low calculation
    pub fn new(lookback_period: usize) -> Self {
        let config = BreakoutConfig::new(
            lookback_period,
            1.5, // Default volume multiplier
            1.0, // Default min ATR %
            3.0, // Default ATR stop multiplier
            2.5, // Default trailing stop %
        );
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    ///
    /// # Panics
    /// Panics if the configuration is invalid
    pub fn from_config(config: BreakoutConfig) -> Self {
        config.validate().expect("Invalid BreakoutConfig");

        // Initialize optional indicators based on configuration
        let rsi = if config.rsi_filter_enabled {
            Some(Rsi::new(config.rsi_period.unwrap_or(14)))
        } else {
            None
        };

        let adx = if config.adx_filter_enabled {
            Some(Adx::new(config.adx_period.unwrap_or(14)))
        } else {
            None
        };

        let lookback_period = config.lookback_period;
        let atr_period = config.atr_period;

        Self {
            config,
            volume_sma: Sma::new(20), // 20-period volume SMA
            atr: Atr::new(atr_period),
            rsi,
            adx,
            price_history: VecDeque::with_capacity(lookback_period),
            volume_history: VecDeque::with_capacity(20),
            entry_price: None,
            entry_date: None,
            highest_since_entry: None,
            lowest_since_entry: None,
            breakout_level: None,
            stop_loss_level: None,
            trailing_stop_level: None,
            position_size: 0.0,
            days_in_position: 0,
            tp1_hit: false,
            tp2_hit: false,
            tp3_hit: false,
        }
    }

    /// Returns the current configuration
    pub fn config(&self) -> &BreakoutConfig {
        &self.config
    }

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

    /// Calculate average volume for filter
    fn avg_volume(&self) -> Option<f64> {
        if self.volume_history.is_empty() {
            return None;
        }
        Some(self.volume_history.iter().sum::<f64>() / self.volume_history.len() as f64)
    }

    /// Check if volume filter allows entry
    fn check_volume_filter(&self, current_volume: f64) -> bool {
        if let Some(avg_vol) = self.avg_volume() {
            return current_volume >= avg_vol * self.config.volume_multiplier;
        }
        true // Not enough history
    }

    /// Check if RSI filter allows entry
    fn check_rsi_filter(&self) -> bool {
        if let (Some(rsi), Some(threshold)) = (&self.rsi, self.config.rsi_threshold) {
            if let Some(current_rsi) = rsi.value() {
                return current_rsi < threshold; // Only enter if not overbought
            }
        }
        true // No filter or RSI not ready
    }

    /// Check if ADX filter allows entry (market is trending)
    fn check_adx_filter(&self) -> bool {
        if let (Some(adx), Some(threshold)) = (&self.adx, self.config.adx_threshold) {
            if let Some(current_adx) = adx.value() {
                return current_adx >= threshold; // Only enter if trending
            }
        }
        true // No filter or ADX not ready
    }

    /// Check if minimum ATR condition is met
    fn check_min_atr(&self, price: f64) -> bool {
        if let Some(atr_value) = self.atr.value() {
            let atr_pct = (atr_value / price) * 100.0;
            return atr_pct >= self.config.min_atr_pct;
        }
        true // ATR not ready yet
    }

    /// Calculate initial stop loss based on ATR
    fn calculate_initial_stop_loss(&self, price: f64) -> Option<f64> {
        self.atr
            .value()
            .map(|atr_value| price - (atr_value * self.config.atr_stop_multiplier))
    }

    /// Calculate trailing stop level
    fn calculate_trailing_stop(&self, price: f64) -> f64 {
        price * (1.0 - self.config.trailing_stop_pct / 100.0)
    }

    /// Check partial take profit conditions
    fn check_partial_exit(&mut self, price: f64, bar: &Bar) -> Option<Signal> {
        if let Some(entry) = self.entry_price {
            let profit_pct = (price - entry) / entry * 100.0;

            // TP1: Close tp1_close_pct at first take profit level
            if !self.tp1_hit && profit_pct >= self.config.tp1_pct {
                let exit_size = self.config.tp1_close_pct / 100.0;
                self.position_size -= exit_size;
                self.tp1_hit = true;

                // Set trailing stop after TP1
                if self.position_size > 0.0 {
                    self.trailing_stop_level = Some(self.calculate_trailing_stop(price));
                }

                return Some(Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: exit_size,
                    metadata: Some(format!(
                        "Take Profit 1: {:.1}% profit, closed {:.0}% of position",
                        profit_pct, self.config.tp1_close_pct
                    )),
                });
            }

            // TP2: Close tp2_close_pct at second take profit level
            if !self.tp2_hit && profit_pct >= self.config.tp2_pct {
                let exit_size = self.config.tp2_close_pct / 100.0;
                self.position_size -= exit_size;
                self.tp2_hit = true;

                return Some(Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: exit_size,
                    metadata: Some(format!(
                        "Take Profit 2: {:.1}% profit, closed {:.0}% of position",
                        profit_pct, self.config.tp2_close_pct
                    )),
                });
            }

            // TP3: Close remaining position at third take profit level
            if !self.tp3_hit && profit_pct >= self.config.tp3_pct {
                let exit_size = self.position_size; // Close remaining position
                self.position_size = 0.0;
                self.tp3_hit = true;

                return Some(Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: exit_size,
                    metadata: Some(format!(
                        "Take Profit 3: {:.1}% profit, closed remaining position",
                        profit_pct
                    )),
                });
            }
        }
        None
    }

    /// Reset strategy state
    fn reset_state(&mut self) {
        self.entry_price = None;
        self.entry_date = None;
        self.highest_since_entry = None;
        self.lowest_since_entry = None;
        self.breakout_level = None;
        self.stop_loss_level = None;
        self.trailing_stop_level = None;
        self.position_size = 0.0;
        self.days_in_position = 0;
        self.tp1_hit = false;
        self.tp2_hit = false;
        self.tp3_hit = false;
    }
}

impl MetadataStrategy for BreakoutStrategy {
    fn metadata(&self) -> StrategyMetadata {
        let mut required_indicators =
            vec!["ATR".to_string(), "Volume".to_string(), "Price".to_string()];
        if self.rsi.is_some() {
            required_indicators.push("RSI".to_string());
        }
        if self.adx.is_some() {
            required_indicators.push("ADX".to_string());
        }

        StrategyMetadata {
            name: self.config.strategy_name().to_string(),
            category: StrategyCategory::TrendFollowing,
            sub_type: Some("price_breakout".to_string()),
            description: format!(
                "Breakout strategy with {} period lookback, {:.1}x volume confirmation, {:.1}% min ATR, {:.1}x ATR stop. \n                Uses {:.1}%/{:.1}%/{:.1}% TP levels with {:.1}% trailing stop. Enhanced with {}RSI filter, {}ADX filter.\n                Generates buy signals on price breakouts above recent highs and sell signals on breakdowns or profit targets.",
                self.config.lookback_period,
                self.config.volume_multiplier,
                self.config.min_atr_pct,
                self.config.atr_stop_multiplier,
                self.config.tp1_pct,
                self.config.tp2_pct,
                self.config.tp3_pct,
                self.config.trailing_stop_pct,
                if self.rsi.is_some() { "" } else { "no " },
                if self.adx.is_some() { "" } else { "no " }
            ),
            hypothesis_path: "hypotheses/trend_following/breakout.md".to_string(),
            required_indicators,
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Trending],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.30,
                volatility_level: VolatilityLevel::High,
                correlation_sensitivity: CorrelationSensitivity::Medium,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::TrendFollowing
    }
}

impl Strategy for BreakoutStrategy {
    fn name(&self) -> &str {
        self.config.strategy_name()
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        // Update price and volume history
        self.price_history.push_back(bar.close);
        if self.price_history.len() > self.config.lookback_period {
            self.price_history.pop_front();
        }

        self.volume_history.push_back(bar.volume);
        if self.volume_history.len() > 20 {
            self.volume_history.pop_front();
        }

        // Update all indicators
        let _ = self.volume_sma.update(bar.volume);
        let _ = self.atr.update(bar);

        // Update optional indicators
        if let Some(rsi) = &mut self.rsi {
            rsi.update(bar.close);
        }
        if let Some(adx) = &mut self.adx {
            adx.update(bar);
        }

        let price = bar.close;

        // EXIT LOGIC FIRST (priority: partial TP > trailing stop > initial SL > max days)
        if let Some(entry) = self.entry_price {
            let mut signals = Vec::new();

            // Check partial take profit first
            if let Some(tp_signal) = self.check_partial_exit(price, bar) {
                signals.push(tp_signal);

                // If position closed completely, reset state and return
                if self.position_size <= 0.0 {
                    self.reset_state();
                    return Some(signals);
                }
            }

            // Update highest/lowest prices for trailing stop
            self.highest_since_entry = Some(self.highest_since_entry.unwrap_or(entry).max(price));
            self.lowest_since_entry = Some(self.lowest_since_entry.unwrap_or(entry).min(price));

            // Check trailing stop (if set)
            if let Some(trailing_stop) = self.trailing_stop_level {
                if price <= trailing_stop && self.position_size > 0.0 {
                    let remaining_size = self.position_size.max(0.1); // Minimum 10% signal
                    self.reset_state();
                    signals.push(Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: remaining_size,
                        metadata: Some(format!(
                            "Trailing Stop Triggered: Price {:.2} <= Stop {:.2}",
                            price, trailing_stop
                        )),
                    });
                    return Some(signals);
                }
            }

            // Check initial stop loss
            if let Some(stop_loss) = self.stop_loss_level {
                if price <= stop_loss && self.position_size > 0.0 {
                    let remaining_size = self.position_size.max(0.1); // Minimum 10% signal
                    self.reset_state();
                    signals.push(Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: remaining_size,
                        metadata: Some(format!(
                            "Stop Loss Triggered: Price {:.2} <= Stop {:.2}",
                            price, stop_loss
                        )),
                    });
                    return Some(signals);
                }
            }

            // Check maximum days in position
            self.days_in_position += 1;
            if self.days_in_position >= self.config.max_days_in_position && self.position_size > 0.0
            {
                let remaining_size = self.position_size.max(0.1); // Minimum 10% signal
                self.reset_state();
                signals.push(Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: remaining_size,
                    metadata: Some(format!(
                        "Max Days in Position: {} days reached",
                        self.config.max_days_in_position
                    )),
                });
                return Some(signals);
            }

            // Return partial exit signals if any
            if !signals.is_empty() {
                return Some(signals);
            }
            return None;
        }

        // ENTRY LOGIC
        if let Some(lookback_high) = self.get_lookback_high() {
            // Check for breakout above recent high
            if price > lookback_high {
                // Apply all filters
                if !self.check_volume_filter(bar.volume) {
                    return None; // Volume filter failed
                }

                if !self.check_rsi_filter() {
                    return None; // RSI filter failed (overbought)
                }

                if !self.check_adx_filter() {
                    return None; // ADX filter failed (not trending)
                }

                if !self.check_min_atr(price) {
                    return None; // Minimum ATR condition not met
                }

                // All filters passed - Breakout entry
                self.entry_price = Some(price);
                self.entry_date = Some(bar.timestamp);
                self.highest_since_entry = Some(price);
                self.lowest_since_entry = Some(price);
                self.breakout_level = Some(lookback_high);
                self.stop_loss_level = self.calculate_initial_stop_loss(price);
                self.position_size = 1.0; // Full position
                self.days_in_position = 0;

                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Buy,
                    strength: 1.0,
                    metadata: Some(format!(
                        "Breakout: Price {:.2} > Lookback High {:.2}, Volume {:.0}x avg, ATR {:.2}%",
                        price,
                        lookback_high,
                        self.config.volume_multiplier,
                        self.config.min_atr_pct
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

    fn create_test_bar(
        timestamp: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Bar {
        Bar {
            timestamp: chrono::DateTime::from_timestamp(timestamp, 0).unwrap(),
            open,
            high,
            low,
            close,
            volume,
        }
    }

    #[test]
    fn test_breakout_creation() {
        let strategy = BreakoutStrategy::new(20);
        assert_eq!(strategy.name(), "Breakout");
    }

    #[test]
    fn test_breakout_config_valid() {
        let config = BreakoutConfig::new(20, 1.5, 1.0, 3.0, 2.5);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_breakout_from_config() {
        let config = BreakoutConfig::new(10, 2.0, 1.5, 2.5, 3.0);
        let strategy = BreakoutStrategy::from_config(config);
        assert_eq!(strategy.config().lookback_period, 10);
        assert_eq!(strategy.config().volume_multiplier, 2.0);
    }

    #[test]
    fn test_breakout_basic_functionality() {
        let mut strategy = BreakoutStrategy::new(5);
        let base_time = Utc::now().timestamp();

        // Feed flat price data to establish range (need 20+ bars for volume SMA and 14+ for ATR)
        for i in 0..25 {
            let price = 100.0;
            let bar = create_test_bar(
                base_time + i * 86400,
                price - 0.5,
                price + 0.5,
                price - 1.0,
                price,
                1000.0,
            );
            // Should not panic
            let _ = strategy.on_bar(&bar);
        }

        // Verify indicators are updating
        assert!(strategy.volume_sma.value().is_some());
        assert!(strategy.atr.value().is_some());
    }

    #[test]
    fn test_breakout_volume_filter() {
        let config = BreakoutConfig::new(5, 2.0, 1.0, 3.0, 2.5);
        let mut strategy = BreakoutStrategy::from_config(config);
        let base_time = Utc::now().timestamp();

        // Feed range-bound data
        for i in 0..10 {
            let price = 100.0;
            let bar = create_test_bar(
                base_time + i * 86400,
                price - 0.5,
                price + 0.5,
                price - 1.0,
                price,
                1000.0,
            );
            strategy.on_bar(&bar);
        }

        // Verify lookback high is established
        assert!(strategy.get_lookback_high().is_some());

        // Try breakout with low volume (should be filtered)
        for i in 10..20 {
            let price = 100.0 + (i - 10) as f64 * 3.0;
            let bar = create_test_bar(
                base_time + i * 86400,
                price - 0.5,
                price + 0.5,
                price - 1.0,
                price,
                1100.0, // Low volume (won't pass 2x filter)
            );
            if let Some(signals) = strategy.on_bar(&bar) {
                if signals
                    .iter()
                    .any(|s| matches!(s.signal_type, SignalType::Buy))
                {
                    panic!("Breakout should be filtered by volume");
                }
            }
        }
    }

    #[test]
    fn test_breakout_metadata() {
        let strategy = BreakoutStrategy::new(20);
        let metadata = strategy.metadata();

        assert_eq!(metadata.name, "Breakout");
        assert_eq!(metadata.category, StrategyCategory::TrendFollowing);
        assert_eq!(metadata.sub_type, Some("price_breakout".to_string()));
        assert_eq!(
            metadata.hypothesis_path,
            "hypotheses/trend_following/breakout.md"
        );
        assert!(metadata.required_indicators.contains(&"ATR".to_string()));
        assert!(metadata.required_indicators.contains(&"Volume".to_string()));
    }

    #[test]
    fn test_breakout_clear_state() {
        // Verify new strategy has clean state
        let strategy = BreakoutStrategy::new(5);

        assert!(
            strategy.entry_price.is_none(),
            "New strategy should not have entry"
        );
        assert_eq!(
            strategy.position_size, 0.0,
            "New strategy should have no position"
        );
        assert!(!strategy.tp1_hit, "TP1 flag should be false");
        assert!(!strategy.tp2_hit, "TP2 flag should be false");
        assert!(!strategy.tp3_hit, "TP3 flag should be false");
        assert!(
            strategy.highest_since_entry.is_none(),
            "Highest since entry should be none"
        );
        assert!(
            strategy.stop_loss_level.is_none(),
            "Stop loss should be none"
        );
        assert!(
            strategy.breakout_level.is_none(),
            "Breakout level should be none"
        );
        assert!(
            strategy.trailing_stop_level.is_none(),
            "Trailing stop should be none"
        );
        assert!(strategy.entry_date.is_none(), "Entry date should be none");
    }

    #[test]
    fn test_breakout_with_rsi_filter() {
        let mut config = BreakoutConfig::new(5, 1.5, 1.0, 3.0, 2.5);
        config.rsi_filter_enabled = true;
        config.rsi_period = Some(14);
        config.rsi_threshold = Some(70.0);

        let strategy = BreakoutStrategy::from_config(config);
        assert!(
            strategy.rsi.is_some(),
            "RSI indicator should be initialized"
        );
    }

    #[test]
    fn test_breakout_with_adx_filter() {
        let mut config = BreakoutConfig::new(5, 1.5, 1.0, 3.0, 2.5);
        config.adx_filter_enabled = true;
        config.adx_period = Some(14);
        config.adx_threshold = Some(25.0);

        let strategy = BreakoutStrategy::from_config(config);
        assert!(
            strategy.adx.is_some(),
            "ADX indicator should be initialized"
        );
    }

    #[test]
    fn test_breakout_invalid_config() {
        let config = BreakoutConfig::new(0, 1.5, 1.0, 3.0, 2.5); // Invalid: lookback = 0
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_breakout_invalid_tp_order() {
        let mut config = BreakoutConfig::new(20, 1.5, 1.0, 3.0, 2.5);
        config.tp2_pct = 3.0; // Less than TP1 (5.0)
        assert!(config.validate().is_err());
    }
}
