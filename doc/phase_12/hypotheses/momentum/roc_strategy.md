# ROC (Rate of Change) Momentum Hypothesis

## Metadata
- **Name**: ROC Momentum
- **Category**: Momentum
- **Sub-Type**: rate_of_change
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/momentum/roc_strategy.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: Rate of Change (ROC) measures price momentum as a percentage. When ROC crosses above a positive threshold (e.g., 2%), it indicates accelerating upward momentum that tends to persist. The strategy captures this momentum by entering on ROC expansion and exiting when ROC turns negative or decelerates.

## 2. Entry/Exit Logic

**Entry**: 
- ROC crosses above entry_threshold (2%)
- Indicates accelerating price momentum

**Exit**:
- Take profit: 5%
- Stop loss: 3%
- ROC crosses below exit_threshold (-1%)

## 3. Expected Performance
- **Sharpe Ratio**: 1.0-1.5
- **Max Drawdown**: <20%
- **Win Rate**: 40-50%
- **Best Regime**: Bull, Trending

## 4. Risk Factors
- **Choppy Markets**: Frequent false signals
- **Lag**: ROC is calculated over period, may miss initial move
- **Whipsaw**: In ranging markets

## 5. Parameters
- roc_period: 10 (range: 5-20)
- entry_threshold: 2.0% (range: 1.0-5.0)
- exit_threshold: -1.0% (range: -2.0 to 0.0)
