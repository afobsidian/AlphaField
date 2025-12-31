use alphafield_core::Bar;
use alphafield_data::{DatabaseClient, UnifiedDataClient};
use serde::Serialize;
use tracing::info;

/// Data fetching status info
#[derive(Serialize, Default, Clone, Debug)]
pub struct DataStatus {
    pub source: String, // "cache" or "api"
    pub bars_loaded: usize,
    pub bars_requested: u32,
    pub date_range_start: Option<String>,
    pub date_range_end: Option<String>,
    pub cached_after: bool, // Whether data was cached after fetch
}

/// Fetch data from cache first, then API if needed
pub async fn fetch_data_with_cache(
    symbol: String,
    interval: String,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> Result<(Vec<Bar>, DataStatus), String> {
    // Calculate expected bar count based on interval
    let total_hours = (end_time - start_time).num_hours() as usize;
    let expected_bars = match interval.as_str() {
        "1h" => total_hours,
        "4h" => total_hours / 4,
        "1d" => total_hours / 24,
        _ => total_hours, // Default to hourly
    };

    // Try database first with strict range check
    if let Ok(db) = DatabaseClient::new_from_env().await {
        if let Ok(cached_bars) = db
            .load_bars_range(&symbol, &interval, start_time, end_time)
            .await
        {
            if !cached_bars.is_empty() {
                let first_ts = cached_bars.first().unwrap().timestamp;
                let last_ts = cached_bars.last().unwrap().timestamp;

                // Check if cache has sufficient data (at least 80% of expected)
                let cache_coverage = cached_bars.len() as f64 / expected_bars.max(1) as f64;

                if cache_coverage >= 0.8 {
                    let status = DataStatus {
                        source: "cache".to_string(),
                        bars_loaded: cached_bars.len(),
                        bars_requested: expected_bars as u32,
                        date_range_start: Some(first_ts.to_rfc3339()),
                        date_range_end: Some(last_ts.to_rfc3339()),
                        cached_after: false,
                    };

                    info!(source = "cache", count = cached_bars.len(), expected = expected_bars, 
                          coverage = %format!("{:.1}%", cache_coverage * 100.0), 
                          "Loaded sufficient data from database cache");
                    return Ok((cached_bars, status));
                } else {
                    info!(source = "cache", count = cached_bars.len(), expected = expected_bars, 
                          coverage = %format!("{:.1}%", cache_coverage * 100.0),
                          "Cache has insufficient data, fetching from API");
                }
            }
        }
    }

    // Fallback to API
    info!(symbol = %symbol, interval = %interval, "Fetching data from API...");
    let client = UnifiedDataClient::new_from_env();

    let bars = client
        .get_bars(&symbol, &interval, Some(start_time), Some(end_time), None)
        .await
        .map_err(|e| e.to_string())?;

    info!(
        source = "api",
        count = bars.len(),
        expected = expected_bars,
        "Fetched data from API"
    );

    // Cache the data for future use
    let mut cached_after = false;
    if let Ok(db) = DatabaseClient::new_from_env().await {
        if db.save_bars(&symbol, &interval, &bars).await.is_ok() {
            cached_after = true;
            info!("Cached {} bars to database", bars.len());
        }
    }

    let status = DataStatus {
        source: "api".to_string(),
        bars_loaded: bars.len(),
        bars_requested: expected_bars as u32,
        date_range_start: bars.first().map(|b| b.timestamp.to_rfc3339()),
        date_range_end: bars.last().map(|b| b.timestamp.to_rfc3339()),
        cached_after,
    };

    Ok((bars, status))
}
