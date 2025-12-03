use crate::api::{create_router, AppState};
use alphafield_core::Result;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

pub async fn run_server(addr: &str) -> Result<()> {
    let state = Arc::new(AppState::new());

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
        .map_err(|e| alphafield_core::QuantError::Io(e))?;

    println!("🚀 Dashboard server running on http://{}", addr);
    println!("📊 Open your browser to view the dashboard");

    axum::serve(listener, app)
        .await
        .map_err(|e| alphafield_core::QuantError::Io(e))?;

    Ok(())
}
