use axum::{
    extract::State,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::mock_data::{
    generate_mock_orders, generate_mock_performance, generate_mock_portfolio,
    generate_mock_positions, PerformanceMetrics, Portfolio, Position,
};
use crate::orders_api::{
    break_even_stop, cancel_all_orders, cancel_bracket_order, cancel_iceberg_order,
    cancel_limit_chase_order, cancel_oco_order, cancel_order, create_bracket_order,
    create_iceberg_order, create_limit_chase_order, create_oco_order, get_bracket_orders,
    get_iceberg_orders, get_limit_chase_orders, get_oco_orders, get_order_queue,
    get_pending_orders, modify_order, partial_take_profit, scale_in_position, scale_out_position,
};
use crate::reports_api::{
    add_journal_entry, calculate_tax, delete_journal_entry, export_report_csv, export_tax_csv,
    generate_summary, get_strategy_breakdown, list_journal, update_journal_entry,
};
use crate::strategies_api::create_strategy_router;
use crate::websocket::DashboardHub;
use alphafield_core::Order;
use alphafield_data::DatabaseClient;

// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: Option<DatabaseClient>,
    pub hub: Arc<DashboardHub>,
    pub registry: Arc<alphafield_strategy::StrategyRegistry>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db: None,
            hub: Arc::new(DashboardHub::new(100)),
            registry: crate::strategies_api::initialize_registry(),
        }
    }

    pub async fn with_database() -> Self {
        let db = match DatabaseClient::new_from_env().await {
            Ok(db) => Some(db),
            Err(e) => {
                eprintln!("Failed to connect to database: {}", e);
                None
            }
        };

        Self {
            db,
            hub: Arc::new(DashboardHub::new(100)),
            registry: crate::strategies_api::initialize_registry(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
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
        // Strategy Management
        .merge(create_strategy_router())
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
        // Advanced Order Management
        .route("/api/orders/pending", get(get_pending_orders))
        .route("/api/orders/queue", get(get_order_queue))
        .route("/api/orders/:id", put(modify_order))
        .route("/api/orders/:id/cancel", post(cancel_order))
        .route("/api/orders/cancel-all/:symbol", post(cancel_all_orders))
        // OCO Orders
        .route("/api/orders/oco", post(create_oco_order))
        .route("/api/orders/oco", get(get_oco_orders))
        .route("/api/orders/oco/:group_id/cancel", post(cancel_oco_order))
        // Bracket Orders
        .route("/api/orders/bracket", post(create_bracket_order))
        .route("/api/orders/bracket", get(get_bracket_orders))
        .route(
            "/api/orders/bracket/:bracket_id/cancel",
            post(cancel_bracket_order),
        )
        // Iceberg Orders
        .route("/api/orders/iceberg", post(create_iceberg_order))
        .route("/api/orders/iceberg", get(get_iceberg_orders))
        .route(
            "/api/orders/iceberg/:iceberg_id/cancel",
            post(cancel_iceberg_order),
        )
        // Limit Chase Orders
        .route("/api/orders/limit-chase", post(create_limit_chase_order))
        .route("/api/orders/limit-chase", get(get_limit_chase_orders))
        .route(
            "/api/orders/limit-chase/:chase_id/cancel",
            post(cancel_limit_chase_order),
        )
        // Position Management
        .route("/api/positions/scale-in", post(scale_in_position))
        .route("/api/positions/scale-out", post(scale_out_position))
        .route("/api/positions/partial-tp", post(partial_take_profit))
        .route("/api/positions/break-even", post(break_even_stop))
        .with_state(state)
}
