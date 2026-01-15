//! # Regime Analysis Module
//!
//! Detects market regimes in historical data and analyzes strategy
//! performance across different market conditions.

use crate::metrics::PerformanceMetrics;

use alphafield_core::{Bar, QuantError as CoreError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Market regime classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketRegime {
    /// Bull market (upward trend)
    Bull,
    /// Bear market (downward trend)
    Bear,
    /// Sideways market (no clear trend)
    Sideways,
    /// High volatility regime
    HighVolatility,
    /// Low volatility regime
    LowVolatility,
    /// Trending regime (either up or down)
    Trending,
    /// Ranging regime (sideways with low volatility)
    Ranging,
}

impl MarketRegime {
    /// Convert regime to display string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bull => "Bull",
            Self::Bear => "Bear",
            Self::Sideways => "Sideways",
            Self::HighVolatility => "High Volatility",
            Self::LowVolatility => "Low Volatility",
            Self::Trending => "Trending",
            Self::Ranging => "Ranging",
        }
    }
}

/// Performance metrics for a specific regime
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegimePerformance {
    /// Total return in this regime
    pub total_return: f64,
    /// Sharpe ratio in this regime
    pub sharpe_ratio: f64,
    /// Max drawdown in this regime
    pub max_drawdown: f64,
    /// Win rate in this regime
    pub win_rate: f64,
    /// Number of trades in this regime
    pub total_trades: usize,
    /// Percentage of time spent in this regime
    pub time_in_regime: f64,
    /// Average return per trade
    pub avg_return_per_trade: f64,
}

impl RegimePerformance {
    /// Create empty regime performance
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if performance meets minimum thresholds
    pub fn is_acceptable(&self, min_sharpe: f64, max_drawdown: f64) -> bool {
        self.sharpe_ratio >= min_sharpe && self.max_drawdown <= max_drawdown
    }
}

/// Regime mismatch warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeMismatch {
    /// Strategy performs best in this regime
    pub best_performing_regime: MarketRegime,
    /// Strategy expected to perform in this regime (from metadata)
    pub expected_regime: MarketRegime,
    /// Performance difference
    pub performance_gap: f64,
    /// Warning message
    pub warning: String,
}

/// Result from regime-based performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeAnalysisResult {
    /// Overall results across all regimes
    pub overall: RegimePerformance,
    /// Performance in bull regime
    pub bull_regime: RegimePerformance,
    /// Performance in bear regime
    pub bear_regime: RegimePerformance,
    /// Performance in sideways regime
    pub sideways_regime: RegimePerformance,
    /// Performance in high volatility regime
    pub high_vol_regime: RegimePerformance,
    /// Performance in low volatility regime
    pub low_vol_regime: RegimePerformance,
    /// Performance in trending regime
    pub trending_regime: RegimePerformance,
    /// Performance in ranging regime
    pub ranging_regime: RegimePerformance,
    /// Regime mismatch warning (if any)
    pub regime_mismatch: Option<RegimeMismatch>,
    /// Expected regimes from strategy metadata
    pub expected_regimes: Vec<MarketRegime>,
}

impl Default for RegimeAnalysisResult {
    fn default() -> Self {
        Self {
            overall: RegimePerformance::new(),
            bull_regime: RegimePerformance::new(),
            bear_regime: RegimePerformance::new(),
            sideways_regime: RegimePerformance::new(),
            high_vol_regime: RegimePerformance::new(),
            low_vol_regime: RegimePerformance::new(),
            trending_regime: RegimePerformance::new(),
            ranging_regime: RegimePerformance::new(),
            regime_mismatch: None,
            expected_regimes: Vec::new(),
        }
    }
}

impl RegimeAnalysisResult {
    /// Get performance for a specific regime
    pub fn get_regime_performance(&self, regime: MarketRegime) -> &RegimePerformance {
        match regime {
            MarketRegime::Bull => &self.bull_regime,
            MarketRegime::Bear => &self.bear_regime,
            MarketRegime::Sideways => &self.sideways_regime,
            MarketRegime::HighVolatility => &self.high_vol_regime,
            MarketRegime::LowVolatility => &self.low_vol_regime,
            MarketRegime::Trending => &self.trending_regime,
            MarketRegime::Ranging => &self.ranging_regime,
        }
    }

    /// Calculate regime match score (0-100)
    pub fn calculate_regime_match_score(&self) -> f64 {
        if self.expected_regimes.is_empty() {
            // No expected regimes specified, assume neutral score
            return 50.0;
        }

        let mut total_score = 0.0;
        let mut weight_sum = 0.0;

        for regime in &self.expected_regimes {
            let performance = self.get_regime_performance(*regime);

            // Weight by time spent in regime
            let weight = performance.time_in_regime.max(0.01);

            // Score based on Sharpe ratio (0 = poor, 100 = excellent)
            let sharpe_score = (performance.sharpe_ratio.min(3.0) / 3.0 * 100.0).max(0.0);

            // Penalize high drawdowns
            let drawdown_penalty = (performance.max_drawdown / 0.50 * 100.0).min(50.0);

            let regime_score = sharpe_score - drawdown_penalty;

            total_score += regime_score * weight;
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            total_score / weight_sum
        } else {
            50.0
        }
    }

    /// Detect regime mismatch
    pub fn detect_mismatch(&mut self) {
        // Find best performing regime
        let regime_performances = vec![
            (MarketRegime::Bull, &self.bull_regime),
            (MarketRegime::Bear, &self.bear_regime),
            (MarketRegime::Sideways, &self.sideways_regime),
            (MarketRegime::Trending, &self.trending_regime),
            (MarketRegime::Ranging, &self.ranging_regime),
        ];

        let best_regime = regime_performances.iter().max_by(|a, b| {
            let score_a = a.1.total_return / (a.1.max_drawdown.abs() + 0.01);
            let score_b = b.1.total_return / (b.1.max_drawdown.abs() + 0.01);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some((best_regime, best_perf)) = best_regime {
            if !self.expected_regimes.is_empty() {
                // Check if best regime is in expected regimes
                if !self.expected_regimes.contains(best_regime) {
                    // Calculate performance gap
                    let expected_perf = if !self.expected_regimes.is_empty() {
                        let first_expected = &self.expected_regimes[0];
                        self.get_regime_performance(*first_expected).total_return
                    } else {
                        0.0
                    };

                    let gap = (best_perf.total_return - expected_perf).abs();

                    if gap > 0.10 && best_perf.total_trades > 10 {
                        self.regime_mismatch = Some(RegimeMismatch {
                            best_performing_regime: *best_regime,
                            expected_regime: self.expected_regimes[0],
                            performance_gap: gap,
                            warning: format!(
                                "Strategy performs best in {} market but is designed for {} market. Consider strategy adaptation.",
                                best_regime.as_str(),
                                self.expected_regimes[0].as_str()
                            ),
                        });
                    }
                }
            }
        }
    }
}

/// Regime analysis engine
pub struct RegimeAnalyzer {
    /// Trend strength threshold for regime classification (0-1)
    trend_threshold: f64,
    /// Volatility threshold for high volatility regime (ATR multiple)
    volatility_threshold: f64,
    /// Short moving average period
    short_ma_period: usize,
    /// Long moving average period
    long_ma_period: usize,
    /// ATR period
    atr_period: usize,
}

impl Default for RegimeAnalyzer {
    fn default() -> Self {
        Self {
            trend_threshold: 0.02,     // 2% price change threshold
            volatility_threshold: 1.5, // 1.5x average ATR
            short_ma_period: 20,
            long_ma_period: 200,
            atr_period: 14,
        }
    }
}

impl RegimeAnalyzer {
    /// Create new regime analyzer with custom parameters
    pub fn new(trend_threshold: f64, volatility_threshold: f64) -> Self {
        Self {
            trend_threshold,
            volatility_threshold,
            ..Default::default()
        }
    }

    /// Calculate Simple Moving Average
    fn calculate_sma(&self, bars: &[Bar], period: usize, end_index: usize) -> Option<f64> {
        if end_index < period || bars.is_empty() {
            return None;
        }

        let start = end_index.saturating_sub(period);
        let sum: f64 = bars[start..=end_index].iter().map(|b| b.close).sum();
        Some(sum / period as f64)
    }

    /// Calculate ATR (Average True Range)
    fn calculate_atr(&self, bars: &[Bar], end_index: usize) -> Option<f64> {
        if end_index < self.atr_period || bars.is_empty() {
            return None;
        }

        let start = end_index.saturating_sub(self.atr_period);
        let mut true_ranges = Vec::new();

        for i in start..=end_index {
            if i == 0 {
                continue;
            }

            let high_low = bars[i].high - bars[i].low;
            let high_close = (bars[i].high - bars[i - 1].close).abs();
            let low_close = (bars[i].low - bars[i - 1].close).abs();

            true_ranges.push(high_low.max(high_close).max(low_close));
        }

        if true_ranges.is_empty() {
            return None;
        }

        let sum: f64 = true_ranges.iter().sum();
        Some(sum / true_ranges.len() as f64)
    }

    /// Detect regime for a set of bars ending at given index
    pub fn detect_regime(&self, bars: &[Bar], end_index: usize) -> MarketRegime {
        if end_index < self.long_ma_period {
            return MarketRegime::Sideways;
        }

        // Calculate moving averages
        let _short_ma = match self.calculate_sma(bars, self.short_ma_period, end_index) {
            Some(ma) => ma,
            None => return MarketRegime::Sideways,
        };

        let long_ma = match self.calculate_sma(bars, self.long_ma_period, end_index) {
            Some(ma) => ma,
            None => return MarketRegime::Sideways,
        };

        // Calculate ATR
        let atr = match self.calculate_atr(bars, end_index) {
            Some(atr) => atr,
            None => return MarketRegime::Sideways,
        };

        // Calculate price as percentage of long MA
        let price = bars[end_index].close;
        let price_ratio = (price - long_ma) / long_ma;

        // Determine trend regime
        let trend_regime = if price_ratio > self.trend_threshold {
            MarketRegime::Bull
        } else if price_ratio < -self.trend_threshold {
            MarketRegime::Bear
        } else {
            MarketRegime::Sideways
        };

        // Calculate volatility (ATR as percentage of price)
        let volatility_pct = atr / price;
        let avg_volatility = self.calculate_avg_volatility(bars, end_index);

        // Determine volatility regime
        let is_high_vol = if avg_volatility > 0.0 {
            volatility_pct > avg_volatility * self.volatility_threshold
        } else {
            volatility_pct > 0.03 // Default 3% threshold
        };

        // Combine trend and volatility
        match trend_regime {
            MarketRegime::Bull if is_high_vol => MarketRegime::HighVolatility,
            MarketRegime::Bear if is_high_vol => MarketRegime::HighVolatility,
            MarketRegime::Bull => MarketRegime::Trending,
            MarketRegime::Bear => MarketRegime::Trending,
            MarketRegime::Sideways if is_high_vol => MarketRegime::HighVolatility,
            MarketRegime::Sideways => MarketRegime::Ranging,
            _ => MarketRegime::Sideways,
        }
    }

    /// Calculate average volatility for reference
    fn calculate_avg_volatility(&self, bars: &[Bar], end_index: usize) -> f64 {
        if end_index < self.atr_period * 2 {
            return 0.0;
        }

        let volatilities: Vec<f64> = bars
            .iter()
            .skip(end_index.saturating_sub(self.atr_period * 2))
            .take(self.atr_period * 2)
            .filter_map(|b| {
                if b.close > 0.0 {
                    Some((b.high - b.low) / b.close)
                } else {
                    None
                }
            })
            .collect();

        if volatilities.is_empty() {
            return 0.0;
        }

        volatilities.iter().sum::<f64>() / volatilities.len() as f64
    }

    /// Analyze strategy performance across regimes
    ///
    /// # Arguments
    /// * `strategy` - Strategy to analyze
    /// * `bars` - Historical bar data (must be sorted by time)
    /// * `expected_regimes` - Regimes where strategy is expected to perform well
    ///
    /// # Returns
    /// Regime analysis result with performance metrics for each regime
    pub fn analyze(
        &self,
        bars: &[Bar],
        expected_regimes: Vec<MarketRegime>,
    ) -> Result<RegimeAnalysisResult, CoreError> {
        if bars.len() < self.long_ma_period {
            return Err(CoreError::DataValidation(
                "Insufficient data for regime analysis".to_string(),
            ));
        }

        // Detect regime for each bar
        let mut regimes: Vec<MarketRegime> = Vec::with_capacity(bars.len());
        let mut regime_counts: HashMap<MarketRegime, usize> = HashMap::new();

        for i in self.long_ma_period..bars.len() {
            let regime = self.detect_regime(bars, i);
            regimes.push(regime);
            *regime_counts.entry(regime).or_insert(0) += 1;
        }

        // Group bars by regime
        let mut regime_bars: HashMap<MarketRegime, Vec<&Bar>> = HashMap::new();

        for (i, regime) in regimes.iter().enumerate() {
            let bar_index = self.long_ma_period + i;
            regime_bars
                .entry(*regime)
                .or_default()
                .push(&bars[bar_index]);
        }

        // Calculate performance metrics for each regime
        let bull_regime = self.calculate_regime_performance(
            regime_bars.get(&MarketRegime::Bull),
            regime_counts.get(&MarketRegime::Bull).copied().unwrap_or(0),
            bars.len(),
        );

        let bear_regime = self.calculate_regime_performance(
            regime_bars.get(&MarketRegime::Bear),
            regime_counts.get(&MarketRegime::Bear).copied().unwrap_or(0),
            bars.len(),
        );

        let sideways_regime = self.calculate_regime_performance(
            regime_bars.get(&MarketRegime::Sideways),
            regime_counts
                .get(&MarketRegime::Sideways)
                .copied()
                .unwrap_or(0),
            bars.len(),
        );

        let high_vol_regime = self.calculate_regime_performance(
            regime_bars.get(&MarketRegime::HighVolatility),
            regime_counts
                .get(&MarketRegime::HighVolatility)
                .copied()
                .unwrap_or(0),
            bars.len(),
        );

        let low_vol_regime = self.calculate_regime_performance(
            regime_bars.get(&MarketRegime::LowVolatility),
            regime_counts
                .get(&MarketRegime::LowVolatility)
                .copied()
                .unwrap_or(0),
            bars.len(),
        );

        let trending_regime = self.calculate_regime_performance(
            regime_bars.get(&MarketRegime::Trending),
            regime_counts
                .get(&MarketRegime::Trending)
                .copied()
                .unwrap_or(0),
            bars.len(),
        );

        let ranging_regime = self.calculate_regime_performance(
            regime_bars.get(&MarketRegime::Ranging),
            regime_counts
                .get(&MarketRegime::Ranging)
                .copied()
                .unwrap_or(0),
            bars.len(),
        );

        // Calculate overall performance (weighted average)
        let overall = self.calculate_overall_performance(&[
            &bull_regime,
            &bear_regime,
            &sideways_regime,
            &high_vol_regime,
            &low_vol_regime,
            &trending_regime,
            &ranging_regime,
        ]);

        let mut result = RegimeAnalysisResult {
            overall,
            bull_regime,
            bear_regime,
            sideways_regime,
            high_vol_regime,
            low_vol_regime,
            trending_regime,
            ranging_regime,
            regime_mismatch: None,
            expected_regimes,
        };

        // Detect regime mismatch
        result.detect_mismatch();

        Ok(result)
    }

    /// Calculate performance metrics for a specific regime
    fn calculate_regime_performance(
        &self,
        regime_bars: Option<&Vec<&Bar>>,
        regime_bar_count: usize,
        total_bars: usize,
    ) -> RegimePerformance {
        let bars = match regime_bars {
            Some(b) if !b.is_empty() => b,
            _ => return RegimePerformance::new(),
        };

        // Calculate buy and hold returns for this regime
        let first_close = bars.first().map(|b| b.close).unwrap_or(1.0);
        let last_close = bars.last().map(|b| b.close).unwrap_or(first_close);
        let total_return = (last_close - first_close) / first_close;

        // Calculate volatility
        let returns: Vec<f64> = bars
            .windows(2)
            .map(|w| (w[1].close - w[0].close) / w[0].close)
            .collect();

        let volatility = if !returns.is_empty() {
            let mean = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance =
                returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
            variance.sqrt()
        } else {
            0.0
        };

        // Calculate max drawdown
        let mut max_drawdown: f64 = 0.0;
        let mut peak = first_close;
        for bar in bars {
            if bar.close > peak {
                peak = bar.close;
            }
            let drawdown = (peak - bar.close) / peak;
            max_drawdown = max_drawdown.max(drawdown);
        }

        // Estimate Sharpe ratio (simplified)
        let sharpe_ratio = if volatility > 0.0 {
            (total_return / volatility) * 2.0 // Approximate annualization
        } else {
            0.0
        };

        // Time in regime (percentage)
        let time_in_regime = if total_bars > 0 {
            regime_bar_count as f64 / total_bars as f64
        } else {
            0.0
        };

        RegimePerformance {
            total_return,
            sharpe_ratio,
            max_drawdown,
            win_rate: 0.0,   // Will be filled by validator with actual trades
            total_trades: 0, // Will be filled by validator with actual trades
            time_in_regime,
            avg_return_per_trade: total_return,
        }
    }

    /// Calculate overall performance from all regimes
    fn calculate_overall_performance(&self, regimes: &[&RegimePerformance]) -> RegimePerformance {
        let mut total_return = 0.0;
        let mut total_weight = 0.0;
        let mut weighted_sharpe = 0.0;
        let mut weighted_drawdown = 0.0;

        for regime in regimes {
            if regime.total_trades > 0 || regime.time_in_regime > 0.0 {
                let weight = regime.time_in_regime.max(0.01);
                total_return += regime.total_return * weight;
                weighted_sharpe += regime.sharpe_ratio * weight;
                weighted_drawdown += regime.max_drawdown * weight;
                total_weight += weight;
            }
        }

        RegimePerformance {
            total_return: if total_weight > 0.0 {
                total_return / total_weight
            } else {
                0.0
            },
            sharpe_ratio: if total_weight > 0.0 {
                weighted_sharpe / total_weight
            } else {
                0.0
            },
            max_drawdown: if total_weight > 0.0 {
                weighted_drawdown / total_weight
            } else {
                0.0
            },
            win_rate: 0.0,
            total_trades: 0,
            time_in_regime: 1.0,
            avg_return_per_trade: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alphafield_core::Bar;
    use chrono::Utc;

    fn create_test_bars(count: usize, trend: f64) -> Vec<Bar> {
        (0..count)
            .map(|i| {
                let base_price = 100.0 + (i as f64 * trend);
                let noise = (i as f64 % 10.0 - 5.0) * 0.5;

                Bar {
                    timestamp: Utc::now() + chrono::Duration::seconds(i as i64 * 3600),
                    open: base_price + noise,
                    high: base_price + noise + 1.0,
                    low: base_price + noise - 1.0,
                    close: base_price + noise + (i as f64 % 2.0 - 1.0) * 0.5,
                    volume: 1000.0,
                }
            })
            .collect()
    }

    #[test]
    fn test_regime_detection_bull() {
        let bars = create_test_bars(300, 0.5); // Upward trend
        let analyzer = RegimeAnalyzer::default();

        let regime = analyzer.detect_regime(&bars, bars.len() - 1);

        assert!(regime == MarketRegime::Bull || regime == MarketRegime::Trending);
    }

    #[test]
    fn test_regime_detection_bear() {
        let bars = create_test_bars(300, -0.5); // Downward trend
        let analyzer = RegimeAnalyzer::default();

        let regime = analyzer.detect_regime(&bars, bars.len() - 1);

        assert!(regime == MarketRegime::Bear || regime == MarketRegime::Trending);
    }

    #[test]
    fn test_regime_detection_sideways() {
        let bars = create_test_bars(300, 0.0); // Sideways
        let analyzer = RegimeAnalyzer::default();

        let regime = analyzer.detect_regime(&bars, bars.len() - 1);

        assert!(regime == MarketRegime::Sideways || regime == MarketRegime::Ranging);
    }

    #[test]
    fn test_insufficient_data() {
        let bars = create_test_bars(100, 0.5);
        let analyzer = RegimeAnalyzer::default();

        // Should not panic with insufficient data
        let regime = analyzer.detect_regime(&bars, bars.len() - 1);
        assert_eq!(regime, MarketRegime::Sideways);
    }

    #[test]
    fn test_regime_performance_acceptable() {
        let perf = RegimePerformance {
            total_return: 0.50,
            sharpe_ratio: 2.0,
            max_drawdown: 0.15,
            win_rate: 0.60,
            total_trades: 50,
            time_in_regime: 0.25,
            avg_return_per_trade: 0.01,
        };

        assert!(perf.is_acceptable(1.5, 0.20));
        assert!(!perf.is_acceptable(2.5, 0.10));
    }

    #[test]
    fn test_regime_match_score() {
        let mut result = RegimeAnalysisResult {
            expected_regimes: vec![MarketRegime::Bull],
            ..Default::default()
        };

        result.bull_regime = RegimePerformance {
            total_return: 0.30,
            sharpe_ratio: 2.5,
            max_drawdown: 0.10,
            time_in_regime: 0.50,
            ..Default::default()
        };

        let score = result.calculate_regime_match_score();
        assert!(score > 50.0); // Should score well
    }

    #[test]
    fn test_regime_mismatch_detection() {
        let mut result = RegimeAnalysisResult {
            expected_regimes: vec![MarketRegime::Bull],
            ..Default::default()
        };

        // Strategy performs best in bear market but designed for bull
        result.bull_regime = RegimePerformance {
            total_return: -0.10,
            total_trades: 20,
            time_in_regime: 0.40,
            ..Default::default()
        };

        result.bear_regime = RegimePerformance {
            total_return: 0.30,
            total_trades: 30,
            time_in_regime: 0.35,
            ..Default::default()
        };

        result.detect_mismatch();

        assert!(result.regime_mismatch.is_some());
    }
}
