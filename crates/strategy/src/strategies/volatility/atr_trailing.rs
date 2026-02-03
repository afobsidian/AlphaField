//! ATR Trailing Stop Strategy
//!
//! This strategy uses Average True Range (ATR) to calculate dynamic trailing stops
//! that adapt to market volatility. In high volatility, the stop is wider; in low
//! volatility, the stop is tighter. This helps protect profits while allowing room
//! for normal price fluctuations.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Atr, Indicator, Sma};
use alphafield_core::{Bar, PositionState, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for ATR Trailing Stop strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ATRTrailingConfig {
    /// Period for ATR calculation
    pub atr_period: usize,
    /// ATR multiplier for stop distance (higher = wider stops)
    pub atr_multiplier: f64,
    /// Fast period for entry signal (MA crossover)
    pub fast_period: usize,
    /// Slow period for entry signal (MA crossover)
    pub slow_period: usize,
    /// Minimum trailing distance (as % of price, prevents stops too close)
    pub min_trailing_pct: f64,
    /// Take Profit percentage (optional, set to 0.0 to disable)
    pub take_profit: f64,
}

impl ATRTrailingConfig {
    pub fn new(
        atr_period: usize,
        atr_multiplier: f64,
        fast_period: usize,
        slow_period: usize,
    ) -> Self {
        Self {
            atr_period,
            atr_multiplier,
            fast_period,
            slow_period,
            min_trailing_pct: 1.0,
            take_profit: 10.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            atr_period: 14,
            atr_multiplier: 2.0,
            fast_period: 10,
            slow_period: 30,
            min_trailing_pct: 1.0,
            take_profit: 10.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.atr_period == 0 {
            return Err("ATR period must be greater than 0".to_string());
        }
        if self.atr_multiplier <= 0.0 {
            return Err("ATR multiplier must be positive".to_string());
        }
        if self.fast_period == 0 || self.slow_period == 0 {
            return Err("MA periods must be greater than 0".to_string());
        }
        if self.fast_period >= self.slow_period {
            return Err("Fast period must be less than slow period".to_string());
        }
        if self.min_trailing_pct < 0.0 {
            return Err("Minimum trailing % cannot be negative".to_string());
        }
        if self.take_profit < 0.0 {
            return Err("Take profit cannot be negative".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for ATRTrailingConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ATRTrailing(atr_period={}, multiplier={:.1}, ma={}/{}, min_trail={:.1}%, tp={:.1}%)",
            self.atr_period,
            self.atr_multiplier,
            self.fast_period,
            self.slow_period,
            self.min_trailing_pct,
            self.take_profit
        )
    }
}

/// ATR Trailing Stop Strategy
///
/// # Strategy Logic
/// - **Entry**: Fast SMA crosses above Slow SMA (golden cross)
/// - **Stop Loss**: Dynamic trailing stop based on ATR: `stop = price - (ATR * multiplier)`
/// - **Take Profit**: Optional fixed percentage profit target
/// - **Exit**: When price hits trailing stop or take profit
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::volatility::ATRTrailingStrategy;
///
/// let strategy = ATRTrailingStrategy::new(14, 2.0, 10, 30);
/// ```
pub struct ATRTrailingStrategy {
    config: ATRTrailingConfig,
    atr: Atr,
    fast_sma: Sma,
    slow_sma: Sma,
    position: PositionState,
    long_entry_price: Option<f64>,
    short_entry_price: Option<f64>,
    trailing_stop: Option<f64>,
    highest_since_entry: Option<f64>, // Track highest price to trail stop
    lowest_since_entry: Option<f64>,  // Track lowest price to trail stop for shorts
}

impl ATRTrailingStrategy {
    /// Creates a new ATR Trailing Stop strategy
    ///
    /// # Arguments
    /// * `atr_period` - ATR calculation period
    /// * `atr_multiplier` - Multiplier for ATR to determine stop distance
    /// * `fast_period` - Fast MA period for entry signal
    /// * `slow_period` - Slow MA period for entry signal
    pub fn new(
        atr_period: usize,
        atr_multiplier: f64,
        fast_period: usize,
        slow_period: usize,
    ) -> Self {
        let config = ATRTrailingConfig::new(atr_period, atr_multiplier, fast_period, slow_period);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: ATRTrailingConfig) -> Self {
        config.validate().expect("Invalid ATRTrailingConfig");

        Self {
            atr: Atr::new(config.atr_period),
            fast_sma: Sma::new(config.fast_period),
            slow_sma: Sma::new(config.slow_period),
            config,
            position: PositionState::Flat,
            long_entry_price: None,
            short_entry_price: None,
            trailing_stop: None,
            highest_since_entry: None,
            lowest_since_entry: None,
        }
    }

    pub fn config(&self) -> &ATRTrailingConfig {
        &self.config
    }

    /// Calculate dynamic trailing stop based on ATR
    fn calculate_trailing_stop(&self, price: f64, atr_value: f64) -> f64 {
        // ATR-based stop for long position
        let atr_stop = price - (atr_value * self.config.atr_multiplier);

        // Percentage-based minimum stop
        let pct_stop = price * (1.0 - self.config.min_trailing_pct / 100.0);

        // Use the higher (more conservative) stop
        atr_stop.max(pct_stop)
    }

    /// Calculate trailing stop for short position
    fn calculate_trailing_stop_short(&self, price: f64, atr_value: f64) -> f64 {
        // ATR-based stop for short position
        let atr_stop = price + (atr_value * self.config.atr_multiplier);

        // Percentage-based minimum stop
        let pct_stop = price * (1.0 + self.config.min_trailing_pct / 100.0);

        // Use the lower (more conservative) stop for shorts
        atr_stop.min(pct_stop)
    }

    /// Update trailing stop (only moves up for long, never down)
    fn update_trailing_stop(&mut self, price: f64, atr_value: Option<f64>) {
        if let Some(atr) = atr_value {
            let new_stop = self.calculate_trailing_stop(price, atr);

            // Only move stop up (lock in profits), never down
            if let Some(current_stop) = self.trailing_stop {
                if new_stop > current_stop {
                    self.trailing_stop = Some(new_stop);
                }
            } else {
                self.trailing_stop = Some(new_stop);
            }
        }
    }

    /// Update trailing stop for short position (only moves down, never up)
    fn update_trailing_stop_short(&mut self, price: f64, atr_value: Option<f64>) {
        if let Some(atr) = atr_value {
            let new_stop = self.calculate_trailing_stop_short(price, atr);

            // Only move stop down (lock in profits), never up
            if let Some(current_stop) = self.trailing_stop {
                if new_stop < current_stop {
                    self.trailing_stop = Some(new_stop);
                }
            } else {
                self.trailing_stop = Some(new_stop);
            }
        }
    }
}

impl Default for ATRTrailingStrategy {
    fn default() -> Self {
        // Default: 14 ATR, 2.0 multiplier, 10/50 SMA, 2% minimum trailing, 10% TP
        Self::from_config(ATRTrailingConfig::default_config())
    }
}

impl ATRTrailingStrategy {
    /// Reset all position-related state
    fn reset_state(&mut self) {
        self.position = PositionState::Flat;
        self.long_entry_price = None;
        self.short_entry_price = None;
        self.trailing_stop = None;
        self.highest_since_entry = None;
        self.lowest_since_entry = None;
    }
}

impl MetadataStrategy for ATRTrailingStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "ATR Trailing".to_string(),
            category: StrategyCategory::VolatilityBased,
            sub_type: Some("atr_trailing_stop".to_string()),
            description: format!(
                "ATR-based trailing stop strategy using {} period ATR with {:.1}x multiplier.
                Uses {}-period and {}-period SMA for trend filtering.
                Trailing stop only moves up for longs (never down) to lock in profits.
                Trailing stop only moves down for shorts (never up) to lock in profits.
                Minimum trailing distance is {:.1}%. Take profit is {:.1}%.",
                self.config.atr_period,
                self.config.atr_multiplier,
                self.config.fast_period,
                self.config.slow_period,
                self.config.min_trailing_pct,
                self.config.take_profit
            ),
            hypothesis_path: "hypotheses/volatility/atr_trailing.md".to_string(),
            required_indicators: vec!["ATR".to_string(), "SMA".to_string()],
            expected_regimes: vec![
                MarketRegime::Trending,
                MarketRegime::Bull,
                MarketRegime::Bear,
            ],
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

impl Strategy for ATRTrailingStrategy {
    fn name(&self) -> &str {
        "ATR Trailing Stop"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;

        // Update indicators
        let fast_ma = self.fast_sma.update(price)?;
        let slow_ma = self.slow_sma.update(price)?;
        let atr_value = self.atr.update(bar)?;

        // Track highest/lowest price since entry
        if self.position == PositionState::Long {
            self.highest_since_entry = Some(self.highest_since_entry.unwrap_or(price).max(price));
        } else if self.position == PositionState::Short {
            self.lowest_since_entry = Some(self.lowest_since_entry.unwrap_or(price).min(price));
        }

        // ENTRY LOGIC (only when in Flat position)
        if self.position == PositionState::Flat {
            // Check for golden cross (fast SMA crosses above slow SMA)
            let prev_fast = self.fast_sma.value()? - (self.fast_sma.value()? - fast_ma); // approximation
            let prev_slow = self.slow_sma.value()? - (self.slow_sma.value()? - slow_ma);

            // Detect crossover
            // === LONG ENTRY: Golden Cross ===
            if prev_fast <= prev_slow && fast_ma > slow_ma {
                self.position = PositionState::Long;
                self.long_entry_price = Some(price);
                self.highest_since_entry = Some(price);
                self.lowest_since_entry = Some(price);

                // Calculate initial trailing stop
                self.trailing_stop = Some(self.calculate_trailing_stop(price, atr_value));

                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Buy,
                    strength: 1.0,
                    metadata: Some(format!(
                        "Golden Cross Entry: Fast MA ({:.2}) > Slow MA ({:.2}), Stop: {:.2}",
                        fast_ma,
                        slow_ma,
                        self.trailing_stop.unwrap()
                    )),
                }]);
            }

            // === SHORT ENTRY: Death Cross ===
            if prev_fast >= prev_slow && fast_ma < slow_ma {
                self.position = PositionState::Short;
                self.short_entry_price = Some(price);
                self.highest_since_entry = Some(price);
                self.lowest_since_entry = Some(price);

                // Calculate initial trailing stop
                self.trailing_stop = Some(self.calculate_trailing_stop_short(price, atr_value));

                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: Some(format!(
                        "Death Cross Entry: Fast MA ({:.2}) < Slow MA ({:.2}), Stop: {:.2}",
                        fast_ma,
                        slow_ma,
                        self.trailing_stop.unwrap()
                    )),
                }]);
            }
        }

        // EXIT LOGIC (only when in position)
        if self.position != PositionState::Flat {
            // === LONG POSITION EXIT LOGIC ===
            if self.position == PositionState::Long {
                if let Some(entry) = self.long_entry_price {
                    let profit_pct = (price - entry) / entry * 100.0;

                    // Update trailing stop based on current highest price
                    self.update_trailing_stop(
                        self.highest_since_entry.unwrap_or(price),
                        Some(atr_value),
                    );

                    // Exit 1: Trailing stop hit
                    if let Some(stop) = self.trailing_stop {
                        if price <= stop {
                            self.reset_state();
                            return Some(vec![Signal {
                                timestamp: bar.timestamp,
                                symbol: "UNKNOWN".to_string(),
                                signal_type: SignalType::Sell,
                                strength: 1.0,
                                metadata: Some(format!(
                                    "Trailing Stop Exit: {:.2}% profit, Stop at {:.2}",
                                    profit_pct, stop
                                )),
                            }]);
                        }
                    }

                    // Exit 2: Take profit (if enabled)
                    if self.config.take_profit > 0.0 && profit_pct >= self.config.take_profit {
                        self.reset_state();
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 1.0,
                            metadata: Some(format!("Take Profit Exit: {:.1}% profit", profit_pct)),
                        }]);
                    }

                    // Exit 3: Death cross (fast SMA crosses below slow SMA)
                    let prev_fast = self.fast_sma.value()? - (self.fast_sma.value()? - fast_ma);
                    let prev_slow = self.slow_sma.value()? - (self.slow_sma.value()? - slow_ma);

                    if prev_fast >= prev_slow && fast_ma < slow_ma {
                        self.reset_state();
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 1.0,
                            metadata: Some(format!(
                                "Death Cross Exit: {:.1}% profit, Fast MA ({:.2}) < Slow MA ({:.2})",
                                profit_pct, fast_ma, slow_ma
                            )),
                        }]);
                    }
                }
            }
            // === SHORT POSITION EXIT LOGIC ===
            else if self.position == PositionState::Short {
                if let Some(entry) = self.short_entry_price {
                    let profit_pct = (entry - price) / entry * 100.0;

                    // Update trailing stop based on current lowest price
                    self.update_trailing_stop_short(
                        self.lowest_since_entry.unwrap_or(price),
                        Some(atr_value),
                    );

                    // Exit 1: Trailing stop hit
                    if let Some(stop) = self.trailing_stop {
                        if price >= stop {
                            self.reset_state();
                            return Some(vec![Signal {
                                timestamp: bar.timestamp,
                                symbol: "UNKNOWN".to_string(),
                                signal_type: SignalType::Buy,
                                strength: 1.0,
                                metadata: Some(format!(
                                    "Trailing Stop Exit: {:.2}% profit, Stop at {:.2}",
                                    profit_pct, stop
                                )),
                            }]);
                        }
                    }

                    // Exit 2: Take profit (if enabled)
                    if self.config.take_profit > 0.0 && profit_pct >= self.config.take_profit {
                        self.reset_state();
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Buy,
                            strength: 1.0,
                            metadata: Some(format!("Take Profit Exit: {:.1}% profit", profit_pct)),
                        }]);
                    }

                    // Exit 3: Golden cross (fast SMA crosses above slow SMA)
                    let prev_fast = self.fast_sma.value()? - (self.fast_sma.value()? - fast_ma);
                    let prev_slow = self.slow_sma.value()? - (self.slow_sma.value()? - slow_ma);

                    if prev_fast <= prev_slow && fast_ma > slow_ma {
                        self.reset_state();
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Buy,
                            strength: 1.0,
                            metadata: Some(format!(
                                "Golden Cross Exit: {:.1}% profit, Fast MA ({:.2}) > Slow MA ({:.2})",
                                profit_pct, fast_ma, slow_ma
                            )),
                        }]);
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
    fn test_atr_trailing_creation() {
        let strategy = ATRTrailingStrategy::new(14, 2.0, 10, 30);
        assert_eq!(strategy.config().atr_period, 14);
        assert_eq!(strategy.config().atr_multiplier, 2.0);
    }

    #[test]
    fn test_atr_trailing_config_valid() {
        let config = ATRTrailingConfig::new(14, 2.0, 10, 30);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_atr_trailing_invalid_config() {
        let config = ATRTrailingConfig {
            atr_period: 0,
            atr_multiplier: 2.0,
            fast_period: 10,
            slow_period: 30,
            min_trailing_pct: 1.0,
            take_profit: 10.0,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_atr_trailing_from_config() {
        let config = ATRTrailingConfig::new(14, 2.5, 5, 20);
        let strategy = ATRTrailingStrategy::from_config(config);
        assert_eq!(strategy.config().atr_multiplier, 2.5);
        assert_eq!(strategy.config().fast_period, 5);
    }

    #[test]
    fn test_atr_trailing_metadata() {
        let strategy = ATRTrailingStrategy::new(14, 2.0, 10, 30);
        let metadata = strategy.metadata();
        assert_eq!(metadata.name, "ATR Trailing");
        assert_eq!(metadata.category, StrategyCategory::VolatilityBased);
    }

    #[test]
    fn test_calculate_trailing_stop() {
        let strategy = ATRTrailingStrategy::new(14, 2.0, 10, 30);
        let price = 100.0;
        let atr_value = 2.0;

        // ATR-based stop: 100 - (2.0 * 2.0) = 96.0
        // Min trailing stop: 100 * (1 - 0.01) = 99.0
        // Should use the higher (more conservative) = 99.0
        let stop = strategy.calculate_trailing_stop(price, atr_value);
        assert_eq!(stop, 99.0);
    }

    #[test]
    fn test_update_trailing_stop_moves_up() {
        let mut strategy = ATRTrailingStrategy::new(14, 2.0, 10, 30);
        strategy.trailing_stop = Some(95.0);

        // Price goes up, stop should move up
        strategy.update_trailing_stop(100.0, Some(2.0));
        assert!(strategy.trailing_stop.unwrap() > 95.0);
    }

    #[test]
    fn test_update_trailing_stop_never_down() {
        let mut strategy = ATRTrailingStrategy::new(14, 2.0, 10, 30);

        // Set initial stop
        strategy.update_trailing_stop(100.0, Some(2.0));
        let initial_stop = strategy.trailing_stop.unwrap();

        // Price goes down, stop should NOT move down
        strategy.update_trailing_stop(95.0, Some(2.0));
        assert_eq!(strategy.trailing_stop.unwrap(), initial_stop);
    }

    #[test]
    fn test_atr_trailing_new_instance_clean_state() {
        let strategy1 = ATRTrailingStrategy::new(14, 2.0, 10, 30);
        assert_eq!(strategy1.position, PositionState::Flat);
        assert!(strategy1.long_entry_price.is_none());
        assert!(strategy1.short_entry_price.is_none());
        assert!(strategy1.trailing_stop.is_none());
    }
}
