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
use alphafield_core::Order;
use alphafield_data::DatabaseClient;

// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: Option<DatabaseClient>,
}

impl AppState {
    pub fn new() -> Self {
        Self { db: None }
    }

    pub async fn with_database() -> Self {
        match DatabaseClient::new_from_env().await {
            Ok(db) => Self { db: Some(db) },
            Err(e) => {
                eprintln!("Warning: Could not connect to database: {}", e);
                Self { db: None }
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
use crate::data_api::{delete_symbol, fetch_symbol, list_symbols};

// Build the API router
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health
        .route("/api/health", get(health))
        // Live trading (mock for now)
        .route("/api/portfolio", get(get_portfolio))
        .route("/api/positions", get(get_positions))
        .route("/api/orders", get(get_orders))
        .route("/api/performance", get(get_performance))
        // Backtesting
        .route("/api/backtest/run", post(run_backtest))
        // Data management
        .route("/api/data/symbols", get(list_symbols))
        .route("/api/data/fetch", post(fetch_symbol))
        .route("/api/data/:symbol/:interval", delete(delete_symbol))
        .with_state(state)
}
