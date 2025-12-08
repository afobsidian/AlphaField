//! Analysis API endpoints for Monte Carlo, sensitivity, and correlation analysis

use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument, warn};

use alphafield_backtest::{
    monte_carlo::{MonteCarloConfig, MonteCarloSimulator, Trade as McTrade},
    sensitivity::HeatmapData,
    CorrelationAnalyzer, CorrelationConfig, CorrelationResult, MonteCarloResult,
};

use crate::api::AppState;

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
    Json(_req): Json<SensitivityRequest>,
) -> Json<SensitivityResponse> {
    // Note: Full sensitivity analysis requires fetching data and creating strategies
    // This is a placeholder that shows the API structure
    // In production, this would:
    // 1. Fetch historical data
    // 2. Run grid search over parameters
    // 3. Return results with heatmap

    Json(SensitivityResponse {
        success: false,
        results: vec![],
        best_params: None,
        heatmap: None,
        error: Some("Sensitivity analysis via API not yet implemented - use programmatic API".to_string()),
    })
}
