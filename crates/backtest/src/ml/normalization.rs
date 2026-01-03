//! Data Normalization Module
//!
//! Provides scalers for normalizing feature data before ML training.
//! Implements fit-transform pattern to prevent data leakage.

use serde::{Deserialize, Serialize};

/// Trait for data scalers
pub trait Scaler: Send + Sync {
    /// Fit the scaler to training data (compute statistics)
    fn fit(&mut self, data: &[Vec<f64>]);

    /// Transform data using fitted statistics
    fn transform(&self, data: &[Vec<f64>]) -> Vec<Vec<f64>>;

    /// Fit and transform in one step
    fn fit_transform(&mut self, data: &[Vec<f64>]) -> Vec<Vec<f64>> {
        self.fit(data);
        self.transform(data)
    }

    /// Inverse transform scaled data back to original scale
    fn inverse_transform(&self, data: &[Vec<f64>]) -> Vec<Vec<f64>>;

    /// Check if the scaler has been fitted
    fn is_fitted(&self) -> bool;
}

/// Standard scaler (z-score normalization)
///
/// Transforms data to have zero mean and unit variance:
/// z = (x - mean) / std
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardScaler {
    /// Mean for each feature
    means: Vec<f64>,
    /// Standard deviation for each feature
    stds: Vec<f64>,
    /// Whether the scaler has been fitted
    fitted: bool,
}

impl StandardScaler {
    /// Create a new unfitted StandardScaler
    pub fn new() -> Self {
        Self {
            means: vec![],
            stds: vec![],
            fitted: false,
        }
    }
}

impl Default for StandardScaler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scaler for StandardScaler {
    fn fit(&mut self, data: &[Vec<f64>]) {
        if data.is_empty() {
            return;
        }

        let n_features = data[0].len();
        let n_samples = data.len() as f64;

        self.means = vec![0.0; n_features];
        self.stds = vec![1.0; n_features];

        // Calculate means
        for row in data {
            for (j, &val) in row.iter().enumerate() {
                self.means[j] += val;
            }
        }
        for mean in &mut self.means {
            *mean /= n_samples;
        }

        // Calculate standard deviations
        for row in data {
            for (j, &val) in row.iter().enumerate() {
                self.stds[j] += (val - self.means[j]).powi(2);
            }
        }
        for std in &mut self.stds {
            *std = (*std / n_samples).sqrt();
            // Prevent division by zero
            if *std < 1e-10 {
                *std = 1.0;
            }
        }

        self.fitted = true;
    }

    fn transform(&self, data: &[Vec<f64>]) -> Vec<Vec<f64>> {
        if !self.fitted || data.is_empty() {
            return data.to_vec();
        }

        data.iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(j, &val)| (val - self.means[j]) / self.stds[j])
                    .collect()
            })
            .collect()
    }

    fn inverse_transform(&self, data: &[Vec<f64>]) -> Vec<Vec<f64>> {
        if !self.fitted || data.is_empty() {
            return data.to_vec();
        }

        data.iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(j, &val)| val * self.stds[j] + self.means[j])
                    .collect()
            })
            .collect()
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

/// Min-Max scaler
///
/// Transforms data to [0, 1] range:
/// x_scaled = (x - min) / (max - min)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinMaxScaler {
    /// Minimum value for each feature
    mins: Vec<f64>,
    /// Range (max - min) for each feature
    ranges: Vec<f64>,
    /// Whether the scaler has been fitted
    fitted: bool,
}

impl MinMaxScaler {
    /// Create a new unfitted MinMaxScaler
    pub fn new() -> Self {
        Self {
            mins: vec![],
            ranges: vec![],
            fitted: false,
        }
    }
}

impl Default for MinMaxScaler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scaler for MinMaxScaler {
    fn fit(&mut self, data: &[Vec<f64>]) {
        if data.is_empty() {
            return;
        }

        let n_features = data[0].len();
        self.mins = vec![f64::INFINITY; n_features];
        self.ranges = vec![1.0; n_features];

        let mut maxs = vec![f64::NEG_INFINITY; n_features];

        for row in data {
            for (j, &val) in row.iter().enumerate() {
                if val < self.mins[j] {
                    self.mins[j] = val;
                }
                if val > maxs[j] {
                    maxs[j] = val;
                }
            }
        }

        for ((range, max), min) in self
            .ranges
            .iter_mut()
            .zip(maxs.iter())
            .zip(self.mins.iter())
        {
            *range = max - min;
            // Prevent division by zero
            if *range < 1e-10 {
                *range = 1.0;
            }
        }

        self.fitted = true;
    }

    fn transform(&self, data: &[Vec<f64>]) -> Vec<Vec<f64>> {
        if !self.fitted || data.is_empty() {
            return data.to_vec();
        }

        data.iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(j, &val)| (val - self.mins[j]) / self.ranges[j])
                    .collect()
            })
            .collect()
    }

    fn inverse_transform(&self, data: &[Vec<f64>]) -> Vec<Vec<f64>> {
        if !self.fitted || data.is_empty() {
            return data.to_vec();
        }

        data.iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(j, &val)| val * self.ranges[j] + self.mins[j])
                    .collect()
            })
            .collect()
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_scaler_basic() {
        let data = vec![vec![1.0, 2.0], vec![2.0, 4.0], vec![3.0, 6.0]];

        let mut scaler = StandardScaler::new();
        let scaled = scaler.fit_transform(&data);

        assert!(scaler.is_fitted());
        assert_eq!(scaled.len(), 3);

        // Check that mean is approximately 0
        let mean0: f64 = scaled.iter().map(|r| r[0]).sum::<f64>() / 3.0;
        let mean1: f64 = scaled.iter().map(|r| r[1]).sum::<f64>() / 3.0;
        assert!((mean0).abs() < 1e-10);
        assert!((mean1).abs() < 1e-10);
    }

    #[test]
    fn test_standard_scaler_inverse() {
        let data = vec![vec![1.0, 2.0], vec![2.0, 4.0], vec![3.0, 6.0]];

        let mut scaler = StandardScaler::new();
        let scaled = scaler.fit_transform(&data);
        let unscaled = scaler.inverse_transform(&scaled);

        for (orig, restored) in data.iter().zip(unscaled.iter()) {
            for (o, r) in orig.iter().zip(restored.iter()) {
                assert!((o - r).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_minmax_scaler_basic() {
        let data = vec![vec![1.0, 10.0], vec![2.0, 20.0], vec![3.0, 30.0]];

        let mut scaler = MinMaxScaler::new();
        let scaled = scaler.fit_transform(&data);

        assert!(scaler.is_fitted());

        // First row should be [0, 0]
        assert!((scaled[0][0] - 0.0).abs() < 1e-10);
        assert!((scaled[0][1] - 0.0).abs() < 1e-10);

        // Last row should be [1, 1]
        assert!((scaled[2][0] - 1.0).abs() < 1e-10);
        assert!((scaled[2][1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_minmax_scaler_inverse() {
        let data = vec![vec![1.0, 10.0], vec![2.0, 20.0], vec![3.0, 30.0]];

        let mut scaler = MinMaxScaler::new();
        let scaled = scaler.fit_transform(&data);
        let unscaled = scaler.inverse_transform(&scaled);

        for (orig, restored) in data.iter().zip(unscaled.iter()) {
            for (o, r) in orig.iter().zip(restored.iter()) {
                assert!((o - r).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_scaler_empty_data() {
        let mut std_scaler = StandardScaler::new();
        std_scaler.fit(&[]);
        assert!(!std_scaler.is_fitted());

        let mut mm_scaler = MinMaxScaler::new();
        mm_scaler.fit(&[]);
        assert!(!mm_scaler.is_fitted());
    }

    #[test]
    fn test_scaler_constant_feature() {
        // All values are the same - should handle gracefully
        let data = vec![vec![5.0, 1.0], vec![5.0, 2.0], vec![5.0, 3.0]];

        let mut scaler = StandardScaler::new();
        let scaled = scaler.fit_transform(&data);

        // Constant feature should not cause NaN
        for row in &scaled {
            assert!(!row[0].is_nan());
        }
    }
}
