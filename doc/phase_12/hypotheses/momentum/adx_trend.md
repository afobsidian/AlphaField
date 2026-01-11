# ADX Trend Strength Hypothesis

## Metadata
- **Name**: ADX Trend
- **Category**: Momentum
- **Sub-Type**: adx_trend
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/momentum/adx_trend.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: ADX (Average Directional Index) measures trend strength regardless of direction. When ADX crosses above 25, it indicates a strong trend is developing. By combining ADX strength with price direction, the strategy enters only during confirmed strong trends, reducing whipsaw in ranging markets.

## 2. Entry/Exit Logic

**Entry**: 
- ADX crosses above strong_trend_threshold (25)
- Price is in uptrend (rising)

**Exit**:
- Take profit: 5%
- Stop loss: 3%
- ADX crosses below weak_trend_threshold (20)
- Trend reversal detected

## 3. Expected Performance
- **Sharpe Ratio**: 1.0-1.5
- **Max Drawdown**: <22%
- **Win Rate**: 45-55%
- **Best Regime**: Trending markets

## 4. Risk Factors
- **Late Entry**: ADX confirms trends after they start
- **Sideways Markets**: Few signals, missed opportunities
- **Calculation Lag**: ADX requires significant warmup (period × 2)

## 5. Parameters
- adx_period: 14 (range: 10-20)
- strong_trend_threshold: 25 (range: 20-30)
- weak_trend_threshold: 20 (range: 15-25)
