//! # Grid Bot
//!
//! Automated grid trading bot that places buy and sell orders at predefined price levels.

use super::{BotStats, BotStatus, TradingBot};
use alphafield_core::{Order, OrderSide, OrderType, OrderStatus, QuantError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Grid bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    /// Trading symbol (e.g., "BTCUSDT")
    pub symbol: String,

    /// Lower bound of the price range
    pub lower_price: f64,

    /// Upper bound of the price range
    pub upper_price: f64,

    /// Number of grid levels
    pub grid_levels: u32,

    /// Total capital allocated to the grid
    pub total_capital: f64,

    /// Minimum profit per grid to close (in percentage)
    pub min_profit_percent: f64,
}

impl GridConfig {
    /// Validate the grid configuration
    pub fn validate(&self) -> Result<()> {
        if self.lower_price <= 0.0 || self.upper_price <= 0.0 {
            return Err(QuantError::DataValidation(
                "Prices must be positive".to_string(),
            ));
        }

        if self.lower_price >= self.upper_price {
            return Err(QuantError::DataValidation(
                "Lower price must be less than upper price".to_string(),
            ));
        }

        if self.grid_levels < 2 {
            return Err(QuantError::DataValidation(
                "Must have at least 2 grid levels".to_string(),
            ));
        }

        if self.total_capital <= 0.0 {
            return Err(QuantError::DataValidation(
                "Total capital must be positive".to_string(),
            ));
        }

        if self.min_profit_percent < 0.0 {
            return Err(QuantError::DataValidation(
                "Minimum profit percent cannot be negative".to_string(),
            ));
        }

        Ok(())
    }

    /// Calculate grid price levels
    pub fn calculate_levels(&self) -> Vec<f64> {
        let step = (self.upper_price - self.lower_price) / (self.grid_levels - 1) as f64;
        (0..self.grid_levels)
            .map(|i| self.lower_price + (i as f64 * step))
            .collect()
    }

    /// Calculate quantity per grid level
    pub fn quantity_per_level(&self) -> f64 {
        // Divide capital evenly across grid levels
        let capital_per_level = self.total_capital / self.grid_levels as f64;
        // Use middle price as reference
        let mid_price = (self.lower_price + self.upper_price) / 2.0;
        capital_per_level / mid_price
    }
}

/// Represents a single grid level with its orders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridLevel {
    /// Price level
    pub price: f64,

    /// Buy order placed at this level
    pub buy_order: Option<String>,

    /// Sell order placed at this level
    pub sell_order: Option<String>,

    /// Quantity at this level
    pub quantity: f64,

    /// Profit realized at this level
    pub profit: f64,

    /// Number of trades executed at this level
    pub trades_count: u32,
}

/// Grid Bot implementation
pub struct GridBot {
    id: String,
    config: GridConfig,
    status: Arc<RwLock<BotStatus>>,
    stats: Arc<RwLock<BotStats>>,
    grid_levels: Arc<RwLock<Vec<GridLevel>>>,
    #[allow(dead_code)]
    active_orders: Arc<RwLock<HashMap<String, Order>>>,
}

impl GridBot {
    /// Create a new Grid bot
    pub fn new(config: GridConfig) -> Result<Self> {
        config.validate()?;

        let levels = config.calculate_levels();
        let qty_per_level = config.quantity_per_level();

        let grid_levels: Vec<GridLevel> = levels
            .iter()
            .map(|&price| GridLevel {
                price,
                buy_order: None,
                sell_order: None,
                quantity: qty_per_level,
                profit: 0.0,
                trades_count: 0,
            })
            .collect();

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            config,
            status: Arc::new(RwLock::new(BotStatus::Stopped)),
            stats: Arc::new(RwLock::new(BotStats::default())),
            grid_levels: Arc::new(RwLock::new(grid_levels)),
            active_orders: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Initialize grid by placing buy orders at all levels
    pub fn initialize_grid(&mut self, current_price: f64) -> Result<Vec<Order>> {
        let mut orders = Vec::new();
        let mut grid_levels = self.grid_levels.write().unwrap();

        for level in grid_levels.iter_mut() {
            // Place buy orders below current price
            if level.price < current_price {
                let order = Order {
                    id: Uuid::new_v4().to_string(),
                    symbol: self.config.symbol.clone(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Limit,
                    quantity: level.quantity,
                    price: Some(level.price),
                    status: OrderStatus::New,
                    timestamp: Utc::now(),
                };

                level.buy_order = Some(order.id.clone());
                orders.push(order);
            }
        }

        Ok(orders)
    }

    /// Handle a filled buy order - place corresponding sell order
    pub fn on_buy_filled(&mut self, price: f64) -> Result<Option<Order>> {
        let mut grid_levels = self.grid_levels.write().unwrap();

        // Calculate sell price with minimum profit
        let sell_price = price * (1.0 + self.config.min_profit_percent / 100.0);

        // Find next grid level above for sell order (before mutable borrow)
        let next_level_price = grid_levels
            .iter()
            .find(|l| l.price > price)
            .map(|l| l.price)
            .unwrap_or(sell_price.max(price * 1.01)); // At least 1% above

        // Find the grid level closest to this price
        if let Some(level) = grid_levels
            .iter_mut()
            .min_by(|a, b| (a.price - price).abs().partial_cmp(&(b.price - price).abs()).unwrap())
        {
            let sell_order = Order {
                id: Uuid::new_v4().to_string(),
                symbol: self.config.symbol.clone(),
                side: OrderSide::Sell,
                order_type: OrderType::Limit,
                quantity: level.quantity,
                price: Some(next_level_price),
                status: OrderStatus::New,
                timestamp: Utc::now(),
            };

            level.sell_order = Some(sell_order.id.clone());
            level.trades_count += 1;

            // Update stats
            let mut stats = self.stats.write().unwrap();
            stats.orders_executed += 1;
            stats.total_volume += price * level.quantity;

            return Ok(Some(sell_order));
        }

        Ok(None)
    }

    /// Handle a filled sell order - place corresponding buy order
    pub fn on_sell_filled(&mut self, price: f64, buy_price: f64) -> Result<Option<Order>> {
        let mut grid_levels = self.grid_levels.write().unwrap();

        // Find the grid level
        if let Some(level) = grid_levels
            .iter_mut()
            .find(|l| l.sell_order.is_some() && (l.price - buy_price).abs() < 0.01)
        {
            // Calculate profit
            let profit = (price - buy_price) * level.quantity;
            level.profit += profit;

            // Update stats
            let mut stats = self.stats.write().unwrap();
            stats.orders_executed += 1;
            stats.total_volume += price * level.quantity;
            stats.realized_pnl += profit;

            // Place new buy order at the grid level
            let buy_order = Order {
                id: Uuid::new_v4().to_string(),
                symbol: self.config.symbol.clone(),
                side: OrderSide::Buy,
                order_type: OrderType::Limit,
                quantity: level.quantity,
                price: Some(level.price),
                status: OrderStatus::New,
                timestamp: Utc::now(),
            };

            level.buy_order = Some(buy_order.id.clone());
            level.sell_order = None;

            return Ok(Some(buy_order));
        }

        Ok(None)
    }

    /// Get configuration
    pub fn config(&self) -> &GridConfig {
        &self.config
    }

    /// Get grid levels
    pub fn grid_levels(&self) -> Vec<GridLevel> {
        self.grid_levels.read().unwrap().clone()
    }

    /// Get total profit across all grid levels
    pub fn total_grid_profit(&self) -> f64 {
        self.grid_levels
            .read()
            .unwrap()
            .iter()
            .map(|l| l.profit)
            .sum()
    }
}

impl TradingBot for GridBot {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        "Grid Bot"
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
    fn test_grid_config_validation() {
        let valid_config = GridConfig {
            symbol: "BTCUSDT".to_string(),
            lower_price: 40000.0,
            upper_price: 60000.0,
            grid_levels: 10,
            total_capital: 10000.0,
            min_profit_percent: 1.0,
        };
        assert!(valid_config.validate().is_ok());

        let invalid_config = GridConfig {
            symbol: "BTCUSDT".to_string(),
            lower_price: 60000.0, // Invalid: lower > upper
            upper_price: 40000.0,
            grid_levels: 10,
            total_capital: 10000.0,
            min_profit_percent: 1.0,
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_grid_levels_calculation() {
        let config = GridConfig {
            symbol: "BTCUSDT".to_string(),
            lower_price: 40000.0,
            upper_price: 60000.0,
            grid_levels: 5,
            total_capital: 10000.0,
            min_profit_percent: 1.0,
        };

        let levels = config.calculate_levels();
        assert_eq!(levels.len(), 5);
        assert_eq!(levels[0], 40000.0);
        assert_eq!(levels[4], 60000.0);
        assert_eq!(levels[2], 50000.0); // Middle level
    }

    #[test]
    fn test_grid_bot_creation() {
        let config = GridConfig {
            symbol: "BTCUSDT".to_string(),
            lower_price: 40000.0,
            upper_price: 60000.0,
            grid_levels: 10,
            total_capital: 10000.0,
            min_profit_percent: 1.0,
        };

        let bot = GridBot::new(config).unwrap();
        assert_eq!(bot.status(), BotStatus::Stopped);
        assert_eq!(bot.grid_levels().len(), 10);
    }

    #[test]
    fn test_grid_initialization() {
        let config = GridConfig {
            symbol: "BTCUSDT".to_string(),
            lower_price: 40000.0,
            upper_price: 60000.0,
            grid_levels: 5,
            total_capital: 10000.0,
            min_profit_percent: 1.0,
        };

        let mut bot = GridBot::new(config).unwrap();
        bot.start().unwrap();

        let current_price = 50000.0;
        let orders = bot.initialize_grid(current_price).unwrap();

        // Should have buy orders for levels below current price
        // With 5 levels at 40k, 45k, 50k, 55k, 60k
        // Only 40k and 45k should have buy orders
        assert!(orders.len() >= 2);
        for order in &orders {
            assert_eq!(order.side, OrderSide::Buy);
            assert!(order.price.unwrap() < current_price);
        }
    }

    #[test]
    fn test_buy_filled_creates_sell() {
        let config = GridConfig {
            symbol: "BTCUSDT".to_string(),
            lower_price: 40000.0,
            upper_price: 60000.0,
            grid_levels: 5,
            total_capital: 10000.0,
            min_profit_percent: 1.0,
        };

        let mut bot = GridBot::new(config).unwrap();
        bot.start().unwrap();

        let buy_price = 45000.0;
        let sell_order = bot.on_buy_filled(buy_price).unwrap();

        assert!(sell_order.is_some());
        let order = sell_order.unwrap();
        assert_eq!(order.side, OrderSide::Sell);
        assert!(order.price.unwrap() > buy_price);
    }

    #[test]
    fn test_sell_filled_creates_buy() {
        let config = GridConfig {
            symbol: "BTCUSDT".to_string(),
            lower_price: 40000.0,
            upper_price: 60000.0,
            grid_levels: 5,
            total_capital: 10000.0,
            min_profit_percent: 1.0,
        };

        let mut bot = GridBot::new(config).unwrap();
        bot.start().unwrap();

        // Simulate buy at 45k
        bot.on_buy_filled(45000.0).unwrap();

        // Simulate sell at 46k
        let buy_order = bot.on_sell_filled(46000.0, 45000.0).unwrap();

        assert!(buy_order.is_some());
        let order = buy_order.unwrap();
        assert_eq!(order.side, OrderSide::Buy);

        // Check profit was recorded
        let stats = bot.stats();
        assert!(stats.realized_pnl > 0.0);
    }

    #[test]
    fn test_grid_profit_tracking() {
        let config = GridConfig {
            symbol: "BTCUSDT".to_string(),
            lower_price: 40000.0,
            upper_price: 60000.0,
            grid_levels: 5,
            total_capital: 10000.0,
            min_profit_percent: 1.0,
        };

        let mut bot = GridBot::new(config).unwrap();
        bot.start().unwrap();

        // Execute multiple buy-sell cycles
        bot.on_buy_filled(45000.0).unwrap();
        bot.on_sell_filled(46000.0, 45000.0).unwrap();

        let total_profit = bot.total_grid_profit();
        assert!(total_profit > 0.0);
    }
}
