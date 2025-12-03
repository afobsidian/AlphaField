//! Dashboard server binary

use alphafield_dashboard::server::run_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_server("0.0.0.0:8080").await?;
    Ok(())
}
