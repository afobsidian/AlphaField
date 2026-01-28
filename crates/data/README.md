# 💾 AlphaField Data Crate

Handles data ingestion, storage (TimescaleDB), and quality monitoring.

## Why This Crate Exists

This crate is the data foundation for AlphaField, providing reliable, validated market data from multiple sources. By centralizing data operations, we ensure:

- **Consistent data quality**: All data passes through the same validation and monitoring
- **Automatic failover**: If one data source fails, the system automatically switches to another
- **Efficient storage**: TimescaleDB optimizes time-series queries and compresses old data
- **Audit trail**: Track where data came from and when it was ingested

## 🔌 Connectors

### UnifiedDataClient (Recommended)

Smart router that automatically fails over between sources. Use this for all production data access.

```rust
use alphafield_data::{UnifiedDataClient, Bar};
use chrono::{Utc, Duration};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with API keys
    let client = UnifiedDataClient::builder()
        .binance_key(env::var("BINANCE_API_KEY")?)
        .coingecko_key(env::var("COINGECKO_API_KEY")?)
        .build()?;

    // Fetch data - automatically routes to best available source
    let end = Utc::now();
    let start = end - Duration::days(30);

    let bars = client.fetch_bars("BTCUSDT", start, end).await?;

    println!("Fetched {} bars", bars.len());
    Ok(())
}
```

### Individual Clients

Use individual clients when you need data from a specific source.

```rust
use alphafield_data::{BinanceClient, CoinGeckoClient};
use chrono::{Utc, Duration};

// Fetch from Binance
let binance = BinanceClient::new()?;
let bars = binance.fetch_bars("BTCUSDT", start, end).await?;

// Fetch from CoinGecko (better for historical data)
let coingecko = CoinGeckoClient::new()?;
let bars = coingecko.fetch_bars("BTCUSDT", start, end).await?;
```

### Why Multiple Data Sources?

Different sources have different strengths:

| Source | Best For | Limitations |
|--------|----------|-------------|
| **Binance** | Real-time, recent data | Rate limits, limited history |
| **CoinGecko** | Historical data, metadata | Slower, no real-time |
| **Coinlayer** | Forex, crypto pairs | Limited free tier |

## 🗄️ TimescaleDB Storage

### Connecting to TimescaleDB

```rust
use alphafield_data::TimescaleDB;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect using DATABASE_URL environment variable
    let db = TimescaleDB::from_env().await?;

    // Or connect manually
    let db = TimescaleDB::connect("postgresql://localhost/alphafield").await?;

    Ok(())
}
```

### Storing Bars

```rust
use alphafield_data::{TimescaleDB, Bar};
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = TimescaleDB::from_env().await?;

    // Prepare a bar
    let bar = Bar {
        symbol: "BTCUSDT".to_string(),
        timestamp: Utc::now(),
        open: 45000.0,
        high: 45100.0,
        low: 44900.0,
        close: 45050.0,
        volume: Decimal::from_str("1.5")?,
    };

    // Store the bar
    db.store_bar(&bar).await?;

    println!("Stored bar for BTCUSDT");
    Ok(())
}
```

### Querying Bars

```rust
use alphafield_data::TimescaleDB;
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = TimescaleDB::from_env().await?;

    // Query bars for time range
    let end = Utc::now();
    let start = end - Duration::days(30);

    let bars = db.query_bars(
        "BTCUSDT",
        start,
        end,
        None,  // No limit
    ).await?;

    println!("Retrieved {} bars", bars.len());
    Ok(())
}
```

### TimescaleDB Features

**Hypertables** for efficient time-series storage:

```sql
-- Hypertables are automatically created for candles and trades
-- Query performance is optimized by time
SELECT * FROM candles
WHERE symbol = 'BTCUSDT'
  AND timestamp >= '2025-01-01'
  AND timestamp <= '2025-01-31'
ORDER BY timestamp;
```

**Automatic Compression**:
- Candles: Compressed after 7 days
- Trades: Compressed after 1 day
- Reduces storage by ~90% with minimal query impact

**Survivorship Bias Prevention**:

```rust
// Check if asset is still active
let is_active = db.check_asset_active("BTCUSDT").await?;

if is_active {
    // Fetch data - asset is still trading
    let bars = db.query_bars("BTCUSDT", start, end, None).await?;
}
```

## 🛡️ Data Quality

### IngestionMonitor

Monitors data quality in real-time and alerts on issues.

```rust
use alphafield_data::{IngestionMonitor, Bar};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut monitor = IngestionMonitor::new();

    // Monitor a symbol
    monitor.monitor_symbol("BTCUSDT", Duration::from_secs(60));

    // Check for gaps
    let gaps = monitor.check_gaps(&bars)?;
    if !gaps.is_empty() {
        eprintln!("Found {} gaps in data", gaps.len());
    }

    // Check for outliers
    let outliers = monitor.check_outliers(&bars)?;
    if !outliers.is_empty() {
        eprintln!("Found {} outliers in data", outliers.len());
    }

    Ok(())
}
```

### Gap Detection

Identifies missing bars in sequences.

```rust
use alphafield_data::IngestionMonitor;

let monitor = IngestionMonitor::new();

let bars = fetch_bars().await?;

// Detect gaps (bars should arrive every 1 hour)
let gaps = monitor.check_gaps(&bars)?;

for gap in gaps {
    println!("Gap detected: {} to {} (missing {} bars)",
        gap.start, gap.end, gap.missing_count);
}
```

### Outlier Detection

Flags significant price deviations (>5% by default).

```rust
use alphafield_data::IngestionMonitor;

let monitor = IngestionMonitor::new();

let bars = fetch_bars().await?;

// Detect outliers (price moves > 5%)
let outliers = monitor.check_outliers(&bars)?;

for outlier in outliers {
    println!("Outlier at {}: {} ({}% deviation)",
        outlier.timestamp, outlier.price, outlier.deviation);
}
```

### Freshness Monitoring

Alerts on stale data feeds.

```rust
use alphafield_data::IngestionMonitor;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut monitor = IngestionMonitor::new();

    // Check if data is fresh (within 5 minutes)
    let is_fresh = monitor.check_freshness("BTCUSDT", Duration::from_secs(300))?;

    if !is_fresh {
        eprintln!("Data for BTCUSDT is stale!");
    }

    Ok(())
}
```

## Common Workflows

### Fetch and Store Data

```rust
use alphafield_data::{UnifiedDataClient, TimescaleDB};
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    let client = UnifiedDataClient::new()?;
    let db = TimescaleDB::from_env().await?;

    // Fetch data
    let end = Utc::now();
    let start = end - Duration::days(30);
    let bars = client.fetch_bars("BTCUSDT", start, end).await?;

    // Store in database
    for bar in bars {
        db.store_bar(&bar).await?;
    }

    println!("Stored {} bars", bars.len());
    Ok(())
}
```

### Fill Missing Data (Gap Filling)

```rust
use alphafield_data::{GapFiller, UnifiedDataClient, TimescaleDB};
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = UnifiedDataClient::new()?;
    let db = TimescaleDB::from_env().await?;
    let gap_filler = GapFiller::new(client, db);

    // Detect and fill gaps
    let end = Utc::now();
    let start = end - Duration::days(30);

    let filled = gap_filler.fill_gaps("BTCUSDT", start, end).await?;

    println!("Filled {} gaps", filled);
    Ok(())
}
```

### Quality Check Pipeline

```rust
use alphafield_data::{IngestionMonitor, UnifiedDataClient};
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = UnifiedDataClient::new()?;
    let monitor = IngestionMonitor::new();

    // Fetch data
    let bars = client.fetch_bars("BTCUSDT", start, end).await?;

    // Quality checks
    let gaps = monitor.check_gaps(&bars)?;
    let outliers = monitor.check_outliers(&bars)?;

    if gaps.is_empty() && outliers.is_empty() {
        println!("Data quality: GOOD");
    } else {
        println!("Data quality: {} gaps, {} outliers",
            gaps.len(), outliers.len());
    }

    Ok(())
}
```

## Best Practices

1. **Use UnifiedDataClient**: Automatically routes to best available source with failover
2. **Always validate data**: Check for gaps, outliers, and freshness before using
3. **Store in TimescaleDB**: Efficient storage for time-series data with automatic compression
4. **Handle rate limits**: Data sources have rate limits; use appropriate delays between requests
5. **Check asset status**: Verify asset is still active before fetching historical data
6. **Monitor data quality**: Set up alerts for gaps, outliers, and stale data
7. **Use connection pooling**: TimescaleDB connection pool for efficient database access
8. **Index by timestamp**: Query by time range for best performance
