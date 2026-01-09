//! Golden Cross Strategy
//!
//! A classic trend-following strategy that generates buy signals when a
//! fast-moving average crosses above a slow-moving average (golden cross),
//! and sell signals when the fast MA crosses below the slow MA (death cross).

use crate::config::{GoldenCrossConfig, StrategyConfig};
use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Indicator, Sma};
use alphafield_core::{Bar, Signal, SignalType, Strategy};

/// Golden Cross Strategy
///
/// # Strategy Logic
/// - **Buy Signal**: Fast SMA crosses above Slow SMA (bullish crossover)
/// - **Sell Signal**: Fast SMA crosses below Slow SMA (bearish crossover)
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::GoldenCrossStrategy;
/// use alphafield_strategy::config::GoldenCrossConfig;
///
/// let config = GoldenCrossConfig::new(10, 30, 5.0, 5.0);
/// let strategy = GoldenCrossStrategy::from_config(config);
/// ```
pub struct GoldenCrossStrategy {
    config: GoldenCrossConfig,
    fast_sma: Sma,
    slow_sma: Sma,
    last_fast: Option<f64>,
    last_slow: Option<f64>,
    entry_price: Option<f64>,
}

impl GoldenCrossStrategy {
    /// Creates a new Golden Cross strategy with specified periods
    ///
    /// # Arguments
    /// * `fast_period` - Period for fast moving average
    /// * `slow_period` - Period for slow moving average
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        // Default to 5% TP/SL for backward compatibility in constructor
        let config = GoldenCrossConfig::new(fast_period, slow_period, 5.0, 5.0);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    ///
    /// # Panics
    /// Panics if the configuration is invalid
    pub fn from_config(config: GoldenCrossConfig) -> Self {
        config.validate().expect("Invalid GoldenCrossConfig");

        Self {
            fast_sma: Sma::new(config.fast_period),
            slow_sma: Sma::new(config.slow_period),
            config,
            last_fast: None,
            last_slow: None,
            entry_price: None,
        }
    }

    /// Returns the current configuration
    pub fn config(&self) -> &GoldenCrossConfig {
        &self.config
    }
}

impl MetadataStrategy for GoldenCrossStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: self.config.strategy_name().to_string(),
            category: StrategyCategory::TrendFollowing,
            sub_type: Some("moving_average_crossover".to_string()),
            description: format!(
                "Golden Cross strategy using {} and {} period SMAs with {:.1}% TP and {:.1}% SL. 
                Generates buy signals on golden cross (fast MA crosses above slow MA) and sell signals on death cross.",
                self.config.fast_period, self.config.slow_period, self.config.take_profit, self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/trend_following/golden_cross.md".to_string(),
            required_indicators: vec!["SMA".to_string(), "Price".to_string()],
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Trending],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.25,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Medium,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::TrendFollowing
    }
}

impl Strategy for GoldenCrossStrategy {
    fn name(&self) -> &str {
        self.config.strategy_name()
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let fast_opt = self.fast_sma.update(bar.close);
        let slow_opt = self.slow_sma.update(bar.close);

        let fast = fast_opt?;
        let slow = slow_opt?;
        let price = bar.close;

        // EXIT LOGIC FIRST
        if let Some(entry) = self.entry_price {
            // Exit 1: Death cross
            if let Some(prev_fast) = self.last_fast {
                if let Some(prev_slow) = self.last_slow {
                    if prev_fast >= prev_slow && fast < slow {
                        self.entry_price = None;
                        self.last_fast = Some(fast);
                        self.last_slow = Some(slow);
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 1.0,
                            metadata: Some(format!(
                                "Death Cross: Fast {:.2} < Slow {:.2}",
                                fast, slow
                            )),
                        }]);
                    }
                }
            }

            // Exit 2: Take profit
            let profit_pct = (price - entry) / entry * 100.0;
            if profit_pct >= self.config.take_profit {
                self.entry_price = None;
                self.last_fast = Some(fast);
                self.last_slow = Some(slow);
                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: Some(format!("Take Profit: {:.1}%", profit_pct)),
                }]);
            }

            // Exit 3: Stop loss
            if profit_pct <= -self.config.stop_loss {
                self.entry_price = None;
                self.last_fast = Some(fast);
                self.last_slow = Some(slow);
                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: Some(format!("Stop Loss: {:.1}%", profit_pct)),
                }]);
            }
        }

        // ENTRY LOGIC
        if let (Some(prev_fast), Some(prev_slow)) = (self.last_fast, self.last_slow) {
            if prev_fast <= prev_slow && fast > slow && self.entry_price.is_none() {
                // Golden Cross - Entry
                self.entry_price = Some(price);
                self.last_fast = Some(fast);
                self.last_slow = Some(slow);
                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Buy,
                    strength: 1.0,
                    metadata: Some(format!("Golden Cross: Fast {:.2} > Slow {:.2}", fast, slow)),
                }]);
            }
        }

        self.last_fast = Some(fast);
        self.last_slow = Some(slow);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_cross_creation() {
        let strategy = GoldenCrossStrategy::new(10, 30);
        assert_eq!(strategy.name(), "Golden Cross");
    }

    #[test]
    fn test_golden_cross_config_valid() {
        let config = GoldenCrossConfig::new(10, 30, 5.0, 5.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_golden_cross_from_config() {
        let config = GoldenCrossConfig::new(5, 20, 10.0, 5.0);
        let strategy = GoldenCrossStrategy::from_config(config);
        assert_eq!(strategy.config().fast_period, 5);
        assert_eq!(strategy.config().slow_period, 20);
        assert_eq!(strategy.config().take_profit, 10.0);
    }

    #[test]
    #[should_panic(expected = "Invalid GoldenCrossConfig")]
    fn test_golden_cross_invalid_config() {
        let config = GoldenCrossConfig::new(50, 20, 5.0, 5.0); // Invalid: fast > slow
        GoldenCrossStrategy::from_config(config);
    }
}
