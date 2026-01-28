# AGENTS.md

This file provides guidelines for AI agents working on the AlphaField codebase.

## Build & Test Commands

### Essential Commands
```bash
# Build workspace
make build        # or: cargo build

# Run all tests
make test         # or: cargo test

# Format code
make fmt          # or: cargo fmt

# Lint (strict)
make lint         # or: cargo clippy --workspace --all-targets -- -D warnings
```

### Running Single Tests
```bash
# Run specific test by name
cargo test test_name

# Run test in specific crate
cargo test -p crate_name test_name

# Run test in specific binary
cargo test --bin bin_name test_name

# Run with output
cargo test -- --nocapture
```

### Examples & Database
```bash
make run-demo       # Run demo example
make run-backtest   # Run backtest example
make run-dashboard  # Run dashboard

make migrate        # Run database migrations
make reset-db       # Reset database
```

## Project Structure

This is a Rust workspace (Cargo workspace) with 6 crates:
- **core**: Core types and traits
- **data**: Market data fetching and storage
- **strategy**: Trading strategies
- **backtest**: Backtesting engine
- **execution**: Order execution
- **dashboard**: Web UI

## Code Style Guidelines

### Imports & Dependencies
```rust
// Timestamps - use chrono
use chrono::{DateTime, NaiveDate, Utc};

// Monetary values - use Decimal for precision
use rust_decimal::Decimal;

// Error handling - use thiserror
use thiserror::Error;

// Serialization - use serde
use serde::{Deserialize, Serialize};

// HTTP client - use reqwest with timeout
use reqwest::Client;
let client = Client::builder()
    .timeout(Duration::from_secs(30))
    .pool_max_idle_per_host(10)
    .build()?;

// Async runtime - use tokio
use tokio::time::sleep;
```

### Naming Conventions
- **Types/Structs/Enums**: PascalCase (`Trade`, `Portfolio`)
- **Functions/Variables**: snake_case (`calculate_metrics`, `total_value`)
- **Constants**: SCREAMING_SNAKE_CASE (`MAX_RETRIES`)
- **Acronyms**: Keep consistent (e.g., `ApiUrl`, not `api_url` or `APIUrl`)

### Types
- **Prices**: `f64` for market prices (crypto floats)
- **Money**: `Decimal` for monetary calculations (precision required)
- **Timestamps**: `chrono::DateTime<Utc>` or `chrono::NaiveDate` for dates
- **Errors**: `thiserror::Error` with `#[error("...")]` attributes

### Error Handling
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuantError {
    #[error("API error: {0}")]
    Api(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Propagate errors with ?
fn fetch_data() -> Result<Data, QuantError> {
    let response = client.get(url).send().await?;
    Ok(response.json().await?)
}
```

### Documentation
```rust
//! Module-level documentation - explains purpose and usage

/// Struct documentation - explains what it represents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Timestamp when trade occurred
    pub timestamp: DateTime<Utc>,

    /// Trade price (in USD)
    pub price: f64,

    /// Quantity traded
    pub quantity: Decimal,
}

/// Calculate Sharpe ratio for returns
///
/// # Arguments
/// * `returns` - Vector of returns
/// * `risk_free_rate` - Risk-free rate (annual)
///
/// # Examples
/// ```
/// let sharpe = calculate_sharpe(&returns, 0.02);
/// ```
pub fn calculate_sharpe(returns: &[f64], risk_free_rate: f64) -> f64 {
    // Implementation
}
```

### Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_metrics() {
        // Test implementation
    }

    #[tokio::test]
    async fn test_async_operation() {
        // Async test
    }
}
```

### Validation
All public types should implement validation:
```rust
impl Trade {
    pub fn validate(&self) -> Result<(), QuantError> {
        if self.price <= 0.0 {
            return Err(QuantError::Parse("Price must be positive".into()));
        }
        if self.quantity.is_negative() {
            return Err(QuantError::Parse("Quantity cannot be negative".into()));
        }
        Ok(())
    }
}
```

### HTTP Requests
Use reqwest with 30s timeout and rate limiting:
```rust
async fn fetch_data(url: &str) -> Result<Data, QuantError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let response = client.get(url).send().await?;

    // Rate limiting between requests
    tokio::time::sleep(Duration::from_millis(100)).await;

    Ok(response.json().await?)
}
```

## Environment Setup

Copy `.env.example` to `.env` and configure:
```bash
DATABASE_URL=postgresql://localhost/alphafield
EXCHANGE_API_KEY=your_api_key
EXCHANGE_API_SECRET=your_secret
RUST_LOG=info  # Set to debug for verbose logs
```

## Development Workflow

1. Always format before committing: `make fmt`
2. Run linter: `make lint` (fix warnings before PR)
3. Run tests: `make test` (ensure all pass)
4. Add tests for new functionality in `#[cfg(test)]` modules
5. Document public APIs with `///`
6. Add module docs with `//!` for new modules

## Copilot Rules Integration

- Always validate external data before using
- Use proper error types (`thiserror::Error`)
- Prefer explicit type annotations for public APIs
- Use async/await for I/O operations
- Rate limit API calls to avoid bans
- Log errors with context (`RUST_LOG=debug`)
