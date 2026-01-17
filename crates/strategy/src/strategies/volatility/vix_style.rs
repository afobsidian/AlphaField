//! VIX-Style Strategy
//!
//! This strategy creates a crypto volatility index similar to the VIX
//! (CBOE Volatility Index) used in traditional markets. The VIX measures
//! market expectations of near-term volatility and is often used as a
//! contrarian indicator: buy when fear (volatility) is high, sell when
//! greed (low volatility) prevails.
//!
//! This implementation uses ATR percentile ranking to create a 0-100
//! volatility index, similar to the Fear & Greed index methodology.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::Atr;
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Configuration for VIX-Style strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VIXStyleConfig {
    /// Period for ATR calculation
    pub atr_period: usize,
    /// Lookback period for percentile calculation (volatility index window)
    /// Typically 100 days for meaningful historical comparison
    pub lookback_period: usize,
    /// Extreme fear threshold (volatility percentile above this is buy signal)
    /// 0.8 = 80th percentile, 0.9 = 90th percentile (more extreme)
    pub extreme_fear_threshold: f64,
    /// Extreme greed threshold (volatility percentile below this is caution)
    /// 0.2 = 20th percentile, 0.1 = 10th percentile (more extreme)
    pub extreme_greed_threshold: f64,
    /// Volume multiplier for entry confirmation
    pub volume_multiplier: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl VIXStyleConfig {
    pub fn new(atr_period: usize, lookback_period: usize) -> Self {
        Self {
            atr_period,
            lookback_period,
            extreme_fear_threshold: 0.85,  // Top 15% = extreme fear
            extreme_greed_threshold: 0.15, // Bottom 15% = extreme greed
            volume_multiplier: 1.2,
            take_profit: 8.0,
            stop_loss: 4.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            atr_period: 14,
            lookback_period: 100,
            extreme_fear_threshold: 0.85,
            extreme_greed_threshold: 0.15,
            volume_multiplier: 1.2,
            take_profit: 8.0,
            stop_loss: 4.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.atr_period == 0 {
            return Err("ATR period must be greater than 0".to_string());
        }
        if self.lookback_period < 50 {
            return Err("Lookback period must be at least 50".to_string());
        }
        if self.extreme_fear_threshold <= 0.0 || self.extreme_fear_threshold >= 1.0 {
            return Err("Extreme fear threshold must be between 0 and 1".to_string());
        }
        if self.extreme_greed_threshold <= 0.0 || self.extreme_greed_threshold >= 1.0 {
            return Err("Extreme greed threshold must be between 0 and 1".to_string());
        }
        if self.extreme_greed_threshold >= self.extreme_fear_threshold {
            return Err(
                "Extreme greed threshold must be less than extreme fear threshold".to_string(),
            );
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

impl fmt::Display for VIXStyleConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VIXStyle(atr_period={}, lookback={}, fear_thresh={:.2}, greed_thresh={:.2}, tp={:.1}%, sl={:.1}%)",
            self.atr_period,
            self.lookback_period,
            self.extreme_fear_threshold,
            self.extreme_greed_threshold,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// VIX-Style Strategy
///
/// # Strategy Logic
/// - **VIX Index Calculation**: ATR percentile rank converted to 0-100 scale
///   - 0-25: Extreme Greed (very low volatility)
///   - 25-45: Greed
///   - 45-55: Neutral
///   - 55-75: Fear
///   - 75-100: Extreme Fear (very high volatility)
/// - **Buy Signal**: VIX > extreme fear threshold (e.g., >85) with volume confirmation
///   - Contrarian: buy when market is most fearful
/// - **Sell Signal**: VIX < extreme greed threshold (e.g., <15) or TP/SL
///   - Take profits when greed is extreme or risk management
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::volatility::VIXStyleStrategy;
///
/// let strategy = VIXStyleStrategy::new(14, 100);
/// ```
pub struct VIXStyleStrategy {
    config: VIXStyleConfig,
    atr: Atr,
    atr_history: VecDeque<f64>,
    volume_history: VecDeque<f64>,
    last_position: SignalType,
    entry_price: Option<f64>,
    entry_vix: Option<f64>, // Track VIX level at entry
}

impl Default for VIXStyleStrategy {
    fn default() -> Self {
        Self::from_config(VIXStyleConfig::default_config())
    }
}

impl VIXStyleStrategy {
    /// Creates a new VIX-Style strategy
    ///
    /// # Arguments
    /// * `atr_period` - ATR calculation period
    /// * `lookback_period` - Historical ATR window for percentile calculation
    pub fn new(atr_period: usize, lookback_period: usize) -> Self {
        let config = VIXStyleConfig::new(atr_period, lookback_period);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: VIXStyleConfig) -> Self {
        config.validate().expect("Invalid VIXStyleConfig");

        Self {
            atr: Atr::new(config.atr_period),
            atr_history: VecDeque::with_capacity(config.lookback_period),
            volume_history: VecDeque::with_capacity(20),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            entry_vix: None,
        }
    }

    pub fn config(&self) -> &VIXStyleConfig {
        &self.config
    }

    /// Calculate VIX index (0-100 scale) based on ATR percentile
    fn calculate_vix_index(&self) -> Option<f64> {
        if self.atr_history.is_empty() {
            return None;
        }

        if let Some(current_atr) = self.atr.value() {
            // Calculate percentile rank: how many historical ATRs are below current
            let count_below = self
                .atr_history
                .iter()
                .filter(|&&v| v < current_atr)
                .count();

            let percentile = count_below as f64 / self.atr_history.len() as f64;

            // Convert to 0-100 VIX scale
            Some(percentile * 100.0)
        } else {
            None
        }
    }

    /// Get average volume
    fn average_volume(&self) -> Option<f64> {
        if self.volume_history.is_empty() {
            return None;
        }
        let sum: f64 = self.volume_history.iter().sum();
        Some(sum / self.volume_history.len() as f64)
    }

    /// Check if volume confirms signal
    fn check_volume_confirmation(&self, current_volume: f64) -> bool {
        if let Some(avg_vol) = self.average_volume() {
            return current_volume >= avg_vol * self.config.volume_multiplier;
        }
        true // Not enough volume history
    }

    /// Get VIX sentiment description
    fn get_vix_sentiment(&self, vix: f64) -> &'static str {
        if vix < 25.0 {
            "Extreme Greed"
        } else if vix < 45.0 {
            "Greed"
        } else if vix < 55.0 {
            "Neutral"
        } else if vix < 75.0 {
            "Fear"
        } else {
            "Extreme Fear"
        }
    }
}

impl MetadataStrategy for VIXStyleStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "VIX-Style".to_string(),
            category: StrategyCategory::VolatilityBased,
            sub_type: Some("vix_contrarian".to_string()),
            description: format!(
                "VIX-style contrarian strategy using {} period ATR with {} period lookback. \
                Calculates volatility index (0-100) from ATR percentile. \
                Buys on extreme fear (VIX > {:.0}), sells on extreme greed (VIX < {:.0}) or TP/SL. \
                Contrarian: buy when market is most fearful. Requires {:.1}x volume confirmation. \
                Uses {:.1}% TP and {:.1}% SL.",
                self.config.atr_period,
                self.config.lookback_period,
                self.config.extreme_fear_threshold * 100.0,
                self.config.extreme_greed_threshold * 100.0,
                self.config.volume_multiplier,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/volatility/vix_style.md".to_string(),
            required_indicators: vec!["ATR".to_string(), "Volume".to_string()],
            expected_regimes: vec![
                MarketRegime::Bear,
                MarketRegime::HighVolatility,
                MarketRegime::Ranging,
            ],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.20,
                volatility_level: VolatilityLevel::High,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::VolatilityBased
    }
}

impl Strategy for VIXStyleStrategy {
    fn name(&self) -> &str {
        "VIX-Style"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;

        // Update ATR
        let _atr_value = self.atr.update(bar)?;

        // Track ATR history for percentile calculation
        if let Some(atr_val) = self.atr.value() {
            self.atr_history.push_back(atr_val);
            if self.atr_history.len() > self.config.lookback_period {
                self.atr_history.pop_front();
            }
        }

        // Track volume history
        self.volume_history.push_back(bar.volume);
        if self.volume_history.len() > 20 {
            self.volume_history.pop_front();
        }

        // Calculate VIX index
        let vix = self.calculate_vix_index()?;

        // ENTRY LOGIC (only when not in position)
        if self.last_position == SignalType::Hold {
            // Check for extreme fear (contrarian buy signal)
            let fear_percentile = self.config.extreme_fear_threshold * 100.0;

            if vix > fear_percentile {
                // Volume confirmation
                if self.check_volume_confirmation(bar.volume) {
                    self.last_position = SignalType::Buy;
                    self.entry_price = Some(price);
                    self.entry_vix = Some(vix);

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Contrarian Buy Entry: VIX {:.1} ({}), Price {:.2}, Volume: {:.0}",
                            vix,
                            self.get_vix_sentiment(vix),
                            price,
                            bar.volume
                        )),
                    }]);
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
                    let exit_vix = self.entry_vix.take().unwrap_or(vix);

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Take Profit: {:.1}% profit (Entry VIX: {:.1}, Current VIX: {:.1})",
                            profit_pct, exit_vix, vix
                        )),
                    }]);
                }

                // Stop Loss
                if profit_pct <= -self.config.stop_loss {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    let exit_vix = self.entry_vix.take().unwrap_or(vix);

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Stop Loss: {:.1}% loss (Entry VIX: {:.1}, Current VIX: {:.1})",
                            profit_pct, exit_vix, vix
                        )),
                    }]);
                }

                // Exit on extreme greed (take profits when market is too greedy)
                let greed_percentile = self.config.extreme_greed_threshold * 100.0;

                if vix < greed_percentile {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    let exit_vix = self.entry_vix.take().unwrap_or(vix);

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Extreme Greed Exit: VIX {:.1} ({}), Profit: {:.1}% (Entry VIX: {:.1})",
                            vix,
                            self.get_vix_sentiment(vix),
                            profit_pct,
                            exit_vix
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
    fn test_vix_style_creation() {
        let strategy = VIXStyleStrategy::new(14, 100);
        assert_eq!(strategy.config().atr_period, 14);
        assert_eq!(strategy.config().lookback_period, 100);
    }

    #[test]
    fn test_vix_style_config_valid() {
        let config = VIXStyleConfig::new(14, 100);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_vix_style_invalid_config() {
        let config = VIXStyleConfig {
            atr_period: 0,
            lookback_period: 100,
            extreme_fear_threshold: 0.85,
            extreme_greed_threshold: 0.15,
            volume_multiplier: 1.2,
            take_profit: 8.0,
            stop_loss: 4.0,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_vix_style_invalid_thresholds() {
        let config = VIXStyleConfig {
            atr_period: 14,
            lookback_period: 100,
            extreme_fear_threshold: 0.15, // fear < greed
            extreme_greed_threshold: 0.85,
            volume_multiplier: 1.2,
            take_profit: 8.0,
            stop_loss: 4.0,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_vix_style_from_config() {
        let config = VIXStyleConfig::new(21, 150);
        let strategy = VIXStyleStrategy::from_config(config);
        assert_eq!(strategy.config().atr_period, 21);
        assert_eq!(strategy.config().lookback_period, 150);
    }

    #[test]
    fn test_vix_style_metadata() {
        let strategy = VIXStyleStrategy::new(14, 100);
        let metadata = strategy.metadata();
        assert_eq!(metadata.name, "VIX-Style");
        assert_eq!(metadata.category, StrategyCategory::VolatilityBased);
    }

    #[test]
    fn test_calculate_vix_index() {
        let mut strategy = VIXStyleStrategy::new(14, 100);

        // Build ATR history by processing bars with known volatility
        // Simulate consistent volatility so we can predict ATR values
        for i in 0..150 {
            let bar = create_test_bar(
                i,
                100.0 + i as f64,
                101.0 + i as f64,
                99.0 + i as f64,
                100.0 + i as f64,
                1000.0,
            );
            strategy.on_bar(&bar);
        }

        // Get current ATR value from the indicator
        let current_atr = strategy.atr.value();
        assert!(current_atr.is_some());

        // The VIX index should be calculated based on the ATR percentile
        let vix = strategy.calculate_vix_index();
        assert!(vix.is_some());

        // VIX should be in valid range (0-100)
        assert!(vix.unwrap() >= 0.0 && vix.unwrap() <= 100.0);
    }

    #[test]
    fn test_get_vix_sentiment() {
        let strategy = VIXStyleStrategy::new(14, 100);

        assert_eq!(strategy.get_vix_sentiment(10.0), "Extreme Greed");
        assert_eq!(strategy.get_vix_sentiment(30.0), "Greed");
        assert_eq!(strategy.get_vix_sentiment(50.0), "Neutral");
        assert_eq!(strategy.get_vix_sentiment(65.0), "Fear");
        assert_eq!(strategy.get_vix_sentiment(85.0), "Extreme Fear");
    }

    #[test]
    fn test_check_volume_confirmation() {
        let mut strategy = VIXStyleStrategy::new(14, 100);

        // Not enough history
        assert!(strategy.check_volume_confirmation(1000.0));

        // Build up volume history
        for _ in 0..21 {
            strategy.volume_history.push_back(1000.0);
        }

        // Volume above average (1.2x multiplier)
        assert!(strategy.check_volume_confirmation(1300.0));

        // Volume below average
        assert!(!strategy.check_volume_confirmation(1100.0));
    }

    #[test]
    fn test_vix_style_new_instance_clean_state() {
        let strategy = VIXStyleStrategy::new(14, 100);
        assert_eq!(strategy.last_position, SignalType::Hold);
        assert!(strategy.entry_price.is_none());
        assert!(strategy.entry_vix.is_none());
    }
}
