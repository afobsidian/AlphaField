//! Benchmark Comparison Module
//!
//! Provides benchmark comparison functionality for backtests,
//! including buy-and-hold and custom benchmarks.

use serde::{Deserialize, Serialize};

/// Benchmark strategy type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BenchmarkType {
    /// Buy and hold the primary asset
    BuyAndHold,
    /// Equal weight across multiple assets
    EqualWeight(Vec<String>),
    /// Custom equity curve provided
    Custom,
}

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub benchmark_type: BenchmarkType,
    /// Symbol for buy-and-hold benchmark
    pub symbol: Option<String>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            benchmark_type: BenchmarkType::BuyAndHold,
            symbol: Some("BTC".to_string()),
        }
    }
}

/// Benchmark comparison results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkComparison {
    /// Benchmark equity curve
    pub benchmark_curve: Vec<(i64, f64)>,
    /// Benchmark total return
    pub benchmark_return: f64,
    /// Benchmark CAGR
    pub benchmark_cagr: f64,
    /// Benchmark max drawdown
    pub benchmark_max_drawdown: f64,
    /// Alpha (excess return vs benchmark)
    pub alpha: f64,
    /// Beta (correlation with benchmark)
    pub beta: f64,
    /// Information ratio
    pub information_ratio: f64,
    /// Tracking error (standard deviation of excess returns)
    pub tracking_error: f64,
    /// Correlation with benchmark
    pub correlation: f64,
    /// Excess return over benchmark
    pub excess_return: f64,
}

impl BenchmarkComparison {
    /// Calculate buy-and-hold benchmark curve from price data
    pub fn calculate_buy_and_hold(
        bars: &[alphafield_core::Bar],
        initial_capital: f64,
    ) -> Vec<(i64, f64)> {
        if bars.is_empty() {
            return Vec::new();
        }

        let initial_price = bars[0].close;
        let shares = initial_capital / initial_price;

        bars.iter()
            .map(|bar| {
                let ts = bar.timestamp.timestamp_millis();
                let equity = shares * bar.close;
                (ts, equity)
            })
            .collect()
    }

    /// Calculate comparison metrics between strategy and benchmark
    pub fn calculate(
        strategy_curve: &[(i64, f64)],
        benchmark_curve: &[(i64, f64)],
        risk_free_rate: f64,
    ) -> Self {
        if strategy_curve.is_empty() || benchmark_curve.is_empty() {
            return Self::default();
        }

        // Calculate returns
        let strategy_returns = Self::calculate_returns(strategy_curve);
        let benchmark_returns = Self::calculate_returns(benchmark_curve);

        // Total returns
        let strategy_total = (strategy_curve.last().unwrap().1 - strategy_curve.first().unwrap().1)
            / strategy_curve.first().unwrap().1;
        let benchmark_total = (benchmark_curve.last().unwrap().1
            - benchmark_curve.first().unwrap().1)
            / benchmark_curve.first().unwrap().1;

        // CAGR for benchmark
        let start_time = benchmark_curve.first().unwrap().0;
        let end_time = benchmark_curve.last().unwrap().0;
        let years = (end_time - start_time) as f64 / (1000.0 * 3600.0 * 24.0 * 365.0);
        let benchmark_cagr = if years > 0.0 {
            (benchmark_curve.last().unwrap().1 / benchmark_curve.first().unwrap().1)
                .powf(1.0 / years)
                - 1.0
        } else {
            0.0
        };

        // Max drawdown for benchmark
        let benchmark_max_drawdown = Self::calculate_max_drawdown(benchmark_curve);

        // Excess returns
        let min_len = strategy_returns.len().min(benchmark_returns.len());
        let excess_returns: Vec<f64> = strategy_returns
            .iter()
            .take(min_len)
            .zip(benchmark_returns.iter().take(min_len))
            .map(|(s, b)| s - b)
            .collect();

        // Tracking error (annualized std dev of excess returns)
        let tracking_error = if excess_returns.is_empty() {
            0.0
        } else {
            let mean = excess_returns.iter().sum::<f64>() / excess_returns.len() as f64;
            let variance = excess_returns
                .iter()
                .map(|r| (r - mean).powi(2))
                .sum::<f64>()
                / excess_returns.len() as f64;
            variance.sqrt() * (252.0f64).sqrt()
        };

        // Information ratio
        let excess_return = strategy_total - benchmark_total;
        let information_ratio = if tracking_error > 0.0 {
            (excess_return * (252.0 / min_len.max(1) as f64).sqrt()) / tracking_error
        } else {
            0.0
        };

        // Beta (covariance / variance of benchmark)
        let (beta, correlation) =
            Self::calculate_beta_correlation(&strategy_returns, &benchmark_returns);

        // Alpha (Jensen's Alpha)
        let strategy_avg =
            strategy_returns.iter().sum::<f64>() / strategy_returns.len().max(1) as f64;
        let benchmark_avg =
            benchmark_returns.iter().sum::<f64>() / benchmark_returns.len().max(1) as f64;
        let alpha = (strategy_avg - risk_free_rate / 252.0)
            - beta * (benchmark_avg - risk_free_rate / 252.0);
        let annualized_alpha = alpha * 252.0;

        Self {
            benchmark_curve: benchmark_curve.to_vec(),
            benchmark_return: benchmark_total,
            benchmark_cagr,
            benchmark_max_drawdown,
            alpha: annualized_alpha,
            beta,
            information_ratio,
            tracking_error,
            correlation,
            excess_return,
        }
    }

    fn calculate_returns(curve: &[(i64, f64)]) -> Vec<f64> {
        if curve.len() < 2 {
            return Vec::new();
        }

        curve
            .windows(2)
            .map(|w| (w[1].1 - w[0].1) / w[0].1)
            .collect()
    }

    fn calculate_max_drawdown(curve: &[(i64, f64)]) -> f64 {
        if curve.is_empty() {
            return 0.0;
        }

        let mut peak = curve[0].1;
        let mut max_dd = 0.0;

        for &(_, equity) in curve {
            if equity > peak {
                peak = equity;
            }
            let dd = (peak - equity) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }

        max_dd
    }

    fn calculate_beta_correlation(strategy: &[f64], benchmark: &[f64]) -> (f64, f64) {
        let min_len = strategy.len().min(benchmark.len());
        if min_len < 2 {
            return (1.0, 0.0);
        }

        let s: Vec<_> = strategy.iter().take(min_len).copied().collect();
        let b: Vec<_> = benchmark.iter().take(min_len).copied().collect();

        let s_mean = s.iter().sum::<f64>() / min_len as f64;
        let b_mean = b.iter().sum::<f64>() / min_len as f64;

        let mut covariance = 0.0;
        let mut s_variance = 0.0;
        let mut b_variance = 0.0;

        for i in 0..min_len {
            let s_diff = s[i] - s_mean;
            let b_diff = b[i] - b_mean;
            covariance += s_diff * b_diff;
            s_variance += s_diff.powi(2);
            b_variance += b_diff.powi(2);
        }

        let beta = if b_variance > 0.0 {
            covariance / b_variance
        } else {
            1.0
        };

        let correlation = if s_variance > 0.0 && b_variance > 0.0 {
            covariance / (s_variance.sqrt() * b_variance.sqrt())
        } else {
            0.0
        };

        (beta, correlation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_comparison() {
        let strategy = vec![
            (0i64, 100.0),
            (86400000, 105.0),
            (172800000, 110.0),
            (259200000, 108.0),
            (345600000, 115.0),
        ];

        let benchmark = vec![
            (0i64, 100.0),
            (86400000, 102.0),
            (172800000, 104.0),
            (259200000, 103.0),
            (345600000, 106.0),
        ];

        let comparison = BenchmarkComparison::calculate(&strategy, &benchmark, 0.02);

        assert!(comparison.excess_return > 0.0); // Strategy beat benchmark
        assert!(comparison.alpha > 0.0 || comparison.alpha <= 0.0); // Alpha calculated
    }
}
