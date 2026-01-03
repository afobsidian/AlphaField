//! Performance Reports Module
//!
//! Provides P&L summaries aggregated by time period (daily, weekly, monthly, yearly),
//! strategy performance breakdowns, and CSV export functionality.

use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::drawdown::DrawdownAnalysis;
use crate::trade::Trade;

/// Report aggregation period
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportPeriod {
    Daily,
    Weekly,
    Monthly,
    Yearly,
    AllTime,
}

impl ReportPeriod {
    /// Get period key from a datetime for grouping
    pub fn period_key(&self, dt: DateTime<Utc>) -> String {
        match self {
            ReportPeriod::Daily => dt.format("%Y-%m-%d").to_string(),
            ReportPeriod::Weekly => {
                // Get ISO week number
                let week = dt.iso_week();
                format!("{}-W{:02}", week.year(), week.week())
            }
            ReportPeriod::Monthly => dt.format("%Y-%m").to_string(),
            ReportPeriod::Yearly => dt.format("%Y").to_string(),
            ReportPeriod::AllTime => "all".to_string(),
        }
    }

    /// Parse period key back to date range
    pub fn parse_key(&self, key: &str) -> Option<(NaiveDate, NaiveDate)> {
        match self {
            ReportPeriod::Daily => {
                let date = NaiveDate::parse_from_str(key, "%Y-%m-%d").ok()?;
                Some((date, date))
            }
            ReportPeriod::Weekly => {
                // Parse "2024-W01" format
                let parts: Vec<&str> = key.split("-W").collect();
                if parts.len() != 2 {
                    return None;
                }
                let year: i32 = parts[0].parse().ok()?;
                let week: u32 = parts[1].parse().ok()?;
                let start = NaiveDate::from_isoywd_opt(year, week, Weekday::Mon)?;
                let end = NaiveDate::from_isoywd_opt(year, week, Weekday::Sun)?;
                Some((start, end))
            }
            ReportPeriod::Monthly => {
                let date = NaiveDate::parse_from_str(&format!("{}-01", key), "%Y-%m-%d").ok()?;
                let end = if date.month() == 12 {
                    NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)?.pred_opt()?
                } else {
                    NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)?.pred_opt()?
                };
                Some((date, end))
            }
            ReportPeriod::Yearly => {
                let year: i32 = key.parse().ok()?;
                let start = NaiveDate::from_ymd_opt(year, 1, 1)?;
                let end = NaiveDate::from_ymd_opt(year, 12, 31)?;
                Some((start, end))
            }
            ReportPeriod::AllTime => None,
        }
    }
}

/// Summary for a single time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodSummary {
    /// Period identifier (e.g., "2024-01-15", "2024-W03", "2024-01", "2024")
    pub period_key: String,
    /// Period type
    pub period_type: ReportPeriod,
    /// Start date of period
    pub start_date: Option<NaiveDate>,
    /// End date of period
    pub end_date: Option<NaiveDate>,
    /// Total P&L for the period
    pub total_pnl: f64,
    /// Number of trades
    pub trade_count: usize,
    /// Winning trades
    pub winning_trades: usize,
    /// Losing trades
    pub losing_trades: usize,
    /// Win rate
    pub win_rate: f64,
    /// Total fees paid
    pub total_fees: f64,
    /// Best trade P&L
    pub best_trade: f64,
    /// Worst trade P&L
    pub worst_trade: f64,
    /// Average trade P&L
    pub avg_trade: f64,
    /// Gross profit (sum of winning trades)
    pub gross_profit: f64,
    /// Gross loss (sum of losing trades, absolute value)
    pub gross_loss: f64,
    /// Profit factor (gross profit / gross loss)
    pub profit_factor: f64,
}

impl PeriodSummary {
    /// Calculate summary from trades for a given period
    pub fn calculate(period_key: String, period_type: ReportPeriod, trades: &[&Trade]) -> Self {
        if trades.is_empty() {
            return Self {
                period_key,
                period_type,
                start_date: None,
                end_date: None,
                total_pnl: 0.0,
                trade_count: 0,
                winning_trades: 0,
                losing_trades: 0,
                win_rate: 0.0,
                total_fees: 0.0,
                best_trade: 0.0,
                worst_trade: 0.0,
                avg_trade: 0.0,
                gross_profit: 0.0,
                gross_loss: 0.0,
                profit_factor: 0.0,
            };
        }

        let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
        let total_fees: f64 = trades.iter().map(|t| t.fees).sum();
        let trade_count = trades.len();

        let winning: Vec<_> = trades.iter().filter(|t| t.pnl > 0.0).collect();
        let losing: Vec<_> = trades.iter().filter(|t| t.pnl <= 0.0).collect();

        let winning_trades = winning.len();
        let losing_trades = losing.len();
        let win_rate = if trade_count > 0 {
            winning_trades as f64 / trade_count as f64
        } else {
            0.0
        };

        let gross_profit: f64 = winning.iter().map(|t| t.pnl).sum();
        let gross_loss: f64 = losing.iter().map(|t| t.pnl.abs()).sum();

        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        let best_trade = trades
            .iter()
            .map(|t| t.pnl)
            .fold(f64::NEG_INFINITY, f64::max);
        let worst_trade = trades.iter().map(|t| t.pnl).fold(f64::INFINITY, f64::min);
        let avg_trade = total_pnl / trade_count as f64;

        // Parse date range from period key
        let (start_date, end_date) = match period_type.parse_key(&period_key) {
            Some((s, e)) => (Some(s), Some(e)),
            None => (None, None),
        };

        Self {
            period_key,
            period_type,
            start_date,
            end_date,
            total_pnl,
            trade_count,
            winning_trades,
            losing_trades,
            win_rate,
            total_fees,
            best_trade,
            worst_trade,
            avg_trade,
            gross_profit,
            gross_loss,
            profit_factor,
        }
    }
}

/// Strategy performance breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyBreakdown {
    /// Strategy name
    pub strategy: String,
    /// Total P&L for this strategy
    pub total_pnl: f64,
    /// Number of trades
    pub trade_count: usize,
    /// Win rate
    pub win_rate: f64,
    /// Profit factor
    pub profit_factor: f64,
    /// Average trade P&L
    pub avg_trade: f64,
    /// Total fees
    pub total_fees: f64,
}

/// Complete performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// Period type for this report
    pub period_type: ReportPeriod,
    /// Summaries for each period
    pub summaries: Vec<PeriodSummary>,
    /// Overall summary across all trades
    pub overall: PeriodSummary,
    /// Strategy breakdown (if strategy info available)
    pub strategy_breakdown: Vec<StrategyBreakdown>,
    /// Drawdown analysis (if equity history provided)
    pub drawdown: Option<DrawdownAnalysis>,
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
}

impl PerformanceReport {
    /// Generate a performance report from trades
    pub fn generate(trades: &[Trade], period: ReportPeriod) -> Self {
        // Group trades by period
        let mut period_groups: HashMap<String, Vec<&Trade>> = HashMap::new();
        for trade in trades {
            let key = period.period_key(trade.exit_time);
            period_groups.entry(key).or_default().push(trade);
        }

        // Calculate summary for each period
        let mut summaries: Vec<PeriodSummary> = period_groups
            .into_iter()
            .map(|(key, trades)| PeriodSummary::calculate(key, period, &trades))
            .collect();

        // Sort by period key
        summaries.sort_by(|a, b| a.period_key.cmp(&b.period_key));

        // Calculate overall summary
        let all_trades: Vec<&Trade> = trades.iter().collect();
        let overall =
            PeriodSummary::calculate("all".to_string(), ReportPeriod::AllTime, &all_trades);

        Self {
            period_type: period,
            summaries,
            overall,
            strategy_breakdown: Vec::new(),
            drawdown: None,
            generated_at: Utc::now(),
        }
    }

    /// Generate report with equity history for drawdown analysis
    pub fn generate_with_equity(
        trades: &[Trade],
        period: ReportPeriod,
        equity_history: &[(i64, f64)],
    ) -> Self {
        let mut report = Self::generate(trades, period);

        // Calculate drawdown if we have equity history
        if !equity_history.is_empty() {
            let total_return = if equity_history.len() >= 2 {
                let first = equity_history.first().unwrap().1;
                let last = equity_history.last().unwrap().1;
                (last - first) / first
            } else {
                0.0
            };
            report.drawdown = Some(DrawdownAnalysis::calculate(equity_history, total_return));
        }

        report
    }

    /// Add strategy breakdown from strategy-tagged trades
    pub fn with_strategy_breakdown(
        mut self,
        strategy_trades: HashMap<String, Vec<&Trade>>,
    ) -> Self {
        self.strategy_breakdown = strategy_trades
            .into_iter()
            .map(|(strategy, trades)| {
                let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
                let total_fees: f64 = trades.iter().map(|t| t.fees).sum();
                let trade_count = trades.len();

                let winning = trades.iter().filter(|t| t.pnl > 0.0).count();
                let win_rate = if trade_count > 0 {
                    winning as f64 / trade_count as f64
                } else {
                    0.0
                };

                let gross_profit: f64 = trades.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).sum();
                let gross_loss: f64 = trades
                    .iter()
                    .filter(|t| t.pnl <= 0.0)
                    .map(|t| t.pnl.abs())
                    .sum();

                let profit_factor = if gross_loss > 0.0 {
                    gross_profit / gross_loss
                } else if gross_profit > 0.0 {
                    f64::INFINITY
                } else {
                    0.0
                };

                let avg_trade = if trade_count > 0 {
                    total_pnl / trade_count as f64
                } else {
                    0.0
                };

                StrategyBreakdown {
                    strategy,
                    total_pnl,
                    trade_count,
                    win_rate,
                    profit_factor,
                    avg_trade,
                    total_fees,
                }
            })
            .collect();

        // Sort by total P&L descending
        self.strategy_breakdown
            .sort_by(|a, b| b.total_pnl.partial_cmp(&a.total_pnl).unwrap());

        self
    }

    /// Export report to CSV format
    pub fn to_csv(&self) -> String {
        let mut csv = String::new();

        // Header
        csv.push_str("Period,Total P&L,Trade Count,Win Rate,Profit Factor,Gross Profit,Gross Loss,Total Fees,Best Trade,Worst Trade,Avg Trade\n");

        // Period rows
        for summary in &self.summaries {
            csv.push_str(&format!(
                "{},{:.2},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
                summary.period_key,
                summary.total_pnl,
                summary.trade_count,
                summary.win_rate * 100.0,
                summary.profit_factor,
                summary.gross_profit,
                summary.gross_loss,
                summary.total_fees,
                summary.best_trade,
                summary.worst_trade,
                summary.avg_trade
            ));
        }

        // Overall summary
        csv.push_str(&format!(
            "TOTAL,{:.2},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
            self.overall.total_pnl,
            self.overall.trade_count,
            self.overall.win_rate * 100.0,
            self.overall.profit_factor,
            self.overall.gross_profit,
            self.overall.gross_loss,
            self.overall.total_fees,
            self.overall.best_trade,
            self.overall.worst_trade,
            self.overall.avg_trade
        ));

        csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trade::TradeSide;
    use chrono::TimeZone;

    fn make_trade(symbol: &str, pnl: f64, exit_time: DateTime<Utc>) -> Trade {
        let entry_time = exit_time - chrono::Duration::hours(1);
        Trade {
            symbol: symbol.to_string(),
            side: TradeSide::Long,
            entry_time,
            exit_time,
            entry_price: 100.0,
            exit_price: if pnl > 0.0 { 110.0 } else { 90.0 },
            quantity: 1.0,
            pnl,
            fees: 0.5,
            mae: 5.0,
            mfe: 15.0,
            duration_secs: 3600,
            exit_reason: Some("Signal".to_string()),
        }
    }

    #[test]
    fn test_daily_aggregation() {
        let trades = vec![
            make_trade(
                "BTC",
                100.0,
                Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap(),
            ),
            make_trade(
                "BTC",
                50.0,
                Utc.with_ymd_and_hms(2024, 1, 15, 14, 0, 0).unwrap(),
            ),
            make_trade(
                "BTC",
                -30.0,
                Utc.with_ymd_and_hms(2024, 1, 16, 10, 0, 0).unwrap(),
            ),
        ];

        let report = PerformanceReport::generate(&trades, ReportPeriod::Daily);

        assert_eq!(report.summaries.len(), 2);
        assert_eq!(report.overall.trade_count, 3);
        assert!((report.overall.total_pnl - 120.0).abs() < 0.01);

        // Check Jan 15 summary
        let jan15 = report
            .summaries
            .iter()
            .find(|s| s.period_key == "2024-01-15")
            .unwrap();
        assert_eq!(jan15.trade_count, 2);
        assert!((jan15.total_pnl - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_monthly_aggregation() {
        let trades = vec![
            make_trade(
                "ETH",
                100.0,
                Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap(),
            ),
            make_trade(
                "ETH",
                50.0,
                Utc.with_ymd_and_hms(2024, 2, 10, 10, 0, 0).unwrap(),
            ),
            make_trade(
                "ETH",
                75.0,
                Utc.with_ymd_and_hms(2024, 2, 20, 10, 0, 0).unwrap(),
            ),
        ];

        let report = PerformanceReport::generate(&trades, ReportPeriod::Monthly);

        assert_eq!(report.summaries.len(), 2);

        let jan = report
            .summaries
            .iter()
            .find(|s| s.period_key == "2024-01")
            .unwrap();
        assert_eq!(jan.trade_count, 1);

        let feb = report
            .summaries
            .iter()
            .find(|s| s.period_key == "2024-02")
            .unwrap();
        assert_eq!(feb.trade_count, 2);
    }

    #[test]
    fn test_csv_export() {
        let trades = vec![
            make_trade(
                "BTC",
                100.0,
                Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap(),
            ),
            make_trade(
                "BTC",
                -30.0,
                Utc.with_ymd_and_hms(2024, 1, 16, 10, 0, 0).unwrap(),
            ),
        ];

        let report = PerformanceReport::generate(&trades, ReportPeriod::Daily);
        let csv = report.to_csv();

        assert!(csv.contains("Period,Total P&L"));
        assert!(csv.contains("2024-01-15"));
        assert!(csv.contains("2024-01-16"));
        assert!(csv.contains("TOTAL"));
    }

    #[test]
    fn test_period_key_parsing() {
        // Daily
        let (start, end) = ReportPeriod::Daily.parse_key("2024-01-15").unwrap();
        assert_eq!(start, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        // Monthly
        let (start, end) = ReportPeriod::Monthly.parse_key("2024-02").unwrap();
        assert_eq!(start, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()); // 2024 is leap year

        // Yearly
        let (start, end) = ReportPeriod::Yearly.parse_key("2024").unwrap();
        assert_eq!(start, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
    }
}
