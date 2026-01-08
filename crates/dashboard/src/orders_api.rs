//! Orders API Module
//!
//! Provides REST endpoints for advanced order management including:
//! - OCO (One-Cancels-Other) orders
//! - Bracket orders (entry + SL + TP)
//! - Iceberg orders (split large orders)
//! - Limit chase orders
//! - Position management (scale in/out, partial TP, break-even stop)
//! - Order queue management

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use alphafield_core::{
    BracketOrder, IcebergOrder, LimitChaseOrder, OcoOrder, Order, OrderSide, OrderStatus, OrderType,
};
use alphafield_execution::{
    CreateBracketRequest, CreateIcebergRequest, CreateLimitChaseRequest, CreateOcoRequest,
    OrderQueueSummary, PartialTakeProfitRequest, ScalePositionRequest,
};

use crate::api::AppState;

// ==================== Order Queue API ====================

/// Get pending orders (optionally filtered by symbol)
pub async fn get_pending_orders(
    State(_state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<GetPendingOrdersQuery>,
) -> impl IntoResponse {
    info!("Getting pending orders, symbol={:?}", params.symbol);

    // In a real implementation, this would query the OrderManager
    // For now, return empty list
    Json(Vec::<Order>::new())
}

#[derive(Debug, Deserialize)]
pub struct GetPendingOrdersQuery {
    pub symbol: Option<String>,
}

/// Get order queue summary
pub async fn get_order_queue(
    State(_state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<GetQueueQuery>,
) -> impl IntoResponse {
    info!("Getting order queue summary");

    // In a real implementation, this would return actual queue data
    let summaries = if let Some(symbol) = params.symbol {
        vec![OrderQueueSummary {
            symbol,
            pending_count: 0,
            total_qty: 0.0,
            orders: Vec::new(),
        }]
    } else {
        Vec::new()
    };

    Json(summaries)
}

#[derive(Debug, Deserialize)]
pub struct GetQueueQuery {
    pub symbol: Option<String>,
}

/// Modify an existing order
pub async fn modify_order(
    Path(order_id): Path<String>,
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ModifyOrderRequest>,
) -> impl IntoResponse {
    info!("Modifying order {}: {:?}", order_id, req);

    // In a real implementation, this would call OrderManager::modify_order
    // For now, return a mock response
    Json(serde_json::json!({
        "status": "success",
        "message": "Order modified",
        "order_id": order_id
    }))
}

#[derive(Debug, Deserialize)]
pub struct ModifyOrderRequest {
    pub symbol: String,
    pub new_price: Option<f64>,
    pub new_quantity: Option<f64>,
}

/// Cancel a single order
pub async fn cancel_order(
    Path(order_id): Path<String>,
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CancelOrderRequest>,
) -> impl IntoResponse {
    info!("Canceling order {} for symbol {}", order_id, req.symbol);

    // In a real implementation, this would call OrderManager::cancel_order
    StatusCode::OK
}

#[derive(Debug, Deserialize)]
pub struct CancelOrderRequest {
    pub symbol: String,
}

/// Cancel all orders for a symbol
pub async fn cancel_all_orders(
    Path(symbol): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Canceling all orders for symbol {}", symbol);

    // In a real implementation, this would call OrderManager::cancel_all_orders
    Json(serde_json::json!({
        "status": "success",
        "message": format!("All orders canceled for {}", symbol),
        "symbol": symbol
    }))
}

// ==================== OCO Order API ====================

/// Create an OCO (One-Cancels-Other) order
pub async fn create_oco_order(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateOcoRequest>,
) -> impl IntoResponse {
    info!("Creating OCO order with {} child orders", req.orders.len());

    // Parse orders from JSON values
    let mut orders = Vec::new();
    for order_json in req.orders {
        match serde_json::from_value::<Order>(order_json) {
            Ok(order) => orders.push(order),
            Err(e) => {
                error!("Failed to parse order: {}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": "Invalid order format"})),
                )
                    .into_response();
            }
        }
    }

    if orders.len() < 2 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "OCO requires at least 2 orders"})),
        )
            .into_response();
    }

    let oco = OcoOrder::new(orders);

    // In a real implementation, this would call OrderManager::submit_oco
    Json(serde_json::json!({
        "status": "success",
        "group_id": oco.group_id,
        "message": "OCO order created"
    }))
    .into_response()
}

/// Get active OCO orders
pub async fn get_oco_orders(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Getting active OCO orders");

    // In a real implementation, this would query OrderManager
    Json(Vec::<OcoOrder>::new())
}

/// Cancel an OCO order group
pub async fn cancel_oco_order(
    Path(group_id): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Canceling OCO order group {}", group_id);

    // In a real implementation, this would call OrderManager::cancel_oco
    Json(serde_json::json!({
        "status": "success",
        "message": format!("OCO group {} canceled", group_id)
    }))
}

// ==================== Bracket Order API ====================

/// Create a bracket order
pub async fn create_bracket_order(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateBracketRequest>,
) -> impl IntoResponse {
    info!(
        "Creating bracket order: {} SL={} TP={}",
        req.symbol, req.stop_loss, req.take_profit
    );

    // Parse entry order
    let entry_order: Order = match serde_json::from_value(req.entry) {
        Ok(order) => order,
        Err(e) => {
            error!("Failed to parse entry order: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid entry order format"})),
            )
                .into_response();
        }
    };

    // Validate order quantities
    let sl_qty = req.stop_loss_qty.unwrap_or(entry_order.quantity);
    let tp_qty = req.take_profit_qty.unwrap_or(entry_order.quantity);

    // Create stop-loss order
    let stop_loss_order = Order {
        id: uuid::Uuid::new_v4().to_string(),
        symbol: req.symbol.clone(),
        side: OrderSide::Sell,
        order_type: OrderType::Limit,
        quantity: sl_qty,
        price: Some(req.stop_loss),
        status: OrderStatus::New,
        timestamp: chrono::Utc::now(),
        oco_group_id: None,
        iceberg_hidden_qty: None,
        stop_loss: None,
        take_profit: None,
        parent_order_id: None,
        limit_chase_amount: None,
    };

    // Create take-profit order
    let take_profit_order = Order {
        id: uuid::Uuid::new_v4().to_string(),
        symbol: req.symbol.clone(),
        side: OrderSide::Sell,
        order_type: OrderType::Limit,
        quantity: tp_qty,
        price: Some(req.take_profit),
        status: OrderStatus::New,
        timestamp: chrono::Utc::now(),
        oco_group_id: None,
        iceberg_hidden_qty: None,
        stop_loss: None,
        take_profit: None,
        parent_order_id: None,
        limit_chase_amount: None,
    };

    let bracket = BracketOrder::new(entry_order, stop_loss_order, take_profit_order);

    // In a real implementation, this would call OrderManager::submit_bracket
    Json(serde_json::json!({
        "status": "success",
        "bracket_id": bracket.bracket_id,
        "message": "Bracket order created"
    }))
    .into_response()
}

/// Get active bracket orders
pub async fn get_bracket_orders(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Getting active bracket orders");

    // In a real implementation, this would query OrderManager
    Json(Vec::<BracketOrder>::new())
}

/// Cancel a bracket order
pub async fn cancel_bracket_order(
    Path(bracket_id): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Canceling bracket order {}", bracket_id);

    // In a real implementation, this would call OrderManager::cancel_bracket
    Json(serde_json::json!({
        "status": "success",
        "message": format!("Bracket order {} canceled", bracket_id)
    }))
}

// ==================== Iceberg Order API ====================

/// Create an iceberg order
pub async fn create_iceberg_order(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateIcebergRequest>,
) -> impl IntoResponse {
    info!(
        "Creating iceberg order: {} Total={} Visible={}",
        req.symbol, req.total_quantity, req.visible_quantity
    );

    // Parse side
    let side = match req.side.to_lowercase().as_str() {
        "buy" => OrderSide::Buy,
        "sell" => OrderSide::Sell,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid side, must be 'buy' or 'sell'"})),
            )
                .into_response();
        }
    };

    // Validate quantities
    if req.visible_quantity <= 0.0 || req.total_quantity <= 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Quantities must be positive"})),
        )
            .into_response();
    }

    if req.visible_quantity > req.total_quantity {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Visible quantity cannot exceed total"})),
        )
            .into_response();
    }

    let iceberg = IcebergOrder::new(
        req.symbol.clone(),
        side,
        req.price,
        req.total_quantity,
        req.visible_quantity,
    );

    // In a real implementation, this would call OrderManager::submit_iceberg
    Json(serde_json::json!({
        "status": "success",
        "iceberg_id": iceberg.iceberg_id,
        "message": "Iceberg order created"
    }))
    .into_response()
}

/// Get active iceberg orders
pub async fn get_iceberg_orders(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Getting active iceberg orders");

    // In a real implementation, this would query OrderManager
    Json(Vec::<IcebergOrder>::new())
}

/// Cancel an iceberg order
pub async fn cancel_iceberg_order(
    Path(iceberg_id): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Canceling iceberg order {}", iceberg_id);

    // In a real implementation, this would call OrderManager::cancel_iceberg
    Json(serde_json::json!({
        "status": "success",
        "message": format!("Iceberg order {} canceled", iceberg_id)
    }))
}

// ==================== Limit Chase Order API ====================

/// Create a limit chase order
pub async fn create_limit_chase_order(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateLimitChaseRequest>,
) -> impl IntoResponse {
    info!(
        "Creating limit chase order: chase_amount={} max_adj={}",
        req.chase_amount, req.max_adjustments
    );

    // Parse order
    let order: Order = match serde_json::from_value(req.order) {
        Ok(order) => order,
        Err(e) => {
            error!("Failed to parse order: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid order format"})),
            )
                .into_response();
        }
    };

    // Validate order type
    if order.order_type != OrderType::Limit {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Limit chase only works with limit orders"})),
        )
            .into_response();
    }

    let chase = LimitChaseOrder::new(
        order,
        req.chase_amount,
        req.is_percentage,
        req.max_adjustments,
    );

    // In a real implementation, this would call OrderManager::submit_limit_chase
    Json(serde_json::json!({
        "status": "success",
        "chase_id": chase.chase_id,
        "message": "Limit chase order created"
    }))
    .into_response()
}

/// Get active limit chase orders
pub async fn get_limit_chase_orders(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Getting active limit chase orders");

    // In a real implementation, this would query OrderManager
    Json(Vec::<LimitChaseOrder>::new())
}

/// Cancel a limit chase order
pub async fn cancel_limit_chase_order(
    Path(chase_id): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Canceling limit chase order {}", chase_id);

    // In a real implementation, this would call OrderManager::cancel_limit_chase
    Json(serde_json::json!({
        "status": "success",
        "message": format!("Limit chase order {} canceled", chase_id)
    }))
}

// ==================== Position Management API ====================

/// Scale into a position (add to existing)
pub async fn scale_in_position(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ScalePositionRequest>,
) -> impl IntoResponse {
    info!(
        "Scaling into {} position: {} units",
        req.symbol, req.quantity
    );

    if req.quantity <= 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Quantity must be positive for scale-in"})),
        )
            .into_response();
    }

    // In a real implementation, this would call OrderManager::scale_in
    Json(serde_json::json!({
        "status": "success",
        "message": format!("Scaled into {} position", req.symbol)
    }))
    .into_response()
}

/// Scale out of a position (partial close)
pub async fn scale_out_position(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ScalePositionRequest>,
) -> impl IntoResponse {
    info!(
        "Scaling out of {} position: {} units",
        req.symbol, req.quantity
    );

    if req.quantity >= 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Quantity must be negative for scale-out"})),
        )
            .into_response();
    }

    // In a real implementation, this would call OrderManager::scale_out
    Json(serde_json::json!({
        "status": "success",
        "message": format!("Scaled out of {} position", req.symbol)
    }))
    .into_response()
}

/// Partial take-profit with scaling
pub async fn partial_take_profit(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PartialTakeProfitRequest>,
) -> impl IntoResponse {
    info!(
        "Partial TP for {}: Close {} @ {}, Keep {}",
        req.symbol, req.close_qty, req.target_price, req.remaining_qty
    );

    if req.close_qty <= 0.0 || req.remaining_qty < 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid quantities"})),
        )
            .into_response();
    }

    if req.target_price <= 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Target price must be positive"})),
        )
            .into_response();
    }

    // In a real implementation, this would call OrderManager::partial_take_profit
    Json(serde_json::json!({
        "status": "success",
        "message": format!("Partial TP submitted for {}", req.symbol)
    }))
    .into_response()
}

/// Request to move stop-loss to break-even
#[derive(Debug, Deserialize)]
pub struct BreakEvenStopRequest {
    pub symbol: String,
    pub entry_price: f64,
    pub stop_offset: f64,
}

/// Move stop-loss to break-even
pub async fn break_even_stop(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BreakEvenStopRequest>,
) -> impl IntoResponse {
    info!(
        "Moving stop-loss to break-even for {}: {} + {}",
        req.symbol, req.entry_price, req.stop_offset
    );

    if req.entry_price <= 0.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Entry price must be positive"})),
        )
            .into_response();
    }

    // In a real implementation, this would call OrderManager::break_even_stop
    Json(serde_json::json!({
        "status": "success",
        "message": format!("Break-even stop set for {}", req.symbol)
    }))
    .into_response()
}

// ==================== Response Types ====================

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub status: String,
    pub order_id: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_oco_request_validation() {
        let req = json!({
            "orders": [
                {
                    "id": "1",
                    "symbol": "BTCUSDT",
                    "side": "Buy",
                    "order_type": "Limit",
                    "quantity": 1.0,
                    "price": 50000.0,
                    "status": "New",
                    "timestamp": "2024-01-01T00:00:00Z"
                },
                {
                    "id": "2",
                    "symbol": "BTCUSDT",
                    "side": "Sell",
                    "order_type": "Limit",
                    "quantity": 1.0,
                    "price": 51000.0,
                    "status": "New",
                    "timestamp": "2024-01-01T00:00:00Z"
                }
            ]
        });

        let create_req: CreateOcoRequest = serde_json::from_value(req).unwrap();
        assert_eq!(create_req.orders.len(), 2);
    }

    #[test]
    fn test_create_bracket_request_validation() {
        let req = json!({
            "entry": {
                "id": "entry",
                "symbol": "BTCUSDT",
                "side": "Buy",
                "order_type": "Market",
                "quantity": 1.0,
                "status": "New",
                "timestamp": "2024-01-01T00:00:00Z"
            },
            "stop_loss": 49000.0,
            "take_profit": 51000.0,
            "symbol": "BTCUSDT"
        });

        let create_req: CreateBracketRequest = serde_json::from_value(req).unwrap();
        assert_eq!(create_req.stop_loss, 49000.0);
        assert_eq!(create_req.take_profit, 51000.0);
        assert_eq!(create_req.symbol, "BTCUSDT");
    }

    #[test]
    fn test_create_iceberg_request_validation() {
        let req = json!({
            "symbol": "BTCUSDT",
            "side": "Buy",
            "total_quantity": 10.0,
            "visible_quantity": 2.5,
            "price": 50000.0
        });

        let create_req: CreateIcebergRequest = serde_json::from_value(req).unwrap();
        assert_eq!(create_req.symbol, "BTCUSDT");
        assert_eq!(create_req.side, "Buy");
        assert_eq!(create_req.total_quantity, 10.0);
        assert_eq!(create_req.visible_quantity, 2.5);
    }

    #[test]
    fn test_create_limit_chase_request_validation() {
        let req = json!({
            "order": {
                "id": "1",
                "symbol": "BTCUSDT",
                "side": "Buy",
                "order_type": "Limit",
                "quantity": 1.0,
                "price": 50000.0,
                "status": "New",
                "timestamp": "2024-01-01T00:00:00Z"
            },
            "chase_amount": 1.0,
            "is_percentage": true,
            "max_adjustments": 5
        });

        let create_req: CreateLimitChaseRequest = serde_json::from_value(req).unwrap();
        assert_eq!(create_req.chase_amount, 1.0);
        assert!(create_req.is_percentage);
        assert_eq!(create_req.max_adjustments, 5);
    }
}
