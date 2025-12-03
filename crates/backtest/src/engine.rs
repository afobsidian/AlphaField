use crate::error::Result;
use crate::exchange::{ExchangeSimulator, SlippageModel};
use crate::metrics::PerformanceMetrics;
use crate::portfolio::Portfolio;
use crate::strategy::Strategy;
use alphafield_core::Bar;
use std::collections::HashMap;

pub struct BacktestEngine {
    pub portfolio: Portfolio,
    pub exchange: ExchangeSimulator,
    pub data: HashMap<String, Vec<Bar>>, // Symbol -> History
    pub strategy: Option<Box<dyn Strategy>>,
}

impl BacktestEngine {
    pub fn new(initial_cash: f64, fee_rate: f64, slippage: SlippageModel) -> Self {
        Self {
            portfolio: Portfolio::new(initial_cash),
            exchange: ExchangeSimulator::new(fee_rate, slippage),
            data: HashMap::new(),
            strategy: None,
        }
    }

    pub fn set_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategy = Some(strategy);
    }

    pub fn add_data(&mut self, symbol: &str, bars: Vec<Bar>) {
        self.data.insert(symbol.to_string(), bars);
    }

    pub fn run(&mut self) -> Result<PerformanceMetrics> {
        // 1. Align data by timestamp
        // For simplicity, assuming all symbols have data for the same timestamps for now.
        // In a real engine, we'd need a priority queue or merged iterator.

        // Find the common time range (intersection) or union depending on strategy requirements
        // Here we just iterate through the first symbol's data as a driver

        let driver_symbol = self
            .data
            .keys()
            .next()
            .ok_or(crate::error::BacktestError::Data(
                "No data loaded".to_string(),
            ))?
            .clone();
        let bars = self.data.get(&driver_symbol).unwrap();

        for bar in bars {
            let timestamp = bar.timestamp.timestamp_millis();

            // 2. Update Portfolio Market Value (Mark to Market)
            let mut current_prices = HashMap::new();
            // In a real loop, we'd get the price for ALL symbols at this timestamp
            // For now, just using the driver symbol's close price
            current_prices.insert(driver_symbol.clone(), bar.close);

            self.portfolio.record_equity(timestamp, &current_prices);

            // 3. Strategy Logic
            if let Some(strategy) = &mut self.strategy {
                let orders = strategy.on_bar(bar)?;
                for order in orders {
                    // Simple execution logic: Execute immediately at close price
                    // In reality, this would go to an order book or next bar open
                    let price = self.exchange.calculate_price(bar.close, order.quantity);
                    let fee = self.exchange.calculate_fee(price, order.quantity);

                    // Check if we have enough cash/position
                    // (Simplified check, real engine needs more robust validation)

                    self.portfolio
                        .update_from_fill(&order.symbol, order.quantity, price, fee)?;
                }
            }
        }

        Ok(PerformanceMetrics::calculate(
            &self.portfolio.equity_history,
            0.02,
        )) // 2% risk free rate
    }
}
