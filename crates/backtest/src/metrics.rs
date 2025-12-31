use crate::trade::{Trade, TradeStats};
use serde::{Deserialize, Serialize};

/// Extended performance metrics including risk-adjusted returns and trade statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    // Core metrics
    pub total_return: f64,
    pub cagr: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub volatility: f64,

    // Risk-adjusted metrics
    pub sortino_ratio: f64,
    pub sqn: f64,
    pub calmar_ratio: f64,
    pub omega_ratio: f64,
    pub recovery_factor: f64,
    pub ulcer_index: f64,

    // Trade-based metrics
    pub total_trades: usize,
    pub win_rate: f64,
    pub loss_rate: f64,
    pub profit_factor: f64,
    pub expectancy: f64,
    pub avg_win_loss_ratio: f64,
}

impl PerformanceMetrics {
    /// Calculate metrics from equity history only (backward compatible)
    pub fn calculate(equity_history: &[(i64, f64)], risk_free_rate: f64) -> Self {
        Self::calculate_with_trades(equity_history, &[], risk_free_rate)
    }

    /// Calculate metrics from equity history and trades
    pub fn calculate_with_trades(
        equity_history: &[(i64, f64)],
        trades: &[Trade],
        risk_free_rate: f64,
    ) -> Self {
        if equity_history.is_empty() {
            return Self::default();
        }

        let initial_equity = equity_history.first().unwrap().1;
        let final_equity = equity_history.last().unwrap().1;

        // Total Return
        let total_return = (final_equity - initial_equity) / initial_equity;

        // CAGR (assuming daily data points for simplicity, can be adjusted)
        // Duration in years
        let start_time = equity_history.first().unwrap().0;
        let end_time = equity_history.last().unwrap().0;
        let duration_years = (end_time - start_time) as f64 / (1000.0 * 3600.0 * 24.0 * 365.0);

        let cagr = if duration_years > 0.0 {
            (final_equity / initial_equity).powf(1.0 / duration_years) - 1.0
        } else {
            0.0
        };

        // Max Drawdown
        let mut max_drawdown = 0.0;
        let mut peak = initial_equity;

        for &(_, equity) in equity_history {
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        // Calculate daily returns
        let mut returns = Vec::new();
        for i in 1..equity_history.len() {
            let prev = equity_history[i - 1].1;
            let curr = equity_history[i].1;
            returns.push((curr - prev) / prev);
        }

        let avg_return = if returns.is_empty() {
            0.0
        } else {
            returns.iter().sum::<f64>() / returns.len() as f64
        };

        let variance = if returns.is_empty() {
            0.0
        } else {
            returns
                .iter()
                .map(|r| (r - avg_return).powi(2))
                .sum::<f64>()
                / returns.len() as f64
        };
        let volatility = variance.sqrt() * (252.0f64).sqrt(); // Annualized volatility

        let annualized_return = avg_return * 252.0; // Simple annualization
        let sharpe_ratio = if volatility > 0.0 {
            (annualized_return - risk_free_rate) / volatility
        } else {
            0.0
        };

        // Sortino Ratio (downside deviation only)
        let downside_returns: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).copied().collect();
        let downside_variance = if downside_returns.is_empty() {
            0.0
        } else {
            downside_returns.iter().map(|r| r.powi(2)).sum::<f64>() / downside_returns.len() as f64
        };
        let downside_deviation = downside_variance.sqrt() * (252.0f64).sqrt();
        let sortino_ratio = if downside_deviation > 0.0 {
            (annualized_return - risk_free_rate) / downside_deviation
        } else if annualized_return > risk_free_rate {
            f64::INFINITY
        } else {
            0.0
        };

        // Trade-based metrics
        let trade_stats = TradeStats::calculate(trades);

        // SQN (System Quality Number)
        // SQN = (Expected Value of Trade Returns / StdDev of Trade Returns) * sqrt(n)
        let sqn = if trades.len() >= 30 {
            let trade_returns: Vec<f64> = trades.iter().map(|t| t.return_pct()).collect();
            let avg_trade_return = trade_returns.iter().sum::<f64>() / trade_returns.len() as f64;
            let trade_variance = trade_returns
                .iter()
                .map(|r| (r - avg_trade_return).powi(2))
                .sum::<f64>()
                / trade_returns.len() as f64;
            let trade_std_dev = trade_variance.sqrt();

            if trade_std_dev > 0.0 {
                (avg_trade_return / trade_std_dev) * (trades.len() as f64).sqrt()
            } else {
                0.0
            }
        } else {
            0.0 // SQN not meaningful with < 30 trades
        };

        // Calmar Ratio (CAGR / Max Drawdown)
        let calmar_ratio = if max_drawdown > 0.0 {
            cagr / max_drawdown
        } else if cagr > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        // Recovery Factor (Total Return / Max Drawdown)
        let recovery_factor = if max_drawdown > 0.0 {
            total_return / max_drawdown
        } else if total_return > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        // Ulcer Index (RMS of drawdowns)
        let mut drawdowns = Vec::new();
        let mut peak_for_ulcer = equity_history.first().map(|e| e.1).unwrap_or(0.0);
        for &(_, equity) in equity_history {
            if equity > peak_for_ulcer {
                peak_for_ulcer = equity;
            }
            let dd = (peak_for_ulcer - equity) / peak_for_ulcer;
            drawdowns.push(dd);
        }
        let ulcer_index = if drawdowns.is_empty() {
            0.0
        } else {
            let sum_sq: f64 = drawdowns.iter().map(|d| d.powi(2)).sum();
            (sum_sq / drawdowns.len() as f64).sqrt()
        };

        // Omega Ratio (sum of returns above threshold / sum below)
        let threshold = risk_free_rate / 252.0; // Daily threshold
        let gains: f64 = returns
            .iter()
            .filter(|&&r| r > threshold)
            .map(|r| r - threshold)
            .sum();
        let losses: f64 = returns
            .iter()
            .filter(|&&r| r < threshold)
            .map(|r| threshold - r)
            .sum();
        let omega_ratio = if losses > 0.0 {
            gains / losses
        } else if gains > 0.0 {
            f64::INFINITY
        } else {
            1.0
        };

        Self {
            total_return,
            cagr,
            sharpe_ratio,
            max_drawdown,
            volatility,
            sortino_ratio,
            sqn,
            calmar_ratio: if calmar_ratio.is_finite() {
                calmar_ratio
            } else {
                0.0
            },
            omega_ratio: if omega_ratio.is_finite() {
                omega_ratio
            } else {
                0.0
            },
            recovery_factor: if recovery_factor.is_finite() {
                recovery_factor
            } else {
                0.0
            },
            ulcer_index,
            total_trades: trade_stats.total_trades,
            win_rate: trade_stats.win_rate,
            loss_rate: trade_stats.loss_rate,
            profit_factor: trade_stats.profit_factor,
            expectancy: trade_stats.expectancy,
            avg_win_loss_ratio: trade_stats.avg_win_loss_ratio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_equity_history() {
        let metrics = PerformanceMetrics::calculate(&[], 0.02);
        assert_eq!(metrics.total_return, 0.0);
        assert_eq!(metrics.sharpe_ratio, 0.0);
    }

    #[test]
    fn test_basic_metrics() {
        // Simulate equity going from 100 to 110 over 10 days
        let equity: Vec<(i64, f64)> = (0..10)
            .map(|i| {
                let ts = i * 86400 * 1000; // Daily timestamps in ms
                let eq = 100.0 + i as f64;
                (ts, eq)
            })
            .collect();

        let metrics = PerformanceMetrics::calculate(&equity, 0.02);

        assert!((metrics.total_return - 0.09).abs() < 0.01); // (109 - 100) / 100
        assert!(metrics.max_drawdown >= 0.0); // No drawdown in uptrend
    }

    #[test]
    fn test_sortino_with_downside() {
        // Create equity with some drawdowns
        let equity = vec![
            (0, 100.0),
            (86400000, 105.0),
            (172800000, 98.0), // Drawdown
            (259200000, 103.0),
            (345600000, 95.0), // Drawdown
            (432000000, 110.0),
        ];

        let metrics = PerformanceMetrics::calculate(&equity, 0.02);

        // Sortino should be finite since we have downside returns
        assert!(metrics.sortino_ratio.is_finite());
    }
}
