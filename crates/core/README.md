# 🧱 AlphaField Core Crate

Defines the fundamental types and traits shared across the entire AlphaField ecosystem.

## Why This Crate Exists

This crate provides the foundational building blocks that all other AlphaField components depend on. By defining types and traits in one place, we ensure:

- **Type safety**: Consistent data structures across the entire system
- **Interface stability**: Clear contracts between components
- **Reusability**: Common patterns shared without code duplication

## 📦 Key Types

### Bar (OHLCV Candlestick)

Represents a single time slice of market data with Open, High, Low, Close, and Volume.

```rust
use alphafield_core::Bar;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

// Create a bar
let bar = Bar {
    symbol: "BTCUSDT".to_string(),
    timestamp: Utc::now(),
    open: 45000.0,
    high: 45100.0,
    low: 44900.0,
    close: 45050.0,
    volume: Decimal::from_str("1.5").unwrap(),
};

// Validate the bar (always validate external data)
assert!(bar.validate().is_ok());
```

### Trade (Completed Trade)

Represents a completed trade with performance metrics including P&L, MAE (Maximum Adverse Excursion), and MFE (Maximum Favorable Excursion).

```rust
use alphafield_core::Trade;
use chrono::{DateTime, Utc};

let trade = Trade {
    id: "trade_123".to_string(),
    symbol: "BTCUSDT".to_string(),
    side: Side::Buy,
    entry_price: 45000.0,
    exit_price: 45500.0,
    quantity: Decimal::from_str("1.0").unwrap(),
    entry_time: Utc::now(),
    exit_time: Utc::now(),
    pnl: Decimal::from_str("500").unwrap(),  // $500 profit
    mae: 100.0,  // Worst drawdown during trade
    mfe: 600.0,  // Best run during trade
};
```

### Order (Order Request)

Represents an order request with type (Market/Limit), side (Buy/Sell), and time-in-force.

```rust
use alphafield_core::{Order, OrderType, OrderSide, TimeInForce};

let order = Order {
    symbol: "BTCUSDT".to_string(),
    side: OrderSide::Buy,
    order_type: OrderType::Limit,
    price: Some(44900.0),
    quantity: Decimal::from_str("1.0").unwrap(),
    time_in_force: TimeInForce::GoodTillCanceled,
    timestamp: Utc::now(),
};
```

### Signal (Strategy Output)

Output from strategies with buy/sell/hold signals, confidence scores, and position sizing.

```rust
use alphafield_core::{Signal, SignalType};

// Strong buy signal with 0.9 confidence
let signal = Signal {
    signal_type: SignalType::Buy,
    confidence: 0.9,  // 0.0 to 1.0
    suggested_size: Decimal::from_str("1.5").unwrap(),
    timestamp: Utc::now(),
    reason: "Golden cross with strong volume".to_string(),
};

// Hold signal
let hold = Signal::hold(Utc::now());
```

### QuantError (Centralized Error Type)

Centralized error type using `thiserror` for the entire workspace.

```rust
use alphafield_core::QuantError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("API error: {0}")]
    Api(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[from]
    Quant(#[from] alphafield_core::QuantError),
}

// Propagate errors
fn process_data(data: &str) -> Result<(), MyError> {
    if data.is_empty() {
        return Err(MyError::InvalidData("Data cannot be empty".to_string()));
    }
    Ok(())
}
```

## 🧬 Traits

### Strategy Trait

The interface that all trading strategies must implement. This trait defines the contract between strategies and the backtesting/execution systems.

```rust
use alphafield_core::{Strategy, Signal, Bar};

// Implement Strategy for your custom strategy
struct MyStrategy {
    parameter_a: f64,
    parameter_b: usize,
}

impl Strategy for MyStrategy {
    /// Generate a trading signal based on market data
    fn generate_signal(&self, bars: &[Bar]) -> Option<Signal> {
        // Your strategy logic here
        if bars.len() < 10 {
            return None;
        }

        // Calculate indicators
        let current_price = bars.last()?.close;
        let sma = calculate_sma(&bars, 10);

        // Generate signal
        if current_price > sma {
            Some(Signal::buy(0.8, Decimal::from(1), Utc::now()))
        } else {
            Some(Signal::sell(0.8, Decimal::from(1), Utc::now()))
        }
    }

    /// Update strategy state with new market data
    fn update(&mut self, bar: &Bar) {
        // Update internal state
    }

    /// Get strategy name
    fn name(&self) -> &str {
        "MyStrategy"
    }
}
```

### DataSource Trait

Interface for data providers. Allows switching between different data sources (exchanges, databases, files).

```rust
use alphafield_core::{DataSource, Bar, QuantError};
use chrono::{DateTime, Utc};

struct MyDataSource {
    api_key: String,
}

impl DataSource for MyDataSource {
    /// Fetch bars for a symbol and time range
    async fn fetch_bars(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Bar>, QuantError> {
        // Implementation to fetch from API
        Ok(vec![])
    }

    /// Fetch latest bar
    async fn fetch_latest_bar(&self, symbol: &str) -> Result<Option<Bar>, QuantError> {
        // Implementation
        Ok(None)
    }
}
```

## Common Workflows

### Validating External Data

Always validate data before using it in calculations:

```rust
use alphafield_core::{Bar, QuantError};

fn process_bar(bar: &Bar) -> Result<f64, QuantError> {
    bar.validate()?;

    // Safe to use bar now
    let range = bar.high - bar.low;
    Ok(range)
}
```

### Creating a Simple Strategy

```rust
use alphafield_core::{Strategy, Signal, Bar};

struct SimpleMovingAverageStrategy {
    short_period: usize,
    long_period: usize,
}

impl SimpleMovingAverageStrategy {
    fn new(short_period: usize, long_period: usize) -> Self {
        Self {
            short_period,
            long_period,
        }
    }
}

impl Strategy for SimpleMovingAverageStrategy {
    fn generate_signal(&self, bars: &[Bar]) -> Option<Signal> {
        // Need enough data for both periods
        if bars.len() < self.long_period {
            return None;
        }

        // Calculate SMAs
        let short_sma = calculate_sma(bars, self.short_period);
        let long_sma = calculate_sma(bars, self.long_period);

        // Generate crossover signal
        if short_sma > long_sma {
            Some(Signal::buy(0.7, Decimal::from(1), Utc::now()))
        } else if short_sma < long_sma {
            Some(Signal::sell(0.7, Decimal::from(1), Utc::now()))
        } else {
            Some(Signal::hold(Utc::now()))
        }
    }

    fn update(&mut self, bar: &Bar) {
        // Update internal state if needed
    }

    fn name(&self) -> &str {
        "SimpleMovingAverage"
    }
}
```

### Error Handling Patterns

```rust
use alphafield_core::QuantError;

fn calculate_indicator(data: &[f64]) -> Result<f64, QuantError> {
    if data.is_empty() {
        return Err(QuantError::Parse("Data cannot be empty".to_string()));
    }

    if data.len() < 3 {
        return Err(QuantError::Parse("Need at least 3 data points".to_string()));
    }

    // Calculate indicator
    Ok(data.iter().sum::<f64>() / data.len() as f64)
}
```

## Best Practices

1. **Always validate external data**: Call `.validate()` on all `Bar` and `Trade` objects before using them
2. **Use `QuantError` for all errors**: Consistent error handling across the workspace
3. **Document public APIs**: Use `///` for all public functions, types, and methods
4. **Handle all error cases**: Use `?` to propagate errors, never `unwrap()` on user data
5. **Use Decimal for money**: Maintain precision for monetary calculations
6. **Use f64 for prices**: Crypto prices are floating-point
7. **Use chrono for timestamps**: Provides timezone awareness and rich operations
