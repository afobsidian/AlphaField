# 🧠 AlphaField Strategy Crate

This crate provides the technical analysis indicators and trading strategy implementations for AlphaField.

## Why This Crate Exists

This crate provides a comprehensive library of technical indicators and ready-to-use trading strategies. By separating strategy logic from data fetching and execution, we enable:

- **Strategy reuse**: Test strategies across different markets and timeframes
- **Easy backtesting**: Swap strategies without changing data or execution logic
- **Combination**: Mix and match strategies for ensemble trading
- **Customization**: Extend existing indicators and strategies for specific needs

## 📊 Indicators

All indicators follow a consistent interface and return `Vec<Decimal>` for precision.

### SMA (Simple Moving Average)

Calculates the simple moving average for a given period.

```rust
use alphafield_strategy::indicators::SMA;
use rust_decimal::Decimal;

// Calculate 20-period SMA
let prices = vec![
    Decimal::from(100.0),
    Decimal::from(101.0),
    Decimal::from(102.0),
    Decimal::from(103.0),
    Decimal::from(104.0),
];

let sma = SMA::calculate(&prices, 5)?;

assert_eq!(sma.last(), Some(&Decimal::from(102.0)));
```

### EMA (Exponential Moving Average)

Gives more weight to recent prices, making it more responsive to new information.

```rust
use alphafield_strategy::indicators::EMA;

// Calculate 12-period EMA
let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0, 105.0];
let ema = EMA::calculate(&prices, 12)?;
```

### RSI (Relative Strength Index)

Measures the speed and change of price movements (0-100 range).

```rust
use alphafield_strategy::indicators::RSI;

let prices = vec![100.0, 101.0, 102.0, 101.0, 100.0];
let rsi = RSI::calculate(&prices, 14)?;

// RSI > 70 = Overbought
// RSI < 30 = Oversold
```

### MACD (Moving Average Convergence Divergence)

Trend-following momentum indicator showing relationship between two moving averages.

```rust
use alphafield_strategy::indicators::MACD;

let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0];
let macd = MACD::calculate(&prices, 12, 26, 9)?;

// MACD line, signal line, and histogram
println!("MACD: {}, Signal: {}, Histogram: {}",
    macd.macd_line, macd.signal_line, macd.histogram);
```

### Bollinger Bands

Mean reversion indicator with upper and lower bands based on standard deviation.

```rust
use alphafield_strategy::indicators::BollingerBands;

let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0];
let bb = BollingerBands::calculate(&prices, 20, 2.0)?;

// Upper band, middle band (SMA), lower band
println!("Upper: {}, Middle: {}, Lower: {}",
    bb.upper_band, bb.middle_band, bb.lower_band);

// Price near upper band = Overbought
// Price near lower band = Oversold
```

### ATR (Average True Range)

Measures market volatility by calculating the average range between high and low.

```rust
use alphafield_strategy::indicators::ATR;

// Use OHLC data for ATR
let highs = vec![102.0, 103.0, 104.0];
let lows = vec![98.0, 99.0, 100.0];
let closes = vec![100.0, 101.0, 102.0];

let atr = ATR::calculate(&highs, &lows, &closes, 14)?;
```

### ADX (Average Directional Index)

Measures trend strength (0-100 range), not direction.

```rust
use alphafield_strategy::indicators::ADX;

// ADX > 25 = Strong trend
// ADX < 20 = Weak or no trend
let adx = ADX::calculate(&highs, &lows, &closes, 14)?;
```

## ♟️ Strategies

All strategies implement the `Strategy` trait from `alphafield_core`.

### GoldenCross

Classic SMA crossover strategy. Uses two SMAs and generates buy/sell signals when they cross.

```rust
use alphafield_strategy::GoldenCross;
use alphafield_core::Bar;

// Create Golden Cross strategy with 50/200 SMAs
let strategy = GoldenCross::new(50, 200);

// Generate signal
let signal = strategy.generate_signal(&bars)?;
// Returns Signal::Buy when 50 SMA crosses above 200 SMA
// Returns Signal::Sell when 50 SMA crosses below 200 SMA
```

### RsiReversionStrategy

Mean reversion strategy based on RSI overbought/oversold levels.

```rust
use alphafield_strategy::RsiReversionStrategy;

// Create RSI reversion strategy
let strategy = RsiReversionStrategy::new(14, 70, 30);
//                                    period, overbought, oversold

// Generate signal
let signal = strategy.generate_signal(&bars)?;
// Returns Signal::Buy when RSI < 30 (oversold)
// Returns Signal::Sell when RSI > 70 (overbought)
```

### MeanReversion

Uses Bollinger Bands to identify mean reversion opportunities.

```rust
use alphafield_strategy::MeanReversion;

let strategy = MeanReversion::new(20, 2.0);
//                              period, std_dev

// Generate signal
let signal = strategy.generate_signal(&bars)?;
// Returns Signal::Buy when price breaks below lower band
// Returns Signal::Sell when price breaks above upper band
```

### Momentum

MACD-based momentum following strategy.

```rust
use alphafield_strategy::Momentum;

let strategy = Momentum::new(12, 26, 9);
//                            fast, slow, signal

// Generate signal
let signal = strategy.generate_signal(&bars)?;
// Returns Signal::Buy when MACD crosses above signal line
// Returns Signal::Sell when MACD crosses below signal line
```

### TrendFollowing

Combines EMA trend with ADX filter to trade strong trends only.

```rust
use alphafield_strategy::TrendFollowing;

let strategy = TrendFollowing::new(50, 25);
//                             ema_period, adx_threshold

// Generate signal
let signal = strategy.generate_signal(&bars)?;
// Returns Signal::Buy when price > EMA and ADX > 25 (strong uptrend)
// Returns Signal::Sell when price < EMA and ADX > 25 (strong downtrend)
```

## 🛠️ Creating Custom Strategies

### Basic Strategy Pattern

Implement the `Strategy` trait:

```rust
use alphafield_core::{Strategy, Signal, Bar};
use chrono::Utc;

pub struct MyStrategy {
    // Strategy parameters
    period: usize,
    threshold: f64,
}

impl MyStrategy {
    pub fn new(period: usize, threshold: f64) -> Self {
        Self { period, threshold }
    }
}

impl Strategy for MyStrategy {
    fn generate_signal(&self, bars: &[Bar]) -> Option<Signal> {
        // Need enough data
        if bars.len() < self.period {
            return None;
        }

        // Calculate indicator
        let current_price = bars.last()?.close;
        let sma = calculate_sma(bars, self.period);

        // Generate signal
        if current_price > sma * (1.0 + self.threshold) {
            Some(Signal::buy(0.8, Decimal::from(1), Utc::now()))
        } else if current_price < sma * (1.0 - self.threshold) {
            Some(Signal::sell(0.8, Decimal::from(1), Utc::now()))
        } else {
            Some(Signal::hold(Utc::now()))
        }
    }

    fn update(&mut self, bar: &Bar) {
        // Update internal state if needed
        // For example, track previous signals
    }

    fn name(&self) -> &str {
        "MyStrategy"
    }
}

// Helper function to calculate SMA
fn calculate_sma(bars: &[Bar], period: usize) -> f64 {
    let recent_bars: Vec<_> = bars.iter().rev().take(period).collect();
    let sum: f64 = recent_bars.iter().map(|b| b.close).sum();
    sum / period as f64
}
```

### Combining Multiple Indicators

```rust
use alphafield_strategy::{SMA, RSI, MACD};
use alphafield_core::{Strategy, Signal, Bar};

pub struct MultiIndicatorStrategy {
    sma_period: usize,
    rsi_period: usize,
    rsi_overbought: f64,
    rsi_oversold: f64,
}

impl MultiIndicatorStrategy {
    pub fn new() -> Self {
        Self {
            sma_period: 50,
            rsi_period: 14,
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,
        }
    }
}

impl Strategy for MultiIndicatorStrategy {
    fn generate_signal(&self, bars: &[Bar]) -> Option<Signal> {
        if bars.len() < 50 {
            return None;
        }

        // Calculate indicators
        let current_price = bars.last()?.close;
        let prices: Vec<f64> = bars.iter().map(|b| b.close).collect();

        let sma = SMA::calculate(&prices, self.sma_period);
        let rsi = RSI::calculate(&prices, self.rsi_period);

        // Combine signals
        let above_trend = current_price > sma;
        let oversold = rsi < self.rsi_oversold;
        let overbought = rsi > self.rsi_overbought;

        if above_trend && oversold {
            // Trend is up but oversold - good buy opportunity
            Some(Signal::buy(0.9, Decimal::from(1), Utc::now()))
        } else if !above_trend && overbought {
            // Trend is down but overbought - good sell opportunity
            Some(Signal::sell(0.9, Decimal::from(1), Utc::now()))
        } else if above_trend {
            // In uptrend, hold
            Some(Signal::hold(Utc::now()))
        } else {
            // No clear signal
            None
        }
    }

    fn update(&mut self, bar: &Bar) {}

    fn name(&self) -> &str {
        "MultiIndicatorStrategy"
    }
}
```

## Common Workflows

### Backtesting a Strategy

```rust
use alphafield_strategy::GoldenCross;
use alphafield_backtest::BacktestEngine;
use alphafield_core::Bar;

// Create strategy
let strategy = GoldenCross::new(50, 200);

// Load historical data
let bars: Vec<Bar> = load_bars_from_db()?;

// Run backtest
let mut engine = BacktestEngine::new(strategy, initial_capital);
let result = engine.run(&bars)?;

// Analyze results
println!("Total Return: {}", result.total_return);
println!("Sharpe Ratio: {}", result.sharpe_ratio);
println!("Max Drawdown: {}", result.max_drawdown);
```

### Strategy Optimization

```rust
use alphafield_strategy::RsiReversionStrategy;

// Define parameter ranges
let rsi_periods = vec![10, 12, 14, 16, 18];
let overbought_levels = vec![65, 70, 75, 80];
let oversold_levels = vec![20, 25, 30, 35];

// Test all combinations
let mut best_sharpe = f64::MIN;
let mut best_params = None;

for period in &rsi_periods {
    for overbought in &overbought_levels {
        for oversold in &oversold_levels {
            let strategy = RsiReversionStrategy::new(*period, *overbought, *oversold);
            let result = run_backtest(&strategy, &bars)?;

            if result.sharpe_ratio > best_sharpe {
                best_sharpe = result.sharpe_ratio;
                best_params = Some((*period, *overbought, *oversold));
            }
        }
    }
}

println!("Best params: period={}, overbought={}, oversold={}",
    best_params.unwrap().0,
    best_params.unwrap().1,
    best_params.unwrap().2);
```

### Strategy Ensemble

Combine multiple strategies for more robust signals.

```rust
use alphafield_strategy::{GoldenCross, RsiReversionStrategy, Momentum};

struct EnsembleStrategy {
    strategies: Vec<Box<dyn Strategy>>,
    consensus_threshold: usize,
}

impl EnsembleStrategy {
    pub fn new() -> Self {
        Self {
            strategies: vec![
                Box::new(GoldenCross::new(50, 200)),
                Box::new(RsiReversionStrategy::new(14, 70, 30)),
                Box::new(Momentum::new(12, 26, 9)),
            ],
            consensus_threshold: 2,  // Need 2/3 strategies to agree
        }
    }
}

impl Strategy for EnsembleStrategy {
    fn generate_signal(&self, bars: &[Bar]) -> Option<Signal> {
        let buy_votes = self.strategies.iter()
            .filter_map(|s| s.generate_signal(bars))
            .filter(|sig| matches!(sig.signal_type, SignalType::Buy))
            .count();

        let sell_votes = self.strategies.iter()
            .filter_map(|s| s.generate_signal(bars))
            .filter(|sig| matches!(sig.signal_type, SignalType::Sell))
            .count();

        if buy_votes >= self.consensus_threshold {
            Some(Signal::buy(0.9, Decimal::from(1), Utc::now()))
        } else if sell_votes >= self.consensus_threshold {
            Some(Signal::sell(0.9, Decimal::from(1), Utc::now()))
        } else {
            Some(Signal::hold(Utc::now()))
        }
    }

    fn update(&mut self, bar: &Bar) {
        for strategy in &mut self.strategies {
            strategy.update(bar);
        }
    }

    fn name(&self) -> &str {
        "EnsembleStrategy"
    }
}
```

## Best Practices

1. **Always test strategies**: Use backtesting before real trading
2. **Use multiple indicators**: Don't rely on a single indicator
3. **Consider market conditions**: Trend following works in trending markets, mean reversion in ranging markets
4. **Set stop losses**: Always define risk management rules
5. **Avoid overfitting**: Don't over-optimize parameters on historical data
6. **Use walk-forward analysis**: Test on out-of-sample data
7. **Monitor performance**: Track strategy performance in real-time
8. **Combine strategies**: Use ensembles for more robust signals
