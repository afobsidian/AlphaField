---
type: feature
priority: high
created: 2026-01-30T00:00:00Z
status: open
tags: [portfolio, multi-strategy, optimization, risk, backtesting, phase-15]
keywords: [portfolio, correlation, optimization, mean-variance, risk-parity, kelly-criterion, stress-testing, monte-carlo, walk-forward, diversification]
patterns: [portfolio construction, optimization algorithms, risk management, multi-asset testing]
---

# FEATURE-015: Multi-Strategy Portfolio Testing (Phase 15)

## Description
Implement comprehensive multi-strategy portfolio testing capabilities including portfolio construction, optimization algorithms, and validation frameworks. This phase enables users to combine multiple trading strategies intelligently, optimize weights, and validate portfolio-level performance through stress testing and sensitivity analysis.

## Context
Phase 15 is part of the V3 roadmap focusing on combining multiple trading strategies into optimized portfolios. This is a critical capability for research-first trading platforms where diversification and risk management are paramount. The foundation exists with correlation analysis, but full portfolio optimization and validation frameworks are needed.

## Requirements

### Portfolio Construction
- [ ] **Correlation Matrix Visualization**: Visual representation of strategy interdependencies
  - Heat map visualization of correlation matrix
  - Interactive filtering by correlation threshold
  - Clustering of highly correlated strategies
  
- [ ] **Portfolio Optimization Algorithms**:
  - **Mean-Variance Optimization** (Markowitz): Maximize Sharpe ratio or minimize volatility
  - **Risk Parity**: Equal risk contribution from each strategy
  - **Minimum Variance**: Portfolio with lowest possible volatility
  - **Maximum Diversification**: Maximize diversification ratio
  - **Inverse Volatility**: Weight by inverse of strategy volatility
  
- [ ] **Dynamic Weighting**: Adjust strategy weights based on detected market regime
  - Regime detection integration
  - Weight transition smoothing (avoid abrupt changes)
  - Historical regime-specific optimal weights
  
- [ ] **Position Sizing Optimization**:
  - **Kelly Criterion**: Optimal position sizing based on win rate and payoff
  - **Fractional Kelly**: Conservative Kelly variants (Half Kelly, Quarter Kelly)
  - **Optimal f**: Fixed fraction position sizing
  - **Volatility-adjusted sizing**: Scale positions by realized volatility
  
- [ ] **Sector/Asset Allocation**: Support for grouping strategies by asset class or sector

### Portfolio Validation
- [ ] **Portfolio-Level Walk-Forward Analysis**: Test portfolio performance with rolling windows
  - Re-optimize weights at each window step
  - Track portfolio metrics consistency over time
  
- [ ] **Portfolio Monte Carlo Simulation**: Test portfolio robustness to trade sequence randomization
  - Correlation-aware trade shuffling
  - Strategy failure scenario simulation
  
- [ ] **Portfolio Sensitivity Analysis**: Measure impact of parameter changes
  - Weight perturbation testing
  - Strategy removal impact (leave-one-out analysis)
  
- [ ] **Stress Testing**: Correlation breakdown scenarios
  - "All correlations go to 1" scenario (2008-style crisis)
  - Worst-case historical periods
  - Tail risk estimation
  
- [ ] **Diversification Benefits Quantification**:
  - Diversification ratio calculation
  - Risk reduction vs. single best strategy
  - Marginal diversification contribution per strategy

### Multi-Asset Testing
- [ ] **Cross-Asset Correlation Analysis**: Extended correlation analysis across different asset classes
- [ ] **Portfolio Backtesting Across Asset Classes**: Test portfolio on multiple asset combinations
- [ ] **Currency Risk Assessment**: For multi-currency portfolios
- [ ] **Liquidity Impact Testing**: Model portfolio-wide liquidity constraints
- [ ] **Slippage Modeling at Portfolio Scale**: Aggregate slippage across simultaneous trades

## Current State

### Already Implemented
1. **Correlation Analysis** (`crates/backtest/src/correlation.rs`):
   - Pearson correlation calculation between equity curves/return series
   - Correlation matrix with labels
   - Correlation alerts for high correlations
   - Diversification score calculation (1 - avg correlation)
   - Configurable alert thresholds

2. **Portfolio Tracking** (`crates/backtest/src/portfolio.rs`):
   - Position management with multiple assets
   - Equity history tracking
   - Trade recording with MAE/MFE
   - Support for both Spot and Margin trading modes

3. **Strategy Registry**: Framework for registering and combining strategies

### Not Yet Implemented
1. Portfolio optimization algorithms (Mean-Variance, Risk Parity, etc.)
2. Kelly Criterion and position sizing optimization
3. Portfolio-level validation (walk-forward, Monte Carlo, sensitivity)
4. Stress testing with correlation breakdown scenarios
5. Dashboard integration for portfolio visualization
6. Multi-asset portfolio backtesting framework

## Desired State
A complete portfolio testing module that allows researchers to:
1. Combine multiple strategies into optimized portfolios
2. Apply various optimization algorithms with configurable objectives
3. Validate portfolio robustness through comprehensive testing
4. Visualize portfolio composition and correlations
5. Compare portfolio performance against individual strategies
6. Export portfolio configurations for deployment

## Research Context

### Keywords to Search
- **portfolio** - Core portfolio functionality and existing implementations
- **correlation** - Correlation analysis module for understanding strategy relationships
- **optimization** - Search for any existing optimization frameworks
- **mean-variance** - Markowitz optimization patterns
- **risk-parity** - Equal risk contribution weighting
- **kelly-criterion** - Optimal position sizing implementations
- **monte-carlo** - Existing Monte Carlo simulation code
- **walk-forward** - Existing walk-forward validation
- **stress-test** - Any stress testing implementations
- **diversification** - Diversification metrics and analysis
- **matrix** - Matrix operations for optimization
- **constraints** - Constraint handling in optimization

### Patterns to Investigate
1. **Optimization Patterns**: How are optimization problems currently structured?
2. **Validation Patterns**: Look at existing validation framework in `crates/backtest/src/validation/`
3. **Matrix Operations**: Check for linear algebra libraries in use (nalgebra, ndarray)
4. **Dashboard Integration**: How strategy results are displayed in the dashboard
5. **Configuration Patterns**: How optimization parameters are configured
6. **Error Handling**: Pattern for numerical optimization failures

### Key Decisions Made
1. **Use existing correlation module** as foundation - correlation.rs provides solid correlation analysis
2. **Follow validation framework patterns** from Phase 12.7/13 - use similar structure for portfolio validation
3. **Dashboard integration required** - portfolio results must be visualizable
4. **Support both Spot and Margin modes** - position sizing must respect trading mode
5. **Maintain statistical rigor** - all portfolio metrics must include confidence intervals where applicable
6. **Regime-aware by default** - portfolio optimization should integrate with regime detection from Phase 13

## Success Criteria

### Automated Verification
- [ ] All new modules have comprehensive unit tests (>80% coverage)
- [ ] Portfolio optimization produces mathematically correct results (verified against known test cases)
- [ ] Stress tests produce expected outcomes for known scenarios
- [ ] Integration tests with existing backtest engine pass
- [ ] `cargo test` passes without warnings
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo fmt` produces no changes

### Manual Verification
- [ ] Can create a portfolio with 3+ strategies
- [ ] Portfolio optimization runs successfully with Mean-Variance objective
- [ ] Correlation matrix visualization displays correctly in dashboard
- [ ] Stress test results show meaningful risk metrics
- [ ] Portfolio backtest produces results comparable to individual strategy backtests
- [ ] Kelly Criterion position sizing produces reasonable allocations (not extreme)

### Performance Criteria
- [ ] Portfolio optimization completes in < 5 seconds for 10 strategies
- [ ] Portfolio backtest performance is within 2x of single strategy backtest
- [ ] Memory usage scales linearly with number of strategies

## Related Information

### Dependencies
- **Phase 12.7**: Validation Framework (complete) - provides validation patterns
- **Phase 13**: Advanced Validation Techniques (complete) - provides statistical tools
- **Phase 14**: ML-Assisted Research (partial) - may use optimization algorithms

### Documentation References
- `doc/roadmap.md` - Phase 15 specification
- `crates/backtest/src/correlation.rs` - Existing correlation analysis
- `crates/backtest/src/portfolio.rs` - Portfolio tracking implementation
- `crates/dashboard/static/app.js` - Dashboard UI patterns

## Implementation Notes

### Suggested Module Structure
```
crates/backtest/src/
├── portfolio_optimization/
│   ├── mod.rs                    # Module exports
│   ├── optimizer.rs              # Core optimization trait and implementations
│   ├── objectives.rs             # Objective functions (Sharpe, Risk, etc.)
│   ├── constraints.rs            # Constraint definitions
│   └── algorithms/
│       ├── mean_variance.rs      # Markowitz optimization
│       ├── risk_parity.rs        # Equal risk contribution
│       └── inverse_volatility.rs # Simple volatility weighting
├── portfolio_validation/
│   ├── mod.rs
│   ├── walk_forward.rs           # Portfolio-level walk-forward
│   ├── monte_carlo.rs            # Portfolio Monte Carlo
│   ├── stress_test.rs            # Stress testing framework
│   └── sensitivity.rs            # Sensitivity analysis
├── position_sizing/
│   ├── mod.rs
│   ├── kelly.rs                  # Kelly Criterion implementations
│   └── volatility_adjusted.rs    # ATR-based sizing
└── visualization/
    ├── mod.rs
    └── correlation_viz.rs        # Correlation matrix visualization data
```

### Key Algorithms to Implement
1. **Mean-Variance Optimization**: Use quadratic programming or closed-form solution for Sharpe maximization
2. **Risk Parity**: Newton-Raphson iteration for equal risk contribution weights
3. **Kelly Criterion**: f* = (bp - q) / b, where b = odds, p = win rate, q = loss rate
4. **Monte Carlo**: Correlation-preserving reshuffling using Cholesky decomposition

### Potential Libraries
- `nalgebra` or `ndarray` for matrix operations
- `argmin` for optimization algorithms
- Existing `statrs` if already used for statistical functions

### Dashboard Integration Points
- Add portfolio creation endpoint to backtest API
- Correlation matrix visualization (heatmap)
- Portfolio performance charts (vs individual strategies)
- Weight adjustment sliders for manual optimization
- Stress test results display

## Open Questions for Planning Phase
1. Should we use an existing optimization crate (argmin) or implement simplex/gradient descent ourselves?
2. How should we handle optimization constraints (min/max weights, sum to 1)?
3. What's the expected number of strategies in a typical portfolio (affects algorithm choice)?
4. Should portfolio backtesting reuse existing engine or require separate implementation?
5. How to handle strategies with different timeframes in same portfolio?
