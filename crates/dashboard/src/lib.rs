//! # AlphaField Dashboard
//!
//! Axum-based REST API and WebSocket server for real-time trading dashboard

pub mod analysis_api;
pub mod api;
pub mod backtest_api;
pub mod chart_api;
pub mod data_api;
pub mod ml_api;
pub mod mock_data;
pub mod orders_api;
pub mod quality_api;
pub mod reports_api;
pub mod sentiment_api;
pub mod server;
pub mod services;
pub mod strategies_api;
pub mod websocket;

pub use analysis_api::*;
pub use api::*;
pub use backtest_api::*;
pub use chart_api::*;
pub use data_api::*;
pub use mock_data::*;
pub use quality_api::*;
pub use reports_api::*;
pub use sentiment_api::*;
pub use server::*;
pub use strategies_api::*;
pub use websocket::*;
