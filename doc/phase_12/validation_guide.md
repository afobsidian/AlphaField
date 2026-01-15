# Strategy Validation Guide

## Overview

The AlphaField validation framework provides automated, comprehensive assessment of trading strategy performance without requiring full dashboard integration. This enables rapid strategy evaluation through rigorous testing including:

- **Backtesting**: Historical performance simulation with realistic fills, slippage, and fees
- **Walk-Forward Analysis**: Rolling train/test validation across different market periods
- **Monte Carlo Simulation**: Trade sequence randomization to test robustness to luck
- **Regime Analysis**: Performance breakdown across different market conditions (bull/bear/sideways/volatile)
- **Risk Assessment**: Comprehensive risk profile analysis including tail risk and drawdown metrics

The framework produces an overall score (0-100) with letter grade (A-F), pass/fail verdict, and actionable recommendations for deployment or optimization.

## Quick Start

### Installation

The validation tool is built as part of the `alphafield-backtest` crate. Build and install:

```bash
cd AlphaField
cargo build --release --bin validate_strategy
```

The binary will be available at:
- `target/release/validate_strategy` (Linux/macOS)
- `target/release/validate_strategy.exe` (Windows)

### Basic Usage

Validate a single strategy with default settings:

```bash
./target/release/validate_strategy validate \
    --strategy golden_cross \
    --symbol BTC \
    --interval 1h \
    --data-file data/BTC_1h.csv
```

This will:
1. Load historical OHLCV data from the CSV file
2. Run backtest, walk-forward, Monte Carlo, and regime analysis
3. Display a comprehensive terminal report
4. Return exit code 0 (pass), 1 (fail/needs optimization), or 2 (error)

### CSV Data Format

Input data files should be in CSV format with the following columns:

```
timestamp,open,high,low,close,volume
1640995200,46000.00,47000.00,45500.00,46800.00,1234.56
1641081600,46800.00,47500.00,46000.00,47200.00,2345.67
...
```

- **timestamp**: Unix timestamp in seconds
- **open, high, low, close**: Price values
- **volume**: Trading volume

## CLI Commands

### Single Strategy Validation

Validate a strategy with custom thresholds:

```bash
./target/release/validate_strategy validate \
    --strategy golden_cross \
    --symbol BTC \
    --interval 4h \
    --data-file data/BTC_4h.csv \
    --min-sharpe 1.5 \
    --max-drawdown 0.20 \
    --min-win-rate 0.65 \
    --min-positive-probability 0.75 \
    --initial-capital 100000.0 \
    --fee-rate 0.001 \
    --risk-free-rate 0.03
```

#### Output Options

**Terminal output (default):**
```bash
--format terminal
```

**JSON output:**
```bash
--format json --output report.json
```

**YAML output:**
```bash
--format yaml --output report.yaml
```

**Markdown report:**
```bash
--format markdown --output report.md
```

### Batch Validation

Validate multiple strategies across multiple symbols:

```bash
./target/release/validate_strategy batch \
    --batch-file strategies.txt \
    --symbols BTC,ETH,SOL \
    --interval 1h \
    --data-dir data/ \
    --output-dir reports/ \
    --format json
```

**strategies.txt** format (one strategy per line):
```
golden_cross
rsi_strategy
macd_strategy
bollinger_bands
```

This will generate individual reports for each strategy/symbol combination:
- `reports/golden_cross_BTC.json`
- `reports/golden_cross_ETH.json`
- `reports/rsi_strategy_BTC.json`
- ...

## Understanding Validation Reports

### Overall Score (0-100)

The overall score is a weighted average of five validation components:

| Component | Weight |
|-----------|--------|
| Backtest | 30% |
| Walk-Forward | 25% |
| Monte Carlo | 20% |
| Regime Match | 15% |
| Risk Metrics | 10% |

**Score Interpretation:**
- **90-100 (A)**: Excellent - Ready for deployment with high confidence
- **80-89 (B)**: Good - Consider for deployment with monitoring
- **70-79 (C)**: Fair - Requires optimization before deployment
- **60-69 (D)**: Poor - Not recommended without major improvements
- **<60 (F)**: Fail - Reject strategy

### Component Scores

#### Backtest (30%)

Measures in-sample historical performance on the provided dataset.

**Key Metrics:**
- **Total Return**: Overall profit/loss percentage
- **Sharpe Ratio**: Risk-adjusted returns (higher is better)
- **Max Drawdown**: Largest peak-to-trough decline (lower is better)
- **Win Rate**: Percentage of profitable trades
- **Profit Factor**: Gross profit divided by gross loss (>1.0 indicates profitability)
- **Total Trades**: Number of executed trades

**Scoring:**
- Sharpe ratio: 0-30 points (scales to 3.0 maximum)
- Max drawdown: 0-30 points (penalizes exceeding threshold)
- Win rate: 0-20 points
- Total return: 0-20 points

**Interpretation:**
- High Sharpe (>2.0) with low drawdown (<20%) indicates excellent risk-adjusted returns
- Win rate above 60% is generally healthy for trend-following strategies
- Profit factor >2.0 suggests strong positive expectancy

#### Walk-Forward Analysis (25%)

Tests strategy robustness across different time periods using rolling train/test windows.

**Key Metrics:**
- **Stability Score**: Consistency of performance across windows (0-100%)
- **Mean Return**: Average return across out-of-sample test windows
- **Win Rate**: Percentage of profitable test windows
- **Median Return**: Typical return (less sensitive to outliers)

**Scoring:**
- Stability score: 0-40 points
- Win rate: 0-30 points
- Mean return: 0-30 points

**Interpretation:**
- Stability score >70% indicates the strategy performs consistently
- Win rate >60% across windows shows robustness to different market conditions
- High stability but low returns may indicate over-conservative parameters

#### Monte Carlo Simulation (20%)

Tests strategy robustness to randomness by shuffling trade sequences.

**Key Metrics:**
- **Positive Probability**: Percentage of simulations with positive returns
- **5th Percentile**: Worst-case scenario (95% of simulations are better)
- **50th Percentile**: Median outcome
- **95th Percentile**: Best-case scenario
- **Mean Return**: Average across all simulations

**Scoring:**
- Positive probability: 0-40 points
- Worst-case (5th percentile): 0-30 points
- Median return: 0-30 points

**Interpretation:**
- Positive probability >80% indicates low risk of ruin
- 5th percentile > -20% suggests acceptable worst-case scenario
- Wide spread between 5th and 95th percentiles indicates high path dependence

#### Regime Match (15%)

Analyzes strategy performance across different market regimes.

**Key Metrics:**
- **Regime Match Score**: Performance in expected regimes (0-100%)
- **Bull Market Performance**: Returns during uptrends
- **Bear Market Performance**: Returns during downtrends
- **Sideways Performance**: Returns in range-bound markets
- **High Volatility Performance**: Returns during turbulent periods
- **Low Volatility Performance**: Returns during calm periods
- **Regime Mismatch Warning**: Alerts if strategy performs best in unexpected regime

**Scoring:**
- Regime match score: 0-80 points
- Regime mismatch penalty: -20 points

**Interpretation:**
- High match score (>70%) with expected regimes indicates proper strategy design
- Mismatch warnings suggest strategy may be misclassified or needs adaptation
- Consider regime-switching strategies if performance varies significantly across regimes

#### Risk Metrics (10%)

Comprehensive risk assessment across multiple dimensions.

**Key Metrics:**
- **Expected Max Drawdown**: User-defined threshold
- **Actual Max Drawdown**: Observed maximum decline
- **Tail Risk**: 5th percentile return from Monte Carlo (worst 5% of outcomes)
- **Average Exposure**: Typical position size as % of capital
- **Leverage**: Effective leverage (should be 1.0 for spot-only trading)
- **Risk Rating**: Low, Medium, High, or Extreme

**Scoring:**
- Drawdown consistency: 0-30 points
- Volatility: 0-35 points (lower is better)
- Tail risk: 0-35 points

**Interpretation:**
- Actual drawdown close to or below expected is ideal
- Tail risk > -30% for most strategies is acceptable
- Leverage should remain at 1.0 for spot-only strategies

## Recommendations

The validation report provides three categories of recommendations:

### Strengths

Examples of strength indicators:
- "Excellent risk-adjusted returns (Sharpe: 2.50)"
- "High win rate (65.0%)"
- "Low maximum drawdown (12.0%)"
- "High out-of-sample stability (80%)"
- "Excellent performance in expected market regimes"

**Action**: These are your strategy's competitive advantages. Consider highlighting them when presenting results.

### Weaknesses

Examples of weakness indicators:
- "Low risk-adjusted returns (Sharpe: 0.80)"
- "Excessive maximum drawdown (35.0% exceeds threshold of 30.0%)"
- "Poor walk-forward win rate (45.0% below threshold of 60.0%)"
- "Low probability of positive returns (55.0% below threshold of 70.0%)"

**Action**: These areas need improvement before deployment. Address the most critical weaknesses first.

### Improvements

Actionable suggestions based on identified weaknesses:
- "Consider adding risk management to improve risk-adjusted returns"
- "Implement stricter stop-loss to reduce drawdown"
- "Consider position sizing based on volatility"
- "Add additional entry filters to improve win rate"
- "Optimize parameters for robustness across different market conditions"
- "Review strategy logic for regime-specific performance"
- "Consider regime detection and parameter switching"

**Action**: Implement these improvements sequentially and re-validate after each change to measure impact.

## Deployment Verdict

The validator provides three deployment recommendations:

### PASS

**Conditions:**
- Overall score ≥70
- No critical failures (Sharpe, drawdown, win rate, Monte Carlo probability)
- Stability score ≥60%

**Confidence Levels:**
- Grade A: 95% confidence
- Grade B: 80% confidence
- Grade C: 60% confidence

**Action**: Strategy is ready for deployment. Consider starting with paper trading or small position sizes to monitor live performance.

### OPTIMIZE THEN VALIDATE

**Conditions:**
- Overall score 60-69 OR
- One or more metrics below optimization thresholds:
  - Sharpe <1.2 × minimum
  - Stability score <60%
  - Regime mismatch detected

**Action**: Strategy shows promise but requires improvement:
1. Review and implement suggested improvements
2. Run parameter optimization workflow
3. Re-validate with optimized parameters
4. Repeat until PASS verdict achieved

### REJECT

**Conditions:**
- Any critical failure:
  - Sharpe < minimum threshold
  - Max drawdown > threshold × 1.5
  - Walk-forward win rate < threshold × 0.8
  - Monte Carlo positive probability < threshold × 0.8

**Action**: Strategy fails fundamental criteria. Consider:
- Revisiting the hypothesis
- Trying different asset classes or timeframes
- Exploring alternative strategy categories

## Advanced Usage

### Custom Weights

Modify the scoring weights via the validation framework API (requires Rust code):

```rust
use alphafield_backtest::validation::ScoreWeights;

let weights = ScoreWeights {
    backtest: 0.40,     // Emphasize backtest performance
    walk_forward: 0.20, // Reduce walk-forward weight
    monte_carlo: 0.20,
    regime_match: 0.10, // Reduce regime weight
    risk_metrics: 0.10,
};
```

### Integration with Strategy Registry

Validate all strategies in the registry:

```rust
use alphafield_backtest::validation::StrategyValidator;
use alphafield_strategy::registry::StrategyRegistry;

let registry = StrategyRegistry::default();
let validator = StrategyValidator::new(config);

for strategy_name in registry.list_all() {
    let strategy = registry.get(&strategy_name).unwrap();
    let report = validator.validate(strategy, &bars)?;
    // Process report...
}
```

### Automated Validation Pipeline

Create a CI/CD pipeline that automatically validates strategies:

```yaml
# .github/workflows/validate.yml
name: Strategy Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Build validator
        run: cargo build --release --bin validate_strategy
      - name: Validate strategies
        run: |
          ./target/release/validate_strategy batch \
            --batch-file strategies.txt \
            --symbols BTC,ETH \
            --interval 1h \
            --data-dir test_data/ \
            --output-dir reports/ \
            --format json
      - name: Upload reports
        uses: actions/upload-artifact@v2
        with:
          name: validation-reports
          path: reports/
```

### Custom Data Sources

Load data from databases or APIs by converting to the expected CSV format:

```python
# Python example: Fetch from database and export to CSV
import pandas as pd
import psycopg2

conn = psycopg2.connect("postgresql://user:pass@localhost/5432/alphafield")
query = """
    SELECT 
        EXTRACT(EPOCH FROM timestamp)::bigint as timestamp,
        open, high, low, close, volume
    FROM candles
    WHERE symbol = 'BTC' AND interval = '1h'
    ORDER BY timestamp
"""
df = pd.read_sql(query, conn)
df.to_csv('BTC_1h.csv', index=False)
```

## Interpreting Common Issues

### High Sharpe but Fails Walk-Forward

**Symptom**: Backtest Sharpe >2.0 but walk-forward stability <50%

**Diagnosis**: Strategy is overfitted to specific historical patterns. Parameters work well in-sample but degrade out-of-sample.

**Solutions**:
- Reduce parameter complexity (fewer tunable parameters)
- Increase regularization in optimization
- Use more conservative default parameters
- Consider simpler strategy logic

### Good Returns but High Drawdown

**Symptom**: Total return >50% but max drawdown >40%

**Diagnosis**: Strategy may be taking excessive risk. Returns are driven by a few large wins but suffers significant losses.

**Solutions**:
- Implement tighter stop-loss levels
- Reduce position sizing during high volatility
- Add drawdown-based risk management (reduce exposure during drawdowns)
- Consider trend filters to avoid trading against strong trends

### Fails Regime Analysis

**Symptom**: Regime mismatch warning indicates strategy performs best in unexpected market type

**Diagnosis**: Strategy classification or hypothesis may be incorrect. Strategy designed for trending markets but actually works best in ranging markets (or vice versa).

**Solutions**:
- Re-evaluate strategy hypothesis
- Consider reclassifying strategy in registry
- Add regime detection to switch parameters dynamically
- Test strategy on different assets/timeframes

### Low Monte Carlo Positive Probability

**Symptom**: Positive probability <60% despite decent backtest

**Diagnosis**: Strategy is highly path-dependent. Results depend heavily on the specific sequence of trades, making outcomes unpredictable.

**Solutions**:
- Reduce leverage/position sizing
- Add diversification across assets or strategies
- Implement risk management to cap maximum loss
- Focus on strategies with more consistent, predictable returns

### Excellent Backtest, Terrible Live Performance

**Symptom**: Backtest shows Grade A performance but live paper trading underperforms

**Diagnosis**: Common issues:
- Look-ahead bias (using future data in backtest)
- Unrealistic fill assumptions (ignoring slippage, liquidity)
- Survivorship bias (testing only surviving assets)
- Data quality issues (gaps, outliers)

**Solutions**:
- Review backtest implementation for bias
- Increase slippage/fee assumptions
- Test on delisted assets
- Use out-of-sample validation
- Implement robust data quality checks

## Best Practices

### 1. Start with Rigorous Hypotheses

Before validating, ensure your strategy has a well-defined hypothesis:
- Clear entry/exit rules
- Expected market regimes
- Risk profile and drawdown expectations
- Performance targets (minimum acceptable Sharpe, max drawdown)

### 2. Use Multiple Assets and Timeframes

Validate across:
- Different assets (BTC, ETH, SOL)
- Different timeframes (1h, 4h, 1d)
- Different market periods (bull, bear, crash)

A robust strategy should perform well across various conditions.

### 3. Prioritize Robustness Over Maximum Returns

Prefer strategies with:
- Consistent performance (high stability score)
- Low parameter sensitivity
- Acceptable returns with low drawdowns

Over high-return, fragile strategies that:
- Excel in one specific period but fail elsewhere
- Are highly sensitive to parameter changes
- Have extreme drawdowns despite high returns

### 4. Iterative Improvement Cycle

1. **Initial Validation**: Establish baseline performance
2. **Identify Weaknesses**: Focus on lowest-scoring components
3. **Targeted Optimization**: Improve specific areas (entry filters, risk management, etc.)
4. **Re-Validate**: Measure improvement with same data
5. **Out-of-Sample Test**: Validate on unseen data
6. **Deploy and Monitor**: Track live performance vs expectations

### 5. Document Everything

For each validation run, save:
- Report in machine-readable format (JSON/YAML)
- Parameter configuration
- Data period and source
- Hypothesis and expected performance
- Changes made and rationale

This enables reproducibility and learning from both successes and failures.

## Troubleshooting

### "No historical data provided" Error

**Cause**: Empty data file or failed to parse CSV format.

**Solution**:
- Verify CSV file contains data
- Check CSV format (timestamp, open, high, low, close, volume columns)
- Ensure timestamp is in seconds (not milliseconds)
- Verify file path is correct

### "Insufficient data for walk-forward" Warning

**Cause**: Not enough bars for rolling window analysis.

**Solution**:
- Use longer historical period
- Reduce window sizes (train_window, test_window)
- Use larger interval (e.g., 4h instead of 1h)

### Validation Takes Too Long

**Cause**: Large dataset or many simulations.

**Solution**:
- Reduce Monte Carlo simulations (default 1000)
- Use larger intervals to reduce bar count
- Disable specific components (e.g., `--walk-forward false`)
- Use subset of data for initial testing

### Memory Errors

**Cause**: Loading very large datasets into memory.

**Solution**:
- Process data in chunks (requires custom implementation)
- Use larger intervals to reduce data size
- Limit validation period to recent history
- Run on machine with more RAM

## API Reference

### ValidationConfig

Configuration for validation runs:

```rust
pub struct ValidationConfig {
    pub data_source: String,        // Database connection or file path
    pub symbol: String,              // Symbol being validated
    pub interval: String,            // Timeframe (1h, 4h, 1d)
    pub walk_forward: WalkForwardConfig,
    pub risk_free_rate: f64,         // For Sharpe calculation
    pub thresholds: ValidationThresholds,
    pub initial_capital: f64,
    pub fee_rate: f64,
}
```

### ValidationThresholds

Pass/fail thresholds:

```rust
pub struct ValidationThresholds {
    pub min_sharpe: f64,                  // Default: 1.0
    pub max_drawdown: f64,                // Default: 0.30
    pub min_win_rate: f64,                // Default: 0.60
    pub min_positive_probability: f64,      // Default: 0.70
}
```

### ValidationReport

Comprehensive validation output:

```rust
pub struct ValidationReport {
    pub strategy_name: String,
    pub validated_at: DateTime<Utc>,
    pub test_period: TestPeriod,
    pub overall_score: f64,       // 0-100
    pub grade: char,              // A-F
    pub verdict: ValidationVerdict,
    pub backtest: BacktestResult,
    pub walk_forward: WalkForwardResult,
    pub monte_carlo: MonteCarloResult,
    pub regime_analysis: RegimeAnalysisResult,
    pub risk_assessment: RiskAssessment,
    pub recommendations: Recommendations,
}
```

## Examples

### Example 1: Golden Cross Validation

```bash
./target/release/validate_strategy validate \
    --strategy golden_cross \
    --symbol BTC \
    --interval 1d \
    --data-file data/BTC_1d.csv \
    --format terminal
```

**Expected Results**:
- Score: 75-85 (trend-following strategies typically perform well in strong trends)
- Grade: B
- Verdict: Optimize Then Validate or Pass
- Strengths: Strong performance in bull markets
- Weaknesses: Poor performance in ranging markets

### Example 2: RSI Strategy Validation

```bash
./target/release/validate_strategy validate \
    --strategy rsi_strategy \
    --symbol ETH \
    --interval 4h \
    --data-file data/ETH_4h.csv \
    --min-sharpe 1.2 \
    --max-drawdown 0.25 \
    --format json --output rsi_eth_validation.json
```

**Expected Results**:
- Score: 65-80 (depends on market conditions)
- Grade: C or B
- Verdict: NeedsOptimization
- Strengths: Good win rate in ranging markets
- Weaknesses: Drawdowns in strong trending markets

### Example 3: Batch Validation Dashboard Integration

```bash
# Validate all strategies across top 10 assets
./target/release/validate_strategy batch \
    --batch-file strategies.txt \
    --symbols BTC,ETH,SOL,ADA,DOT,AVAX,MATIC,ATOM,LINK,UNI \
    --interval 1h \
    --data-dir data/ \
    --output-dir dashboard/reports/ \
    --format json

# Generate summary report
python scripts/summarize_validations.py \
    --input-dir dashboard/reports/ \
    --output dashboard/summary.html
```

## FAQ

**Q: How much historical data do I need for validation?**

A: Minimum 1000 bars, ideally 2000+. For walk-forward analysis, you need enough data for multiple train/test windows. Daily data requires at least 2-3 years; hourly data requires 3-6 months.

**Q: What score is considered "good enough" for deployment?**

A: Grade B (80-89) or higher is recommended. Grade C (70-79) may be acceptable with tight risk controls and monitoring. Avoid deploying Grade D or F strategies.

**Q: Can I use the validation framework for live trading?**

A: The validation framework is for backtesting and analysis. For live trading, use the `execution` crate with real-time data and order execution. Use validation results to guide deployment decisions.

**Q: How do I handle strategies that require external data (e.g., sentiment indicators)?**

A: Currently, the validation framework only uses OHLCV data. For sentiment-based strategies, pre-compute sentiment values and include them in your data file as additional columns, then modify the strategy to read from those columns.

**Q: Can I validate custom strategies not in the standard library?**

A: Yes. Implement the `Strategy` trait from `alphafield-core`, then use the `StrategyValidator` API directly in Rust code, or extend the CLI tool to include your custom strategy.

**Q: What's the difference between backtest Sharpe and walk-forward Sharpe?**

A: Backtest Sharpe is calculated on the entire dataset (in-sample). Walk-forward Sharpe is the average Sharpe across out-of-sample test windows only. Walk-forward Sharpe is more indicative of real-world performance.

**Q: How do I interpret the "Regime Mismatch" warning?**

A: This warning appears when your strategy performs best in a market regime different from what it's designed for. For example, a "trend following" strategy that actually performs best in ranging markets. This suggests your hypothesis may be incorrect or the strategy needs reclassification.

## Additional Resources

- [Phase 12 Plan](phase_12_plan.md) - Detailed implementation roadmap
- [Architecture Documentation](../architecture.md) - System design overview
- [Optimization Workflow](../optimization_workflow.md) - Parameter optimization guide
- [ML Documentation](../ml.md) - Machine learning integration
- [API Reference](../api.md) - Complete API documentation

## Support and Feedback

For issues, questions, or contributions:
- GitHub Issues: [Report bugs or request features](https://github.com/alphafield/alphafield/issues)
- Documentation: [AlphaField Docs](https://docs.alphafield.io)
- Community: Join our [Discord](https://discord.gg/alphafield)

---

**Last Updated**: 2026-01-15
**Version**: 1.0.0
**Framework**: AlphaField Validation Framework v12.7