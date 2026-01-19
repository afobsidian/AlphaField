//! Comprehensive Optimization Workflow Module
//!
//! This module provides an integrated optimization pipeline that combines:
//! - Grid search parameter optimization
//! - Walk-forward validation
//! - Parameter dispersion analysis
//! - Sensitivity analysis for 3D visualization
//!
//! The workflow is designed to avoid overfitting by providing robust validation
//! and stability metrics alongside optimization results.

use crate::engine::BacktestEngine;
use crate::exchange::SlippageModel;
use crate::metrics::PerformanceMetrics;
use crate::monte_carlo::{
    MonteCarloConfig, MonteCarloResult, MonteCarloSimulator, Trade as McTrade,
};
use crate::optimizer::{OptimizationResult, ParamBounds, ParameterOptimizer};
use crate::sensitivity::{
    ParameterRange, SensitivityAnalyzer, SensitivityConfig, SensitivityResult,
};
use crate::strategy::Strategy;
use crate::trade::Trade;
use crate::walk_forward::{WalkForwardAnalyzer, WalkForwardConfig, WalkForwardResult};
use alphafield_core::Bar;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// Configuration for the complete optimization workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Initial capital for all backtests
    pub initial_capital: f64,
    /// Fee rate for simulated trading
    pub fee_rate: f64,
    /// Slippage model
    pub slippage: SlippageModel,
    /// Walk-forward validation configuration
    pub walk_forward_config: WalkForwardConfig,
    /// Whether to run 3D sensitivity analysis (can be expensive)
    pub include_3d_sensitivity: bool,
    /// Data split ratio for in-sample/out-of-sample (default: 0.70)
    pub train_test_split_ratio: f64,
    /// Monte Carlo simulation configuration (optional, None = skip)
    pub monte_carlo_config: Option<MonteCarloConfig>,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            initial_capital: 100_000.0,
            fee_rate: 0.001,
            slippage: SlippageModel::FixedPercent(0.0005),
            walk_forward_config: WalkForwardConfig::default(),
            include_3d_sensitivity: true,
            train_test_split_ratio: 0.70,
            monte_carlo_config: Some(MonteCarloConfig::default()),
        }
    }
}

// Constants for parameter dispersion calculations
const MIN_MEAN_THRESHOLD: f64 = 0.001;

/// Parameter dispersion statistics
///
/// These metrics help identify how sensitive the strategy is to parameter changes,
/// which is crucial for avoiding overfitting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDispersion {
    /// Standard deviation of Sharpe ratios across all parameter combinations
    pub sharpe_std: f64,
    /// Standard deviation of returns across all parameter combinations
    pub return_std: f64,
    /// Coefficient of variation (std/mean) for Sharpe ratio
    pub sharpe_cv: f64,
    /// Coefficient of variation for returns
    pub return_cv: f64,
    /// Percentage of parameter combinations with positive Sharpe ratio
    pub positive_sharpe_pct: f64,
    /// Percentage of parameter combinations with positive returns
    pub positive_return_pct: f64,
    /// Range of Sharpe ratios (max - min)
    pub sharpe_range: f64,
    /// Range of returns (max - min)
    pub return_range: f64,
}

impl ParameterDispersion {
    /// Calculate dispersion statistics from optimization sweep results
    pub fn calculate(sweep_results: &[crate::optimizer::ParamSweepResult]) -> Self {
        if sweep_results.is_empty() {
            return Self::default();
        }

        let sharpe_values: Vec<f64> = sweep_results.iter().map(|r| r.sharpe).collect();
        let return_values: Vec<f64> = sweep_results.iter().map(|r| r.total_return).collect();

        let sharpe_mean = sharpe_values.iter().sum::<f64>() / sharpe_values.len() as f64;
        let return_mean = return_values.iter().sum::<f64>() / return_values.len() as f64;

        let sharpe_variance = sharpe_values
            .iter()
            .map(|x| (x - sharpe_mean).powi(2))
            .sum::<f64>()
            / sharpe_values.len() as f64;
        let return_variance = return_values
            .iter()
            .map(|x| (x - return_mean).powi(2))
            .sum::<f64>()
            / return_values.len() as f64;

        let sharpe_std = sharpe_variance.sqrt();
        let return_std = return_variance.sqrt();

        let sharpe_cv = if sharpe_mean.abs() > MIN_MEAN_THRESHOLD {
            sharpe_std / sharpe_mean.abs()
        } else {
            f64::INFINITY
        };
        let return_cv = if return_mean.abs() > MIN_MEAN_THRESHOLD {
            return_std / return_mean.abs()
        } else {
            f64::INFINITY
        };

        let positive_sharpe = sharpe_values.iter().filter(|&&x| x > 0.0).count() as f64;
        let positive_return = return_values.iter().filter(|&&x| x > 0.0).count() as f64;

        let sharpe_min = sharpe_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let sharpe_max = sharpe_values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let return_min = return_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let return_max = return_values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);

        Self {
            sharpe_std,
            return_std,
            sharpe_cv,
            return_cv,
            positive_sharpe_pct: (positive_sharpe / sharpe_values.len() as f64) * 100.0,
            positive_return_pct: (positive_return / return_values.len() as f64) * 100.0,
            sharpe_range: sharpe_max - sharpe_min,
            return_range: return_max - return_min,
        }
    }
}

impl Default for ParameterDispersion {
    fn default() -> Self {
        Self {
            sharpe_std: 0.0,
            return_std: 0.0,
            sharpe_cv: 0.0,
            return_cv: 0.0,
            positive_sharpe_pct: 0.0,
            positive_return_pct: 0.0,
            sharpe_range: 0.0,
            return_range: 0.0,
        }
    }
}

/// Complete optimization workflow result
///
/// Contains all outputs from the optimization pipeline including
/// optimization results, validation metrics, and dispersion statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    /// Grid search optimization results
    pub optimization: OptimizationResult,
    /// Walk-forward validation results
    pub walk_forward_validation: WalkForwardResult,
    /// Parameter dispersion statistics
    pub parameter_dispersion: ParameterDispersion,
    /// 3D sensitivity analysis (optional, can be expensive)
    pub sensitivity_3d: Option<SensitivityResult>,
    /// Monte Carlo simulation (optional, tests robustness to trade sequence randomness)
    pub monte_carlo: Option<MonteCarloResult>,
    /// In-sample performance with optimized parameters
    pub in_sample_metrics: PerformanceMetrics,
    /// Robustness score (0-100, higher is better)
    /// Combines walk-forward stability, dispersion, and in-sample/out-of-sample consistency
    pub robustness_score: f64,
}

/// Optimization workflow orchestrator
pub struct OptimizationWorkflow {
    config: WorkflowConfig,
}

impl OptimizationWorkflow {
    pub fn new(config: WorkflowConfig) -> Self {
        Self { config }
    }

    /// Run the complete optimization workflow
    ///
    /// # Arguments
    /// * `data` - Historical bar data (will be split for in-sample/out-of-sample)
    /// * `symbol` - Trading symbol
    /// * `strategy_factory` - Function that creates a strategy from parameters
    /// * `bounds` - Parameter bounds for grid search
    /// * `sensitivity_params` - Optional parameters for 3D sensitivity (2 params max)
    ///
    /// # Returns
    /// Complete workflow result with optimization, validation, and dispersion data
    pub fn run<F>(
        &self,
        data: &[Bar],
        symbol: &str,
        strategy_factory: &F,
        bounds: &[ParamBounds],
        sensitivity_params: Option<(ParameterRange, ParameterRange)>,
    ) -> Result<WorkflowResult, String>
    where
        F: Fn(&HashMap<String, f64>) -> Option<Box<dyn Strategy>>,
    {
        info!("Starting comprehensive optimization workflow");

        if data.is_empty() {
            return Err("No data provided for optimization workflow".to_string());
        }

        if bounds.is_empty() {
            return Err("No parameter bounds provided".to_string());
        }

        // Split data: configurable in-sample/out-of-sample ratio
        let split_idx = (data.len() as f64 * self.config.train_test_split_ratio) as usize;
        let in_sample_data = &data[..split_idx];
        let out_of_sample_data = &data[split_idx..];

        info!(
            total_bars = data.len(),
            in_sample = in_sample_data.len(),
            out_of_sample = out_of_sample_data.len(),
            "Data split for optimization"
        );

        // Phase 1: Grid Search Optimization on in-sample data
        info!("Phase 1: Running grid search optimization");
        let optimizer = ParameterOptimizer::new(self.config.initial_capital, self.config.fee_rate);
        let optimization_result = optimizer.optimize(
            in_sample_data,
            symbol,
            |params| strategy_factory(params),
            bounds,
        )?;

        info!(
            best_sharpe = optimization_result.best_sharpe,
            best_return = optimization_result.best_return * 100.0,
            iterations = optimization_result.iterations_tested,
            "Grid search optimization complete"
        );

        // Phase 2: Calculate Parameter Dispersion
        info!("Phase 2: Calculating parameter dispersion statistics");
        let parameter_dispersion = ParameterDispersion::calculate(&optimization_result.all_results);

        info!(
            sharpe_cv = parameter_dispersion.sharpe_cv,
            positive_sharpe_pct = parameter_dispersion.positive_sharpe_pct,
            "Parameter dispersion calculated"
        );

        // Phase 3: Walk-Forward Validation on full dataset
        info!("Phase 3: Running walk-forward validation");
        let walk_forward_analyzer =
            WalkForwardAnalyzer::new(self.config.walk_forward_config.clone());

        // Create strategy factory that uses optimized parameters
        let optimized_params = optimization_result.best_params.clone();
        let wf_factory = || -> Box<dyn Strategy> {
            strategy_factory(&optimized_params)
                .expect("Failed to create strategy with optimized parameters")
        };

        let walk_forward_result = walk_forward_analyzer.analyze(data, symbol, wf_factory)?;

        info!(
            windows = walk_forward_result.windows.len(),
            mean_oos_return = walk_forward_result.aggregate_oos.mean_return * 100.0,
            stability_score = walk_forward_result.stability_score,
            "Walk-forward validation complete"
        );

        // Phase 4: 3D Sensitivity Analysis (optional)
        let sensitivity_3d = if self.config.include_3d_sensitivity {
            if let Some((param_x, param_y)) = sensitivity_params {
                info!("Phase 4: Running 3D sensitivity analysis");

                let sens_config = SensitivityConfig {
                    initial_capital: self.config.initial_capital,
                    fee_rate: self.config.fee_rate,
                    parallel: true,
                };
                let analyzer = SensitivityAnalyzer::new(sens_config);

                // Create factory for sensitivity that varies two parameters
                let fixed_params = optimization_result.best_params.clone();
                let x_name = param_x.name.clone();
                let y_name = param_y.name.clone();

                let sens_factory = |x_val: f64, y_val: f64| -> Option<Box<dyn Strategy>> {
                    let mut params = fixed_params.clone();
                    params.insert(x_name.clone(), x_val);
                    params.insert(y_name.clone(), y_val);
                    strategy_factory(&params)
                };

                match analyzer.analyze_2d(in_sample_data, symbol, &param_x, &param_y, sens_factory)
                {
                    Ok(result) => {
                        info!("3D sensitivity analysis complete");
                        Some(result)
                    }
                    Err(e) => {
                        warn!(error = %e, "3D sensitivity analysis failed");
                        None
                    }
                }
            } else {
                info!("Skipping 3D sensitivity analysis (no parameters specified)");
                None
            }
        } else {
            info!("Skipping 3D sensitivity analysis (disabled in config)");
            None
        };

        // Phase 5: Calculate in-sample metrics with optimized parameters
        info!("Phase 5: Calculating in-sample performance metrics");
        let (in_sample_metrics, trades) = self.run_backtest(
            in_sample_data,
            symbol,
            &optimization_result.best_params,
            strategy_factory,
        )?;

        // Phase 5b: Monte Carlo simulation (optional)
        let monte_carlo = if let Some(mc_config) = &self.config.monte_carlo_config {
            if !trades.is_empty() {
                info!("Phase 5b: Running Monte Carlo simulation");

                // Convert backtest trades to Monte Carlo trades
                let mc_trades: Vec<McTrade> = trades
                    .iter()
                    .map(|t| McTrade {
                        symbol: t.symbol.clone(),
                        pnl: t.pnl,
                        return_pct: t.return_pct(),
                        duration: (t.duration_secs / 3600) as usize, // Convert seconds to hours/bars
                    })
                    .collect();

                let simulator = MonteCarloSimulator::new(mc_config.clone());
                let result = simulator.simulate(&mc_trades);

                info!(
                    simulations = result.num_simulations,
                    prob_profit = result.probability_of_profit * 100.0,
                    "Monte Carlo simulation complete"
                );

                Some(result)
            } else {
                warn!("No trades found for Monte Carlo simulation");
                None
            }
        } else {
            info!("Skipping Monte Carlo simulation (not configured)");
            None
        };

        // Phase 6: Calculate robustness score
        let robustness_score = self.calculate_robustness_score(
            &parameter_dispersion,
            &walk_forward_result,
            &in_sample_metrics,
            &monte_carlo,
        );

        info!(
            robustness_score = robustness_score,
            "Optimization workflow complete"
        );

        Ok(WorkflowResult {
            optimization: optimization_result,
            walk_forward_validation: walk_forward_result,
            parameter_dispersion,
            sensitivity_3d,
            monte_carlo,
            in_sample_metrics,
            robustness_score,
        })
    }

    /// Run a single backtest with given parameters
    fn run_backtest<F>(
        &self,
        data: &[Bar],
        symbol: &str,
        params: &HashMap<String, f64>,
        strategy_factory: &F,
    ) -> Result<(PerformanceMetrics, Vec<Trade>), String>
    where
        F: Fn(&HashMap<String, f64>) -> Option<Box<dyn Strategy>>,
    {
        let strategy = strategy_factory(params)
            .ok_or_else(|| "Failed to create strategy with parameters".to_string())?;

        let mut engine = BacktestEngine::new(
            self.config.initial_capital,
            self.config.fee_rate,
            self.config.slippage.clone(),
        );

        engine.add_data(symbol, data.to_vec());
        engine.set_strategy(strategy);

        engine.run().map_err(|e| e.to_string())?;

        // Extract trades from portfolio
        let trades = engine.portfolio.trades.clone();

        // Calculate performance metrics
        let metrics = PerformanceMetrics::calculate_with_trades(
            &engine.portfolio.equity_history,
            &trades,
            0.02, // Default 2% risk-free rate
        );

        Ok((metrics, trades))
    }

    /// Calculate overall robustness score (0-100)
    ///
    /// Combines multiple factors with configurable weights:
    /// - Walk-forward stability score (30%)
    /// - Parameter dispersion score (30% - inverse of CV, lower CV is better)
    /// - Percentage of positive parameter combinations (20%)
    /// - Out-of-sample win rate from walk-forward (20%)
    fn calculate_robustness_score(
        &self,
        dispersion: &ParameterDispersion,
        walk_forward: &WalkForwardResult,
        _in_sample: &PerformanceMetrics,
        monte_carlo: &Option<MonteCarloResult>,
    ) -> f64 {
        // Robustness score component weights
        const WEIGHT_WF_STABILITY: f64 = 0.25;
        const WEIGHT_DISPERSION: f64 = 0.25;
        const WEIGHT_POSITIVE_COMBOS: f64 = 0.15;
        const WEIGHT_OOS_WIN_RATE: f64 = 0.15;
        const WEIGHT_MONTE_CARLO: f64 = 0.20;

        // Component 1: Walk-forward stability (0-1 scale, higher is better)
        let wf_score = walk_forward.stability_score;

        // Component 2: Parameter dispersion score (inverse of CV, capped)
        // Lower CV means more robust (less sensitive to parameter changes)
        let cv_score = if dispersion.sharpe_cv.is_finite() {
            // Normalize: CV of 0.5 = score 0.5, CV of 0.2 = score 0.8, CV of 1.0 = score 0.2
            (1.0 / (1.0 + dispersion.sharpe_cv)).min(1.0)
        } else {
            0.0
        };

        // Component 3: Positive combinations percentage (0-1 scale)
        let positive_score = dispersion.positive_sharpe_pct / 100.0;

        // Component 4: Out-of-sample win rate from walk-forward
        let oos_win_rate = walk_forward.aggregate_oos.win_rate;

        // Component 5: Monte Carlo robustness (probability of profit and drawdown resilience)
        let mc_score = if let Some(mc) = monte_carlo {
            if mc.num_simulations > 0 {
                // Combine probability of profit with drawdown resilience
                // Higher drawdown_95th means worse worst-case scenario (lower score)
                let drawdown_resilience = (1.0 - mc.drawdown_95th).clamp(0.0, 1.0);
                mc.probability_of_profit * 0.6 + drawdown_resilience * 0.4
            } else {
                0.5 // Neutral score if no simulations run
            }
        } else {
            0.5 // Neutral score if Monte Carlo not configured
        };

        // Weighted combination (all weights sum to 1.0)
        let combined_score = (wf_score * WEIGHT_WF_STABILITY
            + cv_score * WEIGHT_DISPERSION
            + positive_score * WEIGHT_POSITIVE_COMBOS
            + oos_win_rate * WEIGHT_OOS_WIN_RATE
            + mc_score * WEIGHT_MONTE_CARLO)
            .clamp(0.0, 1.0);

        // Scale to 0-100
        combined_score * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::ParamSweepResult;

    #[test]
    fn test_parameter_dispersion_empty() {
        let dispersion = ParameterDispersion::calculate(&[]);
        assert_eq!(dispersion.sharpe_std, 0.0);
        assert_eq!(dispersion.positive_sharpe_pct, 0.0);
    }

    #[test]
    fn test_parameter_dispersion_calculation() {
        let results = vec![
            ParamSweepResult {
                params: HashMap::new(),
                score: 0.5,
                sharpe: 1.0,
                total_return: 0.10,
                max_drawdown: -0.05,
                win_rate: 0.6,
                total_trades: 10,
            },
            ParamSweepResult {
                params: HashMap::new(),
                score: 0.6,
                sharpe: 1.5,
                total_return: 0.15,
                max_drawdown: -0.04,
                win_rate: 0.65,
                total_trades: 12,
            },
            ParamSweepResult {
                params: HashMap::new(),
                score: 0.4,
                sharpe: 0.8,
                total_return: 0.08,
                max_drawdown: -0.06,
                win_rate: 0.55,
                total_trades: 8,
            },
        ];

        let dispersion = ParameterDispersion::calculate(&results);

        // All Sharpe ratios are positive
        assert_eq!(dispersion.positive_sharpe_pct, 100.0);
        // All returns are positive
        assert_eq!(dispersion.positive_return_pct, 100.0);
        // Standard deviation should be non-zero
        assert!(dispersion.sharpe_std > 0.0);
        assert!(dispersion.return_std > 0.0);
        // CV should be positive
        assert!(dispersion.sharpe_cv > 0.0);
        assert!(dispersion.return_cv > 0.0);
    }

    #[test]
    fn test_workflow_config_default() {
        let config = WorkflowConfig::default();
        assert_eq!(config.initial_capital, 100_000.0);
        assert_eq!(config.fee_rate, 0.001);
        assert!(config.include_3d_sensitivity);
        assert!(config.monte_carlo_config.is_some());

        // Verify default Monte Carlo config
        let mc_config = config.monte_carlo_config.unwrap();
        assert_eq!(mc_config.num_simulations, 1000);
        assert_eq!(mc_config.initial_capital, 10_000.0);
        assert!(mc_config.seed.is_none());
    }
}
