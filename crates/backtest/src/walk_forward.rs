//! Walk-Forward Analysis Module
//!
//! Rolling train/test optimization to validate strategy robustness
//! across different market regimes.

use crate::engine::BacktestEngine;
use crate::exchange::SlippageModel;
use crate::metrics::PerformanceMetrics;
use crate::strategy::Strategy;
use alphafield_core::Bar;
use serde::{Deserialize, Serialize};

/// Configuration for walk-forward analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardConfig {
    /// Number of bars in each training window
    pub train_window: usize,
    /// Number of bars in each test window
    pub test_window: usize,
    /// Step size (how many bars to advance between windows)
    pub step_size: usize,
    /// Initial capital for each window
    pub initial_capital: f64,
    /// Fee rate for simulated trading
    pub fee_rate: f64,
}

impl Default for WalkForwardConfig {
    fn default() -> Self {
        Self {
            train_window: 252, // ~1 year of daily bars
            test_window: 63,   // ~3 months
            step_size: 21,     // ~1 month advance
            initial_capital: 10000.0,
            fee_rate: 0.001,
        }
    }
}

/// Result from a single walk-forward window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowResult {
    /// Window index (0-based)
    pub window_index: usize,
    /// Start bar index for training period
    pub train_start: usize,
    /// End bar index for training period
    pub train_end: usize,
    /// Start bar index for test period
    pub test_start: usize,
    /// End bar index for test period
    pub test_end: usize,
    /// Metrics from training period
    pub train_metrics: PerformanceMetrics,
    /// Metrics from test period (out-of-sample)
    pub test_metrics: PerformanceMetrics,
}

/// Aggregated walk-forward analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkForwardResult {
    /// Results from each window
    pub windows: Vec<WindowResult>,
    /// Aggregate out-of-sample metrics
    pub aggregate_oos: AggregateMetrics,
    /// Parameter stability score (0-1, higher = more stable)
    pub stability_score: f64,
}

/// Aggregated metrics across all out-of-sample windows
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AggregateMetrics {
    pub mean_return: f64,
    pub median_return: f64,
    pub mean_sharpe: f64,
    pub worst_drawdown: f64,
    pub win_rate: f64, // % of windows with positive return
}

/// Walk-forward analysis engine
pub struct WalkForwardAnalyzer {
    config: WalkForwardConfig,
}

impl WalkForwardAnalyzer {
    pub fn new(config: WalkForwardConfig) -> Self {
        Self { config }
    }

    /// Run walk-forward analysis on historical data
    ///
    /// # Arguments
    /// * `data` - Historical bar data (must be sorted by time)
    /// * `symbol` - Symbol name
    /// * `strategy_factory` - Function that creates a fresh strategy instance
    ///
    /// # Returns
    /// Walk-forward results with all window metrics
    pub fn analyze<F>(
        &self,
        data: &[Bar],
        symbol: &str,
        strategy_factory: F,
    ) -> Result<WalkForwardResult, String>
    where
        F: Fn() -> Box<dyn Strategy>,
    {
        let total_bars = data.len();
        let min_required = self.config.train_window + self.config.test_window;

        if total_bars < min_required {
            return Err(format!(
                "Insufficient data: need {} bars, have {}",
                min_required, total_bars
            ));
        }

        let mut windows = Vec::new();
        let mut train_start = 0;

        loop {
            let train_end = train_start + self.config.train_window;
            let test_start = train_end;
            let test_end = test_start + self.config.test_window;

            // Check if we have enough data for this window
            if test_end > total_bars {
                break;
            }

            // Run training period
            let train_data: Vec<Bar> = data[train_start..train_end].to_vec();
            let train_metrics = self.run_backtest(&train_data, symbol, strategy_factory())?;

            // Run test period (out-of-sample)
            let test_data: Vec<Bar> = data[test_start..test_end].to_vec();
            let test_metrics = self.run_backtest(&test_data, symbol, strategy_factory())?;

            windows.push(WindowResult {
                window_index: windows.len(),
                train_start,
                train_end,
                test_start,
                test_end,
                train_metrics,
                test_metrics,
            });

            // Advance to next window
            train_start += self.config.step_size;
        }

        if windows.is_empty() {
            return Err("No complete windows could be formed".to_string());
        }

        // Calculate aggregate metrics
        let aggregate_oos = self.calculate_aggregate(&windows);
        let stability_score = self.calculate_stability(&windows);

        Ok(WalkForwardResult {
            windows,
            aggregate_oos,
            stability_score,
        })
    }

    /// Run a single backtest on a data slice
    fn run_backtest(
        &self,
        data: &[Bar],
        symbol: &str,
        strategy: Box<dyn Strategy>,
    ) -> Result<PerformanceMetrics, String> {
        let mut engine = BacktestEngine::new(
            self.config.initial_capital,
            self.config.fee_rate,
            SlippageModel::None,
        );

        engine.set_strategy(strategy);
        engine.add_data(symbol, data.to_vec());

        engine.run().map_err(|e| e.to_string())
    }

    /// Calculate aggregate metrics from all out-of-sample windows
    fn calculate_aggregate(&self, windows: &[WindowResult]) -> AggregateMetrics {
        if windows.is_empty() {
            return AggregateMetrics::default();
        }

        let returns: Vec<f64> = windows
            .iter()
            .map(|w| w.test_metrics.total_return)
            .collect();
        let sharpes: Vec<f64> = windows
            .iter()
            .map(|w| w.test_metrics.sharpe_ratio)
            .collect();
        let drawdowns: Vec<f64> = windows
            .iter()
            .map(|w| w.test_metrics.max_drawdown)
            .collect();

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;

        // Median return
        let mut sorted_returns = returns.clone();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        #[allow(clippy::manual_is_multiple_of)]
        let median_return = if sorted_returns.len() % 2 == 0 {
            let mid = sorted_returns.len() / 2;
            (sorted_returns[mid - 1] + sorted_returns[mid]) / 2.0
        } else {
            sorted_returns[sorted_returns.len() / 2]
        };

        let mean_sharpe = sharpes.iter().sum::<f64>() / sharpes.len() as f64;
        let worst_drawdown = drawdowns.iter().cloned().fold(0.0, f64::max);
        let win_rate = returns.iter().filter(|&&r| r > 0.0).count() as f64 / returns.len() as f64;

        AggregateMetrics {
            mean_return,
            median_return,
            mean_sharpe,
            worst_drawdown,
            win_rate,
        }
    }

    /// Calculate parameter stability score
    /// Higher score = more consistent performance between train and test
    fn calculate_stability(&self, windows: &[WindowResult]) -> f64 {
        if windows.is_empty() {
            return 0.0;
        }

        // Compare train vs test performance consistency
        let mut stability_scores = Vec::new();

        for window in windows {
            let train_return = window.train_metrics.total_return;
            let test_return = window.test_metrics.total_return;

            // If both positive or both negative, more stable
            let same_direction = (train_return >= 0.0) == (test_return >= 0.0);

            // Magnitude similarity (avoid division by zero)
            let magnitude_score = if train_return.abs() > 0.001 {
                let ratio = test_return / train_return;
                // Score: 1.0 if ratio is 1.0, decreasing as it deviates
                (1.0 / (1.0 + (ratio - 1.0).abs())).min(1.0)
            } else {
                0.5 // Unknown if train return near zero
            };

            let window_score = if same_direction {
                0.5 + 0.5 * magnitude_score
            } else {
                0.25 * magnitude_score
            };
            stability_scores.push(window_score);
        }

        stability_scores.iter().sum::<f64>() / stability_scores.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_bar(close: f64, idx: i64) -> Bar {
        Bar {
            timestamp: Utc::now() + chrono::Duration::days(idx),
            open: close * 0.99,
            high: close * 1.01,
            low: close * 0.98,
            close,
            volume: 1000.0,
        }
    }

    // TODO: Re-enable when BuyAndHold strategy is added to backtest module
    // #[test]
    // fn test_insufficient_data() {
    //     let config = WalkForwardConfig {
    //         train_window: 100,
    //         test_window: 50,
    //         ..Default::default()
    //     };
    //     let analyzer = WalkForwardAnalyzer::new(config);

    //     let data: Vec<Bar> = (0..100).map(|i| make_bar(100.0, i)).collect();

    //     // Create a simple buy-and-hold strategy factory
    //     let factory = || -> Box<dyn Strategy> {
    //         Box::new(crate::strategy::BuyAndHold::new())
    //     };

    //     let result = analyzer.analyze(&data, "TEST", factory);
    //     assert!(result.is_err());
    // }
    #[test]
    fn test_insufficient_data() {
        let config = WalkForwardConfig {
            train_window: 100,
            test_window: 50,
            ..Default::default()
        };
        let analyzer = WalkForwardAnalyzer::new(config);

        // Only 100 bars but need 150 (100 train + 50 test)
        let data: Vec<Bar> = (0..100).map(|i| make_bar(100.0, i)).collect();

        // Create a minimal strategy factory
        struct MinimalStrategy;
        impl crate::strategy::Strategy for MinimalStrategy {
            fn on_bar(
                &mut self,
                _bar: &Bar,
            ) -> crate::error::Result<Vec<crate::strategy::OrderRequest>> {
                Ok(vec![])
            }
        }

        let factory = || -> Box<dyn Strategy> { Box::new(MinimalStrategy) };

        let result = analyzer.analyze(&data, "TEST", factory);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient data"));
    }

    #[test]
    fn test_config_defaults() {
        let config = WalkForwardConfig::default();
        assert_eq!(config.train_window, 252);
        assert_eq!(config.test_window, 63);
        assert_eq!(config.step_size, 21);
    }

    #[test]
    fn test_aggregate_metrics_empty() {
        let config = WalkForwardConfig::default();
        let analyzer = WalkForwardAnalyzer::new(config);
        let aggregate = analyzer.calculate_aggregate(&[]);
        assert_eq!(aggregate.mean_return, 0.0);
        assert_eq!(aggregate.win_rate, 0.0);
    }
}
