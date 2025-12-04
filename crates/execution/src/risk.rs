use alphafield_core::{ExecutionService, Order, QuantError, Result, OrderSide};
use async_trait::async_trait;

/// Trait for a single risk check rule
pub trait RiskCheck: Send + Sync {
    fn check(&self, order: &Order) -> Result<()>;
}

/// Checks if order value exceeds a maximum limit
pub struct MaxOrderValue {
    pub max_value: f64,
}

impl RiskCheck for MaxOrderValue {
    fn check(&self, order: &Order) -> Result<()> {
        // Estimate value. For market orders, we need a price estimate.
        // For limit orders, use limit price.
        let price = order.price.unwrap_or(0.0); // If 0, we can't check value accurately without market data

        // If price is 0 (Market order with no estimate), we might skip or fail.
        // For safety, let's assume if price > 0 we check.
        if price > 0.0 {
            let value = price * order.quantity;
            if value > self.max_value {
                return Err(QuantError::DataValidation(format!(
                    "Order value {:.2} exceeds limit {:.2}",
                    value, self.max_value
                )));
            }
        }
        Ok(())
    }
}

/// Risk Manager that wraps an execution service and enforces rules
pub struct RiskManager<S: ExecutionService> {
    service: S,
    checks: Vec<Box<dyn RiskCheck>>,
}

impl<S: ExecutionService> RiskManager<S> {
    pub fn new(service: S) -> Self {
        Self {
            service,
            checks: Vec::new(),
        }
    }

    pub fn add_check<C: RiskCheck + 'static>(&mut self, check: C) {
        self.checks.push(Box::new(check));
    }
}

/// Risk check that prevents short/sell orders in spot-only mode.
pub struct NoShorts;

impl RiskCheck for NoShorts {
    fn check(&self, order: &Order) -> Result<()> {
        if order.side == OrderSide::Sell || order.quantity < 0.0 {
            return Err(QuantError::DataValidation(format!(
                "Short selling is disabled for symbol {}",
                order.symbol
            )));
        }
        Ok(())
    }
}

#[async_trait]
impl<S: ExecutionService> ExecutionService for RiskManager<S> {
    async fn submit_order(&self, order: &Order) -> Result<String> {
        // Run all checks
        for check in &self.checks {
            check.check(order)?;
        }

        // If all pass, forward to service
        self.service.submit_order(order).await
    }

    async fn cancel_order(&self, order_id: &str, symbol: &str) -> Result<()> {
        self.service.cancel_order(order_id, symbol).await
    }

    async fn get_order(&self, order_id: &str, symbol: &str) -> Result<Order> {
        self.service.get_order(order_id, symbol).await
    }
}
