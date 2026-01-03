//! # Trading Bots Module
//!
//! Automated trading bot implementations for AlphaField.
//! This module provides various bot types for hands-off trading strategies.

pub mod dca;
pub mod grid;
pub mod trailing;

pub use dca::{AmountType, DCABot, DCAConfig, Frequency};
pub use grid::{GridBot, GridConfig, GridLevel};
pub use trailing::{TrailingConfig, TrailingOrder, TrailingType};

use alphafield_core::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of a trading bot
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BotStatus {
    /// Bot is actively running
    Active,
    /// Bot is temporarily paused
    Paused,
    /// Bot has completed its task
    Completed,
    /// Bot encountered an error
    Error,
    /// Bot has been manually stopped
    Stopped,
}

/// Common trait for all trading bots
pub trait TradingBot: Send + Sync {
    /// Returns the unique identifier of the bot
    fn id(&self) -> &str;

    /// Returns the bot name/type
    fn name(&self) -> &str;

    /// Returns the current status of the bot
    fn status(&self) -> BotStatus;

    /// Start the bot
    fn start(&mut self) -> Result<()>;

    /// Pause the bot
    fn pause(&mut self) -> Result<()>;

    /// Resume a paused bot
    fn resume(&mut self) -> Result<()>;

    /// Stop the bot permanently
    fn stop(&mut self) -> Result<()>;

    /// Get bot statistics
    fn stats(&self) -> BotStats;
}

/// Statistics tracked by a bot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotStats {
    /// Number of orders executed
    pub orders_executed: u64,

    /// Total volume traded
    pub total_volume: f64,

    /// Total fees paid
    pub total_fees: f64,

    /// Realized profit/loss
    pub realized_pnl: f64,

    /// Bot start time
    pub started_at: Option<DateTime<Utc>>,

    /// Last execution time
    pub last_execution: Option<DateTime<Utc>>,
}

impl Default for BotStats {
    fn default() -> Self {
        Self {
            orders_executed: 0,
            total_volume: 0.0,
            total_fees: 0.0,
            realized_pnl: 0.0,
            started_at: None,
            last_execution: None,
        }
    }
}
