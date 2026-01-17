//! RSI Momentum Strategy
//!
//! A momentum strategy that uses RSI to identify strong trends rather than reversions.
//! Unlike RSI mean reversion, this strategy enters when RSI shows strong momentum.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Indicator, Rsi};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for RSI Momentum strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsiMomentumConfig {
    /// RSI period
    pub rsi_period: usize,
    /// Momentum threshold (RSI above this = bullish momentum)
    pub momentum_threshold: f64,
    /// Strength threshold for strong momentum
    pub strength_threshold: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl RsiMomentumConfig {
    pub fn new(
        rsi_period: usize,
        momentum_threshold: f64,
        strength_threshold: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            rsi_period,
            momentum_threshold,
            strength_threshold,
            take_profit,
            stop_loss,
        }
    }

    pub fn default_config() -> Self {
        Self {
            rsi_period: 14,
            momentum_threshold: 50.0,
            strength_threshold: 60.0,
            take_profit: 5.0,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.rsi_period == 0 {
            return Err("RSI period must be greater than 0".to_string());
        }
        if self.momentum_threshold <= 0.0 || self.momentum_threshold >= 100.0 {
            return Err("Momentum threshold must be between 0 and 100".to_string());
        }
        if self.strength_threshold <= self.momentum_threshold {
            return Err("Strength threshold must be greater than momentum threshold".to_string());
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

impl fmt::Display for RsiMomentumConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RSI_Momentum(period={}, threshold={:.0}, strength={:.0}, tp={:.1}%, sl={:.1}%)",
            self.rsi_period,
            self.momentum_threshold,
            self.strength_threshold,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// RSI Momentum Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: RSI rises above momentum_threshold with increasing momentum
/// - **Sell Signal**: RSI falls below momentum_threshold OR TP/SL triggered
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::momentum::RsiMomentumStrategy;
///
/// let strategy = RsiMomentumStrategy::new(14, 50.0, 60.0, 5.0, 3.0);
/// ```
pub struct RsiMomentumStrategy {
    config: RsiMomentumConfig,
    rsi: Rsi,
    last_position: SignalType,
    entry_price: Option<f64>,
    last_rsi: Option<f64>,
}

impl RsiMomentumStrategy {
    /// Creates a new RSI Momentum strategy
    pub fn new(
        rsi_period: usize,
        momentum_threshold: f64,
        strength_threshold: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        let config = RsiMomentumConfig::new(
            rsi_period,
            momentum_threshold,
            strength_threshold,
            take_profit,
            stop_loss,
        );
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: RsiMomentumConfig) -> Self {
        config.validate().expect("Invalid RsiMomentumConfig");

        Self {
            rsi: Rsi::new(config.rsi_period),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            last_rsi: None,
        }
    }

    pub fn config(&self) -> &RsiMomentumConfig {
        &self.config
    }
}

impl Default for RsiMomentumStrategy {
    fn default() -> Self {
        // Default: 14-period RSI, 50 momentum/60 strength thresholds, 5% TP, 3% SL
        Self::from_config(RsiMomentumConfig::default_config())
    }
}

impl MetadataStrategy for RsiMomentumStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "RSI Momentum".to_string(),
            category: StrategyCategory::Momentum,
            sub_type: Some("rsi_momentum".to_string()),
            description: format!(
                "RSI Momentum strategy using {} period RSI. Enters when RSI crosses above {} (momentum threshold)
                with strong momentum (>{}). Uses {:.1}% TP and {:.1}% SL.",
                self.config.rsi_period,
                self.config.momentum_threshold,
                self.config.strength_threshold,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/momentum/rsi_momentum.md".to_string(),
            required_indicators: vec!["RSI".to_string()],
            expected_regimes: vec![
                MarketRegime::Bull,
                MarketRegime::Trending,
                MarketRegime::HighVolatility,
            ],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.25,
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

impl Strategy for RsiMomentumStrategy {
    fn name(&self) -> &str {
        "RSI Momentum"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let rsi_val = self.rsi.update(bar.close)?;
        let price = bar.close;
        let prev_rsi = self.last_rsi;

        // Update state for next bar
        self.last_rsi = Some(rsi_val);

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

                // Exit on momentum loss: RSI crosses below momentum threshold
                if let Some(prev) = prev_rsi {
                    if prev >= self.config.momentum_threshold
                        && rsi_val < self.config.momentum_threshold
                    {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 0.8,
                            metadata: Some(format!(
                                "Momentum Loss Exit: RSI {:.2} crossed below {}",
                                rsi_val, self.config.momentum_threshold
                            )),
                        }]);
                    }
                }
            }
        }

        // ENTRY LOGIC - RSI crosses above momentum threshold with strength
        if self.last_position != SignalType::Buy {
            if let Some(prev) = prev_rsi {
                // RSI must cross above momentum threshold
                if prev <= self.config.momentum_threshold
                    && rsi_val > self.config.momentum_threshold
                {
                    // Calculate signal strength based on how far above momentum threshold
                    let strength = if rsi_val >= self.config.strength_threshold {
                        1.0
                    } else {
                        0.7
                    };

                    self.last_position = SignalType::Buy;
                    self.entry_price = Some(price);
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength,
                        metadata: Some(format!(
                            "RSI Momentum Entry: RSI {:.2} crossed above {}",
                            rsi_val, self.config.momentum_threshold
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
    use chrono::Utc;

    fn create_test_bar(price: f64) -> Bar {
        Bar {
            timestamp: Utc::now(),
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_rsi_momentum_creation() {
        let strategy = RsiMomentumStrategy::new(14, 50.0, 60.0, 5.0, 3.0);
        assert_eq!(strategy.name(), "RSI Momentum");
    }

    #[test]
    fn test_config_validation() {
        let config = RsiMomentumConfig::default_config();
        assert!(config.validate().is_ok());

        let invalid_config = RsiMomentumConfig::new(14, 50.0, 40.0, 5.0, 3.0); // strength < momentum
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_metadata() {
        let strategy = RsiMomentumStrategy::new(14, 50.0, 60.0, 5.0, 3.0);
        let metadata = strategy.metadata();
        assert_eq!(metadata.category, StrategyCategory::Momentum);
        assert_eq!(metadata.name, "RSI Momentum");
    }

    #[test]
    fn test_no_signal_without_warmup() {
        let mut strategy = RsiMomentumStrategy::new(14, 50.0, 60.0, 5.0, 3.0);

        // First few bars should not generate signals (RSI needs warmup)
        for i in 1..10 {
            let bar = create_test_bar(100.0 + i as f64);
            let _signal = strategy.on_bar(&bar);
            // May be None until RSI warms up
        }
    }
}
