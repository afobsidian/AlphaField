# 🧠 AlphaField Strategy Crate

This crate provides the technical analysis indicators and trading strategy implementations for AlphaField.

## 📊 Indicators

All indicators implement the `Indicator` trait for consistent usage.

- **SMA** (Simple Moving Average)
- **EMA** (Exponential Moving Average)
- **RSI** (Relative Strength Index)
- **MACD** (Moving Average Convergence Divergence)
- **BollingerBands** (Mean reversion bands)
- **ATR** (Average True Range)
- **ADX** (Average Directional Index)

## ♟️ Strategies

Strategies implement the `Strategy` trait defined in `alphafield_core`.

- **GoldenCross**: Classic SMA crossover (e.g., 50/200).
- **RsiReversionStrategy**: Mean reversion based on overbought/oversold levels.
- **MeanReversion**: Bollinger Band breakdown/breakout logic.
- **Momentum**: MACD-based momentum following.
- **TrendFollowing**: EMA trend confirmation with ADX filter.

## 🛠️ Usage

Implement your own strategy by implementing the `Strategy` trait:

```rust
use alphafield_core::{Strategy, Signal, Bar};

pub struct MyStrategy {
    // fields
}

impl Strategy for MyStrategy {
    fn on_bar(&mut self, bar: &Bar) -> Signal {
        // Logic here
        Signal::Hold
    }
}
```
