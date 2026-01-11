//! Adaptive MA Strategy (Kaufman's KAMA)
//!
//! An adaptive moving average strategy that uses Kaufman's Adaptive Moving Average (KAMA)
//! which automatically adjusts its smoothing factor based on market volatility and trend strength.
//!
//! The strategy generates buy signals when price crosses above the KAMA and sell signals
//! when price crosses below the KAMA, with additional filters for trend confirmation.

use crate::config::{AdaptiveMAConfig, StrategyConfig};
use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Adx, Atr, Indicator, Kama, Rsi};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use std::collections::VecDeque;

/// Adaptive MA Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: Price crosses above KAMA with trend confirmation
/// - **Sell Signal**: Price crosses below KAMA, or hits stop loss/take profit
/// - **Filters**: Optional RSI, optional ADX, optional volume, ATR-based stops
///
/// # Example
/// ```
/// use alphafield_strategy::trend_following::AdaptiveMAStrategy;
/// use alphafield_strategy::config::AdaptiveMAConfig;
///
/// let config = AdaptiveMAConfig::new(10, 30, 10, 5.0, 3.0);
/// let strategy = AdaptiveMAStrategy::from_config(config);
/// ```
pub struct AdaptiveMAStrategy {
    config: AdaptiveMAConfig,
    kama: Kama,
    rsi: Option<Rsi>,
    adx: Option<Adx>,
    atr: Atr,
    volume_history: VecDeque<f64>,
    price_history: VecDeque<f64>,
    last_kama: Option<f64>,
    entry_price: Option<f64>,
    highest_since_entry: Option<f64>,
    trailing_stop_level: Option<f64>,
    position_size: f64,
}

impl AdaptiveMAStrategy {
    /// Creates a new Adaptive MA strategy with specified periods
    ///
    /// # Arguments
    /// * `fast_period` - Fast period for efficiency ratio
    /// * `slow_period` - Slow period for efficiency ratio
    /// * `price_period` - Period for price change calculation
    pub fn new(fast_period: usize, slow_period: usize, price_period: usize) -> Self {
        let config = AdaptiveMAConfig::new(fast_period, slow_period, price_period, 5.0, 3.0);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    ///
    /// # Panics
    /// Panics if the configuration is invalid
    pub fn from_config(config: AdaptiveMAConfig) -> Self {
        config.validate().expect("Invalid AdaptiveMAConfig");

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

        let fast_period = config.fast_period;
        let slow_period = config.slow_period;
        let price_period = config.price_period;
        let atr_period = config.atr_period;

        Self {
            config,
            kama: Kama::new(fast_period, slow_period, price_period),
            rsi,
            adx,
            atr: Atr::new(atr_period),
            volume_history: VecDeque::with_capacity(20),
            price_history: VecDeque::with_capacity(price_period),
            last_kama: None,
            entry_price: None,
            highest_since_entry: None,
            trailing_stop_level: None,
            position_size: 1.0,
        }
    }

    /// Returns the current configuration
    pub fn config(&self) -> &AdaptiveMAConfig {
        &self.config
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
        if let Some(vol_mult) = self.config.volume_min_multiplier {
            if let Some(avg_vol) = self.avg_volume() {
                return current_volume >= avg_vol * vol_mult;
            }
        }
        true // No filter or not enough history
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

            // TP: Close full position at take profit level
            if profit_pct >= self.config.take_profit {
                let exit_size = self.position_size;
                self.position_size = 0.0;

                // Set trailing stop after TP
                let atr_val = self.atr.value();
                self.trailing_stop_level = Some(self.calculate_trailing_stop(price, atr_val));

                return Some(Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: exit_size,
                    metadata: Some(format!("Take Profit: {:.1}% profit", profit_pct)),
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
        self.position_size = 1.0;
    }
}

impl MetadataStrategy for AdaptiveMAStrategy {
    fn metadata(&self) -> StrategyMetadata {
        let mut required_indicators = vec!["KAMA".to_string(), "Price".to_string()];
        if self.rsi.is_some() {
            required_indicators.push("RSI".to_string());
        }
        if self.adx.is_some() {
            required_indicators.push("ADX".to_string());
        }
        if self.config.atr_multiplier.is_some() {
            required_indicators.push("ATR".to_string());
        }

        StrategyMetadata {
            name: self.config.strategy_name().to_string(),
            category: StrategyCategory::TrendFollowing,
            sub_type: Some("adaptive_moving_average".to_string()),
            description: format!(
                "Adaptive MA strategy using KAMA with fast={}, slow={}, price={} periods.
                Features {:.1}% TP and {:.1}% SL, enhanced with {}RSI filter, {}ADX filter, {}volume confirmation, and {}ATR-based stops.
                Automatically adjusts smoothing based on market volatility for optimal trend following.",
                self.config.fast_period,
                self.config.slow_period,
                self.config.price_period,
                self.config.take_profit,
                self.config.stop_loss,
                if self.rsi.is_some() { "" } else { "no " },
                if self.adx.is_some() { "" } else { "no " },
                if self.config.volume_min_multiplier.is_some() { "" } else { "no " },
                if self.config.atr_multiplier.is_some() { "" } else { "no " }
            ),
            hypothesis_path: "hypotheses/trend_following/adaptive_ma.md".to_string(),
            required_indicators,
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Trending],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.20,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::TrendFollowing
    }
}

impl Strategy for AdaptiveMAStrategy {
    fn name(&self) -> &str {
        self.config.strategy_name()
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        // Update price history
        self.price_history.push_back(bar.close);
        if self.price_history.len() > self.config.price_period {
            self.price_history.pop_front();
        }

        // Update volume history
        self.volume_history.push_back(bar.volume);
        if self.volume_history.len() > 20 {
            self.volume_history.pop_front();
        }

        // Update all indicators
        let kama_opt = self.kama.update(bar.close);
        let _ = self.atr.update(bar);

        // Update optional indicators
        if let Some(rsi) = &mut self.rsi {
            rsi.update(bar.close);
        }
        if let Some(adx) = &mut self.adx {
            adx.update(bar);
        }

        let kama = kama_opt?;
        let price = bar.close;

        // EXIT LOGIC FIRST (priority: partial TP > trailing stop > SL > KAMA crossover down)
        if let Some(entry) = self.entry_price {
            let mut signals = Vec::new();

            // Check partial take profit first
            if let Some(tp_signal) = self.check_partial_exit(price, bar) {
                signals.push(tp_signal);

                // If position closed completely, reset state and return
                if self.position_size <= 0.0 {
                    self.reset_state();
                    self.last_kama = Some(kama);
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
                    self.last_kama = Some(kama);
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
                self.last_kama = Some(kama);
                signals.push(Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: Some(format!("Stop Loss: {:.1}%", profit_pct)),
                });
                return Some(signals);
            }

            // Check KAMA crossover down (full position exit)
            if let Some(prev_kama) = self.last_kama {
                if prev_kama <= price && kama > price && self.position_size > 0.0 {
                    let remaining_size = self.position_size.max(1.0);
                    self.reset_state();
                    self.last_kama = Some(kama);
                    signals.push(Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: remaining_size,
                        metadata: Some(format!(
                            "KAMA Crossover Down: Price {:.2} < KAMA {:.2}",
                            price, kama
                        )),
                    });
                    return Some(signals);
                }
            }

            // Update stored values
            self.last_kama = Some(kama);

            // Return partial exit signals if any
            if !signals.is_empty() {
                return Some(signals);
            }
            return None;
        }

        // ENTRY LOGIC
        if let Some(prev_kama) = self.last_kama {
            // Check for KAMA crossover up
            if prev_kama >= price && kama < price {
                // Apply all filters
                if !self.check_rsi_filter() {
                    // RSI filter failed (overbought)
                    self.last_kama = Some(kama);
                    return None;
                }

                if !self.check_adx_filter() {
                    // ADX filter failed (not trending)
                    self.last_kama = Some(kama);
                    return None;
                }

                if !self.check_volume_filter(bar.volume) {
                    // Volume filter failed
                    self.last_kama = Some(kama);
                    return None;
                }

                // All filters passed - KAMA crossover entry
                self.entry_price = Some(price);
                self.highest_since_entry = Some(price);
                self.position_size = 1.0; // Full position
                self.last_kama = Some(kama);

                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Buy,
                    strength: 1.0,
                    metadata: Some(format!(
                        "KAMA Crossover Up: Price {:.2} > KAMA {:.2}",
                        price, kama
                    )),
                }]);
            }
        }

        self.last_kama = Some(kama);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_ma_creation() {
        let strategy = AdaptiveMAStrategy::new(10, 30, 10);
        assert_eq!(strategy.name(), "Adaptive MA");
    }

    #[test]
    fn test_adaptive_ma_config_valid() {
        let config = AdaptiveMAConfig::new(10, 30, 10, 5.0, 3.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_adaptive_ma_from_config() {
        let config = AdaptiveMAConfig::new(5, 20, 5, 10.0, 5.0);
        let strategy = AdaptiveMAStrategy::from_config(config);
        assert_eq!(strategy.config().fast_period, 5);
        assert_eq!(strategy.config().slow_period, 20);
        assert_eq!(strategy.config().take_profit, 10.0);
    }

    #[test]
    fn test_adaptive_ma_metadata() {
        let strategy = AdaptiveMAStrategy::new(10, 30, 10);
        let metadata = strategy.metadata();

        assert_eq!(metadata.name, "Adaptive MA");
        assert_eq!(metadata.category, StrategyCategory::TrendFollowing);
        assert_eq!(
            metadata.sub_type,
            Some("adaptive_moving_average".to_string())
        );
        assert_eq!(
            metadata.hypothesis_path,
            "hypotheses/trend_following/adaptive_ma.md"
        );
        assert!(metadata.required_indicators.contains(&"KAMA".to_string()));
    }

    #[test]
    fn test_adaptive_ma_with_filters() {
        let mut config = AdaptiveMAConfig::new(5, 20, 5, 5.0, 3.0);
        config.rsi_filter_enabled = true;
        config.rsi_period = Some(14);
        config.rsi_threshold = Some(70.0);
        config.adx_filter_enabled = true;
        config.adx_period = Some(14);
        config.adx_threshold = Some(25.0);

        let strategy = AdaptiveMAStrategy::from_config(config);
        assert!(strategy.rsi.is_some());
        assert!(strategy.adx.is_some());
    }

    #[test]
    fn test_adaptive_ma_invalid_config() {
        let config = AdaptiveMAConfig::new(0, 30, 10, 5.0, 3.0);
        assert!(config.validate().is_err());
    }
}
