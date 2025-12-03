Technical Design Specification: "Unbiased" Quantitative Trading System
Version: 1.0 Stack: Rust, TimescaleDB, Axum, React Philosophy: Unbiased Trading (Robustness > Overfitting)
1. System Architecture Overview
The system follows a Modular Monolith architecture using a Rust Workspace. This allows shared logic (like data types) to be used across the Backtester, Live Engine, and Data Ingestion services without the complexity of microservices.
1.1 High-Level Data Flow
Ingestion: WebSocket connections (Binance/Bybit) stream Trade and OrderBook events -> Data Crate.
Normalization: Events are converted to a standard Rust struct MarketEvent and stored in TimescaleDB.
Strategy Engine:
Backtest Mode: Replays historical events from DB -> Simulates Latency -> Matches Orders.
Live Mode: Receives real-time events -> Generates Signals -> Risk Checks -> Sends to Exchange.
Dashboard: The Axum API reads the DB state and streams updates to the React frontend via WebSockets.
2. Rust Workspace Structure
We will organize the code into focused crates to ensure separation of concerns.
quant-system/
├── Cargo.toml              # Workspace definition
├── crates/
│   ├── core/               # Shared Types (Candle, Trade, Side, ExchangeEnum)
│   ├── data/               # Ingestion Engine & Database Connectors (sqlx)
│   ├── strategy/           # Strategy Traits & Indicator Logic
│   ├── backtest/           # Event-Driven Simulator (The "Unbiased" Engine)
│   ├── execution/          # Live Trading Logic & Exchange APIs
│   └── dashboard/          # Axum Web Server & API
└── frontend/               # React + TypeScript Dashboard


3. Database Design (TimescaleDB)
We use TimescaleDB (PostgreSQL extension) for its superior handling of time-series data and compatibility with Rust's sqlx.
3.1 Schema Definition
Table: market_candles (Hypertable) Stores standard OHLCV data.
CREATE TABLE market_candles (
    time        TIMESTAMPTZ NOT NULL,
    symbol      TEXT NOT NULL,
    exchange    TEXT NOT NULL,
    open        DOUBLE PRECISION NOT NULL,
    high        DOUBLE PRECISION NOT NULL,
    low         DOUBLE PRECISION NOT NULL,
    close       DOUBLE PRECISION NOT NULL,
    volume      DOUBLE PRECISION NOT NULL,
    -- Critical for Unbiased Backtesting:
    arrived_at  TIMESTAMPTZ NOT NULL, -- When we actually received the data (latency simulation)
    PRIMARY KEY (time, symbol, exchange)
);
SELECT create_hypertable('market_candles', 'time');


Table: strategy_registry (Hypothesis Log) Enforces the "Idea First" philosophy.
CREATE TABLE strategy_registry (
    id          SERIAL PRIMARY KEY,
    name        TEXT NOT NULL,
    hypothesis  TEXT NOT NULL, -- e.g., "Mean reversion occurs after 3-sigma deviations"
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    status      TEXT CHECK (status IN ('Development', 'Incubation', 'Live', 'Retired'))
);


4. Component Details
4.1 The Core Crate (Data Types)
Defines the universal language of the system.
// crates/core/src/lib.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone)]
pub enum Signal {
    Buy(f64),  // Size
    Sell(f64), // Size
    Hold,
}


4.2 The Strategy Crate (Traits)
This is the interface all strategies must implement. It forces a stateless design where possible.
// crates/strategy/src/lib.rs
use core::{Candle, Signal};

pub trait Strategy: Send + Sync {
    // Called on every new candle close
    fn on_candle(&mut self, candle: &Candle) -> Signal;
    
    // Called to update internal state (indicators)
    fn update_indicators(&mut self, candle: &Candle);
    
    // Unique identifier for logging
    fn name(&self) -> &str;
}


4.3 The Backtest Crate (The Engine)
This is the most critical component for "Unbiased" validation. It is Event-Driven, not Vectorized.

**Current Implementation Status (Phase 3 Completed):**
- **Engine**: Implemented `BacktestEngine` that iterates over `Bar` data.
- **Integration**: Uses `StrategyAdapter` to wrap `alphafield_core::Strategy` implementations.
- **Execution**: Simulates execution with `ExchangeSimulator` (supports fees and slippage).
- **Metrics**: Calculates CAGR, Sharpe Ratio, Max Drawdown.

Key Features:
Latency Injection: The engine adds a configurable delay (e.g., 200ms) between Signal generation and Order fill.
Slippage Model: * Limit Orders: Only fill if price moves through the limit price (conservative).
Market Orders: Fill at Worst(Ask, Last) * (1 + VolatilityFactor).

Flow:
// Pseudo-code for Event Loop
while let Some(event) = data_feed.next().await {
    // 1. Update Portfolio State (Mark-to-Market)
    portfolio.update_prices(&event);

    // 2. Feed Strategy via Adapter
    // Adapter converts Signal -> OrderRequest
    if let Some(orders) = strategy.on_bar(&event) {
        // 3. Simulate Execution
        for order in orders {
             exchange.process_order(order);
        }
    }
}


4.4 The Execution Crate (Live Trading)
Handles the "Drift" problem—ensuring live trades match backtest logic.
Risk Manager Module: Intercepts every outgoing order.
MaxDrawdownGuard: Checks if daily PnL < -2%. If true, rejects order.
FatFingerGuard: Rejects orders > Max Position Size.
Exchange Adaptors: Uses reqwest for REST (Orders) and tokio-tungstenite for WebSocket (Data).
5. Web Dashboard (Axum + React)
5.1 Backend (Axum)
Exposes the internal state of the Rust application.
GET /api/status: System health, current PnL.
GET /api/backtest/run: Triggers a new backtest job (runs in background thread).
WS /ws/live: Streams TradeExecuted and LogMessage events to frontend.
5.2 Frontend (React)
Components:
EquityChart: Real-time line chart comparing "Live Equity" vs "Expected (Backtest) Equity".
SignalLog: A table showing every signal generated by the strategy vs. actual fills.
PanicButton: A big red button that calls POST /api/emergency/close_all.
6. Testing & Validation Plan (Unbiased Methodology)
6.1 Walk-Forward Analysis (WFA)
The Backtester must support WFA out of the box.
Process:
Train on Jan-Jun.
Test on Jul.
Train on Feb-Jul.
Test on Aug.
Pass Criteria: The "Test" equity curve must not deviate > 15% from the "Train" curve.
6.2 Monte Carlo Simulation
Before going live, the engine shuffles the sequence of historical trades 1,000 times.
Goal: Determine the mathematical probability of a 50% drawdown.
Rule: If >5% of simulations result in Ruin (bankruptcy), the strategy is discarded.
7. Implementation Steps
Setup: Initialize Rust workspace and Docker container for TimescaleDB.
Ingest: Write the binance-rs collector and let it run for 1 week to gather fresh data (or import CSVs).
Core: Implement Candle struct and Strategy trait.
Backtest: Build the event loop. Verify it matches manual calculation on Excel for 10 trades.
Strategy: Code the "Trend Following" strategy (Unbiased logic).
UI: Build the basic React dashboard to visualize the backtest result