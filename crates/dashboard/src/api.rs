use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::mock_data::{
    generate_mock_orders, generate_mock_performance, generate_mock_portfolio,
    generate_mock_positions, PerformanceMetrics, Portfolio, Position,
};
use alphafield_core::Order;

// Application state
#[derive(Clone)]
pub struct AppState {
    // In a real app, this would hold connections to databases,
    // execution services, etc.
}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

// Health check endpoint
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
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

// Build the API router
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/portfolio", get(get_portfolio))
        .route("/api/positions", get(get_positions))
        .route("/api/orders", get(get_orders))
        .route("/api/performance", get(get_performance))
        .with_state(state)
}
