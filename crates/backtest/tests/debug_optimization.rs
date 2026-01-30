//! Debug tests for optimization issues with multi-indicator strategies

use alphafield_backtest::metrics::PerformanceMetrics;
use alphafield_backtest::optimizer::{
    calculate_composite_score, get_strategy_bounds, ParameterOptimizer,
};
use alphafield_strategy::strategies::multi_indicator::macd_rsi_combo::MACDRSIComboStrategy;
use alphafield_strategy::MetadataStrategy;

/// Test optimizer scoring with actual metrics structure
#[test]
fn test_optimizer_scoring_with_performance_metrics() {
    println!("=== Testing Optimizer Scoring with PerformanceMetrics ===");

    // Test case 1: Zero trades should return negative infinity
    let metrics_zero = PerformanceMetrics {
        total_return: 0.1,
        sharpe_ratio: 1.5,
        max_drawdown: 0.05,
        win_rate: 0.6,
        total_trades: 0,
        profit_factor: 1.2,
        ..Default::default()
    };
    let score_zero = calculate_composite_score(
        metrics_zero.sharpe_ratio,
        metrics_zero.total_return,
        metrics_zero.max_drawdown,
        metrics_zero.win_rate,
        metrics_zero.total_trades,
    );
    assert!(
        score_zero == f64::NEG_INFINITY,
        "Zero trades should return negative infinity"
    );
    println!("✅ Zero trades correctly return negative infinity");

    // Test case 2: Positive score with sufficient trades
    let metrics_positive = PerformanceMetrics {
        total_return: 0.15,
        sharpe_ratio: 1.8,
        max_drawdown: 0.08,
        win_rate: 0.65,
        total_trades: 10,
        profit_factor: 1.5,
        ..Default::default()
    };
    let score_positive = calculate_composite_score(
        metrics_positive.sharpe_ratio,
        metrics_positive.total_return,
        metrics_positive.max_drawdown,
        metrics_positive.win_rate,
        metrics_positive.total_trades,
    );
    assert!(
        score_positive > 0.0,
        "Valid metrics should produce positive score"
    );
    println!(
        "✅ Valid metrics produce positive score: {:.4}",
        score_positive
    );
}

/// Test parameter bounds for MACD+RSI Combo strategy
#[test]
fn test_macd_rsi_combo_parameter_bounds() {
    println!("=== Testing MACD+RSI Combo Parameter Bounds ===");

    // Note: get_strategy_bounds uses strategy_name, so we need the canonical name
    // The MACDRSIComboStrategy uses "MACD+RSI Combo" as display name
    let bounds = get_strategy_bounds("MACDRSICombo");

    println!("Parameter bounds for MACD+RSI Combo:");
    for bound in &bounds {
        println!(
            "  {}: {:.1} to {:.1} (step: {:.1})",
            bound.name, bound.min, bound.max, bound.step
        );
    }

    // The bounds should be populated if the strategy is registered
    if bounds.is_empty() {
        println!("⚠️  No bounds found - strategy may not be registered in optimizer");
    } else {
        println!("✅ Found {} parameter bounds", bounds.len());
    }
}

/// Test MACDRSIComboStrategy creation with correct arguments
#[test]
fn test_macd_rsi_combo_strategy_creation() {
    println!("=== Testing MACDRSIComboStrategy Creation ===");

    // MACDRSIComboStrategy::new() requires 4 arguments: macd_fast, macd_slow, macd_signal, rsi_period
    let strategy = MACDRSIComboStrategy::new(12, 26, 9, 14);

    // Verify strategy has metadata
    let metadata = strategy.metadata();
    assert_eq!(metadata.name, "MACD+RSI Combo");
    assert!(!metadata.description.is_empty());

    println!("✅ Strategy created with name: {}", metadata.name);
    println!(
        "   Description: {}",
        metadata.description.lines().next().unwrap_or("N/A")
    );

    // Verify config is accessible
    let config = strategy.config();
    assert_eq!(config.macd_fast, 12);
    assert_eq!(config.macd_slow, 26);
    assert_eq!(config.rsi_period, 14);

    println!(
        "✅ Config accessible: MACD({}/{}/{}), RSI({})",
        config.macd_fast, config.macd_slow, config.macd_signal, config.rsi_period
    );
}

/// Test ParameterOptimizer structure
#[test]
fn test_parameter_optimizer_creation() {
    println!("=== Testing ParameterOptimizer Creation ===");

    let optimizer = ParameterOptimizer::new(10000.0, 0.001);

    assert_eq!(optimizer.initial_capital, 10000.0);
    assert_eq!(optimizer.fee_rate, 0.001);

    println!("✅ ParameterOptimizer created:");
    println!("   Initial capital: {:.2}", optimizer.initial_capital);
    println!("   Fee rate: {:.4}", optimizer.fee_rate);
}

/// Test trade count penalty in scoring
#[test]
fn test_trade_count_penalty() {
    println!("=== Testing Trade Count Penalty ===");

    // Same base metrics, different trade counts
    let base_sharpe = 1.5;
    let base_return = 0.1;
    let base_drawdown = 0.05;
    let base_win_rate = 0.6;

    // 1 trade: 50% penalty
    let score_1 =
        calculate_composite_score(base_sharpe, base_return, base_drawdown, base_win_rate, 1);

    // 5 trades: no penalty
    let score_5 =
        calculate_composite_score(base_sharpe, base_return, base_drawdown, base_win_rate, 5);

    let ratio = score_1 / score_5;
    assert!(
        (ratio - 0.5).abs() < 0.001,
        "1-trade score should be 50% of 5-trade score"
    );

    println!("✅ Trade penalty correctly applied:");
    println!("   Score with 1 trade: {:.4}", score_1);
    println!("   Score with 5 trades: {:.4}", score_5);
    println!("   Ratio: {:.4} (expected 0.5)", ratio);
}

/// Test edge cases in scoring
#[test]
fn test_scoring_edge_cases() {
    println!("=== Testing Scoring Edge Cases ===");

    // Extreme negative Sharpe
    let score_neg_sharpe = calculate_composite_score(-5.0, 0.1, 0.05, 0.6, 10);
    println!("✅ Negative Sharpe score: {:.4}", score_neg_sharpe);

    // Extreme positive return
    let score_high_return = calculate_composite_score(1.5, 3.0, 0.05, 0.6, 10);
    println!("✅ High return score: {:.4}", score_high_return);

    // High drawdown (should penalize heavily)
    let score_high_dd = calculate_composite_score(1.5, 0.1, 0.5, 0.6, 10);
    println!("✅ High drawdown score: {:.4}", score_high_dd);

    // Zero trades should be negative infinity
    let score_zero_trades = calculate_composite_score(1.5, 0.1, 0.05, 0.6, 0);
    assert_eq!(score_zero_trades, f64::NEG_INFINITY);
    println!("✅ Zero trades score: negative infinity");
}
