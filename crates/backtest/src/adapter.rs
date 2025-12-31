use crate::error::Result;
use crate::strategy::{OrderRequest, OrderSide, OrderType, Strategy as BacktestStrategy};
use alphafield_core::{Bar, Tick};

/// Position state for the adapter
#[derive(Debug, Clone, Copy, PartialEq)]
enum PositionState {
    Flat,
    Long,
    #[allow(dead_code)]
    Short,
}

/// Adapter to bridge alphafield_core::Strategy (Signal-based) to alphafield_backtest::Strategy (Order-based)
///
/// This adapter tracks position state to ensure:
/// - Buy signals only create orders when flat (not already long)
/// - Sell signals close long positions or open short (depending on mode)
pub struct StrategyAdapter<T>
where
    T: alphafield_core::Strategy,
{
    inner: T,
    symbol: String,
    /// Starting capital for position sizing
    capital: f64,
    /// Percentage of capital to use per trade (e.g., 0.1 = 10%)
    trade_pct: f64,
    /// Current position state
    position: PositionState,
    /// Quantity held in current position (for proper exit sizing)
    position_quantity: f64,
}

impl<T> StrategyAdapter<T>
where
    T: alphafield_core::Strategy,
{
    pub fn new(strategy: T, symbol: impl Into<String>, capital: f64) -> Self {
        Self {
            inner: strategy,
            symbol: symbol.into(),
            capital,
            trade_pct: 0.10, // Default to 10% of capital per trade
            position: PositionState::Flat,
            position_quantity: 0.0,
        }
    }

    /// Helper to round quantity to 9 decimal places to match backtest engine precision
    fn round_quantity(quantity: f64) -> f64 {
        (quantity * 1_000_000_000.0).round() / 1_000_000_000.0
    }
}

impl<T> BacktestStrategy for StrategyAdapter<T>
where
    T: alphafield_core::Strategy,
{
    fn on_bar(&mut self, bar: &Bar) -> Result<Vec<OrderRequest>> {
        let signals = self.inner.on_bar(bar);
        let mut orders = Vec::new();

        if let Some(sigs) = signals {
            for sig in sigs {
                match sig.signal_type {
                    alphafield_core::SignalType::Buy => {
                        // Only buy if we're flat (not already in a position)
                        if self.position == PositionState::Flat {
                            // Calculate quantity based on % of capital at current price
                            let trade_value = self.capital * self.trade_pct * sig.strength;
                            let quantity = trade_value / bar.close;
                            let quantity = Self::round_quantity(quantity);

                            if quantity > 0.0 {
                                orders.push(OrderRequest {
                                    symbol: self.symbol.clone(),
                                    side: OrderSide::Buy,
                                    quantity,
                                    order_type: OrderType::Market,
                                });
                                self.position = PositionState::Long;
                                self.position_quantity = quantity;
                            }
                        }
                    }
                    alphafield_core::SignalType::Sell => {
                        // Only sell if we're long (to close position)
                        // Use the FULL position quantity, not a new calculated amount
                        if self.position == PositionState::Long && self.position_quantity > 0.0 {
                            let quantity = Self::round_quantity(self.position_quantity); // Sell entire position

                            if quantity > 0.0 {
                                orders.push(OrderRequest {
                                    symbol: self.symbol.clone(),
                                    side: OrderSide::Sell,
                                    quantity,
                                    order_type: OrderType::Market,
                                });
                                self.position = PositionState::Flat;
                                self.position_quantity = 0.0;
                            }
                        }
                    }
                    alphafield_core::SignalType::Hold => {}
                }
            }
        }

        Ok(orders)
    }

    fn on_tick(&mut self, tick: &Tick) -> Result<Vec<OrderRequest>> {
        let signal = self.inner.on_tick(tick);

        if let Some(sig) = signal {
            match sig.signal_type {
                alphafield_core::SignalType::Buy => {
                    if self.position == PositionState::Flat {
                        // Calculate quantity based on % of capital at current price
                        let trade_value = self.capital * self.trade_pct * sig.strength;
                        let quantity = Self::round_quantity(trade_value / tick.price);

                        self.position = PositionState::Long;
                        self.position_quantity = quantity;

                        Ok(vec![OrderRequest {
                            symbol: self.symbol.clone(),
                            side: OrderSide::Buy,
                            quantity,
                            order_type: OrderType::Market,
                        }])
                    } else {
                        Ok(Vec::new())
                    }
                }
                alphafield_core::SignalType::Sell => {
                    if self.position == PositionState::Long && self.position_quantity > 0.0 {
                        let quantity = Self::round_quantity(self.position_quantity); // Sell entire position
                        self.position = PositionState::Flat;
                        self.position_quantity = 0.0;

                        Ok(vec![OrderRequest {
                            symbol: self.symbol.clone(),
                            side: OrderSide::Sell,
                            quantity,
                            order_type: OrderType::Market,
                        }])
                    } else {
                        Ok(Vec::new())
                    }
                }
                alphafield_core::SignalType::Hold => Ok(Vec::new()),
            }
        } else {
            Ok(Vec::new())
        }
    }
}
