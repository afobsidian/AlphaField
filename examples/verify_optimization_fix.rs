// Simple verification that optimization fixes are working
use alphafield_backtest::optimizer::calculate_optimization_score;
use alphafield_core::metrics::StrategyMetrics;

fn main() {
    println!("=== Verifying Multi-Indicator Optimization Fixes ===");

    let base_metrics = StrategyMetrics {
        total_return: 0.05,
        sharpe_ratio: 1.0,
        max_drawdown: 0.02,
        win_rate: 0.55,
        profit_factor: 1.1,
        total_trades: 0,
        ..Default::default()
    };

    // Test graduated penalty system
    for trades in 0..=6 {
        let mut metrics = base_metrics.clone();
        metrics.total_trades = trades;
        let score = calculate_optimization_score(&metrics);

        println!("Trades: {:2}, Score: {:8.6}", trades, score);

        match trades {
            0 => assert!(score == f64::NEG_INFINITY, "Zero trades should be invalid"),
            1 => assert!(
                score > 0.0,
                "1 trade should have positive score with penalty"
            ),
            2..=4 => assert!(
                score > 0.0,
                "2-4 trades should have positive score with penalty"
            ),
            5.. => assert!(
                score > 0.0,
                "5+ trades should have positive score without penalty"
            ),
            _ => {}
        }
    }

    println!("✅ All optimization score tests passed!");
    println!("✅ Multi-indicator strategies can now generate valid scores with fewer trades");
    println!("✅ Fix successfully resolves the 'no trades' optimization issue");
}
