# AlphaField 🚀

AlphaField is a high-performance, Rust-based algorithmic trading engine designed for crypto markets. It features a robust, multi-source data layer, modular strategy system, and event-driven backtesting engine.

## 🌟 Key Features

*   **Unified Data Layer**: Seamlessly integrates multiple data sources (Binance, CoinGecko, Coinlayer) behind a single interface.
*   **Smart Routing**: Automatically selects the best API for the job (e.g., Binance for OHLC, CoinGecko for market data).
*   **Resilience**: Automatic fallbacks and API key rotation to handle rate limits and downtime.
*   **Performance**: Built in Rust for low-latency execution.
*   **Type Safety**: Strong typing for financial primitives (Bars, Prices, Timestamps).

## 🏗️ Architecture

The project is organized as a Rust workspace with the following crates:

*   `crates/core`: Core data structures and traits (`Bar`, `Trade`, `Strategy`).
*   `crates/data`: Data ingestion and API clients (`UnifiedDataClient`, `BinanceClient`, `CoinGeckoClient`).
*   `crates/strategy`: (Planned) Strategy logic and signal generation.
*   `crates/execution`: (Planned) Order management and execution.
*   `crates/backtest`: (Planned) Event-driven backtesting engine.

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

### Running the Demo

Verify your data connection and see the smart routing in action:

```bash
cargo run --bin data-demo --release
```

## 📚 Documentation

*   [Detailed Design](doc/detailed_design.md)
*   [Project Plan](doc/project_plan.md)
*   [Roadmap](doc/roadmap.md)
*   [Architecture](doc/architecture.md)

## 🛠️ Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

## 📄 License

MIT
