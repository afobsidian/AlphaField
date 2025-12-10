# 🖥️ AlphaField Dashboard

The **Dashboard** crate provides a web-based user interface and REST API for identifying trading opportunities, managing data, and running backtests.

## 🏗️ Architecture

The dashboard is built with:
- **Backend**: Rust (`axum`)
- **Frontend**: Vanilla JavaScript (no build step required) + HTML/CSS
- **Communication**: REST API + WebSocket (`/api/ws`)

## 📂 Structure

```
crates/dashboard/
├── src/
│   ├── api.rs          # Route definitions & app state
│   ├── handlers/       # Request handlers
│   ├── websocket.rs    # Real-time updates hub
│   └── main.rs         # Server entry point
├── static/
│   ├── index.html      # Single Page Application entry
│   ├── app.js          # Frontend logic (SPA routing, charts)
│   └── style.css       # Styling variables & components
└── Cargo.toml
```

## 🚀 Running the Dashboard

### Standalone Mode (Development)
You can run the dashboard crate specifically:

```bash
# From project root
cargo run --bin dashboard_server
```

Open `http://localhost:8080` in your browser.

### Features

- **Real-time Monitoring**: WebSocket connection streams portfolio updates and logs.
- **Data Manager**: View cached symbols, gaps, and outliers. Fetch new data from exchanges.
- **Backtesting Lab**: Run strategies (GoldenCross, MeanReversion, etc.) visually.
- **Advanced Analysis**:
    - **Monte Carlo**: Simulate 1000+ trade permutations.
    - **Correlation**: Check strategy diversity.
    - **Walk Forward**: Validate robustness over sliding windows.
- **Sentiment Analysis**: Fear & Greed index tracking.

## 🔌 API

See [API Documentation](../../doc/api.md) for full endpoint details.

### Common Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/health` | Server status |
| `GET` | `/api/data/symbols` | List available data |
| `POST` | `/api/backtest/run` | Execute backtest |
| `WS` | `/api/ws` | WebSocket stream |
