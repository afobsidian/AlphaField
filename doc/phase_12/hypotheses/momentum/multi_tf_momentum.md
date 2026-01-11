# Multi-Timeframe Momentum Hypothesis

## Metadata
- **Name**: Multi-TF Momentum
- **Category**: Momentum
- **Sub-Type**: multi_timeframe
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/momentum/multi_tf_momentum.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: Multi-timeframe alignment (Price > Fast EMA > Slow EMA) indicates strong, sustainable momentum. By requiring alignment across different timeframes (simulated via EMAs), the strategy filters out weak momentum and enters only during high-conviction trends, reducing whipsaw.

## 2. Entry/Exit Logic

**Entry**: 
- Price crosses above Fast EMA (20)
- Fast EMA > Slow EMA (50)
- Alternative: Fast EMA crosses above Slow EMA while Price > Fast EMA

**Exit**:
- Take profit: 5%
- Stop loss: 3%
- Price crosses below Fast EMA
- Fast EMA crosses below Slow EMA

## 3. Expected Performance
- **Sharpe Ratio**: 1.2-1.7
- **Max Drawdown**: <18%
- **Win Rate**: 50-60%
- **Best Regime**: Strong trending markets

## 4. Risk Factors
- **Late Entry**: Requires multiple confirmations
- **Missed Moves**: Strong trends may start before alignment
- **Ranging Markets**: Frequent alignment breaks

## 5. Parameters
- fast_ema_period: 20 (range: 10-30)
- slow_ema_period: 50 (range: 30-100)
