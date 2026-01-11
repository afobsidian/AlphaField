//! Momentum Factor Strategy
//!
//! Combines multiple momentum factors for stronger signals.
//! Uses price momentum, volume momentum, and RSI momentum together.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Indicator, Rsi};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Configuration for Momentum Factor strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumFactorConfig {
    /// Lookback period for calculations
    pub lookback_period: usize,
    /// RSI period
    pub rsi_period: usize,
    /// Minimum factors that must be positive to enter (out of 3)
    pub min_factors: usize,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl MomentumFactorConfig {
    pub fn new(
        lookback_period: usize,
        rsi_period: usize,
        min_factors: usize,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            lookback_period,
            rsi_period,
            min_factors,
            take_profit,
            stop_loss,
        }
    }

    pub fn default_config() -> Self {
        Self {
            lookback_period: 20,
            rsi_period: 14,
            min_factors: 2,
            take_profit: 5.0,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.lookback_period == 0 {
            return Err("Lookback period must be greater than 0".to_string());
        }
        if self.rsi_period == 0 {
            return Err("RSI period must be greater than 0".to_string());
        }
        if self.min_factors == 0 || self.min_factors > 3 {
            return Err("Min factors must be between 1 and 3".to_string());
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

impl fmt::Display for MomentumFactorConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Momentum_Factor(lookback={}, rsi={}, min_factors={}, tp={:.1}%, sl={:.1}%)",
            self.lookback_period,
            self.rsi_period,
            self.min_factors,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// Momentum Factor Strategy
///
/// # Strategy Logic
/// Combines three momentum factors:
/// 1. Price Momentum: Price > price N periods ago
/// 2. Volume Momentum: Volume > average volume
/// 3. RSI Momentum: RSI > 50 (bullish)
///
/// - **Buy Signal**: At least min_factors out of 3 are positive
/// - **Sell Signal**: Less than min_factors are positive OR TP/SL
pub struct MomentumFactorStrategy {
    config: MomentumFactorConfig,
    rsi: Rsi,
    prices: VecDeque<f64>,
    volumes: VecDeque<f64>,
    last_position: SignalType,
    entry_price: Option<f64>,
}

impl MomentumFactorStrategy {
    /// Creates a new Momentum Factor strategy
    pub fn new(
        lookback_period: usize,
        rsi_period: usize,
        min_factors: usize,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        let config = MomentumFactorConfig::new(
            lookback_period,
            rsi_period,
            min_factors,
            take_profit,
            stop_loss,
        );
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: MomentumFactorConfig) -> Self {
        config.validate().expect("Invalid MomentumFactorConfig");

        Self {
            rsi: Rsi::new(config.rsi_period),
            prices: VecDeque::with_capacity(config.lookback_period + 1),
            volumes: VecDeque::with_capacity(config.lookback_period + 1),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
        }
    }

    pub fn config(&self) -> &MomentumFactorConfig {
        &self.config
    }

    /// Calculate momentum factors
    fn calculate_factors(&self, price: f64, volume: f64, rsi: f64) -> (bool, bool, bool) {
        let mut price_momentum = false;
        let mut volume_momentum = false;
        let rsi_momentum = rsi > 50.0;

        // Factor 1: Price Momentum
        if self.prices.len() >= self.config.lookback_period {
            let old_price = self.prices[0];
            price_momentum = price > old_price;
        }

        // Factor 2: Volume Momentum
        if self.volumes.len() >= self.config.lookback_period {
            let avg_volume: f64 = self.volumes.iter().sum::<f64>() / self.volumes.len() as f64;
            volume_momentum = volume > avg_volume;
        }

        (price_momentum, volume_momentum, rsi_momentum)
    }

    /// Count positive factors
    fn count_positive_factors(&self, factors: (bool, bool, bool)) -> usize {
        let mut count = 0;
        if factors.0 {
            count += 1;
        }
        if factors.1 {
            count += 1;
        }
        if factors.2 {
            count += 1;
        }
        count
    }
}

impl MetadataStrategy for MomentumFactorStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Momentum Factor".to_string(),
            category: StrategyCategory::Momentum,
            sub_type: Some("multi_factor".to_string()),
            description: format!(
                "Multi-factor momentum strategy combining price momentum ({}p), volume momentum, and RSI({}). \
                Requires at least {}/{} factors to be positive for entry. Uses {:.1}% TP and {:.1}% SL.",
                self.config.lookback_period,
                self.config.rsi_period,
                self.config.min_factors,
                3,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/momentum/momentum_factor.md".to_string(),
            required_indicators: vec![
                "Price_Momentum".to_string(),
                "Volume_Momentum".to_string(),
                "RSI".to_string(),
            ],
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Trending],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.20,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::Momentum
    }
}

impl Strategy for MomentumFactorStrategy {
    fn name(&self) -> &str {
        "Momentum Factor"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;
        let volume = bar.volume;

        // Update price and volume history
        self.prices.push_back(price);
        if self.prices.len() > self.config.lookback_period {
            self.prices.pop_front();
        }

        self.volumes.push_back(volume);
        if self.volumes.len() > self.config.lookback_period {
            self.volumes.pop_front();
        }

        // Update RSI
        let rsi_val = self.rsi.update(price)?;

        // Calculate factors
        let factors = self.calculate_factors(price, volume, rsi_val);
        let positive_count = self.count_positive_factors(factors);

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

                // Exit on factor degradation
                if positive_count < self.config.min_factors {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 0.7,
                        metadata: Some(format!(
                            "Factor Degradation Exit: {}/{} factors positive",
                            positive_count, 3
                        )),
                    }]);
                }
            }
        }

        // ENTRY LOGIC - Enough factors are positive
        if self.last_position != SignalType::Buy && positive_count >= self.config.min_factors {
            // Calculate signal strength based on number of positive factors
            let strength = positive_count as f64 / 3.0;

            self.last_position = SignalType::Buy;
            self.entry_price = Some(price);
            return Some(vec![Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Buy,
                strength,
                metadata: Some(format!(
                    "Multi-Factor Entry: {}/{} factors positive (Price:{}, Vol:{}, RSI:{})",
                    positive_count,
                    3,
                    if factors.0 { "✓" } else { "✗" },
                    if factors.1 { "✓" } else { "✗" },
                    if factors.2 { "✓" } else { "✗" }
                )),
            }]);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_momentum_factor_creation() {
        let strategy = MomentumFactorStrategy::new(20, 14, 2, 5.0, 3.0);
        assert_eq!(strategy.name(), "Momentum Factor");
    }

    #[test]
    fn test_config_validation() {
        let config = MomentumFactorConfig::default_config();
        assert!(config.validate().is_ok());

        let invalid_config = MomentumFactorConfig::new(20, 14, 4, 5.0, 3.0); // min_factors > 3
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_metadata() {
        let strategy = MomentumFactorStrategy::new(20, 14, 2, 5.0, 3.0);
        let metadata = strategy.metadata();
        assert_eq!(metadata.category, StrategyCategory::Momentum);
        assert_eq!(metadata.name, "Momentum Factor");
    }

    #[test]
    fn test_count_positive_factors() {
        let strategy = MomentumFactorStrategy::new(20, 14, 2, 5.0, 3.0);

        assert_eq!(strategy.count_positive_factors((true, true, true)), 3);
        assert_eq!(strategy.count_positive_factors((true, true, false)), 2);
        assert_eq!(strategy.count_positive_factors((true, false, false)), 1);
        assert_eq!(strategy.count_positive_factors((false, false, false)), 0);
    }
}
