use alphafield_core::Bar;
use alphafield_data::{DataPersister, DataPipeline, DatabaseClient, MarketEvent};
use chrono::Utc;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Try to load .env, but fallback to manual parsing if it fails
    // (dotenvy doesn't support array syntax like [val1, val2])
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

    println!("🚀 Starting Ingestion Pipeline Demo");

    // 1. Setup Database
    let db = match DatabaseClient::new_from_env().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error connecting to DB: {}", e);
            eprintln!("Please ensure DATABASE_URL is set and Postgres is running.");
            return Ok(());
        }
    };

    // 2. Create Pipeline
    let pipeline = DataPipeline::new(100);

    // 3. Start Persister (Consumer)
    let persister = DataPersister::new(db.clone(), &pipeline, "BTC", "1m");
    tokio::spawn(async move {
        persister.run().await;
    });

    // 4. Simulate Live Data (Producer)
    println!("📡 Simulating live data feed...");
    let mut price = 50000.0;

    for i in 0..5 {
        time::sleep(Duration::from_secs(1)).await;

        price += (i as f64 * 10.0).sin() * 50.0; // Random-ish walk

        let bar = Bar {
            timestamp: Utc::now(),
            open: price,
            high: price + 10.0,
            low: price - 10.0,
            close: price + 5.0,
            volume: 1.5,
        };

        println!("-> Publishing Bar: {}", bar);
        pipeline.publish(MarketEvent::Bar(bar))?;
    }

    // Give persister time to process
    time::sleep(Duration::from_secs(1)).await;
    println!("✅ Demo completed");

    Ok(())
}
