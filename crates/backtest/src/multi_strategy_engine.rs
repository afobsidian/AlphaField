//! Multi-Strategy Portfolio Backtest Engine
//!
//! Event-driven backtesting engine that runs multiple strategies simultaneously
//! with portfolio weight allocations. Supports dynamic rebalancing and
//! strategy-level performance tracking.

use crate::error::{BacktestError, Result};
use crate::exchange::{ExchangeSimulator, SlippageModel};
use crate::metrics::PerformanceMetrics;
use crate::portfolio::Portfolio;
use crate::strategy::Strategy;
use alphafield_core::{Bar, TradingMode};
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};

/// Configuration for multi-strategy backtest
#[derive(Debug, Clone)]
pub struct MultiStrategyConfig {
    /// Initial capital
    pub initial_capital: f64,
    /// Trading fee rate
    pub fee_rate: f64,
    /// Slippage model
    pub slippage: SlippageModel,
    /// Trading mode (Spot or Margin)
    pub trading_mode: TradingMode,
    /// Rebalancing frequency (0 = no rebalancing, N = every N bars)
    pub rebalancing_frequency: usize,
    /// Minimum weight threshold (strategies below this get 0 allocation)
    pub min_weight_threshold: f64,
}

impl Default for MultiStrategyConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10000.0,
            fee_rate: 0.001,
            slippage: SlippageModel::None,
            trading_mode: TradingMode::Spot,
            rebalancing_frequency: 0,
            min_weight_threshold: 0.01,
        }
    }
}

impl MultiStrategyConfig {
    /// Create new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set initial capital
    pub fn with_initial_capital(mut self, capital: f64) -> Self {
        self.initial_capital = capital.max(100.0);
        self
    }

    /// Set fee rate
    pub fn with_fee_rate(mut self, rate: f64) -> Self {
        self.fee_rate = rate.clamp(0.0, 0.1);
        self
    }

    /// Set slippage model
    pub fn with_slippage(mut self, slippage: SlippageModel) -> Self {
        self.slippage = slippage;
        self
    }

    /// Set trading mode
    pub fn with_trading_mode(mut self, mode: TradingMode) -> Self {
        self.trading_mode = mode;
        self
    }

    /// Set rebalancing frequency (in bars)
    pub fn with_rebalancing(mut self, frequency: usize) -> Self {
        self.rebalancing_frequency = frequency;
        self
    }

    /// Set minimum weight threshold
    pub fn with_min_weight_threshold(mut self, threshold: f64) -> Self {
        self.min_weight_threshold = threshold.clamp(0.0, 0.1);
        self
    }
}

/// Individual strategy performance during backtest
#[derive(Debug, Clone)]
pub struct StrategyPerformance {
    /// Strategy name
    pub strategy_name: String,
    /// Strategy weight in portfolio
    pub weight: f64,
    /// Number of trades executed
    pub num_trades: usize,
    /// Gross profit/loss
    pub gross_pnl: f64,
    /// Net profit/loss (after fees)
    pub net_pnl: f64,
    /// Individual strategy return
    pub strategy_return: f64,
    /// Maximum drawdown for this strategy
    pub max_drawdown: f64,
}

/// Complete multi-strategy backtest results
#[derive(Debug, Clone)]
pub struct MultiStrategyBacktestResult {
    /// Combined portfolio performance
    pub portfolio_metrics: PerformanceMetrics,
    /// Individual strategy performances
    pub strategy_performances: Vec<StrategyPerformance>,
    /// Final portfolio equity curve
    pub equity_curve: Vec<f64>,
    /// Weight history over time
    pub weight_history: Vec<HashMap<String, f64>>,
    /// Rebalancing events
    pub rebalancing_events: Vec<RebalancingEvent>,
    /// Total trades across all strategies
    pub total_trades: usize,
    /// Total fees paid
    pub total_fees: f64,
}

/// Rebalancing event record
#[derive(Debug, Clone)]
pub struct RebalancingEvent {
    /// Bar index when rebalancing occurred
    pub bar_index: usize,
    /// Timestamp of rebalancing
    pub timestamp: i64,
    /// Old weights
    pub old_weights: HashMap<String, f64>,
    /// New weights
    pub new_weights: HashMap<String, f64>,
    /// Rebalancing cost (fees from trades)
    pub rebalancing_cost: f64,
}

/// Multi-strategy backtest engine
pub struct MultiStrategyBacktestEngine {
    /// Configuration
    config: MultiStrategyConfig,
    /// Portfolio instance
    portfolio: Portfolio,
    /// Exchange simulator
    exchange: ExchangeSimulator,
    /// Strategy instances mapped by name
    strategies: HashMap<String, Box<dyn Strategy>>,
    /// Portfolio weights (strategy_name -> weight)
    weights: HashMap<String, f64>,
    /// Historical price data
    data: HashMap<String, Vec<Bar>>,
    /// Weight history for tracking
    weight_history: Vec<HashMap<String, f64>>,
    /// Rebalancing events
    rebalancing_events: Vec<RebalancingEvent>,
    /// Strategy trade counters
    strategy_trades: HashMap<String, usize>,
    /// Strategy PnL tracking
    strategy_pnl: HashMap<String, f64>,
}

impl MultiStrategyBacktestEngine {
    /// Create new multi-strategy backtest engine
    pub fn new(config: MultiStrategyConfig) -> Self {
        let portfolio =
            Portfolio::new(config.initial_capital).with_trading_mode(config.trading_mode);

        let exchange = ExchangeSimulator::new(config.fee_rate, config.slippage.clone());

        Self {
            config,
            portfolio,
            exchange,
            strategies: HashMap::new(),
            weights: HashMap::new(),
            data: HashMap::new(),
            weight_history: Vec::new(),
            rebalancing_events: Vec::new(),
            strategy_trades: HashMap::new(),
            strategy_pnl: HashMap::new(),
        }
    }

    /// Add a strategy with its weight
    pub fn add_strategy(
        &mut self,
        name: impl Into<String>,
        strategy: Box<dyn Strategy>,
        weight: f64,
    ) {
        let name = name.into();
        self.strategies.insert(name.clone(), strategy);
        self.weights.insert(name.clone(), weight);
        self.strategy_trades.insert(name.clone(), 0);
        self.strategy_pnl.insert(name, 0.0);
    }

    /// Set portfolio weights (can be used for dynamic rebalancing)
    pub fn set_weights(&mut self, weights: HashMap<String, f64>) {
        // Validate weights sum to approximately 1.0
        let total: f64 = weights.values().sum();
        if (total - 1.0).abs() > 0.01 {
            warn!(total = total, "Weights don't sum to 1.0, normalizing");
            // Normalize weights
            let normalized: HashMap<String, f64> =
                weights.into_iter().map(|(k, v)| (k, v / total)).collect();
            self.weights = normalized;
        } else {
            self.weights = weights;
        }
    }

    /// Add historical data for a symbol
    pub fn add_data(&mut self, symbol: &str, bars: Vec<Bar>) {
        info!(
            symbol = symbol,
            bar_count = bars.len(),
            "Loading data for multi-strategy backtest"
        );
        self.data.insert(symbol.to_string(), bars);
    }

    /// Run the multi-strategy backtest
    #[instrument(skip(self), fields(strategies = %self.strategies.len()))]
    pub fn run(&mut self) -> Result<MultiStrategyBacktestResult> {
        info!(
            initial_capital = self.config.initial_capital,
            num_strategies = self.strategies.len(),
            "Starting multi-strategy backtest"
        );

        // Validate we have strategies
        if self.strategies.is_empty() {
            return Err(BacktestError::Validation(
                "No strategies added to engine".to_string(),
            ));
        }

        // Validate we have data
        let driver_symbol = self
            .data
            .keys()
            .next()
            .ok_or_else(|| BacktestError::Data("No data loaded".to_string()))?
            .clone();
        let bars = self.data.get(&driver_symbol).unwrap().clone();

        // Validate weights sum to 1.0
        let weight_sum: f64 = self.weights.values().sum();
        if (weight_sum - 1.0).abs() > 0.01 {
            return Err(BacktestError::Validation(format!(
                "Portfolio weights must sum to 1.0, got {}",
                weight_sum
            )));
        }

        let mut equity_curve = vec![self.config.initial_capital];
        let mut orders_filled = 0u64;
        let mut orders_skipped = 0u64;
        let mut total_fees = 0.0;

        // Track equity history per strategy for individual performance
        let mut strategy_equity: HashMap<String, Vec<f64>> = self
            .strategies
            .keys()
            .map(|name| {
                (
                    name.clone(),
                    vec![self.config.initial_capital * self.weights.get(name).unwrap_or(&0.0)],
                )
            })
            .collect();

        for (bar_idx, bar) in bars.iter().enumerate() {
            let timestamp = bar.timestamp.timestamp_millis();

            // Check if we need to rebalance
            if self.should_rebalance(bar_idx) {
                let old_weights = self.weights.clone();
                // In a real implementation, you might call an optimizer here
                // For now, we just record the event if weights changed
                if bar_idx > 0 {
                    let event = RebalancingEvent {
                        bar_index: bar_idx,
                        timestamp,
                        old_weights: old_weights.clone(),
                        new_weights: self.weights.clone(),
                        rebalancing_cost: 0.0, // Would calculate actual cost
                    };
                    self.rebalancing_events.push(event);
                }
            }

            // Record weight history
            self.weight_history.push(self.weights.clone());

            // Update portfolio mark-to-market
            let mut current_prices = HashMap::new();
            current_prices.insert(driver_symbol.clone(), bar.close);
            self.portfolio.record_equity(timestamp, &current_prices);

            // Run each strategy and generate orders
            let mut all_orders: Vec<(String, crate::strategy::OrderRequest)> = Vec::new();

            for (strategy_name, strategy) in &mut self.strategies {
                let orders = strategy.on_bar(bar)?;
                for order in orders {
                    all_orders.push((strategy_name.clone(), order));
                }
            }

            // Execute orders for each strategy
            for (strategy_name, order) in all_orders {
                // Calculate position size based on strategy weight
                let strategy_weight = self.weights.get(&strategy_name).copied().unwrap_or(0.0);

                // Skip if weight is below threshold
                if strategy_weight < self.config.min_weight_threshold {
                    orders_skipped += 1;
                    continue;
                }

                // Scale order quantity by strategy weight
                let scaled_quantity = order.quantity * strategy_weight;

                let price = self.exchange.calculate_price(bar.close, scaled_quantity);
                let fee = self.exchange.calculate_fee(price, scaled_quantity);
                total_fees += fee;

                let fill_quantity = if order.side == crate::strategy::OrderSide::Sell {
                    -scaled_quantity
                } else {
                    scaled_quantity
                };

                match self.portfolio.update_from_fill(
                    &order.symbol,
                    fill_quantity,
                    price,
                    fee,
                    Some(format!("Strategy: {}", strategy_name)),
                ) {
                    Ok(_) => {
                        // Update strategy tracking
                        *self.strategy_trades.get_mut(&strategy_name).unwrap() += 1;
                        orders_filled += 1;

                        debug!(
                            strategy = strategy_name,
                            symbol = order.symbol,
                            quantity = scaled_quantity,
                            price = price,
                            "Order filled"
                        );
                    }
                    Err(BacktestError::InsufficientFunds {
                        required,
                        available,
                    }) => {
                        warn!(
                            strategy = strategy_name,
                            required = required,
                            available = available,
                            "Skipping trade: insufficient funds"
                        );
                        orders_skipped += 1;
                    }
                    Err(BacktestError::InsufficientPosition {
                        symbol,
                        required,
                        available,
                    }) => {
                        warn!(
                            strategy = strategy_name,
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

            // Update strategy equity curves
            for (name, equity) in &mut strategy_equity {
                if let Some(weight) = self.weights.get(name) {
                    let strategy_value = self.portfolio.total_equity(&current_prices) * weight;
                    equity.push(strategy_value);
                }
            }

            equity_curve.push(self.portfolio.total_equity(&current_prices));
        }

        // Force-close any open positions at the end
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
                        total_fees += fee;

                        let _ = self.portfolio.update_from_fill(
                            &symbol,
                            close_quantity,
                            price,
                            fee,
                            Some("Force-Close".to_string()),
                        );
                    }
                }
            }

            equity_curve.push(self.portfolio.total_equity(&current_prices));
        }

        // Calculate portfolio metrics
        let portfolio_metrics = PerformanceMetrics::calculate_with_trades(
            &self.portfolio.equity_history,
            &self.portfolio.trades,
            self.config.fee_rate,
        );

        // Calculate individual strategy performances
        let strategy_performances =
            self.calculate_strategy_performances(&strategy_equity, total_fees);

        info!(
            orders_filled = orders_filled,
            orders_skipped = orders_skipped,
            total_return = format!("{:.2}%", portfolio_metrics.total_return * 100.0),
            sharpe_ratio = format!("{:.2}", portfolio_metrics.sharpe_ratio),
            max_drawdown = format!("{:.2}%", portfolio_metrics.max_drawdown * 100.0),
            "Multi-strategy backtest completed"
        );

        Ok(MultiStrategyBacktestResult {
            portfolio_metrics,
            strategy_performances,
            equity_curve,
            weight_history: self.weight_history.clone(),
            rebalancing_events: self.rebalancing_events.clone(),
            total_trades: self.portfolio.trades.len(),
            total_fees,
        })
    }

    /// Check if rebalancing is needed
    fn should_rebalance(&self, bar_idx: usize) -> bool {
        if self.config.rebalancing_frequency == 0 {
            return false;
        }
        bar_idx > 0 && bar_idx.is_multiple_of(self.config.rebalancing_frequency)
    }

    /// Calculate individual strategy performances
    fn calculate_strategy_performances(
        &self,
        strategy_equity: &HashMap<String, Vec<f64>>,
        _total_fees: f64,
    ) -> Vec<StrategyPerformance> {
        let mut performances = Vec::new();

        for (name, equity) in strategy_equity {
            if equity.len() < 2 {
                continue;
            }

            let initial = equity[0];
            let final_val = equity[equity.len() - 1];
            let strategy_return = (final_val - initial) / initial;

            // Calculate max drawdown
            let mut peak = equity[0];
            let mut max_dd = 0.0;
            for &val in equity.iter().skip(1) {
                if val > peak {
                    peak = val;
                }
                let dd = (peak - val) / peak;
                if dd > max_dd {
                    max_dd = dd;
                }
            }

            let num_trades = *self.strategy_trades.get(name).unwrap_or(&0);
            let pnl = *self.strategy_pnl.get(name).unwrap_or(&0.0);

            performances.push(StrategyPerformance {
                strategy_name: name.clone(),
                weight: *self.weights.get(name).unwrap_or(&0.0),
                num_trades,
                gross_pnl: pnl,
                net_pnl: pnl, // Simplified - would subtract fees
                strategy_return,
                max_drawdown: max_dd,
            });
        }

        performances
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::BuyAndHold;

    #[test]
    fn test_multi_strategy_config() {
        let config = MultiStrategyConfig::new()
            .with_initial_capital(50000.0)
            .with_fee_rate(0.002)
            .with_rebalancing(20);

        assert_eq!(config.initial_capital, 50000.0);
        assert_eq!(config.fee_rate, 0.002);
        assert_eq!(config.rebalancing_frequency, 20);
    }

    #[test]
    fn test_engine_creation() {
        let config = MultiStrategyConfig::new();
        let engine = MultiStrategyBacktestEngine::new(config);

        assert!(engine.strategies.is_empty());
        assert!(engine.weights.is_empty());
    }

    #[test]
    fn test_add_strategy() {
        let config = MultiStrategyConfig::new();
        let mut engine = MultiStrategyBacktestEngine::new(config);

        let strategy = Box::new(BuyAndHold::new("TEST", 100.0));
        engine.add_strategy("BuyAndHold", strategy, 1.0);

        assert_eq!(engine.strategies.len(), 1);
        assert_eq!(engine.weights.get("BuyAndHold"), Some(&1.0));
    }

    #[test]
    fn test_set_weights_validation() {
        let config = MultiStrategyConfig::new();
        let mut engine = MultiStrategyBacktestEngine::new(config);

        let mut weights = HashMap::new();
        weights.insert("A".to_string(), 0.6);
        weights.insert("B".to_string(), 0.6); // Sums to 1.2

        engine.set_weights(weights);

        // Should be normalized to sum to 1.0
        let total: f64 = engine.weights.values().sum();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_no_strategies_error() {
        let config = MultiStrategyConfig::new();
        let mut engine = MultiStrategyBacktestEngine::new(config);

        // Add minimal data
        let bars = vec![Bar {
            timestamp: chrono::Utc::now(),
            open: 100.0,
            high: 101.0,
            low: 99.0,
            close: 100.0,
            volume: 1000.0,
        }];
        engine.add_data("TEST", bars);

        let result = engine.run();
        assert!(result.is_err());
    }

    #[test]
    fn test_weight_validation() {
        let config = MultiStrategyConfig::new();
        let mut engine = MultiStrategyBacktestEngine::new(config);

        // Add two strategies with weights that don't sum to 1.0
        engine.add_strategy("A", Box::new(BuyAndHold::new("TEST", 100.0)), 0.5);
        engine.add_strategy("B", Box::new(BuyAndHold::new("TEST", 100.0)), 0.3); // Only 0.8 total

        let bars = vec![
            Bar {
                timestamp: chrono::Utc::now(),
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1000.0,
            },
            Bar {
                timestamp: chrono::Utc::now() + chrono::Duration::hours(1),
                open: 100.0,
                high: 101.0,
                low: 99.0,
                close: 101.0,
                volume: 1000.0,
            },
        ];
        engine.add_data("TEST", bars);

        let result = engine.run();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("sum to 1.0"));
    }

    #[test]
    fn test_should_rebalance() {
        let config = MultiStrategyConfig::new().with_rebalancing(10);
        let engine = MultiStrategyBacktestEngine::new(config);

        assert!(!engine.should_rebalance(0)); // First bar
        assert!(!engine.should_rebalance(5)); // Not yet
        assert!(engine.should_rebalance(10)); // Rebalance time
        assert!(!engine.should_rebalance(11)); // Just after
        assert!(engine.should_rebalance(20)); // Next rebalance
    }

    #[test]
    fn test_no_rebalancing_when_disabled() {
        let config = MultiStrategyConfig::new().with_rebalancing(0); // Disabled
        let engine = MultiStrategyBacktestEngine::new(config);

        assert!(!engine.should_rebalance(0));
        assert!(!engine.should_rebalance(100));
        assert!(!engine.should_rebalance(1000));
    }
}
