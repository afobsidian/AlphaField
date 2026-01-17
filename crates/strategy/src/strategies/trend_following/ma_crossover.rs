//! MA Crossover Strategy
//!
//! A generic moving average crossover strategy that generates buy signals
//! when a fast moving average crosses above a slow moving average, and
//! sell signals when the fast MA crosses below the slow MA.
//!
//! Supports both SMA and EMA types, with configurable periods and multiple
//! filters including MA separation, RSI, ADX, volume confirmation, and ATR-based stops.

use crate::config::{MACrossoverConfig, StrategyConfig};
use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Adx, Atr, Ema, Indicator, Rsi, Sma};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use std::collections::VecDeque;

/// MA Crossover Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: Fast MA crosses above Slow MA with sufficient separation
/// - **Sell Signal**: Fast MA crosses below Slow MA, or hits stop loss/take profit
/// - **Filters**: MA separation, optional RSI, optional ADX, optional volume, ATR-based stops
///
/// # Example
/// ```
/// use alphafield_strategy::trend_following::MACrossoverStrategy;
/// use alphafield_strategy::config::MACrossoverConfig;
///
/// let config = MACrossoverConfig::new(10, 30, 2.0, 5.0, 3.0);
/// let strategy = MACrossoverStrategy::from_config(config);
/// ```
pub struct MACrossoverStrategy {
    config: MACrossoverConfig,
    fast_ma: Box<dyn Indicator>,
    slow_ma: Box<dyn Indicator>,
    rsi: Option<Rsi>,
    adx: Option<Adx>,
    atr: Atr,
    volume_history: VecDeque<f64>,
    last_fast: Option<f64>,
    last_slow: Option<f64>,
    entry_price: Option<f64>,
    highest_since_entry: Option<f64>,
    trailing_stop_level: Option<f64>,
    position_size: f64,
}

impl Default for MACrossoverStrategy {
    fn default() -> Self {
        // Default: 10/30 fast/slow SMA periods, 2% separation, default filters and thresholds
        Self::from_config(MACrossoverConfig::default_config())
    }
}

impl MACrossoverStrategy {
    /// Creates a new MA Crossover strategy with specified periods
    ///
    /// # Arguments
    /// * `fast_period` - Period for fast moving average
    /// * `slow_period` - Period for slow moving average
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        let config = MACrossoverConfig::new(fast_period, slow_period, 2.0, 5.0, 3.0);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    ///
    /// # Panics
    /// Panics if the configuration is invalid
    pub fn from_config(config: MACrossoverConfig) -> Self {
        config.validate().expect("Invalid MACrossoverConfig");

        // Create appropriate MA types based on configuration
        let fast_ma: Box<dyn Indicator> = match config.ma_type.as_str() {
            "SMA" => Box::new(Sma::new(config.fast_period)),
            "EMA" => Box::new(Ema::new(config.fast_period)),
            _ => panic!("Invalid MA type: {}", config.ma_type),
        };

        let slow_ma: Box<dyn Indicator> = match config.ma_type.as_str() {
            "SMA" => Box::new(Sma::new(config.slow_period)),
            "EMA" => Box::new(Ema::new(config.slow_period)),
            _ => panic!("Invalid MA type: {}", config.ma_type),
        };

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

        let atr_period = config.atr_period;

        Self {
            config,
            fast_ma,
            slow_ma,
            rsi,
            adx,
            atr: Atr::new(atr_period),
            volume_history: VecDeque::with_capacity(20),
            last_fast: None,
            last_slow: None,
            entry_price: None,
            highest_since_entry: None,
            trailing_stop_level: None,
            position_size: 1.0,
        }
    }

    /// Returns the current configuration
    pub fn config(&self) -> &MACrossoverConfig {
        &self.config
    }

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

impl MetadataStrategy for MACrossoverStrategy {
    fn metadata(&self) -> StrategyMetadata {
        let mut required_indicators =
            vec![format!("{}MA", self.config.ma_type), "Price".to_string()];
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
            sub_type: Some("moving_average_crossover".to_string()),
            description: format!(
                "{} Crossover strategy using {} and {} period {}s with {:.1}% TP and {:.1}% SL.
                Requires {:.1}% MA separation, enhanced with {}RSI filter, {}ADX filter, {}volume confirmation, and {}ATR-based stops.
                Generates buy signals on {}MA crossovers and sell signals on death crosses.",
                self.config.ma_type,
                self.config.fast_period,
                self.config.slow_period,
                self.config.ma_type,
                self.config.take_profit,
                self.config.stop_loss,
                self.config.min_separation,
                if self.rsi.is_some() { "" } else { "no " },
                if self.adx.is_some() { "" } else { "no " },
                if self.config.volume_min_multiplier.is_some() { "" } else { "no " },
                if self.config.atr_multiplier.is_some() { "" } else { "no " },
                self.config.ma_type
            ),
            hypothesis_path: "hypotheses/trend_following/ma_crossover.md".to_string(),
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

impl Strategy for MACrossoverStrategy {
    fn name(&self) -> &str {
        self.config.strategy_name()
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        // Update all indicators
        let fast_opt = self.fast_ma.update(bar.close);
        let slow_opt = self.slow_ma.update(bar.close);
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

    #[test]
    fn test_macrossover_creation() {
        let strategy = MACrossoverStrategy::new(10, 30);
        assert_eq!(strategy.name(), "MA Crossover");
    }

    #[test]
    fn test_macrossover_config_valid() {
        let config = MACrossoverConfig::new(10, 30, 2.0, 5.0, 3.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_macrossover_from_config() {
        let config = MACrossoverConfig::new(5, 20, 1.0, 10.0, 5.0);
        let strategy = MACrossoverStrategy::from_config(config);
        assert_eq!(strategy.config().fast_period, 5);
        assert_eq!(strategy.config().slow_period, 20);
        assert_eq!(strategy.config().take_profit, 10.0);
    }

    #[test]
    fn test_macrossover_sma_vs_ema() {
        let mut config = MACrossoverConfig::new(5, 10, 2.0, 5.0, 3.0);
        config.ma_type = "SMA".to_string();
        let sma_strategy = MACrossoverStrategy::from_config(config.clone());

        config.ma_type = "EMA".to_string();
        let ema_strategy = MACrossoverStrategy::from_config(config);

        assert_eq!(sma_strategy.name(), ema_strategy.name());
    }

    #[test]
    fn test_macrossover_metadata() {
        let strategy = MACrossoverStrategy::new(10, 30);
        let metadata = strategy.metadata();

        assert_eq!(metadata.name, "MA Crossover");
        assert_eq!(metadata.category, StrategyCategory::TrendFollowing);
        assert_eq!(
            metadata.sub_type,
            Some("moving_average_crossover".to_string())
        );
        assert_eq!(
            metadata.hypothesis_path,
            "hypotheses/trend_following/ma_crossover.md"
        );
        assert!(metadata.required_indicators.contains(&"SMAMA".to_string()));
    }

    #[test]
    fn test_macrossover_with_filters() {
        let mut config = MACrossoverConfig::new(5, 10, 2.0, 5.0, 3.0);
        config.rsi_filter_enabled = true;
        config.rsi_period = Some(14);
        config.rsi_threshold = Some(70.0);
        config.adx_filter_enabled = true;
        config.adx_period = Some(14);
        config.adx_threshold = Some(25.0);

        let strategy = MACrossoverStrategy::from_config(config);
        assert!(strategy.rsi.is_some());
        assert!(strategy.adx.is_some());
    }

    #[test]
    fn test_macrossover_invalid_config() {
        let config = MACrossoverConfig::new(30, 10, 2.0, 5.0, 3.0); // Invalid: fast > slow
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_macrossover_invalid_ma_type() {
        let mut config = MACrossoverConfig::new(10, 30, 2.0, 5.0, 3.0);
        config.ma_type = "INVALID".to_string();
        assert!(config.validate().is_err());
    }
}
