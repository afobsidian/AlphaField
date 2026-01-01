use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

use alphafield_core::Bar;
use alphafield_strategy::{
    indicators::{Ema, Indicator, Macd, Rsi, Sma, BollingerBands},
};

use crate::api::AppState;
use crate::services::data_service::fetch_data_with_cache;

#[derive(Debug, Deserialize)]
pub struct ChartRequest {
    pub symbol: String,
    pub interval: String,
    pub days: u32,
    #[serde(default)]
    pub indicators: Vec<IndicatorConfig>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum IndicatorConfig {
    #[serde(rename = "sma")]
    Sma { period: usize },
    #[serde(rename = "ema")]
    Ema { period: usize },
    #[serde(rename = "rsi")]
    Rsi { period: usize },
    #[serde(rename = "macd")]
    Macd {
        fast: usize,
        slow: usize,
        signal: usize,
    },
    #[serde(rename = "bb")]
    BollingerBands { period: usize, std_dev: f64 },
}

#[derive(Serialize)]
pub struct ChartResponse {
    pub symbol: String,
    pub interval: String,
    pub bars: Vec<OhlcvBar>,
    pub indicators: HashMap<String, IndicatorData>,
}

#[derive(Serialize)]
pub struct OhlcvBar {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum IndicatorData {
    #[serde(rename = "line")]
    Line { values: Vec<IndicatorPoint> },
    #[serde(rename = "oscillator")]
    Oscillator {
        values: Vec<IndicatorPoint>,
        upper_bound: Option<f64>,
        lower_bound: Option<f64>,
    },
    #[serde(rename = "bands")]
    Bands {
        upper: Vec<IndicatorPoint>,
        middle: Vec<IndicatorPoint>,
        lower: Vec<IndicatorPoint>,
    },
    #[serde(rename = "macd")]
    Macd {
        macd: Vec<IndicatorPoint>,
        signal: Vec<IndicatorPoint>,
        histogram: Vec<IndicatorPoint>,
    },
}

#[derive(Serialize)]
pub struct IndicatorPoint {
    pub timestamp: i64,
    pub value: f64,
}

/// Get OHLCV data with technical indicators for charting
pub async fn get_chart_data(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ChartRequest>,
) -> Json<ChartResponse> {
    info!(
        "Chart data request: {} {} {}d with {} indicators",
        req.symbol,
        req.interval,
        req.days,
        req.indicators.len()
    );

    // Calculate time range
    let end_time = chrono::Utc::now();
    let start_time = end_time - chrono::Duration::days(req.days as i64);

    // Fetch bars from cache/API
    let data_result = fetch_data_with_cache(
        req.symbol.clone(),
        req.interval.clone(),
        start_time,
        end_time,
    )
    .await;

    let bars = match data_result {
        Ok((bars, _status)) => bars,
        Err(e) => {
            error!("Failed to fetch chart data: {}", e);
            return Json(ChartResponse {
                symbol: req.symbol,
                interval: req.interval,
                bars: vec![],
                indicators: HashMap::new(),
            });
        }
    };

    // Convert bars to OHLCV format
    let ohlcv_bars: Vec<OhlcvBar> = bars
        .iter()
        .map(|b| OhlcvBar {
            timestamp: b.timestamp.timestamp(),
            open: b.open,
            high: b.high,
            low: b.low,
            close: b.close,
            volume: b.volume,
        })
        .collect();

    // Calculate indicators
    let mut indicators_data = HashMap::new();

    for (idx, indicator_config) in req.indicators.iter().enumerate() {
        let indicator_name = format!("indicator_{}", idx);
        
        match indicator_config {
            IndicatorConfig::Sma { period } => {
                let data = calculate_sma(&bars, *period);
                indicators_data.insert(indicator_name, data);
            }
            IndicatorConfig::Ema { period } => {
                let data = calculate_ema(&bars, *period);
                indicators_data.insert(indicator_name, data);
            }
            IndicatorConfig::Rsi { period } => {
                let data = calculate_rsi(&bars, *period);
                indicators_data.insert(indicator_name, data);
            }
            IndicatorConfig::Macd { fast, slow, signal } => {
                let data = calculate_macd(&bars, *fast, *slow, *signal);
                indicators_data.insert(indicator_name, data);
            }
            IndicatorConfig::BollingerBands { period, std_dev } => {
                let data = calculate_bollinger_bands(&bars, *period, *std_dev);
                indicators_data.insert(indicator_name, data);
            }
        }
    }

    Json(ChartResponse {
        symbol: req.symbol,
        interval: req.interval,
        bars: ohlcv_bars,
        indicators: indicators_data,
    })
}

fn calculate_sma(bars: &[Bar], period: usize) -> IndicatorData {
    let mut sma = Sma::new(period);
    let mut values = Vec::new();

    for bar in bars {
        sma.update(bar.close);
        if let Some(value) = sma.value() {
            values.push(IndicatorPoint {
                timestamp: bar.timestamp.timestamp(),
                value,
            });
        } else {
            // Push NaN for warmup period to maintain alignment
            values.push(IndicatorPoint {
                timestamp: bar.timestamp.timestamp(),
                value: f64::NAN,
            });
        }
    }

    IndicatorData::Line { values }
}

fn calculate_ema(bars: &[Bar], period: usize) -> IndicatorData {
    let mut ema = Ema::new(period);
    let mut values = Vec::new();

    for bar in bars {
        ema.update(bar.close);
        if let Some(value) = ema.value() {
            values.push(IndicatorPoint {
                timestamp: bar.timestamp.timestamp(),
                value,
            });
        } else {
            values.push(IndicatorPoint {
                timestamp: bar.timestamp.timestamp(),
                value: f64::NAN,
            });
        }
    }

    IndicatorData::Line { values }
}

fn calculate_rsi(bars: &[Bar], period: usize) -> IndicatorData {
    let mut rsi = Rsi::new(period);
    let mut values = Vec::new();

    for bar in bars {
        rsi.update(bar.close);
        if let Some(value) = rsi.value() {
            values.push(IndicatorPoint {
                timestamp: bar.timestamp.timestamp(),
                value,
            });
        } else {
            values.push(IndicatorPoint {
                timestamp: bar.timestamp.timestamp(),
                value: f64::NAN,
            });
        }
    }

    IndicatorData::Oscillator {
        values,
        upper_bound: Some(70.0),
        lower_bound: Some(30.0),
    }
}

fn calculate_macd(bars: &[Bar], fast: usize, slow: usize, signal: usize) -> IndicatorData {
    let mut macd_indicator = Macd::new(fast, slow, signal);
    let mut macd_values = Vec::new();
    let mut signal_values = Vec::new();
    let mut histogram_values = Vec::new();

    for bar in bars {
        let timestamp = bar.timestamp.timestamp();
        
        if let Some((macd_val, signal_val, histogram_val)) = macd_indicator.update(bar.close) {
            macd_values.push(IndicatorPoint {
                timestamp,
                value: macd_val,
            });
            signal_values.push(IndicatorPoint {
                timestamp,
                value: signal_val,
            });
            histogram_values.push(IndicatorPoint {
                timestamp,
                value: histogram_val,
            });
        } else {
            macd_values.push(IndicatorPoint {
                timestamp,
                value: f64::NAN,
            });
            signal_values.push(IndicatorPoint {
                timestamp,
                value: f64::NAN,
            });
            histogram_values.push(IndicatorPoint {
                timestamp,
                value: f64::NAN,
            });
        }
    }

    IndicatorData::Macd {
        macd: macd_values,
        signal: signal_values,
        histogram: histogram_values,
    }
}

fn calculate_bollinger_bands(bars: &[Bar], period: usize, std_dev: f64) -> IndicatorData {
    let mut bb = BollingerBands::new(period, std_dev);
    let mut upper_values = Vec::new();
    let mut middle_values = Vec::new();
    let mut lower_values = Vec::new();

    for bar in bars {
        let timestamp = bar.timestamp.timestamp();
        
        if let Some((upper, middle, lower)) = bb.update(bar.close) {
            upper_values.push(IndicatorPoint { timestamp, value: upper });
            middle_values.push(IndicatorPoint { timestamp, value: middle });
            lower_values.push(IndicatorPoint { timestamp, value: lower });
        } else {
            upper_values.push(IndicatorPoint { timestamp, value: f64::NAN });
            middle_values.push(IndicatorPoint { timestamp, value: f64::NAN });
            lower_values.push(IndicatorPoint { timestamp, value: f64::NAN });
        }
    }

    IndicatorData::Bands {
        upper: upper_values,
        middle: middle_values,
        lower: lower_values,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_indicator_config_deserialization() {
        let json = r#"{"type": "sma", "period": 50}"#;
        let config: IndicatorConfig = serde_json::from_str(json).unwrap();
        match config {
            IndicatorConfig::Sma { period } => assert_eq!(period, 50),
            _ => panic!("Wrong indicator type"),
        }
    }

    #[test]
    fn test_calculate_sma() {
        let bars = vec![
            Bar {
                timestamp: Utc::now(),
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 100.0,
                volume: 1000.0,
            },
            Bar {
                timestamp: Utc::now(),
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 110.0,
                volume: 1000.0,
            },
            Bar {
                timestamp: Utc::now(),
                open: 110.0,
                high: 120.0,
                low: 100.0,
                close: 120.0,
                volume: 1000.0,
            },
        ];

        let result = calculate_sma(&bars, 3);
        match result {
            IndicatorData::Line { values } => {
                assert_eq!(values.len(), 3);
                // First two values should be NaN (warmup)
                assert!(values[0].value.is_nan());
                assert!(values[1].value.is_nan());
                // Third value should be average of 100, 110, 120 = 110
                assert_eq!(values[2].value, 110.0);
            }
            _ => panic!("Wrong indicator data type"),
        }
    }

    #[test]
    fn test_calculate_rsi() {
        let bars = vec![
            Bar {
                timestamp: Utc::now(),
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 100.0,
                volume: 1000.0,
            },
            Bar {
                timestamp: Utc::now(),
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 105.0,
                volume: 1000.0,
            },
        ];

        let result = calculate_rsi(&bars, 14);
        match result {
            IndicatorData::Oscillator {
                values,
                upper_bound,
                lower_bound,
            } => {
                assert_eq!(values.len(), 2);
                assert_eq!(upper_bound, Some(70.0));
                assert_eq!(lower_bound, Some(30.0));
            }
            _ => panic!("Wrong indicator data type"),
        }
    }
}
