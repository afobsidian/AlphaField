use alphafield_core::{ExecutionService, Order, OrderStatus, OrderType, QuantError, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Client for simulating trades without real money
#[derive(Clone)]
pub struct PaperTradingClient {
    /// In-memory store of orders: OrderID -> Order
    orders: Arc<Mutex<HashMap<String, Order>>>,
}

impl PaperTradingClient {
    pub fn new() -> Self {
        Self {
            orders: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ExecutionService for PaperTradingClient {
    async fn submit_order(&self, order: &Order) -> Result<String> {
        let mut orders = self.orders.lock().map_err(|_| {
            QuantError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Lock poisoned",
            ))
        })?;

        // Generate a simulated Order ID
        let order_id = Uuid::new_v4().to_string();

        let mut new_order = order.clone();
        new_order.id = order_id.clone();
        new_order.status = OrderStatus::Filled; // Auto-fill for paper trading
        new_order.timestamp = Utc::now();

        // If market order, we need a price. Let's assume a dummy price if not provided.
        // In a real backtest, this would come from the current market data.
        if new_order.price.is_none() {
            new_order.price = Some(100.0); // Dummy fill price
        }

        orders.insert(order_id.clone(), new_order);

        Ok(order_id)
    }

    async fn cancel_order(&self, order_id: &str, _symbol: &str) -> Result<()> {
        let mut orders = self.orders.lock().map_err(|_| {
            QuantError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Lock poisoned",
            ))
        })?;

        if let Some(order) = orders.get_mut(order_id) {
            if order.status == OrderStatus::New || order.status == OrderStatus::PartiallyFilled {
                order.status = OrderStatus::Canceled;
                Ok(())
            } else {
                Err(QuantError::DataValidation(format!(
                    "Cannot cancel order in status {:?}",
                    order.status
                )))
            }
        } else {
            Err(QuantError::NotFound(format!(
                "Order {} not found",
                order_id
            )))
        }
    }

    async fn get_order(&self, order_id: &str, _symbol: &str) -> Result<Order> {
        let orders = self.orders.lock().map_err(|_| {
            QuantError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Lock poisoned",
            ))
        })?;

        orders
            .get(order_id)
            .cloned()
            .ok_or_else(|| QuantError::NotFound(format!("Order {} not found", order_id)))
    }
}
