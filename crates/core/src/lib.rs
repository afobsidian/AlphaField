//! # AlphaField Core
//!
//! Core data structures and traits for quantitative finance applications.
//! This crate provides foundational types used across the entire AlphaField system.
//!
//! ## Performance Characteristics
//! - All price data uses `f64` for numerical precision
//! - Time data uses `chrono::DateTime<Utc>` for accurate timestamp handling
//! - Core structs are `Copy` where possible for zero-cost stack allocation
//! - No heap allocations in hot paths

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

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

/// Trading mode for the strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TradingMode {
    /// Spot trading only (no short positions)
    #[default]
    Spot,
    /// Margin trading (allows short positions)
    Margin,
}

/// Represents the current position state of a strategy
///
/// This enum is used by strategies to track whether they are currently
/// in a long position, short position, or flat (no position).
///
/// # Usage
/// Strategies should update this state as they generate signals:
/// - `Flat` → `Long`: After generating Buy signal to enter long
/// - `Long` → `Flat`: After generating Sell signal to exit long
/// - `Flat` → `Short`: After generating Sell signal to enter short
/// - `Short` → `Flat`: After generating Buy signal to exit short
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PositionState {
    /// No open position (flat)
    #[default]
    Flat,
    /// Currently in a long position
    Long,
    /// Currently in a short position
    Short,
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
///
/// Strategies generate trading signals based on market data analysis.
/// Each strategy tracks its position state and generates appropriate signals:
///
/// # Signal Semantics
/// - **Buy signal**: Enter a long position OR exit a short position
/// - **Sell signal**: Enter a short position OR exit a long position  
/// - **Hold signal**: No action (maintain current position)
///
/// The StrategyAdapter (in the backtest crate) interprets these signals based on TradingMode:
/// - **Spot mode**: Only long positions allowed
///   - Buy when flat → Opens long position
///   - Sell when long → Closes long position, goes flat
/// - **Margin mode**: Both long and short positions allowed
///   - Buy when flat → Opens long position
///   - Buy when short → Closes short position, goes flat
///   - Sell when flat → Opens short position
///   - Sell when long → Closes long position, goes flat
///
/// # Implementation Notes
/// Strategies should track their position state using the `PositionState` enum
/// and update it as signals are generated. The `reset()` method must restore
/// the strategy to its initial state (position = Flat).
pub trait Strategy: Send + Sync {
    /// Returns the name of the strategy
    fn name(&self) -> &str;

    /// Process a new bar and potentially return signals
    ///
    /// This is the primary method for bar-based strategies. It should:
    /// 1. Update any indicators with the new bar data
    /// 2. Check for exit conditions if currently in a position
    /// 3. Check for entry conditions if currently flat
    /// 4. Update position state when generating signals
    /// 5. Return the generated signals (if any)
    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>>;

    /// Process a new tick (optional)
    ///
    /// Override this for tick-based strategies that need real-time updates.
    fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
        None
    }

    /// Process a new quote (optional)
    ///
    /// Override this for quote-based strategies (e.g., order book imbalance).
    fn on_quote(&mut self, _quote: &Quote) -> Option<Signal> {
        None
    }

    /// Reset the strategy to its initial state
    ///
    /// This method should:
    /// - Reset position state to `PositionState::Flat`
    /// - Clear any entry prices or tracking variables
    /// - Reset indicators to their initial state
    /// - Clear any cached data
    ///
    /// It is called between backtest runs and during testing.
    ///
    /// # Default Implementation
    /// The default implementation does nothing. Strategies that maintain state
    /// should override this method to properly reset their internal state.
    fn reset(&mut self) {
        // Default: no-op. Strategies with state should override.
    }
}

/// Blanket implementation for boxed strategies to enable dynamic dispatch
impl<T: Strategy + ?Sized> Strategy for Box<T> {
    fn name(&self) -> &str {
        (**self).name()
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        (**self).on_bar(bar)
    }

    fn on_tick(&mut self, tick: &Tick) -> Option<Signal> {
        (**self).on_tick(tick)
    }

    fn on_quote(&mut self, quote: &Quote) -> Option<Signal> {
        (**self).on_quote(quote)
    }

    fn reset(&mut self) {
        (**self).reset();
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
    /// Part of an OCO group that's waiting to be triggered
    OcoPending,
    /// Canceled as part of OCO (other order filled)
    OcoCanceled,
    /// Iceberg order - visible portion filled, hidden portion pending
    IcebergPending,
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

    /// OCO group ID (if part of a One-Cancels-Other order)
    #[serde(default)]
    pub oco_group_id: Option<String>,

    /// Iceberg hidden quantity remaining (for iceberg orders)
    #[serde(default)]
    pub iceberg_hidden_qty: Option<f64>,

    /// Stop-loss price (for bracket orders)
    #[serde(default)]
    pub stop_loss: Option<f64>,

    /// Take-profit price (for bracket orders)
    #[serde(default)]
    pub take_profit: Option<f64>,

    /// Parent order ID (for child orders in bracket/OCO)
    #[serde(default)]
    pub parent_order_id: Option<String>,

    /// Limit chase adjustment amount (for limit chase orders)
    #[serde(default)]
    pub limit_chase_amount: Option<f64>,
}

/// OCO (One-Cancels-Other) order group
/// When one order fills, all other orders in the group are automatically canceled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcoOrder {
    /// Unique group identifier
    pub group_id: String,

    /// Orders in this OCO group
    pub orders: Vec<Order>,

    /// Creation timestamp
    pub timestamp: DateTime<Utc>,

    /// Whether the group is still active
    pub active: bool,

    /// ID of the order that filled (causing cancellation of others)
    #[serde(default)]
    pub filled_order_id: Option<String>,
}

impl OcoOrder {
    pub fn new(orders: Vec<Order>) -> Self {
        Self {
            group_id: uuid::Uuid::new_v4().to_string(),
            orders,
            timestamp: Utc::now(),
            active: true,
            filled_order_id: None,
        }
    }
}

/// Bracket order consisting of an entry order with attached stop-loss and take-profit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BracketOrder {
    /// Unique bracket identifier
    pub bracket_id: String,

    /// Entry order (market or limit)
    pub entry_order: Order,

    /// Stop-loss order
    pub stop_loss_order: Order,

    /// Take-profit order
    pub take_profit_order: Order,

    /// Creation timestamp
    pub timestamp: DateTime<Utc>,

    /// Current state of the bracket
    pub state: BracketState,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BracketState {
    /// Entry order pending
    EntryPending,
    /// Entry filled, SL/TP active
    Active,
    /// Stop-loss filled
    StopLossFilled,
    /// Take-profit filled
    TakeProfitFilled,
    /// Canceled
    Canceled,
}

impl BracketOrder {
    pub fn new(entry_order: Order, stop_loss_order: Order, take_profit_order: Order) -> Self {
        let bracket_id = uuid::Uuid::new_v4().to_string();

        // Mark child orders with parent ID
        let mut sl_order = stop_loss_order;
        sl_order.parent_order_id = Some(bracket_id.clone());

        let mut tp_order = take_profit_order;
        tp_order.parent_order_id = Some(bracket_id.clone());

        Self {
            bracket_id,
            entry_order,
            stop_loss_order: sl_order,
            take_profit_order: tp_order,
            timestamp: Utc::now(),
            state: BracketState::EntryPending,
        }
    }
}

/// Iceberg order - splits a large order into smaller visible portions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcebergOrder {
    /// Unique iceberg identifier
    pub iceberg_id: String,

    /// Visible display quantity (per slice)
    pub visible_quantity: f64,

    /// Total remaining quantity (visible + hidden)
    pub total_quantity: f64,

    /// Symbol to trade
    pub symbol: String,

    /// Side (Buy/Sell)
    pub side: OrderSide,

    /// Limit price
    pub price: f64,

    /// Creation timestamp
    pub timestamp: DateTime<Utc>,

    /// Whether the iceberg order is still active
    pub active: bool,

    /// List of filled slice quantities
    #[serde(default)]
    pub filled_slices: Vec<(DateTime<Utc>, f64)>,
}

impl IcebergOrder {
    pub fn new(
        symbol: String,
        side: OrderSide,
        price: f64,
        total_quantity: f64,
        visible_quantity: f64,
    ) -> Self {
        Self {
            iceberg_id: Uuid::new_v4().to_string(),
            visible_quantity,
            total_quantity,
            symbol,
            side,
            price,
            timestamp: Utc::now(),
            active: true,
            filled_slices: Vec::new(),
        }
    }

    /// Calculate remaining hidden quantity
    pub fn hidden_quantity(&self) -> f64 {
        let filled_total: f64 = self.filled_slices.iter().map(|(_, qty)| qty).sum();
        let remaining = self.total_quantity - filled_total;
        (remaining - self.visible_quantity).max(0.0)
    }

    /// Check if more slices are needed
    pub fn needs_more_slices(&self) -> bool {
        self.hidden_quantity() > 0.0
    }
}

/// Limit chase order - automatically adjusts limit price if price moves away
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitChaseOrder {
    /// Unique identifier
    pub chase_id: String,

    /// Original order
    pub order: Order,

    /// Maximum chase amount (price units or percentage)
    pub chase_amount: f64,

    /// Whether chase amount is percentage (true) or absolute (false)
    pub is_percentage: bool,

    /// Number of price adjustments made
    pub adjustments: usize,

    /// Maximum number of adjustments allowed
    pub max_adjustments: usize,

    /// Creation timestamp
    pub timestamp: DateTime<Utc>,

    /// Whether the chase order is still active
    pub active: bool,
}

impl LimitChaseOrder {
    pub fn new(
        order: Order,
        chase_amount: f64,
        is_percentage: bool,
        max_adjustments: usize,
    ) -> Self {
        Self {
            chase_id: Uuid::new_v4().to_string(),
            order,
            chase_amount,
            is_percentage,
            adjustments: 0,
            max_adjustments,
            timestamp: Utc::now(),
            active: true,
        }
    }

    /// Calculate new limit price based on current market price
    pub fn calculate_new_limit(&self, current_price: f64) -> Option<f64> {
        if !self.active || self.adjustments >= self.max_adjustments {
            return None;
        }

        let current_limit = self.order.price?;

        let new_limit = if self.is_percentage {
            if self.order.side == OrderSide::Buy && current_price > current_limit {
                current_limit * (1.0 + self.chase_amount / 100.0)
            } else if self.order.side == OrderSide::Sell && current_price < current_limit {
                current_limit * (1.0 - self.chase_amount / 100.0)
            } else {
                return None;
            }
        } else if self.order.side == OrderSide::Buy && current_price > current_limit {
            current_limit + self.chase_amount
        } else if self.order.side == OrderSide::Sell && current_price < current_limit {
            current_limit - self.chase_amount
        } else {
            return None;
        };

        Some(new_limit)
    }

    /// Update order with new limit price
    pub fn update_limit(&mut self, new_price: f64) {
        self.order.price = Some(new_price);
        self.adjustments += 1;
    }
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

    // ==================== Advanced Order Management Methods ====================

    /// Submit an OCO (One-Cancels-Other) order group
    /// When one order fills, all other orders in the group are automatically canceled
    async fn submit_oco_order(&self, oco: &OcoOrder) -> Result<String> {
        // Default implementation submits orders individually
        // Real implementations should use exchange-specific OCO endpoints
        let mut first_order_id = String::new();
        for order in &oco.orders {
            first_order_id = self.submit_order(order).await?;
        }
        Ok(first_order_id)
    }

    /// Cancel an OCO order group
    async fn cancel_oco_order(&self, group_id: &str) -> Result<()> {
        // Default implementation: find and cancel all orders in the group
        // Real implementations would use exchange-specific cancellation
        Err(QuantError::Api(format!(
            "OCO cancellation not implemented for group {}",
            group_id
        )))
    }

    /// Submit a bracket order (entry + SL + TP)
    async fn submit_bracket_order(&self, bracket: &BracketOrder) -> Result<String> {
        // Submit entry order first
        let entry_id = self.submit_order(&bracket.entry_order).await?;

        // In a real implementation, SL and TP would be conditional on entry fill
        // For now, we submit them with OCO behavior
        let oco = OcoOrder {
            group_id: bracket.bracket_id.clone(),
            orders: vec![
                bracket.stop_loss_order.clone(),
                bracket.take_profit_order.clone(),
            ],
            timestamp: bracket.timestamp,
            active: true,
            filled_order_id: None,
        };
        self.submit_oco_order(&oco).await?;

        Ok(entry_id)
    }

    /// Submit an iceberg order (split large orders)
    async fn submit_iceberg_order(&self, iceberg: &IcebergOrder) -> Result<String> {
        // Submit first visible slice
        let initial_order = Order {
            id: uuid::Uuid::new_v4().to_string(),
            symbol: iceberg.symbol.clone(),
            side: iceberg.side,
            order_type: OrderType::Limit,
            quantity: iceberg.visible_quantity.min(iceberg.total_quantity),
            price: Some(iceberg.price),
            status: OrderStatus::New,
            timestamp: iceberg.timestamp,
            oco_group_id: None,
            iceberg_hidden_qty: Some(iceberg.hidden_quantity()),
            stop_loss: None,
            take_profit: None,
            parent_order_id: Some(iceberg.iceberg_id.clone()),
            limit_chase_amount: None,
        };

        self.submit_order(&initial_order).await
    }

    /// Submit a limit chase order
    async fn submit_limit_chase(&self, chase: &LimitChaseOrder) -> Result<String> {
        self.submit_order(&chase.order).await
    }

    /// Modify an existing order (price, quantity, etc.)
    async fn modify_order(
        &self,
        order_id: &str,
        symbol: &str,
        new_price: Option<f64>,
        new_quantity: Option<f64>,
    ) -> Result<Order> {
        // Default implementation: cancel and replace
        // Real implementations would use exchange-specific modify endpoints
        let old_order = self.get_order(order_id, symbol).await?;

        // Cancel old order
        self.cancel_order(order_id, symbol).await?;

        // Create new order with modified parameters
        let mut new_order = old_order.clone();
        if let Some(price) = new_price {
            new_order.price = Some(price);
        }
        if let Some(qty) = new_quantity {
            new_order.quantity = qty;
        }

        let new_id = self.submit_order(&new_order).await?;
        new_order.id = new_id;

        Ok(new_order)
    }

    /// Bulk cancel all orders for a symbol
    async fn cancel_all_orders(&self, symbol: &str) -> Result<()> {
        // Default implementation: would need to list orders first
        // Real implementations would use exchange-specific bulk cancel
        Err(QuantError::Api(format!(
            "Bulk cancel not implemented for symbol {}",
            symbol
        )))
    }

    /// Get all pending orders
    async fn get_pending_orders(&self) -> Result<Vec<Order>> {
        // Default implementation: would need to query all orders
        Ok(Vec::new())
    }
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

    // TradingMode tests
    #[test]
    fn test_trading_mode_default_is_spot() {
        let mode = TradingMode::default();
        assert_eq!(mode, TradingMode::Spot);
    }

    #[test]
    fn test_trading_mode_variants() {
        let spot = TradingMode::Spot;
        let margin = TradingMode::Margin;

        assert_eq!(spot, TradingMode::Spot);
        assert_eq!(margin, TradingMode::Margin);
        assert_ne!(spot, margin);
    }

    #[test]
    fn test_trading_mode_copy_clone() {
        let spot = TradingMode::Spot;
        let copied = spot;
        let cloned = spot;

        assert_eq!(spot, copied);
        assert_eq!(spot, cloned);
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
