use alphafield_core::{Bar, QuantError, Result};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::PathBuf;

/// Storage for historical market data
pub struct HistoricalDataStorage {
    base_dir: PathBuf,
}

impl HistoricalDataStorage {
    /// Creates a new storage instance
    ///
    /// # Arguments
    /// * `base_dir` - Directory to store data files
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        let base_dir = base_dir.into();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).expect("Failed to create storage directory");
        }
        Self { base_dir }
    }

    /// Generates the file path for a given symbol and interval
    fn get_file_path(&self, symbol: &str, interval: &str) -> PathBuf {
        // Sanitize symbol (e.g. remove /)
        let safe_symbol = symbol.replace('/', "_");
        self.base_dir
            .join(format!("{}_{}.json", safe_symbol, interval))
    }

    /// Saves bars to a JSON file
    pub fn save_bars(&self, symbol: &str, interval: &str, bars: &[Bar]) -> Result<()> {
        let path = self.get_file_path(symbol, interval);
        let file = fs::File::create(path).map_err(QuantError::Io)?;
        serde_json::to_writer(file, bars)
            .map_err(|e| QuantError::Parse(format!("Serialization error: {}", e)))?;
        Ok(())
    }

    /// Loads bars from a JSON file
    pub fn load_bars(&self, symbol: &str, interval: &str) -> Result<Vec<Bar>> {
        let path = self.get_file_path(symbol, interval);
        if !path.exists() {
            return Err(QuantError::NotFound(format!(
                "Data file not found: {:?}",
                path
            )));
        }
        let file = fs::File::open(path).map_err(QuantError::Io)?;
        let bars: Vec<Bar> = serde_json::from_reader(file)
            .map_err(|e| QuantError::Parse(format!("Deserialization error: {}", e)))?;
        Ok(bars)
    }

    /// Loads bars filtered by date range
    pub fn load_bars_range(
        &self,
        symbol: &str,
        interval: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Bar>> {
        let bars = self.load_bars(symbol, interval)?;
        Ok(bars
            .into_iter()
            .filter(|b| b.timestamp >= start && b.timestamp <= end)
            .collect())
    }

    /// Checks if data exists for the given symbol and interval
    pub fn exists(&self, symbol: &str, interval: &str) -> bool {
        self.get_file_path(symbol, interval).exists()
    }
}
