use crate::error::Result;
use alphafield_core::{Bar, Tick};
use alphafield_strategy::StrategyMetadata;

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

    /// Get strategy metadata if available
    ///
    /// Returns None for strategies that don't implement MetadataStrategy.
    /// This enables regime-based validation without breaking backward compatibility.
    ///
    /// # Returns
    /// - Some(StrategyMetadata) if the strategy implements MetadataStrategy
    /// - None for strategies without metadata support
    fn metadata(&self) -> Option<StrategyMetadata> {
        None
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

/// Simple buy-and-hold strategy that invests on the first bar and holds until the end.
///
/// This strategy is useful as:
/// - A benchmark for comparing other strategies
/// - A simple strategy for walk-forward analysis tests
pub struct BuyAndHold {
    symbol: String,
    invested: bool,
    quantity: f64,
}

impl BuyAndHold {
    /// Create a new BuyAndHold strategy
    ///
    /// # Arguments
    /// * `symbol` - The trading symbol
    /// * `quantity` - The quantity to buy on the first bar
    pub fn new(symbol: &str, quantity: f64) -> Self {
        Self {
            symbol: symbol.to_string(),
            invested: false,
            quantity,
        }
    }
}

impl Strategy for BuyAndHold {
    fn on_bar(&mut self, _bar: &Bar) -> Result<Vec<OrderRequest>> {
        if !self.invested {
            self.invested = true;
            Ok(vec![OrderRequest {
                symbol: self.symbol.clone(),
                side: OrderSide::Buy,
                quantity: self.quantity,
                order_type: OrderType::Market,
            }])
        } else {
            Ok(Vec::new())
        }
    }
}

// Reverted SignalAdapter addition as StrategyAdapter exists
