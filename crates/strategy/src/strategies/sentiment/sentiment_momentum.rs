#![allow(clippy::cast_possible_truncation)]
//! Sentiment Momentum Strategy
//!
//! This strategy follows sentiment trends, based on the hypothesis that sentiment
//! momentum predicts price momentum. When sentiment is improving and becoming more
//! bullish, the strategy enters long positions. When sentiment is deteriorating
//! and becoming more bearish, it exits positions.
//!
//! The strategy uses technical sentiment indicators derived from price action
//! (RSI-like, momentum, and volume).

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for Sentiment Momentum strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentMomentumConfig {
    /// Lookback period for sentiment trend calculation
    pub lookback_period: usize,
    /// Minimum sentiment score for bullish entry (0-100)
    pub bullish_threshold: f64,
    /// Maximum sentiment score for bearish exit (0-100)
    pub bearish_threshold: f64,
    /// Minimum sentiment momentum change to trigger signal
    pub momentum_threshold: f64,
    /// Volume multiplier for confirmation (1.0 = no confirmation)
    pub volume_confirmation: f64,
}

impl SentimentMomentumConfig {
    pub fn new(
        lookback_period: usize,
        bullish_threshold: f64,
        bearish_threshold: f64,
        momentum_threshold: f64,
    ) -> Self {
        Self {
            lookback_period,
            bullish_threshold,
            bearish_threshold,
            momentum_threshold,
            volume_confirmation: 1.2,
        }
    }

    pub fn default_config() -> Self {
        Self {
            lookback_period: 14,
            bullish_threshold: 30.0,
            bearish_threshold: 70.0,
            momentum_threshold: 5.0,
            volume_confirmation: 1.2,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.lookback_period < 3 {
            return Err("lookback_period must be at least 3".to_string());
        }
        if self.lookback_period > 100 {
            return Err("lookback_period must not exceed 100".to_string());
        }
        if !(0.0..=100.0).contains(&self.bullish_threshold) {
            return Err("bullish_threshold must be between 0 and 100".to_string());
        }
        if !(0.0..=100.0).contains(&self.bearish_threshold) {
            return Err("bearish_threshold must be between 0 and 100".to_string());
        }
        if self.bullish_threshold >= self.bearish_threshold {
            return Err("bullish_threshold must be less than bearish_threshold".to_string());
        }
        if self.momentum_threshold < 0.0 || self.momentum_threshold > 50.0 {
            return Err("momentum_threshold must be between 0 and 50".to_string());
        }
        if self.volume_confirmation < 0.5 || self.volume_confirmation > 3.0 {
            return Err("volume_confirmation must be between 0.5 and 3.0".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for SentimentMomentumConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SentimentMomentumConfig(lookback={}, bullish_threshold={}, bearish_threshold={}, momentum_threshold={}, volume_conf={})",
            self.lookback_period,
            self.bullish_threshold,
            self.bearish_threshold,
            self.momentum_threshold,
            self.volume_confirmation
        )
    }
}

/// Sentiment Momentum Strategy
///
/// Tracks sentiment trends over time and generates signals when sentiment
/// momentum crosses thresholds. The strategy buys when sentiment is improving
/// and becoming bullish, and exits when sentiment is deteriorating.
pub struct SentimentMomentumStrategy {
    config: SentimentMomentumConfig,
    price_history: Vec<Bar>,
    sentiment_history: Vec<f64>,
    last_position: Option<bool>, // true = long, false = flat/none
}

impl SentimentMomentumStrategy {
    /// Create a new Sentiment Momentum strategy with default configuration
    pub fn new() -> Self {
        Self::from_config(SentimentMomentumConfig::default_config())
    }

    /// Create a new Sentiment Momentum strategy with custom configuration
    pub fn from_config(config: SentimentMomentumConfig) -> Self {
        Self {
            config,
            price_history: Vec::new(),
            sentiment_history: Vec::new(),
            last_position: None,
        }
    }

    /// Get the strategy configuration
    pub fn config(&self) -> &SentimentMomentumConfig {
        &self.config
    }

    /// Calculate normalized sentiment score (0-100) from bar data
    /// Combines an RSI-like component, short/long momentum, and a volume factor
    fn calculate_sentiment(&self, bars: &[Bar]) -> f64 {
        if bars.is_empty() {
            return 50.0; // Neutral sentiment if no data
        }

        let period = std::cmp::min(self.config.lookback_period, bars.len());
        if period < 2 {
            return 50.0;
        }

        let recent_bars = &bars[bars.len() - period..];

        // Gains / losses (sums instead of allocations)
        let mut gains: f64 = 0.0;
        let mut losses: f64 = 0.0;
        for window in recent_bars.windows(2) {
            let change = window[1].close - window[0].close;
            if change > 0.0 {
                gains += change;
            } else {
                losses += -change;
            }
        }

        let denom = (period - 1) as f64;
        let avg_gain = if denom > 0.0 { gains / denom } else { 0.0 };
        let avg_loss = if denom > 0.0 { losses / denom } else { 0.0 };

        let rsi_component = if avg_loss == 0.0 {
            100.0
        } else {
            let rs = avg_gain / avg_loss;
            100.0 - (100.0 / (1.0 + rs))
        };

        // Momentum component (percent change over lookback)
        let price_change = if bars.len() >= 2 {
            let start_idx = bars.len() - period;
            let start_price = bars[start_idx].close;
            if start_price == 0.0 {
                0.0
            } else {
                (bars[bars.len() - 1].close - start_price) / start_price * 100.0
            }
        } else {
            0.0
        };
        let momentum_component = (price_change + 20.0).clamp(0.0, 100.0);

        // Volume component (normalized to 0-100)
        let avg_volume = if period > 0 {
            recent_bars.iter().map(|b| b.volume).sum::<f64>() / period as f64
        } else {
            recent_bars.last().map(|b| b.volume).unwrap_or(1.0)
        };
        let current_volume = recent_bars.last().map(|b| b.volume).unwrap_or(1.0);
        let volume_ratio = if avg_volume > 0.0 {
            (current_volume / avg_volume).min(3.0)
        } else {
            1.0
        };
        let volume_component = ((volume_ratio - 0.5) / 2.5 * 100.0).clamp(0.0, 100.0);

        // Combine components with weights: RSI 40%, Momentum 40%, Volume 20%
        rsi_component * 0.4 + momentum_component * 0.4 + volume_component * 0.2
    }

    /// Calculate average volume over the lookback period
    fn calculate_avg_volume(&self, bars: &[Bar]) -> f64 {
        let period = std::cmp::min(self.config.lookback_period, bars.len());
        if period == 0 {
            return 1.0;
        }
        let recent_bars = &bars[bars.len() - period..];
        recent_bars.iter().map(|b| b.volume).sum::<f64>() / period as f64
    }

    /// Check if volume confirms the signal
    fn check_volume_confirmation(&self, current_volume: f64, avg_volume: f64) -> bool {
        if self.config.volume_confirmation <= 1.0 {
            return true; // No volume confirmation required
        }
        if avg_volume == 0.0 {
            return true;
        }
        current_volume >= avg_volume * self.config.volume_confirmation
    }
}

impl Default for SentimentMomentumStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataStrategy for SentimentMomentumStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Sentiment Momentum".to_string(),
            description:
                "Follows sentiment trends by tracking technical sentiment indicators (RSI, momentum, volume) over time. Generates buy signals when sentiment is improving and becoming bullish, and exit signals when sentiment is deteriorating and becoming bearish.".to_string(),
            category: StrategyCategory::SentimentBased,
            sub_type: Some("sentiment_momentum".to_string()),
            hypothesis_path: "hypotheses/sentiment/sentiment_momentum.md".to_string(),
            required_indicators: vec!["RSI".to_string(), "Momentum".to_string(), "Volume".to_string()],
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Sideways, MarketRegime::Trending],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.25,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Medium,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::SentimentBased
    }
}

impl Strategy for SentimentMomentumStrategy {
    fn name(&self) -> &str {
        "Sentiment Momentum"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        // Update histories (Bar is Copy, so dereference)
        self.price_history.push(*bar);
        // Keep a moderate rolling history to bound memory and allow momentum calculations
        let max_hist = self.config.lookback_period * 2;
        if self.price_history.len() > max_hist {
            self.price_history.remove(0);
        }

        // Calculate sentiment using the recent bar history
        let current_sentiment = self.calculate_sentiment(&self.price_history);
        self.sentiment_history.push(current_sentiment);
        if self.sentiment_history.len() > max_hist {
            self.sentiment_history.remove(0);
        }

        // Need sufficient data
        if self.price_history.len() < self.config.lookback_period {
            return None;
        }

        let avg_volume = self.calculate_avg_volume(&self.price_history);
        let current_volume = bar.volume;

        // Sentiment momentum (difference from previous)
        let sentiment_momentum = if self.sentiment_history.len() >= 2 {
            current_sentiment - self.sentiment_history[self.sentiment_history.len() - 2]
        } else {
            0.0
        };

        match self.last_position {
            Some(true) => {
                // Currently long - check for exit
                if current_sentiment < self.config.bearish_threshold
                    || sentiment_momentum < -self.config.momentum_threshold
                    || !self.check_volume_confirmation(current_volume, avg_volume)
                {
                    self.last_position = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some("Exit on sentiment momentum reversal".to_string()),
                    }]);
                }
                None
            }
            None | Some(false) => {
                // Entry signal
                if current_sentiment > self.config.bullish_threshold
                    && sentiment_momentum > self.config.momentum_threshold
                    && self.check_volume_confirmation(current_volume, avg_volume)
                {
                    self.last_position = Some(true);
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: 1.0,
                        metadata: Some("Buy on improving sentiment momentum".to_string()),
                    }]);
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono::Utc;

    fn create_test_bar(timestamp: i64, close: f64, volume: f64) -> Bar {
        Bar {
            timestamp: Utc.timestamp_opt(timestamp, 0).unwrap(),
            open: close * 0.99,
            high: close * 1.01,
            low: close * 0.98,
            close,
            volume,
        }
    }

    #[test]
    fn test_config_creation() {
        let config = SentimentMomentumConfig::new(14, 30.0, 70.0, 5.0);
        assert_eq!(config.lookback_period, 14);
        assert_eq!(config.bullish_threshold, 30.0);
        assert_eq!(config.bearish_threshold, 70.0);
    }

    #[test]
    fn test_default_config() {
        let config = SentimentMomentumConfig::default_config();
        assert_eq!(config.lookback_period, 14);
        assert_eq!(config.bullish_threshold, 30.0);
        assert_eq!(config.volume_confirmation, 1.2);
    }

    #[test]
    fn test_config_valid() {
        let config = SentimentMomentumConfig::new(14, 30.0, 70.0, 5.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_calculate_sentiment_uptrend() {
        let strategy = SentimentMomentumStrategy::new();
        let bars: Vec<Bar> = (0..20)
            .map(|i| create_test_bar(i, 100.0 + i as f64 * 1.0, 1000.0))
            .collect();

        let sentiment = strategy.calculate_sentiment(&bars);
        assert!(sentiment > 50.0, "Sentiment should be bullish in uptrend");
    }

    #[test]
    fn test_calculate_sentiment_downtrend() {
        let strategy = SentimentMomentumStrategy::new();
        let bars: Vec<Bar> = (0..20)
            .map(|i| create_test_bar(i, 100.0 - i as f64 * 1.0, 1000.0))
            .collect();

        let sentiment = strategy.calculate_sentiment(&bars);
        assert!(sentiment < 50.0, "Sentiment should be bearish in downtrend");
    }

    #[test]
    fn test_on_bar_no_signal_insufficient_data() {
        let mut strategy = SentimentMomentumStrategy::new();
        let bar = create_test_bar(0, 100.0, 1000.0);
        let signal = strategy.on_bar(&bar);
        assert!(signal.is_none());
    }

    #[test]
    fn test_on_bar_buy_and_exit() {
        let mut strategy = SentimentMomentumStrategy::new();

        // Build strong improving sentiment scenario with high volume
        let mut bought = false;
        for i in 0..30 {
            let close = 85.0 + i as f64 * 1.5;
            let bar = create_test_bar(i, close, 2000.0); // high volume for confirmation
            if let Some(sigs) = strategy.on_bar(&bar) {
                if sigs
                    .iter()
                    .any(|s| matches!(s.signal_type, SignalType::Buy))
                {
                    bought = true;
                    break;
                }
            }
        }

        assert!(
            bought,
            "Expected buy signal in improving high-volume scenario"
        );

        // Simulate a deterioration to test exit
        strategy.last_position = Some(true);
        let exit_bar = create_test_bar(100, 100.0, 1000.0);
        if let Some(sigs) = strategy.on_bar(&exit_bar) {
            assert!(sigs
                .iter()
                .any(|s| matches!(s.signal_type, SignalType::Sell)));
        }
    }

    #[test]
    fn test_calculate_avg_volume() {
        let bars: Vec<Bar> = (0..20)
            .map(|i| create_test_bar(i, 100.0 + i as f64 * 0.1, 1000.0 + i as f64 * 50.0))
            .collect();

        let strategy = SentimentMomentumStrategy::new();
        let avg_vol = strategy.calculate_avg_volume(&bars);
        assert!(avg_vol > 0.0);
    }

    #[test]
    fn test_new_instance_clean_state() {
        let strategy = SentimentMomentumStrategy::new();
        assert!(strategy.price_history.is_empty());
        assert!(strategy.sentiment_history.is_empty());
        assert!(strategy.last_position.is_none());
    }

    #[test]
    fn test_config_display() {
        let config = SentimentMomentumConfig::new(14, 30.0, 70.0, 5.0);
        let display = format!("{}", config);
        assert!(display.contains("lookback=14"));
        assert!(display.contains("bullish_threshold=30.0"));
    }
}
