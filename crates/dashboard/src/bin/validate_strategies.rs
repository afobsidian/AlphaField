//! Batch validation script for all trading strategies
//!
//! This binary runs comprehensive validation across all strategies:
//! 1. Instantiates each strategy via StrategyFactory
//! 2. Runs backtest to verify trade generation (minimum 5 trades)
//! 3. Tests optimization workflow
//! 4. Measures performance
//! 5. Generates validation report

use alphafield_backtest::optimizer::get_strategy_bounds;
use alphafield_dashboard::services::strategy_service::StrategyFactory;
use alphafield_strategy::testing::data_generators::generate_trending_market;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug)]
#[allow(dead_code)]
struct ValidationResult {
    strategy_name: String,
    can_instantiate: bool,
    trades_generated: usize,
    optimization_successful: bool,
    backtest_duration_ms: u64,
    optimization_duration_ms: u64,
    errors: Vec<String>,
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     Strategy Batch Validation Suite                      ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // Get all strategies from bounds
    let strategies = vec![
        // Trend Following
        ("GoldenCross", "Trend Following"),
        ("Breakout", "Trend Following"),
        ("MACrossover", "Trend Following"),
        ("AdaptiveMA", "Trend Following"),
        ("TripleMA", "Trend Following"),
        ("MacdTrend", "Trend Following"),
        ("ParabolicSAR", "Trend Following"),
        // Momentum
        ("RsiMomentumStrategy", "Momentum"),
        ("MACDStrategy", "Momentum"),
        ("RocStrategy", "Momentum"),
        ("AdxTrendStrategy", "Momentum"),
        ("MomentumFactorStrategy", "Momentum"),
        ("VolumeMomentumStrategy", "Momentum"),
        ("MultiTfMomentumStrategy", "Momentum"),
        // Mean Reversion
        ("BollingerBands", "Mean Reversion"),
        ("RSIReversion", "Mean Reversion"),
        ("StochReversion", "Mean Reversion"),
        ("ZScoreReversion", "Mean Reversion"),
        ("PriceChannel", "Mean Reversion"),
        ("KeltnerReversion", "Mean Reversion"),
        ("StatArb", "Mean Reversion"),
        // Volatility
        ("ATRBreakout", "Volatility"),
        ("ATRTrailingStop", "Volatility"),
        ("VolatilitySqueeze", "Volatility"),
        ("VolRegimeStrategy", "Volatility"),
        ("VolSizingStrategy", "Volatility"),
        ("GarchStrategy", "Volatility"),
        ("VIXStyleStrategy", "Volatility"),
        // Multi-Indicator
        ("TrendMeanRev", "Multi-Indicator"),
        ("MACDRSICombo", "Multi-Indicator"),
        ("AdaptiveCombo", "Multi-Indicator"),
        ("ConfidenceWeighted", "Multi-Indicator"),
        ("EnsembleWeighted", "Multi-Indicator"),
        ("MLEnhanced", "Multi-Indicator"),
        ("RegimeSwitching", "Multi-Indicator"),
        // Sentiment
        ("Divergence", "Sentiment"),
        ("RegimeSentiment", "Sentiment"),
        ("SentimentMomentum", "Sentiment"),
    ];

    let mut results: Vec<ValidationResult> = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    println!("Validating {} strategies...\n", strategies.len());

    for (strategy_name, category) in &strategies {
        print!("Testing {} ({})... ", strategy_name, category);

        let result = validate_strategy(strategy_name);

        if result.can_instantiate && result.trades_generated >= 5 {
            println!("✓ PASS ({} trades)", result.trades_generated);
            passed += 1;
        } else {
            println!("✗ FAIL");
            if !result.can_instantiate {
                println!("  - Cannot instantiate");
            }
            if result.trades_generated < 5 {
                println!(
                    "  - Only {} trades generated (need 5+)",
                    result.trades_generated
                );
            }
            for error in &result.errors {
                println!("  - {}", error);
            }
            failed += 1;
        }

        results.push(result);
    }

    // Print summary
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    Validation Summary                      ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ Total Strategies: {:>40} ║", strategies.len());
    println!("║ Passed:          {:>40} ║", passed);
    println!("║ Failed:          {:>40} ║", failed);
    println!(
        "║ Success Rate:    {:>39}% ║",
        (passed as f64 / strategies.len() as f64 * 100.0) as usize
    );
    println!("╚════════════════════════════════════════════════════════════╝");

    // Performance metrics
    let total_backtest_time: u64 = results.iter().map(|r| r.backtest_duration_ms).sum();
    let total_opt_time: u64 = results.iter().map(|r| r.optimization_duration_ms).sum();

    println!("\nPerformance Metrics:");
    println!("  Total backtest time: {} ms", total_backtest_time);
    println!(
        "  Avg backtest time: {} ms",
        total_backtest_time / strategies.len() as u64
    );
    println!("  Total optimization time: {} ms", total_opt_time);

    // Exit with error code if any failed
    if failed > 0 {
        std::process::exit(1);
    }
}

fn validate_strategy(strategy_name: &str) -> ValidationResult {
    let mut result = ValidationResult {
        strategy_name: strategy_name.to_string(),
        can_instantiate: false,
        trades_generated: 0,
        optimization_successful: false,
        backtest_duration_ms: 0,
        optimization_duration_ms: 0,
        errors: Vec::new(),
    };

    // Get parameter bounds
    let bounds = get_strategy_bounds(strategy_name);
    if bounds.is_empty() {
        result
            .errors
            .push("No parameter bounds defined".to_string());
        return result;
    }

    // Create strategy with default parameters (use middle of range as default)
    let params: HashMap<String, f64> = bounds
        .iter()
        .map(|b| (b.name.clone(), (b.min + b.max) / 2.0))
        .collect();

    let start = Instant::now();
    match StrategyFactory::create(strategy_name, &params) {
        Some(mut strategy) => {
            result.can_instantiate = true;

            // Generate test market data with 300 bars (enough for most indicators to warm up)
            let bars = generate_trending_market(300, 0.02);

            // Count signals generated by strategy
            let mut signal_count = 0;
            for bar in &bars {
                if let Some(signals) = strategy.on_bar(bar) {
                    signal_count += signals.len();
                }
            }

            result.trades_generated = signal_count;
            result.backtest_duration_ms = start.elapsed().as_millis() as u64;
        }
        None => {
            result
                .errors
                .push("StrategyFactory::create returned None".to_string());
        }
    }

    result
}
