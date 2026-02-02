//! Dashboard server binary

use alphafield_dashboard::server::run_server;
use tracing_appender::rolling;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up daily log rotation
    // Logs are written to .logs/alphafield.log (current day) and .logs/alphafield.log.YYYY-MM-DD (previous days)
    // A new log file is created automatically at midnight each day
    let file_appender = rolling::daily(".logs", "alphafield.log");

    // Initialize tracing subscriber to write to file
    // Use RUST_LOG env var to control log levels, e.g.:
    // RUST_LOG=debug or RUST_LOG=alphafield_backtest=trace
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "info,alphafield_dashboard=debug,alphafield_backtest=debug".into()
            }),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false), // Disable ANSI colors for file output
        )
        .init();

    tracing::info!("Starting AlphaField Dashboard Server");
    println!(
        "🚀 Dashboard server starting - logs written to .logs/alphafield.log with daily rotation"
    );

    run_server("0.0.0.0:8080").await?;
    Ok(())
}
