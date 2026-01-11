//! Z-Score Mean Reversion Strategy
//!
//! This strategy uses statistical z-scores to identify when prices are significantly
//! deviated from their mean and trades the reversion.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Configuration for Z-Score Reversion strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZScoreReversionConfig {
    /// Lookback period for calculating mean and standard deviation
    pub lookback_period: usize,
    /// Entry z-score threshold (typically -2.0)
    pub entry_zscore: f64,
    /// Exit z-score threshold (typically 0.0 - back to mean)
    pub exit_zscore: f64,
    /// Minimum price change to avoid flat markets (percentage)
    pub min_price_change: f64,
    /// Stop loss percentage
    pub stop_loss: f64,
}

impl ZScoreReversionConfig {
    pub fn new(lookback_period: usize, entry_zscore: f64, exit_zscore: f64) -> Self {
        Self {
            lookback_period,
            entry_zscore,
            exit_zscore,
            min_price_change: 1.0,
            stop_loss: 3.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            lookback_period: 20,
            entry_zscore: -2.0,
            exit_zscore: 0.0,
            min_price_change: 1.0,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.lookback_period == 0 {
            return Err("Lookback period must be greater than 0".to_string());
        }
        if self.entry_zscore >= 0.0 {
            return Err("Entry z-score must be negative".to_string());
        }
        if self.exit_zscore <= self.entry_zscore {
            return Err("Exit z-score must be greater than entry z-score".to_string());
        }
        if self.min_price_change < 0.0 {
            return Err("Min price change must be non-negative".to_string());
        }
        if self.stop_loss <= 0.0 {
            return Err("Stop loss must be greater than 0".to_string());
        }
        Ok(())
    }
}

impl fmt::Display for ZScoreReversionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ZScoreReversion(period={}, entry_z={:.1}, exit_z={:.1})",
            self.lookback_period, self.entry_zscore, self.exit_zscore
        )
    }
}

/// Z-Score Mean Reversion Strategy
pub struct ZScoreReversionStrategy {
    config: ZScoreReversionConfig,
    prices: VecDeque<f64>,
    last_position: SignalType,
    entry_price: Option<f64>,
}

impl ZScoreReversionStrategy {
    pub fn new(lookback_period: usize) -> Self {
        let config = ZScoreReversionConfig::new(lookback_period, -2.0, 0.0);
        Self::from_config(config)
    }

    pub fn from_config(config: ZScoreReversionConfig) -> Self {
        config.validate().expect("Invalid ZScoreReversionConfig");

        Self {
            prices: VecDeque::with_capacity(config.lookback_period),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
        }
    }

    pub fn config(&self) -> &ZScoreReversionConfig {
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

    /// Calculate standard deviation of prices
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

    /// Calculate z-score for current price
    fn calculate_zscore(&self, price: f64) -> Option<f64> {
        let mean = self.calculate_mean()?;
        let std_dev = self.calculate_std_dev(mean)?;

        if std_dev == 0.0 {
            return None;
        }

        Some((price - mean) / std_dev)
    }

    /// Check if price has moved enough to avoid flat markets
    fn has_sufficient_movement(&self) -> bool {
        if self.prices.len() < 2 {
            return false;
        }

        let first = *self.prices.front().unwrap();
        let last = *self.prices.back().unwrap();
        let change_pct = ((last - first) / first).abs() * 100.0;

        change_pct >= self.config.min_price_change
    }
}

impl MetadataStrategy for ZScoreReversionStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Z-Score Mean Reversion".to_string(),
            category: StrategyCategory::MeanReversion,
            sub_type: Some("zscore_reversion".to_string()),
            description: format!(
                "Statistical z-score mean reversion strategy with {}-period lookback. 
                Entry z-score {:.1}, exit z-score {:.1}. Requires minimum {:.1}% price change to avoid flat markets. 
                Buys when price is > {:.0} standard deviations below mean, sells when returns to mean.",
                self.config.lookback_period, self.config.entry_zscore, self.config.exit_zscore,
                self.config.min_price_change, self.config.entry_zscore.abs()
            ),
            hypothesis_path: "hypotheses/mean_reversion/zscore_reversion.md".to_string(),
            required_indicators: vec!["Mean".to_string(), "StdDev".to_string()],
            expected_regimes: vec![MarketRegime::Sideways, MarketRegime::Ranging],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.25,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::MeanReversion
    }
}

impl Strategy for ZScoreReversionStrategy {
    fn name(&self) -> &str {
        "Z-Score Mean Reversion"
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

        // Check for sufficient price movement
        let has_movement = self.has_sufficient_movement();

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

                // Exit condition 2: Z-score returns to exit threshold (mean)
                if zscore >= self.config.exit_zscore {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("Z-Score Exit: z={:.2} (returned to mean)", zscore)),
                    }]);
                }

                // Exit condition 3: Z-score becomes positive (profit target)
                if zscore >= 1.0 {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("Z-Score Profit Target: z={:.2}", zscore)),
                    }]);
                }
            }
        }

        // ENTRY LOGIC - Z-score extreme negative and sufficient price movement
        if self.last_position != SignalType::Buy
            && zscore <= self.config.entry_zscore
            && has_movement
        {
            self.last_position = SignalType::Buy;
            self.entry_price = Some(price);
            let strength =
                (zscore.abs() - self.config.entry_zscore.abs()) / self.config.entry_zscore.abs();

            return Some(vec![Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Buy,
                strength: strength.clamp(0.3, 1.0),
                metadata: Some(format!(
                    "Z-Score Extreme Entry: z={:.2} (< {:.1})",
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
    fn test_zscore_reversion_creation() {
        let strategy = ZScoreReversionStrategy::new(20);
        assert_eq!(strategy.name(), "Z-Score Mean Reversion");
    }

    #[test]
    fn test_config_validation() {
        let config = ZScoreReversionConfig::default_config();
        assert!(config.validate().is_ok());

        let invalid_config = ZScoreReversionConfig {
            lookback_period: 0,
            ..ZScoreReversionConfig::default_config()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_calculate_mean() {
        let mut strategy = ZScoreReversionStrategy::new(3);
        strategy.prices.push_back(10.0);
        strategy.prices.push_back(20.0);
        strategy.prices.push_back(30.0);

        assert_eq!(strategy.calculate_mean(), Some(20.0));
    }

    #[test]
    fn test_calculate_zscore() {
        let mut strategy = ZScoreReversionStrategy::new(3);
        strategy.prices.push_back(10.0);
        strategy.prices.push_back(20.0);
        strategy.prices.push_back(30.0);

        // Mean = 20, Price = 10, StdDev = 10, Z = (10-20)/10 = -1.0
        let zscore = strategy.calculate_zscore(10.0);
        assert!(zscore.is_some());
        let z = zscore.unwrap();
        assert!((z - (-1.0)).abs() < 0.01);
    }
}
