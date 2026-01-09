//! Baseline Strategies - Phase 12.1
//!
//! This module provides baseline strategies for comparison purposes:
//! - HODL (Buy and Hold) baseline
//! - Market Average baseline (equal-weighted portfolio)

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use alphafield_core::{Bar, Signal, SignalType, Strategy};

// ============ HODL Baseline ============

/// Simple buy and hold strategy for baseline comparison
///
/// # Strategy Logic
/// - **Buy Signal**: Generated on the first bar processed
/// - **Sell Signal**: Never generated (holds indefinitely)
///
/// # Purpose
/// Serves as a baseline to compare active strategies against passive holding.
/// In crypto markets, this represents the "HODL" strategy popular among
/// long-term investors.
///
/// # Example
/// ```
/// use alphafield_strategy::baseline::HoldBaseline;
///
/// let mut strategy = HoldBaseline::new();
/// // Process bars - will buy on first bar
/// ```
pub struct HoldBaseline {
    /// Entry price when position was opened
    entry_price: Option<f64>,
    /// Whether a position has been entered
    entered: bool,
}

impl HoldBaseline {
    /// Creates a new HODL baseline strategy
    pub fn new() -> Self {
        HoldBaseline {
            entry_price: None,
            entered: false,
        }
    }

    /// Returns the entry price if position is open
    pub fn entry_price(&self) -> Option<f64> {
        self.entry_price
    }

    /// Returns whether a position is currently held
    pub fn is_entered(&self) -> bool {
        self.entered
    }
}

impl Default for HoldBaseline {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for HoldBaseline {
    fn name(&self) -> &str {
        "HODL_Baseline"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        if !self.entered {
            self.entry_price = Some(bar.close);
            self.entered = true;

            return Some(vec![Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Buy,
                strength: 1.0,
                metadata: Some("HODL Entry: Buy and hold indefinitely".to_string()),
            }]);
        }

        // Once entered, never sell - true HODL strategy
        None
    }

    fn on_tick(&mut self, _tick: &alphafield_core::Tick) -> Option<Signal> {
        None
    }

    fn on_quote(&mut self, _quote: &alphafield_core::Quote) -> Option<Signal> {
        None
    }
}

impl MetadataStrategy for HoldBaseline {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "HODL_Baseline".to_string(),
            category: StrategyCategory::Baseline,
            sub_type: Some("buy_and_hold".to_string()),
            description: "Simple buy and hold strategy for baseline comparison. Buys on first bar and holds indefinitely. Represents the passive investment approach popular in crypto markets.".to_string(),
            hypothesis_path: "hypotheses/baseline/hodl.md".to_string(),
            required_indicators: vec![],
            expected_regimes: vec![MarketRegime::Bull],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.50, // 50% drawdown potential in crypto
                volatility_level: VolatilityLevel::High,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::Baseline
    }
}

// ============ Market Average Baseline ============

/// Market average baseline strategy
///
/// # Strategy Logic
/// - This strategy is managed at the portfolio level
/// - Represents holding an equally-weighted portfolio of all assets
/// - Individual symbol level doesn't generate signals
///
/// # Purpose
/// Serves as a baseline representing average market performance across
/// a diversified portfolio of crypto assets.
///
/// # Example
/// ```
/// use alphafield_strategy::baseline::MarketAverageBaseline;
///
/// let symbols = vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()];
/// let strategy = MarketAverageBaseline::equal_weighted(symbols);
/// // Managed at portfolio level
/// ```
pub struct MarketAverageBaseline {
    /// List of symbols in the portfolio
    symbols: Vec<String>,
    /// Weight for each symbol (sum should be 1.0)
    weights: Vec<f64>,
}

impl MarketAverageBaseline {
    /// Creates a new market average baseline strategy with custom weights
    ///
    /// # Arguments
    /// * `symbols` - Vector of symbol names
    /// * `weights` - Vector of weights (should sum to 1.0)
    pub fn new(symbols: Vec<String>, weights: Vec<f64>) -> Self {
        MarketAverageBaseline { symbols, weights }
    }

    /// Creates an equally-weighted market average baseline
    ///
    /// # Arguments
    /// * `symbols` - Vector of symbol names
    ///
    /// # Returns
    /// Strategy with equal weights (1/n for each of n symbols)
    pub fn equal_weighted(symbols: Vec<String>) -> Self {
        let n = symbols.len();
        let weights = vec![1.0 / n as f64; n];
        Self::new(symbols, weights)
    }

    /// Returns the list of symbols in the portfolio
    pub fn symbols(&self) -> &[String] {
        &self.symbols
    }

    /// Returns the weights for each symbol
    pub fn weights(&self) -> &[f64] {
        &self.weights
    }

    /// Returns the weight for a specific symbol
    pub fn weight_for_symbol(&self, symbol: &str) -> Option<f64> {
        self.symbols
            .iter()
            .position(|s| s == symbol)
            .map(|idx| self.weights[idx])
    }
}

impl Strategy for MarketAverageBaseline {
    fn name(&self) -> &str {
        "Market_Average_Baseline"
    }

    fn on_bar(&mut self, _bar: &Bar) -> Option<Vec<Signal>> {
        // Managed at portfolio level, not individual strategy level
        // Returns None as position management is done at portfolio level
        None
    }

    fn on_tick(&mut self, _tick: &alphafield_core::Tick) -> Option<Signal> {
        None
    }

    fn on_quote(&mut self, _quote: &alphafield_core::Quote) -> Option<Signal> {
        None
    }
}

impl MetadataStrategy for MarketAverageBaseline {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Market_Average_Baseline".to_string(),
            category: StrategyCategory::Baseline,
            sub_type: Some("market_index".to_string()),
            description: format!(
                "Equally weighted portfolio of {} assets: {}. Managed at portfolio level.",
                self.symbols.len(),
                self.symbols.join(", ")
            ),
            hypothesis_path: "hypotheses/baseline/market_average.md".to_string(),
            required_indicators: vec![],
            expected_regimes: vec![MarketRegime::Bull],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.40,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::Baseline
    }
}

// ============ Unit Tests ============

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    /// Helper to create test bars
    fn create_test_bar(timestamp: i64, price: f64) -> Bar {
        Bar {
            timestamp: Utc
                .timestamp_opt(timestamp, 0)
                .single()
                .expect("valid timestamp for test bar"),
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_hold_baseline_creation() {
        let strategy = HoldBaseline::new();
        assert_eq!(strategy.name(), "HODL_Baseline");
        assert!(!strategy.is_entered());
        assert!(strategy.entry_price().is_none());
    }

    #[test]
    fn test_hold_baseline_default() {
        let strategy = HoldBaseline::default();
        assert_eq!(strategy.name(), "HODL_Baseline");
    }

    #[test]
    fn test_hold_baseline_entry() {
        let mut strategy = HoldBaseline::new();
        let bar = create_test_bar(0, 100.0);

        let signal = strategy.on_bar(&bar);

        assert!(signal.is_some());
        let signals = signal.unwrap();
        assert_eq!(signals.len(), 1);

        let sig = &signals[0];
        assert_eq!(sig.signal_type, SignalType::Buy);
        assert_eq!(sig.strength, 1.0);
        assert!(sig.metadata.is_some());
    }

    #[test]
    fn test_hold_baseline_single_entry() {
        let mut strategy = HoldBaseline::new();

        // First bar - should generate entry
        let signal1 = strategy.on_bar(&create_test_bar(0, 100.0));
        assert!(signal1.is_some());
        assert!(strategy.is_entered());

        // Second bar - should not generate signal
        let signal2 = strategy.on_bar(&create_test_bar(1, 101.0));
        assert!(signal2.is_none());
        assert!(strategy.is_entered());
    }

    #[test]
    fn test_hold_baseline_reset_behavior() {
        let mut strategy = HoldBaseline::new();

        strategy.on_bar(&create_test_bar(0, 100.0));
        assert!(strategy.is_entered());

        // HODL doesn't have a reset method, but we can test that it continues holding
        strategy.on_bar(&create_test_bar(1, 101.0));
        assert!(strategy.is_entered());

        strategy.on_bar(&create_test_bar(2, 102.0));
        assert!(strategy.is_entered());

        // No sell signals ever
        let signal = strategy.on_bar(&create_test_bar(3, 90.0));
        assert!(signal.is_none());
    }

    #[test]
    fn test_hold_baseline_metadata() {
        let strategy = HoldBaseline::new();
        let metadata = strategy.metadata();

        assert_eq!(metadata.name, "HODL_Baseline");
        assert_eq!(metadata.category, StrategyCategory::Baseline);
        assert_eq!(metadata.sub_type, Some("buy_and_hold".to_string()));
        assert_eq!(metadata.hypothesis_path, "hypotheses/baseline/hodl.md");
        assert!(metadata.required_indicators.is_empty());
        assert!(metadata.expected_regimes.contains(&MarketRegime::Bull));
        assert_eq!(metadata.risk_profile.max_drawdown_expected, 0.50);
        assert_eq!(
            metadata.risk_profile.volatility_level,
            VolatilityLevel::High
        );
        assert_eq!(metadata.risk_profile.leverage_requirement, 1.0);
    }

    #[test]
    fn test_market_average_creation() {
        let symbols = vec!["BTC".to_string(), "ETH".to_string()];
        let weights = vec![0.6, 0.4];
        let strategy = MarketAverageBaseline::new(symbols, weights);

        assert_eq!(strategy.name(), "Market_Average_Baseline");
        assert_eq!(strategy.symbols().len(), 2);
        assert_eq!(strategy.weights().len(), 2);
    }

    #[test]
    fn test_market_average_equal_weighted() {
        let symbols = vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()];
        let strategy = MarketAverageBaseline::equal_weighted(symbols);

        assert_eq!(strategy.weights().len(), 3);
        assert_eq!(strategy.weights()[0], 1.0 / 3.0);
        assert_eq!(strategy.weights()[1], 1.0 / 3.0);
        assert_eq!(strategy.weights()[2], 1.0 / 3.0);
    }

    #[test]
    fn test_market_average_custom_weights() {
        let symbols = vec!["BTC".to_string(), "ETH".to_string()];
        let weights = vec![0.6, 0.4];
        let strategy = MarketAverageBaseline::new(symbols, weights);

        assert_eq!(strategy.weight_for_symbol("BTC"), Some(0.6));
        assert_eq!(strategy.weight_for_symbol("ETH"), Some(0.4));
        assert_eq!(strategy.weight_for_symbol("SOL"), None);
    }

    #[test]
    fn test_market_average_metadata() {
        let symbols = vec!["BTC".to_string(), "ETH".to_string()];
        let strategy = MarketAverageBaseline::equal_weighted(symbols);
        let metadata = strategy.metadata();

        assert_eq!(metadata.name, "Market_Average_Baseline");
        assert_eq!(metadata.category, StrategyCategory::Baseline);
        assert_eq!(metadata.sub_type, Some("market_index".to_string()));
        assert!(metadata.description.contains("2 assets"));
        assert!(metadata.description.contains("BTC"));
        assert!(metadata.description.contains("ETH"));
        assert_eq!(metadata.risk_profile.max_drawdown_expected, 0.40);
    }

    #[test]
    fn test_market_average_no_signals() {
        let symbols = vec!["BTC".to_string()];
        let mut strategy = MarketAverageBaseline::equal_weighted(symbols);
        let bar = create_test_bar(0, 100.0);

        // Market average doesn't generate signals at strategy level
        let signal = strategy.on_bar(&bar);
        assert!(signal.is_none());
    }

    #[test]
    fn test_hold_baseline_price_tracking() {
        let mut strategy = HoldBaseline::new();
        let bar = create_test_bar(0, 100.0);

        strategy.on_bar(&bar);

        assert_eq!(strategy.entry_price(), Some(100.0));
        assert!(strategy.is_entered());
    }
}
