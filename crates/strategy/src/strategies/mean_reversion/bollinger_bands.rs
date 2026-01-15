//! Bollinger Bands Mean Reversion Strategy
//!
//! This strategy uses Bollinger Bands to identify price extremes
//! and trade the reversion to the mean, with RSI confirmation to avoid catching falling knives.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{BollingerBands, Indicator, Rsi};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::debug;

/// Configuration for Bollinger Bands Reversion strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BollingerBandsConfig {
    /// Period for Bollinger Bands calculation
    pub period: usize,
    /// Number of standard deviations
    pub num_std_dev: f64,
    /// RSI period for confirmation
    pub rsi_period: usize,
    /// RSI oversold threshold
    pub rsi_oversold: f64,
    /// RSI overbought threshold
    pub rsi_overbought: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl BollingerBandsConfig {
    pub fn new(period: usize, num_std_dev: f64) -> Self {
        Self {
            period,
            num_std_dev,
            rsi_period: 14,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
            take_profit: 3.0,
            stop_loss: 5.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            period: 20,
            num_std_dev: 2.0,
            rsi_period: 14,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
            take_profit: 3.0,
            stop_loss: 5.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.period == 0 {
            return Err("Period must be greater than 0".to_string());
        }
        if self.num_std_dev <= 0.0 {
            return Err("Standard deviations must be positive".to_string());
        }
        if self.rsi_period == 0 {
            return Err("RSI period must be greater than 0".to_string());
        }
        if self.rsi_oversold <= 0.0 || self.rsi_oversold >= 100.0 {
            return Err("RSI oversold must be between 0 and 100".to_string());
        }
        if self.rsi_overbought <= 0.0 || self.rsi_overbought >= 100.0 {
            return Err("RSI overbought must be between 0 and 100".to_string());
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

impl fmt::Display for BollingerBandsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BollingerBands(period={}, std_dev={:.1}, rsi_period={}, tp={:.1}%, sl={:.1}%)",
            self.period, self.num_std_dev, self.rsi_period, self.take_profit, self.stop_loss
        )
    }
}

/// Bollinger Bands Mean Reversion Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: Price crosses below lower Bollinger Band AND RSI <= oversold
/// - **Sell Signal**: Price crosses above middle band OR RSI >= overbought OR TP/SL
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::mean_reversion::BollingerBandsStrategy;
///
/// let strategy = BollingerBandsStrategy::new(20, 2.0);
/// ```
pub struct BollingerBandsStrategy {
    config: BollingerBandsConfig,
    bb: BollingerBands,
    rsi: Rsi,
    last_position: SignalType,
    entry_price: Option<f64>,
    // State tracking for crossover detection
    last_price: Option<f64>,
    last_middle: Option<f64>,
    last_lower: Option<f64>,
}

impl BollingerBandsStrategy {
    /// Creates a new Bollinger Bands Reversion strategy
    ///
    /// # Arguments
    /// * `period` - Bollinger Bands period
    /// * `num_std_dev` - Number of standard deviations for bands
    pub fn new(period: usize, num_std_dev: f64) -> Self {
        let config = BollingerBandsConfig::new(period, num_std_dev);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: BollingerBandsConfig) -> Self {
        config.validate().expect("Invalid BollingerBandsConfig");

        Self {
            bb: BollingerBands::new(config.period, config.num_std_dev),
            rsi: Rsi::new(config.rsi_period),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            last_price: None,
            last_middle: None,
            last_lower: None,
        }
    }

    pub fn config(&self) -> &BollingerBandsConfig {
        &self.config
    }
}

impl Default for BollingerBandsStrategy {
    fn default() -> Self {
        // Default Bollinger Bands: 20-period, 2 standard deviations
        Self::new(20, 2.0)
    }
}

impl MetadataStrategy for BollingerBandsStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Bollinger Bands Mean Reversion".to_string(),
            category: StrategyCategory::MeanReversion,
            sub_type: Some("bollinger_bands".to_string()),
            description: format!(
                "Mean Reversion strategy using Bollinger Bands with period {} and {:.1} standard deviations,
                with RSI confirmation (period {}, oversold {:.0}, overbought {:.0}).
                Uses {:.1}% TP and {:.1}% SL. Buys when price crosses below lower band and RSI is oversold,
                sells on middle band crossover or RSI overbought.",
                self.config.period, self.config.num_std_dev, self.config.rsi_period,
                self.config.rsi_oversold, self.config.rsi_overbought,
                self.config.take_profit, self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/mean_reversion/bollinger_bands.md".to_string(),
            required_indicators: vec!["BollingerBands".to_string(), "RSI".to_string()],
            expected_regimes: vec![MarketRegime::Sideways, MarketRegime::LowVolatility, MarketRegime::Ranging],
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

impl Strategy for BollingerBandsStrategy {
    fn name(&self) -> &str {
        "Bollinger Bands Mean Reversion"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let (_upper, middle, lower) = self.bb.update(bar.close)?;
        let rsi_value = self.rsi.update(bar.close)?;

        let price = bar.close;

        // Get previous state
        let prev_price = self.last_price;
        let prev_middle = self.last_middle;
        let prev_lower = self.last_lower;

        // Update state for next bar
        self.last_price = Some(price);
        self.last_middle = Some(middle);
        self.last_lower = Some(lower);

        // EXIT LOGIC FIRST (before entry) - check if we should close position
        if self.last_position == SignalType::Buy {
            if let Some(entry) = self.entry_price {
                let profit_pct = (price - entry) / entry * 100.0;

                // Exit condition 1: Take profit (price-based)
                if profit_pct >= self.config.take_profit {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("Take Profit Exit: {:.1}% gain", profit_pct)),
                    }]);
                }

                // Exit condition 2: Stop loss (price-based)
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

                // Exit condition 3: RSI overbought
                if rsi_value >= self.config.rsi_overbought {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!("RSI Overbought Exit: RSI = {:.1}", rsi_value)),
                    }]);
                }

                // Exit condition 4: Price crosses above middle band (actual crossover)
                if let (Some(prev_p), Some(prev_m)) = (prev_price, prev_middle) {
                    if prev_p < prev_m && price >= middle {
                        debug!(
                            price = price,
                            middle = middle,
                            "Bollinger Bands Crossover Exit Triggered!"
                        );
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 1.0,
                            metadata: Some(format!(
                                "BB Middle Band Crossover Exit: Price crossed above {:.2}",
                                middle
                            )),
                        }]);
                    }
                }
            }
        }

        // ENTRY LOGIC - Price crosses below lower band AND RSI oversold (actual crossover)
        if self.last_position != SignalType::Buy {
            if let (Some(prev_p), Some(prev_l)) = (prev_price, prev_lower) {
                // Check for lower band crossover AND RSI oversold confirmation
                if prev_p >= prev_l && price < lower && rsi_value <= self.config.rsi_oversold {
                    self.last_position = SignalType::Buy;
                    self.entry_price = Some(price);
                    let distance = (middle - price) / middle;
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: distance.min(1.0),
                        metadata: Some(format!(
                            "BB Lower Band + RSI Oversold Entry: Price={:.2}, Lower={:.2}, RSI={:.1}",
                            price, lower, rsi_value
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
    fn test_bollinger_bands_creation() {
        let strategy = BollingerBandsStrategy::new(20, 2.0);
        assert_eq!(strategy.name(), "Bollinger Bands Mean Reversion");
    }

    #[test]
    fn test_config_validation() {
        let config = BollingerBandsConfig::new(20, 2.0);
        assert!(config.validate().is_ok());

        let invalid_config = BollingerBandsConfig {
            period: 0,
            num_std_dev: 2.0,
            rsi_period: 14,
            rsi_oversold: 30.0,
            rsi_overbought: 70.0,
            take_profit: 3.0,
            stop_loss: 5.0,
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_rsi_confirmation() {
        let config = BollingerBandsConfig::default_config();
        assert_eq!(config.rsi_period, 14);
        assert_eq!(config.rsi_oversold, 30.0);
        assert_eq!(config.rsi_overbought, 70.0);
    }
}
