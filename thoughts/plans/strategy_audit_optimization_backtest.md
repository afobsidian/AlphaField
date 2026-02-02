# Strategy Audit & Optimization Integration Implementation Plan

## Overview

Comprehensive audit and remediation of all 44 trading strategies to ensure proper integration with the optimization workflow, correct trade generation, and TradingMode compliance. This plan follows a phased approach with category-by-category fixes and a shared test harness.

**Scope:** 44 strategies across 6 categories (trend_following: 8, momentum: 7, mean_reversion: 7, volatility: 8, multi_indicator: 8, sentiment: 3)  
**Exclusions:** HODL_Baseline and Market_Average_Baseline will be removed from optimization workflow  
**Timeline:** 5 phases with incremental validation

## Current State Analysis

**Working Components:**
- Strategy framework and registry (`crates/strategy/src/framework.rs`)
- Parameter bounds for 43 strategies (`crates/backtest/src/optimizer.rs:306-679`)
- Optimization workflow engine with grid search, walk-forward, Monte Carlo
- Backtest engine with TradingMode support
- StrategyAdapter enforces TradingMode constraints

**Identified Issues:**
1. Parameter name mismatches between `get_strategy_bounds()` and `StrategyFactory`
2. Some strategies panic in constructors instead of returning None gracefully
3. TradingMode may not be consistently passed through all optimization code paths
4. Strategies generating few trades (<5) get heavily penalized in optimization scoring
5. Missing comprehensive test coverage (need 200+ tests)
6. Baseline strategies (HODL, Market_Average) included but shouldn't be optimizable

**Root Causes:**
- Parameter names must match exactly between bounds definition and factory instantiation
- Two-layer validation: StrategyFactory validates, but some strategies also call `config.validate().expect()`
- "UNKNOWN" symbol in signals is intentional design, not a bug

## Desired End State

### All Strategies Will:
1. Generate valid trades when market conditions meet entry criteria
2. Respect `TradingMode::Spot` constraints (no short selling, enforced by StrategyAdapter)
3. Accept parameters from optimizer and validate gracefully (return None, never panic)
4. Work with walk-forward validation's rolling window approach
5. Function correctly across all asset categories (selected via dashboard)
6. Have comprehensive test coverage via shared test harness

### Optimization Workflow Will:
1. Successfully optimize all 44 strategies without errors
2. Generate parameter sweep visualizations for all strategies
3. Perform walk-forward validation across all strategies
4. Complete within 5 minutes for standard parameter grids (50-100 combinations)

### Verification:
- `cargo test` passes with >200 tests
- `make lint` passes with zero warnings
- Dashboard optimization works for random strategy sample
- All strategies generate trades in backtest

## What We're NOT Doing

1. **NOT changing the "UNKNOWN" symbol pattern** - This is intentional design, StrategyAdapter handles mapping
2. **NOT modifying the core optimization algorithm** - Only fixing integration issues
3. **NOT adding new strategies** - Only fixing existing 44 strategies
4. **NOT changing TradingMode logic** - StrategyAdapter already enforces this correctly
5. **NOT creating individual test files per strategy** - Using shared test harness instead

## Implementation Approach

Follow the priority order from research:
1. **Priority 1:** Fix Parameter Consistency (Phase 1)
2. **Priority 2:** Fix Trading Mode Propagation (Phase 2)
3. **Priority 3:** Fix Strategy Validation (Phase 2)
4. **Priority 4:** Add Tests (Phase 3)
5. **Priority 5:** Asset Category Testing (Phase 4)

Category-by-category approach within Phase 1 for incremental validation.

---

## Phase 1: Foundation & Parameter Audit

### Overview
Audit all parameter names between `get_strategy_bounds()` and `StrategyFactory`, fix mismatches category-by-category, remove baseline strategies, and create shared test harness infrastructure.

### Changes Required:

#### 1.1 Audit Parameter Consistency
**Files:** 
- `crates/backtest/src/optimizer.rs:306-679` (get_strategy_bounds)
- `crates/dashboard/src/services/strategy_service.rs` (StrategyFactory)

**Process for each of 6 categories:**
1. Read current parameter bounds in `get_strategy_bounds()`
2. Read current StrategyFactory implementation
3. Compare parameter names (must match exactly)
4. Document mismatches in audit log
5. Fix mismatches (usually in StrategyFactory to match bounds)

**Categories to audit:**
- [ ] Trend Following (8 strategies): GoldenCross, Breakout, MACrossover, AdaptiveMA, TripleMA, MacdTrend, ParabolicSAR
- [ ] Momentum (7 strategies): RsiMomentumStrategy, MACDStrategy, RocStrategy, AdxTrendStrategy, MomentumFactorStrategy, VolumeMomentumStrategy, MultiTfMomentumStrategy
- [ ] Mean Reversion (7 strategies): BollingerBands, RSIReversion, StochReversion, ZScoreReversion, PriceChannel, KeltnerReversion, StatArb
- [ ] Volatility (8 strategies): ATRBreakout, ATRTrailingStop, VolatilitySqueeze, VolRegimeStrategy, VolSizingStrategy, GarchStrategy, VIXStyleStrategy
- [ ] Multi-Indicator (8 strategies): TrendMeanRev, MACDRSICombo, AdaptiveCombo, ConfidenceWeighted, EnsembleWeighted, MLEnhanced, RegimeSwitching
- [ ] Sentiment (3 strategies): Divergence, RegimeSentiment, SentimentMomentum

#### 1.2 Remove Baseline Strategies
**Files:**
- `crates/backtest/src/optimizer.rs:306-679`
- `crates/dashboard/src/services/strategy_service.rs`
- `crates/strategy/src/framework.rs` (if registered)

**Changes:**
- [ ] Remove HODL_Baseline from `get_strategy_bounds()`
- [ ] Remove Market_Average_Baseline from `get_strategy_bounds()`
- [ ] Remove HODL_Baseline from StrategyFactory
- [ ] Remove Market_Average_Baseline from StrategyFactory

#### 1.3 Fix Parameter Mismatches
**Common mismatches to check:**
- Period naming: `fast_period` vs `fast` vs `fast_ma`
- Stop/Take profit: `stop_loss` vs `sl` vs `stop_loss_pct`
- Threshold naming: `threshold` vs `entry_threshold` vs `buy_threshold`
- Window naming: `window` vs `period` vs `length`

**Example fix:**
```rust
// If get_strategy_bounds uses:
ParamBounds::new("fast_period", 5.0, 30.0, 5.0)

// StrategyFactory must use:
let fast = params.get("fast_period").copied().unwrap_or(10.0) as usize;
```

#### 1.4 Create Parameter Audit Log
Create `thoughts/audit/parameter_audit_log.md` to track:
- Strategy name
- Parameters in bounds
- Parameters in factory
- Mismatches found
- Fixes applied

### Success Criteria:

#### Automated Verification:
- [ ] All 44 strategies have matching parameter names between bounds and factory
- [ ] `cargo check` passes without errors
- [ ] No compile-time warnings about unused parameters

#### Manual Verification:
- [ ] Spot-check 5 random strategies to verify parameter consistency
- [ ] Verify baseline strategies no longer appear in optimization dropdown

---

## Phase 2: Trading Mode & Validation Fixes

### Overview
Fix panic handling in strategy constructors and verify TradingMode propagation through all optimization paths.

### Changes Required:

#### 2.1 Fix Panic Handling in Strategy Constructors
**Pattern to find:**
```rust
// BAD - Will panic during optimization
pub fn from_config(config: Config) -> Self {
    config.validate().expect("Invalid config");
    Self { ... }
}
```

**Pattern to use:**
```rust
// GOOD - Returns None gracefully
pub fn from_config(config: Config) -> Option<Self> {
    config.validate().ok()?;
    Some(Self { ... })
}
```

**Files to check (all strategy files):**
- [ ] All 8 trend_following strategies
- [ ] All 7 momentum strategies
- [ ] All 7 mean_reversion strategies
- [ ] All 8 volatility strategies
- [ ] All 8 multi_indicator strategies
- [ ] All 3 sentiment strategies

#### 2.2 Update StrategyFactory for Option Return
**File:** `crates/dashboard/src/services/strategy_service.rs`

**Changes:**
Update StrategyFactory to handle Option returns from `from_config()`:
```rust
// Current:
let config = alphafield_strategy::config::GoldenCrossConfig::new(fast, slow, tp, sl);
Some(Box::new(GoldenCrossStrategy::from_config(config)))

// Updated (if from_config returns Option):
let config = alphafield_strategy::config::GoldenCrossConfig::new(fast, slow, tp, sl)?;
GoldenCrossStrategy::from_config(config)
    .map(|s| Box::new(s) as Box<dyn Strategy>)
```

#### 2.3 Verify TradingMode Propagation
**Files to check:**
- `crates/backtest/src/optimization_workflow.rs` - WorkflowConfig has trading_mode
- `crates/backtest/src/engine.rs` - BacktestEngine has trading_mode
- `crates/backtest/src/adapter.rs` - StrategyAdapter has trading_mode
- `crates/dashboard/src/backtest_api.rs` - API endpoints

**Verification checklist:**
- [ ] WorkflowConfig includes `trading_mode: TradingMode`
- [ ] OptimizationWorkflow passes trading_mode to all components
- [ ] BacktestEngine receives trading_mode from workflow
- [ ] StrategyAdapter receives trading_mode from engine
- [ ] All API endpoints accept and forward trading_mode parameter

#### 2.4 Add TradingMode to StrategyAdapter Creation
**File:** `crates/dashboard/src/services/strategy_service.rs`

**Update `create_backtest` method:**
```rust
pub fn create_backtest(
    name: &str,
    params: &HashMap<String, f64>,
    symbol: &str,
    capital: f64,
    trading_mode: TradingMode,  // Add this parameter
) -> Option<Box<dyn BacktestStrategy>> {
    Self::create(name, params).map(|s| {
        Box::new(
            StrategyAdapter::new(s, symbol, capital)
                .with_trading_mode(trading_mode)  // Add this call
        ) as Box<dyn BacktestStrategy>
    })
}
```

### Success Criteria:

#### Automated Verification:
- [ ] No `expect()` calls remain in strategy constructors (use `grep -r "\.expect\(" crates/strategy/src/strategies/`)
- [ ] `cargo test` compiles successfully
- [ ] All strategy factory methods return `Option<Box<dyn Strategy>>`

#### Manual Verification:
- [ ] Run optimization with invalid parameters - should return None gracefully, not panic
- [ ] Verify TradingMode::Spot prevents short selling in backtest

---

## Phase 3: Testing Implementation

### Overview
Create modular shared test harness and implement comprehensive tests for all strategies.

### Changes Required:

#### 3.1 Create Shared Test Harness
**File:** `crates/strategy/src/testing/harness.rs` (new file)

**Components:**
```rust
pub struct StrategyTestHarness {
    // Mock data generator
    // Signal verification helpers
    // Optimization workflow tester
    // Backtest runner
}

impl StrategyTestHarness {
    pub fn test_signal_generation<S: Strategy>(
        strategy: &mut S,
        bars: &[Bar],
        expected_signals: Vec<ExpectedSignal>
    ) -> Result<(), TestError>
    
    pub fn test_optimization_integration(
        strategy_name: &str,
        bounds: &[ParamBounds]
    ) -> Result<OptimizationResult, TestError>
    
    pub fn test_backtest_workflow(
        strategy_name: &str,
        symbol: &str,
        bars: &[Bar]
    ) -> Result<BacktestResult, TestError>
}
```

#### 3.2 Create Test Data Generators
**File:** `crates/strategy/src/testing/data_generators.rs`

**Functions:**
```rust
pub fn generate_trending_market(periods: usize, trend: f64) -> Vec<Bar>
pub fn generate_ranging_market(periods: usize, volatility: f64) -> Vec<Bar>
pub fn generate_choppy_market(periods: usize) -> Vec<Bar>
pub fn generate_volatile_market(periods: usize, volatility: f64) -> Vec<Bar>
```

#### 3.3 Implement Unit Tests for Signal Generation
**File:** `crates/strategy/src/testing/mod.rs` or `crates/strategy/tests/integration_tests.rs`

**Test structure using harness:**
```rust
#[test]
fn test_golden_cross_signal_generation() {
    let harness = StrategyTestHarness::new();
    let bars = harness.generate_trending_market(100, 0.02);
    let strategy = GoldenCrossStrategy::default();
    
    let result = harness.test_signal_generation(
        &mut strategy,
        &bars,
        vec![ExpectedSignal::new(SignalType::Buy).at_bar(45)]
    );
    
    assert!(result.is_ok());
}
```

**Unit tests for all 44 strategies:**
- [ ] Trend Following (8 tests)
- [ ] Momentum (7 tests)
- [ ] Mean Reversion (7 tests)
- [ ] Volatility (8 tests)
- [ ] Multi-Indicator (8 tests)
- [ ] Sentiment (3 tests)

#### 3.4 Implement Integration Tests
**File:** `crates/backtest/tests/optimization_integration_tests.rs`

**Tests:**
- [ ] Test each strategy can be instantiated via StrategyFactory
- [ ] Test each strategy works with ParameterOptimizer
- [ ] Test each strategy generates at least 5 trades in test backtest
- [ ] Test walk-forward validation works for each strategy
- [ ] Test parameter bounds produce valid results

#### 3.5 Implement E2E Tests
**File:** `crates/dashboard/tests/api_integration_tests.rs`

**Tests:**
- [ ] Test `/api/backtest/workflow` endpoint for each strategy
- [ ] Test `/api/backtest/run` endpoint generates trades
- [ ] Test dashboard optimization tab works end-to-end

### Success Criteria:

#### Automated Verification:
- [ ] `cargo test` passes with >200 tests
- [ ] `cargo test --package alphafield-backtest` passes with all optimization tests
- [ ] `cargo test --package alphafield-strategy` passes with all strategy tests
- [ ] `cargo test --package alphafield-dashboard` passes with API tests
- [ ] Code coverage report shows >80% coverage for strategy module

#### Manual Verification:
- [ ] Run sample of 5 strategies through full test suite manually to verify
- [ ] Verify test failure messages are clear and actionable

---

## Phase 4: Batch Validation & Performance

### Overview
Run batch validation across all strategies, test performance requirements, and fix any edge cases.

### Changes Required:

#### 4.1 Create Batch Validation Script
**File:** `scripts/validate_all_strategies.rs` or `crates/backtest/src/bin/validate_all.rs`

**Features:**
- Run backtest for all 44 strategies
- Verify trade generation (minimum 5 trades)
- Test optimization workflow for each
- Generate validation report

#### 4.2 Performance Testing
**Metrics to verify:**
- [ ] Optimization completes in <5 minutes for 50-100 parameter combinations
- [ ] Walk-forward validation completes in <3 minutes per strategy
- [ ] Backtest completes in <30 seconds per strategy per symbol
- [ ] Memory usage remains stable during batch testing

#### 4.3 Edge Case Fixes
**Common issues to address:**
- Strategies failing with insufficient historical data
- Division by zero in indicator calculations
- Invalid parameter combinations not caught by validation
- State management issues in walk-forward validation

### Success Criteria:

#### Automated Verification:
- [ ] Batch validation script passes for all 44 strategies
- [ ] Performance benchmarks pass (add to CI)
- [ ] `cargo test` still passes after all fixes

#### Manual Verification:
- [ ] Run batch validation manually and review report
- [ ] Test dashboard with random sample of 5 strategies
- [ ] Verify optimization produces valid parameter heatmaps

---

## Phase 5: Cleanup & Documentation

### Overview
Final linting, formatting, documentation updates, and code review preparation.

### Changes Required:

#### 5.1 Code Quality
- [ ] Run `make fmt` - ensure no changes
- [ ] Run `make lint` - fix all warnings
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Fix any compiler warnings

#### 5.2 Documentation Updates
- [ ] Update `doc/optimization_workflow.md` with any API changes
- [ ] Update strategy documentation with fixed parameter names
- [ ] Document shared test harness usage
- [ ] Create troubleshooting guide for common optimization issues

#### 5.3 Create Implementation Summary
**File:** `thoughts/completed/strategy_audit_summary.md`

**Include:**
- Summary of changes made per phase
- List of strategies fixed
- Performance improvements achieved
- Test coverage metrics

### Success Criteria:

#### Automated Verification:
- [ ] `make lint` passes with zero warnings
- [ ] `make fmt` produces no changes
- [ ] `cargo build` succeeds with no warnings

#### Manual Verification:
- [ ] Code review completed
- [ ] Documentation reviewed and approved
- [ ] Final dashboard test successful

---

## Testing Strategy

### Unit Tests (Per Strategy):
- Signal generation with known market conditions
- Entry/exit logic validation
- Parameter validation
- Edge cases (division by zero, insufficient data)

### Integration Tests:
- StrategyFactory instantiation
- ParameterOptimizer compatibility
- BacktestEngine integration
- Walk-forward validation

### E2E Tests:
- Dashboard API endpoints
- Full optimization workflow
- Parameter sweep visualization
- Trade result display

### Shared Test Harness Features:
- Mock data generators (trending, ranging, choppy, volatile markets)
- Signal verification helpers
- Optimization workflow wrapper
- Backtest runner with assertions
- Performance benchmarking

---

## Risk Mitigation

### Risk: Breaking Changes
**Mitigation:**
- Comprehensive test coverage before changes
- Incremental validation per category
- Backward compatibility checks

### Risk: Performance Regression
**Mitigation:**
- Performance benchmarks in CI
- Monitor optimization time during testing
- Profile memory usage during batch validation

### Risk: Data Requirements
**Mitigation:**
- Ensure test data is available for all strategies
- Mock data generators for testing
- Document minimum data requirements per strategy

---

## Performance Considerations

1. **Optimization Grid Size:** Keep parameter step sizes reasonable to prevent explosion of combinations
2. **Walk-Forward Windows:** Balance between statistical significance and performance
3. **Memory Management:** Ensure strategies don't leak memory between walk-forward iterations
4. **Parallelization:** Consider parallel optimization for independent parameter combinations

---

## Migration Notes

### For Users:
- No breaking changes to existing strategy API
- Parameter names may change in UI (if mismatches fixed)
- Optimization workflow remains compatible

### For Developers:
- StrategyFactory now handles Option returns gracefully
- TradingMode propagated through all paths
- Shared test harness available for new strategies

---

## References

- Original ticket: `thoughts/tickets/debt_strategy_audit_optimization_backtest.md`
- Research document: `thoughts/research/2026-02-01_strategy_audit_optimization_backtest.md`
- Implementation plan: `thoughts/plans/strategy_audit_optimization_backtest.md`

## Key Files

- `crates/backtest/src/optimizer.rs` - Parameter bounds (line 306-679)
- `crates/dashboard/src/services/strategy_service.rs` - StrategyFactory
- `crates/backtest/src/adapter.rs` - StrategyAdapter with TradingMode
- `crates/backtest/src/optimization_workflow.rs` - Workflow orchestration
- `crates/strategy/src/framework.rs` - Strategy registry and metadata

## Progress Tracking

- [ ] Phase 1: Foundation & Parameter Audit
  - [ ] Category 1: Trend Following
  - [ ] Category 2: Momentum
  - [ ] Category 3: Mean Reversion
  - [ ] Category 4: Volatility
  - [ ] Category 5: Multi-Indicator
  - [ ] Category 6: Sentiment
  - [ ] Remove baseline strategies
  - [ ] Create audit log
- [ ] Phase 2: Trading Mode & Validation Fixes
  - [ ] Fix panic handling in all strategies
  - [ ] Update StrategyFactory for Option returns
  - [ ] Verify TradingMode propagation
- [ ] Phase 3: Testing Implementation
  - [ ] Create shared test harness
  - [ ] Implement unit tests
  - [ ] Implement integration tests
  - [ ] Implement E2E tests
- [ ] Phase 4: Batch Validation & Performance
  - [ ] Create batch validation script
  - [ ] Performance testing
  - [ ] Edge case fixes
- [ ] Phase 5: Cleanup & Documentation
  - [ ] Linting and formatting
  - [ ] Documentation updates
  - [ ] Implementation summary
