//! # Advanced Order Management Module
//!
//! Provides sophisticated order types and management capabilities including:
//! - OCO (One-Cancels-Other) orders
//! - Bracket orders (entry + SL + TP)
//! - Iceberg orders (split large orders)
//! - Limit chase orders
//! - Position management (scale in/out, partial TP)
//! - Order queue management

use alphafield_core::{
    BracketOrder, BracketState, ExecutionService, IcebergOrder, LimitChaseOrder, OcoOrder, Order,
    OrderSide, OrderStatus, OrderType, QuantError, Result,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Manager for advanced order types
pub struct OrderManager<S: ExecutionService> {
    /// Underlying execution service
    service: Arc<S>,
    /// Active OCO orders
    oco_orders: Arc<RwLock<HashMap<String, OcoOrder>>>,
    /// Active bracket orders
    bracket_orders: Arc<RwLock<HashMap<String, BracketOrder>>>,
    /// Active iceberg orders
    iceberg_orders: Arc<RwLock<HashMap<String, IcebergOrder>>>,
    /// Active limit chase orders
    limit_chase_orders: Arc<RwLock<HashMap<String, LimitChaseOrder>>>,
    /// Order queue (pending orders by symbol)
    order_queue: Arc<RwLock<HashMap<String, Vec<Order>>>>,
}

impl<S: ExecutionService + 'static> OrderManager<S> {
    pub fn new(service: S) -> Self {
        Self {
            service: Arc::new(service),
            oco_orders: Arc::new(RwLock::new(HashMap::new())),
            bracket_orders: Arc::new(RwLock::new(HashMap::new())),
            iceberg_orders: Arc::new(RwLock::new(HashMap::new())),
            limit_chase_orders: Arc::new(RwLock::new(HashMap::new())),
            order_queue: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ==================== OCO Order Management ====================

    /// Submit an OCO order group
    pub async fn submit_oco(&self, oco: OcoOrder) -> Result<String> {
        info!("Submitting OCO order group: {}", oco.group_id);

        // Store the OCO group
        let group_id = oco.group_id.clone();
        self.oco_orders.write().await.insert(group_id.clone(), oco);

        // Submit all orders in the group
        let oco_guard = self.oco_orders.read().await;
        let oco_ref = oco_guard.get(&group_id).unwrap();

        let mut first_order_id = String::new();
        for order in &oco_ref.orders {
            let order_id = self.service.submit_order(order).await?;
            if first_order_id.is_empty() {
                first_order_id = order_id.clone();
            }
        }

        Ok(first_order_id)
    }

    /// Handle order fill for OCO group - cancel other orders
    pub async fn handle_oco_fill(&self, filled_order_id: &str) -> Result<()> {
        debug!("Checking OCO for filled order: {}", filled_order_id);

        // Find OCO group containing this order
        let oco_id = {
            let oco_guard = self.oco_orders.read().await;
            let mut found_id = None;
            for (group_id, oco) in oco_guard.iter() {
                if oco.active {
                    for order in &oco.orders {
                        if order.id == filled_order_id {
                            found_id = Some(group_id.clone());
                            break;
                        }
                    }
                }
                if found_id.is_some() {
                    break;
                }
            }
            found_id
        };

        if let Some(group_id) = oco_id {
            // Mark OCO as filled and cancel other orders
            let mut oco_guard = self.oco_orders.write().await;
            if let Some(oco) = oco_guard.get_mut(&group_id) {
                if oco.active {
                    oco.active = false;
                    oco.filled_order_id = Some(filled_order_id.to_string());

                    info!(
                        "OCO order {} filled, canceling {} other orders",
                        filled_order_id,
                        oco.orders.len() - 1
                    );

                    // Cancel all other orders in the group
                    for order in &oco.orders {
                        if order.id != filled_order_id {
                            if let Err(e) =
                                self.service.cancel_order(&order.id, &order.symbol).await
                            {
                                warn!("Failed to cancel OCO order {}: {}", order.id, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Cancel an OCO order group
    pub async fn cancel_oco(&self, group_id: &str) -> Result<()> {
        info!("Canceling OCO order group: {}", group_id);

        let mut oco_guard = self.oco_orders.write().await;
        if let Some(oco) = oco_guard.get(group_id) {
            if oco.active {
                // Cancel all orders in the group
                for order in &oco.orders {
                    if let Err(e) = self.service.cancel_order(&order.id, &order.symbol).await {
                        warn!("Failed to cancel OCO order {}: {}", order.id, e);
                    }
                }
            }
            // Mark as inactive
            let oco_mut = oco_guard.get_mut(group_id).unwrap();
            oco_mut.active = false;
        } else {
            return Err(QuantError::NotFound(format!(
                "OCO group {} not found",
                group_id
            )));
        }

        Ok(())
    }

    // ==================== Bracket Order Management ====================

    /// Submit a bracket order
    pub async fn submit_bracket(&self, bracket: BracketOrder) -> Result<String> {
        info!(
            "Submitting bracket order: {} (Entry: {} SL: {} TP: {})",
            bracket.bracket_id,
            bracket.entry_order.id,
            bracket.stop_loss_order.price.unwrap_or(0.0),
            bracket.take_profit_order.price.unwrap_or(0.0)
        );

        let bracket_id = bracket.bracket_id.clone();

        // Store the bracket order
        self.bracket_orders
            .write()
            .await
            .insert(bracket_id.clone(), bracket);

        // Submit entry order
        let bracket_guard = self.bracket_orders.read().await;
        let bracket_ref = bracket_guard.get(&bracket_id).unwrap();
        let entry_id = self.service.submit_order(&bracket_ref.entry_order).await?;

        // Clone orders before dropping the guard
        let stop_loss = bracket_ref.stop_loss_order.clone();
        let take_profit = bracket_ref.take_profit_order.clone();

        // In a real implementation, we'd wait for entry fill before submitting SL/TP
        // For now, we submit SL/TP with OCO behavior
        drop(bracket_guard);

        let oco = OcoOrder {
            group_id: format!("{}_sltp", bracket_id),
            orders: vec![stop_loss, take_profit],
            timestamp: Utc::now(),
            active: true,
            filled_order_id: None,
        };
        self.submit_oco(oco).await?;

        Ok(entry_id)
    }

    /// Handle bracket order state transitions
    pub async fn handle_bracket_fill(&self, order_id: &str) -> Result<()> {
        debug!("Checking bracket for filled order: {}", order_id);

        let bracket_id = {
            let bracket_guard = self.bracket_orders.read().await;
            let mut found_id = None;
            for (bracket_id, bracket) in bracket_guard.iter() {
                if bracket.entry_order.id == order_id {
                    found_id = Some(bracket_id.clone());
                    break;
                }
                if bracket.stop_loss_order.id == order_id {
                    found_id = Some(bracket_id.clone());
                    break;
                }
                if bracket.take_profit_order.id == order_id {
                    found_id = Some(bracket_id.clone());
                    break;
                }
            }
            found_id
        };

        if let Some(bracket_id) = bracket_id {
            let mut bracket_guard = self.bracket_orders.write().await;
            if let Some(bracket) = bracket_guard.get_mut(&bracket_id) {
                // Update bracket state based on which order filled
                if bracket.entry_order.id == order_id {
                    // Entry filled - SL/TP should now be active
                    if bracket.state == BracketState::EntryPending {
                        bracket.state = BracketState::Active;
                        info!("Bracket {} entry filled, SL/TP now active", bracket_id);
                    }
                } else if bracket.stop_loss_order.id == order_id {
                    // Stop loss filled
                    bracket.state = BracketState::StopLossFilled;
                    self.cancel_bracket_take_profit(&bracket_id).await?;
                    info!("Bracket {} stopped out", bracket_id);
                } else if bracket.take_profit_order.id == order_id {
                    // Take profit filled
                    bracket.state = BracketState::TakeProfitFilled;
                    self.cancel_bracket_stop_loss(&bracket_id).await?;
                    info!("Bracket {} took profit", bracket_id);
                }
            }
        }

        Ok(())
    }

    async fn cancel_bracket_take_profit(&self, bracket_id: &str) -> Result<()> {
        let bracket_guard = self.bracket_orders.read().await;
        if let Some(bracket) = bracket_guard.get(bracket_id) {
            if bracket.state == BracketState::Active {
                if let Err(e) = self
                    .service
                    .cancel_order(
                        &bracket.take_profit_order.id,
                        &bracket.take_profit_order.symbol,
                    )
                    .await
                {
                    warn!("Failed to cancel TP for bracket {}: {}", bracket_id, e);
                }
            }
        }
        Ok(())
    }

    async fn cancel_bracket_stop_loss(&self, bracket_id: &str) -> Result<()> {
        let bracket_guard = self.bracket_orders.read().await;
        if let Some(bracket) = bracket_guard.get(bracket_id) {
            if bracket.state == BracketState::Active {
                if let Err(e) = self
                    .service
                    .cancel_order(&bracket.stop_loss_order.id, &bracket.stop_loss_order.symbol)
                    .await
                {
                    warn!("Failed to cancel SL for bracket {}: {}", bracket_id, e);
                }
            }
        }
        Ok(())
    }

    // ==================== Iceberg Order Management ====================

    /// Submit an iceberg order
    pub async fn submit_iceberg(&self, iceberg: IcebergOrder) -> Result<String> {
        info!(
            "Submitting iceberg order: {} (Total: {}, Visible: {})",
            iceberg.iceberg_id, iceberg.total_quantity, iceberg.visible_quantity
        );

        let iceberg_id = iceberg.iceberg_id.clone();

        // Store the iceberg order
        self.iceberg_orders
            .write()
            .await
            .insert(iceberg_id.clone(), iceberg);

        // Submit first visible slice
        self.submit_next_iceberg_slice(&iceberg_id).await?;

        Ok(iceberg_id)
    }

    /// Submit the next slice of an iceberg order
    async fn submit_next_iceberg_slice(&self, iceberg_id: &str) -> Result<()> {
        let slice_order = {
            let iceberg_guard = self.iceberg_orders.read().await;
            let iceberg = iceberg_guard
                .get(iceberg_id)
                .ok_or_else(|| QuantError::NotFound(format!("Iceberg {} not found", iceberg_id)))?;

            if !iceberg.active || !iceberg.needs_more_slices() {
                return Ok(());
            }

            let slice_qty = iceberg.visible_quantity.min(iceberg.total_quantity);

            Order {
                id: uuid::Uuid::new_v4().to_string(),
                symbol: iceberg.symbol.clone(),
                side: iceberg.side,
                order_type: OrderType::Limit,
                quantity: slice_qty,
                price: Some(iceberg.price),
                status: OrderStatus::New,
                timestamp: iceberg.timestamp,
                oco_group_id: None,
                iceberg_hidden_qty: Some(iceberg.hidden_quantity()),
                stop_loss: None,
                take_profit: None,
                parent_order_id: Some(iceberg_id.to_string()),
                limit_chase_amount: None,
            }
        };

        let _order_id = self.service.submit_order(&slice_order).await?;
        Ok(())
    }

    /// Handle iceberg order slice fill
    pub async fn handle_iceberg_fill(&self, slice_order_id: &str) -> Result<()> {
        debug!("Handling iceberg slice fill: {}", slice_order_id);

        let iceberg_id = {
            let iceberg_guard = self.iceberg_orders.read().await;
            let mut found_id = None;

            for (iceberg_id, iceberg) in iceberg_guard.iter() {
                if iceberg.active {
                    // We can't directly match slice_order_id to stored iceberg
                    // In production, we'd track slice orders separately
                    // For now, assume any fill triggers next slice submission
                    found_id = Some(iceberg_id.clone());
                    break;
                }
            }
            found_id
        };

        if let Some(iceberg_id) = iceberg_id {
            // Submit next slice if needed
            self.submit_next_iceberg_slice(&iceberg_id).await?;
        }

        Ok(())
    }

    // ==================== Limit Chase Order Management ====================

    /// Submit a limit chase order
    pub async fn submit_limit_chase(&self, chase: LimitChaseOrder) -> Result<String> {
        info!(
            "Submitting limit chase order: {} (Max adjustments: {})",
            chase.chase_id, chase.max_adjustments
        );

        let chase_id = chase.chase_id.clone();

        // Store the limit chase order
        self.limit_chase_orders
            .write()
            .await
            .insert(chase_id.clone(), chase);

        // Submit initial order
        let chase_guard = self.limit_chase_orders.read().await;
        let chase_ref = chase_guard.get(&chase_id).unwrap();
        self.service.submit_order(&chase_ref.order).await
    }

    /// Update limit chase orders based on current market price
    pub async fn update_limit_chases(&self, symbol: &str, current_price: f64) -> Result<()> {
        let mut chase_guard = self.limit_chase_orders.write().await;
        let mut updated_chases = Vec::new();

        for (chase_id, chase) in chase_guard.iter_mut() {
            if !chase.active {
                continue;
            }

            if chase.order.symbol == symbol {
                if let Some(new_limit) = chase.calculate_new_limit(current_price) {
                    info!(
                        "Limit chase {} adjusting price: {} -> {}",
                        chase_id,
                        chase.order.price.unwrap_or(0.0),
                        new_limit
                    );

                    chase.update_limit(new_limit);

                    // Submit modified order
                    let order_id = chase.order.id.clone();
                    let order_symbol = chase.order.symbol.clone();
                    updated_chases.push((order_id, order_symbol, chase.order.clone()));
                }
            }
        }

        // Submit updated orders
        for (order_id, symbol, order) in updated_chases {
            if let Err(e) = self.service.cancel_order(&order_id, &symbol).await {
                warn!("Failed to cancel limit chase order {}: {}", order_id, e);
            }
            if let Err(e) = self.service.submit_order(&order).await {
                error!("Failed to resubmit limit chase order: {}", e);
            }
        }

        Ok(())
    }

    // ==================== Position Management ====================

    /// Scale into a position (add to existing)
    pub async fn scale_in(
        &self,
        symbol: &str,
        additional_qty: f64,
        price: Option<f64>,
    ) -> Result<String> {
        info!("Scaling into {} position: {} units", symbol, additional_qty);

        let order = Order {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: symbol.to_string(),
            side: OrderSide::Buy,
            order_type: if price.is_some() {
                OrderType::Limit
            } else {
                OrderType::Market
            },
            quantity: additional_qty,
            price,
            status: OrderStatus::New,
            timestamp: Utc::now(),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        self.service.submit_order(&order).await
    }

    /// Scale out of a position (partial close)
    pub async fn scale_out(
        &self,
        symbol: &str,
        reduce_qty: f64,
        price: Option<f64>,
        _reason: Option<String>,
    ) -> Result<String> {
        info!("Scaling out of {} position: {} units", symbol, reduce_qty);

        let order = Order {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: symbol.to_string(),
            side: OrderSide::Sell,
            order_type: if price.is_some() {
                OrderType::Limit
            } else {
                OrderType::Market
            },
            quantity: reduce_qty,
            price,
            status: OrderStatus::New,
            timestamp: Utc::now(),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        self.service.submit_order(&order).await
    }

    /// Partial take-profit with scaling
    pub async fn partial_take_profit(
        &self,
        symbol: &str,
        close_qty: f64,
        target_price: f64,
        remaining_qty: f64,
    ) -> Result<String> {
        info!(
            "Partial TP for {}: Close {} @ {}, Keep {}",
            symbol, close_qty, target_price, remaining_qty
        );

        // Submit partial close order
        let tp_order = Order {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: symbol.to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            quantity: close_qty,
            price: Some(target_price),
            status: OrderStatus::New,
            timestamp: Utc::now(),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        self.service.submit_order(&tp_order).await
    }

    /// Move stop-loss to break-even
    pub async fn break_even_stop(
        &self,
        symbol: &str,
        entry_price: f64,
        stop_offset: f64,
    ) -> Result<String> {
        info!(
            "Moving stop-loss to break-even for {}: {} + {}",
            symbol, entry_price, stop_offset
        );

        // In a real implementation, we'd cancel existing stop and submit new one
        let new_stop_price = entry_price + stop_offset;

        let stop_order = Order {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: symbol.to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            quantity: 0.0, // Would be set to position size
            price: Some(new_stop_price),
            status: OrderStatus::New,
            timestamp: Utc::now(),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        self.service.submit_order(&stop_order).await
    }

    // ==================== Order Queue Management ====================

    /// Get all pending orders in the queue
    pub async fn get_pending_orders(&self, symbol: Option<&str>) -> Vec<Order> {
        let queue_guard = self.order_queue.read().await;

        if let Some(sym) = symbol {
            queue_guard.get(sym).cloned().unwrap_or_default()
        } else {
            let mut all_orders = Vec::new();
            for orders in queue_guard.values() {
                all_orders.extend(orders.clone());
            }
            all_orders
        }
    }

    /// Add order to queue
    pub async fn add_to_queue(&self, order: Order) {
        let mut queue_guard = self.order_queue.write().await;
        queue_guard
            .entry(order.symbol.clone())
            .or_insert_with(Vec::new)
            .push(order);
    }

    /// Remove order from queue
    pub async fn remove_from_queue(&self, order_id: &str, symbol: &str) {
        let mut queue_guard = self.order_queue.write().await;
        if let Some(orders) = queue_guard.get_mut(symbol) {
            orders.retain(|o| o.id != order_id);
        }
    }

    /// Cancel all orders for a symbol
    pub async fn cancel_all_orders(&self, symbol: &str) -> Result<()> {
        info!("Canceling all orders for symbol: {}", symbol);

        // Cancel from queue
        {
            let mut queue_guard = self.order_queue.write().await;
            queue_guard.remove(symbol);
        }

        // Cancel from exchange
        if let Err(e) = self.service.cancel_all_orders(symbol).await {
            warn!("Failed to cancel all orders for {}: {}", symbol, e);
        }

        Ok(())
    }

    /// Modify an existing order
    pub async fn modify_order(
        &self,
        order_id: &str,
        symbol: &str,
        new_price: Option<f64>,
        new_quantity: Option<f64>,
    ) -> Result<Order> {
        info!(
            "Modifying order {}: price={:?}, qty={:?}",
            order_id, new_price, new_quantity
        );

        let modified_order = self
            .service
            .modify_order(order_id, symbol, new_price, new_quantity)
            .await?;

        // Update queue if present
        self.remove_from_queue(order_id, symbol).await;
        self.add_to_queue(modified_order.clone()).await;

        Ok(modified_order)
    }

    // ==================== Query Methods ====================

    /// Get active OCO orders
    pub async fn get_active_oco_orders(&self) -> Vec<OcoOrder> {
        let oco_guard = self.oco_orders.read().await;
        oco_guard
            .values()
            .filter(|oco| oco.active)
            .cloned()
            .collect()
    }

    /// Get active bracket orders
    pub async fn get_active_bracket_orders(&self) -> Vec<BracketOrder> {
        let bracket_guard = self.bracket_orders.read().await;
        bracket_guard
            .values()
            .filter(|b| b.state == BracketState::EntryPending || b.state == BracketState::Active)
            .cloned()
            .collect()
    }

    /// Get active iceberg orders
    pub async fn get_active_iceberg_orders(&self) -> Vec<IcebergOrder> {
        let iceberg_guard = self.iceberg_orders.read().await;
        iceberg_guard
            .values()
            .filter(|ice| ice.active)
            .cloned()
            .collect()
    }

    /// Get active limit chase orders
    pub async fn get_active_limit_chase_orders(&self) -> Vec<LimitChaseOrder> {
        let chase_guard = self.limit_chase_orders.read().await;
        chase_guard
            .values()
            .filter(|chase| chase.active)
            .cloned()
            .collect()
    }
}

// ==================== API Request/Response Types ====================

/// Request to create an OCO order
#[derive(Debug, serde::Deserialize)]
pub struct CreateOcoRequest {
    /// Orders to include in the OCO group
    pub orders: Vec<serde_json::Value>,
}

/// Request to create a bracket order
#[derive(Debug, serde::Deserialize)]
pub struct CreateBracketRequest {
    /// Entry order details
    pub entry: serde_json::Value,
    /// Stop-loss price
    pub stop_loss: f64,
    /// Stop-loss quantity (optional, defaults to entry qty)
    pub stop_loss_qty: Option<f64>,
    /// Take-profit price
    pub take_profit: f64,
    /// Take-profit quantity (optional, defaults to entry qty)
    pub take_profit_qty: Option<f64>,
    /// Symbol
    pub symbol: String,
}

/// Request to create an iceberg order
#[derive(Debug, serde::Deserialize)]
pub struct CreateIcebergRequest {
    /// Symbol to trade
    pub symbol: String,
    /// Side (Buy/Sell)
    pub side: String,
    /// Total quantity to trade
    pub total_quantity: f64,
    /// Visible display quantity per slice
    pub visible_quantity: f64,
    /// Limit price
    pub price: f64,
}

/// Request to create a limit chase order
#[derive(Debug, serde::Deserialize)]
pub struct CreateLimitChaseRequest {
    /// Order details
    pub order: serde_json::Value,
    /// Maximum chase amount
    pub chase_amount: f64,
    /// Whether chase amount is percentage (true) or absolute (false)
    pub is_percentage: bool,
    /// Maximum number of adjustments allowed
    pub max_adjustments: usize,
}

/// Request to scale position
#[derive(Debug, serde::Deserialize)]
pub struct ScalePositionRequest {
    /// Symbol
    pub symbol: String,
    /// Quantity to add/remove (positive = scale in, negative = scale out)
    pub quantity: f64,
    /// Optional price (None = market order)
    pub price: Option<f64>,
}

/// Request for partial take-profit
#[derive(Debug, serde::Deserialize)]
pub struct PartialTakeProfitRequest {
    /// Symbol
    pub symbol: String,
    /// Quantity to close
    pub close_qty: f64,
    /// Target price
    pub target_price: f64,
    /// Remaining quantity to hold
    pub remaining_qty: f64,
}

/// Order queue summary
#[derive(Debug, serde::Serialize)]
pub struct OrderQueueSummary {
    pub symbol: String,
    pub pending_count: usize,
    pub total_qty: f64,
    pub orders: Vec<Order>,
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use alphafield_core::{OrderSide, OrderType};
    use chrono::Utc;

    #[test]
    fn test_oco_order_creation() {
        let order1 = Order {
            id: "1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: 1.0,
            price: Some(50000.0),
            status: OrderStatus::New,
            timestamp: Utc::now(),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        let order2 = Order {
            id: "2".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            quantity: 1.0,
            price: Some(51000.0),
            status: OrderStatus::New,
            timestamp: Utc::now(),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        let oco = OcoOrder::new(vec![order1, order2]);
        assert_eq!(oco.orders.len(), 2);
        assert!(oco.active);
        assert!(oco.filled_order_id.is_none());
    }

    #[test]
    fn test_bracket_order_creation() {
        let entry = Order {
            id: "entry".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: 1.0,
            price: None,
            status: OrderStatus::New,
            timestamp: Utc::now(),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        let stop_loss = Order {
            id: "sl".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            quantity: 1.0,
            price: Some(49000.0),
            status: OrderStatus::New,
            timestamp: Utc::now(),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        let take_profit = Order {
            id: "tp".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            quantity: 1.0,
            price: Some(51000.0),
            status: OrderStatus::New,
            timestamp: Utc::now(),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        let bracket = BracketOrder::new(entry, stop_loss, take_profit);
        assert_eq!(bracket.state, BracketState::EntryPending);
        assert!(bracket.stop_loss_order.parent_order_id.is_some());
        assert!(bracket.take_profit_order.parent_order_id.is_some());
    }

    #[test]
    fn test_iceberg_order_slices() {
        let iceberg = IcebergOrder::new("BTCUSDT".to_string(), OrderSide::Buy, 50000.0, 10.0, 2.5);

        assert_eq!(iceberg.total_quantity, 10.0);
        assert_eq!(iceberg.visible_quantity, 2.5);
        assert_eq!(iceberg.hidden_quantity(), 7.5);
        assert!(iceberg.needs_more_slices());
    }

    #[test]
    fn test_limit_chase_calculation() {
        let order = Order {
            id: "1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: 1.0,
            price: Some(50000.0),
            status: OrderStatus::New,
            timestamp: Utc::now(),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        let mut chase = LimitChaseOrder::new(order, 1.0, true, 5);

        // Price moved up, should chase
        let new_limit = chase.calculate_new_limit(50100.0);
        assert!(new_limit.is_some());
        assert!((new_limit.unwrap() - 50500.0).abs() < 1.0);

        // Update the limit
        chase.update_limit(new_limit.unwrap());
        assert_eq!(chase.adjustments, 1);

        // Exhaust remaining allowed adjustments.
        //
        // Important: each time we update the limit upward, we must keep the market price
        // ABOVE the new limit, otherwise `calculate_new_limit` returns None.
        for _ in 0..4 {
            let market_price = chase.order.price.unwrap() * 1.02; // ensure > current_limit
            let next_limit = chase.calculate_new_limit(market_price);
            assert!(next_limit.is_some());
            chase.update_limit(next_limit.unwrap());
        }
        assert_eq!(chase.adjustments, 5);

        // Now at max adjustments, should not chase
        let another_limit = chase.calculate_new_limit(chase.order.price.unwrap() * 1.02);
        assert!(another_limit.is_none());
    }
}
