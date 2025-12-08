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
use crate::websocket::DashboardHub;
use alphafield_core::Order;
use alphafield_data::DatabaseClient;

// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: Option<DatabaseClient>,
    pub hub: Arc<DashboardHub>,
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
        database: if state.db.is_some() { "connected" } else { "disconnected" }.to_string(),
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

use crate::backtest_api::run_backtest;
use crate::data_api::{delete_symbol, fetch_symbol, get_trading_pairs, list_symbols};
use crate::websocket::websocket_handler;
use crate::analysis_api::{run_monte_carlo, calculate_correlation, run_sensitivity};
use crate::quality_api::{check_freshness, check_gaps, check_outliers, get_quality_summary};
use crate::sentiment_api::{get_current_sentiment, get_sentiment_history, sync_sentiment_data};

// Build the API router
pub fn create_router(state: Arc<AppState>) -> Router {
    // Clone hub for WebSocket route
    let hub = state.hub.clone();
    
    Router::new()
        // Health
        .route("/api/health", get(health))
        // WebSocket for real-time updates
        .route("/api/ws", get(move |ws| websocket_handler(ws, axum::extract::State(hub.clone()))))
        // Live trading (mock for now)
        .route("/api/portfolio", get(get_portfolio))
        .route("/api/positions", get(get_positions))
        .route("/api/orders", get(get_orders))
        .route("/api/performance", get(get_performance))
        // Backtesting
        .route("/api/backtest/run", post(run_backtest))
        // Analysis
        .route("/api/monte-carlo", post(run_monte_carlo))
        .route("/api/correlation", post(calculate_correlation))
        .route("/api/sensitivity", post(run_sensitivity))
        // Data management
        .route("/api/data/symbols", get(list_symbols))
        .route("/api/data/pairs", get(get_trading_pairs))
        .route("/api/data/fetch", post(fetch_symbol))
        .route("/api/data/:symbol/:interval", delete(delete_symbol))
        // Data quality
        .route("/api/quality/gaps/:symbol/:interval", get(check_gaps))
        .route("/api/quality/outliers/:symbol/:interval", get(check_outliers))
        .route("/api/quality/freshness", get(check_freshness))
        .route("/api/quality/summary", get(get_quality_summary))
        // Sentiment analysis
        .route("/api/sentiment/current", get(get_current_sentiment))
        .route("/api/sentiment/history", get(get_sentiment_history))
        .route("/api/sentiment/sync", post(sync_sentiment_data))
        .with_state(state)
}
