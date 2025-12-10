# 🧱 AlphaField Core Crate

Defines the fundamental types and traits shared across the entire AlphaField ecosystem.

## 📦 Key Types

- **`Bar`**: Standard OHLCV candlestick with strict timestamp handling.
- **`Trade`**: Represents a completed trade with P&L, MAE, and MFE data.
- **`Order`**: Order requests with Type (Market/Limit), Side (Buy/Sell), and TIF.
- **`Signal`**: Output from strategies (`Buy`, `Sell`, `Hold`) with confidence/size.
- **`QuantError`**: Centralized error type for the workspace.

## 🧬 Traits

- **`Strategy`**: The interface that all trading strategies must implement.
- **`DataSource`**: Interface for data providers.
