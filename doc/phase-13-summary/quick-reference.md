# Phase 13 Quick Reference: Advanced Validation Techniques

**Status**: Complete | **Date**: January 2026

---

## 🎯 What Phase 13 Delivered

Four advanced validation modules that provide institutional-grade statistical analysis:

| Module | Key Function | Primary Use Case |
|--------|-------------|------------------|
| **Statistical Significance** | `sharpe_significance()`, `bootstrap_sharpe()` | Prove results aren't luck |
| **Regime Testing** | `detect_regimes()`, `test_regime_specific()` | Understand market conditions |
| **Temporal Validation** | `analyze_expanding_windows()`, `analyze_rolling_stability()` | Ensure stability over time |
| **Robustness** | `calculate_complexity()`, `analyze_outliers()` | Detect overfitting |

---

## 🚀 Quick Start Examples

### Statistical Significance

```rust
use alphafield_backtest::validation::statistical_significance::*;

// Test if Sharpe is significant (not just luck)
let significance = sharpe_significance(1.5, &returns);
if significance.is_significant {
    println!("Sharpe is significant! p={:.4}", significance.p_value);
}

// Get confidence interval via bootstrap
let bootstrap = bootstrap_sharpe(1.5, &trades, 1000, 0.02);
println!("95% CI: [{:.2}, {:.2}]", bootstrap.ci_lower, bootstrap.ci_upper);
```

### Regime Testing

```rust
use alphafield_backtest::validation::regime_testing::*;

// Detect market regimes
let regimes = detect_regimes(&returns, 30);

// Test performance per regime
let regime_perf = test_regime_specific(&trades, &regimes, 0.02);
for rp in &regime_perf {
    println!("{:?}: Sharpe={:.2}, WinRate={:.1}%", 
        rp.regime_type, rp.sharpe, rp.win_rate * 100.0);
}

// Predict next regime
let pred = predict_regime(RegimeType::Bull, &regimes);
println!("Predicted: {:?} ({:.0}% confidence)", 
    pred.predicted_regime, pred.confidence * 100.0);
```

### Temporal Validation

```rust
use alphafield_backtest::validation::temporal::*;

// Expanding window: detect overfitting
let windows = analyze_expanding_windows(&trades, 100, 0.02);
for w in &windows {
    println!("Window {}: Sharpe={:.2}", w.window_size, w.cumulative_sharpe);
}

// Rolling stability: measure consistency
let stability = analyze_rolling_stability(&trades, 252, 0.02);
println!("Stability Rating: {:?}", stability.rating); // Excellent to VeryPoor
```

### Robustness

```rust
use alphafield_backtest::validation::robustness::*;

// Complexity penalty: simpler is better
let complexity = calculate_complexity(5, 3, 10);
println!("Complexity: {:?} (penalty={:.2})", 
    complexity.rating, complexity.penalty_score);

// Outlier dependency: should be < 30%
let outliers = analyze_outliers(&trades, 0.02);
println!("Outlier dependency: {:.1}%", 
    outliers.outlier_dependency * 100.0);
```

---

## 📊 Key Metrics & Thresholds

### Statistical Significance
- **P-value < 0.05**: Statistically significant
- **Bootstrap CI**: Narrower = more precise
- **Permutation P-value < 0.05**: Not due to chance

### Regime Testing
- **Regime Confidence**: 70-80% accuracy typical
- **Regime Sharpe Variance**: Lower = more consistent
- **Transition Impact Score**: Higher = more risky

### Temporal Validation
- **Stability Rating**: 
  - Excellent: CV < 0.2
  - Good: CV < 0.3
  - Moderate: CV < 0.4
- **Forward-Looking Degradation**: Target < 20%

### Robustness
- **Complexity Penalty**: 0-1 scale, lower is better
- **Perturbation Degradation**: Target < 20% at 5% noise
- **Outlier Dependency**: Target < 30%

---

## 🔧 Common Patterns

### Running All Validations Together

```rust
use alphafield_backtest::validation::*;

let result = ValidationResult {
    // Core validations
    backtest: backtest_result,
    walk_forward: wf_result,
    monte_carlo: mc_result,
    regime: regime_result,
    
    // Phase 13 validations
    statistical_significance: Some(stat_sig_result),
    regime_testing: Some(regime_test_result),
    temporal_validation: Some(temporal_result),
    robustness: Some(robustness_result),
    
    config: validation_config,
};

// Get overall score and recommendations
let score = ScoreCalculator::default().calculate(&result);
let recommendation = RecommendationsGenerator::default().generate(&result);
```

### Handling Missing Data

```rust
// Optional fields for partial data
let result = ValidationResult {
    // ...
    statistical_significance: Some(stat_sig),  // If trades available
    regime_testing: Some(regime_test),         // If regime data available
    temporal_validation: Some(temporal),       // If sufficient history
    robustness: Some(robustness),              // Always available
    // ...
};
```

---

## 🐛 Bug Fixes Applied

| File | Issue | Fix |
|------|-------|-----|
| `regime_testing.rs:267` | `current` undefined → `current_regime_type` |
| `regime_testing.rs:595` | `current_regime` undefined → `start_time_regime` |
| `regime_testing.rs:401` | Duplicate `.num_days()` call → remove duplicate |
| `regime_testing.rs:410` | Duplicate `.num_days()` call → remove duplicate |
| `statistical_significance.rs:378` | Unused `variance` → removed |
| `statistical_significance.rs:359` | Unused `risk_free_rate` param → removed |

---

## 📚 Module Files

- `statistical_significance.rs` - Bootstrap, permutation, ADF, Sharpe, correlation
- `regime_testing.rs` - Regime detection, performance, transitions, prediction
- `temporal.rs` - Expanding windows, rolling stability, periods, cycles
- `robustness.rs` - Complexity, perturbation, outliers
- `scoring.rs` - Integrated scoring with Phase 13 weights

---

## ⚡ Performance Notes

- **Bootstrap**: O(iterations × n), 1000 iterations typical (~1s for 1000 trades)
- **Permutation**: O(permutations × n), 1000 permutations typical (~1s)
- **Regime Detection**: O(n), very fast (~10ms)
- **Rolling Stability**: O(n × window), 252 bars typical (~100ms)
- **Correlation**: O(m² × n), 10 strategies typical (~500ms)

---

## 🎖️ Key Achievements

✅ **Statistical Rigor**: Prevents false positives  
✅ **Regime Awareness**: Understands market conditions  
✅ **Temporal Stability**: Ensures consistency over time  
✅ **Robustness**: Identifies overfitting  
✅ **Zero Warnings**: Clean build, production-ready  
✅ **Comprehensive Docs**: Full API documentation + examples  

---

**For detailed documentation**: See `implementation-summary.md` in this directory.