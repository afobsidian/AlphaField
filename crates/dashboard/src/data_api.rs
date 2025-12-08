use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use alphafield_data::{CachedSymbol, UnifiedDataClient};

use crate::api::AppState;

#[derive(Deserialize)]
pub struct FetchRequest {
    pub symbol: String,
    pub interval: String,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct FetchResponse {
    pub success: bool,
    pub message: String,
    pub bars_fetched: usize,
}

/// List all cached symbols in the database
pub async fn list_symbols(State(state): State<Arc<AppState>>) -> Json<Vec<CachedSymbol>> {
    let db = match &state.db {
        Some(db) => db,
        None => return Json(vec![]),
    };

    match db.list_symbols().await {
        Ok(symbols) => Json(symbols),
        Err(_) => Json(vec![]),
    }
}

/// Get available trading pairs from Binance
pub async fn get_trading_pairs() -> Json<Vec<String>> {
    let client = UnifiedDataClient::new_from_env();
    
    match client.get_exchange_info().await {
        Ok(pairs) => Json(pairs),
        Err(_) => {
            // Return fallback popular pairs if API fails
            Json(vec![
                "BTC".to_string(), "ETH".to_string(), "SOL".to_string(),
                "XRP".to_string(), "ADA".to_string(), "DOGE".to_string(),
                "AVAX".to_string(), "DOT".to_string(), "LINK".to_string(),
                "MATIC".to_string(), "UNI".to_string(), "ATOM".to_string(),
            ])
        }
    }
}

/// Fetch new data for a symbol
pub async fn fetch_symbol(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FetchRequest>,
) -> Json<FetchResponse> {
    let db = match &state.db {
        Some(db) => db,
        None => {
            return Json(FetchResponse {
                success: false,
                message: "Database not connected".to_string(),
                bars_fetched: 0,
            })
        }
    };

    // Fetch from API
    let client = UnifiedDataClient::new_from_env();
    let limit = req.limit.unwrap_or(1000);

    match client.get_bars(&req.symbol, &req.interval, None, None, Some(limit)).await {
        Ok(bars) => {
            let count = bars.len();
            // Save to DB
            if let Err(e) = db.save_bars(&req.symbol, &req.interval, &bars).await {
                return Json(FetchResponse {
                    success: false,
                    message: format!("Failed to save: {}", e),
                    bars_fetched: 0,
                });
            }
            Json(FetchResponse {
                success: true,
                message: format!("Fetched {} bars for {}", count, req.symbol),
                bars_fetched: count,
            })
        }
        Err(e) => Json(FetchResponse {
            success: false,
            message: format!("Failed to fetch: {}", e),
            bars_fetched: 0,
        }),
    }
}

/// Delete data for a symbol
pub async fn delete_symbol(
    State(state): State<Arc<AppState>>,
    Path((symbol, interval)): Path<(String, String)>,
) -> Json<FetchResponse> {
    let db = match &state.db {
        Some(db) => db,
        None => {
            return Json(FetchResponse {
                success: false,
                message: "Database not connected".to_string(),
                bars_fetched: 0,
            })
        }
    };

    match db.delete_bars(&symbol, &interval).await {
        Ok(_) => Json(FetchResponse {
            success: true,
            message: format!("Deleted {} {}", symbol, interval),
            bars_fetched: 0,
        }),
        Err(e) => Json(FetchResponse {
            success: false,
            message: format!("Failed to delete: {}", e),
            bars_fetched: 0,
        }),
    }
}
