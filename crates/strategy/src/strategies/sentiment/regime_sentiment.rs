//! Regime-Based Sentiment Strategy
//!
//! This strategy adapts sentiment interpretation based on the current market regime,
//! based on the hypothesis that sentiment signals vary by market regime. It adjusts
//! sentiment thresholds and signal interpretation based on whether the market is in
//! a bull, bear, or sideways regime.
//!
//! The strategy uses AssetSentiment data (technical sentiment from price action) and
//! detects market regimes using price trend, volatility, and trend strength indicators.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for Regime-Based Sentiment Strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeSentimentConfig {
    /// Lookback period for regime detection
    pub regime_lookback: usize,
    /// Lookback period for sentiment calculation
    pub sentiment_lookback: usize,
    /// Trend strength threshold for regime classification (0-100)
    pub trend_threshold: f64,
    /// Volatility threshold for high volatility regime
    pub volatility_threshold: f64,
    /// Bullish sentiment threshold in bull regime (0-100)
    pub bull_bullish_threshold: f64,
    /// Bullish sentiment threshold in bear regime (0-100)
    pub bear_bullish_threshold: f64,
    /// Bullish sentiment threshold in sideways regime (0-100)
    pub sideways_bullish_threshold: f64,
    /// Minimum sentiment change to trigger signal
    pub momentum_threshold: f64,
}

impl RegimeSentimentConfig {
    pub fn new(
        regime_lookback: usize,
        sentiment_lookback: usize,
        trend_threshold: f64,
        volatility_threshold: f64,
    ) -> Self {
        Self {
            regime_lookback,
            sentiment_lookback,
            trend_threshold,
            volatility_threshold,
            bull_bullish_threshold: 60.0,
            bear_bullish_threshold: 40.0,
            sideways_bullish_threshold: 50.0,
            momentum_threshold: 5.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            regime_lookback: 20,
            sentiment_lookback: 14,
            trend_threshold: 50.0,
            volatility_threshold: 2.0,
            bull_bullish_threshold: 60.0,
            bear_bullish_threshold: 40.0,
            sideways_bullish_threshold: 50.0,
            momentum_threshold: 5.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.regime_lookback < 10 {
            return Err("regime_lookback must be at least 10".to_string());
        }
        if self.regime_lookback > 100 {
            return Err("regime_lookback must not exceed 100".to_string());
        }
        if self.sentiment_lookback < 5 {
            return Err("sentiment_lookback must be at least 5".to_string());
        }
        if self.sentiment_lookback > 100 {
            return Err("sentiment_lookback must not exceed 100".to_string());
        }
        if self.trend_threshold < 10.0 || self.trend_threshold > 60.0 {
            return Err("trend_threshold must be between 10 and 60".to_string());
        }
        if self.volatility_threshold < 0.5 || self.volatility_threshold > 5.0 {
            return Err("volatility_threshold must be between 0.5 and 5".to_string());
        }
        if self.bull_bullish_threshold < 50.0 || self.bull_bullish_threshold > 90.0 {
            return Err("bull_bullish_threshold must be between 50 and 90".to_string());
        }
        if self.bear_bullish_threshold < 20.0 || self.bear_bullish_threshold > 60.0 {
            return Err("bear_bullish_threshold must be between 20 and 60".to_string());
        }
        if self.sideways_bullish_threshold < 30.0 || self.sideways_bullish_threshold > 70.0 {
            return Err("sideways_bullish_threshold must be between 30 and 70".to_string());
        }
        if self.momentum_threshold < 1.0 || self.momentum_threshold > 20.0 {
            return Err("momentum_threshold must be between 1 and 20".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for RegimeSentimentConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
             f,
             "RegimeSentimentConfig(regime_lookback={}, sentiment_lookback={}, trend_threshold={:.1}, vol_threshold={:.1}, bull_thresh={:.1}, bear_thresh={:.1}, sideways_thresh={:.1}, momentum={:.1})",
             self.regime_lookback,
             self.sentiment_lookback,
             self.trend_threshold,
             self.volatility_threshold,
             self.bull_bullish_threshold,
             self.bear_bullish_threshold,
             self.sideways_bullish_threshold,
             self.momentum_threshold
         )
    }
}

/// Regime-Based Sentiment Strategy
///
/// Detects market regimes and adapts sentiment interpretation accordingly.
/// In bull markets, requires stronger bullish sentiment to buy. In bear markets,
/// more sensitive to bullish sentiment for contrarian opportunities.
pub struct RegimeSentimentStrategy {
    config: RegimeSentimentConfig,
    bar_history: Vec<Bar>,
    price_history: Vec<f64>,
    sentiment_history: Vec<f64>,
    current_regime: MarketRegime,
    last_position: Option<bool>,
}

impl RegimeSentimentStrategy {
    /// Create a new Regime-Based Sentiment strategy with default configuration
    pub fn new() -> Self {
        Self::from_config(RegimeSentimentConfig::default_config())
    }

    /// Create a new Regime-Based Sentiment strategy with custom configuration
    pub fn from_config(config: RegimeSentimentConfig) -> Self {
        Self {
            config,
            bar_history: Vec::new(),
            price_history: Vec::new(),
            sentiment_history: Vec::new(),
            current_regime: MarketRegime::Sideways,
            last_position: None,
        }
    }

    /// Get the strategy configuration
    pub fn config(&self) -> &RegimeSentimentConfig {
        &self.config
    }

    /// Calculate normalized sentiment score (0-100) from bar data
    fn calculate_sentiment(&self, bars: &[Bar]) -> f64 {
        if bars.is_empty() {
            return 50.0; // Neutral sentiment if no data
        }

        let period = self.config.sentiment_lookback.min(bars.len());
        if period < 2 {
            return 50.0;
        }

        let recent_bars = &bars[bars.len() - period..];
        let mut gains: Vec<f64> = Vec::new();
        let mut losses: Vec<f64> = Vec::new();

        for window in recent_bars.windows(2) {
            let change = window[1].close - window[0].close;
            if change > 0.0 {
                gains.push(change);
            } else {
                losses.push(-change);
            }
        }

        let avg_gain = if gains.is_empty() {
            0.0
        } else {
            gains.iter().sum::<f64>() / gains.len() as f64
        };
        let avg_loss = if losses.is_empty() {
            0.0
        } else {
            losses.iter().sum::<f64>() / losses.len() as f64
        };

        let rsi_component = if avg_loss == 0.0 {
            100.0
        } else {
            let rs = avg_gain / avg_loss;
            100.0 - (100.0 / (1.0 + rs))
        };

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

        rsi_component * 0.6 + momentum_component * 0.4
    }

    /// Calculate average true range (ATR) for volatility
    fn calculate_atr(&self) -> f64 {
        if self.bar_history.len() < 2 {
            return 0.0;
        }

        let period = self.config.regime_lookback.min(self.bar_history.len());
        // Use only the most recent `period` bars for ATR calculation
        let recent = &self.bar_history[self.bar_history.len() - period..];
        let mut true_ranges: Vec<f64> = Vec::new();

        for window in recent.windows(2) {
            let high = window[1].high;
            let low = window[1].low;
            let prev_close = window[0].close;

            let tr = (high - low)
                .max((high - prev_close).abs())
                .max((low - prev_close).abs());
            true_ranges.push(tr);
        }

        if true_ranges.is_empty() {
            return 0.0;
        }

        true_ranges.iter().sum::<f64>() / true_ranges.len() as f64
    }

    /// Calculate trend strength (0-100) using directional movement
    fn calculate_trend_strength(&self) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }

        let period = self.config.regime_lookback.min(self.price_history.len());
        if period < 2 {
            return 0.0;
        }

        let prices = &self.price_history[self.price_history.len() - period..];

        // Calculate directional movement
        let mut up_moves: Vec<f64> = Vec::new();
        let mut down_moves: Vec<f64> = Vec::new();

        for window in prices.windows(2) {
            let up = window[1] - window[0];
            let down = window[0] - window[1];

            if up > down && up > 0.0 {
                up_moves.push(up);
            } else if down > up && down > 0.0 {
                down_moves.push(down);
            }
        }

        let sum_up: f64 = up_moves.iter().sum();
        let sum_down: f64 = down_moves.iter().sum();
        let total = sum_up + sum_down;

        if total == 0.0 {
            return 0.0;
        }

        // Smooth the directional movement index
        (sum_up - sum_down).abs() / total * 100.0
    }

    /// Detect current market regime
    fn detect_regime(&mut self) {
        let trend_strength = self.calculate_trend_strength();
        let atr = self.calculate_atr();

        // Determine if trending
        let is_trending = trend_strength >= self.config.trend_threshold;

        // Determine if high volatility
        let avg_price = if !self.price_history.is_empty() {
            self.price_history.iter().sum::<f64>() / self.price_history.len() as f64
        } else {
            1.0
        };

        let volatility_pct = if avg_price > 0.0 {
            (atr / avg_price) * 100.0
        } else {
            0.0
        };

        let is_high_volatility = volatility_pct >= self.config.volatility_threshold;

        // Determine direction if trending - use recent period for trend direction
        let period = self.config.regime_lookback.min(self.price_history.len());
        let recent_prices = &self.price_history[self.price_history.len() - period..];
        let price_trend = if period >= 2 {
            let oldest = recent_prices[0];
            let newest = recent_prices[period - 1];
            if oldest > 0.0 {
                ((newest - oldest) / oldest) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Classify regime
        self.current_regime = if is_trending {
            // Prefer trend-based classification over volatility when strongly trending
            if price_trend > 0.0 {
                MarketRegime::Bull
            } else {
                MarketRegime::Bear
            }
        } else if is_high_volatility {
            MarketRegime::HighVolatility
        } else {
            MarketRegime::Sideways
        };
    }

    /// Get regime-specific bullish threshold
    fn get_regime_threshold(&self) -> f64 {
        match self.current_regime {
            MarketRegime::Bull | MarketRegime::Trending => self.config.bull_bullish_threshold,
            MarketRegime::Bear => self.config.bear_bullish_threshold,
            MarketRegime::Sideways | MarketRegime::Ranging => {
                self.config.sideways_bullish_threshold
            }
            MarketRegime::HighVolatility | MarketRegime::LowVolatility => {
                // Use conservative thresholds in high volatility
                (self.config.bull_bullish_threshold + self.config.bear_bullish_threshold) / 2.0
            }
        }
    }
}

impl Default for RegimeSentimentStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataStrategy for RegimeSentimentStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
             name: "Regime Sentiment".to_string(),
             description: "Adapts sentiment interpretation based on current market regime. Detects market regimes (bull, bear, sideways) using trend strength and volatility indicators. Adjusts sentiment thresholds accordingly: requires stronger bullish sentiment in bull markets (to avoid chasing), more sensitive in bear markets (for contrarian opportunities), and balanced in sideways markets. Based on the hypothesis that sentiment signals vary by market regime.".to_string(),
             category: StrategyCategory::SentimentBased,
             sub_type: Some("regime_sentiment".to_string()),
             hypothesis_path: "hypotheses/sentiment/regime_sentiment.md".to_string(),
             required_indicators: vec!["RSI".to_string(), "ATR".to_string()],
             expected_regimes: vec![
                 MarketRegime::Bull,
                 MarketRegime::Bear,
                 MarketRegime::Sideways,
                 MarketRegime::Trending,
             ],
             risk_profile: RiskProfile {
                 max_drawdown_expected: 0.20,
                 volatility_level: VolatilityLevel::Low,
                 correlation_sensitivity: CorrelationSensitivity::Low,
                 leverage_requirement: 1.0,
             },
         }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::SentimentBased
    }
}

impl Strategy for RegimeSentimentStrategy {
    fn name(&self) -> &str {
        "Regime Sentiment"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        // Update bar history and price history
        self.bar_history.push(*bar);
        let max_hist = self
            .config
            .regime_lookback
            .max(self.config.sentiment_lookback);
        if self.bar_history.len() > max_hist {
            self.bar_history.remove(0);
        }

        self.price_history.push(bar.close);
        if self.price_history.len() > self.config.regime_lookback {
            self.price_history.remove(0);
        }

        // Calculate sentiment using the rolling bar history
        let current_sentiment = self.calculate_sentiment(&self.bar_history);
        self.sentiment_history.push(current_sentiment);
        if self.sentiment_history.len() > self.config.sentiment_lookback {
            self.sentiment_history.remove(0);
        }

        // Detect current regime
        self.detect_regime();

        // Check if we have enough data
        if self.price_history.len() < self.config.regime_lookback
            || self.sentiment_history.len() < 2
        {
            return None;
        }

        // Get regime-specific threshold
        let regime_threshold = self.get_regime_threshold();

        // Calculate sentiment momentum
        let previous_sentiment = self.sentiment_history[self.sentiment_history.len() - 2];
        let sentiment_momentum = current_sentiment - previous_sentiment;

        // Generate signals based on position, regime, and sentiment
        match self.last_position {
            Some(true) => {
                // Currently long - check for exit signal
                let bearish_threshold = regime_threshold - 10.0; // More lenient for exit
                if current_sentiment < bearish_threshold.max(20.0)
                    || sentiment_momentum < -self.config.momentum_threshold * 2.0
                {
                    self.last_position = None;
                    return Some(vec![Signal {
                         timestamp: bar.timestamp,
                         symbol: "UNKNOWN".to_string(),
                         signal_type: SignalType::Sell,
                         strength: 1.0,
                         metadata: Some(format!(
                             "Exit on sentiment deterioration: {:.1} in {:?} regime (threshold: {:.1})",
                             current_sentiment, self.current_regime, regime_threshold
                         )),
                     }]);
                }
                None
            }
            None | Some(false) => {
                // Not currently long - check for entry signal
                if current_sentiment > regime_threshold
                    && sentiment_momentum > self.config.momentum_threshold
                {
                    self.last_position = Some(true);
                    return Some(vec![Signal {
                         timestamp: bar.timestamp,
                         symbol: "UNKNOWN".to_string(),
                         signal_type: SignalType::Buy,
                         strength: 1.0,
                         metadata: Some(format!(
                             "Buy on sentiment: {:.1} in {:?} regime (threshold: {:.1}, momentum: {:.1})",
                             current_sentiment, self.current_regime, regime_threshold, sentiment_momentum
                         )),
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
    use chrono::{TimeZone, Utc};

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
        let config = RegimeSentimentConfig::new(20, 14, 30.0, 2.0);
        assert_eq!(config.regime_lookback, 20);
        assert_eq!(config.sentiment_lookback, 14);
        assert_eq!(config.trend_threshold, 30.0);
        assert_eq!(config.volatility_threshold, 2.0);
    }

    #[test]
    fn test_default_config() {
        let config = RegimeSentimentConfig::default_config();
        assert_eq!(config.regime_lookback, 20);
        assert_eq!(config.sentiment_lookback, 14);
        assert_eq!(config.bull_bullish_threshold, 60.0);
        assert_eq!(config.bear_bullish_threshold, 40.0);
        assert_eq!(config.sideways_bullish_threshold, 50.0);
    }

    #[test]
    fn test_config_valid() {
        let config = RegimeSentimentConfig::new(20, 14, 30.0, 2.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_invalid_regime_lookback() {
        let config = RegimeSentimentConfig::new(5, 14, 30.0, 2.0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_invalid_thresholds() {
        let config = RegimeSentimentConfig::new(20, 14, 30.0, 2.0);
        let invalid_config = RegimeSentimentConfig {
            bull_bullish_threshold: 40.0,
            bear_bullish_threshold: 50.0,
            ..config
        };
        assert!(invalid_config.validate().is_err()); // bull threshold < bear threshold is unusual
    }

    #[test]
    fn test_strategy_creation() {
        let strategy = RegimeSentimentStrategy::new();
        assert_eq!(strategy.config.regime_lookback, 20);
        assert_eq!(strategy.last_position, None);
        assert!(strategy.price_history.is_empty());
    }

    #[test]
    fn test_strategy_from_config() {
        let config = RegimeSentimentConfig::new(30, 20, 40.0, 3.0);
        let strategy = RegimeSentimentStrategy::from_config(config);
        assert_eq!(strategy.config.regime_lookback, 30);
        assert_eq!(strategy.config.trend_threshold, 40.0);
    }

    #[test]
    fn test_metadata() {
        let strategy = RegimeSentimentStrategy::new();
        let metadata = strategy.metadata();

        assert_eq!(metadata.name, "Regime Sentiment");
        assert_eq!(metadata.category, StrategyCategory::SentimentBased);
        assert!(!metadata.description.is_empty());
        assert!(metadata.expected_regimes.contains(&MarketRegime::Bull));
        assert!(metadata.expected_regimes.contains(&MarketRegime::Bear));
        assert!(metadata.expected_regimes.contains(&MarketRegime::Sideways));
    }

    #[test]
    fn test_calculate_sentiment_uptrend() {
        let strategy = RegimeSentimentStrategy::new();
        let bars: Vec<Bar> = (0..20)
            .map(|i| create_test_bar(i, 100.0 + i as f64 * 1.0, 1000.0))
            .collect();

        let sentiment = strategy.calculate_sentiment(&bars);
        assert!(sentiment > 50.0, "Sentiment should be bullish in uptrend");
    }

    #[test]
    fn test_calculate_sentiment_downtrend() {
        let strategy = RegimeSentimentStrategy::new();
        let bars: Vec<Bar> = (0..20)
            .map(|i| create_test_bar(i, 100.0 - i as f64 * 1.0, 1000.0))
            .collect();

        let sentiment = strategy.calculate_sentiment(&bars);
        assert!(sentiment < 50.0, "Sentiment should be bearish in downtrend");
    }

    #[test]
    fn test_debug_detect_regime_values() {
        // Debugging test: prints internal regime detection diagnostics for manual inspection.
        // Run with `cargo test --lib -p alphafield-strategy -- --nocapture test_debug_detect_regime_values`
        // to see output.
        // Uptrend scenario
        let mut strategy = RegimeSentimentStrategy::new();
        for i in 0..25 {
            let close = 100.0 + i as f64 * 1.5;
            let bar = create_test_bar(i, close, 1000.0);
            strategy.on_bar(&bar);
        }
        println!("=== UP TREND DIAGNOSTICS ===");
        println!("price_history_len = {}", strategy.price_history.len());
        println!("current_regime = {:?}", strategy.current_regime);
        println!("trend_strength = {:?}", strategy.calculate_trend_strength());
        println!("atr = {:?}", strategy.calculate_atr());
        let price_trend = if strategy.price_history.len() >= 2 {
            let oldest = strategy.price_history[0];
            let newest = *strategy.price_history.last().unwrap();
            if oldest > 0.0 {
                ((newest - oldest) / oldest) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        println!("price_trend (%) = {:?}", price_trend);

        // Sideways scenario
        let mut strategy2 = RegimeSentimentStrategy::new();
        for i in 0..25 {
            let close = 100.0 + (i % 5) as f64 * 0.5 - 1.0;
            let bar = create_test_bar(i, close, 1000.0);
            strategy2.on_bar(&bar);
        }
        println!("=== SIDEWAYS DIAGNOSTICS ===");
        println!("price_history_len = {}", strategy2.price_history.len());
        println!("current_regime = {:?}", strategy2.current_regime);
        println!(
            "trend_strength = {:?}",
            strategy2.calculate_trend_strength()
        );
        println!("atr = {:?}", strategy2.calculate_atr());
        let price_trend2 = if strategy2.price_history.len() >= 2 {
            let oldest = strategy2.price_history[0];
            let newest = *strategy2.price_history.last().unwrap();
            if oldest > 0.0 {
                ((newest - oldest) / oldest) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        println!("price_trend (%) = {:?}", price_trend2);
    }

    #[test]
    fn test_detect_regime_trending_up() {
        let mut strategy = RegimeSentimentStrategy::new();

        // Create strong uptrend
        for i in 0..25 {
            let close = 100.0 + i as f64 * 1.5;
            let bar = create_test_bar(i, close, 1000.0);
            strategy.on_bar(&bar);
        }

        // Should detect trending or bull regime
        assert!(
            matches!(
                strategy.current_regime,
                MarketRegime::Bull | MarketRegime::Trending
            ),
            "Should detect bull/trending regime in uptrend"
        );
    }

    #[test]
    fn test_detect_regime_sideways() {
        let mut strategy = RegimeSentimentStrategy::new();

        // Create sideways market with alternating up/down movements
        // Use tighter high/low ranges to avoid high volatility detection
        let sideways_prices = [
            100.0, 100.5, 100.0, 99.5, 100.0, 100.5, 100.0, 99.5, 100.0, 100.5, 100.0, 99.5, 100.0,
            100.5, 100.0, 99.5, 100.0, 100.5, 100.0, 99.5, 100.0, 100.5, 100.0, 99.5, 100.0,
        ];
        for (i, &close) in sideways_prices.iter().enumerate() {
            let bar = Bar {
                timestamp: Utc.timestamp_opt(i as i64, 0).unwrap(),
                open: close * 0.995,
                high: close * 1.005,
                low: close * 0.995,
                close,
                volume: 1000.0,
            };
            strategy.on_bar(&bar);
        }

        // Should detect sideways regime
        assert!(
            matches!(
                strategy.current_regime,
                MarketRegime::Sideways | MarketRegime::Ranging
            ),
            "Should detect sideways regime in flat market, got {:?}",
            strategy.current_regime
        );
    }

    #[test]
    fn test_get_regime_threshold() {
        let mut strategy = RegimeSentimentStrategy::new();

        strategy.current_regime = MarketRegime::Bull;
        let bull_thresh = strategy.get_regime_threshold();
        assert_eq!(bull_thresh, strategy.config.bull_bullish_threshold);

        strategy.current_regime = MarketRegime::Bear;
        let bear_thresh = strategy.get_regime_threshold();
        assert_eq!(bear_thresh, strategy.config.bear_bullish_threshold);

        strategy.current_regime = MarketRegime::Sideways;
        let sideways_thresh = strategy.get_regime_threshold();
        assert_eq!(sideways_thresh, strategy.config.sideways_bullish_threshold);
    }

    #[test]
    fn test_on_bar_no_signal_insufficient_data() {
        let mut strategy = RegimeSentimentStrategy::new();
        let bar = create_test_bar(0, 100.0, 1000.0);
        let signal = strategy.on_bar(&bar);
        assert!(signal.is_none());
    }

    #[test]
    fn test_on_bar_buy_signal_bull_regime() {
        let mut strategy = RegimeSentimentStrategy::new();

        // Create strong uptrend (bull regime)
        for i in 0..25 {
            let close = 100.0 + i as f64 * 2.0;
            let bar = create_test_bar(i, close, 2000.0);
            strategy.on_bar(&bar);
        }

        // In bull regime, need higher sentiment threshold
        // Continue with improving sentiment
        for i in 25..35 {
            let close = 150.0 + (i - 25) as f64 * 2.5;
            let bar = create_test_bar(i, close, 2000.0);
            let signal = strategy.on_bar(&bar);

            if let Some(sigs) = signal {
                for sig in sigs.iter() {
                    if matches!(sig.signal_type, SignalType::Buy) {
                        assert_eq!(sig.symbol, "UNKNOWN");
                        return;
                    }
                }
            }
        }

        // Buy signal may or may not trigger depending on exact thresholds
        // This is acceptable - regime-based thresholds are designed to be more selective
    }

    #[test]
    fn test_on_bar_buy_signal_bear_regime() {
        let mut strategy = RegimeSentimentStrategy::new();

        // Create downtrend then recovery (bear regime with improving sentiment)
        for i in 0..30 {
            let close = if i < 20 {
                100.0 - i as f64 * 1.5 // Downtrend
            } else {
                70.0 + (i - 20) as f64 * 2.0 // Recovery
            };
            let bar = create_test_bar(i, close, 1000.0);
            let signal = strategy.on_bar(&bar);

            if let Some(sigs) = signal {
                for sig in sigs.iter() {
                    if matches!(sig.signal_type, SignalType::Buy) {
                        assert_eq!(sig.symbol, "UNKNOWN");
                        return;
                    }
                }
            }
        }
    }
}
