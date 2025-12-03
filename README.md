# AlphaField 🚀

AlphaField is a high-performance, Rust-based algorithmic trading engine designed for crypto markets. It features a robust, multi-source data layer, modular strategy system, and event-driven backtesting engine.

## 🌟 Key Features

*   **Unified Data Layer**: Seamlessly integrates multiple data sources (Binance, CoinGecko, Coinlayer) behind a single interface.
*   **Smart Routing**: Automatically selects the best API for the job (e.g., Binance for OHLC, CoinGecko for market data).
*   **Backtesting Engine**: Event-driven engine to simulate strategies with historical data, slippage, and fee models.
*   **Strategy Integration**: Run any strategy (Golden Cross, RSI, etc.) in both backtest and live modes using the `StrategyAdapter`.
*   **Resilience**: Automatic fallbacks and API key rotation to handle rate limits and downtime.
*   **Performance**: Built in Rust for low-latency execution.
*   **Type Safety**: Strong typing for financial primitives (Bars, Prices, Timestamps).

## 🏗️ Architecture

The project is organized as a Rust workspace with the following crates:

*   `crates/core`: Core data structures and traits (`Bar`, `Trade`, `Strategy`).
*   `crates/data`: Data ingestion and API clients (`UnifiedDataClient`, `BinanceClient`, `CoinGeckoClient`).
*   `crates/strategy`: Strategy logic, indicators and example strategies (`Sma`, `Ema`, `Rsi`, `GoldenCross`, `RsiStrategy`).
*   `crates/execution`: Execution & risk wrappers (risk checks, `RiskManager` pattern).
*   `crates/backtest`: Event-driven backtesting engine (`BacktestEngine`, `Portfolio`, `ExchangeSimulator`) with `StrategyAdapter` for seamless integration.
*   `crates/dashboard`: Backend/dashboard glue (Axum + React intended integration).

## 🚀 Getting Started

### Prerequisites

*   Rust (latest stable)
*   Cargo

### Configuration

1.  Copy `.env.example` to `.env` (if available, otherwise create `.env`):
    ```bash
    cp .env.example .env
    ```
2.  Add your API keys to `.env`:
    ```env
    # Binance (Optional but recommended for OHLC)
    BINANCE_API_KEYS=your_key1,your_key2
    
    # CoinGecko (Optional, good for market data)
    COINGECKO_API_KEYS=your_key1
    
    # Coinlayer (Optional, fallback)
    COINLAYER_API_KEYS=your_key1
    ```

Note: the data layer supports both singular and plural env names for key rotation. You can set either `BINANCE_API_KEY` or `BINANCE_API_KEYS` (comma-separated) and similarly for `COINGECKO` / `COINLAYER`.

### Running the Demo

Verify your data connection and see the smart routing in action:

```bash
cargo run --bin data-demo --release
```

Run the simple buy and hold example:

```bash
cargo run --example buy_and_hold -p alphafield_backtest
```

Run the full Golden Cross strategy backtest:

```bash
cargo run --example golden_cross_backtest -p alphafield_backtest
```

## 📚 Documentation

*   [Detailed Design](doc/detailed_design.md)
*   [Project Plan](doc/project_plan.md)
*   [Roadmap](doc/roadmap.md)
*   [Architecture](doc/architecture.md)

## 🛠️ Development Commands

We use a `Makefile` to simplify common development tasks.

### Core Commands

| Command | Description |
|---------|-------------|
| `make build` | Build the entire project |
| `make test` | Run all tests |
| `make fmt` | Format code using `cargo fmt` |
| `make lint` | Run clippy linter |
| `make clean` | Remove build artifacts |
| `make reset` | Clean and rebuild |

### Running Examples

| Command | Description |
|---------|-------------|
| `make run-demo` | Run the data layer demo |
| `make run-backtest` | Run the Golden Cross backtest example |

## 📄 License

MIT
