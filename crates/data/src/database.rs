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
