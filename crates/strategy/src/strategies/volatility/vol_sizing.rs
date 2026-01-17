//! Volatility-Adjusted Position Sizing Strategy
//!
//! This strategy adapts position size based on market volatility. The core
//! hypothesis is that position size should be inversely proportional to
//! volatility: smaller positions in high volatility (to control risk),
//! larger positions in low volatility (to maximize returns).
//!
//! The strategy uses ATR as volatility measure and generates entry
//! signals based on a simple trend-following approach (MA crossover),
//! with position size dynamically calculated from current ATR levels.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Atr, Indicator, Sma};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Configuration for Volatility-Adjusted Position Sizing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolSizingConfig {
    /// Period for ATR calculation
    pub atr_period: usize,
    /// Base position size (as % of account equity)
    pub base_size_pct: f64,
    /// Minimum position size (as % of account equity)
    pub min_size_pct: f64,
    /// Maximum position size (as % of account equity)
    pub max_size_pct: f64,
    /// Volatility scaling factor (higher = more aggressive size reduction)
    pub vol_scaling_factor: f64,
    /// Period for ATR baseline calculation (for normalization)
    pub baseline_period: usize,
    /// Fast MA for trend following
    pub fast_period: usize,
    /// Slow MA for trend following
    pub slow_period: usize,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl VolSizingConfig {
    pub fn new(atr_period: usize, base_size_pct: f64, baseline_period: usize) -> Self {
        Self {
            atr_period,
            base_size_pct,
            min_size_pct: 1.0,
            max_size_pct: 25.0,
            vol_scaling_factor: 2.0,
            baseline_period,
            fast_period: 10,
            slow_period: 30,
            take_profit: 5.0,
            stop_loss: 3.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            atr_period: 14,
            base_size_pct: 10.0,
            min_size_pct: 1.0,
            max_size_pct: 25.0,
            vol_scaling_factor: 2.0,
            baseline_period: 100,
            fast_period: 10,
            slow_period: 30,
            take_profit: 5.0,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.atr_period == 0 {
            return Err("ATR period must be greater than 0".to_string());
        }
        if self.base_size_pct <= 0.0 {
            return Err("Base size % must be positive".to_string());
        }
        if self.min_size_pct <= 0.0 {
            return Err("Min size % must be positive".to_string());
        }
        if self.max_size_pct <= 0.0 {
            return Err("Max size % must be positive".to_string());
        }
        if self.min_size_pct > self.max_size_pct {
            return Err("Min size % must be less than or equal to max size %".to_string());
        }
        if self.base_size_pct < self.min_size_pct || self.base_size_pct > self.max_size_pct {
            return Err("Base size % must be between min and max size %".to_string());
        }
        if self.vol_scaling_factor <= 0.0 {
            return Err("Volatility scaling factor must be positive".to_string());
        }
        if self.baseline_period < 20 {
            return Err("Baseline period must be at least 20".to_string());
        }
        if self.fast_period == 0 || self.slow_period == 0 {
            return Err("MA periods must be greater than 0".to_string());
        }
        if self.fast_period >= self.slow_period {
            return Err("Fast period must be less than slow period".to_string());
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

impl fmt::Display for VolSizingConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VolSizing(atr_period={}, base_size={:.1}%, min={:.1}%, max={:.1}%, vol_scale={:.1}, baseline={}, ma={}/{}, tp={:.1}%, sl={:.1}%)",
            self.atr_period,
            self.base_size_pct,
            self.min_size_pct,
            self.max_size_pct,
            self.vol_scaling_factor,
            self.baseline_period,
            self.fast_period,
            self.slow_period,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// Volatility-Adjusted Position Sizing Strategy
///
/// # Strategy Logic
/// - **Entry**: Golden cross (fast MA crosses above slow MA) with volatility-adjusted size
/// - **Position Size**: `size = base_size * (baseline_atr / current_atr)^scaling_factor`
///   - Higher volatility → smaller position size
///   - Lower volatility → larger position size
/// - **Exit**: Death cross, take profit, or stop loss
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::volatility::VolSizingStrategy;
///
/// let strategy = VolSizingStrategy::new(14, 10.0, 100);
/// ```
pub struct VolSizingStrategy {
    config: VolSizingConfig,
    atr: Atr,
    atr_baseline: Option<f64>,
    atr_history: VecDeque<f64>,
    fast_sma: Sma,
    slow_sma: Sma,
    last_position: SignalType,
    entry_price: Option<f64>,
    entry_size_pct: Option<f64>,
}

impl Default for VolSizingStrategy {
    fn default() -> Self {
        Self::from_config(VolSizingConfig::default_config())
    }
}

impl VolSizingStrategy {
    /// Creates a new Volatility-Adjusted Position Sizing strategy
    ///
    /// # Arguments
    /// * `atr_period` - ATR calculation period
    /// * `base_size_pct` - Base position size as % of account equity
    /// * `baseline_period` - Historical ATR window for baseline calculation
    pub fn new(atr_period: usize, base_size_pct: f64, baseline_period: usize) -> Self {
        let config = VolSizingConfig::new(atr_period, base_size_pct, baseline_period);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: VolSizingConfig) -> Self {
        config.validate().expect("Invalid VolSizingConfig");

        Self {
            atr: Atr::new(config.atr_period),
            atr_baseline: None,
            atr_history: VecDeque::with_capacity(config.baseline_period),
            fast_sma: Sma::new(config.fast_period),
            slow_sma: Sma::new(config.slow_period),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            entry_size_pct: None,
        }
    }

    pub fn config(&self) -> &VolSizingConfig {
        &self.config
    }

    /// Calculate volatility-adjusted position size
    fn calculate_position_size(&mut self, current_atr: f64) -> f64 {
        // Calculate ATR baseline if not yet set
        if self.atr_baseline.is_none() && self.atr_history.len() >= self.config.baseline_period {
            let sum: f64 = self.atr_history.iter().sum();
            self.atr_baseline = Some(sum / self.atr_history.len() as f64);
        }

        if let Some(baseline) = self.atr_baseline {
            if current_atr == 0.0 {
                return self.config.min_size_pct;
            }

            // Volatility ratio: current vs baseline
            // If vol > baseline (1.0), we reduce size
            // If vol < baseline (1.0), we increase size
            let vol_ratio = current_atr / baseline;

            // Apply scaling factor
            // Higher scaling factor = more aggressive size adjustment
            let size_adjustment = vol_ratio.powf(self.config.vol_scaling_factor);

            // Calculate size: base size adjusted by volatility
            let mut size = self.config.base_size_pct / size_adjustment;

            // Clamp to min/max bounds
            size = size.clamp(self.config.min_size_pct, self.config.max_size_pct);

            size
        } else {
            // Not enough history for baseline, use base size
            self.config.base_size_pct
        }
    }
}

impl MetadataStrategy for VolSizingStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Volatility-Adjusted Position Sizing".to_string(),
            category: StrategyCategory::VolatilityBased,
            sub_type: Some("volatility_sizing".to_string()),
            description: format!(
                "Volatility-adjusted position sizing using {} period ATR with {} period baseline. \
                Position size = base ({:.1}%) / (current_ATR / baseline_ATR)^{:.1}. \
                Higher volatility → smaller size, Lower volatility → larger size. \
                Size range: {:.1}% to {:.1}%. Enters on {} / {} MA crossover. \
                Uses {:.1}% TP and {:.1}% SL.",
                self.config.atr_period,
                self.config.baseline_period,
                self.config.base_size_pct,
                self.config.vol_scaling_factor,
                self.config.min_size_pct,
                self.config.max_size_pct,
                self.config.fast_period,
                self.config.slow_period,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/volatility/vol_sizing.md".to_string(),
            required_indicators: vec!["ATR".to_string(), "SMA".to_string()],
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Trending],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.15,
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

impl Strategy for VolSizingStrategy {
    fn name(&self) -> &str {
        "Volatility-Adjusted Position Sizing"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;

        // Update indicators
        let fast_ma = self.fast_sma.update(price)?;
        let slow_ma = self.slow_sma.update(price)?;
        let atr_value = self.atr.update(bar)?;

        // Track ATR history for baseline calculation
        self.atr_history.push_back(atr_value);
        if self.atr_history.len() > self.config.baseline_period {
            self.atr_history.pop_front();
        }

        // Calculate current position size based on volatility
        let current_size_pct = self.calculate_position_size(atr_value);

        // ENTRY LOGIC (only when not in position)
        if self.last_position == SignalType::Hold {
            // Check for golden cross (fast SMA crosses above slow SMA)
            // Simple crossover detection
            let prev_fast = self.fast_sma.value()? - (self.fast_sma.value()? - fast_ma);
            let prev_slow = self.slow_sma.value()? - (self.slow_sma.value()? - slow_ma);

            if prev_fast <= prev_slow && fast_ma > slow_ma {
                self.last_position = SignalType::Buy;
                self.entry_price = Some(price);
                self.entry_size_pct = Some(current_size_pct);

                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Buy,
                    strength: current_size_pct / 100.0, // Normalize to 0-1 range
                    metadata: Some(format!(
                        "Golden Cross Entry: Fast MA ({:.2}) > Slow MA ({:.2}), \
                        Position Size: {:.1}% (ATR: {:.2}, Baseline: {:.2})",
                        fast_ma,
                        slow_ma,
                        current_size_pct,
                        atr_value,
                        self.atr_baseline.unwrap_or(atr_value)
                    )),
                }]);
            }
        }

        // EXIT LOGIC (only when in position)
        if self.last_position == SignalType::Buy {
            if let Some(entry) = self.entry_price {
                let profit_pct = (price - entry) / entry * 100.0;

                // Take Profit
                if profit_pct >= self.config.take_profit {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    let exit_size = self.entry_size_pct.unwrap_or(self.config.base_size_pct);
                    self.entry_size_pct = None;

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: exit_size / 100.0,
                        metadata: Some(format!(
                            "Take Profit: {:.1}% profit (Position was {:.1}%)",
                            profit_pct, exit_size
                        )),
                    }]);
                }

                // Stop Loss
                if profit_pct <= -self.config.stop_loss {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    let exit_size = self.entry_size_pct.unwrap_or(self.config.base_size_pct);
                    self.entry_size_pct = None;

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: exit_size / 100.0,
                        metadata: Some(format!(
                            "Stop Loss: {:.1}% loss (Position was {:.1}%)",
                            profit_pct, exit_size
                        )),
                    }]);
                }

                // Death cross (fast SMA crosses below slow SMA)
                let prev_fast = self.fast_sma.value()? - (self.fast_sma.value()? - fast_ma);
                let prev_slow = self.slow_sma.value()? - (self.slow_sma.value()? - slow_ma);

                if prev_fast >= prev_slow && fast_ma < slow_ma {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    let exit_size = self.entry_size_pct.unwrap_or(self.config.base_size_pct);
                    self.entry_size_pct = None;

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: exit_size / 100.0,
                        metadata: Some(format!(
                            "Death Cross Exit: {:.1}% profit, Fast MA ({:.2}) < Slow MA ({:.2}) (Position was {:.1}%)",
                            profit_pct, fast_ma, slow_ma, exit_size
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
    fn test_vol_sizing_creation() {
        let strategy = VolSizingStrategy::new(14, 10.0, 100);
        assert_eq!(strategy.config().atr_period, 14);
        assert_eq!(strategy.config().base_size_pct, 10.0);
        assert_eq!(strategy.config().baseline_period, 100);
    }

    #[test]
    fn test_vol_sizing_config_valid() {
        let config = VolSizingConfig::new(14, 10.0, 100);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_vol_sizing_invalid_config() {
        let config = VolSizingConfig {
            atr_period: 0,
            base_size_pct: 10.0,
            min_size_pct: 1.0,
            max_size_pct: 25.0,
            vol_scaling_factor: 2.0,
            baseline_period: 100,
            fast_period: 10,
            slow_period: 30,
            take_profit: 5.0,
            stop_loss: 3.0,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_vol_sizing_invalid_size_bounds() {
        let config = VolSizingConfig {
            atr_period: 14,
            base_size_pct: 10.0,
            min_size_pct: 15.0, // min > max
            max_size_pct: 5.0,
            vol_scaling_factor: 2.0,
            baseline_period: 100,
            fast_period: 10,
            slow_period: 30,
            take_profit: 5.0,
            stop_loss: 3.0,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_vol_sizing_from_config() {
        let config = VolSizingConfig::new(21, 15.0, 150);
        let strategy = VolSizingStrategy::from_config(config);
        assert_eq!(strategy.config().atr_period, 21);
        assert_eq!(strategy.config().base_size_pct, 15.0);
        assert_eq!(strategy.config().baseline_period, 150);
    }

    #[test]
    fn test_vol_sizing_metadata() {
        let strategy = VolSizingStrategy::new(14, 10.0, 100);
        let metadata = strategy.metadata();
        assert_eq!(metadata.name, "Volatility-Adjusted Position Sizing");
        assert_eq!(metadata.category, StrategyCategory::VolatilityBased);
    }

    #[test]
    fn test_calculate_position_size_no_baseline() {
        let mut strategy = VolSizingStrategy::new(14, 10.0, 100);

        // No baseline yet, should return base size
        let size = strategy.calculate_position_size(2.0);
        assert_eq!(size, 10.0);
    }

    #[test]
    fn test_calculate_position_size_with_baseline() {
        let mut strategy = VolSizingStrategy::new(14, 10.0, 100);

        // Build up ATR history to establish baseline
        for _ in 0..100 {
            strategy.atr_history.push_back(2.0);
        }

        // Baseline should be 2.0
        // With current ATR = 2.0 (same as baseline), size should be base size
        let size1 = strategy.calculate_position_size(2.0);
        assert_eq!(size1, 10.0);

        // With current ATR = 4.0 (2x baseline), size should be reduced
        // size = 10 / (4/2)^2 = 10 / 4 = 2.5
        let size2 = strategy.calculate_position_size(4.0);
        assert!((size2 - 2.5).abs() < 0.01);

        // With current ATR = 1.0 (0.5x baseline), size should be increased
        // size = 10 / (1/2)^2 = 10 / 0.25 = 40, but capped at max 25
        let size3 = strategy.calculate_position_size(1.0);
        assert_eq!(size3, 25.0);
    }

    #[test]
    fn test_calculate_position_size_bounds() {
        let mut strategy = VolSizingStrategy::new(14, 10.0, 100);

        // Build up ATR history
        for _ in 0..100 {
            strategy.atr_history.push_back(2.0);
        }

        // Extreme low volatility → should hit max
        let size_low = strategy.calculate_position_size(0.5);
        assert_eq!(size_low, 25.0);

        // Extreme high volatility → should hit min
        let size_high = strategy.calculate_position_size(10.0);
        assert_eq!(size_high, 1.0);
    }

    #[test]
    fn test_vol_sizing_new_instance_clean_state() {
        let strategy = VolSizingStrategy::new(14, 10.0, 100);
        assert_eq!(strategy.last_position, SignalType::Hold);
        assert!(strategy.entry_price.is_none());
        assert!(strategy.entry_size_pct.is_none());
        assert!(strategy.atr_baseline.is_none());
    }
}
