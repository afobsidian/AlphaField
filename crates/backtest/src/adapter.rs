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
    /// Position size as a fraction of portfolio value (0.0 to 1.0)
    position_size: f64,
}

impl<T> StrategyAdapter<T>
where
    T: alphafield_core::Strategy,
{
    pub fn new(strategy: T, symbol: impl Into<String>, position_size: f64) -> Self {
        Self {
            inner: strategy,
            symbol: symbol.into(),
            position_size: position_size.clamp(0.0, 1.0),
        }
    }
}

impl<T> BacktestStrategy for StrategyAdapter<T>
where
    T: alphafield_core::Strategy,
{
    fn on_bar(&mut self, bar: &Bar) -> Result<Vec<OrderRequest>> {
        let signal = self.inner.on_bar(bar);

        if let Some(sig) = signal {
            match sig.signal_type {
                alphafield_core::SignalType::Buy => {
                    // For now, we use a fixed quantity calculation
                    // In a real system, this would query the portfolio for available cash
                    // and calculate position size accordingly
                    let quantity = 100.0 * self.position_size * sig.strength;

                    Ok(vec![OrderRequest {
                        symbol: self.symbol.clone(),
                        side: OrderSide::Buy,
                        quantity,
                        order_type: OrderType::Market,
                    }])
                }
                alphafield_core::SignalType::Sell => {
                    // Sell signal - close position or short
                    // For simplicity, we'll assume we're closing an existing position
                    let quantity = -100.0 * self.position_size * sig.strength;

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

    fn on_tick(&mut self, tick: &Tick) -> Result<Vec<OrderRequest>> {
        let signal = self.inner.on_tick(tick);

        if let Some(sig) = signal {
            match sig.signal_type {
                alphafield_core::SignalType::Buy => {
                    let quantity = 100.0 * self.position_size * sig.strength;

                    Ok(vec![OrderRequest {
                        symbol: self.symbol.clone(),
                        side: OrderSide::Buy,
                        quantity,
                        order_type: OrderType::Market,
                    }])
                }
                alphafield_core::SignalType::Sell => {
                    let quantity = -100.0 * self.position_size * sig.strength;

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
