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
