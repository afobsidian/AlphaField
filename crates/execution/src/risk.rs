use alphafield_core::{ExecutionService, Order, OrderSide, QuantError, Result};
use async_trait::async_trait;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

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

// =============================================================================
// Production Safeguards - Live Trading Risk Checks
// =============================================================================

/// Tracks daily PnL and rejects orders when loss limit is breached.
/// Circuit breaker for maximum daily loss protection.
pub struct MaxDailyLoss {
    /// Maximum allowed loss (positive number, e.g., 500.0 for $500 max loss)
    pub max_loss: f64,
    /// Shared state tracking realized + unrealized PnL
    pnl_tracker: Arc<RwLock<DailyPnLState>>,
}

#[derive(Debug, Default)]
struct DailyPnLState {
    /// Current day (Unix timestamp / 86400)
    current_day: u64,
    /// Realized PnL for the day
    realized_pnl: f64,
    /// Unrealized PnL (mark-to-market)
    unrealized_pnl: f64,
    /// Whether circuit breaker has been triggered
    breaker_triggered: bool,
}

impl MaxDailyLoss {
    pub fn new(max_loss: f64) -> Self {
        Self {
            max_loss,
            pnl_tracker: Arc::new(RwLock::new(DailyPnLState::default())),
        }
    }

    /// Update realized PnL after a fill
    pub fn record_realized_pnl(&self, pnl: f64) {
        let mut state = self.pnl_tracker.write().unwrap();
        self.maybe_reset_day(&mut state);
        state.realized_pnl += pnl;
    }

    /// Update unrealized PnL for mark-to-market
    pub fn update_unrealized_pnl(&self, unrealized: f64) {
        let mut state = self.pnl_tracker.write().unwrap();
        self.maybe_reset_day(&mut state);
        state.unrealized_pnl = unrealized;
    }

    /// Check if breaker is triggered
    pub fn is_breaker_triggered(&self) -> bool {
        self.pnl_tracker.read().unwrap().breaker_triggered
    }

    /// Get current total PnL
    pub fn current_pnl(&self) -> f64 {
        let state = self.pnl_tracker.read().unwrap();
        state.realized_pnl + state.unrealized_pnl
    }

    /// Reset day if we've crossed midnight
    fn maybe_reset_day(&self, state: &mut DailyPnLState) {
        let today = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 86400;

        if state.current_day != today {
            state.current_day = today;
            state.realized_pnl = 0.0;
            state.unrealized_pnl = 0.0;
            state.breaker_triggered = false;
        }
    }
}

impl RiskCheck for MaxDailyLoss {
    fn check(&self, _order: &Order) -> Result<()> {
        let mut state = self.pnl_tracker.write().unwrap();
        self.maybe_reset_day(&mut state);

        // Already triggered - reject all orders
        if state.breaker_triggered {
            return Err(QuantError::DataValidation(
                "Circuit breaker active: max daily loss exceeded. Trading halted.".to_string(),
            ));
        }

        let total_pnl = state.realized_pnl + state.unrealized_pnl;

        // Check if loss exceeds threshold (loss is negative PnL)
        if total_pnl < -self.max_loss {
            state.breaker_triggered = true;
            return Err(QuantError::DataValidation(format!(
                "Max daily loss breached: {:.2} (limit: -{:.2}). Trading halted.",
                total_pnl, self.max_loss
            )));
        }

        Ok(())
    }
}

/// Monitors fill price drift from expected price.
/// Alerts when slippage exceeds threshold.
pub struct PositionDrift {
    /// Maximum allowed drift percentage (e.g., 0.005 for 0.5%)
    pub max_drift_pct: f64,
    /// Expected prices for pending orders (order_id -> expected_price)
    expected_prices: Arc<RwLock<std::collections::HashMap<String, f64>>>,
}

impl PositionDrift {
    pub fn new(max_drift_pct: f64) -> Self {
        Self {
            max_drift_pct,
            expected_prices: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Record expected price before order submission
    pub fn expect_price(&self, order_id: &str, expected_price: f64) {
        self.expected_prices
            .write()
            .unwrap()
            .insert(order_id.to_string(), expected_price);
    }

    /// Check fill price and alert if drift exceeds threshold
    /// Returns Ok(drift_pct) if within limits, Err otherwise
    pub fn check_fill(&self, order_id: &str, fill_price: f64) -> Result<f64> {
        let expected = {
            let prices = self.expected_prices.read().unwrap();
            prices.get(order_id).copied()
        };

        if let Some(expected_price) = expected {
            let drift = (fill_price - expected_price).abs() / expected_price;

            // Clean up
            self.expected_prices.write().unwrap().remove(order_id);

            if drift > self.max_drift_pct {
                return Err(QuantError::DataValidation(format!(
                    "Position drift alert: fill at {:.6} vs expected {:.6} ({:.2}% drift, limit {:.2}%)",
                    fill_price, expected_price, drift * 100.0, self.max_drift_pct * 100.0
                )));
            }

            Ok(drift)
        } else {
            Ok(0.0) // No expected price recorded
        }
    }
}

impl RiskCheck for PositionDrift {
    fn check(&self, order: &Order) -> Result<()> {
        // For limit orders, record the expected price
        if let Some(price) = order.price {
            // We'll use the order timestamp as a temporary ID for pre-check
            // Real implementation would use order.id after submission
            self.expect_price(&format!("pre_{}", order.symbol), price);
        }
        Ok(())
    }
}

/// Volatility-scaled position sizing using ATR.
/// Reduces position size when volatility is high.
pub struct VolatilityScaledSize {
    /// Base position size (units or value)
    pub base_size: f64,
    /// ATR multiplier for scaling (higher = more conservative)
    pub atr_multiplier: f64,
    /// Current ATR values by symbol
    atr_values: Arc<RwLock<std::collections::HashMap<String, f64>>>,
    /// Baseline ATR for normalization (e.g., 14-day average ATR)
    baseline_atr: Arc<RwLock<std::collections::HashMap<String, f64>>>,
}

impl VolatilityScaledSize {
    pub fn new(base_size: f64, atr_multiplier: f64) -> Self {
        Self {
            base_size,
            atr_multiplier,
            atr_values: Arc::new(RwLock::new(std::collections::HashMap::new())),
            baseline_atr: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Update current ATR for a symbol
    pub fn update_atr(&self, symbol: &str, current_atr: f64, baseline_atr: f64) {
        self.atr_values
            .write()
            .unwrap()
            .insert(symbol.to_string(), current_atr);
        self.baseline_atr
            .write()
            .unwrap()
            .insert(symbol.to_string(), baseline_atr);
    }

    /// Calculate adjusted position size based on current volatility
    pub fn adjusted_size(&self, symbol: &str) -> f64 {
        let current_atr = self.atr_values.read().unwrap().get(symbol).copied();
        let baseline = self.baseline_atr.read().unwrap().get(symbol).copied();

        match (current_atr, baseline) {
            (Some(current), Some(base)) if current > 0.0 && base > 0.0 => {
                // Scale inversely with volatility
                // Higher ATR = smaller position
                let volatility_ratio = base / current;
                let scaled = self.base_size * volatility_ratio * self.atr_multiplier;

                // Cap at base size (don't increase beyond normal)
                scaled.min(self.base_size)
            }
            _ => self.base_size, // No ATR data, use base size
        }
    }
}

impl RiskCheck for VolatilityScaledSize {
    fn check(&self, order: &Order) -> Result<()> {
        let max_size = self.adjusted_size(&order.symbol);

        if order.quantity > max_size {
            return Err(QuantError::DataValidation(format!(
                "Order size {:.6} exceeds volatility-adjusted limit {:.6} for {}",
                order.quantity, max_size, order.symbol
            )));
        }

        Ok(())
    }
}

/// Fat-finger protection: Reject orders that exceed a percentage of account value.
/// Prevents accidental large orders that could significantly impact the account.
pub struct FatFingerProtection {
    /// Maximum order value as percentage of account (e.g., 0.1 for 10%)
    pub max_order_pct: f64,
    /// Current account value (updated periodically)
    account_value: Arc<RwLock<f64>>,
}

impl FatFingerProtection {
    pub fn new(max_order_pct: f64, initial_account_value: f64) -> Self {
        Self {
            max_order_pct,
            account_value: Arc::new(RwLock::new(initial_account_value)),
        }
    }

    /// Update the current account value
    pub fn update_account_value(&self, value: f64) {
        *self.account_value.write().unwrap() = value;
    }

    /// Get current account value
    pub fn account_value(&self) -> f64 {
        *self.account_value.read().unwrap()
    }

    /// Calculate maximum allowed order value
    pub fn max_order_value(&self) -> f64 {
        self.account_value() * self.max_order_pct
    }
}

impl RiskCheck for FatFingerProtection {
    fn check(&self, order: &Order) -> Result<()> {
        let account = self.account_value();

        // Need a price to calculate order value
        let price = match order.price {
            Some(p) if p > 0.0 => p,
            _ => return Ok(()), // Can't check without price
        };

        let order_value = price * order.quantity.abs();
        let max_value = account * self.max_order_pct;

        if order_value > max_value {
            return Err(QuantError::DataValidation(format!(
                "Fat-finger protection: Order value ${:.2} exceeds {:.1}% of account (${:.2}). Max allowed: ${:.2}",
                order_value, self.max_order_pct * 100.0, account, max_value
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_daily_loss_rejects_when_breached() {
        let check = MaxDailyLoss::new(100.0);

        // Simulate a -$150 loss
        check.record_realized_pnl(-150.0);

        let order = Order {
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            quantity: 0.1,
            price: Some(50000.0),
            order_type: alphafield_core::OrderType::Limit,
            timestamp: chrono::Utc::now(),
            id: String::new(),
            status: alphafield_core::OrderStatus::New,
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        let result = check.check(&order);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Max daily loss"));
    }

    #[test]
    fn test_max_daily_loss_allows_within_limit() {
        let check = MaxDailyLoss::new(100.0);
        check.record_realized_pnl(-50.0);

        let order = Order {
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            quantity: 0.1,
            price: Some(50000.0),
            order_type: alphafield_core::OrderType::Limit,
            timestamp: chrono::Utc::now(),
            id: String::new(),
            status: alphafield_core::OrderStatus::New,
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        assert!(check.check(&order).is_ok());
    }

    #[test]
    fn test_volatility_scaled_size() {
        let check = VolatilityScaledSize::new(1.0, 1.0);

        // When current ATR is 2x baseline, position should be halved
        check.update_atr("BTCUSDT", 2000.0, 1000.0);

        let adjusted = check.adjusted_size("BTCUSDT");
        assert!((adjusted - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_fat_finger_rejects_large_order() {
        // Account value $10,000, max 10% per order = $1,000 max
        let check = FatFingerProtection::new(0.10, 10000.0);

        // Order worth $2,000 (0.04 BTC @ $50,000) should be rejected
        let order = Order {
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            quantity: 0.04,
            price: Some(50000.0),
            order_type: alphafield_core::OrderType::Limit,
            timestamp: chrono::Utc::now(),
            id: String::new(),
            status: alphafield_core::OrderStatus::New,
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        let result = check.check(&order);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Fat-finger"));
    }

    #[test]
    fn test_fat_finger_allows_small_order() {
        // Account value $10,000, max 10% per order = $1,000 max
        let check = FatFingerProtection::new(0.10, 10000.0);

        // Order worth $500 (0.01 BTC @ $50,000) should be allowed
        let order = Order {
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            quantity: 0.01,
            price: Some(50000.0),
            order_type: alphafield_core::OrderType::Limit,
            timestamp: chrono::Utc::now(),
            id: String::new(),
            status: alphafield_core::OrderStatus::New,
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        };

        assert!(check.check(&order).is_ok());
    }
}
