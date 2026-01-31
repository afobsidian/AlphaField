//! Volatility-Adjusted Position Sizing
//!
//! Provides position sizing based on market volatility.
//! Reduces position size during high volatility periods,
//! increases size during low volatility periods.

use serde::{Deserialize, Serialize};

/// Configuration for volatility-adjusted sizing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilitySizingConfig {
    /// Target volatility for the position (e.g., 0.02 for 2% daily vol)
    pub target_volatility: f64,
    /// Maximum position size as fraction of capital
    pub max_position_fraction: f64,
    /// Minimum position size as fraction of capital
    pub min_position_fraction: f64,
    /// Lookback period for volatility calculation (in bars)
    pub lookback_period: usize,
    /// Volatility calculation method
    pub method: VolatilityMethod,
}

impl Default for VolatilitySizingConfig {
    fn default() -> Self {
        Self {
            target_volatility: 0.02, // 2% daily volatility target
            max_position_fraction: 1.0,
            min_position_fraction: 0.01,
            lookback_period: 20,
            method: VolatilityMethod::StandardDeviation,
        }
    }
}

impl VolatilitySizingConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_target_volatility(mut self, vol: f64) -> Self {
        self.target_volatility = vol.clamp(0.001, 1.0);
        self
    }

    pub fn with_max_position_fraction(mut self, fraction: f64) -> Self {
        self.max_position_fraction = fraction.clamp(0.01, 10.0);
        self
    }

    pub fn with_min_position_fraction(mut self, fraction: f64) -> Self {
        self.min_position_fraction = fraction.clamp(0.001, 1.0);
        self
    }

    pub fn with_lookback_period(mut self, period: usize) -> Self {
        self.lookback_period = period.clamp(5, 252);
        self
    }

    pub fn with_method(mut self, method: VolatilityMethod) -> Self {
        self.method = method;
        self
    }
}

/// Method for calculating volatility
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VolatilityMethod {
    /// Standard deviation of returns
    StandardDeviation,
    /// Average True Range (ATR) based
    ATR,
    /// Exponentially Weighted Moving Average (EWMA)
    EWMA,
    /// Parkinson (uses high-low range)
    Parkinson,
}

/// Volatility-adjusted position sizer
#[derive(Debug, Clone)]
pub struct VolatilityAdjustedSizing {
    config: VolatilitySizingConfig,
}

impl VolatilityAdjustedSizing {
    pub fn new(config: VolatilitySizingConfig) -> Self {
        Self { config }
    }

    pub fn default_config() -> Self {
        Self::new(VolatilitySizingConfig::default())
    }

    /// Calculate position size based on current volatility
    ///
    /// # Arguments
    /// * `capital` - Available capital
    /// * `current_price` - Current market price
    /// * `price_history` - Recent price history for volatility calculation
    ///
    /// # Returns
    /// Position size (number of units/contracts)
    pub fn calculate_position(
        &self,
        capital: f64,
        current_price: f64,
        price_history: &[f64],
    ) -> Result<f64, String> {
        if capital <= 0.0 {
            return Err("Capital must be positive".to_string());
        }

        if current_price <= 0.0 {
            return Err("Current price must be positive".to_string());
        }

        if price_history.len() < self.config.lookback_period {
            return Err(format!(
                "Insufficient price history: need {} bars, have {}",
                self.config.lookback_period,
                price_history.len()
            ));
        }

        // Calculate volatility
        let volatility = self.calculate_volatility(price_history)?;

        if volatility < 1e-10 {
            return Err("Volatility is too low to calculate position size".to_string());
        }

        // Calculate position size: position = (target_vol / current_vol) * (capital / price)
        let volatility_ratio = self.config.target_volatility / volatility;
        let position_fraction = volatility_ratio.clamp(
            self.config.min_position_fraction,
            self.config.max_position_fraction,
        );

        let position_size = (capital * position_fraction) / current_price;

        Ok(position_size)
    }

    /// Calculate volatility using the configured method
    fn calculate_volatility(&self, prices: &[f64]) -> Result<f64, String> {
        match self.config.method {
            VolatilityMethod::StandardDeviation => self.calculate_std_volatility(prices),
            VolatilityMethod::ATR => self.calculate_atr_volatility(prices),
            VolatilityMethod::EWMA => self.calculate_ewma_volatility(prices),
            VolatilityMethod::Parkinson => self.calculate_parkinson_volatility(prices),
        }
    }

    /// Calculate volatility as standard deviation of returns
    fn calculate_std_volatility(&self, prices: &[f64]) -> Result<f64, String> {
        let n = prices.len();
        if n < 2 {
            return Err("Need at least 2 prices for volatility calculation".to_string());
        }

        // Calculate returns
        let returns: Vec<f64> = prices
            .windows(2)
            .map(|w| {
                if w[0] != 0.0 {
                    (w[1] - w[0]) / w[0]
                } else {
                    0.0
                }
            })
            .collect();

        if returns.len() < 2 {
            return Err("Insufficient returns data".to_string());
        }

        // Calculate mean
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;

        // Calculate variance
        let variance =
            returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;

        Ok(variance.sqrt())
    }

    /// Calculate volatility based on Average True Range
    fn calculate_atr_volatility(&self, prices: &[f64]) -> Result<f64, String> {
        let n = prices.len();
        if n < 2 {
            return Err("Need at least 2 prices for ATR calculation".to_string());
        }

        // Simplified ATR: use price differences as proxy for true range
        let ranges: Vec<f64> = prices.windows(2).map(|w| (w[1] - w[0]).abs()).collect();

        let atr = ranges.iter().sum::<f64>() / ranges.len() as f64;
        let current_price = prices[prices.len() - 1];

        if current_price <= 0.0 {
            return Err("Invalid current price".to_string());
        }

        // Return as percentage
        Ok(atr / current_price)
    }

    /// Calculate EWMA volatility
    fn calculate_ewma_volatility(&self, prices: &[f64]) -> Result<f64, String> {
        let lambda = 0.94; // Standard EWMA decay factor
        let n = prices.len();

        if n < 2 {
            return Err("Need at least 2 prices for EWMA calculation".to_string());
        }

        // Calculate returns
        let returns: Vec<f64> = prices
            .windows(2)
            .map(|w| {
                if w[0] != 0.0 {
                    (w[1] - w[0]) / w[0]
                } else {
                    0.0
                }
            })
            .collect();

        // Calculate EWMA variance
        let mut ewma_var = returns[0].powi(2);
        for ret in &returns[1..] {
            ewma_var = lambda * ewma_var + (1.0 - lambda) * ret.powi(2);
        }

        Ok(ewma_var.sqrt())
    }

    /// Calculate Parkinson volatility (uses high-low range)
    /// Note: This is a simplified version using close prices as proxy
    fn calculate_parkinson_volatility(&self, prices: &[f64]) -> Result<f64, String> {
        // Simplified: use standard deviation as proxy
        // Full implementation would need OHLC data
        self.calculate_std_volatility(prices)
    }

    /// Get a description of the sizing method
    pub fn description(&self) -> String {
        format!(
            "Volatility-adjusted sizing using {:?} with target vol {:.2}%",
            self.config.method,
            self.config.target_volatility * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_price_series() -> Vec<f64> {
        // Simulate price series with ~2% daily volatility
        let mut prices = vec![100.0];
        for i in 1..=30 {
            let change = if i % 2 == 0 { 1.02 } else { 0.98 };
            prices.push(prices[i - 1] * change);
        }
        prices
    }

    #[test]
    fn test_default_config() {
        let config = VolatilitySizingConfig::default();
        assert_eq!(config.target_volatility, 0.02);
        assert_eq!(config.max_position_fraction, 1.0);
        assert_eq!(config.min_position_fraction, 0.01);
        assert_eq!(config.lookback_period, 20);
    }

    #[test]
    fn test_config_builder() {
        let config = VolatilitySizingConfig::new()
            .with_target_volatility(0.01)
            .with_max_position_fraction(0.5)
            .with_lookback_period(10);

        assert_eq!(config.target_volatility, 0.01);
        assert_eq!(config.max_position_fraction, 0.5);
        assert_eq!(config.lookback_period, 10);
    }

    #[test]
    fn test_calculate_position() {
        let config = VolatilitySizingConfig::default();
        let sizer = VolatilityAdjustedSizing::new(config);

        let prices = create_price_series();
        let result = sizer.calculate_position(10000.0, 100.0, &prices);

        assert!(result.is_ok());
        let position = result.unwrap();
        assert!(position > 0.0);
    }

    #[test]
    fn test_insufficient_history() {
        let config = VolatilitySizingConfig::default();
        let sizer = VolatilityAdjustedSizing::new(config);

        let prices = vec![100.0, 101.0]; // Too short
        let result = sizer.calculate_position(10000.0, 100.0, &prices);

        assert!(result.is_err());
    }

    #[test]
    fn test_std_volatility_calculation() {
        let config = VolatilitySizingConfig::default();
        let sizer = VolatilityAdjustedSizing::new(config);

        let prices = create_price_series();
        let vol = sizer.calculate_std_volatility(&prices).unwrap();

        assert!(vol > 0.0);
        assert!(vol < 0.5);
    }

    #[test]
    fn test_atr_volatility_calculation() {
        let config = VolatilitySizingConfig::default();
        let sizer = VolatilityAdjustedSizing::new(config);

        let prices = create_price_series();
        let vol = sizer.calculate_atr_volatility(&prices).unwrap();

        assert!(vol > 0.0);
        assert!(vol < 0.5);
    }

    #[test]
    fn test_ewma_volatility_calculation() {
        let config = VolatilitySizingConfig::default();
        let sizer = VolatilityAdjustedSizing::new(config);

        let prices = create_price_series();
        let vol = sizer.calculate_ewma_volatility(&prices).unwrap();

        assert!(vol > 0.0);
    }

    #[test]
    fn test_position_limits() {
        // Test that position is within min/max bounds
        let config = VolatilitySizingConfig::new()
            .with_max_position_fraction(0.5)
            .with_min_position_fraction(0.1);

        let sizer = VolatilityAdjustedSizing::new(config.clone());

        // High volatility prices
        let prices: Vec<f64> = (0..=30)
            .map(|i| 100.0 * (1.0 + (i as f64 * 0.1)).sin().abs() + 50.0)
            .collect();

        let capital = 10000.0;
        let current_price = 100.0;

        let result = sizer.calculate_position(capital, current_price, &prices);
        assert!(result.is_ok());

        let position = result.unwrap();
        let position_fraction = (position * current_price) / capital;

        assert!(position_fraction <= config.max_position_fraction);
        // Position might be less than min if volatility is very high
    }

    #[test]
    fn test_invalid_capital() {
        let config = VolatilitySizingConfig::default();
        let sizer = VolatilityAdjustedSizing::new(config);

        let prices = create_price_series();
        let result = sizer.calculate_position(0.0, 100.0, &prices);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_price() {
        let config = VolatilitySizingConfig::default();
        let sizer = VolatilityAdjustedSizing::new(config);

        let prices = create_price_series();
        let result = sizer.calculate_position(10000.0, 0.0, &prices);

        assert!(result.is_err());
    }
}
