//! MACD + RSI Combo Strategy
//!
//! This strategy combines MACD momentum signals with RSI overbought/oversold
//! filters to generate higher-confidence entry signals. Both indicators must agree
//! for entry, reducing false signals while maintaining momentum capture.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Indicator, Macd, Rsi};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for MACD + RSI Combo strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACDRSIConfig {
    /// MACD fast period
    pub macd_fast: usize,
    /// MACD slow period
    pub macd_slow: usize,
    /// MACD signal period
    pub macd_signal: usize,
    /// RSI period
    pub rsi_period: usize,
    /// RSI overbought threshold (default 70)
    pub rsi_overbought: f64,
    /// RSI oversold threshold (default 30)
    pub rsi_oversold: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl MACDRSIConfig {
    pub fn new(macd_fast: usize, macd_slow: usize, macd_signal: usize, rsi_period: usize) -> Self {
        Self {
            macd_fast,
            macd_slow,
            macd_signal,
            rsi_period,
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,
            take_profit: 5.0,
            stop_loss: 5.0,
        }
    }

    pub fn with_thresholds(
        macd_fast: usize,
        macd_slow: usize,
        macd_signal: usize,
        rsi_period: usize,
        overbought: f64,
        oversold: f64,
    ) -> Self {
        let mut config = Self::new(macd_fast, macd_slow, macd_signal, rsi_period);
        config.rsi_overbought = overbought;
        config.rsi_oversold = oversold;
        config
    }

    pub fn with_exits(
        macd_fast: usize,
        macd_slow: usize,
        macd_signal: usize,
        rsi_period: usize,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            macd_fast,
            macd_slow,
            macd_signal,
            rsi_period,
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,
            take_profit,
            stop_loss,
        }
    }

    pub fn default_config() -> Self {
        Self {
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            rsi_period: 14,
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,
            take_profit: 5.0,
            stop_loss: 5.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.macd_fast >= self.macd_slow {
            return Err("MACD fast period must be less than slow period".to_string());
        }
        if self.macd_signal == 0 {
            return Err("MACD signal period must be greater than 0".to_string());
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

impl fmt::Display for MACDRSIConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MACD+RSI(macd={}/{}/{}, rsi={}, ob={:.0}, os={:.0}, tp={:.1}%, sl={:.1}%)",
            self.macd_fast,
            self.macd_slow,
            self.macd_signal,
            self.rsi_period,
            self.rsi_overbought,
            self.rsi_oversold,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// MACD + RSI Combo Strategy
///
/// # Strategy Logic
///
/// **Buy Signal (Entry)**:
/// - MACD crosses above signal line (bullish momentum)
/// - AND RSI is not overbought (< overbought threshold)
/// - AND RSI is showing strength (> 50 or trending up)
///
/// **Sell Signal (Exit)**:
/// - Take Profit hit
/// - Stop Loss hit
/// - RSI becomes overbought (> overbought threshold) while in position
/// - MACD crosses below signal line (momentum reversal)
///
/// # Why This Combination Works
/// - **MACD**: Captures momentum changes and trend direction
/// - **RSI**: Filters out entries at extremes (overbought conditions)
/// - **Combo**: Reduces false signals by requiring both indicators to agree
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::multi_indicator::MACDRSIComboStrategy;
///
/// let strategy = MACDRSIComboStrategy::new(12, 26, 9, 14);
/// ```
pub struct MACDRSIComboStrategy {
    config: MACDRSIConfig,
    macd: Macd,
    rsi: Rsi,
    last_position: SignalType,
    entry_price: Option<f64>,
    // State tracking for crossover detection
    last_macd: Option<f64>,
    last_signal: Option<f64>,
}

impl MACDRSIComboStrategy {
    /// Creates a new MACD + RSI Combo strategy with default thresholds
    pub fn new(macd_fast: usize, macd_slow: usize, macd_signal: usize, rsi_period: usize) -> Self {
        let config = MACDRSIConfig::new(macd_fast, macd_slow, macd_signal, rsi_period);
        Self::from_config(config)
    }

    /// Creates a strategy with custom RSI thresholds
    pub fn with_thresholds(
        macd_fast: usize,
        macd_slow: usize,
        macd_signal: usize,
        rsi_period: usize,
        overbought: f64,
        oversold: f64,
    ) -> Self {
        let config = MACDRSIConfig::with_thresholds(
            macd_fast,
            macd_slow,
            macd_signal,
            rsi_period,
            overbought,
            oversold,
        );
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: MACDRSIConfig) -> Self {
        config.validate().expect("Invalid MACDRSIConfig");

        Self {
            macd: Macd::new(config.macd_fast, config.macd_slow, config.macd_signal),
            rsi: Rsi::new(config.rsi_period),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            last_macd: None,
            last_signal: None,
        }
    }

    pub fn config(&self) -> &MACDRSIConfig {
        &self.config
    }

    /// Calculate signal confidence based on indicator alignment
    fn calculate_confidence(&self, macd_strength: f64, rsi_value: f64, is_entry: bool) -> f64 {
        if is_entry {
            // Entry confidence: stronger when RSI is not near overbought
            let rsi_confidence = 1.0 - (rsi_value - 50.0).abs() / 50.0; // 1.0 at 50, lower at extremes
            let macd_confidence = macd_strength.min(1.0);
            (rsi_confidence + macd_confidence) / 2.0
        } else {
            // Exit confidence: stronger when RSI is overbought or MACD is bearish
            let rsi_confidence = if rsi_value > self.config.rsi_overbought {
                (rsi_value - self.config.rsi_overbought) / (100.0 - self.config.rsi_overbought)
            } else {
                0.5
            };
            rsi_confidence.min(1.0)
        }
    }
}

impl MetadataStrategy for MACDRSIComboStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "MACD+RSI Combo".to_string(),
            category: StrategyCategory::MultiIndicator,
            sub_type: Some("indicator_confluence".to_string()),
            description: format!(
                "Multi-indicator strategy combining MACD({}/{}/{}) momentum with RSI({}) filter.
                Buy when MACD crosses above signal AND RSI < {:.0} (not overbought).
                Sell on TP ({:.1}%), SL ({:.1}%), or RSI overbought (> {:.0}).
                Reduces false signals by requiring both indicators to agree.",
                self.config.macd_fast,
                self.config.macd_slow,
                self.config.macd_signal,
                self.config.rsi_period,
                self.config.rsi_overbought,
                self.config.take_profit,
                self.config.stop_loss,
                self.config.rsi_overbought
            ),
            hypothesis_path: "hypotheses/multi_indicator/macd_rsi_combo.md".to_string(),
            required_indicators: vec!["MACD".to_string(), "RSI".to_string(), "Price".to_string()],
            expected_regimes: vec![
                MarketRegime::Bull,
                MarketRegime::Trending,
                MarketRegime::HighVolatility,
            ],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.25,
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

impl Strategy for MACDRSIComboStrategy {
    fn name(&self) -> &str {
        "MACD+RSI Combo"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let (macd_line, signal_line, _histogram) = self.macd.update(bar.close)?;
        let rsi_value = self.rsi.update(bar.close)?;

        let price = bar.close;
        let is_rsi_overbought = rsi_value > self.config.rsi_overbought;
        let is_rsi_bullish = rsi_value > 50.0 && rsi_value < self.config.rsi_overbought;

        // Need previous state for crossover detection
        let prev_macd = self.last_macd;
        let prev_signal = self.last_signal;

        // Update state for next bar
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

                // Exit on RSI overbought (momentum exhaustion)
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

                // Exit on MACD crossover below signal (momentum reversal)
                if let (Some(pm), Some(ps)) = (prev_macd, prev_signal) {
                    if pm >= ps && macd_line < signal_line {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 0.8,
                            metadata: Some(format!(
                                "MACD Crossover Exit: MACD {:.4} crossed below Signal {:.4}",
                                macd_line, signal_line
                            )),
                        }]);
                    }
                }
            }
        }

        // ENTRY: Requires MACD crossover above signal AND RSI conditions
        if self.last_position != SignalType::Buy {
            // RSI must be bullish (between 50 and overbought) - not oversold or extremely overbought
            if is_rsi_bullish {
                if let (Some(pm), Some(ps)) = (prev_macd, prev_signal) {
                    // MACD must cross above signal (not just be above)
                    if pm <= ps && macd_line > signal_line {
                        self.last_position = SignalType::Buy;
                        self.entry_price = Some(price);

                        let macd_strength = (macd_line - signal_line).abs();
                        let confidence = self.calculate_confidence(macd_strength, rsi_value, true);

                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Buy,
                            strength: confidence,
                            metadata: Some(format!(
                                "Bullish Entry: MACD crossed above signal, RSI {:.2} (not overbought)",
                                rsi_value
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
            symbol: "BTCUSDT".to_string(),
        }
    }

    #[test]
    fn test_strategy_creation() {
        let strategy = MACDRSIComboStrategy::new(12, 26, 9, 14);
        assert_eq!(strategy.name(), "MACD+RSI Combo");
        assert_eq!(strategy.config().macd_fast, 12);
        assert_eq!(strategy.config().rsi_period, 14);
    }

    #[test]
    fn test_config_validation() {
        let config = MACDRSIConfig::new(12, 26, 9, 14);
        assert!(config.validate().is_ok());

        // Invalid: fast >= slow
        let invalid = MACDRSIConfig::new(26, 12, 9, 14);
        assert!(invalid.validate().is_err());

        // Invalid: overbought <= oversold
        let invalid2 = MACDRSIConfig::with_thresholds(12, 26, 9, 14, 30.0, 70.0);
        assert!(invalid2.validate().is_err());

        // Invalid: thresholds out of range
        let invalid3 = MACDRSIConfig::with_thresholds(12, 26, 9, 14, 110.0, 30.0);
        assert!(invalid3.validate().is_err());
    }

    #[test]
    fn test_custom_thresholds() {
        let strategy = MACDRSIComboStrategy::with_thresholds(12, 26, 9, 14, 75.0, 25.0);
        assert_eq!(strategy.config().rsi_overbought, 75.0);
        assert_eq!(strategy.config().rsi_oversold, 25.0);
    }

    #[test]
    fn test_metadata() {
        let strategy = MACDRSIComboStrategy::new(12, 26, 9, 14);
        let metadata = strategy.metadata();

        assert_eq!(metadata.name, "MACD+RSI Combo");
        assert_eq!(metadata.category, StrategyCategory::MultiIndicator);
        assert_eq!(metadata.sub_type, Some("indicator_confluence".to_string()));
        assert!(metadata.description.contains("MACD"));
        assert!(metadata.description.contains("RSI"));
        assert_eq!(
            metadata.hypothesis_path,
            "hypotheses/multi_indicator/macd_rsi_combo.md"
        );
        assert!(metadata.required_indicators.contains(&"MACD".to_string()));
        assert!(metadata.required_indicators.contains(&"RSI".to_string()));
        assert!(metadata.expected_regimes.contains(&MarketRegime::Bull));
        assert_eq!(metadata.risk_profile.max_drawdown_expected, 0.25);
    }

    #[test]
    fn test_category() {
        let strategy = MACDRSIComboStrategy::new(12, 26, 9, 14);
        assert_eq!(strategy.category(), StrategyCategory::MultiIndicator);
    }

    #[test]
    fn test_confidence_calculation() {
        let strategy = MACDRSIComboStrategy::new(12, 26, 9, 14);

        // Entry confidence at RSI 50 (neutral)
        let conf1 = strategy.calculate_confidence(0.5, 50.0, true);
        assert!((conf1 - 0.5).abs() < 0.01);

        // Entry confidence at RSI 60 (bullish but not extreme)
        let conf2 = strategy.calculate_confidence(0.5, 60.0, true);
        assert!(conf2 > 0.3 && conf2 < 0.7);

        // Entry confidence at RSI 70 (near overbought, lower confidence)
        let conf3 = strategy.calculate_confidence(0.5, 70.0, true);
        assert!(conf3 < conf2);
    }

    #[test]
    fn test_indicator_warmup() {
        let mut strategy = MACDRSIComboStrategy::new(12, 26, 9, 14);

        // Need enough bars for indicators to warm up
        let base_time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let mut price = 100.0;

        // Feed initial bars (MACD needs 26+9=35, RSI needs 14+1=15)
        for i in 0..50 {
            price = 100.0 + (i as f64) * 0.5; // Trending up
            let bar = create_test_bar(base_time + chrono::Duration::hours(i), price);
            strategy.on_bar(&bar);
        }

        // After warmup, indicators should be ready
        // Strategy should have processed bars without panicking
        assert!(true);
    }

    #[test]
    fn test_config_display() {
        let config = MACDRSIConfig::new(12, 26, 9, 14);
        let display = format!("{}", config);
        assert!(display.contains("MACD+RSI"));
        assert!(display.contains("12/26/9"));
        assert!(display.contains("rsi=14"));
    }
}
