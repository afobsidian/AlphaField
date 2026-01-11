# Momentum Factor Hypothesis

## Metadata
- **Name**: Momentum Factor
- **Category**: Momentum
- **Sub-Type**: multi_factor
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/momentum/momentum_factor.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: Combining multiple momentum factors (price momentum, volume momentum, RSI momentum) provides stronger signals than any single factor. The strategy requires at least 2 out of 3 factors to be positive, reducing false signals while capturing high-conviction momentum moves.

## 2. Entry/Exit Logic

**Entry**: 
- At least min_factors (2) out of 3 are positive:
  1. Price > price N periods ago
  2. Volume > average volume
  3. RSI > 50

**Exit**:
- Take profit: 5%
- Stop loss: 3%
- Less than min_factors are positive

## 3. Expected Performance
- **Sharpe Ratio**: 1.2-1.7
- **Max Drawdown**: <20%
- **Win Rate**: 50-60%
- **Best Regime**: Bull, Trending

## 4. Risk Factors
- **Complexity**: More factors = more ways to fail
- **Correlation**: Factors may become correlated
- **Calibration**: Optimal min_factors may change

## 5. Parameters
- lookback_period: 20 (range: 10-30)
- rsi_period: 14 (range: 10-20)
- min_factors: 2 (range: 1-3)
