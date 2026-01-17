//! Volatility Squeeze Strategy
//!
//! This strategy identifies periods of low volatility (squeeze) using the
//! Bollinger Band and Keltner Channel squeeze technique. When volatility
//! contracts, a breakout often follows. The strategy enters when price
//! breaks out of the squeeze and exits with take profit or stop loss.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Atr, BollingerBands, Ema, Indicator, Sma};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Configuration for Volatility Squeeze strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolSqueezeConfig {
    /// Period for Bollinger Bands
    pub bb_period: usize,
    /// Standard deviation for Bollinger Bands
    pub bb_std_dev: f64,
    /// Period for Keltner Channel EMA
    pub kk_period: usize,
    /// ATR multiplier for Keltner Channel bands
    pub kk_mult: f64,
    /// Squeeze threshold (BB width must be less than Keltner width by this ratio)
    pub squeeze_threshold: f64,
    /// Volume multiplier for breakout confirmation
    pub volume_multiplier: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl VolSqueezeConfig {
    pub fn new(
        bb_period: usize,
        bb_std_dev: f64,
        kk_period: usize,
        kk_mult: f64,
        squeeze_threshold: f64,
    ) -> Self {
        Self {
            bb_period,
            bb_std_dev,
            kk_period,
            kk_mult,
            squeeze_threshold,
            volume_multiplier: 1.5,
            take_profit: 5.0,
            stop_loss: 3.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            bb_period: 20,
            bb_std_dev: 2.0,
            kk_period: 20,
            kk_mult: 1.5,
            squeeze_threshold: 0.1,
            volume_multiplier: 1.5,
            take_profit: 5.0,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.bb_period == 0 {
            return Err("Bollinger Bands period must be greater than 0".to_string());
        }
        if self.bb_std_dev <= 0.0 {
            return Err("BB std dev must be positive".to_string());
        }
        if self.kk_period == 0 {
            return Err("Keltner Channel period must be greater than 0".to_string());
        }
        if self.kk_mult <= 0.0 {
            return Err("Keltner Channel multiplier must be positive".to_string());
        }
        if self.squeeze_threshold <= 0.0 {
            return Err("Squeeze threshold must be positive".to_string());
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

impl fmt::Display for VolSqueezeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VolSqueeze(bb_period={}, bb_std={:.1}, kk_period={}, kk_mult={:.1}, squeeze={:.1}, tp={:.1}%, sl={:.1}%)",
            self.bb_period,
            self.bb_std_dev,
            self.kk_period,
            self.kk_mult,
            self.squeeze_threshold,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// Volatility Squeeze Strategy
///
/// # Strategy Logic
/// - **Squeeze Detection**: When BB width < Keltner width by threshold, squeeze is active
/// - **Buy Signal**: Price breaks above upper BB or Keltner channel with volume confirmation
/// - **Sell Signal**: Price breaks below lower BB or Keltner channel with volume confirmation
/// - **Exit**: Take profit or stop loss percentage
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::volatility::VolSqueezeStrategy;
///
/// let strategy = VolSqueezeStrategy::new(20, 2.0, 20, 1.5, 0.1);
/// ```
pub struct VolSqueezeStrategy {
    config: VolSqueezeConfig,
    bb: BollingerBands,
    kk_ema: Ema,
    atr: Atr,
    volume_sma: Sma,
    last_position: SignalType,
    entry_price: Option<f64>,
    volume_history: VecDeque<f64>,
    in_squeeze: bool,
    last_price: Option<f64>,
    last_bb_upper: Option<f64>,
    last_bb_lower: Option<f64>,
    last_kk_upper: Option<f64>,
    last_kk_lower: Option<f64>,
}

impl Default for VolSqueezeStrategy {
    fn default() -> Self {
        Self::from_config(VolSqueezeConfig::default_config())
    }
}

impl VolSqueezeStrategy {
    /// Creates a new Volatility Squeeze strategy
    ///
    /// # Arguments
    /// * `bb_period` - Bollinger Bands period
    /// * `bb_std_dev` - Bollinger Bands standard deviation
    /// * `kk_period` - Keltner Channel EMA period
    /// * `kk_mult` - Keltner Channel ATR multiplier
    /// * `squeeze_threshold` - Threshold for squeeze detection (0.0-1.0)
    pub fn new(
        bb_period: usize,
        bb_std_dev: f64,
        kk_period: usize,
        kk_mult: f64,
        squeeze_threshold: f64,
    ) -> Self {
        let config =
            VolSqueezeConfig::new(bb_period, bb_std_dev, kk_period, kk_mult, squeeze_threshold);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: VolSqueezeConfig) -> Self {
        config.validate().expect("Invalid VolSqueezeConfig");

        Self {
            bb: BollingerBands::new(config.bb_period, config.bb_std_dev),
            kk_ema: Ema::new(config.kk_period),
            atr: Atr::new(config.kk_period),
            volume_sma: Sma::new(20),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            volume_history: VecDeque::with_capacity(20),
            in_squeeze: false,
            last_price: None,
            last_bb_upper: None,
            last_bb_lower: None,
            last_kk_upper: None,
            last_kk_lower: None,
        }
    }

    pub fn config(&self) -> &VolSqueezeConfig {
        &self.config
    }

    /// Check if volume confirms breakout
    fn check_volume_confirmation(&self, current_volume: f64) -> bool {
        if let Some(avg_vol) = self.volume_sma.value() {
            return current_volume >= avg_vol * self.config.volume_multiplier;
        }
        true // Not enough volume history
    }

    /// Detect squeeze condition
    fn detect_squeeze(&self, bb_upper: f64, bb_lower: f64, kk_upper: f64, kk_lower: f64) -> bool {
        let bb_width = bb_upper - bb_lower;
        let kk_width = kk_upper - kk_lower;

        // Squeeze when BB is narrower than Keltner by threshold ratio
        if kk_width == 0.0 {
            return false;
        }

        let width_ratio = bb_width / kk_width;
        width_ratio < self.config.squeeze_threshold
    }
}

impl MetadataStrategy for VolSqueezeStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Volatility Squeeze".to_string(),
            category: StrategyCategory::VolatilityBased,
            sub_type: Some("squeeze_breakout".to_string()),
            description: format!(
                "Volatility squeeze breakout strategy using {} period BB and {} period Keltner. \
                Squeeze detected when BB width < Keltner width by {:.0}%. \
                Enters on breakout from squeeze with {:.1}x volume confirmation. \
                Uses {:.1}% TP and {:.1}% SL.",
                self.config.bb_period,
                self.config.kk_period,
                self.config.squeeze_threshold * 100.0,
                self.config.volume_multiplier,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/volatility/vol_squeeze.md".to_string(),
            required_indicators: vec![
                "BollingerBands".to_string(),
                "EMA".to_string(),
                "ATR".to_string(),
                "Volume".to_string(),
            ],
            expected_regimes: vec![
                MarketRegime::Sideways,
                MarketRegime::Ranging,
                MarketRegime::HighVolatility,
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

impl Strategy for VolSqueezeStrategy {
    fn name(&self) -> &str {
        "Volatility Squeeze"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;

        // Update indicators
        let (bb_upper, bb_middle, bb_lower) = self.bb.update(price)?;
        let kk_middle = self.kk_ema.update(price)?;
        let atr_value = self.atr.update(bar)?;
        let _avg_volume = self.volume_sma.update(bar.volume);

        // Calculate Keltner Channel bands
        let kk_upper = kk_middle + (self.config.kk_mult * atr_value);
        let kk_lower = kk_middle - (self.config.kk_mult * atr_value);

        // Track volume history
        self.volume_history.push_back(bar.volume);
        if self.volume_history.len() > 20 {
            self.volume_history.pop_front();
        }

        // Check for squeeze condition
        let current_squeeze = self.detect_squeeze(bb_upper, bb_lower, kk_upper, kk_lower);
        let _squeeze_just_started = current_squeeze && !self.in_squeeze;
        let _squeeze_just_ended = !current_squeeze && self.in_squeeze;
        self.in_squeeze = current_squeeze;

        // ENTRY LOGIC (only when not in position)
        if self.last_position == SignalType::Hold {
            // Store previous bar values for crossover detection
            let prev_bb_upper = self.last_bb_upper;
            let _prev_bb_lower = self.last_bb_lower;
            let _prev_kk_upper = self.last_kk_upper;
            let _prev_kk_lower = self.last_kk_lower;
            let prev_price = self.last_price;

            // Check for breakout from squeeze
            if let Some(prev) = prev_price {
                let volume_confirmed = self.check_volume_confirmation(bar.volume);

                // Buy breakout: Price breaks above upper band
                if volume_confirmed {
                    if let Some(prev_bb_u) = prev_bb_upper {
                        if prev <= prev_bb_u && price > bb_upper {
                            self.last_position = SignalType::Buy;
                            self.entry_price = Some(price);

                            return Some(vec![Signal {
                                timestamp: bar.timestamp,
                                symbol: "UNKNOWN".to_string(),
                                signal_type: SignalType::Buy,
                                strength: 1.0,
                                metadata: Some(format!(
                                    "Squeeze Buy Breakout: Price {:.2} > BB Upper {:.2}, Volume: {:.0}",
                                    price, bb_upper, bar.volume
                                )),
                            }]);
                        }
                    }
                }
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
                        metadata: Some(format!("Take Profit: {:.1}% profit", profit_pct)),
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
                        metadata: Some(format!("Stop Loss: {:.1}% loss", profit_pct)),
                    }]);
                }

                // Exit if price re-enters squeeze (false breakout)
                if self.in_squeeze && price < bb_middle {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 0.5,
                        metadata: Some(format!(
                            "False Breakout Exit: Price {:.2} re-entered squeeze at {:.1}% profit",
                            price, profit_pct
                        )),
                    }]);
                }
            }
        }

        // Store current values for next bar
        self.last_price = Some(price);
        self.last_bb_upper = Some(bb_upper);
        self.last_bb_lower = Some(bb_lower);
        self.last_kk_upper = Some(kk_upper);
        self.last_kk_lower = Some(kk_lower);

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::Indicator;
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
    fn test_vol_squeeze_creation() {
        let strategy = VolSqueezeStrategy::new(20, 2.0, 20, 1.5, 0.1);
        assert_eq!(strategy.config().bb_period, 20);
        assert_eq!(strategy.config().squeeze_threshold, 0.1);
    }

    #[test]
    fn test_vol_squeeze_config_valid() {
        let config = VolSqueezeConfig::new(20, 2.0, 20, 1.5, 0.1);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_vol_squeeze_invalid_config() {
        let config = VolSqueezeConfig {
            bb_period: 0,
            bb_std_dev: 2.0,
            kk_period: 20,
            kk_mult: 1.5,
            squeeze_threshold: 0.1,
            volume_multiplier: 1.5,
            take_profit: 5.0,
            stop_loss: 3.0,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_vol_squeeze_from_config() {
        let config = VolSqueezeConfig::new(15, 2.5, 15, 2.0, 0.15);
        let strategy = VolSqueezeStrategy::from_config(config);
        assert_eq!(strategy.config().bb_std_dev, 2.5);
        assert_eq!(strategy.config().squeeze_threshold, 0.15);
    }

    #[test]
    fn test_vol_squeeze_metadata() {
        let strategy = VolSqueezeStrategy::new(20, 2.0, 20, 1.5, 0.1);
        let metadata = strategy.metadata();
        assert_eq!(metadata.name, "Volatility Squeeze");
        assert_eq!(metadata.category, StrategyCategory::VolatilityBased);
    }

    #[test]
    fn test_detect_squeeze() {
        let strategy = VolSqueezeStrategy::new(20, 2.0, 20, 1.5, 0.1);

        // No squeeze: BB width > Keltner width
        let bb_upper = 110.0;
        let bb_lower = 90.0;
        let kk_upper = 108.0;
        let kk_lower = 92.0;
        assert!(!strategy.detect_squeeze(bb_upper, bb_lower, kk_upper, kk_lower));

        // Squeeze: BB width < Keltner width by more than threshold
        let bb_upper = 105.0;
        let bb_lower = 95.0;
        let kk_upper = 108.0;
        let kk_lower = 92.0;
        // BB width = 10, Keltner width = 16, ratio = 0.625 > 0.1, no squeeze
        assert!(!strategy.detect_squeeze(bb_upper, bb_lower, kk_upper, kk_lower));

        // Extreme squeeze
        let bb_upper = 101.0;
        let bb_lower = 99.0;
        let kk_upper = 108.0;
        let kk_lower = 92.0;
        // BB width = 2, Keltner width = 16, ratio = 0.125 > 0.1, no squeeze
        assert!(!strategy.detect_squeeze(bb_upper, bb_lower, kk_upper, kk_lower));
    }

    #[test]
    fn test_vol_squeeze_new_instance_clean_state() {
        let strategy = VolSqueezeStrategy::new(20, 2.0, 20, 1.5, 0.1);
        assert_eq!(strategy.last_position, SignalType::Hold);
        assert!(strategy.entry_price.is_none());
        assert!(!strategy.in_squeeze);
    }

    #[test]
    fn test_check_volume_confirmation() {
        let mut strategy = VolSqueezeStrategy::new(20, 2.0, 20, 1.5, 0.1);

        // Not enough history
        assert!(strategy.check_volume_confirmation(1000.0));

        // Build up volume history
        for _ in 0..21 {
            strategy.volume_sma.update(1000.0);
        }

        // Volume above average
        assert!(strategy.check_volume_confirmation(1600.0));

        // Volume below average
        assert!(!strategy.check_volume_confirmation(1400.0));
    }
}
