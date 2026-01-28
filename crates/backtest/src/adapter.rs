use crate::error::Result;
use crate::strategy::{OrderRequest, OrderSide, OrderType, Strategy as BacktestStrategy};
use alphafield_core::{Bar, Tick, TradingMode};
use alphafield_strategy::MetadataStrategy;

/// Position state for the adapter
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionState {
    Flat,
    Long,
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
    /// Trading mode (Spot or Margin)
    trading_mode: TradingMode,
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
            trade_pct: 0.10,                 // Default to 10% of capital per trade
            trading_mode: TradingMode::Spot, // Default to Spot mode
            position: PositionState::Flat,
            position_quantity: 0.0,
        }
    }

    /// Set the trading mode (Spot or Margin)
    pub fn with_trading_mode(mut self, trading_mode: TradingMode) -> Self {
        self.trading_mode = trading_mode;
        self
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
                        match (self.trading_mode, self.position) {
                            // Spot mode: Only buy when flat
                            (TradingMode::Spot, PositionState::Flat) |
                            // Margin mode: Buy when flat (open long) or when short (close short)
                            (TradingMode::Margin, PositionState::Flat | PositionState::Short) => {
                                let quantity = if self.position == PositionState::Short {
                                    // Close short position - use full position quantity
                                    Self::round_quantity(self.position_quantity.abs())
                                } else {
                                    // Open long position - calculate new quantity
                                    let trade_value = self.capital * self.trade_pct * sig.strength;
                                    Self::round_quantity(trade_value / bar.close)
                                };

                                if quantity > 0.0 {
                                    orders.push(OrderRequest {
                                        symbol: self.symbol.clone(),
                                        side: OrderSide::Buy,
                                        quantity,
                                        order_type: OrderType::Market,
                                    });

                                    // Update position state
                                    if self.position == PositionState::Short {
                                        // Closing short -> Flat
                                        self.position = PositionState::Flat;
                                        self.position_quantity = 0.0;
                                    } else {
                                        // Opening long -> Long
                                        self.position = PositionState::Long;
                                        self.position_quantity = quantity;
                                    }
                                }
                            }
                            _ => {} // No action in other states
                        }
                    }
                    alphafield_core::SignalType::Sell => {
                        match (self.trading_mode, self.position) {
                            // Spot mode: Only sell when long (to close position)
                            (TradingMode::Spot, PositionState::Long) |
                            // Margin mode: Sell when flat (open short) or when long (close long)
                            (TradingMode::Margin, PositionState::Flat | PositionState::Long) => {
                                let quantity = if self.position == PositionState::Long {
                                    // Close long position - use full position quantity
                                    Self::round_quantity(self.position_quantity.abs())
                                } else {
                                    // Open short position - calculate new quantity
                                    let trade_value = self.capital * self.trade_pct * sig.strength;
                                    Self::round_quantity(trade_value / bar.close)
                                };

                                if quantity > 0.0 {
                                    orders.push(OrderRequest {
                                        symbol: self.symbol.clone(),
                                        side: OrderSide::Sell,
                                        quantity,
                                        order_type: OrderType::Market,
                                    });

                                    // Update position state
                                    if self.position == PositionState::Long {
                                        // Closing long -> Flat
                                        self.position = PositionState::Flat;
                                        self.position_quantity = 0.0;
                                    } else {
                                        // Opening short -> Short
                                        self.position = PositionState::Short;
                                        self.position_quantity = -quantity; // Store as negative for short
                                    }
                                }
                            }
                            _ => {} // No action in other states
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
                    match (self.trading_mode, self.position) {
                        // Spot mode: Only buy when flat
                        (TradingMode::Spot, PositionState::Flat) |
                        // Margin mode: Buy when flat (open long) or when short (close short)
                        (TradingMode::Margin, PositionState::Flat | PositionState::Short) => {
                            let quantity = if self.position == PositionState::Short {
                                Self::round_quantity(self.position_quantity.abs())
                            } else {
                                let trade_value = self.capital * self.trade_pct * sig.strength;
                                Self::round_quantity(trade_value / tick.price)
                            };

                            if self.position == PositionState::Short {
                                self.position = PositionState::Flat;
                                self.position_quantity = 0.0;
                            } else {
                                self.position = PositionState::Long;
                                self.position_quantity = quantity;
                            }

                            Ok(vec![OrderRequest {
                                symbol: self.symbol.clone(),
                                side: OrderSide::Buy,
                                quantity,
                                order_type: OrderType::Market,
                            }])
                        }
                        _ => Ok(Vec::new()),
                    }
                }
                alphafield_core::SignalType::Sell => {
                    match (self.trading_mode, self.position) {
                        // Spot mode: Only sell when long (to close position)
                        (TradingMode::Spot, PositionState::Long) |
                        // Margin mode: Sell when flat (open short) or when long (close long)
                        (TradingMode::Margin, PositionState::Flat | PositionState::Long) => {
                            let quantity = if self.position == PositionState::Long {
                                Self::round_quantity(self.position_quantity.abs())
                            } else {
                                let trade_value = self.capital * self.trade_pct * sig.strength;
                                Self::round_quantity(trade_value / tick.price)
                            };

                            if self.position == PositionState::Long {
                                self.position = PositionState::Flat;
                                self.position_quantity = 0.0;
                            } else {
                                self.position = PositionState::Short;
                                self.position_quantity = -quantity; // Store as negative for short
                            }

                            Ok(vec![OrderRequest {
                                symbol: self.symbol.clone(),
                                side: OrderSide::Sell,
                                quantity,
                                order_type: OrderType::Market,
                            }])
                        }
                        _ => Ok(Vec::new()),
                    }
                }
                alphafield_core::SignalType::Hold => Ok(Vec::new()),
            }
        } else {
            Ok(Vec::new())
        }
    }
}

/// Extension methods for StrategyAdapter when wrapped strategy implements MetadataStrategy
///
/// This provides a way to access strategy metadata for strategies that implement
/// the MetadataStrategy trait from alphafield_strategy.
impl<T> StrategyAdapter<T>
where
    T: alphafield_core::Strategy + MetadataStrategy,
{
    /// Get strategy metadata
    ///
    /// This method is only available when the wrapped strategy implements MetadataStrategy.
    /// It allows accessing strategy metadata such as category, description, and expected regimes.
    ///
    /// # Returns
    /// Strategy metadata including name, category, description, and expected market regimes
    pub fn get_metadata(&self) -> alphafield_strategy::StrategyMetadata {
        self.inner.metadata()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alphafield_core::{Signal, SignalType};

    // Mock strategy for testing
    struct MockStrategy {
        name: &'static str,
    }

    impl alphafield_core::Strategy for MockStrategy {
        fn name(&self) -> &str {
            self.name
        }

        fn on_bar(&mut self, _bar: &Bar) -> Option<Vec<Signal>> {
            Some(vec![Signal {
                timestamp: chrono::Utc::now(),
                symbol: "BTCUSDT".to_string(),
                signal_type: SignalType::Buy,
                strength: 1.0,
                metadata: None,
            }])
        }

        fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
            None
        }

        fn on_quote(&mut self, _quote: &alphafield_core::Quote) -> Option<Signal> {
            None
        }
    }

    #[test]
    fn test_strategy_adapter_default_trading_mode_is_spot() {
        let strategy = MockStrategy {
            name: "TestStrategy",
        };
        let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0);
        assert_eq!(adapter.trading_mode, TradingMode::Spot);
    }

    #[test]
    fn test_strategy_adapter_with_trading_mode() {
        let strategy = MockStrategy {
            name: "TestStrategy",
        };
        let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0)
            .with_trading_mode(TradingMode::Margin);
        assert_eq!(adapter.trading_mode, TradingMode::Margin);
    }

    #[test]
    fn test_strategy_adapter_trading_mode_spot() {
        let strategy = MockStrategy {
            name: "TestStrategy",
        };
        let adapter =
            StrategyAdapter::new(strategy, "BTCUSDT", 10000.0).with_trading_mode(TradingMode::Spot);
        assert_eq!(adapter.trading_mode, TradingMode::Spot);
    }

    // Test Spot mode behavior - only long positions
    #[test]
    fn test_spot_mode_buy_when_flat() {
        let strategy = MockStrategy {
            name: "TestStrategy",
        };
        let mut adapter =
            StrategyAdapter::new(strategy, "BTCUSDT", 10000.0).with_trading_mode(TradingMode::Spot);

        let bar = Bar {
            timestamp: chrono::Utc::now(),
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 100.0,
        };

        let orders = adapter.on_bar(&bar).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].side, OrderSide::Buy);
        // Should calculate quantity: 10000 * 0.10 * 1.0 / 50050 ≈ 0.01998
        assert!(orders[0].quantity > 0.0);
    }

    #[test]
    fn test_spot_mode_sell_when_long() {
        let strategy = MockStrategy {
            name: "TestStrategy",
        };
        // First call returns Buy signal
        let mut adapter =
            StrategyAdapter::new(strategy, "BTCUSDT", 10000.0).with_trading_mode(TradingMode::Spot);

        let bar1 = Bar {
            timestamp: chrono::Utc::now(),
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 100.0,
        };

        let orders1 = adapter.on_bar(&bar1).unwrap();
        assert_eq!(orders1.len(), 1);
        assert_eq!(orders1[0].side, OrderSide::Buy);

        // Simulate being in Long position by manually setting state
        adapter.position = PositionState::Long;
        adapter.position_quantity = 0.02;

        // Now test sell when long - create a mock that returns Sell
        struct SellStrategy;
        impl alphafield_core::Strategy for SellStrategy {
            fn name(&self) -> &str {
                "SellStrategy"
            }

            fn on_bar(&mut self, _bar: &Bar) -> Option<Vec<Signal>> {
                Some(vec![Signal {
                    timestamp: chrono::Utc::now(),
                    symbol: "BTCUSDT".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: None,
                }])
            }

            fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
                None
            }

            fn on_quote(&mut self, _quote: &alphafield_core::Quote) -> Option<Signal> {
                None
            }
        }

        let mut adapter2 = StrategyAdapter::new(SellStrategy, "BTCUSDT", 10000.0)
            .with_trading_mode(TradingMode::Spot);
        adapter2.position = PositionState::Long;
        adapter2.position_quantity = 0.02;

        let orders2 = adapter2.on_bar(&bar1).unwrap();
        assert_eq!(orders2.len(), 1);
        assert_eq!(orders2[0].side, OrderSide::Sell);
        assert_eq!(orders2[0].quantity, 0.02);
    }

    #[test]
    fn test_spot_mode_no_short_when_flat() {
        struct SellStrategy;
        impl alphafield_core::Strategy for SellStrategy {
            fn name(&self) -> &str {
                "SellStrategy"
            }

            fn on_bar(&mut self, _bar: &Bar) -> Option<Vec<Signal>> {
                Some(vec![Signal {
                    timestamp: chrono::Utc::now(),
                    symbol: "BTCUSDT".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: None,
                }])
            }

            fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
                None
            }

            fn on_quote(&mut self, _quote: &alphafield_core::Quote) -> Option<Signal> {
                None
            }
        }

        let mut adapter = StrategyAdapter::new(SellStrategy, "BTCUSDT", 10000.0)
            .with_trading_mode(TradingMode::Spot);

        let bar = Bar {
            timestamp: chrono::Utc::now(),
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 100.0,
        };

        let orders = adapter.on_bar(&bar).unwrap();
        // In Spot mode, flat + sell signal should NOT open short
        assert_eq!(orders.len(), 0);
    }

    // Test Margin mode behavior - both long and short positions
    #[test]
    fn test_margin_mode_short_when_flat() {
        struct SellStrategy;
        impl alphafield_core::Strategy for SellStrategy {
            fn name(&self) -> &str {
                "SellStrategy"
            }

            fn on_bar(&mut self, _bar: &Bar) -> Option<Vec<Signal>> {
                Some(vec![Signal {
                    timestamp: chrono::Utc::now(),
                    symbol: "BTCUSDT".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: None,
                }])
            }

            fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
                None
            }

            fn on_quote(&mut self, _quote: &alphafield_core::Quote) -> Option<Signal> {
                None
            }
        }

        let mut adapter = StrategyAdapter::new(SellStrategy, "BTCUSDT", 10000.0)
            .with_trading_mode(TradingMode::Margin);

        let bar = Bar {
            timestamp: chrono::Utc::now(),
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 100.0,
        };

        let orders = adapter.on_bar(&bar).unwrap();
        // In Margin mode, flat + sell signal should open short
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].side, OrderSide::Sell);
        assert!(orders[0].quantity > 0.0);
        assert_eq!(adapter.position, PositionState::Short);
    }

    #[test]
    fn test_margin_mode_close_short_with_buy() {
        struct BuyStrategy;
        impl alphafield_core::Strategy for BuyStrategy {
            fn name(&self) -> &str {
                "BuyStrategy"
            }

            fn on_bar(&mut self, _bar: &Bar) -> Option<Vec<Signal>> {
                Some(vec![Signal {
                    timestamp: chrono::Utc::now(),
                    symbol: "BTCUSDT".to_string(),
                    signal_type: SignalType::Buy,
                    strength: 1.0,
                    metadata: None,
                }])
            }

            fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
                None
            }

            fn on_quote(&mut self, _quote: &alphafield_core::Quote) -> Option<Signal> {
                None
            }
        }

        let mut adapter = StrategyAdapter::new(BuyStrategy, "BTCUSDT", 10000.0)
            .with_trading_mode(TradingMode::Margin);

        // Start in Short position
        adapter.position = PositionState::Short;
        adapter.position_quantity = -0.02;

        let bar = Bar {
            timestamp: chrono::Utc::now(),
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 100.0,
        };

        let orders = adapter.on_bar(&bar).unwrap();
        // In Margin mode, short + buy signal should close short
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].side, OrderSide::Buy);
        assert_eq!(orders[0].quantity, 0.02); // Should buy back full position
        assert_eq!(adapter.position, PositionState::Flat);
    }

    #[test]
    fn test_margin_mode_position_transitions() {
        // Test full cycle: Flat -> Long -> Flat -> Short -> Flat

        // Test 1: Flat -> Long with buy signal
        let bar = Bar {
            timestamp: chrono::Utc::now(),
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 100.0,
        };

        struct BuyStrategy;
        impl alphafield_core::Strategy for BuyStrategy {
            fn name(&self) -> &str {
                "BuyStrategy"
            }

            fn on_bar(&mut self, _bar: &Bar) -> Option<Vec<Signal>> {
                Some(vec![Signal {
                    timestamp: chrono::Utc::now(),
                    symbol: "BTCUSDT".to_string(),
                    signal_type: SignalType::Buy,
                    strength: 1.0,
                    metadata: None,
                }])
            }

            fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
                None
            }

            fn on_quote(&mut self, _quote: &alphafield_core::Quote) -> Option<Signal> {
                None
            }
        }

        // Start flat
        let mut adapter = StrategyAdapter::new(BuyStrategy, "BTCUSDT", 10000.0)
            .with_trading_mode(TradingMode::Margin);

        assert_eq!(adapter.position, PositionState::Flat);
        assert_eq!(adapter.position_quantity, 0.0);

        // Buy signal - go Long
        let orders = adapter.on_bar(&bar).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].side, OrderSide::Buy);
        assert_eq!(adapter.position, PositionState::Long);
        assert!(adapter.position_quantity > 0.0);
    }

    #[test]
    fn test_margin_mode_long_to_short_via_flat() {
        // Test transition: Long -> Flat -> Short (cannot go directly from Long to Short)

        let bar = Bar {
            timestamp: chrono::Utc::now(),
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 100.0,
        };

        // Create adapter with sell strategy
        struct SellStrategy;
        impl alphafield_core::Strategy for SellStrategy {
            fn name(&self) -> &str {
                "SellStrategy"
            }

            fn on_bar(&mut self, _bar: &Bar) -> Option<Vec<Signal>> {
                Some(vec![Signal {
                    timestamp: chrono::Utc::now(),
                    symbol: "BTCUSDT".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: None,
                }])
            }

            fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
                None
            }

            fn on_quote(&mut self, _quote: &alphafield_core::Quote) -> Option<Signal> {
                None
            }
        }

        let mut adapter = StrategyAdapter::new(SellStrategy, "BTCUSDT", 10000.0)
            .with_trading_mode(TradingMode::Margin);

        // Start in Long position
        adapter.position = PositionState::Long;
        adapter.position_quantity = 0.02;

        // Sell signal - close Long, go Flat
        let orders = adapter.on_bar(&bar).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].side, OrderSide::Sell);
        assert_eq!(adapter.position, PositionState::Flat);
        assert_eq!(adapter.position_quantity, 0.0);

        // Another sell signal - open Short
        let orders = adapter.on_bar(&bar).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].side, OrderSide::Sell);
        assert_eq!(adapter.position, PositionState::Short);
        assert!(adapter.position_quantity < 0.0);
    }

    #[test]
    fn test_margin_mode_short_to_long_via_flat() {
        // Test transition: Short -> Flat -> Long (cannot go directly from Short to Long)

        let bar = Bar {
            timestamp: chrono::Utc::now(),
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 100.0,
        };

        // Create adapter with buy strategy
        struct BuyStrategy;
        impl alphafield_core::Strategy for BuyStrategy {
            fn name(&self) -> &str {
                "BuyStrategy"
            }

            fn on_bar(&mut self, _bar: &Bar) -> Option<Vec<Signal>> {
                Some(vec![Signal {
                    timestamp: chrono::Utc::now(),
                    symbol: "BTCUSDT".to_string(),
                    signal_type: SignalType::Buy,
                    strength: 1.0,
                    metadata: None,
                }])
            }

            fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
                None
            }

            fn on_quote(&mut self, _quote: &alphafield_core::Quote) -> Option<Signal> {
                None
            }
        }

        let mut adapter = StrategyAdapter::new(BuyStrategy, "BTCUSDT", 10000.0)
            .with_trading_mode(TradingMode::Margin);

        // Start in Short position
        adapter.position = PositionState::Short;
        adapter.position_quantity = -0.02;

        // Buy signal - close Short, go Flat
        let orders = adapter.on_bar(&bar).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].side, OrderSide::Buy);
        assert_eq!(adapter.position, PositionState::Flat);
        assert_eq!(adapter.position_quantity, 0.0);

        // Another buy signal - open Long
        let orders = adapter.on_bar(&bar).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].side, OrderSide::Buy);
        assert_eq!(adapter.position, PositionState::Long);
        assert!(adapter.position_quantity > 0.0);
    }
}
