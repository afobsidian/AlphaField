//! # Temporal Validation
//!
//! Tests strategy performance over time including expanding window analysis,
//! rolling stability testing, period decomposition, market cycle analysis,
//! and forward-looking validation.

use crate::metrics::PerformanceMetrics;
use crate::trade::Trade;
use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Expanding window result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandingWindowResult {
    /// Number of bars in window
    pub window_size: usize,
    /// Cumulative Sharpe ratio at this window size
    pub cumulative_sharpe: f64,
    /// Cumulative return at this window size
    pub cumulative_return: f64,
    /// Maximum drawdown at this window size
    pub max_drawdown: f64,
    /// Window start time
    pub window_start: DateTime<Utc>,
    /// Window end time
    pub window_end: DateTime<Utc>,
}

/// Rolling stability result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingStabilityResult {
    /// Window size (in bars)
    pub window_size: usize,
    /// Mean Sharpe across all windows
    pub mean_sharpe: f64,
    /// Standard deviation of Sharpe
    pub std_sharpe: f64,
    /// Coefficient of variation (CV = std/mean)
    pub cv_sharpe: f64,
    /// Minimum Sharpe across windows
    pub min_sharpe: f64,
    /// Maximum Sharpe across windows
    pub max_sharpe: f64,
    /// Number of windows analyzed
    pub n_windows: usize,
    /// Stability rating
    pub rating: StabilityRating,
}

/// Period decomposition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodResult {
    /// Period type (year, quarter, month)
    pub period_type: PeriodType,
    /// Period identifier (e.g., "2024", "2024-Q1", "2024-01")
    pub period_id: String,
    /// Period start
    pub start: DateTime<Utc>,
    /// Period end
    pub end: DateTime<Utc>,
    /// Number of trades in period
    pub n_trades: usize,
    /// Sharpe ratio for period
    pub sharpe: f64,
    /// Return for period
    pub period_return: f64,
    /// Max drawdown for period
    pub max_drawdown: f64,
}

/// Market cycle result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketCycleResult {
    /// Cycle type
    pub cycle_type: MarketCycleType,
    /// Cycle start
    pub start: DateTime<Utc>,
    /// Cycle end
    pub end: DateTime<Utc>,
    /// Duration in days
    pub duration_days: i64,
    /// Sharpe ratio during cycle
    pub sharpe: f64,
    /// Return during cycle
    pub cycle_return: f64,
    /// Max drawdown during cycle
    pub max_drawdown: f64,
    /// Number of trades during cycle
    pub n_trades: usize,
}

/// Forward-looking validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardLookingResult {
    /// Backtest Sharpe ratio
    pub backtest_sharpe: f64,
    /// Paper trading Sharpe ratio (if available)
    pub paper_sharpe: Option<f64>,
    /// Live trading Sharpe ratio (if available)
    pub live_sharpe: Option<f64>,
    /// Backtest to paper degradation percentage
    pub backtest_to_paper_degradation: Option<f64>,
    /// Paper to live degradation percentage
    pub paper_to_live_degradation: Option<f64>,
    /// Overall forward-looking rating
    pub rating: ForwardLookingRating,
}

/// Stability rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StabilityRating {
    /// Very stable (CV < 0.25)
    Excellent,
    /// Stable (CV < 0.50)
    Good,
    /// Moderately stable (CV < 0.75)
    Moderate,
    /// Unstable (CV < 1.0)
    Poor,
    /// Very unstable (CV >= 1.0)
    VeryPoor,
}

/// Period type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeriodType {
    Year,
    Quarter,
    Month,
}

/// Market cycle type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketCycleType {
    Bull,
    Bear,
    Sideways,
    Volatile,
}

/// Forward-looking rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForwardLookingRating {
    /// Excellent - performance holds in live trading
    Excellent,
    /// Good - some degradation but still profitable
    Good,
    /// Moderate - significant degradation
    Moderate,
    /// Poor - does not generalize
    Poor,
    /// Insufficient data
    InsufficientData,
}

/// Comprehensive temporal validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalValidationResult {
    /// Expanding window analysis
    pub expanding_windows: Vec<ExpandingWindowResult>,
    /// Rolling stability analysis
    pub rolling_stability: RollingStabilityResult,
    /// Period decomposition
    pub periods: Vec<PeriodResult>,
    /// Market cycle analysis
    pub market_cycles: Vec<MarketCycleResult>,
    /// Forward-looking validation
    pub forward_looking: ForwardLookingResult,
}

impl TemporalValidationResult {
    /// Calculate overall temporal validation score (0-100)
    pub fn overall_score(&self) -> f64 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;

        // Expanding windows: performance consistency
        if self.expanding_windows.len() > 1 {
            let sharpes: Vec<f64> = self
                .expanding_windows
                .iter()
                .map(|w| w.cumulative_sharpe)
                .collect();
            let mean_sharpe: f64 = sharpes.iter().sum::<f64>() / sharpes.len() as f64;
            let variance: f64 = sharpes
                .iter()
                .map(|s| (s - mean_sharpe).powi(2))
                .sum::<f64>()
                / sharpes.len() as f64;
            let std_dev = variance.sqrt();
            // Positive mean with low variation = good
            let window_score = if mean_sharpe > 0.0 {
                100.0 * (1.0 - (std_dev / (mean_sharpe + 0.01).min(2.0)))
            } else {
                0.0
            };
            score += window_score * 0.25;
            weight_sum += 0.25;
        }

        // Rolling stability
        let stability_score = match self.rolling_stability.rating {
            StabilityRating::Excellent => 100.0,
            StabilityRating::Good => 80.0,
            StabilityRating::Moderate => 60.0,
            StabilityRating::Poor => 40.0,
            StabilityRating::VeryPoor => 20.0,
        };
        score += stability_score * 0.25;
        weight_sum += 0.25;

        // Period decomposition: consistency across periods
        if self.periods.len() > 1 {
            let sharpes: Vec<f64> = self
                .periods
                .iter()
                .filter(|p| p.n_trades > 0)
                .map(|p| p.sharpe)
                .collect();
            if !sharpes.is_empty() {
                let mean_sharpe: f64 = sharpes.iter().sum::<f64>() / sharpes.len() as f64;
                let variance: f64 = sharpes
                    .iter()
                    .map(|s| (s - mean_sharpe).powi(2))
                    .sum::<f64>()
                    / sharpes.len() as f64;
                let std_dev = variance.sqrt();
                let period_score = if mean_sharpe > 0.0 {
                    100.0 * (1.0 - (std_dev / (mean_sharpe + 0.01).min(2.0)))
                } else {
                    0.0
                };
                score += period_score * 0.20;
                weight_sum += 0.20;
            }
        }

        // Market cycles: performance in different regimes
        if self.market_cycles.len() >= 3 {
            let bull_sharpe: f64 = self
                .market_cycles
                .iter()
                .filter(|c| matches!(c.cycle_type, MarketCycleType::Bull))
                .map(|c| c.sharpe)
                .sum::<f64>();
            let bear_sharpe: f64 = self
                .market_cycles
                .iter()
                .filter(|c| matches!(c.cycle_type, MarketCycleType::Bear))
                .map(|c| c.sharpe)
                .sum::<f64>();

            // Good performance in both bull and bear markets
            let cycle_score = if bull_sharpe > 0.0 && bear_sharpe > 0.0 {
                100.0
            } else if bull_sharpe > 0.0 {
                60.0
            } else {
                30.0
            };
            score += cycle_score * 0.15;
            weight_sum += 0.15;
        }

        // Forward-looking validation
        let forward_score = match self.forward_looking.rating {
            ForwardLookingRating::Excellent => 100.0,
            ForwardLookingRating::Good => 80.0,
            ForwardLookingRating::Moderate => 60.0,
            ForwardLookingRating::Poor => 40.0,
            ForwardLookingRating::InsufficientData => 50.0,
        };
        score += forward_score * 0.15;
        weight_sum += 0.15;

        if weight_sum > 0.0 {
            score / weight_sum
        } else {
            score
        }
    }
}

/// Analyze expanding window performance
///
/// # Arguments
/// * `trades` - Vector of trades sorted by entry time
/// * `risk_free_rate` - Annual risk-free rate
/// * `window_sizes` - Window sizes to analyze (in bars)
pub fn analyze_expanding_windows(
    trades: &[Trade],
    risk_free_rate: f64,
    window_sizes: Vec<usize>,
) -> Vec<ExpandingWindowResult> {
    if trades.is_empty() || window_sizes.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    let n_trades = trades.len();

    for &window_size in &window_sizes {
        if window_size == 0 || window_size > n_trades {
            continue;
        }

        let window_trades = &trades[..window_size];

        let returns: Vec<f64> = window_trades
            .iter()
            .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
            .collect();

        let cumulative_return: f64 = returns.iter().sum();
        let cumulative_sharpe = calculate_sharpe_from_returns(&returns, risk_free_rate);
        let max_drawdown = calculate_max_drawdown(&returns);

        let window_start = window_trades.first().map(|t| t.entry_time).unwrap();
        let window_end = window_trades.last().map(|t| t.exit_time).unwrap();

        results.push(ExpandingWindowResult {
            window_size,
            cumulative_sharpe,
            cumulative_return,
            max_drawdown,
            window_start,
            window_end,
        });
    }

    results
}

/// Analyze rolling stability
///
/// # Arguments
/// * `trades` - Vector of trades sorted by entry time
/// * `risk_free_rate` - Annual risk-free rate
/// * `window_size` - Rolling window size (in bars)
/// * `step_size` - Step size between windows (default: 1)
pub fn analyze_rolling_stability(
    trades: &[Trade],
    risk_free_rate: f64,
    window_size: usize,
    step_size: usize,
) -> RollingStabilityResult {
    if trades.len() < window_size {
        return RollingStabilityResult {
            window_size,
            mean_sharpe: 0.0,
            std_sharpe: 0.0,
            cv_sharpe: 0.0,
            min_sharpe: 0.0,
            max_sharpe: 0.0,
            n_windows: 0,
            rating: StabilityRating::VeryPoor,
        };
    }

    let mut sharpes = Vec::new();
    let step_size = step_size.max(1);

    for start in (0..trades.len() - window_size + 1).step_by(step_size) {
        let window_trades = &trades[start..start + window_size];

        let returns: Vec<f64> = window_trades
            .iter()
            .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
            .collect();

        let sharpe = calculate_sharpe_from_returns(&returns, risk_free_rate);
        sharpes.push(sharpe);
    }

    let n_windows = sharpes.len();
    if n_windows == 0 {
        return RollingStabilityResult {
            window_size,
            mean_sharpe: 0.0,
            std_sharpe: 0.0,
            cv_sharpe: 0.0,
            min_sharpe: 0.0,
            max_sharpe: 0.0,
            n_windows: 0,
            rating: StabilityRating::VeryPoor,
        };
    }

    let mean_sharpe: f64 = sharpes.iter().sum::<f64>() / n_windows as f64;
    let variance: f64 = sharpes
        .iter()
        .map(|s| (s - mean_sharpe).powi(2))
        .sum::<f64>()
        / n_windows as f64;
    let std_sharpe = variance.sqrt();
    let cv_sharpe = if mean_sharpe.abs() > 1e-10 {
        std_sharpe / mean_sharpe.abs()
    } else {
        std_sharpe
    };

    let min_sharpe = *sharpes
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(&0.0);
    let max_sharpe = *sharpes
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(&0.0);

    let rating = match cv_sharpe {
        cv if cv < 0.25 => StabilityRating::Excellent,
        cv if cv < 0.50 => StabilityRating::Good,
        cv if cv < 0.75 => StabilityRating::Moderate,
        cv if cv < 1.0 => StabilityRating::Poor,
        _ => StabilityRating::VeryPoor,
    };

    RollingStabilityResult {
        window_size,
        mean_sharpe,
        std_sharpe,
        cv_sharpe,
        min_sharpe,
        max_sharpe,
        n_windows,
        rating,
    }
}

/// Decompose performance by period
///
/// # Arguments
/// * `trades` - Vector of trades sorted by entry time
/// * `risk_free_rate` - Annual risk-free rate
/// * `period_type` - Type of period to decompose by
pub fn decompose_by_period(
    trades: &[Trade],
    risk_free_rate: f64,
    period_type: PeriodType,
) -> Vec<PeriodResult> {
    if trades.is_empty() {
        return Vec::new();
    }

    // Group trades by period
    let mut period_trades: std::collections::HashMap<String, Vec<&Trade>> =
        std::collections::HashMap::new();

    for trade in trades {
        let period_id = match period_type {
            PeriodType::Year => format!("{}", trade.entry_time.year()),
            PeriodType::Quarter => format!(
                "{}-Q{}",
                trade.entry_time.year(),
                (trade.entry_time.month() - 1) / 3 + 1
            ),
            PeriodType::Month => format!(
                "{}-{:02}",
                trade.entry_time.year(),
                trade.entry_time.month()
            ),
        };

        period_trades
            .entry(period_id)
            .or_insert_with(Vec::new)
            .push(trade);
    }

    let mut results = Vec::new();

    for (period_id, period_trade_list) in period_trades {
        if period_trade_list.is_empty() {
            continue;
        }

        let returns: Vec<f64> = period_trade_list
            .iter()
            .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
            .collect();

        let period_return: f64 = returns.iter().sum();
        let sharpe = calculate_sharpe_from_returns(&returns, risk_free_rate);
        let max_drawdown = calculate_max_drawdown(&returns);

        let start = period_trade_list.first().map(|t| t.entry_time).unwrap();
        let end = period_trade_list.last().map(|t| t.exit_time).unwrap();

        results.push(PeriodResult {
            period_type: period_type.clone(),
            period_id,
            start,
            end,
            n_trades: period_trade_list.len(),
            sharpe,
            period_return,
            max_drawdown,
        });
    }

    results.sort_by(|a, b| a.start.cmp(&b.start));
    results
}

/// Analyze performance across market cycles
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `price_data` - Price data for market cycle detection (optional)
/// * `risk_free_rate` - Annual risk-free rate
pub fn analyze_market_cycles(
    trades: &[Trade],
    _price_data: Option<&[f64]>,
    risk_free_rate: f64,
) -> Vec<MarketCycleResult> {
    if trades.is_empty() {
        return Vec::new();
    }

    // For now, use simple price trend detection if price data is provided
    // This is a simplified implementation - Phase 14 will add ML-based detection

    let mut results = Vec::new();

    // If no price data, create a single cycle result
    let returns: Vec<f64> = trades
        .iter()
        .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
        .collect();

    let period_return: f64 = returns.iter().sum();
    let sharpe = calculate_sharpe_from_returns(&returns, risk_free_rate);
    let max_drawdown = calculate_max_drawdown(&returns);

    let start = trades.first().map(|t| t.entry_time).unwrap();
    let end = trades.last().map(|t| t.exit_time).unwrap();
    let duration = (end - start).num_days();

    // Determine cycle type based on return
    let cycle_type = if period_return > 0.1 {
        MarketCycleType::Bull
    } else if period_return < -0.1 {
        MarketCycleType::Bear
    } else {
        MarketCycleType::Sideways
    };

    results.push(MarketCycleResult {
        cycle_type,
        start,
        end,
        duration_days: duration,
        sharpe,
        cycle_return: period_return,
        max_drawdown,
        n_trades: trades.len(),
    });

    results
}

/// Forward-looking validation
///
/// # Arguments
/// * `backtest_sharpe` - Sharpe from backtest
/// * `paper_sharpe` - Sharpe from paper trading (optional)
/// * `live_sharpe` - Sharpe from live trading (optional)
pub fn forward_looking_validation(
    backtest_sharpe: f64,
    paper_sharpe: Option<f64>,
    live_sharpe: Option<f64>,
) -> ForwardLookingResult {
    let backtest_to_paper_degradation = paper_sharpe.map(|ps| {
        if backtest_sharpe.abs() > 1e-10 {
            ((backtest_sharpe - ps) / backtest_sharpe.abs()) * 100.0
        } else {
            0.0
        }
    });

    let paper_to_live_degradation = match (paper_sharpe, live_sharpe) {
        (Some(ps), Some(ls)) => {
            if ps.abs() > 1e-10 {
                Some(((ps - ls) / ps.abs()) * 100.0)
            } else {
                Some(0.0)
            }
        }
        _ => None,
    };

    let rating = if let Some(live_sharpe) = live_sharpe {
        // Have live data
        if live_sharpe > 0.5 * backtest_sharpe {
            ForwardLookingRating::Excellent
        } else if live_sharpe > 0.0 {
            ForwardLookingRating::Good
        } else {
            ForwardLookingRating::Poor
        }
    } else if let Some(paper_sharpe) = paper_sharpe {
        // Have paper data only
        if paper_sharpe > 0.7 * backtest_sharpe {
            ForwardLookingRating::Good
        } else if paper_sharpe > 0.3 * backtest_sharpe {
            ForwardLookingRating::Moderate
        } else {
            ForwardLookingRating::Poor
        }
    } else {
        // No forward data
        ForwardLookingRating::InsufficientData
    };

    ForwardLookingResult {
        backtest_sharpe,
        paper_sharpe,
        live_sharpe,
        backtest_to_paper_degradation,
        paper_to_live_degradation,
        rating,
    }
}

/// Run comprehensive temporal validation
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `metrics` - Performance metrics
/// * `risk_free_rate` - Annual risk-free rate
/// * `paper_sharpe` - Optional paper trading Sharpe
/// * `live_sharpe` - Optional live trading Sharpe
pub fn validate_temporal(
    trades: &[Trade],
    metrics: &PerformanceMetrics,
    risk_free_rate: f64,
    paper_sharpe: Option<f64>,
    live_sharpe: Option<f64>,
) -> TemporalValidationResult {
    // Expanding window analysis (use multiple window sizes)
    let n_trades = trades.len();
    let window_sizes = if n_trades > 0 {
        let step = (n_trades / 5).max(10);
        (step..=n_trades).step_by(step).collect()
    } else {
        Vec::new()
    };
    let expanding_windows = analyze_expanding_windows(trades, risk_free_rate, window_sizes);

    // Rolling stability analysis (6-month window)
    let window_size = (n_trades as f64 * 0.5) as usize;
    let rolling_stability = analyze_rolling_stability(trades, risk_free_rate, window_size, 1);

    // Period decomposition (by year)
    let periods = decompose_by_period(trades, risk_free_rate, PeriodType::Year);

    // Market cycle analysis
    let market_cycles = analyze_market_cycles(trades, None, risk_free_rate);

    // Forward-looking validation
    let forward_looking =
        forward_looking_validation(metrics.sharpe_ratio, paper_sharpe, live_sharpe);

    TemporalValidationResult {
        expanding_windows,
        rolling_stability,
        periods,
        market_cycles,
        forward_looking,
    }
}

/// Helper: Calculate Sharpe ratio from returns
fn calculate_sharpe_from_returns(returns: &[f64], risk_free_rate: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;

    let mut variance = 0.0;
    for &r in returns {
        variance += (r - mean_return).powi(2);
    }
    variance /= (returns.len() - 1) as f64;

    let std_dev = variance.sqrt();

    if std_dev < 1e-10 {
        0.0
    } else {
        // Annualize (assuming daily returns)
        let annual_mean = mean_return * 252.0;
        let annual_std = std_dev * (252.0_f64).sqrt();
        (annual_mean - risk_free_rate) / annual_std
    }
}

/// Helper: Calculate maximum drawdown from returns
fn calculate_max_drawdown(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let mut cumulative = 0.0;
    let mut peak: f64 = 0.0;
    let mut max_drawdown: f64 = 0.0;

    for &r in returns {
        cumulative += r;
        peak = peak.max(cumulative);
        let drawdown = (peak - cumulative) / peak.max(1.0).abs();
        max_drawdown = max_drawdown.max(drawdown);
    }

    max_drawdown
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use rust_decimal::Decimal;

    fn create_test_trades() -> Vec<Trade> {
        let start_time =
            DateTime::from_utc(DateTime::from_timestamp(1000000, 0).unwrap(), Utc);

        (0..20)
            .map(|i| Trade {
                id: String::new(),
                symbol: "BTC".to_string(),
                entry_price: 100.0 + i as f64,
                exit_price: 105.0 + i as f64,
                quantity: Decimal::from(10),
                entry_time: start_time + Duration::days(i),
                exit_time: start_time + Duration::days(i + 1),
                side: crate::trade::TradeSide::Long,
            })
            .collect()
    }

    #[test]
    fn test_analyze_expanding_windows() {
        let trades = create_test_trades();
        let window_sizes = vec![5, 10, 15, 20];
        let results = analyze_expanding_windows(&trades, 0.02, window_sizes);

        assert_eq!(results.len(), 4);
        assert_eq!(results[0].window_size, 5);
        assert_eq!(results[3].window_size, 20);
    }

    #[test]
    fn test_analyze_rolling_stability() {
        let trades = create_test_trades();
        let result = analyze_rolling_stability(&trades, 0.02, 10, 1);

        assert_eq!(result.window_size, 10);
        assert!(result.mean_sharpe.is_finite());
        assert!(result.std_sharpe >= 0.0);
        assert!(result.n_windows > 0);
    }

    #[test]
    fn test_decompose_by_period() {
        let trades = create_test_trades();
        let results = decompose_by_period(&trades, 0.02, PeriodType::Year);

        assert!(!results.is_empty());
        assert_eq!(results[0].period_type, PeriodType::Year);
    }

    #[test]
    fn test_analyze_market_cycles() {
        let trades = create_test_trades();
        let results = analyze_market_cycles(&trades, None, 0.02);

        assert_eq!(results.len(), 1);
        assert!(results[0].n_trades > 0);
    }

    #[test]
    fn test_forward_looking_validation() {
        let result = forward_looking_validation(1.5, Some(1.2), Some(1.0));

        assert_eq!(result.backtest_sharpe, 1.5);
        assert_eq!(result.paper_sharpe, Some(1.2));
        assert_eq!(result.live_sharpe, Some(1.0));
        assert!(result.backtest_to_paper_degradation.is_some());
        assert!(result.paper_to_live_degradation.is_some());
    }

    #[test]
    fn test_validate_temporal() {
        let trades = create_test_trades();
        let metrics = PerformanceMetrics {
            total_return: 10.0,
            sharpe_ratio: 1.5,
            sortino_ratio: 1.5,
            max_drawdown: 5.0,
            win_rate: 0.6,
            profit_factor: 2.0,
            average_win: 100.0,
            average_loss: -50.0,
            total_trades: 20,
            total_wins: 12,
            total_losses: 8,
            calmar_ratio: 1.2,
            omega_ratio: 1.8,
        };

        let result = validate_temporal(&trades, &metrics, 0.02, None, None);

        assert!(!result.expanding_windows.is_empty());
        assert_eq!(result.rolling_stability.n_windows > 0, true);
        assert!(!result.periods.is_empty());
    }
}
