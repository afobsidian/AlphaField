use crate::database::DatabaseClient;
use alphafield_core::{Bar, QuantError, Result};
use chrono::{Duration, Utc};

/// Simple gap-filler that forward-fills missing bars using previous close price.
/// This is intentionally conservative: volumes are set to 0 and OHLC all equal to last close.
pub struct GapFiller {
    client: DatabaseClient,
}

impl GapFiller {
    pub fn new(client: DatabaseClient) -> Self {
        Self { client }
    }

    /// Finds gaps for `symbol`/`timeframe` assuming `interval_seconds` between bars.
    /// Fills gaps by creating synthetic bars where open/high/low/close == previous close.
    pub async fn fill_gaps(&self, symbol: &str, timeframe: &str, interval_seconds: i64) -> Result<usize> {
        let bars = self.client.load_bars(symbol, timeframe).await?;

        if bars.is_empty() {
            return Ok(0);
        }

        let mut to_insert: Vec<Bar> = Vec::new();

        for window in bars.windows(2) {
            let prev = window[0];
            let next = window[1];
            let expected = prev.timestamp + Duration::seconds(interval_seconds);

            let mut cur = expected;
            while cur < next.timestamp {
                // Create synthetic bar forward-filling previous close
                let synthetic = Bar {
                    timestamp: cur,
                    open: prev.close,
                    high: prev.close,
                    low: prev.close,
                    close: prev.close,
                    volume: 0.0,
                };

                to_insert.push(synthetic);
                cur = cur + Duration::seconds(interval_seconds);
            }
        }

        if !to_insert.is_empty() {
            // Save in one batch
            self.client.save_bars(symbol, timeframe, &to_insert).await?;
        }

        Ok(to_insert.len())
    }
}
