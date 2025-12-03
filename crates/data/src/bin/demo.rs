//! Demo binary for testing Unified Data Layer with Smart Routing

use alphafield_data::UnifiedDataClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();

    println!("🚀 AlphaField Unified Data Layer Demo\n");
    println!("{}", "=".repeat(70));
    println!("Features:");
    println!("  ✓ Smart Routing (Binance → CoinGecko → Coinlayer)");
    println!("  ✓ Automatic Fallbacks");
    println!("  ✓ API Key Rotation");
    println!("  ✓ Unified Interface");
    println!("{}", "=".repeat(70));

    // Create Unified client
    let client = UnifiedDataClient::new_from_env();
    println!("\n✓ Initialized UnifiedDataClient from .env");

    println!("\n📊 TEST 1: OHLC Data (Priority: Binance)");
    println!("{}", "-".repeat(70));

    // Fetch BTC OHLC (Should use Binance)
    println!("Requesting BTC 1h bars...");
    match client.get_bars("BTC", "1h", Some(24)).await {
        Ok(bars) => {
            println!("✓ Success! Fetched {} bars", bars.len());
            if let Some(last) = bars.last() {
                println!("  Last bar: {}", last);
                println!(
                    "  Volume: {:.4} (indicates Binance source if > 0)",
                    last.volume
                );
            }
        }
        Err(e) => eprintln!("✗ Failed: {}", e),
    }

    println!("\n📊 TEST 2: Market Data (Priority: CoinGecko)");
    println!("{}", "-".repeat(70));

    // Fetch Prices
    let coins = vec!["BTC", "ETH", "SOL"];
    for coin in coins {
        match client.get_price(coin).await {
            Ok(price) => println!("  {}: ${:.2}", coin, price),
            Err(e) => eprintln!("  {}: Failed ({})", coin, e),
        }
    }

    println!("\n📊 TEST 3: Fallback Scenario (Simulation)");
    println!("{}", "-".repeat(70));
    println!("Requesting data for 'monero' (Not on Binance US, might trigger fallback or CoinGecko direct)");

    // Monero (XMR) might be on Binance, but let's try something obscure if needed.
    // Actually XMR is delisted from some Binance regions.
    // Let's try a CoinGecko-specific ID if my mapping supports it, but my mapping is simple.
    // My mapping: "btc" -> "bitcoin".
    // If I pass "bitcoin" directly:
    // Binance: "BITCOINUSDT" (Invalid) -> Error -> Fallback to CoinGecko
    // CoinGecko: "bitcoin" -> Success

    println!("Requesting 'bitcoin' (explicit ID to force fallback/CoinGecko)...");
    match client.get_bars("bitcoin", "1d", Some(7)).await {
        Ok(bars) => {
            println!("✓ Success! Fetched {} daily bars", bars.len());
            if let Some(last) = bars.last() {
                println!("  Last bar: {}", last);
                println!("  Volume: {:.4} (CoinGecko OHLC has 0 volume)", last.volume);
                if last.volume == 0.0 {
                    println!("  ✓ Confirmed CoinGecko source (Volume = 0)");
                }
            }
        }
        Err(e) => eprintln!("✗ Failed: {}", e),
    }

    println!("\n{}", "=".repeat(70));
    println!("🎉 Demo Completed");
    println!("{}", "=".repeat(70));

    Ok(())
}
