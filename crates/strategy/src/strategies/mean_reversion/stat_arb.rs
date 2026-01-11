//! Statistical Arbitrage / Correlation-Based Mean Reversion Strategy
//!
//! This strategy uses z-score of price spread (or correlation) to identify
//! mean reversion opportunities. Adapted for spot-only trading by using
//! correlation filtering instead of pairs trading.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Configuration for Statistical Arbitrage strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatArbConfig {
    /// Lookback period for correlation/spread calculation
    pub lookback_period: usize,
    /// Entry z-score threshold
    pub entry_zscore: f64,
    /// Exit z-score threshold
    pub exit_zscore: f64,
    /// Minimum correlation requirement (not used in spot-only adaptation)
    pub min_correlation: f64,
    /// Stop loss percentage
    pub stop_loss: f64,
}

impl StatArbConfig {
    pub fn new(lookback_period: usize, entry_zscore: f64, exit_zscore: f64) -> Self {
        Self {
            lookback_period,
            entry_zscore,
            exit_zscore,
            min_correlation: 0.8,
            stop_loss: 3.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            lookback_period: 30,
            entry_zscore: 2.0,
            exit_zscore: 0.0,
            min_correlation: 0.8,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.lookback_period == 0 {
            return Err("Lookback period must be greater than 0".to_string());
        }
        if self.entry_zscore <= 0.0 {
            return Err("Entry z-score must be positive".to_string());
        }
        if self.exit_zscore >= self.entry_zscore {
            return Err("Exit z-score must be less than entry z-score".to_string());
        }
        if self.min_correlation < 0.0 || self.min_correlation > 1.0 {
            return Err("Min correlation must be between 0 and 1".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be greater than 0".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for StatArbConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StatArb(period={}, entry_z={:.1}, exit_z={:.1})",
            self.lookback_period, self.entry_zscore, self.exit_zscore
        )
    }
}

/// Statistical Arbitrage Mean Reversion Strategy
///
/// Note: This is adapted for spot-only trading. In a full pairs trading implementation,
/// this would calculate correlation and spread between two assets. For spot-only,
/// we use normalized price deviation from moving average as a proxy.
pub struct StatArbStrategy {
    config: StatArbConfig,
    prices: VecDeque<f64>,
    last_position: SignalType,
    entry_price: Option<f64>,
}

impl StatArbStrategy {
    pub fn new(lookback_period: usize) -> Self {
        let config = StatArbConfig::new(lookback_period, 2.0, 0.0);
        Self::from_config(config)
    }

    pub fn from_config(config: StatArbConfig) -> Self {
        config.validate().expect("Invalid StatArbConfig");

        Self {
            prices: VecDeque::with_capacity(config.lookback_period),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
        }
    }

    pub fn config(&self) -> &StatArbConfig {
        &self.config
    }

    /// Calculate mean of prices
    fn calculate_mean(&self) -> Option<f64> {
        if self.prices.is_empty() {
            return None;
        }
        let sum: f64 = self.prices.iter().sum();
        Some(sum / self.prices.len() as f64)
    }

    /// Calculate standard deviation
    fn calculate_std_dev(&self, mean: f64) -> Option<f64> {
        if self.prices.len() < 2 {
            return None;
        }
        let variance: f64 = self
            .prices
            .iter()
            .map(|&price| (price - mean).powi(2))
            .sum::<f64>()
            / (self.prices.len() - 1) as f64;
        Some(variance.sqrt())
    }

    /// Calculate z-score (similar to pairs trading spread z-score)
    fn calculate_zscore(&self, price: f64) -> Option<f64> {
        let mean = self.calculate_mean()?;
        let std_dev = self.calculate_std_dev(mean)?;

        if std_dev == 0.0 {
            return None;
        }

        Some((price - mean) / std_dev)
    }
}

impl MetadataStrategy for StatArbStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Statistical Arbitrage Mean Reversion".to_string(),
            category: StrategyCategory::MeanReversion,
            sub_type: Some("statistical_arbitrage".to_string()),
            description: format!(
                "Statistical arbitrage mean reversion strategy with {}-period lookback. 
                Entry z-score {:.1}, exit z-score {:.1}. 
                Adapted for spot-only trading using price deviation from mean. 
                In a full implementation, this would trade pairs based on correlation and spread z-scores.",
                self.config.lookback_period, self.config.entry_zscore, self.config.exit_zscore
            ),
            hypothesis_path: "hypotheses/mean_reversion/stat_arb.md".to_string(),
            required_indicators: vec!["Mean".to_string(), "StdDev".to_string(), "ZScore".to_string()],
            expected_regimes: vec![MarketRegime::Sideways, MarketRegime::Ranging],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.25,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::High,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::MeanReversion
    }
}

impl Strategy for StatArbStrategy {
    fn name(&self) -> &str {
        "Statistical Arbitrage Mean Reversion"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;

        // Add price to window
        self.prices.push_back(price);
        if self.prices.len() > self.config.lookback_period {
            self.prices.pop_front();
        }

        // Need full lookback period
        if self.prices.len() < self.config.lookback_period {
            return None;
        }

        // Calculate z-score
        let zscore = self.calculate_zscore(price)?;
        let abs_zscore = zscore.abs();

        // EXIT LOGIC
        if self.last_position == SignalType::Buy {
            if let Some(entry) = self.entry_price {
                let profit_pct = (price - entry) / entry * 100.0;

                // Exit condition 1: Stop loss
                if profit_pct <= -self.config.stop_loss {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("Stop Loss Exit: {:.1}% loss", profit_pct)),
                    }]);
                }

                // Exit condition 2: Z-score returns to exit threshold
                if abs_zscore <= self.config.exit_zscore {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!(
                            "StatArb Exit: z-score={:.2} (returned to mean)",
                            zscore
                        )),
                    }]);
                }
            }
        }

        // ENTRY LOGIC - Z-score exceeds entry threshold
        // Note: In pairs trading, we would check if spread is overextended
        // Here we trade when price is significantly below mean (negative z-score)
        if self.last_position != SignalType::Buy && zscore <= -self.config.entry_zscore {
            self.last_position = SignalType::Buy;
            self.entry_price = Some(price);
            let strength = ((abs_zscore - self.config.entry_zscore) / self.config.entry_zscore)
                .clamp(0.3, 1.0);

            return Some(vec![Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Buy,
                strength,
                metadata: Some(format!(
                    "StatArb Entry: z-score={:.2} (< -{:.1})",
                    zscore, self.config.entry_zscore
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
    fn test_stat_arb_creation() {
        let strategy = StatArbStrategy::new(30);
        assert_eq!(strategy.name(), "Statistical Arbitrage Mean Reversion");
    }

    #[test]
    fn test_config_validation() {
        let config = StatArbConfig::default_config();
        assert!(config.validate().is_ok());

        let invalid_config = StatArbConfig {
            lookback_period: 0,
            ..StatArbConfig::default_config()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_zscore_calculation() {
        let mut strategy = StatArbStrategy::new(3);
        strategy.prices.push_back(10.0);
        strategy.prices.push_back(20.0);
        strategy.prices.push_back(30.0);

        let zscore = strategy.calculate_zscore(10.0);
        assert!(zscore.is_some());
        let z = zscore.unwrap();
        assert!((z - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn test_mean_and_stddev() {
        let mut strategy = StatArbStrategy::new(3);
        strategy.prices.push_back(10.0);
        strategy.prices.push_back(20.0);
        strategy.prices.push_back(30.0);

        assert_eq!(strategy.calculate_mean(), Some(20.0));

        let std_dev = strategy.calculate_std_dev(20.0);
        assert!(std_dev.is_some());
        assert!((std_dev.unwrap() - 10.0).abs() < 0.01);
    }
}
