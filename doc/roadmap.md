# 🗺️ AlphaField Roadmap

## ✅ Phase 1: Foundation & Data Layer (Completed)
- [x] **Core Types**: Defined `Bar`, `Trade`, `Order` structs in `crates/core`.
- [x] **Data Clients**: Implemented clients for Binance, CoinGecko, and Coinlayer.
- [x] **Unified Data Interface**: Created `UnifiedDataClient` for seamless access.
- [x] **Smart Routing**: Implemented logic to route requests based on data type and availability.
- [x] **Resilience**: Added API key rotation and automatic fallbacks.

## ✅ Phase 2: Strategy Engine (Completed)
- [x] **Strategy Trait**: Define the standard interface for all strategies (`on_bar`, `on_trade`).
- [x] **Indicator Library**: Implement common technical indicators (SMA, EMA, RSI).
- [x] **Signal Generation**: Create logic to generate `Signal` events from data.
- [x] **Example Strategies**: Implement basic strategies (e.g., Moving Average Crossover) for testing.

## 📅 Phase 3: Backtesting Engine
- [ ] **Event Loop**: Create the core event loop to replay historical data.
- [ ] **Portfolio Manager**: Track positions, cash, and equity over time.
- [ ] **Order Matching**: Simulate order execution (slippage, fees).
- [ ] **Performance Metrics**: Calculate Sharpe ratio, Max Drawdown, CAGR.
- [ ] **Reporting**: Generate performance reports.

## 📅 Phase 4: Live Execution
- [ ] **Exchange Connectivity**: Implement `ExecutionClient` for placing real orders (Binance).
- [ ] **Risk Management**: Pre-trade risk checks (position sizing, max loss).
- [ ] **State Management**: Persist strategy state to disk/DB.
- [ ] **Paper Trading**: Mode to run live strategies with virtual money.

## 📅 Phase 5: Dashboard & Analytics
- [ ] **Web UI**: Real-time dashboard for monitoring strategies.
- [ ] **Data Visualization**: Charts for equity curves and trade history.
- [ ] **Control Panel**: Start/stop strategies, adjust parameters.
