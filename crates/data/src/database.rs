use alphafield_core::{Bar, QuantError, Result, Tick};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};
use std::env;

/// Database client for storing and retrieving market data
#[derive(Clone)]
pub struct DatabaseClient {
    pool: Pool<Postgres>,
}

impl DatabaseClient {
    /// Creates a new database client from the DATABASE_URL environment variable
    pub async fn new_from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL").map_err(|_| {
            QuantError::Api("DATABASE_URL environment variable not set".to_string())
        })?;

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let client = Self { pool };
        client.initialize().await?;
        Ok(client)
    }

    /// Initializes the database schema
    async fn initialize(&self) -> Result<()> {
        // Create table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS candles (
                symbol VARCHAR(20) NOT NULL,
                timeframe VARCHAR(10) NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                open DOUBLE PRECISION NOT NULL,
                high DOUBLE PRECISION NOT NULL,
                low DOUBLE PRECISION NOT NULL,
                close DOUBLE PRECISION NOT NULL,
                volume DOUBLE PRECISION NOT NULL,
                PRIMARY KEY (symbol, timeframe, timestamp)
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Create index separately
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_candles_timestamp ON candles(timestamp)"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // If TimescaleDB is available, enable extension and convert to hypertable
        let _ = sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;")
            .execute(&self.pool)
            .await;

        // Create hypertable for candles if using TimescaleDB
        let _ = sqlx::query("SELECT create_hypertable('candles', 'timestamp', if_not_exists => TRUE);")
            .execute(&self.pool)
            .await;

        // Enable compression on candles hypertable (compress data older than 7 days)
        let _ = sqlx::query(
            r#"
            ALTER TABLE candles SET (
                timescaledb.compress,
                timescaledb.compress_segmentby = 'symbol, timeframe'
            );
            "#
        )
        .execute(&self.pool)
        .await;

        // Add compression policy for candles (compress chunks older than 7 days)
        let _ = sqlx::query(
            "SELECT add_compression_policy('candles', INTERVAL '7 days', if_not_exists => TRUE);"
        )
        .execute(&self.pool)
        .await;

        // Create trades table for tick-level data
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trades (
                id BIGSERIAL PRIMARY KEY,
                symbol VARCHAR(20) NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                price DOUBLE PRECISION NOT NULL,
                quantity DOUBLE PRECISION NOT NULL,
                is_buyer_maker BOOLEAN NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Index for trades
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_trades_timestamp ON trades(timestamp)")
            .execute(&self.pool)
            .await
            .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Convert trades to hypertable if possible
        let _ = sqlx::query("SELECT create_hypertable('trades', 'timestamp', if_not_exists => TRUE);")
            .execute(&self.pool)
            .await;

        // Enable compression on trades hypertable
        let _ = sqlx::query(
            r#"
            ALTER TABLE trades SET (
                timescaledb.compress,
                timescaledb.compress_segmentby = 'symbol'
            );
            "#
        )
        .execute(&self.pool)
        .await;

        // Add compression policy for trades (compress chunks older than 1 day)
        let _ = sqlx::query(
            "SELECT add_compression_policy('trades', INTERVAL '1 day', if_not_exists => TRUE);"
        )
        .execute(&self.pool)
        .await;

        Ok(())
    }

    /// Saves a batch of bars to the database
    pub async fn save_bars(&self, symbol: &str, timeframe: &str, bars: &[Bar]) -> Result<()> {
        if bars.is_empty() {
            return Ok(());
        }

        // Use a transaction for atomicity
        let mut tx = self.pool.begin().await.map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        for bar in bars {
            sqlx::query(
                r#"
                INSERT INTO candles (symbol, timeframe, timestamp, open, high, low, close, volume)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (symbol, timeframe, timestamp) 
                DO UPDATE SET 
                    open = EXCLUDED.open,
                    high = EXCLUDED.high,
                    low = EXCLUDED.low,
                    close = EXCLUDED.close,
                    volume = EXCLUDED.volume
                "#
            )
            .bind(symbol)
            .bind(timeframe)
            .bind(bar.timestamp)
            .bind(bar.open)
            .bind(bar.high)
            .bind(bar.low)
            .bind(bar.close)
            .bind(bar.volume)
            .execute(&mut *tx)
            .await
            .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        }

        tx.commit().await.map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        Ok(())
    }

    /// Saves a single tick/trade to the trades table
    pub async fn save_tick(&self, symbol: &str, tick: &Tick) -> Result<()> {
        // Validate tick first
        tick.validate()?;

        sqlx::query(
            r#"
            INSERT INTO trades (symbol, timestamp, price, quantity, is_buyer_maker)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(symbol)
        .bind(tick.timestamp)
        .bind(tick.price)
        .bind(tick.quantity)
        .bind(tick.is_buyer_maker)
        .execute(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(())
    }

    /// Loads bars from the database
    pub async fn load_bars(&self, symbol: &str, timeframe: &str) -> Result<Vec<Bar>> {
        let rows = sqlx::query(
            r#"
            SELECT timestamp, open, high, low, close, volume
            FROM candles
            WHERE symbol = $1 AND timeframe = $2
            ORDER BY timestamp ASC
            "#
        )
        .bind(symbol)
        .bind(timeframe)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let bars = rows.into_iter().map(|row| {
            Bar {
                timestamp: row.get("timestamp"),
                open: row.get("open"),
                high: row.get("high"),
                low: row.get("low"),
                close: row.get("close"),
                volume: row.get("volume"),
            }
        }).collect();

        Ok(bars)
    }

    /// Loads bars for a specific time range
    pub async fn load_bars_range(
        &self,
        symbol: &str,
        timeframe: &str,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Bar>> {
        let rows = sqlx::query(
            r#"
            SELECT timestamp, open, high, low, close, volume
            FROM candles
            WHERE symbol = $1 AND timeframe = $2 AND timestamp >= $3 AND timestamp <= $4
            ORDER BY timestamp ASC
            "#
        )
        .bind(symbol)
        .bind(timeframe)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let bars = rows.into_iter().map(|row| {
            Bar {
                timestamp: row.get("timestamp"),
                open: row.get("open"),
                high: row.get("high"),
                low: row.get("low"),
                close: row.get("close"),
                volume: row.get("volume"),
            }
        }).collect();

        Ok(bars)
    }

    /// Checks if data exists for the given symbol and timeframe
    pub async fn exists(&self, symbol: &str, timeframe: &str) -> Result<bool> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM candles WHERE symbol = $1 AND timeframe = $2"
        )
        .bind(symbol)
        .bind(timeframe)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(row.0 > 0)
    }

    /// Lists all symbols and their metadata in the database
    pub async fn list_symbols(&self) -> Result<Vec<CachedSymbol>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                symbol,
                timeframe,
                COUNT(*) as bar_count,
                MIN(timestamp) as first_bar,
                MAX(timestamp) as last_bar
            FROM candles
            GROUP BY symbol, timeframe
            ORDER BY symbol, timeframe
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let symbols = rows.into_iter().map(|row| {
            let first: Option<chrono::DateTime<chrono::Utc>> = row.get("first_bar");
            let last: Option<chrono::DateTime<chrono::Utc>> = row.get("last_bar");
            CachedSymbol {
                symbol: row.get("symbol"),
                timeframe: row.get("timeframe"),
                bar_count: row.get("bar_count"),
                first_bar: first.map(|d| d.format("%Y-%m-%d %H:%M").to_string()),
                last_bar: last.map(|d| d.format("%Y-%m-%d %H:%M").to_string()),
            }
        }).collect();

        Ok(symbols)
    }

    /// Deletes all bars for a symbol/timeframe
    pub async fn delete_bars(&self, symbol: &str, timeframe: &str) -> Result<()> {
        sqlx::query("DELETE FROM candles WHERE symbol = $1 AND timeframe = $2")
            .bind(symbol)
            .bind(timeframe)
            .execute(&self.pool)
            .await
            .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        Ok(())
    }

    // =========================================================================
    // Survivorship Bias Handling
    // =========================================================================

    /// Initialize asset status table for tracking delisted/migrated assets
    pub async fn initialize_asset_status(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS asset_status (
                symbol VARCHAR(20) PRIMARY KEY,
                status VARCHAR(20) NOT NULL DEFAULT 'active',
                delist_date TIMESTAMPTZ,
                migration_to VARCHAR(20),
                notes TEXT,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Create index on status for quick filtering
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_asset_status ON asset_status(status)")
            .execute(&self.pool)
            .await
            .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(())
    }

    /// Record an asset as delisted
    pub async fn mark_delisted(
        &self, 
        symbol: &str, 
        delist_date: chrono::DateTime<chrono::Utc>,
        notes: Option<&str>
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO asset_status (symbol, status, delist_date, notes, updated_at)
            VALUES ($1, 'delisted', $2, $3, NOW())
            ON CONFLICT (symbol) DO UPDATE SET
                status = 'delisted',
                delist_date = EXCLUDED.delist_date,
                notes = COALESCE(EXCLUDED.notes, asset_status.notes),
                updated_at = NOW()
            "#
        )
        .bind(symbol)
        .bind(delist_date)
        .bind(notes)
        .execute(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(())
    }

    /// Record an asset migration (e.g., LUNA -> LUNC)
    pub async fn mark_migrated(
        &self,
        old_symbol: &str,
        new_symbol: &str,
        migration_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO asset_status (symbol, status, delist_date, migration_to, updated_at)
            VALUES ($1, 'migrated', $2, $3, NOW())
            ON CONFLICT (symbol) DO UPDATE SET
                status = 'migrated',
                delist_date = EXCLUDED.delist_date,
                migration_to = EXCLUDED.migration_to,
                updated_at = NOW()
            "#
        )
        .bind(old_symbol)
        .bind(migration_date)
        .bind(new_symbol)
        .execute(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(())
    }

    /// Get status of an asset
    pub async fn get_asset_status(&self, symbol: &str) -> Result<Option<AssetStatus>> {
        let row = sqlx::query(
            r#"
            SELECT symbol, status, delist_date, migration_to, notes
            FROM asset_status
            WHERE symbol = $1
            "#
        )
        .bind(symbol)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(row.map(|r| AssetStatus {
            symbol: r.get("symbol"),
            status: r.get("status"),
            delist_date: r.get("delist_date"),
            migration_to: r.get("migration_to"),
            notes: r.get("notes"),
        }))
    }

    /// List all delisted/migrated assets (for survivorship-bias-free backtests)
    pub async fn list_delisted_assets(&self) -> Result<Vec<AssetStatus>> {
        let rows = sqlx::query(
            r#"
            SELECT symbol, status, delist_date, migration_to, notes
            FROM asset_status
            WHERE status IN ('delisted', 'migrated')
            ORDER BY delist_date DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(rows.into_iter().map(|r| AssetStatus {
            symbol: r.get("symbol"),
            status: r.get("status"),
            delist_date: r.get("delist_date"),
            migration_to: r.get("migration_to"),
            notes: r.get("notes"),
        }).collect())
    }

    /// Load bars including delisted assets (for survivorship-bias-free backtests)
    pub async fn load_bars_with_delisted(
        &self,
        symbol: &str,
        timeframe: &str,
        include_delisted: bool,
    ) -> Result<Vec<Bar>> {
        // First check if this is a delisted asset
        if !include_delisted {
            if let Some(status) = self.get_asset_status(symbol).await? {
                if status.status == "delisted" || status.status == "migrated" {
                    return Err(QuantError::DataValidation(format!(
                        "Symbol {} is {}, use include_delisted=true for backtest",
                        symbol, status.status
                    )));
                }
            }
        }

        // Load the bars
        self.load_bars(symbol, timeframe).await
    }

    // =========================================================================
    // Data Integrity Checks
    // =========================================================================

    /// Check for missing bars (gaps) in the data
    pub async fn check_data_gaps(
        &self,
        symbol: &str,
        timeframe: &str,
    ) -> Result<Vec<DataGap>> {
        // Get all timestamps
        let rows = sqlx::query(
            r#"
            SELECT timestamp
            FROM candles
            WHERE symbol = $1 AND timeframe = $2
            ORDER BY timestamp ASC
            "#
        )
        .bind(symbol)
        .bind(timeframe)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| QuantError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let timestamps: Vec<chrono::DateTime<chrono::Utc>> = rows
            .iter()
            .map(|r| r.get("timestamp"))
            .collect();

        // Determine expected interval based on timeframe
        let interval_minutes = match timeframe {
            "1m" => 1,
            "5m" => 5,
            "15m" => 15,
            "1h" => 60,
            "4h" => 240,
            "1d" => 1440,
            _ => 60, // Default to 1 hour
        };

        let mut gaps = Vec::new();
        for window in timestamps.windows(2) {
            let expected_next = window[0] + chrono::Duration::minutes(interval_minutes);
            if window[1] > expected_next + chrono::Duration::minutes(interval_minutes / 2) {
                gaps.push(DataGap {
                    start: window[0],
                    end: window[1],
                    expected_bars: ((window[1] - window[0]).num_minutes() / interval_minutes) as usize - 1,
                });
            }
        }

        Ok(gaps)
    }

    /// Check for price outliers (abnormal price movements)
    pub async fn check_price_outliers(
        &self,
        symbol: &str,
        timeframe: &str,
        threshold_pct: f64,
    ) -> Result<Vec<PriceOutlier>> {
        let bars = self.load_bars(symbol, timeframe).await?;
        let mut outliers = Vec::new();

        for i in 1..bars.len() {
            let prev_close = bars[i - 1].close;
            let curr_open = bars[i].open;
            let gap_pct = ((curr_open - prev_close) / prev_close).abs();

            if gap_pct > threshold_pct {
                outliers.push(PriceOutlier {
                    timestamp: bars[i].timestamp,
                    previous_close: prev_close,
                    current_open: curr_open,
                    gap_percent: gap_pct * 100.0,
                });
            }
        }

        Ok(outliers)
    }
}

/// Asset status information
#[derive(Clone, Debug, serde::Serialize)]
pub struct AssetStatus {
    pub symbol: String,
    pub status: String,
    pub delist_date: Option<chrono::DateTime<chrono::Utc>>,
    pub migration_to: Option<String>,
    pub notes: Option<String>,
}

/// Metadata about a cached symbol in the database
#[derive(Clone, Debug, serde::Serialize)]
pub struct CachedSymbol {
    pub symbol: String,
    pub timeframe: String,
    pub bar_count: i64,
    pub first_bar: Option<String>,
    pub last_bar: Option<String>,
}

/// Represents a gap in the data
#[derive(Clone, Debug, serde::Serialize)]
pub struct DataGap {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
    pub expected_bars: usize,
}

/// Represents a price outlier
#[derive(Clone, Debug, serde::Serialize)]
pub struct PriceOutlier {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub previous_close: f64,
    pub current_open: f64,
    pub gap_percent: f64,
}

