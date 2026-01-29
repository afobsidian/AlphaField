# Phase 13 User Guide: Advanced Validation Techniques

**Status**: Complete | **Last Updated**: January 2026

---

## Table of Contents
1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Understanding the Four Validation Types](#understanding-the-four-validation-types)
   - [Statistical Significance](#statistical-significance)
   - [Regime Testing](#regime-testing)
   - [Temporal Validation](#temporal-validation)
   - [Robustness](#robustness)
4. [Configuration Guide](#configuration-guide)
5. [Interpreting Results](#interpreting-results)
6. [Common Pitfalls](#common-pitfalls)
7. [Troubleshooting](#troubleshooting)
8. [Best Practices](#best-practices)

---

## Overview

Phase 13 introduces four advanced validation modules that provide institutional-grade analysis of trading strategies:

- **Statistical Significance**: Prove your results aren't just luck
- **Regime Testing**: Understand how your strategy performs in different market conditions
- **Temporal Validation**: Ensure your strategy performs consistently over time
- **Robustness**: Detect overfitting and fragile strategies

All validations integrate seamlessly with the existing AlphaField validation framework. You don't need to change your strategy code - just configure and run!

---

## Quick Start

### Running Phase 13 Validations (Automatic)

The simplest way to run Phase 13 validations is to let `StrategyValidator` handle everything:

```rust
use alphafield_backtest::validation::*;

// Create validator with configuration
let config = ValidationConfig {
    data_source: "postgresql://localhost/alphafield".to_string(),
    symbol: "BTCUSDT".to_string(),
    interval: "1d".to_string(),
    risk_free_rate: 0.02,
    initial_capital: 10000.0,
    fee_rate: 0.001,
    // Phase 13 configuration (optional - uses defaults if not set)
    max_parameters: 10,
    max_indicators: 5,
    max_branches: 20,
    perturbation_noise_levels: vec![0.01, 0.02, 0.05],
    rolling_window_fraction: 0.5,
    expanding_window_step_fraction: 0.2,
    max_statistical_iterations: 1000,
    enable_early_stopping: true,
    statistical_timeout_seconds: Some(30),
    walk_forward: WalkForwardConfig::default(),
    thresholds: ValidationThresholds::default(),
};

let validator = StrategyValidator::new(config.clone());

// This automatically runs all Phase 13 validations
let report = validator.validate(strategy_box, "BTCUSDT", &bars)?;

// Check Phase 13 results
if let Some(stat_sig) = report.statistical_significance {
    println!("Bootstrap 95% CI: [{:.3}, {:.3}]", 
        stat_sig.bootstrap.ci_lower, stat_sig.bootstrap.ci_upper);
    println!("P-value: {:.4}", stat_sig.sharpe_significance.p_value);
}

if let Some(regime_test) = report.regime_testing {
    println!("Detected {} regimes", regime_test.regimes.len());
    for regime in &regime_test.regime_performance {
        println!("{:?} Sharpe: {:.2}", regime.regime_type, regime.sharpe);
    }
}
```

---

## Understanding the Four Validation Types

### Statistical Significance

**Purpose**: Determine whether your strategy's performance is statistically significant or could have occurred by random chance.

**When to Use**:
- **Always**: Every strategy should have statistical significance testing
- **Before Deployment**: Never deploy without confirming significance (p < 0.05)
- **After Optimization**: Confirm improved metrics aren't overfit

**Key Metrics**:

| Metric | Good | Bad | Interpretation |
|---------|-------|-------|----------------|
| Bootstrap CI Width | < 0.5 | > 1.0 | Narrower = more precise estimate |
| Permutation P-value | < 0.05 | > 0.10 | Lower = less likely due to chance |
| ADF Is Stationary | Yes | No | Stationary returns are more predictable |
| Sharpe P-value | < 0.05 | > 0.10 | Sharpe is statistically significant |

**Interpretation Example**:

```rust
// Bootstrap result
let bootstrap = stat_sig.bootstrap;
if bootstrap.ci_upper - bootstrap.ci_lower < 0.5 {
    println!("✓ High confidence: Narrow confidence interval");
} else {
    println!("⚠ Low confidence: Wide interval - need more data");
}

// Permutation result
let perm = stat_sig.permutation;
if perm.p_value < 0.05 {
    println!("✓ Statistically significant: Results not due to chance");
} else {
    println!("✗ Not significant: Could be random luck");
}
```

**Common Issues**:

1. **Wide Confidence Intervals**
   - **Cause**: Insufficient trade history (< 100 trades)
   - **Fix**: Collect more data or validate on longer periods

2. **Non-Stationary Returns**
   - **Cause**: Strategy has regime-dependent behavior
   - **Fix**: Use regime testing to understand transitions

3. **High P-value (> 0.05)**
   - **Cause**: Strategy performance matches random distribution
   - **Fix**: Improve strategy logic or accept it's not working

---

### Regime Testing

**Purpose**: Analyze how your strategy performs across different market conditions (bull, bear, sideways, volatile, transition).

**When to Use**:
- **Always**: Every strategy has regime-dependent behavior
- **Before Deployment**: Understand worst-case performance
- **Strategy Selection**: Choose strategies that work in multiple regimes

**Key Metrics**:

| Metric | Good | Bad | Interpretation |
|---------|-------|-------|----------------|
| Regime Sharpe Variance | < 0.5 | > 1.0 | Consistent across regimes |
| Transition Impact | > 0.0 | < -0.5 | Positive = adapts well to changes |
| Stress Test Survival | High | Low | Survives worst regimes |

**Interpretation Example**:

```rust
let regime_test = report.regime_testing.unwrap();

// Check consistency
let sharpes: Vec<f64> = regime_test.regime_performance
    .iter()
    .map(|r| r.sharpe)
    .collect();
let variance = calculate_variance(&sharpes);
if variance < 0.5 {
    println!("✓ Consistent: Low variance across regimes");
} else {
    println!("⚠ Inconsistent: High variance - fragile to regime changes");
}

// Check stress survival
let survived = regime_test.stress_tests
    .iter()
    .filter(|s| s.survived)
    .count();
if survived == regime_test.stress_tests.len() {
    println!("✓ Robust: Survived all stress tests");
} else {
    println!("⚠ Risky: Failed {} stress regimes", 
        regime_test.stress_tests.len() - survived);
}
```

**Common Issues**:

1. **No Regimes Detected**
   - **Cause**: Insufficient data or stable market
   - **Fix**: Use longer validation period or check market conditions

2. **Very Poor Performance in One Regime**
   - **Cause**: Strategy optimized for specific market type
   - **Fix**: Add regime filters or diversify strategies

3. **High Negative Transition Impact**
   - **Cause**: Strategy slow to adapt to regime changes
   - **Fix**: Add regime detection to switch strategy parameters

---

### Temporal Validation

**Purpose**: Ensure your strategy performs consistently over time and doesn't degrade.

**When to Use**:
- **Always**: Check for overfitting and temporal decay
- **Before Deployment**: Confirm stability over full period
- **Strategy Maintenance**: Detect performance degradation early

**Key Metrics**:

| Metric | Good | Bad | Interpretation |
|---------|-------|-------|----------------|
| Stability Rating | Excellent/Good | Poor/VeryPoor | Consistent performance over time |
| Expanding Window Degradation | < 10% | > 30% | Low degradation = robust |
| Period Sharpe Variance | < 0.3 | > 0.7 | Consistent across years/quarters |
| Forward-Looking Degradation | < 20% | > 50% | Backtest matches reality |

**Interpretation Example**:

```rust
let temporal = report.temporal_validation.unwrap();

// Check stability
match temporal.rolling_stability.rating {
    StabilityRating::Excellent | StabilityRating::Good => {
        println!("✓ Stable: Coefficient of variation {:.2}", 
            temporal.rolling_stability.cv_sharpe);
    }
    _ => {
        println!("⚠ Unstable: CV {:.2} - high variance", 
            temporal.rolling_stability.cv_sharpe);
    }
}

// Check period consistency
let period_sharpes: Vec<f64> = temporal.periods
    .iter()
    .map(|p| p.sharpe)
    .collect();
if period_sharpes.iter().all(|&s| s > 0.0) {
    println!("✓ Consistent: Positive Sharpe in all periods");
} else {
    let negative_periods = period_sharpes.iter().filter(|&&s| s <= 0.0).count();
    println!("⚠ Inconsistent: {} periods with negative Sharpe", negative_periods);
}
```

**Common Issues**:

1. **Poor Stability Rating**
   - **Cause**: Strategy overfitted to specific period
   - **Fix**: Reduce parameters, simplify logic, use walk-forward

2. **Expanding Window Degradation**
   - **Cause**: Strategy works well on early data but fails later
   - **Fix**: Check for regime changes or data quality issues

3. **Large Forward-Looking Degradation**
   - **Cause**: Backtest overfitting or unrealistic assumptions
   - **Fix**: Use walk-forward, paper trading, reduce lookahead bias

---

### Robustness

**Purpose**: Measure strategy resilience to parameter changes, data perturbations, and market variations.

**When to Use**:
- **Always**: Detect overfitting early
- **Strategy Development**: Compare parameter sets
- **Before Deployment**: Ensure strategy isn't fragile

**Key Metrics**:

| Metric | Good | Bad | Interpretation |
|---------|-------|-------|----------------|
| Complexity Rating | Excellent/Good | Poor/VeryPoor | Simple is usually better |
| Perturbation Degradation | < 20% | > 40% | Resilient to noise |
| Outlier Dependency | < 30% | > 50% | Performance not from lucky trades |

**Interpretation Example**:

```rust
let robustness = report.robustness.unwrap();

// Check complexity
match robustness.complexity.rating {
    ComplexityRating::Excellent | ComplexityRating::Good => {
        println!("✓ Robust: {} parameters, {} indicators", 
            robustness.complexity.n_parameters,
            robustness.complexity.n_indicators);
    }
    ComplexityRating::Poor | ComplexityRating::VeryPoor => {
        println!("⚠ Overfitting: Too complex - {} parameters, {} indicators", 
            robustness.complexity.n_parameters,
            robustness.complexity.n_indicators);
    }
    _ => {}
}

// Check perturbation
let avg_degradation: f64 = robustness.perturbations
    .iter()
    .map(|p| p.degradation_pct)
    .sum::<f64>() / robustness.perturbations.len() as f64;
if avg_degradation < 20.0 {
    println!("✓ Resilient: Avg degradation {:.1}%", avg_degradation);
} else {
    println!("⚠ Fragile: High degradation {:.1}%", avg_degradation);
}

// Check outlier dependency
if robustness.outlier.outlier_dependency < 0.3 {
    println!("✓ Robust: Low outlier dependency {:.1}%", 
        robustness.outlier.outlier_dependency * 100.0);
} else {
    println!("⚠ Fragile: High outlier dependency {:.1}% - reliant on lucky trades", 
        robustness.outlier.outlier_dependency * 100.0);
}
```

**Common Issues**:

1. **Poor Complexity Rating**
   - **Cause**: Too many parameters or decision branches
   - **Fix**: Simplify strategy, remove unnecessary indicators, reduce parameter count

2. **High Perturbation Degradation**
   - **Cause**: Strategy sensitive to small data changes
   - **Fix**: Add noise tolerance, use smoothing, simplify logic

3. **High Outlier Dependency**
   - **Cause**: Strategy relies on a few big wins
   - **Fix**: Improve win rate, add risk management, reduce position size volatility

---

## Configuration Guide

### ValidationConfig Phase 13 Fields

```rust
pub struct ValidationConfig {
    // ... existing fields ...
    
    // Phase 13: Complexity thresholds
    pub max_parameters: usize,        // Default: 10
    pub max_indicators: usize,       // Default: 5
    pub max_branches: usize,          // Default: 20
    
    // Phase 13: Noise levels for perturbation (as decimal)
    pub perturbation_noise_levels: Vec<f64>,  // Default: [0.01, 0.02, 0.05]
    
    // Phase 13: Window sizes (fraction of total trades)
    pub rolling_window_fraction: f64,           // Default: 0.5
    pub expanding_window_step_fraction: f64,      // Default: 0.2
    
    // Phase 13: Statistical test limits
    pub max_statistical_iterations: usize,        // Default: 1000
    pub enable_early_stopping: bool,             // Default: true
    pub statistical_timeout_seconds: Option<u64>,  // Default: Some(30)
}
```

### Recommended Configurations

#### Conservative (Production-Ready)

```rust
ValidationConfig {
    // ... base config ...
    
    // Complexity: Strict thresholds
    max_parameters: 5,
    max_indicators: 3,
    max_branches: 10,
    
    // Perturbation: Test at multiple noise levels
    perturbation_noise_levels: vec![0.01, 0.02, 0.05, 0.10],
    
    // Temporal: Larger windows for stability
    rolling_window_fraction: 0.6,
    expanding_window_step_fraction: 0.15,
    
    // Statistical: More iterations for precision
    max_statistical_iterations: 2000,
    enable_early_stopping: true,
    statistical_timeout_seconds: Some(60),
    // ...
}
```

#### Moderate (Development/Research)

```rust
ValidationConfig {
    // ... base config ...
    
    // Complexity: Moderate thresholds
    max_parameters: 10,
    max_indicators: 5,
    max_branches: 20,
    
    // Perturbation: Standard noise levels
    perturbation_noise_levels: vec![0.01, 0.02, 0.05],
    
    // Temporal: Standard window sizes
    rolling_window_fraction: 0.5,
    expanding_window_step_fraction: 0.2,
    
    // Statistical: Standard iterations
    max_statistical_iterations: 1000,
    enable_early_stopping: true,
    statistical_timeout_seconds: Some(30),
    // ...
}
```

#### Aggressive (Exploratory)

```rust
ValidationConfig {
    // ... base config ...
    
    // Complexity: Allow more complex strategies
    max_parameters: 15,
    max_indicators: 8,
    max_branches: 30,
    
    // Perturbation: Test at more levels
    perturbation_noise_levels: vec![0.005, 0.01, 0.02, 0.05, 0.10],
    
    // Temporal: Smaller windows for more granular analysis
    rolling_window_fraction: 0.3,
    expanding_window_step_fraction: 0.1,
    
    // Statistical: More iterations, longer timeout
    max_statistical_iterations: 5000,
    enable_early_stopping: false,  // Don't stop early
    statistical_timeout_seconds: Some(120),
    // ...
}
```

---

## Interpreting Results

### Overall Score Interpretation

The overall Phase 13 score is 0-100, calculated from:
- 10% Statistical Significance
- 10% Robustness
- 5% Temporal Validation
- 25% Regime Testing (if available)

| Score Range | Grade | Meaning | Action |
|-------------|--------|---------|--------|
| 80-100 | A | Excellent | Ready for deployment with monitoring |
| 70-79 | B | Good | Deploy with caution, monitor closely |
| 60-69 | C | Acceptable | Optimize before deployment |
| 50-59 | D | Poor | Significant issues - need rework |
| 0-49 | F | Failing | Do not deploy - major problems |

### Conflicting Results

When validation modules disagree, use this decision tree:

```
Statistical Significance FAIL
└─> STOP: Not statistically significant - fix strategy first

Regime Testing FAIL (specific regime)
└─> Add regime filter: Don't trade in that regime
    OR
    └─> Use different strategy for that regime

Temporal Validation FAIL (poor stability)
└─> Reduce complexity (fewer parameters)
    OR
    └─> Use longer validation period

Robustness FAIL (high complexity)
└─> Simplify strategy (fewer indicators/parameters)
```

### Missing Phase 13 Data

If Phase 13 results are `None`, it's likely due to:

1. **Insufficient Trades**: Statistical significance needs 100+ trades
2. **Short History**: Temporal validation needs 252+ trades (~1 year)
3. **Config Disabled**: Check `enable_early_stopping` and timeout settings

The scoring system penalizes missing data with 30.0 (neutral) instead of 50.0 to encourage comprehensive validation.

---

## Common Pitfalls

### Pitfall 1: Over-Reliance on Backtest Sharpe

**Problem**: Backtest Sharpe can be misleading.

**Solution**: Always validate with Phase 13:
```rust
// Don't just check backtest Sharpe
if report.backtest.metrics.sharpe_ratio > 2.0 {
    println!("Great Sharpe!");  // Wrong!
}

// Check all Phase 13 metrics too
if report.statistical_significance.as_ref()
    .map(|s| s.sharpe_significance.is_significant)
    .unwrap_or(false) 
    && report.robustness.as_ref()
        .map(|r| r.outlier.outlier_dependency < 0.3)
        .unwrap_or(false)
{
    println!("Robust strategy with significant Sharpe");  // Right!
}
```

### Pitfall 2: Ignoring Regime Performance

**Problem**: Strategy works great in bull markets but fails in bears.

**Solution**: Check regime-specific performance:
```rust
let regime_test = report.regime_testing.unwrap();
let bull_sharpe = regime_test.regime_performance
    .iter()
    .find(|r| r.regime_type == RegimeType::Bull)
    .map(|r| r.sharpe)
    .unwrap_or(0.0);
let bear_sharpe = regime_test.regime_performance
    .iter()
    .find(|r| r.regime_type == RegimeType::Bear)
    .map(|r| r.sharpe)
    .unwrap_or(0.0);

// Only deploy if both are decent
if bull_sharpe > 0.5 && bear_sharpe > 0.0 {
    println!("Deploy: Works in both regimes");
} else {
    println!("Hold: Poor performance in {:?} market", 
        if bull_sharpe < 0.5 { "bull" } else { "bear" });
}
```

### Pitfall 3: Not Checking Outlier Dependency

**Problem**: Strategy looks great due to 2-3 lucky trades.

**Solution**: Always check outlier impact:
```rust
let robustness = report.robustness.unwrap();
let outlier_dep = robustness.outlier.outlier_dependency;
if outlier_dep > 0.5 {
    println!("⚠ WARNING: {:.0}% of performance from outliers - fragile!",
        outlier_dep * 100.0);
    println!("   Consider: Reduce position size, add stop losses, improve win rate");
}
```

### Pitfall 4: Using Wrong Timeframes

**Problem**: Validating 1-hour strategy on 1-day data.

**Solution**: Match validation timeframe to strategy timeframe:
```rust
// Wrong
let config = ValidationConfig {
    interval: "1d".to_string(),  // Using daily bars
    symbol: "BTCUSDT".to_string(),
    // ... strategy is 1-hour ...
};

// Right
let config = ValidationConfig {
    interval: "1h".to_string(),  // Using hourly bars
    symbol: "BTCUSDT".to_string(),
    // ... matches strategy ...
};
```

---

## Troubleshooting

### Issue: Statistical Significance Returns Empty Results

**Symptoms**: `report.statistical_significance` is `None`

**Possible Causes**:

1. **Insufficient Trades**
   - Need 100+ trades for meaningful statistics
   - Fix: Validate on longer period or combine data

2. **All Trades Have Same Return**
   - Variance = 0, Sharpe calculation fails
   - Fix: Check strategy logic - should have varied outcomes

3. **Timeout Exceeded**
   - `statistical_timeout_seconds` too low for large datasets
   - Fix: Increase timeout or reduce `max_statistical_iterations`

**Debug Steps**:
```rust
// Check number of trades
println!("Trade count: {}", report.backtest.total_trades);
if report.backtest.total_trades < 100 {
    println!("⚠ Too few trades for statistical significance");
}

// Check for zero variance
let returns: Vec<f64> = report.backtest.trades
    .iter()
    .map(|t| (t.exit_price - t.entry_price) / t.entry_price)
    .collect();
let variance = calculate_variance(&returns);
if variance < 1e-10 {
    println!("⚠ Zero variance - all returns are identical");
}
```

### Issue: Regime Testing Detects No Regimes

**Symptoms**: `report.regime_testing.regimes` is empty

**Possible Causes**:

1. **Insufficient History**
   - Need multiple market cycles (3+ years preferred)
   - Fix: Extend validation period

2. **Stable Market Period**
   - No clear regime changes during validation period
   - Fix: Validate during more volatile period

3. **All Data in Single Regime**
   - Validation period only covered one market type
   - Fix: Use longer period with multiple cycles

**Debug Steps**:
```rust
// Check validation period
let days = (report.test_period.end - report.test_period.start).num_days();
println!("Validation period: {} days", days);
if days < 365 {
    println!("⚠ Less than 1 year - may not capture multiple regimes");
}

// Check regimes detected
let regimes = report.regime_testing.as_ref()
    .map(|r| &r.regimes)
    .unwrap_or(&vec![]);
println!("Regimes detected: {}", regimes.len());
if regimes.is_empty() {
    println!("⚠ No regimes - use longer validation period");
}
```

### Issue: Poor Stability Rating

**Symptoms**: `temporal.rolling_stability.rating` is `Poor` or `VeryPoor`

**Possible Causes**:

1. **Regime Changes in Data**
   - Strategy performs well in one regime, poorly in another
   - Fix: Use regime testing to understand transitions

2. **Overfitted Parameters**
   - Too many parameters for available data
   - Fix: Simplify strategy, reduce parameters

3. **Lookahead Bias**
   - Strategy uses future data inadvertently
   - Fix: Check for data leakage, use proper train/test split

**Debug Steps**:
```rust
let stability = report.temporal_validation.as_ref()
    .unwrap().rolling_stability;

println!("Stability CV: {:.2}", stability.cv_sharpe);
println!("Sharpe range: [{:.2}, {:.2}]", 
    stability.min_sharpe, stability.max_sharpe);

// Check for regime changes
if let Some(regime_test) = &report.regime_testing {
    println!("Regime changes: {}", regime_test.transitions.len());
    if regime_test.transitions.len() > 0 {
        println!("Strategy adapts to regime changes: {}",
            regime_test.transitions.iter().all(|t| t.impact_score > 0.0));
    }
}
```

### Issue: High Complexity Penalty

**Symptoms**: `robustness.complexity.rating` is `Poor` or `VeryPoor`

**Possible Causes**:

1. **Too Many Parameters**
   - Strategy has 10+ tunable parameters
   - Fix: Fix parameters, reduce to essential ones

2. **Too Many Indicators**
   - Strategy uses 8+ technical indicators
   - Fix: Remove redundant indicators, use composite

3. **Complex Decision Logic**
   - Many nested if/else statements or switches
   - Fix: Simplify logic, use state machine

**Debug Steps**:
```rust
let complexity = report.robustness.as_ref()
    .unwrap().complexity;

println!("Parameters: {}/{} (max)", 
    complexity.n_parameters, max_parameters);
println!("Indicators: {}/{} (max)", 
    complexity.n_indicators, max_indicators);
println!("Branches: {}/{} (max)", 
    complexity.n_branches, max_branches);

if complexity.rating == ComplexityRating::Poor {
    println!("⚠ Overfitting risk: Reduce parameters or indicators");
    
    // Suggest which to reduce
    if complexity.n_parameters > max_parameters * 8 / 10 {
        println!("  → Reduce parameters to ~{}", max_parameters * 8 / 10);
    }
    if complexity.n_indicators > max_indicators * 8 / 10 {
        println!("  → Remove 1-2 indicators");
    }
}
```

---

## Best Practices

### 1. Always Run All Phase 13 Validations

Don't cherry-pick validations - run all of them:

```rust
// Don't do this
if config.enable_statistical_significance {
    let stat_sig = validate_statistical_significance(...);
}

// Do this instead
let report = validator.validate(strategy, symbol, &bars)?;
// All Phase 13 validations run automatically
```

### 2. Use Production-Like Configuration

Don't use settings that won't work in production:

```rust
// Don't do this
let config = ValidationConfig {
    initial_capital: 1000000.0,  // Unrealistic
    fee_rate: 0.0,             // No fees - unrealistic
    // ...
};

// Do this instead
let config = ValidationConfig {
    initial_capital: 10000.0,   // Realistic
    fee_rate: 0.001,             // Real exchange fees
    // ...
};
```

### 3. Monitor Phase 13 Metrics in Production

Don't just validate once - monitor ongoing:

```rust
// Calculate Phase 13 metrics from live trades
let live_trades = fetch_live_trades(strategy_name, last_30_days);
let live_stat_sig = validate_statistical_significance(
    &live_trades, &live_metrics, risk_free_rate, None, None)?;

// Compare to backtest
let backtest_sharpe = report.backtest.metrics.sharpe_ratio;
let live_sharpe = live_metrics.sharpe_ratio;
let degradation = (backtest_sharpe - live_sharpe) / backtest_sharpe.abs();

if degradation > 0.2 {
    println!("⚠ WARNING: {:.0}% degradation - investigate!", 
        degradation * 100.0);
}
```

### 4. Keep Validation Scripts Versioned

Track which validation results correspond to which strategy version:

```rust
let validation_output = serde_json::to_string_pretty(&report)?;
let filename = format!("validation_{}_{}.json", 
    strategy_name, 
    Utc::now().format("%Y%m%d_%H%M%S"));
std::fs::write(&filename, validation_output)?;

// Store in git to track changes
Command::new("git")
    .args(&["add", &filename])
    .status()?;
```

### 5. Use Walk-Forward for Final Validation

Phase 13 is great, but walk-forward is still the gold standard:

```rust
// Use walk-forward as final gate
let report = validator.validate_with_factory(
    strategy_factory, symbol, &bars)?;

// Check walk-forward performance
let wf = &report.walk_forward;
if wf.stability_score < 0.6 {
    println!("⚠ Walk-forward unstable - don't deploy");
    return Err("Unstable walk-forward performance".into());
}

// THEN check Phase 13 metrics
if let Some(regime_test) = &report.regime_testing {
    let survived = regime_test.stress_tests
        .iter()
        .filter(|s| s.survived)
        .count();
    if survived < regime_test.stress_tests.len() * 8 / 10 {
        println!("⚠ Failed stress tests - improve robustness");
    }
}
```

---

## Quick Reference

### Validation Checklist

Before deploying a strategy, ensure:

- [ ] **Statistical Significance**: p-value < 0.05, CI width < 1.0
- [ ] **Regime Testing**: Survived all stress tests, regime Sharpe variance < 0.5
- [ ] **Temporal Validation**: Stability rating Good or better
- [ ] **Robustness**: Complexity rating Good or better, outlier dependency < 30%
- [ ] **Overall Score**: 70+ (B grade or better)
- [ ] **Walk-Forward**: Stability score > 0.6
- [ ] **Configuration**: Uses production-like settings (realistic capital, fees)

### Metric Thresholds

| Metric | Excellent | Good | Acceptable | Poor | Very Poor |
|---------|-----------|-------|-------------|--------|------------|
| Bootstrap CI Width | < 0.3 | < 0.5 | < 1.0 | < 1.5 | ≥ 1.5 |
| Permutation P-value | < 0.01 | < 0.05 | < 0.10 | < 0.20 | ≥ 0.20 |
| Sharpe Significance | p < 0.01 | p < 0.05 | p < 0.10 | p < 0.20 | ≥ 0.20 |
| Regime Sharpe Variance | < 0.3 | < 0.5 | < 0.8 | < 1.2 | ≥ 1.2 |
| Transition Impact | > 0.3 | > 0.0 | > -0.3 | > -0.6 | ≤ -0.6 |
| Stability CV | < 0.2 | < 0.3 | < 0.5 | < 0.7 | ≥ 0.7 |
| Expanding Window Degradation | < 5% | < 15% | < 30% | < 50% | ≥ 50% |
| Perturbation Degradation | < 10% | < 20% | < 35% | < 50% | ≥ 50% |
| Outlier Dependency | < 20% | < 30% | < 40% | < 50% | ≥ 50% |
| Complexity Rating | Excellent | Good | Moderate | Poor | Very Poor |

### Common Commands

```bash
# Validate strategy with all Phase 13 tests
cargo run --bin validate_strategy \
  --strategy GoldenCross \
  --symbol BTCUSDT \
  --interval 1d

# Check validation results
cat validation_results.json | jq '.overall_score, .grade, .verdict'

# View statistical significance
cat validation_results.json | jq '.statistical_significance'

# View regime testing
cat validation_results.json | jq '.regime_testing'

# View robustness
cat validation_results.json | jq '.robustness'

# View temporal validation
cat validation_results.json | jq '.temporal_validation'
```

---

## Summary

Phase 13 provides institutional-grade validation capabilities that help you:

1. **Prove Results Are Real**: Statistical significance testing
2. **Understand Market Context**: Regime-aware performance analysis
3. **Ensure Stability**: Temporal validation for consistency
4. **Detect Overfitting**: Robustness metrics for fragility

**Key Takeaways**:

- Always run all four validations before deployment
- Pay attention to regime-specific performance
- Don't rely solely on backtest Sharpe
- Monitor Phase 13 metrics in production
- Use walk-forward as final validation gate

**Next Steps**:

1. Run Phase 13 validations on all your strategies
2. Fix strategies that fail key metrics
3. Implement monitoring for Phase 13 metrics in production
4. Track validation results over time to detect strategy drift

**For More Information**:

- Implementation Summary: `implementation-summary.md`
- Quick Reference: `quick-reference.md`
- API Documentation: See inline Rust docs for each module

---

**Last Updated**: January 2026  
**Version**: 1.0.0