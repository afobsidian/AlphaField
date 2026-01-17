//! Parameter Optimization Module
//!
//! Grid search-based parameter optimization to find optimal strategy parameters
//! using Sharpe ratio as the primary fitness metric.

use crate::engine::BacktestEngine;
use crate::exchange::SlippageModel;
use alphafield_strategy::framework::canonicalize_strategy_name;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Bounds for a single parameter during optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamBounds {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

impl ParamBounds {
    pub fn new(name: impl Into<String>, min: f64, max: f64, step: f64) -> Self {
        Self {
            name: name.into(),
            min,
            max,
            step,
        }
    }

    /// Generate all values in this parameter's range
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

/// Result of parameter optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub best_params: HashMap<String, f64>,
    pub best_score: f64,
    pub best_sharpe: f64,
    pub best_return: f64,
    pub best_max_drawdown: f64,
    pub best_win_rate: f64,
    pub best_trades: usize,
    pub iterations_tested: usize,
    pub elapsed_ms: u64,
    /// All tested combinations for visualization (params, score, sharpe, return, drawdown)
    pub all_results: Vec<ParamSweepResult>,
}

/// Single result in the parameter sweep for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSweepResult {
    pub params: HashMap<String, f64>,
    pub score: f64,
    pub sharpe: f64,
    pub total_return: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub total_trades: usize,
}

impl Default for OptimizationResult {
    fn default() -> Self {
        Self {
            best_params: HashMap::new(),
            best_score: f64::NEG_INFINITY,
            best_sharpe: f64::NEG_INFINITY,
            best_return: 0.0,
            best_max_drawdown: 0.0,
            best_win_rate: 0.0,
            best_trades: 0,
            iterations_tested: 0,
            elapsed_ms: 0,
            all_results: Vec::new(),
        }
    }
}

/// Calculate composite fitness score from multiple metrics
///
/// Weighted combination of:
/// - Sharpe Ratio (40%): Risk-adjusted return
/// - Total Return (25%): Raw profitability
/// - Win Rate (20%): Consistency
/// - Drawdown penalty (15%): Risk control
pub fn calculate_composite_score(
    sharpe: f64,
    total_return: f64,
    max_drawdown: f64,
    win_rate: f64,
    total_trades: usize,
) -> f64 {
    // Require at least 5 trades for valid scoring
    if total_trades < 5 {
        return f64::NEG_INFINITY;
    }

    // Normalize components
    let sharpe_component = sharpe.clamp(-3.0, 5.0) / 5.0; // Normalize to roughly 0-1
    let return_component = (total_return * 100.0).clamp(-50.0, 200.0) / 200.0; // Normalize
    let win_rate_component = win_rate; // Already 0-1
    let drawdown_penalty = (max_drawdown.abs()).clamp(0.0, 0.5) * 2.0; // 0-1 penalty

    // Weighted combination
    0.40 * sharpe_component + 0.25 * return_component + 0.20 * win_rate_component
        - 0.15 * drawdown_penalty
}

/// Parameter optimizer using grid search
pub struct ParameterOptimizer {
    pub initial_capital: f64,
    pub fee_rate: f64,
    pub slippage: SlippageModel,
}

impl ParameterOptimizer {
    pub fn new(initial_capital: f64, fee_rate: f64) -> Self {
        Self {
            initial_capital,
            fee_rate,
            slippage: SlippageModel::FixedPercent(0.0005),
        }
    }

    /// Generate all parameter combinations from bounds
    pub fn generate_param_combinations(bounds: &[ParamBounds]) -> Vec<HashMap<String, f64>> {
        if bounds.is_empty() {
            return vec![HashMap::new()];
        }

        let mut combinations = vec![HashMap::new()];

        for bound in bounds {
            let values = bound.values();
            let mut new_combinations = Vec::new();

            for combo in &combinations {
                for &value in &values {
                    let mut new_combo = combo.clone();
                    new_combo.insert(bound.name.clone(), value);
                    new_combinations.push(new_combo);
                }
            }

            combinations = new_combinations;
        }

        combinations
    }

    /// Run grid search optimization
    ///
    /// # Arguments
    /// * `data` - Historical bar data
    /// * `symbol` - Trading symbol
    /// * `strategy_factory` - Function that creates a strategy from params
    /// * `bounds` - Parameter bounds to search
    ///
    /// # Returns
    /// OptimizationResult with best parameters and all sweep results for visualization
    pub fn optimize<F>(
        &self,
        data: &[alphafield_core::Bar],
        symbol: &str,
        strategy_factory: F,
        bounds: &[ParamBounds],
    ) -> Result<OptimizationResult, String>
    where
        F: Fn(&HashMap<String, f64>) -> Option<Box<dyn crate::strategy::Strategy>>,
    {
        let start = std::time::Instant::now();

        if data.is_empty() {
            return Err("No data provided for optimization".to_string());
        }

        if bounds.is_empty() {
            return Err("No parameter bounds provided".to_string());
        }

        let combinations = Self::generate_param_combinations(bounds);
        let total_combinations = combinations.len();

        info!(
            combinations = total_combinations,
            "Starting parameter optimization"
        );

        let mut best_result = OptimizationResult::default();
        let mut all_results = Vec::with_capacity(total_combinations);

        for (i, params) in combinations.iter().enumerate() {
            // Create strategy with current params
            let strategy = match strategy_factory(params) {
                Some(s) => s,
                None => {
                    debug!(?params, "Invalid params, skipping");
                    continue;
                }
            };

            // Run backtest
            let mut engine =
                BacktestEngine::new(self.initial_capital, self.fee_rate, self.slippage.clone());
            engine.add_data(symbol, data.to_vec());
            engine.set_strategy(strategy);

            let metrics = match engine.run() {
                Ok(m) => m,
                Err(e) => {
                    debug!(?params, error = %e, "Backtest failed, skipping");
                    continue;
                }
            };

            // Calculate composite score
            let score = calculate_composite_score(
                metrics.sharpe_ratio,
                metrics.total_return,
                metrics.max_drawdown,
                metrics.win_rate,
                metrics.total_trades,
            );

            // Store result for visualization
            all_results.push(ParamSweepResult {
                params: params.clone(),
                score,
                sharpe: metrics.sharpe_ratio,
                total_return: metrics.total_return,
                max_drawdown: metrics.max_drawdown,
                win_rate: metrics.win_rate,
                total_trades: metrics.total_trades,
            });

            // Check if this is the best result
            if score > best_result.best_score {
                best_result.best_params = params.clone();
                best_result.best_score = score;
                best_result.best_sharpe = metrics.sharpe_ratio;
                best_result.best_return = metrics.total_return;
                best_result.best_max_drawdown = metrics.max_drawdown;
                best_result.best_win_rate = metrics.win_rate;
                best_result.best_trades = metrics.total_trades;

                debug!(
                    iteration = i + 1,
                    score = score,
                    sharpe = metrics.sharpe_ratio,
                    return_pct = metrics.total_return * 100.0,
                    "New best params found"
                );
            }

            best_result.iterations_tested += 1;

            // Log progress every 10%
            if (i + 1) % (total_combinations / 10).max(1) == 0 {
                info!(
                    progress = format!("{}%", (i + 1) * 100 / total_combinations),
                    tested = i + 1,
                    "Optimization progress"
                );
            }
        }

        best_result.elapsed_ms = start.elapsed().as_millis() as u64;
        best_result.all_results = all_results;

        info!(
            iterations = best_result.iterations_tested,
            best_score = best_result.best_score,
            best_sharpe = best_result.best_sharpe,
            elapsed_ms = best_result.elapsed_ms,
            "Optimization complete"
        );

        if best_result.iterations_tested == 0 {
            return Err("No valid parameter combinations found".to_string());
        }

        Ok(best_result)
    }
}

/// Get default parameter bounds for a strategy
pub fn get_strategy_bounds(strategy_name: &str) -> Vec<ParamBounds> {
    // Canonicalize strategy identifiers so optimization accepts display names from the registry/UI.
    // Uses the shared canonicalization function from alphafield_strategy::framework.
    let strategy_name = canonicalize_strategy_name(strategy_name);

    match strategy_name.as_str() {
        "GoldenCross" => vec![
            ParamBounds::new("fast_period", 5.0, 30.0, 5.0),
            ParamBounds::new("slow_period", 30.0, 100.0, 10.0),
            ParamBounds::new("take_profit", 2.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 10.0, 2.0),
        ],

        // --------------------------------------------------------------------
        // Phase 12.2: Trend Following Strategies
        // --------------------------------------------------------------------
        "Breakout" => vec![
            ParamBounds::new("lookback", 10.0, 100.0, 10.0),
            ParamBounds::new("take_profit", 2.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 10.0, 2.0),
        ],
        "MACrossover" => vec![
            ParamBounds::new("fast_period", 5.0, 30.0, 5.0),
            ParamBounds::new("slow_period", 20.0, 120.0, 10.0),
            ParamBounds::new("take_profit", 2.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 10.0, 2.0),
        ],
        "AdaptiveMA" => vec![
            ParamBounds::new("fast_period", 2.0, 20.0, 2.0),
            ParamBounds::new("slow_period", 10.0, 80.0, 10.0),
            ParamBounds::new("price_period", 2.0, 30.0, 2.0),
            ParamBounds::new("take_profit", 2.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 10.0, 2.0),
        ],
        "TripleMA" => vec![
            ParamBounds::new("fast_period", 3.0, 20.0, 2.0),
            ParamBounds::new("medium_period", 10.0, 60.0, 10.0),
            ParamBounds::new("slow_period", 20.0, 120.0, 20.0),
            ParamBounds::new("take_profit", 2.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 10.0, 2.0),
        ],
        "MacdTrend" => vec![
            ParamBounds::new("fast_period", 8.0, 15.0, 1.0),
            ParamBounds::new("slow_period", 20.0, 35.0, 3.0),
            ParamBounds::new("signal_period", 5.0, 15.0, 2.0),
            ParamBounds::new("take_profit", 2.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 10.0, 2.0),
        ],
        "ParabolicSAR" => vec![
            ParamBounds::new("af_step", 0.01, 0.05, 0.01),
            ParamBounds::new("af_max", 0.1, 0.3, 0.05),
            ParamBounds::new("take_profit", 2.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 10.0, 2.0),
        ],

        // --------------------------------------------------------------------
        // Phase 12.3: Mean Reversion Strategies
        // --------------------------------------------------------------------
        "BollingerBands" => vec![
            ParamBounds::new("period", 10.0, 30.0, 5.0),
            ParamBounds::new("std_dev", 1.5, 2.5, 0.5),
            ParamBounds::new("take_profit", 2.0, 8.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "RSIReversion" => vec![
            ParamBounds::new("period", 7.0, 21.0, 7.0),
            ParamBounds::new("lower_bound", 20.0, 35.0, 5.0),
            ParamBounds::new("upper_bound", 65.0, 80.0, 5.0),
            ParamBounds::new("take_profit", 2.0, 8.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "StochReversion" => vec![
            ParamBounds::new("k_period", 10.0, 20.0, 5.0),
            ParamBounds::new("d_period", 3.0, 5.0, 1.0),
            ParamBounds::new("lower_bound", 20.0, 30.0, 5.0),
            ParamBounds::new("upper_bound", 70.0, 80.0, 5.0),
            ParamBounds::new("take_profit", 2.0, 8.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "ZScoreReversion" => vec![
            ParamBounds::new("period", 20.0, 60.0, 10.0),
            ParamBounds::new("entry_threshold", 1.5, 2.5, 0.5),
            ParamBounds::new("exit_threshold", 0.5, 1.0, 0.5),
            ParamBounds::new("take_profit", 2.0, 8.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "PriceChannel" => vec![
            ParamBounds::new("period", 10.0, 40.0, 10.0),
            ParamBounds::new("take_profit", 2.0, 8.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "KeltnerReversion" => vec![
            ParamBounds::new("period", 10.0, 30.0, 5.0),
            ParamBounds::new("multiplier", 1.5, 2.5, 0.5),
            ParamBounds::new("take_profit", 2.0, 8.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "StatArb" => vec![
            ParamBounds::new("lookback", 20.0, 60.0, 10.0),
            ParamBounds::new("entry_threshold", 1.5, 2.5, 0.5),
            ParamBounds::new("exit_threshold", 0.5, 1.0, 0.5),
            ParamBounds::new("take_profit", 2.0, 8.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],

        // --------------------------------------------------------------------
        // Phase 12.4: Momentum Strategies
        // --------------------------------------------------------------------
        "RsiMomentumStrategy" => vec![
            ParamBounds::new("period", 10.0, 20.0, 5.0),
            ParamBounds::new("threshold", 50.0, 60.0, 5.0),
            ParamBounds::new("take_profit", 3.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "MACDStrategy" => vec![
            ParamBounds::new("fast_period", 8.0, 15.0, 2.0),
            ParamBounds::new("slow_period", 20.0, 30.0, 5.0),
            ParamBounds::new("signal_period", 7.0, 12.0, 2.0),
            ParamBounds::new("take_profit", 3.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "RocStrategy" => vec![
            ParamBounds::new("period", 10.0, 30.0, 5.0),
            ParamBounds::new("threshold", 0.5, 2.0, 0.5),
            ParamBounds::new("take_profit", 3.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "AdxTrendStrategy" => vec![
            ParamBounds::new("period", 10.0, 20.0, 5.0),
            ParamBounds::new("threshold", 20.0, 30.0, 5.0),
            ParamBounds::new("take_profit", 3.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "MomentumFactorStrategy" => vec![
            ParamBounds::new("lookback", 20.0, 60.0, 10.0),
            ParamBounds::new("formation_period", 60.0, 120.0, 20.0),
            ParamBounds::new("skip_period", 20.0, 40.0, 10.0),
            ParamBounds::new("take_profit", 3.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "VolumeMomentumStrategy" => vec![
            ParamBounds::new("price_period", 10.0, 30.0, 5.0),
            ParamBounds::new("volume_period", 10.0, 30.0, 5.0),
            ParamBounds::new("volume_threshold", 1.2, 2.0, 0.2),
            ParamBounds::new("take_profit", 3.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "MultiTfMomentumStrategy" => vec![
            ParamBounds::new("fast_ema", 5.0, 15.0, 5.0),
            ParamBounds::new("medium_ema", 20.0, 40.0, 10.0),
            ParamBounds::new("slow_ema", 50.0, 100.0, 25.0),
            ParamBounds::new("take_profit", 3.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],

        // --------------------------------------------------------------------
        // Baseline Strategies
        // --------------------------------------------------------------------
        "HODL_Baseline" => vec![ParamBounds::new("take_profit", 10.0, 50.0, 10.0)],
        "Market_Average_Baseline" => vec![
            ParamBounds::new("rebalance_period", 20.0, 60.0, 10.0),
            ParamBounds::new("take_profit", 10.0, 50.0, 10.0),
        ],

        // --------------------------------------------------------------------
        // Volatility-Based Strategies
        // --------------------------------------------------------------------
        "ATRBreakout" => vec![
            ParamBounds::new("atr_period", 10.0, 20.0, 2.0),
            ParamBounds::new("atr_multiplier", 1.0, 3.0, 0.5),
            ParamBounds::new("lookback_period", 15.0, 30.0, 5.0),
            ParamBounds::new("volume_multiplier", 1.2, 2.0, 0.2),
            ParamBounds::new("sma_period", 40.0, 60.0, 10.0),
            ParamBounds::new("take_profit", 5.0, 12.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 6.0, 1.0),
        ],
        "ATRTrailingStop" => vec![
            ParamBounds::new("atr_period", 10.0, 20.0, 2.0),
            ParamBounds::new("atr_multiplier", 1.5, 3.0, 0.5),
            ParamBounds::new("fast_period", 8.0, 15.0, 2.0),
            ParamBounds::new("slow_period", 25.0, 40.0, 5.0),
            ParamBounds::new("min_trailing_distance", 0.5, 2.0, 0.5),
            ParamBounds::new("take_profit", 8.0, 15.0, 2.0),
        ],
        "VolatilitySqueeze" => vec![
            ParamBounds::new("bb_period", 15.0, 25.0, 5.0),
            ParamBounds::new("bb_std", 1.5, 2.5, 0.5),
            ParamBounds::new("kc_period", 15.0, 25.0, 5.0),
            ParamBounds::new("kc_multiplier", 1.5, 2.5, 0.5),
            ParamBounds::new("take_profit", 3.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 6.0, 1.0),
        ],
        "VolRegimeStrategy" => vec![
            ParamBounds::new("atr_period", 10.0, 20.0, 2.0),
            ParamBounds::new("regime_period", 50.0, 150.0, 25.0),
            ParamBounds::new("low_threshold", 20.0, 40.0, 10.0),
            ParamBounds::new("high_threshold", 60.0, 80.0, 10.0),
            ParamBounds::new("take_profit", 3.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 6.0, 1.0),
        ],
        "VolSizingStrategy" => vec![
            ParamBounds::new("atr_period", 10.0, 20.0, 2.0),
            ParamBounds::new("baseline_period", 80.0, 120.0, 20.0),
            ParamBounds::new("risk_per_trade", 0.5, 2.5, 0.5),
            ParamBounds::new("take_profit", 3.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 6.0, 1.0),
        ],
        "GarchStrategy" => vec![
            ParamBounds::new("lookback", 20.0, 60.0, 10.0),
            ParamBounds::new("alpha", 0.05, 0.2, 0.05),
            ParamBounds::new("beta", 0.7, 0.95, 0.05),
            ParamBounds::new("volatility_threshold", 1.0, 2.5, 0.5),
            ParamBounds::new("take_profit", 3.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 6.0, 1.0),
        ],
        "VIXStyleStrategy" => vec![
            ParamBounds::new("atr_period", 10.0, 20.0, 2.0),
            ParamBounds::new("high_threshold", 25.0, 40.0, 5.0),
            ParamBounds::new("low_threshold", 15.0, 25.0, 5.0),
            ParamBounds::new("lookback", 10.0, 30.0, 5.0),
            ParamBounds::new("take_profit", 5.0, 15.0, 2.5),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.0),
        ],

        // --------------------------------------------------------------------
        // Existing strategies
        // --------------------------------------------------------------------
        "Momentum" => vec![
            ParamBounds::new("ema_period", 30.0, 70.0, 20.0),
            ParamBounds::new("macd_fast", 8.0, 15.0, 4.0),
            ParamBounds::new("macd_slow", 20.0, 30.0, 5.0),
            ParamBounds::new("macd_signal", 7.0, 11.0, 2.0),
            ParamBounds::new("take_profit", 3.0, 8.0, 2.5),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],

        // --------------------------------------------------------------------
        // Existing strategies that were missing
        // --------------------------------------------------------------------
        "Rsi" => vec![
            ParamBounds::new("period", 7.0, 21.0, 7.0),
            ParamBounds::new("lower_bound", 20.0, 35.0, 5.0),
            ParamBounds::new("upper_bound", 65.0, 80.0, 5.0),
            ParamBounds::new("take_profit", 2.0, 8.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],
        "MeanReversion" => vec![
            ParamBounds::new("period", 15.0, 30.0, 5.0),
            ParamBounds::new("std_dev", 1.5, 2.5, 0.5),
            ParamBounds::new("take_profit", 2.0, 6.0, 2.0),
            ParamBounds::new("stop_loss", 3.0, 8.0, 2.5),
        ],

        // --------------------------------------------------------------------
        // Additional baseline strategies (missing ones only)
        // --------------------------------------------------------------------
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_bounds_values() {
        let bounds = ParamBounds::new("test", 1.0, 5.0, 1.0);
        let values = bounds.values();
        assert_eq!(values, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_param_bounds_fractional() {
        let bounds = ParamBounds::new("test", 1.0, 2.0, 0.5);
        let values = bounds.values();
        assert_eq!(values.len(), 3);
        assert!((values[0] - 1.0).abs() < f64::EPSILON);
        assert!((values[1] - 1.5).abs() < f64::EPSILON);
        assert!((values[2] - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_generate_combinations() {
        let bounds = vec![
            ParamBounds::new("a", 1.0, 2.0, 1.0),
            ParamBounds::new("b", 10.0, 20.0, 10.0),
        ];
        let combos = ParameterOptimizer::generate_param_combinations(&bounds);
        // 2 values for "a" × 2 values for "b" = 4 combinations
        assert_eq!(combos.len(), 4);
    }

    #[test]
    fn test_get_strategy_bounds() {
        let bounds = get_strategy_bounds("GoldenCross");
        assert!(!bounds.is_empty());
        assert!(bounds.iter().any(|b| b.name == "fast_period"));
        assert!(bounds.iter().any(|b| b.name == "slow_period"));
    }

    #[test]
    fn test_unknown_strategy_bounds() {
        let bounds = get_strategy_bounds("UnknownStrategy");
        assert!(bounds.is_empty());
    }

    // Tests for canonicalize_strategy_name integration
    #[test]
    fn test_canonicalize_golden_cross_display_name() {
        let bounds = get_strategy_bounds("Golden Cross");
        assert!(!bounds.is_empty());
        assert!(bounds.iter().any(|b| b.name == "fast_period"));
        assert!(bounds.iter().any(|b| b.name == "slow_period"));
    }

    #[test]
    fn test_canonicalize_parabolic_sar_display_name() {
        let bounds = get_strategy_bounds("Parabolic SAR");
        assert!(!bounds.is_empty());
        assert!(bounds.iter().any(|b| b.name == "af_step"));
        assert!(bounds.iter().any(|b| b.name == "af_max"));
    }

    #[test]
    fn test_canonicalize_adaptive_ma_display_name() {
        let bounds = get_strategy_bounds("Adaptive MA");
        assert!(!bounds.is_empty());
        assert!(bounds.iter().any(|b| b.name == "fast_period"));
        assert!(bounds.iter().any(|b| b.name == "price_period"));
    }

    #[test]
    fn test_canonicalize_ma_crossover_display_name() {
        let bounds = get_strategy_bounds("MA Crossover");
        assert!(!bounds.is_empty());
        assert!(bounds.iter().any(|b| b.name == "fast_period"));
        assert!(bounds.iter().any(|b| b.name == "slow_period"));
    }

    #[test]
    fn test_canonicalize_macd_trend_display_name() {
        let bounds = get_strategy_bounds("MACD Trend");
        assert!(!bounds.is_empty());
        assert!(bounds.iter().any(|b| b.name == "fast_period"));
        assert!(bounds.iter().any(|b| b.name == "signal_period"));
    }

    #[test]
    fn test_canonicalize_already_canonical_names() {
        // Verify that internal keys still work
        let bounds1 = get_strategy_bounds("GoldenCross");
        let bounds2 = get_strategy_bounds("Golden Cross");

        assert_eq!(bounds1.len(), bounds2.len());

        // Verify same parameters are present
        let names1: Vec<_> = bounds1.iter().map(|b| b.name.clone()).collect();
        let names2: Vec<_> = bounds2.iter().map(|b| b.name.clone()).collect();
        assert_eq!(names1, names2);
    }

    #[test]
    fn test_canonicalize_whitespace_handling() {
        // Test with extra whitespace
        let bounds = get_strategy_bounds("  Golden Cross  ");
        assert!(!bounds.is_empty());
        assert!(bounds.iter().any(|b| b.name == "fast_period"));
    }
}
