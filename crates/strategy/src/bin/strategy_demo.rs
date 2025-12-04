//! Demo binary for testing Strategy Engine

use alphafield_core::Strategy;
use alphafield_data::UnifiedDataClient;
use alphafield_strategy::{
    GoldenCrossStrategy, MeanReversionStrategy, MomentumStrategy, RsiStrategy,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load .env
    dotenvy::dotenv().ok();

    println!("🚀 AlphaField Strategy Engine Demo\n");

    // 1. Fetch Data
    let client = UnifiedDataClient::new_from_env();
    println!("Fetching BTC 1h data...");

    // Fetch enough data for indicators to warm up
    let bars = client.get_bars("BTC", "1h", Some(500)).await?;
    println!("✓ Fetched {} bars\n", bars.len());

    // 2. Initialize Strategies
    let mut golden_cross = GoldenCrossStrategy::new(10, 30); // Fast=10, Slow=30 for demo
    let mut rsi_strat = RsiStrategy::new(14, 30.0, 70.0);
    let mut mean_reversion = MeanReversionStrategy::new(20, 2.0);
    let mut momentum = MomentumStrategy::new(50, 12, 26, 9);

    println!("📊 Running Strategies:");
    println!("{}", "=".repeat(70));
    println!(
        "{:<20} | {:<10} | {:<10} | Info",
        "Time", "Strategy", "Signal"
    );
    println!("{}", "-".repeat(70));

    // 3. Run Event Loop
    for bar in bars {
        // Update Golden Cross
        if let Some(signals) = golden_cross.on_bar(&bar) {
            for mut signal in signals {
                signal.symbol = "BTC".to_string(); // Inject symbol
                print_signal(&signal, "GoldenCross");
            }
        }

        // Update RSI
        if let Some(signals) = rsi_strat.on_bar(&bar) {
            for mut signal in signals {
                signal.symbol = "BTC".to_string();
                print_signal(&signal, "RSI");
            }
        }

        // Update Mean Reversion
        if let Some(signals) = mean_reversion.on_bar(&bar) {
            for mut signal in signals {
                signal.symbol = "BTC".to_string();
                print_signal(&signal, "MeanRev");
            }
        }

        // Update Momentum
        if let Some(signals) = momentum.on_bar(&bar) {
            for mut signal in signals {
                signal.symbol = "BTC".to_string();
                print_signal(&signal, "Momentum");
            }
        }
    }
    println!("{}", "=".repeat(70));

    Ok(())
}

fn print_signal(signal: &alphafield_core::Signal, strategy_name: &str) {
    let type_str = match signal.signal_type {
        alphafield_core::SignalType::Buy => "🟢 BUY",
        alphafield_core::SignalType::Sell => "🔴 SELL",
        alphafield_core::SignalType::Hold => "⚪ HOLD",
    };

    println!(
        "{:<20} | {:<10} | {:<10} | {}",
        signal.timestamp.format("%Y-%m-%d %H:%M"),
        strategy_name,
        type_str,
        signal.metadata.as_deref().unwrap_or("")
    );
}
