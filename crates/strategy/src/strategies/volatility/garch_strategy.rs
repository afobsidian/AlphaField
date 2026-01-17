//! GARCH-Based Strategy
//!
//! This strategy uses a simplified GARCH (Generalized Autoregressive
//! Conditional Heteroskedasticity) model to predict future volatility.
//! The implementation uses EWMA (Exponentially Weighted Moving Average)
//! which is a computationally efficient approximation of GARCH(1,1).
//!
//! The core hypothesis: Volatility is clustered. Periods of high volatility
//! tend to be followed by high volatility, and vice versa. By predicting
//! volatility, we can:
//! - Enter positions when predicted volatility is low (lower risk)
//! - Exit or reduce size when predicted volatility spikes (high risk)
//! - Use volatility-adjusted position sizing

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Atr, Indicator, Sma};
use alphafield_core::{Bar, Signal, SignalType, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;

/// Configuration for GARCH-Based strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GARCHConfig {
    /// Smoothing factor λ (lambda) for EWMA (0.0-1.0)
    /// Higher values = slower adaptation, more persistent volatility
    /// Lower values = faster adaptation, more reactive
    /// Typical values: 0.94-0.97 for daily data
    pub lambda: f64,
    /// Window size for calculating returns
    pub return_window: usize,
    /// Fast MA for trend following
    pub fast_period: usize,
    /// Slow MA for trend following
    pub slow_period: usize,
    /// Volatility threshold for entry (enter when predicted vol < threshold)
    /// Expressed as percentile of historical predicted volatility (0.0-1.0)
    pub vol_entry_threshold: f64,
    /// Volatility multiplier for exit (exit when predicted vol spikes)
    /// Expressed as multiple of entry volatility
    pub vol_exit_multiplier: f64,
    /// Take Profit percentage
    pub take_profit: f64,
    /// Stop Loss percentage
    pub stop_loss: f64,
}

impl GARCHConfig {
    pub fn new(lambda: f64, return_window: usize) -> Self {
        Self {
            lambda,
            return_window,
            fast_period: 10,
            slow_period: 30,
            vol_entry_threshold: 0.4, // Enter when predicted vol is in bottom 40%
            vol_exit_multiplier: 2.0, // Exit when predicted vol is 2x entry vol
            take_profit: 6.0,
            stop_loss: 3.0,
        }
    }

    pub fn default_config() -> Self {
        Self {
            lambda: 0.94,
            return_window: 20,
            fast_period: 10,
            slow_period: 30,
            vol_entry_threshold: 0.4,
            vol_exit_multiplier: 2.0,
            take_profit: 6.0,
            stop_loss: 3.0,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.lambda <= 0.0 || self.lambda >= 1.0 {
            return Err("Lambda must be between 0 and 1".to_string());
        }
        if self.return_window == 0 {
            return Err("Return window must be greater than 0".to_string());
        }
        if self.fast_period == 0 || self.slow_period == 0 {
            return Err("MA periods must be greater than 0".to_string());
        }
        if self.fast_period >= self.slow_period {
            return Err("Fast period must be less than slow period".to_string());
        }
        if self.vol_entry_threshold <= 0.0 || self.vol_entry_threshold >= 1.0 {
            return Err("Volatility entry threshold must be between 0 and 1".to_string());
        }
        if self.vol_exit_multiplier <= 1.0 {
            return Err("Volatility exit multiplier must be greater than 1".to_string());
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

impl fmt::Display for GARCHConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GARCH(lambda={:.2}, return_win={}, fast_ma={}, slow_ma={}, vol_thresh={:.2}, vol_exit_mult={:.1}, tp={:.1}%, sl={:.1}%)",
            self.lambda,
            self.return_window,
            self.fast_period,
            self.slow_period,
            self.vol_entry_threshold,
            self.vol_exit_multiplier,
            self.take_profit,
            self.stop_loss
        )
    }
}

/// GARCH-Based Strategy
///
/// # Strategy Logic
/// - **Volatility Prediction**: EWMA model: σ²_t = λ * σ²_{t-1} + (1-λ) * r²_{t-1}
///   - Where r is log return, σ² is variance, λ is smoothing factor
/// - **Entry**: When predicted volatility is below historical threshold AND golden cross
///   - Lower predicted volatility = lower expected risk = good entry point
/// - **Exit**: When predicted volatility spikes > entry_vol * multiplier, TP, or SL
///   - Volatility spike = increased risk = exit or reduce position
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::volatility::GARCHStrategy;
///
/// let strategy = GARCHStrategy::new(0.94, 20);
/// ```
pub struct GARCHStrategy {
    config: GARCHConfig,
    ewma_variance: Option<f64>,
    predicted_vol_history: VecDeque<f64>, // Track predicted volatility for percentile calculation
    price_history: VecDeque<f64>,         // Track prices for return calculation
    fast_sma: Sma,
    slow_sma: Sma,
    atr: Atr,
    last_position: SignalType,
    entry_price: Option<f64>,
    entry_predicted_vol: Option<f64>,
}

impl Default for GARCHStrategy {
    fn default() -> Self {
        Self::from_config(GARCHConfig::default_config())
    }
}

impl GARCHStrategy {
    /// Creates a new GARCH-Based strategy
    ///
    /// # Arguments
    /// * `lambda` - Smoothing factor for EWMA (0.94-0.97 typical for daily data)
    /// * `return_window` - Window size for calculating returns
    pub fn new(lambda: f64, return_window: usize) -> Self {
        let config = GARCHConfig::new(lambda, return_window);
        Self::from_config(config)
    }

    /// Creates a strategy from a configuration object
    pub fn from_config(config: GARCHConfig) -> Self {
        config.validate().expect("Invalid GARCHConfig");

        Self {
            ewma_variance: None,
            predicted_vol_history: VecDeque::with_capacity(100),
            price_history: VecDeque::with_capacity(config.return_window + 1),
            fast_sma: Sma::new(config.fast_period),
            slow_sma: Sma::new(config.slow_period),
            atr: Atr::new(14),
            config,
            last_position: SignalType::Hold,
            entry_price: None,
            entry_predicted_vol: None,
        }
    }

    pub fn config(&self) -> &GARCHConfig {
        &self.config
    }

    /// Calculate log return from two prices
    fn calculate_log_return(old_price: f64, new_price: f64) -> f64 {
        if old_price <= 0.0 {
            return 0.0;
        }
        (new_price / old_price).ln()
    }

    /// Update EWMA variance using GARCH(1,1) simplified (EWMA)
    /// σ²_t = λ * σ²_{t-1} + (1-λ) * r²_{t-1}
    fn update_ewma_variance(&mut self, log_return: f64) -> f64 {
        let new_variance = if let Some(prev_variance) = self.ewma_variance {
            // EWMA update: σ²_t = λ * σ²_{t-1} + (1-λ) * r²_{t-1}
            (self.config.lambda * prev_variance) + ((1.0 - self.config.lambda) * log_return.powi(2))
        } else {
            // Initialize with current squared return
            log_return.powi(2)
        };

        self.ewma_variance = Some(new_variance);
        new_variance
    }

    /// Get predicted volatility (standard deviation from variance)
    fn get_predicted_volatility(&self) -> Option<f64> {
        self.ewma_variance.map(|v| v.sqrt())
    }

    /// Calculate percentile rank of current predicted volatility
    fn calculate_vol_percentile(&self) -> Option<f64> {
        if self.predicted_vol_history.is_empty() {
            return None;
        }

        if let Some(current_vol) = self.get_predicted_volatility() {
            let count = self
                .predicted_vol_history
                .iter()
                .filter(|&&v| v < current_vol)
                .count();
            Some(count as f64 / self.predicted_vol_history.len() as f64)
        } else {
            None
        }
    }
}

impl MetadataStrategy for GARCHStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "GARCH-Based".to_string(),
            category: StrategyCategory::VolatilityBased,
            sub_type: Some("garch_ewma".to_string()),
            description: format!(
                "GARCH(1,1) strategy using EWMA with λ={:.2} and {} return window. \
                Predicts volatility: σ² = λ*σ² + (1-λ)*r². \
                Enters when predicted vol < {:.0}% percentile (low vol) + {} / {} golden cross. \
                Exits when predicted vol > {:.1}x entry vol (vol spike), TP, or SL. \
                Uses {:.1}% TP and {:.1}% SL.",
                self.config.lambda,
                self.config.return_window,
                self.config.vol_entry_threshold * 100.0,
                self.config.fast_period,
                self.config.slow_period,
                self.config.vol_exit_multiplier,
                self.config.take_profit,
                self.config.stop_loss
            ),
            hypothesis_path: "hypotheses/volatility/garch_strategy.md".to_string(),
            required_indicators: vec![
                "EWMA Variance".to_string(),
                "SMA".to_string(),
                "Price".to_string(),
            ],
            expected_regimes: vec![
                MarketRegime::Bull,
                MarketRegime::Trending,
                MarketRegime::Ranging,
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
        StrategyCategory::VolatilityBased
    }
}

impl Strategy for GARCHStrategy {
    fn name(&self) -> &str {
        "GARCH-Based"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        let price = bar.close;

        // Update price history for return calculation
        self.price_history.push_back(price);
        if self.price_history.len() > self.config.return_window + 1 {
            self.price_history.pop_front();
        }

        // Calculate log return and update EWMA variance
        if self.price_history.len() >= 2 {
            let old_price = self.price_history[self.price_history.len() - 2];
            let log_return = Self::calculate_log_return(old_price, price);

            let variance = self.update_ewma_variance(log_return);
            let predicted_vol = variance.sqrt();

            // Track predicted volatility history for percentile calculation
            self.predicted_vol_history.push_back(predicted_vol);
            if self.predicted_vol_history.len() > 100 {
                self.predicted_vol_history.pop_front();
            }
        }

        // Update MAs
        let fast_ma = self.fast_sma.update(price);
        let slow_ma = self.slow_sma.update(price);

        // Update ATR for additional context
        let _atr_value = self.atr.update(bar);

        // Get current predicted volatility
        let predicted_vol = self.get_predicted_volatility()?;

        // ENTRY LOGIC (only when not in position)
        if self.last_position == SignalType::Hold {
            // Check if volatility is low enough for entry
            if let Some(vol_percentile) = self.calculate_vol_percentile() {
                if vol_percentile < self.config.vol_entry_threshold {
                    // Check for golden cross (trend confirmation)
                    if let (Some(fast_ma_val), Some(slow_ma_val)) = (fast_ma, slow_ma) {
                        // Simple crossover detection
                        let prev_fast =
                            self.fast_sma.value()? - (self.fast_sma.value()? - fast_ma_val);
                        let prev_slow =
                            self.slow_sma.value()? - (self.slow_sma.value()? - slow_ma_val);

                        if prev_fast <= prev_slow && fast_ma_val > slow_ma_val {
                            self.last_position = SignalType::Buy;
                            self.entry_price = Some(price);
                            self.entry_predicted_vol = Some(predicted_vol);

                            return Some(vec![Signal {
                                timestamp: bar.timestamp,
                                symbol: "UNKNOWN".to_string(),
                                signal_type: SignalType::Buy,
                                strength: 1.0,
                                metadata: Some(format!(
                                    "Low Vol Entry: Pred Vol {:.4} ({:.0}% percentile), Golden Cross (Fast {:.2} > Slow {:.2})",
                                    predicted_vol,
                                    vol_percentile * 100.0,
                                    fast_ma_val,
                                    slow_ma_val
                                )),
                            }]);
                        }
                    }
                }
            }
        }

        // EXIT LOGIC (only when in position)
        if self.last_position == SignalType::Buy {
            if let Some(entry) = self.entry_price {
                let profit_pct = (price - entry) / entry * 100.0;

                // Take Profit
                if profit_pct >= self.config.take_profit {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    self.entry_predicted_vol = None;

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Take Profit: {:.1}% profit, Pred Vol {:.4}",
                            profit_pct, predicted_vol
                        )),
                    }]);
                }

                // Stop Loss
                if profit_pct <= -self.config.stop_loss {
                    self.last_position = SignalType::Hold;
                    self.entry_price = None;
                    self.entry_predicted_vol = None;

                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: 1.0,
                        metadata: Some(format!(
                            "Stop Loss: {:.1}% loss, Pred Vol {:.4}",
                            profit_pct, predicted_vol
                        )),
                    }]);
                }

                // Volatility spike exit
                if let Some(entry_vol) = self.entry_predicted_vol {
                    let vol_threshold = entry_vol * self.config.vol_exit_multiplier;

                    if predicted_vol > vol_threshold {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        let exit_vol = self.entry_predicted_vol.take().unwrap();

                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 0.5, // Reduced strength for vol exit
                            metadata: Some(format!(
                                "Volatility Spike Exit: Pred Vol {:.4} > {:.4} ({:.1}x entry), Profit: {:.1}%",
                                predicted_vol,
                                vol_threshold,
                                predicted_vol / exit_vol,
                                profit_pct
                            )),
                        }]);
                    }
                }

                // Death cross exit
                if let (Some(fast_ma_val), Some(slow_ma_val)) = (fast_ma, slow_ma) {
                    let prev_fast = self.fast_sma.value()? - (self.fast_sma.value()? - fast_ma_val);
                    let prev_slow = self.slow_sma.value()? - (self.slow_sma.value()? - slow_ma_val);

                    if prev_fast >= prev_slow && fast_ma_val < slow_ma_val {
                        self.last_position = SignalType::Hold;
                        self.entry_price = None;
                        self.entry_predicted_vol = None;

                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 0.5,
                            metadata: Some(format!(
                                "Death Cross Exit: {:.1}% profit, Fast MA ({:.2}) < Slow MA ({:.2})",
                                profit_pct, fast_ma_val, slow_ma_val
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
    use chrono::{TimeZone, Utc};

    #[allow(dead_code)]
    fn create_test_bar(
        timestamp: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Bar {
        Bar {
            timestamp: Utc.timestamp_opt(timestamp, 0).unwrap(),
            open,
            high,
            low,
            close,
            volume,
        }
    }

    #[test]
    fn test_garch_creation() {
        let strategy = GARCHStrategy::new(0.94, 20);
        assert_eq!(strategy.config().lambda, 0.94);
        assert_eq!(strategy.config().return_window, 20);
    }

    #[test]
    fn test_garch_config_valid() {
        let config = GARCHConfig::new(0.94, 20);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_garch_invalid_config() {
        let config = GARCHConfig {
            lambda: 1.5, // > 1.0
            return_window: 20,
            fast_period: 10,
            slow_period: 30,
            vol_entry_threshold: 0.4,
            vol_exit_multiplier: 2.0,
            take_profit: 6.0,
            stop_loss: 3.0,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_garch_invalid_lambda_zero() {
        let config = GARCHConfig {
            lambda: 0.0, // = 0.0
            return_window: 20,
            fast_period: 10,
            slow_period: 30,
            vol_entry_threshold: 0.4,
            vol_exit_multiplier: 2.0,
            take_profit: 6.0,
            stop_loss: 3.0,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_garch_from_config() {
        let config = GARCHConfig::new(0.97, 25);
        let strategy = GARCHStrategy::from_config(config);
        assert_eq!(strategy.config().lambda, 0.97);
        assert_eq!(strategy.config().return_window, 25);
    }

    #[test]
    fn test_garch_metadata() {
        let strategy = GARCHStrategy::new(0.94, 20);
        let metadata = strategy.metadata();
        assert_eq!(metadata.name, "GARCH-Based");
        assert_eq!(metadata.category, StrategyCategory::VolatilityBased);
    }

    #[test]
    fn test_calculate_log_return() {
        let _strategy = GARCHStrategy::new(0.94, 20);

        // Positive return
        let ret1 = GARCHStrategy::calculate_log_return(100.0, 105.0);
        assert!((ret1 - 0.048790).abs() < 0.0001);

        // Negative return
        let ret2 = GARCHStrategy::calculate_log_return(100.0, 95.0);
        assert!((ret2 - (-0.051293)).abs() < 0.0001);

        // Zero price protection
        let ret3 = GARCHStrategy::calculate_log_return(0.0, 100.0);
        assert_eq!(ret3, 0.0);
    }

    #[test]
    fn test_update_ewma_variance() {
        let mut strategy = GARCHStrategy::new(0.94, 20);

        // First update - initialize
        let var1 = strategy.update_ewma_variance(0.05);
        assert!((var1 - 0.0025).abs() < 0.0001); // 0.05^2 = 0.0025
        assert!(strategy.ewma_variance.is_some());
        assert!((strategy.ewma_variance.unwrap() - 0.0025).abs() < 0.0001);

        // Second update - EWMA
        // σ² = 0.94 * 0.0025 + 0.06 * 0.03^2 = 0.00235 + 0.000054 = 0.002404
        let var2 = strategy.update_ewma_variance(0.03);
        assert!((var2 - 0.002404).abs() < 0.0001);
    }

    #[test]
    fn test_get_predicted_volatility() {
        let mut strategy = GARCHStrategy::new(0.94, 20);

        // No variance yet
        assert!(strategy.get_predicted_volatility().is_none());

        // Set variance
        strategy.ewma_variance = Some(0.0025);
        let vol = strategy.get_predicted_volatility();
        assert_eq!(vol, Some(0.05)); // sqrt(0.0025) = 0.05
    }

    #[test]
    fn test_calculate_vol_percentile() {
        let mut strategy = GARCHStrategy::new(0.94, 20);

        // Build up volatility history: [0.01, 0.02, 0.03, 0.04, 0.05]
        for vol in &[0.01, 0.02, 0.03, 0.04, 0.05] {
            strategy.predicted_vol_history.push_back(*vol);
        }

        // Current vol = 0.03, 2 values below it, percentile = 2/5 = 0.4
        strategy.ewma_variance = Some(0.0009); // 0.03^2 = 0.0009
        let percentile = strategy.calculate_vol_percentile();
        assert_eq!(percentile, Some(0.4));

        // Current vol = 0.05, 4 values below it, percentile = 4/5 = 0.8
        strategy.ewma_variance = Some(0.0025); // 0.05^2 = 0.0025
        let percentile2 = strategy.calculate_vol_percentile();
        assert_eq!(percentile2, Some(0.8));
    }

    #[test]
    fn test_garch_new_instance_clean_state() {
        let strategy = GARCHStrategy::new(0.94, 20);
        assert_eq!(strategy.last_position, SignalType::Hold);
        assert!(strategy.entry_price.is_none());
        assert!(strategy.entry_predicted_vol.is_none());
        assert!(strategy.ewma_variance.is_none());
        assert!(strategy.predicted_vol_history.is_empty());
    }

    #[test]
    fn test_config_display() {
        let config = GARCHConfig::new(0.94, 20);
        let display = format!("{}", config);
        assert!(display.contains("lambda=0.94"));
        assert!(display.contains("return_win=20"));
    }
}
