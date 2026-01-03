//! Feature Engineering Module
//!
//! Extracts features from OHLCV bar data for ML models.
//! Includes technical indicators, price-derived features, and lagged values.

use alphafield_core::Bar;
use serde::{Deserialize, Serialize};

/// Configuration for feature extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Lookback periods for lagged features
    pub lookback_periods: Vec<usize>,
    /// SMA periods to include
    pub sma_periods: Vec<usize>,
    /// RSI period (0 = disabled)
    pub rsi_period: usize,
    /// Include volume features
    pub include_volume: bool,
    /// Include volatility features
    pub include_volatility: bool,
    /// Prediction horizon in bars (for label generation)
    pub prediction_horizon: usize,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            lookback_periods: vec![1, 2, 3, 5, 10],
            sma_periods: vec![5, 10, 20, 50],
            rsi_period: 14,
            include_volume: true,
            include_volatility: true,
            prediction_horizon: 1,
        }
    }
}

/// Extracted feature set with labels for training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSet {
    /// Feature matrix: each row is a sample, each column is a feature
    pub features: Vec<Vec<f64>>,
    /// Target labels (future returns or direction)
    pub labels: Vec<f64>,
    /// Names of each feature column
    pub feature_names: Vec<String>,
    /// Timestamps corresponding to each sample
    pub timestamps: Vec<i64>,
}

impl FeatureSet {
    /// Returns the number of samples
    pub fn n_samples(&self) -> usize {
        self.features.len()
    }

    /// Returns the number of features
    pub fn n_features(&self) -> usize {
        self.feature_names.len()
    }

    /// Check if feature set is empty
    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }
}

/// Feature extractor that processes Bar data into ML features
#[derive(Debug, Clone)]
pub struct FeatureExtractor {
    config: FeatureConfig,
}

impl FeatureExtractor {
    /// Create a new feature extractor with the given configuration
    pub fn new(config: FeatureConfig) -> Self {
        Self { config }
    }

    /// Create a feature extractor with default configuration
    pub fn default_config() -> Self {
        Self::new(FeatureConfig::default())
    }

    /// Extract features from bar data
    ///
    /// Returns a FeatureSet with features and labels for each valid sample.
    /// The first `max_lookback` bars are used for warmup and not included as samples.
    pub fn extract(&self, bars: &[Bar]) -> FeatureSet {
        let max_lookback = self.max_lookback();
        let horizon = self.config.prediction_horizon;

        // Need enough data for lookback and horizon
        if bars.len() < max_lookback + horizon + 1 {
            return FeatureSet {
                features: vec![],
                labels: vec![],
                feature_names: self.feature_names(),
                timestamps: vec![],
            };
        }

        let mut features = Vec::new();
        let mut labels = Vec::new();
        let mut timestamps = Vec::new();

        // Precompute indicators for efficiency
        let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
        let volumes: Vec<f64> = bars.iter().map(|b| b.volume).collect();
        let returns = self.compute_returns(&closes);
        let smas = self.compute_smas(&closes);
        let rsi = if self.config.rsi_period > 0 {
            self.compute_rsi(&closes, self.config.rsi_period)
        } else {
            vec![]
        };
        let volatility = self.compute_volatility(&returns, 20);

        // Extract features for each valid index
        let end_idx = bars.len() - horizon;
        for i in max_lookback..end_idx {
            let mut row = Vec::new();

            // Price return features
            row.push(returns[i]);

            // Lagged returns
            for &lag in &self.config.lookback_periods {
                if i >= lag {
                    row.push(returns[i - lag]);
                } else {
                    row.push(0.0);
                }
            }

            // SMA ratio features (price / SMA - 1)
            for (period_idx, _period) in self.config.sma_periods.iter().enumerate() {
                if let Some(sma_values) = smas.get(period_idx) {
                    if let Some(&sma_val) = sma_values.get(i) {
                        if sma_val > 0.0 {
                            row.push(closes[i] / sma_val - 1.0);
                        } else {
                            row.push(0.0);
                        }
                    } else {
                        row.push(0.0);
                    }
                } else {
                    row.push(0.0);
                }
            }

            // RSI feature (normalized to 0-1)
            if !rsi.is_empty() && i < rsi.len() {
                row.push(rsi[i] / 100.0);
            }

            // Volume ratio (current / average)
            if self.config.include_volume && i >= 20 {
                let avg_vol: f64 = volumes[i - 20..i].iter().sum::<f64>() / 20.0;
                if avg_vol > 0.0 {
                    row.push(volumes[i] / avg_vol);
                } else {
                    row.push(1.0);
                }
            }

            // Volatility feature
            if self.config.include_volatility && i < volatility.len() {
                row.push(volatility[i]);
            }

            // High-low range (normalized)
            let range = (bars[i].high - bars[i].low) / bars[i].close;
            row.push(range);

            // Bar body ratio
            let body = (bars[i].close - bars[i].open).abs() / bars[i].close;
            row.push(body);

            // Label: future return (for regression) or direction (for classification)
            let future_price = bars[i + horizon].close;
            let label = (future_price - bars[i].close) / bars[i].close;

            features.push(row);
            labels.push(label);
            timestamps.push(bars[i].timestamp.timestamp());
        }

        FeatureSet {
            features,
            labels,
            feature_names: self.feature_names(),
            timestamps,
        }
    }

    /// Generate feature names based on configuration
    fn feature_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        names.push("return".to_string());

        for &lag in &self.config.lookback_periods {
            names.push(format!("return_lag_{}", lag));
        }

        for &period in &self.config.sma_periods {
            names.push(format!("sma_{}_ratio", period));
        }

        if self.config.rsi_period > 0 {
            names.push(format!("rsi_{}", self.config.rsi_period));
        }

        if self.config.include_volume {
            names.push("volume_ratio".to_string());
        }

        if self.config.include_volatility {
            names.push("volatility_20".to_string());
        }

        names.push("range_ratio".to_string());
        names.push("body_ratio".to_string());

        names
    }

    /// Calculate maximum lookback period needed
    fn max_lookback(&self) -> usize {
        let mut max = 0;

        // Lookback periods
        if let Some(&m) = self.config.lookback_periods.iter().max() {
            max = max.max(m);
        }

        // SMA periods
        if let Some(&m) = self.config.sma_periods.iter().max() {
            max = max.max(m);
        }

        // RSI period
        max = max.max(self.config.rsi_period);

        // Volatility window
        if self.config.include_volatility {
            max = max.max(21);
        }

        // Volume ratio window
        if self.config.include_volume {
            max = max.max(21);
        }

        max
    }

    /// Compute simple returns from close prices
    fn compute_returns(&self, closes: &[f64]) -> Vec<f64> {
        let mut returns = vec![0.0];
        for i in 1..closes.len() {
            if closes[i - 1] > 0.0 {
                returns.push((closes[i] - closes[i - 1]) / closes[i - 1]);
            } else {
                returns.push(0.0);
            }
        }
        returns
    }

    /// Compute SMAs for configured periods
    fn compute_smas(&self, closes: &[f64]) -> Vec<Vec<f64>> {
        self.config
            .sma_periods
            .iter()
            .map(|&period| self.compute_sma(closes, period))
            .collect()
    }

    /// Compute single SMA series
    fn compute_sma(&self, values: &[f64], period: usize) -> Vec<f64> {
        let mut sma = vec![0.0; values.len()];
        if period == 0 || values.len() < period {
            return sma;
        }

        let mut sum: f64 = values[..period].iter().sum();
        sma[period - 1] = sum / period as f64;

        for i in period..values.len() {
            sum = sum - values[i - period] + values[i];
            sma[i] = sum / period as f64;
        }

        sma
    }

    /// Compute RSI indicator
    fn compute_rsi(&self, closes: &[f64], period: usize) -> Vec<f64> {
        let mut rsi = vec![50.0; closes.len()];
        if period == 0 || closes.len() < period + 1 {
            return rsi;
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..closes.len() {
            let change = closes[i] - closes[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        // Calculate initial averages
        let mut avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
        let mut avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

        for i in period..gains.len() {
            avg_gain = (avg_gain * (period - 1) as f64 + gains[i]) / period as f64;
            avg_loss = (avg_loss * (period - 1) as f64 + losses[i]) / period as f64;

            if avg_loss > 0.0 {
                let rs = avg_gain / avg_loss;
                rsi[i + 1] = 100.0 - 100.0 / (1.0 + rs);
            } else {
                rsi[i + 1] = 100.0;
            }
        }

        rsi
    }

    /// Compute rolling volatility (standard deviation of returns)
    fn compute_volatility(&self, returns: &[f64], window: usize) -> Vec<f64> {
        let mut vol = vec![0.0; returns.len()];
        if window == 0 || returns.len() < window {
            return vol;
        }

        for i in window..returns.len() {
            let slice = &returns[i - window..i];
            let mean: f64 = slice.iter().sum::<f64>() / window as f64;
            let variance: f64 =
                slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / window as f64;
            vol[i] = variance.sqrt();
        }

        vol
    }
}

/// Convert regression labels to classification labels (-1, 0, 1)
pub fn to_direction_labels(returns: &[f64], threshold: f64) -> Vec<f64> {
    returns
        .iter()
        .map(|&r| {
            if r > threshold {
                1.0
            } else if r < -threshold {
                -1.0
            } else {
                0.0
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn make_test_bars(n: usize) -> Vec<Bar> {
        let start = Utc::now();
        (0..n)
            .map(|i| {
                let price = 100.0 + (i as f64) * 0.5 + (i as f64 * 0.1).sin() * 5.0;
                Bar {
                    timestamp: start + Duration::hours(i as i64),
                    open: price - 0.5,
                    high: price + 1.0,
                    low: price - 1.0,
                    close: price,
                    volume: 1000.0 + (i as f64) * 10.0,
                }
            })
            .collect()
    }

    #[test]
    fn test_feature_extraction_basic() {
        let bars = make_test_bars(100);
        let extractor = FeatureExtractor::default_config();
        let features = extractor.extract(&bars);

        assert!(!features.is_empty());
        assert!(!features.feature_names.is_empty());
        assert_eq!(features.features.len(), features.labels.len());
        assert_eq!(features.features.len(), features.timestamps.len());
    }

    #[test]
    fn test_feature_extraction_insufficient_data() {
        let bars = make_test_bars(10);
        let extractor = FeatureExtractor::default_config();
        let features = extractor.extract(&bars);

        assert!(features.is_empty());
    }

    #[test]
    fn test_direction_labels() {
        let returns = vec![0.05, -0.03, 0.001, -0.001, 0.02];
        let labels = to_direction_labels(&returns, 0.01);

        assert_eq!(labels, vec![1.0, -1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_feature_names_count() {
        let config = FeatureConfig::default();
        let extractor = FeatureExtractor::new(config.clone());
        let names = extractor.feature_names();

        // Count expected features
        let expected = 1  // return
            + config.lookback_periods.len()  // lagged returns
            + config.sma_periods.len()  // SMA ratios
            + 1  // RSI
            + 1  // volume ratio
            + 1  // volatility
            + 2; // range + body ratio

        assert_eq!(names.len(), expected);
    }
}
