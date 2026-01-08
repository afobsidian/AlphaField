
# AlphaField – AI Coding Agent Instructions

## 🚦 Project Overview

AlphaField is a modular, event-driven algorithmic trading engine for crypto markets, written in Rust. It emphasizes **unbiased backtesting** and robust risk management. The workspace is organized as a Cargo monorepo with six main crates:

- **core**: Fundamental types/traits (`Bar`, `Trade`, `Order`, `Signal`, `Strategy`, `QuantError`)
- **data**: Multi-source ingestion (Binance, CoinGecko, Coinlayer), TimescaleDB storage, data quality monitoring
- **strategy**: Technical indicators (`SMA`, `EMA`, `RSI`, etc.) and strategies (`GoldenCross`, `MeanReversion`, etc.)
- **backtest**: Event-driven simulation, walk-forward, Monte Carlo, sensitivity, correlation analysis, **ML-based trading models**
- **execution**: Risk management (circuit breakers, position limits, drift, volatility scaling), **advanced order types (OCO, bracket, iceberg, limit-chase)**
- **dashboard**: Axum REST/WebSocket API, web UI (served at http://localhost:8080)

## 🏗️ Architecture & Data Flow

- **Data** flows: External APIs → `UnifiedDataClient` → TimescaleDB → Backtest/Strategy/Execution
- **Backtest**: Loads bars → feeds to strategy → generates signals/orders → simulates fills (slippage, latency, fees)
- **Execution**: All orders pass through `RiskManager` and composable `RiskCheck` traits before simulated or live execution
- **Dashboard**: REST endpoints and `/api/ws` WebSocket for real-time monitoring, analysis, and control

See [doc/architecture.md](../../doc/architecture.md) for diagrams and [doc/detailed_design.md](../../doc/detailed_design.md) for design rationale.

## 🧩 Key Patterns & Conventions

- **Strategy Traits**: Two versions—core (`Option<Signal>`) and backtest (`Result<Vec<OrderRequest>>`). Choose based on context ([crates/core/src/lib.rs], [crates/backtest/src/strategy.rs]).
- **Indicators**: Implement `Indicator` trait (`update`, `value`, `reset`). Use `VecDeque` for windowed calcs ([crates/strategy/src/indicators.rs]).
- **Data Validation**: All core types have `validate()`. Always call after constructing from external data.
- **Smart Routing**: `UnifiedDataClient` auto-routes OHLC/price requests and rotates API keys ([crates/data/src/lib.rs]).
- **Risk Management**: All orders go through `RiskManager` and checks like `MaxOrderValue`, `NoShorts`, `MaxDailyLoss`, `PositionDrift`, `VolatilityScaledSize` ([crates/execution/src/lib.rs]).
- **Advanced Orders**: OCO, bracket, iceberg, and limit-chase orders managed by `OrderManager` with comprehensive lifecycle management ([crates/execution/src/orders.rs]).
- **Machine Learning**: Feature engineering, model training, and validation with time-series aware data splitting and overfitting detection ([crates/backtest/src/ml/]).
- **Testing**: Unit tests in `#[cfg(test)]` modules, focus on edge cases and known-value checks.
- **Error Handling**: Use `QuantError` for all error types; backtest has `BacktestError` for simulation-specific errors.
- **Types**: All monetary values are `f64`; timestamps use `chrono::DateTime<Utc>`.
- **Async**: HTTP via `reqwest` (30s timeout, connection pooling); async traits via `#[async_trait::async_trait]`.
- **Serialization**: Use `serde` with `#[derive(Serialize, Deserialize)]`; errors via `thiserror::Error`.

## 🛠️ Developer Workflow

- **Build**: `make build` or `cargo build`
- **Test**: `make test` or `cargo test`
- **Lint/Format**: `make lint` / `make fmt` (Clippy, rustfmt)
- **Run Data Demo**: `make run-demo` or `cargo run --bin data-demo --release`
- **Run Backtest Example**: `make run-backtest` or `cargo run --example golden_cross_backtest -p alphafield-backtest --release`
- **Run Dashboard**: `make run-dashboard` or `cargo run --bin dashboard_server --release`
- **Docker**: `make docker-up`/`docker-down`/`docker-build` for full stack

## ⚙️ Environment Setup

Create `.env` in project root (see `.env.example`):

```env
DATABASE_URL=postgres://user:pass@localhost:5432/alphafield
BINANCE_API_KEYS=key1,key2
COINGECKO_API_KEYS=key1
COINLAYER_API_KEYS=key1
RUST_LOG=info
```

## 📚 References

- [README.md](../../README.md): Features, architecture, and quickstart
- [doc/architecture.md](../../doc/architecture.md): System diagrams and flows
- [doc/detailed_design.md](../../doc/detailed_design.md): Design rationale and patterns
- [doc/api.md](../../doc/api.md): API endpoints and usage
- [doc/roadmap.md](../../doc/roadmap.md): Feature roadmap
- [doc/ml.md](../../doc/ml.md): Machine learning features and usage
- [doc/orders.md](../../doc/orders.md): Advanced order types and management
- [Makefile](../../Makefile): All supported dev commands

## 🛑 Project-Specific Rules

- **No shorting or leverage**: Spot-only, enforced at backtest and execution layers
- **Survivorship bias prevention**: Database includes delisted assets
- **Walk-forward and Monte Carlo**: Required for strategy validation
- **All strategies must start with a written hypothesis** (see [doc/project_plan.md])

---
For any unclear or missing conventions, consult the referenced docs or ask for clarification.
