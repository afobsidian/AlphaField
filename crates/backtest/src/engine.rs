use crate::error::Result;
use crate::exchange::{ExchangeSimulator, SlippageModel};
use crate::metrics::PerformanceMetrics;
use crate::portfolio::Portfolio;
use crate::strategy::Strategy;
use alphafield_core::Bar;
use std::collections::HashMap;
use tracing::{debug, info, instrument, trace, warn};

pub struct BacktestEngine {
    pub portfolio: Portfolio,
    pub exchange: ExchangeSimulator,
    pub data: HashMap<String, Vec<Bar>>, // Symbol -> History
    pub strategy: Option<Box<dyn Strategy>>,
}

impl BacktestEngine {
    pub fn new(initial_cash: f64, fee_rate: f64, slippage: SlippageModel) -> Self {
        debug!(
            initial_cash = initial_cash,
            fee_rate = fee_rate,
            "Creating new BacktestEngine"
        );
        Self {
            portfolio: Portfolio::new(initial_cash),
            exchange: ExchangeSimulator::new(fee_rate, slippage),
            data: HashMap::new(),
            strategy: None,
        }
    }

    pub fn set_strategy(&mut self, strategy: Box<dyn Strategy>) {
        debug!("Strategy set on BacktestEngine");
        self.strategy = Some(strategy);
    }

    pub fn add_data(&mut self, symbol: &str, bars: Vec<Bar>) {
        info!(
            symbol = symbol,
            bar_count = bars.len(),
            "Data loaded for backtest"
        );
        self.data.insert(symbol.to_string(), bars);
    }

    #[instrument(skip(self), fields(symbols = ?self.data.keys().collect::<Vec<_>>()))]
    pub fn run(&mut self) -> Result<PerformanceMetrics> {
        info!("Starting backtest run");

        let driver_symbol = self
            .data
            .keys()
            .next()
            .ok_or(crate::error::BacktestError::Data(
                "No data loaded".to_string(),
            ))?
            .clone();
        let bars = self.data.get(&driver_symbol).unwrap();

        info!(
            symbol = driver_symbol,
            total_bars = bars.len(),
            "Processing bars"
        );

        let mut orders_filled = 0u64;
        let mut orders_skipped = 0u64;

        for (bar_idx, bar) in bars.iter().enumerate() {
            let timestamp = bar.timestamp.timestamp_millis();

            // Update Portfolio Market Value (Mark to Market)
            let mut current_prices = HashMap::new();
            current_prices.insert(driver_symbol.clone(), bar.close);

            self.portfolio.record_equity(timestamp, &current_prices);

            trace!(
                bar_idx = bar_idx,
                timestamp = %bar.timestamp,
                close = bar.close,
                "Processing bar"
            );

            // Strategy Logic
            if let Some(strategy) = &mut self.strategy {
                let orders = strategy.on_bar(bar)?;
                for order in orders {
                    let price = self.exchange.calculate_price(bar.close, order.quantity);
                    let fee = self.exchange.calculate_fee(price, order.quantity);

                    let fill_quantity = if order.side == crate::strategy::OrderSide::Sell {
                        -order.quantity
                    } else {
                        order.quantity
                    };

                    match self.portfolio.update_from_fill(
                        &order.symbol,
                        fill_quantity,
                        price,
                        fee,
                        None,
                    ) {
                        Ok(_) => {
                            debug!(
                                symbol = order.symbol,
                                quantity = order.quantity,
                                price = price,
                                fee = fee,
                                "Order filled"
                            );
                            orders_filled += 1;
                        }
                        Err(crate::error::BacktestError::InsufficientFunds {
                            required,
                            available,
                        }) => {
                            warn!(
                                required = required,
                                available = available,
                                "Skipping trade: insufficient funds"
                            );
                            orders_skipped += 1;
                        }
                        Err(crate::error::BacktestError::InsufficientPosition {
                            symbol,
                            required,
                            available,
                        }) => {
                            warn!(
                                symbol = symbol,
                                required = required,
                                available = available,
                                "Skipping trade: insufficient position"
                            );
                            orders_skipped += 1;
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
        }

        // Force-close any open positions at the end of the backtest
        // to ensure they are recorded as completed trades
        if let Some(last_bar) = bars.last() {
            let mut current_prices = HashMap::new();
            current_prices.insert(driver_symbol.clone(), last_bar.close);

            let open_symbols: Vec<String> = self.portfolio.positions.keys().cloned().collect();
            for symbol in open_symbols {
                if let Some(pos) = self.portfolio.positions.get(&symbol) {
                    if pos.quantity.abs() > 1e-9 {
                        let close_quantity = -pos.quantity;
                        let price = last_bar.close;
                        let fee = self.exchange.calculate_fee(price, close_quantity.abs());

                        // Force close the position
                        let _ = self.portfolio.update_from_fill(
                            &symbol,
                            close_quantity,
                            price,
                            fee,
                            Some("Force-Close".to_string()),
                        );
                        debug!(
                            symbol = symbol,
                            quantity = close_quantity,
                            "Force-closed open position at backtest end"
                        );
                    }
                }
            }
        }

        let metrics = PerformanceMetrics::calculate_with_trades(
            &self.portfolio.equity_history,
            &self.portfolio.trades,
            0.02,
        );

        info!(
            orders_filled = orders_filled,
            orders_skipped = orders_skipped,
            total_trades = self.portfolio.trades.len(),
            total_return = format!("{:.2}%", metrics.total_return * 100.0),
            sharpe_ratio = format!("{:.2}", metrics.sharpe_ratio),
            max_drawdown = format!("{:.2}%", metrics.max_drawdown * 100.0),
            "Backtest completed"
        );

        Ok(metrics)
    }
}
