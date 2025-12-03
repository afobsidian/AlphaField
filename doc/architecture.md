# 🏗️ AlphaField Architecture

## System Overview

AlphaField is designed as a modular, event-driven system. The core components are separated into distinct crates to ensure separation of concerns and testability.

```mermaid
graph TD
    Data[crates/data] --> Core[crates/core]
    Strategy[crates/strategy] --> Core
    Backtest[crates/backtest] --> Core
    Backtest --> Data
    Backtest --> Strategy
    Execution[crates/execution] --> Core
    Execution --> Strategy
```

## 📦 Components

### 1. Core (`crates/core`)
Contains the fundamental data structures and traits shared across the system.
- **Types**: `Bar`, `Trade`, `Order`, `Position`, `Signal`.
- **Traits**: `Strategy`, `DataSource`, `ExecutionService`.

### 2. Data Layer (`crates/data`)
Responsible for fetching and normalizing market data.
- **UnifiedDataClient**: The main entry point. Abstracts away specific APIs.
- **Smart Routing**:
    - **OHLC**: Binance (Primary) -> CoinGecko -> Coinlayer.
    - **Market Data**: CoinGecko (Primary) -> Binance.
- **Resilience**: `ApiKeyPool` handles rotation and rate limiting.

### 3. Strategy (`crates/strategy`)
Contains trading logic.
- **Indicators**: Technical analysis tools.
- **Signals**: Logic to convert market data into trade signals.

### 4. Backtest (`crates/backtest`)
Simulates strategy performance.
- **Engine**: Replays historical data.
- **Portfolio**: Tracks virtual account state.
- **Matcher**: Simulates order fills.

## 🔄 Data Flow (Unified Data Layer)

The Data Layer uses a smart routing approach to ensure high availability and data quality.

```mermaid
sequenceDiagram
    participant User
    participant UnifiedClient
    participant Binance
    participant CoinGecko
    participant Coinlayer

    User->>UnifiedClient: get_bars("BTC", "1h")
    
    rect rgb(240, 255, 240)
        Note over UnifiedClient: Priority 1: Binance
        UnifiedClient->>Binance: get_klines()
        alt Success
            Binance-->>UnifiedClient: OHLC Data
        else Rate Limit / Error
            Note over UnifiedClient: Fallback to Priority 2
            UnifiedClient->>CoinGecko: get_ohlc()
            alt Success
                CoinGecko-->>UnifiedClient: OHLC Data
            else Error
                Note over UnifiedClient: Fallback to Priority 3
                UnifiedClient->>Coinlayer: get_historical()
                Coinlayer-->>UnifiedClient: Daily Data
            end
        end
    end
    
    UnifiedClient-->>User: Vec<Bar>
```
