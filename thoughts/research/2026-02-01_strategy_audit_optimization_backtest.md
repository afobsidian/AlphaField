---
date: 2026-02-01T21:16:35+11:00
git_commit: 7b67a41bec3d24272d373fe666128f683237f8d8
branch: feature/validation-strategies
repository: AlphaField
topic: Complete Strategy Audit - Fix Optimization & Backtest Integration
status: researched
tags: [research, strategies, optimization, backtest, audit, debt]
last_updated: 2026-02-01
---

## Ticket Synopsis

Comprehensive audit and remediation of all 47+ trading strategies in the AlphaField codebase to ensure proper integration with optimization workflow, trade generation, and trading mode constraints. The ticket addresses critical issues where strategies fail to generate trades or optimize properly through the dashboard.

**Primary Issues:**
1. No trades generated during backtesting for many strategies
2. Trading mode violations (Spot vs Margin not respected)
3. Broken optimization (parameter sweeps fail or return invalid results)
4. Missing functional connections between strategies and optimizer

**Key Components:**
- 44 strategy files across 6 categories (trend_following, momentum, mean_reversion, volatility, multi_indicator, sentiment)
- Parameter bounds in optimizer.rs:306-679
- StrategyFactory for strategy instantiation
- Optimization workflow with walk-forward validation
- Backtest engine with TradingMode support

## Summary

The AlphaField codebase has a sophisticated strategy optimization and backtest system with proper infrastructure in place. However, **43 out of 44 strategies have parameter bounds defined in `get_strategy_bounds()`**, suggesting most strategies should be optimizable. The main integration points are:

1. **StrategyFactory** (`crates/dashboard/src/services/strategy_service.rs`) - Creates strategies from parameters
2. **get_strategy_bounds** (`crates/backtest/src/optimizer.rs:306-679`) - Defines optimization parameter ranges
3. **StrategyAdapter** (`crates/backtest/src/adapter.rs`) - Bridges Signal-based strategies to Order-based backtest
4. **OptimizationWorkflow** (`crates/backtest/src/optimization_workflow.rs`) - Orchestrates grid search, walk-forward, and Monte Carlo

**Critical Finding:** Strategies DO generate signals properly, but the hardcoded "UNKNOWN" symbol in signals is by design - the StrategyAdapter handles proper symbol mapping. The real issues are likely:
- Parameter bounds mismatch between StrategyFactory and get_strategy_bounds
- Trading mode not being consistently passed through the optimization workflow
- Some strategies may fail validation in StrategyFactory, returning None

## Detailed Findings

### Strategy Inventory

The codebase contains **44 strategy implementations** across 6 categories:

**Trend Following (8 strategies):**
- `crates/strategy/src/strategies/trend_following/golden_cross.rs` - GoldenCross
- `crates/strategy/src/strategies/trend_following/breakout.rs` - Breakout
- `crates/strategy/src/strategies/trend_following/ma_crossover.rs` - MACrossover
- `crates/strategy/src/strategies/trend_following/adaptive_ma.rs` - AdaptiveMA
- `crates/strategy/src/strategies/trend_following/triple_ma.rs` - TripleMA
- `crates/strategy/src/strategies/trend_following/macd_trend.rs` - MacdTrend
- `crates/strategy/src/strategies/trend_following/parabolic_sar.rs` - ParabolicSAR

**Momentum (7 strategies):**
- `crates/strategy/src/strategies/momentum/rsi_momentum.rs` - RsiMomentumStrategy
- `crates/strategy/src/strategies/momentum/macd_strategy.rs` - MACDStrategy (momentum-based)
- `crates/strategy/src/strategies/momentum/roc_strategy.rs` - RocStrategy
- `crates/strategy/src/strategies/momentum/adx_trend.rs` - AdxTrendStrategy
- `crates/strategy/src/strategies/momentum/momentum_factor.rs` - MomentumFactorStrategy
- `crates/strategy/src/strategies/momentum/volume_momentum.rs` - VolumeMomentumStrategy
- `crates/strategy/src/strategies/momentum/multi_tf_momentum.rs` - MultiTfMomentumStrategy

**Mean Reversion (7 strategies):**
- `crates/strategy/src/strategies/mean_reversion/bollinger_bands.rs` - BollingerBands
- `crates/strategy/src/strategies/mean_reversion/rsi_reversion.rs` - RSIReversion
- `crates/strategy/src/strategies/mean_reversion/stoch_reversion.rs` - StochReversion
- `crates/strategy/src/strategies/mean_reversion/zscore_reversion.rs` - ZScoreReversion
- `crates/strategy/src/strategies/mean_reversion/price_channel.rs` - PriceChannel
- `crates/strategy/src/strategies/mean_reversion/keltner_reversion.rs` - KeltnerReversion
- `crates/strategy/src/strategies/mean_reversion/stat_arb.rs` - StatArb

**Volatility (8 strategies):**
- `crates/strategy/src/strategies/volatility/atr_breakout.rs` - ATRBreakout
- `crates/strategy/src/strategies/volatility/atr_trailing_stop.rs` - ATRTrailingStop
- `crates/strategy/src/strategies/volatility/volatility_squeeze.rs` - VolatilitySqueeze
- `crates/strategy/src/strategies/volatility/vol_regime.rs` - VolRegimeStrategy
- `crates/strategy/src/strategies/volatility/vol_sizing.rs` - VolSizingStrategy
- `crates/strategy/src/strategies/volatility/garch.rs` - GarchStrategy
- `crates/strategy/src/strategies/volatility/vix_style.rs` - VIXStyleStrategy

**Multi-Indicator (8 strategies):**
- `crates/strategy/src/strategies/multi_indicator/trend_mean_rev.rs` - TrendMeanRev
- `crates/strategy/src/strategies/multi_indicator/macd_rsi_combo.rs` - MACDRSICombo
- `crates/strategy/src/strategies/multi_indicator/adaptive_combo.rs` - AdaptiveCombo
- `crates/strategy/src/strategies/multi_indicator/confidence_weighted.rs` - ConfidenceWeighted
- `crates/strategy/src/strategies/multi_indicator/ensemble_weighted.rs` - EnsembleWeighted
- `crates/strategy/src/strategies/multi_indicator/ml_enhanced.rs` - MLEnhanced
- `crates/strategy/src/strategies/multi_indicator/regime_switching.rs` - RegimeSwitching

**Sentiment (3 strategies):**
- `crates/strategy/src/strategies/sentiment/divergence.rs` - Divergence
- `crates/strategy/src/strategies/sentiment/regime_sentiment.rs` - RegimeSentiment
- `crates/strategy/src/strategies/sentiment/sentiment_momentum.rs` - SentimentMomentum

**Baselines (2 strategies):**
- HODL_Baseline
- Market_Average_Baseline

### Optimization & Backtest Integration Architecture

#### 1. Parameter Bounds System (`crates/backtest/src/optimizer.rs:306-679`)

The `get_strategy_bounds()` function defines parameter ranges for optimization. It includes **43 strategies** with parameter bounds:

```rust
pub fn get_strategy_bounds(strategy_name: &str) -> Vec<ParamBounds> {
    let strategy_name = canonicalize_strategy_name(strategy_name);
    match strategy_name.as_str() {
        "GoldenCross" => vec![
            ParamBounds::new("fast_period", 5.0, 30.0, 5.0),
            ParamBounds::new("slow_period", 30.0, 100.0, 10.0),
            ParamBounds::new("take_profit", 2.0, 10.0, 2.0),
            ParamBounds::new("stop_loss", 2.0, 10.0, 2.0),
        ],
        // ... 42 more strategies
        _ => vec![], // Returns empty for unknown strategies
    }
}
```

**Key Points:**
- Uses `canonicalize_strategy_name()` to handle both display names ("Golden Cross") and internal keys ("GoldenCross")
- All 44 strategies except baselines have parameter bounds defined
- Parameter bounds include ranges and step sizes for grid search

#### 2. StrategyFactory (`crates/dashboard/src/services/strategy_service.rs`)

The `StrategyFactory` creates strategy instances from parameters for the optimization workflow:

```rust
impl StrategyFactory {
    pub fn create(name: &str, params: &HashMap<String, f64>) -> Option<Box<dyn Strategy>> {
        let name = canonicalize_strategy_name(name);
        match name.as_str() {
            "GoldenCross" => {
                let fast = params.get("fast_period").copied().unwrap_or(10.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(30.0) as usize;
                let tp = params.get("take_profit").copied().unwrap_or(5.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if fast >= slow || fast == 0 || slow == 0 {
                    return None; // Validation failure
                }
                let config = alphafield_strategy::config::GoldenCrossConfig::new(fast, slow, tp, sl);
                Some(Box::new(GoldenCrossStrategy::from_config(config)))
            }
            // ... other strategies
        }
    }

    pub fn create_backtest(
        name: &str,
        params: &HashMap<String, f64>,
        symbol: &str,
        capital: f64,
    ) -> Option<Box<dyn BacktestStrategy>> {
        // Creates strategy wrapped in StrategyAdapter
        Self::create(name, params).map(|s| {
            Box::new(StrategyAdapter::new(s, symbol, capital)) as Box<dyn BacktestStrategy>
        })
    }
}
```

**Issues Identified:**
- StrategyFactory validates parameters and returns `None` for invalid configs (causing optimization to skip)
- Parameter names must match exactly between `get_strategy_bounds()` and `StrategyFactory`
- Some strategies ignore certain parameters (e.g., Breakout ignores take_profit/stop_loss)

#### 3. Optimization Workflow (`crates/backtest/src/optimization_workflow.rs`)

The workflow orchestrates the full optimization pipeline:

```rust
pub struct WorkflowConfig {
    pub initial_capital: f64,
    pub fee_rate: f64,
    pub slippage: SlippageModel,
    pub walk_forward_config: WalkForwardConfig,
    pub include_3d_sensitivity: bool,
    pub train_test_split_ratio: f64,
    pub monte_carlo_config: Option<MonteCarloConfig>,
    pub risk_free_rate: f64,
    pub trading_mode: TradingMode,  // CRITICAL: Spot vs Margin
}

impl OptimizationWorkflow {
    pub fn run<F>(
        &self,
        data: &[Bar],
        symbol: &str,
        strategy_factory: &F,
        bounds: &[ParamBounds],
        sensitivity_params: Option<(ParameterRange, ParameterRange)>,
    ) -> Result<WorkflowResult, String>
    where
        F: Fn(&HashMap<String, f64>) -> Option<Box<dyn Strategy>>,
    {
        // Phase 1: Grid Search Optimization
        // Phase 2: Parameter Dispersion
        // Phase 3: Walk-Forward Validation
        // Phase 4: 3D Sensitivity Analysis
        // Phase 5: Monte Carlo Simulation
        // Phase 6: Calculate Robustness Score
    }
}
```

**Key Points:**
- WorkflowConfig includes `trading_mode: TradingMode` (defaults to Spot)
- The strategy_factory closure is called for each parameter combination
- If factory returns `None`, that combination is skipped

#### 4. Backtest Engine (`crates/backtest/src/engine.rs`)

```rust
pub struct BacktestEngine {
    pub portfolio: Portfolio,
    pub exchange: ExchangeSimulator,
    pub data: HashMap<String, Vec<Bar>>,
    pub strategy: Option<Box<dyn Strategy>>,
    pub trading_mode: TradingMode,  // Spot or Margin
}

impl BacktestEngine {
    pub fn with_trading_mode(mut self, trading_mode: TradingMode) -> Self {
        self.trading_mode = trading_mode;
        self.portfolio = self.portfolio.with_trading_mode(trading_mode);
        self
    }
}
```

**Critical Issue:** The engine supports TradingMode but it's not always passed through consistently.

#### 5. StrategyAdapter (`crates/backtest/src/adapter.rs`)

Bridges Signal-based strategies to Order-based backtest engine:

```rust
pub struct StrategyAdapter<T: alphafield_core::Strategy> {
    inner: T,
    symbol: String,
    capital: f64,
    trade_pct: f64,
    trading_mode: TradingMode,  // CRITICAL
    position: PositionState,
    position_quantity: f64,
}

impl<T: alphafield_core::Strategy> BacktestStrategy for StrategyAdapter<T> {
    fn on_bar(&mut self, bar: &Bar) -> Result<Vec<OrderRequest>> {
        let signals = self.inner.on_bar(bar);
        // Converts Signal to OrderRequest
        // Respects TradingMode: Spot = long-only, Margin = long/short
    }
}
```

**Trading Mode Logic:**
- **Spot Mode:** Only buys when flat, only sells when long (closes position)
- **Margin Mode:** Can open long from flat, open short from flat, close long with sell, close short with buy

### Signal Generation Patterns

All strategies follow a consistent pattern for signal generation:

**Golden Cross Example (`golden_cross.rs:261-446`):**
```rust
fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
    // Update indicators
    let fast_opt = self.fast_sma.update(bar.close);
    let slow_opt = self.slow_sma.update(bar.close);
    
    let fast = fast_opt?;
    let slow = slow_opt?;
    
    // EXIT LOGIC FIRST
    if let Some(entry) = self.entry_price {
        // Check TP, SL, trailing stop, death cross
    }
    
    // ENTRY LOGIC
    if prev_fast <= prev_slow && fast > slow {
        // Apply filters: separation, RSI, ADX, volume
        // Return Buy signal
        return Some(vec![Signal {
            timestamp: bar.timestamp,
            symbol: "UNKNOWN".to_string(),  // Hardcoded by design
            signal_type: SignalType::Buy,
            strength: 1.0,
            metadata: Some("Golden Cross".to_string()),
        }]);
    }
    
    None
}
```

**Key Pattern:** All strategies use hardcoded "UNKNOWN" symbol - this is intentional as StrategyAdapter handles proper symbol mapping.

### Trading Mode Compliance

TradingMode is defined in `core/src/lib.rs:373-380`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TradingMode {
    #[default]
    Spot,    // No short positions
    Margin,  // Allows short positions
}
```

**StrategyAdapter enforces TradingMode (`adapter.rs:69-157`):**

```rust
// Buy signal handling
match sig.signal_type {
    SignalType::Buy => {
        match (self.trading_mode, self.position) {
            // Spot: Only buy when flat
            (TradingMode::Spot, PositionState::Flat) |
            // Margin: Buy when flat or short
            (TradingMode::Margin, PositionState::Flat | PositionState::Short) => {
                // Create order
            }
            _ => {} // No action
        }
    }
    SignalType::Sell => {
        match (self.trading_mode, self.position) {
            // Spot: Only sell when long (close position)
            (TradingMode::Spot, PositionState::Long) |
            // Margin: Sell when flat (open short) or long (close long)
            (TradingMode::Margin, PositionState::Flat | PositionState::Long) => {
                // Create order
            }
            _ => {} // No action
        }
    }
}
```

### Dashboard API Integration

The dashboard API endpoints use the optimization workflow (`backtest_api.rs:520-704`):

```rust
pub async fn optimize_params(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<OptimizeRequest>,
) -> Json<OptimizeResponse> {
    // 1. Fetch data
    // 2. Get parameter bounds
    let bounds = get_strategy_bounds(strategy_name);
    if bounds.is_empty() {
        return error_response("Unknown strategy");
    }
    
    // 3. Run optimizer
    let optimization_result = tokio::task::spawn_blocking(move || {
        optimizer.optimize(
            &bars,
            &symbol,
            |params| {
                StrategyFactory::create_backtest(&strategy_name, params, &symbol, 100_000.0)
            },
            &bounds,
        )
    }).await;
}
```

## Code References

### Core Files

- `crates/core/src/lib.rs:373-380` - TradingMode enum definition
- `crates/core/src/lib.rs:401-418` - Strategy trait definition
- `crates/core/src/lib.rs:382-400` - Signal struct with hardcoded symbol

### Strategy Framework

- `crates/strategy/src/framework.rs:36-54` - StrategyMetadata struct
- `crates/strategy/src/framework.rs:132-140` - MetadataStrategy trait
- `crates/strategy/src/framework.rs:518-655` - canonicalize_strategy_name function
- `crates/strategy/src/framework.rs:341-489` - StrategyRegistry implementation

### Backtest Engine

- `crates/backtest/src/engine.rs:10-17` - BacktestEngine struct with trading_mode
- `crates/backtest/src/engine.rs:41-45` - with_trading_mode method
- `crates/backtest/src/engine.rs:96-152` - run() method that processes signals

### Strategy Adapter

- `crates/backtest/src/adapter.rs:14-35` - StrategyAdapter struct
- `crates/backtest/src/adapter.rs:53-57` - with_trading_mode method
- `crates/backtest/src/adapter.rs:65-158` - on_bar() with TradingMode logic

### Optimization

- `crates/backtest/src/optimizer.rs:14-42` - ParamBounds struct
- `crates/backtest/src/optimizer.rs:89-126` - calculate_composite_score function
- `crates/backtest/src/optimizer.rs:180-303` - ParameterOptimizer::optimize method
- `crates/backtest/src/optimizer.rs:306-679` - get_strategy_bounds function with 43 strategies

- `crates/backtest/src/optimization_workflow.rs:31-67` - WorkflowConfig with trading_mode
- `crates/backtest/src/optimization_workflow.rs:209-411` - OptimizationWorkflow::run method

### Dashboard API

- `crates/dashboard/src/services/strategy_service.rs:10-150` - StrategyFactory implementation
- `crates/dashboard/src/backtest_api.rs:19-39` - BacktestRequest with trading_mode
- `crates/dashboard/src/backtest_api.rs:336-342` - TradingMode parsing in run_backtest
- `crates/dashboard/src/backtest_api.rs:520-704` - optimize_params endpoint
- `crates/dashboard/src/backtest_api.rs:780-1054` - run_optimization_workflow endpoint

## Architecture Insights

### 1. Symbol Handling Design Pattern

The hardcoded "UNKNOWN" symbol in all strategy signals is **intentional by design** (documented in `framework.rs:9-26`):

- The `Strategy` trait's `on_bar` method only receives a `Bar` reference
- The `Signal` struct requires an owned `String` for symbol
- `StrategyAdapter` wraps strategies and handles proper symbol mapping from `Bar` to `Signal`
- This keeps strategies symbol-agnostic and reusable across assets

**Implication:** The "UNKNOWN" symbol is NOT a bug - it's a design pattern. The StrategyAdapter resolves it properly.

### 2. Trading Mode Enforcement Pattern

Trading mode is enforced at the **StrategyAdapter level**, not inside individual strategies:

- Strategies generate raw Buy/Sell signals without knowing the trading mode
- StrategyAdapter filters signals based on current position and TradingMode
- This centralizes trading mode logic and keeps strategies simpler

**Potential Issue:** Strategies don't know about TradingMode, so they can't adjust their logic (e.g., a strategy might want to be less aggressive in Spot mode).

### 3. Parameter Validation Pattern

Two-layer validation system:

1. **StrategyFactory validation** - Checks parameter values are valid (e.g., fast < slow for MA)
2. **Strategy config validation** - `from_config()` calls `config.validate()` which may panic

**Issue:** If StrategyFactory returns `None` for invalid params, the optimizer skips that combination. But if a strategy panics in `from_config()`, it could crash the optimization.

### 4. Optimization Score Calculation

The composite score (`optimizer.rs:96-126`) includes trade count penalties:

```rust
let trade_penalty = match total_trades {
    0 => f64::NEG_INFINITY, // No trades is invalid
    1 => 0.5,               // 50% penalty
    2 => 0.7,               // 30% penalty
    3 => 0.85,              // 15% penalty
    4 => 0.95,              // 5% penalty
    _ => 1.0,               // No penalty
};
```

**Implication:** Strategies that generate very few trades get heavily penalized in optimization, which could explain why some strategies appear to "fail" optimization.

### 5. Walk-Forward Validation Pattern

The workflow runs walk-forward validation using the optimized parameters:

```rust
// Create strategy factory that uses optimized parameters
let optimized_params = optimization_result.best_params.clone();
let wf_factory = || -> Box<dyn Strategy> {
    strategy_factory(&optimized_params)
        .expect("Failed to create strategy with optimized parameters")
};

let walk_forward_result = walk_forward_analyzer.analyze(data, symbol, wf_factory)?;
```

**Issue:** If `strategy_factory` returns `None` for the optimized params, the expect will panic.

## Historical Context (from thoughts/)

The codebase has the following documented research and plans:

- `thoughts/tickets/` contains the debt tickets for tracking issues
- `thoughts/tickets/debt_strategy_audit_optimization_backtest.md` - This research document
- Strategy framework was enhanced in Phase 12 with metadata support
- The canonicalize_strategy_name function was added to bridge UI display names with internal keys

## Related Research

- `doc/phase_12/12.1_summary.md` - Strategy framework enhancements
- `doc/phase_12/12.2_summary.md` - Trend following strategies implementation
- `doc/optimization_workflow.md` - Optimization workflow documentation
- `doc/architecture.md` - System architecture

## Open Questions

1. **Missing Strategies:** Ticket mentions 47+ strategies but only 44 files found. Are there 3 strategies not yet implemented or tracked elsewhere?

2. **Trading Mode Consistency:** The optimization workflow includes TradingMode in WorkflowConfig, but does StrategyFactory need to know about it for any strategies?

3. **Parameter Name Mismatches:** Need to verify all parameter names in `get_strategy_bounds()` exactly match what `StrategyFactory` expects

4. **Strategy Validation:** Some strategies call `config.validate().expect()` in `from_config()` - this could panic during optimization if invalid params are passed

5. **Asset Category Support:** Ticket mentions testing across 5 asset categories (Market, Large Cap, Mid Cap, Small Cap, DeFi) - where are these defined?

6. **Test Coverage:** Need to assess current test coverage for each strategy and identify gaps

## Recommendations for Fix Phase

### Priority 1: Fix Parameter Consistency
- Audit all parameter names in `get_strategy_bounds()` vs `StrategyFactory`
- Ensure all strategies have matching bounds and factory implementations
- Add missing strategies to StrategyFactory if any

### Priority 2: Fix Trading Mode Propagation
- Verify TradingMode is passed consistently through all optimization paths
- Ensure StrategyAdapter receives correct TradingMode in all code paths

### Priority 3: Fix Strategy Validation
- Replace `expect()` calls in strategy constructors with graceful error handling
- Ensure StrategyFactory returns `None` instead of panicking

### Priority 4: Add Tests
- Unit tests for signal generation with known inputs
- Integration tests for optimization workflow with each strategy
- E2E tests through dashboard API

### Priority 5: Asset Category Testing
- Define the 5 asset category baskets
- Create test data for each category
- Run batch validation across all strategies and categories
