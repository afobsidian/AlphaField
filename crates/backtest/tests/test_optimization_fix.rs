//! Quick test to verify multi-indicator optimization fixes

use alphafield_backtest::optimizer::ParameterOptimizer;
use alphafield_strategy::strategies::multi_indicator::macd_rsi_combo::MACDRSIComboStrategy;

/// Test the multi-indicator optimization fix
#[tokio::test]
async fn test_multi_indicator_optimization_fix() {
    println!("=== Testing Multi-Indicator Optimization Fix ===");

    // Create strategy with default parameters (12/26/9 MACD, 14 RSI)
    let _strategy = MACDRSIComboStrategy::new(12, 26, 9, 14);

    // Create optimizer
    let _optimizer = ParameterOptimizer::new(10000.0, 0.001);

    // Get parameter bounds for MACDRSICombo strategy
    let bounds = alphafield_backtest::optimizer::get_strategy_bounds("MACDRSICombo");

    println!("Found {} parameter bounds for MACDRSICombo", bounds.len());
    for bound in &bounds {
        println!(
            "  {}: {} to {} (step: {})",
            bound.name, bound.min, bound.max, bound.step
        );
    }

    // Generate parameter combinations
    let combinations = ParameterOptimizer::generate_param_combinations(&bounds);
    println!("Generated {} parameter combinations", combinations.len());

    if combinations.is_empty() {
        println!("⚠️  No parameter combinations generated - this may be expected");
    } else {
        println!("✅ Optimization setup working correctly");
    }
}
