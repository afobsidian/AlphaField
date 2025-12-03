//! # AlphaField Data
//!
//! Data ingestion and API client implementations for cryptocurrency market data.
//! Supports both Coinlayer API (daily rates) and Binance API (OHLC candlesticks).
//!
//! ## Features
//! - Async HTTP client with connection pooling
//! - API key authentication via .env file
//! - Data transformation to core types
//! - Coinlayer: Live rates, historical data, conversions
//! - Binance: True OHLC candlesticks, volume data, multiple intervals

use alphafield_core::{Bar, QuantError, Result};
use chrono::{NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub mod storage;
pub use storage::HistoricalDataStorage;

// ============================================================================
// COINLAYER API CLIENT
// ============================================================================

/// Coinlayer API client for fetching cryptocurrency exchange rates
///
/// # Authentication
/// Requires an API key from https://coinlayer.com
/// Set the `COINLAYER_API_KEY` environment variable or pass it directly
///
/// # Example
/// ```no_run
/// use alphafield_data::CoinlayerClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = CoinlayerClient::new_from_env()?;
///     let rates = client.get_live_rates(None, None).await?;
///     println!("Fetched {} rates", rates.rates.len());
///     Ok(())
/// }
/// ```
pub struct CoinlayerClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl CoinlayerClient {
    /// Creates a new Coinlayer API client with the given API key
    ///
    /// # Arguments
    /// * `api_key` - Your Coinlayer API key
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: "http://api.coinlayer.com".to_string(),
            api_key,
        }
    }

    /// Creates a new client by reading the API key from the COINLAYER_API_KEY environment variable
    ///
    /// # Errors
    /// Returns an error if the environment variable is not set
    pub fn new_from_env() -> Result<Self> {
        let api_key = std::env::var("COINLAYER_API_KEY").map_err(|_| {
            QuantError::Api(
                "COINLAYER_API_KEY environment variable not set. Please add it to your .env file"
                    .to_string(),
            )
        })?;
        Ok(Self::new(api_key))
    }

    /// Fetches live exchange rates for all or specific cryptocurrencies
    ///
    /// # Arguments
    /// * `target` - Target currency (e.g., "USD", "EUR"). Default: USD
    /// * `symbols` - Comma-separated list of crypto symbols (e.g., "BTC,ETH"). None = all
    ///
    /// # Returns
    /// LiveRatesResponse with current exchange rates
    pub async fn get_live_rates(
        &self,
        target: Option<&str>,
        symbols: Option<&str>,
    ) -> Result<LiveRatesResponse> {
        let url = format!("{}/live", self.base_url);

        let mut params = vec![("access_key", self.api_key.as_str())];

        if let Some(t) = target {
            params.push(("target", t));
        }

        if let Some(s) = symbols {
            params.push(("symbols", s));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(QuantError::Api(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        let data = response
            .json::<LiveRatesResponse>()
            .await
            .map_err(|e| QuantError::Parse(format!("Failed to parse response: {}", e)))?;

        if !data.success {
            return Err(QuantError::Api(format!("API error: {:?}", data.error)));
        }

        Ok(data)
    }

    /// Fetches historical exchange rates for a specific date
    ///
    /// # Arguments
    /// * `date` - Date in YYYY-MM-DD format
    /// * `target` - Target currency (e.g., "USD", "EUR"). Default: USD
    /// * `symbols` - Comma-separated list of crypto symbols. None = all
    ///
    /// # Returns
    /// HistoricalRatesResponse with rates for the specified date
    pub async fn get_historical_rates(
        &self,
        date: &str,
        target: Option<&str>,
        symbols: Option<&str>,
    ) -> Result<HistoricalRatesResponse> {
        let url = format!("{}/{}", self.base_url, date);

        let mut params = vec![("access_key", self.api_key.as_str())];

        if let Some(t) = target {
            params.push(("target", t));
        }

        if let Some(s) = symbols {
            params.push(("symbols", s));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(QuantError::Api(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        let data = response
            .json::<HistoricalRatesResponse>()
            .await
            .map_err(|e| QuantError::Parse(format!("Failed to parse response: {}", e)))?;

        if !data.success {
            return Err(QuantError::Api(format!("API error: {:?}", data.error)));
        }

        Ok(data)
    }

    /// Converts an amount from one currency to another
    ///
    /// # Arguments
    /// * `from` - Source currency symbol (e.g., "BTC")
    /// * `to` - Target currency symbol (e.g., "ETH" or "USD")
    /// * `amount` - Amount to convert
    /// * `date` - Optional historical date for conversion
    ///
    /// # Returns
    /// ConversionResponse with the converted amount
    pub async fn convert(
        &self,
        from: &str,
        to: &str,
        amount: f64,
        date: Option<&str>,
    ) -> Result<ConversionResponse> {
        let url = format!("{}/convert", self.base_url);

        let amount_str = amount.to_string();
        let mut params = vec![
            ("access_key", self.api_key.as_str()),
            ("from", from),
            ("to", to),
            ("amount", amount_str.as_str()),
        ];

        if let Some(d) = date {
            params.push(("date", d));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(QuantError::Api(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        let data = response
            .json::<ConversionResponse>()
            .await
            .map_err(|e| QuantError::Parse(format!("Failed to parse response: {}", e)))?;

        if !data.success {
            return Err(QuantError::Api(format!("API error: {:?}", data.error)));
        }

        Ok(data)
    }

    /// Fetches list of all available cryptocurrencies and fiat currencies
    ///
    /// # Returns
    /// ListResponse with crypto and fiat currency information
    pub async fn get_list(&self) -> Result<ListResponse> {
        let url = format!("{}/list", self.base_url);

        let params = vec![("access_key", self.api_key.as_str())];

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(QuantError::Api(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        let data = response
            .json::<ListResponse>()
            .await
            .map_err(|e| QuantError::Parse(format!("Failed to parse response: {}", e)))?;

        if !data.success {
            return Err(QuantError::Api(format!("API error: {:?}", data.error)));
        }

        Ok(data)
    }

    /// Fetches historical rates for multiple days and converts to Bar structs
    ///
    /// # Arguments
    /// * `symbol` - Cryptocurrency symbol (e.g., "BTC")
    /// * `target` - Target currency (e.g., "USD")
    /// * `start_date` - Start date (YYYY-MM-DD)
    /// * `days` - Number of days to fetch
    ///
    /// # Returns
    /// Vector of Bar structs with daily data (open=close=rate, high=low=rate, volume=0)
    ///
    /// # Note
    /// Coinlayer doesn't provide OHLC data, so we create synthetic bars
    /// where open=high=low=close=rate. This is suitable for daily snapshots
    /// but not for intraday analysis.
    pub async fn get_historical_bars(
        &self,
        symbol: &str,
        target: &str,
        start_date: NaiveDate,
        days: u32,
    ) -> Result<Vec<Bar>> {
        let mut bars = Vec::new();

        for i in 0..days {
            let date = start_date + chrono::Duration::days(i as i64);
            let date_str = date.format("%Y-%m-%d").to_string();

            match self
                .get_historical_rates(&date_str, Some(target), Some(symbol))
                .await
            {
                Ok(response) => {
                    if let Some(rate) = response.rates.get(symbol) {
                        let timestamp = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());

                        let bar = Bar {
                            timestamp,
                            open: *rate,
                            high: *rate,
                            low: *rate,
                            close: *rate,
                            volume: 0.0, // Coinlayer doesn't provide volume
                        };

                        bars.push(bar);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch data for {}: {}", date_str, e);
                    // Continue with next date
                }
            }

            // Rate limiting: sleep for 1 second between requests on free tier
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }

        Ok(bars)
    }
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Response from /live endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveRatesResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default)]
    pub rates: HashMap<String, f64>,
}

/// Response from historical endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalRatesResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub historical: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default)]
    pub rates: HashMap<String, f64>,
}

/// Response from /convert endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<ConversionQuery>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<ConversionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionQuery {
    pub from: String,
    pub to: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionInfo {
    pub timestamp: i64,
    pub rate: f64,
}

/// Response from /list endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(default)]
    pub crypto: HashMap<String, CryptoInfo>,
    #[serde(default)]
    pub fiat: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoInfo {
    pub symbol: String,
    pub name: String,
    pub name_full: String,
    pub max_supply: Option<f64>,
    pub icon_url: String,
}

// ============================================================================
// BINANCE API CLIENT
// ============================================================================

use hmac::{Hmac, Mac};
use sha2::Sha256;

/// HMAC-SHA256 type alias for signing authenticated requests
#[allow(dead_code)]
type HmacSha256 = Hmac<Sha256>;

/// Binance API client for fetching OHLC candlestick data
///
/// # Authentication
/// Optional API key/secret for higher rate limits
/// Set `BINANCE_API_KEY` and `BINANCE_SECRET_KEY` environment variables
///
/// # Example
/// ```no_run
/// use alphafield_data::BinanceClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = BinanceClient::new(None, None);
///     let klines = client.get_klines("BTCUSDT", "1h", None, None, Some(100)).await?;
///     println!("Fetched {} candlesticks", klines.len());
///     Ok(())
/// }
/// ```
pub struct BinanceClient {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    /// Secret key for signing authenticated requests (reserved for future use)
    #[allow(dead_code)]
    secret_key: Option<String>,
}

impl BinanceClient {
    /// Creates a new Binance API client
    ///
    /// # Arguments
    /// * `api_key` - Optional API key for authenticated requests
    /// * `secret_key` - Optional secret key for signing requests
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: "https://api.binance.com".to_string(),
            api_key,
            secret_key,
        }
    }

    /// Creates a new client by reading credentials from environment variables
    ///
    /// # Returns
    /// Client with credentials if available, otherwise unauthenticated client
    pub fn new_from_env() -> Self {
        let api_key = std::env::var("BINANCE_API_KEY").ok();
        let secret_key = std::env::var("BINANCE_SECRET_KEY").ok();
        Self::new(api_key, secret_key)
    }

    /// Signs a query string with HMAC-SHA256 (reserved for authenticated endpoints)
    #[allow(dead_code)]
    fn sign_request(&self, query: &str) -> Result<String> {
        let secret = self
            .secret_key
            .as_ref()
            .ok_or_else(|| QuantError::Api("Secret key required for signing".to_string()))?;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| QuantError::Api(format!("Failed to create HMAC: {}", e)))?;

        mac.update(query.as_bytes());
        let result = mac.finalize();
        Ok(hex::encode(result.into_bytes()))
    }

    /// Fetches kline/candlestick data
    ///
    /// # Arguments
    /// * `symbol` - Trading pair symbol (e.g., "BTCUSDT", "ETHUSDT")
    /// * `interval` - Kline interval: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M
    /// * `start_time` - Start time in milliseconds (optional)
    /// * `end_time` - End time in milliseconds (optional)
    /// * `limit` - Number of klines to return (default 500, max 1000)
    ///
    /// # Returns
    /// Vector of Bar structs with OHLCV data
    ///
    /// # Example
    /// ```no_run
    /// # use alphafield_data::BinanceClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = BinanceClient::new(None, None);
    /// // Get last 100 1-hour candles for BTC/USDT
    /// let bars = client.get_klines("BTCUSDT", "1h", None, None, Some(100)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<Bar>> {
        let url = format!("{}/api/v3/klines", self.base_url);

        let mut params = vec![
            ("symbol", symbol.to_string()),
            ("interval", interval.to_string()),
        ];

        if let Some(start) = start_time {
            params.push(("startTime", start.to_string()));
        }

        if let Some(end) = end_time {
            params.push(("endTime", end.to_string()));
        }

        if let Some(lim) = limit {
            params.push(("limit", lim.to_string()));
        }

        let mut request = self.client.get(&url).query(&params);

        if let Some(ref key) = self.api_key {
            request = request.header("X-MBX-APIKEY", key);
        }

        let response = request
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(QuantError::Api(format!(
                "API returned status {}: {}",
                status, body
            )));
        }

        // Binance returns: [[openTime, open, high, low, close, volume, closeTime, ...], ...]
        let data = response
            .json::<Vec<Vec<serde_json::Value>>>()
            .await
            .map_err(|e| QuantError::Parse(format!("Failed to parse klines: {}", e)))?;

        let bars: Result<Vec<Bar>> = data
            .into_iter()
            .map(|kline| {
                if kline.len() < 7 {
                    return Err(QuantError::Parse(format!(
                        "Invalid kline format: expected at least 7 elements, got {}",
                        kline.len()
                    )));
                }

                let open_time = kline[0]
                    .as_i64()
                    .ok_or_else(|| QuantError::Parse("Invalid openTime".to_string()))?;

                let timestamp = Utc
                    .timestamp_millis_opt(open_time)
                    .single()
                    .ok_or_else(|| {
                        QuantError::Parse(format!("Invalid timestamp: {}", open_time))
                    })?;

                let open = kline[1]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .ok_or_else(|| QuantError::Parse("Invalid open price".to_string()))?;

                let high = kline[2]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .ok_or_else(|| QuantError::Parse("Invalid high price".to_string()))?;

                let low = kline[3]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .ok_or_else(|| QuantError::Parse("Invalid low price".to_string()))?;

                let close = kline[4]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .ok_or_else(|| QuantError::Parse("Invalid close price".to_string()))?;

                let volume = kline[5]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .ok_or_else(|| QuantError::Parse("Invalid volume".to_string()))?;

                let bar = Bar {
                    timestamp,
                    open,
                    high,
                    low,
                    close,
                    volume,
                };

                bar.validate()?;
                Ok(bar)
            })
            .collect();

        bars
    }

    /// Fetches 24-hour ticker price change statistics
    ///
    /// # Arguments
    /// * `symbol` - Trading pair symbol (e.g., "BTCUSDT"). None = all symbols
    ///
    /// # Returns
    /// Ticker24hrResponse with price statistics
    pub async fn get_ticker_24hr(&self, symbol: Option<&str>) -> Result<Ticker24hrResponse> {
        let url = format!("{}/api/v3/ticker/24hr", self.base_url);

        let mut request = self.client.get(&url);

        if let Some(sym) = symbol {
            request = request.query(&[("symbol", sym)]);
        }

        if let Some(ref key) = self.api_key {
            request = request.header("X-MBX-APIKEY", key);
        }

        let response = request
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(QuantError::Api(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        if symbol.is_some() {
            // Single symbol returns object
            let data = response
                .json::<Ticker24hr>()
                .await
                .map_err(|e| QuantError::Parse(format!("Failed to parse ticker: {}", e)))?;

            Ok(Ticker24hrResponse::Single(Box::new(data)))
        } else {
            // All symbols returns array
            let data = response
                .json::<Vec<Ticker24hr>>()
                .await
                .map_err(|e| QuantError::Parse(format!("Failed to parse tickers: {}", e)))?;

            Ok(Ticker24hrResponse::Multiple(data))
        }
    }

    /// Fetches exchange information (trading rules and symbol information)
    ///
    /// # Returns
    /// ExchangeInfo with all trading pairs and their details
    pub async fn get_exchange_info(&self) -> Result<ExchangeInfo> {
        let url = format!("{}/api/v3/exchangeInfo", self.base_url);

        let mut request = self.client.get(&url);

        if let Some(ref key) = self.api_key {
            request = request.header("X-MBX-APIKEY", key);
        }

        let response = request
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(QuantError::Api(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        let data = response
            .json::<ExchangeInfo>()
            .await
            .map_err(|e| QuantError::Parse(format!("Failed to parse exchange info: {}", e)))?;

        Ok(data)
    }
}

// ============================================================================
// BINANCE DATA STRUCTURES
// ============================================================================

/// 24-hour ticker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker24hr {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub weighted_avg_price: String,
    pub prev_close_price: String,
    pub last_price: String,
    pub last_qty: String,
    pub bid_price: String,
    pub bid_qty: String,
    pub ask_price: String,
    pub ask_qty: String,
    pub open_price: String,
    pub high_price: String,
    pub low_price: String,
    pub volume: String,
    pub quote_volume: String,
    pub open_time: i64,
    pub close_time: i64,
    pub first_id: i64,
    pub last_id: i64,
    pub count: i64,
}

/// Response from ticker/24hr endpoint
#[derive(Debug, Clone)]
pub enum Ticker24hrResponse {
    Single(Box<Ticker24hr>),
    Multiple(Vec<Ticker24hr>),
}

/// Exchange information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo {
    pub timezone: String,
    pub server_time: i64,
    pub rate_limits: Vec<RateLimit>,
    pub symbols: Vec<SymbolInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: String,
    pub interval: String,
    pub interval_num: i32,
    pub limit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInfo {
    pub symbol: String,
    pub status: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub base_asset_precision: i32,
    pub quote_asset_precision: i32,
}

// ============================================================================
// COINLAYER DATA STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: i32,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub info: Option<String>,
}

// ============================================================================
// COINGECKO API CLIENT
// ============================================================================

/// CoinGecko API client for fetching market data
///
/// # Authentication
/// Optional API key (Pro) via `x-cg-pro-api-key` header
///
/// # Example
/// ```no_run
/// use alphafield_data::CoinGeckoClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = CoinGeckoClient::new(None);
///     let price = client.get_price("bitcoin", "usd").await?;
///     println!("BTC Price: ${}", price);
///     Ok(())
/// }
/// ```
pub struct CoinGeckoClient {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl CoinGeckoClient {
    /// Creates a new CoinGecko API client
    pub fn new(api_key: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to build HTTP client");

        // Use Pro URL if key is present, otherwise public URL
        let base_url = if api_key.is_some() {
            "https://pro-api.coingecko.com/api/v3".to_string()
        } else {
            "https://api.coingecko.com/api/v3".to_string()
        };

        Self {
            client,
            base_url,
            api_key,
        }
    }

    /// Creates a new client from environment variables
    pub fn new_from_env() -> Self {
        let api_key = std::env::var("COINGECKO_API_KEY").ok();
        Self::new(api_key)
    }

    /// Fetches current price for a coin
    pub async fn get_price(&self, coin_id: &str, vs_currency: &str) -> Result<f64> {
        let url = format!("{}/simple/price", self.base_url);

        let mut request = self
            .client
            .get(&url)
            .query(&[("ids", coin_id), ("vs_currencies", vs_currency)]);

        if let Some(ref key) = self.api_key {
            request = request.header("x-cg-pro-api-key", key);
        }

        let response = request
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(QuantError::Api(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        let data: HashMap<String, HashMap<String, f64>> = response
            .json()
            .await
            .map_err(|e| QuantError::Parse(format!("Failed to parse price: {}", e)))?;

        data.get(coin_id)
            .and_then(|prices| prices.get(vs_currency))
            .copied()
            .ok_or_else(|| {
                QuantError::Api(format!(
                    "Price not found for {} in {}",
                    coin_id, vs_currency
                ))
            })
    }

    /// Fetches OHLC data
    /// Note: CoinGecko OHLC does not include volume
    pub async fn get_ohlc(&self, coin_id: &str, vs_currency: &str, days: u32) -> Result<Vec<Bar>> {
        let url = format!("{}/coins/{}/ohlc", self.base_url, coin_id);

        let mut request = self
            .client
            .get(&url)
            .query(&[("vs_currency", vs_currency), ("days", &days.to_string())]);

        if let Some(ref key) = self.api_key {
            request = request.header("x-cg-pro-api-key", key);
        }

        let response = request
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(QuantError::Api(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        // Response: [[time, open, high, low, close], ...]
        let data: Vec<Vec<f64>> = response
            .json()
            .await
            .map_err(|e| QuantError::Parse(format!("Failed to parse OHLC: {}", e)))?;

        let bars: Result<Vec<Bar>> = data
            .into_iter()
            .map(|candle| {
                if candle.len() < 5 {
                    return Err(QuantError::Parse("Invalid candle format".to_string()));
                }

                let timestamp = Utc
                    .timestamp_millis_opt(candle[0] as i64)
                    .single()
                    .ok_or_else(|| {
                        QuantError::Parse(format!("Invalid timestamp: {}", candle[0]))
                    })?;

                let bar = Bar {
                    timestamp,
                    open: candle[1],
                    high: candle[2],
                    low: candle[3],
                    close: candle[4],
                    volume: 0.0, // CoinGecko OHLC endpoint doesn't provide volume
                };

                Ok(bar)
            })
            .collect();

        bars
    }

    /// Fetches list of all supported coins
    pub async fn get_coins_list(&self) -> Result<Vec<CoinGeckoCoin>> {
        let url = format!("{}/coins/list", self.base_url);

        let mut request = self.client.get(&url);

        if let Some(ref key) = self.api_key {
            request = request.header("x-cg-pro-api-key", key);
        }

        let response = request
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(QuantError::Api(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        let coins: Vec<CoinGeckoCoin> = response
            .json()
            .await
            .map_err(|e| QuantError::Parse(format!("Failed to parse coins list: {}", e)))?;

        Ok(coins)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinGeckoCoin {
    pub id: String,
    pub symbol: String,
    pub name: String,
}

// ============================================================================
// API KEY MANAGEMENT & SMART ROUTING
// ============================================================================

/// Manages a pool of API keys with rotation and rate limit tracking
pub struct ApiKeyPool {
    keys: Vec<String>,
    current_index: AtomicUsize,
    rate_limited: RwLock<HashMap<String, Instant>>,
}

impl ApiKeyPool {
    pub fn new(keys: Vec<String>) -> Self {
        Self {
            keys,
            current_index: AtomicUsize::new(0),
            rate_limited: RwLock::new(HashMap::new()),
        }
    }

    /// Gets the next available API key, skipping rate-limited ones
    pub fn get_next_key(&self) -> Option<String> {
        if self.keys.is_empty() {
            return None;
        }

        let start_index = self.current_index.load(Ordering::Relaxed);
        let mut attempts = 0;

        loop {
            if attempts >= self.keys.len() {
                return None; // All keys are rate limited
            }

            let index = (start_index + attempts) % self.keys.len();
            let key = &self.keys[index];

            // Check if key is rate limited
            let is_limited = {
                let guard = self.rate_limited.read().unwrap();
                if let Some(expiry) = guard.get(key) {
                    if Instant::now() < *expiry {
                        true
                    } else {
                        false // Rate limit expired
                    }
                } else {
                    false
                }
            };

            if !is_limited {
                // If it was limited but expired, remove it from the map
                {
                    let mut guard = self.rate_limited.write().unwrap();
                    if let Some(expiry) = guard.get(key) {
                        if Instant::now() >= *expiry {
                            guard.remove(key);
                        }
                    }
                }

                // Update current index for round-robin
                self.current_index
                    .store((index + 1) % self.keys.len(), Ordering::Relaxed);
                return Some(key.clone());
            }

            attempts += 1;
        }
    }

    /// Marks a key as rate limited for a specific duration
    pub fn mark_rate_limited(&self, key: &str, duration: Duration) {
        let mut guard = self.rate_limited.write().unwrap();
        guard.insert(key.to_string(), Instant::now() + duration);
    }
}

/// Configuration for smart routing
#[derive(Clone)]
pub struct RoutingConfig {
    pub ohlc_priority: Vec<DataSource>,
    pub market_data_priority: Vec<DataSource>,
    pub max_retries: u32,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            ohlc_priority: vec![
                DataSource::Binance,
                DataSource::CoinGecko,
                DataSource::Coinlayer,
            ],
            market_data_priority: vec![DataSource::CoinGecko, DataSource::Binance],
            max_retries: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataSource {
    Binance,
    CoinGecko,
    Coinlayer,
}

/// Unified client that routes requests to the best available API
pub struct UnifiedDataClient {
    binance_keys: ApiKeyPool,
    coingecko_keys: ApiKeyPool,
    coinlayer_keys: ApiKeyPool,
    config: RoutingConfig,
}

impl UnifiedDataClient {
    /// Creates a new unified client from environment variables
    /// Supports both singular (BINANCE_API_KEY) and plural (BINANCE_API_KEYS) env vars
    pub fn new_from_env() -> Self {
        let binance_keys = Self::get_keys("BINANCE_API_KEY", "BINANCE_API_KEYS");
        let coingecko_keys = Self::get_keys("COINGECKO_API_KEY", "COINGECKO_API_KEYS");
        let coinlayer_keys = Self::get_keys("COINLAYER_API_KEY", "COINLAYER_API_KEYS");

        Self {
            binance_keys: ApiKeyPool::new(binance_keys),
            coingecko_keys: ApiKeyPool::new(coingecko_keys),
            coinlayer_keys: ApiKeyPool::new(coinlayer_keys),
            config: RoutingConfig::default(),
        }
    }

    fn get_keys(singular: &str, plural: &str) -> Vec<String> {
        let mut keys = Vec::new();

        // Try plural first
        if let Ok(val) = std::env::var(plural) {
            keys.extend(
                val.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()),
            );
        }

        // Try singular if empty
        if keys.is_empty() {
            if let Ok(val) = std::env::var(singular) {
                if !val.trim().is_empty() {
                    keys.push(val.trim().to_string());
                }
            }
        }

        keys
    }

    /// Fetches OHLC data using smart routing
    pub async fn get_bars(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Bar>> {
        let mut last_error = None;

        for source in &self.config.ohlc_priority {
            match source {
                DataSource::Binance => {
                    // Try to get a key
                    let key = self.binance_keys.get_next_key();
                    // Even if no key (public API), we can try
                    let client = BinanceClient::new(key.clone(), None); // Secret not needed for public endpoints

                    // Map symbol (e.g., "BTC" -> "BTCUSDT") - simplified logic
                    let pair = if !symbol.contains("USDT") {
                        format!("{}USDT", symbol.to_uppercase())
                    } else {
                        symbol.to_string()
                    };

                    match client.get_klines(&pair, interval, None, None, limit).await {
                        Ok(bars) => return Ok(bars),
                        Err(e) => {
                            // If rate limited, mark key
                            if e.to_string().contains("429") || e.to_string().contains("418") {
                                if let Some(k) = key {
                                    self.binance_keys
                                        .mark_rate_limited(&k, Duration::from_secs(60));
                                }
                            }
                            last_error = Some(e);
                        }
                    }
                }
                DataSource::CoinGecko => {
                    let key = self.coingecko_keys.get_next_key();
                    let client = CoinGeckoClient::new(key.clone());

                    // Map symbol to ID (simplified - in real app need a map)
                    let symbol_lower = symbol.to_lowercase();
                    let coin_id = match symbol_lower.as_str() {
                        "btc" | "btcusdt" => "bitcoin",
                        "eth" | "ethusdt" => "ethereum",
                        "sol" | "solusdt" => "solana",
                        s => s, // fallback
                    };

                    // Map interval to days
                    let days = match interval {
                        "1d" => 1,
                        "1h" => 1, // CoinGecko min is 1 day for free, or hourly for < 90 days
                        _ => 1,
                    };

                    match client.get_ohlc(coin_id, "usd", days).await {
                        Ok(bars) => return Ok(bars),
                        Err(e) => {
                            if e.to_string().contains("429") {
                                if let Some(k) = key {
                                    self.coingecko_keys
                                        .mark_rate_limited(&k, Duration::from_secs(60));
                                }
                            }
                            last_error = Some(e);
                        }
                    }
                }
                DataSource::Coinlayer => {
                    if let Some(key) = self.coinlayer_keys.get_next_key() {
                        let client = CoinlayerClient::new(key);

                        // Coinlayer only supports daily data via historical endpoint
                        // We need to fetch multiple days. For now, let's just fetch today's rate
                        // and return it as a single bar if interval is 1d.
                        // This is a limitation of Coinlayer.
                        // For true OHLC, we need get_historical_bars which makes multiple requests.

                        if interval == "1d" {
                            // Calculate start date based on limit
                            let days = limit.unwrap_or(1);
                            let start_date =
                                Utc::now().date_naive() - chrono::Duration::days(days as i64);

                            match client
                                .get_historical_bars(symbol, "USD", start_date, days)
                                .await
                            {
                                Ok(bars) => return Ok(bars),
                                Err(e) => last_error = Some(e),
                            }
                        } else {
                            // Coinlayer doesn't support intraday
                            continue;
                        }
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| QuantError::Api("All data sources failed".to_string())))
    }

    /// Fetches current price using smart routing
    pub async fn get_price(&self, symbol: &str) -> Result<f64> {
        let mut last_error = None;

        for source in &self.config.market_data_priority {
            match source {
                DataSource::CoinGecko => {
                    let key = self.coingecko_keys.get_next_key();
                    let client = CoinGeckoClient::new(key.clone());
                    let symbol_lower = symbol.to_lowercase();
                    let coin_id = match symbol_lower.as_str() {
                        "btc" | "btcusdt" => "bitcoin",
                        "eth" | "ethusdt" => "ethereum",
                        _ => symbol_lower.as_str(),
                    };

                    match client.get_price(coin_id, "usd").await {
                        Ok(price) => return Ok(price),
                        Err(e) => last_error = Some(e),
                    }
                }
                DataSource::Binance => {
                    let key = self.binance_keys.get_next_key();
                    let client = BinanceClient::new(key, None);
                    let pair = format!("{}USDT", symbol.to_uppercase());

                    match client.get_ticker_24hr(Some(&pair)).await {
                        Ok(Ticker24hrResponse::Single(ticker)) => {
                            if let Ok(price) = ticker.last_price.parse::<f64>() {
                                return Ok(price);
                            }
                        }
                        Err(e) => last_error = Some(e),
                        _ => {}
                    }
                }
                DataSource::Coinlayer => {
                    if let Some(key) = self.coinlayer_keys.get_next_key() {
                        let client = CoinlayerClient::new(key);
                        match client.get_live_rates(Some("USD"), Some(symbol)).await {
                            Ok(rates) => {
                                if let Some(rate) = rates.rates.get(symbol.to_uppercase().as_str())
                                {
                                    return Ok(*rate);
                                }
                            }
                            Err(e) => last_error = Some(e),
                        }
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| QuantError::Api("All data sources failed".to_string())))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = CoinlayerClient::new("test_key".to_string());
        assert_eq!(client.base_url, "http://api.coinlayer.com");
        assert_eq!(client.api_key, "test_key");
    }

    #[test]
    fn test_deserialize_live_response() {
        let json = r#"{
            "success": true,
            "terms": "https://coinlayer.com/terms",
            "privacy": "https://coinlayer.com/privacy",
            "timestamp": 1529571067,
            "target": "USD",
            "rates": {
                "BTC": 50000.0,
                "ETH": 3000.0
            }
        }"#;

        let response: LiveRatesResponse = serde_json::from_str(json).unwrap();
        assert!(response.success);
        assert_eq!(response.target, Some("USD".to_string()));
        assert_eq!(response.rates.get("BTC"), Some(&50000.0));
    }

    #[test]
    fn test_deserialize_error_response() {
        let json = r#"{
            "success": false,
            "error": {
                "code": 101,
                "type": "invalid_access_key",
                "info": "You have not supplied a valid API Access Key."
            }
        }"#;

        let response: LiveRatesResponse = serde_json::from_str(json).unwrap();
        assert!(!response.success);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, 101);
    }
}
