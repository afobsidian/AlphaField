//! Strategy Framework - Phase 12.1
//!
//! This module provides the foundation for strategy management including:
//! - Metadata types for strategy documentation
//! - Strategy registry for registration and lookup
//! - Strategy classifier for automatic classification
//! - Trait extensions for metadata
//!
//! # Important: Symbol Handling in Strategy Signals
//!
//! **Current Limitation**: All strategy implementations in the trend-following module use a
//! hardcoded "UNKNOWN" symbol in generated signals. This is intentional design because:
//!
//! 1. The `Strategy` trait's `on_bar` method only receives a `Bar` reference, which contains
//!    the symbol information in `bar.symbol`, but the `Signal` struct requires an owned `String`.
//! 2. The dashboard's `StrategyAdapter` wraps strategies during backtesting and handles the
//!    proper symbol mapping from the `Bar` to the `Signal`.
//! 3. This abstraction allows strategies to remain symbol-agnostic and reusable across different
//!    assets during backtesting.
//!
//! **For Live Trading**: When using strategies in live execution scenarios, the execution layer
//! (not the strategy) is responsible for ensuring the correct symbol is associated with orders.
//!
//! **Future Enhancement**: The `Strategy` trait could be updated to accept the symbol explicitly
//! in the `on_bar` signature, or the `Signal` struct could reference `bar.symbol` directly.
//! However, this would require breaking changes to the trait interface.

use alphafield_core::{QuantError, Strategy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ============ Type Definitions ============

/// Complete metadata for a trading strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetadata {
    /// Unique strategy name
    pub name: String,
    /// Primary category of the strategy
    pub category: StrategyCategory,
    /// Optional sub-type for finer classification
    pub sub_type: Option<String>,
    /// Human-readable description
    pub description: String,
    /// Path to hypothesis documentation file
    pub hypothesis_path: String,
    /// List of required indicator names
    pub required_indicators: Vec<String>,
    /// Expected market regimes for this strategy
    pub expected_regimes: Vec<MarketRegime>,
    /// Risk profile information
    pub risk_profile: RiskProfile,
}

/// Primary strategy categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StrategyCategory {
    /// Follows market trends
    TrendFollowing,
    /// Reverts to mean/average
    MeanReversion,
    /// Follows momentum
    Momentum,
    /// Based on volatility patterns
    VolatilityBased,
    /// Based on sentiment data
    SentimentBased,
    /// Combines multiple indicators
    MultiIndicator,
}

/// Market regimes for strategy classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MarketRegime {
    /// Bullish market
    Bull,
    /// Bearish market
    Bear,
    /// Sideways/ranging market
    Sideways,
    /// High volatility conditions
    HighVolatility,
    /// Low volatility conditions
    LowVolatility,
    /// Trending market
    Trending,
    /// Ranging market
    Ranging,
}

/// Risk profile for a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfile {
    /// Expected maximum drawdown (as percentage, e.g., 0.15 = 15%)
    pub max_drawdown_expected: f64,
    /// Volatility level of the strategy
    pub volatility_level: VolatilityLevel,
    /// Sensitivity to market correlation
    pub correlation_sensitivity: CorrelationSensitivity,
    /// Leverage requirement (1.0 = no leverage)
    pub leverage_requirement: f64,
}

/// Volatility level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VolatilityLevel {
    /// Low volatility strategy
    Low,
    /// Medium volatility strategy
    Medium,
    /// High volatility strategy
    High,
}

/// Correlation sensitivity classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CorrelationSensitivity {
    /// Low correlation with market
    Low,
    /// Medium correlation with market
    Medium,
    /// High correlation with market
    High,
}

// ============ Extended Traits ============

/// Trait for strategies that provide metadata
pub trait MetadataStrategy {
    /// Returns the complete metadata for this strategy
    fn metadata(&self) -> StrategyMetadata;

    /// Returns the category of this strategy
    fn category(&self) -> StrategyCategory {
        self.metadata().category
    }
}

/// Combined trait for strategies with metadata - used for trait objects
pub trait StrategyWithMetadata: Strategy + MetadataStrategy {}

/// Blanket implementation for all types that implement both Strategy and MetadataStrategy
impl<T: Strategy + MetadataStrategy> StrategyWithMetadata for T {}

// ============ Strategy Classification ============

/// Classification result for a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyClassification {
    /// Strategy name
    pub strategy_name: String,
    /// Primary category
    pub primary_category: StrategyCategory,
    /// Sub-type if available
    pub sub_type: Option<String>,
    /// Calculated characteristics
    pub characteristics: StrategyCharacteristics,
}

/// Characteristics automatically derived from strategy behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyCharacteristics {
    /// Whether strategy is sensitive to market trends
    pub trend_sensitive: bool,
    /// Whether strategy is sensitive to volatility changes
    pub volatility_sensitive: bool,
    /// Whether strategy is sensitive to market correlation
    pub correlation_sensitive: bool,
    /// Estimated time horizon of trades
    pub time_horizon: TimeHorizon,
    /// Estimated frequency of signals
    pub signal_frequency: SignalFrequency,
    /// Risk/reward ratio (average win / average loss)
    pub risk_reward_ratio: f64,
}

/// Time horizon for strategy trades
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeHorizon {
    /// Less than 1 hour
    Scalping,
    /// 1 hour to 1 day
    Intraday,
    /// 1 day to 1 week
    Swing,
    /// 1 week to 1 month
    ShortTerm,
    /// 1 month to 3 months
    MediumTerm,
    /// Greater than 3 months
    LongTerm,
}

/// Frequency of signal generation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalFrequency {
    /// Multiple signals per day
    VeryHigh,
    /// Daily signals
    High,
    /// Weekly signals
    Medium,
    /// Monthly signals
    Low,
    /// Quarterly or less
    VeryLow,
}

// ============ Backtest Results ============

/// Simplified backtest results for classification
#[derive(Debug, Clone)]
pub struct BacktestResults {
    /// Total return percentage
    pub total_return: f64,
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    /// Maximum drawdown
    pub max_drawdown: f64,
    /// Win rate (0-100)
    pub win_rate: f64,
    /// Total number of trades
    pub total_trades: usize,
    /// Average winning trade
    pub avg_win: f64,
    /// Average losing trade
    pub avg_loss: f64,
    /// Average trade duration in hours
    pub avg_trade_duration_hours: f64,
    /// Test period in days
    pub test_period_days: f64,
    /// Correlation with market
    pub market_correlation: f64,
    /// Return in bull regime
    pub regime_bull_return: f64,
    /// Return in bear regime
    pub regime_bear_return: f64,
    /// Return in high volatility regime
    pub regime_high_vol_return: f64,
    /// Return in low volatility regime
    pub regime_low_vol_return: f64,
}

// ============ Strategy Classifier ============

/// Automatically classifies strategies based on backtest results
pub struct StrategyClassifier;

impl StrategyClassifier {
    /// Analyzes a strategy and returns its classification
    ///
    /// # Arguments
    /// * `strategy_name` - Name of the strategy
    /// * `metadata` - Strategy metadata
    /// * `test_results` - Backtest results from strategy testing
    ///
    /// # Returns
    /// Complete classification including characteristics
    pub fn analyze_strategy(
        strategy_name: &str,
        metadata: &StrategyMetadata,
        test_results: &BacktestResults,
    ) -> StrategyClassification {
        let characteristics = StrategyCharacteristics {
            trend_sensitive: Self::calculate_trend_sensitivity(test_results),
            volatility_sensitive: Self::calculate_volatility_sensitivity(test_results),
            correlation_sensitive: Self::calculate_correlation_sensitivity(test_results),
            time_horizon: Self::estimate_time_horizon(test_results),
            signal_frequency: Self::estimate_signal_frequency(test_results),
            risk_reward_ratio: if test_results.avg_loss != 0.0 {
                test_results.avg_win / test_results.avg_loss.abs()
            } else {
                0.0
            },
        };

        StrategyClassification {
            strategy_name: strategy_name.to_string(),
            primary_category: metadata.category.clone(),
            sub_type: metadata.sub_type.clone(),
            characteristics,
        }
    }

    /// Calculates if strategy is trend-sensitive based on regime performance
    fn calculate_trend_sensitivity(results: &BacktestResults) -> bool {
        // Compare bull market performance vs bear market performance
        // If significantly better in trending markets, return true
        let diff = (results.regime_bull_return - results.regime_bear_return).abs();
        diff > 10.0
    }

    /// Calculates if strategy is volatility-sensitive based on regime performance
    fn calculate_volatility_sensitivity(results: &BacktestResults) -> bool {
        // Compare high vol vs low vol performance
        let diff = (results.regime_high_vol_return - results.regime_low_vol_return).abs();
        diff > 10.0
    }

    /// Calculates if strategy is correlation-sensitive
    fn calculate_correlation_sensitivity(results: &BacktestResults) -> bool {
        // Check performance vs market correlation
        results.market_correlation.abs() > 0.7
    }

    /// Estimates time horizon based on average trade duration
    fn estimate_time_horizon(results: &BacktestResults) -> TimeHorizon {
        let avg_hours = results.avg_trade_duration_hours;
        match avg_hours {
            h if h < 1.0 => TimeHorizon::Scalping,
            h if h < 24.0 => TimeHorizon::Intraday,
            h if h < 168.0 => TimeHorizon::Swing,
            h if h < 720.0 => TimeHorizon::ShortTerm,
            h if h < 2160.0 => TimeHorizon::MediumTerm,
            _ => TimeHorizon::LongTerm,
        }
    }

    /// Estimates signal frequency based on trades per day
    fn estimate_signal_frequency(results: &BacktestResults) -> SignalFrequency {
        let trades_per_day = results.total_trades as f64 / results.test_period_days;
        match trades_per_day {
            t if t > 5.0 => SignalFrequency::VeryHigh,
            t if t >= 1.0 => SignalFrequency::High,
            t if t > 0.2 => SignalFrequency::Medium,
            t if t > 0.03 => SignalFrequency::Low,
            _ => SignalFrequency::VeryLow,
        }
    }
}

// ============ Strategy Registry ============

/// Registry for managing strategy instances and metadata
///
/// The registry provides centralized management of all strategies, allowing
/// registration, lookup, and querying by category or regime.
pub struct StrategyRegistry {
    /// Map of strategy name to strategy instance
    strategies: Arc<RwLock<HashMap<String, Arc<dyn StrategyWithMetadata>>>>,
    /// Map of strategy name to metadata
    metadata: Arc<RwLock<HashMap<String, StrategyMetadata>>>,
}

impl StrategyRegistry {
    /// Creates a new empty strategy registry
    pub fn new() -> Self {
        StrategyRegistry {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a strategy in the registry
    ///
    /// # Arguments
    /// * `strategy` - Strategy instance to register
    ///
    /// # Returns
    /// * `Ok(())` if registration successful
    /// * `Err(QuantError)` if validation fails or lock error occurs
    pub fn register(&self, strategy: Arc<dyn StrategyWithMetadata>) -> Result<(), QuantError> {
        let name = strategy.name().to_string();
        let metadata = strategy.metadata();

        // Validate metadata
        if metadata.name != name {
            return Err(QuantError::DataValidation(format!(
                "Strategy name mismatch: metadata says '{}' but strategy name is '{}'",
                metadata.name, name
            )));
        }

        {
            let mut strategies_guard = self
                .strategies
                .write()
                .map_err(|e| QuantError::DataValidation(format!("Lock error: {}", e)))?;
            let mut metadata_guard = self
                .metadata
                .write()
                .map_err(|e| QuantError::DataValidation(format!("Lock error: {}", e)))?;

            strategies_guard.insert(name.clone(), strategy);
            metadata_guard.insert(name, metadata);
        }

        Ok(())
    }

    /// Retrieves a strategy by name
    ///
    /// # Arguments
    /// * `name` - Strategy name to retrieve
    ///
    /// # Returns
    /// * `Some(Arc<Strategy>)` if found
    /// * `None` if not found
    pub fn get(&self, name: &str) -> Option<Arc<dyn StrategyWithMetadata>> {
        self.strategies.read().ok()?.get(name).cloned()
    }

    /// Retrieves metadata for a strategy
    ///
    /// # Arguments
    /// * `name` - Strategy name to get metadata for
    ///
    /// # Returns
    /// * `Some(StrategyMetadata)` if found
    /// * `None` if not found
    pub fn get_metadata(&self, name: &str) -> Option<StrategyMetadata> {
        self.metadata.read().ok()?.get(name).cloned()
    }

    /// Lists all registered strategy names
    ///
    /// # Returns
    /// Vector of all strategy names
    pub fn list_all(&self) -> Vec<String> {
        self.metadata
            .read()
            .ok()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Lists strategies by category
    ///
    /// # Arguments
    /// * `category` - Category to filter by
    ///
    /// # Returns
    /// Vector of strategy names in the category
    pub fn list_by_category(&self, category: StrategyCategory) -> Vec<String> {
        self.metadata
            .read()
            .ok()
            .map(|m| {
                m.iter()
                    .filter(|(_, meta)| meta.category == category)
                    .map(|(name, _)| name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Lists strategies suitable for a market regime
    ///
    /// # Arguments
    /// * `regime` - Market regime to filter by
    ///
    /// # Returns
    /// Vector of strategy names suitable for the regime
    pub fn get_for_regime(&self, regime: MarketRegime) -> Vec<String> {
        self.metadata
            .read()
            .ok()
            .map(|m| {
                m.iter()
                    .filter(|(_, meta)| meta.expected_regimes.contains(&regime))
                    .map(|(name, _)| name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns the number of registered strategies
    pub fn count(&self) -> usize {
        self.metadata.read().ok().map(|m| m.len()).unwrap_or(0)
    }

    /// Checks if a strategy is registered
    ///
    /// # Arguments
    /// * `name` - Strategy name to check
    ///
    /// # Returns
    /// `true` if strategy is registered, `false` otherwise
    pub fn contains(&self, name: &str) -> bool {
        self.metadata
            .read()
            .ok()
            .map(|m| m.contains_key(name))
            .unwrap_or(false)
    }
}

// ============ Strategy Name Canonicalization ============

/// Canonicalizes strategy identifiers to internal keys used by the optimizer and StrategyFactory.
///
/// This function accepts both human-friendly display names from the dashboard API (e.g., "Golden Cross",
/// "Parabolic SAR") and internal strategy keys (e.g., "GoldenCross", "ParabolicSAR"), and always
/// returns the internal key format.
///
/// The canonicalization is necessary because:
/// - The dashboard UI displays user-friendly names (with spaces, capitalization)
/// - The optimizer and StrategyFactory expect internal keys (CamelCase)
/// - Both endpoints in the workflow need to understand each other's naming
///
/// # Arguments
/// * `name` - Strategy name in either display format or internal key format
///
/// # Returns
/// The canonical internal strategy key (e.g., "GoldenCross", "Rsi")
///
/// # Example
/// ```
/// use alphafield_strategy::framework::canonicalize_strategy_name;
///
/// assert_eq!(canonicalize_strategy_name("Golden Cross"), "GoldenCross");
/// assert_eq!(canonicalize_strategy_name("Parabolic SAR"), "ParabolicSAR");
/// assert_eq!(canonicalize_strategy_name("GoldenCross"), "GoldenCross");
/// ```
pub fn canonicalize_strategy_name(name: &str) -> String {
    match name.trim() {
        // Batch file names → Registry keys (remove "Strategy" suffix if present)
        // Batch file names → Registry keys
        "AdaptiveMA" => "AdaptiveMA".to_string(),
        "AdxTrendStrategy" => "AdxTrendStrategy".to_string(),
        "ATRBreakout" => "ATRBreakout".to_string(),
        "Breakout" => "Breakout".to_string(),
        "Divergence" => "Divergence".to_string(),
        "GoldenCross" => "GoldenCross".to_string(),
        "KeltnerReversion" => "KeltnerReversion".to_string(),
        "MacdTrend" => "MacdTrend".to_string(),
        "MACrossover" => "MACrossover".to_string(),
        "MeanReversion" => "MeanReversion".to_string(),
        "Momentum" => "Momentum".to_string(),
        "MomentumFactorStrategy" => "MomentumFactorStrategy".to_string(),
        "MultiTfMomentumStrategy" => "MultiTfMomentumStrategy".to_string(),
        "ParabolicSAR" => "ParabolicSAR".to_string(),
        "PriceChannel" => "PriceChannel".to_string(),
        "RSIReversion" => "RSIReversion".to_string(),
        "RocStrategy" => "RocStrategy".to_string(),
        "RsiMomentumStrategy" => "RsiMomentumStrategy".to_string(),
        "VolumeMomentumStrategy" => "VolumeMomentumStrategy".to_string(),
        "StatArb" => "StatArb".to_string(),
        "RegimeSentiment" => "RegimeSentiment".to_string(),
        "SentimentMomentum" => "SentimentMomentum".to_string(),
        "StochReversion" => "StochReversion".to_string(),
        "TripleMA" => "TripleMA".to_string(),
        "ZScoreReversion" => "ZScoreReversion".to_string(),
        "BollingerBands" => "BollingerBands".to_string(),

        // Multi-Indicator Strategies (canonical name → canonical name)
        "TrendMeanRev" => "TrendMeanRev".to_string(),
        "MACDRSICombo" => "MACDRSICombo".to_string(),
        "AdaptiveCombo" => "AdaptiveCombo".to_string(),
        "ConfidenceWeighted" => "ConfidenceWeighted".to_string(),
        "EnsembleWeighted" => "EnsembleWeighted".to_string(),
        "MLEnhanced" => "MLEnhanced".to_string(),
        "RegimeSwitching" => "RegimeSwitching".to_string(),
        "TrendMeanRevStrategy" => "TrendMeanRev".to_string(),
        "MACDRSIComboStrategy" => "MACDRSICombo".to_string(),
        "AdaptiveComboStrategy" => "AdaptiveCombo".to_string(),
        "ConfidenceWeightedStrategy" => "ConfidenceWeighted".to_string(),
        "EnsembleWeightedStrategy" => "EnsembleWeighted".to_string(),
        "MLEnhancedStrategy" => "MLEnhanced".to_string(),
        "RegimeSwitchingStrategy" => "RegimeSwitching".to_string(),

        // Display names from strategy metadata → Registry keys
        "Golden Cross" => "GoldenCross".to_string(),
        "Adaptive MA" => "AdaptiveMA".to_string(),
        "MA Crossover" => "MACrossover".to_string(),
        "MACD Trend" => "MacdTrend".to_string(),
        "Parabolic SAR" => "ParabolicSAR".to_string(),
        "Triple MA" => "TripleMA".to_string(),
        "Bollinger Bands" => "BollingerBands".to_string(),
        "RSI Reversion" => "RSIReversion".to_string(),
        "Keltner Channel" => "KeltnerReversion".to_string(),
        "Price Channel (Donchian)" => "PriceChannel".to_string(),
        "Stochastic Reversion" => "StochReversion".to_string(),
        "Stochastic Mean Reversion" => "StochReversion".to_string(),
        "Z-Score Reversion" => "ZScoreReversion".to_string(),
        "Z-Score Mean Reversion" => "ZScoreReversion".to_string(),
        "Price Channel Mean Reversion" => "PriceChannel".to_string(),
        "Keltner Channel Mean Reversion" => "KeltnerReversion".to_string(),
        "Statistical Arbitrage" => "StatArb".to_string(),
        "Statistical Arbitrage Mean Reversion" => "StatArb".to_string(),
        "RSI Mean Reversion" => "RSIReversion".to_string(),
        "RSI Momentum" => "RsiMomentumStrategy".to_string(),
        "MACD Momentum" => "RocStrategy".to_string(),
        "Rate of Change (ROC)" => "RocStrategy".to_string(),
        "ROC Momentum" => "RocStrategy".to_string(),
        "ADX Trend" => "AdxTrendStrategy".to_string(),
        "Momentum Factor" => "MomentumFactorStrategy".to_string(),
        "Volume Momentum" => "VolumeMomentumStrategy".to_string(),
        "Multi-Timeframe Momentum" => "MultiTfMomentumStrategy".to_string(),
        "Multi-TF Momentum" => "MultiTfMomentumStrategy".to_string(),
        "ATR Breakout" => "ATRBreakout".to_string(),
        "ATR Trailing Stop" => "ATRTrailingStop".to_string(),
        "Volatility Squeeze" => "VolatilitySqueeze".to_string(),
        "Volatility Regime" => "VolRegimeStrategy".to_string(),
        "Volatility-Adjusted Position Sizing" => "VolSizingStrategy".to_string(),
        "GARCH-Based" => "GarchStrategy".to_string(),
        "VIX-Style" => "VIXStyleStrategy".to_string(),
        "Regime Sentiment" => "RegimeSentiment".to_string(),
        "Sentiment Momentum" => "SentimentMomentum".to_string(),
        "Divergence Strategy" => "Divergence".to_string(),
        "Regime Sentiment Strategy" => "RegimeSentiment".to_string(),
        "Sentiment Momentum Strategy" => "SentimentMomentum".to_string(),

        // Multi-Indicator Strategies (display name → key)
        "Trend+Mean Reversion Hybrid" => "TrendMeanRev".to_string(),
        "MACD+RSI Combo" => "MACDRSICombo".to_string(),
        "Adaptive Combination" => "AdaptiveCombo".to_string(),
        "Confidence-Weighted" => "ConfidenceWeighted".to_string(),
        "Ensemble Weighted" => "EnsembleWeighted".to_string(),
        "ML-Enhanced Multi-Indicator" => "MLEnhanced".to_string(),
        "Regime-Switching" => "RegimeSwitching".to_string(),

        // Existing "core" strategies (display name → key)
        "Bollinger Bands Mean Reversion" => "BollingerBands".to_string(),
        "EMA-MACD Momentum" => "Momentum".to_string(),
        "EMA-MACD" => "Momentum".to_string(),
        "RSI" => "RSIReversion".to_string(),
        "Stochastic" => "StochReversion".to_string(),
        "Z Score" => "ZScoreReversion".to_string(),
        "Donchian" => "PriceChannel".to_string(),
        "Keltner" => "KeltnerReversion".to_string(),
        "Stat Arb" => "StatArb".to_string(),
        "ATR" => "ATRBreakout".to_string(),
        "GARCH" => "GarchStrategy".to_string(),
        "VIX" => "VIXStyleStrategy".to_string(),
        "EMA MACD" => "Momentum".to_string(),
        "Rate of Change" => "RocStrategy".to_string(),
        "ADX" => "AdxTrendStrategy".to_string(),
        "SAR" => "ParabolicSAR".to_string(),
        "KAMA" => "AdaptiveMA".to_string(),
        "EMA" => "Momentum".to_string(),
        "SMA" => "Breakout".to_string(),

        // Volatility-based strategy variations
        "Volatility" => "ATRBreakout".to_string(),

        // Strategy + "Strategy" suffix variations
        "Golden Cross Strategy" => "GoldenCross".to_string(),
        "MA Crossover Strategy" => "MACrossover".to_string(),
        "Breakout Strategy" => "Breakout".to_string(),
        "Mean Reversion Strategy" => "MeanReversion".to_string(),
        "Trend Following Strategy" => "GoldenCross".to_string(),
        "Momentum Strategy" => "Momentum".to_string(),
        "Volatility Strategy" => "ATRBreakout".to_string(),

        // Already canonical / fallback - return as-is
        // This handles the case where the internal key is passed directly
        other => other.to_string(),
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BacktestResults {
    fn default() -> Self {
        BacktestResults {
            total_return: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            win_rate: 0.0,
            total_trades: 0,
            avg_win: 0.0,
            avg_loss: 0.0,
            avg_trade_duration_hours: 0.0,
            test_period_days: 1.0,
            market_correlation: 0.0,
            regime_bull_return: 0.0,
            regime_bear_return: 0.0,
            regime_high_vol_return: 0.0,
            regime_low_vol_return: 0.0,
        }
    }
}

// ============ Module Tests ============

#[cfg(test)]
mod tests {
    use super::*;
    use alphafield_core::{Bar, Signal};

    #[test]
    fn test_canonicalize_core_strategies_display_names() {
        assert_eq!(canonicalize_strategy_name("Golden Cross"), "GoldenCross");
        assert_eq!(
            canonicalize_strategy_name("RSI Mean Reversion"),
            "RSIReversion"
        );
        assert_eq!(
            canonicalize_strategy_name("Bollinger Bands Mean Reversion"),
            "BollingerBands"
        );
        assert_eq!(canonicalize_strategy_name("EMA-MACD Momentum"), "Momentum");
    }

    #[test]
    fn test_canonicalize_trend_following_strategies_display_names() {
        assert_eq!(canonicalize_strategy_name("Adaptive MA"), "AdaptiveMA");
        assert_eq!(canonicalize_strategy_name("MA Crossover"), "MACrossover");
        assert_eq!(canonicalize_strategy_name("MACD Trend"), "MacdTrend");
        assert_eq!(canonicalize_strategy_name("Parabolic SAR"), "ParabolicSAR");
    }

    #[test]
    fn test_canonicalize_already_canonical_names() {
        assert_eq!(canonicalize_strategy_name("GoldenCross"), "GoldenCross");
        assert_eq!(canonicalize_strategy_name("Rsi"), "Rsi");
        assert_eq!(canonicalize_strategy_name("MeanReversion"), "MeanReversion");
        assert_eq!(canonicalize_strategy_name("Momentum"), "Momentum");
        assert_eq!(canonicalize_strategy_name("AdaptiveMA"), "AdaptiveMA");
        assert_eq!(canonicalize_strategy_name("MACrossover"), "MACrossover");
        assert_eq!(canonicalize_strategy_name("MacdTrend"), "MacdTrend");
        assert_eq!(canonicalize_strategy_name("ParabolicSAR"), "ParabolicSAR");
    }

    #[test]
    fn test_canonicalize_whitespace_handling() {
        assert_eq!(
            canonicalize_strategy_name("  Golden Cross  "),
            "GoldenCross"
        );
        assert_eq!(canonicalize_strategy_name("\tAdaptive MA\n"), "AdaptiveMA");
        assert_eq!(canonicalize_strategy_name("  GoldenCross  "), "GoldenCross");
    }

    #[test]
    fn test_canonicalize_unknown_names() {
        assert_eq!(
            canonicalize_strategy_name("UnknownStrategy"),
            "UnknownStrategy"
        );
        assert_eq!(
            canonicalize_strategy_name("Custom Strategy"),
            "Custom Strategy"
        );
        assert_eq!(canonicalize_strategy_name("SomeOtherName"), "SomeOtherName");
    }

    #[test]
    fn test_canonicalize_case_sensitivity() {
        // The function is case-sensitive for display names
        assert_eq!(canonicalize_strategy_name("golden cross"), "golden cross");
        assert_eq!(canonicalize_strategy_name("GOLDEN CROSS"), "GOLDEN CROSS");
        // Internal keys are case-sensitive
        assert_eq!(canonicalize_strategy_name("goldencross"), "goldencross");
        assert_eq!(canonicalize_strategy_name("GOLDENCROSS"), "GOLDENCROSS");
    }

    // Mock strategy for testing
    struct MockStrategy {
        name: String,
    }

    impl MockStrategy {
        fn new(name: &str) -> Self {
            MockStrategy {
                name: name.to_string(),
            }
        }
    }

    impl Strategy for MockStrategy {
        fn name(&self) -> &str {
            &self.name
        }

        fn on_bar(&mut self, _bar: &Bar) -> Option<Vec<Signal>> {
            None
        }

        fn on_tick(&mut self, _tick: &alphafield_core::Tick) -> Option<Signal> {
            None
        }

        fn on_quote(&mut self, _quote: &alphafield_core::Quote) -> Option<Signal> {
            None
        }
    }

    impl MetadataStrategy for MockStrategy {
        fn metadata(&self) -> StrategyMetadata {
            StrategyMetadata {
                name: self.name.clone(),
                category: StrategyCategory::TrendFollowing,
                sub_type: Some("mock".to_string()),
                description: "Mock strategy for testing".to_string(),
                hypothesis_path: "hypotheses/mock.md".to_string(),
                required_indicators: vec![],
                expected_regimes: vec![MarketRegime::Bull],
                risk_profile: RiskProfile {
                    max_drawdown_expected: 0.10,
                    volatility_level: VolatilityLevel::Low,
                    correlation_sensitivity: CorrelationSensitivity::Low,
                    leverage_requirement: 1.0,
                },
            }
        }
    }

    #[test]
    fn test_strategy_registry_new() {
        let registry = StrategyRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_strategy_registry_register() {
        let registry = StrategyRegistry::new();
        let strategy = Arc::new(MockStrategy::new("TestStrategy"));

        let result = registry.register(strategy);
        assert!(result.is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_strategy_registry_get() {
        let registry = StrategyRegistry::new();
        let strategy = Arc::new(MockStrategy::new("TestStrategy"));

        registry.register(strategy).unwrap();

        let retrieved = registry.get("TestStrategy");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "TestStrategy");
    }

    #[test]
    fn test_strategy_registry_get_metadata() {
        let registry = StrategyRegistry::new();
        let strategy = Arc::new(MockStrategy::new("TestStrategy"));

        registry.register(strategy).unwrap();

        let metadata = registry.get_metadata("TestStrategy");
        assert!(metadata.is_some());

        let meta = metadata.unwrap();
        assert_eq!(meta.name, "TestStrategy");
        assert_eq!(meta.category, StrategyCategory::TrendFollowing);
    }

    #[test]
    fn test_strategy_registry_list_all() {
        let registry = StrategyRegistry::new();

        registry
            .register(Arc::new(MockStrategy::new("Strategy1")))
            .unwrap();
        registry
            .register(Arc::new(MockStrategy::new("Strategy2")))
            .unwrap();

        let strategies = registry.list_all();
        assert_eq!(strategies.len(), 2);
        assert!(strategies.contains(&"Strategy1".to_string()));
        assert!(strategies.contains(&"Strategy2".to_string()));
    }

    #[test]
    fn test_strategy_registry_list_by_category() {
        let registry = StrategyRegistry::new();

        registry
            .register(Arc::new(MockStrategy::new("Baseline1")))
            .unwrap();
        registry
            .register(Arc::new(MockStrategy::new("Baseline2")))
            .unwrap();

        let strategies = registry.list_by_category(StrategyCategory::TrendFollowing);
        assert_eq!(strategies.len(), 2);
    }

    #[test]
    fn test_strategy_registry_contains() {
        let registry = StrategyRegistry::new();
        let strategy = Arc::new(MockStrategy::new("TestStrategy"));

        assert!(!registry.contains("TestStrategy"));
        registry.register(strategy).unwrap();
        assert!(registry.contains("TestStrategy"));
    }

    #[test]
    fn test_strategy_classifier_trend_sensitivity() {
        let results = BacktestResults {
            regime_bull_return: 20.0,
            regime_bear_return: -10.0,
            ..Default::default()
        };

        assert!(StrategyClassifier::calculate_trend_sensitivity(&results));
    }

    #[test]
    fn test_strategy_classifier_time_horizon() {
        let mut results = BacktestResults {
            avg_trade_duration_hours: 2.0,
            ..Default::default()
        };

        assert_eq!(
            StrategyClassifier::estimate_time_horizon(&results),
            TimeHorizon::Intraday
        );

        results.avg_trade_duration_hours = 200.0;
        assert_eq!(
            StrategyClassifier::estimate_time_horizon(&results),
            TimeHorizon::ShortTerm
        );
    }

    #[test]
    fn test_strategy_classifier_signal_frequency() {
        let mut results = BacktestResults {
            total_trades: 100,
            test_period_days: 100.0,
            ..Default::default()
        };

        assert_eq!(
            StrategyClassifier::estimate_signal_frequency(&results),
            SignalFrequency::High
        );

        results.total_trades = 10;
        results.test_period_days = 100.0;
        assert_eq!(
            StrategyClassifier::estimate_signal_frequency(&results),
            SignalFrequency::Low
        );
    }
}
