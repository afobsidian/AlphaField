//! # Robustness Validation
//!
//! Tests the robustness of trading strategies including complexity penalties,
//! data perturbation, multiple timeframe testing, cross-asset validation,
//! and outlier impact analysis.

use crate::metrics::PerformanceMetrics;
use crate::trade::Trade;
use chrono::{DateTime, Datelike, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Complexity penalty result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityResult {
    /// Number of parameters in strategy
    pub n_parameters: usize,
    /// Number of technical indicators used
    pub n_indicators: usize,
    /// Number of decision branches in logic
    pub n_branches: usize,
    /// Calculated complexity penalty score (0-1, higher is worse)
    pub penalty_score: f64,
    /// Complexity rating
    pub rating: ComplexityRating,
}

/// Data perturbation test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerturbationResult {
    /// Noise level used (as percentage, e.g., 0.01 for 1%)
    pub noise_level: f64,
    /// Original Sharpe ratio
    pub original_sharpe: f64,
    /// Perturbed Sharpe ratio
    pub perturbed_sharpe: f64,
    /// Degradation percentage
    pub degradation_pct: f64,
    /// Is degradation acceptable (< 20%)?
    pub is_acceptable: bool,
}

/// Multiple timeframe test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeframeResult {
    /// Timeframe tested (e.g., "1h", "4h", "1d")
    pub timeframe: String,
    /// Sharpe ratio on this timeframe
    pub sharpe: f64,
    /// Win rate on this timeframe
    pub win_rate: f64,
    /// Max drawdown on this timeframe
    pub max_drawdown: f64,
    /// Total trades on this timeframe
    pub total_trades: usize,
}

/// Cross-asset validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossAssetResult {
    /// Asset symbol tested
    pub symbol: String,
    /// Sharpe ratio on this asset
    pub sharpe: f64,
    /// Win rate on this asset
    pub win_rate: f64,
    /// Max drawdown on this asset
    pub max_drawdown: f64,
    /// Correlation with primary asset (if available)
    pub correlation: Option<f64>,
}

/// Outlier impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierResult {
    /// Original Sharpe ratio
    pub original_sharpe: f64,
    /// Sharpe after removing top 10% trades
    pub sharpe_no_top: f64,
    /// Sharpe after removing bottom 10% trades
    pub sharpe_no_bottom: f64,
    /// Sharpe after removing both top and bottom 10%
    pub sharpe_no_outliers: f64,
    /// Impact of top trades on Sharpe
    pub top_impact_pct: f64,
    /// Impact of bottom trades on Sharpe
    pub bottom_impact_pct: f64,
    /// Overall outlier dependency (0-1, higher is more dependent)
    pub outlier_dependency: f64,
}

/// Complexity rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityRating {
    /// Very simple, highly robust
    Excellent,
    /// Simple, robust
    Good,
    /// Moderate complexity
    Moderate,
    /// Complex, may overfit
    Poor,
    /// Very complex, likely overfitted
    VeryPoor,
}

/// Comprehensive robustness validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobustnessResult {
    /// Complexity penalty analysis
    pub complexity: ComplexityResult,
    /// Data perturbation tests
    pub perturbations: Vec<PerturbationResult>,
    /// Multiple timeframe analysis
    pub timeframes: Vec<TimeframeResult>,
    /// Cross-asset validation
    pub cross_assets: Vec<CrossAssetResult>,
    /// Outlier impact analysis
    pub outlier: OutlierResult,
}

impl RobustnessResult {
    /// Calculate overall robustness score (0-100)
    pub fn overall_score(&self) -> f64 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;

        // Complexity: lower penalty is better
        let complexity_score = match self.complexity.rating {
            ComplexityRating::Excellent => 100.0,
            ComplexityRating::Good => 80.0,
            ComplexityRating::Moderate => 60.0,
            ComplexityRating::Poor => 40.0,
            ComplexityRating::VeryPoor => 20.0,
        };
        score += complexity_score * 0.15;
        weight_sum += 0.15;

        // Perturbation: average degradation (lower is better)
        if !self.perturbations.is_empty() {
            let avg_degradation: f64 = self
                .perturbations
                .iter()
                .map(|p| p.degradation_pct)
                .sum::<f64>()
                / self.perturbations.len() as f64;
            let perturbation_score = 100.0 * (1.0 - (avg_degradation / 100.0).min(1.0));
            score += perturbation_score * 0.25;
            weight_sum += 0.25;
        }

        // Timeframes: Sharpe standard deviation (lower is better)
        if self.timeframes.len() > 1 {
            let sharpes: Vec<f64> = self.timeframes.iter().map(|t| t.sharpe).collect();
            let mean_sharpe: f64 = sharpes.iter().sum::<f64>() / sharpes.len() as f64;
            let variance: f64 = sharpes
                .iter()
                .map(|s| (s - mean_sharpe).powi(2))
                .sum::<f64>()
                / sharpes.len() as f64;
            let std_dev = variance.sqrt();
            // Normalize: std_dev of 0 = 100, std_dev of 2 = 0
            let timeframe_score = 100.0 * (1.0 - (std_dev / 2.0).min(1.0));
            score += timeframe_score * 0.20;
            weight_sum += 0.20;
        } else if !self.timeframes.is_empty() {
            // Single timeframe: check if Sharpe is positive
            let timeframe_score = if self.timeframes[0].sharpe > 0.0 {
                80.0
            } else {
                40.0
            };
            score += timeframe_score * 0.20;
            weight_sum += 0.20;
        }

        // Cross-asset: average Sharpe (higher is better)
        if !self.cross_assets.is_empty() {
            let avg_sharpe: f64 = self.cross_assets.iter().map(|c| c.sharpe).sum::<f64>()
                / self.cross_assets.len() as f64;
            // Normalize: Sharpe of 0 = 50, Sharpe of 2 = 100
            let cross_asset_score = 50.0 + 25.0 * (avg_sharpe / 2.0).min(2.0).max(-2.0);
            score += cross_asset_score * 0.20;
            weight_sum += 0.20;
        }

        // Outlier dependency: lower is better
        let outlier_score = 100.0 * (1.0 - self.outlier.outlier_dependency);
        score += outlier_score * 0.20;
        weight_sum += 0.20;

        if weight_sum > 0.0 {
            score / weight_sum
        } else {
            score
        }
    }
}

/// Calculate complexity penalty for a strategy
///
/// # Arguments
/// * `n_parameters` - Number of tunable parameters
/// * `n_indicators` - Number of technical indicators
/// * `n_branches` - Number of decision branches
pub fn calculate_complexity(
    n_parameters: usize,
    n_indicators: usize,
    n_branches: usize,
) -> ComplexityResult {
    // Complexity score based on normalized values
    // Assume reasonable maximums: 10 parameters, 5 indicators, 20 branches
    let param_score = (n_parameters as f64 / 10.0).min(1.0);
    let indicator_score = (n_indicators as f64 / 5.0).min(1.0);
    let branch_score = (n_branches as f64 / 20.0).min(1.0);

    let penalty_score = (param_score * 0.4 + indicator_score * 0.3 + branch_score * 0.3).min(1.0);

    let rating = match penalty_score {
        s if s < 0.2 => ComplexityRating::Excellent,
        s if s < 0.4 => ComplexityRating::Good,
        s if s < 0.6 => ComplexityRating::Moderate,
        s if s < 0.8 => ComplexityRating::Poor,
        _ => ComplexityRating::VeryPoor,
    };

    ComplexityResult {
        n_parameters,
        n_indicators,
        n_branches,
        penalty_score,
        rating,
    }
}

/// Test strategy robustness to data perturbation
///
/// # Arguments
/// * `original_sharpe` - Original Sharpe ratio
/// * `perturbed_sharpe` - Sharpe ratio after perturbation
/// * `noise_level` - Noise level used (as percentage)
pub fn test_perturbation(
    original_sharpe: f64,
    perturbed_sharpe: f64,
    noise_level: f64,
) -> PerturbationResult {
    let degradation_pct = if original_sharpe > 0.0 {
        ((original_sharpe - perturbed_sharpe) / original_sharpe.abs()) * 100.0
    } else {
        (perturbed_sharpe - original_sharpe).abs() * 100.0
    };

    let is_acceptable = degradation_pct < 20.0;

    PerturbationResult {
        noise_level,
        original_sharpe,
        perturbed_sharpe,
        degradation_pct,
        is_acceptable,
    }
}

/// Add noise to a vector of values
///
/// # Arguments
/// * `values` - Original values
/// * `noise_level` - Noise level as percentage (e.g., 0.01 for 1%)
pub fn add_noise(values: &mut [f64], noise_level: f64) {
    let mut rng = rand::thread_rng();
    for v in values.iter_mut() {
        let noise = rng.gen_range(-noise_level..noise_level) * *v;
        *v += noise;
    }
}

/// Calculate Sharpe ratio from trade returns
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

/// Analyze outlier impact on performance
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `risk_free_rate` - Annual risk-free rate
pub fn analyze_outliers(trades: &[Trade], risk_free_rate: f64) -> OutlierResult {
    if trades.is_empty() {
        return OutlierResult {
            original_sharpe: 0.0,
            sharpe_no_top: 0.0,
            sharpe_no_bottom: 0.0,
            sharpe_no_outliers: 0.0,
            top_impact_pct: 0.0,
            bottom_impact_pct: 0.0,
            outlier_dependency: 0.0,
        };
    }

    // Calculate original Sharpe
    let original_returns: Vec<f64> = trades
        .iter()
        .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
        .collect();
    let original_sharpe = calculate_sharpe_from_returns(&original_returns, risk_free_rate);

    if trades.len() < 5 {
        return OutlierResult {
            original_sharpe,
            sharpe_no_top: original_sharpe,
            sharpe_no_bottom: original_sharpe,
            sharpe_no_outliers: original_sharpe,
            top_impact_pct: 0.0,
            bottom_impact_pct: 0.0,
            outlier_dependency: 0.0,
        };
    }

    // Calculate returns with trade indices
    let mut indexed_returns: Vec<(usize, f64)> = original_returns
        .iter()
        .enumerate()
        .map(|(i, &r)| (i, r))
        .collect();

    // Sort by return
    indexed_returns.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let n = trades.len();
    let n_outliers = (n as f64 * 0.1) as usize;

    // Remove top outliers (best trades)
    let mut returns_no_top: Vec<f64> = original_returns.clone();
    if n_outliers > 0 {
        let top_outlier_indices: Vec<usize> = indexed_returns
            .iter()
            .rev()
            .take(n_outliers)
            .map(|(i, _)| *i)
            .collect();
        returns_no_top = original_returns
            .iter()
            .enumerate()
            .filter(|(i, _)| !top_outlier_indices.contains(i))
            .map(|(_, &r)| r)
            .collect();
    }
    let sharpe_no_top = calculate_sharpe_from_returns(&returns_no_top, risk_free_rate);

    // Remove bottom outliers (worst trades)
    let mut returns_no_bottom: Vec<f64> = original_returns.clone();
    if n_outliers > 0 {
        let bottom_outlier_indices: Vec<usize> = indexed_returns
            .iter()
            .take(n_outliers)
            .map(|(i, _)| *i)
            .collect();
        returns_no_bottom = original_returns
            .iter()
            .enumerate()
            .filter(|(i, _)| !bottom_outlier_indices.contains(i))
            .map(|(_, &r)| r)
            .collect();
    }
    let sharpe_no_bottom = calculate_sharpe_from_returns(&returns_no_bottom, risk_free_rate);

    // Remove both top and bottom outliers
    let mut returns_no_outliers: Vec<f64> = original_returns.clone();
    if n_outliers > 0 {
        let all_outlier_indices: Vec<usize> = indexed_returns
            .iter()
            .take(n_outliers)
            .chain(indexed_returns.iter().rev().take(n_outliers))
            .map(|(i, _)| *i)
            .collect();
        returns_no_outliers = original_returns
            .iter()
            .enumerate()
            .filter(|(i, _)| !all_outlier_indices.contains(i))
            .map(|(_, &r)| r)
            .collect();
    }
    let sharpe_no_outliers = calculate_sharpe_from_returns(&returns_no_outliers, risk_free_rate);

    // Calculate impact percentages
    let top_impact_pct = if original_sharpe != 0.0 {
        ((original_sharpe - sharpe_no_top) / original_sharpe.abs()) * 100.0
    } else {
        0.0
    };

    let bottom_impact_pct = if original_sharpe != 0.0 {
        ((original_sharpe - sharpe_no_bottom) / original_sharpe.abs()) * 100.0
    } else {
        0.0
    };

    // Outlier dependency: how much Sharpe changes without outliers
    let outlier_dependency = if original_sharpe.abs() > 1e-10 {
        ((original_sharpe - sharpe_no_outliers).abs() / original_sharpe.abs()).min(1.0)
    } else {
        0.0
    };

    OutlierResult {
        original_sharpe,
        sharpe_no_top,
        sharpe_no_bottom,
        sharpe_no_outliers,
        top_impact_pct,
        bottom_impact_pct,
        outlier_dependency,
    }
}

/// Run comprehensive robustness validation
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `metrics` - Performance metrics
/// * `risk_free_rate` - Annual risk-free rate
/// * `strategy_params` - Strategy configuration for complexity analysis
pub fn validate_robustness(
    trades: &[Trade],
    metrics: &PerformanceMetrics,
    risk_free_rate: f64,
    strategy_params: &StrategyParams,
) -> RobustnessResult {
    // Complexity penalty
    let complexity = calculate_complexity(
        strategy_params.n_parameters,
        strategy_params.n_indicators,
        strategy_params.n_branches,
    );

    // Data perturbation tests (test at multiple noise levels)
    let mut perturbations = Vec::new();
    let original_returns: Vec<f64> = trades
        .iter()
        .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
        .collect();
    let original_sharpe = metrics.sharpe_ratio;

    for &noise_level in &[0.01, 0.02, 0.05] {
        let mut perturbed_returns = original_returns.clone();
        add_noise(&mut perturbed_returns, noise_level);
        let perturbed_sharpe = calculate_sharpe_from_returns(&perturbed_returns, risk_free_rate);
        perturbations.push(test_perturbation(
            original_sharpe,
            perturbed_sharpe,
            noise_level,
        ));
    }

    // Multiple timeframe analysis (placeholder - requires data for different timeframes)
    let timeframes = strategy_params.timeframe_results.clone();

    // Cross-asset validation (placeholder - requires data for other assets)
    let cross_assets = strategy_params.cross_asset_results.clone();

    // Outlier impact analysis
    let outlier = analyze_outliers(trades, risk_free_rate);

    RobustnessResult {
        complexity,
        perturbations,
        timeframes,
        cross_assets,
        outlier,
    }
}

/// Strategy parameters for complexity analysis
#[derive(Debug, Clone, Default)]
pub struct StrategyParams {
    /// Number of tunable parameters
    pub n_parameters: usize,
    /// Number of technical indicators
    pub n_indicators: usize,
    /// Number of decision branches
    pub n_branches: usize,
    /// Results from different timeframes (if available)
    pub timeframe_results: Vec<TimeframeResult>,
    /// Results from cross-asset tests (if available)
    pub cross_asset_results: Vec<CrossAssetResult>,
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

    fn create_metrics(sharpe: f64) -> PerformanceMetrics {
        PerformanceMetrics {
            total_return: 10.0,
            sharpe_ratio: sharpe,
            sortino_ratio: 1.5,
            max_drawdown: 5.0,
            win_rate: 0.6,
            profit_factor: 2.0,
            average_win: 100.0,
            average_loss: -50.0,
            total_trades: 10,
            total_wins: 6,
            total_losses: 4,
            calmar_ratio: sharpe * 0.8,
            omega_ratio: 1.8,
        }
    }

    #[test]
    fn test_calculate_complexity() {
        let result = calculate_complexity(5, 3, 10);
        assert_eq!(result.n_parameters, 5);
        assert_eq!(result.n_indicators, 3);
        assert_eq!(result.n_branches, 10);
        assert!(result.penalty_score >= 0.0 && result.penalty_score <= 1.0);
    }

    #[test]
    fn test_complexity_ratings() {
        let excellent = calculate_complexity(1, 1, 2);
        assert!(matches!(excellent.rating, ComplexityRating::Excellent));

        let moderate = calculate_complexity(5, 3, 10);
        assert!(matches!(
            moderate.rating,
            ComplexityRating::Good | ComplexityRating::Moderate
        ));

        let very_poor = calculate_complexity(15, 10, 30);
        assert!(matches!(very_poor.rating, ComplexityRating::VeryPoor));
    }

    #[test]
    fn test_test_perturbation() {
        let result = test_perturbation(1.5, 1.2, 0.02);
        assert_eq!(result.original_sharpe, 1.5);
        assert_eq!(result.perturbed_sharpe, 1.2);
        assert_eq!(result.noise_level, 0.02);
        assert!((result.degradation_pct - 20.0).abs() < 0.1);
        assert!(!result.is_acceptable); // 20% is not < 20%
    }

    #[test]
    fn test_analyze_outliers() {
        let trades = create_test_trades();
        let result = analyze_outliers(&trades, 0.02);

        assert!(result.original_sharpe.is_finite());
        assert!(result.sharpe_no_top.is_finite());
        assert!(result.sharpe_no_bottom.is_finite());
        assert!(result.sharpe_no_outliers.is_finite());
        assert!(result.outlier_dependency >= 0.0 && result.outlier_dependency <= 1.0);
    }

    #[test]
    fn test_validate_robustness() {
        let trades = create_test_trades();
        let metrics = create_metrics(1.5);
        let params = StrategyParams::default();

        let result = validate_robustness(&trades, &metrics, 0.02, &params);

        assert_eq!(result.complexity.n_parameters, 0);
        assert_eq!(result.perturbations.len(), 3); // 3 noise levels
        assert!(result.outlier.original_sharpe.is_finite());
    }
}
