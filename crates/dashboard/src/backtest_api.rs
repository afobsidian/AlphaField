use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

use alphafield_backtest::{
    AssetSentimentCalculator, AssetSentimentSummary,
    BacktestEngine, BenchmarkComparison, DrawdownAnalysis, DrawdownPoint,
    MonthlyReturn, PerformanceMetrics, RollingStats, SlippageModel, StrategyAdapter, Trade,
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
    pub returns: f64,       // Period return
    pub cumulative: f64,    // Cumulative return from start
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

        let avg_duration = trades.iter().map(|t| t.duration_secs).sum::<i64>() as f64 / total as f64 / 3600.0;
        
        let avg_mae = trades.iter().map(|t| {
            if t.entry_price > 0.0 { t.mae / t.entry_price * 100.0 } else { 0.0 }
        }).sum::<f64>() / total as f64;
        
        let avg_mfe = trades.iter().map(|t| {
            if t.entry_price > 0.0 { t.mfe / t.entry_price * 100.0 } else { 0.0 }
        }).sum::<f64>() / total as f64;

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

    history.iter().map(|(ts, equity)| {
        let returns = if prev > 0.0 { (*equity - prev) / prev } else { 0.0 };
        let cumulative = if initial > 0.0 { (*equity - initial) / initial } else { 0.0 };
        prev = *equity;
        EquityPoint {
            timestamp: *ts,
            equity: *equity,
            returns,
            cumulative,
        }
    }).collect()
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
            .unwrap_or_else(|| Utc::now())
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
    }).await.map_err(|e| e.to_string()).and_then(|res| res);

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
        let mut engine = BacktestEngine::new(
            100_000.0,
            0.001,
            SlippageModel::FixedPercent(0.0005),
        );

        engine.add_data(&req.symbol, bars.clone());

        if let Some(strategy) = StrategyFactory::create(&req.strategy, &req.params) {
            let adapter = StrategyAdapter::new(
                strategy,
                &req.symbol,
                0.5,
            );
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

    // 6. Benchmark Comparison (BTC buy-and-hold)
    let benchmark = if req.include_benchmark || true {
        let benchmark_curve = BenchmarkComparison::calculate_buy_and_hold(&bars, 100_000.0);
        if !benchmark_curve.is_empty() {
            let comparison = BenchmarkComparison::calculate(&equity_history, &benchmark_curve, 0.02);
            Some(BenchmarkData {
                curve: build_equity_curve(&benchmark_curve),
                comparison,
            })
        } else {
            None
        }
    } else {
        None
    };


    // 7. Market Sentiment
    // Concurrent fetch logic would be ideal, but for now we sequentialize it to ensure stability
    // The days count is calculated from the range
    let sentiment_days = (end_time - start_time).num_days() as u32;
    // Spawn to avoid Handler trait bounds issues with complex async futures
    let market_sentiment = tokio::spawn(async move {
        fetch_market_sentiment(sentiment_days).await
    }).await.unwrap_or(None);

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
