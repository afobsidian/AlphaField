# Strategy Registry Enhancement - Phase 12.7

## Overview

This enhancement replaces the hardcoded strategy mapping in the validation CLI with a dynamic, extensible strategy registry system. This provides better maintainability, automatic strategy discovery, and improved user experience.

## Problem Statement

### Before Enhancement
```rust
// Hardcoded mapping in validate_strategy.rs
fn create_strategy(name: &str) -> Result<Box<dyn Strategy>> {
    let normalized_name = name.to_lowercase().replace("_", "");
    match normalized_name.as_str() {
        "golden_cross" => Ok(Box::new(StrategyAdapter::new(
            GoldenCrossStrategy::default(), "BTCUSDT", 10000.0
        ))),
        "rsi" => Ok(Box::new(StrategyAdapter::new(
            RsiStrategy::default(), "BTCUSDT", 10000.0
        ))),
        // ... more hardcoded cases
        _ => Err(anyhow::anyhow!("Unknown strategy"))
    }
}
```

**Issues:**
- Adding new strategies requires modifying CLI code
- No dynamic discovery of available strategies
- Inconsistent error messages
- No way to list strategies from the CLI
- Hardcoded symbol and capital values

## Solution Implemented

### Architecture

The solution uses a **factory-based registry** pattern:

```rust
// Factory function type
type StrategyFactory = fn() -> Box<dyn StrategyWithMetadata>;

// Global registry with lazy initialization
lazy_static! {
    static ref STRATEGY_FACTORY_REGISTRY: HashMap<String, (StrategyFactory, StrategyMetadata)> = {
        let mut registry = HashMap::new();
        
        // Macro for clean registration
        macro_rules! register_strategy {
            ($strategy:ty) => {
                let instance = <$strategy>::default();
                let metadata = instance.metadata();
                let canonical_name = canonicalize_strategy_name(&metadata.name);
                let factory: StrategyFactory = || Box::new(<$strategy>::default());
                registry.insert(canonical_name, (factory, metadata));
            };
        }
        
        // Register strategies
        register_strategy!(GoldenCrossStrategy);
        register_strategy!(RsiStrategy);
        // ... more strategies
    };
}
```

### Key Components

1. **Strategy Registry**
   - Lazy-initialized static HashMap
   - Maps canonical names → (factory function, metadata)
   - Thread-safe via lazy_static

2. **Factory Pattern**
   - Each registration includes a factory function
   - Creates fresh instances on demand
   - Ensures clean state for each backtest

3. **Name Normalization**
   - Supports multiple input formats:
     - Underscored: `golden_cross`, `macd_trend`
     - Spaced: `Golden Cross`, `MACD Trend`
     - Dashed: `golden-cross`, `macd-trend`
   - Uses existing `canonicalize_strategy_name()` function

4. **StrategyAdapter Integration**
   - Wraps core strategies (Signal-based) for backtest (Order-based)
   - Properly handles symbol and capital parameters
   - Maintains position state

## Implementation Details

### Files Modified

**`crates/backtest/src/bin/validate_strategy.rs`**
- Replaced hardcoded `create_strategy()` with registry lookup
- Added `STRATEGY_FACTORY_REGISTRY` with factory functions
- Added `list-strategies` CLI command
- Enhanced `create_strategy()` to accept symbol and capital
- Added `print_strategy_info()` helper for formatted output

**`crates/backtest/Cargo.toml`**
- Added `lazy_static` dependency

### Registered Strategies (with Default impl)

| Category | Strategies |
|----------|-----------|
| Trend Following | GoldenCross, MacdTrend |
| Mean Reversion | Rsi, BollingerBands |
| Sentiment | Divergence, RegimeSentiment, SentimentMomentum |

**Note:** Only strategies with `Default` implementations are registered. This ensures all strategies can be created without parameters.

## Usage Examples

### List All Strategies
```bash
cargo run --bin validate_strategy -- list-strategies
```

**Output:**
```
Available Strategies
==================================================

Trend Following (trendfollowing)

  • GoldenCross
    Category: trendfollowing
    Type: moving_average_crossover
    Golden Cross strategy using 50 and 200 period SMAs with 5.0% TP and 5.0% SL.
    Expected regimes: Bull, Trending

  • MacdTrend
    Category: trendfollowing
    Type: macd_trend
    MACD Trend strategy using 12, 26, 9 periods with 5.0% TP and 3.0% SL.
    Expected regimes: Bull, Trending

Mean Reversion (meanreversion)

  • Rsi
    Category: meanreversion
    Type: rsi_based
    RSI Mean Reversion strategy using 14 period RSI with bounds [30, 70]...
    Expected regimes: Sideways

  • BollingerBands
    Category: meanreversion
    Type: bollinger_bands
    Mean Reversion strategy using Bollinger Bands with period 20 and 2.0...
    Expected regimes: Sideways

...

Total strategies available: 7
```

### List by Category
```bash
cargo run --bin validate_strategy -- list-strategies --category trend_following
```

### Validate Strategy (Multiple Name Formats)
```bash
# All of these work:
cargo run --bin validate_strategy -- validate --strategy golden_cross --symbol BTC --interval 1d
cargo run --bin validate_strategy -- validate --strategy "Golden Cross" --symbol BTC --interval 1d
cargo run --bin validate_strategy -- validate --strategy golden-cross --symbol BTC --interval 1d
cargo run --bin validate_strategy -- validate --strategy GoldenCross --symbol BTC --interval 1d
```

### Validate with Custom Capital
```bash
cargo run --bin validate_strategy -- validate \
  --strategy rsi \
  --symbol ETH \
  --interval 4h \
  --initial-capital 50000.0 \
  --format terminal
```

## Benefits

### For Developers
1. **Easy to Add New Strategies**
   - Implement `Strategy` and `MetadataStrategy` traits
   - Add `Default` implementation
   - Register in registry (one line)
   - No CLI code changes needed!

2. **Type Safety**
   - Compiler ensures all registered strategies have required traits
   - Metadata is extracted from strategy, not duplicated

3. **Consistent Error Handling**
   - Helpful error messages show all available strategies
   - Automatic name canonicalization

### For Users
1. **Discoverable Strategies**
   - `--list-strategies` shows all available options
   - Category filtering for easy browsing
   - Descriptions and metadata displayed

2. **Flexible Naming**
   - Multiple input formats supported
   - Case-insensitive matching
   - Underscores, spaces, dashes all work

3. **Better Error Messages**
   ```bash
   $ validate_strategy validate --strategy unknown_strategy
   Error: Unknown strategy: 'unknown_strategy'. Available strategies: GoldenCross, MacdTrend, 
   Rsi, BollingerBands, Sentiment Momentum, Regime Sentiment, Divergence. 
   Use --list-strategies to see all options.
   ```

## Testing

### Manual Testing Commands

```bash
# Test list all strategies
cargo run --bin validate_strategy -- list-strategies

# Test category filter
cargo run --bin validate_strategy -- list-strategies --category trend_following
cargo run --bin validate_strategy -- list-strategies --category mean_reversion
cargo run --bin validate_strategy -- list-strategies --category sentiment

# Test name variations
cargo run --bin validate_strategy -- validate --strategy golden_cross --symbol BTC --interval 1d --format terminal
cargo run --bin validate_strategy -- validate --strategy "Golden Cross" --symbol BTC --interval 1d --format terminal
cargo run --bin validate_strategy -- validate --strategy RSI --symbol ETH --interval 4h --format terminal
cargo run --bin validate_strategy -- validate --strategy MacdTrend --symbol SOL --interval 1h --format terminal

# Test invalid strategy
cargo run --bin validate_strategy -- validate --strategy nonexistent --symbol BTC --interval 1d
```

### Expected Results

✅ All registered strategies should be listed with metadata
✅ Category filtering should work correctly
✅ Multiple name formats should resolve to the same strategy
✅ Validation should run successfully for all strategies
✅ Invalid strategy names should show helpful error with available options
✅ Symbol and capital parameters should be used correctly

## Future Enhancements

### Phase 1: Expand Registry
- Add more strategies to the registry as they get `Default` implementations
- Support parameterized strategies (with user-specified params)
- Add strategy aliases (e.g., "GC" → "Golden Cross")

### Phase 2: Enhanced Discovery
- Show strategy parameters and defaults in `--list-strategies`
- Display strategy performance metrics from historical backtests
- Add search/filter by metadata (regime, indicators, etc.)

### Phase 3: Dynamic Loading
- Support loading strategies from external crates
- Plugin architecture for custom strategies
- Hot-reloading of strategies without recompilation

### Phase 4: Advanced Features
- Strategy composition (combine multiple strategies)
- Strategy templates with parameter presets
- Strategy comparison and benchmarking tools

## Backward Compatibility

This enhancement is **fully backward compatible**:
- Existing strategy validation commands continue to work
- No changes to core strategy traits
- No database schema changes
- No API changes

## Migration Guide

For developers adding new strategies:

### Before
```rust
// Add to create_strategy() function in validate_strategy.rs
match normalized_name.as_str() {
    "my_strategy" => Ok(Box::new(StrategyAdapter::new(
        MyStrategy::default(), "BTCUSDT", 10000.0
    ))),
    // ...
}
```

### After
```rust
// 1. Implement Strategy and MetadataStrategy traits
impl Strategy for MyStrategy { /* ... */ }
impl MetadataStrategy for MyStrategy {
    fn metadata(&self) -> StrategyMetadata { /* ... */ }
}

// 2. Add Default implementation
impl Default for MyStrategy {
    fn default() -> Self { /* ... */ }
}

// 3. Register in STRATEGY_FACTORY_REGISTRY
register_strategy!(MyStrategy);

// That's it! No other changes needed.
```

## Conclusion

The Strategy Registry enhancement significantly improves the maintainability and usability of the validation CLI. By replacing hardcoded mappings with a dynamic factory-based registry, we've:

- Made it easier to add new strategies (one line vs multiple)
- Improved user experience with discoverable strategies
- Enhanced error messages with helpful suggestions
- Maintained backward compatibility
- Set the foundation for future enhancements

This enhancement follows best practices for extensibility and maintainability, ensuring AlphaField can scale to support dozens of strategies without increasing complexity.