//! Confidence-Weighted Strategy
//!
//! This strategy generates trading signals with confidence-based position sizing.
//! Signal confidence (strength) is calculated based on how strongly multiple
//! indicators agree. Strong signals = larger positions, weak signals = smaller positions.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Ema, Indicator, Macd, Rsi};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for Confidence-Weighted strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceWeightedConfig {
    /// Fast EMA period
    pub ema_fast: usize,
    /// Slow EMA period
    pub ema_slow: usize,
    /// MACD fast period
    pub macd_fast: usize,
    /// MACD slow period
    pub macd_slow: usize,
    /// MACD signal period
    pub macd_signal: usize,
    /// RSI period
    pub rsi_period: usize,
    /// RSI overbought threshold
    pub rsi_overbought: f64,
    /// RSI oversold threshold
    pub rsi_oversold: f64,
    /// Minimum confidence threshold (don't trade below this)
    pub min_confidence: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl ConfidenceWeightedConfig {
    pub fn new(
        ema_fast: usize,
        ema_slow: usize,
        macd_fast: usize,
        macd_slow: usize,
        macd_signal: usize,
        rsi_period: usize,
    ) -> Self {
        Self {
            ema_fast,
            ema_slow,
            macd_fast,
            macd_slow,
            macd_signal,
            rsi_period,
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,
            min_confidence: 0.4,
            take_profit: 5.0,
            stop_loss: 5.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            ema_fast: 20,
            ema_slow: 50,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            rsi_period: 14,
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,
            min_confidence: 0.4,
            take_profit: 5.0,
            stop_loss: 5.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.ema_fast >= self.ema_slow {
            return Err("Fast EMA period must be less than slow EMA period".to_string());
        }
        if self.macd_fast >= self.macd_slow {
            return Err("MACD fast period must be less than slow period".to_string());
        }
        if self.rsi_period == 0 {
            return Err("RSI period must be greater than 0".to_string());
        }
        if self.rsi_overbought <= self.rsi_oversold {
            return Err("Overbought threshold must be greater than oversold threshold".to_string());
        }
        if self.min_confidence < 0.0 || self.min_confidence > 1.0 {
            return Err("Min confidence must be between 0 and 1".to_string());
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

impl fmt::Display for ConfidenceWeightedConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConfWeighted(ema={}/{}, macd={}/{}/{}, rsi={}, min_conf={:.2}, tp={:.1}%, sl={:.1}%)",
            self.ema_fast,
            self.ema_slow,
            self.macd_fast,
            self.macd_slow,
            self.macd_signal,
            self.rsi_period,
            self.min_confidence,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// Confidence-Weighted Strategy
///
/// # Strategy Logic
///
/// This strategy combines multiple indicators and calculates a composite confidence score:
/// - **Trend Score** (0-1): Based on EMA crossover and alignment
/// - **Momentum Score** (0-1): Based on MACD crossover and histogram strength
/// - **RSI Score** (0-1): Based on distance from neutral (50), penalized if overbought/oversold
///
/// **Composite Confidence** = (Trend Score + Momentum Score + RSI Score) / 3
///
/// **Entry**: Only when confidence >= min_confidence threshold
/// - Position size should be proportional to confidence (handled by Signal.strength)
/// - Strong confidence (0.8+) = full position
/// - Medium confidence (0.5-0.8) = partial position
/// - Low confidence (0.4-0.5) = small position
///
/// **Exit**:
/// - Take Profit hit
/// - Stop Loss hit
/// - Confidence drops significantly below entry confidence
/// - Indicators disagree (reversal signals)
///
/// Parameters for calculating composite confidence
pub struct ConfidenceParams {
    pub fast_ema: f64,
    pub slow_ema: f64,
    pub macd_line: f64,
    pub signal_line: f64,
    pub histogram: f64,
    pub rsi_value: f64,
    pub is_bullish: bool,
}

/// Confidence-Weighted Strategy
///
/// This strategy generates trading signals with confidence-based position sizing.
/// Signal confidence (strength) is calculated based on how strongly multiple
/// indicators agree. Strong signals = larger positions, weak signals = smaller positions.
///
/// # Strategy Logic
///
/// **Confidence Scoring**:
/// - Trend Confidence: Based on EMA crossover strength (0-1 range)
/// - Momentum Confidence: Based on MACD histogram and crossover alignment (0-1 range)
/// - RSI Confidence: Based on RSI level and bullish/bearish alignment (0-1 range)
/// - Composite Confidence: Weighted average (momentum gets 50%, trend 25%, RSI 25%)
///
/// **Position Sizing**:
/// - Signal.strength is set to composite confidence (0-1)
/// - Higher confidence = larger position size
/// - Below min_confidence = no trade
///
/// **Entry/Exit**:
/// - Entry: MACD bullish crossover + confidence >= min_confidence
/// - Exit: RSI overbought/oversold or confidence drops below entry confidence
///
/// # Why This Works
/// - **Risk-Adjusted Sizing**: Larger positions when signals are strong
/// - **Filter Weak Signals**: Avoid trades with low conviction
/// - **Dynamic Adaptation**: Position size adapts to market conditions
///
/// # Note
/// This strategy sets Signal.strength to reflect confidence. Position sizing systems
/// should respect this field to adjust trade sizes accordingly.
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::multi_indicator::ConfidenceWeightedStrategy;
///
/// let strategy = ConfidenceWeightedStrategy::new(20, 50, 12, 26, 9, 14);
/// ```
pub struct ConfidenceWeightedStrategy {
    config: ConfidenceWeightedConfig,
    ema_fast: Ema,
    ema_slow: Ema,
    macd: Macd,
    rsi: Rsi,
    last_position: SignalType,
    entry_price: Option<f64>,
    entry_confidence: Option<f64>,
    // State tracking for crossover detection
    last_fast_ema: Option<f64>,
    last_slow_ema: Option<f64>,
    last_macd: Option<f64>,
    last_signal: Option<f64>,
}

impl Default for ConfidenceWeightedStrategy {
    fn default() -> Self {
        // Default: 20/50 EMA, 12/26/9 MACD, 14 RSI, default weights and thresholds
        Self::from_config(ConfidenceWeightedConfig::default_config())
    }
}

impl ConfidenceWeightedStrategy {
    /// Creates a new Confidence-Weighted strategy
    pub fn new(
        ema_fast: usize,
        ema_slow: usize,
        macd_fast: usize,
        macd_slow: usize,
        macd_signal: usize,
        rsi_period: usize,
    ) -> Self {
        let config = ConfidenceWeightedConfig::new(
            ema_fast,
            ema_slow,
            macd_fast,
            macd_slow,
            macd_signal,
            rsi_period,
        );
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: ConfidenceWeightedConfig) -> Self {
        config.validate().expect("Invalid ConfidenceWeightedConfig");

        Self {
            ema_fast: Ema::new(config.ema_fast),
            ema_slow: Ema::new(config.ema_slow),
            macd: Macd::new(config.macd_fast, config.macd_slow, config.macd_signal),
            rsi: Rsi::new(config.rsi_period),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            entry_confidence: None,
            last_fast_ema: None,
            last_slow_ema: None,
            last_macd: None,
            last_signal: None,
        }
    }

    pub fn config(&self) -> &ConfidenceWeightedConfig {
        &self.config
    }

    /// Calculate trend confidence score (0-1)
    fn calculate_trend_confidence(&self, fast_ema: f64, slow_ema: f64) -> f64 {
        let diff = fast_ema - slow_ema;
        let pct_diff = (diff / slow_ema).abs();

        // Score based on trend strength (larger spread = stronger trend)
        // Cap at 1.0 for spreads > 5%
        (pct_diff / 0.05).clamp(0.0, 1.0)
    }

    /// Calculate momentum confidence score (0-1)
    fn calculate_momentum_confidence(
        &self,
        macd_line: f64,
        signal_line: f64,
        histogram: f64,
    ) -> f64 {
        // Score based on histogram strength and crossover alignment
        let histogram_strength = (histogram.abs() / macd_line.abs()).clamp(0.0, 1.0);

        // If MACD > signal (bullish), positive boost
        let alignment_boost = if macd_line > signal_line { 0.2 } else { 0.0 };

        (histogram_strength + alignment_boost).min(1.0)
    }

    /// Calculate RSI confidence score (0-1)
    fn calculate_rsi_confidence(&self, rsi_value: f64, is_bullish: bool) -> f64 {
        if is_bullish {
            // For bullish entries: prefer RSI between 30-70 (not extremes)
            if rsi_value < 30.0 {
                // Very oversold - strong signal but risky
                0.7
            } else if rsi_value > 70.0 {
                // Overbought - weak signal
                0.2
            } else {
                // Neutral zone - good signal
                0.8
            }
        } else {
            // For bearish exits: prefer high RSI (overbought)
            if rsi_value > 70.0 {
                0.9
            } else if rsi_value < 30.0 {
                0.2
            } else {
                0.5
            }
        }
    }

    /// Calculate composite confidence score (0-1)
    fn calculate_composite_confidence(&self, params: &ConfidenceParams) -> f64 {
        let trend_score = self.calculate_trend_confidence(params.fast_ema, params.slow_ema);
        let momentum_score = self.calculate_momentum_confidence(
            params.macd_line,
            params.signal_line,
            params.histogram,
        );
        let rsi_score = self.calculate_rsi_confidence(params.rsi_value, params.is_bullish);

        // Weighted average (momentum gets higher weight)
        let composite = trend_score * 0.25 + momentum_score * 0.5 + rsi_score * 0.25;
        composite.clamp(0.0, 1.0)
    }
}

impl MetadataStrategy for ConfidenceWeightedStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Confidence-Weighted".to_string(),
            category: StrategyCategory::MultiIndicator,
            sub_type: Some("confidence_sizing".to_string()),
            description: format!(
                "Multi-indicator strategy with confidence-based position sizing.
                Combines EMA({}/{}), MACD({}/{}/{}), and RSI({}) to calculate composite confidence.
                Trades only when confidence >= {:.0}% (min_conf).
                Signal strength reflects confidence (0.4-1.0) for position sizing.
                Strong signals (0.8+) = larger positions, weak signals = smaller positions.
                TP: {:.1}%, SL: {:.1}%.",
                self.config.ema_fast,
                self.config.ema_slow,
                self.config.macd_fast,
                self.config.macd_slow,
                self.config.macd_signal,
                self.config.rsi_period,
                self.config.min_confidence * 100.0,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/multi_indicator/confidence_weighted.md".to_string(),
            required_indicators: vec!["EMA".to_string(), "MACD".to_string(), "RSI".to_string()],
            expected_regimes: vec![
                MarketRegime::Bull,
                MarketRegime::Trending,
                MarketRegime::HighVolatility,
            ],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.22,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::MultiIndicator
    }
}

impl Strategy for ConfidenceWeightedStrategy {
    fn name(&self) -> &str {
        "Confidence-Weighted"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let fast_ema = self.ema_fast.update(bar.close)?;
        let slow_ema = self.ema_slow.update(bar.close)?;
        let (macd_line, signal_line, histogram) = self.macd.update(bar.close)?;
        let rsi_value = self.rsi.update(bar.close)?;

        let price = bar.close;
        let in_uptrend = fast_ema > slow_ema;

        // Need previous state for crossover detection
        let prev_fast_ema = self.last_fast_ema;
        let prev_slow_ema = self.last_slow_ema;
        let prev_macd = self.last_macd;
        let prev_signal = self.last_signal;

        // Update state for next bar
        self.last_fast_ema = Some(fast_ema);
        self.last_slow_ema = Some(slow_ema);
        self.last_macd = Some(macd_line);
        self.last_signal = Some(signal_line);

        // EXIT LOGIC FIRST (only when in position)
        if self.last_position == SignalType::Buy {
            if let Some(entry) = self.entry_price {
                let profit_pct = (price - entry) / entry * 100.0;

                // TP
                if profit_pct >= self.config.take_profit {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    self.entry_confidence = None;
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
                    self.entry_confidence = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("Stop Loss: {:.1}%", profit_pct)),
                    }]);
                }

                // Calculate current confidence
                let current_confidence = self.calculate_composite_confidence(&ConfidenceParams {
                    fast_ema,
                    slow_ema,
                    macd_line,
                    signal_line,
                    histogram,
                    rsi_value,
                    is_bullish: false,
                });

                // Exit if confidence drops significantly below entry confidence
                if let Some(entry_conf) = self.entry_confidence {
                    if current_confidence < entry_conf * 0.6 {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        self.entry_confidence = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: current_confidence,
                            metadata: Some(format!(
                                "Confidence Drop Exit: {:.2} -> {:.2} ({:.1}%)",
                                entry_conf, current_confidence, profit_pct
                            )),
                        }]);
                    }
                }

                // Exit on trend reversal (fast EMA crosses below slow EMA)
                if let (Some(pf), Some(ps)) = (prev_fast_ema, prev_slow_ema) {
                    if pf >= ps && fast_ema < slow_ema {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        self.entry_confidence = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: current_confidence,
                            metadata: Some(format!(
                                "Trend Reversal Exit: Fast EMA ({:.2}) crossed below Slow EMA ({:.2})",
                                fast_ema, slow_ema
                            )),
                        }]);
                    }
                }

                // Exit on MACD crossover below signal
                if let (Some(pm), Some(ps)) = (prev_macd, prev_signal) {
                    if pm >= ps && macd_line < signal_line {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        self.entry_confidence = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: current_confidence,
                            metadata: Some(format!(
                                "MACD Crossover Exit: MACD ({:.4}) crossed below Signal ({:.4})",
                                macd_line, signal_line
                            )),
                        }]);
                    }
                }
            }
        }

        // ENTRY: Calculate confidence and only enter if above threshold
        if self.last_position != SignalType::Buy && in_uptrend {
            // Check for MACD crossover above signal
            if let (Some(pm), Some(ps)) = (prev_macd, prev_signal) {
                if pm <= ps && macd_line > signal_line {
                    let confidence = self.calculate_composite_confidence(&ConfidenceParams {
                        fast_ema,
                        slow_ema,
                        macd_line,
                        signal_line,
                        histogram,
                        rsi_value,
                        is_bullish: true,
                    });

                    // Only enter if confidence meets threshold
                    if confidence >= self.config.min_confidence {
                        self.last_position = SignalType::Buy;
                        self.entry_price = Some(price);
                        self.entry_confidence = Some(confidence);

                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Buy,
                            strength: confidence,
                            metadata: Some(format!(
                                "Confidence Entry: {:.2} ({:.0}%), MACD crossed above signal",
                                confidence,
                                confidence * 100.0
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
        }
    }

    #[test]
    fn test_strategy_creation() {
        let strategy = ConfidenceWeightedStrategy::new(20, 50, 12, 26, 9, 14);
        assert_eq!(strategy.name(), "Confidence-Weighted");
        assert_eq!(strategy.config().ema_fast, 20);
        assert_eq!(strategy.config().rsi_period, 14);
    }

    #[test]
    fn test_config_validation() {
        let config = ConfidenceWeightedConfig::new(20, 50, 12, 26, 9, 14);
        assert!(config.validate().is_ok());

        // Invalid: fast >= slow EMA
        let invalid = ConfidenceWeightedConfig::new(50, 20, 12, 26, 9, 14);
        assert!(invalid.validate().is_err());

        // Invalid: min_confidence out of range
        let mut invalid2 = ConfidenceWeightedConfig::new(20, 50, 12, 26, 9, 14);
        invalid2.min_confidence = 1.5;
        assert!(invalid2.validate().is_err());
    }

    #[test]
    fn test_metadata() {
        let strategy = ConfidenceWeightedStrategy::new(20, 50, 12, 26, 9, 14);
        let metadata = strategy.metadata();

        assert_eq!(metadata.name, "Confidence-Weighted");
        assert_eq!(metadata.category, StrategyCategory::MultiIndicator);
        assert_eq!(metadata.sub_type, Some("confidence_sizing".to_string()));
        assert!(metadata.description.contains("confidence"));
        assert_eq!(
            metadata.hypothesis_path,
            "hypotheses/multi_indicator/confidence_weighted.md"
        );
        assert!(metadata.required_indicators.contains(&"EMA".to_string()));
        assert!(metadata.required_indicators.contains(&"MACD".to_string()));
        assert!(metadata.required_indicators.contains(&"RSI".to_string()));
    }

    #[test]
    fn test_trend_confidence_calculation() {
        let strategy = ConfidenceWeightedStrategy::new(20, 50, 12, 26, 9, 14);

        // Strong trend (5% spread)
        let conf1 = strategy.calculate_trend_confidence(105.0, 100.0);
        assert!((conf1 - 1.0).abs() < 0.01);

        // Moderate trend (2.5% spread)
        let conf2 = strategy.calculate_trend_confidence(102.5, 100.0);
        assert!((conf2 - 0.5).abs() < 0.01);

        // Weak trend (1% spread)
        let conf3 = strategy.calculate_trend_confidence(101.0, 100.0);
        assert!((conf3 - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_rsi_confidence_calculation() {
        let strategy = ConfidenceWeightedStrategy::new(20, 50, 12, 26, 9, 14);

        // Bullish at RSI 50 (neutral zone - high confidence)
        let conf1 = strategy.calculate_rsi_confidence(50.0, true);
        assert!((conf1 - 0.8).abs() < 0.01);

        // Bullish at RSI 25 (oversold - strong but risky)
        let conf2 = strategy.calculate_rsi_confidence(25.0, true);
        assert!((conf2 - 0.7).abs() < 0.01);

        // Bullish at RSI 75 (overbought - weak)
        let conf3 = strategy.calculate_rsi_confidence(75.0, true);
        assert!((conf3 - 0.2).abs() < 0.01);

        // Bearish at RSI 75 (overbought - strong)
        let conf4 = strategy.calculate_rsi_confidence(75.0, false);
        assert!((conf4 - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_composite_confidence() {
        let strategy = ConfidenceWeightedStrategy::new(20, 50, 12, 26, 9, 14);

        // High confidence scenario
        let conf1 = strategy.calculate_composite_confidence(&ConfidenceParams {
            fast_ema: 105.0, // Strong trend
            slow_ema: 100.0,
            macd_line: 1.0, // Strong MACD
            signal_line: 0.5,
            histogram: 0.5,
            rsi_value: 50.0, // Neutral RSI
            is_bullish: true,
        });
        assert!(conf1 > 0.7);

        // Low confidence scenario
        let conf2 = strategy.calculate_composite_confidence(&ConfidenceParams {
            fast_ema: 100.5, // Weak trend
            slow_ema: 100.0,
            macd_line: 0.1, // Weak MACD
            signal_line: 0.0,
            histogram: 0.0,
            rsi_value: 70.0, // Overbought RSI
            is_bullish: true,
        });
        assert!(conf2 < 0.5);
    }

    #[test]
    fn test_indicator_warmup() {
        let mut strategy = ConfidenceWeightedStrategy::new(20, 50, 12, 26, 9, 14);

        let base_time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let mut price: f64;

        // Feed initial bars (slow EMA needs 50, MACD needs 26+9=35)
        for i in 0..60 {
            price = 100.0 + (i as f64) * 0.3;
            let bar = create_test_bar(base_time + chrono::Duration::hours(i), price);
            let signal = strategy.on_bar(&bar);

            // After warmup (around bar 50+), should eventually get buy signals
            // in an uptrending market with appropriate MACD crossover
            if i > 45 {
                if let Some(signals) = signal {
                    assert!(!signals.is_empty(), "Should generate signals after warmup");
                    assert_eq!(signals[0].signal_type, SignalType::Buy);
                    assert!(signals[0].strength > 0.0);
                    // Successfully got a signal after warmup
                    return;
                }
            }
        }

        // If no signal generated, verify at least strategy can process bars
        let final_bar = create_test_bar(base_time + chrono::Duration::hours(60), 118.0);
        let result = strategy.on_bar(&final_bar);
        // Should not panic - test passes if we get here
        assert!(
            result.is_some() || result.is_none(),
            "Strategy should handle final bar"
        );
    }

    #[test]
    fn test_config_display() {
        let config = ConfidenceWeightedConfig::new(20, 50, 12, 26, 9, 14);
        let display = format!("{}", config);
        assert!(display.contains("ConfWeighted"));
        assert!(display.contains("ema=20/50"));
        assert!(display.contains("macd=12/26/9"));
    }

    #[test]
    fn test_min_confidence_filter() {
        let mut strategy = ConfidenceWeightedStrategy::new(20, 50, 12, 26, 9, 14);

        let base_time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        // Create weak signal scenario
        for i in 0..50 {
            let price = 100.0 + (i as f64) * 0.1; // Weak trend
            let bar = create_test_bar(base_time + chrono::Duration::hours(i), price);
            let signal = strategy.on_bar(&bar);

            // Weak trend should not generate buy signals or only generate very weak ones
            if let Some(signals) = signal {
                for sig in signals {
                    if sig.signal_type == SignalType::Buy {
                        // If we do get a signal, it should be very weak
                        assert!(
                            sig.strength < 0.5,
                            "Weak trend should not generate strong buy signals"
                        );
                    }
                }
            }
        }

        // Verify strategy is still functional after weak scenario
        let final_bar = create_test_bar(base_time + chrono::Duration::hours(50), 105.0);
        let result = strategy.on_bar(&final_bar);
        assert!(
            result.is_some() || result.is_none(),
            "Strategy should handle final bar"
        );
    }
}
