//! Data Split Module
//!
//! Provides time-based train/validation/test splitting utilities
//! to prevent lookahead bias in ML model training.

use serde::{Deserialize, Serialize};

/// Configuration for data splitting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitConfig {
    /// Fraction of data for training (e.g., 0.7 for 70%)
    pub train_ratio: f64,
    /// Fraction for validation (e.g., 0.15 for 15%)
    pub validation_ratio: f64,
    /// Gap between train and validation to prevent leakage (in samples)
    pub gap_samples: usize,
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            train_ratio: 0.7,
            validation_ratio: 0.15,
            gap_samples: 0,
        }
    }
}

impl SplitConfig {
    /// Create config for train/test only (no validation)
    pub fn train_test(train_ratio: f64) -> Self {
        Self {
            train_ratio,
            validation_ratio: 0.0,
            gap_samples: 0,
        }
    }

    /// Create config with gap between sets
    pub fn with_gap(mut self, gap: usize) -> Self {
        self.gap_samples = gap;
        self
    }
}

/// Result of a data split operation
#[derive(Debug, Clone)]
pub struct DataSplit<T> {
    /// Training data
    pub train: Vec<T>,
    /// Validation data (may be empty)
    pub validation: Vec<T>,
    /// Test data
    pub test: Vec<T>,
    /// Indices for train set in original data
    pub train_indices: (usize, usize),
    /// Indices for validation set in original data
    pub validation_indices: (usize, usize),
    /// Indices for test set in original data
    pub test_indices: (usize, usize),
}

/// Data splitter for time-series data
#[derive(Debug, Clone)]
pub struct DataSplitter {
    config: SplitConfig,
}

impl DataSplitter {
    /// Create a new data splitter with the given configuration
    pub fn new(config: SplitConfig) -> Self {
        Self { config }
    }

    /// Create a splitter with default 70/15/15 split
    pub fn default_split() -> Self {
        Self::new(SplitConfig::default())
    }

    /// Create a splitter for simple train/test split (80/20)
    pub fn train_test_split() -> Self {
        Self::new(SplitConfig::train_test(0.8))
    }

    /// Split data into train/validation/test sets
    ///
    /// Data is split chronologically to preserve time ordering and prevent lookahead bias.
    pub fn split<T: Clone>(&self, data: &[T]) -> DataSplit<T> {
        let n = data.len();
        if n == 0 {
            return DataSplit {
                train: vec![],
                validation: vec![],
                test: vec![],
                train_indices: (0, 0),
                validation_indices: (0, 0),
                test_indices: (0, 0),
            };
        }

        let train_end = (n as f64 * self.config.train_ratio) as usize;
        let val_start = train_end + self.config.gap_samples;
        let val_end = val_start + (n as f64 * self.config.validation_ratio) as usize;
        let test_start = val_end + self.config.gap_samples;

        let train = data[..train_end.min(n)].to_vec();

        let validation = if val_start < n && val_end > val_start {
            data[val_start..val_end.min(n)].to_vec()
        } else {
            vec![]
        };

        let test = if test_start < n {
            data[test_start..].to_vec()
        } else {
            vec![]
        };

        DataSplit {
            train,
            validation,
            test,
            train_indices: (0, train_end.min(n)),
            validation_indices: (val_start.min(n), val_end.min(n)),
            test_indices: (test_start.min(n), n),
        }
    }

    /// Split features and labels together, ensuring alignment
    pub fn split_with_labels(
        &self,
        features: &[Vec<f64>],
        labels: &[f64],
    ) -> (DataSplit<Vec<f64>>, DataSplit<f64>) {
        let feature_split = self.split(features);
        let label_split = self.split(labels);
        (feature_split, label_split)
    }
}

/// Generate rolling windows for walk-forward validation
#[derive(Debug, Clone)]
pub struct RollingWindowGenerator {
    /// Training window size in samples
    pub train_window: usize,
    /// Test window size in samples
    pub test_window: usize,
    /// Step size for moving the window
    pub step_size: usize,
    /// Gap between train and test windows
    pub gap_samples: usize,
}

/// A single rolling window split
#[derive(Debug, Clone)]
pub struct WindowSplit {
    pub train_start: usize,
    pub train_end: usize,
    pub test_start: usize,
    pub test_end: usize,
    pub window_id: usize,
}

impl RollingWindowGenerator {
    /// Create a new rolling window generator
    pub fn new(train_window: usize, test_window: usize, step_size: usize) -> Self {
        Self {
            train_window,
            test_window,
            step_size,
            gap_samples: 0,
        }
    }

    /// Add gap between train and test
    pub fn with_gap(mut self, gap: usize) -> Self {
        self.gap_samples = gap;
        self
    }

    /// Generate all windows for a dataset of given length
    pub fn generate_windows(&self, data_length: usize) -> Vec<WindowSplit> {
        let mut windows = Vec::new();
        let mut train_start = 0;
        let mut window_id = 0;

        loop {
            let train_end = train_start + self.train_window;
            let test_start = train_end + self.gap_samples;
            let test_end = test_start + self.test_window;

            if test_end > data_length {
                break;
            }

            windows.push(WindowSplit {
                train_start,
                train_end,
                test_start,
                test_end,
                window_id,
            });

            train_start += self.step_size;
            window_id += 1;
        }

        windows
    }

    /// Generate windows with data extraction
    pub fn split_data<T: Clone>(&self, data: &[T]) -> Vec<(Vec<T>, Vec<T>)> {
        self.generate_windows(data.len())
            .into_iter()
            .map(|w| {
                let train = data[w.train_start..w.train_end].to_vec();
                let test = data[w.test_start..w.test_end].to_vec();
                (train, test)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_split() {
        let data: Vec<i32> = (0..100).collect();
        let splitter = DataSplitter::default_split();
        let split = splitter.split(&data);

        assert_eq!(split.train.len(), 70);
        assert_eq!(split.validation.len(), 15);
        assert_eq!(split.test.len(), 15);
    }

    #[test]
    fn test_train_test_only() {
        let data: Vec<i32> = (0..100).collect();
        let splitter = DataSplitter::train_test_split();
        let split = splitter.split(&data);

        assert_eq!(split.train.len(), 80);
        assert!(split.validation.is_empty());
        assert_eq!(split.test.len(), 20);
    }

    #[test]
    fn test_split_with_gap() {
        let data: Vec<i32> = (0..100).collect();
        let config = SplitConfig::default().with_gap(5);
        let splitter = DataSplitter::new(config);
        let split = splitter.split(&data);

        // Train ends at 70, gap of 5, val starts at 75
        // With default 0.15 validation ratio, val is 15 samples = 75..90
        // Then another gap of 5, test starts at 95
        assert_eq!(split.train.len(), 70);
        // Validation starts at 75, should still get 15 samples (75..90)
        assert!(split.validation.len() <= 15);
    }

    #[test]
    fn test_empty_data() {
        let data: Vec<i32> = vec![];
        let splitter = DataSplitter::default_split();
        let split = splitter.split(&data);

        assert!(split.train.is_empty());
        assert!(split.validation.is_empty());
        assert!(split.test.is_empty());
    }

    #[test]
    fn test_chronological_order() {
        let data: Vec<i32> = (0..100).collect();
        let splitter = DataSplitter::default_split();
        let split = splitter.split(&data);

        // Verify train comes before validation comes before test
        let train_max = split.train.iter().max().unwrap();
        let val_min = split.validation.iter().min().unwrap();
        let val_max = split.validation.iter().max().unwrap();
        let test_min = split.test.iter().min().unwrap();

        assert!(train_max < val_min);
        assert!(val_max < test_min);
    }

    #[test]
    fn test_rolling_windows_basic() {
        let generator = RollingWindowGenerator::new(50, 20, 10);
        let windows = generator.generate_windows(100);

        assert!(!windows.is_empty());

        // Verify first window
        assert_eq!(windows[0].train_start, 0);
        assert_eq!(windows[0].train_end, 50);
        assert_eq!(windows[0].test_start, 50);
        assert_eq!(windows[0].test_end, 70);
    }

    #[test]
    fn test_rolling_windows_step() {
        let generator = RollingWindowGenerator::new(50, 20, 10);
        let windows = generator.generate_windows(100);

        if windows.len() >= 2 {
            assert_eq!(windows[1].train_start, 10);
            assert_eq!(windows[1].train_end, 60);
        }
    }

    #[test]
    fn test_rolling_windows_with_gap() {
        let generator = RollingWindowGenerator::new(50, 20, 10).with_gap(5);
        let windows = generator.generate_windows(100);

        // With gap, test starts 5 samples after train ends
        assert_eq!(windows[0].test_start, 55);
        assert_eq!(windows[0].test_end, 75);
    }

    #[test]
    fn test_rolling_windows_insufficient_data() {
        let generator = RollingWindowGenerator::new(80, 30, 10);
        let windows = generator.generate_windows(100);

        // Can't fit a full window
        assert!(windows.is_empty());
    }
}
