use crate::error::{BacktestError, Result};
use crate::trade::{Trade, TradeSide};
use alphafield_core::TradingMode;
use chrono::{DateTime, TimeZone, Utc};
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

/// Tracks an open trade for MAE/MFE calculation
#[derive(Debug, Clone)]
struct OpenTrade {
    symbol: String,
    side: TradeSide,
    entry_time: DateTime<Utc>,
    entry_price: f64,
    quantity: f64,
    fees_paid: f64,
    /// Best price seen during trade (for MFE)
    best_price: f64,
    /// Worst price seen during trade (for MAE)
    worst_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub cash: f64,
    pub positions: HashMap<String, Position>,
    pub equity_history: Vec<(i64, f64)>, // Timestamp (ms), Equity
    /// Completed trades for metrics calculation
    pub trades: Vec<Trade>,
    /// Open trades being tracked (not serialized)
    #[serde(skip)]
    open_trades: HashMap<String, OpenTrade>,
    /// Current timestamp for trade tracking
    #[serde(skip)]
    current_timestamp: i64,
    /// Trading mode (Spot or Margin)
    pub trading_mode: TradingMode,
}

impl Portfolio {
    pub fn new(initial_cash: f64) -> Self {
        Self {
            cash: initial_cash,
            positions: HashMap::new(),
            equity_history: Vec::new(),
            trades: Vec::new(),
            open_trades: HashMap::new(),
            current_timestamp: 0,
            trading_mode: TradingMode::Spot,
        }
    }

    /// Set the trading mode (Spot or Margin)
    pub fn with_trading_mode(mut self, trading_mode: TradingMode) -> Self {
        self.trading_mode = trading_mode;
        self
    }

    /// Set current timestamp for trade tracking
    pub fn set_timestamp(&mut self, timestamp_ms: i64) {
        self.current_timestamp = timestamp_ms;
    }

    /// Update MAE/MFE for all open trades based on current prices
    pub fn update_open_trades(&mut self, current_prices: &HashMap<String, f64>) {
        for (symbol, open_trade) in &mut self.open_trades {
            if let Some(&price) = current_prices.get(symbol) {
                match open_trade.side {
                    TradeSide::Long => {
                        if price > open_trade.best_price {
                            open_trade.best_price = price;
                        }
                        if price < open_trade.worst_price {
                            open_trade.worst_price = price;
                        }
                    }
                    TradeSide::Short => {
                        // For short: lower price is better
                        if price < open_trade.best_price {
                            open_trade.best_price = price;
                        }
                        if price > open_trade.worst_price {
                            open_trade.worst_price = price;
                        }
                    }
                }
            }
        }
    }

    pub fn update_from_fill(
        &mut self,
        symbol: &str,
        quantity: f64,
        price: f64,
        fee: f64,
        exit_reason: Option<String>,
    ) -> Result<()> {
        let cost = quantity * price;

        // In Spot mode, prevent selling more than currently held (no shorting)
        if self.trading_mode == TradingMode::Spot && quantity < 0.0 {
            let available_qty = self
                .positions
                .get(symbol)
                .map(|p| p.quantity)
                .unwrap_or(0.0);
            if available_qty + quantity < -1e-9 {
                return Err(BacktestError::InsufficientPosition {
                    symbol: symbol.to_string(),
                    required: -quantity,
                    available: available_qty,
                });
            }
        }

        if self.cash - cost - fee < 0.0 {
            return Err(BacktestError::InsufficientFunds {
                required: cost + fee,
                available: self.cash,
            });
        }

        self.cash -= cost + fee;

        let prev_quantity = self
            .positions
            .get(symbol)
            .map(|p| p.quantity)
            .unwrap_or(0.0);

        let position = self
            .positions
            .entry(symbol.to_string())
            .or_insert(Position::new(symbol, 0.0, 0.0));

        let _entry_price_before = position.avg_price;
        position.update(quantity, price);

        // Track trades: opening new position
        if prev_quantity.abs() < 1e-9 && quantity.abs() > 1e-9 {
            // New position opened
            let side = if quantity > 0.0 {
                TradeSide::Long
            } else {
                TradeSide::Short
            };
            let entry_time = Utc.timestamp_millis_opt(self.current_timestamp).unwrap();

            self.open_trades.insert(
                symbol.to_string(),
                OpenTrade {
                    symbol: symbol.to_string(),
                    side,
                    entry_time,
                    entry_price: price,
                    quantity: quantity.abs(),
                    fees_paid: fee,
                    best_price: price,
                    worst_price: price,
                },
            );
        } else if position.quantity.abs() < 1e-9 {
            // Position closed - record completed trade
            if let Some(open) = self.open_trades.remove(symbol) {
                let exit_time = Utc.timestamp_millis_opt(self.current_timestamp).unwrap();
                let duration_secs = (exit_time - open.entry_time).num_seconds();

                // Calculate PnL
                let pnl = match open.side {
                    TradeSide::Long => {
                        (price - open.entry_price) * open.quantity - open.fees_paid - fee
                    }
                    TradeSide::Short => {
                        (open.entry_price - price) * open.quantity - open.fees_paid - fee
                    }
                };

                // Calculate MAE/MFE as dollar amounts
                let (mae, mfe) = match open.side {
                    TradeSide::Long => {
                        let mae = (open.entry_price - open.worst_price) * open.quantity;
                        let mfe = (open.best_price - open.entry_price) * open.quantity;
                        (mae.max(0.0), mfe.max(0.0))
                    }
                    TradeSide::Short => {
                        let mae = (open.worst_price - open.entry_price) * open.quantity;
                        let mfe = (open.entry_price - open.best_price) * open.quantity;
                        (mae.max(0.0), mfe.max(0.0))
                    }
                };

                self.trades.push(Trade {
                    symbol: open.symbol,
                    side: open.side,
                    entry_time: open.entry_time,
                    exit_time,
                    entry_price: open.entry_price,
                    exit_price: price,
                    quantity: open.quantity,
                    pnl,
                    fees: open.fees_paid + fee,
                    mae,
                    mfe,
                    duration_secs,
                    exit_reason: exit_reason.or_else(|| Some("Signal".to_string())),
                });
            }
        } else if (prev_quantity > 0.0 && quantity > 0.0) || (prev_quantity < 0.0 && quantity < 0.0)
        {
            // Adding to existing position - update fees
            if let Some(open) = self.open_trades.get_mut(symbol) {
                open.fees_paid += fee;
                open.quantity += quantity.abs();
            }
        }

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
                equity += position.market_value(position.avg_price);
            }
        }
        equity
    }

    pub fn record_equity(&mut self, timestamp: i64, current_prices: &HashMap<String, f64>) {
        self.set_timestamp(timestamp);
        self.update_open_trades(current_prices);
        let equity = self.total_equity(current_prices);
        self.equity_history.push((timestamp, equity));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_default_trading_mode_is_spot() {
        let portfolio = Portfolio::new(10000.0);
        assert_eq!(portfolio.trading_mode, TradingMode::Spot);
    }

    #[test]
    fn test_portfolio_with_trading_mode() {
        let portfolio = Portfolio::new(10000.0).with_trading_mode(TradingMode::Margin);
        assert_eq!(portfolio.trading_mode, TradingMode::Margin);
    }

    #[test]
    fn test_portfolio_trading_mode_spot() {
        let portfolio = Portfolio::new(10000.0).with_trading_mode(TradingMode::Spot);
        assert_eq!(portfolio.trading_mode, TradingMode::Spot);
    }

    // Phase 2: Test short position behavior
    #[test]
    fn test_spot_mode_prevents_short() {
        let mut portfolio = Portfolio::new(10000.0).with_trading_mode(TradingMode::Spot);

        // Try to sell (open short) when flat - should fail
        let result = portfolio.update_from_fill("BTCUSDT", -0.1, 50000.0, 5.0, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Insufficient position"));
    }

    #[test]
    fn test_margin_mode_allows_short() {
        let mut portfolio = Portfolio::new(10000.0).with_trading_mode(TradingMode::Margin);

        // Sell to open short - should succeed
        let result = portfolio.update_from_fill("BTCUSDT", -0.1, 50000.0, 5.0, None);
        assert!(result.is_ok());

        // Check that position is negative
        let position = portfolio.positions.get("BTCUSDT").unwrap();
        assert_eq!(position.quantity, -0.1);
    }

    #[test]
    fn test_margin_requirement_check() {
        let mut portfolio = Portfolio::new(10000.0).with_trading_mode(TradingMode::Margin);

        // Sell to open short
        portfolio
            .update_from_fill("BTCUSDT", -0.1, 50000.0, 5.0, None)
            .unwrap();

        // Check cash increased (from selling)
        assert!(portfolio.cash > 10000.0);

        // Position should be negative
        let position = portfolio.positions.get("BTCUSDT").unwrap();
        assert_eq!(position.quantity, -0.1);
    }
}
