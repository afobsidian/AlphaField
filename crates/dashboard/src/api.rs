use axum::{
    extract::State,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::mock_data::{
    generate_mock_orders, generate_mock_performance, generate_mock_portfolio,
    generate_mock_positions, PerformanceMetrics, Portfolio, Position,
};
use crate::reports_api::{
    add_journal_entry, calculate_tax, delete_journal_entry, export_report_csv, export_tax_csv,
    generate_summary, get_strategy_breakdown, list_journal, update_journal_entry,
};
use crate::websocket::DashboardHub;
use alphafield_core::Order;
use alphafield_data::DatabaseClient;

// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: Option<DatabaseClient>,
    pub hub: Arc<DashboardHub>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db: None,
            hub: Arc::new(DashboardHub::default()),
        }
    }

    pub async fn with_database() -> Self {
        let hub = Arc::new(DashboardHub::default());
        match DatabaseClient::new_from_env().await {
            Ok(db) => Self { db: Some(db), hub },
            Err(e) => {
                eprintln!("Warning: Could not connect to database: {}", e);
                Self { db: None, hub }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database: String,
}

// Health check endpoint
pub async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: if state.db.is_some() {
            "connected"
        } else {
            "disconnected"
        }
        .to_string(),
    })
}

// Get portfolio summary
pub async fn get_portfolio(State(_state): State<Arc<AppState>>) -> Json<Portfolio> {
    Json(generate_mock_portfolio())
}

// Get active positions
pub async fn get_positions(State(_state): State<Arc<AppState>>) -> Json<Vec<Position>> {
    Json(generate_mock_positions())
}

// Get order history
pub async fn get_orders(State(_state): State<Arc<AppState>>) -> Json<Vec<Order>> {
    Json(generate_mock_orders())
}

// Get performance metrics
pub async fn get_performance(State(_state): State<Arc<AppState>>) -> Json<PerformanceMetrics> {
    Json(generate_mock_performance())
}

use crate::analysis_api::{
    calculate_correlation, run_monte_carlo, run_sensitivity, run_walk_forward,
};
use crate::backtest_api::{
    optimize_params, run_backtest, run_multi_symbol_workflow, run_optimization_workflow,
};
use crate::bots_api::{
    create_dca_bot, create_grid_bot, create_trailing_order, delete_dca_bot, delete_grid_bot,
    delete_trailing_order, get_bot_status, list_dca_bots, list_grid_bots, list_trailing_orders,
    pause_dca_bot, start_dca_bot, start_grid_bot, start_trailing_order, stop_dca_bot,
    stop_grid_bot, stop_trailing_order,
};
use crate::chart_api::get_chart_data;
use crate::data_api::{delete_symbol, fetch_symbol, get_trading_pairs, list_symbols};
use crate::ml_api::{delete_model, list_models, train_model, train_multi_symbol, validate_model};
use crate::quality_api::{check_freshness, check_gaps, check_outliers, get_quality_summary};
use crate::sentiment_api::{get_current_sentiment, get_sentiment_history, sync_sentiment_data};
use crate::websocket::websocket_handler;

// Build the API router
pub fn create_router(state: Arc<AppState>) -> Router {
    // Clone hub for WebSocket route
    let hub = state.hub.clone();

    Router::new()
        // Health
        .route("/api/health", get(health))
        // WebSocket for real-time updates
        .route(
            "/api/ws",
            get(move |ws| websocket_handler(ws, axum::extract::State(hub.clone()))),
        )
        // Live trading (mock for now)
        .route("/api/portfolio", get(get_portfolio))
        .route("/api/positions", get(get_positions))
        .route("/api/orders", get(get_orders))
        .route("/api/performance", get(get_performance))
        // Backtesting
        .route("/api/backtest/run", post(run_backtest))
        .route("/api/backtest/optimize", post(optimize_params))
        .route("/api/backtest/workflow", post(run_optimization_workflow))
        .route(
            "/api/backtest/workflow/multi",
            post(run_multi_symbol_workflow),
        )
        // Analysis
        .route("/api/monte-carlo", post(run_monte_carlo))
        .route("/api/correlation", post(calculate_correlation))
        .route("/api/sensitivity", post(run_sensitivity))
        .route("/api/walk-forward", post(run_walk_forward))
        // Data management
        .route("/api/data/symbols", get(list_symbols))
        .route("/api/data/pairs", get(get_trading_pairs))
        .route("/api/data/fetch", post(fetch_symbol))
        .route("/api/data/:symbol/:interval", delete(delete_symbol))
        // Chart data with indicators
        .route("/api/chart/ohlcv", post(get_chart_data))
        // Data quality
        .route("/api/quality/gaps/:symbol/:interval", get(check_gaps))
        .route(
            "/api/quality/outliers/:symbol/:interval",
            get(check_outliers),
        )
        .route("/api/quality/freshness", get(check_freshness))
        .route("/api/quality/summary", get(get_quality_summary))
        // Sentiment analysis
        .route("/api/sentiment/current", get(get_current_sentiment))
        .route("/api/sentiment/history", get(get_sentiment_history))
        .route("/api/sentiment/sync", post(sync_sentiment_data))
        // ML models
        .route("/api/ml/train", post(train_model))
        .route("/api/ml/train/multi", post(train_multi_symbol))
        .route("/api/ml/models", get(list_models))
        .route("/api/ml/models/:id", delete(delete_model))
        .route("/api/ml/validate", post(validate_model))
        // Reports
        .route("/api/reports/summary", post(generate_summary))
        .route("/api/reports/strategy", post(get_strategy_breakdown))
        .route("/api/reports/export", post(export_report_csv))
        // Journal
        .route("/api/journal", get(list_journal).post(add_journal_entry))
        .route(
            "/api/journal/:id",
            axum::routing::put(update_journal_entry).delete(delete_journal_entry),
        )
        // Tax
        .route("/api/tax/calculate", post(calculate_tax))
        .route("/api/tax/export", post(export_tax_csv))
        // Bots - DCA
        .route("/api/bots/dca", get(list_dca_bots).post(create_dca_bot))
        .route("/api/bots/dca/:id/start", post(start_dca_bot))
        .route("/api/bots/dca/:id/pause", post(pause_dca_bot))
        .route("/api/bots/dca/:id/stop", post(stop_dca_bot))
        .route("/api/bots/dca/:id", delete(delete_dca_bot))
        // Bots - Grid
        .route("/api/bots/grid", get(list_grid_bots).post(create_grid_bot))
        .route("/api/bots/grid/:id/start", post(start_grid_bot))
        .route("/api/bots/grid/:id/stop", post(stop_grid_bot))
        .route("/api/bots/grid/:id", delete(delete_grid_bot))
        // Bots - Trailing
        .route(
            "/api/bots/trailing",
            get(list_trailing_orders).post(create_trailing_order),
        )
        .route("/api/bots/trailing/:id/start", post(start_trailing_order))
        .route("/api/bots/trailing/:id/stop", post(stop_trailing_order))
        .route("/api/bots/trailing/:id", delete(delete_trailing_order))
        // Bots - Status
        .route("/api/bots/status", get(get_bot_status))
        .with_state(state)
}
