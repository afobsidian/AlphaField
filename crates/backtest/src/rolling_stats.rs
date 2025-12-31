//! Rolling Statistics Module
//!
//! Provides rolling window calculations for metrics like Sharpe, Sortino, volatility.

use serde::{Deserialize, Serialize};

/// Rolling statistics over time
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RollingStats {
    /// Window size in periods (e.g., bars)
    pub window_size: usize,
    /// Rolling Sharpe ratio
    pub rolling_sharpe: Vec<(i64, f64)>,
    /// Rolling Sortino ratio
    pub rolling_sortino: Vec<(i64, f64)>,
    /// Rolling volatility (annualized)
    pub rolling_volatility: Vec<(i64, f64)>,
    /// Rolling beta vs benchmark
    pub rolling_beta: Vec<(i64, f64)>,
    /// Rolling returns
    pub rolling_returns: Vec<(i64, f64)>,
}

impl RollingStats {
    /// Calculate rolling statistics from equity history
    pub fn calculate(
        equity_history: &[(i64, f64)],
        window_size: usize,
        risk_free_rate: f64,
    ) -> Self {
        if equity_history.len() < window_size + 1 || window_size == 0 {
            return Self {
                window_size,
                ..Default::default()
            };
        }

        // Calculate returns
        let returns: Vec<(i64, f64)> = equity_history
            .windows(2)
            .map(|w| {
                let ts = w[1].0;
                let ret = (w[1].1 - w[0].1) / w[0].1;
                (ts, ret)
            })
            .collect();

        let mut rolling_sharpe = Vec::new();
        let mut rolling_sortino = Vec::new();
        let mut rolling_volatility = Vec::new();
        let mut rolling_returns = Vec::new();

        for i in window_size..returns.len() {
            let window: Vec<f64> = returns[i - window_size..i]
                .iter()
                .map(|(_, r)| *r)
                .collect();
            let ts = returns[i].0;

            // Mean return
            let mean = window.iter().sum::<f64>() / window.len() as f64;

            // Volatility
            let variance =
                window.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / window.len() as f64;
            let volatility = variance.sqrt() * (252.0f64).sqrt();

            // Sharpe
            let annualized_return = mean * 252.0;
            let sharpe = if volatility > 0.0 {
                (annualized_return - risk_free_rate) / volatility
            } else {
                0.0
            };

            // Sortino (downside deviation)
            let downside: Vec<f64> = window.iter().filter(|&&r| r < 0.0).copied().collect();
            let downside_variance = if downside.is_empty() {
                0.0
            } else {
                downside.iter().map(|r| r.powi(2)).sum::<f64>() / downside.len() as f64
            };
            let downside_dev = downside_variance.sqrt() * (252.0f64).sqrt();
            let sortino = if downside_dev > 0.0 {
                (annualized_return - risk_free_rate) / downside_dev
            } else if annualized_return > risk_free_rate {
                f64::INFINITY
            } else {
                0.0
            };

            // Rolling cumulative return
            let cum_return = window.iter().fold(1.0, |acc, r| acc * (1.0 + r)) - 1.0;

            rolling_sharpe.push((ts, sharpe));
            rolling_sortino.push((ts, if sortino.is_finite() { sortino } else { 0.0 }));
            rolling_volatility.push((ts, volatility));
            rolling_returns.push((ts, cum_return));
        }

        Self {
            window_size,
            rolling_sharpe,
            rolling_sortino,
            rolling_volatility,
            rolling_beta: Vec::new(), // Requires benchmark
            rolling_returns,
        }
    }

    /// Calculate rolling beta vs benchmark
    pub fn calculate_with_benchmark(
        equity_history: &[(i64, f64)],
        benchmark_history: &[(i64, f64)],
        window_size: usize,
        risk_free_rate: f64,
    ) -> Self {
        let mut stats = Self::calculate(equity_history, window_size, risk_free_rate);

        if equity_history.len() < window_size + 1
            || benchmark_history.len() < window_size + 1
            || window_size == 0
        {
            return stats;
        }

        // Calculate returns
        let strategy_returns: Vec<(i64, f64)> = equity_history
            .windows(2)
            .map(|w| (w[1].0, (w[1].1 - w[0].1) / w[0].1))
            .collect();

        let benchmark_returns: Vec<(i64, f64)> = benchmark_history
            .windows(2)
            .map(|w| (w[1].0, (w[1].1 - w[0].1) / w[0].1))
            .collect();

        let min_len = strategy_returns.len().min(benchmark_returns.len());
        let mut rolling_beta = Vec::new();

        for i in window_size..min_len {
            let s_window: Vec<f64> = strategy_returns[i - window_size..i]
                .iter()
                .map(|(_, r)| *r)
                .collect();
            let b_window: Vec<f64> = benchmark_returns[i - window_size..i]
                .iter()
                .map(|(_, r)| *r)
                .collect();

            let ts = strategy_returns[i].0;

            let s_mean = s_window.iter().sum::<f64>() / s_window.len() as f64;
            let b_mean = b_window.iter().sum::<f64>() / b_window.len() as f64;

            let mut covariance = 0.0;
            let mut b_variance = 0.0;

            for j in 0..s_window.len() {
                let s_diff = s_window[j] - s_mean;
                let b_diff = b_window[j] - b_mean;
                covariance += s_diff * b_diff;
                b_variance += b_diff.powi(2);
            }

            let beta = if b_variance > 0.0 {
                covariance / b_variance
            } else {
                1.0
            };

            rolling_beta.push((ts, beta));
        }

        stats.rolling_beta = rolling_beta;
        stats
    }
}

/// Monthly returns for heatmap visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyReturn {
    pub year: i32,
    pub month: u32,
    pub return_pct: f64,
}

impl MonthlyReturn {
    /// Calculate monthly returns from equity history
    pub fn calculate(equity_history: &[(i64, f64)]) -> Vec<Self> {
        if equity_history.is_empty() {
            return Vec::new();
        }

        use chrono::{Datelike, TimeZone, Utc};
        use std::collections::BTreeMap;

        // Group by year-month
        let mut monthly: BTreeMap<(i32, u32), (f64, f64)> = BTreeMap::new();

        for &(ts, equity) in equity_history {
            let dt = Utc
                .timestamp_millis_opt(ts)
                .single()
                .unwrap_or_else(Utc::now);
            let key = (dt.year(), dt.month());

            monthly
                .entry(key)
                .and_modify(|(_, last)| *last = equity)
                .or_insert((equity, equity));
        }

        // Calculate returns
        let mut result = Vec::new();
        let mut prev_equity = None;

        for ((year, month), (first, last)) in monthly {
            let start = prev_equity.unwrap_or(first);
            let return_pct = (last - start) / start;

            result.push(MonthlyReturn {
                year,
                month,
                return_pct,
            });

            prev_equity = Some(last);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_stats() {
        let equity: Vec<(i64, f64)> = (0..100)
            .map(|i| {
                let ts = i * 86400000i64;
                let eq = 100.0 + (i as f64) * 0.5 + ((i as f64) * 0.1).sin() * 5.0;
                (ts, eq)
            })
            .collect();

        let stats = RollingStats::calculate(&equity, 20, 0.02);

        assert!(!stats.rolling_sharpe.is_empty());
        assert!(!stats.rolling_volatility.is_empty());
    }
}
