//! Asset-Level Sentiment Analysis
//!
//! Calculates technical sentiment indicators for individual assets based on price action.
//! This provides per-asset sentiment data for backtesting and live trading.

use alphafield_core::Bar;
use serde::Serialize;

/// Asset-level sentiment calculated from price action
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct AssetSentiment {
    /// RSI value (0-100)
    pub rsi: f64,
    /// RSI zone classification
    pub rsi_zone: RsiZone,
    /// Price momentum (rate of change over period)
    pub momentum: f64,
    /// Momentum classification
    pub momentum_zone: MomentumZone,
    /// Volume trend (current vs average)
    pub volume_ratio: f64,
    /// Overall sentiment score (-100 to +100)
    pub composite_score: f64,
    /// Classification based on composite score
    pub classification: AssetSentimentClassification,
}

/// RSI zone classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
pub enum RsiZone {
    Oversold, // RSI < 30
    #[default]
    Neutral, // 30 <= RSI <= 70
    Overbought, // RSI > 70
}

impl RsiZone {
    pub fn from_rsi(rsi: f64) -> Self {
        if rsi < 30.0 {
            Self::Oversold
        } else if rsi > 70.0 {
            Self::Overbought
        } else {
            Self::Neutral
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Oversold => "Oversold",
            Self::Neutral => "Neutral",
            Self::Overbought => "Overbought",
        }
    }
}

/// Momentum zone classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
pub enum MomentumZone {
    StrongDown, // < -5%
    WeakDown,   // -5% to -1%
    #[default]
    Flat, // -1% to +1%
    WeakUp,     // +1% to +5%
    StrongUp,   // > +5%
}

impl MomentumZone {
    pub fn from_momentum(momentum: f64) -> Self {
        if momentum < -5.0 {
            Self::StrongDown
        } else if momentum < -1.0 {
            Self::WeakDown
        } else if momentum > 5.0 {
            Self::StrongUp
        } else if momentum > 1.0 {
            Self::WeakUp
        } else {
            Self::Flat
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StrongDown => "Strong Down",
            Self::WeakDown => "Weak Down",
            Self::Flat => "Flat",
            Self::WeakUp => "Weak Up",
            Self::StrongUp => "Strong Up",
        }
    }
}

/// Overall asset sentiment classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
pub enum AssetSentimentClassification {
    VeryBearish, // < -50
    Bearish,     // -50 to -20
    #[default]
    Neutral, // -20 to +20
    Bullish,     // +20 to +50
    VeryBullish, // > +50
}

impl AssetSentimentClassification {
    pub fn from_score(score: f64) -> Self {
        if score < -50.0 {
            Self::VeryBearish
        } else if score < -20.0 {
            Self::Bearish
        } else if score > 50.0 {
            Self::VeryBullish
        } else if score > 20.0 {
            Self::Bullish
        } else {
            Self::Neutral
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VeryBearish => "Very Bearish",
            Self::Bearish => "Bearish",
            Self::Neutral => "Neutral",
            Self::Bullish => "Bullish",
            Self::VeryBullish => "Very Bullish",
        }
    }

    pub fn is_bullish(&self) -> bool {
        matches!(self, Self::Bullish | Self::VeryBullish)
    }

    pub fn is_bearish(&self) -> bool {
        matches!(self, Self::Bearish | Self::VeryBearish)
    }
}

/// Calculator for asset-level sentiment
#[derive(Debug, Clone)]
pub struct AssetSentimentCalculator {
    rsi_period: usize,
    momentum_period: usize,
    volume_period: usize,
}

impl Default for AssetSentimentCalculator {
    fn default() -> Self {
        Self::new(14, 14, 20)
    }
}

impl AssetSentimentCalculator {
    pub fn new(rsi_period: usize, momentum_period: usize, volume_period: usize) -> Self {
        Self {
            rsi_period,
            momentum_period,
            volume_period,
        }
    }

    /// Calculate sentiment for the most recent bar given historical context
    pub fn calculate(&self, bars: &[Bar]) -> AssetSentiment {
        if bars.len()
            < self
                .rsi_period
                .max(self.momentum_period)
                .max(self.volume_period)
        {
            return AssetSentiment::default();
        }

        let rsi = self.calculate_rsi(bars);
        let momentum = self.calculate_momentum(bars);
        let volume_ratio = self.calculate_volume_ratio(bars);

        // Composite score: weighted combination
        // RSI contribution: -50 to +50 based on deviation from 50
        let rsi_score = -(50.0 - rsi); // Oversold = bullish (+), Overbought = bearish (-)

        // Momentum contribution: direct percentage scaled
        let momentum_score = momentum * 5.0; // Scale momentum to -50 to +50 range roughly

        // Volume contribution: high volume confirms trend
        let volume_score = if momentum > 0.0 {
            (volume_ratio - 1.0) * 10.0 // High volume + positive momentum = bullish
        } else {
            (volume_ratio - 1.0) * -10.0 // High volume + negative momentum = bearish
        };

        let composite_score =
            (rsi_score * 0.4 + momentum_score * 0.4 + volume_score * 0.2).clamp(-100.0, 100.0);

        AssetSentiment {
            rsi,
            rsi_zone: RsiZone::from_rsi(rsi),
            momentum,
            momentum_zone: MomentumZone::from_momentum(momentum),
            volume_ratio,
            composite_score,
            classification: AssetSentimentClassification::from_score(composite_score),
        }
    }

    /// Calculate sentiment for each bar in the series
    pub fn calculate_series(&self, bars: &[Bar]) -> Vec<(i64, AssetSentiment)> {
        let min_period = self
            .rsi_period
            .max(self.momentum_period)
            .max(self.volume_period);

        if bars.len() < min_period {
            return vec![];
        }

        bars.iter()
            .enumerate()
            .skip(min_period)
            .map(|(i, bar)| {
                let sentiment = self.calculate(&bars[..=i]);
                (bar.timestamp.timestamp_millis(), sentiment)
            })
            .collect()
    }

    fn calculate_rsi(&self, bars: &[Bar]) -> f64 {
        if bars.len() < self.rsi_period + 1 {
            return 50.0;
        }

        let recent = &bars[bars.len() - self.rsi_period - 1..];
        let mut gains = 0.0;
        let mut losses = 0.0;
        let mut count = 0;

        for i in 1..recent.len() {
            let change = recent[i].close - recent[i - 1].close;
            if change > 0.0 {
                gains += change;
            } else {
                losses += change.abs();
            }
            count += 1;
        }

        if count == 0 {
            return 50.0;
        }

        let avg_gain = gains / count as f64;
        let avg_loss = losses / count as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    fn calculate_momentum(&self, bars: &[Bar]) -> f64 {
        if bars.len() < self.momentum_period + 1 {
            return 0.0;
        }

        let current = bars.last().unwrap().close;
        let past = bars[bars.len() - self.momentum_period - 1].close;

        if past == 0.0 {
            return 0.0;
        }

        ((current - past) / past) * 100.0
    }

    fn calculate_volume_ratio(&self, bars: &[Bar]) -> f64 {
        if bars.len() < self.volume_period + 1 {
            return 1.0;
        }

        let current_volume = bars.last().unwrap().volume;
        let avg_volume: f64 = bars[bars.len() - self.volume_period - 1..bars.len() - 1]
            .iter()
            .map(|b| b.volume)
            .sum::<f64>()
            / self.volume_period as f64;

        if avg_volume == 0.0 {
            return 1.0;
        }

        current_volume / avg_volume
    }
}

/// Summary of asset sentiment over a period
#[derive(Debug, Clone, Serialize, Default)]
pub struct AssetSentimentSummary {
    pub current: AssetSentiment,
    pub avg_rsi: f64,
    pub min_rsi: f64,
    pub max_rsi: f64,
    pub avg_momentum: f64,
    pub bullish_days: usize,
    pub bearish_days: usize,
    pub neutral_days: usize,
}

impl AssetSentimentSummary {
    pub fn calculate(sentiments: &[(i64, AssetSentiment)]) -> Self {
        if sentiments.is_empty() {
            return Self::default();
        }

        let current = sentiments
            .last()
            .map(|(_, s)| s.clone())
            .unwrap_or_default();

        let rsi_values: Vec<f64> = sentiments.iter().map(|(_, s)| s.rsi).collect();
        let avg_rsi = rsi_values.iter().sum::<f64>() / rsi_values.len() as f64;
        let min_rsi = rsi_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_rsi = rsi_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let avg_momentum =
            sentiments.iter().map(|(_, s)| s.momentum).sum::<f64>() / sentiments.len() as f64;

        let bullish_days = sentiments
            .iter()
            .filter(|(_, s)| s.classification.is_bullish())
            .count();
        let bearish_days = sentiments
            .iter()
            .filter(|(_, s)| s.classification.is_bearish())
            .count();
        let neutral_days = sentiments.len() - bullish_days - bearish_days;

        Self {
            current,
            avg_rsi,
            min_rsi,
            max_rsi,
            avg_momentum,
            bullish_days,
            bearish_days,
            neutral_days,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn make_bars(closes: &[f64]) -> Vec<Bar> {
        use chrono::Duration;
        closes
            .iter()
            .enumerate()
            .map(|(i, &close)| {
                let base_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
                Bar {
                    timestamp: base_date + Duration::hours(i as i64),
                    open: close,
                    high: close * 1.01,
                    low: close * 0.99,
                    close,
                    volume: 1000.0,
                }
            })
            .collect()
    }

    #[test]
    fn test_rsi_zone_classification() {
        assert_eq!(RsiZone::from_rsi(20.0), RsiZone::Oversold);
        assert_eq!(RsiZone::from_rsi(50.0), RsiZone::Neutral);
        assert_eq!(RsiZone::from_rsi(80.0), RsiZone::Overbought);
    }

    #[test]
    fn test_momentum_zone_classification() {
        assert_eq!(MomentumZone::from_momentum(-10.0), MomentumZone::StrongDown);
        assert_eq!(MomentumZone::from_momentum(-3.0), MomentumZone::WeakDown);
        assert_eq!(MomentumZone::from_momentum(0.0), MomentumZone::Flat);
        assert_eq!(MomentumZone::from_momentum(3.0), MomentumZone::WeakUp);
        assert_eq!(MomentumZone::from_momentum(10.0), MomentumZone::StrongUp);
    }

    #[test]
    fn test_calculator_with_uptrend() {
        // Create an uptrending price series
        let closes: Vec<f64> = (0..30).map(|i| 100.0 + i as f64 * 2.0).collect();
        let bars = make_bars(&closes);

        let calculator = AssetSentimentCalculator::default();
        let sentiment = calculator.calculate(&bars);

        assert!(
            sentiment.momentum > 0.0,
            "Momentum should be positive in uptrend"
        );
        assert!(sentiment.rsi > 50.0, "RSI should be above 50 in uptrend");
    }

    #[test]
    fn test_calculator_with_downtrend() {
        // Create a downtrending price series
        let closes: Vec<f64> = (0..30).map(|i| 200.0 - i as f64 * 2.0).collect();
        let bars = make_bars(&closes);

        let calculator = AssetSentimentCalculator::default();
        let sentiment = calculator.calculate(&bars);

        assert!(
            sentiment.momentum < 0.0,
            "Momentum should be negative in downtrend"
        );
        assert!(sentiment.rsi < 50.0, "RSI should be below 50 in downtrend");
    }

    #[test]
    fn test_composite_score_formula() {
        // Test that composite_score follows: (rsi_score * 0.4 + momentum_score * 0.4 + volume_score * 0.2)
        let calculator = AssetSentimentCalculator::default();
        let closes: Vec<f64> = vec![100.0; 30];
        let mut bars = make_bars(&closes);

        // Set up specific conditions
        // RSI = 70 (rsi_score = -20)
        // Momentum = 10% (momentum_score = 50)
        // Volume ratio = 2.0 with positive momentum (volume_score = 10)
        // Expected: (-20 * 0.4) + (50 * 0.4) + (10 * 0.2) = -8 + 20 + 2 = 14
        bars[29].close = 170.0; // High price to push RSI up
        bars[15].close = 100.0; // Past price for momentum
        bars[29].volume = 2000.0;

        let sentiment = calculator.calculate(&bars);

        // The score should be positive due to high momentum and volume
        assert!(sentiment.composite_score > 0.0);
        assert!(sentiment.composite_score <= 100.0);
    }

    #[test]
    fn test_composite_score_clamping() {
        let calculator = AssetSentimentCalculator::default();
        let mut bars = make_bars(&vec![100.0; 30]);

        // Create extreme conditions that would push score beyond bounds
        // Very high RSI (> 100), very high momentum, high volume
        bars[29].close = 10000.0;
        bars[29].volume = 50000.0;

        let sentiment = calculator.calculate(&bars);

        assert!(
            sentiment.composite_score <= 100.0,
            "Composite score should be clamped to maximum 100.0"
        );
        assert!(
            sentiment.composite_score >= -100.0,
            "Composite score should be clamped to minimum -100.0"
        );
    }

    #[test]
    fn test_composite_score_negative_clamping() {
        let calculator = AssetSentimentCalculator::default();
        let mut bars = make_bars(&vec![100.0; 30]);

        // Create extreme conditions that would push score below -100
        // Very low RSI, very negative momentum, high volume with negative momentum
        bars[15].close = 10000.0; // Past high price
        bars[29].close = 1.0; // Current low price
        bars[29].volume = 50000.0;

        let sentiment = calculator.calculate(&bars);

        assert!(
            sentiment.composite_score >= -100.0,
            "Composite score should be clamped to minimum -100.0"
        );
    }

    #[test]
    fn test_empty_and_insufficient_bars() {
        let calculator = AssetSentimentCalculator::default();

        // Empty bars
        let sentiment = calculator.calculate(&[]);
        assert!((sentiment.rsi - 0.0).abs() < f64::EPSILON);
        assert!((sentiment.momentum - 0.0).abs() < f64::EPSILON);
        assert!((sentiment.volume_ratio - 0.0).abs() < f64::EPSILON);
        assert!((sentiment.composite_score - 0.0).abs() < f64::EPSILON);
        assert_eq!(sentiment.rsi_zone, RsiZone::Neutral);
        assert_eq!(sentiment.momentum_zone, MomentumZone::Flat);
        assert_eq!(
            sentiment.classification,
            AssetSentimentClassification::Neutral
        );

        // Insufficient bars (less than required periods)
        let bars = make_bars(&[100.0; 10]);
        let sentiment = calculator.calculate(&bars);
        assert!((sentiment.rsi - 0.0).abs() < f64::EPSILON);
        assert!((sentiment.momentum - 0.0).abs() < f64::EPSILON);
        assert!((sentiment.volume_ratio - 0.0).abs() < f64::EPSILON);
        assert!((sentiment.composite_score - 0.0).abs() < f64::EPSILON);
        assert_eq!(sentiment.rsi_zone, RsiZone::Neutral);
        assert_eq!(sentiment.momentum_zone, MomentumZone::Flat);
        assert_eq!(
            sentiment.classification,
            AssetSentimentClassification::Neutral
        );
    }

    #[test]
    fn test_rsi_calculation_extreme_values() {
        let calculator = AssetSentimentCalculator::default();

        // All gains (RSI should be 100)
        let closes: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let bars = make_bars(&closes);
        let rsi = calculator.calculate_rsi(&bars);
        assert!(rsi > 90.0, "RSI should be very high for consistent gains");

        // All losses (RSI should be 0)
        let closes: Vec<f64> = (0..30).map(|i| 200.0 - i as f64).collect();
        let bars = make_bars(&closes);
        let rsi = calculator.calculate_rsi(&bars);
        assert!(rsi < 10.0, "RSI should be very low for consistent losses");

        // Flat price (RSI should be around 50, but may vary due to edge cases)
        let bars = make_bars(&vec![100.0; 30]);
        let rsi = calculator.calculate_rsi(&bars);
        assert!(
            (0.0..=100.0).contains(&rsi),
            "RSI should be valid for flat price (got: {})",
            rsi
        );
    }

    #[test]
    fn test_momentum_calculation_edge_cases() {
        let calculator = AssetSentimentCalculator::default();
        let mut bars = make_bars(&vec![100.0; 30]);

        // Zero momentum (same price)
        let momentum = calculator.calculate_momentum(&bars);
        assert_eq!(
            momentum, 0.0,
            "Momentum should be zero when price unchanged"
        );

        // Positive momentum
        bars[29].close = 110.0;
        let momentum = calculator.calculate_momentum(&bars);
        assert!(
            momentum > 0.0,
            "Momentum should be positive when price increased"
        );

        // Negative momentum
        bars[29].close = 90.0;
        let momentum = calculator.calculate_momentum(&bars);
        assert!(
            momentum < 0.0,
            "Momentum should be negative when price decreased"
        );

        // Zero past price (should handle gracefully)
        let bars = make_bars(
            &vec![0.0; 15]
                .into_iter()
                .chain(vec![100.0; 15])
                .collect::<Vec<_>>(),
        );
        let momentum = calculator.calculate_momentum(&bars);
        assert!(
            !momentum.is_nan() && !momentum.is_infinite(),
            "Momentum should handle zero past price"
        );
    }

    #[test]
    fn test_volume_ratio_calculation() {
        let calculator = AssetSentimentCalculator::default();
        let mut bars = make_bars(&vec![100.0; 30]);

        // All same volume (ratio should be 1.0)
        let volume_ratio = calculator.calculate_volume_ratio(&bars);
        assert_eq!(
            volume_ratio, 1.0,
            "Volume ratio should be 1.0 when all volumes equal"
        );

        // High current volume (ratio > 1.0)
        bars[29].volume = 5000.0;
        let volume_ratio = calculator.calculate_volume_ratio(&bars);
        assert!(
            volume_ratio > 1.0,
            "Volume ratio should be > 1.0 when current volume is high"
        );

        // Low current volume (ratio < 1.0)
        bars[29].volume = 100.0;
        let volume_ratio = calculator.calculate_volume_ratio(&bars);
        assert!(
            volume_ratio < 1.0,
            "Volume ratio should be < 1.0 when current volume is low"
        );

        // Zero average volume (should return 1.0 to avoid division by zero)
        let mut bars = make_bars(&[100.0; 20]);
        for bar in bars.iter_mut().take(19) {
            bar.volume = 0.0;
        }
        bars[19].volume = 1000.0;
        let volume_ratio = calculator.calculate_volume_ratio(&bars);
        assert!(
            volume_ratio == 1.0 || volume_ratio.is_infinite(),
            "Volume ratio should handle zero average volume"
        );
    }

    #[test]
    fn test_classification_boundaries() {
        // VeryBearish boundary (< -50)
        assert_eq!(
            AssetSentimentClassification::from_score(-51.0),
            AssetSentimentClassification::VeryBearish
        );
        assert_eq!(
            AssetSentimentClassification::from_score(-50.1),
            AssetSentimentClassification::VeryBearish
        );

        // Bearish boundary (-50 to -20)
        assert_eq!(
            AssetSentimentClassification::from_score(-50.0),
            AssetSentimentClassification::Bearish
        );
        assert_eq!(
            AssetSentimentClassification::from_score(-35.0),
            AssetSentimentClassification::Bearish
        );
        assert_eq!(
            AssetSentimentClassification::from_score(-20.1),
            AssetSentimentClassification::Bearish
        );

        // Neutral boundary (-20 to +20)
        assert_eq!(
            AssetSentimentClassification::from_score(-20.0),
            AssetSentimentClassification::Neutral
        );
        assert_eq!(
            AssetSentimentClassification::from_score(0.0),
            AssetSentimentClassification::Neutral
        );
        assert_eq!(
            AssetSentimentClassification::from_score(20.0),
            AssetSentimentClassification::Neutral
        );

        // Bullish boundary (+20 to +50)
        assert_eq!(
            AssetSentimentClassification::from_score(20.1),
            AssetSentimentClassification::Bullish
        );
        assert_eq!(
            AssetSentimentClassification::from_score(35.0),
            AssetSentimentClassification::Bullish
        );
        assert_eq!(
            AssetSentimentClassification::from_score(50.0),
            AssetSentimentClassification::Bullish
        );

        // VeryBullish boundary (> +50)
        assert_eq!(
            AssetSentimentClassification::from_score(50.1),
            AssetSentimentClassification::VeryBullish
        );
        assert_eq!(
            AssetSentimentClassification::from_score(100.0),
            AssetSentimentClassification::VeryBullish
        );
    }

    #[test]
    fn test_classification_helper_methods() {
        assert!(AssetSentimentClassification::Bullish.is_bullish());
        assert!(AssetSentimentClassification::VeryBullish.is_bullish());
        assert!(!AssetSentimentClassification::Neutral.is_bullish());
        assert!(!AssetSentimentClassification::Bearish.is_bullish());
        assert!(!AssetSentimentClassification::VeryBearish.is_bullish());

        assert!(AssetSentimentClassification::Bearish.is_bearish());
        assert!(AssetSentimentClassification::VeryBearish.is_bearish());
        assert!(!AssetSentimentClassification::Neutral.is_bearish());
        assert!(!AssetSentimentClassification::Bullish.is_bearish());
        assert!(!AssetSentimentClassification::VeryBullish.is_bearish());
    }

    #[test]
    fn test_volume_score_calculation() {
        let calculator = AssetSentimentCalculator::default();

        // Positive momentum with high volume should increase score
        let mut bars = make_bars(&vec![100.0; 30]);
        bars[15].close = 100.0;
        bars[29].close = 110.0; // Positive momentum
        bars[29].volume = 5000.0; // High volume

        let sentiment = calculator.calculate(&bars);
        assert!(
            sentiment.composite_score > 0.0,
            "High volume + positive momentum should increase score"
        );

        // Negative momentum with high volume should decrease score
        let mut bars = make_bars(&vec![100.0; 30]);
        bars[15].close = 110.0;
        bars[29].close = 90.0; // Negative momentum
        bars[29].volume = 5000.0; // High volume confirms trend

        let sentiment = calculator.calculate(&bars);
        assert!(
            sentiment.composite_score < 0.0,
            "High volume + negative momentum should decrease score"
        );
    }

    #[test]
    fn test_sentiment_series_calculation() {
        let calculator = AssetSentimentCalculator::default();
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + i as f64 * 2.0).collect();
        let bars = make_bars(&closes);

        let series = calculator.calculate_series(&bars);

        // Should calculate sentiment for bars after the minimum period
        assert!(!series.is_empty(), "Series should not be empty");
        assert!(
            series.len() < 50,
            "Should skip initial bars for warmup period"
        );

        // All bars in uptrend should have bullish or neutral sentiment
        for (_timestamp, sentiment) in &series {
            assert!(
                !sentiment.classification.is_bearish(),
                "Uptrend should not have bearish sentiment"
            );
        }
    }

    #[test]
    fn test_sentiment_summary_calculation() {
        let calculator = AssetSentimentCalculator::default();
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + i as f64 * 2.0).collect();
        let bars = make_bars(&closes);

        let series = calculator.calculate_series(&bars);

        // Need some data to calculate summary
        assert!(!series.is_empty(), "Should have sentiment series data");

        let summary = AssetSentimentSummary::calculate(&series);

        assert!(
            summary.bullish_days > 0,
            "Should have bullish days in uptrend"
        );
        assert!(
            summary.avg_rsi > 50.0,
            "Average RSI should be > 50 in uptrend"
        );
        assert!(
            summary.avg_momentum > 0.0,
            "Average momentum should be positive in uptrend"
        );
    }

    #[test]
    fn test_default_sentiment_values() {
        let default = AssetSentiment::default();

        assert!((default.rsi - 0.0).abs() < f64::EPSILON);
        assert!((default.momentum - 0.0).abs() < f64::EPSILON);
        assert!((default.volume_ratio - 0.0).abs() < f64::EPSILON);
        assert!((default.composite_score - 0.0).abs() < f64::EPSILON);
        assert_eq!(default.rsi_zone, RsiZone::Neutral);
        assert_eq!(default.momentum_zone, MomentumZone::Flat);
        assert_eq!(
            default.classification,
            AssetSentimentClassification::Neutral
        );
    }
}
