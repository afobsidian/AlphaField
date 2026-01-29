//! # Regime Testing
//!
//! Advanced regime-based validation including automatic regime detection,
//! regime-specific backtesting, transition testing, stress testing, and
//! regime prediction models.

use crate::metrics::PerformanceMetrics;
use crate::Trade;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Market regime with automatic detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRegimeDetected {
    /// Regime type
    pub regime_type: RegimeType,
    /// Confidence score (0-1)
    pub confidence: f64,
    /// Start of regime
    pub start: DateTime<Utc>,
    /// End of regime
    pub end: DateTime<Utc>,
    /// Duration in days
    pub duration_days: i64,
    /// Market characteristics
    pub characteristics: RegimeCharacteristics,
}

/// Market regime type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum RegimeType {
    Bull,
    Bear,
    Sideways,
    Volatile,
    Transition,
}

/// Regime characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeCharacteristics {
    /// Average return during regime
    pub avg_return: f64,
    /// Volatility (std dev of returns)
    pub volatility: f64,
    /// Trend strength (0-1)
    pub trend_strength: f64,
    /// Number of regime changes
    pub n_regime_changes: usize,
}

/// Regime-specific backtest result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeSpecificResult {
    /// Regime type tested
    pub regime_type: RegimeType,
    /// Sharpe ratio in this regime
    pub sharpe: f64,
    /// Return in this regime
    pub regime_return: f64,
    /// Max drawdown in this regime
    pub max_drawdown: f64,
    /// Win rate in this regime
    pub win_rate: f64,
    /// Number of trades in this regime
    pub n_trades: usize,
    /// Is performance acceptable in this regime?
    pub is_acceptable: bool,
}

/// Regime transition result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeTransitionResult {
    /// Transition from regime
    pub from_regime: RegimeType,
    /// Transition to regime
    pub to_regime: RegimeType,
    /// Transition time
    pub transition_time: DateTime<Utc>,
    /// Sharpe N bars before transition
    pub sharpe_before: f64,
    /// Sharpe N bars after transition
    pub sharpe_after: f64,
    /// Change in Sharpe
    pub sharpe_change: f64,
    /// Performance impact (positive = better after transition)
    pub impact_score: f64,
}

/// Stress test by regime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    /// Regime type being stressed
    pub regime_type: RegimeType,
    /// Sharpe ratio
    pub sharpe: f64,
    /// Max drawdown
    pub max_drawdown: f64,
    /// Duration of stress test
    pub duration_days: i64,
    /// Stress level (0-1, higher is more stressful)
    pub stress_level: f64,
    /// Did strategy survive (acceptable performance)?
    pub survived: bool,
}

/// Regime prediction result (placeholder for Phase 14 ML models)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimePredictionResult {
    /// Predicted next regime
    pub predicted_regime: RegimeType,
    /// Confidence in prediction (0-1)
    pub confidence: f64,
    /// Time horizon (days)
    pub horizon_days: i64,
    /// Current regime
    pub current_regime: RegimeType,
}

/// Comprehensive regime testing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeTestingResult {
    /// Detected regimes
    pub regimes: Vec<MarketRegimeDetected>,
    /// Regime-specific performance
    pub regime_performance: Vec<RegimeSpecificResult>,
    /// Regime transition analysis
    pub transitions: Vec<RegimeTransitionResult>,
    /// Stress test results
    pub stress_tests: Vec<StressTestResult>,
    /// Regime prediction (placeholder)
    pub prediction: Option<RegimePredictionResult>,
}

impl RegimeTestingResult {
    /// Calculate overall regime testing score (0-100)
    pub fn overall_score(&self) -> f64 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;

        // Regime-specific performance: consistency across regimes
        if !self.regime_performance.is_empty() {
            let sharpes: Vec<f64> = self.regime_performance.iter().map(|r| r.sharpe).collect();
            let positive_regimes = sharpes.iter().filter(|&&s| s > 0.0).count();

            // Good performance in multiple regimes
            let regime_score = if positive_regimes >= self.regime_performance.len() / 2 {
                80.0 + (positive_regimes as f64 / self.regime_performance.len() as f64) * 20.0
            } else if positive_regimes > 0 {
                50.0 + (positive_regimes as f64 / self.regime_performance.len() as f64) * 30.0
            } else {
                20.0
            };
            score += regime_score * 0.35;
            weight_sum += 0.35;
        }

        // Transitions: ability to adapt to regime changes
        if !self.transitions.is_empty() {
            let avg_impact: f64 = self.transitions.iter().map(|t| t.impact_score).sum::<f64>()
                / self.transitions.len() as f64;
            // Positive impact = good adaptation
            let transition_score = 50.0 + avg_impact * 25.0;
            score += transition_score.clamp(0.0, 100.0) * 0.25;
            weight_sum += 0.25;
        }

        // Stress tests: survival in worst regimes
        if !self.stress_tests.is_empty() {
            let survived = self.stress_tests.iter().filter(|s| s.survived).count();
            let stress_score = (survived as f64 / self.stress_tests.len() as f64) * 100.0;
            score += stress_score * 0.25;
            weight_sum += 0.25;
        }

        // Regime detection quality (confidence scores)
        if !self.regimes.is_empty() {
            let avg_confidence: f64 =
                self.regimes.iter().map(|r| r.confidence).sum::<f64>() / self.regimes.len() as f64;
            let detection_score = avg_confidence * 100.0;
            score += detection_score * 0.15;
            weight_sum += 0.15;
        }

        if weight_sum > 0.0 {
            score / weight_sum
        } else {
            score
        }
    }
}

/// Detect market regimes automatically
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `price_data` - Price data for regime detection (optional)
pub fn detect_regimes(trades: &[Trade], _price_data: Option<&[f64]>) -> Vec<MarketRegimeDetected> {
    if trades.is_empty() {
        return Vec::new();
    }

    // For now, use simple return-based detection
    // Phase 14 will add ML-based detection

    let returns: Vec<(DateTime<Utc>, f64)> = trades
        .iter()
        .map(|t| (t.entry_time, (t.exit_price - t.entry_price) / t.entry_price))
        .collect();

    if returns.is_empty() {
        return Vec::new();
    }

    // Calculate rolling statistics for regime detection
    let window_size = (returns.len() / 5).max(10);
    let mut regimes = Vec::new();

    let mut current_regime_type = RegimeType::Sideways;
    let mut regime_start = returns[0].0;

    for (i, &(_, _ret)) in returns.iter().enumerate() {
        // Calculate local statistics
        let start_idx = i.saturating_sub(window_size);
        let end_idx = (i + window_size).min(returns.len());

        if end_idx - start_idx < 5 {
            continue;
        }

        let window_returns: Vec<f64> = returns[start_idx..end_idx]
            .iter()
            .map(|&(_, r)| r)
            .collect();

        let avg_return: f64 = window_returns.iter().sum::<f64>() / window_returns.len() as f64;
        let variance: f64 = window_returns
            .iter()
            .map(|r| (r - avg_return).powi(2))
            .sum::<f64>()
            / window_returns.len() as f64;
        let volatility = variance.sqrt();

        // Determine regime type
        let new_regime_type = if volatility > 0.05 {
            RegimeType::Volatile
        } else if avg_return > 0.01 {
            RegimeType::Bull
        } else if avg_return < -0.01 {
            RegimeType::Bear
        } else {
            RegimeType::Sideways
        };

        // Check for regime change
        if new_regime_type != current_regime_type {
            // Save previous regime
            let duration = (returns[i].0 - regime_start).num_days();
            let characteristics = RegimeCharacteristics {
                avg_return,
                volatility,
                trend_strength: (avg_return / (volatility + 0.01)).clamp(-1.0, 1.0).abs(),
                n_regime_changes: 1,
            };

            regimes.push(MarketRegimeDetected {
                regime_type: current_regime_type,
                confidence: 0.7, // Simplified confidence
                start: regime_start,
                end: returns[i].0,
                duration_days: duration,
                characteristics,
            });

            current_regime_type = new_regime_type;
            regime_start = returns[i].0;
        }
    }

    // Add final regime
    if !returns.is_empty() {
        let duration = (returns.last().unwrap().0 - regime_start).num_days();

        let final_returns: Vec<f64> = returns.iter().map(|&(_, r)| r).collect();
        let avg_return: f64 = final_returns.iter().sum::<f64>() / final_returns.len() as f64;
        let variance: f64 = final_returns
            .iter()
            .map(|r| (r - avg_return).powi(2))
            .sum::<f64>()
            / final_returns.len() as f64;
        let volatility = variance.sqrt();

        let characteristics = RegimeCharacteristics {
            avg_return,
            volatility,
            trend_strength: (avg_return / (volatility + 0.01)).clamp(-1.0, 1.0).abs(),
            n_regime_changes: 0,
        };

        regimes.push(MarketRegimeDetected {
            regime_type: current_regime_type,
            confidence: 0.7,
            start: regime_start,
            end: returns.last().unwrap().0,
            duration_days: duration,
            characteristics,
        });
    }

    regimes
}

/// Test regime-specific performance
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `regimes` - Detected regimes
/// * `risk_free_rate` - Annual risk-free rate
pub fn test_regime_specific(
    trades: &[Trade],
    regimes: &[MarketRegimeDetected],
    risk_free_rate: f64,
) -> Vec<RegimeSpecificResult> {
    if regimes.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();

    for regime in regimes {
        // Filter trades for this regime
        let regime_trades: Vec<&Trade> = trades
            .iter()
            .filter(|t| t.entry_time >= regime.start && t.entry_time < regime.end)
            .collect();

        if regime_trades.is_empty() {
            continue;
        }

        let returns: Vec<f64> = regime_trades
            .iter()
            .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
            .collect();

        let regime_return: f64 = returns.iter().sum();
        let sharpe = calculate_sharpe_from_returns(&returns, risk_free_rate);
        let max_drawdown = calculate_max_drawdown(&returns);

        let wins = regime_trades
            .iter()
            .filter(|t| t.exit_price > t.entry_price)
            .count();
        let win_rate = wins as f64 / regime_trades.len() as f64;

        // Acceptable if Sharpe > 0 and win_rate > 0.4
        let is_acceptable = sharpe > 0.0 && win_rate > 0.4;

        results.push(RegimeSpecificResult {
            regime_type: regime.regime_type,
            sharpe,
            regime_return,
            max_drawdown,
            win_rate,
            n_trades: regime_trades.len(),
            is_acceptable,
        });
    }

    results
}

/// Analyze regime transitions
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `regimes` - Detected regimes
/// * `risk_free_rate` - Annual risk-free rate
/// * `window_bars` - Number of bars before/after transition (default: 5)
pub fn analyze_transitions(
    trades: &[Trade],
    regimes: &[MarketRegimeDetected],
    risk_free_rate: f64,
    window_bars: usize,
) -> Vec<RegimeTransitionResult> {
    if regimes.len() < 2 {
        return Vec::new();
    }

    let mut results = Vec::new();

    for i in 0..regimes.len() - 1 {
        let transition_time = regimes[i].end;
        let from_regime = regimes[i].regime_type;
        let to_regime = regimes[i + 1].regime_type;

        // Get trades before transition
        let trades_before: Vec<&Trade> = trades
            .iter()
            .filter(|t| {
                let days_before =
                    (transition_time - t.entry_time).num_days().unsigned_abs() as usize;
                days_before <= window_bars && t.entry_time <= transition_time
            })
            .collect();

        // Get trades after transition
        let trades_after: Vec<&Trade> = trades
            .iter()
            .filter(|t| {
                let days_after =
                    (t.entry_time - transition_time).num_days().unsigned_abs() as usize;
                days_after <= window_bars && t.entry_time >= transition_time
            })
            .collect();

        let sharpe_before = if !trades_before.is_empty() {
            let returns: Vec<f64> = trades_before
                .iter()
                .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
                .collect();
            calculate_sharpe_from_returns(&returns, risk_free_rate)
        } else {
            0.0
        };

        let sharpe_after = if !trades_after.is_empty() {
            let returns: Vec<f64> = trades_after
                .iter()
                .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
                .collect();
            calculate_sharpe_from_returns(&returns, risk_free_rate)
        } else {
            0.0
        };

        let sharpe_change = sharpe_after - sharpe_before;
        let impact_score = if sharpe_before.abs() > 1e-10 {
            sharpe_change / sharpe_before.abs()
        } else {
            sharpe_change
        };

        results.push(RegimeTransitionResult {
            from_regime,
            to_regime,
            transition_time,
            sharpe_before,
            sharpe_after,
            sharpe_change,
            impact_score,
        });
    }

    results
}

/// Stress test strategy by regime
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `regimes` - Detected regimes
/// * `risk_free_rate` - Annual risk-free rate
pub fn stress_test_regimes(
    trades: &[Trade],
    regimes: &[MarketRegimeDetected],
    risk_free_rate: f64,
) -> Vec<StressTestResult> {
    // Focus on worst regimes (most negative returns, highest drawdowns)
    let mut regime_results: Vec<StressTestResult> = Vec::new();

    for regime in regimes {
        let regime_trades: Vec<&Trade> = trades
            .iter()
            .filter(|t| t.entry_time >= regime.start && t.entry_time < regime.end)
            .collect();

        if regime_trades.is_empty() {
            continue;
        }

        let returns: Vec<f64> = regime_trades
            .iter()
            .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
            .collect();

        let regime_return: f64 = returns.iter().sum();
        let sharpe = calculate_sharpe_from_returns(&returns, risk_free_rate);
        let max_drawdown = calculate_max_drawdown(&returns);

        // Calculate stress level based on return and drawdown
        let stress_level = ((regime_return.abs() + max_drawdown) / 2.0).min(1.0);

        // Survive if Sharpe > -1.0 and max_drawdown < 0.3
        let survived = sharpe > -1.0 && max_drawdown < 0.3;

        regime_results.push(StressTestResult {
            regime_type: regime.regime_type,
            sharpe,
            max_drawdown,
            duration_days: regime.duration_days,
            stress_level,
            survived,
        });
    }

    // Sort by stress level (most stressful first)
    regime_results.sort_by(|a, b| {
        b.stress_level
            .partial_cmp(&a.stress_level)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    regime_results
}

/// Predict next regime (placeholder for Phase 14)
///
/// # Arguments
/// * `current_regime` - Current regime
/// * `regimes` - Historical regimes
pub fn predict_regime(
    current_regime: RegimeType,
    regimes: &[MarketRegimeDetected],
) -> RegimePredictionResult {
    if regimes.is_empty() {
        return RegimePredictionResult {
            predicted_regime: RegimeType::Sideways,
            confidence: 0.3,
            horizon_days: 30,
            current_regime,
        };
    }

    // Simple prediction: next regime is most common transition from current
    let mut transition_counts: std::collections::HashMap<RegimeType, usize> =
        std::collections::HashMap::new();

    for i in 0..regimes.len() - 1 {
        if regimes[i].regime_type == current_regime {
            *transition_counts
                .entry(regimes[i + 1].regime_type)
                .or_insert(0) += 1;
        }
    }

    let predicted_regime =
        if let Some((regime, _)) = transition_counts.iter().max_by_key(|&(_, count)| count) {
            *regime
        } else {
            RegimeType::Sideways
        };

    let confidence = if let Some(&max_count) = transition_counts.values().max() {
        let total_transitions: usize = transition_counts.values().sum();
        (max_count as f64 / total_transitions as f64).max(0.3)
    } else {
        0.3
    };

    RegimePredictionResult {
        predicted_regime,
        confidence,
        horizon_days: 30,
        current_regime,
    }
}

/// Run comprehensive regime testing
///
/// # Arguments
/// * `trades` - Vector of trades
/// * `metrics` - Performance metrics
/// * `risk_free_rate` - Annual risk-free rate
pub fn validate_regime_testing(
    trades: &[Trade],
    _metrics: &PerformanceMetrics,
    risk_free_rate: f64,
) -> RegimeTestingResult {
    // Detect regimes
    let regimes = detect_regimes(trades, None);

    // Regime-specific performance
    let regime_performance = test_regime_specific(trades, &regimes, risk_free_rate);

    // Regime transitions
    let transitions = analyze_transitions(trades, &regimes, risk_free_rate, 5);

    // Stress tests
    let stress_tests = stress_test_regimes(trades, &regimes, risk_free_rate);

    // Regime prediction (placeholder)
    let start_time_regime = regimes
        .last()
        .map(|r| r.regime_type)
        .unwrap_or(RegimeType::Sideways);
    let prediction = Some(predict_regime(start_time_regime, &regimes));

    RegimeTestingResult {
        regimes,
        regime_performance,
        transitions,
        stress_tests,
        prediction,
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
