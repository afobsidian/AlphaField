# 📡 AlphaField API Reference

Complete API documentation for the AlphaField dashboard server.

---

## Base URL

```
http://localhost:8080
```

---

## Health & Status

### GET /api/health

Check server health and database connectivity.

**Response:**
```json
{
  "status": "healthy",
  "database": true,
  "timestamp": "2025-12-08T10:00:00Z"
}
```

---

## Portfolio & Trading (Mock)

### GET /api/portfolio

Get current portfolio state.

**Response:**
```json
{
  "total_value": 105000.00,
  "cash": 50000.00,
  "positions_value": 55000.00,
  "pnl": 5000.00,
  "pnl_percent": 5.0
}
```

### GET /api/positions

Get open positions.

**Response:**
```json
[
  {
    "symbol": "BTC",
    "quantity": 1.5,
    "entry_price": 35000.00,
    "current_price": 36000.00,
    "pnl": 1500.00,
    "pnl_percent": 4.28
  }
]
```

### GET /api/orders

Get order history.

**Response:**
```json
[
  {
    "id": "order_123",
    "symbol": "BTC",
    "side": "Buy",
    "quantity": 1.5,
    "price": 35000.00,
    "status": "Filled",
    "timestamp": "2025-12-08T09:00:00Z"
  }
]
```

### GET /api/performance

Get performance metrics.

**Response:**
```json
{
  "total_return": 0.05,
  "sharpe_ratio": 1.25,
  "max_drawdown": 0.08,
  "volatility": 0.15,
  "win_rate": 0.65
}
```

---

## Backtesting

### POST /api/backtest/run

Run a backtest with specified parameters.

**Request:**
```json
{
  "strategy": "GoldenCross",
  "symbol": "BTC",
  "interval": "1h",
  "days": 30,
  "initial_capital": 10000,
  "params": {
    "short_period": 50,
    "long_period": 200
  }
}
```

**Response:**
```json
{
  "success": true,
  "metrics": {
    "total_return": 0.12,
    "cagr": 0.45,
    "sharpe_ratio": 1.8,
    "max_drawdown": 0.05,
    "volatility": 0.18,
    "win_rate": 0.58,
    "profit_factor": 1.65
  },
  "equity_curve": [10000, 10150, 10320, ...],
  "trades": [...]
}
```

---

## Advanced Analysis

### POST /api/monte-carlo

Run Monte Carlo simulation on trade sequence.

**Request:**
```json
{
  "trades": [
    {"symbol": "BTC", "pnl": 100, "return_pct": 0.01, "duration": 5},
    {"symbol": "BTC", "pnl": -50, "return_pct": -0.005, "duration": 3}
  ],
  "num_simulations": 1000,
  "initial_capital": 10000,
  "seed": 42
}
```

**Response:**
```json
{
  "success": true,
  "result": {
    "num_simulations": 1000,
    "probability_of_profit": 0.72,
    "equity_5th": 9500,
    "equity_50th": 10250,
    "equity_95th": 11200,
    "return_5th": -0.05,
    "return_50th": 0.025,
    "return_95th": 0.12
  }
}
```

### POST /api/correlation

Calculate correlation between strategy equity curves.

**Request:**
```json
{
  "curves": [
    {"label": "Strategy A", "values": [100, 102, 105, 103]},
    {"label": "Strategy B", "values": [100, 101, 104, 106]}
  ],
  "alert_threshold": 0.7
}
```

**Response:**
```json
{
  "success": true,
  "result": {
    "matrix": [[1.0, 0.85], [0.85, 1.0]],
    "labels": ["Strategy A", "Strategy B"],
    "alerts": [{"pair": ["Strategy A", "Strategy B"], "correlation": 0.85}]
  }
}
```

### POST /api/sensitivity

Run parameter sensitivity analysis.

**Request:**
```json
{
  "strategy": "GoldenCross",
  "symbol": "BTC",
  "interval": "1h",
  "days": 90,
  "param": {"name": "short_period", "min": 10, "max": 100, "step": 10}
}
```

### POST /api/walk-forward

Run Walk-Forward Analysis (WFA) to validate strategy robustness.

**Request:**
```json
{
  "strategy": "GoldenCross",
  "symbol": "BTC",
  "interval": "1h",
  "params": {
    "short_period": 50,
    "long_period": 200
  },
  "train_window_days": 365,
  "test_window_days": 90
}
```

**Response:**
```json
{
  "success": true,
  "result": {
    "windows": [
      {
        "train_start": "2024-01-01",
        "test_end": "2024-04-01",
        "metrics": { ... }
      }
    ],
    "overall_metrics": { ... }
  }
}
```

---

## Data Management

### GET /api/data/symbols

List all cached symbols in database.

**Response:**
```json
[
  {
    "symbol": "BTC",
    "timeframe": "1h",
    "bar_count": 720,
    "first_bar": "2025-11-08 00:00",
    "last_bar": "2025-12-08 00:00"
  }
]
```

### GET /api/data/pairs

Get available trading pairs from exchange.

**Response:**
```json
["BTC", "ETH", "SOL", "XRP", "ADA", ...]
```

### POST /api/data/fetch

Fetch new market data from exchange.

**Request:**
```json
{
  "symbol": "BTC",
  "interval": "1h",
  "limit": 1000
}
```

**Response:**
```json
{
  "success": true,
  "message": "Fetched 1000 bars for BTC",
  "bars_fetched": 1000
}
```

### DELETE /api/data/:symbol/:interval

Delete cached data for a symbol/interval.

**Example:** `DELETE /api/data/BTC/1h`

---

## Sentiment Analysis

### GET /api/sentiment/current

Get current Fear & Greed Index data.

**Response:**
```json
{
  "value": 65,
  "classification": "Greed",
  "timestamp": "2025-12-09T00:00:00Z"
}
```

### GET /api/sentiment/history

Get historical sentiment data.

**Request:**
- `days` (optional): Number of days to fetch (start date).
- `start_date` (optional): Start date YYYY-MM-DD.
- `end_date` (optional): End date YYYY-MM-DD.

**Response:**
      "trend": "Stable"
  }
}
```

### POST /api/sentiment/sync

Trigger a synchronization of historical sentiment data from external providers.

**Response:**
```json
{
  "success": true,
  "message": "Synced 365 days of sentiment data",
  "count": 365
}
```

---

## Data Quality

### GET /api/quality/gaps/:symbol/:interval

Check for missing bars (gaps) in data.

**Example:** `GET /api/quality/gaps/BTC/1h`

**Response:**
```json
{
  "success": true,
  "symbol": "BTC",
  "interval": "1h",
  "gaps": [
    {"start": "2025-12-01 10:00:00", "end": "2025-12-01 14:00:00", "expected_bars": 3}
  ],
  "total_missing_bars": 3
}
```

### GET /api/quality/outliers/:symbol/:interval

Check for price outliers (>5% gaps between bars).

**Response:**
```json
{
  "success": true,
  "symbol": "BTC",
  "interval": "1h",
  "outliers": [
    {
      "timestamp": "2025-12-05 08:00:00",
      "previous_close": 40000,
      "current_open": 43000,
      "gap_percent": 7.5
    }
  ],
  "total_outliers": 1
}
```

### GET /api/quality/freshness

Check data freshness for all symbols.

**Response:**
```json
{
  "success": true,
  "symbols": [
    {"symbol": "BTC", "interval": "1h", "hours_since_update": 2.5, "status": "healthy"},
    {"symbol": "ETH", "interval": "1d", "hours_since_update": 25.0, "status": "critical"}
  ],
  "stale_count": 1,
  "healthy_count": 1
}
```

### GET /api/quality/summary

Get overall data quality health score.

**Response:**
```json
{
  "success": true,
  "total_symbols": 10,
  "total_bars": 50000,
  "symbols_with_gaps": 2,
  "symbols_with_outliers": 1,
  "stale_symbols": 1,
  "health_score": 0.85
}
```

---

## WebSocket

### WS /api/ws

Real-time updates for portfolio, positions, trades, and logs.

**Outgoing Messages (Server → Client):**

```json
// Portfolio update
{"type": "Portfolio", "data": {"total_value": 105000, "cash": 50000, ...}}

// Position update
{"type": "Position", "data": {"symbol": "BTC", "quantity": 1.5, ...}}

// Trade executed
{"type": "Trade", "data": {"symbol": "BTC", "side": "Buy", "quantity": 0.5, ...}}

// Log message
{"type": "Log", "data": {"level": "info", "message": "Connected to exchange"}}

// Engine status
{"type": "EngineStatus", "data": {"running": true, "strategy": "GoldenCross", "mode": "paper"}}

// Heartbeat
{"type": "Heartbeat", "data": {"timestamp": "2025-12-08T10:00:00Z"}}
```

**Incoming Commands (Client → Server):**

```json
// Start trading
{"command": "Start", "strategy": "GoldenCross", "mode": "paper"}

// Stop trading
{"command": "Stop"}

// Emergency close all positions
{"command": "PanicClose"}

// Request current state snapshot
{"command": "Snapshot"}

// Ping for connection check
{"command": "Ping"}
```

---

## Error Responses

All endpoints return errors in this format:

```json
{
  "success": false,
  "error": "Error description here"
}
```

Common HTTP status codes:
- `200` - Success
- `400` - Bad request (invalid parameters)
- `404` - Not found
- `500` - Internal server error
