//! Golden Cross Strategy
//!
//! A classic trend-following strategy that generates buy signals when a
//! fast-moving average crosses above a slow-moving average (golden cross),
//! and sell signals when the fast MA crosses below the slow MA (death cross).
//!
//! Enhanced with filters for trend confirmation, separation threshold, and
//! optional RSI/ADX filters to reduce whipsaws in ranging markets.

use crate::config::{GoldenCrossConfig, StrategyConfig};
use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Adx, Atr, Indicator, Rsi, Sma};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use std::collections::VecDeque;

/// Golden Cross Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: Fast SMA crosses above Slow SMA (bullish crossover)
/// - **Sell Signal**: Fast SMA crosses below Slow SMA (bearish crossover)
/// - **Filters**: MA separation, optional RSI, optional ADX, optional volume
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::GoldenCrossStrategy;
/// use alphafield_strategy::config::GoldenCrossConfig;
///
/// let config = GoldenCrossConfig::new(50, 200, 5.0, 5.0);
/// let strategy = GoldenCrossStrategy::from_config(config);
/// ```
pub struct GoldenCrossStrategy {
    config: GoldenCrossConfig,
    fast_sma: Sma,
    slow_sma: Sma,
    rsi: Option<Rsi>,
    adx: Option<Adx>,
    atr: Atr,
    last_fast: Option<f64>,
    last_slow: Option<f64>,
    entry_price: Option<f64>,
    highest_since_entry: Option<f64>,
    trailing_stop_level: Option<f64>,
    position_size: f64, // Track remaining position for partial exits
    volume_history: VecDeque<f64>,
}

impl GoldenCrossStrategy {
    /// Creates a new Golden Cross strategy with specified periods
    ///
    /// # Arguments
    /// * `fast_period` - Period for fast moving average
    /// * `slow_period` - Period for slow moving average
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        // Default to 5% TP/SL for backward compatibility in constructor
        let config = GoldenCrossConfig::new(fast_period, slow_period, 5.0, 5.0);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    ///
    /// # Panics
    /// Panics if the configuration is invalid
    pub fn from_config(config: GoldenCrossConfig) -> Self {
        config.validate().expect("Invalid GoldenCrossConfig");

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

        Self {
            fast_sma: Sma::new(config.fast_period),
            slow_sma: Sma::new(config.slow_period),
            rsi,
            adx,
            atr: Atr::new(config.atr_period.unwrap_or(14)),
            config,
            last_fast: None,
            last_slow: None,
            entry_price: None,
            highest_since_entry: None,
            trailing_stop_level: None,
            position_size: 0.0,
            volume_history: VecDeque::with_capacity(20),
        }
    }

    /// Returns the current configuration
    pub fn config(&self) -> &GoldenCrossConfig {
        &self.config
    }
}

impl Default for GoldenCrossStrategy {
    fn default() -> Self {
        // Default golden cross: 50-day and 200-day SMA with 5% TP/SL
        Self::new(50, 200)
    }
}

impl GoldenCrossStrategy {
    /// Check if MA separation meets minimum threshold
    fn check_separation(&self, fast: f64, slow: f64) -> bool {
        if self.config.min_separation <= 0.0 {
            return true; // No separation filter
        }
        let separation_pct = ((fast - slow) / slow).abs() * 100.0;
        separation_pct >= self.config.min_separation
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
            let min_vol_multiplier = self.config.volume_min_multiplier.unwrap_or(1.0);
            return current_volume >= avg_vol * min_vol_multiplier;
        }
        true // Not enough history
    }

    /// Calculate trailing stop level based on ATR
    fn calculate_trailing_stop(&self, price: f64, atr_value: Option<f64>) -> f64 {
        if let (Some(atr), Some(trailing_pct)) = (atr_value, self.config.trailing_stop) {
            let atr_based_stop = price - (atr * self.config.atr_multiplier.unwrap_or(2.0));
            let pct_based_stop = price * (1.0 - trailing_pct / 100.0);
            atr_based_stop.min(pct_based_stop)
        } else if let Some(trailing_pct) = self.config.trailing_stop {
            price * (1.0 - trailing_pct / 100.0)
        } else {
            price * 0.95 // Default 5% trailing stop
        }
    }

    /// Check partial take profit conditions
    fn check_partial_exit(&mut self, price: f64, bar: &Bar) -> Option<Signal> {
        if let Some(entry) = self.entry_price {
            let profit_pct = (price - entry) / entry * 100.0;

            // TP1: Close 50% at first take profit level
            if self.position_size > 0.0 && profit_pct >= self.config.take_profit {
                let exit_size = self.position_size * 0.5; // Close 50%
                self.position_size -= exit_size;

                // Set trailing stop after TP1
                if self.position_size > 0.0 {
                    let atr_val = self.atr.value();
                    self.trailing_stop_level = Some(self.calculate_trailing_stop(price, atr_val));
                }

                return Some(Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 0.5, // Partial exit strength
                    metadata: Some(format!(
                        "Take Profit 1: {:.1}% profit, closed 50% of position",
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
        self.highest_since_entry = None;
        self.trailing_stop_level = None;
        self.position_size = 0.0;
    }
}

impl MetadataStrategy for GoldenCrossStrategy {
    fn metadata(&self) -> StrategyMetadata {
        let mut required_indicators = vec!["SMA".to_string(), "Price".to_string()];
        if self.rsi.is_some() {
            required_indicators.push("RSI".to_string());
        }
        if self.adx.is_some() {
            required_indicators.push("ADX".to_string());
        }

        StrategyMetadata {
            name: self.config.strategy_name().to_string(),
            category: StrategyCategory::TrendFollowing,
            sub_type: Some("moving_average_crossover".to_string()),
            description: format!(
                "Golden Cross strategy using {} and {} period SMAs with {:.1}% TP and {:.1}% SL. \
                Enhanced with MA separation filter ({:.1}%), {}RSI filter, {}ADX filter, and {}volume confirmation. \
                Generates buy signals on golden cross (fast MA crosses above slow MA) and sell signals on death cross.",
                self.config.fast_period,
                self.config.slow_period,
                self.config.take_profit,
                self.config.stop_loss,
                self.config.min_separation,
                if self.rsi.is_some() { "" } else { "no " },
                if self.adx.is_some() { "" } else { "no " },
                if self.config.volume_min_multiplier.is_some() { "" } else { "no " }
            ),
            hypothesis_path: "hypotheses/trend_following/golden_cross.md".to_string(),
            required_indicators,
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Trending],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.25,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Medium,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::TrendFollowing
    }
}

impl Strategy for GoldenCrossStrategy {
    fn name(&self) -> &str {
        self.config.strategy_name()
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        // Update all indicators
        let fast_opt = self.fast_sma.update(bar.close);
        let slow_opt = self.slow_sma.update(bar.close);
        let _ = self.atr.update(bar);

        // Update volume history
        self.volume_history.push_back(bar.volume);
        if self.volume_history.len() > 20 {
            self.volume_history.pop_front();
        }

        // Update optional indicators
        if let Some(rsi) = &mut self.rsi {
            rsi.update(bar.close);
        }
        if let Some(adx) = &mut self.adx {
            adx.update(bar);
        }

        let fast = fast_opt?;
        let slow = slow_opt?;
        let price = bar.close;

        // Initialize last_fast and last_slow when SMAs first produce values
        // This enables crossover detection from the very first valid SMA values
        if self.last_fast.is_none() {
            self.last_fast = Some(fast);
        }
        if self.last_slow.is_none() {
            self.last_slow = Some(slow);
        }

        // EXIT LOGIC FIRST (priority: partial TP > trailing stop > SL > death cross)
        if let Some(entry) = self.entry_price {
            let mut signals = Vec::new();

            // Check partial take profit first
            if let Some(tp_signal) = self.check_partial_exit(price, bar) {
                signals.push(tp_signal);

                // If position closed completely, reset state and return
                if self.position_size <= 0.0 {
                    self.reset_state();
                    self.last_fast = Some(fast);
                    self.last_slow = Some(slow);
                    return Some(signals);
                }
            }

            // Update highest price for trailing stop
            self.highest_since_entry = Some(self.highest_since_entry.unwrap_or(entry).max(price));

            // Calculate and update trailing stop
            let atr_val = self.atr.value();
            let new_trailing_stop = self.calculate_trailing_stop(price, atr_val);
            self.trailing_stop_level = Some(
                self.trailing_stop_level
                    .unwrap_or(new_trailing_stop)
                    .max(new_trailing_stop),
            );

            // Check trailing stop
            if let Some(trailing_stop) = self.trailing_stop_level {
                if price <= trailing_stop && self.position_size > 0.0 {
                    self.reset_state();
                    self.last_fast = Some(fast);
                    self.last_slow = Some(slow);
                    signals.push(Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: self.position_size.max(1.0),
                        metadata: Some(format!(
                            "Trailing Stop Triggered: Price {:.2} <= Stop {:.2}",
                            price, trailing_stop
                        )),
                    });
                    return Some(signals);
                }
            }

            // Check initial stop loss (if no trailing stop yet)
            let profit_pct = (price - entry) / entry * 100.0;
            if profit_pct <= -self.config.stop_loss && self.position_size > 0.0 {
                self.reset_state();
                self.last_fast = Some(fast);
                self.last_slow = Some(slow);
                signals.push(Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: Some(format!("Stop Loss: {:.1}%", profit_pct)),
                });
                return Some(signals);
            }

            // Check death cross (full position exit)
            if let (Some(prev_fast), Some(prev_slow)) = (self.last_fast, self.last_slow) {
                if prev_fast >= prev_slow && fast < slow && self.position_size > 0.0 {
                    let remaining_size = self.position_size.max(1.0);
                    self.reset_state();
                    self.last_fast = Some(fast);
                    self.last_slow = Some(slow);
                    signals.push(Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: remaining_size,
                        metadata: Some(format!("Death Cross: Fast {:.2} < Slow {:.2}", fast, slow)),
                    });
                    return Some(signals);
                }
            }

            // Update stored values
            self.last_fast = Some(fast);
            self.last_slow = Some(slow);

            // Return partial exit signals if any
            if !signals.is_empty() {
                return Some(signals);
            }
            return None;
        }

        // ENTRY LOGIC
        if let (Some(prev_fast), Some(prev_slow)) = (self.last_fast, self.last_slow) {
            // Check for golden cross
            if prev_fast <= prev_slow && fast > slow {
                // Apply all filters
                if !self.check_separation(fast, slow) {
                    // Separation filter failed
                    self.last_fast = Some(fast);
                    self.last_slow = Some(slow);
                    return None;
                }

                if !self.check_rsi_filter() {
                    // RSI filter failed (overbought)
                    self.last_fast = Some(fast);
                    self.last_slow = Some(slow);
                    return None;
                }

                if !self.check_adx_filter() {
                    // ADX filter failed (not trending)
                    self.last_fast = Some(fast);
                    self.last_slow = Some(slow);
                    return None;
                }

                if !self.check_volume_filter(bar.volume) {
                    // Volume filter failed
                    self.last_fast = Some(fast);
                    self.last_slow = Some(slow);
                    return None;
                }

                // All filters passed - Golden Cross entry
                self.entry_price = Some(price);
                self.highest_since_entry = Some(price);
                self.position_size = 1.0; // Full position
                self.last_fast = Some(fast);
                self.last_slow = Some(slow);

                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Buy,
                    strength: 1.0,
                    metadata: Some(format!(
                        "Golden Cross: Fast {:.2} > Slow {:.2}, Separation: {:.2}%",
                        fast,
                        slow,
                        ((fast - slow) / slow) * 100.0
                    )),
                }]);
            }
        }

        self.last_fast = Some(fast);
        self.last_slow = Some(slow);
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
    fn test_golden_cross_creation() {
        let strategy = GoldenCrossStrategy::new(10, 30);
        assert_eq!(strategy.name(), "Golden Cross");
    }

    #[test]
    fn test_golden_cross_config_valid() {
        let config = GoldenCrossConfig::new(10, 30, 5.0, 5.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_golden_cross_from_config() {
        let config = GoldenCrossConfig::new(5, 20, 10.0, 5.0);
        let strategy = GoldenCrossStrategy::from_config(config);
        assert_eq!(strategy.config().fast_period, 5);
        assert_eq!(strategy.config().slow_period, 20);
        assert_eq!(strategy.config().take_profit, 10.0);
    }

    #[test]
    #[should_panic(expected = "Invalid GoldenCrossConfig")]
    fn test_golden_cross_invalid_config() {
        let config = GoldenCrossConfig::new(50, 20, 5.0, 5.0); // Invalid: fast > slow
        GoldenCrossStrategy::from_config(config);
    }

    #[test]
    fn test_golden_cross_crossover_detection() {
        let mut strategy = GoldenCrossStrategy::new(3, 5);
        let base_time = Utc::now().timestamp();

        // Feed prices to establish declining trend (fast MA < slow MA)
        for i in 0..10 {
            let price = 100.0 - i as f64 * 1.0;
            let bar = create_test_bar(
                base_time + i * 86400,
                price - 0.2,
                price + 0.2,
                price - 0.3,
                price,
                1000.0,
            );
            strategy.on_bar(&bar);
        }

        // Now feed rising prices to trigger crossover
        let mut crossover_detected = false;
        for i in 10..25 {
            // Strong uptrend to ensure crossover
            let price = 90.0 + (i - 10) as f64 * 4.0;
            let bar = create_test_bar(
                base_time + i * 86400,
                price - 0.5,
                price + 0.5,
                price - 1.0,
                price,
                1000.0,
            );
            if let Some(signals) = strategy.on_bar(&bar) {
                if signals
                    .iter()
                    .any(|s| matches!(s.signal_type, SignalType::Buy))
                {
                    crossover_detected = true;
                    break;
                }
            }
        }

        assert!(
            crossover_detected,
            "Golden cross crossover should be detected"
        );
    }

    #[test]
    fn test_golden_cross_separation_filter() {
        let config = GoldenCrossConfig::new(5, 10, 5.0, 5.0);
        let strategy_config = GoldenCrossConfig {
            min_separation: 5.0, // Require 5% separation
            ..config
        };
        let mut strategy = GoldenCrossStrategy::from_config(strategy_config);
        let base_time = Utc::now().timestamp();

        // Feed data with small crossover (< 5%)
        for i in 0..20 {
            let price = 100.0 + i as f64 * 0.3; // Slow growth, small separation
            let bar = create_test_bar(
                base_time + i * 86400,
                price - 0.1,
                price + 0.1,
                price - 0.2,
                price,
                1000.0,
            );
            strategy.on_bar(&bar);
        }

        // Check that no buy signal was generated due to separation filter
        let mut has_buy_signal = false;
        for i in 20..30 {
            let price = 100.0 + i as f64 * 0.3;
            let bar = create_test_bar(
                base_time + i * 86400,
                price - 0.1,
                price + 0.1,
                price - 0.2,
                price,
                1000.0,
            );
            if let Some(signals) = strategy.on_bar(&bar) {
                if signals
                    .iter()
                    .any(|s| matches!(s.signal_type, SignalType::Buy))
                {
                    has_buy_signal = true;
                }
            }
        }

        assert!(
            !has_buy_signal,
            "Buy signal generated despite separation filter"
        );
    }

    #[test]
    fn test_golden_cross_rsi_filter_config() {
        let mut config = GoldenCrossConfig::new(5, 10, 5.0, 5.0);

        // Test enabling RSI filter
        config.rsi_filter_enabled = true;
        config.rsi_period = Some(14);
        config.rsi_threshold = Some(70.0);

        // Should validate successfully
        assert!(config.validate().is_ok());

        // Create strategy with RSI filter
        let strategy = GoldenCrossStrategy::from_config(config);
        assert!(
            strategy.rsi.is_some(),
            "RSI indicator should be initialized"
        );

        // Test invalid config (RSI enabled but no period)
        let mut invalid_config = GoldenCrossConfig::new(5, 10, 5.0, 5.0);
        invalid_config.rsi_filter_enabled = true;
        // rsi_period is None
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_golden_cross_position_tracking() {
        let mut strategy = GoldenCrossStrategy::new(3, 5);
        let base_time = Utc::now().timestamp();

        // Initial state: no position
        assert!(strategy.entry_price.is_none());
        assert_eq!(strategy.position_size, 0.0);

        // Feed declining prices
        for i in 0..8 {
            let price = 100.0 - i as f64 * 1.0;
            let bar = create_test_bar(
                base_time + i * 86400,
                price - 0.2,
                price + 0.2,
                price - 0.3,
                price,
                1000.0,
            );
            strategy.on_bar(&bar);
        }

        // Trigger crossover
        for i in 8..15 {
            let price = 92.0 + (i - 8) as f64 * 3.0;
            let bar = create_test_bar(
                base_time + i * 86400,
                price - 0.5,
                price + 0.5,
                price - 1.0,
                price,
                1000.0,
            );

            if let Some(signals) = strategy.on_bar(&bar) {
                if signals
                    .iter()
                    .any(|s| matches!(s.signal_type, SignalType::Buy))
                {
                    // Should now be in position
                    assert!(strategy.entry_price.is_some());
                    assert_eq!(strategy.position_size, 1.0);
                    return;
                }
            }
        }
    }

    #[test]
    fn test_golden_cross_config_with_filters() {
        let mut config = GoldenCrossConfig::new(10, 30, 5.0, 5.0);

        // Test all filter configurations
        config.min_separation = 2.0;
        config.rsi_filter_enabled = true;
        config.rsi_period = Some(14);
        config.rsi_threshold = Some(70.0);
        config.adx_filter_enabled = true;
        config.adx_period = Some(14);
        config.adx_threshold = Some(25.0);
        config.volume_min_multiplier = Some(1.2);

        // Should validate
        assert!(config.validate().is_ok());

        // Create strategy
        let strategy = GoldenCrossStrategy::from_config(config);

        // Verify indicators are initialized
        assert!(strategy.rsi.is_some());
        assert!(strategy.adx.is_some());
    }

    #[test]
    fn test_golden_cross_metadata() {
        let strategy = GoldenCrossStrategy::new(50, 200);

        let metadata = strategy.metadata();

        assert_eq!(metadata.name, "Golden Cross");
        assert_eq!(metadata.category, StrategyCategory::TrendFollowing);
        assert_eq!(
            metadata.sub_type,
            Some("moving_average_crossover".to_string())
        );
        assert_eq!(
            metadata.hypothesis_path,
            "hypotheses/trend_following/golden_cross.md"
        );
        assert!(metadata.description.contains("50 and 200 period SMAs"));
        assert!(metadata.required_indicators.contains(&"SMA".to_string()));
    }

    #[test]
    fn test_golden_cross_indicator_updates() {
        let mut strategy = GoldenCrossStrategy::new(3, 5);
        let base_time = Utc::now().timestamp();

        // Feed enough bars to update indicators (ATR needs 14+ bars)
        for i in 0..20 {
            let price = 100.0 + i as f64 * 1.0;
            let bar = create_test_bar(
                base_time + i * 86400,
                price - 0.5,
                price + 0.5,
                price - 1.0,
                price,
                1000.0,
            );

            // Should not crash
            let _ = strategy.on_bar(&bar);
        }

        // Indicators should have values after warmup
        assert!(strategy.fast_sma.value().is_some());
        assert!(strategy.slow_sma.value().is_some());
        assert!(strategy.atr.value().is_some());
    }

    #[test]
    fn test_golden_cross_new_instance_clean_state() {
        let strategy = GoldenCrossStrategy::new(5, 10);

        // Verify clean initial state
        assert!(strategy.entry_price.is_none());
        assert!(strategy.highest_since_entry.is_none());
        assert!(strategy.trailing_stop_level.is_none());
        assert_eq!(strategy.position_size, 0.0);
        assert!(strategy.last_fast.is_none());
        assert!(strategy.last_slow.is_none());
    }
}
