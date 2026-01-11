//! Volume Weighted Momentum Strategy
//!
//! Confirms price momentum with volume.
//! Only enters when momentum is supported by above-average and increasing volume.

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Ema, Indicator};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Configuration for Volume Weighted Momentum strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMomentumConfig {
    /// EMA period for price trend
    pub price_ema_period: usize,
    /// Period for volume average
    pub volume_period: usize,
    /// Volume multiplier threshold (e.g., 1.5 = 150% of average)
    pub volume_multiplier: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl VolumeMomentumConfig {
    pub fn new(
        price_ema_period: usize,
        volume_period: usize,
        volume_multiplier: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        Self {
            price_ema_period,
            volume_period,
            volume_multiplier,
            take_profit,
            stop_loss,
        }
    }

    pub fn default_config() -> Self {
        Self {
            price_ema_period: 20,
            volume_period: 20,
            volume_multiplier: 1.5,
            take_profit: 5.0,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.price_ema_period == 0 {
            return Err("Price EMA period must be greater than 0".to_string());
        }
        if self.volume_period == 0 {
            return Err("Volume period must be greater than 0".to_string());
        }
        if self.volume_multiplier <= 0.0 {
            return Err("Volume multiplier must be greater than 0".to_string());
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

impl fmt::Display for VolumeMomentumConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Volume_Momentum(price_ema={}, vol_period={}, vol_mult={:.1}x, tp={:.1}%, sl={:.1}%)",
            self.price_ema_period,
            self.volume_period,
            self.volume_multiplier,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// Volume Weighted Momentum Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: Price crosses above EMA AND volume > volume_multiplier * avg_volume
/// - **Sell Signal**: Price crosses below EMA OR volume drops OR TP/SL
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::momentum::VolumeMomentumStrategy;
///
/// let strategy = VolumeMomentumStrategy::new(20, 20, 1.5, 5.0, 3.0);
/// ```
pub struct VolumeMomentumStrategy {
    config: VolumeMomentumConfig,
    price_ema: Ema,
    volumes: VecDeque<f64>,
    last_position: SignalType,
    entry_price: Option<f64>,
    last_price_above_ema: Option<bool>,
    last_volume: Option<f64>,
}

impl VolumeMomentumStrategy {
    /// Creates a new Volume Momentum strategy
    pub fn new(
        price_ema_period: usize,
        volume_period: usize,
        volume_multiplier: f64,
        take_profit: f64,
        stop_loss: f64,
    ) -> Self {
        let config = VolumeMomentumConfig::new(
            price_ema_period,
            volume_period,
            volume_multiplier,
            take_profit,
            stop_loss,
        );
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: VolumeMomentumConfig) -> Self {
        config.validate().expect("Invalid VolumeMomentumConfig");

        Self {
            price_ema: Ema::new(config.price_ema_period),
            volumes: VecDeque::with_capacity(config.volume_period + 1),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            last_price_above_ema: None,
            last_volume: None,
        }
    }

    pub fn config(&self) -> &VolumeMomentumConfig {
        &self.config
    }

    /// Calculate average volume
    fn calculate_avg_volume(&self) -> Option<f64> {
        if self.volumes.is_empty() {
            return None;
        }
        let sum: f64 = self.volumes.iter().sum();
        Some(sum / self.volumes.len() as f64)
    }

    /// Check if volume confirms momentum
    fn volume_confirms(&self, current_volume: f64) -> bool {
        if let Some(avg_vol) = self.calculate_avg_volume() {
            current_volume >= avg_vol * self.config.volume_multiplier
        } else {
            false
        }
    }

    /// Check if volume is increasing
    fn volume_increasing(&self, current_volume: f64) -> bool {
        if let Some(last_vol) = self.last_volume {
            current_volume > last_vol
        } else {
            false
        }
    }
}

impl MetadataStrategy for VolumeMomentumStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Volume Momentum".to_string(),
            category: StrategyCategory::Momentum,
            sub_type: Some("volume_weighted".to_string()),
            description: format!(
                "Volume-confirmed momentum strategy. Uses {}-period EMA for price trend and requires volume >= {:.1}x the {}-period average for entry. Only enters when both price momentum and volume momentum align. Uses {:.1}% TP and {:.1}% SL.",
                self.config.price_ema_period,
                self.config.volume_multiplier,
                self.config.volume_period,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/momentum/volume_momentum.md".to_string(),
            required_indicators: vec![
                "EMA".to_string(),
                "Volume".to_string(),
            ],
            expected_regimes: vec![
                MarketRegime::Bull,
                MarketRegime::Trending,
            ],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.18,
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

impl Strategy for VolumeMomentumStrategy {
    fn name(&self) -> &str {
        "Volume Momentum"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;
        let volume = bar.volume;

        // Update volume history
        self.volumes.push_back(volume);
        if self.volumes.len() > self.config.volume_period {
            self.volumes.pop_front();
        }

        // Update price EMA
        let ema_val = self.price_ema.update(price)?;
        let price_above_ema = price > ema_val;
        let prev_price_above_ema = self.last_price_above_ema;

        // Check volume conditions
        let volume_confirmed = self.volume_confirms(volume);
        let volume_inc = self.volume_increasing(volume);

        // Update state for next bar
        self.last_price_above_ema = Some(price_above_ema);
        self.last_volume = Some(volume);

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

                // Exit on price crossing below EMA
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

                // Exit on volume decline (loss of momentum confirmation)
                if !volume_confirmed {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    let avg_vol = self.calculate_avg_volume().unwrap_or(0.0);
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 0.7,
                        metadata: Some(format!(
                            "Volume Decline Exit: Vol {:.0} < {:.1}x avg {:.0}",
                            volume, self.config.volume_multiplier, avg_vol
                        )),
                    }]);
                }
            }
        }

        // ENTRY LOGIC - Price crosses above EMA with strong volume
        if self.last_position != SignalType::Buy {
            if let Some(was_above) = prev_price_above_ema {
                // Price must cross above EMA
                if !was_above && price_above_ema {
                    // Volume must confirm momentum
                    if volume_confirmed {
                        // Calculate signal strength based on volume strength and increase
                        let strength = if volume_inc { 1.0 } else { 0.8 };

                        self.last_position = SignalType::Buy;
                        self.entry_price = Some(price);
                        let avg_vol = self.calculate_avg_volume().unwrap_or(0.0);
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Buy,
                            strength,
                            metadata: Some(format!(
                                "Volume Momentum Entry: Price {:.2} > EMA {:.2}, Vol {:.0} ({:.1}x avg {:.0}){}",
                                price,
                                ema_val,
                                volume,
                                volume / avg_vol,
                                avg_vol,
                                if volume_inc { " increasing" } else { "" }
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

    #[test]
    fn test_volume_momentum_creation() {
        let strategy = VolumeMomentumStrategy::new(20, 20, 1.5, 5.0, 3.0);
        assert_eq!(strategy.name(), "Volume Momentum");
    }

    #[test]
    fn test_config_validation() {
        let config = VolumeMomentumConfig::default_config();
        assert!(config.validate().is_ok());

        let invalid_config = VolumeMomentumConfig::new(20, 20, 0.0, 5.0, 3.0); // volume_multiplier = 0
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_metadata() {
        let strategy = VolumeMomentumStrategy::new(20, 20, 1.5, 5.0, 3.0);
        let metadata = strategy.metadata();
        assert_eq!(metadata.category, StrategyCategory::Momentum);
        assert_eq!(metadata.name, "Volume Momentum");
    }

    #[test]
    fn test_avg_volume_calculation() {
        let mut strategy = VolumeMomentumStrategy::new(20, 5, 1.5, 5.0, 3.0);

        // Build up volume history
        for i in 1..=5 {
            strategy.volumes.push_back(100.0 * i as f64);
        }

        let avg = strategy.calculate_avg_volume().unwrap();
        assert_eq!(avg, 300.0); // (100+200+300+400+500)/5 = 300
    }

    #[test]
    fn test_volume_confirms() {
        let mut strategy = VolumeMomentumStrategy::new(20, 5, 1.5, 5.0, 3.0);

        // Build up volume history with avg = 100
        for _ in 1..=5 {
            strategy.volumes.push_back(100.0);
        }

        assert!(strategy.volume_confirms(150.0)); // 150 >= 100 * 1.5
        assert!(!strategy.volume_confirms(140.0)); // 140 < 100 * 1.5
    }
}
