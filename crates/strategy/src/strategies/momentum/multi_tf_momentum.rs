//! Multi-Timeframe Momentum Strategy
//!
//! Requires momentum confirmation on both short and long timeframes.
//! Uses moving averages on different timeframes for alignment.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Ema, Indicator};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for Multi-Timeframe Momentum strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTfMomentumConfig {
    /// Fast (short-term) EMA period
    pub fast_ema_period: usize,
    /// Slow (long-term) EMA period
    pub slow_ema_period: usize,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl MultiTfMomentumConfig {
    pub fn new(
        fast_ema_period: usize,
        slow_ema_period: usize,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            fast_ema_period,
            slow_ema_period,
            take_profit,
            stop_loss,
        }
    }

    pub fn default_config() -> Self {
        Self {
            fast_ema_period: 20, // Short-term trend
            slow_ema_period: 50, // Long-term trend
            take_profit: 5.0,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.fast_ema_period == 0 {
            return Err("Fast EMA period must be greater than 0".to_string());
        }
        if self.slow_ema_period == 0 {
            return Err("Slow EMA period must be greater than 0".to_string());
        }
        if self.fast_ema_period >= self.slow_ema_period {
            return Err("Fast EMA period must be less than slow EMA period".to_string());
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

impl fmt::Display for MultiTfMomentumConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Multi_TF_Momentum(fast={}, slow={}, tp={:.1}%, sl={:.1}%)",
            self.fast_ema_period, self.slow_ema_period, self.take_profit, self.stop_loss
        )
    }
}

/// Multi-Timeframe Momentum Strategy
///
/// # Strategy Logic
/// Simulates multi-timeframe analysis using EMAs of different periods:
/// - Fast EMA (e.g., 20) represents short-term trend
/// - Slow EMA (e.g., 50) represents long-term trend
///
/// - **Buy Signal**: Price > Fast EMA AND Fast EMA > Slow EMA (alignment)
/// - **Sell Signal**: Price < Fast EMA OR Fast EMA < Slow EMA OR TP/SL
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::momentum::MultiTfMomentumStrategy;
///
/// let strategy = MultiTfMomentumStrategy::new(20, 50, 5.0, 3.0);
/// ```
pub struct MultiTfMomentumStrategy {
    config: MultiTfMomentumConfig,
    fast_ema: Ema,
    slow_ema: Ema,
    last_position: SignalType,
    entry_price: Option<f64>,
    last_price_above_fast: Option<bool>,
    last_fast_above_slow: Option<bool>,
}

impl Default for MultiTfMomentumStrategy {
    fn default() -> Self {
        // Default: 20/50 EMA, 5% TP, 3% SL
        Self::from_config(MultiTfMomentumConfig::default_config())
    }
}

impl MultiTfMomentumStrategy {
    /// Creates a new Multi-Timeframe Momentum strategy
    pub fn new(
        fast_ema_period: usize,
        slow_ema_period: usize,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        let config =
            MultiTfMomentumConfig::new(fast_ema_period, slow_ema_period, take_profit, stop_loss);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: MultiTfMomentumConfig) -> Self {
        config.validate().expect("Invalid MultiTfMomentumConfig");

        Self {
            fast_ema: Ema::new(config.fast_ema_period),
            slow_ema: Ema::new(config.slow_ema_period),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            last_price_above_fast: None,
            last_fast_above_slow: None,
        }
    }

    pub fn config(&self) -> &MultiTfMomentumConfig {
        &self.config
    }

    /// Check if all timeframes are aligned for bullish momentum
    /// This helper is test-only and not needed in non-test builds.
    #[cfg(test)]
    fn is_aligned_bullish(&self, price: f64, fast_ema: f64, slow_ema: f64) -> bool {
        price > fast_ema && fast_ema > slow_ema
    }
}

impl MetadataStrategy for MultiTfMomentumStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Multi-TF Momentum".to_string(),
            category: StrategyCategory::Momentum,
            sub_type: Some("multi_timeframe".to_string()),
            description: format!(
                "Multi-timeframe momentum strategy requiring alignment across timeframes.
                Uses {}-period EMA (short-term) and {}-period EMA (long-term).
                Enters when Price > Fast EMA > Slow EMA (all aligned).
                Exits when alignment breaks or on TP/SL. Uses {:.1}% TP and {:.1}% SL.",
                self.config.fast_ema_period,
                self.config.slow_ema_period,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/momentum/multi_tf_momentum.md".to_string(),
            required_indicators: vec!["Fast_EMA".to_string(), "Slow_EMA".to_string()],
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Trending],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.18,
                volatility_level: VolatilityLevel::Low,
                correlation_sensitivity: CorrelationSensitivity::Medium,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::Momentum
    }
}

impl Strategy for MultiTfMomentumStrategy {
    fn name(&self) -> &str {
        "Multi-TF Momentum"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;

        // Update EMAs
        let fast_ema_val = self.fast_ema.update(price)?;
        let slow_ema_val = self.slow_ema.update(price)?;

        // Check alignment
        let price_above_fast = price > fast_ema_val;
        let fast_above_slow = fast_ema_val > slow_ema_val;

        // Get previous states for crossover detection
        let prev_price_above_fast = self.last_price_above_fast;
        let prev_fast_above_slow = self.last_fast_above_slow;

        // Update state for next bar
        self.last_price_above_fast = Some(price_above_fast);
        self.last_fast_above_slow = Some(fast_above_slow);

        // EXIT LOGIC FIRST (only when in position)
        if self.last_position == SignalType::Buy {
            if let Some(entry) = self.entry_price {
                let profit_pct = (price - entry) / entry * 100.0;

                // Take Profit
                if profit_pct >= self.config.take_profit {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("Take Profit: {:.1}%", profit_pct)),
                    }]);
                }

                // Stop Loss
                if profit_pct <= -self.config.stop_loss {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("Stop Loss: {:.1}%", profit_pct)),
                    }]);
                }

                // Exit on alignment break: price crosses below fast EMA
                if let Some(was_above) = prev_price_above_fast {
                    if was_above && !price_above_fast {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 0.9,
                            metadata: Some(format!(
                                "Short-term Alignment Break: Price {:.2} crossed below Fast EMA {:.2}",
                                price, fast_ema_val
                            )),
                        }]);
                    }
                }

                // Exit on alignment break: fast EMA crosses below slow EMA
                if let Some(was_above) = prev_fast_above_slow {
                    if was_above && !fast_above_slow {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 0.9,
                            metadata: Some(format!(
                                "Long-term Alignment Break: Fast EMA {:.2} crossed below Slow EMA {:.2}",
                                fast_ema_val, slow_ema_val
                            )),
                        }]);
                    }
                }
            }
        }

        // ENTRY LOGIC - All timeframes align bullish
        if self.last_position != SignalType::Buy {
            // Check for alignment establishment (price crosses above fast EMA while fast > slow)
            if let (Some(was_above_fast), Some(was_fast_above_slow)) =
                (prev_price_above_fast, prev_fast_above_slow)
            {
                // Entry when price crosses above fast EMA and fast EMA is already above slow EMA
                if !was_above_fast && price_above_fast && fast_above_slow {
                    // Calculate signal strength based on separation between EMAs
                    let ema_separation = (fast_ema_val - slow_ema_val) / slow_ema_val;
                    let strength = (ema_separation * 100.0).clamp(0.6, 1.0);

                    self.last_position = SignalType::Buy;
                    self.entry_price = Some(price);
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength,
                        metadata: Some(format!(
                            "Multi-TF Alignment Entry: Price {:.2} > Fast EMA {:.2} > Slow EMA {:.2}",
                            price, fast_ema_val, slow_ema_val
                        )),
                    }]);
                }

                // Alternative entry: fast EMA crosses above slow EMA while price is already above fast
                if price_above_fast && !was_fast_above_slow && fast_above_slow {
                    let strength = 0.8; // Slightly weaker signal

                    self.last_position = SignalType::Buy;
                    self.entry_price = Some(price);
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength,
                        metadata: Some(format!(
                            "Long-term Alignment Entry: Fast EMA {:.2} crossed above Slow EMA {:.2}",
                            fast_ema_val, slow_ema_val
                        )),
                    }]);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_bar(price: f64) -> Bar {
        Bar {
            timestamp: Utc::now(),
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_multi_tf_momentum_creation() {
        let strategy = MultiTfMomentumStrategy::new(20, 50, 5.0, 3.0);
        assert_eq!(strategy.name(), "Multi-TF Momentum");
    }

    #[test]
    fn test_config_validation() {
        let config = MultiTfMomentumConfig::default_config();
        assert!(config.validate().is_ok());

        let invalid_config = MultiTfMomentumConfig::new(50, 20, 5.0, 3.0); // fast >= slow
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_metadata() {
        let strategy = MultiTfMomentumStrategy::new(20, 50, 5.0, 3.0);
        let metadata = strategy.metadata();
        assert_eq!(metadata.category, StrategyCategory::Momentum);
        assert_eq!(metadata.name, "Multi-TF Momentum");
    }

    #[test]
    fn test_is_aligned_bullish() {
        let strategy = MultiTfMomentumStrategy::new(20, 50, 5.0, 3.0);

        // Aligned: 110 > 105 > 100
        assert!(strategy.is_aligned_bullish(110.0, 105.0, 100.0));

        // Not aligned: 110 > 100 but 100 < 105 (fast < slow)
        assert!(!strategy.is_aligned_bullish(110.0, 100.0, 105.0));

        // Not aligned: 95 < 105 (price < fast)
        assert!(!strategy.is_aligned_bullish(95.0, 105.0, 100.0));
    }

    #[test]
    fn test_no_signal_without_warmup() {
        let mut strategy = MultiTfMomentumStrategy::new(20, 50, 5.0, 3.0);

        // First few bars should not generate signals (EMAs need warmup)
        for i in 1..10 {
            let bar = create_test_bar(100.0 + i as f64);
            let _signal = strategy.on_bar(&bar);
            // May be None until EMAs warm up
        }
    }
}
