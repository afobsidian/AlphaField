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
| **Interactive Charting** | Candlestick/line/area charts with technical indicators (SMA, EMA, RSI, MACD, BB) |
| **Optimization-First Workflow** | Automated parameter optimization before backtesting with multi-symbol validation |
| **Asset Category Training** | Train strategies across predefined symbol baskets (Market, Large/Mid/Small Cap, DeFi) |
| **TimescaleDB Storage** | Time-series optimized with hypertables and compression |
| **Event-Driven Backtesting** | Simulate strategies with slippage, fees, and latency modeling |
| **Sentiment Analysis** | Fear & Greed Index integration + Asset-specific sentiment metrics |
| **Comprehensive Optimization** | Parameter sweep, walk-forward validation, sensitivity analysis, and robustness scoring |
| **Data Quality Monitoring** | Gap detection, outlier detection, ingestion alerting |
| **Risk Management** | Circuit breakers, position limits, drift monitoring |
| **Machine Learning** | **NEW**: ML-based trading models with feature engineering, model training, and validation |
| **Advanced Order Types** | **NEW**: OCO, bracket, iceberg, and limit-chase orders with comprehensive order management |

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
| `Kama` | Kaufman's Adaptive Moving Average |
| `Stochastic` | Stochastic Oscillator (%K/%D) |

**Strategies:**
| Strategy | Logic |
|----------|-------|
| `GoldenCross` | SMA crossover (50/200) |
| `RsiStrategy` | RSI oversold/overbought reversal |
| `MeanReversion` | Bollinger Band reversion (legacy name) |
| `Momentum` | MACD crossover |
| `TrendFollowing` | EMA trend with ADX filter |

**Mean Reversion Strategies (Phase 12.3):**
| Strategy | Logic |
|----------|-------|
| `BollingerBandsStrategy` | BB band reversion with RSI confirmation |
| `RSIReversionStrategy` | Pure RSI mean reversion with trend filter |
| `StochReversionStrategy` | Stochastic oscillator reversion with crossovers |
| `ZScoreReversionStrategy` | Statistical z-score reversion (±2σ) |
| `PriceChannelStrategy` | Donchian channel breakout reversion |
| `KeltnerReversionStrategy` | Keltner channel (EMA+ATR) with volume confirmation |
| `StatArbStrategy` | Statistical arbitrage (spot-only adaptation) |

---

### `crates/backtest`
Event-driven backtesting engine with comprehensive optimization workflow.

| Component | Description |
|-----------|-------------|
| `BacktestEngine` | Main engine processing bars sequentially |
| `Portfolio` | Virtual account state (cash, positions, equity) |
| `ExchangeSimulator` | Order matching with slippage/fees |
| `StrategyAdapter` | Bridges Strategy trait to backtest engine |
| `PerformanceMetrics` | CAGR, Sharpe, Sortino, Max Drawdown, etc. |
| `OptimizationWorkflow` | 6-phase pipeline: grid search, dispersion stats, walk-forward, sensitivity, robustness scoring |

**Advanced Analysis:**
| Module | Description |
|--------|-------------|
| `WalkForwardAnalyzer` | Rolling train/test validation across time windows |
| `MonteCarloSimulator` | Trade sequence shuffling for robustness |
| `SensitivityAnalyzer` | Parameter grid search with 3D heatmaps |
| `CorrelationAnalyzer` | Multi-strategy correlation matrix |
| `ParameterDispersion` | Statistical analysis (CV, ranges, positive %) to detect overfitting |

**Machine Learning (NEW):**
| Module | Description |
|--------|-------------|
| `FeatureExtractor` | Feature engineering from OHLCV data (returns, volatility, momentum, volume) |
| `DataSplitter` | Time-series aware train/test splits with walk-forward support |
| `MLModels` | Linear regression, decision trees, random forests, ensemble models |
| `MLStrategy` | ML-based trading strategies with model persistence |
| `MLValidation` | Walk-forward validation and overfitting detection |

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

**Advanced Order Types (NEW):**
| Order Type | Description |
|------------|-------------|
| `OCO` | One-Cancels-Other orders (primary + secondary order) |
| `Bracket` | Entry order with stop-loss and take-profit |
| `Iceberg` | Large orders split into smaller visible portions |
| `LimitChase` | Automatically adjust limit orders to follow price |
| `OrderManager` | Comprehensive order queue and lifecycle management |

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
| `/api/backtest/workflow` | POST | **NEW**: Comprehensive optimization workflow (grid search + validation) |
| `/api/monte-carlo` | POST | Run Monte Carlo simulation |
| `/api/correlation` | POST | Calculate strategy correlation |
| `/api/sensitivity` | POST | Parameter sensitivity analysis |
| `/api/chart/ohlcv` | POST | **NEW**: Get OHLCV data with technical indicators for charting |
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

**Machine Learning Endpoints (NEW):**
| `/api/ml/train` | POST | Train ML model with specified features and parameters |
| `/api/ml/predict` | POST | Generate predictions using trained ML model |
| `/api/ml/validate` | POST | Validate ML model with walk-forward analysis |
| `/api/ml/models` | GET | List available ML models |
| `/api/ml/models/:id` | GET | Get specific ML model details |

**Advanced Orders Endpoints (NEW):**
| `/api/orders/pending` | GET | Get pending orders (filter by symbol) |
| `/api/orders/queue` | GET | Get order queue summary |
| `/api/orders/oco` | POST | Create OCO (One-Cancels-Other) order |
| `/api/orders/bracket` | POST | Create bracket order (entry + SL + TP) |
| `/api/orders/iceberg` | POST | Create iceberg order (split large order) |
| `/api/orders/limit-chase` | POST | Create limit-chase order |
| `/api/orders/:id` | PUT | Modify existing order |
| `/api/orders/:id` | DELETE | Cancel order |
| `/api/orders/partial-tp` | POST | Partial take-profit on position |
| `/api/orders/scale` | POST | Scale in/out of position |

**WebSocket:**
| Endpoint | Description |
|----------|-------------|
| `/api/ws` | Real-time updates (portfolio, positions, trades, logs) |

### 🖥️ Dashboard UI

The `crates/dashboard` crate serves a Vanilla JS frontend at `http://localhost:8080`.

**Optimization-First Workflow (Restructured):**
1. **Build Tab**: Select trading strategy + Asset category (Market/Large Cap/Mid Cap/Small Cap/DeFi)
2. **Optimize Tab**: One-click Auto-Optimize across all symbols in category with comprehensive results
   - Parameter sweep scatter plot
   - 3D sensitivity heatmap
   - Walk-forward validation charts
   - Robustness score (0-100) with overfitting detection
   - Parameters automatically applied for backtesting
3. **Backtest Tab**: Select specific symbol from category → Run backtest with optimized parameters
   - Equity curve with trade markers
   - **Interactive price chart** with candlestick/line/area display
   - Technical indicators (SMA, EMA, RSI, MACD, Bollinger Bands)
   - Trade entry/exit markers on price chart
4. **Deploy Tab**: Deploy validated strategy

**Additional Views:**
- **Data Manager**: Interface to fetch, view, and inspect market data (gaps, outliers).
- **Analysis**: Advanced tools for Monte Carlo simulation, Correlation matrix, and custom Walk Forward Analysis.
- **Sentiment**: Fear & Greed index history and asset-specific sentiment metrics.
- **Orders Tab (NEW)**: Advanced order management interface for OCO, bracket, iceberg, and limit-chase orders
- **ML Tab (NEW)**: Machine learning model training, validation, and deployment interface

---

## 🧪 Strategy Validation

The `validate_strategy` binary provides comprehensive strategy validation through backtesting, walk-forward analysis, Monte Carlo simulation, and regime-based performance analysis.

### Batch Validation

Validate multiple strategies against multiple symbols in parallel:

```bash
cargo run --bin validate_strategy -- batch \
  --batch-file validation/strategies_batch.txt \
  --symbols "BTC,ETH,SOL,BNB,XRP" \
  --interval 1h \
  --output-dir validation/reports \
  --format json \
  --max-concurrent 4
```

**⚠️ Important**: Use `--max-concurrent` to limit parallel validation and prevent CPU overload. Each validation runs multiple CPU-intensive operations:
- Full backtest over historical data
- Walk-forward analysis (5-10 windows × 2 backtests each)
- Monte Carlo simulation (thousands of trade sequences)
- Regime analysis

Recommended values:
- **Conservative**: `--max-concurrent 2` (lowest CPU usage, slower execution)
- **Balanced**: `--max-concurrent 4` or `num_cores/2` (good balance of speed and CPU usage)
- **Aggressive**: `--max-concurrent num_cores` (default, fastest but highest CPU usage)

**Batch File Format**: One strategy name per line
```
RSIReversion
GoldenCross
AdaptiveMA
MacdTrend
```

### List Available Strategies

```bash
# View all strategies
cargo run --bin validate_strategy -- list-strategies

# Filter by category
cargo run --bin validate_strategy -- list-strategies --category mean_reversion
```

### Validate Single Strategy

```bash
cargo run --bin validate_strategy -- validate \
  --strategy RSIReversion \
  --symbol BTC \
  --interval 1h \
  --output validation_report.json \
  --format json
```

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
| `make build` | Build all crates (debug) |
| `make test` | Run all tests |
| `make fmt` | Format code |
| `make lint` | Run Clippy (matches CI) |
| `make ci` | Run local CI (fmt + lint + test) |
| `make clean` | Remove build artifacts |
| `make run-demo` | Run data demo |
| `make run-backtest` | Run Golden Cross backtest example |
| `make run-dashboard` | Run dashboard server |
| `make docker-build` | Build Docker image |
| `make docker-up` | Start local Docker env |
| `make docker-down` | Stop local Docker env |
| `make docker-reset` | Reset local Docker env |

### Running Examples

```bash
# Data layer demo
cargo run --bin data-demo --release

# Golden Cross backtest
cargo run --example golden_cross_backtest -p alphafield-backtest --release

# Dashboard server
cargo run --bin dashboard_server --release
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
| [Optimization Workflow](doc/optimization_workflow.md) | **NEW**: Comprehensive parameter optimization guide |
| [Detailed Design](doc/detailed_design.md) | Technical specifications |
| [Roadmap](doc/roadmap.md) | Development phases and progress |
| [Project Plan](doc/project_plan.md) | Implementation timeline |
| [API Reference](doc/api.md) | Complete API documentation |
| [ML Documentation](doc/ml.md) | **NEW**: Machine learning features and usage |
| [Advanced Orders](doc/orders.md) | **NEW**: Advanced order types and management |

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
