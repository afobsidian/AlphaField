# 🚀 AlphaField Quickstart Guide

Get up and running with AlphaField in 5 minutes.

---

## Prerequisites

- **Rust** (stable, 1.70+)
- **PostgreSQL 14+** with TimescaleDB
- **Docker** (recommended for easy setup)

---

## Option 1: Docker Compose (Recommended)

The easiest way to run AlphaField with all dependencies:

```bash
# Clone the repository
git clone https://github.com/alphafield/alphafield.git
cd alphafield

# Copy and configure environment
cp .env.example .env
# Edit .env with your API keys (optional)

# Start the full stack
docker-compose up -d

# Access the dashboard
open http://localhost:8080
```

This starts:
- **AlphaField API** on port 8080
- **TimescaleDB** on port 5432

---

## Option 2: Local Development

### 1. Install TimescaleDB

**macOS:**
```bash
brew install timescaledb
brew services start timescaledb
```

**Ubuntu/Debian:**
```bash
sudo apt install timescaledb-2-postgresql-14
sudo systemctl start postgresql
```

### 2. Configure Database

```bash
# Create database
createdb alphafield

# Enable TimescaleDB extension
psql -d alphafield -c "CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;"
```

### 3. Configure Environment

```bash
cp .env.example .env
```

Edit `.env`:
```env
DATABASE_URL=postgres://localhost/alphafield

# Optional: API keys for data fetching
BINANCE_API_KEY=your_key
COINGECKO_API_KEY=your_key
```

### 4. Build and Run

```bash
# Build release
cargo build --release

# Run dashboard server
cargo run --bin dashboard_server
```

---

## First Steps

### 1. Access Dashboard

Open http://localhost:8080 in your browser.

### 2. Fetch Market Data

Go to **Data** tab and fetch BTC data:
- Select "BTC" from dropdown
- Choose "1h" interval
- Set days to "30"
- Click "Fetch Data"

### 3. Run a Backtest

Go to **Backtest** tab:
- Select "Golden Cross" strategy
- Choose "BTC" symbol
- Set capital to 10000
- Click "Run Backtest"

### 4. View Results

- **Equity Curve**: See portfolio value over time
- **Metrics**: Total return, Sharpe ratio, max drawdown
- **Trades**: Entry/exit points

### 5. Explore Advanced Features (Optional)

**Machine Learning:**
- Go to **ML** tab
- Select a model type (Linear Regression, Random Forest, etc.)
- Configure features and parameters
- Train model on historical data
- Validate with walk-forward analysis

**Advanced Orders:**
- Go to **Orders** tab
- Create OCO orders (One-Cancels-Other)
- Set up bracket orders with stop-loss and take-profit
- Use iceberg orders for large position management
- Configure limit-chase orders for dynamic price following

---

## Running Examples

```bash
# Data layer demo
cargo run --bin data-demo

# Golden Cross backtest
cargo run --example golden_cross_backtest -p alphafield-backtest
```

---

## API Quick Reference

### Health Check
```bash
curl http://localhost:8080/api/health
```

### List Cached Symbols
```bash
curl http://localhost:8080/api/data/symbols
```

### Fetch Data
```bash
curl -X POST http://localhost:8080/api/data/fetch \
  -H "Content-Type: application/json" \
  -d '{"symbol": "ETH", "interval": "1h", "limit": 500}'
```

### Run Backtest
```bash
curl -X POST http://localhost:8080/api/backtest/run \
  -H "Content-Type: application/json" \
  -d '{"strategy": "GoldenCross", "symbol": "BTC", "interval": "1h", "days": 30}'
```

### Train ML Model (NEW)
```bash
curl -X POST http://localhost:8080/api/ml/train \
  -H "Content-Type: application/json" \
  -d '{
    "model_type": "RandomForest",
    "features": ["returns_5", "volatility_20", "rsi_14"],
    "symbol": "BTC",
    "interval": "1h",
    "days": 365
  }'
```

### Create OCO Order (NEW)
```bash
curl -X POST http://localhost:8080/api/orders/oco \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTC",
    "primary": {"side": "Buy", "price": 40000, "quantity": 0.5},
    "secondary": {"side": "Sell", "price": 39000, "quantity": 0.5}
  }'
```

---

## WebSocket Connection

Connect to real-time updates:

```javascript
const ws = new WebSocket('ws://localhost:8080/api/ws');

ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    console.log(msg.type, msg.data);
};

// Request snapshot
ws.send(JSON.stringify({ command: 'Snapshot' }));
```

---

## Common Issues

### Database Connection Failed

Check that:
1. PostgreSQL is running
2. `DATABASE_URL` is correct in `.env`
3. TimescaleDB extension is installed

### No Data Available

Fetch data first using the Data tab or API:
```bash
curl -X POST http://localhost:8080/api/data/fetch \
  -d '{"symbol": "BTC", "interval": "1h", "limit": 1000}'
```

### API Key Errors

API keys are optional but recommended for higher rate limits:
- Binance: https://www.binance.com/en/my/settings/api-management
- CoinGecko: https://www.coingecko.com/api/pricing

---

## Next Steps

- 📖 [API Reference](api.md) - Complete endpoint documentation
- 🏗️ [Architecture](architecture.md) - System design overview
- 📊 [Roadmap](roadmap.md) - Development progress
