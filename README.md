# AlphaField 🚀

**A high-performance, Rust-based algorithmic trading engine for cryptocurrency markets.**

AlphaField features a robust multi-source data layer, modular strategy system, event-driven backtesting engine, and real-time dashboard with WebSocket streaming.

[![Build Status](https://github.com/alphafield/alphafield/workflows/CI/badge.svg)](https://github.com/alphafield/alphafield/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## 🌟 Key Features

| Feature | Description |
|---------|-------------|
| **Unified Data Layer** | Multi-source integration (Binance, CoinGecko, Coinlayer) with smart routing |
| **Interactive Dashboard** | Real-time data management, symbol search, and visual backtesting |
| **TimescaleDB Storage** | Time-series optimized with hypertables and compression |
| **Event-Driven Backtesting** | Simulate strategies with slippage, fees, and latency modeling |
| **Sentiment Analysis** | Fear & Greed Index integration + Asset-specific sentiment metrics |
| **Advanced Analytics** | Monte Carlo, Walk-Forward, Sensitivity Analysis, Correlation Matrix |
| **Data Quality Monitoring** | Gap detection, outlier detection, ingestion alerting |
| **Risk Management** | Circuit breakers, position limits, drift monitoring |

---

## 🏗️ Architecture

```
alphafield/
├── crates/
│   ├── core/           # Fundamental types: Bar, Trade, Order, Signal
│   ├── data/           # Data ingestion, APIs, TimescaleDB, monitoring
│   ├── strategy/       # Technical indicators and trading strategies
│   ├── backtest/       # Event-driven backtesting engine
│   ├── execution/      # Risk management and order execution
│   └── dashboard/      # Axum web server with REST & WebSocket APIs
├── doc/                # Documentation
├── Dockerfile          # Multi-stage production build
└── docker-compose.yml  # Full stack with TimescaleDB
```

### Crate Dependencies

```mermaid
graph TD
    Data[crates/data] --> Core[crates/core]
    Strategy[crates/strategy] --> Core
    Backtest[crates/backtest] --> Core
    Backtest --> Data
    Backtest --> Strategy
    Execution[crates/execution] --> Core
    Dashboard[crates/dashboard] --> Data
    Dashboard --> Backtest
    Dashboard --> Strategy
```

---

## 📦 Crates Overview

### `crates/core`
Core data structures and traits shared across the system.

| Type | Description |
|------|-------------|
| `Bar` | OHLCV candlestick data |
| `Trade` | Individual trade execution |
| `Order` | Order request with side, quantity, price |
| `Signal` | Strategy output (Buy/Sell/Hold) |
| `Strategy` trait | Interface for all trading strategies |

---

### `crates/data`
Data ingestion and storage with multi-source support.

| Module | Description |
|--------|-------------|
| `UnifiedDataClient` | Smart router across Binance/CoinGecko/Coinlayer |
| `BinanceClient` | OHLC klines, 24hr ticker, exchange info |
| `CoinGeckoClient` | Market data, OHLC, coin info |
| `DatabaseClient` | TimescaleDB with hypertables & compression |
| `DataPipeline` | Real-time streaming with callbacks |
| `IngestionMonitor` | Failure alerting & freshness tracking |
| `GapFiller` | Forward-fill missing bars |

**TimescaleDB Features:**
- Hypertables for `candles` and `trades`
- Compression policies (7 days for candles, 1 day for trades)
- Survivorship bias tracking (`asset_status` table)
- Data integrity checks (gaps, outliers)

---

### `crates/strategy`
Technical indicators and example strategies.

**Indicators:**
| Indicator | Description |
|-----------|-------------|
| `Sma` | Simple Moving Average |
| `Ema` | Exponential Moving Average |
| `Rsi` | Relative Strength Index |
| `Macd` | MACD with signal line |
| `BollingerBands` | Mean reversion bands |
| `Atr` | Average True Range |
| `Adx` | Average Directional Index |

**Strategies:**
| Strategy | Logic |
|----------|-------|
| `GoldenCross` | SMA crossover (50/200) |
| `RsiStrategy` | RSI oversold/overbought reversal |
| `MeanReversion` | Bollinger Band reversion |
| `Momentum` | MACD crossover |
| `TrendFollowing` | EMA trend with ADX filter |

---

### `crates/backtest`
Event-driven backtesting engine with advanced analytics.

| Component | Description |
|-----------|-------------|
| `BacktestEngine` | Main engine processing bars sequentially |
| `Portfolio` | Virtual account state (cash, positions, equity) |
| `ExchangeSimulator` | Order matching with slippage/fees |
| `StrategyAdapter` | Bridges Strategy trait to backtest engine |
| `PerformanceMetrics` | CAGR, Sharpe, Sortino, Max Drawdown, etc. |

**Advanced Analysis:**
| Module | Description |
|--------|-------------|
| `WalkForwardAnalyzer` | Rolling train/test validation |
| `MonteCarloSimulator` | Trade sequence shuffling for robustness |
| `SensitivityAnalyzer` | Parameter grid search with heatmaps |
| `CorrelationAnalyzer` | Multi-strategy correlation matrix |

---

### `crates/execution`
Risk management and order execution safeguards.

| Risk Check | Description |
|------------|-------------|
| `MaxOrderValue` | Reject orders exceeding value limit |
| `NoShorts` | Prevent short selling |
| `MaxDailyLoss` | Circuit breaker on daily PnL |
| `PositionDrift` | Alert on slippage exceeding threshold |
| `VolatilityScaledSize` | ATR-based sizing |

---

### `crates/dashboard`
Axum web server with REST API and WebSocket streaming.

**REST API Endpoints:**

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Server health check |
| `/api/portfolio` | GET | Current portfolio state |
| `/api/positions` | GET | Open positions |
| `/api/orders` | GET | Order history |
| `/api/performance` | GET | Performance metrics |
| `/api/backtest/run` | POST | Run backtest with optional sentiment data |
| `/api/monte-carlo` | POST | Run Monte Carlo simulation |
| `/api/correlation` | POST | Calculate strategy correlation |
| `/api/sensitivity` | POST | Parameter sensitivity analysis |
| `/api/data/symbols` | GET | List cached symbols |
| `/api/data/pairs` | GET | Available trading pairs (Binance) |
| `/api/data/fetch` | POST | Fetch new market data |
| `/api/data/:symbol/:interval` | DELETE | Delete cached data |
| `/api/sentiment/current` | GET | Current Fear & Greed Index |
| `/api/sentiment/history` | GET | Historical sentiment data |
| `/api/quality/gaps/:symbol/:interval` | GET | Find data gaps |
| `/api/quality/outliers/:symbol/:interval` | GET | Find price outliers |
| `/api/quality/freshness` | GET | Data freshness check |
| `/api/quality/summary` | GET | Data quality health score |

**WebSocket:**
| Endpoint | Description |
|----------|-------------|
| `/api/ws` | Real-time updates (portfolio, positions, trades, logs) |

---

## 🚀 Getting Started

### Prerequisites

- **Rust** (latest stable)
- **PostgreSQL 14+** with TimescaleDB extension
- **Docker** (optional, for containerized deployment)

### Quick Start

1. **Clone and configure:**
   ```bash
   git clone https://github.com/alphafield/alphafield.git
   cd alphafield
   cp .env.example .env
   # Edit .env with your API keys
   ```

2. **Set up database:**
   ```bash
   docker-compose up -d timescaledb
   # Or configure DATABASE_URL in .env
   ```

3. **Build and run:**
   ```bash
   cargo build --release
   cargo run --bin dashboard_server
   ```

4. **Access dashboard:**
   Open http://127.0.0.1:8080

### Configuration

Create a `.env` file with:

```env
# Database
DATABASE_URL=postgres://user:pass@localhost:5432/alphafield

# API Keys (optional, for data fetching)
BINANCE_API_KEY=your_key
BINANCE_SECRET_KEY=your_secret
COINGECKO_API_KEY=your_key
COINLAYER_API_KEY=your_key

# Logging
RUST_LOG=info
```

---

## 🛠️ Development

### Makefile Commands

| Command | Description |
|---------|-------------|
| `make build` | Build all crates |
| `make test` | Run all tests |
| `make fmt` | Format code |
| `make lint` | Run clippy |
| `make clean` | Remove build artifacts |

### Running Examples

```bash
# Data layer demo
cargo run --bin data-demo --release

# Golden Cross backtest
cargo run --example golden_cross_backtest -p alphafield-backtest

# Dashboard server
cargo run --bin dashboard_server
```

### Docker Deployment

```bash
# Build and run full stack
docker-compose up -d

# Or build image only
docker build -t alphafield .
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [Architecture](doc/architecture.md) | System design and data flow |
| [Detailed Design](doc/detailed_design.md) | Technical specifications |
| [Roadmap](doc/roadmap.md) | Development phases and progress |
| [Project Plan](doc/project_plan.md) | Implementation timeline |
| [API Reference](doc/api.md) | Complete API documentation |

---

## ⚠️ Trading Model

AlphaField is configured as a **spot-only** trading engine:

- **No margin/borrowing** - Cash-constrained model
- **No shorting** - Prevented at backtest and execution layers
- **No liquidation mechanics** - No leverage support

---

## 🔒 Risk Management

Built-in safeguards include:

- **Max Daily Loss** - Auto-halt trading on excessive losses
- **Position Limits** - Prevent oversized orders
- **Drift Monitor** - Alert on execution slippage
- **Volatility Scaling** - Reduce size in high volatility

---

## 📈 Performance

- **Rust-native** for low-latency execution
- **Async I/O** with Tokio runtime
- **Connection pooling** for database and HTTP
- **TimescaleDB compression** for storage efficiency

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🤝 Contributing

Contributions welcome! Please read the contribution guidelines before submitting PRs.
