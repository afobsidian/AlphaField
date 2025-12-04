use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use alphafield_backtest::{BacktestEngine, PerformanceMetrics, SlippageModel, StrategyAdapter};
use alphafield_core::Strategy;
use alphafield_data::UnifiedDataClient;
use alphafield_strategy::{GoldenCrossStrategy, MeanReversionStrategy, MomentumStrategy, RsiStrategy};

use crate::api::AppState;

#[derive(Deserialize)]
pub struct BacktestRequest {
    pub strategy: String,
    pub symbol: String,
    pub interval: String,
    pub days: u32,
    pub params: HashMap<String, f64>,
}

#[derive(Serialize)]
pub struct BacktestResponse {
    pub metrics: PerformanceMetrics,
    pub equity_curve: Vec<(i64, f64)>, // Timestamp (ms), Equity
}

struct StrategyFactory;

impl StrategyFactory {
    fn create(name: &str, params: &HashMap<String, f64>) -> Option<Box<dyn Strategy>> {
        match name {
            "GoldenCross" => {
                let fast = params.get("fast_period").copied().unwrap_or(10.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(30.0) as usize;
                Some(Box::new(GoldenCrossStrategy::new(fast, slow)))
            }
            "Rsi" => {
                let period = params.get("period").copied().unwrap_or(14.0) as usize;
                let lower = params.get("lower_bound").copied().unwrap_or(30.0);
                let upper = params.get("upper_bound").copied().unwrap_or(70.0);
                Some(Box::new(RsiStrategy::new(period, lower, upper)))
            }
            "MeanReversion" => {
                let period = params.get("period").copied().unwrap_or(20.0) as usize;
                let std_dev = params.get("std_dev").copied().unwrap_or(2.0);
                Some(Box::new(MeanReversionStrategy::new(period, std_dev)))
            }
            "Momentum" => {
                let ema_period = params.get("ema_period").copied().unwrap_or(50.0) as usize;
                let macd_fast = params.get("macd_fast").copied().unwrap_or(12.0) as usize;
                let macd_slow = params.get("macd_slow").copied().unwrap_or(26.0) as usize;
                let macd_signal = params.get("macd_signal").copied().unwrap_or(9.0) as usize;
                Some(Box::new(MomentumStrategy::new(ema_period, macd_fast, macd_slow, macd_signal)))
            }
            _ => None,
        }
    }
}

pub async fn run_backtest(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BacktestRequest>,
) -> Json<BacktestResponse> {
    // 1. Fetch Data
    let client = UnifiedDataClient::new_from_env();
    // Calculate number of bars based on interval (simplified approximation)
    let hours_per_day = 24;
    let limit = match req.interval.as_str() {
        "1h" => req.days * hours_per_day,
        "1d" => req.days,
        _ => req.days * hours_per_day, // Default to hourly count
    };

    let bars = match client.get_bars(&req.symbol, &req.interval, Some(limit)).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to fetch data: {}", e);
            return Json(BacktestResponse {
                metrics: PerformanceMetrics::default(),
                equity_curve: vec![],
            });
        }
    };

    if bars.is_empty() {
        return Json(BacktestResponse {
            metrics: PerformanceMetrics::default(),
            equity_curve: vec![],
        });
    }

    // 2. Setup Engine
    let mut engine = BacktestEngine::new(
        100_000.0,                           // Initial Cash
        0.001,                               // 0.1% Fee
        SlippageModel::FixedPercent(0.0005), // 0.05% Slippage
    );

    engine.add_data(&req.symbol, bars.clone());

    // 3. Create Strategy
    if let Some(strategy) = StrategyFactory::create(&req.strategy, &req.params) {
        // Wrap in adapter to handle sizing/execution logic
        let adapter = StrategyAdapter::new(
            strategy,
            &req.symbol,
            0.5, // Fixed position size ratio for now
        );
        engine.set_strategy(Box::new(adapter));
    } else {
        eprintln!("Unknown strategy: {}", req.strategy);
        return Json(BacktestResponse {
            metrics: PerformanceMetrics::default(),
            equity_curve: vec![],
        });
    }

    // 4. Run Backtest
    let metrics = match engine.run() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Backtest failed: {}", e);
            return Json(BacktestResponse {
                metrics: PerformanceMetrics::default(),
                equity_curve: vec![],
            });
        }
    };

    // 5. Format Results
    let equity_curve = engine
        .portfolio
        .equity_history
        .iter()
        .map(|(ts, val)| (*ts, *val))
        .collect();

    Json(BacktestResponse {
        metrics,
        equity_curve,
    })
}
