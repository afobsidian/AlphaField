# Strategy Selection & Dashboard Integration Guide

## Overview

AlphaField's dashboard features a **modular, scalable strategy selection system** that automatically discovers and organizes trading strategies. This guide explains how the system works and provides a checklist for ensuring new strategies integrate seamlessly into the dashboard.

## System Architecture

### Strategy Registration Flow

```
1. Strategy Implementation (crates/strategy/src/strategies/)
   ↓
2. Module Export (mod.rs exports)
   ↓
3. Dashboard Registration (crates/dashboard/src/strategies_api.rs)
   ↓
4. Runtime Discovery (initialize_registry)
   ↓
5. API Exposure (GET /api/strategies)
   ↓
6. Frontend Rendering (category-based accordions with search)
```

### Key Components

#### Backend: Strategy Registry
- **Location**: `crates/strategy/src/framework.rs`
- **Purpose**: Central registry of all available strategies
- **Features**:
  - Dynamic strategy registration at runtime
  - Metadata retrieval (name, category, description, risk profile)
  - Category filtering and regime filtering
  - Canonical name normalization

#### API Layer: Strategies Endpoint
- **Location**: `crates/dashboard/src/strategies_api.rs`
- **Endpoint**: `GET /api/strategies`
- **Features**:
  - Returns all strategies as JSON
  - Supports category filtering: `?category=TrendFollowing`
  - Supports regime filtering: `?regime=Bull`
  - Automatic strategy registration on server startup

#### Frontend: Category-Based UI
- **Location**: `crates/dashboard/static/app.js` + `index.html`
- **Features**:
  - Real-time search filtering
  - Category accordions (expandable sections)
  - Strategy count badges
  - Selection state management
  - Automatic rendering from API data

## Integration Requirements Checklist

### ✅ For Every New Strategy

#### 1. Implement Required Traits

```rust
use alphafield_core::Strategy;
use alphafield_strategy::framework::MetadataStrategy;

pub struct MyStrategy {
    // strategy fields
}

impl Strategy for MyStrategy {
    fn name(&self) -> &str {
        "MyStrategy"  // ← This becomes the strategy ID
    }
    
    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        // strategy logic
    }
    
    fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
        None
    }
    
    fn on_quote(&mut self, _quote: &Quote) -> Option<Signal> {
        None
    }
}

impl MetadataStrategy for MyStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "MyStrategy".to_string(),
            category: StrategyCategory::TrendFollowing,  // ← Determines category
            sub_type: Some("My Strategy Type".to_string()),
            description: "Clear description of strategy logic".to_string(),
            hypothesis_path: "doc/hypotheses/my_strategy.md".to_string(),
            required_indicators: vec!["SMA".to_string()],
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Trending],
            risk_profile: RiskProfile {
                max_drawdown_expected: "Medium".to_string(),
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }
    
    fn category(&self) -> StrategyCategory {
        StrategyCategory::TrendFollowing  // ← Must match metadata
    }
}
```

#### 2. Export From Module

**File**: `crates/strategy/src/strategies/my_category/mod.rs`

```rust
pub mod my_strategy;

// Re-export for use
pub use my_strategy::{MyStrategy, MyStrategyConfig};
```

**File**: `crates/strategy/src/strategies/mod.rs`

```rust
pub mod my_category;

// Re-export category
pub use my_category::*;
```

**File**: `crates/strategy/src/lib.rs`

```rust
pub mod strategies;

// Re-export all strategies
pub use strategies::*;
```

#### 3. Register in Dashboard

**File**: `crates/dashboard/src/strategies_api.rs`

```rust
pub fn initialize_registry() -> Arc<StrategyRegistry> {
    let registry = Arc::new(StrategyRegistry::new());
    
    // ... existing strategies ...
    
    // Register MyStrategy
    let my_strategy = Arc::new(
        alphafield_strategy::strategies::MyStrategy::new(/* params */)
    ) as Arc<dyn StrategyWithMetadata>;
    
    if let Err(e) = registry.register(my_strategy) {
        eprintln!("Failed to register MyStrategy strategy: {}", e);
    }
    
    registry
}
```

#### 4. Add UI Override (Optional but Recommended)

**File**: `crates/dashboard/static/app.js`

```javascript
const STRATEGY_UI_OVERRIDES = {
  // ... existing overrides ...
  
  MyStrategy: {
    displayName: "My Awesome Strategy",
    description: "Clear, user-friendly description of what this does",
  },
};
```

#### 5. Add Parameter Schema (If Strategy Has Custom Parameters)

**File**: `crates/dashboard/static/app.js`

```javascript
function buildDefaultParamSchema(strategyKey) {
  const baseSchema = {
    // common parameters for all strategies
    symbol: { name: 'symbol', label: 'Symbol', default: 'BTC', type: 'select' },
    days: { name: 'days', label: 'Backtest Days', default: 365, min: 30, max: 730 },
    interval: { name: 'interval', label: 'Interval', default: '1d', type: 'select' },
  };
  
  // Add strategy-specific parameters
  if (strategyKey === 'MyStrategy') {
    return {
      ...baseSchema,
      my_param: {
        name: 'my_param',
        label: 'My Custom Parameter',
        default: 50,
        min: 10,
        max: 200,
        step: 5,
        description: 'Description of what this parameter does'
      }
    };
  }
  
  return baseSchema;
}
```

### 📋 Complete Integration Checklist

Before considering a strategy "dashboard-ready", verify:

- [ ] **Strategy implements `Strategy` trait**
  - [ ] `name()` returns unique identifier
  - [ ] `on_bar()` generates signals
  - [ ] `on_tick()` handles tick updates
  - [ ] `on_quote()` handles quote updates

- [ ] **Strategy implements `MetadataStrategy` trait**
  - [ ] `metadata()` returns complete `StrategyMetadata`
  - [ ] `category()` returns valid `StrategyCategory`
  - [ ] `name` in metadata matches `name()` from Strategy trait
  - [ ] `description` is clear and user-friendly
  - [ ] `hypothesis_path` points to existing document

- [ ] **Module exports are complete**
  - [ ] Exported from category `mod.rs`
  - [ ] Exported from `strategies/mod.rs`
  - [ ] Exported from `lib.rs`
  - [ ] Can import with `use alphafield_strategy::strategies::MyStrategy`

- [ ] **Dashboard registration is complete**
  - [ ] Registered in `initialize_registry()` in `strategies_api.rs`
  - [ ] Uses correct constructor parameters
  - [ ] Wrapped in `Arc<dyn StrategyWithMetadata>`
  - [ ] Error handling with `eprintln!`

- [ ] **UI integration is complete**
  - [ ] Added to `STRATEGY_UI_OVERRIDES` (optional but recommended)
  - [ ] Display name is user-friendly
  - [ ] Description is concise and informative
  - [ ] Parameter schema added if strategy has custom params

- [ ] **Documentation is complete**
  - [ ] Hypothesis document exists at specified path
  - [ ] Hypothesis includes expected performance metrics
  - [ ] Risk factors documented
  - [ ] Entry/exit rules clearly explained
  - [ ] Parameter ranges specified

- [ ] **Testing is complete**
  - [ ] Unit tests cover strategy creation
  - [ ] Unit tests cover signal generation
  - [ ] Unit tests cover edge cases
  - [ ] All tests pass

- [ ] **Build verification**
  - [ ] `cargo build -p alphafield-dashboard` succeeds
  - [ ] No warnings or errors
  - [ ] Strategy appears in `/api/strategies` endpoint
  - [ ] Strategy appears in dashboard UI

## Strategy Categories

### Available Categories

| Category | Enum Value | Icon | Description | Example Strategies |
|-----------|-------------|-------|-------------|------------------|
| Baseline | `StrategyCategory::Baseline` | 📊 | Buy & Hold, Market Average | HODL Baseline, Market Average |
| Trend Following | `StrategyCategory::TrendFollowing` | 📈 | Follows market trends | Golden Cross, MA Crossover, Breakout |
| Mean Reversion | `StrategyCategory::MeanReversion` | 🔄 | Fades extreme moves | Bollinger Bands, RSI Reversion |
| Momentum | `StrategyCategory::Momentum` | ⚡ | Captures price momentum | RSI Momentum, MACD Momentum |
| Volatility Based | `StrategyCategory::VolatilityBased` | 🌊 | Uses volatility levels | ATR Breakout (planned) |
| Sentiment Based | `StrategyCategory::SentimentBased` | 💭 | Uses sentiment data | Fear & Greed (planned) |
| Multi Indicator | `StrategyCategory::MultiIndicator` | 🎯 | Combines multiple signals | Ensemble (planned) |

### Choosing the Right Category

When adding a new strategy, select the category based on the **primary logic**:

- **Trend Following**: Strategy makes money when trends continue, loses when trend reverses
- **Mean Reversion**: Strategy bets on price returning to mean after extreme moves
- **Momentum**: Strategy bets that recent momentum will continue
- **Volatility Based**: Strategy relies on volatility measurements or breakout conditions
- **Sentiment Based**: Strategy uses external sentiment or social data
- **Multi Indicator**: Strategy requires 3+ indicators aligning to generate signal

## Common Integration Issues

### Issue: Strategy Doesn't Appear in Dashboard

**Symptoms**: Strategy exists in code but not in `/api/strategies`

**Causes**:
1. Not registered in `initialize_registry()`
2. Wrong constructor parameters
3. Compilation error in registration code
4. Category enum variant doesn't match

**Solution**:
```rust
// Check registration matches this pattern
let my_strategy = Arc::new(
    alphafield_strategy::strategies::MyStrategy::new(/* correct params */)
) as Arc<dyn StrategyWithMetadata>;

if let Err(e) = registry.register(my_strategy) {
    eprintln!("Failed to register MyStrategy: {}", e);  // Check logs
}
```

### Issue: Strategy Name Doesn't Match

**Symptoms**: Strategy appears but with wrong name or no UI overrides

**Causes**:
1. `Strategy::name()` returns different string than `metadata().name`
2. UI override uses wrong key (e.g., `MyStrategy` vs `my_strategy`)

**Solution**:
```rust
impl Strategy for MyStrategy {
    fn name(&self) -> &str {
        "MyStrategy"  // ← This exact string
    }
}

impl MetadataStrategy for MyStrategy {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "MyStrategy".to_string(),  // ← Must match exactly
            // ...
        }
    }
}

// UI override key must match
const STRATEGY_UI_OVERRIDES = {
  MyStrategy: {  // ← Case-sensitive, must match Strategy::name()
    displayName: "My Strategy",
    description: "..."
  }
};
```

### Issue: Strategy Has No Parameters

**Symptoms**: Strategy appears in dashboard but can't be configured

**Causes**: Strategy has custom parameters but no UI schema defined

**Solution**:
```javascript
function buildDefaultParamSchema(strategyKey) {
  // Always return parameters
  if (strategyKey === 'MyStrategy') {
    return {
      symbol: { /* ... */ },
      days: { /* ... */ },
      my_custom_param: {
        name: 'my_custom_param',
        label: 'Custom Parameter',
        default: 50,
        min: 10,
        max: 100
      }
    };
  }
  return {};  // ← Wrong! Should return base params
}
```

### Issue: Category Not Grouping

**Symptoms**: Strategy shows in wrong category or no category

**Causes**: `StrategyCategory::` variant doesn't match one of the defined categories

**Solution**:
```rust
// Must use predefined enum variants
impl MetadataStrategy for MyStrategy {
    fn category(&self) -> StrategyCategory {
        StrategyCategory::TrendFollowing  // ← Valid variant
    }
}

// Invalid:
StrategyCategory::CustomCategory  // ← Won't compile
```

## Best Practices

### 1. Naming Conventions

- **Backend Name**: PascalCase, no spaces (e.g., `GoldenCross`, `MyStrategy`)
- **Display Name**: Human-friendly, can have spaces (e.g., "Golden Cross")
- **Module Name**: snake_case (e.g., `my_strategy.rs`)

### 2. Metadata Quality

```rust
StrategyMetadata {
    name: "MyStrategy".to_string(),
    
    // Category - choose carefully
    category: StrategyCategory::TrendFollowing,
    
    // Sub-type - specific variant (optional)
    sub_type: Some("MA Crossover".to_string()),
    
    // Description - what the strategy does, not how
    description: "Follows price trends using moving average crossovers".to_string(),
    
    // Hypothesis - must exist
    hypothesis_path: "doc/hypotheses/trend/my_strategy.md".to_string(),
    
    // Required indicators - list what's needed
    required_indicators: vec![
        "SMA".to_string(),
        "EMA".to_string()
    ],
    
    // Expected regimes - be honest
    expected_regimes: vec![
        MarketRegime::Bull,      // Works well
        MarketRegime::Trending    // Works well
    ],
    
    // Risk profile - accurate expectations
    risk_profile: RiskProfile {
        max_drawdown_expected: "Medium".to_string(),
        volatility_level: VolatilityLevel::Medium,
        correlation_sensitivity: CorrelationSensitivity::Medium,
        leverage_requirement: 1.0,  // Spot trading
    },
}
```

### 3. Constructor Design

Keep constructors simple with sensible defaults:

```rust
impl MyStrategy {
    // Simple constructor with required params only
    pub fn new(period: usize, threshold: f64) -> Self {
        Self {
            period,
            threshold,
            take_profit: 5.0,      // Reasonable default
            stop_loss: 3.0,         // Reasonable default
        }
    }
    
    // Optional: Config-based constructor for advanced users
    pub fn from_config(config: MyStrategyConfig) -> Self {
        config.validate().expect("Invalid MyStrategyConfig");
        // ... create from config
    }
}
```

### 4. Error Handling

Always validate in registration:

```rust
let my_strategy = Arc::new(
    alphafield_strategy::strategies::MyStrategy::new(
        /* params */
    )
) as Arc<dyn StrategyWithMetadata>;

if let Err(e) = registry.register(my_strategy) {
    eprintln!("Failed to register MyStrategy: {}", e);
    // This ensures you're notified at startup if something is wrong
}
```

### 5. Documentation First

Before coding, write the hypothesis:

1. Create `doc/hypotheses/[category]/my_strategy.md`
2. Document the hypothesis clearly
3. Define entry/exit rules
4. Specify expected market regimes
5. Identify risk factors
6. Set performance expectations

Then implement the strategy to match the hypothesis.

## Testing Integration

### Verify API Response

```bash
# Start dashboard server
cargo run --bin dashboard_server --release

# Test strategies endpoint
curl http://localhost:8080/api/strategies | jq

# Look for your strategy in the output
curl http://localhost:8080/api/strategies | jq '.[] | select(.name=="MyStrategy")'

# Test category filtering
curl "http://localhost:8080/api/strategies?category=TrendFollowing" | jq
```

### Verify Dashboard Display

1. Open http://localhost:8080 in browser
2. Go to Strategy Configuration section
3. Search for your strategy name
4. Verify it appears in correct category
5. Click strategy and verify selection works
6. Check that parameters appear if strategy has custom params
7. Run a test backtest to ensure end-to-end works

## Documentation Requirements

### Required Documentation Locations

When adding a strategy, ensure documentation exists in:

1. **Hypothesis Document**:
   - Path: `doc/hypotheses/[category]/[strategy_name].md`
   - Content: Hypothesis, logic, expected regimes, risks

2. **API Documentation** (if strategy adds new endpoints):
   - Path: `doc/api.md`
   - Content: Endpoint specification, request/response formats

3. **UI Documentation** (if strategy has special UI requirements):
   - Path: `doc/ui/strategy_selection.md` (this file)
   - Content: Special UI patterns, custom controls

4. **Roadmap Update**:
   - Path: `doc/roadmap.md`
   - Content: Mark strategy category as complete

## Automated Integration Testing

The system includes integration tests to verify strategy registration:

**Test**: `crates/dashboard/tests/strategies_api_integration_test.rs`

```rust
#[tokio::test]
async fn test_all_strategies_registered() {
    let registry = initialize_registry();
    let strategies = registry.list_all();
    
    // Verify expected strategies are present
    assert!(strategies.contains(&"GoldenCross".to_string()));
    assert!(strategies.contains(&"BollingerBandsStrategy".to_string()));
    assert!(strategies.contains(&"MyStrategy".to_string()));
    
    // Verify each strategy has valid metadata
    for strategy_name in strategies {
        let metadata = registry.get_metadata(&strategy_name);
        assert!(metadata.is_some(), "{} has no metadata", strategy_name);
        
        let meta = metadata.unwrap();
        assert!(!meta.name.is_empty());
        assert!(!meta.description.is_empty());
    }
}
```

Run tests with:
```bash
cargo test -p alphafield-dashboard strategies_api_integration
```

## Summary

The AlphaField strategy selection system is designed for **automatic discovery and integration**. By following this checklist and best practices, new strategies will:

- ✅ Automatically appear in dashboard without manual UI changes
- ✅ Be properly categorized and searchable
- ✅ Have complete metadata for user understanding
- ✅ Support parameter configuration if needed
- ✅ Be documented with hypothesis-driven approach
- ✅ Pass integration tests

**Key Takeaway**: The system is modular and extensible. Focus on implementing the strategy correctly (traits, metadata, registration), and the dashboard will handle the rest automatically.

---

*Last Updated: January 2026*  
*Maintained By: AlphaField Development Team*