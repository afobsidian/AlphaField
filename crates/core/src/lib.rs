//! # AlphaField Core
//!
//! Core data structures and traits for quantitative finance applications.
//! This crate provides the foundational types used across the entire AlphaField system.
//!
//! ## Performance Characteristics
//! - All price data uses `f64` for numerical precision
//! - Time data uses `chrono::DateTime<Utc>` for accurate timestamp handling
//! - Core structs are `Copy` where possible for zero-cost stack allocation
//! - No heap allocations in hot paths

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Custom error type for quantitative finance operations
#[derive(Debug, thiserror::Error)]
pub enum QuantError {
    /// IO-related errors (file reading, network, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Data parsing errors
    #[error("Parse error: {0}")]
    Parse(String),

    /// Data validation errors
    #[error("Validation error: {0}")]
    DataValidation(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// API-related errors
    #[error("API error: {0}")]
    Api(String),
}

/// Type alias for Results using QuantError
pub type Result<T> = std::result::Result<T, QuantError>;

// ============================================================================
// CORE DATA STRUCTURES
// ============================================================================

/// OHLCV candlestick bar representing aggregated price data over a time period
///
/// # Performance Notes
/// - This struct is `Copy` for zero-cost passing by value
/// - Stack-allocated (no heap allocations)
/// - Uses `f64` for all price/volume data for numerical precision
///
/// # Example
/// ```
/// use alphafield_core::Bar;
/// use chrono::Utc;
///
/// let bar = Bar {
///     timestamp: Utc::now(),
///     open: 50000.0,
///     high: 51000.0,
///     low: 49500.0,
///     close: 50500.0,
///     volume: 1250.5,
/// };
///
/// assert!(bar.validate().is_ok());
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    /// Timestamp of the bar (typically the opening time)
    pub timestamp: DateTime<Utc>,

    /// Opening price
    pub open: f64,

    /// Highest price during the period
    pub high: f64,

    /// Lowest price during the period
    pub low: f64,

    /// Closing price
    pub close: f64,

    /// Trading volume during the period
    pub volume: f64,
}

impl Bar {
    /// Validates the bar data for logical consistency
    ///
    /// # Validation Rules
    /// - High must be >= Low
    /// - High must be >= Open and Close
    /// - Low must be <= Open and Close
    /// - All prices must be positive
    /// - Volume must be non-negative
    ///
    /// # Returns
    /// `Ok(())` if valid, `Err(QuantError::DataValidation)` otherwise
    pub fn validate(&self) -> Result<()> {
        if self.high < self.low {
            return Err(QuantError::DataValidation(format!(
                "High ({}) cannot be less than Low ({})",
                self.high, self.low
            )));
        }

        if self.high < self.open || self.high < self.close {
            return Err(QuantError::DataValidation(format!(
                "High ({}) must be >= Open ({}) and Close ({})",
                self.high, self.open, self.close
            )));
        }

        if self.low > self.open || self.low > self.close {
            return Err(QuantError::DataValidation(format!(
                "Low ({}) must be <= Open ({}) and Close ({})",
                self.low, self.open, self.close
            )));
        }

        if self.open <= 0.0 || self.high <= 0.0 || self.low <= 0.0 || self.close <= 0.0 {
            return Err(QuantError::DataValidation(
                "All prices must be positive".to_string(),
            ));
        }

        if self.volume < 0.0 {
            return Err(QuantError::DataValidation(
                "Volume cannot be negative".to_string(),
            ));
        }

        Ok(())
    }

    /// Returns the typical price: (High + Low + Close) / 3
    #[inline]
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    /// Returns the price range: High - Low
    #[inline]
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Returns true if the bar is bullish (close > open)
    #[inline]
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Returns the body size: |Close - Open|
    #[inline]
    pub fn body_size(&self) -> f64 {
        (self.close - self.open).abs()
    }
}

impl fmt::Display for Bar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bar[{} O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{:.2}]",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume
        )
    }
}

// ============================================================================
// TICK DATA
// ============================================================================

/// Individual trade tick representing a single executed trade
///
/// # Performance Notes
/// - `Copy` trait for efficient passing
/// - Minimal memory footprint
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tick {
    /// Timestamp of the trade execution
    pub timestamp: DateTime<Utc>,

    /// Execution price
    pub price: f64,

    /// Trade quantity/size
    pub quantity: f64,

    /// True if the buyer was the maker (sell-side aggressor)
    pub is_buyer_maker: bool,
}

impl Tick {
    /// Validates the tick data
    pub fn validate(&self) -> Result<()> {
        if self.price <= 0.0 {
            return Err(QuantError::DataValidation(
                "Price must be positive".to_string(),
            ));
        }

        if self.quantity <= 0.0 {
            return Err(QuantError::DataValidation(
                "Quantity must be positive".to_string(),
            ));
        }

        Ok(())
    }

    /// Returns the trade direction as a string
    pub fn direction(&self) -> &str {
        if self.is_buyer_maker {
            "SELL"
        } else {
            "BUY"
        }
    }
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tick[{} {} {:.2} @ {:.2}]",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.direction(),
            self.quantity,
            self.price
        )
    }
}

// ============================================================================
// QUOTE DATA
// ============================================================================

/// Bid/Ask quote snapshot representing the order book top-of-book
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Quote {
    /// Timestamp of the quote
    pub timestamp: DateTime<Utc>,

    /// Best bid price
    pub bid_price: f64,

    /// Bid size/quantity
    pub bid_size: f64,

    /// Best ask price
    pub ask_price: f64,

    /// Ask size/quantity
    pub ask_size: f64,
}

impl Quote {
    /// Validates the quote data
    pub fn validate(&self) -> Result<()> {
        if self.bid_price <= 0.0 || self.ask_price <= 0.0 {
            return Err(QuantError::DataValidation(
                "Prices must be positive".to_string(),
            ));
        }

        if self.bid_size < 0.0 || self.ask_size < 0.0 {
            return Err(QuantError::DataValidation(
                "Sizes cannot be negative".to_string(),
            ));
        }

        if self.bid_price >= self.ask_price {
            return Err(QuantError::DataValidation(format!(
                "Bid ({}) must be less than Ask ({})",
                self.bid_price, self.ask_price
            )));
        }

        Ok(())
    }

    /// Returns the bid-ask spread
    #[inline]
    pub fn spread(&self) -> f64 {
        self.ask_price - self.bid_price
    }

    /// Returns the mid price: (Bid + Ask) / 2
    #[inline]
    pub fn mid_price(&self) -> f64 {
        (self.bid_price + self.ask_price) / 2.0
    }

    /// Returns the spread as a percentage of mid price
    #[inline]
    pub fn spread_bps(&self) -> f64 {
        (self.spread() / self.mid_price()) * 10000.0
    }
}

impl fmt::Display for Quote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Quote[{} Bid:{:.2}x{:.2} Ask:{:.2}x{:.2} Spread:{:.4}]",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.bid_price,
            self.bid_size,
            self.ask_price,
            self.ask_size,
            self.spread()
        )
    }
}

// ============================================================================
// FINANCIAL INSTRUMENT TRAIT
// ============================================================================

/// Trait that all financial instrument data types must implement
///
/// This enables polymorphic handling of different market data types
/// (bars, ticks, quotes) through a common interface.
pub trait FinancialInstrument {
    /// Returns the instrument's symbol/identifier
    fn symbol(&self) -> &str;

    /// Returns the timestamp of this data point
    fn timestamp(&self) -> DateTime<Utc>;

    /// Returns a representative price for this instrument
    /// - For bars: typically the close price
    /// - For ticks: the trade price
    /// - For quotes: the mid price
    fn price(&self) -> f64;

    /// Validates the data integrity
    fn validate(&self) -> Result<()>;
}

// ============================================================================
// STRATEGY & SIGNALS
// ============================================================================

/// Type of trading signal
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SignalType {
    /// Signal to buy/long
    Buy,
    /// Signal to sell/short
    Sell,
    /// Signal to hold/neutral (or close position)
    Hold,
}

/// A trading signal generated by a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    /// Time the signal was generated
    pub timestamp: DateTime<Utc>,

    /// Symbol the signal is for
    pub symbol: String,

    /// Type of signal (Buy/Sell/Hold)
    pub signal_type: SignalType,

    /// Strength/Confidence of the signal (0.0 to 1.0)
    pub strength: f64,

    /// Optional metadata or reason for the signal
    pub metadata: Option<String>,
}

/// Trait that all trading strategies must implement
pub trait Strategy {
    /// Returns the name of the strategy
    fn name(&self) -> &str;

    /// Process a new bar and potentially return a signal
    fn on_bar(&mut self, bar: &Bar) -> Option<Signal>;

    /// Process a new tick (optional)
    fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
        None
    }

    /// Process a new quote (optional)
    fn on_quote(&mut self, _quote: &Quote) -> Option<Signal> {
        None
    }
}

// ============================================================================
// EXECUTION & ORDERS
// ============================================================================

/// Side of an order (Buy/Sell)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Type of order (Market/Limit)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
}

/// Current status of an order
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
}

/// Represents an order to be placed or tracked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Unique identifier for the order
    pub id: String,

    /// Symbol to trade (e.g., "BTCUSDT")
    pub symbol: String,

    /// Side (Buy/Sell)
    pub side: OrderSide,

    /// Type (Market/Limit)
    pub order_type: OrderType,

    /// Quantity to trade
    pub quantity: f64,

    /// Limit price (None for Market orders)
    pub price: Option<f64>,

    /// Current status
    pub status: OrderStatus,

    /// Creation timestamp
    pub timestamp: DateTime<Utc>,
}

/// Represents a trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Unique trade identifier
    pub id: String,

    /// ID of the order this trade belongs to
    pub order_id: String,

    /// Symbol traded
    pub symbol: String,

    /// Side (Buy/Sell)
    pub side: OrderSide,

    /// Quantity executed
    pub quantity: f64,

    /// Execution price
    pub price: f64,

    /// Commission/Fee paid
    pub fee: f64,

    /// Asset the fee was paid in
    pub fee_asset: String,

    /// Execution timestamp
    pub timestamp: DateTime<Utc>,
}

/// Trait for execution services (exchanges, paper trading)
#[async_trait::async_trait]
pub trait ExecutionService: Send + Sync {
    /// Submit a new order
    async fn submit_order(&self, order: &Order) -> Result<String>;

    /// Cancel an existing order
    async fn cancel_order(&self, order_id: &str, symbol: &str) -> Result<()>;

    /// Get current order status
    async fn get_order(&self, order_id: &str, symbol: &str) -> Result<Order>;
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_valid_bar() {
        let bar = Bar {
            timestamp: Utc::now(),
            open: 50000.0,
            high: 51000.0,
            low: 49500.0,
            close: 50500.0,
            volume: 1250.5,
        };

        assert!(bar.validate().is_ok());
        assert!(bar.is_bullish());
        assert_eq!(bar.range(), 1500.0);
    }

    #[test]
    fn test_invalid_bar_high_low() {
        let bar = Bar {
            timestamp: Utc::now(),
            open: 50000.0,
            high: 49000.0, // Invalid: high < low
            low: 49500.0,
            close: 50500.0,
            volume: 1250.5,
        };

        assert!(bar.validate().is_err());
    }

    #[test]
    fn test_valid_tick() {
        let tick = Tick {
            timestamp: Utc::now(),
            price: 50000.0,
            quantity: 1.5,
            is_buyer_maker: false,
        };

        assert!(tick.validate().is_ok());
        assert_eq!(tick.direction(), "BUY");
    }

    #[test]
    fn test_valid_quote() {
        let quote = Quote {
            timestamp: Utc::now(),
            bid_price: 49999.0,
            bid_size: 2.5,
            ask_price: 50001.0,
            ask_size: 3.0,
        };

        assert!(quote.validate().is_ok());
        assert_eq!(quote.spread(), 2.0);
        assert_eq!(quote.mid_price(), 50000.0);
    }

    #[test]
    fn test_invalid_quote_crossed() {
        let quote = Quote {
            timestamp: Utc::now(),
            bid_price: 50001.0, // Invalid: bid >= ask
            bid_size: 2.5,
            ask_price: 50000.0,
            ask_size: 3.0,
        };

        assert!(quote.validate().is_err());
    }
}
