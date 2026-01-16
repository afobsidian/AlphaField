//! Correlation & Diversification Analysis Module
//!
//! Provides tools for analyzing correlation between multiple assets or strategies
//! to assess diversification benefits and identify over-correlated portfolios.

use serde::{Deserialize, Serialize};

/// A correlation matrix showing pairwise correlations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    /// Labels for each row/column (asset or strategy names)
    pub labels: Vec<String>,
    /// NxN matrix of correlation values (-1.0 to 1.0)
    pub values: Vec<Vec<f64>>,
}

impl CorrelationMatrix {
    /// Get correlation between two items by index
    pub fn get(&self, i: usize, j: usize) -> Option<f64> {
        self.values.get(i).and_then(|row| row.get(j).copied())
    }

    /// Get correlation between two items by label
    pub fn get_by_label(&self, label_a: &str, label_b: &str) -> Option<f64> {
        let i = self.labels.iter().position(|l| l == label_a)?;
        let j = self.labels.iter().position(|l| l == label_b)?;
        self.get(i, j)
    }
}

/// Alert for high correlation between assets/strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationAlert {
    /// First item in correlated pair
    pub item_a: String,
    /// Second item in correlated pair
    pub item_b: String,
    /// Correlation value
    pub correlation: f64,
    /// Threshold that was exceeded
    pub threshold: f64,
}

/// Configuration for correlation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationConfig {
    /// Threshold above which to trigger alerts (0.0 - 1.0)
    pub alert_threshold: f64,
    /// Minimum number of data points required
    pub min_data_points: usize,
}

impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            alert_threshold: 0.7,
            min_data_points: 30,
        }
    }
}

/// Result from correlation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    /// The correlation matrix
    pub matrix: CorrelationMatrix,
    /// Any alerts triggered
    pub alerts: Vec<CorrelationAlert>,
    /// Average correlation (excluding diagonal)
    pub average_correlation: f64,
    /// Maximum correlation (excluding diagonal)
    pub max_correlation: f64,
    /// Diversification ratio (1 - avg correlation)
    pub diversification_score: f64,
}

/// Analyzer for computing correlations between equity curves or return series
pub struct CorrelationAnalyzer {
    config: CorrelationConfig,
}

impl CorrelationAnalyzer {
    pub fn new(config: CorrelationConfig) -> Self {
        Self { config }
    }

    /// Calculate Pearson correlation between two series
    fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.len() < 2 {
            return 0.0;
        }

        let n = x.len() as f64;
        let sum_x: f64 = x.iter().sum();
        let sum_y: f64 = y.iter().sum();
        let sum_xy: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
        let sum_x2: f64 = x.iter().map(|a| a * a).sum();
        let sum_y2: f64 = y.iter().map(|a| a * a).sum();

        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();

        if denominator == 0.0 {
            0.0
        } else {
            (numerator / denominator).clamp(-1.0, 1.0)
        }
    }

    /// Convert equity curves to return series
    fn equity_to_returns(equity: &[f64]) -> Vec<f64> {
        if equity.len() < 2 {
            return vec![];
        }

        equity
            .windows(2)
            .map(|w| {
                if w[0] != 0.0 {
                    (w[1] - w[0]) / w[0]
                } else {
                    0.0
                }
            })
            .collect()
    }

    /// Analyze correlation between multiple equity curves
    ///
    /// # Arguments
    /// * `curves` - List of (label, equity_values) pairs
    ///
    /// # Returns
    /// Correlation result with matrix and alerts
    pub fn analyze_equity_curves(
        &self,
        curves: &[(String, Vec<f64>)],
    ) -> Result<CorrelationResult, String> {
        if curves.len() < 2 {
            return Err("Need at least 2 curves for correlation analysis".to_string());
        }

        // Convert to returns
        let return_series: Vec<(String, Vec<f64>)> = curves
            .iter()
            .map(|(label, equity)| (label.clone(), Self::equity_to_returns(equity)))
            .collect();

        self.analyze_return_series(&return_series)
    }

    /// Analyze correlation between multiple return series
    ///
    /// # Arguments
    /// * `series` - List of (label, return_values) pairs
    ///
    /// # Returns
    /// Correlation result with matrix and alerts
    pub fn analyze_return_series(
        &self,
        series: &[(String, Vec<f64>)],
    ) -> Result<CorrelationResult, String> {
        if series.len() < 2 {
            return Err("Need at least 2 series for correlation analysis".to_string());
        }

        // Find minimum length
        let min_len = series.iter().map(|(_, s)| s.len()).min().unwrap_or(0);

        if min_len < self.config.min_data_points {
            return Err(format!(
                "Insufficient data: need {} points, have {}",
                self.config.min_data_points, min_len
            ));
        }

        let n = series.len();
        let labels: Vec<String> = series.iter().map(|(l, _)| l.clone()).collect();

        // Build correlation matrix
        let mut values = vec![vec![0.0; n]; n];
        let mut correlations = Vec::new();

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    values[i][j] = 1.0;
                } else if i < j {
                    let x: Vec<f64> = series[i].1.iter().take(min_len).copied().collect();
                    let y: Vec<f64> = series[j].1.iter().take(min_len).copied().collect();
                    let corr = Self::pearson_correlation(&x, &y);
                    values[i][j] = corr;
                    values[j][i] = corr;
                    correlations.push(corr.abs());
                }
            }
        }

        let matrix = CorrelationMatrix { labels, values };

        // Check for alerts
        let alerts = self.check_alerts(&matrix);

        // Calculate summary stats
        let average_correlation = if correlations.is_empty() {
            0.0
        } else {
            correlations.iter().sum::<f64>() / correlations.len() as f64
        };

        let max_correlation = correlations.iter().copied().fold(0.0, f64::max);

        let diversification_score = 1.0 - average_correlation;

        Ok(CorrelationResult {
            matrix,
            alerts,
            average_correlation,
            max_correlation,
            diversification_score,
        })
    }

    /// Check for pairs exceeding the correlation threshold
    pub fn check_alerts(&self, matrix: &CorrelationMatrix) -> Vec<CorrelationAlert> {
        let mut alerts = Vec::new();
        let n = matrix.labels.len();

        for i in 0..n {
            for j in (i + 1)..n {
                if let Some(corr) = matrix.get(i, j) {
                    if corr.abs() > self.config.alert_threshold {
                        alerts.push(CorrelationAlert {
                            item_a: matrix.labels[i].clone(),
                            item_b: matrix.labels[j].clone(),
                            correlation: corr,
                            threshold: self.config.alert_threshold,
                        });
                    }
                }
            }
        }

        alerts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pearson_correlation_perfect_positive() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let corr = CorrelationAnalyzer::pearson_correlation(&x, &y);
        assert!((corr - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_pearson_correlation_perfect_negative() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let corr = CorrelationAnalyzer::pearson_correlation(&x, &y);
        assert!((corr - (-1.0)).abs() < 1e-9);
    }

    #[test]
    fn test_pearson_correlation_zero() {
        // Orthogonal data
        let x = vec![1.0, 0.0, -1.0, 0.0];
        let y = vec![0.0, 1.0, 0.0, -1.0];
        let corr = CorrelationAnalyzer::pearson_correlation(&x, &y);
        assert!(corr.abs() < 0.1);
    }

    #[test]
    fn test_analyze_equity_curves() {
        let config = CorrelationConfig {
            alert_threshold: 0.7,
            min_data_points: 5,
        };
        let analyzer = CorrelationAnalyzer::new(config);

        // Two correlated equity curves
        let curves = vec![
            (
                "Strategy A".to_string(),
                vec![100.0, 102.0, 105.0, 103.0, 108.0, 110.0],
            ),
            (
                "Strategy B".to_string(),
                vec![100.0, 101.0, 104.0, 102.0, 106.0, 109.0],
            ),
        ];

        let result = analyzer.analyze_equity_curves(&curves).unwrap();

        assert_eq!(result.matrix.labels.len(), 2);
        assert!(result.matrix.get(0, 1).unwrap() > 0.8); // Highly correlated
        assert_eq!(result.matrix.get(0, 0).unwrap(), 1.0); // Self-correlation
    }

    #[test]
    fn test_correlation_alerts() {
        let config = CorrelationConfig {
            alert_threshold: 0.5,
            min_data_points: 5,
        };
        let analyzer = CorrelationAnalyzer::new(config);

        // Highly correlated curves should trigger alert
        let curves = vec![
            (
                "A".to_string(),
                vec![100.0, 110.0, 120.0, 130.0, 140.0, 150.0],
            ),
            (
                "B".to_string(),
                vec![100.0, 112.0, 124.0, 136.0, 148.0, 160.0],
            ),
        ];

        let result = analyzer.analyze_equity_curves(&curves).unwrap();
        assert!(!result.alerts.is_empty());
    }

    #[test]
    fn test_insufficient_curves() {
        let config = CorrelationConfig::default();
        let analyzer = CorrelationAnalyzer::new(config);

        let curves = vec![("Only One".to_string(), vec![100.0, 110.0])];
        let result = analyzer.analyze_equity_curves(&curves);
        assert!(result.is_err());
    }

    #[test]
    fn test_diversification_score_formula() {
        // Test diversification_score = 1.0 - average_correlation
        let config = CorrelationConfig {
            alert_threshold: 0.7,
            min_data_points: 4, // Lower threshold for this test
        };
        let analyzer = CorrelationAnalyzer::new(config);

        // Perfectly correlated (avg correlation = 1.0, diversification = 0.0)
        // Need 5 points to get 4 returns
        let curves = vec![
            ("A".to_string(), vec![100.0, 110.0, 120.0, 130.0, 140.0]),
            ("B".to_string(), vec![100.0, 110.0, 120.0, 130.0, 140.0]),
        ];
        let result = analyzer.analyze_equity_curves(&curves).unwrap();
        assert!(
            (result.diversification_score - 0.0).abs() < 0.1,
            "Perfectly correlated should have diversification score near 0.0"
        );

        // Uncorrelated (avg correlation ≈ 0.0, diversification ≈ 1.0)
        // Need 5 points to get 4 returns
        let curves = vec![
            ("A".to_string(), vec![100.0, 110.0, 90.0, 120.0, 105.0]),
            ("B".to_string(), vec![100.0, 90.0, 110.0, 80.0, 95.0]),
        ];
        let result = analyzer.analyze_equity_curves(&curves).unwrap();
        assert!(
            result.diversification_score > 0.0,
            "Uncorrelated should have positive diversification score"
        );
        assert!(
            result.diversification_score <= 1.0,
            "Diversification score should not exceed 1.0"
        );

        // Anti-correlated: diversification > 1.0 when correlation is negative
        // Note: Perfect anti-correlation in returns is difficult to construct
        // In practice, anti-correlated data will have diversification > 1.0
        // but exact values depend on the data characteristics
    }

    #[test]
    fn test_average_correlation_calculation() {
        let config = CorrelationConfig {
            alert_threshold: 0.7,
            min_data_points: 5, // Match the data we're using
        };
        let analyzer = CorrelationAnalyzer::new(config);

        // Three assets with known correlations
        // Need 6 points to get 5 returns (more than min_data_points)
        let curves = vec![
            (
                "A".to_string(),
                vec![100.0, 105.0, 110.0, 115.0, 120.0, 125.0],
            ),
            (
                "B".to_string(),
                vec![100.0, 106.0, 112.0, 118.0, 124.0, 130.0],
            ),
            (
                "C".to_string(),
                vec![100.0, 102.0, 104.0, 106.0, 108.0, 110.0],
            ),
        ];

        let result = analyzer.analyze_equity_curves(&curves).unwrap();

        // Should have 3 assets
        assert_eq!(result.matrix.labels.len(), 3);

        // Average correlation should exclude diagonal (1.0 for self-correlation)
        // Average correlation should exclude diagonal (1.0 for self-correlation)
        // For 3 assets, we have 6 off-diagonal pairs
        let sum_off_diagonal: f64 = result.matrix.values[0][1]
            + result.matrix.values[0][2]
            + result.matrix.values[1][0]
            + result.matrix.values[1][2]
            + result.matrix.values[2][0]
            + result.matrix.values[2][1];

        let expected_avg = sum_off_diagonal / 6.0;
        assert!(
            (result.average_correlation - expected_avg).abs() < 0.01,
            "Average correlation should match manual calculation"
        );
    }

    #[test]
    fn test_max_correlation_calculation() {
        let config = CorrelationConfig {
            alert_threshold: 0.7,
            min_data_points: 5, // Match the data we're using
        };
        let analyzer = CorrelationAnalyzer::new(config);

        // Mix of high and low correlations
        // Need 6 points to get 5 returns
        let curves = vec![
            (
                "HighCorr".to_string(),
                vec![100.0, 110.0, 120.0, 130.0, 140.0, 150.0],
            ),
            (
                "HighCorr2".to_string(),
                vec![100.0, 112.0, 124.0, 136.0, 148.0, 160.0],
            ),
            (
                "LowCorr".to_string(),
                vec![100.0, 90.0, 110.0, 80.0, 105.0, 95.0],
            ),
        ];

        let result = analyzer.analyze_equity_curves(&curves).unwrap();

        // Max correlation should be between the two highly correlated assets
        let high_corr = result.matrix.get_by_label("HighCorr", "HighCorr2").unwrap();
        let low_corr = result.matrix.get_by_label("HighCorr", "LowCorr").unwrap();

        assert_eq!(
            result.max_correlation,
            high_corr.abs(),
            "Max correlation should match the highest pairwise correlation"
        );
        assert!(
            high_corr.abs() > low_corr.abs(),
            "High correlation pair should have higher correlation than low correlation pair"
        );
    }

    #[test]
    fn test_alert_generation_with_different_thresholds() {
        // Test that different thresholds work correctly
        let curves = vec![
            (
                "HighCorr1".to_string(),
                vec![100.0, 110.0, 120.0, 130.0, 140.0, 150.0],
            ),
            (
                "HighCorr2".to_string(),
                vec![100.0, 112.0, 124.0, 136.0, 148.0, 160.0],
            ),
        ];

        // Very low threshold - should generate alerts
        let config = CorrelationConfig {
            alert_threshold: 0.1,
            min_data_points: 5,
        };
        let analyzer = CorrelationAnalyzer::new(config);
        let result = analyzer.analyze_equity_curves(&curves).unwrap();
        assert!(
            !result.alerts.is_empty(),
            "Should generate alerts with very low threshold"
        );

        // Test that alerts have correct structure
        if !result.alerts.is_empty() {
            let alert = &result.alerts[0];
            assert_eq!(alert.threshold, 0.1, "Alert should store threshold");
            assert!(!alert.item_a.is_empty(), "Alert should have item_a");
            assert!(!alert.item_b.is_empty(), "Alert should have item_b");
        }
    }

    #[test]
    fn test_single_asset_no_correlation() {
        let config = CorrelationConfig::default();
        let analyzer = CorrelationAnalyzer::new(config);

        // Two assets, but let's test the single asset case
        let curves = vec![("Solo".to_string(), vec![100.0, 110.0, 120.0, 130.0, 140.0])];
        let result = analyzer.analyze_equity_curves(&curves);

        // Should return an error for insufficient curves (need at least 2)
        assert!(result.is_err());
    }

    #[test]
    fn test_all_identical_assets() {
        let config = CorrelationConfig {
            alert_threshold: 0.7,
            min_data_points: 4, // Match the returns we'll get
        };
        let analyzer = CorrelationAnalyzer::new(config);

        // All assets are identical - maximum correlation
        // Need 5 points to get 4 returns (which matches min_data_points)
        let curves = vec![
            ("A".to_string(), vec![100.0, 110.0, 120.0, 130.0, 140.0]),
            ("B".to_string(), vec![100.0, 110.0, 120.0, 130.0, 140.0]),
            ("C".to_string(), vec![100.0, 110.0, 120.0, 130.0, 140.0]),
        ];

        let result = analyzer.analyze_equity_curves(&curves).unwrap();

        // All pairwise correlations should be 1.0
        assert!(
            (result.average_correlation - 1.0).abs() < 0.01,
            "Average correlation should be 1.0 for identical assets"
        );
        assert!(
            (result.max_correlation - 1.0).abs() < 0.01,
            "Max correlation should be 1.0 for identical assets"
        );
        assert!(
            (result.diversification_score - 0.0).abs() < 0.01,
            "Diversification score should be 0.0 for identical assets"
        );
    }

    #[test]
    fn test_insufficient_data_points() {
        let config = CorrelationConfig {
            alert_threshold: 0.7,
            min_data_points: 10, // Require 10 return points
        };
        let analyzer = CorrelationAnalyzer::new(config);

        // Only 5 equity points = 4 returns, but need 10
        let curves = vec![
            (
                "A".to_string(),
                vec![100.0, 110.0, 120.0, 130.0, 140.0, 150.0],
            ),
            (
                "B".to_string(),
                vec![100.0, 112.0, 124.0, 136.0, 148.0, 160.0],
            ),
        ];

        let result = analyzer.analyze_equity_curves(&curves);

        // Should fail due to insufficient data points
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient data"));
    }

    #[test]
    fn test_correlation_matrix_getters() {
        let matrix = CorrelationMatrix {
            labels: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            values: vec![
                vec![1.0, 0.5, 0.3],
                vec![0.5, 1.0, 0.4],
                vec![0.3, 0.4, 1.0],
            ],
        };

        // Test get by index
        assert_eq!(matrix.get(0, 1), Some(0.5));
        assert_eq!(matrix.get(1, 0), Some(0.5));
        assert_eq!(matrix.get(0, 0), Some(1.0));
        assert_eq!(matrix.get(3, 0), None); // Out of bounds

        // Test get by label
        assert_eq!(matrix.get_by_label("A", "B"), Some(0.5));
        assert_eq!(matrix.get_by_label("B", "A"), Some(0.5));
        assert_eq!(matrix.get_by_label("A", "C"), Some(0.3));
        assert_eq!(matrix.get_by_label("X", "Y"), None); // Invalid labels
    }

    #[test]
    fn test_negative_correlation() {
        // Test that we can handle anti-correlated data
        // Note: exact correlation values depend on data, so we just verify it runs
        let config = CorrelationConfig {
            alert_threshold: 0.7,
            min_data_points: 5,
        };
        let analyzer = CorrelationAnalyzer::new(config);

        let curves = vec![
            (
                "SeriesA".to_string(),
                vec![100.0, 110.0, 100.0, 110.0, 100.0, 110.0],
            ),
            (
                "SeriesB".to_string(),
                vec![100.0, 90.0, 100.0, 90.0, 100.0, 90.0],
            ),
        ];

        // Should not panic and return valid results
        let result = analyzer.analyze_equity_curves(&curves).unwrap();
        assert_eq!(result.matrix.labels.len(), 2);
        assert!(
            result.diversification_score >= 0.0,
            "Diversification score should be non-negative"
        );
    }

    #[test]
    fn test_correlation_alert_properties() {
        let config = CorrelationConfig {
            alert_threshold: 0.7,
            min_data_points: 5,
        };
        let analyzer = CorrelationAnalyzer::new(config);

        let curves = vec![
            (
                "A".to_string(),
                vec![100.0, 110.0, 120.0, 130.0, 140.0, 150.0],
            ),
            (
                "B".to_string(),
                vec![100.0, 112.0, 124.0, 136.0, 148.0, 160.0],
            ),
        ];

        let result = analyzer.analyze_equity_curves(&curves).unwrap();

        assert!(
            !result.alerts.is_empty(),
            "Should generate an alert for highly correlated data"
        );

        let alert = &result.alerts[0];
        assert_eq!(alert.threshold, 0.7, "Alert should store the threshold");
        assert!(
            alert.correlation.abs() > alert.threshold,
            "Alert correlation should exceed threshold"
        );
        assert_eq!(
            result.alerts.len(),
            1,
            "Should only have one alert for two assets"
        );
    }
}
