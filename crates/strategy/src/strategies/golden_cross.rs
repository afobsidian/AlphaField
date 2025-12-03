//! Golden Cross Strategy
//!
//! A classic trend-following strategy that generates buy signals when a
//! fast-moving average crosses above a slow-moving average (golden cross),
//! and sell signals when the fast MA crosses below the slow MA (death cross).

use crate::config::{GoldenCrossConfig, StrategyConfig};
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
/// let config = GoldenCrossConfig::new(10, 30);
/// let strategy = GoldenCrossStrategy::from_config(config);
/// ```
pub struct GoldenCrossStrategy {
    config: GoldenCrossConfig,
    fast_sma: Sma,
    slow_sma: Sma,
    last_fast: Option<f64>,
    last_slow: Option<f64>,
}

impl GoldenCrossStrategy {
    /// Creates a new Golden Cross strategy with specified periods
    ///
    /// # Arguments
    /// * `fast_period` - Period for fast moving average
    /// * `slow_period` - Period for slow moving average
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        let config = GoldenCrossConfig::new(fast_period, slow_period);
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
        }
    }

    /// Returns the current configuration
    pub fn config(&self) -> &GoldenCrossConfig {
        &self.config
    }
}

impl Strategy for GoldenCrossStrategy {
    fn name(&self) -> &str {
        self.config.strategy_name()
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Signal> {
        let fast_opt = self.fast_sma.update(bar.close);
        let slow_opt = self.slow_sma.update(bar.close);

        let fast = fast_opt?;
        let slow = slow_opt?;

        let mut signal = None;

        if let (Some(prev_fast), Some(prev_slow)) = (self.last_fast, self.last_slow) {
            // Check for crossover
            if prev_fast <= prev_slow && fast > slow {
                // Golden Cross (Bullish)
                signal = Some(Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Buy,
                    strength: 1.0,
                    metadata: Some(format!("Golden Cross: Fast {:.2} > Slow {:.2}", fast, slow)),
                });
            } else if prev_fast >= prev_slow && fast < slow {
                // Death Cross (Bearish)
                signal = Some(Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: Some(format!("Death Cross: Fast {:.2} < Slow {:.2}", fast, slow)),
                });
            }
        }

        self.last_fast = Some(fast);
        self.last_slow = Some(slow);

        signal
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
    fn test_golden_cross_from_config() {
        let config = GoldenCrossConfig::new(5, 20);
        let strategy = GoldenCrossStrategy::from_config(config);
        assert_eq!(strategy.config().fast_period, 5);
        assert_eq!(strategy.config().slow_period, 20);
    }

    #[test]
    #[should_panic(expected = "Invalid GoldenCrossConfig")]
    fn test_golden_cross_invalid_config() {
        let config = GoldenCrossConfig::new(50, 20); // Invalid: fast > slow
        GoldenCrossStrategy::from_config(config);
    }
}
