# Phase 12: Strategy Library Expansion - AI Agent Execution Plan

> **Target**: Q1 2026  
> **Priority**: Critical  
> **Status**: In Progress  
> **Execution Mode**: AI Agent with Human Review  
> **Total Duration**: 22 weeks  

---

## 🎯 Mission Statement

Build a comprehensive, well-documented, and rigorously validated library of 50+ trading strategies through systematic AI-assisted development. Each strategy must follow a hypothesis-first approach, pass rigorous validation, and be fully documented before deployment.

---

## 🤖 AI Agent Execution Guidelines

### Agent Capabilities Required
- **Code Generation**: Implement Rust strategies following the `Strategy` trait
- **Documentation**: Generate hypothesis documents from templates
- **Testing**: Write unit tests, integration tests, and validation tests
- **Data Analysis**: Analyze backtest results and generate metrics
- **API Development**: Create REST endpoints for strategy management

### Agent Constraints
- **Never** implement a strategy without a written hypothesis
- **Never** skip validation steps (walk-forward, Monte Carlo, regime analysis)
- **Never** commit code without tests passing
- **Always** follow the established code structure and patterns
- **Always** use existing types and traits from the codebase

### Human Review Checkpoints
Human review REQUIRED at these stages:
- After framework setup (Phase 12.1)
- After each strategy category completion (Phases 12.2-12.7)
- After final integration (Phase 12.8)
- For any critical failures or unexpected results

### Agent Communication Protocol
- **Success**: Report completion with metrics and test results
- **Failure**: Report error with stack trace, logs, and attempted fixes
- **Ambiguity**: Request clarification with specific questions
- **Blockers**: Report immediately with dependency information

---

## 📋 Success Criteria

### Quantitative Targets (MUST MEET)
- [ ] **50+ strategies** implemented and validated
- [ ] **6 strategy categories** with 7+ strategies each
- [ ] **100% test coverage** (unit + integration)
- [ ] **100% documentation coverage** (hypothesis + failure modes)
- [ ] **100% baseline comparison** (vs HODL, Market Average)
- [ ] **90% walk-forward passing rate** (robustness score >50)
- [ ] **100% database integration** (performance metrics stored)

### Quality Gates (MUST PASS)
- [ ] All strategies compile without warnings (`cargo clippy -- -D warnings`)
- [ ] All tests pass (`cargo test`)
- [ ] All strategies pass walk-forward validation
- [ ] All strategies have documented failure modes
- [ ] All strategies are benchmarked against baselines
- [ ] All API endpoints are tested and documented

### Performance Benchmarks (MINIMUM)
- [ ] **Average Sharpe Ratio**: >1.0
- [ ] **Average Max Drawdown**: <25%
- [ ] **Average Win Rate**: >40%
- [ ] **Statistical Significance**: p < 0.05 for 90% of strategies
- [ ] **Parameter Stability**: CV < 30% for 90% of strategies

---

## 🏗️ Technical Architecture

### Directory Structure (MUST FOLLOW)

```
AlphaField/
├── crates/
│   ├── core/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs              # Bar, Trade, Order, Signal, Strategy traits
│   │       └── error.rs              # QuantError
│   ├── strategy/
│   │   └── src/
│   │       ├── lib.rs                # Main library entry
│   │       ├── framework.rs          # Strategy trait extensions, registry
│   │       ├── indicators.rs         # All indicator implementations
│   │       ├── baseline.rs           # Baseline strategies (HODL, etc.)
│   │       ├── registry.rs           # Strategy registration system
│   │       ├── classification.rs     # Strategy classification logic
│   │       ├── validation.rs         # Validation utilities
│   │       └── strategies/           # Strategy implementations
│   │           ├── mod.rs             # Strategy module exports
│   │           ├── trend_following/
│   │           │   ├── mod.rs
│   │           │   ├── golden_cross.rs
│   │           │   ├── breakout.rs
│   │           │   ├── ma_crossover.rs
│   │           │   ├── adaptive_ma.rs
│   │           │   ├── triple_ma.rs
│   │           │   ├── macd_trend.rs
│   │           │   └── parabolic_sar.rs
│   │           ├── mean_reversion/
│   │           │   ├── mod.rs
│   │           │   ├── bollinger_bands.rs
│   │           │   ├── rsi_reversion.rs
│   │           │   ├── stat_arb.rs
│   │           │   ├── stoch_reversion.rs
│   │           │   ├── keltner_reversion.rs
│   │           │   ├── price_channel.rs
│   │           │   └── zscore_reversion.rs
│   │           ├── momentum/
│   │           │   ├── mod.rs
│   │           │   ├── rsi_momentum.rs
│   │           │   ├── macd_strategy.rs
│   │           │   ├── roc_strategy.rs
│   │           │   ├── adx_trend.rs
│   │           │   ├── momentum_factor.rs
│   │           │   ├── volume_momentum.rs
│   │           │   └── multi_tf_momentum.rs
│   │           ├── volatility/
│   │           │   ├── mod.rs
│   │           │   ├── atr_breakout.rs
│   │           │   ├── vol_squeeze.rs
│   │           │   ├── vol_regime.rs
│   │           │   ├── atr_trailing.rs
│   │           │   ├── vol_sizing.rs
│   │           │   ├── garch_strategy.rs
│   │           │   └── vix_style.rs
│   │           ├── sentiment/
│   │           │   ├── mod.rs
│   │           │   ├── fear_greed_contrarian.rs
│   │           │   ├── sentiment_momentum.rs
│   │           │   ├── divergence_strategy.rs
│   │           │   ├── news_sentiment.rs
│   │           │   ├── social_volume.rs
│   │           │   ├── composite_sentiment.rs
│   │           │   └── regime_sentiment.rs
│   │           └── multi_indicator/
│   │               ├── mod.rs
│   │               ├── trend_mean_rev.rs
│   │               ├── macd_rsi_combo.rs
│   │               ├── adaptive_combo.rs
│   │               ├── ensemble_weighted.rs
│   │               ├── regime_switching.rs
│   │               ├── confidence_weighted.rs
│   │               └── ml_enhanced.rs
│   ├── backtest/
│   │   └── src/
│   │       ├── lib.rs                # Backtest engine
│   │       ├── strategy.rs          # Strategy trait (backtest version)
│   │       ├── execution.rs         # Order execution simulation
│   │       ├── metrics.rs           # Performance metrics calculation
│   │       ├── validation.rs         # Walk-forward, Monte Carlo, sensitivity
│   │       └── ml/                   # ML validation utilities
│   ├── execution/
│   │   └── src/
│   │       ├── lib.rs                # Risk management, order types
│   │       └── orders.rs             # Advanced order management
│   └── data/
│       └── src/
│           ├── lib.rs                # Data ingestion, database
│           └── client.rs             # Unified data client
├── doc/
│   └── phase_12/
│       ├── plan.md                   # This file
│       ├── strategy_template.md      # AI agent template for strategies
│       └── hypotheses/               # Strategy hypothesis documents
│           ├── template.md           # Hypothesis template
│           ├── trend_following/
│           ├── mean_reversion/
│           ├── momentum/
│           ├── volatility/
│           ├── sentiment/
│           └── multi_indicator/
└── migrations/
    └── phase_12/
        ├── 001_strategies.sql        # Strategy metadata tables
        ├── 002_performance.sql       # Performance metrics tables
        └── 003_failures.sql          # Failure modes tables
```

### Core Type Definitions (REFERENCE)

```rust
// Existing in crates/core/src/types.rs - DO NOT MODIFY
pub struct Bar {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub struct Trade {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub size: f64,
    pub side: OrderSide,
}

pub enum OrderSide {
    Buy,
    Sell,
}

pub enum SignalType {
    LongEntry,
    LongExit,
}

pub struct Signal {
    pub timestamp: DateTime<Utc>,
    pub strategy: String,
    pub signal_type: SignalType,
    pub price: f64,
}

// Existing in crates/backtest/src/strategy.rs - DO NOT MODIFY
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn on_bar(&mut self, bar: &Bar) -> Result<Option<Signal>>;
    fn on_tick(&mut self, tick: &Trade) -> Result<Option<Signal>>;
    fn reset(&mut self);
}
```

### New Type Definitions (IMPLEMENT IN PHASE 12.1)

```rust
// Create in crates/strategy/src/framework.rs
use crate::core::{Bar, Trade, Signal, SignalType, QuantError};

/// Strategy metadata for registry and database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetadata {
    pub name: String,
    pub category: StrategyCategory,
    pub sub_type: Option<String>,
    pub description: String,
    pub hypothesis_path: String,
    pub required_indicators: Vec<String>,
    pub expected_regimes: Vec<MarketRegime>,
    pub risk_profile: RiskProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyCategory {
    TrendFollowing,
    MeanReversion,
    Momentum,
    VolatilityBased,
    SentimentBased,
    MultiIndicator,
    Baseline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketRegime {
    Bull,
    Bear,
    Sideways,
    HighVolatility,
    LowVolatility,
    Trending,
    Ranging,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfile {
    pub max_drawdown_expected: f64,
    pub volatility_level: VolatilityLevel,
    pub correlation_sensitivity: CorrelationSensitivity,
    pub leverage_requirement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolatilityLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrelationSensitivity {
    Low,
    Medium,
    High,
}

/// Extended strategy trait for metadata
pub trait MetadataStrategy {
    fn metadata(&self) -> StrategyMetadata;
    fn category(&self) -> StrategyCategory;
}

// Strategy classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyClassification {
    pub strategy_name: String,
    pub primary_category: StrategyCategory,
    pub sub_type: Option<String>,
    pub characteristics: StrategyCharacteristics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyCharacteristics {
    pub trend_sensitive: bool,
    pub volatility_sensitive: bool,
    pub correlation_sensitive: bool,
    pub time_horizon: TimeHorizon,
    pub signal_frequency: SignalFrequency,
    pub risk_reward_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeHorizon {
    Scalping,      // < 1 hour
    Intraday,      // 1 hour - 1 day
    Swing,         // 1 day - 1 week
    ShortTerm,     // 1 week - 1 month
    MediumTerm,    // 1 month - 3 months
    LongTerm,      // > 3 months
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalFrequency {
    VeryHigh,   // Multiple signals per day
    High,       // Daily signals
    Medium,     // Weekly signals
    Low,        // Monthly signals
    VeryLow,    // Quarterly or less
}
```

### Database Schema (IMPLEMENT IN MIGRATIONS)

```sql
-- Create in migrations/phase_12/001_strategies.sql
CREATE TABLE IF NOT EXISTS strategies (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    category VARCHAR(50) NOT NULL,
    sub_type VARCHAR(50),
    description TEXT,
    hypothesis_path TEXT NOT NULL,
    required_indicators JSONB,
    expected_regimes JSONB,
    risk_profile JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE,
    UNIQUE(name)
);

CREATE INDEX idx_strategies_name ON strategies(name);
CREATE INDEX idx_strategies_category ON strategies(category);

-- Create in migrations/phase_12/002_performance.sql
CREATE TABLE IF NOT EXISTS strategy_performance (
    id SERIAL PRIMARY KEY,
    strategy_id INTEGER REFERENCES strategies(id) ON DELETE CASCADE,
    symbol VARCHAR(20) NOT NULL,
    timeframe VARCHAR(10) NOT NULL,
    test_period_start TIMESTAMPTZ NOT NULL,
    test_period_end TIMESTAMPTZ NOT NULL,
    
    -- Performance metrics
    total_return DECIMAL(10, 4) NOT NULL,
    sharpe_ratio DECIMAL(10, 4),
    sortino_ratio DECIMAL(10, 4),
    max_drawdown DECIMAL(10, 4) NOT NULL,
    max_drawdown_duration_days INTEGER,
    win_rate DECIMAL(5, 2),
    profit_factor DECIMAL(10, 2),
    expectancy DECIMAL(10, 4),
    sqn DECIMAL(10, 4),
    total_trades INTEGER NOT NULL,
    avg_trade_duration_hours DECIMAL(10, 2),
    avg_win DECIMAL(10, 4),
    avg_loss DECIMAL(10, 4),
    
    -- Regime performance
    regime_bull_return DECIMAL(10, 4),
    regime_bear_return DECIMAL(10, 4),
    regime_sideways_return DECIMAL(10, 4),
    regime_high_vol_return DECIMAL(10, 4),
    regime_low_vol_return DECIMAL(10, 4),
    
    -- Validation metrics
    robustness_score DECIMAL(5, 2),
    walk_forward_stability DECIMAL(5, 2),
    overfitting_risk VARCHAR(20),
    monte_carlo_95ci_low DECIMAL(10, 4),
    monte_carlo_95ci_high DECIMAL(10, 4),
    parameter_cv DECIMAL(5, 2),
    
    -- Comparison to baselines
    vs_hodl_return DECIMAL(10, 4),
    vs_market_avg_return DECIMAL(10, 4),
    
    tested_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(strategy_id, symbol, timeframe, test_period_start, test_period_end)
);

CREATE INDEX idx_perf_strategy ON strategy_performance(strategy_id);
CREATE INDEX idx_perf_symbol ON strategy_performance(symbol);
CREATE INDEX idx_perf_return ON strategy_performance(total_return DESC);
CREATE INDEX idx_perf_sharpe ON strategy_performance(sharpe_ratio DESC);
CREATE INDEX idx_perf_robustness ON strategy_performance(robustness_score DESC);

-- Create in migrations/phase_12/003_failures.sql
CREATE TABLE IF NOT EXISTS strategy_failures (
    id SERIAL PRIMARY KEY,
    strategy_id INTEGER REFERENCES strategies(id) ON DELETE CASCADE,
    failure_type VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    trigger_conditions TEXT,
    mitigation_strategy TEXT,
    severity VARCHAR(20) DEFAULT 'medium',
    documented_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_failures_strategy ON strategy_failures(strategy_id);
CREATE INDEX idx_failures_severity ON strategy_failures(severity);
```

---

## 📊 Overall Progress Summary

### Status: Phase 12.2 Complete, Phase 12.1 Complete

**Current Phase**: 12.2 (Trend Following Strategies)  
**Phase Status**: ✅ Complete (2026-01-11)  
**Next Phase**: 12.3 (Mean Reversion Strategies)

### Progress by Phase

| Phase | Name | Status | Completion Date | Strategies Completed | Notes |
|--------|-------|---------|---------------------|--------|
| 12.1 | Foundation | ✅ Complete | 2026-01-09 | Framework, baselines, templates, API integration |
| 12.2 | Trend Following | ✅ Complete | 2026-01-11 | 7 strategies implemented, documented, tested, integrated |
| 12.3 | Mean Reversion | ⏳ Not Started | TBD | Target: 7 strategies |
| 12.4 | Momentum | ⏳ Not Started | TBD | Target: 7 strategies |
| 12.5 | Volatility-Based | ⏳ Not Started | TBD | Target: 7 strategies |
| 12.6 | Sentiment-Based | ⏳ Not Started | TBD | Target: 7 strategies |
| 12.7 | Multi-Indicator | ⏳ Not Started | TBD | Target: 7 strategies |
| 12.8 | Research & Documentation | ⏳ Not Started | TBD | Final validation, documentation, reports |

### Overall Metrics

- **Total Phases**: 8
- **Completed**: 2 (25%)
- **In Progress**: 0 (0%)
- **Not Started**: 6 (75%)

- **Target Strategies**: 50+
- **Implemented**: 11 (22%)
  - Baselines: 2 (HODL, Market Average)
  - Existing: 2 (RSI, Momentum, Mean Reversion)
  - Trend Following: 7 (Golden Cross, Breakout, MA Crossover, Adaptive MA, Triple MA, MACD Trend, Parabolic SAR)
  - Remaining: 39

- **Total Code Written**: ~8,200+ lines
  - Framework: ~1,100 lines
  - Baselines: ~415 lines
  - Trend Following: ~4,040 lines
  - Dashboard API: ~630 lines
  - Documentation: ~2,300 lines
  - Migrations: ~260 lines
  - Tests: ~1,800 lines

- **Test Coverage**: 230+ tests, 100% pass rate
- **Documentation**: 100% coverage for completed strategies

### Quality Metrics

- **Compilation Warnings**: 0
- **Test Pass Rate**: 100% (230/230)
- **Code Review**: All Phase 12.1 and 12.2 code reviewed
- **Integration Status**: Dashboard API fully integrated

### Key Achievements

1. **Phase 12.1 (Foundation)**
   - ✅ Strategy framework with metadata and classification system
   - ✅ StrategyRegistry for dynamic strategy management
   - ✅ Baseline strategies (HODL, Market Average)
   - ✅ Database schema (strategies, performance, failures)
   - ✅ Hypothesis and strategy templates
   - ✅ Dashboard API integration

2. **Phase 12.2 (Trend Following)**
   - ✅ 7 trend-following strategies fully implemented
   - ✅ All strategies with comprehensive hypothesis documents
   - ✅ 3 new indicators (ATR, ADX, KAMA)
   - ✅ Dashboard API with strategy management endpoints
   - ✅ Strategy name canonicalization utility
   - ✅ Optimizer integration with parameter bounds
   - ✅ 230+ tests with 100% pass rate

### Next Immediate Steps

1. **Start Phase 12.3 (Mean Reversion Strategies)**
   - Implement 7 mean reversion strategies
   - Follow same pattern as Phase 12.2
   - Document, test, and integrate

2. **Run Validation on Phase 12.2 Strategies**
   - Execute walk-forward analysis
   - Run Monte Carlo simulations
   - Populate validation report
   - Generate rankings and recommendations

3. **Database Migration Execution**
   - Execute all Phase 12 migrations
   - Store strategy metadata
   - Initialize strategy performance tables

---

## 📊 Phase 12.1: Foundation (Weeks 1-2)

### Dependencies
- None (new development)

### Deliverables
- Strategy framework with registry
- Database schema migrations
- Hypothesis template
- Baseline strategies
- API endpoints for strategy management

---

### Task 12.1.1: Create Hypothesis Template (0.5 days)

**Objective**: Create a comprehensive template that AI agents can use to document strategy hypotheses.

**Instructions**:
1. Create file `doc/phase_12/hypotheses/template.md`
2. Include all sections from the example below
3. Use Markdown formatting with clear headings
4. Include placeholder values in square brackets `[like this]`

**Template Content**:

```markdown
# [Strategy Name] Hypothesis

## Metadata
- **Name**: [Strategy Name]
- **Category**: [TrendFollowing / MeanReversion / Momentum / VolatilityBased / SentimentBased / MultiIndicator]
- **Sub-Type**: [e.g., Golden Cross, Bollinger Bands, etc.]
- **Author**: AI Agent
- **Date**: [YYYY-MM-DD]
- **Status**: [Proposed / Testing / Validated / Deployed / Rejected]
- **Code Location**: [crates/strategy/src/strategies/[category]/[strategy].rs]

## 1. Hypothesis Statement

**Primary Hypothesis**: 
[One clear, testable statement about market behavior. Must be falsifiable.]

Example: "When the 50-day SMA crosses above the 200-day SMA (Golden Cross), the asset enters a sustainable uptrend and will generate positive returns over the next 30 trading days with an average return of >3%."

**Null Hypothesis**: 
[The opposite of the primary hypothesis - must be testable]

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
[Explain WHY this should work based on market mechanics, behavioral finance, or technical principles]

### 2.2 Market Inefficiency Exploited
[What market inefficiency does this strategy exploit? Why hasn't it been arbitraged away?]

### 2.3 Expected Duration of Edge
[How long will this edge persist? Under what conditions will it degrade?]

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: [High / Medium / Low]
- **Rationale**: [Why it works in bull markets]
- **Historical Evidence**: [Any supporting examples]

### 3.2 Bearish Markets
- **Expected Performance**: [High / Medium / Low]
- **Rationale**: [Why it works/doesn't work in bear markets]
- **Adaptations**: [Any modifications needed for bears]

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: [High / Medium / Low]
- **Rationale**: [Why it works/doesn't work in sideways markets]
- **Filters**: [Any filters to avoid sideways markets]

### 3.4 Volatility Conditions
- **High Volatility**: [Performance and adaptations]
- **Low Volatility**: [Performance and adaptations]
- **Volatility Filter**: [ATR threshold, etc.]

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: [e.g., 15%]
- **Average Drawdown**: [e.g., 5%]
- **Drawdown Duration**: [e.g., 1-3 weeks]
- **Worst Case Scenario**: [Describe worst possible outcome]

### 4.2 Failure Modes

#### Failure Mode 1: [Name]
- **Trigger**: [What causes this failure]
- **Impact**: [How bad is it - drawdown, frequency, etc.]
- **Mitigation**: [How to prevent or minimize]
- **Detection**: [How to detect this failure in real-time]

#### Failure Mode 2: [Name]
- **Trigger**: ...
- **Impact**: ...
- **Mitigation**: ...
- **Detection**: ...

### 4.3 Correlation Analysis
- **Correlation with Market**: [Low / Medium / High]
- **Correlation with Other Strategies**: [Which strategies?]
- **Diversification Value**: [What does this add to a portfolio?]

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1**: [Specific technical condition]
   - **Indicator**: [e.g., SMA crossover]
   - **Parameters**: [e.g., 50-day > 200-day]
   - **Confirmation**: [Any additional requirements]
   - **Priority**: [Required / Optional]

2. **Condition 2**: [Additional conditions as needed]

### 5.2 Entry Filters
- **Time of Day**: [If applicable]
- **Volume Requirements**: [e.g., Volume > 1.5x average]
- **Market Regime Filter**: [e.g., Only in trending markets]
- **Volatility Filter**: [e.g., ATR < 2x average]
- **Price Filter**: [e.g., Price above $X]

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: [e.g., Volume spike]
- **Confirmation Indicator 2**: [e.g., RSI not overbought]
- **Minimum Confirmed**: [e.g., 2 out of 3]

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP 1**: [e.g., 3% profit] - Close [e.g., 50%]
- **TP 2**: [e.g., 5% profit] - Close [e.g., 30%]
- **TP 3**: [e.g., 10% profit] - Close [e.g., 20%]
- **Trailing**: [Trailing stop after TP 1?]

### 6.2 Stop Loss
- **Initial SL**: [e.g., 2% below entry]
- **Trailing SL**: [e.g., 2% trailing after TP 1]
- **Breakeven**: [Move to breakeven after TP 1?]
- **Time-based Exit**: [e.g., Close after 30 days if no TP]

### 6.3 Exit Conditions
- **Reversal Signal**: [e.g., Death cross, opposite signal]
- **Regime Change**: [e.g., Market turns bearish]
- **Volatility Spike**: [e.g., ATR > 2x average]
- **Time Limit**: [e.g., Maximum 60 days in position]

## 7. Position Sizing

- **Base Position Size**: [e.g., 1% of portfolio]
- **Volatility Adjustment**: [e.g., Scale down if ATR > 1.5x avg]
- **Conviction Levels**: [Adjust size based on signal strength]
- **Max Position Size**: [e.g., 5% of portfolio]
- **Risk per Trade**: [e.g., Max 1% risk]

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| [param1] | [value] | [min-max] | [description] | [int/float] |
| [param2] | [value] | [min-max] | [description] | [int/float] |

### 8.2 Optimization Notes
- **Parameters to Optimize**: [Which parameters]
- **Optimization Method**: [Grid search / Walk-forward / Bayesian]
- **Optimization Period**: [Time period for optimization]
- **Expected Overfitting Risk**: [Low / Medium / High]
- **Sensitivity Analysis Required**: [Yes / No]

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- **Timeframes**: [e.g., Daily, 4H, 1H]
- **Test Period**: [e.g., 2019-2025, 6 years]
- **Assets**: [e.g., BTC, ETH, SOL, 10+ others]
- **Minimum Trades**: [For statistical significance]
- **Slippage**: [e.g., 0.1% per trade]
- **Commission**: [e.g., 0.1% per trade]

### 9.2 Validation Techniques
- [ ] Walk-forward analysis (rolling window)
- [ ] Monte Carlo simulation (trade sequence randomization)
- [ ] Parameter sweep (sensitivity analysis)
- [ ] Regime analysis (bull/bear/sideways)
- [ ] Cross-asset validation (multiple symbols)
- [ ] Bootstrap validation (resampling)
- [ ] Permutation testing (randomness check)

### 9.3 Success Criteria
- **Minimum Sharpe Ratio**: [e.g., 1.0]
- **Minimum Sortino Ratio**: [e.g., 1.5]
- **Maximum Max Drawdown**: [e.g., 20%]
- **Minimum Win Rate**: [e.g., 40%]
- **Minimum Profit Factor**: [e.g., 1.3]
- **Minimum Robustness Score**: [e.g., >70]
- **Statistical Significance**: [e.g., p < 0.05]
- **Walk-Forward Stability**: [e.g., >50]

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: [e.g., 20-30%]
- **Sharpe Ratio**: [e.g., 1.5-2.0]
- **Max Drawdown**: [e.g., <15%]
- **Win Rate**: [e.g., 45-55%]
- **Profit Factor**: [e.g., >1.5]
- **Expectancy**: [e.g., >0.02]

### 10.2 Comparison to Baselines
- **vs. HODL**: [Expected outperformance in %]
- **vs. Market Average**: [Expected outperformance in %]
- **vs. Similar Strategies**: [Unique advantages]
- **vs. Buy & Hold**: [Risk-adjusted comparison]

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: [List of indicators]
- **Data Requirements**: [OHLCV, volume, sentiment, etc.]
- **Latency Sensitivity**: [Low / Medium / High]
- **Computational Complexity**: [Low / Medium / High]
- **Memory Requirements**: [Approximate]

### 11.2 Code Structure
- **File Location**: [crates/strategy/src/strategies/[category]/[strategy].rs]
- **Strategy Type**: [Simple / Multi-indicator / ML-based]
- **Dependencies**: [External crates or internal modules]
- **State Management**: [How to track state between bars]

### 11.3 Indicator Calculations
[Describe how each indicator is calculated or reference existing implementations]

## 12. Testing Plan

### 12.1 Unit Tests
- [ ] Entry conditions (all signals generated correctly)
- [ ] Exit conditions (all exits triggered correctly)
- [ ] Edge cases (empty data, single bar, etc.)
- [ ] Parameter validation (invalid params rejected)
- [ ] State management (reset works correctly)

### 12.2 Integration Tests
- [ ] Backtest execution (runs without errors)
- [ ] Performance calculation (metrics are correct)
- [ ] Dashboard integration (API works)
- [ ] Database integration (saves correctly)

### 12.3 Research Tests
- [ ] Hypothesis validation (supports primary hypothesis)
- [ ] Statistical significance (p < 0.05)
- [ ] Regime analysis (performance as expected)
- [ ] Robustness testing (stable across parameters)

## 13. Research Journal

### [Date]: Initial Implementation
**Observation**: [What did you notice during implementation?]
**Hypothesis Impact**: [Does code support or contradict hypothesis?]
**Issues Found**: [Any problems or edge cases]
**Action Taken**: [What did you change?]

### [Date]: Initial Backtest Results
**Test Period**: [Date range]
**Symbols Tested**: [List of symbols]
**Results**: [Summary of metrics]
**Observation**: [Are results as expected?]
**Action Taken**: [Proceed to validation or refine strategy?]

### [Date]: Parameter Optimization
**Optimization Method**: [Grid search, etc.]
**Best Parameters**: [Results]
**Optimization Score**: [Performance score]
**Overfitting Check**: [Any concerns about overfitting?]
**Action Taken**: [Accept parameters or continue optimizing?]

### [Date]: Walk-Forward Validation
**Configuration**: [Window sizes, etc.]
**Results**: [Summary of WFA performance]
**Stability Score**: [How stable across windows?]
**Decision**: [Accept, reject, or modify?]

### [Date]: Monte Carlo Simulation
**Number of Simulations**: [e.g., 1000]
**95% Confidence Interval**: [Range]
**Best Case**: [Best outcome]
**Worst Case**: [Worst outcome]
**Observation**: [Is strategy robust to luck?]

### [Date]: Final Decision
**Final Verdict**: [Accept / Reject / Needs More Work]
**Reasoning**: [Why this decision?]
**Deployment**: [If accepted, when to deploy?]
**Monitoring**: [What to monitor in live trading?]

## 14. References

### Academic Sources
- [Citation of relevant academic papers]

### Books
- [Citation of relevant books]

### Online Resources
- [URLs to blog posts, articles, etc.]

### Similar Strategies
- [References to similar strategies and their performance]

### Historical Examples
- [Examples of this strategy working/failing in real markets]

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| [date] | 1.0 | Initial hypothesis | AI Agent |
| [date] | 1.1 | [Changes made] | AI Agent |
| [date] | 1.2 | [Changes made] | AI Agent |
```

**Validation**:
- [ ] Template created in correct location
- [ ] All sections present
- [ ] Placeholder values clear
- [ ] Markdown formatting correct

**Acceptance Criteria**:
- AI agent can use this template to generate hypothesis documents
- Template covers all required aspects of strategy documentation
- Template is human-readable and well-structured

---

### Task 12.1.2: Create Database Schema Migrations (0.5 days)

**Objective**: Create database schema migrations for storing strategy metadata, performance, and failure modes.

**Instructions**:
1. Create `migrations/phase_12/001_strategies.sql`
2. Create `migrations/phase_12/002_performance.sql`
3. Create `migrations/phase_12/003_failures.sql`
4. Use the SQL from the Database Schema section above
5. Include proper indexes for performance
6. Use `IF NOT EXISTS` for safety

**Validation**:
- [ ] All three migration files created
- [ ] SQL is valid and properly formatted
- [ ] All required tables present
- [ ] Indexes created for performance
- [ ] Foreign key relationships correct

**Acceptance Criteria**:
- Migrations can be run without errors
- Schema supports all required functionality
- Performance indexes optimize common queries

---

### Task 12.1.3: Implement Strategy Framework (2 days)

**Objective**: Implement the core framework for strategy management, including the registry, metadata trait, and classification system.

**File to Create**: `crates/strategy/src/framework.rs`

**Instructions**:
1. Implement all type definitions from the "New Type Definitions" section
2. Implement `StrategyRegistry` with registration and lookup
3. Implement `StrategyClassifier` with automatic classification
4. Ensure all types are serializable with `serde`
5. Include comprehensive error handling

**Code Structure**:

```rust
use crate::core::{Bar, Trade, Signal, SignalType, QuantError};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ============ Type Definitions ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetadata {
    pub name: String,
    pub category: StrategyCategory,
    pub sub_type: Option<String>,
    pub description: String,
    pub hypothesis_path: String,
    pub required_indicators: Vec<String>,
    pub expected_regimes: Vec<MarketRegime>,
    pub risk_profile: RiskProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyCategory {
    TrendFollowing,
    MeanReversion,
    Momentum,
    VolatilityBased,
    SentimentBased,
    MultiIndicator,
    Baseline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketRegime {
    Bull,
    Bear,
    Sideways,
    HighVolatility,
    LowVolatility,
    Trending,
    Ranging,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfile {
    pub max_drawdown_expected: f64,
    pub volatility_level: VolatilityLevel,
    pub correlation_sensitivity: CorrelationSensitivity,
    pub leverage_requirement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolatilityLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrelationSensitivity {
    Low,
    Medium,
    High,
}

// ============ Extended Traits ============

pub trait MetadataStrategy {
    fn metadata(&self) -> StrategyMetadata;
    fn category(&self) -> StrategyCategory;
}

// ============ Strategy Classification ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyClassification {
    pub strategy_name: String,
    pub primary_category: StrategyCategory,
    pub sub_type: Option<String>,
    pub characteristics: StrategyCharacteristics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyCharacteristics {
    pub trend_sensitive: bool,
    pub volatility_sensitive: bool,
    pub correlation_sensitive: bool,
    pub time_horizon: TimeHorizon,
    pub signal_frequency: SignalFrequency,
    pub risk_reward_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeHorizon {
    Scalping,      // < 1 hour
    Intraday,      // 1 hour - 1 day
    Swing,         // 1 day - 1 week
    ShortTerm,     // 1 week - 1 month
    MediumTerm,    // 1 month - 3 months
    LongTerm,      // > 3 months
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalFrequency {
    VeryHigh,   // Multiple signals per day
    High,       // Daily signals
    Medium,     // Weekly signals
    Low,        // Monthly signals
    VeryLow,    // Quarterly or less
}

// ============ Backtest Results (for Classification) ============

// Note: This should match the actual BacktestResults type in the backtest crate
// For now, we'll use a simplified version for classification
#[derive(Debug, Clone)]
pub struct BacktestResults {
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub total_trades: usize,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub avg_trade_duration_hours: f64,
    pub test_period_days: f64,
    pub market_correlation: f64,
    pub regime_bull_return: f64,
    pub regime_bear_return: f64,
    pub regime_high_vol_return: f64,
    pub regime_low_vol_return: f64,
}

// ============ Strategy Classifier ============

pub struct StrategyClassifier;

impl StrategyClassifier {
    pub fn analyze_strategy(
        strategy_name: &str,
        metadata: &StrategyMetadata,
        test_results: &BacktestResults,
    ) -> StrategyClassification {
        let characteristics = StrategyCharacteristics {
            trend_sensitive: Self::calculate_trend_sensitivity(test_results),
            volatility_sensitive: Self::calculate_volatility_sensitivity(test_results),
            correlation_sensitive: Self::calculate_correlation_sensitivity(test_results),
            time_horizon: Self::estimate_time_horizon(test_results),
            signal_frequency: Self::estimate_signal_frequency(test_results),
            risk_reward_ratio: if test_results.avg_loss != 0.0 {
                test_results.avg_win / test_results.avg_loss.abs()
            } else {
                0.0
            },
        };

        StrategyClassification {
            strategy_name: strategy_name.to_string(),
            primary_category: metadata.category.clone(),
            sub_type: metadata.sub_type.clone(),
            characteristics,
        }
    }

    fn calculate_trend_sensitivity(results: &BacktestResults) -> bool {
        // Compare bull market performance vs bear market performance
        // If significantly better in trending markets, return true
        let diff = (results.regime_bull_return - results.regime_bear_return).abs();
        diff > 10.0
    }

    fn calculate_volatility_sensitivity(results: &BacktestResults) -> bool {
        // Compare high vol vs low vol performance
        let diff = (results.regime_high_vol_return - results.regime_low_vol_return).abs();
        diff > 10.0
    }

    fn calculate_correlation_sensitivity(results: &BacktestResults) -> bool {
        // Check performance vs market correlation
        results.market_correlation.abs() > 0.7
    }

    fn estimate_time_horizon(results: &BacktestResults) -> TimeHorizon {
        let avg_hours = results.avg_trade_duration_hours;
        match avg_hours {
            h if h < 1.0 => TimeHorizon::Scalping,
            h if h < 24.0 => TimeHorizon::Intraday,
            h if h < 168.0 => TimeHorizon::Swing,
            h if h < 720.0 => TimeHorizon::ShortTerm,
            h if h < 2160.0 => TimeHorizon::MediumTerm,
            _ => TimeHorizon::LongTerm,
        }
    }

    fn estimate_signal_frequency(results: &BacktestResults) -> SignalFrequency {
        let trades_per_day = results.total_trades as f64 / results.test_period_days;
        match trades_per_day {
            t if t > 5.0 => SignalFrequency::VeryHigh,
            t if t > 1.0 => SignalFrequency::High,
            t if t > 0.2 => SignalFrequency::Medium,
            t if t > 0.03 => SignalFrequency::Low,
            _ => SignalFrequency::VeryLow,
        }
    }
}

// ============ Strategy Registry ============

pub struct StrategyRegistry {
    strategies: Arc<RwLock<HashMap<String, Arc<dyn Strategy + MetadataStrategy>>>>,
    metadata: Arc<RwLock<HashMap<String, StrategyMetadata>>>,
}

impl StrategyRegistry {
    pub fn new() -> Self {
        StrategyRegistry {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&self, strategy: Arc<dyn Strategy + MetadataStrategy>) -> Result<()> {
        let name = strategy.name().to_string();
        let metadata = strategy.metadata();

        // Validate metadata
        if metadata.name != name {
            return Err(QuantError::Validation(
                "Strategy name mismatch".to_string()
            ));
        }

        {
            let mut strategies = self.strategies.write()
                .map_err(|e| QuantError::Internal(e.to_string()))?;
            let mut metadata_map = self.metadata.write()
                .map_err(|e| QuantError::Internal(e.to_string()))?;

            strategies.insert(name.clone(), strategy);
            metadata_map.insert(name, metadata);
        }

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Strategy + MetadataStrategy>> {
        self.strategies.read().ok()?.get(name).cloned()
    }

    pub fn get_metadata(&self, name: &str) -> Option<StrategyMetadata> {
        self.metadata.read().ok()?.get(name).cloned()
    }

    pub fn list_all(&self) -> Vec<String> {
        self.metadata.read()
            .ok()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn list_by_category(&self, category: StrategyCategory) -> Vec<String> {
        self.metadata.read()
            .ok()
            .map(|m| {
                m.iter()
                    .filter(|(_, meta)| meta.category == category)
                    .map(|(name, _)| name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_for_regime(&self, regime: MarketRegime) -> Vec<String> {
        self.metadata.read()
            .ok()
            .map(|m| {
                m.iter()
                    .filter(|(_, meta)| meta.expected_regimes.contains(&regime))
                    .map(|(name, _)| name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

**Note**: This assumes `Strategy` trait is in `crate::core`. You may need to import it from the correct location.

**Validation**:
- [ ] `framework.rs` created
- [ ] All types compile
- [ ] `StrategyRegistry` works correctly
- [ ] `StrategyClassifier` works correctly
- [ ] All traits are properly implemented
- [ ] Error handling is comprehensive

**Acceptance Criteria**:
- Code compiles without warnings
- Unit tests pass for all components
- Registry can register and retrieve strategies
- Classifier produces sensible classifications

---

### Task 12.1.4: Implement Baseline Strategies (2 days)

**Objective**: Implement baseline strategies (HODL, Market Average) for comparison purposes.

**File to Create**: `crates/strategy/src/baseline.rs`

**Instructions**:
1. Implement `HoldBaseline` strategy
2. Implement `MarketAverageBaseline` strategy
3. Both must implement `Strategy` and `MetadataStrategy` traits
4. Include comprehensive unit tests
5. Add strategies to module exports

**Code Structure**:

```rust
use crate::core::{Bar, Trade, Signal, SignalType, Strategy, QuantError};
use crate::framework::{
    MetadataStrategy, StrategyMetadata, StrategyCategory, MarketRegime,
    RiskProfile, VolatilityLevel, CorrelationSensitivity
};
use serde::{Serialize, Deserialize};

// ============ HODL Baseline ============

pub struct HoldBaseline {
    entry_price: Option<f64>,
    entered: bool,
}

impl HoldBaseline {
    pub fn new() -> Self {
        HoldBaseline {
            entry_price: None,
            entered: false,
        }
    }
}

impl Default for HoldBaseline {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for HoldBaseline {
    fn name(&self) -> &str {
        "HODL_Baseline"
    }

    fn on_bar(&mut self, bar: &Bar) -> Result<Option<Signal>> {
        if !self.entered {
            self.entry_price = Some(bar.close);
            self.entered = true;
            
            Ok(Some(Signal {
                timestamp: bar.timestamp,
                strategy: self.name().to_string(),
                signal_type: SignalType::LongEntry,
                price: bar.close,
            }))
        } else {
            Ok(None)
        }
    }

    fn on_tick(&mut self, _tick: &Trade) -> Result<Option<Signal>> {
        Ok(None)
    }

    fn reset(&mut self) {
        self.entry_price = None;
        self.entered = false;
    }
}

impl MetadataStrategy for HoldBaseline {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "HODL_Baseline".to_string(),
            category: StrategyCategory::Baseline,
            sub_type: Some("buy_and_hold".to_string()),
            description: "Simple buy and hold strategy for baseline comparison. Buys on first bar and holds indefinitely.".to_string(),
            hypothesis_path: "hypotheses/baseline/hodl.md".to_string(),
            required_indicators: vec![],
            expected_regimes: vec![MarketRegime::Bull],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.50, // 50% drawdown potential
                volatility_level: VolatilityLevel::High,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::Baseline
    }
}

// ============ Market Average Baseline ============

pub struct MarketAverageBaseline {
    symbols: Vec<String>,
    weights: Vec<f64>,
}

impl MarketAverageBaseline {
    pub fn new(symbols: Vec<String>, weights: Vec<f64>) -> Self {
        MarketAverageBaseline {
            symbols,
            weights,
        }
    }

    pub fn equal_weighted(symbols: Vec<String>) -> Self {
        let n = symbols.len();
        let weights = vec![1.0 / n as f64; n];
        Self::new(symbols, weights)
    }
}

impl Strategy for MarketAverageBaseline {
    fn name(&self) -> &str {
        "Market_Average_Baseline"
    }

    fn on_bar(&mut self, _bar: &Bar) -> Result<Option<Signal>> {
        // Managed at portfolio level, not individual strategy level
        Ok(None)
    }

    fn on_tick(&mut self, _tick: &Trade) -> Result<Option<Signal>> {
        Ok(None)
    }

    fn reset(&mut self) {}
}

impl MetadataStrategy for MarketAverageBaseline {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Market_Average_Baseline".to_string(),
            category: StrategyCategory::Baseline,
            sub_type: Some("market_index".to_string()),
            description: "Equally weighted portfolio of all assets. Managed at portfolio level.".to_string(),
            hypothesis_path: "hypotheses/baseline/market_average.md".to_string(),
            required_indicators: vec![],
            expected_regimes: vec![MarketRegime::Bull],
            risk_profile: RiskProfile {
                max_drawdown_expected: 0.40,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::Low,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::Baseline
    }
}

// ============ Unit Tests ============

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_bar(price: f64) -> Bar {
        Bar {
            timestamp: Utc::now(),
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_hold_baseline_entry() {
        let mut strategy = HoldBaseline::new();
        let bar = create_test_bar(100.0);

        let signal = strategy.on_bar(&bar).unwrap().unwrap();
        
        assert_eq!(signal.signal_type, SignalType::LongEntry);
        assert_eq!(signal.price, 100.0);
        assert_eq!(signal.strategy, "HODL_Baseline");
    }

    #[test]
    fn test_hold_baseline_single_entry() {
        let mut strategy = HoldBaseline::new();
        
        // First bar - should generate entry
        let signal1 = strategy.on_bar(&create_test_bar(100.0)).unwrap();
        assert!(signal1.is_some());

        // Second bar - should not generate signal
        let signal2 = strategy.on_bar(&create_test_bar(101.0)).unwrap();
        assert!(signal2.is_none());
    }

    #[test]
    fn test_hold_baseline_reset() {
        let mut strategy = HoldBaseline::new();
        
        strategy.on_bar(&create_test_bar(100.0)).unwrap();
        strategy.reset();

        // After reset, should generate entry again
        let signal = strategy.on_bar(&create_test_bar(101.0)).unwrap();
        assert!(signal.is_some());
    }

    #[test]
    fn test_hold_baseline_metadata() {
        let strategy = HoldBaseline::new();
        let metadata = strategy.metadata();

        assert_eq!(metadata.name, "HODL_Baseline");
        assert_eq!(metadata.category, StrategyCategory::Baseline);
        assert_eq!(metadata.sub_type, Some("buy_and_hold".to_string()));
    }

    #[test]
    fn test_market_average_equal_weighted() {
        let symbols = vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()];
        let strategy = MarketAverageBaseline::equal_weighted(symbols);

        assert_eq!(strategy.weights, vec![1.0/3.0, 1.0/3.0, 1.0/3.0]);
    }

    #[test]
    fn test_market_average_custom_weights() {
        let symbols = vec!["BTC".to_string(), "ETH".to_string()];
        let weights = vec![0.6, 0.4];
        let strategy = MarketAverageBaseline::new(symbols, weights);

        assert_eq!(strategy.weights, vec![0.6, 0.4]);
    }
}
```

**Validation**:
- [ ] `baseline.rs` created
- [ ] Both baseline strategies implemented
- [ ] All tests pass
- [ ] Metadata is correct
- [ ] Strategies work in backtests

**Acceptance Criteria**:
- HODL baseline generates entry on first bar
- Market average baseline has correct weights
- Both strategies have proper metadata
- Unit tests cover all scenarios

---

### Task 12.1.5: Implement Strategy Registry and API (2 days)

**Objective**: Implement the strategy registry system and API endpoints for strategy management.

**Files to Modify**:
- `crates/strategy/src/lib.rs` - Add registry and framework exports
- `crates/dashboard/src/main.rs` - Add API endpoints for strategies

**Instructions**:
1. Update `crates/strategy/src/lib.rs` to export new modules
2. Implement API handlers in dashboard for:
   - List all strategies
   - Get strategy details
   - List strategies by category
   - Get strategies for regime
3. Add database integration for storing strategy metadata
4. Include comprehensive error handling

**Code Structure for `crates/strategy/src/lib.rs`**:

```rust
pub mod framework;
pub mod indicators;
pub mod baseline;
pub mod registry;
pub mod validation;

pub use framework::{
    StrategyMetadata, StrategyCategory, MarketRegime, RiskProfile,
    VolatilityLevel, CorrelationSensitivity, MetadataStrategy,
    StrategyClassification, StrategyCharacteristics, TimeHorizon, SignalFrequency,
    StrategyRegistry, StrategyClassifier
};

pub use baseline::{HoldBaseline, MarketAverageBaseline};

// Re-export from core
pub use alphafield_core::{Bar, Trade, Signal, SignalType, Strategy, QuantError};
```

**Code Structure for API Handlers (add to dashboard)**:

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use alphafield_strategy::{
    StrategyRegistry, StrategyMetadata, StrategyCategory, MarketRegime,
};

// ============ API Types ============

#[derive(Debug, Serialize)]
pub struct StrategySummary {
    pub name: String,
    pub category: String,
    pub sub_type: Option<String>,
    pub description: String,
    pub expected_regimes: Vec<String>,
    pub risk_profile: RiskProfileSummary,
}

#[derive(Debug, Serialize)]
pub struct RiskProfileSummary {
    pub max_drawdown_expected: f64,
    pub volatility_level: String,
    pub correlation_sensitivity: String,
    pub leverage_requirement: f64,
}

#[derive(Debug, Deserialize)]
pub struct StrategyQuery {
    pub category: Option<String>,
    pub regime: Option<String>,
}

// ============ Handlers ============

pub async fn list_strategies(
    State(registry): State<Arc<StrategyRegistry>>,
    Query(query): Query<StrategyQuery>,
) -> Result<Json<Vec<StrategySummary>>, AppError> {
    let strategy_names = if let Some(category_str) = query.category {
        let category = parse_strategy_category(&category_str)?;
        registry.list_by_category(category)
    } else if let Some(regime_str) = query.regime {
        let regime = parse_market_regime(&regime_str)?;
        registry.get_for_regime(regime)
    } else {
        registry.list_all()
    };

    let mut summaries = Vec::new();
    for name in strategy_names {
        if let Some(metadata) = registry.get_metadata(&name) {
            summaries.push(StrategySummary {
                name: metadata.name.clone(),
                category: format!("{:?}", metadata.category),
                sub_type: metadata.sub_type,
                description: metadata.description,
                expected_regimes: metadata.expected_regimes
                    .iter()
                    .map(|r| format!("{:?}", r))
                    .collect(),
                risk_profile: RiskProfileSummary {
                    max_drawdown_expected: metadata.risk_profile.max_drawdown_expected,
                    volatility_level: format!("{:?}", metadata.risk_profile.volatility_level),
                    correlation_sensitivity: format!("{:?}", metadata.risk_profile.correlation_sensitivity),
                    leverage_requirement: metadata.risk_profile.leverage_requirement,
                },
            });
        }
    }

    Ok(Json(summaries))
}

pub async fn get_strategy_details(
    State(registry): State<Arc<StrategyRegistry>>,
    Path(name): Path<String>,
) -> Result<Json<StrategyMetadata>, AppError> {
    let metadata = registry
        .get_metadata(&name)
        .ok_or_else(|| AppError::NotFound(format!("Strategy '{}' not found", name)))?;

    Ok(Json(metadata))
}

// ============ Helper Functions ============

fn parse_strategy_category(s: &str) -> Result<StrategyCategory, AppError> {
    match s.to_lowercase().as_str() {
        "trendfollowing" | "trend" => Ok(StrategyCategory::TrendFollowing),
        "meanreversion" | "mean" => Ok(StrategyCategory::MeanReversion),
        "momentum" => Ok(StrategyCategory::Momentum),
        "volatilitybased" | "volatility" => Ok(StrategyCategory::VolatilityBased),
        "sentimentbased" | "sentiment" => Ok(StrategyCategory::SentimentBased),
        "multiindicator" | "multi" => Ok(StrategyCategory::MultiIndicator),
        "baseline" => Ok(StrategyCategory::Baseline),
        _ => Err(AppError::BadRequest(format!("Invalid category: {}", s))),
    }
}

fn parse_market_regime(s: &str) -> Result<MarketRegime, AppError> {
    match s.to_lowercase().as_str() {
        "bull" => Ok(MarketRegime::Bull),
        "bear" => Ok(MarketRegime::Bear),
        "sideways" => Ok(MarketRegime::Sideways),
        "highvolatility" => Ok(MarketRegime::HighVolatility),
        "lowvolatility" => Ok(MarketRegime::LowVolatility),
        "trending" => Ok(MarketRegime::Trending),
        "ranging" => Ok(MarketRegime::Ranging),
        _ => Err(AppError::BadRequest(format!("Invalid regime: {}", s))),
    }
}

// ============ Error Types ============

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

**Validation**:
- [ ] Module exports updated in lib.rs
- [ ] API endpoints implemented
- [ ] Error handling comprehensive
- [ ] API tests pass
- [ ] Database integration works

**Acceptance Criteria**:
- Can list all strategies via API
- Can get strategy details via API
- Can filter by category and regime
- Error responses are clear and helpful
- Tests cover all endpoints

---

### Task 12.1.6: Create Strategy Template for AI Agents (0.5 days)

**Objective**: Create a comprehensive template that AI agents can use to implement new strategies.

**File to Create**: `doc/phase_12/strategy_template.md`

**Instructions**:
1. Create a detailed template for implementing strategies
2. Include code templates for all required components
3. Provide clear instructions for each section
4. Include validation checklists
5. Provide examples of common patterns

**Template Content**:

```markdown
# AI Agent Strategy Implementation Template

## Overview
This template provides a standardized structure for implementing trading strategies in AlphaField. AI agents should follow this template exactly to ensure consistency and quality.

## Required Components

Every strategy implementation MUST include:

1. **Strategy Struct**: Holds strategy state and parameters
2. **Strategy Trait Implementation**: Implements `Strategy` and `MetadataStrategy`
3. **Constructor**: Creates strategy instances with validation
4. **Signal Generation**: Implements `on_bar` and `on_tick` methods
5. **Unit Tests**: Tests for all major functionality
6. **Hypothesis Document**: Complete hypothesis documentation
7. **Module Export**: Added to category module exports

---

## Code Template

```rust
// File: crates/strategy/src/strategies/[category]/[strategy_name].rs

use crate::core::{Bar, Trade, Signal, SignalType, Strategy, QuantError};
use crate::framework::{
    MetadataStrategy, StrategyMetadata, StrategyCategory, MarketRegime,
    RiskProfile, VolatilityLevel, CorrelationSensitivity
};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

// ============ Strategy Struct ============

/// [Brief description of what this strategy does]
pub struct [StrategyName] {
    // Required parameters
    [param1_type] [param1_name],  // [Description]
    [param2_type] [param2_name],  // [Description]
    
    // State variables
    [state1_type] [state1_name],  // [Description]
    [state2_type] [state2_name],  // [Description]
}

// ============ Constructor ============

impl [StrategyName] {
    /// Creates a new [StrategyName] strategy instance
    /// 
    /// # Arguments
    /// * `[param1_name]` - [Description]
    /// * `[param2_name]` - [Description]
    /// 
    /// # Returns
    /// * `Result<Self>` - New strategy instance or error
    /// 
    /// # Errors
    /// Returns error if parameters are invalid
    pub fn new([param1_name]: [param1_type], [param2_name]: [param2_type]) -> Result<Self> {
        // Validate parameters
        if [validation_condition] {
            return Err(QuantError::Validation(
                "[Error message]".to_string()
            ));
        }

        Ok([StrategyName] {
            [param1_name],
            [param2_name],
            
            // Initialize state
            [state1_name]: [initial_value],
            [state2_name]: [initial_value],
        })
    }
}

// ============ Default Implementation ============

impl Default for [StrategyName] {
    fn default() -> Self {
        Self::new([default_param1], [default_param2])
            .expect("Default parameters should be valid")
    }
}

// ============ Strategy Trait Implementation ============

impl Strategy for [StrategyName] {
    fn name(&self) -> &str {
        "[StrategyName]"
    }

    fn on_bar(&mut self, bar: &Bar) -> Result<Option<Signal>> {
        // Update indicators/state
        [update_indicators_or_state];

        // Check entry conditions
        if [entry_condition_1] && [entry_condition_2] {
            return Ok(Some(Signal {
                timestamp: bar.timestamp,
                strategy: self.name().to_string(),
                signal_type: SignalType::LongEntry,
                price: bar.close,
            }));
        }

        // Check exit conditions
        if [exit_condition_1] || [exit_condition_2] {
            return Ok(Some(Signal {
                timestamp: bar.timestamp,
                strategy: self.name().to_string(),
                signal_type: SignalType::LongExit,
                price: bar.close,
            }));
        }

        Ok(None)
    }

    fn on_tick(&mut self, _tick: &Trade) -> Result<Option<Signal>> {
        // Most strategies don't use ticks, but can if needed
        Ok(None)
    }

    fn reset(&mut self) {
        // Reset all state variables
        [state1_name] = [initial_value];
        [state2_name] = [initial_value];
    }
}

// ============ MetadataStrategy Trait Implementation ============

impl MetadataStrategy for [StrategyName] {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "[StrategyName]".to_string(),
            category: StrategyCategory::[StrategyCategory],
            sub_type: Some("[sub_type]".to_string()),
            description: "[Full description of the strategy]".to_string(),
            hypothesis_path: "hypotheses/[category]/[strategy_name].md".to_string(),
            required_indicators: vec![
                "[indicator1]".to_string(),
                "[indicator2]".to_string(),
            ],
            expected_regimes: vec![
                MarketRegime::[Regime1],
                MarketRegime::[Regime2],
            ],
            risk_profile: RiskProfile {
                max_drawdown_expected: [value],
                volatility_level: VolatilityLevel::[Level],
                correlation_sensitivity: CorrelationSensitivity::[Sensitivity],
                leverage_requirement: [value],
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::[StrategyCategory]
    }
}

// ============ Unit Tests ============

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Helper to create test bars
    fn create_test_bar(timestamp: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Bar {
        Bar {
            timestamp: Utc.timestamp(timestamp, 0),
            open,
            high,
            low,
            close,
            volume,
        }
    }

    #[test]
    fn test_strategy_creation() {
        let strategy = [StrategyName]::new([param1_value], [param2_value]);
        assert!(strategy.is_ok());
    }

    #[test]
    fn test_invalid_parameters() {
        let strategy = [StrategyName]::new([invalid_param1], [invalid_param2]);
        assert!(strategy.is_err());
    }

    #[test]
    fn test_entry_signal_generation() {
        let mut strategy = [StrategyName]::default();
        
        // Create bars that should trigger entry
        let bar1 = create_test_bar(0, 100.0, 105.0, 95.0, 102.0, 1000.0);
        let signal1 = strategy.on_bar(&bar1).unwrap();
        assert!(signal1.is_none()); // No signal yet

        let bar2 = create_test_bar(1, 102.0, 108.0, 100.0, 106.0, 1200.0);
        let signal2 = strategy.on_bar(&bar2).unwrap();
        assert!(signal2.is_some()); // Should have entry signal
        
        if let Some(signal) = signal2 {
            assert_eq!(signal.signal_type, SignalType::LongEntry);
            assert_eq!(signal.price, 106.0);
        }
    }

    #[test]
    fn test_exit_signal_generation() {
        let mut strategy = [StrategyName]::default();
        
        // Generate entry first
        let entry_bar = create_test_bar(0, 100.0, 105.0, 95.0, 102.0, 1000.0);
        strategy.on_bar(&entry_bar).unwrap();
        
        // Create bars that should trigger exit
        let exit_bar = create_test_bar(1, 102.0, 103.0, 90.0, 91.0, 1000.0);
        let signal = strategy.on_bar(&exit_bar).unwrap();
        assert!(signal.is_some());
        
        if let Some(signal) = signal {
            assert_eq!(signal.signal_type, SignalType::LongExit);
        }
    }

    #[test]
    fn test_strategy_reset() {
        let mut strategy = [StrategyName]::default();
        
        // Generate some signals
        let bar1 = create_test_bar(0, 100.0, 105.0, 95.0, 102.0, 1000.0);
        strategy.on_bar(&bar1).unwrap();
        
        // Reset
        strategy.reset();
        
        // After reset, should behave like new
        let bar2 = create_test_bar(1, 102.0, 108.0, 100.0, 106.0, 1200.0);
        let signal = strategy.on_bar(&bar2).unwrap();
        
        // Should generate entry signal again if conditions met
        assert!([assertion_based_on_strategy_logic]);
    }

    #[test]
    fn test_metadata() {
        let strategy = [StrategyName]::default();
        let metadata = strategy.metadata();
        
        assert_eq!(metadata.name, "[StrategyName]");
        assert_eq!(metadata.category, StrategyCategory::[StrategyCategory]);
    }

    // Add more tests as needed for your specific strategy
}
```

---

## Implementation Checklist

Use this checklist to ensure your strategy is complete:

### Code Implementation
- [ ] Strategy struct with parameters and state
- [ ] Constructor with parameter validation
- [ ] `Strategy` trait implementation
- [ ] `MetadataStrategy` trait implementation
- [ ] `on_bar` method with entry/exit logic
- [ ] `on_tick` method (even if just returns None)
- [ ] `reset` method clears all state
- [ ] `Default` implementation
- [ ] Comprehensive unit tests

### Documentation
- [ ] Hypothesis document created in `doc/phase_12/hypotheses/[category]/[strategy_name].md`
- [ ] All sections of hypothesis template completed
- [ ] Code comments explain logic
- [ ] Metadata fields accurate

### Integration
- [ ] Strategy added to category module exports
- [ ] Strategy added to main module exports
- [ ] Registered in strategy registry (in main or initialization)
- [ ] API endpoint can retrieve strategy metadata

### Testing
- [ ] Unit tests cover entry logic
- [ ] Unit tests cover exit logic
- [ ] Unit tests cover edge cases
- [ ] Unit tests cover parameter validation
- [ ] Integration tests pass (backtest execution)

### Validation
- [ ] Backtest runs successfully on real data
- [ ] Walk-forward validation passes
- [ ] Monte Carlo simulation completed
- [ ] Performance metrics calculated
- [ ] Failure modes documented

---

## Common Patterns

### Indicator Window with VecDeque
```rust
use std::collections::VecDeque;

pub struct ExampleStrategy {
    window_size: usize,
    prices: VecDeque<f64>,
}

impl ExampleStrategy {
    pub fn new(window_size: usize) -> Result<Self> {
        if window_size < 2 {
            return Err(QuantError::Validation("Window size must be >= 2".to_string()));
        }
        
        Ok(ExampleStrategy {
            window_size,
            prices: VecDeque::with_capacity(window_size),
        })
    }
    
    fn calculate_sma(&self) -> Option<f64> {
        if self.prices.len() < self.window_size {
            return None;
        }
        
        let sum: f64 = self.prices.iter().sum();
        Some(sum / self.window_size as f64)
    }
}

impl Strategy for ExampleStrategy {
    fn on_bar(&mut self, bar: &Bar) -> Result<Option<Signal>> {
        // Add to window
        self.prices.push_back(bar.close);
        
        // Maintain window size
        if self.prices.len() > self.window_size {
            self.prices.pop_front();
        }
        
        // Use indicator
        if let Some(sma) = self.calculate_sma() {
            // Generate signals based on SMA
        }
        
        Ok(None)
    }
    
    fn reset(&mut self) {
        self.prices.clear();
    }
    
    // ... other trait methods
}
```

### Tracking Position State
```rust
pub struct ExampleStrategy {
    in_position: bool,
    entry_price: Option<f64>,
    entry_time: Option<DateTime<Utc>>,
}

impl Strategy for ExampleStrategy {
    fn on_bar(&mut self, bar: &Bar) -> Result<Option<Signal>> {
        if !self.in_position {
            // Check entry conditions
            if [entry_condition] {
                self.in_position = true;
                self.entry_price = Some(bar.close);
                self.entry_time = Some(bar.timestamp);
                
                return Ok(Some(Signal {
                    timestamp: bar.timestamp,
                    strategy: self.name().to_string(),
                    signal_type: SignalType::LongEntry,
                    price: bar.close,
                }));
            }
        } else {
            // Check exit conditions
            if [exit_condition] {
                self.in_position = false;
                self.entry_price = None;
                self.entry_time = None;
                
                return Ok(Some(Signal {
                    timestamp: bar.timestamp,
                    strategy: self.name().to_string(),
                    signal_type: SignalType::LongExit,
                    price: bar.close,
                }));
            }
        }
        
        Ok(None)
    }
    
    fn reset(&mut self) {
        self.in_position = false;
        self.entry_price = None;
        self.entry_time = None;
    }
    
    // ... other trait methods
}
```

### Multiple Indicator Filters
```rust
pub struct ExampleStrategy {
    rsi_period: usize,
    rsi_overbought: f64,
    rsi_oversold: f64,
    rsi_values: VecDeque<f64>,
    
    sma_short: usize,
    sma_long: usize,
    prices: VecDeque<f64>,
}

impl Strategy for ExampleStrategy {
    fn on_bar(&mut self, bar: &Bar) -> Result<Option<Signal>> {
        // Update indicators
        self.update_rsi(bar);
        self.update_sma(bar);
        
        // Get indicator values
        let rsi = self.get_current_rsi();
        let sma_short = self.get_sma_short();
        let sma_long = self.get_sma_long();
        
        // Entry conditions (all must be true)
        let entry_conditions = [
            rsi.map(|r| r < self.rsi_oversold).unwrap_or(false),      // RSI oversold
            sma_short.map(|s| s > sma_long.unwrap_or(0.0)).unwrap_or(false), // SMA bullish
            bar.volume > self.avg_volume(),                           // Volume confirmation
        ];
        
        if entry_conditions.iter().all(|&x| x) {
            return Ok(Some(entry_signal(bar)));
        }
        
        // Exit conditions (any can be true)
        let exit_conditions = [
            rsi.map(|r| r > self.rsi_overbought).unwrap_or(false),  // RSI overbought
            self.holding_duration() > Duration::days(30),             // Time stop
            self.drawdown() > 0.05,                                    // 5% stop loss
        ];
        
        if exit_conditions.iter().any(|&x| x) {
            return Ok(Some(exit_signal(bar)));
        }
        
        Ok(None)
    }
    
    // ... helper methods and other trait methods
}
```

---

## Validation Checklist

Before marking a strategy as complete, verify:

### Code Quality
- [ ] Code compiles without warnings (`cargo clippy -- -D warnings`)
- [ ] All tests pass (`cargo test`)
- [ ] Code follows Rust best practices
- [ ] Error handling is comprehensive
- [ ] State management is correct

### Documentation
- [ ] Hypothesis document is complete
- [ ] All parameters are documented
- [ ] Entry/exit rules are clear
- [ ] Failure modes are identified
- [ ] Risk profile is accurate

### Testing
- [ ] Unit tests cover all scenarios
- [ ] Edge cases are tested
- [ ] Parameter validation works
- [ ] Reset functionality works

### Performance
- [ ] Backtest runs successfully
- [ ] Performance metrics are reasonable
- [ ] Walk-forward validation passes
- [ ] No obvious overfitting

### Integration
- [ ] API endpoint works
- [ ] Database integration works
- [ ] Dashboard can display strategy
- [ ] Strategy can be backtested via API

---

## Troubleshooting

### Common Issues and Solutions

**Issue**: "Strategy generates too many signals"
- **Solution**: Add filters (volume, time, volatility), increase parameter thresholds

**Issue**: "Strategy never generates signals"
- **Solution**: Lower thresholds, remove filters, check indicator calculations

**Issue**: "Strategy performs poorly in backtest"
- **Solution**: Optimize parameters, add regime filters, adjust entry/exit rules

**Issue**: "Strategy fails walk-forward validation"
- **Solution**: Parameters are overfitted, reduce complexity, increase robustness

**Issue**: "Tests pass but backtest fails"
- **Solution**: Check state management between bars, verify indicator calculations

**Issue**: "Memory usage is high"
- **Solution**: Limit window sizes, use fixed-size arrays instead of VecDeque where possible

---

## Example: Complete Strategy Implementation

See `doc/phase_12/examples/golden_cross_complete.md` for a complete example of a strategy implementation including:
- Full Rust code
- Complete hypothesis document
- Comprehensive tests
- Backtest results
- Validation outcomes
```

**Validation**:
- [ ] Template created in correct location
- [ ] Template is comprehensive and clear
- [ ] Code templates are correct
- [ ] Examples are helpful
- [ ] Checklists are complete

**Acceptance Criteria**:
- AI agent can use this template to implement strategies
- Template covers all required components
- Template is easy to follow and understand
- Examples demonstrate best practices

---

### Task 12.1.7: Run Phase 12.1 Validation (0.5 days)

**Objective**: Validate that all Phase 12.1 components work together correctly.

**Instructions**:
1. Run full test suite: `cargo test`
2. Check for compilation warnings: `cargo clippy -- -D warnings`
3. Test API endpoints manually or with integration tests
4. Verify database migrations work
5. Test strategy registry with baseline strategies
6. Verify all documentation is in place

**Validation Checklist**:
- [x] All code compiles without warnings
- [x] All tests pass (39 tests passed)
- [ ] API endpoints work correctly (to be implemented in Task 12.1.5)
- [x] Database schema created successfully (3 migration files created)
- [x] Baseline strategies registered (HODL and Market Average implemented)
- [x] Hypothesis template created (template.md in hypotheses directory)
- [x] Strategy template created (strategy_template.md)
- [x] Framework documentation complete (framework.rs with comprehensive docs)

**Acceptance Criteria**:
- [x] Zero compilation warnings (cargo clippy -- -D warnings passes)
- [x] 100% test pass rate (39/39 tests passed)
- [ ] API endpoints return correct data (deferred to Phase 12.1.5)
- [x] Database contains required tables (strategies, strategy_performance, strategy_failures)
- [ ] Baselines can be retrieved via API (deferred to Phase 12.1.5)

---

### Phase 12.1 Summary

**Total Duration**: 2 weeks (10 working days)

**Deliverables**:
- ✅ Strategy framework with metadata and classification (framework.rs)
- ✅ Strategy registry for dynamic strategy management (StrategyRegistry)
- ✅ Baseline strategies for comparison (HoldBaseline, MarketAverageBaseline)
- ✅ Database schema for strategies, performance, and failures (3 SQL migrations)
- ✅ Hypothesis template for documentation (hypotheses/template.md)
- ✅ Strategy implementation template for AI agents (strategy_template.md)
- ⏳ API endpoints for strategy management (deferred - requires dashboard integration)

**Success Metrics**:
- ✅ Framework code compiles and tests pass (0 warnings, 39/39 tests passed)
- ✅ Registry can register and retrieve strategies (unit tests verify)
- ✅ Baselines generate expected signals (HODL on first bar, Market Average at portfolio level)
- ⏳ API endpoints work correctly (deferred to Phase 12.1.5)
- ✅ Templates are comprehensive and usable (both templates created with examples)

**Next Phase**: Phase 12.2 - Trend Following Strategies (7 strategies)

---

## 📚 Phase 12.2: Trend Following Strategies (Weeks 3-5)

### Dependencies
- Phase 12.1 complete and validated
- Database schema deployed
- Framework working correctly

### Deliverables
- 7 trend-following strategies implemented
- All strategies have complete hypothesis documents
- All strategies pass walk-forward validation
- Performance metrics stored in database
- API endpoints work for all strategies

---

### Task 12.2.1: Implement Golden Cross Strategy (1 day)

**Status**: Already exists in codebase, needs documentation and validation

**Objective**: Complete the Golden Cross strategy with full documentation and validation.

**File**: `crates/strategy/src/strategies/trend_following/golden_cross.rs`

**Instructions**:
1. Review existing implementation
2. Add parameter validation if missing
3. Ensure `MetadataStrategy` trait is implemented
4. Write comprehensive unit tests
5. Create hypothesis document
6. Run backtest and validation

**Hypothesis Document**: `doc/phase_12/hypotheses/trend_following/golden_cross.md`

**Key Hypothesis**:
"When the 50-day SMA crosses above the 200-day SMA (Golden Cross), the asset enters a sustainable medium-term uptrend. The strategy generates a long entry on the cross and exits on the opposite cross (Death Cross) or a 15% drawdown."

**Parameters**:
- `short_period`: 50 days (default)
- `long_period`: 200 days (default)
- `stop_loss_percent`: 15% (default)

**Validation Requirements**:
- Walk-forward analysis: Passing (>50 stability)
- Monte Carlo: 95% CI reasonable
- Sharpe Ratio: >1.0
- Max Drawdown: <20%

**Acceptance Criteria**:
- Strategy compiles and tests pass
- Hypothesis document complete
- Backtest generates signals as expected
- Walk-forward validation passes
- Performance stored in database

---

### Task 12.2.2: Implement Breakout Strategy (1.5 days)

**Objective**: Implement a breakout strategy using Donchian channels or ATR.

**File**: `crates/strategy/src/strategies/trend_following/breakout.rs`

**Instructions**:
1. Use strategy template from Task 12.1.6
2. Implement Donchian channel breakout (highest high / lowest low)
3. Add ATR filter for volatility
4. Add volume confirmation
5. Write comprehensive tests
6. Create hypothesis document
7. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/trend_following/breakout.md`

**Key Hypothesis**:
"When price breaks above the highest high of the last N periods, confirmed by high volume and expanding ATR, a new uptrend is beginning. The strategy enters long on the breakout and exits on a trailing ATR stop or opposite breakout."

**Parameters**:
- `lookback_period`: 20 periods (default)
- `atr_period`: 14 periods (default)
- `atr_multiplier`: 2.0 (default)
- `volume_multiplier`: 1.5x average (default)

**Entry Rules**:
1. Price > highest high of lookback period
2. Volume > volume_multiplier * average volume
3. ATR expanding (ATR > 1.2x average ATR)

**Exit Rules**:
1. Price < lowest low of lookback period (reversal)
2. Price < entry_price - atr_multiplier * ATR (trailing stop)
3. Time stop: 30 bars if no exit

**Acceptance Criteria**:
- Strategy implemented per template
- Tests cover all scenarios
- Hypothesis document complete
- Validation passes all checks
- Performance stored in database

---

### Task 12.2.3: Implement Moving Average Crossover (1 day)

**Objective**: Implement a configurable MA crossover strategy.

**File**: `crates/strategy/src/strategies/trend_following/ma_crossover.rs`

**Instructions**:
1. Implement generic MA types (SMA, EMA)
2. Allow configurable periods
3. Add trend strength filter (distance between MAs)
4. Write comprehensive tests
5. Create hypothesis document
6. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/trend_following/ma_crossover.md`

**Key Hypothesis**:
"When a faster moving average crosses above a slower moving average, confirmed by sufficient separation, a trend change has occurred. The strategy enters on the crossover and exits on the opposite crossover or when MAs converge."

**Parameters**:
- `fast_period`: 10 periods (default)
- `slow_period`: 30 periods (default)
- `ma_type`: SMA or EMA (default: EMA)
- `min_separation_percent`: 1% (default)

**Entry Rules**:
1. Fast MA crosses above slow MA
2. Separation between MAs > min_separation_percent
3. Optional: Volume confirmation

**Exit Rules**:
1. Fast MA crosses below slow MA
2. MA separation < min_separation_percent (convergence)
3. Stop loss: 5% below entry

**Acceptance Criteria**:
- Generic implementation supports SMA/EMA
- Configurable periods
- Trend strength filter works
- Validation passes
- Hypothesis complete

---

### Task 12.2.4: Implement Adaptive Moving Average (1.5 days)

**Objective**: Implement an adaptive MA strategy (Kaufman's Adaptive Moving Average).

**File**: `crates/strategy/src/strategies/trend_following/adaptive_ma.rs`

**Instructions**:
1. Implement Kaufman's Adaptive Moving Average (KAMA)
2. Use efficiency ratio for adaptation
3. Signal when price crosses KAMA
4. Write comprehensive tests
5. Create hypothesis document
6. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/trend_following/adaptive_ma.md`

**Key Hypothesis**:
"An adaptive moving average that adjusts its speed based on market efficiency provides better trend signals than fixed-period MAs. The strategy enters when price crosses above KAMA and exits when price crosses below."

**Parameters**:
- `er_period`: 10 periods (efficiency ratio)
- `fast_sc`: 2 (fastest smoothing constant)
- `slow_sc`: 30 (slowest smoothing constant)

**KAMA Formula**:
1. Calculate efficiency ratio: ER = Direction / Volatility
2. Calculate smoothing constant: SC = ER * (fast_sc - slow_sc) + slow_sc
3. Calculate KAMA: KAMA = prior_KAMA + SC^2 * (price - prior_KAMA)

**Entry/Exit Rules**:
1. Long entry: Price crosses above KAMA
2. Long exit: Price crosses below KAMA
3. Stop loss: 3% ATR-based

**Acceptance Criteria**:
- KAMA calculation correct
- Adaptive behavior visible in different markets
- Validation passes
- Hypothesis complete

---

### Task 12.2.5: Implement Triple Moving Average System (1 day)

**Objective**: Implement a triple MA system (fast, medium, slow).

**File**: `crates/strategy/src/strategies/trend_following/triple_ma.rs`

**Instructions**:
1. Implement three MAs (e.g., 5, 15, 30)
2. Generate entry when all aligned (fast > medium > slow)
3. Generate exit when fast < medium
4. Write comprehensive tests
5. Create hypothesis document
6. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/trend_following/triple_ma.md`

**Key Hypothesis**:
"When three moving averages are properly aligned (fast > medium > slow), the trend is strongly established and likely to continue. The strategy enters on alignment and exits when the fastest MA crosses below the medium MA."

**Parameters**:
- `fast_period`: 5 periods (default)
- `medium_period`: 15 periods (default)
- `slow_period`: 30 periods (default)

**Entry Rules**:
1. Fast MA > Medium MA
2. Medium MA > Slow MA
3. All MAs are rising (slope > 0)

**Exit Rules**:
1. Fast MA crosses below Medium MA
2. Fast MA turns negative (slope < 0)
3. Stop loss: 5% below entry

**Acceptance Criteria**:
- Triple alignment logic correct
- Slope calculation correct
- Validation passes
- Hypothesis complete

---

### Task 12.2.6: Implement MACD Trend Following (1 day)

**Objective**: Implement MACD-based trend following strategy.

**File**: `crates/strategy/src/strategies/trend_following/macd_trend.rs`

**Instructions**:
1. Use existing MACD indicator
2. Enter on MACD line crossing above signal line
3. Exit on MACD line crossing below signal line
4. Add histogram confirmation
5. Write comprehensive tests
6. Create hypothesis document
7. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/trend_following/macd_trend.md`

**Key Hypothesis**:
"The MACD indicator provides reliable trend signals when used with appropriate parameters. The strategy enters on bullish MACD crossovers and exits on bearish crossovers, with histogram confirmation to avoid whipsaws."

**Parameters**:
- `fast_period`: 12 periods (default)
- `slow_period`: 26 periods (default)
- `signal_period`: 9 periods (default)
- `histogram_threshold`: 0 (default)

**Entry Rules**:
1. MACD line crosses above signal line
2. MACD histogram > histogram_threshold
3. Optional: Price above 200 SMA (trend filter)

**Exit Rules**:
1. MACD line crosses below signal line
2. MACD histogram < 0
3. Stop loss: 3% ATR-based

**Acceptance Criteria**:
- MACD calculations correct
- Crossover logic correct
- Histogram filter works
- Validation passes
- Hypothesis complete

---

### Task 12.2.7: Implement Parabolic SAR Strategy (1 day)

**Objective**: Implement a strategy using Parabolic SAR for trend following.

**File**: `crates/strategy/src/strategies/trend_following/parabolic_sar.rs`

**Instructions**:
1. Implement Parabolic SAR calculation
2. Use SAR as trailing stop
3. Enter when price crosses SAR
4. Write comprehensive tests
5. Create hypothesis document
6. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/trend_following/parabolic_sar.md`

**Key Hypothesis**:
"Parabolic SAR provides effective trailing stops for trend following. The strategy enters long when price crosses above SAR (which is below price) and uses SAR as a dynamic trailing stop to exit."

**Parameters**:
- `step`: 0.02 (default)
- `max_step`: 0.2 (default)
- `trend_filter`: True (default - require 50 SMA confirmation)

**Entry Rules**:
1. Price crosses above SAR (SAR moves below price)
2. (Optional) Price > 50 SMA (trend filter)

**Exit Rules**:
1. Price crosses below SAR (SAR moves above price)
2. SAR acts as trailing stop

**Parabolic SAR Calculation**:
- Start with SAR = prior SAR + step * (extreme_point - prior_SAR)
- Switch direction when price crosses SAR
- Use extreme point (highest high or lowest low)

**Acceptance Criteria**:
- SAR calculation correct
- Direction switching works
- Trailing stop behavior correct
- Validation passes
- Hypothesis complete

---

### Task 12.2.8: Trend Following Module Integration (0.5 days)

**Objective**: Integrate all trend following strategies into the system.

**Instructions**:
1. Update `crates/strategy/src/strategies/trend_following/mod.rs`
2. Export all strategies
3. Register all strategies in main initialization
4. Update API tests
5. Run category-level validation

**Module Export**:
```rust
pub mod golden_cross;
pub mod breakout;
pub mod ma_crossover;
pub mod adaptive_ma;
pub mod triple_ma;
pub mod macd_trend;
pub mod parabolic_sar;

pub use golden_cross::GoldenCross;
pub use breakout::BreakoutStrategy;
pub use ma_crossover::MACrossover;
pub use adaptive_ma::AdaptiveMAStrategy;
pub use triple_ma::TripleMAStrategy;
pub use macd_trend::MacdTrendStrategy;
pub use parabolic_sar::ParabolicSARStrategy;
```

**Validation Checklist**:
- [ ] All 7 strategies exported
- [ ] All strategies registered in registry
- [ ] API can list all trend strategies
- [ ] API can get details for each strategy
- [ ] All strategies compile
- [ ] All tests pass
- [ ] Walk-forward validation results compiled

**Acceptance Criteria**:
- Module exports correct
- Registry contains all 7 strategies
- API works for all strategies
- All validation metrics stored in database

---

### Task 12.2.9: Run Phase 12.2 Validation (1 day)

**Objective**: Validate all trend following strategies comprehensively.

**Instructions**:
1. Run walk-forward analysis on all strategies
2. Run Monte Carlo simulation on all strategies
3. Compile performance comparison report
4. Generate strategy ranking
5. Document findings
6. HUMAN REVIEW REQUIRED

**Validation Report Template**:

```markdown
# Trend Following Strategies Validation Report

## Summary
- **Total Strategies**: 7
- **Date**: [Date]
- **Test Period**: [Date Range]
- **Test Assets**: [List]

## Overall Results

| Strategy | Sharpe | Max DD | Win Rate | Robustness | WFA Status |
|----------|--------|--------|----------|------------|------------|
| Golden Cross | | | | | |
| Breakout | | | | | |
| MA Crossover | | | | | |
| Adaptive MA | | | | | |
| Triple MA | | | | | |
| MACD Trend | | | | | |
| Parabolic SAR | | | | | |

## Per-Strategy Details

### Golden Cross
**Hypothesis**: [Summary]
**Validation Status**: [Pass/Fail]
**Performance**: [Metrics]
**Strengths**: [List]
**Weaknesses**: [List]
**Regimes**: [Bull/Bear/Sideways performance]
**Recommendation**: [Deploy/Reject/Improve]

[Repeat for each strategy]

## Comparative Analysis

### Best Performing
1. [Strategy] - Sharpe: [Value], Max DD: [Value]%
2. [Strategy] - Sharpe: [Value], Max DD: [Value]%
3. [Strategy] - Sharpe: [Value], Max DD: [Value]%

### Most Robust
1. [Strategy] - Robustness: [Value]
2. [Strategy] - Robustness: [Value]
3. [Strategy] - Robustness: [Value]

### Best in Bull Markets
1. [Strategy] - Return: [Value]%
2. [Strategy] - Return: [Value]%

### Best in Bear Markets
1. [Strategy] - Return: [Value]% (least negative)
2. [Strategy] - Return: [Value]%

### Best in Sideways Markets
1. [Strategy] - Return: [Value]%
2. [Strategy] - Return: [Value]%

## Key Findings

### What Works
- [Observation 1]
- [Observation 2]

### What Doesn't
- [Observation 1]
- [Observation 2]

### Regime Dependencies
- [How strategies perform in different regimes]
- [Which strategies work best in which regimes]

### Parameter Sensitivity
- [Which strategies are most sensitive to parameters]
- [Which strategies are most robust]

## Recommendations

### For Deployment
1. [Strategy] - [Reason]
2. [Strategy] - [Reason]

### For Further Development
1. [Strategy] - [Reason]
2. [Strategy] - [Reason]

### To Reject
1. [Strategy] - [Reason]

## Next Steps
[What to do based on findings]
```

**Acceptance Criteria**:
- All strategies validated
- Walk-forward results compiled
- Monte Carlo results compiled
- Comparison report generated
- Human review completed
- Recommendations documented

---

### Phase 12.2 Summary

**Total Duration**: 3 weeks (15 working days)

**Deliverables**:
- ✅ 7 trend-following strategies implemented
- ✅ All strategies have complete hypothesis documents
- ✅ All strategies pass unit tests
- ✅ All strategies validated with walk-forward and Monte Carlo
- ✅ Performance metrics stored in database
- ✅ API endpoints work for all strategies
- ✅ Validation report generated and reviewed

**Success Metrics**:
- 7 strategies complete
- Average Sharpe >1.0
- Average Max DD <25%
- 90% walk-forward pass rate
- 100% documentation coverage

**Next Phase**: Phase 12.3 - Mean Reversion Strategies (7 strategies)

---

## 📚 Phase 12.3: Mean Reversion Strategies (Weeks 6-8)

### Dependencies
- Phase 12.2 complete and validated
- Trend following strategies deployed
- Validation report approved

### Deliverables
- 7 mean-reversion strategies implemented
- All strategies have complete hypothesis documents
- All strategies pass walk-forward validation
- Performance metrics stored in database

---

### Task 12.3.1: Implement Bollinger Bands Reversion (1 day)

**Status**: Already exists in codebase, needs documentation and validation

**Objective**: Complete the Bollinger Bands reversion strategy.

**File**: `crates/strategy/src/strategies/mean_reversion/bollinger_bands.rs`

**Instructions**:
1. Review existing implementation
2. Add RSI filter to avoid catching falling knives
3. Add volatility filter (avoid bands in high vol)
4. Write comprehensive tests
5. Create hypothesis document
6. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/mean_reversion/bollinger_bands.md`

**Key Hypothesis**:
"Prices that move 2+ standard deviations from the mean tend to revert. The strategy enters long when price touches or crosses below the lower band (confirmed by oversold RSI) and exits when price returns to the mean or crosses above the upper band."

**Parameters**:
- `period`: 20 periods (default)
- `std_dev_multiplier`: 2.0 (default)
- `rsi_period`: 14 periods (default)
- `rsi_oversold`: 30 (default)
- `rsi_overbought`: 70 (default)

**Entry Rules**:
1. Price <= lower Bollinger Band
2. RSI <= rsi_oversold (confirmation)
3. Volatility not extreme (ATR < 3x average)

**Exit Rules**:
1. Price >= middle band (SMA)
2. Price >= upper band (profit target)
3. RSI >= rsi_overbought (exit early)
4. Stop loss: 3% below entry

**Acceptance Criteria**:
- RSI filter added
- Volatility filter added
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.3.2: Implement RSI Reversion (1 day)

**Objective**: Implement a pure RSI-based mean reversion strategy.

**File**: `crates/strategy/src/strategies/mean_reversion/rsi_reversion.rs`

**Instructions**:
1. Use existing RSI indicator
2. Enter on oversold RSI
3. Exit on overbought RSI or mean reversion
4. Add trend filter (don't fight trends)
5. Write comprehensive tests
6. Create hypothesis document
7. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/mean_reversion/rsi_reversion.md`

**Key Hypothesis**:
"RSI provides reliable oversold/overbought signals for mean reversion. The strategy enters long when RSI becomes oversold in a sideways market and exits when RSI returns to neutral or becomes overbought."

**Parameters**:
- `rsi_period`: 14 periods (default)
- `oversold_threshold`: 30 (default)
- `overbought_threshold`: 70 (default)
- `exit_threshold`: 50 (default - neutral RSI)
- `trend_filter`: True (default - avoid strong trends)

**Entry Rules**:
1. RSI <= oversold_threshold
2. Price not in strong downtrend (price > 200 SMA)
3. Optional: Volume confirmation

**Exit Rules**:
1. RSI >= exit_threshold (50)
2. RSI >= overbought_threshold (70)
3. Stop loss: 5% below entry

**Acceptance Criteria**:
- Trend filter implemented
- Exit thresholds configurable
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.3.3: Implement Statistical Arbitrage / Pairs Trading (2 days)

**Objective**: Implement a pairs trading strategy for correlated assets.

**File**: `crates/strategy/src/strategies/mean_reversion/stat_arb.rs`

**Instructions**:
1. Implement correlation calculation
2. Implement spread calculation (z-score)
3. Enter when spread deviates from mean
4. Exit when spread returns to mean
5. Write comprehensive tests
6. Create hypothesis document
7. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/mean_reversion/stat_arb.md`

**Key Hypothesis**:
"Highly correlated assets tend to maintain a stable spread relationship. When the spread diverges significantly (z-score > 2), it will revert. The strategy enters when spread is overextended and exits when it returns to normal."

**Parameters**:
- `asset1`: First asset symbol (e.g., BTC)
- `asset2`: Second asset symbol (e.g., ETH)
- `lookback_period`: 30 periods (default - for correlation)
- `entry_zscore`: 2.0 (default)
- `exit_zscore`: 0.0 (default)
- `min_correlation`: 0.8 (default - require correlation)

**Spread Calculation**:
1. Calculate correlation over lookback_period
2. If correlation < min_correlation, skip signal
3. Calculate spread: spread = price1 - hedge_ratio * price2
4. Calculate z-score: z = (spread - mean) / std_dev

**Entry Rules**:
1. Correlation >= min_correlation
2. Z-score >= entry_zscore (long asset1, short asset2) OR Z-score <= -entry_zscore (short asset1, long asset2)
3. Both assets have sufficient liquidity

**Exit Rules**:
1. Z-score returns to exit_zscore (0)
2. Time stop: 10 bars
3. Stop loss: Z-score moves further to 3.0

**Note**: For spot-only implementation, this may need to be adapted to single-asset mean reversion with correlation filtering.

**Acceptance Criteria**:
- Correlation calculation correct
- Z-score calculation correct
- Entry/exit logic correct
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.3.4: Implement Stochastic Reversion (1 day)

**Objective**: Implement a stochastic oscillator-based reversion strategy.

**File**: `crates/strategy/src/strategies/mean_reversion/stoch_reversion.rs`

**Instructions**:
1. Use existing Stochastic indicator
2. Enter on oversold stochastic
3. Exit on overbought stochastic or crossover
4. Add trend filter
5. Write comprehensive tests
6. Create hypothesis document
7. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/mean_reversion/stoch_reversion.md`

**Key Hypothesis**:
"The Stochastic oscillator provides reliable mean reversion signals. The strategy enters long when %K drops below 20 (oversold) and exits when %K rises above 80 (overbought) or when %K crosses below %D."

**Parameters**:
- `k_period`: 14 periods (default)
- `d_period`: 3 periods (default)
- `oversold`: 20 (default)
- `overbought`: 80 (default)
- `smooth_period`: 3 periods (default)

**Entry Rules**:
1. %K <= oversold (20)
2. Price not in strong downtrend
3. Optional: %K crosses above %D confirmation

**Exit Rules**:
1. %K >= overbought (80)
2. %K crosses below %D (bearish crossover)
3. Stop loss: 5% below entry

**Acceptance Criteria**:
- Stochastic calculations correct
- Crossover logic correct
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.3.5: Implement Keltner Channel Reversion (1 day)

**Objective**: Implement a Keltner channel-based reversion strategy.

**File**: `crates/strategy/src/strategies/mean_reversion/keltner_reversion.rs`

**Instructions**:
1. Implement Keltner channels (EMA + ATR)
2. Enter on lower band touch
3. Exit on middle band or upper band
4. Add volume confirmation
5. Write comprehensive tests
6. Create hypothesis document
7. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/mean_reversion/keltner_reversion.md`

**Key Hypothesis**:
"Keltner channels based on ATR provide better mean reversion signals than standard deviation-based channels. The strategy enters long when price touches the lower channel (confirmed by high volume) and exits when price returns to the EMA."

**Parameters**:
- `ema_period`: 20 periods (default)
- `atr_period`: 10 periods (default)
- `atr_multiplier`: 2.0 (default)
- `volume_multiplier`: 1.5x (default)

**Keltner Channel Calculation**:
- Middle band: EMA(period)
- Upper band: EMA + atr_multiplier * ATR
- Lower band: EMA - atr_multiplier * ATR

**Entry Rules**:
1. Price <= lower band
2. Volume >= volume_multiplier * average volume
3. Price not in strong downtrend

**Exit Rules**:
1. Price >= middle band (EMA)
2. Price >= upper band (profit target)
3. Stop loss: 3% below entry

**Acceptance Criteria**:
- Keltner channel calculation correct
- Volume filter works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.3.6: Implement Price Channel Reversion (1 day)

**Objective**: Implement a simple price channel (Donchian) reversion strategy.

**File**: `crates/strategy/src/strategies/mean_reversion/price_channel.rs`

**Instructions**:
1. Use highest high / lowest low over lookback
2. Enter on lowest low touch (oversold)
3. Exit on middle of channel or highest high
4. Add trend filter
5. Write comprehensive tests
6. Create hypothesis document
7. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/mean_reversion/price_channel.md`

**Key Hypothesis**:
"When price reaches the lowest low of N periods, it's likely oversold and will revert. The strategy enters long at the lowest low and exits when price returns to the middle of the channel (average of high and low)."

**Parameters**:
- `lookback_period`: 20 periods (default)
- `exit_percent`: 50% (default - exit at middle of channel)

**Channel Calculation**:
- Highest high: max(high, lookback_period)
- Lowest low: min(low, lookback_period)
- Middle: (highest high + lowest low) / 2

**Entry Rules**:
1. Price <= lowest low
2. Price not in strong downtrend
3. Optional: RSI confirmation

**Exit Rules**:
1. Price >= middle of channel
2. Price >= highest high (profit target)
3. Stop loss: 3% below entry

**Acceptance Criteria**:
- Channel calculation correct
- Exit logic correct
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.3.7: Implement Z-Score Reversion (1.5 days)

**Objective**: Implement a statistical z-score reversion strategy.

**File**: `crates/strategy/src/strategies/mean_reversion/zscore_reversion.rs`

**Instructions**:
1. Calculate rolling mean and standard deviation
2. Calculate z-score of price
3. Enter on extreme negative z-score
4. Exit when z-score returns to neutral
5. Write comprehensive tests
6. Create hypothesis document
7. Run validation

**Hypothesis Document**: `doc/phase_12/hypotheses/mean_reversion/zscore_reversion.md`

**Key Hypothesis**:
"Prices more than 2 standard deviations from the mean are statistically rare and tend to revert. The strategy enters long when z-score <= -2 (2 standard deviations below mean) and exits when z-score returns to 0 (mean) or becomes positive."

**Parameters**:
- `lookback_period`: 20 periods (default)
- `entry_zscore`: -2.0 (default)
- `exit_zscore`: 0.0 (default)
- `min_price_change`: 1% (default - avoid flat markets)

**Z-Score Calculation**:
1. Calculate rolling mean: mean = average(close, lookback_period)
2. Calculate rolling std_dev: std = stdev(close, lookback_period)
3. Calculate z-score: z = (close - mean) / std

**Entry Rules**:
1. Z-score <= entry_zscore (-2.0)
2. Price change over lookback > min_price_change (avoid flat markets)
3. Price not in strong downtrend

**Exit Rules**:
1. Z-score >= exit_zscore (0.0)
2. Z-score >= 1.0 (profit target)
3. Stop loss: 3% below entry

**Acceptance Criteria**:
- Z-score calculation correct
- Rolling statistics correct
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.3.8: Mean Reversion Module Integration (0.5 days)

**Objective**: Integrate all mean reversion strategies into the system.

**Instructions**:
1. Update `crates/strategy/src/strategies/mean_reversion/mod.rs`
2. Export all strategies
3. Register all strategies in registry
4. Update API tests
5. Run category-level validation

**Module Export**:
```rust
pub mod bollinger_bands;
pub mod rsi_reversion;
pub mod stat_arb;
pub mod stoch_reversion;
pub mod keltner_reversion;
pub mod price_channel;
pub mod zscore_reversion;

pub use bollinger_bands::BollingerBandsStrategy;
pub use rsi_reversion::RSIReversionStrategy;
pub use stat_arb::StatArbStrategy;
pub use stoch_reversion::StochReversionStrategy;
pub use keltner_reversion::KeltnerReversionStrategy;
pub use price_channel::PriceChannelStrategy;
pub use zscore_reversion::ZScoreReversionStrategy;
```

**Validation Checklist**:
- [ ] All 7 strategies exported
- [ ] All strategies registered
- [ ] API can list all mean reversion strategies
- [ ] All strategies compile
- [ ] All tests pass
- [ ] Walk-forward results compiled

**Acceptance Criteria**:
- Module exports correct
- Registry contains all 7 strategies
- API works for all strategies
- Validation metrics stored

---

### Task 12.3.9: Run Phase 12.3 Validation (1 day)

**Objective**: Validate all mean reversion strategies.

**Instructions**:
1. Run walk-forward analysis on all strategies
2. Run Monte Carlo simulation
3. Compile performance comparison report
4. Compare with trend following strategies
5. HUMAN REVIEW REQUIRED

**Validation Report**: Same template as Phase 12.2

**Acceptance Criteria**:
- All strategies validated
- Comparison report generated
- Human review completed
- Recommendations documented

---

### Phase 12.3 Summary

**Total Duration**: 3 weeks (15 working days)

**Deliverables**:
- ✅ 7 mean-reversion strategies implemented
- ✅ All strategies have complete hypothesis documents
- ✅ All strategies pass validation
- ✅ Performance metrics stored in database

**Success Metrics**:
- 7 strategies complete
- Average Sharpe >1.0
- Average Max DD <25%
- 90% walk-forward pass rate

**Next Phase**: Phase 12.4 - Momentum Strategies (7 strategies)

---

## 📚 Phase 12.4: Momentum Strategies (Weeks 9-11)

### Dependencies
- Phase 12.3 complete and validated
- Mean reversion strategies deployed

### Deliverables
- 7 momentum strategies implemented
- All strategies have complete hypothesis documents
- All strategies pass validation

---

### Task 12.4.1: Document RSI Momentum (0.5 days)

**Status**: Already exists in codebase, needs documentation

**File**: `crates/strategy/src/strategies/momentum/rsi_momentum.rs`

**Objective**: Complete documentation and validation.

**Hypothesis Document**: `doc/phase_12/hypotheses/momentum/rsi_momentum.md`

**Key Hypothesis**:
"RSI momentum signals indicate strong trends. The strategy enters long when RSI rises above 50 with increasing momentum and exits when RSI falls below 50 or shows bearish divergence."

**Parameters**:
- `rsi_period`: 14 periods
- `momentum_threshold`: 50
- `divergence_lookback`: 5 periods

**Acceptance Criteria**:
- Hypothesis document complete
- Validation passes
- Tests pass

---

### Task 12.4.2: Document MACD Strategy (0.5 days)

**Status**: Already exists in codebase, needs documentation

**File**: `crates/strategy/src/strategies/momentum/macd_strategy.rs`

**Objective**: Complete documentation and validation.

**Hypothesis Document**: `doc/phase_12/hypotheses/momentum/macd_strategy.md`

**Key Hypothesis**:
"MACD momentum captures trend strength. The strategy enters on bullish MACD crossovers with histogram confirmation and exits on bearish crossovers or histogram decline."

**Acceptance Criteria**:
- Hypothesis document complete
- Validation passes
- Tests pass

---

### Task 12.4.3: Implement ROC Strategy (1 day)

**Objective**: Implement Rate of Change momentum strategy.

**File**: `crates/strategy/src/strategies/momentum/roc_strategy.rs`

**Hypothesis Document**: `doc/phase_12/hypotheses/momentum/roc_strategy.md`

**Key Hypothesis**:
"Rate of Change measures price momentum. The strategy enters long when ROC is positive and accelerating, exits when ROC turns negative or decelerates."

**Parameters**:
- `roc_period`: 10 periods
- `entry_threshold`: 2%
- `exit_threshold`: -1%

**Acceptance Criteria**:
- Strategy implemented per template
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.4.4: Implement ADX Trend Strategy (1 day)

**Objective**: Implement ADX-based trend momentum strategy.

**File**: `crates/strategy/src/strategies/momentum/adx_trend.rs`

**Hypothesis Document**: `doc/phase_12/hypotheses/momentum/adx_trend.md`

**Key Hypothesis**:
"ADX measures trend strength. The strategy only enters when ADX > 25 (strong trend), using +DI/-DI for direction, and exits when ADX falls below 20 or trend reverses."

**Parameters**:
- `adx_period`: 14 periods
- `strong_trend_threshold`: 25
- `weak_trend_threshold`: 20

**Acceptance Criteria**:
- ADX calculations correct
- Trend strength filtering works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.4.5: Implement Momentum Factor Strategy (1.5 days)

**Objective**: Implement a multi-factor momentum strategy.

**File**: `crates/strategy/src/strategies/momentum/momentum_factor.rs`

**Hypothesis Document**: `doc/phase_12/hypotheses/momentum/momentum_factor.md`

**Key Hypothesis**:
"Combining multiple momentum factors provides stronger signals. The strategy uses price momentum, volume momentum, and RSI momentum to generate entry/exit signals."

**Parameters**:
- `lookback_period`: 20 periods
- `min_factors`: 2 (out of 3)

**Acceptance Criteria**:
- Multiple factors combined
- Factor scoring works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.4.6: Implement Volume Weighted Momentum (1 day)

**Objective**: Implement volume-weighted momentum strategy.

**File**: `crates/strategy/src/strategies/momentum/volume_momentum.rs`

**Hypothesis Document**: `doc/phase_12/hypotheses/momentum/volume_momentum.md`

**Key Hypothesis**:
"Volume confirms momentum. The strategy only enters on price momentum when volume is above average and increasing."

**Acceptance Criteria**:
- Volume calculations correct
- Momentum confirmed by volume
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.4.7: Implement Multi-Timeframe Momentum (1.5 days)

**Objective**: Implement a multi-timeframe momentum strategy.

**File**: `crates/strategy/src/strategies/momentum/multi_tf_momentum.rs`

**Hypothesis Document**: `doc/phase_12/hypotheses/momentum/multi_tf_momentum.md`

**Key Hypothesis**:
"Multi-timeframe alignment strengthens momentum signals. The strategy requires momentum confirmation on both short and long timeframes."

**Parameters**:
- `fast_tf`: 4H (default)
- `slow_tf`: 1D (default)

**Acceptance Criteria**:
- Multi-timeframe data handling
- Alignment logic correct
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.4.8: Momentum Module Integration (0.5 days)

**Objective**: Integrate all momentum strategies.

**Module Export**:
```rust
pub mod rsi_momentum;
pub mod macd_strategy;
pub mod roc_strategy;
pub mod adx_trend;
pub mod momentum_factor;
pub mod volume_momentum;
pub mod multi_tf_momentum;

pub use rsi_momentum::RSIMomentumStrategy;
pub use macd_strategy::MACDStrategy;
pub use roc_strategy::ROCStrategy;
pub use adx_trend::ADXTrendStrategy;
pub use momentum_factor::MomentumFactorStrategy;
pub use volume_momentum::VolumeMomentumStrategy;
pub use multi_tf_momentum::MultiTFMomentumStrategy;
```

**Acceptance Criteria**:
- All strategies exported and registered
- API works for all strategies
- Validation compiled

---

### Task 12.4.9: Run Phase 12.4 Validation (1 day)

**Objective**: Validate all momentum strategies.

**Acceptance Criteria**:
- All strategies validated
- Comparison report generated
- Human review completed

---

### Phase 12.4 Summary

**Total Duration**: 3 weeks (15 working days)

**Deliverables**:
- ✅ 7 momentum strategies implemented
- ✅ All strategies documented
- ✅ All strategies validated

**Next Phase**: Phase 12.5 - Volatility Strategies (7 strategies)

---

## 📚 Phase 12.5: Volatility-Based Strategies (Weeks 12-14)

### Dependencies
- Phase 12.4 complete and validated

### Deliverables
- 7 volatility strategies implemented
- All strategies documented and validated

---

### Task 12.5.1: Implement ATR Breakout (1 day)

**File**: `crates/strategy/src/strategies/volatility/atr_breakout.rs`

**Hypothesis**: "ATR breakout captures volatility expansions. Enter long when price breaks above previous high + ATR, exit on opposite breakout."

**Parameters**:
- `atr_period`: 14
- `atr_multiplier`: 1.5

**Acceptance Criteria**:
- Strategy implemented
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.5.2: Implement Volatility Squeeze (1 day)

**File**: `crates/strategy/src/strategies/volatility/vol_squeeze.rs`

**Hypothesis**: "Volatility squeeze precedes big moves. Enter long when price breaks out of a squeeze (low volatility)."

**Parameters**:
- `bb_period`: 20
- `bb_std_dev`: 2.0
- `kk_period`: 20
- `kk_mult`: 1.5
- `squeeze_threshold`: 0.1

**Acceptance Criteria**:
- Squeeze detection works
- Breakout logic correct
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.5.3: Implement Volatility Regime Strategy (1 day)

**File**: `crates/strategy/src/strategies/volatility/vol_regime.rs`

**Hypothesis**: "Strategies should adapt to volatility regimes. Use low-volatility strategies in calm markets, high-volatility strategies in volatile markets."

**Acceptance Criteria**:
- Regime detection works
- Strategy switching works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.5.4: Implement ATR Trailing Stop (1 day)

**File**: `crates/strategy/src/strategies/volatility/atr_trailing.rs`

**Hypothesis**: "ATR-based trailing stops adapt to volatility. The strategy uses ATR for dynamic stop losses."

**Acceptance Criteria**:
- Trailing stop calculation correct
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.5.5: Implement Volatility-Adjusted Position Sizing (1.5 days)

**File**: `crates/strategy/src/strategies/volatility/vol_sizing.rs`

**Hypothesis**: "Position size should be inversely proportional to volatility. Reduce size in high volatility, increase in low volatility."

**Acceptance Criteria**:
- Sizing calculation correct
- Integration with backtest engine
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.5.6: Implement GARCH-Based Strategy (2 days)

**File**: `crates/strategy/src/strategies/volatility/garch_strategy.rs`

**Hypothesis**: "GARCH models predict volatility. Use predicted volatility for position sizing and timing."

**Acceptance Criteria**:
- GARCH model implemented
- Prediction logic works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.5.7: Implement VIX-Style Strategy (1 day)

**File**: `crates/strategy/src/strategies/volatility/vix_style.rs`

**Hypothesis**: "Crypto volatility index (like VIX) predicts market stress. Use high VIX as contrarian signal (buy when fear high)."

**Acceptance Criteria**:
- Volatility index calculated
- Contrarian logic works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.5.8: Volatility Module Integration (0.5 days)

**Module Export**:
```rust
pub mod atr_breakout;
pub mod vol_squeeze;
pub mod vol_regime;
pub mod atr_trailing;
pub mod vol_sizing;
pub mod garch_strategy;
pub mod vix_style;

pub use atr_breakout::ATRBreakoutStrategy;
pub use vol_squeeze::VolSqueezeStrategy;
pub use vol_regime::VolRegimeStrategy;
pub use atr_trailing::ATRTrailingStrategy;
pub use vol_sizing::VolSizingStrategy;
pub use garch_strategy::GARCHStrategy;
pub use vix_style::VIXStyleStrategy;
```

---

### Task 12.5.9: Run Phase 12.5 Validation (1 day)

**Objective**: Validate all volatility strategies.

**Acceptance Criteria**:
- All strategies validated
- Comparison report generated
- Human review completed

---

### Phase 12.5 Summary

**Total Duration**: 3 weeks (15 working days)

**Next Phase**: Phase 12.6 - Sentiment Strategies (7 strategies)

---

## 📚 Phase 12.6: Sentiment-Based Strategies (Weeks 15-17)

### Dependencies
- Phase 12.5 complete and validated
- Sentiment data available (Phase 13 already complete)

### Deliverables
- 7 sentiment strategies implemented
- All strategies documented and validated

---

### Task 12.6.1: Document Fear & Greed Contrarian (0.5 days)

**Status**: Already exists, needs documentation

**File**: `crates/strategy/src/strategies/sentiment/fear_greed_contrarian.rs`

**Hypothesis Document**: `doc/phase_12/hypotheses/sentiment/fear_greed_contrarian.md`

**Acceptance Criteria**:
- Hypothesis document complete
- Validation passes
- Tests pass

---

### Task 12.6.2: Implement Sentiment Momentum (1 day)

**File**: `crates/strategy/src/strategies/sentiment/sentiment_momentum.rs`

**Hypothesis**: "Sentiment momentum predicts price momentum. Follow sentiment trends."

**Acceptance Criteria**:
- Strategy implemented
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.6.3: Implement Divergence Strategy (1 day)

**File**: `crates/strategy/src/strategies/sentiment/divergence_strategy.rs`

**Hypothesis**: "Price-sentiment divergence predicts reversals. Buy when price low but sentiment improving."

**Acceptance Criteria**:
- Divergence detection works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.6.4: Implement News Sentiment Strategy (1.5 days)

**File**: `crates/strategy/src/strategies/sentiment/news_sentiment.rs`

**Hypothesis**: "News sentiment impacts prices. React to news sentiment changes."

**Acceptance Criteria**:
- News sentiment integrated
- Signal generation works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.6.5: Implement Social Volume Strategy (1 day)

**File**: `crates/strategy/src/strategies/sentiment/social_volume.rs`

**Hypothesis**: "Social volume indicates attention. High social volume + sentiment = strong signal."

**Acceptance Criteria**:
- Social volume integrated
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.6.6: Implement Composite Sentiment Strategy (1 day)

**File**: `crates/strategy/src/strategies/sentiment/composite_sentiment.rs`

**Hypothesis**: "Multiple sentiment sources combined are stronger. Aggregate Fear/Greed, news, social, etc."

**Acceptance Criteria**:
- Composite calculation works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.6.7: Implement Regime-Based Sentiment Strategy (1.5 days)

**File**: `crates/strategy/src/strategies/sentiment/regime_sentiment.rs`

**Hypothesis**: "Sentiment signals vary by market regime. Adjust sentiment interpretation based on bull/bear regime."

**Acceptance Criteria**:
- Regime detection works
- Sentiment adaptation works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.6.8: Sentiment Module Integration (0.5 days)

**Module Export**:
```rust
pub mod fear_greed_contrarian;
pub mod sentiment_momentum;
pub mod divergence_strategy;
pub mod news_sentiment;
pub mod social_volume;
pub mod composite_sentiment;
pub mod regime_sentiment;

pub use fear_greed_contrarian::FearGreedContrarianStrategy;
pub use sentiment_momentum::SentimentMomentumStrategy;
pub use divergence_strategy::DivergenceStrategy;
pub use news_sentiment::NewsSentimentStrategy;
pub use social_volume::SocialVolumeStrategy;
pub use composite_sentiment::CompositeSentimentStrategy;
pub use regime_sentiment::RegimeSentimentStrategy;
```

---

### Task 12.6.9: Run Phase 12.6 Validation (1 day)

**Objective**: Validate all sentiment strategies.

**Acceptance Criteria**:
- All strategies validated
- Comparison report generated
- Human review completed

---

### Phase 12.6 Summary

**Total Duration**: 3 weeks (15 working days)

**Next Phase**: Phase 12.7 - Multi-Indicator Strategies (7 strategies)

---

## 📚 Phase 12.7: Multi-Indicator Strategies (Weeks 18-20)

### Dependencies
- Phase 12.6 complete and validated

### Deliverables
- 7 multi-indicator strategies implemented
- All strategies documented and validated

---

### Task 12.7.1: Implement Trend + Mean Reversion Hybrid (1.5 days)

**File**: `crates/strategy/src/strategies/multi_indicator/trend_mean_rev.rs`

**Hypothesis**: "Combining trend and mean reversion improves performance. Use trend for direction, mean reversion for timing."

**Acceptance Criteria**:
- Hybrid logic works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.7.2: Implement MACD + RSI Combo (1 day)

**File**: `crates/strategy/src/strategies/multi_indicator/macd_rsi_combo.rs`

**Hypothesis**: "MACD + RSI combination provides stronger signals. Both must agree for entry."

**Acceptance Criteria**:
- Combo logic works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.7.3: Implement Adaptive Combination Strategy (1.5 days)

**File**: `crates/strategy/src/strategies/multi_indicator/adaptive_combo.rs`

**Hypothesis**: "Adaptively weight indicators based on market conditions. Use best-performing indicators for current regime."

**Acceptance Criteria**:
- Adaptive weighting works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.7.4: Implement Ensemble Weighted Strategy (1.5 days)

**File**: `crates/strategy/src/strategies/multi_indicator/ensemble_weighted.rs`

**Hypothesis**: "Ensemble of multiple strategies is more robust. Weight strategies by recent performance."

**Acceptance Criteria**:
- Ensemble logic works
- Weighting calculation correct
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.7.5: Implement Regime-Switching Strategy (1.5 days)

**File**: `crates/strategy/src/strategies/multi_indicator/regime_switching.rs`

**Hypothesis**: "Switch strategies based on market regime. Use trend following in bull, mean reversion in sideways."

**Acceptance Criteria**:
- Regime detection works
- Strategy switching works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.7.6: Implement Confidence-Weighted Strategy (1 day)

**File**: `crates/strategy/src/strategies/multi_indicator/confidence_weighted.rs`

**Hypothesis**: "Position size based on signal confidence. Strong signals = larger positions."

**Acceptance Criteria**:
- Confidence scoring works
- Position sizing adapts
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.7.7: Implement ML-Enhanced Multi-Indicator Strategy (2 days)

**File**: `crates/strategy/src/strategies/multi_indicator/ml_enhanced.rs`

**Hypothesis**: "ML can find optimal indicator combinations. Train model to combine indicators optimally."

**Acceptance Criteria**:
- ML model integrated
- Inference works
- Tests pass
- Validation passes
- Hypothesis complete

---

### Task 12.7.8: Multi-Indicator Module Integration (0.5 days)

**Module Export**:
```rust
pub mod trend_mean_rev;
pub mod macd_rsi_combo;
pub mod adaptive_combo;
pub mod ensemble_weighted;
pub mod regime_switching;
pub mod confidence_weighted;
pub mod ml_enhanced;

pub use trend_mean_rev::TrendMeanRevStrategy;
pub use macd_rsi_combo::MACDRSIComboStrategy;
pub use adaptive_combo::AdaptiveComboStrategy;
pub use ensemble_weighted::EnsembleWeightedStrategy;
pub use regime_switching::RegimeSwitchingStrategy;
pub use confidence_weighted::ConfidenceWeightedStrategy;
pub use ml_enhanced::MLEnhancedStrategy;
```

---

### Task 12.7.9: Run Phase 12.7 Validation (1 day)

**Objective**: Validate all multi-indicator strategies.

**Acceptance Criteria**:
- All strategies validated
- Comparison report generated
- Human review completed

---

### Phase 12.7 Summary

**Total Duration**: 3 weeks (15 working days)

**Next Phase**: Phase 12.8 - Research & Documentation (Weeks 21-22)

---

## 📚 Phase 12.8: Research & Documentation (Weeks 21-22)

### Dependencies
- All previous phases complete and validated
- 49 strategies implemented

### Deliverables
- Complete strategy library documentation
- Cross-asset validation
- Strategy comparison report
- Strategy selection guide

---

### Task 12.8.1: Cross-Asset Validation (3 days)

**Objective**: Validate all strategies across multiple assets and timeframes.

**Instructions**:
1. Test each strategy on 20+ assets
2. Test on multiple timeframes (1h, 4h, 1d)
3. Analyze performance by asset class
4. Identify asset-specific strengths/weaknesses

**Deliverables**:
- Cross-asset performance matrix
- Timeframe performance analysis
- Asset class recommendations

**Acceptance Criteria**:
- All strategies tested on 20+ assets
- All strategies tested on 3+ timeframes
- Performance matrix complete
- Recommendations documented

---

### Task 12.8.2: Long-Term Stability Testing (2 days)

**Objective**: Test strategy stability over long time periods.

**Instructions**:
1. Test on 5+ years of data
2. Analyze performance by year
3. Check for strategy degradation
4. Identify regime dependencies

**Deliverables**:
- Long-term performance analysis
- Year-by-year breakdown
- Degradation detection report

**Acceptance Criteria**:
- All strategies tested on 5+ years
- Yearly analysis complete
- Degradation documented

---

### Task 12.8.3: Stress Testing (2 days)

**Objective**: Stress test strategies in extreme market conditions.

**Instructions**:
1. Test on crash periods (COVID, FTX, etc.)
2. Test on extreme volatility periods
3. Test on correlation breakdown scenarios
4. Analyze worst-case drawdowns

**Deliverables**:
- Stress test results
- Crash analysis
- Volatility analysis

**Acceptance Criteria**:
- All strategies stress tested
- Crash analysis complete
- Risk documented

---

### Task 12.8.4: Correlation Matrix Analysis (2 days)

**Objective**: Analyze correlations between all strategies.

**Instructions**:
1. Calculate correlation matrix for all strategies
2. Identify highly correlated strategies
3. Identify uncorrelated strategies
4. Build diversified portfolios

**Deliverables**:
- Correlation matrix
- Strategy clustering
- Portfolio recommendations

**Acceptance Criteria**:
- Correlation matrix complete
- Clustering analysis done
- Portfolio recommendations made

---

### Task 12.8.5: Strategy Comparison Report (2 days)

**Objective**: Generate comprehensive comparison report.

**Instructions**:
1. Rank strategies by multiple metrics
2. Identify best performers by category
3. Compare vs. baselines
4. Document strengths and weaknesses

**Deliverables**:
- Strategy ranking table
- Category comparisons
- Baseline comparisons
- Detailed analysis

**Acceptance Criteria**:
- Ranking complete
- Comparisons documented
- Report comprehensive

---

### Task 12.8.6: Strategy Selection Guide (2 days)

**Objective**: Create guide for selecting strategies.

**Instructions**:
1. Document decision criteria
2. Create selection flowcharts
3. Provide regime-specific recommendations
4. Create portfolio construction guidelines

**Deliverables**:
- Selection guide document
- Decision flowcharts
- Regime recommendations
- Portfolio guidelines

**Acceptance Criteria**:
- Guide comprehensive
- Flowcharts clear
- Recommendations actionable

---

### Task 12.8.7: Complete Documentation (2 days)

**Objective**: Ensure all documentation is complete and consistent.

**Instructions**:
1. Review all hypothesis documents
2. Ensure consistency across documents
3. Update API documentation
4. Create user guides

**Deliverables**:
- Complete documentation set
- API documentation updated
- User guides created

**Acceptance Criteria**:
- All documents complete
- Consistency ensured
- Documentation comprehensive

---

### Task 12.8.8: Final Validation and Handoff (1 day)

**Objective**: Final validation and preparation for deployment.

**Instructions**:
1. Run final test suite
2. Verify database integrity
3. Generate final metrics
4. Prepare deployment package
5. HUMAN REVIEW REQUIRED

**Deliverables**:
- Final test results
- Database export
- Deployment package
- Final metrics report

**Acceptance Criteria**:
- All tests pass
- Database verified
- Deployment ready
- Metrics complete

---

### Phase 12.8 Summary

**Total Duration**: 2 weeks (10 working days)

**Deliverables**:
- ✅ Cross-asset validation complete
- ✅ Long-term stability tested
- ✅ Stress testing complete
- ✅ Correlation analysis done
- ✅ Comparison report generated
- ✅ Selection guide created
- ✅ Documentation complete
- ✅ Deployment ready

---

## 🎯 Phase 12 Final Summary

**Total Duration**: 22 weeks (110 working days, ~5.5 months)

**Total Deliverables**:
- ✅ Strategy framework with registry
- ✅ 49 strategies implemented (7 per category × 7 categories)
- ✅ 49 hypothesis documents
- ✅ Comprehensive validation for all strategies
- ✅ Database schema and performance metrics
- ✅ API endpoints for all strategies
- ✅ Cross-asset validation
- ✅ Strategy comparison reports
- ✅ Selection guides and documentation

**Strategy Breakdown**:
- Trend Following: 7 strategies
- Mean Reversion: 7 strategies
- Momentum: 7 strategies
- Volatility: 7 strategies
- Sentiment: 7 strategies
- Multi-Indicator: 7 strategies
- Baselines: 2 strategies (included in framework)

**Success Metrics**:
- ✅ 49+ strategies implemented
- ✅ 100% documentation coverage
- ✅ 90% walk-forward pass rate
- ✅ Average Sharpe >1.0
- ✅ Average Max DD <25%
- ✅ 100% test coverage
- ✅ Zero compilation warnings

**Quality Gates**:
- ✅ All code compiles without warnings
- ✅ All tests pass
- ✅ All strategies validated
- ✅ All performance metrics stored
- ✅ Human review checkpoints completed

**Key Accomplishments**:
- Systematic AI-assisted development
- Hypothesis-first approach
- Rigorous validation
- Comprehensive documentation
- Research-focused architecture
- Extensible framework for future strategies

**Next Steps**:
1. Deploy validated strategies to production
2. Monitor strategy performance in live market
3. Collect feedback and iterate
4. Begin Phase 13 (Advanced Validation) or Phase 14 (ML-Assisted Research)

---

## 📝 Notes for AI Agents

### Code Style
- Follow Rust best practices
- Use descriptive variable names
- Add comments for complex logic
- Keep functions focused and small
- Handle errors gracefully

### Testing
- Write tests first when possible (TDD)
- Test edge cases thoroughly
- Test parameter validation
- Test state management
- Test integration with backtest engine

### Documentation
- Be thorough and detailed
- Explain "why" not just "what"
- Include examples
- Document edge cases
- Keep documentation up to date

### Validation
- Never skip validation steps
- Be honest about results
- Document failures honestly
- Suggest improvements
- Learn from failures

### Communication
- Report progress clearly
- Ask questions when uncertain
- Report blockers immediately
- Provide actionable feedback
- Be collaborative

---

## 🚨 Known Issues and Limitations

### Current Limitations
- Spot-only trading (no leverage, no shorting)
- Limited to single-asset strategies (pairs trading needs adaptation)
- ML strategies require pre-trained models
- Sentiment strategies require external data feeds
- Some strategies may be overfitted to historical data

### Potential Issues
- Market regime changes may degrade strategy performance
- New market conditions may not match historical patterns
- Transaction costs may reduce actual returns
- Slippage may be higher in volatile markets
- Strategies may be correlated during market crashes

### Mitigation Strategies
- Continuous monitoring of strategy performance
- Regular re-validation with new data
- Adaptive strategies that adjust to market conditions
- Diversification across strategy types
- Risk limits and automatic shutdown

---

## 📚 References

### Project References
- AlphaField README: `AlphaField/README.md`
- Architecture: `AlphaField/doc/architecture.md`
- API Documentation: `AlphaField/doc/api.md`
- Optimization Workflow: `AlphaField/doc/optimization_workflow.md`
- Project Plan: `AlphaField/doc/project_plan.md`

### External References
- "Evidence-Based Technical Analysis" by David Aronson
- "Quantitative Trading" by Ernest Chan
- "Algorithmic Trading" by Ernie Chan
- "Advances in Financial Machine Learning" by Marcos Lopez de Prado
- "Walk-Forward Analysis: The Best Kept Secret" by Robert Pardo

### Rust References
- Rust Book: https://doc.rust-lang.org/book/
- Rust by Example: https://doc.rust-lang.org/rust-by-example/
- Tokio Documentation: https://tokio.rs/
- Polars Documentation: https://pola-rs.github.io/polars/

---

**Last Updated**: January 2026  
**Document Version**: 3.0 (AI Agent Optimized)  
**Status**: Ready for Execution