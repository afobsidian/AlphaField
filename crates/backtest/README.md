# 🧪 AlphaField Backtest Crate

Event-driven backtesting engine and advanced analysis tools.

## ⚙️ Core Components

- **BacktestEngine**: Simulates market replay bar-by-bar.
- **Portfolio**: Tracks cash, positions, equity, and margin.
- **ExchangeSimulator**: Handles order matching, slippage, and fee modeling.

## 📊 Advanced Analysis

This crate includes sophisticated tools for strategy validation:

### Walk Forward Analysis
Validates robustness by training and testing on rolling time windows to detect overfitting.

### Monte Carlo Simulation
Randomizes trade sequencing to estimate worst-case drawdowns and ruin probability.

### Sensitivity Analysis
Parameter grid search to identify stable parameter regions and avoid "curve fitting".

### Correlation Analysis
Calculates correlation matrices between multiple strategy equity curves to build diversified portfolios.

## 🚀 Example

```rust
let mut engine = BacktestEngine::new(100_000.0, 0.001, SlippageModel::FixedPercent(0.0005));
engine.add_data("BTC", bars);
engine.set_strategy(Box::new(my_strategy));

let metrics = engine.run()?;
println!("Sharpe Ratio: {}", metrics.sharpe_ratio);
```

## 🔧 Strategy Validation Binary

The `validate_strategy` binary provides a command-line interface for comprehensive strategy validation including backtesting, walk-forward analysis, Monte Carlo simulation, and regime-based performance analysis.

### Batch Validation

Validate multiple strategies against multiple symbols in parallel:

```bash
cargo run --bin validate_strategy -- batch \
  --batch-file validation/strategies_batch.txt \
  --symbols "BTC,ETH,SOL,BNB,XRP" \
  --interval 1h \
  --output-dir validation/reports \
  --format json \
  --max-concurrent 4
```

**Important**: Use `--max-concurrent` to limit parallel validation and prevent CPU overload. Each validation runs multiple CPU-intensive operations (backtest, walk-forward analysis, Monte Carlo simulation, regime analysis). 

Recommended values:
- Conservative: `--max-concurrent 2` (lowest CPU usage, slower execution)
- Balanced: `--max-concurrent 4` or `num_cores/2` (good balance of speed and CPU usage)
- Aggressive: `--max-concurrent num_cores` (default, fastest but highest CPU usage)

The batch file should contain one strategy name per line:

```
RSIReversion
GoldenCross
AdaptiveMA
MacdTrend
```

### List Available Strategies

View all registered strategies:

```bash
cargo run --bin validate_strategy -- list-strategies

# Filter by category
cargo run --bin validate_strategy -- list-strategies --category mean_reversion
```

### Validate Single Strategy

```bash
cargo run --bin validate_strategy -- validate \
  --strategy RSIReversion \
  --symbol BTC \
  --interval 1h \
  --output validation_report.json \
  --format json
```
