use crate::error::{BacktestError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub avg_price: f64,
}

impl Position {
    pub fn new(symbol: impl Into<String>, quantity: f64, price: f64) -> Self {
        Self {
            symbol: symbol.into(),
            quantity,
            avg_price: price,
        }
    }

    pub fn update(&mut self, quantity: f64, price: f64) {
        let total_cost = self.quantity * self.avg_price;
        let new_cost = quantity * price;
        let new_quantity = self.quantity + quantity;

        if new_quantity == 0.0 {
            self.quantity = 0.0;
            self.avg_price = 0.0;
        } else {
            self.avg_price = (total_cost + new_cost) / new_quantity;
            self.quantity = new_quantity;
        }
    }

    pub fn market_value(&self, current_price: f64) -> f64 {
        self.quantity * current_price
    }

    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        (current_price - self.avg_price) * self.quantity
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub cash: f64,
    pub positions: HashMap<String, Position>,
    pub equity_history: Vec<(i64, f64)>, // Timestamp (ms), Equity
}

impl Portfolio {
    pub fn new(initial_cash: f64) -> Self {
        Self {
            cash: initial_cash,
            positions: HashMap::new(),
            equity_history: Vec::new(),
        }
    }

    pub fn update_from_fill(
        &mut self,
        symbol: &str,
        quantity: f64,
        price: f64,
        fee: f64,
    ) -> Result<()> {
        let cost = quantity * price;

        if self.cash - cost - fee < 0.0 {
            return Err(BacktestError::InsufficientFunds {
                required: cost + fee,
                available: self.cash,
            });
        }

        self.cash -= cost + fee;

        let position = self
            .positions
            .entry(symbol.to_string())
            .or_insert(Position::new(symbol, 0.0, 0.0));
        position.update(quantity, price);

        if position.quantity.abs() < 1e-9 {
            self.positions.remove(symbol);
        }

        Ok(())
    }

    pub fn total_equity(&self, current_prices: &HashMap<String, f64>) -> f64 {
        let mut equity = self.cash;
        for (symbol, position) in &self.positions {
            if let Some(&price) = current_prices.get(symbol) {
                equity += position.market_value(price);
            } else {
                // Fallback to last known price (avg_price) if current price is missing
                // In a real system, this might be an error or handled differently
                equity += position.market_value(position.avg_price);
            }
        }
        equity
    }

    pub fn record_equity(&mut self, timestamp: i64, current_prices: &HashMap<String, f64>) {
        let equity = self.total_equity(current_prices);
        self.equity_history.push((timestamp, equity));
    }
}
