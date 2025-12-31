use crate::api::{create_router, AppState};
use crate::websocket::start_heartbeat_task;
use alphafield_core::Result;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

pub async fn run_server(addr: &str) -> Result<()> {
    // Load .env with fallback parsing for non-standard format
    if dotenvy::dotenv().is_err() {
        if let Ok(contents) = std::fs::read_to_string(".env") {
            for line in contents.lines() {
                if line.starts_with("DATABASE_URL=") {
                    std::env::set_var("DATABASE_URL", line.trim_start_matches("DATABASE_URL="));
                    break;
                }
            }
        }
    }

    let state = Arc::new(AppState::with_database().await);

    // Start heartbeat background task
    let _heartbeat_handle = start_heartbeat_task(state.hub.clone());

    // Create API router
    let api_router = create_router(state);

    // Combine with static file serving
    let app = Router::new()
        .merge(api_router)
        .nest_service("/", ServeDir::new("crates/dashboard/static"))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(alphafield_core::QuantError::Io)?;

    println!("🚀 Dashboard server running on http://{}", addr);
    println!("📊 Open your browser to view the dashboard");
    println!("🔌 WebSocket available at ws://{}/api/ws", addr);

    axum::serve(listener, app)
        .await
        .map_err(alphafield_core::QuantError::Io)?;

    Ok(())
}
