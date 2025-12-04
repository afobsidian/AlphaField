//! Multi-Strategy Backtest
//! 
//! Runs multiple strategies on the same data and compares their performance.

use alphafield_backtest::{BacktestEngine, SlippageModel, StrategyAdapter};
use alphafield_strategy::{GoldenCrossStrategy, MeanReversionStrategy, MomentumStrategy, RsiStrategy};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Load .env with fallback parsing
    if dotenvy::dotenv().is_err() {
        if let Ok(contents) = std::fs::read_to_string(".env") {
            for line in contents.lines() {
                if line.starts_with("DATABASE_URL=") {
                    std::env::set_var("DATABASE_URL", line.trim_start_matches("DATABASE_URL="));
                    break;
                }
            }
        }
    }
    
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║           AlphaField Multi-Strategy Backtest                     ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // 1. Load Historical Data
    let symbol = "BTC";
    let interval = "1h";
    
    println!("📡 Connecting to database...");
    let db = match alphafield_data::DatabaseClient::new_from_env().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Ok(());
        }
    };
    
    let bars = if db.exists(symbol, interval).await? {
        println!("📊 Loading historical data from database...");
        db.load_bars(symbol, interval).await?
    } else {
        println!("📥 Fetching historical data from API...");
        let client = alphafield_data::UnifiedDataClient::new_from_env();
        let bars = client.get_bars(symbol, interval, Some(1000)).await?;
        db.save_bars(symbol, interval, &bars).await?;
        bars
    };
    
    println!("✓ Loaded {} bars ({} to {})\n", 
        bars.len(),
        bars.first().map(|b| b.timestamp.format("%Y-%m-%d").to_string()).unwrap_or_default(),
        bars.last().map(|b| b.timestamp.format("%Y-%m-%d").to_string()).unwrap_or_default()
    );

    // 2. Define Strategies to Test
    let strategies: Vec<(&str, Box<dyn Fn() -> Box<dyn alphafield_core::Strategy + Send + Sync>>)> = vec![
        ("Golden Cross (20/50)", Box::new(|| Box::new(GoldenCrossStrategy::new(20, 50)))),
        ("Golden Cross (10/30)", Box::new(|| Box::new(GoldenCrossStrategy::new(10, 30)))),
        ("RSI (14, 30/70)", Box::new(|| Box::new(RsiStrategy::new(14, 30.0, 70.0)))),
        ("RSI (7, 25/75)", Box::new(|| Box::new(RsiStrategy::new(7, 25.0, 75.0)))),
        ("Mean Reversion (20, 2.0)", Box::new(|| Box::new(MeanReversionStrategy::new(20, 2.0)))),
        ("Mean Reversion (20, 1.5)", Box::new(|| Box::new(MeanReversionStrategy::new(20, 1.5)))),
        ("Momentum (50, 12/26/9)", Box::new(|| Box::new(MomentumStrategy::new(50, 12, 26, 9)))),
    ];

    // 3. Run Backtests
    println!("┌────────────────────────────┬──────────┬──────────┬──────────┬──────────┐");
    println!("│ Strategy                   │ Return   │ Sharpe   │ Max DD   │ Final $  │");
    println!("├────────────────────────────┼──────────┼──────────┼──────────┼──────────┤");

    let mut results = Vec::new();

    for (name, strategy_fn) in &strategies {
        let mut engine = BacktestEngine::new(
            100_000.0,
            0.001,
            SlippageModel::FixedPercent(0.0005),
        );
        
        engine.add_data(symbol, bars.clone());
        
        let adapter = StrategyAdapter::new(
            strategy_fn(),
            symbol,
            0.01, // Trade 0.01 BTC per signal
        );
        
        engine.set_strategy(Box::new(adapter));
        
        match engine.run() {
            Ok(metrics) => {
                let final_prices = std::collections::HashMap::from([
                    (symbol.to_string(), bars.last().unwrap().close)
                ]);
                let final_equity = engine.portfolio.total_equity(&final_prices);
                
                println!("│ {:<26} │ {:>7.2}% │ {:>8.2} │ {:>7.2}% │ ${:>7.0} │",
                    name,
                    metrics.total_return * 100.0,
                    metrics.sharpe_ratio,
                    metrics.max_drawdown * 100.0,
                    final_equity
                );
                
                results.push((name.to_string(), metrics.total_return, metrics.sharpe_ratio, final_equity));
            }
            Err(e) => {
                println!("│ {:<26} │ ERROR: {} │", name, e);
            }
        }
    }

    println!("└────────────────────────────┴──────────┴──────────┴──────────┴──────────┘\n");

    // 4. Summary
    if let Some((best_name, best_return, _, _)) = results.iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    {
        println!("🏆 Best Strategy: {} ({:+.2}%)", best_name, best_return * 100.0);
    }

    if let Some((worst_name, worst_return, _, _)) = results.iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    {
        println!("📉 Worst Strategy: {} ({:+.2}%)", worst_name, worst_return * 100.0);
    }

    Ok(())
}
