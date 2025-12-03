use crate::error::Result;
use alphafield_core::{Bar, Tick};

#[derive(Debug, Clone, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderType {
    Market,
    Limit(f64),
}

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub order_type: OrderType,
}

pub trait Strategy {
    fn on_bar(&mut self, bar: &Bar) -> Result<Vec<OrderRequest>>;
    fn on_tick(&mut self, _tick: &Tick) -> Result<Vec<OrderRequest>> {
        // Default implementation does nothing for ticks
        Ok(Vec::new())
    }
}

pub struct StrategyCombiner {
    strategies: Vec<Box<dyn Strategy>>,
}

impl StrategyCombiner {
    pub fn new(strategies: Vec<Box<dyn Strategy>>) -> Self {
        Self { strategies }
    }
}

impl Strategy for StrategyCombiner {
    fn on_bar(&mut self, bar: &Bar) -> Result<Vec<OrderRequest>> {
        let mut all_orders = Vec::new();
        for strategy in &mut self.strategies {
            let orders = strategy.on_bar(bar)?;
            all_orders.extend(orders);
        }
        Ok(all_orders)
    }

    fn on_tick(&mut self, tick: &Tick) -> Result<Vec<OrderRequest>> {
        let mut all_orders = Vec::new();
        for strategy in &mut self.strategies {
            let orders = strategy.on_tick(tick)?;
            all_orders.extend(orders);
        }
        Ok(all_orders)
    }
}
