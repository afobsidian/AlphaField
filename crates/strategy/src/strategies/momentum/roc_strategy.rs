//! Rate of Change (ROC) Momentum Strategy
//!
//! Measures price momentum using the rate of change indicator.
//! Enters on positive accelerating ROC, exits on negative or decelerating ROC.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use alphafield_core::{Bar, PositionState, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Configuration for ROC strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RocConfig {
    /// ROC calculation period
    pub roc_period: usize,
    /// Entry threshold (ROC percentage)
    pub entry_threshold: f64,
    /// Exit threshold (ROC percentage)
    pub exit_threshold: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl RocConfig {
    pub fn new(
        roc_period: usize,
        entry_threshold: f64,
        exit_threshold: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            roc_period,
            entry_threshold,
            exit_threshold,
            take_profit,
            stop_loss,
        }
    }

    pub fn default_config() -> Self {
        Self {
            roc_period: 10,
            entry_threshold: 2.0,
            exit_threshold: -1.0,
            take_profit: 5.0,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.roc_period == 0 {
            return Err("ROC period must be greater than 0".to_string());
        }
        if self.entry_threshold <= self.exit_threshold {
            return Err("Entry threshold must be greater than exit threshold".to_string());
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

impl fmt::Display for RocConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ROC(period={}, entry={:.1}%, exit={:.1}%, tp={:.1}%, sl={:.1}%)",
            self.roc_period,
            self.entry_threshold,
            self.exit_threshold,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// Rate of Change Momentum Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: ROC is positive and accelerating (crosses above entry threshold)
/// - **Sell Signal**: ROC turns negative or decelerates (crosses below exit threshold)
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::momentum::RocStrategy;
///
/// let _strategy = RocStrategy::new(10, 2.0, -1.0, 5.0, 3.0);
/// ```
pub struct RocStrategy {
    config: RocConfig,
    prices: VecDeque<f64>,
    position: PositionState,
    long_entry_price: Option<f64>,
    short_entry_price: Option<f64>,
    last_roc: Option<f64>,
}

impl Default for RocStrategy {
    fn default() -> Self {
        // Default: 10-period ROC, 2% entry, -1% exit, 5% TP, 3% SL
        Self::from_config(RocConfig::default_config())
    }
}

impl RocStrategy {
    /// Creates a new ROC strategy
    pub fn new(
        roc_period: usize,
        entry_threshold: f64,
        exit_threshold: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        let config = RocConfig::new(
            roc_period,
            entry_threshold,
            exit_threshold,
            take_profit,
            stop_loss,
        );
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: RocConfig) -> Self {
        config.validate().expect("Invalid RocConfig");

        Self {
            prices: VecDeque::with_capacity(config.roc_period + 1),
            config,
            position: PositionState::Flat,
            long_entry_price: None,
            short_entry_price: None,
            last_roc: None,
        }
    }

    pub fn config(&self) -> &RocConfig {
        &self.config
    }

    /// Reset all position-related state
    fn reset_state(&mut self) {
        self.position = PositionState::Flat;
        self.long_entry_price = None;
        self.short_entry_price = None;
    }

    /// Calculate Rate of Change
    fn calculate_roc(&self, current_price: f64) -> Option<f64> {
        if self.prices.len() < self.config.roc_period {
            return None;
        }

        let old_price = self.prices[0];
        if old_price == 0.0 {
            return None;
        }

        // ROC = ((current_price - old_price) / old_price) * 100
        Some(((current_price - old_price) / old_price) * 100.0)
    }
}

impl MetadataStrategy for RocStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "ROC Momentum".to_string(),
            category: StrategyCategory::Momentum,
            sub_type: Some("rate_of_change".to_string()),
            description: format!(
                "Rate of Change momentum strategy using {} period ROC. Enters when ROC crosses above {:.1}%
                (positive momentum), exits when ROC crosses below {:.1}% (negative momentum).
                Uses {:.1}% TP and {:.1}% SL.",
                self.config.roc_period,
                self.config.entry_threshold,
                self.config.exit_threshold,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/momentum/roc_strategy.md".to_string(),
            required_indicators: vec!["ROC".to_string()],
            expected_regimes: vec![
                MarketRegime::Bull,
                MarketRegime::Trending,
                MarketRegime::Bear,
            ],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.20,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::Momentum
    }
}

impl Strategy for RocStrategy {
    fn name(&self) -> &str {
        "ROC Momentum"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;

        // Add current price to history
        self.prices.push_back(price);
        if self.prices.len() > self.config.roc_period {
            self.prices.pop_front();
        }

        // Calculate ROC
        let roc_val = self.calculate_roc(price)?;
        let prev_roc = self.last_roc;

        // Update state for next bar
        self.last_roc = Some(roc_val);

        // EXIT LOGIC FIRST (only when in position)
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
                            metadata: Some(format!("Take Profit: {:.1}%", profit_pct)),
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
                            metadata: Some(format!("Stop Loss: {:.1}%", profit_pct)),
                        });
                        return Some(signals);
                    }

                    // Exit on ROC crossing below exit threshold
                    if let Some(prev) = prev_roc {
                        if prev >= self.config.exit_threshold
                            && roc_val < self.config.exit_threshold
                        {
                            self.reset_state();
                            signals.push(Signal {
                                timestamp: bar.timestamp,
                                symbol: "UNKNOWN".to_string(),
                                signal_type: SignalType::Sell,
                                strength: 0.8,
                                metadata: Some(format!(
                                    "ROC Exit: ROC {:.2}% crossed below {:.1}%",
                                    roc_val, self.config.exit_threshold
                                )),
                            });
                            return Some(signals);
                        }
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
                            metadata: Some(format!("Take Profit: {:.1}%", profit_pct)),
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
                            metadata: Some(format!("Stop Loss: {:.1}%", profit_pct)),
                        });
                        return Some(signals);
                    }

                    // Exit on ROC crossing above -exit_threshold (positive momentum)
                    if let Some(prev) = prev_roc {
                        if prev <= -self.config.exit_threshold
                            && roc_val > -self.config.exit_threshold
                        {
                            self.reset_state();
                            signals.push(Signal {
                                timestamp: bar.timestamp,
                                symbol: "UNKNOWN".to_string(),
                                signal_type: SignalType::Buy,
                                strength: 0.8,
                                metadata: Some(format!(
                                    "ROC Exit: ROC {:.2}% crossed above {:.1}%",
                                    roc_val, -self.config.exit_threshold
                                )),
                            });
                            return Some(signals);
                        }
                    }
                }
            }
        }

        // ENTRY LOGIC - Only enter if in Flat position
        if self.position == PositionState::Flat {
            if let Some(prev) = prev_roc {
                // === LONG ENTRY: ROC crosses above entry threshold ===
                if prev <= self.config.entry_threshold && roc_val > self.config.entry_threshold {
                    // Calculate signal strength based on ROC magnitude
                    let strength = (roc_val / (self.config.entry_threshold * 2.0)).clamp(0.5, 1.0);

                    self.position = PositionState::Long;
                    self.long_entry_price = Some(price);
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength,
                        metadata: Some(format!(
                            "ROC Entry: ROC {:.2}% crossed above {:.1}%",
                            roc_val, self.config.entry_threshold
                        )),
                    }]);
                }
                // === SHORT ENTRY: ROC crosses below -entry threshold ===
                else if prev >= -self.config.entry_threshold
                    && roc_val < -self.config.entry_threshold
                {
                    // Calculate signal strength based on ROC magnitude
                    let strength =
                        (roc_val.abs() / (self.config.entry_threshold * 2.0)).clamp(0.5, 1.0);

                    self.position = PositionState::Short;
                    self.short_entry_price = Some(price);
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength,
                        metadata: Some(format!(
                            "ROC Entry: ROC {:.2}% crossed below {:.1}%",
                            roc_val, -self.config.entry_threshold
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
    fn test_roc_creation() {
        let strategy = RocStrategy::new(10, 2.0, -1.0, 5.0, 3.0);
        assert_eq!(strategy.name(), "ROC Momentum");
    }

    #[test]
    fn test_config_validation() {
        let config = RocConfig::default_config();
        assert!(config.validate().is_ok());

        let invalid_config = RocConfig::new(10, -1.0, 2.0, 5.0, 3.0); // entry < exit
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_metadata() {
        let strategy = RocStrategy::new(10, 2.0, -1.0, 5.0, 3.0);
        let metadata = strategy.metadata();
        assert_eq!(metadata.category, StrategyCategory::Momentum);
        assert_eq!(metadata.name, "ROC Momentum");
    }

    #[test]
    fn test_roc_calculation() {
        let _strategy = RocStrategy::new(10, 2.0, -1.0, 5.0, 3.0);

        // ROC calculation test
        // If price goes from 100 to 110 over 10 periods, ROC = (110-100)/100 * 100 = 10%
        // Need to build up history first
    }

    #[test]
    fn test_no_signal_without_warmup() {
        let mut strategy = RocStrategy::new(10, 2.0, -1.0, 5.0, 3.0);

        // First 9 bars should not generate signals (ROC needs warmup)
        for i in 1..10 {
            let bar = create_test_bar(100.0 + i as f64);
            let signal = strategy.on_bar(&bar);
            assert!(signal.is_none());
        }
    }
}
