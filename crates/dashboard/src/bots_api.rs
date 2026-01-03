//! # Bots API
//!
//! REST API endpoints for automated trading bot management.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use alphafield_core::OrderSide;
use alphafield_execution::{
    AmountType, BotStats, BotStatus, DCABot, DCAConfig, Frequency, GridBot, GridConfig, GridLevel,
    TradingBot, TrailingConfig, TrailingOrder, TrailingType,
};

use crate::api::AppState;

// =============================================================================
// Bot Registry - In-memory bot management
// =============================================================================

/// Registry to manage active bots
#[derive(Default)]
pub struct BotRegistry {
    dca_bots: Arc<RwLock<HashMap<String, DCABot>>>,
    grid_bots: Arc<RwLock<HashMap<String, GridBot>>>,
    trailing_orders: Arc<RwLock<HashMap<String, TrailingOrder>>>,
}

impl BotRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}

// Global bot registry (for simplicity in Phase 14)
lazy_static::lazy_static! {
    static ref BOT_REGISTRY: BotRegistry = BotRegistry::new();
}

// =============================================================================
// DCA Bot API
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDCABotRequest {
    pub symbol: String,
    pub amount_type: AmountType,
    pub frequency: Frequency,
    pub max_price: Option<f64>,
    pub total_budget: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DCABotResponse {
    pub id: String,
    pub config: DCAConfig,
    pub status: BotStatus,
    pub stats: BotStats,
    pub total_spent: f64,
    pub next_execution: Option<chrono::DateTime<chrono::Utc>>,
}

/// Create a new DCA bot
pub async fn create_dca_bot(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateDCABotRequest>,
) -> Result<Json<DCABotResponse>, StatusCode> {
    let config = DCAConfig {
        symbol: req.symbol,
        amount_type: req.amount_type,
        frequency: req.frequency,
        max_price: req.max_price,
        total_budget: req.total_budget,
    };

    let bot = DCABot::new(config);
    let id = bot.id().to_string();

    let response = DCABotResponse {
        id: id.clone(),
        config: bot.config().clone(),
        status: bot.status(),
        stats: bot.stats(),
        total_spent: bot.total_spent(),
        next_execution: bot.next_execution(),
    };

    BOT_REGISTRY.dca_bots.write().unwrap().insert(id, bot);

    Ok(Json(response))
}

/// List all DCA bots
pub async fn list_dca_bots(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<DCABotResponse>>, StatusCode> {
    let bots = BOT_REGISTRY.dca_bots.read().unwrap();

    let responses: Vec<DCABotResponse> = bots
        .iter()
        .map(|(id, bot)| DCABotResponse {
            id: id.clone(),
            config: bot.config().clone(),
            status: bot.status(),
            stats: bot.stats(),
            total_spent: bot.total_spent(),
            next_execution: bot.next_execution(),
        })
        .collect();

    Ok(Json(responses))
}

/// Start a DCA bot
pub async fn start_dca_bot(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<DCABotResponse>, StatusCode> {
    let mut bots = BOT_REGISTRY.dca_bots.write().unwrap();

    if let Some(bot) = bots.get_mut(&id) {
        bot.start().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(DCABotResponse {
            id: id.clone(),
            config: bot.config().clone(),
            status: bot.status(),
            stats: bot.stats(),
            total_spent: bot.total_spent(),
            next_execution: bot.next_execution(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Pause a DCA bot
pub async fn pause_dca_bot(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<DCABotResponse>, StatusCode> {
    let mut bots = BOT_REGISTRY.dca_bots.write().unwrap();

    if let Some(bot) = bots.get_mut(&id) {
        bot.pause().map_err(|_| StatusCode::BAD_REQUEST)?;

        Ok(Json(DCABotResponse {
            id: id.clone(),
            config: bot.config().clone(),
            status: bot.status(),
            stats: bot.stats(),
            total_spent: bot.total_spent(),
            next_execution: bot.next_execution(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Stop a DCA bot
pub async fn stop_dca_bot(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<DCABotResponse>, StatusCode> {
    let mut bots = BOT_REGISTRY.dca_bots.write().unwrap();

    if let Some(bot) = bots.get_mut(&id) {
        bot.stop().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(DCABotResponse {
            id: id.clone(),
            config: bot.config().clone(),
            status: bot.status(),
            stats: bot.stats(),
            total_spent: bot.total_spent(),
            next_execution: bot.next_execution(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Delete a DCA bot
pub async fn delete_dca_bot(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut bots = BOT_REGISTRY.dca_bots.write().unwrap();

    if bots.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// =============================================================================
// Grid Bot API
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGridBotRequest {
    pub symbol: String,
    pub lower_price: f64,
    pub upper_price: f64,
    pub grid_levels: u32,
    pub total_capital: f64,
    pub min_profit_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GridBotResponse {
    pub id: String,
    pub config: GridConfig,
    pub status: BotStatus,
    pub stats: BotStats,
    pub grid_levels: Vec<GridLevel>,
    pub total_profit: f64,
}

/// Create a new Grid bot
pub async fn create_grid_bot(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateGridBotRequest>,
) -> Result<Json<GridBotResponse>, StatusCode> {
    let config = GridConfig {
        symbol: req.symbol,
        lower_price: req.lower_price,
        upper_price: req.upper_price,
        grid_levels: req.grid_levels,
        total_capital: req.total_capital,
        min_profit_percent: req.min_profit_percent,
    };

    let bot = GridBot::new(config).map_err(|_| StatusCode::BAD_REQUEST)?;
    let id = bot.id().to_string();

    let response = GridBotResponse {
        id: id.clone(),
        config: bot.config().clone(),
        status: bot.status(),
        stats: bot.stats(),
        grid_levels: bot.grid_levels(),
        total_profit: bot.total_grid_profit(),
    };

    BOT_REGISTRY.grid_bots.write().unwrap().insert(id, bot);

    Ok(Json(response))
}

/// List all Grid bots
pub async fn list_grid_bots(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<GridBotResponse>>, StatusCode> {
    let bots = BOT_REGISTRY.grid_bots.read().unwrap();

    let responses: Vec<GridBotResponse> = bots
        .iter()
        .map(|(id, bot)| GridBotResponse {
            id: id.clone(),
            config: bot.config().clone(),
            status: bot.status(),
            stats: bot.stats(),
            grid_levels: bot.grid_levels(),
            total_profit: bot.total_grid_profit(),
        })
        .collect();

    Ok(Json(responses))
}

/// Start a Grid bot
pub async fn start_grid_bot(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<GridBotResponse>, StatusCode> {
    let mut bots = BOT_REGISTRY.grid_bots.write().unwrap();

    if let Some(bot) = bots.get_mut(&id) {
        bot.start().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(GridBotResponse {
            id: id.clone(),
            config: bot.config().clone(),
            status: bot.status(),
            stats: bot.stats(),
            grid_levels: bot.grid_levels(),
            total_profit: bot.total_grid_profit(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Stop a Grid bot
pub async fn stop_grid_bot(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<GridBotResponse>, StatusCode> {
    let mut bots = BOT_REGISTRY.grid_bots.write().unwrap();

    if let Some(bot) = bots.get_mut(&id) {
        bot.stop().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(GridBotResponse {
            id: id.clone(),
            config: bot.config().clone(),
            status: bot.status(),
            stats: bot.stats(),
            grid_levels: bot.grid_levels(),
            total_profit: bot.total_grid_profit(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Delete a Grid bot
pub async fn delete_grid_bot(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut bots = BOT_REGISTRY.grid_bots.write().unwrap();

    if bots.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// =============================================================================
// Trailing Order API
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTrailingOrderRequest {
    pub symbol: String,
    pub trailing_type: TrailingType,
    pub side: OrderSide,
    pub quantity: f64,
    pub trailing_percent: f64,
    pub activation_price: Option<f64>,
    pub limit_price: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrailingOrderResponse {
    pub id: String,
    pub config: TrailingConfig,
    pub status: BotStatus,
    pub stats: BotStats,
    pub trigger_price: Option<f64>,
    pub extreme_price: Option<f64>,
    pub activated: bool,
    pub triggered: bool,
}

/// Create a new Trailing order
pub async fn create_trailing_order(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateTrailingOrderRequest>,
) -> Result<Json<TrailingOrderResponse>, StatusCode> {
    let config = TrailingConfig {
        symbol: req.symbol,
        trailing_type: req.trailing_type,
        side: req.side,
        quantity: req.quantity,
        trailing_percent: req.trailing_percent,
        activation_price: req.activation_price,
        limit_price: req.limit_price,
    };

    let order = TrailingOrder::new(config).map_err(|_| StatusCode::BAD_REQUEST)?;
    let id = order.id().to_string();

    let response = TrailingOrderResponse {
        id: id.clone(),
        config: order.config().clone(),
        status: order.status(),
        stats: order.stats(),
        trigger_price: order.trigger_price(),
        extreme_price: order.extreme_price(),
        activated: order.is_activated(),
        triggered: order.is_triggered(),
    };

    BOT_REGISTRY
        .trailing_orders
        .write()
        .unwrap()
        .insert(id, order);

    Ok(Json(response))
}

/// List all Trailing orders
pub async fn list_trailing_orders(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<TrailingOrderResponse>>, StatusCode> {
    let orders = BOT_REGISTRY.trailing_orders.read().unwrap();

    let responses: Vec<TrailingOrderResponse> = orders
        .iter()
        .map(|(id, order)| TrailingOrderResponse {
            id: id.clone(),
            config: order.config().clone(),
            status: order.status(),
            stats: order.stats(),
            trigger_price: order.trigger_price(),
            extreme_price: order.extreme_price(),
            activated: order.is_activated(),
            triggered: order.is_triggered(),
        })
        .collect();

    Ok(Json(responses))
}

/// Start a Trailing order
pub async fn start_trailing_order(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<TrailingOrderResponse>, StatusCode> {
    let mut orders = BOT_REGISTRY.trailing_orders.write().unwrap();

    if let Some(order) = orders.get_mut(&id) {
        order
            .start()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(TrailingOrderResponse {
            id: id.clone(),
            config: order.config().clone(),
            status: order.status(),
            stats: order.stats(),
            trigger_price: order.trigger_price(),
            extreme_price: order.extreme_price(),
            activated: order.is_activated(),
            triggered: order.is_triggered(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Stop a Trailing order
pub async fn stop_trailing_order(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<TrailingOrderResponse>, StatusCode> {
    let mut orders = BOT_REGISTRY.trailing_orders.write().unwrap();

    if let Some(order) = orders.get_mut(&id) {
        order
            .stop()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(TrailingOrderResponse {
            id: id.clone(),
            config: order.config().clone(),
            status: order.status(),
            stats: order.stats(),
            trigger_price: order.trigger_price(),
            extreme_price: order.extreme_price(),
            activated: order.is_activated(),
            triggered: order.is_triggered(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Delete a Trailing order
pub async fn delete_trailing_order(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut orders = BOT_REGISTRY.trailing_orders.write().unwrap();

    if orders.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// =============================================================================
// Bot Status Overview
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct BotStatusOverview {
    pub total_bots: usize,
    pub active_bots: usize,
    pub paused_bots: usize,
    pub completed_bots: usize,
    pub dca_count: usize,
    pub grid_count: usize,
    pub trailing_count: usize,
}

/// Get overview of all bot statuses
pub async fn get_bot_status(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<BotStatusOverview>, StatusCode> {
    let dca_bots = BOT_REGISTRY.dca_bots.read().unwrap();
    let grid_bots = BOT_REGISTRY.grid_bots.read().unwrap();
    let trailing_orders = BOT_REGISTRY.trailing_orders.read().unwrap();

    let mut active = 0;
    let mut paused = 0;
    let mut completed = 0;

    for bot in dca_bots.values() {
        match bot.status() {
            BotStatus::Active => active += 1,
            BotStatus::Paused => paused += 1,
            BotStatus::Completed => completed += 1,
            _ => {}
        }
    }

    for bot in grid_bots.values() {
        match bot.status() {
            BotStatus::Active => active += 1,
            BotStatus::Paused => paused += 1,
            BotStatus::Completed => completed += 1,
            _ => {}
        }
    }

    for order in trailing_orders.values() {
        match order.status() {
            BotStatus::Active => active += 1,
            BotStatus::Paused => paused += 1,
            BotStatus::Completed => completed += 1,
            _ => {}
        }
    }

    let total = dca_bots.len() + grid_bots.len() + trailing_orders.len();

    Ok(Json(BotStatusOverview {
        total_bots: total,
        active_bots: active,
        paused_bots: paused,
        completed_bots: completed,
        dca_count: dca_bots.len(),
        grid_count: grid_bots.len(),
        trailing_count: trailing_orders.len(),
    }))
}
