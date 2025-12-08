//! Sentiment API endpoints
//!
//! Provides endpoints for fetching and managing sentiment data.

use axum::{extract::Query, response::Json};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use alphafield_data::{FearGreedData, SentimentClient, SentimentIndicator};

/// Query parameters for sentiment history
#[derive(Debug, Deserialize)]
pub struct SentimentHistoryQuery {
    /// Number of days of history (default: 30)
    #[serde(default = "default_days")]
    pub days: u32,
}

fn default_days() -> u32 {
    30
}

/// Response for current sentiment
#[derive(Serialize)]
pub struct CurrentSentimentResponse {
    pub success: bool,
    pub data: Option<SentimentData>,
    pub error: Option<String>,
}

/// Response for sentiment history
#[derive(Serialize)]
pub struct SentimentHistoryResponse {
    pub success: bool,
    pub data: Vec<SentimentData>,
    pub stats: Option<SentimentStats>,
    pub error: Option<String>,
}

/// Simplified sentiment data for API response
#[derive(Serialize)]
pub struct SentimentData {
    pub timestamp: i64,
    pub value: u8,
    pub classification: String,
    pub is_fear: bool,
    pub is_greed: bool,
}

impl From<&FearGreedData> for SentimentData {
    fn from(data: &FearGreedData) -> Self {
        Self {
            timestamp: data.timestamp.timestamp(),
            value: data.value,
            classification: data.classification.to_string(),
            is_fear: data.classification.is_fear(),
            is_greed: data.classification.is_greed(),
        }
    }
}

/// Sentiment statistics over the period
#[derive(Serialize)]
pub struct SentimentStats {
    pub average: f64,
    pub min: u8,
    pub max: u8,
    pub current: u8,
    pub sma_7: Option<f64>,
    pub sma_14: Option<f64>,
    pub sma_30: Option<f64>,
    pub momentum_7: Option<f64>,
    pub days_in_fear: usize,
    pub days_in_greed: usize,
    pub days_neutral: usize,
}

/// GET /api/sentiment/current - Get current Fear & Greed Index
pub async fn get_current_sentiment() -> Json<CurrentSentimentResponse> {
    let client = SentimentClient::new();

    match client.get_current().await {
        Ok(data) => {
            info!(value = data.value, "Fetched current sentiment");
            Json(CurrentSentimentResponse {
                success: true,
                data: Some(SentimentData::from(&data)),
                error: None,
            })
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch current sentiment");
            Json(CurrentSentimentResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
            })
        }
    }
}

/// GET /api/sentiment/history - Get historical Fear & Greed data
pub async fn get_sentiment_history(
    Query(query): Query<SentimentHistoryQuery>,
) -> Json<SentimentHistoryResponse> {
    let client = SentimentClient::new();

    match client.get_history(query.days).await {
        Ok(data) => {
            info!(days = query.days, count = data.len(), "Fetched sentiment history");

            let indicator = SentimentIndicator::new(data.clone());
            let now = chrono::Utc::now();

            let stats = if !data.is_empty() {
                let values: Vec<u8> = data.iter().map(|d| d.value).collect();
                let sum: u64 = values.iter().map(|&v| v as u64).sum();
                let average = sum as f64 / values.len() as f64;
                let min = *values.iter().min().unwrap_or(&0);
                let max = *values.iter().max().unwrap_or(&0);
                let current = data.first().map(|d| d.value).unwrap_or(0);

                let days_in_fear = data.iter()
                    .filter(|d| d.classification.is_fear())
                    .count();
                let days_in_greed = data.iter()
                    .filter(|d| d.classification.is_greed())
                    .count();
                let days_neutral = data.len() - days_in_fear - days_in_greed;

                Some(SentimentStats {
                    average,
                    min,
                    max,
                    current,
                    sma_7: indicator.sma(now, 7),
                    sma_14: indicator.sma(now, 14),
                    sma_30: indicator.sma(now, 30),
                    momentum_7: indicator.momentum(now, 7),
                    days_in_fear,
                    days_in_greed,
                    days_neutral,
                })
            } else {
                None
            };

            let response_data: Vec<SentimentData> = data.iter().map(SentimentData::from).collect();

            Json(SentimentHistoryResponse {
                success: true,
                data: response_data,
                stats,
                error: None,
            })
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch sentiment history");
            Json(SentimentHistoryResponse {
                success: false,
                data: vec![],
                stats: None,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Response for sync operation
#[derive(Serialize)]
pub struct SyncResponse {
    pub success: bool,
    pub message: String,
    pub count: usize,
}

/// POST /api/sentiment/sync - Fetch and cache all historical sentiment data
pub async fn sync_sentiment_data() -> Json<SyncResponse> {
    let client = SentimentClient::new();

    match client.get_all_history().await {
        Ok(data) => {
            info!(count = data.len(), "Synced all sentiment history");
            Json(SyncResponse {
                success: true,
                message: format!("Synced {} days of sentiment data", data.len()),
                count: data.len(),
            })
        }
        Err(e) => {
            error!(error = %e, "Failed to sync sentiment data");
            Json(SyncResponse {
                success: false,
                message: e.to_string(),
                count: 0,
            })
        }
    }
}
