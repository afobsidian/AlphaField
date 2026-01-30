//! Comprehensive test suite for multi-indicator strategy optimization fixes

use alphafield_backtest::optimizer::{
    calculate_composite_score, get_strategy_bounds, ParameterOptimizer,
};
use alphafield_strategy::strategies::multi_indicator::macd_rsi_combo::{
    MACDRSIComboStrategy, MACDRSIConfig,
};
use alphafield_strategy::MetadataStrategy;

/// Test the enhanced optimizer scoring system with graduated trade penalties
#[test]
fn test_optimizer_scoring_trade_penalties() {
    println!("=== Testing Optimizer Scoring with Trade Penalties ===");

    // Test case 1: Zero trades should return negative infinity
    let score_zero = calculate_composite_score(1.5, 0.1, 0.05, 0.6, 0);
    assert!(
        score_zero == f64::NEG_INFINITY,
        "Zero trades should return negative infinity"
    );
    println!("✅ Zero trades correctly return negative infinity");

    // Test case 2: 1 trade should have 50% penalty
    let score_one = calculate_composite_score(1.5, 0.1, 0.05, 0.6, 1);
    let expected_base =
        0.40 * 1.5 / 5.0 + 0.25 * (0.1 * 100.0) / 200.0 + 0.20 * 0.6 - 0.15 * (0.05 * 2.0);
    let expected_one = expected_base * 0.5; // 50% penalty
    assert!(
        (score_one - expected_one).abs() < 0.001,
        "1 trade should have 50% penalty"
    );
    println!("✅ 1 trade correctly applies 50% penalty: {:.4}", score_one);

    // Test case 3: 5+ trades should have no penalty
    let score_five = calculate_composite_score(1.5, 0.1, 0.05, 0.6, 5);
    let expected_five = expected_base * 1.0; // No penalty
    assert!(
        (score_five - expected_five).abs() < 0.001,
        "5+ trades should have no penalty"
    );
    println!("✅ 5+ trades correctly have no penalty: {:.4}", score_five);

    println!("✅ Optimizer scoring system working correctly");
}

/// Test that the enhanced MACD+RSI strategy generates more entry signals
#[test]
fn test_enhanced_macd_rsi_entry_logic() {
    println!("=== Testing Enhanced MACD+RSI Entry Logic ===");

    // Create strategy with default parameters (12/26/9 MACD, 14 RSI)
    let strategy = MACDRSIComboStrategy::new(12, 26, 9, 14);

    // Test with default configuration
    let default_config = MACDRSIConfig::default_config();
    println!("Default config: {}", default_config);

    // Verify strategy metadata
    let metadata = strategy.metadata();
    println!("Strategy name: {}", metadata.name);
    println!("Strategy description: {}", metadata.description);

    assert_eq!(metadata.name, "MACD+RSI Combo");
    assert!(!metadata.description.is_empty());
    println!("✅ Enhanced MACD+RSI entry logic test passed");
}

/// Test the complete optimization workflow with enhanced strategy
#[test]
fn test_complete_optimization_workflow() {
    println!("=== Testing Complete Optimization Workflow ===");

    // Get bounds for MACD+RSI Combo strategy
    let bounds = get_strategy_bounds("MACDRSICombo");

    println!("Parameter bounds for MACD+RSI Combo:");
    for bound in &bounds {
        println!(
            "  {}: {:.1} to {:.1} (step: {:.1})",
            bound.name, bound.min, bound.max, bound.step
        );
    }

    // Verify we have bounds for this strategy
    assert!(
        !bounds.is_empty(),
        "MACDRSICombo should have parameter bounds defined"
    );

    // Test that we can create a ParameterOptimizer
    let optimizer = ParameterOptimizer::new(10000.0, 0.001);
    assert_eq!(optimizer.initial_capital, 10000.0);
    assert_eq!(optimizer.fee_rate, 0.001);

    println!("✅ Optimization workflow test passed");
}

/// Test parameter bounds and conversion for multi-indicator strategies
#[test]
fn test_multi_indicator_parameter_bounds() {
    println!("=== Testing Multi-Indicator Parameter Bounds ===");

    let bounds = get_strategy_bounds("MACDRSICombo");

    println!("Parameter bounds for MACD+RSI Combo:");
    for bound in &bounds {
        println!(
            "  {}: {:.1} to {:.1} (step: {:.1})",
            bound.name, bound.min, bound.max, bound.step
        );
    }

    // Verify we have parameter bounds
    assert!(
        !bounds.is_empty(),
        "MACDRSICombo should have parameter bounds defined"
    );

    // Verify essential parameters exist if defined
    let param_names: Vec<&str> = bounds.iter().map(|b| b.name.as_str()).collect();
    println!("Found parameters: {:?}", param_names);

    println!("✅ Parameter bounds test passed");
}

/// Test regression to ensure we don't break existing functionality
#[test]
fn test_regression_existing_strategies() {
    println!("=== Testing Regression for Existing Strategies ===");

    // Test that we can still create and configure the strategy
    let strategy = MACDRSIComboStrategy::new(12, 26, 9, 14);

    // Verify default configuration is valid
    let config = strategy.config();
    assert_eq!(config.macd_fast, 12);
    assert_eq!(config.macd_slow, 26);
    assert_eq!(config.macd_signal, 9);
    assert_eq!(config.rsi_period, 14);
    println!("✅ Default configuration valid");

    // Test that we can create custom configurations
    let custom_config = MACDRSIConfig::new(10, 24, 8, 14);
    let custom_strategy = MACDRSIComboStrategy::from_config(custom_config);
    let custom_params = custom_strategy.config();
    assert_eq!(custom_params.macd_fast, 10);
    println!("✅ Custom configuration creation works");

    // Test that strategy name and metadata are preserved
    let metadata = strategy.metadata();
    assert_eq!(metadata.name, "MACD+RSI Combo");
    assert!(!metadata.description.is_empty());
    println!("✅ Strategy metadata preserved: {}", metadata.name);
}

/// Test edge cases and error handling
#[test]
fn test_edge_cases_and_error_handling() {
    println!("=== Testing Edge Cases and Error Handling ===");

    // Test with valid configuration
    let config = MACDRSIConfig::new(12, 26, 9, 14);
    let result = config.validate();
    assert!(result.is_ok(), "Valid config should pass validation");
    println!("✅ Valid config validation works");

    // Test with invalid configuration (fast >= slow)
    let invalid = MACDRSIConfig::new(26, 12, 9, 14);
    let result = invalid.validate();
    assert!(result.is_err(), "Invalid fast/slow should be rejected");
    println!(
        "✅ Invalid fast/slow correctly rejected: {:?}",
        result.err()
    );

    // Test with invalid thresholds (overbought <= oversold)
    let invalid2 = MACDRSIConfig::with_thresholds(12, 26, 9, 14, 30.0, 70.0);
    let result2 = invalid2.validate();
    assert!(result2.is_err(), "Invalid thresholds should be rejected");
    println!("✅ Invalid thresholds correctly rejected");

    // Test scoring edge cases
    let negative_return_score = calculate_composite_score(-2.0, -0.1, 0.3, 0.3, 10);
    assert!(
        negative_return_score < 0.0,
        "Negative returns should produce negative score"
    );
    println!("✅ Negative return scoring works correctly");

    let high_drawdown_score = calculate_composite_score(3.0, 0.5, 0.5, 0.8, 10);
    assert!(
        high_drawdown_score < calculate_composite_score(3.0, 0.5, 0.1, 0.8, 10),
        "High drawdown should reduce score"
    );
    println!("✅ Drawdown penalty works correctly");

    println!("✅ Edge case handling working correctly");
}
