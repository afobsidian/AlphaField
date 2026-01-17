//! Momentum Strategy using EMA and MACD
//!
//! This strategy combines multiple indicators to identify strong momentum trends.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Ema, Indicator, Macd};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for Momentum strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumConfig {
    /// EMA period for trend confirmation
    pub ema_period: usize,
    /// MACD fast period
    pub macd_fast: usize,
    /// MACD slow period
    pub macd_slow: usize,
    /// MACD signal period
    pub macd_signal: usize,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl MomentumConfig {
    pub fn new(ema_period: usize, macd_fast: usize, macd_slow: usize, macd_signal: usize) -> Self {
        Self {
            ema_period,
            macd_fast,
            macd_slow,
            macd_signal,
            take_profit: 5.0,
            stop_loss: 5.0,
        }
    }

    pub fn new_with_exits(
        ema_period: usize,
        macd_fast: usize,
        macd_slow: usize,
        macd_signal: usize,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            ema_period,
            macd_fast,
            macd_slow,
            macd_signal,
            take_profit,
            stop_loss,
        }
    }

    pub fn default_config() -> Self {
        Self {
            ema_period: 50,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            take_profit: 5.0,
            stop_loss: 5.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.ema_period == 0 {
            return Err("EMA period must be greater than 0".to_string());
        }
        if self.macd_fast >= self.macd_slow {
            return Err("MACD fast period must be less than slow period".to_string());
        }
        if self.macd_signal == 0 {
            return Err("MACD signal period must be greater than 0".to_string());
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

impl fmt::Display for MomentumConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Momentum(ema={}, macd={}/{}/{}, tp={:.1}%, sl={:.1}%)",
            self.ema_period,
            self.macd_fast,
            self.macd_slow,
            self.macd_signal,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// Momentum Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: Price > EMA AND MACD crosses above signal line
/// - **Sell Signal**: Price < EMA AND MACD crosses below signal line
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::MACDStrategy;
///
/// let strategy = MACDStrategy::new(50, 12, 26, 9);
/// ```
pub struct MomentumStrategy {
    config: MomentumConfig,
    ema: Ema,
    macd: Macd,
    last_position: SignalType,
    entry_price: Option<f64>,
    // State tracking for crossover detection
    last_macd: Option<f64>,
    last_signal: Option<f64>,
    last_price_above_ema: Option<bool>,
}

impl MomentumStrategy {
    /// Creates a new Momentum strategy
    pub fn new(ema_period: usize, macd_fast: usize, macd_slow: usize, macd_signal: usize) -> Self {
        let config =
            MomentumConfig::new_with_exits(ema_period, macd_fast, macd_slow, macd_signal, 5.0, 5.0);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: MomentumConfig) -> Self {
        config.validate().expect("Invalid MomentumConfig");

        Self {
            ema: Ema::new(config.ema_period),
            macd: Macd::new(config.macd_fast, config.macd_slow, config.macd_signal),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            last_macd: None,
            last_signal: None,
            last_price_above_ema: None,
        }
    }

    pub fn config(&self) -> &MomentumConfig {
        &self.config
    }
}

impl Default for MomentumStrategy {
    fn default() -> Self {
        // Default: 50 EMA, 12/26/9 MACD, 5% TP, 5% SL
        Self::from_config(MomentumConfig::default_config())
    }
}

impl MetadataStrategy for MomentumStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "EMA-MACD Momentum".to_string(),
            category: StrategyCategory::Momentum,
            sub_type: Some("indicator_combination".to_string()),
            description: format!(
                "Momentum strategy combining EMA({}) with MACD({}/{}/{}) for trend confirmation.
                Uses {:.1}% TP and {:.1}% SL. Generates buy signals when price > EMA and MACD crosses above signal line.",
                self.config.ema_period, self.config.macd_fast, self.config.macd_slow, self.config.macd_signal,
                self.config.take_profit, self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/momentum/macd_strategy.md".to_string(),
            required_indicators: vec!["EMA".to_string(), "MACD".to_string(), "Price".to_string()],
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Trending, MarketRegime::HighVolatility],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.30,
                volatility_level: VolatilityLevel::High,
                correlation_sensitivity: CorrelationSensitivity::Medium,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::Momentum
    }
}

impl Strategy for MomentumStrategy {
    fn name(&self) -> &str {
        "EMA-MACD Momentum"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let ema_val = self.ema.update(bar.close)?;
        let (macd_line, signal_line, _histogram) = self.macd.update(bar.close)?;

        let price = bar.close;
        let price_above_ema = price > ema_val;

        // Need previous state for crossover detection
        let prev_macd = self.last_macd;
        let prev_signal = self.last_signal;
        let prev_price_above_ema = self.last_price_above_ema;

        // Update state for next bar
        self.last_macd = Some(macd_line);
        self.last_signal = Some(signal_line);
        self.last_price_above_ema = Some(price_above_ema);

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

                // Exit on MACD crossover below signal (actual crossover, not just condition)
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

                // Exit on trend break (price crosses below EMA - actual crossover)
                if let Some(was_above) = prev_price_above_ema {
                    if was_above && !price_above_ema {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 0.9,
                            metadata: Some(format!(
                                "Trend Break Exit: Price {:.2} crossed below EMA {:.2}",
                                price, ema_val
                            )),
                        }]);
                    }
                }
            }
        }

        // Entry: Requires MACD crossover above signal AND price above EMA
        if self.last_position != SignalType::Buy && price_above_ema {
            if let (Some(pm), Some(ps)) = (prev_macd, prev_signal) {
                // MACD must cross above signal (not just be above)
                if pm <= ps && macd_line > signal_line {
                    self.last_position = SignalType::Buy;
                    self.entry_price = Some(price);
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: ((macd_line - signal_line) / macd_line.abs()).abs().min(1.0),
                        metadata: Some(format!(
                            "Bullish Entry: Price {:.2} > EMA {:.2}, MACD crossed above Signal",
                            price, ema_val
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

    #[test]
    fn test_momentum_creation() {
        let strategy = MomentumStrategy::new(50, 12, 26, 9);
        assert_eq!(strategy.name(), "EMA-MACD Momentum");
    }

    #[test]
    fn test_config_validation() {
        let config = MomentumConfig::new(50, 12, 26, 9);
        assert!(config.validate().is_ok());

        let invalid_config = MomentumConfig::new_with_exits(50, 26, 12, 9, 5.0, 5.0); // fast > slow
        assert!(invalid_config.validate().is_err());
    }
}
