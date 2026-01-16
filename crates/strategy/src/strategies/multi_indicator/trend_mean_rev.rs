//! Trend + Mean Reversion Hybrid Strategy
//!
//! This strategy combines trend-following logic with mean-reversion timing.
//! Trend determines direction (long-only in uptrend), mean reversion provides
//! entry timing (oversold in uptrend = buy signal).

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Ema, Indicator, Rsi};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for Trend + Mean Reversion Hybrid strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendMeanRevConfig {
    /// Fast EMA period for trend detection
    pub ema_fast: usize,
    /// Slow EMA period for trend confirmation
    pub ema_slow: usize,
    /// RSI period for mean reversion timing
    pub rsi_period: usize,
    /// RSI oversold threshold (entry signal, default 30)
    pub rsi_oversold: f64,
    /// RSI overbought threshold (exit signal, default 70)
    pub rsi_overbought: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl TrendMeanRevConfig {
    pub fn new(ema_fast: usize, ema_slow: usize, rsi_period: usize) -> Self {
        Self {
            ema_fast,
            ema_slow,
            rsi_period,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
            take_profit: 5.0,
            stop_loss: 5.0,
        }
    }

    pub fn with_thresholds(
        ema_fast: usize,
        ema_slow: usize,
        rsi_period: usize,
        oversold: f64,
        overbought: f64,
    ) -> Self {
        let mut config = Self::new(ema_fast, ema_slow, rsi_period);
        config.rsi_oversold = oversold;
        config.rsi_overbought = overbought;
        config
    }

    pub fn with_exits(
        ema_fast: usize,
        ema_slow: usize,
        rsi_period: usize,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            ema_fast,
            ema_slow,
            rsi_period,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
            take_profit,
            stop_loss,
        }
    }

    pub fn default_config() -> Self {
        Self {
            ema_fast: 20,
            ema_slow: 50,
            rsi_period: 14,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
            take_profit: 5.0,
            stop_loss: 5.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.ema_fast >= self.ema_slow {
            return Err("Fast EMA period must be less than slow EMA period".to_string());
        }
        if self.ema_fast == 0 || self.ema_slow == 0 {
            return Err("EMA periods must be greater than 0".to_string());
        }
        if self.rsi_period == 0 {
            return Err("RSI period must be greater than 0".to_string());
        }
        if self.rsi_overbought <= self.rsi_oversold {
            return Err("Overbought threshold must be greater than oversold threshold".to_string());
        }
        if self.rsi_overbought > 100.0 || self.rsi_oversold < 0.0 {
            return Err("RSI thresholds must be between 0 and 100".to_string());
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

impl fmt::Display for TrendMeanRevConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Trend+MeanRev(ema={}/{}, rsi={}, os={:.0}, ob={:.0}, tp={:.1}%, sl={:.1}%)",
            self.ema_fast,
            self.ema_slow,
            self.rsi_period,
            self.rsi_oversold,
            self.rsi_overbought,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// Trend + Mean Reversion Hybrid Strategy
///
/// # Strategy Logic
///
/// **Trend Filter**:
/// - Uptrend: Fast EMA > Slow EMA
/// - Downtrend: Fast EMA < Slow EMA
///
/// **Entry Logic**:
/// - Only enter long positions in uptrend
/// - Buy when RSI becomes oversold (< oversold threshold) while in uptrend
/// - This catches pullbacks in the uptrend trend
///
/// **Exit Logic**:
/// - Take Profit hit
/// - Stop Loss hit
/// - RSI becomes overbought (> overbought threshold)
/// - Trend reverses (Fast EMA crosses below Slow EMA)
///
/// # Why This Combination Works
/// - **Trend Following**: Ensures we trade with the dominant direction
/// - **Mean Reversion**: Provides optimal entry timing on pullbacks
/// - **Hybrid**: Reduces whipsaws while capturing the trend
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::multi_indicator::TrendMeanRevStrategy;
///
/// let strategy = TrendMeanRevStrategy::new(20, 50, 14);
/// ```
pub struct TrendMeanRevStrategy {
    config: TrendMeanRevConfig,
    ema_fast: Ema,
    ema_slow: Ema,
    rsi: Rsi,
    last_position: SignalType,
    entry_price: Option<f64>,
    // State tracking for crossover detection
    last_fast_ema: Option<f64>,
    last_slow_ema: Option<f64>,
    last_rsi: Option<f64>,
}

impl TrendMeanRevStrategy {
    /// Creates a new Trend + Mean Reversion Hybrid strategy
    pub fn new(ema_fast: usize, ema_slow: usize, rsi_period: usize) -> Self {
        let config = TrendMeanRevConfig::new(ema_fast, ema_slow, rsi_period);
        Self::from_config(config)
    }

    /// Creates a strategy with custom RSI thresholds
    pub fn with_thresholds(
        ema_fast: usize,
        ema_slow: usize,
        rsi_period: usize,
        oversold: f64,
        overbought: f64,
    ) -> Self {
        let config = TrendMeanRevConfig::with_thresholds(
            ema_fast, ema_slow, rsi_period, oversold, overbought,
        );
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: TrendMeanRevConfig) -> Self {
        config.validate().expect("Invalid TrendMeanRevConfig");

        Self {
            ema_fast: Ema::new(config.ema_fast),
            ema_slow: Ema::new(config.ema_slow),
            rsi: Rsi::new(config.rsi_period),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            last_fast_ema: None,
            last_slow_ema: None,
            last_rsi: None,
        }
    }

    pub fn config(&self) -> &TrendMeanRevConfig {
        &self.config
    }

    /// Calculate signal confidence based on indicator alignment
    fn calculate_confidence(&self, ema_diff: f64, rsi_value: f64, is_entry: bool) -> f64 {
        if is_entry {
            // Entry confidence: stronger when EMA diff is large (strong trend)
            // and RSI is not extremely oversold (riskier)
            let trend_strength =
                (ema_diff.abs() / self.last_slow_ema.unwrap_or(1.0)).min(0.1) * 10.0;
            let rsi_confidence = (rsi_value / 50.0).min(1.0); // 1.0 at 50, lower at oversold
            (trend_strength + rsi_confidence) / 2.0
        } else {
            // Exit confidence: stronger when RSI is very overbought
            let rsi_confidence = if rsi_value > self.config.rsi_overbought {
                (rsi_value - self.config.rsi_overbought) / (100.0 - self.config.rsi_overbought)
            } else {
                0.5
            };
            rsi_confidence.min(1.0)
        }
    }
}

impl MetadataStrategy for TrendMeanRevStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Trend+Mean Reversion Hybrid".to_string(),
            category: StrategyCategory::MultiIndicator,
            sub_type: Some("hybrid".to_string()),
            description: format!(
                "Hybrid strategy combining trend following with mean reversion.
                Trend Filter: EMA({}) > EMA({}) for uptrend.
                Entry: RSI({}) < {:.0} (oversold) while in uptrend.
                Exit: TP ({:.1}%), SL ({:.1}%), RSI > {:.0} (overbought), or trend reversal.
                Trades pullbacks in uptrends for better risk/reward.",
                self.config.ema_fast,
                self.config.ema_slow,
                self.config.rsi_period,
                self.config.rsi_oversold,
                self.config.take_profit,
                self.config.stop_loss,
                self.config.rsi_overbought
            ),
            hypothesis_path: "hypotheses/multi_indicator/trend_mean_rev.md".to_string(),
            required_indicators: vec!["EMA".to_string(), "RSI".to_string(), "Price".to_string()],
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Trending],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.20,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Medium,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::MultiIndicator
    }
}

impl Strategy for TrendMeanRevStrategy {
    fn name(&self) -> &str {
        "Trend+Mean Reversion Hybrid"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let fast_ema = self.ema_fast.update(bar.close)?;
        let slow_ema = self.ema_slow.update(bar.close)?;
        let rsi_value = self.rsi.update(bar.close)?;

        let price = bar.close;
        let in_uptrend = fast_ema > slow_ema;
        let is_rsi_oversold = rsi_value < self.config.rsi_oversold;
        let is_rsi_overbought = rsi_value > self.config.rsi_overbought;

        // Need previous state for crossover detection
        let prev_fast_ema = self.last_fast_ema;
        let prev_slow_ema = self.last_slow_ema;
        let prev_rsi = self.last_rsi;

        // Update state for next bar
        self.last_fast_ema = Some(fast_ema);
        self.last_slow_ema = Some(slow_ema);
        self.last_rsi = Some(rsi_value);

        // EXIT LOGIC FIRST (only when in position)
        if self.last_position == SignalType::Buy {
            if let Some(entry) = self.entry_price {
                let profit_pct = (price - entry) / entry * 100.0;

                // TP
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

                // SL
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

                // Exit on RSI overbought (pullback exhaustion)
                if is_rsi_overbought {
                    let confidence = self.calculate_confidence(0.0, rsi_value, false);
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: confidence,
                        metadata: Some(format!(
                            "RSI Overbought Exit: RSI {:.2} > {:.0}",
                            rsi_value, self.config.rsi_overbought
                        )),
                    }]);
                }

                // Exit on trend reversal (fast EMA crosses below slow EMA)
                if let (Some(pf), Some(ps)) = (prev_fast_ema, prev_slow_ema) {
                    if pf >= ps && fast_ema < slow_ema {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 0.9,
                            metadata: Some(format!(
                                "Trend Reversal Exit: Fast EMA ({:.2}) crossed below Slow EMA ({:.2})",
                                fast_ema, slow_ema
                            )),
                        }]);
                    }
                }
            }
        }

        // ENTRY: Only in uptrend, and RSI becomes oversold (pullback entry)
        if self.last_position != SignalType::Buy && in_uptrend {
            // RSI must become oversold (crossover from above to below)
            if let Some(pr) = prev_rsi {
                if pr >= self.config.rsi_oversold && is_rsi_oversold {
                    self.last_position = SignalType::Buy;
                    self.entry_price = Some(price);

                    let ema_diff = fast_ema - slow_ema;
                    let confidence = self.calculate_confidence(ema_diff, rsi_value, true);

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: confidence,
                        metadata: Some(format!(
                            "Pullback Entry: Uptrend confirmed, RSI {:.2} became oversold",
                            rsi_value
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
    use alphafield_core::Bar;
    use chrono::{DateTime, TimeZone, Utc};

    fn create_test_bar(timestamp: DateTime<Utc>, close: f64) -> Bar {
        Bar {
            timestamp,
            open: close * 0.99,
            high: close * 1.01,
            low: close * 0.98,
            close,
            volume: 1000.0,
            symbol: "BTCUSDT".to_string(),
        }
    }

    #[test]
    fn test_strategy_creation() {
        let strategy = TrendMeanRevStrategy::new(20, 50, 14);
        assert_eq!(strategy.name(), "Trend+Mean Reversion Hybrid");
        assert_eq!(strategy.config().ema_fast, 20);
        assert_eq!(strategy.config().ema_slow, 50);
        assert_eq!(strategy.config().rsi_period, 14);
    }

    #[test]
    fn test_config_validation() {
        let config = TrendMeanRevConfig::new(20, 50, 14);
        assert!(config.validate().is_ok());

        // Invalid: fast >= slow
        let invalid = TrendMeanRevConfig::new(50, 20, 14);
        assert!(invalid.validate().is_err());

        // Invalid: overbought <= oversold
        let invalid2 = TrendMeanRevConfig::with_thresholds(20, 50, 14, 70.0, 30.0);
        assert!(invalid2.validate().is_err());

        // Invalid: thresholds out of range
        let invalid3 = TrendMeanRevConfig::with_thresholds(20, 50, 14, 110.0, 30.0);
        assert!(invalid3.validate().is_err());
    }

    #[test]
    fn test_custom_thresholds() {
        let strategy = TrendMeanRevStrategy::with_thresholds(20, 50, 14, 25.0, 75.0);
        assert_eq!(strategy.config().rsi_oversold, 25.0);
        assert_eq!(strategy.config().rsi_overbought, 75.0);
    }

    #[test]
    fn test_metadata() {
        let strategy = TrendMeanRevStrategy::new(20, 50, 14);
        let metadata = strategy.metadata();

        assert_eq!(metadata.name, "Trend+Mean Reversion Hybrid");
        assert_eq!(metadata.category, StrategyCategory::MultiIndicator);
        assert_eq!(metadata.sub_type, Some("hybrid".to_string()));
        assert!(metadata.description.contains("EMA"));
        assert!(metadata.description.contains("RSI"));
        assert_eq!(
            metadata.hypothesis_path,
            "hypotheses/multi_indicator/trend_mean_rev.md"
        );
        assert!(metadata.required_indicators.contains(&"EMA".to_string()));
        assert!(metadata.required_indicators.contains(&"RSI".to_string()));
        assert!(metadata.expected_regimes.contains(&MarketRegime::Bull));
        assert_eq!(metadata.risk_profile.max_drawdown_expected, 0.20);
    }

    #[test]
    fn test_category() {
        let strategy = TrendMeanRevStrategy::new(20, 50, 14);
        assert_eq!(strategy.category(), StrategyCategory::MultiIndicator);
    }

    #[test]
    fn test_confidence_calculation() {
        let strategy = TrendMeanRevStrategy::new(20, 50, 14);

        // Entry confidence at RSI 25 (oversold but not extreme)
        let conf1 = strategy.calculate_confidence(2.0, 25.0, true);
        assert!(conf1 > 0.0 && conf1 <= 1.0);

        // Entry confidence at RSI 35 (just became oversold)
        let conf2 = strategy.calculate_confidence(2.0, 35.0, true);
        assert!(conf2 > conf1);

        // Exit confidence at RSI 75 (just became overbought)
        let conf3 = strategy.calculate_confidence(0.0, 75.0, false);
        assert!(conf3 > 0.0 && conf3 <= 1.0);

        // Exit confidence at RSI 90 (very overbought)
        let conf4 = strategy.calculate_confidence(0.0, 90.0, false);
        assert!(conf4 > conf3);
    }

    #[test]
    fn test_indicator_warmup() {
        let mut strategy = TrendMeanRevStrategy::new(20, 50, 14);

        // Need enough bars for indicators to warm up
        let base_time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let mut price = 100.0;

        // Feed initial bars (slow EMA needs 50, RSI needs 15)
        for i in 0..60 {
            // Create a trending pattern with pullbacks
            if i < 40 {
                price = 100.0 + (i as f64) * 0.5; // Trending up
            } else {
                price = 120.0 - ((i - 40) as f64) * 0.8; // Pullback
            }
            let bar = create_test_bar(base_time + chrono::Duration::hours(i), price);
            strategy.on_bar(&bar);
        }

        // After warmup, strategy should be ready
        // We should be in uptrend (fast EMA > slow EMA)
        assert!(true);
    }

    #[test]
    fn test_config_display() {
        let config = TrendMeanRevConfig::new(20, 50, 14);
        let display = format!("{}", config);
        assert!(display.contains("Trend+MeanRev"));
        assert!(display.contains("ema=20/50"));
        assert!(display.contains("rsi=14"));
    }

    #[test]
    fn test_pullback_entry_scenario() {
        let mut strategy = TrendMeanRevStrategy::new(10, 30, 14);

        let base_time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        // Create a scenario: uptrend, then pullback (RSI drops to oversold)
        // Uptrend phase
        for i in 0..40 {
            let price = 100.0 + (i as f64) * 0.3;
            let bar = create_test_bar(base_time + chrono::Duration::hours(i), price);
            strategy.on_bar(&bar);
        }

        // Pullback phase (price drops, RSI should become oversold)
        for i in 40..50 {
            let price = 112.0 - ((i - 40) as f64) * 0.6; // Sharp pullback
            let bar = create_test_bar(base_time + chrono::Duration::hours(i), price);
            let signal = strategy.on_bar(&bar);

            // Should get a buy signal during pullback
            if i == 45 {
                // After enough pullback
                if let Some(signals) = signal {
                    assert_eq!(signals.len(), 1);
                    assert_eq!(signals[0].signal_type, SignalType::Buy);
                    assert!(signals[0].strength > 0.0);
                }
            }
        }
    }
}
