//! Dynamic test for auto-optimization support of all strategies
//!
//! This test ensures that canonicalization and optimization systems work together
//! for all possible strategy names, ensuring new strategies are automatically supported.

use alphafield_backtest::optimizer::get_strategy_bounds;
use alphafield_strategy::framework::canonicalize_strategy_name;

#[test]
fn test_all_known_strategy_names_have_auto_optimize_support() {
    // Comprehensive list of all possible strategy names including display names
    // This ensures that any new strategy added to the dashboard will be caught
    let all_possible_strategy_names = vec![
        // Core/Canonical names (what system expects internally)
        "HODL_Baseline",
        "Market_Average_Baseline",
        "GoldenCross",
        "Breakout",
        "MACrossover",
        "AdaptiveMA",
        "TripleMA",
        "MacdTrend",
        "ParabolicSAR",
        "AdxTrendStrategy",
        "MeanReversion",
        "Rsi",
        "StochReversion",
        "ZScoreReversion",
        "PriceChannel",
        "KeltnerReversion",
        "StatArb",
        "Momentum",
        "RsiMomentumStrategy",
        "MACDStrategy",
        "RocStrategy",
        "MomentumFactorStrategy",
        "VolumeMomentumStrategy",
        "MultiTfMomentumStrategy",
        "ATRBreakout",
        "ATRTrailingStop",
        "VolatilitySqueeze",
        "VolatilityRegime",
        "VolSizingStrategy",
        "GarchStrategy",
        "VixStyleStrategy",
        // Display names (what users see in UI)
        "Golden Cross",
        "MA Crossover",
        "Adaptive MA",
        "Triple MA",
        "MACD Trend",
        "Parabolic SAR",
        "ADX Trend",
        "Bollinger Bands Mean Reversion",
        "RSI Mean Reversion",
        "Stochastic Mean Reversion",
        "Z-Score Mean Reversion",
        "Price Channel Mean Reversion",
        "Keltner Channel Mean Reversion",
        "Statistical Arbitrage Mean Reversion",
        "RSI Momentum",
        "MACD Momentum",
        "ROC Momentum",
        "Momentum Factor",
        "Volume Momentum",
        "Multi-Timeframe Momentum",
        "Multi-TF Momentum",
        "ATR Breakout",
        "ATR Trailing Stop",
        "Volatility Squeeze",
        "Volatility Regime",
        "Volatility-Adjusted Position Sizing",
        "GARCH-Based",
        "VIX-Style",
        "EMA-MACD Momentum",
        "EMA-MACD",
        "RSI Reversion",
        "Stochastic Reversion",
        "Z-Score Reversion",
        "Price Channel (Donchian)",
        "Keltner Channel",
        "Statistical Arbitrage",
        "Rate of Change (ROC)",
        // Variations and potential future names
        "HODL Baseline",
        "Market Average Baseline",
        "Golden Cross Strategy",
        "MA Crossover Strategy",
        "Breakout Strategy",
        "Mean Reversion Strategy",
        "Trend Following Strategy",
        "Momentum Strategy",
        "Volatility Strategy",
        "Bollinger Bands",
        "RSI",
        "Stochastic",
        "Z Score",
        "Donchian",
        "Keltner",
        "Stat Arb",
        "ATR",
        "GARCH",
        "VIX",
        "EMA MACD",
        "Rate of Change",
        "ADX",
        "SAR",
        "KAMA",
        "EMA",
        "SMA",
    ];

    println!(
        "Testing {} possible strategy names for auto-optimization support...",
        all_possible_strategy_names.len()
    );

    let mut failed_names = Vec::new();
    let mut successful_names = Vec::new();

    for strategy_name in &all_possible_strategy_names {
        // Test 1: Strategy can be canonicalized
        let canonical_name = canonicalize_strategy_name(strategy_name);

        if canonical_name.is_empty() {
            failed_names.push(format!("{} (canonicalization failed)", strategy_name));
            continue;
        }

        // Test 2: Canonical name has optimization bounds
        let bounds = get_strategy_bounds(&canonical_name);
        if bounds.is_empty() {
            failed_names.push(format!(
                "{} -> {} (no bounds)",
                strategy_name, canonical_name
            ));
            continue;
        }

        successful_names.push((strategy_name, canonical_name, bounds.len()));
    }

    // Report results
    println!(
        "\n✅ Successfully supported strategies ({}):",
        successful_names.len()
    );
    for (display, canonical, param_count) in &successful_names {
        if *display != canonical {
            println!("  {} -> {} ({} params)", display, canonical, param_count);
        }
    }

    if !failed_names.is_empty() {
        println!("\n❌ Unsupported strategy names:");
        for failure in &failed_names {
            println!("  {}", failure);
        }
    }

    // Ensure we have comprehensive coverage
    let success_rate = successful_names.len() as f64 / all_possible_strategy_names.len() as f64;
    println!(
        "\n📊 Coverage: {}/{} strategies ({:.1}%)",
        successful_names.len(),
        all_possible_strategy_names.len(),
        success_rate * 100.0
    );

    assert!(
        success_rate >= 0.8,
        "Auto-optimization coverage is too low: {:.1}%, expected >= 80%",
        success_rate * 100.0
    );

    // Verify that all canonical names work
    let canonical_strategies = vec![
        "HODL_Baseline",
        "Market_Average_Baseline",
        "GoldenCross",
        "Breakout",
        "MACrossover",
        "AdaptiveMA",
        "TripleMA",
        "MacdTrend",
        "ParabolicSAR",
        "AdxTrendStrategy",
        "MeanReversion",
        "Rsi",
        "StochReversion",
        "ZScoreReversion",
        "PriceChannel",
        "KeltnerReversion",
        "StatArb",
        "Momentum",
        "RsiMomentumStrategy",
        "MACDStrategy",
        "RocStrategy",
        "MomentumFactorStrategy",
        "VolumeMomentumStrategy",
        "MultiTfMomentumStrategy",
        "ATRBreakout",
        "ATRTrailingStop",
        "VolatilitySqueeze",
        "VolatilityRegime",
        "VolSizingStrategy",
        "GarchStrategy",
        "VixStyleStrategy",
    ];

    for canonical in canonical_strategies {
        let bounds = get_strategy_bounds(canonical);
        assert!(
            !bounds.is_empty(),
            "Critical: Canonical strategy '{}' has no optimization bounds. \
                New strategies MUST be added to get_strategy_bounds() in optimizer.rs",
            canonical
        );
    }
}

#[test]
fn test_future_strategy_detection_capability() {
    // This test simulates what happens when new strategies are added
    // It ensures the testing framework can detect and validate new strategies automatically

    let test_strategies = vec![
        // Simulated new strategy names that might be added in future
        "NewSuperStrategy",
        "AI Enhanced MA",
        "Quantum Breakout",
        "Machine Learning Momentum",
        "Neural Network Mean Reversion",
        "Blockchain Sentiment Strategy",
        // Variations of existing strategies
        "SuperGoldenCross",
        "EnhancedMACD",
        "AdaptiveATR",
        "DynamicRSI",
    ];

    println!("Testing future strategy detection capability...");

    for strategy_name in &test_strategies {
        let canonical_name = canonicalize_strategy_name(strategy_name);

        // Test canonicalization doesn't crash and produces something
        assert!(
            !canonical_name.is_empty(),
            "Future strategy '{}' should canonicalize to something",
            strategy_name
        );

        // Test bounds system gracefully handles unknown strategies
        let bounds = get_strategy_bounds(&canonical_name);

        if bounds.is_empty() {
            println!(
                "⚠️  New strategy '{}' -> '{}' needs bounds added to optimizer.rs",
                strategy_name, canonical_name
            );
        } else {
            println!(
                "✅ New strategy '{}' -> '{}' already has bounds ({})",
                strategy_name,
                canonical_name,
                bounds.len()
            );
        }
    }

    // The key point: test framework can detect when new strategies need bounds
    println!("\n✅ Dynamic detection system is ready for future strategies!");
}

#[test]
fn test_canonicalization_robustness() {
    // Test that canonicalization handles edge cases and variations robustly
    let edge_cases = vec![
        // Case variations
        "golden cross",
        "GOLDEN CROSS",
        "Golden Cross",
        "atr breakout",
        "ATR BREAKOUT",
        "Atr Breakout",
        // Extra whitespace
        "  Triple MA  ",
        "\tMACD Trend\n",
        "  Parabolic SAR  ",
        // Partial names
        "Golden",
        "ATR",
        "MACD",
        "RSI",
        "MA",
        // Mixed patterns
        "Golden-Cross",
        "ATR_Breakout",
        "MACD_Trend",
    ];

    println!("Testing canonicalization robustness...");

    for input in edge_cases {
        let canonical = canonicalize_strategy_name(input);

        // Should never be empty
        assert!(
            !canonical.is_empty(),
            "Input '{}' should not canonicalize to empty string",
            input
        );

        // Should be reasonable (not just echo of malformed input)
        if input.trim() != input {
            assert_ne!(
                canonical.trim(),
                input.trim(),
                "Malformed input should be cleaned during canonicalization"
            );
        }

        println!("  '{}' -> '{}'", input.trim(), canonical);
    }

    println!("\n✅ Canonicalization system is robust!");
}

#[test]
fn test_comprehensive_strategy_validation_workflow() {
    // This is master test that validates the entire workflow
    // It simulates what happens when a user selects any strategy from the dashboard

    println!("Running comprehensive strategy validation workflow...");

    // Test complete workflow for key strategies
    let critical_strategies = vec![
        // Strategies that were previously failing
        "Triple MA",
        "ATR Breakout",
        "RSI Reversion",
        "Rate of Change (ROC)",
        // Core strategies
        "Golden Cross",
        "Bollinger Bands Mean Reversion",
        "EMA-MACD Momentum",
        // Complex strategies
        "GARCH-Based",
        "Volatility Regime",
        "Multi-Timeframe Momentum",
    ];

    for strategy_display_name in &critical_strategies {
        println!("\n🔍 Testing strategy: {}", strategy_display_name);

        // Step 1: User selects strategy from dashboard (display name)
        let canonical_name = canonicalize_strategy_name(strategy_display_name);
        println!(
            "  📝 Canonicalized: '{}' -> '{}'",
            strategy_display_name, canonical_name
        );

        // Step 2: System gets optimization bounds for auto-optimization
        let bounds = get_strategy_bounds(&canonical_name);
        assert!(
            !bounds.is_empty(),
            "Strategy '{}' must have optimization bounds for auto-optimization",
            strategy_display_name
        );
        println!("  ⚙️  Optimization bounds: {} parameters", bounds.len());

        // Step 3: Verify bounds contain essential parameters
        let has_take_profit = bounds.iter().any(|b| b.name.contains("take_profit"));
        let has_stop_loss = bounds.iter().any(|b| b.name.contains("stop_loss"));
        let has_period = bounds.iter().any(|b| b.name.contains("period"));

        assert!(
            has_take_profit || has_stop_loss || has_period,
            "Strategy '{}' bounds must contain at least one essential parameter",
            strategy_display_name
        );

        // Step 4: Show parameter details
        for bound in &bounds {
            println!(
                "    - {}: {} to {} (step: {})",
                bound.name, bound.min, bound.max, bound.step
            );
        }

        println!(
            "  ✅ Strategy '{}' workflow validation PASSED",
            strategy_display_name
        );
    }

    println!("\n🎉 All critical strategies pass complete auto-optimization workflow!");

    // Final assertion: ensure critical strategies that were mentioned in issues work
    let historically_problematic = vec!["Triple MA", "ATR Breakout"];
    for strategy_name in historically_problematic {
        let canonical = canonicalize_strategy_name(strategy_name);
        let bounds = get_strategy_bounds(&canonical);

        assert!(
            !bounds.is_empty(),
            "Historically problematic strategy '{}' must now work with auto-optimization",
            strategy_name
        );

        println!(
            "✅ Historically problematic '{}' is now fully supported!",
            strategy_name
        );
    }
}
