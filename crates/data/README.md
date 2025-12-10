# 💾 AlphaField Data Crate

Handles data ingestion, storage (TimescaleDB), and quality monitoring.

## 🔌 Connectors

- **`BinanceClient`**: Primary crypto market data source.
- **`CoinGeckoClient`**: Supplemental historical and metadata source.
- **`CoinlayerClient`**: Forex/Crypto fallback data.
- **`UnifiedDataClient`**: Smart router that fails over between sources automatically.

## 🗄️ Storage

Uses **TimescaleDB** for efficient time-series storage.

- **Hypertables**: `candles`, `trades`.
- **Compression**: Automatic compression after 7 days (candles) / 1 day (trades).
- **Survivorship**: `asset_status` table tracks delisted or renamed assets.

## 🛡️ Data Quality

The `IngestionMonitor` and `GapFiller` ensure data integrity:
- **Gap Detection**: Identifies missing bars in sequences.
- **Outlier Detection**: Flags >5% deviations between closes.
- **Freshness**: Alerts on stale data feeds.
