//! Parameter Sensitivity Analysis Module
//!
//! Grid search over strategy parameters with heatmap generation
//! for identifying robust parameter regions.

use crate::engine::BacktestEngine;
use crate::exchange::SlippageModel;
use crate::metrics::PerformanceMetrics;
use crate::strategy::Strategy;
use alphafield_core::Bar;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for sensitivity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityConfig {
    /// Initial capital for each backtest
    pub initial_capital: f64,
    /// Fee rate
    pub fee_rate: f64,
    /// Whether to run in parallel (if available)
    pub parallel: bool,
}

impl Default for SensitivityConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10000.0,
            fee_rate: 0.001,
            parallel: true,
        }
    }
}

/// Parameter range definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterRange {
    /// Parameter name
    pub name: String,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Step size
    pub step: f64,
}

impl ParameterRange {
    pub fn new(name: &str, min: f64, max: f64, step: f64) -> Self {
        Self {
            name: name.to_string(),
            min,
            max,
            step,
        }
    }

    /// Generate all values in this range
    pub fn values(&self) -> Vec<f64> {
        let mut values = Vec::new();
        let mut current = self.min;
        while current <= self.max + f64::EPSILON {
            values.push(current);
            current += self.step;
        }
        values
    }
}

/// Result from a single parameter combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterResult {
    /// Parameter values used
    pub params: HashMap<String, f64>,
    /// Performance metrics from backtest
    pub metrics: PerformanceMetrics,
}

/// Aggregated sensitivity analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityResult {
    /// All parameter combination results
    pub results: Vec<ParameterResult>,
    /// Best parameters by Sharpe ratio
    pub best_sharpe: Option<ParameterResult>,
    /// Best parameters by total return
    pub best_return: Option<ParameterResult>,
    /// Best parameters by max drawdown (lowest)
    pub best_drawdown: Option<ParameterResult>,
    /// Heatmap data (2D parameter sweep results)
    pub heatmap: Option<HeatmapData>,
}

/// 2D heatmap data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapData {
    /// X-axis parameter name
    pub x_param: String,
    /// Y-axis parameter name
    pub y_param: String,
    /// X-axis values
    pub x_values: Vec<f64>,
    /// Y-axis values
    pub y_values: Vec<f64>,
    /// Sharpe ratio at each (x, y) combination
    pub sharpe_matrix: Vec<Vec<f64>>,
    /// Total return at each (x, y) combination
    pub return_matrix: Vec<Vec<f64>>,
    /// Max drawdown at each (x, y) combination
    pub drawdown_matrix: Vec<Vec<f64>>,
}

/// Sensitivity analyzer
pub struct SensitivityAnalyzer {
    config: SensitivityConfig,
}

impl SensitivityAnalyzer {
    pub fn new(config: SensitivityConfig) -> Self {
        Self { config }
    }

    /// Run 1D sensitivity analysis over a single parameter
    ///
    /// # Arguments
    /// * `data` - Historical bar data
    /// * `symbol` - Symbol name
    /// * `param` - Parameter range to sweep
    /// * `strategy_factory` - Function that creates strategy given parameter value
    pub fn analyze_1d<F>(
        &self,
        data: &[Bar],
        symbol: &str,
        param: &ParameterRange,
        strategy_factory: F,
    ) -> Result<SensitivityResult, String>
    where
        F: Fn(f64) -> Option<Box<dyn Strategy>>,
    {
        let mut results = Vec::new();

        for value in param.values() {
            if let Some(strategy) = strategy_factory(value) {
                let metrics = self.run_backtest(data, symbol, strategy)?;

                let mut params = HashMap::new();
                params.insert(param.name.clone(), value);

                results.push(ParameterResult { params, metrics });
            }
        }

        Ok(self.aggregate_results(results, None))
    }

    /// Run 2D sensitivity analysis over two parameters
    ///
    /// # Arguments
    /// * `data` - Historical bar data
    /// * `symbol` - Symbol name
    /// * `param_x` - First parameter range (X-axis)
    /// * `param_y` - Second parameter range (Y-axis)
    /// * `strategy_factory` - Function that creates strategy given (x, y) values
    pub fn analyze_2d<F>(
        &self,
        data: &[Bar],
        symbol: &str,
        param_x: &ParameterRange,
        param_y: &ParameterRange,
        strategy_factory: F,
    ) -> Result<SensitivityResult, String>
    where
        F: Fn(f64, f64) -> Option<Box<dyn Strategy>>,
    {
        let x_values = param_x.values();
        let y_values = param_y.values();

        let mut results = Vec::new();
        let mut sharpe_matrix = vec![vec![0.0; y_values.len()]; x_values.len()];
        let mut return_matrix = vec![vec![0.0; y_values.len()]; x_values.len()];
        let mut drawdown_matrix = vec![vec![0.0; y_values.len()]; x_values.len()];

        for (xi, &x_val) in x_values.iter().enumerate() {
            for (yi, &y_val) in y_values.iter().enumerate() {
                if let Some(strategy) = strategy_factory(x_val, y_val) {
                    let metrics = self.run_backtest(data, symbol, strategy)?;

                    sharpe_matrix[xi][yi] = metrics.sharpe_ratio;
                    return_matrix[xi][yi] = metrics.total_return;
                    drawdown_matrix[xi][yi] = metrics.max_drawdown;

                    let mut params = HashMap::new();
                    params.insert(param_x.name.clone(), x_val);
                    params.insert(param_y.name.clone(), y_val);

                    results.push(ParameterResult { params, metrics });
                }
            }
        }

        let heatmap = HeatmapData {
            x_param: param_x.name.clone(),
            y_param: param_y.name.clone(),
            x_values,
            y_values,
            sharpe_matrix,
            return_matrix,
            drawdown_matrix,
        };

        Ok(self.aggregate_results(results, Some(heatmap)))
    }

    /// Run a single backtest
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

    /// Aggregate results and find best parameters
    fn aggregate_results(
        &self,
        results: Vec<ParameterResult>,
        heatmap: Option<HeatmapData>,
    ) -> SensitivityResult {
        let best_sharpe = results
            .iter()
            .max_by(|a, b| {
                a.metrics
                    .sharpe_ratio
                    .partial_cmp(&b.metrics.sharpe_ratio)
                    .unwrap()
            })
            .cloned();

        let best_return = results
            .iter()
            .max_by(|a, b| {
                a.metrics
                    .total_return
                    .partial_cmp(&b.metrics.total_return)
                    .unwrap()
            })
            .cloned();

        let best_drawdown = results
            .iter()
            .min_by(|a, b| {
                a.metrics
                    .max_drawdown
                    .partial_cmp(&b.metrics.max_drawdown)
                    .unwrap()
            })
            .cloned();

        SensitivityResult {
            results,
            best_sharpe,
            best_return,
            best_drawdown,
            heatmap,
        }
    }

    /// Identify robust parameter regions (where nearby parameters also perform well)
    pub fn find_robust_regions(
        &self,
        result: &SensitivityResult,
        min_sharpe: f64,
    ) -> Vec<HashMap<String, f64>> {
        result
            .results
            .iter()
            .filter(|r| r.metrics.sharpe_ratio >= min_sharpe && r.metrics.max_drawdown < 0.20)
            .map(|r| r.params.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_range() {
        let range = ParameterRange::new("sma_period", 10.0, 30.0, 5.0);
        let values = range.values();

        assert_eq!(values, vec![10.0, 15.0, 20.0, 25.0, 30.0]);
    }

    #[test]
    fn test_parameter_range_fractional() {
        let range = ParameterRange::new("threshold", 0.0, 0.1, 0.02);
        let values = range.values();

        assert_eq!(values.len(), 6);
        assert!((values[0] - 0.0).abs() < 0.001);
        assert!((values[5] - 0.1).abs() < 0.001);
    }
}
