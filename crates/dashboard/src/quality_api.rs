//! Data Quality API endpoints for monitoring data integrity
//!
//! Provides endpoints to check data gaps, outliers, and freshness.

use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::AppState;

/// Response for data gap check
#[derive(Serialize)]
pub struct GapCheckResponse {
    pub success: bool,
    pub symbol: String,
    pub interval: String,
    pub gaps: Vec<GapInfo>,
    pub total_missing_bars: usize,
}

#[derive(Serialize)]
pub struct GapInfo {
    pub start: String,
    pub end: String,
    pub expected_bars: usize,
}

/// Response for outlier check
#[derive(Serialize)]
pub struct OutlierCheckResponse {
    pub success: bool,
    pub symbol: String,
    pub interval: String,
    pub outliers: Vec<OutlierInfo>,
    pub total_outliers: usize,
}

#[derive(Serialize)]
pub struct OutlierInfo {
    pub timestamp: String,
    pub previous_close: f64,
    pub current_open: f64,
    pub gap_percent: f64,
}

/// Response for data freshness check
#[derive(Serialize)]
pub struct FreshnessResponse {
    pub success: bool,
    pub symbols: Vec<SymbolFreshness>,
    pub stale_count: usize,
    pub healthy_count: usize,
}

#[derive(Serialize)]
pub struct SymbolFreshness {
    pub symbol: String,
    pub interval: String,
    pub last_bar: Option<String>,
    pub hours_since_update: Option<f64>,
    pub status: String, // "healthy", "stale", "critical"
}

/// Response for data quality summary
#[derive(Serialize)]
pub struct QualitySummaryResponse {
    pub success: bool,
    pub total_symbols: usize,
    pub total_bars: i64,
    pub symbols_with_gaps: usize,
    pub symbols_with_outliers: usize,
    pub stale_symbols: usize,
    pub health_score: f64, // 0.0 - 1.0
}

/// Check for data gaps in a symbol
pub async fn check_gaps(
    State(state): State<Arc<AppState>>,
    Path((symbol, interval)): Path<(String, String)>,
) -> Json<GapCheckResponse> {
    let db = match &state.db {
        Some(db) => db,
        None => {
            return Json(GapCheckResponse {
                success: false,
                symbol,
                interval,
                gaps: vec![],
                total_missing_bars: 0,
            })
        }
    };

    match db.check_data_gaps(&symbol, &interval).await {
        Ok(gaps) => {
            let total_missing: usize = gaps.iter().map(|g| g.expected_bars).sum();
            let gap_infos: Vec<GapInfo> = gaps
                .iter()
                .map(|g| GapInfo {
                    start: g.start.format("%Y-%m-%d %H:%M:%S").to_string(),
                    end: g.end.format("%Y-%m-%d %H:%M:%S").to_string(),
                    expected_bars: g.expected_bars,
                })
                .collect();

            Json(GapCheckResponse {
                success: true,
                symbol,
                interval,
                gaps: gap_infos,
                total_missing_bars: total_missing,
            })
        }
        Err(_) => Json(GapCheckResponse {
            success: false,
            symbol,
            interval,
            gaps: vec![],
            total_missing_bars: 0,
        }),
    }
}

/// Check for price outliers in a symbol
pub async fn check_outliers(
    State(state): State<Arc<AppState>>,
    Path((symbol, interval)): Path<(String, String)>,
) -> Json<OutlierCheckResponse> {
    let db = match &state.db {
        Some(db) => db,
        None => {
            return Json(OutlierCheckResponse {
                success: false,
                symbol,
                interval,
                outliers: vec![],
                total_outliers: 0,
            })
        }
    };

    // Default threshold: 5% gap
    match db.check_price_outliers(&symbol, &interval, 0.05).await {
        Ok(outliers) => {
            let outlier_infos: Vec<OutlierInfo> = outliers
                .iter()
                .map(|o| OutlierInfo {
                    timestamp: o.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    previous_close: o.previous_close,
                    current_open: o.current_open,
                    gap_percent: o.gap_percent,
                })
                .collect();

            let count = outlier_infos.len();
            Json(OutlierCheckResponse {
                success: true,
                symbol,
                interval,
                outliers: outlier_infos,
                total_outliers: count,
            })
        }
        Err(_) => Json(OutlierCheckResponse {
            success: false,
            symbol,
            interval,
            outliers: vec![],
            total_outliers: 0,
        }),
    }
}

/// Check data freshness for all symbols
pub async fn check_freshness(State(state): State<Arc<AppState>>) -> Json<FreshnessResponse> {
    let db = match &state.db {
        Some(db) => db,
        None => {
            return Json(FreshnessResponse {
                success: false,
                symbols: vec![],
                stale_count: 0,
                healthy_count: 0,
            })
        }
    };

    match db.list_symbols().await {
        Ok(symbols) => {
            let now = chrono::Utc::now();
            let mut freshness_list = Vec::new();
            let mut stale = 0;
            let mut healthy = 0;

            for sym in symbols {
                let hours_since = sym.last_bar.as_ref().and_then(|lb| {
                    chrono::NaiveDateTime::parse_from_str(lb, "%Y-%m-%d %H:%M")
                        .ok()
                        .map(|dt| {
                            let last = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                                dt,
                                chrono::Utc,
                            );
                            (now - last).num_minutes() as f64 / 60.0
                        })
                });

                let status = match hours_since {
                    Some(h) if h > 24.0 => {
                        stale += 1;
                        "critical".to_string()
                    }
                    Some(h) if h > 6.0 => {
                        stale += 1;
                        "stale".to_string()
                    }
                    Some(_) => {
                        healthy += 1;
                        "healthy".to_string()
                    }
                    None => {
                        stale += 1;
                        "unknown".to_string()
                    }
                };

                freshness_list.push(SymbolFreshness {
                    symbol: sym.symbol,
                    interval: sym.timeframe,
                    last_bar: sym.last_bar,
                    hours_since_update: hours_since,
                    status,
                });
            }

            Json(FreshnessResponse {
                success: true,
                symbols: freshness_list,
                stale_count: stale,
                healthy_count: healthy,
            })
        }
        Err(_) => Json(FreshnessResponse {
            success: false,
            symbols: vec![],
            stale_count: 0,
            healthy_count: 0,
        }),
    }
}

/// Get data quality summary
pub async fn get_quality_summary(
    State(state): State<Arc<AppState>>,
) -> Json<QualitySummaryResponse> {
    let db = match &state.db {
        Some(db) => db,
        None => {
            return Json(QualitySummaryResponse {
                success: false,
                total_symbols: 0,
                total_bars: 0,
                symbols_with_gaps: 0,
                symbols_with_outliers: 0,
                stale_symbols: 0,
                health_score: 0.0,
            })
        }
    };

    let symbols = match db.list_symbols().await {
        Ok(s) => s,
        Err(_) => {
            return Json(QualitySummaryResponse {
                success: false,
                total_symbols: 0,
                total_bars: 0,
                symbols_with_gaps: 0,
                symbols_with_outliers: 0,
                stale_symbols: 0,
                health_score: 0.0,
            })
        }
    };

    let total_symbols = symbols.len();
    let total_bars: i64 = symbols.iter().map(|s| s.bar_count).sum();
    let mut symbols_with_gaps = 0;
    let mut symbols_with_outliers = 0;
    let mut stale_symbols = 0;
    let now = chrono::Utc::now();

    for sym in &symbols {
        // Check gaps
        if let Ok(gaps) = db.check_data_gaps(&sym.symbol, &sym.timeframe).await {
            if !gaps.is_empty() {
                symbols_with_gaps += 1;
            }
        }

        // Check outliers
        if let Ok(outliers) = db
            .check_price_outliers(&sym.symbol, &sym.timeframe, 0.05)
            .await
        {
            if !outliers.is_empty() {
                symbols_with_outliers += 1;
            }
        }

        // Check freshness
        if let Some(last_bar) = &sym.last_bar {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(last_bar, "%Y-%m-%d %H:%M") {
                let last =
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc);
                if (now - last).num_hours() > 6 {
                    stale_symbols += 1;
                }
            }
        }
    }

    // Calculate health score
    let health_score = if total_symbols > 0 {
        let gap_penalty = symbols_with_gaps as f64 / total_symbols as f64 * 0.3;
        let outlier_penalty = symbols_with_outliers as f64 / total_symbols as f64 * 0.3;
        let stale_penalty = stale_symbols as f64 / total_symbols as f64 * 0.4;
        (1.0 - gap_penalty - outlier_penalty - stale_penalty).max(0.0)
    } else {
        0.0
    };

    Json(QualitySummaryResponse {
        success: true,
        total_symbols,
        total_bars,
        symbols_with_gaps,
        symbols_with_outliers,
        stale_symbols,
        health_score,
    })
}

/// Request for outlier check with threshold
#[derive(Deserialize)]
pub struct OutlierRequest {
    pub threshold_pct: Option<f64>,
}
