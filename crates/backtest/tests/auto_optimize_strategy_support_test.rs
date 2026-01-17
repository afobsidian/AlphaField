//! Test for auto-optimization support of all strategies
//!
//! This test verifies that every strategy defined in the UI can be successfully
//! canonicalized and has optimization bounds defined.

use alphafield_backtest::optimizer::get_strategy_bounds;
use alphafield_strategy::framework::canonicalize_strategy_name;

/// All strategies defined in the dashboard UI (30 total strategies)
const UI_STRATEGIES: &[&str] = &[
    // Baseline Strategies (2)
    "HODL_Baseline",
    "Market_Average_Baseline",
    // Trend Following Strategies (8)
    "Golden Cross",
    "Breakout",
    "MA Crossover",
    "Adaptive MA",
    "Triple MA",
    "MACD Trend",
    "Parabolic SAR",
    "ADX Trend",
    // Mean Reversion Strategies (7)
    "Bollinger Bands Mean Reversion",
    "RSI Mean Reversion",
    "Stochastic Mean Reversion",
    "Z-Score Mean Reversion",
    "Price Channel Mean Reversion",
    "Keltner Channel Mean Reversion",
    "Statistical Arbitrage Mean Reversion",
    // Momentum Strategies (7)
    "RSI Momentum",
    "EMA-MACD Momentum",
    "ROC Momentum",
    "Momentum Factor",
    "Volume Momentum",
    "Multi-TF Momentum",
    // Volatility-Based Strategies (6)
    "ATR Breakout",
    "ATR Trailing Stop",
    "Volatility Squeeze",
    "VolRegimeStrategy",
    "Volatility-Adjusted Position Sizing",
    "GARCH-Based",
    "VIX-Style",
];

#[test]
fn test_all_strategies_canonicalize() {
    for strategy_name in UI_STRATEGIES {
        let canonical = canonicalize_strategy_name(strategy_name);

        // Verify canonicalization produces a non-empty result
        assert!(
            !canonical.is_empty(),
            "Strategy '{}' canonicalized to empty string",
            strategy_name
        );

        // Verify the canonical name is different from display name (unless already canonical)
        if strategy_name.contains(&canonical) {
            println!(
                "Strategy '{}' appears to be already canonical: '{}'",
                strategy_name, canonical
            );
        }

        println!("'{}' -> '{}'", strategy_name, canonical);
    }
}

#[test]
fn test_all_strategies_have_bounds() {
    for strategy_name in UI_STRATEGIES {
        let canonical = canonicalize_strategy_name(strategy_name);
        let bounds = get_strategy_bounds(&canonical);

        // Verify bounds exist for optimization
        assert!(
            !bounds.is_empty(),
            "Strategy '{}' (canonical: '{}') has no optimization bounds",
            strategy_name,
            canonical
        );

        println!(
            "Strategy '{}' has {} optimization parameters",
            strategy_name,
            bounds.len()
        );
    }
}

#[test]
fn test_all_strategies_have_required_components() {
    for strategy_name in UI_STRATEGIES {
        let canonical = canonicalize_strategy_name(strategy_name);

        // Verify canonicalization produces a non-empty result
        assert!(
            !canonical.is_empty(),
            "Strategy '{}' canonicalized to empty string",
            strategy_name
        );

        // Verify bounds exist for optimization
        let bounds = get_strategy_bounds(&canonical);
        assert!(
            !bounds.is_empty(),
            "Strategy '{}' (canonical: '{}') has no optimization bounds",
            strategy_name,
            canonical
        );

        println!(
            "Strategy '{}' -> '{}' ({} params)",
            strategy_name,
            canonical,
            bounds.len()
        );
    }
}

#[test]
fn test_problematic_triple_ma_strategy() {
    // Specific test for the "Triple MA" strategy that was failing
    let strategy_name = "Triple MA";
    let canonical = canonicalize_strategy_name(strategy_name);

    assert_eq!(
        canonical, "TripleMA",
        "Triple MA should canonicalize to TripleMA"
    );

    // Verify bounds exist
    let bounds = get_strategy_bounds(&canonical);
    assert!(
        !bounds.is_empty(),
        "TripleMA should have optimization bounds"
    );

    println!(
        "Triple MA strategy test passed - canonicalizes to '{}' with {} optimization parameters",
        canonical,
        bounds.len()
    );

    // Print the actual bounds for debugging
    for bound in &bounds {
        println!(
            "  - {}: {} to {} (step: {})",
            bound.name, bound.min, bound.max, bound.step
        );
    }
}

#[test]
fn test_canonicalization_idempotency() {
    // Test that canonicalizing an already canonical name returns the same name
    let canonical_names = vec![
        "GoldenCross",
        "TripleMA",
        "BollingerBands",
        "RSIReversion",
        "MacdTrend",
        "AdaptiveMA",
    ];

    for name in canonical_names {
        let canonicalized = canonicalize_strategy_name(name);
        assert_eq!(
            name, canonicalized,
            "Already canonical name '{}' should remain unchanged",
            name
        );
    }
}

#[test]
fn test_specific_strategies_that_were_failing() {
    // Test specific strategies that were mentioned in the error
    let failing_strategies = vec![
        ("Triple MA", "TripleMA"),
        ("Bollinger Bands", "BollingerBands"),
        ("RSI Reversion", "RSIReversion"),
        ("Rate of Change (ROC)", "RocStrategy"),
        ("ADX Trend", "AdxTrendStrategy"),
        ("ATR Breakout", "ATRBreakout"),
        ("ATR Trailing Stop", "ATRTrailingStop"),
        ("Volatility Squeeze", "VolatilitySqueeze"),
        ("GARCH-Based", "GarchStrategy"),
    ];

    for (display_name, expected_canonical) in failing_strategies {
        let canonical = canonicalize_strategy_name(display_name);
        assert_eq!(
            canonical, expected_canonical,
            "Strategy '{}' should canonicalize to '{}'",
            display_name, expected_canonical
        );

        let bounds = get_strategy_bounds(&canonical);
        assert!(
            !bounds.is_empty(),
            "Strategy '{}' (canonical: '{}') should have optimization bounds",
            display_name,
            canonical
        );
    }
}

#[test]
fn test_atr_strategies_specifically() {
    // Specific test for ATR strategies that were mentioned in the original error
    let atr_strategies = vec![
        ("ATR Breakout", "ATRBreakout"),
        ("ATR Trailing Stop", "ATRTrailingStop"),
    ];

    for (display_name, expected_canonical) in atr_strategies {
        let canonical = canonicalize_strategy_name(display_name);
        assert_eq!(
            canonical, expected_canonical,
            "ATR Strategy '{}' should canonicalize to '{}'",
            display_name, expected_canonical
        );

        let bounds = get_strategy_bounds(&canonical);
        assert!(
            !bounds.is_empty(),
            "ATR Strategy '{}' (canonical: '{}') should have optimization bounds",
            display_name,
            canonical
        );

        // Verify ATR-specific parameters exist
        let param_names: Vec<String> = bounds.iter().map(|b| b.name.clone()).collect();
        assert!(
            param_names.contains(&"atr_period".to_string()),
            "ATR Strategy '{}' should have atr_period parameter",
            display_name
        );
        assert!(
            param_names.contains(&"atr_multiplier".to_string()),
            "ATR Strategy '{}' should have atr_multiplier parameter",
            display_name
        );
    }
}
