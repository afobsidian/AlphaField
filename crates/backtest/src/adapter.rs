use crate::error::Result;
use crate::strategy::{OrderRequest, OrderSide, OrderType, Strategy as BacktestStrategy};
use alphafield_core::{Bar, Tick};

/// Adapter to bridge alphafield_core::Strategy (Signal-based) to alphafield_backtest::Strategy (Order-based)
pub struct StrategyAdapter<T>
where
    T: alphafield_core::Strategy,
{
    inner: T,
    symbol: String,
    /// Base quantity to trade (e.g. number of units)
    quantity: f64,
}

impl<T> StrategyAdapter<T>
where
    T: alphafield_core::Strategy,
{
    pub fn new(strategy: T, symbol: impl Into<String>, quantity: f64) -> Self {
        Self {
            inner: strategy,
            symbol: symbol.into(),
            quantity,
        }
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
                        // Use fixed quantity scaled by signal strength
                        let quantity = self.quantity * sig.strength;

                        orders.push(OrderRequest {
                            symbol: self.symbol.clone(),
                            side: OrderSide::Buy,
                            quantity,
                            order_type: OrderType::Market,
                        });
                    }
                    alphafield_core::SignalType::Sell => {
                        // Sell signal - close position or short
                        let quantity = -self.quantity * sig.strength;

                        orders.push(OrderRequest {
                            symbol: self.symbol.clone(),
                            side: OrderSide::Sell,
                            quantity: quantity.abs(),
                            order_type: OrderType::Market,
                        });
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
                    let quantity = self.quantity * sig.strength;

                    Ok(vec![OrderRequest {
                        symbol: self.symbol.clone(),
                        side: OrderSide::Buy,
                        quantity,
                        order_type: OrderType::Market,
                    }])
                }
                alphafield_core::SignalType::Sell => {
                    let quantity = -self.quantity * sig.strength;

                    Ok(vec![OrderRequest {
                        symbol: self.symbol.clone(),
                        side: OrderSide::Sell,
                        quantity: quantity.abs(),
                        order_type: OrderType::Market,
                    }])
                }
                alphafield_core::SignalType::Hold => Ok(Vec::new()),
            }
        } else {
            Ok(Vec::new())
        }
    }
}
