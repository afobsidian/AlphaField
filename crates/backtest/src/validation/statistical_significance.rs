//! # Statistical Significance Validation
//!
//! Advanced statistical tests for validating trading strategy performance
//! including bootstrap confidence intervals, permutation testing, stationarity
//! tests, and correlation analysis.

use crate::metrics::PerformanceMetrics;
use crate::Trade;
use ndarray::{Array2, ArrayView1};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Bootstrap confidence intervals for Sharpe ratio and other metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapResult {
    /// Mean Sharpe ratio across bootstrap samples
    pub mean_sharpe: f64,
    /// 95% confidence interval lower bound
    pub ci_lower: f64,
    /// 95% confidence interval upper bound
    pub ci_upper: f64,
    /// Number of bootstrap iterations performed
    pub iterations: usize,
    /// Original (non-bootstrapped) Sharpe ratio
    pub original_sharpe: f64,
}

/// Permutation test results for randomness detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermutationResult {
    /// P-value from permutation test
    pub p_value: f64,
    /// Number of permutations performed
    pub permutations: usize,
    /// Original metric value (e.g., Sharpe ratio)
    pub original_value: f64,
    /// Mean of permuted metric values
    pub permuted_mean: f64,
    /// Is the result statistically significant (p < 0.05)?
    pub is_significant: bool,
}

/// Augmented Dickey-Fuller test for stationarity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationarityResult {
    /// ADF test statistic
    pub adf_statistic: f64,
    /// Critical value at 5% significance
    pub critical_value_5pct: f64,
    /// Critical value at 1% significance
    pub critical_value_1pct: f64,
    /// Is the series stationary (reject unit root hypothesis)?
    pub is_stationary: bool,
    /// Test interpretation
    pub interpretation: String,
}

/// Statistical significance of Sharpe ratio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharpeSignificance {
    /// Sharpe ratio
    pub sharpe: f64,
    /// Standard error of Sharpe
    pub standard_error: f64,
    /// t-statistic
    pub t_statistic: f64,
    /// P-value (two-tailed)
    pub p_value: f64,
    /// 95% confidence interval
    pub confidence_interval: (f64, f64),
    /// Is statistically significant (p < 0.05)?
    pub is_significant: bool,
}

/// Correlation between multiple strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    /// Correlation matrix (flattened for JSON serialization)
    pub correlation_matrix: Vec<f64>,
    /// Strategy names in order of matrix
    pub strategy_names: Vec<String>,
    /// Average correlation (off-diagonal)
    pub avg_correlation: f64,
    /// Number of strategies analyzed
    pub n_strategies: usize,
}

/// Comprehensive statistical significance results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSignificanceResult {
    /// Bootstrap confidence intervals
    pub bootstrap: BootstrapResult,
    /// Permutation test results
    pub permutation: PermutationResult,
    /// Stationarity test results
    pub stationarity: StationarityResult,
    /// Sharpe significance
    pub sharpe_significance: SharpeSignificance,
    /// Correlation analysis (if multiple strategies provided)
    pub correlation: Option<CorrelationResult>,
}

impl StatisticalSignificanceResult {
    /// Calculate overall statistical significance score (0-100)
    pub fn overall_score(&self) -> f64 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;

        // Bootstrap: width of CI relative to mean (narrower is better)
        let ci_width = self.bootstrap.ci_upper - self.bootstrap.ci_lower;
        let bootstrap_score = if self.bootstrap.mean_sharpe > 0.0 {
            100.0 * (1.0 - (ci_width / (self.bootstrap.mean_sharpe.abs() + 0.01).min(1.0)))
        } else {
            0.0
        };
        score += bootstrap_score * 0.25;
        weight_sum += 0.25;

        // Permutation: p-value (lower is better)
        let perm_score = (1.0 - self.permutation.p_value) * 100.0;
        score += perm_score * 0.20;
        weight_sum += 0.20;

        // Stationarity: stationary is better
        let stat_score = if self.stationarity.is_stationary {
            100.0
        } else {
            0.0
        };
        score += stat_score * 0.20;
        weight_sum += 0.20;

        // Sharpe significance: p-value (lower is better)
        let sharpe_sig_score = (1.0 - self.sharpe_significance.p_value) * 100.0;
        score += sharpe_sig_score * 0.20;
        weight_sum += 0.20;

        // Correlation: lower average correlation is better (diversification)
        if let Some(ref corr) = self.correlation {
            let corr_score = (1.0 - corr.avg_correlation.abs()) * 100.0;
            score += corr_score * 0.15;
            weight_sum += 0.15;
        }

        if weight_sum > 0.0 {
            score / weight_sum
        } else {
            score
        }
    }
}

/// Bootstrap confidence intervals for Sharpe ratio
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `risk_free_rate` - Annual risk-free rate
/// * `iterations` - Number of bootstrap samples (default: 1000)
/// * `confidence_level` - Confidence level (0-1, default: 0.95)
pub fn bootstrap_sharpe(
    trades: &[Trade],
    risk_free_rate: f64,
    iterations: usize,
    confidence_level: f64,
) -> Result<BootstrapResult, Box<dyn std::error::Error>> {
    if trades.is_empty() {
        return Err("Cannot bootstrap with empty trades".into());
    }

    // Calculate original Sharpe
    let original_sharpe = calculate_sharpe_from_trades(trades, risk_free_rate)?;

    // Bootstrap resampling
    let mut rng = rand::thread_rng();
    let mut sharpe_samples = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        // Resample trades with replacement
        let resampled: Vec<Trade> = (0..trades.len())
            .map(|_| trades[rng.gen_range(0..trades.len())].clone())
            .collect();

        let sharpe = calculate_sharpe_from_trades(&resampled, risk_free_rate)?;
        sharpe_samples.push(sharpe);
    }

    // Calculate statistics
    sharpe_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mean_sharpe: f64 = sharpe_samples.iter().sum::<f64>() / sharpe_samples.len() as f64;

    // Calculate confidence interval
    let alpha = 1.0 - confidence_level;
    let lower_idx = ((alpha / 2.0) * iterations as f64) as usize;
    let upper_idx = ((1.0 - alpha / 2.0) * iterations as f64) as usize;

    let ci_lower = sharpe_samples[lower_idx];
    let ci_upper = sharpe_samples[upper_idx];

    Ok(BootstrapResult {
        mean_sharpe,
        ci_lower,
        ci_upper,
        iterations,
        original_sharpe,
    })
}

/// Permutation test for randomness
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `metric_fn` - Function to calculate metric from trades
/// * `permutations` - Number of permutations (default: 1000)
pub fn permutation_test<F>(
    trades: &[Trade],
    metric_fn: F,
    permutations: usize,
) -> Result<PermutationResult, Box<dyn std::error::Error>>
where
    F: Fn(&[Trade]) -> Result<f64, Box<dyn std::error::Error>>,
{
    if trades.is_empty() {
        return Err("Cannot perform permutation test with empty trades".into());
    }

    // Calculate original metric
    let original_value = metric_fn(trades)?;

    // Get trade returns
    let mut returns: Vec<f64> = trades
        .iter()
        .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
        .collect();

    // Permute and calculate metric
    let mut rng = rand::thread_rng();
    let mut permuted_values = Vec::with_capacity(permutations);

    for _ in 0..permutations {
        returns.shuffle(&mut rng);

        // Create permuted trades with shuffled returns
        let permuted_trades: Vec<Trade> = trades
            .iter()
            .zip(returns.iter())
            .map(|(trade, &ret): (&Trade, &_)| {
                let mut permuted = trade.clone();
                permuted.exit_price = trade.entry_price * (1.0 + ret);
                permuted
            })
            .collect();

        let value = metric_fn(&permuted_trades)?;
        permuted_values.push(value);
    }

    // Calculate p-value: proportion of permuted values as extreme as original
    let more_extreme = permuted_values
        .iter()
        .filter(|&&v| v >= original_value)
        .count();

    let p_value = (more_extreme as f64 + 1.0) / (permutations as f64 + 1.0); // +1 for continuity correction

    let permuted_mean: f64 = permuted_values.iter().sum::<f64>() / permuted_values.len() as f64;

    Ok(PermutationResult {
        p_value,
        permutations,
        original_value,
        permuted_mean,
        is_significant: p_value < 0.05,
    })
}

/// Augmented Dickey-Fuller test for stationarity (simplified implementation)
///
/// Tests the null hypothesis that the time series has a unit root (non-stationary).
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `returns_only` - Use trade returns instead of equity curve (default: true)
pub fn adf_test(
    trades: &[Trade],
    returns_only: bool,
) -> Result<StationarityResult, Box<dyn std::error::Error>> {
    if trades.len() < 10 {
        return Err("ADF test requires at least 10 trades".into());
    }

    // Get time series data
    let series: Vec<f64> = if returns_only {
        // Use trade returns
        trades
            .iter()
            .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
            .collect()
    } else {
        // Use cumulative equity curve
        let mut equity = 10000.0;
        let mut series = Vec::with_capacity(trades.len());
        for trade in trades {
            let pnl = (trade.exit_price - trade.entry_price) * trade.quantity;
            equity += pnl;
            series.push(equity);
        }
        series
    };

    // Simplified ADF test using first differences
    let n = series.len() - 1;
    let delta_y: Vec<f64> = (1..series.len())
        .map(|i| series[i] - series[i - 1])
        .collect();
    let y_lagged: Vec<f64> = (0..n).map(|i| series[i]).collect();

    // Calculate mean
    let delta_mean: f64 = delta_y.iter().sum::<f64>() / n as f64;
    let y_lagged_mean: f64 = y_lagged.iter().sum::<f64>() / n as f64;

    // Calculate covariance and variance
    let mut cov = 0.0;
    let mut var = 0.0;
    for i in 0..n {
        cov += (delta_y[i] - delta_mean) * (y_lagged[i] - y_lagged_mean);
        var += (y_lagged[i] - y_lagged_mean).powi(2);
    }

    // Calculate ADF statistic (negative value)
    let adf_statistic = if var > 1e-10 { cov / var } else { 0.0 };

    // Critical values (approximate for large samples)
    let critical_value_5pct = -2.86;
    let critical_value_1pct = -3.43;

    let is_stationary = adf_statistic < critical_value_5pct;
    let interpretation = if is_stationary {
        "Series is stationary (reject unit root hypothesis)".to_string()
    } else {
        "Series may be non-stationary (cannot reject unit root hypothesis)".to_string()
    };

    Ok(StationarityResult {
        adf_statistic,
        critical_value_5pct,
        critical_value_1pct,
        is_stationary,
        interpretation,
    })
}

/// Calculate statistical significance of Sharpe ratio
///
/// # Arguments
/// * `sharpe` - Sharpe ratio
/// * `returns` - Vector of returns
/// * `risk_free_rate` - Annual risk-free rate
pub fn sharpe_significance(
    sharpe: f64,
    returns: &[f64],
    risk_free_rate: f64,
) -> SharpeSignificance {
    if returns.len() < 2 {
        return SharpeSignificance {
            sharpe: 0.0,
            standard_error: 0.0,
            t_statistic: 0.0,
            p_value: 1.0,
            confidence_interval: (0.0, 0.0),
            is_significant: false,
        };
    }

    // Calculate standard error of Sharpe
    let n = returns.len() as f64;
    let mean_return: f64 = returns.iter().sum::<f64>() / n;

    let mut variance = 0.0;
    for &r in returns {
        variance += (r - mean_return).powi(2);
    }
    variance /= n - 1.0;

    // Approximate standard error of Sharpe (Jobson & Korkie, 1981)
    let standard_error = ((1.0 + 0.5 * sharpe.powi(2)) / n).sqrt();

    let t_statistic = if standard_error > 1e-10 {
        sharpe / standard_error
    } else {
        0.0
    };

    // Approximate p-value (two-tailed)
    let z = t_statistic.abs();
    let p_value = 2.0 * (1.0 - normal_cdf(z));

    // 95% confidence interval
    let confidence_interval = (
        sharpe - 1.96 * standard_error,
        sharpe + 1.96 * standard_error,
    );

    SharpeSignificance {
        sharpe,
        standard_error,
        t_statistic,
        p_value,
        confidence_interval,
        is_significant: p_value < 0.05,
    }
}

/// Calculate correlation matrix between multiple strategies
///
/// # Arguments
/// * `returns_list` - List of return series for each strategy
/// * `strategy_names` - Names of strategies
pub fn calculate_correlations(
    returns_list: &[Vec<f64>],
    strategy_names: &[String],
) -> Option<CorrelationResult> {
    if returns_list.len() < 2 || returns_list.len() != strategy_names.len() {
        return None;
    }

    let n_strategies = returns_list.len();

    // Check all return series have same length
    let n_obs = returns_list[0].len();
    if returns_list.iter().any(|r| r.len() != n_obs) {
        return None;
    }

    // Create 2D array
    let mut array_data = vec![0.0f64; n_obs * n_strategies];
    for (i, returns) in returns_list.iter().enumerate() {
        for (j, &r) in returns.iter().enumerate() {
            array_data[j * n_strategies + i] = r;
        }
    }

    let array = Array2::from_shape_vec((n_obs, n_strategies), array_data).ok()?;

    // Calculate correlation matrix using column views
    let mut corr_matrix = vec![0.0f64; n_strategies * n_strategies];
    let mut off_diag_sum = 0.0;
    let mut off_diag_count = 0;

    for i in 0..n_strategies {
        for j in 0..n_strategies {
            let col_i = array.column(i);
            let col_j = array.column(j);

            let corr = pearson_correlation(col_i, col_j);
            corr_matrix[i * n_strategies + j] = corr;

            if i != j {
                off_diag_sum += corr.abs();
                off_diag_count += 1;
            }
        }
    }

    let avg_correlation = if off_diag_count > 0 {
        off_diag_sum / off_diag_count as f64
    } else {
        0.0
    };

    Some(CorrelationResult {
        correlation_matrix: corr_matrix,
        strategy_names: strategy_names.to_vec(),
        avg_correlation,
        n_strategies,
    })
}

/// Run comprehensive statistical significance validation
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `metrics` - Performance metrics
/// * `risk_free_rate` - Annual risk-free rate
/// * `additional_returns` - Optional list of return series for correlation analysis
pub fn validate_statistical_significance(
    trades: &[Trade],
    metrics: &PerformanceMetrics,
    risk_free_rate: f64,
    additional_returns: Option<&[Vec<f64>]>,
    additional_names: Option<&[String]>,
) -> Result<StatisticalSignificanceResult, Box<dyn std::error::Error>> {
    if trades.is_empty() {
        return Err("Cannot validate statistical significance with empty trades".into());
    }

    // Bootstrap Sharpe confidence intervals
    let bootstrap = bootstrap_sharpe(trades, risk_free_rate, 1000, 0.95)?;

    // Permutation test for Sharpe
    let permutation = permutation_test(
        trades,
        |t| calculate_sharpe_from_trades(t, risk_free_rate),
        1000,
    )?;

    // Stationarity test
    let stationarity = adf_test(trades, true)?;

    // Sharpe significance
    let returns: Vec<f64> = trades
        .iter()
        .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
        .collect();
    let sharpe_significance = sharpe_significance(metrics.sharpe_ratio, &returns, risk_free_rate);

    // Correlation analysis
    let correlation = if let Some(add_returns) = additional_returns {
        let all_returns = vec![returns.clone()];
        let all_returns: Vec<Vec<f64>> = all_returns.iter().chain(add_returns).cloned().collect();

        let mut names = vec!["Main Strategy".to_string()];
        if let Some(add_names) = additional_names {
            names.extend(add_names.iter().cloned());
        }

        calculate_correlations(&all_returns, &names)
    } else {
        None
    };

    Ok(StatisticalSignificanceResult {
        bootstrap,
        permutation,
        stationarity,
        sharpe_significance,
        correlation,
    })
}

/// Helper: Calculate Sharpe ratio from trades
fn calculate_sharpe_from_trades(
    trades: &[Trade],
    risk_free_rate: f64,
) -> Result<f64, Box<dyn std::error::Error>> {
    if trades.is_empty() {
        return Err("Cannot calculate Sharpe from empty trades".into());
    }

    let returns: Vec<f64> = trades
        .iter()
        .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
        .collect();

    if returns.is_empty() {
        return Err("No returns to calculate".into());
    }

    let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;

    let mut _variance = 0.0;
    for r in &returns {
        _variance += (r - mean_return).powi(2);
    }
    _variance /= (returns.len() - 1) as f64;

    let std_dev = _variance.sqrt();

    if std_dev < 1e-10 {
        Ok(0.0)
    } else {
        // Annualize (assuming daily returns)
        let annual_mean = mean_return * 252.0;
        let annual_std = std_dev * (252.0_f64).sqrt();
        Ok((annual_mean - risk_free_rate) / annual_std)
    }
}

/// Helper: Standard normal CDF (approximation)
fn normal_cdf(x: f64) -> f64 {
    const A1: f64 = 0.254829592;
    const A2: f64 = -0.284496736;
    const A3: f64 = 1.421413741;
    const A4: f64 = -1.453152027;
    const A5: f64 = 1.061405429;
    const P: f64 = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + P * x);
    let y = 1.0 - (((((A5 * t + A4) * t) + A3) * t + A2) * t + A1) * t * (-0.5 * x * x).exp();

    0.5 * (1.0 + sign * y)
}

/// Helper: Pearson correlation coefficient
fn pearson_correlation(x: ArrayView1<f64>, y: ArrayView1<f64>) -> f64 {
    let n = x.len();

    if n == 0 {
        return 0.0;
    }

    let mean_x: f64 = x.iter().sum::<f64>() / n as f64;
    let mean_y: f64 = y.iter().sum::<f64>() / n as f64;

    let mut covariance = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        covariance += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    let denominator = (var_x * var_y).sqrt();
    if denominator < 1e-10 {
        0.0
    } else {
        covariance / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use rust_decimal::Decimal;

    fn create_test_trades() -> Vec<Trade> {
        vec![
            Trade {
                id: String::new(),
                symbol: "BTC".to_string(),
                entry_price: 100.0,
                exit_price: 105.0,
                quantity: Decimal::from(10),
                entry_time: DateTime::from_utc(
                    DateTime::from_timestamp(1000000, 0).unwrap(),
                    Utc,
                ),
                exit_time: DateTime::from_utc(
                    DateTime::from_timestamp(1000001, 0).unwrap(),
                    Utc,
                ),
                side: crate::trade::TradeSide::Long,
            },
            Trade {
                id: String::new(),
                symbol: "BTC".to_string(),
                entry_price: 105.0,
                exit_price: 103.0,
                quantity: Decimal::from(10),
                entry_time: DateTime::from_utc(
                    DateTime::from_timestamp(1000002, 0).unwrap(),
                    Utc,
                ),
                exit_time: DateTime::from_utc(
                    DateTime::from_timestamp(1000003, 0).unwrap(),
                    Utc,
                ),
                side: crate::trade::TradeSide::Long,
            },
            Trade {
                id: String::new(),
                symbol: "BTC".to_string(),
                entry_price: 103.0,
                exit_price: 110.0,
                quantity: Decimal::from(10),
                entry_time: DateTime::from_utc(
                    DateTime::from_timestamp(1000004, 0).unwrap(),
                    Utc,
                ),
                exit_time: DateTime::from_utc(
                    DateTime::from_timestamp(1000005, 0).unwrap(),
                    Utc,
                ),
                side: crate::trade::TradeSide::Long,
            },
        ]
    }

    #[test]
    fn test_bootstrap_sharpe() {
        let trades = create_test_trades();
        let result = bootstrap_sharpe(&trades, 0.02, 100, 0.95).unwrap();

        assert_eq!(result.iterations, 100);
        assert!(result.mean_sharpe.is_finite());
        assert!(result.ci_lower <= result.mean_sharpe);
        assert!(result.ci_upper >= result.mean_sharpe);
    }

    #[test]
    fn test_permutation_test() {
        let trades = create_test_trades();
        let result =
            permutation_test(&trades, |t| calculate_sharpe_from_trades(t, 0.02), 100).unwrap();

        assert_eq!(result.permutations, 100);
        assert!(result.p_value >= 0.0 && result.p_value <= 1.0);
        assert!(result.permuted_mean.is_finite());
    }

    #[test]
    fn test_adf_test() {
        let trades = create_test_trades();
        let result = adf_test(&trades, true).unwrap();

        assert!(result.adf_statistic.is_finite());
        assert_eq!(result.critical_value_5pct, -2.86);
        assert_eq!(result.critical_value_1pct, -3.43);
    }

    #[test]
    fn test_sharpe_significance() {
        let returns = vec![0.05, -0.02, 0.03, 0.01, -0.01];
        let result = sharpe_significance(0.5, &returns, 0.02);

        assert_eq!(result.sharpe, 0.5);
        assert!(result.standard_error >= 0.0);
        assert!(result.p_value >= 0.0 && result.p_value <= 1.0);
    }

    #[test]
    fn test_calculate_correlations() {
        let returns1 = vec![0.05, -0.02, 0.03, 0.01, -0.01];
        let returns2 = vec![0.04, -0.01, 0.02, 0.02, -0.02];
        let names = vec!["Strategy A".to_string(), "Strategy B".to_string()];

        let result = calculate_correlations(&[returns1, returns2], &names).unwrap();

        assert_eq!(result.n_strategies, 2);
        assert_eq!(result.strategy_names.len(), 2);
        assert!((result.correlation_matrix[0] - 1.0).abs() < 0.01); // Diagonal = 1
        assert!((result.correlation_matrix[3] - 1.0).abs() < 0.01); // Diagonal = 1
    }
}
