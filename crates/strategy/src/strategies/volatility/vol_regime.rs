//! Volatility Regime Strategy
//!
//! This strategy adapts its behavior based on the current market volatility regime.
//! It classifies market conditions into low, medium, and high volatility using ATR
//! percentile ranking. Different entry conditions are used for each regime:
//! - Low volatility: Breakout signals (expect volatility expansion)
//! - Medium volatility: Trend-following signals
//! - High volatility: Mean reversion signals (expect mean reversion)

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Atr, Indicator, Rsi, Sma};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Volatility regime classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolatilityRegime {
    Low,    // Calm market, expect expansion
    Medium, // Normal market, trend following
    High,   // Volatile market, expect mean reversion
}

impl fmt::Display for VolatilityRegime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VolatilityRegime::Low => write!(f, "Low"),
            VolatilityRegime::Medium => write!(f, "Medium"),
            VolatilityRegime::High => write!(f, "High"),
        }
    }
}

/// Configuration for Volatility Regime strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolRegimeConfig {
    /// Period for ATR calculation
    pub atr_period: usize,
    /// Period for percentile calculation (historical ATR window)
    pub percentile_period: usize,
    /// Low volatility threshold (percentile, 0.0-1.0)
    pub low_threshold: f64,
    /// High volatility threshold (percentile, 0.0-1.0)
    pub high_threshold: f64,
    /// Fast MA for trend following
    pub fast_period: usize,
    /// Slow MA for trend following
    pub slow_period: usize,
    /// RSI period for mean reversion (high vol)
    pub rsi_period: usize,
    /// RSI oversold threshold for high vol regime
    pub rsi_oversold: f64,
    /// Lookback period for breakout (low vol regime)
    pub breakout_lookback: usize,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl VolRegimeConfig {
    pub fn new(atr_period: usize, percentile_period: usize) -> Self {
        Self {
            atr_period,
            percentile_period,
            low_threshold: 0.25,  // Bottom 25%
            high_threshold: 0.75, // Top 25%
            fast_period: 10,
            slow_period: 30,
            rsi_period: 14,
            rsi_oversold: 30.0,
            breakout_lookback: 20,
            take_profit: 5.0,
            stop_loss: 3.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            atr_period: 14,
            percentile_period: 100,
            low_threshold: 0.25,
            high_threshold: 0.75,
            fast_period: 10,
            slow_period: 30,
            rsi_period: 14,
            rsi_oversold: 30.0,
            breakout_lookback: 20,
            take_profit: 5.0,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.atr_period == 0 {
            return Err("ATR period must be greater than 0".to_string());
        }
        if self.percentile_period < 20 {
            return Err("Percentile period must be at least 20".to_string());
        }
        if self.low_threshold <= 0.0 || self.low_threshold >= 1.0 {
            return Err("Low threshold must be between 0 and 1".to_string());
        }
        if self.high_threshold <= 0.0 || self.high_threshold >= 1.0 {
            return Err("High threshold must be between 0 and 1".to_string());
        }
        if self.high_threshold <= self.low_threshold {
            return Err("High threshold must be greater than low threshold".to_string());
        }
        if self.fast_period == 0 || self.slow_period == 0 {
            return Err("MA periods must be greater than 0".to_string());
        }
        if self.fast_period >= self.slow_period {
            return Err("Fast period must be less than slow period".to_string());
        }
        if self.rsi_period == 0 {
            return Err("RSI period must be greater than 0".to_string());
        }
        if self.rsi_oversold <= 0.0 || self.rsi_oversold >= 100.0 {
            return Err("RSI oversold must be between 0 and 100".to_string());
        }
        if self.breakout_lookback == 0 {
            return Err("Breakout lookback must be greater than 0".to_string());
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

impl fmt::Display for VolRegimeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VolRegime(atr_period={}, pct_period={}, low_thresh={:.2}, high_thresh={:.2}, tp={:.1}%, sl={:.1}%)",
            self.atr_period,
            self.percentile_period,
            self.low_threshold,
            self.high_threshold,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// Volatility Regime Strategy
///
/// # Strategy Logic
/// - **Regime Detection**: Classify volatility as Low, Medium, or High based on ATR percentile
/// - **Low Vol Regime**: Breakout signals - buy when price breaks above recent high
/// - **Medium Vol Regime**: Trend following - buy on golden cross (fast MA > slow MA)
/// - **High Vol Regime**: Mean reversion - buy when RSI is oversold
/// - **Exit**: Take profit or stop loss percentage
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::volatility::VolRegimeStrategy;
///
/// let strategy = VolRegimeStrategy::new(14, 100);
/// ```
pub struct VolRegimeStrategy {
    config: VolRegimeConfig,
    atr: Atr,
    atr_history: VecDeque<f64>,
    fast_sma: Sma,
    slow_sma: Sma,
    rsi: Rsi,
    price_history: VecDeque<f64>,
    last_position: SignalType,
    entry_price: Option<f64>,
    current_regime: VolatilityRegime,
}

impl Default for VolRegimeStrategy {
    fn default() -> Self {
        Self::from_config(VolRegimeConfig::default_config())
    }
}

impl VolRegimeStrategy {
    /// Creates a new Volatility Regime strategy
    ///
    /// # Arguments
    /// * `atr_period` - ATR calculation period
    /// * `percentile_period` - Historical ATR window for percentile calculation
    pub fn new(atr_period: usize, percentile_period: usize) -> Self {
        let config = VolRegimeConfig::new(atr_period, percentile_period);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: VolRegimeConfig) -> Self {
        config.validate().expect("Invalid VolRegimeConfig");

        Self {
            atr: Atr::new(config.atr_period),
            atr_history: VecDeque::with_capacity(config.percentile_period),
            fast_sma: Sma::new(config.fast_period),
            slow_sma: Sma::new(config.slow_period),
            rsi: Rsi::new(config.rsi_period),
            price_history: VecDeque::with_capacity(config.breakout_lookback),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            current_regime: VolatilityRegime::Medium,
        }
    }

    pub fn config(&self) -> &VolRegimeConfig {
        &self.config
    }

    /// Calculate percentile rank of current ATR
    fn calculate_atr_percentile(&self) -> Option<f64> {
        if self.atr_history.is_empty() {
            return None;
        }

        if let Some(current_atr) = self.atr.value() {
            let count = self
                .atr_history
                .iter()
                .filter(|&&v| v < current_atr)
                .count();
            Some(count as f64 / self.atr_history.len() as f64)
        } else {
            None
        }
    }

    /// Classify volatility regime based on ATR percentile
    fn classify_regime(&self, atr_percentile: f64) -> VolatilityRegime {
        if atr_percentile < self.config.low_threshold {
            VolatilityRegime::Low
        } else if atr_percentile > self.config.high_threshold {
            VolatilityRegime::High
        } else {
            VolatilityRegime::Medium
        }
    }

    /// Check for breakout entry (low volatility regime)
    fn check_breakout_entry(&self, price: f64) -> bool {
        if self.price_history.len() < self.config.breakout_lookback {
            return false;
        }

        // Get highest price in lookback period
        let lookback_high = *self
            .price_history
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        // Entry when price breaks above lookback high
        price > lookback_high
    }

    /// Check for trend-following entry (medium volatility regime)
    fn check_trend_entry(&self) -> Option<bool> {
        let fast_ma = self.fast_sma.value()?;
        let slow_ma = self.slow_sma.value()?;

        // Entry on golden cross
        Some(fast_ma > slow_ma)
    }

    /// Check for mean reversion entry (high volatility regime)
    fn check_reversion_entry(&self) -> Option<bool> {
        let rsi_value = self.rsi.value()?;

        // Entry when RSI is oversold
        Some(rsi_value <= self.config.rsi_oversold)
    }
}

impl MetadataStrategy for VolRegimeStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Volatility Regime".to_string(),
            category: StrategyCategory::VolatilityBased,
            sub_type: Some("regime_adaptive".to_string()),
            description: format!(
                "Volatility regime-adaptive strategy using {} period ATR with {} period percentile window. \
                Low vol (< {:.0}%): Breakout entries. Medium vol ({:.0}%-{:.0}%): Trend following ({} / {} MA crossover). \
                High vol (> {:.0}%): Mean reversion (RSI <= {:.0}). Uses {:.1}% TP and {:.1}% SL.",
                self.config.atr_period,
                self.config.percentile_period,
                self.config.low_threshold * 100.0,
                self.config.low_threshold * 100.0,
                self.config.high_threshold * 100.0,
                self.config.fast_period,
                self.config.slow_period,
                self.config.high_threshold * 100.0,
                self.config.rsi_oversold,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/volatility/vol_regime.md".to_string(),
            required_indicators: vec![
                "ATR".to_string(),
                "SMA".to_string(),
                "RSI".to_string(),
                "Price".to_string(),
            ],
            expected_regimes: vec![
                MarketRegime::Bull,
                MarketRegime::Bear,
                MarketRegime::Sideways,
                MarketRegime::Trending,
                MarketRegime::Ranging,
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
        StrategyCategory::VolatilityBased
    }
}

impl Strategy for VolRegimeStrategy {
    fn name(&self) -> &str {
        "Volatility Regime"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;

        // Update indicators
        let atr_value = self.atr.update(bar)?;
        let fast_ma = self.fast_sma.update(price);
        let slow_ma = self.slow_sma.update(price);
        let rsi_value = self.rsi.update(price);

        // Update ATR history for percentile calculation
        self.atr_history.push_back(atr_value);
        if self.atr_history.len() > self.config.percentile_period {
            self.atr_history.pop_front();
        }

        // Update price history for breakout detection
        self.price_history.push_back(price);
        if self.price_history.len() > self.config.breakout_lookback {
            self.price_history.pop_front();
        }

        // Classify current volatility regime
        let atr_percentile = self.calculate_atr_percentile().unwrap_or(0.5);
        let regime = self.classify_regime(atr_percentile);
        self.current_regime = regime;

        // ENTRY LOGIC (only when not in position)
        if self.last_position == SignalType::Hold {
            let entry_reason = match regime {
                VolatilityRegime::Low => {
                    // Low volatility: Breakout entry
                    if self.check_breakout_entry(price) {
                        Some("Low Vol Breakout".to_string())
                    } else {
                        None
                    }
                }
                VolatilityRegime::Medium => {
                    // Medium volatility: Trend following
                    if let Some(is_trending) = self.check_trend_entry() {
                        if is_trending {
                            Some("Medium Vol Trend Following".to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                VolatilityRegime::High => {
                    // High volatility: Mean reversion
                    if let Some(is_oversold) = self.check_reversion_entry() {
                        if is_oversold {
                            Some("High Vol Mean Reversion".to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            };

            if let Some(reason) = entry_reason {
                self.last_position = SignalType::Buy;
                self.entry_price = Some(price);

                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Buy,
                    strength: 1.0,
                    metadata: Some(format!(
                        "{} Entry at {:.2}, ATR Percentile: {:.1}%",
                        reason,
                        price,
                        atr_percentile * 100.0
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

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Take Profit: {:.1}% profit, Regime: {}",
                            profit_pct, regime
                        )),
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
                        metadata: Some(format!(
                            "Stop Loss: {:.1}% loss, Regime: {}",
                            profit_pct, regime
                        )),
                    }]);
                }

                // Regime-specific exit conditions
                match regime {
                    VolatilityRegime::Low => {
                        // In low vol, exit if momentum stalls (price below MA)
                        if let Some(slow_ma_val) = slow_ma {
                            if price < slow_ma_val {
                                self.last_position = SignalType::Hold;
                                self.entry_price = None;

                                return Some(vec![Signal {
                                    timestamp: bar.timestamp,
                                    symbol: "UNKNOWN".to_string(),
                                    signal_type: SignalType::Sell,
                                    strength: 0.5,
                                    metadata: Some(format!(
                                        "Low Vol Exit: Price {:.2} < Slow MA {:.2}",
                                        price, slow_ma_val
                                    )),
                                }]);
                            }
                        }
                    }
                    VolatilityRegime::Medium => {
                        // In medium vol, exit on death cross
                        if let (Some(fast_ma_val), Some(slow_ma_val)) = (fast_ma, slow_ma) {
                            if fast_ma_val < slow_ma_val {
                                self.last_position = SignalType::Hold;
                                self.entry_price = None;

                                return Some(vec![Signal {
                                    timestamp: bar.timestamp,
                                    symbol: "UNKNOWN".to_string(),
                                    signal_type: SignalType::Sell,
                                    strength: 0.5,
                                    metadata: Some(format!(
                                        "Medium Vol Exit: Death Cross ({:.2} < {:.2})",
                                        fast_ma_val, slow_ma_val
                                    )),
                                }]);
                            }
                        }
                    }
                    VolatilityRegime::High => {
                        // In high vol, exit when RSI is overbought
                        if let Some(rsi_val) = rsi_value {
                            if rsi_val >= (100.0 - self.config.rsi_oversold) {
                                self.last_position = SignalType::Hold;
                                self.entry_price = None;

                                return Some(vec![Signal {
                                    timestamp: bar.timestamp,
                                    symbol: "UNKNOWN".to_string(),
                                    signal_type: SignalType::Sell,
                                    strength: 0.5,
                                    metadata: Some(format!(
                                        "High Vol Exit: RSI {:.1} overbought (>{:.0})",
                                        rsi_val,
                                        100.0 - self.config.rsi_oversold
                                    )),
                                }]);
                            }
                        }
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
    fn test_vol_regime_creation() {
        let strategy = VolRegimeStrategy::new(14, 100);
        assert_eq!(strategy.config().atr_period, 14);
        assert_eq!(strategy.config().percentile_period, 100);
    }

    #[test]
    fn test_vol_regime_config_valid() {
        let config = VolRegimeConfig::new(14, 100);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_vol_regime_invalid_config() {
        let config = VolRegimeConfig {
            atr_period: 0,
            percentile_period: 100,
            low_threshold: 0.25,
            high_threshold: 0.75,
            fast_period: 10,
            slow_period: 30,
            rsi_period: 14,
            rsi_oversold: 30.0,
            breakout_lookback: 20,
            take_profit: 5.0,
            stop_loss: 3.0,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_vol_regime_from_config() {
        let config = VolRegimeConfig::new(21, 150);
        let strategy = VolRegimeStrategy::from_config(config);
        assert_eq!(strategy.config().atr_period, 21);
        assert_eq!(strategy.config().percentile_period, 150);
    }

    #[test]
    fn test_vol_regime_metadata() {
        let strategy = VolRegimeStrategy::new(14, 100);
        let metadata = strategy.metadata();
        assert_eq!(metadata.name, "Volatility Regime");
        assert_eq!(metadata.category, StrategyCategory::VolatilityBased);
    }

    #[test]
    fn test_classify_regime() {
        let config = VolRegimeConfig::new(14, 100);
        let strategy = VolRegimeStrategy::from_config(config);

        // Low regime (below 25th percentile)
        assert_eq!(strategy.classify_regime(0.20), VolatilityRegime::Low);

        // Medium regime (between 25th and 75th percentile)
        assert_eq!(strategy.classify_regime(0.50), VolatilityRegime::Medium);

        // High regime (above 75th percentile)
        assert_eq!(strategy.classify_regime(0.80), VolatilityRegime::High);
    }

    #[test]
    fn test_vol_regime_new_instance_clean_state() {
        let strategy = VolRegimeStrategy::new(14, 100);
        assert_eq!(strategy.last_position, SignalType::Hold);
        assert!(strategy.entry_price.is_none());
        assert_eq!(strategy.current_regime, VolatilityRegime::Medium);
    }

    #[test]
    fn test_check_breakout_entry() {
        let mut strategy = VolRegimeStrategy::new(14, 100);

        // Build price history
        for i in 0..20 {
            strategy.price_history.push_back(100.0 + i as f64);
        }

        // No breakout - price equals high
        assert!(!strategy.check_breakout_entry(119.0));

        // Breakout - price above high
        assert!(strategy.check_breakout_entry(120.0));
    }

    #[test]
    fn test_volatility_regime_display() {
        assert_eq!(format!("{}", VolatilityRegime::Low), "Low");
        assert_eq!(format!("{}", VolatilityRegime::Medium), "Medium");
        assert_eq!(format!("{}", VolatilityRegime::High), "High");
    }
}
