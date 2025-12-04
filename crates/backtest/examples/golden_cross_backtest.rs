use alphafield_backtest::{BacktestEngine, SlippageModel, StrategyAdapter};
use alphafield_strategy::GoldenCrossStrategy;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Try to load .env, but fallback to manual parsing if it fails
    if dotenvy::dotenv().is_err() {
        // Manual fallback: read DATABASE_URL directly from .env
        if let Ok(contents) = std::fs::read_to_string(".env") {
            for line in contents.lines() {
                if line.starts_with("DATABASE_URL=") {
                    let value = line.trim_start_matches("DATABASE_URL=");
                    std::env::set_var("DATABASE_URL", value);
                    break;
                }
            }
        }
    }
    
    println!("=== Golden Cross Strategy Backtest ===\n");

    // 1. Setup Engine
    let mut engine = BacktestEngine::new(
        100_000.0,                           // Initial Cash
        0.001,                               // 0.1% Fee
        SlippageModel::FixedPercent(0.0005), // 0.05% Slippage
    );

    // 2. Fetch/Load Historical Data
    let symbol = "BTC";
    let interval = "1h";
    
    println!("Connecting to database...");
    let db = match alphafield_data::DatabaseClient::new_from_env().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            eprintln!("Ensure DATABASE_URL is set in .env and Postgres is running.");
            return Ok(());
        }
    };
    
    let bars = if db.exists(symbol, interval).await? {
        println!("Loading historical data from database...");
        db.load_bars(symbol, interval).await?
    } else {
        println!("Fetching historical data from API...");
        let client = alphafield_data::UnifiedDataClient::new_from_env();
        // Fetch 1000 hours (~41 days)
        let bars = client.get_bars(symbol, interval, Some(1000)).await?;
        println!("Saving {} bars to database...", bars.len());
        db.save_bars(symbol, interval, &bars).await?;
        bars
    };

    println!("Loaded {} bars of historical data", bars.len());
    if let Some(first) = bars.first() {
        println!("Start: {}", first.timestamp);
    }
    if let Some(last) = bars.last() {
        println!("End:   {}", last.timestamp);
    }
    println!(
        "Price range: ${:.2} - ${:.2}\n",
        bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min),
        bars.iter()
            .map(|b| b.high)
            .fold(f64::NEG_INFINITY, f64::max)
    );

    engine.add_data(symbol, bars.clone());

    // 3. Create Strategy
    // Golden Cross: Fast SMA(20) crosses Slow SMA(50)
    let golden_cross = GoldenCrossStrategy::new(20, 50);

    // 4. Wrap with Adapter
    let adapter = StrategyAdapter::new(
        golden_cross,
        symbol,
        0.01, // Trade 0.01 BTC per signal (~$1k at $100k/BTC)
    );

    engine.set_strategy(Box::new(adapter));

    // 5. Run Backtest
    println!("Running backtest...\n");
    let metrics = match engine.run() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Backtest failed: {}", e);
            return Ok(());
        }
    };

    // 6. Report Results
    println!("=== Backtest Results ===");
    println!("Total Return:    {:.2}%", metrics.total_return * 100.0);
    println!("CAGR:            {:.2}%", metrics.cagr * 100.0);
    println!("Sharpe Ratio:    {:.2}", metrics.sharpe_ratio);
    println!("Max Drawdown:    {:.2}%", metrics.max_drawdown * 100.0);
    println!("Volatility:      {:.2}%", metrics.volatility * 100.0);

    let final_prices =
        std::collections::HashMap::from([(symbol.to_string(), bars.last().unwrap().close)]);
    let final_equity = engine.portfolio.total_equity(&final_prices);
    println!("Final Equity:    ${:.2}", final_equity);
    println!("Initial Capital: $100,000.00");
    println!("Profit/Loss:     ${:.2}", final_equity - 100_000.0);

    Ok(())
}
