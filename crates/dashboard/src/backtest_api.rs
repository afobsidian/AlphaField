use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

use alphafield_backtest::{
    AssetSentimentCalculator, AssetSentimentSummary, BacktestEngine, BenchmarkComparison,
    DrawdownAnalysis, DrawdownPoint, MonthlyReturn, PerformanceMetrics, RollingStats,
    SlippageModel, StrategyAdapter, Trade, TradeSide,
};
use alphafield_core::TradingMode;
use alphafield_data::SentimentClient;

use crate::api::AppState;
use crate::services::data_service::{fetch_data_with_cache, DataStatus};
use crate::services::strategy_service::StrategyFactory;

#[derive(Debug, Deserialize)]
pub struct BacktestRequest {
    #[serde(default)]
    pub strategy: Option<String>,
    #[serde(default)]
    pub strategies: Vec<String>,
    pub symbol: String,
    pub interval: String,
    pub days: u32,
    pub params: HashMap<String, f64>,
    #[serde(default)]
    pub include_benchmark: bool,
    #[serde(default = "default_trading_mode")]
    pub trading_mode: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

fn default_trading_mode() -> String {
    "Spot".to_string()
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
    /// Long/Short breakdown metrics
    pub long_trades_count: usize,
    pub short_trades_count: usize,
    pub long_win_rate: f64,
    pub short_win_rate: f64,
    pub avg_long_profit: f64,
    pub avg_short_profit: f64,
    pub total_long_profit: f64,
    pub total_short_profit: f64,
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

        // Calculate long/short breakdown
        let long_trades: Vec<_> = trades
            .iter()
            .filter(|t| matches!(t.side, TradeSide::Long))
            .collect();
        let short_trades: Vec<_> = trades
            .iter()
            .filter(|t| matches!(t.side, TradeSide::Short))
            .collect();

        let long_trades_count = long_trades.len();
        let short_trades_count = short_trades.len();

        let long_winning: Vec<_> = long_trades.iter().filter(|t| t.pnl > 0.0).collect();
        let short_winning: Vec<_> = short_trades.iter().filter(|t| t.pnl > 0.0).collect();

        let long_win_rate = if long_trades_count > 0 {
            long_winning.len() as f64 / long_trades_count as f64
        } else {
            0.0
        };

        let short_win_rate = if short_trades_count > 0 {
            short_winning.len() as f64 / short_trades_count as f64
        } else {
            0.0
        };

        let total_long_profit: f64 = long_trades.iter().map(|t| t.pnl).sum();
        let total_short_profit: f64 = short_trades.iter().map(|t| t.pnl).sum();

        let avg_long_profit = if long_trades_count > 0 {
            total_long_profit / long_trades_count as f64
        } else {
            0.0
        };

        let avg_short_profit = if short_trades_count > 0 {
            total_short_profit / short_trades_count as f64
        } else {
            0.0
        };

        Self {
            total_trades: total,
            winning_trades: winners.len(),
            losing_trades: losers.len(),
            avg_trade_duration_hours: avg_duration,
            avg_mae_percent: avg_mae,
            avg_mfe_percent: avg_mfe,
            longest_winning_streak: max_winning,
            longest_losing_streak: max_losing,
            long_trades_count,
            short_trades_count,
            long_win_rate,
            short_win_rate,
            avg_long_profit,
            avg_short_profit,
            total_long_profit,
            total_short_profit,
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
        // Parse trading mode from request (default to Spot if invalid)
        let trading_mode = match req.trading_mode.as_str() {
            "Margin" => TradingMode::Margin,
            _ => TradingMode::Spot,
        };

        let mut engine = BacktestEngine::new(100_000.0, 0.001, SlippageModel::FixedPercent(0.0005))
            .with_trading_mode(trading_mode);

        engine.add_data(&req.symbol, bars.clone());

        // Determine which strategy to use (support both single and multi-strategy requests)
        let strategy_name = if !req.strategies.is_empty() {
            // Use first strategy from the array if provided
            &req.strategies[0]
        } else if let Some(ref strategy) = req.strategy {
            // Fall back to single strategy field
            strategy
        } else {
            warn!("No strategy provided in request");
            return Json(empty_response(start.elapsed().as_millis() as u64));
        };

        if let Some(strategy) = StrategyFactory::create(strategy_name, &req.params) {
            let adapter = StrategyAdapter::new(strategy, &req.symbol, 100_000.0);
            engine.set_strategy(Box::new(adapter));
        } else {
            warn!(strategy = strategy_name, "Unknown strategy requested");
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
    #[serde(default)]
    pub strategy: Option<String>,
    #[serde(default)]
    pub strategies: Vec<String>,
    pub symbol: String,
    pub interval: String,
    pub days: u32,
    #[serde(default = "default_trading_mode")]
    pub trading_mode: String,
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

    // Determine which strategy to use (support both single and multi-strategy requests)
    let strategy_name = if !req.strategies.is_empty() {
        // Use first strategy from the array if provided
        &req.strategies[0]
    } else if let Some(ref strategy) = req.strategy {
        // Fall back to single strategy field
        strategy
    } else {
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
            error: Some("No strategy provided in request".to_string()),
        });
    };

    info!(strategy = %strategy_name, symbol = %req.symbol, "Starting parameter optimization");

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
    let bounds = get_strategy_bounds(strategy_name);
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
            error: Some(format!("Unknown strategy: {}", strategy_name)),
        });
    }

    let optimizer = ParameterOptimizer::new(100_000.0, 0.001);
    let strategy_name_owned = strategy_name.to_string(); // Convert &str to owned String for closure
    let symbol = req.symbol.clone();
    let trading_mode = match req.trading_mode.to_lowercase().as_str() {
        "margin" => TradingMode::Margin,
        _ => TradingMode::Spot,
    };

    // Run optimizer in blocking task to avoid blocking async runtime
    let optimization_result = tokio::task::spawn_blocking(move || {
        optimizer.optimize(
            &bars,
            &symbol,
            |params| {
                // Create strategy adapter for backtest
                StrategyFactory::create_backtest(
                    &strategy_name_owned,
                    params,
                    &symbol,
                    100_000.0,
                    trading_mode,
                )
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
    /// Trading mode (Spot or Margin)
    pub trading_mode: String,
    /// Optional: Risk-free rate for Sharpe ratio calculation (default: 0.02)
    pub risk_free_rate: Option<f64>,
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
    /// Walk-forward validation results (aggregate metrics - kept for backward compatibility)
    pub walk_forward_mean_return: f64,
    pub walk_forward_median_return: f64,
    pub walk_forward_stability_score: f64,
    pub walk_forward_worst_drawdown: f64,
    pub walk_forward_windows: usize,
    /// Full walk-forward validation results (includes windows data for visualization)
    pub walk_forward_validation: Option<alphafield_backtest::WalkForwardResult>,
    /// Parameter dispersion statistics
    pub parameter_dispersion: ParameterDispersion,
    /// Monte Carlo simulation results (path dependency robustness testing)
    pub monte_carlo: Option<alphafield_backtest::MonteCarloResult>,
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
                walk_forward_validation: None,
                parameter_dispersion: ParameterDispersion::default(),
                monte_carlo: None,
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
            walk_forward_validation: None,
            parameter_dispersion: ParameterDispersion::default(),
            monte_carlo: None,
            robustness_score: 0.0,
            sweep_results: vec![],
            sensitivity_heatmap: None,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some("No data available for specified date range".to_string()),
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
            walk_forward_validation: None,
            parameter_dispersion: ParameterDispersion::default(),
            monte_carlo: None,
            robustness_score: 0.0,
            sweep_results: vec![],
            sensitivity_heatmap: None,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some("No parameter bounds specified".to_string()),
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
        monte_carlo_config: Some(alphafield_backtest::MonteCarloConfig::default()), // Monte Carlo enabled by default
        risk_free_rate: req.risk_free_rate.unwrap_or(0.02), // Default 2% risk-free rate
        trading_mode: match req.trading_mode.as_str() {
            "Margin" => TradingMode::Margin,
            _ => TradingMode::Spot,
        },
    };
    let workflow_trading_mode = workflow_config.trading_mode;

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
            StrategyFactory::create_backtest(
                &strategy_name,
                params,
                &symbol,
                100_000.0,
                workflow_trading_mode,
            )
        };

        workflow.run(&bars, &symbol, &factory, &bounds, sensitivity_params)
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
                walk_forward_median_return: result
                    .walk_forward_validation
                    .aggregate_oos
                    .median_return,
                walk_forward_stability_score: result.walk_forward_validation.stability_score,
                walk_forward_worst_drawdown: result
                    .walk_forward_validation
                    .aggregate_oos
                    .worst_drawdown,
                walk_forward_windows: result.walk_forward_validation.windows.len(),
                walk_forward_validation: Some(result.walk_forward_validation),
                parameter_dispersion: result.parameter_dispersion,
                monte_carlo: result.monte_carlo,
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
                walk_forward_validation: None,
                parameter_dispersion: ParameterDispersion::default(),
                monte_carlo: None,
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
                walk_forward_validation: None,
                parameter_dispersion: ParameterDispersion::default(),
                monte_carlo: None,
                robustness_score: 0.0,
                sweep_results: vec![],
                sensitivity_heatmap: None,
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: Some("Workflow task panicked".to_string()),
            })
        }
    }
}

// ===========================
// Multi-Symbol Optimization Workflow API
// ===========================

/// Request for multi-symbol optimization workflow
#[derive(Debug, Deserialize)]
pub struct MultiSymbolWorkflowRequest {
    pub strategy: String,
    /// Multiple symbols to optimize across
    pub symbols: Vec<String>,
    pub interval: String,
    pub days: u32,
    /// Optional: Enable/disable 3D sensitivity analysis (default: false for multi-symbol)
    pub include_3d_sensitivity: Option<bool>,
    /// Optional: Training window for walk-forward (in days, default: 252)
    pub train_window_days: Option<usize>,
    /// Optional: Testing window for walk-forward (in days, default: 63)
    pub test_window_days: Option<usize>,
    #[serde(default = "default_trading_mode")]
    pub trading_mode: String,
}

/// Result for a single symbol in multi-symbol optimization
#[derive(Serialize)]
pub struct SymbolResult {
    pub symbol: String,
    pub sharpe_ratio: f64,
    pub total_return: f64,
    pub max_drawdown: f64,
    pub total_trades: usize,
}

/// Response for multi-symbol optimization workflow
#[derive(Serialize)]
pub struct MultiSymbolWorkflowResponse {
    pub success: bool,
    /// Optimized parameters (aggregated across all symbols)
    pub optimized_params: HashMap<String, f64>,
    /// Average robustness score across symbols
    pub robustness_score: f64,
    /// Average Sharpe ratio across symbols
    pub avg_sharpe: f64,
    /// Average return across symbols
    pub avg_return: f64,
    /// Worst drawdown across symbols
    pub worst_drawdown: f64,
    /// Per-symbol results
    pub symbol_results: Vec<SymbolResult>,
    /// Number of symbols successfully processed
    pub symbols_processed: usize,
    /// Parameter dispersion (from primary symbol)
    pub parameter_dispersion: ParameterDispersion,
    /// Sweep results from primary symbol (for visualization)
    pub sweep_results: Vec<ParamSweepResult>,
    /// Walk-forward validation from primary symbol (for visualization)
    pub walk_forward_validation: Option<alphafield_backtest::WalkForwardResult>,
    /// Sensitivity heatmap from primary symbol (for visualization)
    pub sensitivity_heatmap: Option<alphafield_backtest::sensitivity::HeatmapData>,
    /// Execution time
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

/// Run optimization workflow across multiple symbols
///
/// This performs combined optimization:
/// - Parameter sweep tests each param combo across ALL symbols, averaging scores
/// - Walk-forward randomly selects symbols for each window to validate generalization
pub async fn run_multi_symbol_workflow(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<MultiSymbolWorkflowRequest>,
) -> Json<MultiSymbolWorkflowResponse> {
    let start = std::time::Instant::now();
    info!(
        strategy = %req.strategy,
        symbols = ?req.symbols,
        "Starting combined multi-symbol optimization workflow"
    );

    if req.symbols.is_empty() {
        return Json(MultiSymbolWorkflowResponse {
            success: false,
            optimized_params: HashMap::new(),
            robustness_score: 0.0,
            avg_sharpe: 0.0,
            avg_return: 0.0,
            worst_drawdown: 0.0,
            symbol_results: vec![],
            symbols_processed: 0,
            parameter_dispersion: ParameterDispersion::default(),
            sweep_results: vec![],
            walk_forward_validation: None,
            sensitivity_heatmap: None,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some("No symbols provided".to_string()),
        });
    }

    // Get parameter bounds for the strategy
    let bounds = get_strategy_bounds(&req.strategy);
    if bounds.is_empty() {
        return Json(MultiSymbolWorkflowResponse {
            success: false,
            optimized_params: HashMap::new(),
            robustness_score: 0.0,
            avg_sharpe: 0.0,
            avg_return: 0.0,
            worst_drawdown: 0.0,
            symbol_results: vec![],
            symbols_processed: 0,
            parameter_dispersion: ParameterDispersion::default(),
            sweep_results: vec![],
            walk_forward_validation: None,
            sensitivity_heatmap: None,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("Unknown strategy: {}", req.strategy)),
        });
    }

    // Date range
    use chrono::{Duration, Utc};
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(req.days as i64);

    // 1. Pre-fetch data for ALL symbols
    info!("Fetching data for {} symbols...", req.symbols.len());
    let mut symbol_data: Vec<(String, Vec<alphafield_core::Bar>)> = Vec::new();

    for symbol in &req.symbols {
        let fetch_symbol = symbol.clone();
        let fetch_interval = req.interval.clone();

        let fetch_result = tokio::spawn(async move {
            fetch_data_with_cache(fetch_symbol, fetch_interval, start_time, end_time).await
        })
        .await
        .map_err(|e| e.to_string())
        .and_then(|res| res);

        match fetch_result {
            Ok((bars, _status)) if !bars.is_empty() => {
                symbol_data.push((symbol.clone(), bars));
            }
            Ok(_) => {
                warn!(symbol = %symbol, "No data available for symbol, skipping");
            }
            Err(e) => {
                warn!(symbol = %symbol, error = %e, "Failed to fetch data for symbol, skipping");
            }
        }
    }

    if symbol_data.is_empty() {
        return Json(MultiSymbolWorkflowResponse {
            success: false,
            optimized_params: HashMap::new(),
            robustness_score: 0.0,
            avg_sharpe: 0.0,
            avg_return: 0.0,
            worst_drawdown: 0.0,
            symbol_results: vec![],
            symbols_processed: 0,
            parameter_dispersion: ParameterDispersion::default(),
            sweep_results: vec![],
            walk_forward_validation: None,
            sensitivity_heatmap: None,
            elapsed_ms: start.elapsed().as_millis() as u64,
            error: Some("No symbols had valid data".to_string()),
        });
    }

    info!(
        "Fetched data for {} symbols successfully",
        symbol_data.len()
    );

    // 2. Run combined parameter sweep across all symbols
    let strategy_name = req.strategy.clone();
    let bounds_clone = bounds.clone();
    let symbol_data_clone = symbol_data.clone();
    let trading_mode = match req.trading_mode.to_lowercase().as_str() {
        "margin" => TradingMode::Margin,
        _ => TradingMode::Spot,
    };

    let sweep_result = tokio::task::spawn_blocking(move || {
        run_combined_parameter_sweep(
            &symbol_data_clone,
            &strategy_name,
            &bounds_clone,
            trading_mode,
        )
    })
    .await;

    let (
        sweep_results,
        best_params,
        _best_score,
        best_sharpe,
        best_return,
        best_max_drawdown,
        _best_trades,
        symbol_results,
    ) = match sweep_result {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => {
            error!(error = %e, "Combined parameter sweep failed");
            return Json(MultiSymbolWorkflowResponse {
                success: false,
                optimized_params: HashMap::new(),
                robustness_score: 0.0,
                avg_sharpe: 0.0,
                avg_return: 0.0,
                worst_drawdown: 0.0,
                symbol_results: vec![],
                symbols_processed: 0,
                parameter_dispersion: ParameterDispersion::default(),
                sweep_results: vec![],
                walk_forward_validation: None,
                sensitivity_heatmap: None,
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: Some(e),
            });
        }
        Err(e) => {
            error!(error = %e, "Combined parameter sweep task panicked");
            return Json(MultiSymbolWorkflowResponse {
                success: false,
                optimized_params: HashMap::new(),
                robustness_score: 0.0,
                avg_sharpe: 0.0,
                avg_return: 0.0,
                worst_drawdown: 0.0,
                symbol_results: vec![],
                symbols_processed: 0,
                parameter_dispersion: ParameterDispersion::default(),
                sweep_results: vec![],
                walk_forward_validation: None,
                sensitivity_heatmap: None,
                elapsed_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("Internal error: {}", e)),
            });
        }
    };

    // 3. Run multi-symbol walk-forward with random symbol selection
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

    let strategy_name_wf = req.strategy.clone();
    let best_params_wf = best_params.clone();
    let symbol_data_wf = symbol_data.clone();
    let trading_mode_wf = trading_mode;

    let walk_forward_result = tokio::task::spawn_blocking(move || {
        run_multi_symbol_walk_forward(
            &symbol_data_wf,
            &strategy_name_wf,
            &best_params_wf,
            train_window_bars,
            test_window_bars,
            TRADING_DAYS_PER_MONTH * bars_per_day,
            trading_mode_wf,
        )
    })
    .await;

    let walk_forward_validation = match walk_forward_result {
        Ok(Ok(result)) => Some(result),
        Ok(Err(e)) => {
            warn!(error = %e, "Multi-symbol walk-forward failed, continuing without it");
            None
        }
        Err(e) => {
            warn!(error = %e, "Walk-forward task panicked, continuing without it");
            None
        }
    };

    // 4. Calculate parameter dispersion and robustness
    let parameter_dispersion = ParameterDispersion::calculate(&sweep_results);

    let robustness_score = if let Some(ref wf) = walk_forward_validation {
        calculate_multi_symbol_robustness(&parameter_dispersion, wf)
    } else {
        // Without walk-forward, use dispersion only
        let dispersion_score = 100.0 * (1.0 - parameter_dispersion.sharpe_cv.min(1.0));
        dispersion_score.clamp(0.0, 100.0)
    };

    info!(
        symbols_processed = symbol_data.len(),
        best_sharpe = best_sharpe,
        robustness_score = robustness_score,
        elapsed_ms = start.elapsed().as_millis(),
        "Combined multi-symbol optimization complete"
    );

    Json(MultiSymbolWorkflowResponse {
        success: true,
        optimized_params: best_params,
        robustness_score,
        avg_sharpe: best_sharpe,
        avg_return: best_return,
        worst_drawdown: best_max_drawdown,
        symbol_results,
        symbols_processed: symbol_data.len(),
        parameter_dispersion,
        sweep_results,
        walk_forward_validation,
        sensitivity_heatmap: None, // Removed from display as requested
        elapsed_ms: start.elapsed().as_millis() as u64,
        error: None,
    })
}

/// Run combined parameter sweep across all symbols
/// Each parameter combination is tested on ALL symbols and scores are averaged
#[allow(clippy::type_complexity)]
fn run_combined_parameter_sweep(
    symbol_data: &[(String, Vec<alphafield_core::Bar>)],
    strategy_name: &str,
    bounds: &[alphafield_backtest::ParamBounds],
    trading_mode: TradingMode,
) -> Result<
    (
        Vec<ParamSweepResult>,
        HashMap<String, f64>,
        f64,
        f64,
        f64,
        f64,
        usize,
        Vec<SymbolResult>,
    ),
    String,
> {
    use alphafield_backtest::optimizer::{calculate_composite_score, ParameterOptimizer};
    use alphafield_backtest::{BacktestEngine, SlippageModel};

    let param_combinations = ParameterOptimizer::generate_param_combinations(bounds);

    if param_combinations.is_empty() {
        return Err("No parameter combinations generated".to_string());
    }

    info!(
        "Testing {} parameter combinations across {} symbols",
        param_combinations.len(),
        symbol_data.len()
    );

    let mut sweep_results: Vec<ParamSweepResult> = Vec::new();
    let mut best_params = HashMap::new();
    let mut best_score = f64::NEG_INFINITY;
    let mut best_sharpe = 0.0;
    let mut best_return = 0.0;
    let mut best_max_drawdown = 0.0;
    let mut best_trades = 0;

    // For each parameter combination
    for params in &param_combinations {
        let mut combo_sharpes: Vec<f64> = Vec::new();
        let mut combo_returns: Vec<f64> = Vec::new();
        let mut combo_drawdowns: Vec<f64> = Vec::new();
        let mut combo_win_rates: Vec<f64> = Vec::new();
        let mut combo_trades: Vec<usize> = Vec::new();

        // Test on ALL symbols
        for (symbol, bars) in symbol_data {
            // Create strategy
            let strategy = match StrategyFactory::create_backtest(
                strategy_name,
                params,
                symbol,
                100_000.0,
                trading_mode,
            ) {
                Some(s) => s,
                None => continue,
            };

            // Run backtest
            let mut engine =
                BacktestEngine::new(100_000.0, 0.001, SlippageModel::FixedPercent(0.0005));
            engine.add_data(symbol, bars.clone());
            engine.set_strategy(strategy);

            if let Ok(metrics) = engine.run() {
                combo_sharpes.push(metrics.sharpe_ratio);
                combo_returns.push(metrics.total_return);
                combo_drawdowns.push(metrics.max_drawdown);
                combo_win_rates.push(metrics.win_rate);
                combo_trades.push(engine.portfolio.trades.len());
            }
        }

        if combo_sharpes.is_empty() {
            continue;
        }

        // Calculate averages across all symbols
        let avg_sharpe = combo_sharpes.iter().sum::<f64>() / combo_sharpes.len() as f64;
        let avg_return = combo_returns.iter().sum::<f64>() / combo_returns.len() as f64;
        let max_drawdown = combo_drawdowns.iter().cloned().fold(0.0, f64::max);
        let avg_win_rate = combo_win_rates.iter().sum::<f64>() / combo_win_rates.len() as f64;
        let total_trades: usize = combo_trades.iter().sum();

        // Calculate combined score
        let score = calculate_composite_score(
            avg_sharpe,
            avg_return,
            max_drawdown,
            avg_win_rate,
            total_trades,
        );

        // Store sweep result
        sweep_results.push(ParamSweepResult {
            params: params.clone(),
            sharpe: avg_sharpe,
            total_return: avg_return,
            max_drawdown,
            win_rate: avg_win_rate,
            total_trades,
            score,
        });

        // Track best
        if score > best_score {
            best_score = score;
            best_params = params.clone();
            best_sharpe = avg_sharpe;
            best_return = avg_return;
            best_max_drawdown = max_drawdown;
            best_trades = total_trades;
        }
    }

    // Calculate per-symbol results with best params
    let mut symbol_results: Vec<SymbolResult> = Vec::new();
    for (symbol, bars) in symbol_data {
        let strategy = match StrategyFactory::create_backtest(
            strategy_name,
            &best_params,
            symbol,
            100_000.0,
            trading_mode,
        ) {
            Some(s) => s,
            None => continue,
        };

        let mut engine = BacktestEngine::new(100_000.0, 0.001, SlippageModel::FixedPercent(0.0005));
        engine.add_data(symbol, bars.clone());
        engine.set_strategy(strategy);

        if let Ok(metrics) = engine.run() {
            symbol_results.push(SymbolResult {
                symbol: symbol.clone(),
                sharpe_ratio: metrics.sharpe_ratio,
                total_return: metrics.total_return,
                max_drawdown: metrics.max_drawdown,
                total_trades: engine.portfolio.trades.len(),
            });
        }
    }

    info!(
        "Parameter sweep complete: best score = {:.3}, best sharpe = {:.3}",
        best_score, best_sharpe
    );

    Ok((
        sweep_results,
        best_params,
        best_score,
        best_sharpe,
        best_return,
        best_max_drawdown,
        best_trades,
        symbol_results,
    ))
}

/// Run walk-forward validation with random symbol selection per window
fn run_multi_symbol_walk_forward(
    symbol_data: &[(String, Vec<alphafield_core::Bar>)],
    strategy_name: &str,
    params: &HashMap<String, f64>,
    train_window: usize,
    test_window: usize,
    step_size: usize,
    trading_mode: TradingMode,
) -> Result<alphafield_backtest::WalkForwardResult, String> {
    use alphafield_backtest::{BacktestEngine, SlippageModel};
    use rand::seq::SliceRandom;

    if symbol_data.is_empty() {
        return Err("No symbol data available".to_string());
    }

    let min_required = train_window + test_window;

    // Filter symbols with enough data
    let valid_symbols: Vec<_> = symbol_data
        .iter()
        .filter(|(_, bars)| bars.len() >= min_required)
        .collect();

    if valid_symbols.is_empty() {
        return Err(format!(
            "No symbols have enough data (need {} bars)",
            min_required
        ));
    }

    let mut rng = rand::thread_rng();
    let mut windows: Vec<alphafield_backtest::walk_forward::WindowResult> = Vec::new();
    let mut window_idx = 0;

    // Walk through time, randomly selecting symbols
    while let Some((symbol, bars)) = valid_symbols.choose(&mut rng) {
        let start_idx = window_idx * step_size;
        let train_end = start_idx + train_window;
        let test_end = train_end + test_window;

        if test_end > bars.len() {
            break;
        }

        // Split data
        let train_data = &bars[start_idx..train_end];
        let test_data = &bars[train_end..test_end];

        // Run train backtest
        let train_metrics = {
            let strategy = match StrategyFactory::create_backtest(
                strategy_name,
                params,
                symbol,
                100_000.0,
                trading_mode,
            ) {
                Some(s) => s,
                None => {
                    window_idx += 1;
                    continue;
                }
            };

            let mut engine =
                BacktestEngine::new(100_000.0, 0.001, SlippageModel::FixedPercent(0.0005));
            engine.add_data(symbol, train_data.to_vec());
            engine.set_strategy(strategy);

            match engine.run() {
                Ok(m) => m,
                Err(_) => {
                    window_idx += 1;
                    continue;
                }
            }
        };

        // Run test backtest
        let test_metrics = {
            let strategy = match StrategyFactory::create_backtest(
                strategy_name,
                params,
                symbol,
                100_000.0,
                trading_mode,
            ) {
                Some(s) => s,
                None => {
                    window_idx += 1;
                    continue;
                }
            };

            let mut engine =
                BacktestEngine::new(100_000.0, 0.001, SlippageModel::FixedPercent(0.0005));
            engine.add_data(symbol, test_data.to_vec());
            engine.set_strategy(strategy);

            match engine.run() {
                Ok(m) => m,
                Err(_) => {
                    window_idx += 1;
                    continue;
                }
            }
        };

        windows.push(alphafield_backtest::walk_forward::WindowResult {
            window_index: window_idx,
            train_start: start_idx,
            train_end,
            test_start: train_end,
            test_end,
            train_metrics,
            test_metrics,
        });

        window_idx += 1;

        // Limit to reasonable number of windows
        if windows.len() >= 20 {
            break;
        }
    }

    if windows.is_empty() {
        return Err("Could not create any walk-forward windows".to_string());
    }

    // Calculate aggregate metrics
    let test_returns: Vec<f64> = windows
        .iter()
        .map(|w| w.test_metrics.total_return)
        .collect();
    let test_sharpes: Vec<f64> = windows
        .iter()
        .map(|w| w.test_metrics.sharpe_ratio)
        .collect();
    let test_drawdowns: Vec<f64> = windows
        .iter()
        .map(|w| w.test_metrics.max_drawdown)
        .collect();
    let test_win_rates: Vec<f64> = windows.iter().map(|w| w.test_metrics.win_rate).collect();

    let mean_return = test_returns.iter().sum::<f64>() / test_returns.len() as f64;
    let mean_sharpe = test_sharpes.iter().sum::<f64>() / test_sharpes.len() as f64;
    let worst_drawdown = test_drawdowns.iter().cloned().fold(0.0, f64::max);
    let win_rate = test_win_rates.iter().sum::<f64>() / test_win_rates.len() as f64;

    // Calculate median return
    let mut sorted_returns = test_returns.clone();
    sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_return = if sorted_returns.len().is_multiple_of(2) {
        (sorted_returns[sorted_returns.len() / 2 - 1] + sorted_returns[sorted_returns.len() / 2])
            / 2.0
    } else {
        sorted_returns[sorted_returns.len() / 2]
    };

    // Calculate stability score (percentage of profitable windows)
    let profitable_windows = test_returns.iter().filter(|&&r| r > 0.0).count();
    let stability_score = (profitable_windows as f64 / windows.len() as f64) * 100.0;

    let aggregate_oos = alphafield_backtest::walk_forward::AggregateMetrics {
        mean_return,
        median_return,
        mean_sharpe,
        worst_drawdown,
        win_rate,
    };

    Ok(alphafield_backtest::WalkForwardResult {
        windows,
        aggregate_oos,
        stability_score,
    })
}

/// Calculate robustness score for multi-symbol optimization
fn calculate_multi_symbol_robustness(
    dispersion: &ParameterDispersion,
    walk_forward: &alphafield_backtest::WalkForwardResult,
) -> f64 {
    // Walk-forward stability (40%)
    let wf_score = walk_forward.stability_score;

    // Parameter dispersion (30%) - lower CV is better
    let dispersion_score = 100.0 * (1.0 - dispersion.sharpe_cv.min(1.0));

    // Out-of-sample performance (30%)
    let oos_sharpe = walk_forward.aggregate_oos.mean_sharpe;
    let oos_score = (oos_sharpe.clamp(-1.0, 3.0) + 1.0) * 25.0; // Maps [-1, 3] to [0, 100]

    let robustness = 0.4 * wf_score + 0.3 * dispersion_score + 0.3 * oos_score;
    robustness.clamp(0.0, 100.0)
}
