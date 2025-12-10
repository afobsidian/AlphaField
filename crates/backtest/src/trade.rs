//! Trade tracking and trade-level metrics
//!
//! Provides data structures for tracking individual trades and calculating
//! trade-level statistics like win rate, profit factor, and expectancy.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents the side of a trade
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeSide {
    Long,
    Short,
}

/// Represents a completed roundtrip trade (entry + exit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Symbol traded
    pub symbol: String,
    /// Trade side (Long or Short)
    pub side: TradeSide,
    /// Entry timestamp
    pub entry_time: DateTime<Utc>,
    /// Exit timestamp  
    pub exit_time: DateTime<Utc>,
    /// Entry price
    pub entry_price: f64,
    /// Exit price
    pub exit_price: f64,
    /// Quantity traded
    pub quantity: f64,
    /// Realized profit/loss (after fees)
    pub pnl: f64,
    /// Total fees paid
    pub fees: f64,
    /// Maximum Adverse Excursion (worst drawdown during trade)
    pub mae: f64,
    /// Maximum Favorable Excursion (best profit during trade)
    pub mfe: f64,
    /// Duration in seconds
    pub duration_secs: i64,
    /// Exit reason (e.g., "Signal", "Force-Close", "Take-Profit", "Stop-Loss")
    #[serde(default)]
    pub exit_reason: Option<String>,
}

impl Trade {
    /// Calculate return as a percentage
    pub fn return_pct(&self) -> f64 {
        let cost = self.entry_price * self.quantity;
        if cost == 0.0 {
            0.0
        } else {
            self.pnl / cost
        }
    }

    /// Check if trade was profitable
    pub fn is_winner(&self) -> bool {
        self.pnl > 0.0
    }
}

/// Statistics calculated from a collection of trades
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TradeStats {
    /// Total number of trades
    pub total_trades: usize,
    /// Number of winning trades
    pub winning_trades: usize,
    /// Number of losing trades
    pub losing_trades: usize,
    /// Win rate (0.0 - 1.0)
    pub win_rate: f64,
    /// Loss rate (0.0 - 1.0)
    pub loss_rate: f64,
    /// Average profit on winning trades
    pub avg_win: f64,
    /// Average loss on losing trades (absolute value)
    pub avg_loss: f64,
    /// Average win / average loss ratio
    pub avg_win_loss_ratio: f64,
    /// Profit factor (gross profit / gross loss)
    pub profit_factor: f64,
    /// Expectancy (average $ per trade)
    pub expectancy: f64,
    /// Average trade duration in seconds
    pub avg_duration_secs: f64,
    /// Average MAE across all trades
    pub avg_mae: f64,
    /// Average MFE across all trades
    pub avg_mfe: f64,
}

impl TradeStats {
    /// Calculate statistics from a list of trades
    pub fn calculate(trades: &[Trade]) -> Self {
        if trades.is_empty() {
            return Self::default();
        }

        let total_trades = trades.len();
        let winning_trades: Vec<_> = trades.iter().filter(|t| t.is_winner()).collect();
        let losing_trades: Vec<_> = trades.iter().filter(|t| !t.is_winner()).collect();

        let win_count = winning_trades.len();
        let loss_count = losing_trades.len();

        let win_rate = win_count as f64 / total_trades as f64;
        let loss_rate = loss_count as f64 / total_trades as f64;

        let gross_profit: f64 = winning_trades.iter().map(|t| t.pnl).sum();
        let gross_loss: f64 = losing_trades.iter().map(|t| t.pnl.abs()).sum();

        let avg_win = if win_count > 0 {
            gross_profit / win_count as f64
        } else {
            0.0
        };

        let avg_loss = if loss_count > 0 {
            gross_loss / loss_count as f64
        } else {
            0.0
        };

        let avg_win_loss_ratio = if avg_loss > 0.0 {
            avg_win / avg_loss
        } else if avg_win > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let expectancy = total_pnl / total_trades as f64;

        let total_duration: i64 = trades.iter().map(|t| t.duration_secs).sum();
        let avg_duration_secs = total_duration as f64 / total_trades as f64;

        let total_mae: f64 = trades.iter().map(|t| t.mae).sum();
        let avg_mae = total_mae / total_trades as f64;

        let total_mfe: f64 = trades.iter().map(|t| t.mfe).sum();
        let avg_mfe = total_mfe / total_trades as f64;

        Self {
            total_trades,
            winning_trades: win_count,
            losing_trades: loss_count,
            win_rate,
            loss_rate,
            avg_win,
            avg_loss,
            avg_win_loss_ratio,
            profit_factor,
            expectancy,
            avg_duration_secs,
            avg_mae,
            avg_mfe,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_trade(pnl: f64, duration_secs: i64) -> Trade {
        let entry = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let exit = entry + chrono::Duration::seconds(duration_secs);
        Trade {
            symbol: "BTC".to_string(),
            side: TradeSide::Long,
            entry_time: entry,
            exit_time: exit,
            entry_price: 100.0,
            exit_price: if pnl > 0.0 { 110.0 } else { 90.0 },
            quantity: 1.0,
            pnl,
            fees: 0.1,
            mae: 5.0,
            mfe: 15.0,
            duration_secs,
            exit_reason: Some("Signal".to_string()),
        }
    }

    #[test]
    fn test_trade_stats_calculation() {
        let trades = vec![
            make_trade(100.0, 3600),   // Win
            make_trade(50.0, 7200),    // Win
            make_trade(-30.0, 1800),   // Loss
            make_trade(-20.0, 900),    // Loss
        ];

        let stats = TradeStats::calculate(&trades);

        assert_eq!(stats.total_trades, 4);
        assert_eq!(stats.winning_trades, 2);
        assert_eq!(stats.losing_trades, 2);
        assert!((stats.win_rate - 0.5).abs() < 1e-9);
        assert!((stats.loss_rate - 0.5).abs() < 1e-9);
        assert!((stats.avg_win - 75.0).abs() < 1e-9); // (100 + 50) / 2
        assert!((stats.avg_loss - 25.0).abs() < 1e-9); // (30 + 20) / 2
        assert!((stats.profit_factor - 3.0).abs() < 1e-9); // 150 / 50
        assert!((stats.expectancy - 25.0).abs() < 1e-9); // (100 + 50 - 30 - 20) / 4
    }

    #[test]
    fn test_empty_trades() {
        let stats = TradeStats::calculate(&[]);
        assert_eq!(stats.total_trades, 0);
        assert_eq!(stats.win_rate, 0.0);
    }
}
