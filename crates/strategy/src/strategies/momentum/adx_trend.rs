//! ADX Trend Strength Strategy
//!
//! Uses the Average Directional Index (ADX) to measure trend strength.
//! Only enters trades when ADX indicates a strong trend.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::Adx;
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for ADX Trend strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdxTrendConfig {
    /// ADX period
    pub adx_period: usize,
    /// Strong trend threshold (ADX above this = strong trend)
    pub strong_trend_threshold: f64,
    /// Weak trend threshold (ADX below this = weak trend, exit)
    pub weak_trend_threshold: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl AdxTrendConfig {
    pub fn new(
        adx_period: usize,
        strong_trend_threshold: f64,
        weak_trend_threshold: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            adx_period,
            strong_trend_threshold,
            weak_trend_threshold,
            take_profit,
            stop_loss,
        }
    }

    pub fn default_config() -> Self {
        Self {
            adx_period: 14,
            strong_trend_threshold: 25.0,
            weak_trend_threshold: 20.0,
            take_profit: 5.0,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.adx_period == 0 {
            return Err("ADX period must be greater than 0".to_string());
        }
        if self.strong_trend_threshold <= 0.0 || self.strong_trend_threshold > 100.0 {
            return Err("Strong trend threshold must be between 0 and 100".to_string());
        }
        if self.weak_trend_threshold >= self.strong_trend_threshold {
            return Err(
                "Weak trend threshold must be less than strong trend threshold".to_string(),
            );
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

impl fmt::Display for AdxTrendConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ADX_Trend(period={}, strong={:.0}, weak={:.0}, tp={:.1}%, sl={:.1}%)",
            self.adx_period,
            self.strong_trend_threshold,
            self.weak_trend_threshold,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// ADX Trend Strength Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: ADX crosses above strong_trend_threshold (strong uptrend)
/// - **Sell Signal**: ADX falls below weak_trend_threshold (trend weakening) OR TP/SL
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::momentum::AdxTrendStrategy;
///
/// let strategy = AdxTrendStrategy::new(14, 25.0, 20.0, 5.0, 3.0);
/// ```
pub struct AdxTrendStrategy {
    config: AdxTrendConfig,
    adx: Adx,
    last_position: SignalType,
    entry_price: Option<f64>,
    last_adx: Option<f64>,
    last_close: Option<f64>,
    trend_direction: Option<i8>, // +1 for up, -1 for down, 0 for neutral
}

impl AdxTrendStrategy {
    /// Creates a new ADX Trend strategy
    pub fn new(
        adx_period: usize,
        strong_trend_threshold: f64,
        weak_trend_threshold: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        let config = AdxTrendConfig::new(
            adx_period,
            strong_trend_threshold,
            weak_trend_threshold,
            take_profit,
            stop_loss,
        );
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: AdxTrendConfig) -> Self {
        config.validate().expect("Invalid AdxTrendConfig");

        Self {
            adx: Adx::new(config.adx_period),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            last_adx: None,
            last_close: None,
            trend_direction: None,
        }
    }

    pub fn config(&self) -> &AdxTrendConfig {
        &self.config
    }

    /// Determine trend direction from price action
    fn update_trend_direction(&mut self, close: f64) {
        if let Some(last) = self.last_close {
            if close > last {
                self.trend_direction = Some(1); // Uptrend
            } else if close < last {
                self.trend_direction = Some(-1); // Downtrend
            }
            // else keep previous direction
        }
        self.last_close = Some(close);
    }
}

impl MetadataStrategy for AdxTrendStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "ADX Trend".to_string(),
            category: StrategyCategory::Momentum,
            sub_type: Some("adx_trend".to_string()),
            description: format!(
                "ADX-based trend strength strategy using {} period ADX. Only enters when ADX > {} 
                (strong trend detected). Exits when ADX falls below {} (trend weakening) or on TP/SL. 
                Uses {:.1}% TP and {:.1}% SL.",
                self.config.adx_period,
                self.config.strong_trend_threshold,
                self.config.weak_trend_threshold,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/momentum/adx_trend.md".to_string(),
            required_indicators: vec!["ADX".to_string()],
            expected_regimes: vec![
                MarketRegime::Trending,
                MarketRegime::Bull,
            ],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.22,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Medium,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::Momentum
    }
}

impl Strategy for AdxTrendStrategy {
    fn name(&self) -> &str {
        "ADX Trend"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;

        // Update trend direction
        self.update_trend_direction(price);

        // Update ADX
        let adx_val = self.adx.update(bar)?;
        let prev_adx = self.last_adx;

        // Update state for next bar
        self.last_adx = Some(adx_val);

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

                // Exit on weak trend: ADX crosses below weak threshold
                if let Some(prev) = prev_adx {
                    if prev >= self.config.weak_trend_threshold
                        && adx_val < self.config.weak_trend_threshold
                    {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 0.8,
                            metadata: Some(format!(
                                "Weak Trend Exit: ADX {:.2} crossed below {}",
                                adx_val, self.config.weak_trend_threshold
                            )),
                        }]);
                    }
                }

                // Exit on trend reversal (price direction changes)
                if let Some(direction) = self.trend_direction {
                    if direction == -1 {
                        // Downtrend detected while in long position
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 0.9,
                            metadata: Some("Trend Reversal Exit".to_string()),
                        }]);
                    }
                }
            }
        }

        // ENTRY LOGIC - ADX crosses above strong trend threshold in uptrend
        if self.last_position != SignalType::Buy {
            if let Some(prev) = prev_adx {
                // ADX must cross above strong trend threshold
                if prev <= self.config.strong_trend_threshold
                    && adx_val > self.config.strong_trend_threshold
                {
                    // Only enter if in uptrend
                    if let Some(direction) = self.trend_direction {
                        if direction == 1 {
                            // Calculate signal strength based on ADX magnitude
                            let strength = ((adx_val - self.config.strong_trend_threshold) / 20.0)
                                .min(1.0)
                                .max(0.6);

                            self.last_position = SignalType::Buy;
                            self.entry_price = Some(price);
                            return Some(vec![Signal {
                                timestamp: bar.timestamp,
                                symbol: "UNKNOWN".to_string(),
                                signal_type: SignalType::Buy,
                                strength,
                                metadata: Some(format!(
                                    "Strong Trend Entry: ADX {:.2} crossed above {}",
                                    adx_val, self.config.strong_trend_threshold
                                )),
                            }]);
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
    use chrono::Utc;

    fn create_test_bar(high: f64, low: f64, close: f64) -> Bar {
        Bar {
            timestamp: Utc::now(),
            open: close,
            high,
            low,
            close,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_adx_trend_creation() {
        let strategy = AdxTrendStrategy::new(14, 25.0, 20.0, 5.0, 3.0);
        assert_eq!(strategy.name(), "ADX Trend");
    }

    #[test]
    fn test_config_validation() {
        let config = AdxTrendConfig::default_config();
        assert!(config.validate().is_ok());

        let invalid_config = AdxTrendConfig::new(14, 20.0, 25.0, 5.0, 3.0); // weak > strong
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_metadata() {
        let strategy = AdxTrendStrategy::new(14, 25.0, 20.0, 5.0, 3.0);
        let metadata = strategy.metadata();
        assert_eq!(metadata.category, StrategyCategory::Momentum);
        assert_eq!(metadata.name, "ADX Trend");
    }

    #[test]
    fn test_no_signal_without_warmup() {
        let mut strategy = AdxTrendStrategy::new(14, 25.0, 20.0, 5.0, 3.0);

        // ADX needs significant warmup (period * 2)
        for i in 1..20 {
            let bar = create_test_bar(100.0 + i as f64, 99.0, 100.0 + i as f64);
            let _signal = strategy.on_bar(&bar);
            // May be None until ADX warms up
        }
    }

    #[test]
    fn test_trend_direction_detection() {
        let mut strategy = AdxTrendStrategy::new(14, 25.0, 20.0, 5.0, 3.0);

        // Uptrend
        for i in 1..5 {
            let bar = create_test_bar(100.0 + i as f64, 99.0 + i as f64, 100.0 + i as f64);
            strategy.update_trend_direction(bar.close);
        }
        assert_eq!(strategy.trend_direction, Some(1));

        // Downtrend
        for i in (1..5).rev() {
            let bar = create_test_bar(100.0 + i as f64, 99.0 + i as f64, 100.0 + i as f64);
            strategy.update_trend_direction(bar.close);
        }
        assert_eq!(strategy.trend_direction, Some(-1));
    }
}
