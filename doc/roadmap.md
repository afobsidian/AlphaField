# 🗺️ AlphaField Roadmap

> Updated: December 2025  
> Based on: Original Project Plan & Current Codebase State

---

## ✅ Phase 1: Foundation & Data Layer (Completed)

### Core Types
- [x] Defined `Bar`, `Tick`, `Quote`, `Order`, `Trade` structs in `crates/core`
- [x] Added validation methods and helper functions (typical price, spread, etc.)
- [x] Implemented `Copy` trait for zero-cost stack allocation

### Data Clients
- [x] Binance client (OHLCV with volume)
- [x] CoinGecko client (market data, prices)
- [x] Coinlayer client (daily OHLC fallback)

### Unified Data Interface
- [x] `UnifiedDataClient` as single entry point
- [x] Smart routing based on data type (OHLC → Binance, Prices → CoinGecko)
- [x] API key rotation via `ApiKeyPool`
- [x] Automatic fallbacks on rate limits or errors

### Database Layer
- [x] PostgreSQL integration via `sqlx`
- [x] `DatabaseClient` for OHLCV storage (`crates/data/src/database.rs`)
- [x] Schema: `candles` table with symbol, timeframe, OHLCV
- [x] Batch save/load, upsert on conflict, index on timestamp
- [x] TimescaleDB hypertable conversion for large-scale time-series
- [x] Gap-filler service for missing candles

---

## ✅ Phase 2: Strategy Engine (Completed)

### Strategy Trait
- [x] Core `Strategy` trait (`on_bar`, `on_tick`, `on_quote`)
- [x] Backtest-specific `Strategy` trait returning `Vec<OrderRequest>`
- [x] `StrategyAdapter` for seamless backtest ↔ live integration

### Indicator Library
- [x] SMA (Simple Moving Average)
- [x] EMA (Exponential Moving Average)
- [x] RSI (Relative Strength Index)
- [x] Common `Indicator` trait with `update()`, `value()`, `reset()`

### Example Strategies
- [x] Golden Cross (SMA crossover)
- [x] RSI Strategy (overbought/oversold)
- [x] Buy-and-Hold baseline

---

## ✅ Phase 3: Backtesting Engine (Completed)

### Event-Driven Architecture
- [x] `BacktestEngine` processes data bar-by-bar
- [x] No look-ahead bias (candle-by-candle event loop)

### Portfolio Manager
- [x] `Portfolio` tracks cash, positions, equity history
- [x] `Position` struct with avg price, quantity, PnL

### Order Matching & Simulation
- [x] `ExchangeSimulator` with configurable fee rate
- [x] `SlippageModel` for realistic fills
- [x] Latency injection support

### Spot-Only Enforcement (New)
- [x] `InsufficientPosition` error blocks shorting in backtest
- [x] Cash constraint validation (`InsufficientFunds`)

### Performance Metrics
- [x] Total Return, CAGR
- [x] Sharpe Ratio (annualized)
- [x] Max Drawdown
- [x] Volatility (annualized)

---

## ✅ Phase 4: Live Execution (Completed)

### Exchange Connectivity
- [x] `ExecutionService` trait for order submission
- [x] Binance client implementation (`crates/execution/src/clients/`)

### Risk Management
- [x] `RiskManager<S>` wrapper pattern
- [x] `RiskCheck` trait for composable rules
- [x] `MaxOrderValue` check
- [x] `NoShorts` check (spot-only enforcement)

### Paper Trading
- [x] Simulated execution via `ExchangeSimulator`

---

## 🔄 Phase 5: Dashboard & Analytics (In Progress)

### Backend (Axum)
- [x] Server setup (`crates/dashboard/src/server.rs`)
- [x] API endpoints (`api.rs`, `backtest_api.rs`, `data_api.rs`)
- [x] Mock data for development

### Frontend (React)
- [x] Static HTML/JS/CSS scaffold (`crates/dashboard/static/`)

---

## 📅 Phase 6: Production Hardening (Planned)

### Data Infrastructure
- [ ] TimescaleDB hypertables for candles/trades
- [ ] Survivorship bias handling (delisted coins in dataset)

### Advanced Backtesting
- [ ] Walk-Forward Analysis module
- [ ] Monte Carlo stress testing
- [ ] Parameter sensitivity analysis
- [ ] Multi-asset correlation checks

### Live Trading Safeguards
- [ ] Max daily loss auto-flatten
- [ ] Position drift alerting
- [ ] Multi-strategy correlation monitor
- [ ] Volatility-scaled position sizing (ATR-based)

### Deployment
- [ ] Docker Compose setup (Rust + PostgreSQL/TimescaleDB)
- [ ] CI/CD pipeline (build, test, lint)
- [ ] Minimal capital deployment ($500–$1000)
- [ ] Full-scale rollout with monitoring

---

## 📊 Progress Summary

| Phase | Status | Completion |
|-------|--------|------------|
| 1. Data Layer | ✅ Complete | 100% |
| 2. Strategy Engine | ✅ Complete | 100% |
| 3. Backtesting | ✅ Complete | 80% |
| 4. Live Execution | ✅ Complete | 70% |
| 5. Dashboard | 🔄 In Progress | 40% |
| 6. Production | 📅 Planned | 0% |

---

## 🎯 Phase 7: Enhanced Metrics & Validation (Short-term)

> Target: 2–4 weeks

### Performance Metrics Expansion
- [ ] Sortino Ratio (downside deviation only)
- [ ] SQN (System Quality Number) for strategy robustness
- [ ] Win Rate, Loss Rate
- [ ] Expectancy (average $ per trade)
- [ ] Profit Factor (gross profit / gross loss)
- [ ] Average Win / Average Loss ratio

### Walk-Forward Analysis
- [ ] Define train/test window parameters
- [ ] Rolling optimization: train on window N, test on N+1
- [ ] Aggregate out-of-sample results across all windows
- [ ] Report parameter stability across windows

### Trade-Level Reporting
- [ ] Trade log export (CSV/JSON)
- [ ] Per-trade metrics (duration, MAE, MFE)
- [ ] Entry/exit timing analysis

---

## 🎯 Phase 8: Real-Time Dashboard (Medium-term)

> Target: 1–2 months

### WebSocket Streaming
- [ ] Axum WebSocket handler for live updates
- [ ] Broadcast equity curve, positions, recent trades
- [ ] Heartbeat/reconnect logic on frontend

### Live Data Endpoints
- [ ] `/api/positions` – current holdings
- [ ] `/api/balance` – cash + equity
- [ ] `/api/orders` – open/recent orders
- [ ] `/api/trades` – execution history

### Frontend Enhancements
- [ ] Real-time equity curve (Recharts or Lightweight Charts)
- [ ] Position table with live PnL
- [ ] Trade history with entry/exit markers on chart
- [ ] Control panel: Start / Stop / Panic Close buttons
- [ ] Log console with streaming `tracing` events

### Paper Trading Integration
- [ ] Binance Testnet API connectivity
- [ ] Paper vs Live mode toggle in dashboard
- [ ] Side-by-side backtest vs paper comparison

---

## 🎯 Phase 9: Stress Testing & Robustness (Medium-term)

> Target: 2–3 months

### Monte Carlo Simulation
- [ ] Shuffle trade order to test path dependency
- [ ] Generate N equity curves from same trades
- [ ] Report confidence intervals (5th/50th/95th percentile)
- [ ] Visualize Monte Carlo fan chart

### Parameter Sensitivity
- [ ] Grid search over key parameters
- [ ] Heatmap of Sharpe/Drawdown vs parameters
- [ ] Identify robust parameter regions

### Correlation & Diversification
- [ ] Multi-asset backtest support
- [ ] Cross-strategy correlation matrix
- [ ] Alert when strategies are too correlated

---

## 🎯 Phase 10: Data Infrastructure Scale-Up (Long-term)

> Target: 3–4 months

### TimescaleDB Migration
- [ ] Convert `candles` table to hypertable
- [ ] Add `trades` hypertable for tick data
- [ ] Enable compression policies for old data
- [ ] Benchmark query performance vs plain PostgreSQL

### Survivorship Bias Handling
- [ ] Track delisted/dead coins in dataset
- [ ] Include LUNA, FTT, etc. in historical tests
- [ ] Flag assets by status (active, delisted, migrated)

### Data Quality
- [ ] Data integrity checks (missing bars, outliers)
- [ ] Alerting on ingestion failures

---

## 🎯 Phase 11: Production Deployment (Long-term)

> Target: 4–6 months

### Live Trading Safeguards
- [ ] Max Daily Loss circuit breaker (auto-flatten)
- [ ] Fat-finger protection (reject orders > X% of account)
- [ ] Drift monitor (alert if fill deviates > 0.5% from expected)
- [ ] Volatility-scaled sizing (reduce size when ATR spikes)

### Infrastructure
- [ ] Docker Compose (Rust services + TimescaleDB)
- [ ] CI/CD pipeline (GitHub Actions: build, test, clippy, fmt)
- [ ] Secrets management (API keys via Vault or env injection)
- [ ] Logging & monitoring (Prometheus + Grafana or similar)

### Staged Rollout
- [ ] **Alpha**: $500–$1,000 capital, single strategy
- [ ] **Beta**: $5,000 capital, 2–3 strategies, correlation checks
- [ ] **Full Scale**: Target allocation with automated rebalancing

### Operational Runbooks
- [ ] Incident response (exchange outage, API errors)
- [ ] Daily reconciliation (DB vs exchange balances)
- [ ] Strategy performance review cadence
