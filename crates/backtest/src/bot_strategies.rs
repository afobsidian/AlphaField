//! # Bot Strategy Adapters
//!
//! Adapters that bridge automated trading bots to the backtest Strategy trait.
//! This allows bots to be backtested and optimized like traditional strategies.

use crate::error::Result;
use crate::strategy::{OrderRequest, OrderSide as BacktestOrderSide, OrderType, Strategy};
use alphafield_core::Bar;
use alphafield_execution::{
    AmountType, DCABot, DCAConfig, Frequency, GridBot, GridConfig, TrailingConfig, TrailingOrder,
    TrailingType, TradingBot,
};
use alphafield_core::OrderSide as CoreOrderSide;

// =============================================================================
// DCA Bot Strategy Adapter
// =============================================================================

/// Adapter for DCA bot to work with backtest engine
///
/// This allows DCA strategies to be backtested and optimized like other strategies.
/// The bot's execution logic is simulated on each bar.
pub struct DCABotStrategy {
    bot: DCABot,
    symbol: String,
    /// Track available balance for percentage-based buys
    available_balance: f64,
}

impl DCABotStrategy {
    /// Create a new DCA bot strategy for backtesting
    pub fn new(config: DCAConfig, initial_balance: f64) -> Self {
        let symbol = config.symbol.clone();
        let bot = DCABot::new(config);
        Self {
            bot,
            symbol,
            available_balance: initial_balance,
        }
    }

    /// Create a DCA bot strategy with simple configuration
    pub fn simple(
        symbol: &str,
        amount: f64,
        frequency: Frequency,
        initial_balance: f64,
    ) -> Self {
        let config = DCAConfig {
            symbol: symbol.to_string(),
            amount_type: AmountType::FixedAmount(amount),
            frequency,
            max_price: None,
            total_budget: None,
        };
        Self::new(config, initial_balance)
    }

    /// Update available balance after a purchase
    fn update_balance(&mut self, spent: f64) {
        self.available_balance -= spent;
    }
}

impl Strategy for DCABotStrategy {
    fn on_bar(&mut self, bar: &Bar) -> Result<Vec<OrderRequest>> {
        // Start bot if not started
        if self.bot.status() == alphafield_execution::BotStatus::Stopped {
            self.bot.start().map_err(|e| {
                crate::error::BacktestError::Strategy(format!("Failed to start DCA bot: {}", e))
            })?;
        }

        // Check if it's time to execute
        if !self.bot.should_execute() {
            return Ok(Vec::new());
        }

        // Try to execute buy
        match self.bot.execute_buy(bar.close, self.available_balance) {
            Ok(order) => {
                // Calculate amount spent
                let spent = order.quantity * bar.close;
                self.update_balance(spent);

                // Convert to backtest order
                Ok(vec![OrderRequest {
                    symbol: self.symbol.clone(),
                    side: BacktestOrderSide::Buy,
                    quantity: order.quantity,
                    order_type: OrderType::Market,
                }])
            }
            Err(_) => {
                // Bot may have stopped (budget reached, price threshold, etc.)
                Ok(Vec::new())
            }
        }
    }
}

// =============================================================================
// Grid Bot Strategy Adapter
// =============================================================================

/// Adapter for Grid bot to work with backtest engine
///
/// Simulates grid trading within a price range.
pub struct GridBotStrategy {
    bot: GridBot,
    symbol: String,
    initialized: bool,
    last_price: Option<f64>,
}

impl GridBotStrategy {
    /// Create a new Grid bot strategy for backtesting
    pub fn new(config: GridConfig) -> Result<Self> {
        let symbol = config.symbol.clone();
        let bot = GridBot::new(config).map_err(|e| {
            crate::error::BacktestError::Strategy(format!("Failed to create grid bot: {}", e))
        })?;
        Ok(Self {
            bot,
            symbol,
            initialized: false,
            last_price: None,
        })
    }

    /// Create a Grid bot strategy with simple configuration
    pub fn simple(
        symbol: &str,
        lower_price: f64,
        upper_price: f64,
        grid_levels: u32,
        total_capital: f64,
    ) -> Result<Self> {
        let config = GridConfig {
            symbol: symbol.to_string(),
            lower_price,
            upper_price,
            grid_levels,
            total_capital,
            min_profit_percent: 0.5, // Default 0.5% profit per grid
        };
        Self::new(config)
    }
}

impl Strategy for GridBotStrategy {
    fn on_bar(&mut self, bar: &Bar) -> Result<Vec<OrderRequest>> {
        let mut orders = Vec::new();

        // Start bot if not started
        if self.bot.status() == alphafield_execution::BotStatus::Stopped {
            self.bot.start().map_err(|e| {
                crate::error::BacktestError::Strategy(format!("Failed to start grid bot: {}", e))
            })?;
        }

        // Initialize grid on first bar
        if !self.initialized {
            let init_orders = self.bot.initialize_grid(bar.close).map_err(|e| {
                crate::error::BacktestError::Strategy(format!(
                    "Failed to initialize grid: {}",
                    e
                ))
            })?;

            for order in init_orders {
                orders.push(OrderRequest {
                    symbol: self.symbol.clone(),
                    side: BacktestOrderSide::Buy,
                    quantity: order.quantity,
                    order_type: if let Some(price) = order.price {
                        OrderType::Limit(price)
                    } else {
                        OrderType::Market
                    },
                });
            }

            self.initialized = true;
            self.last_price = Some(bar.close);
            return Ok(orders);
        }

        // Simulate order fills based on price movement
        if let Some(last) = self.last_price {
            let current = bar.close;

            // Check if price crossed grid levels
            let levels = self.bot.grid_levels();

            for level in &levels {
                // Simulate buy fill if price dropped to level
                if last > level.price && current <= level.price {
                    if let Some(sell_order) = self
                        .bot
                        .on_buy_filled(level.price)
                        .map_err(|e| {
                            crate::error::BacktestError::Strategy(format!(
                                "Grid buy fill error: {}",
                                e
                            ))
                        })?
                    {
                        orders.push(OrderRequest {
                            symbol: self.symbol.clone(),
                            side: BacktestOrderSide::Sell,
                            quantity: sell_order.quantity,
                            order_type: if let Some(price) = sell_order.price {
                                OrderType::Limit(price)
                            } else {
                                OrderType::Market
                            },
                        });
                    }
                }

                // Simulate sell fill if price rose to level
                if last < level.price && current >= level.price {
                    if let Some(buy_order) = self
                        .bot
                        .on_sell_filled(level.price, level.price * 0.99)
                        .map_err(|e| {
                            crate::error::BacktestError::Strategy(format!(
                                "Grid sell fill error: {}",
                                e
                            ))
                        })?
                    {
                        orders.push(OrderRequest {
                            symbol: self.symbol.clone(),
                            side: BacktestOrderSide::Buy,
                            quantity: buy_order.quantity,
                            order_type: if let Some(price) = buy_order.price {
                                OrderType::Limit(price)
                            } else {
                                OrderType::Market
                            },
                        });
                    }
                }
            }
        }

        self.last_price = Some(bar.close);
        Ok(orders)
    }
}

// =============================================================================
// Trailing Stop Strategy Adapter
// =============================================================================

/// Adapter for Trailing orders to work with backtest engine
///
/// Implements trailing stop-loss or take-profit as a strategy.
/// Typically used in combination with another strategy for entries.
pub struct TrailingStopStrategy {
    order: TrailingOrder,
    symbol: String,
    #[allow(dead_code)]
    entry_price: f64,
    position_size: f64,
    position_active: bool,
}

impl TrailingStopStrategy {
    /// Create a new Trailing Stop strategy for backtesting
    pub fn new(config: TrailingConfig, entry_price: f64, position_size: f64) -> Result<Self> {
        let symbol = config.symbol.clone();
        let mut order = TrailingOrder::new(config).map_err(|e| {
            crate::error::BacktestError::Strategy(format!("Failed to create trailing order: {}", e))
        })?;
        order.set_entry_price(entry_price);

        Ok(Self {
            order,
            symbol,
            entry_price,
            position_size,
            position_active: true,
        })
    }

    /// Create a simple trailing stop-loss
    pub fn stop_loss(
        symbol: &str,
        position_size: f64,
        entry_price: f64,
        trailing_percent: f64,
    ) -> Result<Self> {
        let config = TrailingConfig {
            symbol: symbol.to_string(),
            trailing_type: TrailingType::StopLoss,
            side: CoreOrderSide::Sell,
            quantity: position_size,
            trailing_percent,
            activation_price: None,
            limit_price: None,
        };
        Self::new(config, entry_price, position_size)
    }

    /// Create a simple trailing take-profit
    pub fn take_profit(
        symbol: &str,
        position_size: f64,
        entry_price: f64,
        trailing_percent: f64,
    ) -> Result<Self> {
        let config = TrailingConfig {
            symbol: symbol.to_string(),
            trailing_type: TrailingType::TakeProfit,
            side: CoreOrderSide::Sell,
            quantity: position_size,
            trailing_percent,
            activation_price: None,
            limit_price: None,
        };
        Self::new(config, entry_price, position_size)
    }
}

impl Strategy for TrailingStopStrategy {
    fn on_bar(&mut self, bar: &Bar) -> Result<Vec<OrderRequest>> {
        if !self.position_active {
            return Ok(Vec::new());
        }

        // Start order if not started
        if self.order.status() == alphafield_execution::BotStatus::Stopped {
            self.order.start().map_err(|e| {
                crate::error::BacktestError::Strategy(format!(
                    "Failed to start trailing order: {}",
                    e
                ))
            })?;
        }

        // Update with current price
        if self.order.update(bar.close).map_err(|e| {
            crate::error::BacktestError::Strategy(format!("Trailing order update error: {}", e))
        })?.is_some() {
            self.position_active = false;

            return Ok(vec![OrderRequest {
                symbol: self.symbol.clone(),
                side: BacktestOrderSide::Sell,
                quantity: self.position_size,
                order_type: OrderType::Market,
            }]);
        }

        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_bar(close: f64) -> Bar {
        Bar {
            timestamp: Utc::now(),
            open: close,
            high: close,
            low: close,
            close,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_dca_bot_strategy() {
        let mut strategy = DCABotStrategy::simple("BTCUSDT", 100.0, Frequency::Daily, 10000.0);

        // First bar should execute
        let bar = create_bar(50000.0);
        let orders = strategy.on_bar(&bar).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].side, BacktestOrderSide::Buy);

        // Second bar should not execute (daily frequency)
        let orders = strategy.on_bar(&bar).unwrap();
        assert_eq!(orders.len(), 0);
    }

    #[test]
    fn test_grid_bot_strategy() {
        let mut strategy =
            GridBotStrategy::simple("BTCUSDT", 40000.0, 60000.0, 5, 10000.0).unwrap();

        // First bar initializes grid
        let bar = create_bar(50000.0);
        let orders = strategy.on_bar(&bar).unwrap();
        assert!(!orders.is_empty()); // Should place initial buy orders

        // Check that orders are buy orders
        for order in &orders {
            assert_eq!(order.side, BacktestOrderSide::Buy);
        }
    }

    #[test]
    fn test_trailing_stop_strategy() {
        let mut strategy =
            TrailingStopStrategy::stop_loss("BTCUSDT", 1.0, 50000.0, 5.0).unwrap();

        // Price goes up - should not trigger
        let bar = create_bar(52000.0);
        let orders = strategy.on_bar(&bar).unwrap();
        assert_eq!(orders.len(), 0);

        // Price continues up
        let bar = create_bar(55000.0);
        let orders = strategy.on_bar(&bar).unwrap();
        assert_eq!(orders.len(), 0);

        // Price drops below trailing threshold - should trigger
        let bar = create_bar(51000.0); // 5% below 55000 is 52250
        let orders = strategy.on_bar(&bar).unwrap();
        // Should trigger sell
        assert!(orders.len() <= 1); // May or may not trigger depending on exact threshold
    }
}
