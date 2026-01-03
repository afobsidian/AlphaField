//! # DCA Bot (Dollar Cost Averaging)
//!
//! Automated recurring buy bot with configurable frequency and price thresholds.

use super::{BotStats, BotStatus, TradingBot};
use alphafield_core::{Order, OrderSide, OrderStatus, OrderType, QuantError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// DCA bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DCAConfig {
    /// Trading symbol (e.g., "BTCUSDT")
    pub symbol: String,

    /// Amount to buy each interval
    pub amount_type: AmountType,

    /// Frequency of buys
    pub frequency: Frequency,

    /// Optional: Stop buying if price exceeds this threshold
    pub max_price: Option<f64>,

    /// Optional: Total budget limit (stop when reached)
    pub total_budget: Option<f64>,
}

/// Type of amount to purchase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmountType {
    /// Fixed dollar amount per buy
    FixedAmount(f64),
    /// Percentage of available balance
    PercentOfBalance(f64),
}

/// Frequency of DCA buys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Frequency {
    /// Buy every N minutes (for testing/high-frequency DCA)
    Minutes(u32),
    /// Buy every N hours
    Hours(u32),
    /// Buy daily
    Daily,
    /// Buy weekly
    Weekly,
    /// Buy monthly
    Monthly,
}

impl Frequency {
    /// Convert frequency to duration
    fn to_duration(&self) -> Duration {
        match self {
            Frequency::Minutes(n) => Duration::minutes(*n as i64),
            Frequency::Hours(n) => Duration::hours(*n as i64),
            Frequency::Daily => Duration::days(1),
            Frequency::Weekly => Duration::weeks(1),
            Frequency::Monthly => Duration::days(30), // Approximate
        }
    }
}

/// DCA Bot implementation
pub struct DCABot {
    id: String,
    config: DCAConfig,
    status: Arc<RwLock<BotStatus>>,
    stats: Arc<RwLock<BotStats>>,
    next_execution: Arc<RwLock<Option<DateTime<Utc>>>>,
    total_spent: Arc<RwLock<f64>>,
}

impl DCABot {
    /// Create a new DCA bot
    pub fn new(config: DCAConfig) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            config,
            status: Arc::new(RwLock::new(BotStatus::Stopped)),
            stats: Arc::new(RwLock::new(BotStats::default())),
            next_execution: Arc::new(RwLock::new(None)),
            total_spent: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Check if it's time to execute a buy
    pub fn should_execute(&self) -> bool {
        if *self.status.read().unwrap() != BotStatus::Active {
            return false;
        }

        let next = self.next_execution.read().unwrap();
        if let Some(next_time) = *next {
            Utc::now() >= next_time
        } else {
            // First execution
            true
        }
    }

    /// Execute a DCA buy order
    pub fn execute_buy(&mut self, current_price: f64, available_balance: f64) -> Result<Order> {
        // Check price threshold
        if let Some(max_price) = self.config.max_price {
            if current_price > max_price {
                return Err(QuantError::DataValidation(format!(
                    "Current price {:.2} exceeds max price threshold {:.2}",
                    current_price, max_price
                )));
            }
        }

        // Calculate buy amount
        let buy_amount = match self.config.amount_type {
            AmountType::FixedAmount(amount) => amount,
            AmountType::PercentOfBalance(pct) => available_balance * (pct / 100.0),
        };

        // Check total budget
        let total_spent = *self.total_spent.read().unwrap();
        if let Some(budget) = self.config.total_budget {
            if total_spent + buy_amount > budget {
                let mut status = self.status.write().unwrap();
                *status = BotStatus::Completed;
                return Err(QuantError::DataValidation(
                    "Total budget limit reached".to_string(),
                ));
            }
        }

        // Calculate quantity
        let quantity = buy_amount / current_price;

        // Create order
        let order = Order {
            id: Uuid::new_v4().to_string(),
            symbol: self.config.symbol.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity,
            price: None, // Market order
            status: OrderStatus::New,
            timestamp: Utc::now(),
        };

        // Update stats
        let mut stats = self.stats.write().unwrap();
        stats.orders_executed += 1;
        stats.total_volume += buy_amount;
        stats.last_execution = Some(Utc::now());

        // Update total spent
        let mut spent = self.total_spent.write().unwrap();
        *spent += buy_amount;

        // Schedule next execution
        let mut next = self.next_execution.write().unwrap();
        *next = Some(Utc::now() + self.config.frequency.to_duration());

        Ok(order)
    }

    /// Get configuration
    pub fn config(&self) -> &DCAConfig {
        &self.config
    }

    /// Get total amount spent so far
    pub fn total_spent(&self) -> f64 {
        *self.total_spent.read().unwrap()
    }

    /// Get next execution time
    pub fn next_execution(&self) -> Option<DateTime<Utc>> {
        *self.next_execution.read().unwrap()
    }
}

impl TradingBot for DCABot {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "DCA Bot"
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

        // Set first execution to now
        let mut next = self.next_execution.write().unwrap();
        *next = Some(Utc::now());

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
    fn test_dca_bot_creation() {
        let config = DCAConfig {
            symbol: "BTCUSDT".to_string(),
            amount_type: AmountType::FixedAmount(100.0),
            frequency: Frequency::Daily,
            max_price: Some(50000.0),
            total_budget: Some(10000.0),
        };

        let bot = DCABot::new(config);
        assert_eq!(bot.status(), BotStatus::Stopped);
        assert_eq!(bot.total_spent(), 0.0);
    }

    #[test]
    fn test_dca_bot_start_stop() {
        let config = DCAConfig {
            symbol: "BTCUSDT".to_string(),
            amount_type: AmountType::FixedAmount(100.0),
            frequency: Frequency::Daily,
            max_price: None,
            total_budget: None,
        };

        let mut bot = DCABot::new(config);
        assert_eq!(bot.status(), BotStatus::Stopped);

        bot.start().unwrap();
        assert_eq!(bot.status(), BotStatus::Active);

        bot.pause().unwrap();
        assert_eq!(bot.status(), BotStatus::Paused);

        bot.resume().unwrap();
        assert_eq!(bot.status(), BotStatus::Active);

        bot.stop().unwrap();
        assert_eq!(bot.status(), BotStatus::Stopped);
    }

    #[test]
    fn test_dca_fixed_amount() {
        let config = DCAConfig {
            symbol: "BTCUSDT".to_string(),
            amount_type: AmountType::FixedAmount(100.0),
            frequency: Frequency::Daily,
            max_price: None,
            total_budget: None,
        };

        let mut bot = DCABot::new(config);
        bot.start().unwrap();

        let order = bot.execute_buy(50000.0, 10000.0).unwrap();
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.symbol, "BTCUSDT");
        assert!((order.quantity - 0.002).abs() < 0.0001); // 100 / 50000

        let stats = bot.stats();
        assert_eq!(stats.orders_executed, 1);
        assert_eq!(stats.total_volume, 100.0);
    }

    #[test]
    fn test_dca_percent_of_balance() {
        let config = DCAConfig {
            symbol: "ETHUSDT".to_string(),
            amount_type: AmountType::PercentOfBalance(10.0),
            frequency: Frequency::Weekly,
            max_price: None,
            total_budget: None,
        };

        let mut bot = DCABot::new(config);
        bot.start().unwrap();

        let balance = 1000.0;
        let order = bot.execute_buy(3000.0, balance).unwrap();

        // Should buy 10% of 1000 = 100
        let expected_qty = 100.0 / 3000.0;
        assert!((order.quantity - expected_qty).abs() < 0.0001);
    }

    #[test]
    fn test_dca_max_price_threshold() {
        let config = DCAConfig {
            symbol: "BTCUSDT".to_string(),
            amount_type: AmountType::FixedAmount(100.0),
            frequency: Frequency::Daily,
            max_price: Some(40000.0),
            total_budget: None,
        };

        let mut bot = DCABot::new(config);
        bot.start().unwrap();

        // Should succeed when under threshold
        let result = bot.execute_buy(35000.0, 10000.0);
        assert!(result.is_ok());

        // Should fail when over threshold
        let result = bot.execute_buy(45000.0, 10000.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_dca_total_budget_limit() {
        let config = DCAConfig {
            symbol: "BTCUSDT".to_string(),
            amount_type: AmountType::FixedAmount(100.0),
            frequency: Frequency::Daily,
            max_price: None,
            total_budget: Some(250.0),
        };

        let mut bot = DCABot::new(config);
        bot.start().unwrap();

        // Execute 2 buys successfully
        bot.execute_buy(50000.0, 10000.0).unwrap();
        assert_eq!(bot.total_spent(), 100.0);

        bot.execute_buy(50000.0, 10000.0).unwrap();
        assert_eq!(bot.total_spent(), 200.0);

        // Third buy should fail (would exceed 250)
        let result = bot.execute_buy(50000.0, 10000.0);
        assert!(result.is_err());
        assert_eq!(bot.status(), BotStatus::Completed);
    }
}
