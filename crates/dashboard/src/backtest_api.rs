use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

use alphafield_backtest::{
    AssetSentimentCalculator, AssetSentimentSummary, BacktestEngine, BenchmarkComparison,
    DrawdownAnalysis, DrawdownPoint, MonthlyReturn, PerformanceMetrics, RollingStats,
    SlippageModel, StrategyAdapter, Trade,
};
use alphafield_data::SentimentClient;

use crate::api::AppState;
use crate::services::data_service::{fetch_data_with_cache, DataStatus};
use crate::services::strategy_service::StrategyFactory;

#[derive(Debug, Deserialize)]
pub struct BacktestRequest {
    pub strategy: String,
    pub symbol: String,
    pub interval: String,
    pub days: u32,
    pub params: HashMap<String, f64>,
    #[serde(default)]
    pub include_benchmark: bool,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// Enhanced backtest response with comprehensive analytics
#[derive(Serialize)]
pub struct BacktestResponse {
    // Core Results
    pub metrics: PerformanceMetrics,
    pub equity_curve: Vec<EquityPoint>,
    pub trades: Vec<Trade>,

    // Benchmark Comparison (BTC buy-and-hold)
    pub benchmark: Option<BenchmarkData>,

    // Drawdown Analysis
    pub drawdown_curve: Vec<DrawdownPoint>,
    pub drawdown_analysis: DrawdownAnalysis,

    // Time-Series Analytics
    pub rolling_stats: RollingStats,
    pub monthly_returns: Vec<MonthlyReturn>,

    // Trade Analysis
    pub trade_summary: TradeSummary,

    // Sentiment Analysis
    pub market_sentiment: Option<MarketSentimentData>,
    pub asset_sentiment: AssetSentimentSummary,

    // Data Info
    pub data_status: DataStatus,

    // Metadata
    pub execution_time_ms: u64,
}

/// Global market sentiment data
#[derive(Serialize)]
pub struct MarketSentimentData {
    pub current_value: u8,
    pub current_classification: String,
    pub period_average: f64,
    pub fear_days: usize,
    pub greed_days: usize,
}

/// Equity curve point with detailed breakdown
#[derive(Serialize)]
pub struct EquityPoint {
    pub timestamp: i64,
    pub equity: f64,
    pub returns: f64,    // Period return
    pub cumulative: f64, // Cumulative return from start
}

/// Benchmark comparison data
#[derive(Serialize)]
pub struct BenchmarkData {
    pub curve: Vec<EquityPoint>,
    pub comparison: BenchmarkComparison,
}

/// Trade visualization summary
#[derive(Serialize, Default)]
pub struct TradeSummary {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub avg_trade_duration_hours: f64,
    pub avg_mae_percent: f64,
    pub avg_mfe_percent: f64,
    pub longest_winning_streak: usize,
    pub longest_losing_streak: usize,
}

impl TradeSummary {
    fn from_trades(trades: &[Trade]) -> Self {
        if trades.is_empty() {
            return Self::default();
        }

        let total = trades.len();
        let winners: Vec<_> = trades.iter().filter(|t| t.pnl > 0.0).collect();
        let losers: Vec<_> = trades.iter().filter(|t| t.pnl <= 0.0).collect();

        let avg_duration =
            trades.iter().map(|t| t.duration_secs).sum::<i64>() as f64 / total as f64 / 3600.0;

        let avg_mae = trades
            .iter()
            .map(|t| {
                if t.entry_price > 0.0 {
                    t.mae / t.entry_price * 100.0
                } else {
                    0.0
                }
            })
            .sum::<f64>()
            / total as f64;

        let avg_mfe = trades
            .iter()
            .map(|t| {
                if t.entry_price > 0.0 {
                    t.mfe / t.entry_price * 100.0
                } else {
                    0.0
                }
            })
            .sum::<f64>()
            / total as f64;

        // Calculate streaks
        let mut current_winning = 0usize;
        let mut current_losing = 0usize;
        let mut max_winning = 0usize;
        let mut max_losing = 0usize;

        for trade in trades {
            if trade.pnl > 0.0 {
                current_winning += 1;
                current_losing = 0;
                max_winning = max_winning.max(current_winning);
            } else {
                current_losing += 1;
                current_winning = 0;
                max_losing = max_losing.max(current_losing);
            }
        }

        Self {
            total_trades: total,
            winning_trades: winners.len(),
            losing_trades: losers.len(),
            avg_trade_duration_hours: avg_duration,
            avg_mae_percent: avg_mae,
            avg_mfe_percent: avg_mfe,
            longest_winning_streak: max_winning,
            longest_losing_streak: max_losing,
        }
    }
}

fn build_equity_curve(history: &[(i64, f64)]) -> Vec<EquityPoint> {
    if history.is_empty() {
        return Vec::new();
    }

    let initial = history[0].1;
    let mut prev = initial;

    history
        .iter()
        .map(|(ts, equity)| {
            let returns = if prev > 0.0 {
                (*equity - prev) / prev
            } else {
                0.0
            };
            let cumulative = if initial > 0.0 {
                (*equity - initial) / initial
            } else {
                0.0
            };
            prev = *equity;
            EquityPoint {
                timestamp: *ts,
                equity: *equity,
                returns,
                cumulative,
            }
        })
        .collect()
}

// Removed #[instrument] due to Handler trait issues with complex async futures
pub async fn run_backtest(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BacktestRequest>,
) -> Json<BacktestResponse> {
    let start = std::time::Instant::now();
    info!("Backtest API request received");

    // 1. Determine Date Range
    use chrono::{NaiveDate, TimeZone, Utc};

    // Parse end_date or default to now
    let end_time = if let Some(d) = &req.end_date {
        NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .ok()
            .and_then(|d| d.and_hms_opt(23, 59, 59))
            .map(|dt| Utc.from_utc_datetime(&dt))
            .unwrap_or_else(Utc::now)
    } else {
        Utc::now()
    };

    // Parse start_date or default to end_time - days
    let start_time = if let Some(d) = &req.start_date {
        NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .ok()
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| Utc.from_utc_datetime(&dt))
            .unwrap_or_else(|| end_time - chrono::Duration::days(req.days as i64))
    } else {
        end_time - chrono::Duration::days(req.days as i64)
    };

    info!(start = %start_time, end = %end_time, "Determined backtest range");

    // 2. Fetch Data (Cache -> API) with strict range check
    let fetch_symbol = req.symbol.clone();
    let fetch_interval = req.interval.clone();

    // Spawn to isolate async future and ensure Send safety
    let fetch_result = tokio::spawn(async move {
        fetch_data_with_cache(fetch_symbol, fetch_interval, start_time, end_time).await
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|res| res);

    let (bars, data_status) = match fetch_result {
        Ok((bars, status)) => (bars, status),
        Err(e) => {
            error!(error = %e, "Failed to fetch data");
            return Json(empty_response(start.elapsed().as_millis() as u64));
        }
    };

    if bars.is_empty() {
        warn!("No data returned for backtest");
        return Json(empty_response(start.elapsed().as_millis() as u64));
    }

    // 3. Run Backtest & Extract Data
    let (metrics, trades, equity_history) = {
        let mut engine = BacktestEngine::new(100_000.0, 0.001, SlippageModel::FixedPercent(0.0005));

        engine.add_data(&req.symbol, bars.clone());

        if let Some(strategy) = StrategyFactory::create(&req.strategy, &req.params) {
            let adapter = StrategyAdapter::new(strategy, &req.symbol, 100_000.0);
            engine.set_strategy(Box::new(adapter));
        } else {
            warn!(strategy = req.strategy, "Unknown strategy requested");
            return Json(empty_response(start.elapsed().as_millis() as u64));
        }

        let metrics = match engine.run() {
            Ok(m) => m,
            Err(e) => {
                error!(error = %e, "Backtest execution failed");
                return Json(empty_response(start.elapsed().as_millis() as u64));
            }
        };

        let equity_history: Vec<(i64, f64)> = engine
            .portfolio
            .equity_history
            .iter()
            .map(|(ts, val)| (*ts, *val))
            .collect();

        (metrics, engine.portfolio.trades.clone(), equity_history)
    };

    // 5. Build Analytics
    let equity_curve = build_equity_curve(&equity_history);
    let drawdown_curve = DrawdownAnalysis::generate_curve(&equity_history);
    let drawdown_analysis = DrawdownAnalysis::calculate(&equity_history, metrics.total_return);
    let rolling_stats = RollingStats::calculate(&equity_history, 20, 0.02);
    let monthly_returns = MonthlyReturn::calculate(&equity_history);
    let trade_summary = TradeSummary::from_trades(&trades);

    // 6. Benchmark Comparison (BTC buy-and-hold) - always included
    let benchmark = {
        let benchmark_curve = BenchmarkComparison::calculate_buy_and_hold(&bars, 100_000.0);
        if !benchmark_curve.is_empty() {
            let comparison =
                BenchmarkComparison::calculate(&equity_history, &benchmark_curve, 0.02);
            Some(BenchmarkData {
                curve: build_equity_curve(&benchmark_curve),
                comparison,
            })
        } else {
            None
        }
    };

    // 7. Market Sentiment
    // Concurrent fetch logic would be ideal, but for now we sequentialize it to ensure stability
    // The days count is calculated from the range
    let sentiment_days = (end_time - start_time).num_days() as u32;
    // Spawn to avoid Handler trait bounds issues with complex async futures
    let market_sentiment =
        tokio::spawn(async move { fetch_market_sentiment(sentiment_days).await })
            .await
            .unwrap_or(None);

    // 8. Asset Sentiment (Technical indicators)
    let asset_sentiment_calc = AssetSentimentCalculator::default();
    let asset_sentiment_series = asset_sentiment_calc.calculate_series(&bars);
    let asset_sentiment = AssetSentimentSummary::calculate(&asset_sentiment_series);

    info!(
        equity_points = equity_curve.len(),
        trade_count = trades.len(),
        execution_ms = start.elapsed().as_millis(),
        "Backtest API response ready"
    );

    Json(BacktestResponse {
        metrics,
        equity_curve,
        trades,
        benchmark,
        drawdown_curve,
        drawdown_analysis,
        rolling_stats,
        monthly_returns,
        trade_summary,
        market_sentiment,
        asset_sentiment,
        data_status,
        execution_time_ms: start.elapsed().as_millis() as u64,
    })
}

/// Fetch market sentiment
async fn fetch_market_sentiment(days: u32) -> Option<MarketSentimentData> {
    let client = SentimentClient::new();
    match client.get_history(days).await {
        Ok(data) if !data.is_empty() => {
            let current = data.first()?;
            let avg: f64 = data.iter().map(|d| d.value as f64).sum::<f64>() / data.len() as f64;
            let fear_days = data.iter().filter(|d| d.classification.is_fear()).count();
            let greed_days = data.iter().filter(|d| d.classification.is_greed()).count();

            Some(MarketSentimentData {
                current_value: current.value,
                current_classification: current.classification.to_string(),
                period_average: avg,
                fear_days,
                greed_days,
            })
        }
        _ => None,
    }
}

fn empty_response(execution_time_ms: u64) -> BacktestResponse {
    BacktestResponse {
        metrics: PerformanceMetrics::default(),
        equity_curve: vec![],
        trades: vec![],
        benchmark: None,
        drawdown_curve: vec![],
        drawdown_analysis: DrawdownAnalysis::default(),
        rolling_stats: RollingStats::default(),
        monthly_returns: vec![],
        trade_summary: TradeSummary::default(),
        market_sentiment: None,
        asset_sentiment: AssetSentimentSummary::default(),
        data_status: DataStatus::default(),
        execution_time_ms,
    }
}

// ===========================
// Parameter Optimization API
// ===========================

use alphafield_backtest::{get_strategy_bounds, ParamSweepResult, ParameterOptimizer};

#[derive(Debug, Deserialize)]
pub struct OptimizeRequest {
    pub strategy: String,
    pub symbol: String,
    pub interval: String,
    pub days: u32,
}

#[derive(Serialize)]
pub struct OptimizeResponse {
    pub success: bool,
    pub optimized_params: HashMap<String, f64>,
    pub best_score: f64,
    pub best_sharpe: f64,
    pub best_return: f64,
    pub best_max_drawdown: f64,
    pub best_win_rate: f64,
    pub best_trades: usize,
    pub iterations: usize,
    pub elapsed_ms: u64,
    /// All tested param combinations for visualization
    pub sweep_results: Vec<ParamSweepResult>,
    pub error: Option<String>,
}

pub async fn optimize_params(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<OptimizeRequest>,
) -> Json<OptimizeResponse> {
    let start = std::time::Instant::now();
    info!(strategy = %req.strategy, symbol = %req.symbol, "Starting parameter optimization");

    // 1. Date Range (same as backtest)
    use chrono::{Duration, Utc};
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(req.days as i64);

    // 2. Fetch Data
    let fetch_symbol = req.symbol.clone();
    let fetch_interval = req.interval.clone();

    let fetch_result = tokio::spawn(async move {
        fetch_data_with_cache(fetch_symbol, fetch_interval, start_time, end_time).await
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|res| res);

    let bars = match fetch_result {
        Ok((bars, _status)) => bars,
        Err(e) => {
            error!(error = %e, "Failed to fetch data for optimization");
            return Json(OptimizeResponse {
                success: false,
                optimized_params: HashMap::new(),
                best_score: 0.0,
                best_sharpe: 0.0,
                best_return: 0.0,
                best_max_drawdown: 0.0,
                best_win_rate: 0.0,
                best_trades: 0,
                iterations: 0,
                elapsed_ms: start.elapsed().as_millis() as u64,
                sweep_results: vec![],
                error: Some(format!("Failed to fetch data: {}", e)),
            });
        }
    };

    if bars.is_empty() {
        return Json(OptimizeResponse {
            success: false,
            optimized_params: HashMap::new(),
            best_score: 0.0,
            best_sharpe: 0.0,
            best_return: 0.0,
            best_max_drawdown: 0.0,
            best_win_rate: 0.0,
            best_trades: 0,
            iterations: 0,
            elapsed_ms: start.elapsed().as_millis() as u64,
            sweep_results: vec![],
            error: Some("No data available for optimization".to_string()),
        });
    }

    // 3. Get parameter bounds and run optimizer
    let bounds = get_strategy_bounds(&req.strategy);
    if bounds.is_empty() {
        return Json(OptimizeResponse {
            success: false,
            optimized_params: HashMap::new(),
            best_score: 0.0,
            best_sharpe: 0.0,
            best_return: 0.0,
            best_max_drawdown: 0.0,
            best_win_rate: 0.0,
            best_trades: 0,
            iterations: 0,
            elapsed_ms: start.elapsed().as_millis() as u64,
            sweep_results: vec![],
            error: Some(format!("Unknown strategy: {}", req.strategy)),
        });
    }

    let optimizer = ParameterOptimizer::new(100_000.0, 0.001);
    let strategy_name = req.strategy.clone();
    let symbol = req.symbol.clone();

    // Run optimizer in blocking task to avoid blocking async runtime
    let optimization_result = tokio::task::spawn_blocking(move || {
        optimizer.optimize(
            &bars,
            &symbol,
            |params| {
                // Create strategy adapter for backtest
                StrategyFactory::create_backtest(&strategy_name, params, &symbol, 100_000.0)
            },
            &bounds,
        )
    })
    .await;

    match optimization_result {
        Ok(Ok(result)) => {
            info!(
                score = result.best_score,
                sharpe = result.best_sharpe,
                return_pct = result.best_return * 100.0,
                iterations = result.iterations_tested,
                elapsed_ms = result.elapsed_ms,
                "Optimization complete"
            );

            Json(OptimizeResponse {
                success: true,
                optimized_params: result.best_params,
                best_score: result.best_score,
                best_sharpe: result.best_sharpe,
                best_return: result.best_return,
                best_max_drawdown: result.best_max_drawdown,
                best_win_rate: result.best_win_rate,
                best_trades: result.best_trades,
                iterations: result.iterations_tested,
                elapsed_ms: start.elapsed().as_millis() as u64,
                sweep_results: result.all_results,
                error: None,
            })
        }
        Ok(Err(e)) => {
            error!(error = %e, "Optimization failed");
            Json(OptimizeResponse {
                success: false,
                optimized_params: HashMap::new(),
                best_score: 0.0,
                best_sharpe: 0.0,
                best_return: 0.0,
                best_max_drawdown: 0.0,
                best_win_rate: 0.0,
                best_trades: 0,
                iterations: 0,
                elapsed_ms: start.elapsed().as_millis() as u64,
                sweep_results: vec![],
                error: Some(e),
            })
        }
        Err(e) => {
            error!(error = %e, "Optimization task panicked");
            Json(OptimizeResponse {
                success: false,
                optimized_params: HashMap::new(),
                best_score: 0.0,
                best_sharpe: 0.0,
                best_return: 0.0,
                best_max_drawdown: 0.0,
                best_win_rate: 0.0,
                best_trades: 0,
                iterations: 0,
                elapsed_ms: start.elapsed().as_millis() as u64,
                sweep_results: vec![],
                error: Some(format!("Internal error: {}", e)),
            })
        }
    }
}

// ===========================
// Comprehensive Optimization Workflow API
// ===========================

use alphafield_backtest::{OptimizationWorkflow, ParameterDispersion, WorkflowConfig};

// Time interval constants for bars per day calculation
const BARS_PER_DAY_1M: usize = 1440;
const BARS_PER_DAY_5M: usize = 288;
const BARS_PER_DAY_15M: usize = 96;
const BARS_PER_DAY_1H: usize = 24;
const BARS_PER_DAY_4H: usize = 6;
const BARS_PER_DAY_1D: usize = 1;
const BARS_PER_DAY_DEFAULT: usize = 24; // Default to hourly

// Walk-forward step size constant (approximate trading days per month)
const TRADING_DAYS_PER_MONTH: usize = 21;

#[derive(Debug, Deserialize)]
pub struct WorkflowRequest {
    pub strategy: String,
    pub symbol: String,
    pub interval: String,
    pub days: u32,
    /// Optional: Enable/disable 3D sensitivity analysis (default: true)
    pub include_3d_sensitivity: Option<bool>,
    /// Optional: Training window for walk-forward (in days, default: 252)
    pub train_window_days: Option<usize>,
    /// Optional: Testing window for walk-forward (in days, default: 63)
    pub test_window_days: Option<usize>,
}

#[derive(Serialize)]
pub struct WorkflowResponse {
    pub success: bool,
    /// Optimized parameters from grid search
    pub optimized_params: HashMap<String, f64>,
    /// Best composite score from optimization
    pub best_score: f64,
    /// Best Sharpe ratio from optimization
    pub best_sharpe: f64,
    /// Best return from optimization
    pub best_return: f64,
    /// In-sample performance metrics
    pub in_sample_sharpe: f64,
    pub in_sample_return: f64,
    pub in_sample_max_drawdown: f64,
    /// Walk-forward validation results
    pub walk_forward_mean_return: f64,
    pub walk_forward_median_return: f64,
    pub walk_forward_stability_score: f64,
    pub walk_forward_worst_drawdown: f64,
    pub walk_forward_windows: usize,
    /// Parameter dispersion statistics
    pub parameter_dispersion: ParameterDispersion,
    /// Overall robustness score (0-100)
    pub robustness_score: f64,
    /// All tested parameter combinations for visualization
    pub sweep_results: Vec<ParamSweepResult>,
    /// 3D sensitivity heatmap data (if enabled)
    pub sensitivity_heatmap: Option<alphafield_backtest::sensitivity::HeatmapData>,
    /// Execution time
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

pub async fn run_optimization_workflow(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<WorkflowRequest>,
) -> Json<WorkflowResponse> {
    let start = std::time::Instant::now();
    info!(
        strategy = %req.strategy, 
        symbol = %req.symbol, 
        "Starting comprehensive optimization workflow"
    );

    // 1. Date Range
    use chrono::{Duration, Utc};
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(req.days as i64);

    // 2. Fetch Data
    let fetch_symbol = req.symbol.clone();
    let fetch_interval = req.interval.clone();

    let fetch_result = tokio::spawn(async move {
        fetch_data_with_cache(fetch_symbol, fetch_interval, start_time, end_time).await
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|res| res);

    let bars = match fetch_result {
        Ok((bars, _status)) => bars,
        Err(e) => {
            error!(error = %e, "Failed to fetch data for optimization workflow");
            return Json(WorkflowResponse {
                success: false,
                optimized_params: HashMap::new(),
                best_score: 0.0,
                best_sharpe: 0.0,
                best_return: 0.0,
                in_sample_sharpe: 0.0,
                in_sample_return: 0.0,
                in_sample_max_drawdown: 0.0,
                walk_forward_mean_return: 0.0,
                walk_forward_median_return: 0.0,
                walk_forward_stability_score: 0.0,
                walk_forward_worst_drawdown: 0.0,
                walk_forward_windows: 0,
                parameter_dispersion: ParameterDispersion::default(),
                robustness_score: 0.0,
                sweep_results: vec![],
                sensitivity_heatmap: None,
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("Failed to fetch data: {}", e)),
            });
        }
    };

    if bars.is_empty() {
        return Json(WorkflowResponse {
            success: false,
            optimized_params: HashMap::new(),
            best_score: 0.0,
            best_sharpe: 0.0,
            best_return: 0.0,
            in_sample_sharpe: 0.0,
            in_sample_return: 0.0,
            in_sample_max_drawdown: 0.0,
            walk_forward_mean_return: 0.0,
            walk_forward_median_return: 0.0,
            walk_forward_stability_score: 0.0,
            walk_forward_worst_drawdown: 0.0,
            walk_forward_windows: 0,
            parameter_dispersion: ParameterDispersion::default(),
            robustness_score: 0.0,
            sweep_results: vec![],
            sensitivity_heatmap: None,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some("No data available for optimization".to_string()),
        });
    }

    // 3. Get parameter bounds
    let bounds = get_strategy_bounds(&req.strategy);
    if bounds.is_empty() {
        return Json(WorkflowResponse {
            success: false,
            optimized_params: HashMap::new(),
            best_score: 0.0,
            best_sharpe: 0.0,
            best_return: 0.0,
            in_sample_sharpe: 0.0,
            in_sample_return: 0.0,
            in_sample_max_drawdown: 0.0,
            walk_forward_mean_return: 0.0,
            walk_forward_median_return: 0.0,
            walk_forward_stability_score: 0.0,
            walk_forward_worst_drawdown: 0.0,
            walk_forward_windows: 0,
            parameter_dispersion: ParameterDispersion::default(),
            robustness_score: 0.0,
            sweep_results: vec![],
            sensitivity_heatmap: None,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("Unknown strategy: {}", req.strategy)),
        });
    }

    // 4. Configure workflow
    let bars_per_day = match req.interval.as_str() {
        "1m" => BARS_PER_DAY_1M,
        "5m" => BARS_PER_DAY_5M,
        "15m" => BARS_PER_DAY_15M,
        "1h" => BARS_PER_DAY_1H,
        "4h" => BARS_PER_DAY_4H,
        "1d" => BARS_PER_DAY_1D,
        _ => BARS_PER_DAY_DEFAULT,
    };

    let train_window_bars = req.train_window_days.unwrap_or(252) * bars_per_day;
    let test_window_bars = req.test_window_days.unwrap_or(63) * bars_per_day;

    let workflow_config = WorkflowConfig {
        initial_capital: 100_000.0,
        fee_rate: 0.001,
        slippage: alphafield_backtest::SlippageModel::FixedPercent(0.0005),
        walk_forward_config: alphafield_backtest::WalkForwardConfig {
            train_window: train_window_bars,
            test_window: test_window_bars,
            step_size: TRADING_DAYS_PER_MONTH * bars_per_day,
            initial_capital: 100_000.0,
            fee_rate: 0.001,
        },
        include_3d_sensitivity: req.include_3d_sensitivity.unwrap_or(true),
        train_test_split_ratio: 0.70, // Use default 70/30 split
    };

    // 5. Determine sensitivity parameters (first two from bounds for 3D visualization)
    let sensitivity_params = if workflow_config.include_3d_sensitivity && bounds.len() >= 2 {
        let param_x = alphafield_backtest::ParameterRange::new(
            &bounds[0].name,
            bounds[0].min,
            bounds[0].max,
            bounds[0].step,
        );
        let param_y = alphafield_backtest::ParameterRange::new(
            &bounds[1].name,
            bounds[1].min,
            bounds[1].max,
            bounds[1].step,
        );
        Some((param_x, param_y))
    } else {
        None
    };

    // 6. Run workflow in blocking task
    let strategy_name = req.strategy.clone();
    let symbol = req.symbol.clone();

    let workflow_result = tokio::task::spawn_blocking(move || {
        let workflow = OptimizationWorkflow::new(workflow_config);
        
        // Create factory closure
        let factory = |params: &HashMap<String, f64>| {
            StrategyFactory::create_backtest(&strategy_name, params, &symbol, 100_000.0)
        };
        
        workflow.run(
            &bars,
            &symbol,
            &factory,
            &bounds,
            sensitivity_params,
        )
    })
    .await;

    match workflow_result {
        Ok(Ok(result)) => {
            info!(
                robustness_score = result.robustness_score,
                best_sharpe = result.optimization.best_sharpe,
                wf_stability = result.walk_forward_validation.stability_score,
                elapsed_ms = start.elapsed().as_millis(),
                "Optimization workflow complete"
            );

            Json(WorkflowResponse {
                success: true,
                optimized_params: result.optimization.best_params,
                best_score: result.optimization.best_score,
                best_sharpe: result.optimization.best_sharpe,
                best_return: result.optimization.best_return,
                in_sample_sharpe: result.in_sample_metrics.sharpe_ratio,
                in_sample_return: result.in_sample_metrics.total_return,
                in_sample_max_drawdown: result.in_sample_metrics.max_drawdown,
                walk_forward_mean_return: result.walk_forward_validation.aggregate_oos.mean_return,
                walk_forward_median_return: result.walk_forward_validation.aggregate_oos.median_return,
                walk_forward_stability_score: result.walk_forward_validation.stability_score,
                walk_forward_worst_drawdown: result.walk_forward_validation.aggregate_oos.worst_drawdown,
                walk_forward_windows: result.walk_forward_validation.windows.len(),
                parameter_dispersion: result.parameter_dispersion,
                robustness_score: result.robustness_score,
                sweep_results: result.optimization.all_results,
                sensitivity_heatmap: result.sensitivity_3d.and_then(|s| s.heatmap),
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: None,
            })
        }
        Ok(Err(e)) => {
            error!(error = %e, "Optimization workflow failed");
            Json(WorkflowResponse {
                success: false,
                optimized_params: HashMap::new(),
                best_score: 0.0,
                best_sharpe: 0.0,
                best_return: 0.0,
                in_sample_sharpe: 0.0,
                in_sample_return: 0.0,
                in_sample_max_drawdown: 0.0,
                walk_forward_mean_return: 0.0,
                walk_forward_median_return: 0.0,
                walk_forward_stability_score: 0.0,
                walk_forward_worst_drawdown: 0.0,
                walk_forward_windows: 0,
                parameter_dispersion: ParameterDispersion::default(),
                robustness_score: 0.0,
                sweep_results: vec![],
                sensitivity_heatmap: None,
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: Some(e),
            })
        }
        Err(e) => {
            error!(error = %e, "Optimization workflow task panicked");
            Json(WorkflowResponse {
                success: false,
                optimized_params: HashMap::new(),
                best_score: 0.0,
                best_sharpe: 0.0,
                best_return: 0.0,
                in_sample_sharpe: 0.0,
                in_sample_return: 0.0,
                in_sample_max_drawdown: 0.0,
                walk_forward_mean_return: 0.0,
                walk_forward_median_return: 0.0,
                walk_forward_stability_score: 0.0,
                walk_forward_worst_drawdown: 0.0,
                walk_forward_windows: 0,
                parameter_dispersion: ParameterDispersion::default(),
                robustness_score: 0.0,
                sweep_results: vec![],
                sensitivity_heatmap: None,
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("Internal error: {}", e)),
            })
        }
    }
}

