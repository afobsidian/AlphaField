# MACD Momentum Hypothesis

## Metadata
- **Name**: MACD Momentum  
- **Category**: Momentum
- **Sub-Type**: indicator_combination
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/momentum/macd_strategy.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: MACD crossovers combined with EMA trend confirmation provide reliable momentum signals. The strategy enters when MACD crosses above signal line while price is above EMA, indicating both short-term momentum (MACD) and longer-term trend (EMA) alignment.

## 2. Entry/Exit Logic

**Entry**: 
- Price > EMA(50)
- MACD line crosses above signal line

**Exit**:
- Take profit: 5%
- Stop loss: 5%
- MACD crosses below signal line
- Price crosses below EMA

## 3. Expected Performance
- **Sharpe Ratio**: 1.0-1.5
- **Max Drawdown**: <30%
- **Win Rate**: 45-55%
- **Best Regime**: Bull, Trending

## 4. Risk Factors
- **Whipsaw Risk**: Medium in sideways markets
- **Late Entry**: MACD lags price
- **Correlation**: High with other momentum strategies

## 5. Parameters
- ema_period: 50 (range: 20-100)
- macd_fast: 12 (range: 10-15)
- macd_slow: 26 (range: 20-30)
- macd_signal: 9 (range: 7-12)

## References
- Original MACD development by Gerald Appel
- "Technical Analysis of the Financial Markets" by John Murphy
