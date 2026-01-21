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

### POST /api/backtest/optimize

Run parameter optimization using grid search.

**Request:**
```json
{
  "strategy": "GoldenCross",
  "symbol": "BTCUSDT",
  "interval": "1h",
  "days": 180
}
```

**Response:**
```json
{
  "success": true,
  "optimized_params": {
    "fast_period": 10.0,
    "slow_period": 50.0
  },
  "best_score": 0.68,
  "best_sharpe": 1.85,
  "best_return": 0.42,
  "iterations": 144,
  "elapsed_ms": 25430,
  "sweep_results": [...]
}
```

### POST /api/backtest/workflow

**NEW**: Run comprehensive optimization workflow with validation.

Combines grid search optimization, parameter dispersion analysis, walk-forward validation, and 3D sensitivity analysis to find robust parameters and avoid overfitting.

**Request:**
```json
{
  "strategy": "GoldenCross",
  "symbol": "BTCUSDT",
  "interval": "1h",
  "days": 730,
  "include_3d_sensitivity": true,
  "train_window_days": 252,
  "test_window_days": 63,
  "risk_free_rate": 0.02
}
```

**Response:**
```json
{
  "success": true,
  "optimized_params": {
    "fast_period": 10.0,
    "slow_period": 50.0
  },
  "robustness_score": 73.5,
  "parameter_dispersion": {
    "sharpe_cv": 0.35,
    "positive_sharpe_pct": 68.5
  },
  "walk_forward_stability_score": 0.72,
  "monte_carlo": {
    "num_simulations": 1000,
    "probability_of_profit": 0.72,
    "equity_5th": 95000.0,
    "equity_50th": 102500.0,
    "equity_95th": 112000.0,
    "return_5th": -0.05,
    "return_50th": 0.025,
    "return_95th": 0.12,
    "drawdown_95th": 0.15
  },
  "sensitivity_heatmap": {...},
  "elapsed_ms": 45230
}
```

See [Optimization Workflow Documentation](optimization_workflow.md) for complete details.

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

## Machine Learning (NEW)

### POST /api/ml/train

Train a machine learning model with specified features and parameters.

**Request:**
```json
{
  "model_type": "RandomForest",
  "features": ["returns_5", "volatility_20", "rsi_14", "macd_histogram"],
  "symbol": "BTC",
  "interval": "1h",
  "days": 365,
  "target": "next_return_5",
  "train_days": 252,
  "test_days": 63,
  "model_name": "btc_5min_predictor"
}
```

**Response:**
```json
{
  "success": true,
  "model_id": "ml_12345",
  "model_type": "RandomForest",
  "features_used": ["returns_5", "volatility_20", "rsi_14"],
  "training_metrics": {
    "r2_score": 0.72,
    "mse": 0.0012,
    "mae": 0.025
  },
  "test_metrics": {
    "r2_score": 0.68,
    "mse": 0.0015,
    "mae": 0.028
  },
  "training_time_ms": 4520,
  "message": "Model trained successfully"
}
```

### POST /api/ml/predict

Generate predictions using a trained ML model.

**Request:**
```json
{
  "model_id": "ml_12345",
  "symbol": "BTC",
  "interval": "1h",
  "days": 30
}
```

**Response:**
```json
{
  "success": true,
  "predictions": [
    {
      "timestamp": "2026-01-01T00:00:00Z",
      "predicted_return": 0.012,
      "confidence": 0.85,
      "features": {"returns_5": 0.02, "volatility_20": 0.03, "rsi_14": 65}
    }
  ],
  "model_info": {
    "model_type": "RandomForest",
    "features": ["returns_5", "volatility_20", "rsi_14"],
    "training_date": "2026-01-01T10:00:00Z"
  }
}
```

### POST /api/ml/validate

Validate ML model with walk-forward analysis to detect overfitting.

**Request:**
```json
{
  "model_id": "ml_12345",
  "symbol": "BTC",
  "interval": "1h",
  "train_window_days": 252,
  "test_window_days": 63,
  "num_windows": 5
}
```

**Response:**
```json
{
  "success": true,
  "validation_results": {
    "windows": [
      {
        "train_start": "2024-01-01",
        "test_end": "2024-04-01",
        "train_r2": 0.75,
        "test_r2": 0.72,
        "overfit_score": 0.04
      }
    ],
    "overall_metrics": {
      "avg_train_r2": 0.74,
      "avg_test_r2": 0.71,
      "overall_overfit_score": 0.05,
      "stability_score": 0.92
    },
    "overfitting_detected": false,
    "recommendation": "Model is robust and suitable for deployment"
  }
}
```

### GET /api/ml/models

List all available ML models.

**Response:**
```json
{
  "success": true,
  "models": [
    {
      "model_id": "ml_12345",
      "model_type": "RandomForest",
      "symbol": "BTC",
      "interval": "1h",
      "features": ["returns_5", "volatility_20", "rsi_14"],
      "target": "next_return_5",
      "training_date": "2026-01-01T10:00:00Z",
      "performance": {
        "train_r2": 0.72,
        "test_r2": 0.68
      }
    }
  ]
}
```

### GET /api/ml/models/:id

Get detailed information about a specific ML model.

**Response:**
```json
{
  "success": true,
  "model": {
    "model_id": "ml_12345",
    "model_type": "RandomForest",
    "symbol": "BTC",
    "interval": "1h",
    "features": ["returns_5", "volatility_20", "rsi_14"],
    "target": "next_return_5",
    "training_config": {
      "train_days": 252,
      "test_days": 63,
      "random_state": 42
    },
    "training_date": "2026-01-01T10:00:00Z",
    "performance": {
      "train_r2": 0.72,
      "test_r2": 0.68,
      "mse": 0.0015,
      "mae": 0.028
    },
    "feature_importance": [
      {"feature": "rsi_14", "importance": 0.45},
      {"feature": "volatility_20", "importance": 0.35},
      {"feature": "returns_5", "importance": 0.20}
    ]
  }
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

## Advanced Orders (NEW)

### GET /api/orders/pending

Get pending orders, optionally filtered by symbol.

**Query Parameters:**
- `symbol` (optional): Filter by symbol

**Response:**
```json
{
  "success": true,
  "orders": [
    {
      "order_id": "order_123",
      "symbol": "BTC",
      "side": "Buy",
      "price": 40000.00,
      "quantity": 0.5,
      "order_type": "Limit",
      "status": "Pending",
      "created_at": "2026-01-01T10:00:00Z"
    }
  ]
}
```

### GET /api/orders/queue

Get order queue summary.

**Query Parameters:**
- `symbol` (optional): Filter by symbol

**Response:**
```json
{
  "success": true,
  "queues": [
    {
      "symbol": "BTC",
      "pending_count": 5,
      "total_qty": 2.5,
      "orders": [
        {
          "order_id": "order_123",
          "side": "Buy",
          "price": 40000.00,
          "quantity": 0.5,
          "position": 1
        }
      ]
    }
  ]
}
```

### POST /api/orders/oco

Create OCO (One-Cancels-Other) order.

**Request:**
```json
{
  "symbol": "BTC",
  "primary": {
    "side": "Buy",
    "price": 40000.00,
    "quantity": 0.5,
    "order_type": "Limit"
  },
  "secondary": {
    "side": "Sell",
    "price": 39000.00,
    "quantity": 0.5,
    "order_type": "Stop"
  }
}
```

**Response:**
```json
{
  "success": true,
  "oco_id": "oco_12345",
  "primary_order_id": "order_123",
  "secondary_order_id": "order_124",
  "status": "Active",
  "message": "OCO order created successfully"
}
```

### POST /api/orders/bracket

Create bracket order (entry + stop-loss + take-profit).

**Request:**
```json
{
  "symbol": "BTC",
  "entry": {
    "side": "Buy",
    "price": 40000.00,
    "quantity": 0.5,
    "order_type": "Limit"
  },
  "stop_loss": {
    "price": 39000.00,
    "order_type": "Stop"
  },
  "take_profit": {
    "price": 41000.00,
    "order_type": "Limit"
  }
}
```

**Response:**
```json
{
  "success": true,
  "bracket_id": "bracket_12345",
  "entry_order_id": "order_123",
  "stop_loss_order_id": "order_124",
  "take_profit_order_id": "order_125",
  "status": "Pending",
  "message": "Bracket order created successfully"
}
```

### POST /api/orders/iceberg

Create iceberg order (split large order into smaller visible portions).

**Request:**
```json
{
  "symbol": "BTC",
  "side": "Buy",
  "total_quantity": 5.0,
  "visible_quantity": 0.5,
  "price": 40000.00,
  "order_type": "Limit",
  "interval_ms": 1000
}
```

**Response:**
```json
{
  "success": true,
  "iceberg_id": "iceberg_12345",
  "total_quantity": 5.0,
  "visible_quantity": 0.5,
  "remaining_quantity": 5.0,
  "status": "Active",
  "message": "Iceberg order created successfully"
}
```

### POST /api/orders/limit-chase

Create limit-chase order (automatically adjust limit order to follow price).

**Request:**
```json
{
  "symbol": "BTC",
  "side": "Buy",
  "initial_price": 40000.00,
  "quantity": 0.5,
  "chase_distance": 50.0,
  "max_price": 40500.0,
  "expiry_seconds": 3600
}
```

**Response:**
```json
{
  "success": true,
  "limit_chase_id": "chase_12345",
  "current_price": 40000.00,
  "quantity": 0.5,
  "status": "Active",
  "message": "Limit-chase order created successfully"
}
```

### PUT /api/orders/:id

Modify an existing order.

**Request:**
```json
{
  "new_price": 40500.00,
  "new_quantity": 0.75
}
```

**Response:**
```json
{
  "success": true,
  "order_id": "order_123",
  "status": "Modified",
  "message": "Order modified successfully"
}
```

### DELETE /api/orders/:id

Cancel an order.

**Response:**
```json
{
  "success": true,
  "order_id": "order_123",
  "status": "Cancelled",
  "message": "Order cancelled successfully"
}
```

### POST /api/orders/partial-tp

Partial take-profit on a position.

**Request:**
```json
{
  "symbol": "BTC",
  "position_id": "pos_123",
  "percentage": 0.5,
  "take_profit_price": 41000.00
}
```

**Response:**
```json
{
  "success": true,
  "position_id": "pos_123",
  "tp_order_id": "order_123",
  "tp_quantity": 0.25,
  "remaining_quantity": 0.25,
  "message": "Partial take-profit order created"
}
```

### POST /api/orders/scale

Scale in/out of a position.

**Request:**
```json
{
  "symbol": "BTC",
  "action": "scale_in",
  "quantity": 0.25,
  "price": 39500.00,
  "target_position_size": 1.0
}
```

**Response:**
```json
{
  "success": true,
  "position_id": "pos_123",
  "scale_order_id": "order_123",
  "new_position_size": 0.75,
  "message": "Scale-in order created successfully"
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
