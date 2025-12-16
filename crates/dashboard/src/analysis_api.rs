//! Analysis API endpoints for Monte Carlo, sensitivity, and correlation analysis

use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument, warn};

use alphafield_backtest::{
    monte_carlo::{MonteCarloConfig, MonteCarloSimulator, Trade as McTrade},
    sensitivity::{HeatmapData, SensitivityAnalyzer, SensitivityConfig, ParameterRange},
    walk_forward::{WalkForwardAnalyzer, WalkForwardConfig, WalkForwardResult},
    CorrelationAnalyzer, CorrelationConfig, CorrelationResult, MonteCarloResult, StrategyAdapter,
};

use crate::api::AppState;
use crate::services::data_service::fetch_data_with_cache;
use crate::services::strategy_service::StrategyFactory;
use alphafield_backtest::strategy::Strategy as BacktestStrategy;
use chrono::{Duration, Utc};
use std::collections::HashMap;

// ============= Monte Carlo API =============

#[derive(Debug, Deserialize)]
pub struct MonteCarloRequest {
    /// List of trades (pnl, return_pct, duration)
    pub trades: Vec<TradeInput>,
    /// Number of simulations (default: 1000)
    pub num_simulations: Option<usize>,
    /// Initial capital (default: 10000)
    pub initial_capital: Option<f64>,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TradeInput {
    pub symbol: String,
    pub pnl: f64,
    pub return_pct: f64,
    pub duration: usize,
}

#[derive(Serialize)]
pub struct MonteCarloResponse {
    pub success: bool,
    pub result: Option<MonteCarloResult>,
    pub error: Option<String>,
}

#[instrument(skip(_state), fields(trade_count = req.trades.len(), simulations = req.num_simulations))]
pub async fn run_monte_carlo(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<MonteCarloRequest>,
) -> Json<MonteCarloResponse> {
    info!("Monte Carlo API request received");
    
    if req.trades.is_empty() {
        warn!("No trades provided for Monte Carlo simulation");
        return Json(MonteCarloResponse {
            success: false,
            result: None,
            error: Some("No trades provided".to_string()),
        });
    }

    let config = MonteCarloConfig {
        num_simulations: req.num_simulations.unwrap_or(1000),
        initial_capital: req.initial_capital.unwrap_or(10000.0),
        seed: req.seed,
    };

    let trades: Vec<McTrade> = req
        .trades
        .iter()
        .map(|t| McTrade {
            symbol: t.symbol.clone(),
            pnl: t.pnl,
            return_pct: t.return_pct,
            duration: t.duration,
        })
        .collect();

    let simulator = MonteCarloSimulator::new(config);
    let result = simulator.simulate(&trades);

    info!(simulations = result.num_simulations, "Monte Carlo simulation completed");

    Json(MonteCarloResponse {
        success: true,
        result: Some(result),
        error: None,
    })
}

// ============= Correlation API =============

#[derive(Debug, Deserialize)]
pub struct CorrelationRequest {
    /// List of equity curves with labels
    pub curves: Vec<EquityCurveInput>,
    /// Alert threshold (default: 0.7)
    pub alert_threshold: Option<f64>,
    /// Minimum data points (default: 30)
    pub min_data_points: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct EquityCurveInput {
    pub label: String,
    pub values: Vec<f64>,
}

#[derive(Serialize)]
pub struct CorrelationResponse {
    pub success: bool,
    pub result: Option<CorrelationResult>,
    pub error: Option<String>,
}

#[instrument(skip(_state), fields(curve_count = req.curves.len()))]
pub async fn calculate_correlation(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CorrelationRequest>,
) -> Json<CorrelationResponse> {
    info!("Correlation API request received");
    
    if req.curves.len() < 2 {
        warn!("Insufficient curves for correlation analysis");
        return Json(CorrelationResponse {
            success: false,
            result: None,
            error: Some("Need at least 2 equity curves".to_string()),
        });
    }

    let config = CorrelationConfig {
        alert_threshold: req.alert_threshold.unwrap_or(0.7),
        min_data_points: req.min_data_points.unwrap_or(30),
    };

    let curves: Vec<(String, Vec<f64>)> = req
        .curves
        .iter()
        .map(|c| (c.label.clone(), c.values.clone()))
        .collect();

    let analyzer = CorrelationAnalyzer::new(config);
    match analyzer.analyze_equity_curves(&curves) {
        Ok(result) => {
            info!(
                avg_correlation = format!("{:.2}", result.average_correlation),
                alerts = result.alerts.len(),
                "Correlation analysis completed"
            );
            Json(CorrelationResponse {
                success: true,
                result: Some(result),
                error: None,
            })
        }
        Err(e) => {
            warn!(error = %e, "Correlation analysis failed");
            Json(CorrelationResponse {
                success: false,
                result: None,
                error: Some(e),
            })
        }
    }
}

// ============= Sensitivity API =============

#[derive(Deserialize)]
pub struct SensitivityRequest {
    /// Strategy name to analyze
    pub strategy: String,
    /// Symbol to backtest
    pub symbol: String,
    /// Interval (e.g., "1h", "1d")
    pub interval: String,
    /// Number of days of data
    pub days: u32,
    /// Parameter to sweep (name, min, max, step)
    pub param: ParameterInput,
    /// Optional second parameter for 2D analysis
    pub param_y: Option<ParameterInput>,
    /// Any fixed parameters to include
    #[serde(default)]
    pub fixed_params: HashMap<String, f64>,
}

#[derive(Deserialize)]
pub struct ParameterInput {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

#[derive(Serialize)]
pub struct SensitivityResponse {
    pub success: bool,
    pub results: Vec<ParameterResultOutput>,
    pub best_params: Option<std::collections::HashMap<String, f64>>,
    pub heatmap: Option<HeatmapData>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ParameterResultOutput {
    pub params: std::collections::HashMap<String, f64>,
    pub sharpe_ratio: f64,
    pub total_return: f64,
    pub max_drawdown: f64,
}

pub async fn run_sensitivity(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SensitivityRequest>,
) -> Json<SensitivityResponse> {
    info!("Sensitivity Analysis requested for {} {}", req.symbol, req.strategy);

    // 1. Fetch Data
    let days = if req.days > 0 { req.days } else { 180 };
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(days as i64);

    let (bars, _) = match fetch_data_with_cache(
        req.symbol.clone(),
        req.interval.clone(),
        start_time,
        end_time
    ).await {
        Ok(res) => res,
        Err(e) => return Json(SensitivityResponse {
            success: false,
            results: vec![],
            best_params: None,
            heatmap: None,
            error: Some(format!("Failed to fetch data: {}", e)),
        }),
    };

    if bars.len() < 100 {
         return Json(SensitivityResponse {
            success: false,
            results: vec![],
            best_params: None,
            heatmap: None,
            error: Some("Insufficient data for sensitivity analysis".to_string()),
        });
    }

    // 2. Configure Analyzer
    let config = SensitivityConfig {
        initial_capital: 100_000.0,
        fee_rate: 0.001,
        parallel: true,
    };
    let analyzer = SensitivityAnalyzer::new(config);

    // 3. Run Analysis
    let strategy_name = req.strategy.clone();
    let fixed_params = req.fixed_params.clone();
    let req_symbol = req.symbol.clone();
    
    let range_x = ParameterRange::new(&req.param.name, req.param.min, req.param.max, req.param.step);
    
    let result_raw = if let Some(py) = &req.param_y {
        let range_y = ParameterRange::new(&py.name, py.min, py.max, py.step);
        let sym = req_symbol.clone();
        let s_name = strategy_name.clone();
        
        // Use create_backtest which returns properly wrapped strategies
        let factory_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            analyzer.analyze_2d(&bars, &req.symbol, &range_x, &range_y, |v1, v2| {
                let mut p = fixed_params.clone();
                p.insert(req.param.name.clone(), v1);
                p.insert(py.name.clone(), v2);
                
                // Use create_backtest for properly adapted strategies
                // If invalid params, return strategy with default params but mark as invalid or let it fail?
                // Better to return None and handle it in result processing
                StrategyFactory::create_backtest(&s_name, &p, &sym, 100_000.0)
            })
        }));
        
        match factory_result {
            Ok(res) => res,
            Err(_) => return Json(SensitivityResponse {
                success: false,
                results: vec![],
                best_params: None,
                heatmap: None,
                error: Some("Strategy creation failed - invalid parameter combination".to_string()),
            }),
        }
    } else {
        let sym = req_symbol.clone();
        let s_name = strategy_name.clone();
        
        let factory_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            analyzer.analyze_1d(&bars, &req.symbol, &range_x, |v1| {
                let mut p = fixed_params.clone();
                p.insert(req.param.name.clone(), v1);
                
                // Use create_backtest for properly adapted strategies
                StrategyFactory::create_backtest(&s_name, &p, &sym, 100_000.0)
            })
        }));
        
        match factory_result {
            Ok(res) => res,
            Err(_) => return Json(SensitivityResponse {
                success: false,
                results: vec![],
                best_params: None,
                heatmap: None,
                error: Some("Strategy creation failed - invalid parameter combination".to_string()),
            }),
        }
    };

    match result_raw {
        Ok(res) => {
            // Convert to response format
             let results: Vec<ParameterResultOutput> = res.results.iter().map(|r| {
                ParameterResultOutput {
                    params: r.params.clone(),
                    sharpe_ratio: r.metrics.sharpe_ratio,
                    total_return: r.metrics.total_return,
                    max_drawdown: r.metrics.max_drawdown,
                }
            }).collect();
            
            let best_params = res.best_sharpe.map(|r| r.params);

            Json(SensitivityResponse {
                success: true,
                results,
                best_params,
                heatmap: res.heatmap,
                error: None,
            })
        },
        Err(e) => Json(SensitivityResponse {
            success: false,
            results: vec![],
            best_params: None,
            heatmap: None,
            error: Some(e),
        })
    }
}
// ============= Walk-Forward API =============

#[derive(Deserialize)]
pub struct WalkForwardRequest {
    pub strategy: String,
    pub symbol: String,
    pub interval: String,
    pub params: HashMap<String, f64>,
    pub train_window_days: Option<usize>, // Default 365
    pub test_window_days: Option<usize>,  // Default 90
}

#[derive(Serialize)]
pub struct WalkForwardResponse {
    pub success: bool,
    pub result: Option<WalkForwardResult>,
    pub error: Option<String>,
}

pub async fn run_walk_forward(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<WalkForwardRequest>,
) -> Json<WalkForwardResponse> {
    info!("Walk-Forward Analysis requested for {}", req.symbol);

    // 1. Fetch 4 years of data (pagination now supported)
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(4 * 365);

    let (bars, data_status) = match fetch_data_with_cache(
        req.symbol.clone(),
        req.interval.clone(),
        start_time,
        end_time
    ).await {
        Ok(res) => res,
        Err(e) => return Json(WalkForwardResponse {
            success: false,
            result: None,
            error: Some(format!("Failed to fetch data: {}", e)),
        }),
    };

    info!(bars_fetched = bars.len(), source = %data_status.source, "Data fetch complete");

    // 2. Configure Analysis
    // Dynamic bars per day calculation
    let bars_per_day = match req.interval.as_str() {
        "1m" => 1440,
        "5m" => 288,
        "15m" => 96,
        "1h" => 24,
        "4h" => 6,
        "1d" => 1,
        _ => if req.interval.ends_with('h') { 24 } else { 1 }, // Fallback
    };
    
    let train_window_bars = req.train_window_days.unwrap_or(365) * bars_per_day;
    let test_window_bars = req.test_window_days.unwrap_or(90) * bars_per_day;
    let required_bars = train_window_bars + test_window_bars;

    if bars.len() < required_bars {
        return Json(WalkForwardResponse {
            success: false,
            result: None,
            error: Some(format!(
                "Insufficient data: have {} bars, need at least {} (train + test window). Fetch more data or reduce window sizes.",
                bars.len(), required_bars
            )),
        });
    }
    
    let config = WalkForwardConfig {
        train_window: train_window_bars,
        test_window: test_window_bars,
        step_size: 30 * bars_per_day, // 1 month step default
        initial_capital: 100_000.0,
        fee_rate: 0.001,
    };

    // 3. Create Strategy Factory
    let strategy_name = req.strategy.clone();
    let strategy_params = req.params.clone();
    let req_symbol = req.symbol.clone();
    
    // Safely handle strategy creation
    if StrategyFactory::create(&strategy_name, &strategy_params).is_none() {
         return Json(WalkForwardResponse {
            success: false,
            result: None,
            error: Some(format!("Invalid strategy parameters for {}", strategy_name)),
        });
    }
    
    let factory = move || -> Box<dyn BacktestStrategy> {
        // We verified it above, but factory runs in thread, so unwrap is "safer" but lets still be careful
        let core_strat = StrategyFactory::create(&strategy_name, &strategy_params)
            .expect("Strategy creation failed in factory execution");
        
        let adapter = StrategyAdapter::new(core_strat, req_symbol.clone(), 100_000.0);
        Box::new(adapter)
    };

    // 4. Run Analysis
    let analyzer = WalkForwardAnalyzer::new(config);
    match analyzer.analyze(&bars, &req.symbol, factory) {
        Ok(result) => Json(WalkForwardResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(WalkForwardResponse {
            success: false,
            result: None,
            error: Some(e),
        }),
    }
}
