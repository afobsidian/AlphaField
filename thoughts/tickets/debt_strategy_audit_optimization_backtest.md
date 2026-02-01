---
type: debt
priority: high
created: 2026-02-01T15:00:00Z
created_by: Opus
status: implemented
tags: [strategies, optimization, backtest, audit, testing, parameters]
keywords: [Strategy, OptimizationWorkflow, BacktestEngine, ParameterOptimizer, get_strategy_bounds, walk_forward, TradingMode, asset_category, signal_generation, trade_execution]
patterns: [strategy_factory, param_bounds, on_bar, generate_signals, optimization_sweep, backtest_adapter]
research_document: thoughts/research/2026-02-01_strategy_audit_optimization_backtest.md
implementation_plan: thoughts/plans/strategy_audit_optimization_backtest.md
implementation_summary: thoughts/audit/implementation_status.md
completed_date: 2026-02-01
---

# DEBT-001: Complete Strategy Audit - Fix Optimization & Backtest Integration

## Description

A comprehensive audit and remediation of all 47+ trading strategies in the AlphaField codebase to ensure they properly integrate with the optimization workflow, generate trades correctly, and respect trading mode constraints. This addresses critical issues where strategies fail to generate trades or optimize properly when run through the dashboard.

## Context

The AlphaField trading engine includes 47+ strategies across 6 categories (trend following, momentum, mean reversion, volatility, multi-indicator, sentiment). The dashboard provides an "Optimize" tab that should enable:

- **Grid search parameter optimization** (sweep across parameter combinations)
- **Walk-forward validation** (rolling train/test windows)
- **3D sensitivity analysis** (parameter heatmaps)
- **Backtesting** with optimized parameters

**Current Issues Identified:**
1. **No trades generated**: Many strategies return empty trade lists during backtesting
2. **Trading mode violations**: Strategies not respecting Spot vs Margin mode settings
3. **Broken optimization**: Parameter sweeps fail or return invalid results
4. **Missing functional connections**: Strategies cannot be properly instantiated with parameters for optimization

**Impact:** Users cannot reliably optimize or backtest strategies through the dashboard, rendering the optimization-first workflow unusable.

## Requirements

### Functional Requirements

#### 1. Strategy Audit (All 47+ Strategies)
- [ ] Audit trend_following strategies (8 strategies: GoldenCross, Breakout, MACrossover, AdaptiveMA, TripleMA, MacdTrend, ParabolicSAR)
- [ ] Audit momentum strategies (8 strategies: RsiMomentumStrategy, MACDStrategy, RocStrategy, AdxTrendStrategy, MomentumFactorStrategy, VolumeMomentumStrategy, MultiTfMomentumStrategy)
- [ ] Audit mean_reversion strategies (7 strategies: BollingerBands, RSIReversion, StochReversion, ZScoreReversion, PriceChannel, KeltnerReversion, StatArb)
- [ ] Audit volatility strategies (8 strategies: ATRBreakout, ATRTrailingStop, VolatilitySqueeze, VolRegimeStrategy, VolSizingStrategy, GarchStrategy, VIXStyleStrategy)
- [ ] Audit multi_indicator strategies (8 strategies: TrendMeanRev, MACDRSICombo, AdaptiveCombo, ConfidenceWeighted, EnsembleWeighted, MLEnhanced, RegimeSwitching)
- [ ] Audit sentiment strategies (3 strategies: Divergence, RegimeSentiment, SentimentMomentum)
- [ ] Audit baseline strategies (2 strategies: HODL_Baseline, Market_Average_Baseline)

#### 2. Fix Parameter Bounds & Configuration
- [ ] Verify all strategies have complete parameter bounds in `get_strategy_bounds()` function
- [ ] Ensure parameter bounds match strategy implementation requirements
- [ ] Add missing parameter bounds for strategies returning empty bounds
- [ ] Validate parameter bounds don't create impossible combinations
- [ ] Fix parameter name mismatches between strategy implementation and bounds

#### 3. Fix Trade Generation
- [ ] Ensure all strategies generate valid `Signal` instances from `on_bar()` method
- [ ] Fix strategies that return `None` or empty signal lists incorrectly
- [ ] Verify signal generation works with minimum data requirements
- [ ] Ensure signals have proper side (Buy/Sell) and metadata
- [ ] Fix hardcoded "UNKNOWN" symbol issue in signals (use proper symbol mapping)

#### 4. Fix Trading Mode Compliance
- [ ] Ensure all strategies respect `TradingMode::Spot` vs `TradingMode::Margin`
- [ ] Fix strategies that attempt short selling in Spot mode
- [ ] Verify position sizing respects cash constraints in Spot mode
- [ ] Add trading mode validation in strategy initialization
- [ ] Fix strategies that generate incompatible signals for current trading mode

#### 5. Fix Optimization Integration
- [ ] Ensure all strategies can be instantiated via `strategy_factory` closure in optimizer
- [ ] Verify parameter injection works correctly for all strategies
- [ ] Fix strategies that panic or error when given optimization parameters
- [ ] Ensure strategies validate parameters and return `None` for invalid configs (not panic)
- [ ] Fix composite score calculation for strategies with low trade counts

#### 6. Fix Walk-Forward Validation
- [ ] Ensure all strategies work with rolling train/test window splits
- [ ] Fix strategies that fail when given limited historical data
- [ ] Verify strategies maintain state correctly across window boundaries
- [ ] Fix memory/state leaks between walk-forward iterations

#### 7. Asset Category Support
- [ ] Verify strategies work across all asset categories:
  - Market basket (top liquid assets)
  - Large Cap basket
  - Mid Cap basket
  - Small Cap basket
  - DeFi basket
- [ ] Fix strategies with hardcoded symbol dependencies
- [ ] Ensure strategies handle different volatility characteristics per category
- [ ] Test with representative symbols from each category

#### 8. Add Comprehensive Testing
- [ ] **Unit Tests**: Signal generation for each strategy with known inputs
- [ ] **Integration Tests**: Optimization workflow compatibility for each strategy
- [ ] **E2E Tests**: Full backtest through dashboard API for each strategy
- [ ] **Parameter Validation Tests**: Verify all parameter bounds work correctly
- [ ] **Trading Mode Tests**: Spot vs Margin compliance for each strategy
- [ ] **Asset Category Tests**: Cross-category compatibility verification
- [ ] **Regression Tests**: Prevent future breaking changes

### Non-Functional Requirements

- **Performance**: Optimization workflow should complete within 5 minutes for standard parameter grids
- **Reliability**: All strategies should have >95% success rate in optimization
- **Maintainability**: Clear error messages when strategies fail validation
- **Compatibility**: Maintain backward compatibility with existing strategy API

## Current State

### Working Components
- Strategy framework and registry (`crates/strategy/src/framework.rs`)
- Parameter bounds defined for most strategies (`crates/backtest/src/optimizer.rs:306-679`)
- Optimization workflow engine (`crates/backtest/src/optimization_workflow.rs`)
- Backtest engine (`crates/backtest/src/engine.rs`)
- Dashboard UI and API endpoints

### Broken Components
- **Trade generation**: Many strategies return no trades during backtesting
- **Trading mode**: Strategies not respecting Spot/Margin constraints
- **Optimization integration**: Strategy factory fails for multiple strategies
- **Parameter validation**: Inconsistent parameter bounds vs implementation
- **Signal generation**: Hardcoded symbols and missing signal metadata

### Known Issues
1. `Signal` struct uses hardcoded "UNKNOWN" symbol in many strategies
2. Some strategies don't implement proper `Clone` for optimizer usage
3. Parameter bounds mismatch between `get_strategy_bounds()` and strategy constructors
4. Missing validation in `strategy_factory` closure before strategy creation
5. Trading mode not passed to strategy initialization in backtest engine

## Desired State

### All Strategies Should:
1. Generate valid trades when market conditions meet entry criteria
2. Respect `TradingMode::Spot` constraints (no short selling, cash-limited)
3. Accept parameters from optimizer and validate them gracefully
4. Work with walk-forward validation's rolling window approach
5. Function correctly across all asset category baskets
6. Have comprehensive test coverage (unit + integration + e2e)

### Optimization Workflow Should:
1. Successfully optimize all 47+ strategies without errors
2. Generate parameter sweep visualizations for all strategies
3. Perform walk-forward validation across all strategies
4. Calculate robustness scores accurately for all strategies
5. Complete within reasonable time (5-10 minutes per strategy)

### Dashboard Should:
1. Allow users to select any strategy and optimize it successfully
2. Display trade results for all strategies in backtest view
3. Show parameter sensitivity heatmaps for all strategies
4. Support all asset categories for all strategies

## Research Context

### Keywords to Search

**Strategy Implementation:**
- `on_bar` - Signal generation method, core of every strategy
- `generate_signals` - Signal creation logic
- `Strategy` trait - Core trait all strategies implement
- `Signal` - Output type from strategies
- `Trade` - Result of signal execution
- `strategy_factory` - Closure that creates strategies from params

**Optimization Integration:**
- `ParameterOptimizer` - Grid search optimizer
- `get_strategy_bounds` - Function defining parameter ranges
- `ParamBounds` - Parameter range definition struct
- `optimize` - Main optimization method
- `calculate_composite_score` - Fitness function for optimization

**Backtest Integration:**
- `BacktestEngine` - Main backtesting engine
- `StrategyAdapter` - Bridge between Strategy trait and engine
- `run` - Execute backtest method
- `TradingMode` - Spot vs Margin enum
- `set_strategy` - Add strategy to engine

**Walk-Forward:**
- `WalkForwardAnalyzer` - Rolling window validation
- `train_test_split_ratio` - Data split for validation
- `stability_score` - Consistency metric across windows

**Asset Categories:**
- `AssetCategory` - Basket definitions (Market, Large Cap, etc.)
- `symbol_basket` - Group of symbols for testing
- `multi_symbol` - Cross-asset testing

### Patterns to Investigate

**Strategy Registration Pattern:**
- How strategies register with `StrategyRegistry`
- Metadata provision via `MetadataStrategy` trait
- Category classification and lookup

**Parameter Injection Pattern:**
- How optimizer passes parameters to `strategy_factory`
- Parameter validation in factory before strategy creation
- Default parameter handling for missing values

**Signal Generation Pattern:**
- Indicator calculation in `on_bar`
- Signal condition evaluation
- Signal metadata construction (side, confidence, etc.)

**Trading Mode Pattern:**
- Where `TradingMode` is set in backtest engine
- How strategies access current trading mode
- Short-selling prevention in Spot mode

**Test Pattern:**
- Unit test structure for signal generation
- Integration test with mock data
- E2E test through dashboard API endpoints

### Key Decisions Made

1. **Simultaneous Fix**: All strategies will be audited and fixed together, not incrementally
2. **Parameter Bounds**: Must update `get_strategy_bounds()` in optimizer.rs for any changes
3. **Trading Mode**: All strategies must explicitly handle Spot mode (no shorts)
4. **Testing**: Must add unit, integration, AND e2e tests for every strategy
5. **Asset Categories**: Must verify all strategies work across all 5 categories
6. **ML Strategies**: Include MLEnhanced in audit scope despite complexity
7. **Baseline Strategies**: Include HODL_Baseline and Market_Average_Baseline for completeness

### Critical Files to Examine

```
crates/strategy/src/strategies/
├── trend_following/     # 8 strategies
├── momentum/           # 8 strategies  
├── mean_reversion/     # 7 strategies
├── volatility/         # 8 strategies
├── multi_indicator/    # 8 strategies
└── sentiment/          # 3 strategies

crates/backtest/src/
├── optimizer.rs        # get_strategy_bounds() function
├── optimization_workflow.rs  # Workflow integration
├── engine.rs           # BacktestEngine
└── adapter.rs          # StrategyAdapter

crates/dashboard/src/
└── api.rs              # API endpoints for optimization
```

### Verification Checklist

**Per Strategy:**
- [ ] Has complete parameter bounds in `get_strategy_bounds()`
- [ ] Can be instantiated via `strategy_factory` closure
- [ ] Generates at least 5 trades in test backtest
- [ ] Works with walk-forward validation (8+ windows)
- [ ] Respects Spot trading mode (no short signals)
- [ ] Works with all 5 asset categories
- [ ] Has unit tests for signal generation
- [ ] Has integration tests for optimization
- [ ] Has e2e test through dashboard API

**Global:**
- [ ] All 47+ strategies pass audit
- [ ] `make test` passes with all new tests
- [ ] `make lint` passes without warnings
- [ ] Dashboard optimization works for random strategy sample
- [ ] Backtest generates trades for all strategies
- [ ] Walk-forward validation completes for all strategies

## Success Criteria

### Automated Verification
- [ ] `cargo test` passes with >200 new tests added
- [ ] `cargo test --package alphafield-backtest` passes with all optimization tests
- [ ] `cargo test --package alphafield-strategy` passes with all strategy tests
- [ ] `cargo test --package alphafield-dashboard` passes with API tests
- [ ] `make lint` passes with zero warnings
- [ ] `make fmt` produces no changes

### Manual Verification
- [ ] Run `/api/backtest/workflow` for sample of 5 strategies successfully
- [ ] Run `/api/backtest/run` for all 47 strategies and verify trade generation
- [ ] Test dashboard Optimize tab with each strategy category
- [ ] Verify walk-forward validation produces stability scores for all strategies
- [ ] Test with different asset baskets and confirm results

### Performance Verification
- [ ] Optimization completes in <5 minutes for standard grid (50-100 combinations)
- [ ] Walk-forward validation completes in <3 minutes per strategy
- [ ] Backtest completes in <30 seconds per strategy per symbol
- [ ] Memory usage remains stable during batch testing

## Related Information

### Dependencies
- `crates/core` - Core types (Bar, Trade, Signal, Strategy trait)
- `crates/strategy` - Strategy implementations
- `crates/backtest` - Optimization and backtest engine
- `crates/dashboard` - API endpoints and UI
- `crates/data` - Data ingestion for testing

### Related Documentation
- `doc/optimization_workflow.md` - Optimization workflow documentation
- `doc/architecture.md` - System architecture
- `doc/api.md` - API documentation

### Reference Implementation
- `GoldenCross` strategy is often the reference implementation - examine for patterns
- `validate_strategy` binary shows validation approach

## Notes

### Implementation Approach
1. **Inventory Phase**: List all strategies and their current state
2. **Audit Phase**: Test each strategy individually to identify issues
3. **Fix Phase**: Fix parameter bounds, signal generation, and trading mode
4. **Test Phase**: Add comprehensive test coverage
5. **Integration Phase**: Verify all strategies work with optimization workflow
6. **Validation Phase**: Run batch validation across all strategies

### Risk Factors
- **Breaking Changes**: Fixing strategy interfaces may require changes to dependent code
- **Performance**: Adding validation may slow down optimization - monitor performance
- **Data Requirements**: Some strategies need substantial historical data - ensure test data available

### Questions for Research/Planning
1. Are there strategies that should be deprecated or merged?
2. Should we add a strategy capability matrix (which features each strategy supports)?
3. How should we handle strategies that fundamentally can't work with certain asset categories?
4. Should we add a "strategy health" dashboard metric?

### Definition of Done
- All 47+ strategies audited and fixed
- All strategies generate trades in backtest
- All strategies optimize successfully
- All strategies respect Spot trading mode
- Comprehensive test suite (>200 tests) passing
- Dashboard optimization works for all strategies
- Documentation updated with any API changes
- Code review completed and approved
