# Contributing to AlphaField

Thank you for your interest in contributing to AlphaField! This guide will help you get started.

## Why This Guide Exists

This guide exists to ensure consistency across contributions and help you understand not just HOW to contribute, but WHY certain patterns are used. By following these guidelines, you'll produce code that:

- **Maintains consistency** with the existing codebase
- **Follows Rust best practices** for trading systems
- **Passes all automated checks** (formatting, linting, tests)
- **Is easy to review** and understand by maintainers

## Table of Contents

- [Prerequisites](#prerequisites)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Code Style Guidelines](#code-style-guidelines)
- [Testing Guidelines](#testing-guidelines)
- [Submitting Changes](#submitting-changes)

## Prerequisites

Before you begin, ensure you have the following installed:

### Required Tools

- **Rust** (stable toolchain) - Required for building the workspace
  - Why: AlphaField is a Rust workspace with 6 crates; stable toolchain ensures compatibility
  ```bash
  rustup install stable
  rustup default stable
  ```

- **PostgreSQL 14+** - Required for TimescaleDB and data storage
  - Why: The system uses TimescaleDB for time-series data storage and analytics
  ```bash
  # On Ubuntu/Debian
  sudo apt-get install postgresql postgresql-contrib

  # On macOS
  brew install postgresql@14
  ```

- **Docker & Docker Compose** - For running database in container
  - Why: Simplifies database setup and ensures consistent development environment
  ```bash
  # Install Docker Engine
  # Then Docker Compose
  ```

### Optional Tools

- **cargo-watch** - For auto-running tests on file changes
  ```bash
  cargo install cargo-watch
  ```

## Development Setup

### 1. Clone and Navigate

```bash
git clone https://github.com/yourorg/AlphaField.git
cd AlphaField
```

### 2. Set Up Database

**Option A: Docker (Recommended)**

Why Docker? Isolates the database, ensures version consistency, and simplifies cleanup.

```bash
# Start PostgreSQL with TimescaleDB extension
docker-compose up -d

# Wait for database to be ready (usually 10-15 seconds)
docker-compose logs -f
```

**Option B: Local PostgreSQL**

```bash
# Create database
createdb alphafield

# Enable TimescaleDB extension
psql -d alphafield -c "CREATE EXTENSION IF NOT EXISTS timescaledb;"
```

### 3. Configure Environment

Copy the example environment file and configure your settings:

```bash
cp .env.example .env
```

Edit `.env` with your settings:

```env
# Database connection
DATABASE_URL=postgresql://localhost/alphafield

# Exchange API credentials (if testing with real data)
EXCHANGE_API_KEY=your_api_key_here
EXCHANGE_API_SECRET=your_secret_here

# Logging level: error, warn, info, debug, trace
RUST_LOG=info  # Use debug for development
```

Why environment variables? Keeps secrets out of version control and allows configuration per environment.

### 4. Run Database Migrations

```bash
make migrate
# Or
cargo run --bin migrate
```

Why migrations? Ensures database schema matches the codebase and tracks structural changes.

### 5. Build the Workspace

```bash
make build
# Or
cargo build --workspace
```

If this succeeds, your environment is ready!

## Development Workflow

### Common Commands

AlphaField uses a Makefile for common tasks. Here's why these commands matter:

```bash
# Build the workspace
make build
# Why: Ensures all crates compile together, catches type mismatches

# Run all tests
make test
# Why: Verifies existing functionality isn't broken

# Format code (Rustfmt)
make fmt
# Why: Enforces consistent formatting across the codebase

# Lint code (Clippy)
make lint
# Why: Catches common mistakes, anti-patterns, and unsafe code

# Run a specific test
cargo test test_name
# Why: Fast iteration when debugging a specific test

# Run tests in a specific crate
cargo test -p crate_name test_name
# Why: Focus testing on changed components

# Run with output visible
cargo test -- --nocapture
# Why: Debug test failures by seeing print statements
```

### Running Examples

```bash
# Run demo example
make run-demo

# Run backtest example
make run-backtest

# Run dashboard
make run-dashboard
```

### Working with Individual Crates

The workspace has 6 crates with clear responsibilities:

| Crate | Purpose | When to Work On |
|-------|---------|-----------------|
| **core** | Types and traits (`Strategy`, `Bar`, `Trade`, `Order`) | Foundation changes |
| **data** | Market data fetching, TimescaleDB storage, quality monitoring | Data pipeline issues |
| **strategy** | Trading strategies, indicators, signals | Strategy implementations |
| **backtest** | Backtesting engine, optimization, Monte Carlo | Strategy validation |
| **execution** | Order execution, exchange API integration | Trading operations |
| **dashboard** | Web UI, REST API | Frontend and API changes |

### Example: Adding a New Indicator

When adding a new indicator to `crates/strategy`:

1. **Create the indicator implementation**
   ```bash
   # Navigate to strategy crate
   cd crates/strategy/src/indicators

   # Create new file: my_indicator.rs
   ```

2. **Implement the indicator**
   - Follow existing indicator patterns (e.g., `sma.rs`, `rsi.rs`)
   - Use `chrono` for timestamps
   - Return `Vec<Decimal>` for precision

3. **Add to module**
   - Edit `crates/strategy/src/indicators/mod.rs`
   - Add: `pub mod my_indicator;`

4. **Write tests**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_my_indicator() {
           // Test implementation
       }
   }
   ```

5. **Build and test**
   ```bash
   cargo build -p strategy
   cargo test -p strategy test_my_indicator
   ```

6. **Format and lint**
   ```bash
   make fmt
   make lint
   ```

## Code Style Guidelines

AlphaField follows Rust conventions with project-specific patterns. Here's WHY these patterns are used:

### Type Conventions

```rust
// Use chrono for timestamps - provides timezone awareness and rich operations
use chrono::{DateTime, Utc};

// Use Decimal for money - maintains precision for financial calculations
use rust_decimal::Decimal;

// Use f64 for prices - crypto prices are floating point
let price: f64 = 45000.50;

// Use Decimal for quantities, monetary values, and indicators
let quantity: Decimal = Decimal::from_str("1.5")?;
```

### Error Handling

Why `thiserror::Error`? Provides clean error types and automatic conversions.

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

// Propagate errors with ? operator
fn fetch_data() -> Result<Data, QuantError> {
    let response = client.get(url).send().await?;
    Ok(response.json().await?)
}
```

### Async Programming

Why `tokio`? Most widely used async runtime in Rust, excellent tooling support.

```rust
use tokio::time::sleep;

// Use async for I/O operations (HTTP, database)
async fn fetch_data() -> Result<Data, QuantError> {
    let response = client.get(url).send().await?;
    Ok(response.json().await?)
}

// Rate limit API calls
async fn fetch_with_rate_limit(url: &str) -> Result<Data, QuantError> {
    let response = client.get(url).send().await?;
    sleep(Duration::from_millis(100)).await; // 100ms between requests
    Ok(response.json().await?)
}
```

### HTTP Requests

Why 30s timeout? Balances reliability with responsiveness. Too short fails on slow networks; too long causes long hangs.

```rust
use reqwest::Client;

let client = Client::builder()
    .timeout(Duration::from_secs(30))
    .pool_max_idle_per_host(10)  // Reuse connections
    .build()?;
```

### Documentation Standards

- **Module docs**: Use `//!` at the top of the file to explain purpose
- **Item docs**: Use `///` for public functions, structs, methods
- **Examples**: Include `# Examples` sections in documentation

```rust
//! Calculate technical indicators for market data.

/// Calculate Simple Moving Average (SMA)
///
/// # Arguments
/// * `data` - Array of price data
/// * `period` - Period for SMA calculation
///
/// # Examples
/// ```
/// let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0];
/// let sma = calculate_sma(&prices, 3);
/// assert_eq!(sma, Some(vec![101.0, 102.0, 103.0]));
/// ```
pub fn calculate_sma(data: &[f64], period: usize) -> Option<Vec<f64>> {
    // Implementation
}
```

### Naming Conventions

```rust
// Types, structs, enums: PascalCase
pub struct Trade { ... }
pub enum QuantError { ... }

// Functions, variables: snake_case
fn calculate_metrics() { ... }
let total_value = ...;

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;

// Acronyms: Keep consistent
pub struct ApiUrl { ... }  // Not API_URL or api_url
```

## Testing Guidelines

Testing is critical for financial systems. Here's why:

### Why Test Thoroughly

- **Financial impact**: Bugs can cause real monetary loss
- **Data integrity**: Backtests must be reproducible
- **Strategy correctness**: Strategies must behave as intended
- **Performance**: Backtests must complete in reasonable time

### Unit Tests

Test individual functions and methods in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_returns() {
        let prices = vec![100.0, 101.0, 102.0];
        let returns = calculate_returns(&prices);
        assert_eq!(returns, vec![0.01, 0.0099...]);
    }
}
```

### Integration Tests

Test multiple components working together:

```rust
#[tokio::test]
async fn test_fetch_and_store_data() {
    // Start with clean database
    // Fetch data from API
    // Store in database
    // Verify storage
}
```

### Async Tests

Use `#[tokio::test]` for async functions:

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = fetch_data().await.unwrap();
    assert!(result.is_valid());
}
```

### Test Organization

- Keep tests in `#[cfg(test)]` modules
- Use descriptive test names (`test_calculate_sma_edge_case` vs `test1`)
- Test both success and error paths
- Include edge cases (empty data, single data point, large datasets)

## Submitting Changes

### Before Submitting

1. **Format code**
   ```bash
   make fmt
   ```

2. **Run linter** (fix all warnings)
   ```bash
   make lint
   ```

3. **Run tests** (ensure all pass)
   ```bash
   make test
   ```

4. **Update documentation**
   - Add `///` docs for new public APIs
   - Update READMEs with examples
   - Document WHY changes were made

### Creating a Pull Request

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Commit changes** (use clear, descriptive messages)
   ```bash
   git add .
   git commit -m "Add Bollinger Bands indicator with tests"
   ```

   Good commit messages explain WHY, not just WHAT:
   - ✅ "Add Bollinger Bands indicator with tests - needed for mean reversion strategies"
   - ❌ "Add indicator"

3. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   # Then create PR on GitHub
   ```

### Pull Request Template

When creating a PR, include:

```markdown
## Description
Brief description of changes.

## Why
Why are these changes needed? What problem do they solve?

## Changes
- [ ] Code changes
- [ ] Tests added/updated
- [ ] Documentation updated

## Testing
How did you test these changes?

- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing

## Checklist
- [ ] Code formatted with `make fmt`
- [ ] Linter passes with `make lint`
- [ ] All tests pass with `make test`
- [ ] Documentation updated (README, doc comments)
```

## Getting Help

- **Issues**: Report bugs or request features via GitHub Issues
- **Discussions**: Use GitHub Discussions for questions
- **Docs**: Check the `doc/` directory for detailed guides

## Thank You!

By following these guidelines, you help ensure AlphaField remains a high-quality, maintainable, and reliable trading system.
