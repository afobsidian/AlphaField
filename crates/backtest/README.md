# 🧪 AlphaField Backtest Crate

Event-driven backtesting engine and advanced analysis tools.

## Why This Crate Exists

This crate provides a robust backtesting framework to validate trading strategies before risking real capital. By simulating historical market conditions, we enable:

- **Strategy validation**: Test strategies against historical data to assess viability
- **Risk management**: Evaluate drawdown, volatility, and ruin probability
- **Parameter optimization**: Find optimal strategy parameters without overfitting
- **Portfolio construction**: Analyze correlations between multiple strategies

## ⚙️ Core Components

### BacktestEngine

Simulates market replay bar-by-bar, processing trades and tracking portfolio performance.

```rust
use alphafield_backtest::{BacktestEngine, SlippageModel};
use alphafield_core::{Bar, Strategy};
use rust_decimal::Decimal;

// Create engine with initial capital
let mut engine = BacktestEngine::new(
    Decimal::from(100_000),  // Initial capital
    Decimal::from_str("0.001")?,  // Fee rate (0.1%)
    SlippageModel::FixedPercent(Decimal::from_str("0.0005")?),  // 0.05% slippage
);

// Load historical data
let bars: Vec<Bar> = load_bars_from_db()?;

// Set strategy
engine.add_data("BTCUSDT", bars);
engine.set_strategy(Box::new(my_strategy));

// Run backtest
let metrics = engine.run()?;

// Analyze results
println!("Total Return: {}", metrics.total_return);
println!("Sharpe Ratio: {}", metrics.sharpe_ratio);
println!("Max Drawdown: {}", metrics.max_drawdown);
println!("Win Rate: {}", metrics.win_rate);
println!("Profit Factor: {}", metrics.profit_factor);
```

### Portfolio

Tracks cash, positions, equity, and margin throughout the backtest.

```rust
use alphafield_backtest::Portfolio;
use rust_decimal::Decimal;

// Access portfolio state
let portfolio = engine.portfolio();

println!("Cash: {}", portfolio.cash());
println!("Equity: {}", portfolio.equity());
println!("Net Worth: {}", portfolio.net_worth());
println!("Positions: {}", portfolio.positions().len());

// Get position for a symbol
if let Some(position) = portfolio.get_position("BTCUSDT") {
    println!("Quantity: {}", position.quantity);
    println!("Entry Price: {}", position.entry_price);
    println!("Unrealized P&L: {}", position.unrealized_pnl());
}
```

### ExchangeSimulator

Handles order matching, slippage, and fee modeling.

```rust
use alphafield_backtest::{ExchangeSimulator, OrderType, OrderSide};
use rust_decimal::Decimal;

// Simulator is created by BacktestEngine
// Configure slippage model
let slippage = SlippageModel::FixedPercent(Decimal::from_str("0.001")?);  // 0.1%

// Or use more sophisticated models
let slippage = SlippageModel::VolumeBased {
    base_slippage: Decimal::from_str("0.0005")?,
    volume_impact: Decimal::from_str("0.00001")?,
};

// Orders are executed by the simulator
let result = simulator.execute_order(&order)?;
println!("Execution Price: {}", result.price);
println!("Execution Time: {}", result.timestamp);
```

## 📊 Advanced Analysis

### Walk Forward Analysis

Validates robustness by training and testing on rolling time windows to detect overfitting.

**Why Walk Forward?** Prevents curve fitting by testing on out-of-sample data.

```rust
use alphafield_backtest::WalkForwardAnalyzer;

let analyzer = WalkForwardAnalyzer::new(
    my_strategy,          // Strategy to analyze
    Duration::days(30),   // Training window (1 month)
    Duration::days(10),   // Test window (10 days)
    Duration::days(10),   // Step size (rolling window)
);

// Run walk-forward analysis
let results = analyzer.run(&bars)?;

// Analyze stability
let avg_return = results.iter().map(|r| r.total_return).sum::<f64>() / results.len() as f64;
let return_std = calculate_std_dev(&results.iter().map(|r| r.total_return).collect::<Vec<_>>());

println!("Average Return: {}", avg_return);
println!("Return Std Dev: {}", return_std);
println!("Stability Ratio: {}", avg_return / return_std);

// Stable strategies have low standard deviation across test windows
```

### Monte Carlo Simulation

Randomizes trade sequencing to estimate worst-case drawdowns and ruin probability.

**Why Monte Carlo?** Estimates tail risk and worst-case scenarios not visible in single backtest.

```rust
use alphafield_backtest::MonteCarloSimulator;

let simulator = MonteCarloSimulator::new(1000);  // 1000 simulations

// Run Monte Carlo on backtest trades
let results = simulator.run(&trades)?;

// Analyze results
let worst_drawdown_5pct = percentile(&results, 5.0);  // 5th percentile
let worst_drawdown_1pct = percentile(&results, 1.0);  // 1st percentile
let median_drawdown = percentile(&results, 50.0);

println!("Median Drawdown: {}", median_drawdown);
println!("Worst 5% Drawdown: {}", worst_drawdown_5pct);
println!("Worst 1% Drawdown: {}", worst_drawdown_1pct);

// Calculate ruin probability
let ruin_prob = results.iter().filter(|r| *r < -0.5).count() as f64 / results.len() as f64;
println!("Ruin Probability (50% loss): {}%", ruin_prob * 100.0);
```

### Sensitivity Analysis (Grid Search)

Parameter grid search to identify stable parameter regions and avoid "curve fitting".

**Why Sensitivity Analysis?** Identifies robust parameter ranges, not just single optimal values.

```rust
use alphafield_backtest::SensitivityAnalyzer;

// Define parameter ranges
let rsi_periods = vec![10, 12, 14, 16, 18, 20];
let overbought_levels = vec![65, 70, 75, 80];
let oversold_levels = vec![20, 25, 30, 35];

let analyzer = SensitivityAnalyzer::new();

// Run grid search
let results = analyzer.grid_search(
    &bars,
    |params| RsiReversionStrategy::new(
        params.rsi_period,
        params.overbought,
        params.oversold,
    ),
    &rsi_periods,
    &overbought_levels,
    &oversold_levels,
)?;

// Find best Sharpe ratio
let best_result = results.iter().max_by(|a, b| a.sharpe_ratio.partial_cmp(&b.sharpe_ratio).unwrap()).unwrap();
println!("Best Sharpe: {}", best_result.sharpe_ratio);
println!("Best Params: RSI={}, OB={}, OS={}",
    best_result.rsi_period, best_result.overbought, best_result.oversold);

// Analyze parameter stability
// Good strategies have high Sharpe in a neighborhood around best params
```

### Correlation Analysis

Calculates correlation matrices between multiple strategy equity curves to build diversified portfolios.

**Why Correlation Analysis?** Low correlation = better diversification = smoother equity curve.

```rust
use alphafield_backtest::CorrelationAnalyzer;

// Run multiple strategies
let strategy_a = GoldenCross::new(50, 200);
let strategy_b = RsiReversion::new(14, 70, 30);
let strategy_c = MacdTrend::new(12, 26, 9);

let results_a = run_backtest(&strategy_a, &bars)?;
let results_b = run_backtest(&strategy_b, &bars)?;
let results_c = run_backtest(&strategy_c, &bars)?;

// Calculate correlation matrix
let analyzer = CorrelationAnalyzer::new();
let correlation_matrix = analyzer.calculate(&[
    &results_a.equity_curve,
    &results_b.equity_curve,
    &results_c.equity_curve,
])?;

println!("Correlation A-B: {}", correlation_matrix[0][1]);
println!("Correlation A-C: {}", correlation_matrix[0][2]);
println!("Correlation B-C: {}", correlation_matrix[1][2]);

// Low correlation (< 0.5) = good diversification
// High correlation (> 0.8) = strategies are similar
```

## 🚀 Common Workflows

### Basic Backtest

```rust
use alphafield_backtest::BacktestEngine;
use alphafield_strategy::GoldenCross;

// Setup
let strategy = GoldenCross::new(50, 200);
let bars = load_bars_from_db()?;

// Run backtest
let mut engine = BacktestEngine::new(
    Decimal::from(100_000),
    Decimal::from_str("0.001")?,
    SlippageModel::FixedPercent(Decimal::from_str("0.0005")?),
);

engine.add_data("BTCUSDT", bars);
engine.set_strategy(Box::new(strategy));

let metrics = engine.run()?;

// Analyze results
println!("Total Return: {:.2}%", metrics.total_return * 100.0);
println!("Sharpe Ratio: {:.2}", metrics.sharpe_ratio);
println!("Max Drawdown: {:.2}%", metrics.max_drawdown * 100.0);
println!("Win Rate: {:.2}%", metrics.win_rate * 100.0);
```

### Complete Validation Pipeline

```rust
use alphafield_backtest::{BacktestEngine, WalkForwardAnalyzer, MonteCarloSimulator, SensitivityAnalyzer};

let strategy = RsiReversionStrategy::new(14, 70, 30);
let bars = load_bars_from_db()?;

// 1. Basic backtest
let mut engine = BacktestEngine::new(...);
let backtest_result = engine.run()?;

println!("Backtest Sharpe: {}", backtest_result.sharpe_ratio);

// 2. Walk-forward analysis
let wf_analyzer = WalkForwardAnalyzer::new(
    strategy.clone(),
    Duration::days(30),
    Duration::days(10),
    Duration::days(10),
);
let wf_results = wf_analyzer.run(&bars)?;

println!("Walk-Forward Stability: {:.2}",
    wf_results.iter().map(|r| r.sharpe_ratio).sum::<f64>() / wf_results.len() as f64);

// 3. Monte Carlo simulation
let mc_simulator = MonteCarloSimulator::new(1000);
let mc_results = mc_simulator.run(&backtest_result.trades)?;

let worst_5pct = percentile(&mc_results, 5.0);
println!("Worst 5% Drawdown: {}", worst_5pct);

// 4. Sensitivity analysis
let sens_analyzer = SensitivityAnalyzer::new();
let sens_results = sens_analyzer.grid_search(&bars, &strategy, ...)?;

// Analyze results to ensure strategy is robust across all tests
```

## 🔧 Strategy Validation Binary

The `validate_strategy` binary provides a command-line interface for comprehensive strategy validation.

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

## Best Practices

1. **Always use walk-forward analysis**: Tests robustness on out-of-sample data
2. **Run Monte Carlo simulation**: Estimates tail risk and worst-case scenarios
3. **Check parameter sensitivity**: Ensure stable results, not just optimal at one point
4. **Analyze drawdowns**: Max drawdown should be acceptable for your risk tolerance
5. **Look at multiple metrics**: Sharpe ratio, Sortino ratio, Calmar ratio, win rate
6. **Consider transaction costs**: Always include slippage and fees in backtests
7. **Test on multiple markets**: Strategy should work across different symbols
8. **Avoid overfitting**: Don't optimize too many parameters on limited data
9. **Use sufficient data**: Minimum 2-3 years of data for reliable results
10. **Review individual trades**: Understand why trades succeeded or failed
