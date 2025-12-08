//! Dashboard server binary

use alphafield_dashboard::server::run_server;
use std::fs;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create logs directory if it doesn't exist
    fs::create_dir_all("logs")?;

    // Create log file with append mode
    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/alphafield.log")?;

    // Initialize tracing subscriber to write to file
    // Use RUST_LOG env var to control log levels, e.g.:
    // RUST_LOG=debug or RUST_LOG=alphafield_backtest=trace
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,alphafield_dashboard=debug,alphafield_backtest=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(log_file)
                .with_ansi(false) // Disable ANSI colors for file output
        )
        .init();

    tracing::info!("Starting AlphaField Dashboard Server");
    println!("🚀 Dashboard server starting - logs written to logs/alphafield.log");
    
    run_server("0.0.0.0:8080").await?;
    Ok(())
}
