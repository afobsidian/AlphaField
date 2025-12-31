//! Sentiment Data Client
//!
//! Provides access to cryptocurrency market sentiment data.
//! Primary source: Alternative.me Fear & Greed Index

use alphafield_core::{QuantError, Result};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

/// Sentiment data point from Fear & Greed Index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FearGreedData {
    /// Timestamp of the data point
    pub timestamp: DateTime<Utc>,
    /// Sentiment value (0-100)
    /// 0-24: Extreme Fear
    /// 25-44: Fear
    /// 45-55: Neutral
    /// 56-75: Greed
    /// 76-100: Extreme Greed
    pub value: u8,
    /// Human-readable classification
    pub classification: SentimentClassification,
}

/// Sentiment classification categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentimentClassification {
    ExtremeFear,
    Fear,
    Neutral,
    Greed,
    ExtremeGreed,
}

impl SentimentClassification {
    /// Parse from API string
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "extreme fear" => Self::ExtremeFear,
            "fear" => Self::Fear,
            "neutral" => Self::Neutral,
            "greed" => Self::Greed,
            "extreme greed" => Self::ExtremeGreed,
            _ => Self::Neutral,
        }
    }

    /// Get classification from value
    pub fn from_value(value: u8) -> Self {
        match value {
            0..=24 => Self::ExtremeFear,
            25..=44 => Self::Fear,
            45..=55 => Self::Neutral,
            56..=75 => Self::Greed,
            76..=100 => Self::ExtremeGreed,
            _ => Self::ExtremeGreed,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExtremeFear => "Extreme Fear",
            Self::Fear => "Fear",
            Self::Neutral => "Neutral",
            Self::Greed => "Greed",
            Self::ExtremeGreed => "Extreme Greed",
        }
    }

    /// Check if sentiment suggests buying opportunity
    pub fn is_fear(&self) -> bool {
        matches!(self, Self::ExtremeFear | Self::Fear)
    }

    /// Check if sentiment suggests selling opportunity
    pub fn is_greed(&self) -> bool {
        matches!(self, Self::Greed | Self::ExtremeGreed)
    }
}

impl std::fmt::Display for SentimentClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// API response from Alternative.me Fear & Greed Index
#[derive(Debug, Deserialize)]
struct FearGreedApiResponse {
    #[serde(default)]
    data: Vec<FearGreedApiData>,
    #[serde(default)]
    metadata: Option<FearGreedMetadata>,
}

#[derive(Debug, Deserialize)]
struct FearGreedApiData {
    value: String,
    value_classification: String,
    timestamp: String,
    #[serde(default)]
    #[allow(dead_code)]
    time_until_update: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FearGreedMetadata {
    #[serde(default)]
    error: Option<String>,
}

/// Client for fetching sentiment data
pub struct SentimentClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for SentimentClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SentimentClient {
    /// Create a new sentiment client
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(5)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: "https://api.alternative.me".to_string(),
        }
    }

    /// Fetch the current Fear & Greed Index value
    pub async fn get_current(&self) -> Result<FearGreedData> {
        let history = self.get_history(1).await?;
        history
            .into_iter()
            .next()
            .ok_or_else(|| QuantError::Api("No sentiment data returned".to_string()))
    }

    /// Fetch historical Fear & Greed Index data
    ///
    /// # Arguments
    /// * `days` - Number of days of history to fetch (0 = all available)
    pub async fn get_history(&self, days: u32) -> Result<Vec<FearGreedData>> {
        let url = format!("{}/fng/", self.base_url);

        debug!(days = days, "Fetching Fear & Greed history");

        let response = self
            .client
            .get(&url)
            .query(&[("limit", days.to_string()), ("format", "json".to_string())])
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(QuantError::Api(format!(
                "API returned status: {}",
                response.status()
            )));
        }

        let api_response: FearGreedApiResponse = response
            .json()
            .await
            .map_err(|e| QuantError::Parse(format!("Failed to parse response: {}", e)))?;

        if let Some(metadata) = &api_response.metadata {
            if let Some(error) = &metadata.error {
                return Err(QuantError::Api(format!("API error: {}", error)));
            }
        }

        let data: Vec<FearGreedData> = api_response
            .data
            .into_iter()
            .filter_map(|d| {
                let timestamp_secs: i64 = d.timestamp.parse().ok()?;
                let timestamp = Utc.timestamp_opt(timestamp_secs, 0).single()?;
                let value: u8 = d.value.parse().ok()?;

                Some(FearGreedData {
                    timestamp,
                    value,
                    classification: SentimentClassification::parse(&d.value_classification),
                })
            })
            .collect();

        info!(count = data.len(), "Fetched Fear & Greed data");
        Ok(data)
    }

    /// Fetch all available historical data (since Feb 2018)
    pub async fn get_all_history(&self) -> Result<Vec<FearGreedData>> {
        self.get_history(0).await
    }
}

/// Sentiment indicator for use in strategies
#[derive(Debug, Clone, Default)]
pub struct SentimentIndicator {
    /// Historical sentiment data, sorted by timestamp ascending
    data: Vec<FearGreedData>,
}

impl SentimentIndicator {
    /// Create a new sentiment indicator with historical data
    pub fn new(mut data: Vec<FearGreedData>) -> Self {
        // Sort by timestamp ascending
        data.sort_by_key(|d| d.timestamp);
        Self { data }
    }

    /// Check if indicator has data
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the number of data points
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Get all data
    pub fn data(&self) -> &[FearGreedData] {
        &self.data
    }

    /// Get sentiment value at or before the given timestamp
    pub fn value_at(&self, timestamp: DateTime<Utc>) -> Option<u8> {
        self.data
            .iter()
            .rev()
            .find(|d| d.timestamp <= timestamp)
            .map(|d| d.value)
    }

    /// Get sentiment classification at or before the given timestamp
    pub fn classification_at(&self, timestamp: DateTime<Utc>) -> Option<SentimentClassification> {
        self.data
            .iter()
            .rev()
            .find(|d| d.timestamp <= timestamp)
            .map(|d| d.classification)
    }

    /// Calculate simple moving average of sentiment values
    pub fn sma(&self, timestamp: DateTime<Utc>, period: usize) -> Option<f64> {
        let relevant: Vec<_> = self
            .data
            .iter()
            .filter(|d| d.timestamp <= timestamp)
            .collect();

        if relevant.len() < period {
            return None;
        }

        let values: Vec<f64> = relevant
            .iter()
            .rev()
            .take(period)
            .map(|d| d.value as f64)
            .collect();

        Some(values.iter().sum::<f64>() / values.len() as f64)
    }

    /// Check if current sentiment is extreme fear
    pub fn is_extreme_fear(&self, timestamp: DateTime<Utc>) -> bool {
        self.classification_at(timestamp) == Some(SentimentClassification::ExtremeFear)
    }

    /// Check if current sentiment is extreme greed
    pub fn is_extreme_greed(&self, timestamp: DateTime<Utc>) -> bool {
        self.classification_at(timestamp) == Some(SentimentClassification::ExtremeGreed)
    }

    /// Check if sentiment is in fear zone (extreme fear or fear)
    pub fn is_fearful(&self, timestamp: DateTime<Utc>) -> bool {
        self.classification_at(timestamp)
            .map(|c| c.is_fear())
            .unwrap_or(false)
    }

    /// Check if sentiment is in greed zone (greed or extreme greed)
    pub fn is_greedy(&self, timestamp: DateTime<Utc>) -> bool {
        self.classification_at(timestamp)
            .map(|c| c.is_greed())
            .unwrap_or(false)
    }

    /// Detect sentiment divergence from price action
    /// Returns positive value if price up but sentiment down (bullish divergence)
    /// Returns negative value if price down but sentiment up (bearish divergence)
    pub fn divergence(
        &self,
        timestamp: DateTime<Utc>,
        price_change_pct: f64,
        lookback_days: usize,
    ) -> Option<f64> {
        let current = self.value_at(timestamp)?;

        // Get value from lookback_days ago
        let past_timestamp = timestamp - chrono::Duration::days(lookback_days as i64);
        let past = self.value_at(past_timestamp)?;

        let sentiment_change = current as f64 - past as f64;

        // Divergence: price and sentiment moving in opposite directions
        if price_change_pct > 0.0 && sentiment_change < 0.0 {
            // Price up, sentiment down = bullish divergence
            Some(price_change_pct - sentiment_change / 100.0)
        } else if price_change_pct < 0.0 && sentiment_change > 0.0 {
            // Price down, sentiment up = bearish divergence
            Some(price_change_pct - sentiment_change / 100.0)
        } else {
            Some(0.0) // No divergence
        }
    }

    /// Get sentiment momentum (rate of change over period)
    pub fn momentum(&self, timestamp: DateTime<Utc>, period: usize) -> Option<f64> {
        let current = self.value_at(timestamp)? as f64;
        let past_timestamp = timestamp - chrono::Duration::days(period as i64);
        let past = self.value_at(past_timestamp)? as f64;

        Some(current - past)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_from_value() {
        assert_eq!(
            SentimentClassification::from_value(10),
            SentimentClassification::ExtremeFear
        );
        assert_eq!(
            SentimentClassification::from_value(30),
            SentimentClassification::Fear
        );
        assert_eq!(
            SentimentClassification::from_value(50),
            SentimentClassification::Neutral
        );
        assert_eq!(
            SentimentClassification::from_value(65),
            SentimentClassification::Greed
        );
        assert_eq!(
            SentimentClassification::from_value(85),
            SentimentClassification::ExtremeGreed
        );
    }

    #[test]
    fn test_classification_is_fear() {
        assert!(SentimentClassification::ExtremeFear.is_fear());
        assert!(SentimentClassification::Fear.is_fear());
        assert!(!SentimentClassification::Neutral.is_fear());
        assert!(!SentimentClassification::Greed.is_fear());
    }

    #[test]
    fn test_sentiment_indicator_sma() {
        let data = vec![
            FearGreedData {
                timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
                value: 20,
                classification: SentimentClassification::ExtremeFear,
            },
            FearGreedData {
                timestamp: Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(),
                value: 30,
                classification: SentimentClassification::Fear,
            },
            FearGreedData {
                timestamp: Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(),
                value: 50,
                classification: SentimentClassification::Neutral,
            },
        ];

        let indicator = SentimentIndicator::new(data);
        let ts = Utc.with_ymd_and_hms(2024, 1, 3, 12, 0, 0).unwrap();

        let sma3 = indicator.sma(ts, 3).unwrap();
        assert!((sma3 - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_sentiment_indicator_value_at() {
        let data = vec![
            FearGreedData {
                timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
                value: 25,
                classification: SentimentClassification::Fear,
            },
            FearGreedData {
                timestamp: Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(),
                value: 75,
                classification: SentimentClassification::Greed,
            },
        ];

        let indicator = SentimentIndicator::new(data);

        // Query between two data points should return earlier value
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        assert_eq!(indicator.value_at(ts), Some(25));

        // Query after last data point should return last value
        let ts = Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap();
        assert_eq!(indicator.value_at(ts), Some(75));
    }
}
