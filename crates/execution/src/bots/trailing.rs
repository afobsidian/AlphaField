//! # Trailing Orders
//!
//! Dynamic stop-loss and take-profit orders that follow price movements.

use super::{BotStats, BotStatus, TradingBot};
use alphafield_core::{Order, OrderSide, OrderType, OrderStatus, QuantError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Type of trailing order
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TrailingType {
    /// Trailing stop-loss (follows price up, triggers on drop)
    StopLoss,
    /// Trailing take-profit (follows price down, triggers on rise)
    TakeProfit,
}

/// Trailing order configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailingConfig {
    /// Trading symbol (e.g., "BTCUSDT")
    pub symbol: String,

    /// Type of trailing order
    pub trailing_type: TrailingType,

    /// Side of the order when triggered
    pub side: OrderSide,

    /// Quantity to trade
    pub quantity: f64,

    /// Trailing distance (percentage)
    pub trailing_percent: f64,

    /// Optional: Activation price (price must reach this before trailing starts)
    pub activation_price: Option<f64>,

    /// Optional: Maximum price for stop-loss or minimum price for take-profit
    pub limit_price: Option<f64>,
}

impl TrailingConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.quantity <= 0.0 {
            return Err(QuantError::DataValidation(
                "Quantity must be positive".to_string(),
            ));
        }

        if self.trailing_percent <= 0.0 || self.trailing_percent > 100.0 {
            return Err(QuantError::DataValidation(
                "Trailing percent must be between 0 and 100".to_string(),
            ));
        }

        if let Some(price) = self.activation_price {
            if price <= 0.0 {
                return Err(QuantError::DataValidation(
                    "Activation price must be positive".to_string(),
                ));
            }
        }

        if let Some(price) = self.limit_price {
            if price <= 0.0 {
                return Err(QuantError::DataValidation(
                    "Limit price must be positive".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Trailing order state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrailingState {
    /// Whether the order has been activated
    activated: bool,

    /// Highest price seen (for stop-loss) or lowest price seen (for take-profit)
    extreme_price: Option<f64>,

    /// Current trigger price
    trigger_price: Option<f64>,

    /// Whether the order has been triggered
    triggered: bool,
}

/// Trailing Order implementation
pub struct TrailingOrder {
    id: String,
    config: TrailingConfig,
    status: Arc<RwLock<BotStatus>>,
    stats: Arc<RwLock<BotStats>>,
    state: Arc<RwLock<TrailingState>>,
    entry_price: Arc<RwLock<Option<f64>>>,
}

impl TrailingOrder {
    /// Create a new trailing order
    pub fn new(config: TrailingConfig) -> Result<Self> {
        config.validate()?;

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            config,
            status: Arc::new(RwLock::new(BotStatus::Stopped)),
            stats: Arc::new(RwLock::new(BotStats::default())),
            state: Arc::new(RwLock::new(TrailingState {
                activated: false,
                extreme_price: None,
                trigger_price: None,
                triggered: false,
            })),
            entry_price: Arc::new(RwLock::new(None)),
        })
    }

    /// Create a trailing stop-loss order
    pub fn stop_loss(
        symbol: String,
        quantity: f64,
        trailing_percent: f64,
        activation_price: Option<f64>,
    ) -> Result<Self> {
        let config = TrailingConfig {
            symbol,
            trailing_type: TrailingType::StopLoss,
            side: OrderSide::Sell,
            quantity,
            trailing_percent,
            activation_price,
            limit_price: None,
        };
        Self::new(config)
    }

    /// Create a trailing take-profit order
    pub fn take_profit(
        symbol: String,
        quantity: f64,
        trailing_percent: f64,
        activation_price: Option<f64>,
    ) -> Result<Self> {
        let config = TrailingConfig {
            symbol,
            trailing_type: TrailingType::TakeProfit,
            side: OrderSide::Sell,
            quantity,
            trailing_percent,
            activation_price,
            limit_price: None,
        };
        Self::new(config)
    }

    /// Update the trailing order with current market price
    pub fn update(&mut self, current_price: f64) -> Result<Option<Order>> {
        if *self.status.read().unwrap() != BotStatus::Active {
            return Ok(None);
        }

        let mut state = self.state.write().unwrap();

        // Check activation
        if !state.activated {
            if let Some(activation_price) = self.config.activation_price {
                let should_activate = match self.config.trailing_type {
                    TrailingType::StopLoss => current_price >= activation_price,
                    TrailingType::TakeProfit => current_price <= activation_price,
                };

                if should_activate {
                    state.activated = true;
                    state.extreme_price = Some(current_price);
                } else {
                    return Ok(None);
                }
            } else {
                // No activation price, activate immediately
                state.activated = true;
                state.extreme_price = Some(current_price);
            }
        }

        // Update extreme price
        let extreme = state.extreme_price.unwrap();
        let new_extreme = match self.config.trailing_type {
            TrailingType::StopLoss => {
                // Track highest price
                if current_price > extreme {
                    current_price
                } else {
                    extreme
                }
            }
            TrailingType::TakeProfit => {
                // Track lowest price
                if current_price < extreme {
                    current_price
                } else {
                    extreme
                }
            }
        };

        state.extreme_price = Some(new_extreme);

        // Calculate trigger price
        let trigger = match self.config.trailing_type {
            TrailingType::StopLoss => {
                // Trigger price is below extreme by trailing percent
                new_extreme * (1.0 - self.config.trailing_percent / 100.0)
            }
            TrailingType::TakeProfit => {
                // Trigger price is above extreme by trailing percent
                new_extreme * (1.0 + self.config.trailing_percent / 100.0)
            }
        };

        state.trigger_price = Some(trigger);

        // Check if triggered
        let should_trigger = match self.config.trailing_type {
            TrailingType::StopLoss => current_price <= trigger,
            TrailingType::TakeProfit => current_price >= trigger,
        };

        if should_trigger && !state.triggered {
            state.triggered = true;

            // Create market order
            let order = Order {
                id: Uuid::new_v4().to_string(),
                symbol: self.config.symbol.clone(),
                side: self.config.side,
                order_type: OrderType::Market,
                quantity: self.config.quantity,
                price: None,
                status: OrderStatus::New,
                timestamp: Utc::now(),
            };

            // Update stats
            let mut stats = self.stats.write().unwrap();
            stats.orders_executed += 1;
            stats.last_execution = Some(Utc::now());

            // Calculate PnL if we have entry price
            if let Some(entry) = *self.entry_price.read().unwrap() {
                let pnl = match self.config.side {
                    OrderSide::Sell => (current_price - entry) * self.config.quantity,
                    OrderSide::Buy => (entry - current_price) * self.config.quantity,
                };
                stats.realized_pnl = pnl;
            }

            // Mark as completed
            let mut status = self.status.write().unwrap();
            *status = BotStatus::Completed;

            return Ok(Some(order));
        }

        Ok(None)
    }

    /// Set the entry price for PnL calculation
    pub fn set_entry_price(&mut self, price: f64) {
        let mut entry = self.entry_price.write().unwrap();
        *entry = Some(price);
    }

    /// Get current trigger price
    pub fn trigger_price(&self) -> Option<f64> {
        self.state.read().unwrap().trigger_price
    }

    /// Get extreme price tracked
    pub fn extreme_price(&self) -> Option<f64> {
        self.state.read().unwrap().extreme_price
    }

    /// Check if activated
    pub fn is_activated(&self) -> bool {
        self.state.read().unwrap().activated
    }

    /// Check if triggered
    pub fn is_triggered(&self) -> bool {
        self.state.read().unwrap().triggered
    }

    /// Get configuration
    pub fn config(&self) -> &TrailingConfig {
        &self.config
    }
}

impl TradingBot for TrailingOrder {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        match self.config.trailing_type {
            TrailingType::StopLoss => "Trailing Stop-Loss",
            TrailingType::TakeProfit => "Trailing Take-Profit",
        }
    }

    fn status(&self) -> BotStatus {
        *self.status.read().unwrap()
    }

    fn start(&mut self) -> Result<()> {
        let mut status = self.status.write().unwrap();
        *status = BotStatus::Active;

        let mut stats = self.stats.write().unwrap();
        if stats.started_at.is_none() {
            stats.started_at = Some(Utc::now());
        }

        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        let mut status = self.status.write().unwrap();
        if *status == BotStatus::Active {
            *status = BotStatus::Paused;
            Ok(())
        } else {
            Err(QuantError::DataValidation(
                "Bot must be active to pause".to_string(),
            ))
        }
    }

    fn resume(&mut self) -> Result<()> {
        let mut status = self.status.write().unwrap();
        if *status == BotStatus::Paused {
            *status = BotStatus::Active;
            Ok(())
        } else {
            Err(QuantError::DataValidation(
                "Bot must be paused to resume".to_string(),
            ))
        }
    }

    fn stop(&mut self) -> Result<()> {
        let mut status = self.status.write().unwrap();
        *status = BotStatus::Stopped;
        Ok(())
    }

    fn stats(&self) -> BotStats {
        self.stats.read().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trailing_stop_loss_creation() {
        let order = TrailingOrder::stop_loss(
            "BTCUSDT".to_string(),
            1.0,
            5.0,
            Some(50000.0),
        );
        assert!(order.is_ok());

        let order = order.unwrap();
        assert_eq!(order.config.trailing_type, TrailingType::StopLoss);
        assert_eq!(order.config.trailing_percent, 5.0);
    }

    #[test]
    fn test_trailing_activation() {
        let mut order = TrailingOrder::stop_loss(
            "BTCUSDT".to_string(),
            1.0,
            5.0,
            Some(50000.0),
        ).unwrap();

        order.start().unwrap();

        // Should not activate below activation price
        order.update(49000.0).unwrap();
        assert!(!order.is_activated());

        // Should activate at activation price
        order.update(50000.0).unwrap();
        assert!(order.is_activated());
    }

    #[test]
    fn test_trailing_stop_loss_tracking() {
        let mut order = TrailingOrder::stop_loss(
            "BTCUSDT".to_string(),
            1.0,
            5.0,
            None, // No activation price
        ).unwrap();

        order.start().unwrap();

        // Start at 50000
        order.update(50000.0).unwrap();
        assert_eq!(order.extreme_price(), Some(50000.0));
        assert_eq!(order.trigger_price(), Some(47500.0)); // 5% below

        // Price goes up to 52000
        order.update(52000.0).unwrap();
        assert_eq!(order.extreme_price(), Some(52000.0));
        assert_eq!(order.trigger_price(), Some(49400.0)); // 5% below 52000

        // Should not trigger yet at 50000
        let result = order.update(50000.0).unwrap();
        assert!(result.is_none());
        assert!(!order.is_triggered());
    }

    #[test]
    fn test_trailing_stop_loss_trigger() {
        let mut order = TrailingOrder::stop_loss(
            "BTCUSDT".to_string(),
            1.0,
            5.0,
            None,
        ).unwrap();

        order.start().unwrap();

        // Start at 50000
        order.update(50000.0).unwrap();

        // Price goes up to 52000
        order.update(52000.0).unwrap();

        // Price drops to trigger level (49400)
        let result = order.update(49000.0).unwrap();
        assert!(result.is_some());
        assert!(order.is_triggered());
        assert_eq!(order.status(), BotStatus::Completed);

        let triggered_order = result.unwrap();
        assert_eq!(triggered_order.side, OrderSide::Sell);
        assert_eq!(triggered_order.quantity, 1.0);
    }

    #[test]
    fn test_trailing_take_profit_tracking() {
        let mut order = TrailingOrder::take_profit(
            "BTCUSDT".to_string(),
            1.0,
            5.0,
            None,
        ).unwrap();

        order.start().unwrap();

        // Start at 50000
        order.update(50000.0).unwrap();
        assert_eq!(order.extreme_price(), Some(50000.0));
        assert_eq!(order.trigger_price(), Some(52500.0)); // 5% above

        // Price drops to 48000
        order.update(48000.0).unwrap();
        assert_eq!(order.extreme_price(), Some(48000.0));
        assert_eq!(order.trigger_price(), Some(50400.0)); // 5% above 48000

        // Should trigger when price reaches 50400
        let result = order.update(51000.0).unwrap();
        assert!(result.is_some());
        assert!(order.is_triggered());
    }

    #[test]
    fn test_pnl_calculation() {
        let mut order = TrailingOrder::stop_loss(
            "BTCUSDT".to_string(),
            1.0,
            5.0,
            None,
        ).unwrap();

        order.start().unwrap();
        order.set_entry_price(50000.0);

        // Price goes up then triggers
        order.update(52000.0).unwrap();
        let result = order.update(49000.0).unwrap();

        assert!(result.is_some());

        let stats = order.stats();
        // PnL should be negative (bought at 50k, sold at 49k)
        assert!(stats.realized_pnl < 0.0);
        assert!((stats.realized_pnl + 1000.0).abs() < 1.0); // Approx -1000
    }

    #[test]
    fn test_validation() {
        // Invalid quantity
        let config = TrailingConfig {
            symbol: "BTCUSDT".to_string(),
            trailing_type: TrailingType::StopLoss,
            side: OrderSide::Sell,
            quantity: -1.0,
            trailing_percent: 5.0,
            activation_price: None,
            limit_price: None,
        };
        assert!(config.validate().is_err());

        // Invalid trailing percent
        let config = TrailingConfig {
            symbol: "BTCUSDT".to_string(),
            trailing_type: TrailingType::StopLoss,
            side: OrderSide::Sell,
            quantity: 1.0,
            trailing_percent: 150.0,
            activation_price: None,
            limit_price: None,
        };
        assert!(config.validate().is_err());
    }
}
