use alphafield_core::{Tick, Quote};
use crate::error::{BacktestError, Result};

#[derive(Debug, Clone)]
pub struct ExchangeSimulator {
    pub fee_rate: f64,
    pub slippage_model: SlippageModel,
}

#[derive(Debug, Clone)]
pub enum SlippageModel {
    None,
    FixedPercent(f64),
    // Add more complex models later (e.g., based on volume/depth)
}

impl ExchangeSimulator {
    pub fn new(fee_rate: f64, slippage_model: SlippageModel) -> Self {
        Self {
            fee_rate,
            slippage_model,
        }
    }

    pub fn calculate_price(&self, price: f64, quantity: f64) -> f64 {
        match self.slippage_model {
            SlippageModel::None => price,
            SlippageModel::FixedPercent(pct) => {
                if quantity > 0.0 {
                    price * (1.0 + pct) // Buy slippage (higher price)
                } else {
                    price * (1.0 - pct) // Sell slippage (lower price)
                }
            }
        }
    }

    pub fn calculate_fee(&self, price: f64, quantity: f64) -> f64 {
        (price * quantity).abs() * self.fee_rate
    }
}
