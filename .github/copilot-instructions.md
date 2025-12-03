# AlphaField - AI Coding Agent Instructions

## Project Overview

AlphaField is a Rust-based algorithmic trading engine for crypto markets with an **event-driven backtesting philosophy** focused on avoiding overfitting ("Unbiased Trading"). The system is organized as a Cargo workspace with modular crates.

## Architecture & Crate Dependencies

```
core ← data, strategy, backtest, execution
data ← backtest
strategy ← backtest, execution
```

- **core** ([crates/core/src/lib.rs](crates/core/src/lib.rs)): Foundation types (`Bar`, `Tick`, `Quote`, `Signal`, `Order`) and traits (`Strategy`, `ExecutionService`). All price data uses `f64`, timestamps use `chrono::DateTime<Utc>`. Core structs are `Copy` for zero-cost stack allocation.
- **data** ([crates/data/src/lib.rs](crates/data/src/lib.rs)): API clients with **smart routing** and automatic fallbacks (Binance → CoinGecko → Coinlayer). Uses `UnifiedDataClient` as the main entry point with `ApiKeyPool` for key rotation.
- **strategy** ([crates/strategy/src/lib.rs](crates/strategy/src/lib.rs)): Indicators (`Sma`, `Ema`, `Rsi`) and example strategies (`GoldenCrossStrategy`, `RsiStrategy`).
- **backtest** ([crates/backtest/src/lib.rs](crates/backtest/src/lib.rs)): Event-driven simulator with latency injection, slippage modeling, and performance metrics (Sharpe, CAGR, Max Drawdown).
- **execution** ([crates/execution/src/lib.rs](crates/execution/src/lib.rs)): Risk management wrapper pattern using `RiskManager<S: ExecutionService>` with composable `RiskCheck` traits.

## Key Patterns

### Strategy Trait (Two Different Versions)
- **Core's `Strategy`** ([crates/core/src/lib.rs](crates/core/src/lib.rs#L290-L310)): Returns `Option<Signal>` - for signal generation
- **Backtest's `Strategy`** ([crates/backtest/src/strategy.rs](crates/backtest/src/strategy.rs)): Returns `Result<Vec<OrderRequest>>` - for order execution

When implementing strategies, choose the appropriate trait based on context.

### Indicator Pattern
Indicators implement the `Indicator` trait with `update()`, `value()`, `reset()`. Use `VecDeque` for windowed calculations. See [crates/strategy/src/indicators.rs](crates/strategy/src/indicators.rs) for examples.

### Data Validation
All core types have a `validate()` method. Always call `bar.validate()?` after constructing `Bar` from external data. Validation rules are documented in doc comments.

### Smart Routing
`UnifiedDataClient` automatically routes requests based on data type:
- **OHLC**: Binance (has volume) → CoinGecko (no volume) → Coinlayer (daily only)
- **Prices**: CoinGecko → Binance

Rate-limited keys are automatically rotated via `ApiKeyPool`.

## Development Workflow

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run the data demo (tests smart routing)
cargo run --bin data-demo --release

# Run backtest example
cargo run --example buy_and_hold -p alphafield_backtest
```

## Environment Setup

Create `.env` in project root with API keys (comma-separated for rotation):
```env
BINANCE_API_KEYS=key1,key2
COINGECKO_API_KEYS=key1
COINLAYER_API_KEYS=key1
```

Both singular (`BINANCE_API_KEY`) and plural forms are supported.

## Error Handling

Use `QuantError` from core for all error types. Pattern:
```rust
use alphafield_core::{QuantError, Result};
// QuantError variants: Io, Parse, DataValidation, NotFound, Api
```

Backtest has its own `BacktestError` with `InsufficientFunds` variant.

## Code Conventions

- All monetary values: `f64`
- Timestamps: `chrono::DateTime<Utc>`
- Async HTTP: `reqwest` with 30s timeout and connection pooling
- Serialization: `serde` with `#[derive(Serialize, Deserialize)]`
- Error derive: `thiserror::Error`
- Async traits: `#[async_trait::async_trait]`

## Testing Strategy

Unit tests live alongside code in `#[cfg(test)]` modules. Focus on:
- Data validation edge cases
- Indicator calculations with known values
- Portfolio position updates
