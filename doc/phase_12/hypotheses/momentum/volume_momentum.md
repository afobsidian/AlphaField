# Volume Weighted Momentum Hypothesis

## Metadata
- **Name**: Volume Momentum
- **Category**: Momentum
- **Sub-Type**: volume_weighted
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/momentum/volume_momentum.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: Price momentum is more reliable when confirmed by volume. High volume indicates strong conviction and reduces false breakout risk. The strategy only enters when price crosses above EMA while volume is significantly above average (1.5x), ensuring momentum is genuine.

## 2. Entry/Exit Logic

**Entry**: 
- Price crosses above EMA
- Volume ≥ volume_multiplier × average_volume (1.5x)

**Exit**:
- Take profit: 5%
- Stop loss: 3%
- Price crosses below EMA
- Volume drops below threshold

## 3. Expected Performance
- **Sharpe Ratio**: 1.1-1.6
- **Max Drawdown**: <18%
- **Win Rate**: 48-58%
- **Best Regime**: Bull, Trending with volume

## 4. Risk Factors
- **Low Volume Assets**: May have few signals
- **Volume Spikes**: False signals from news events
- **Exit Timing**: Volume may drop before trend ends

## 5. Parameters
- price_ema_period: 20 (range: 10-50)
- volume_period: 20 (range: 10-30)
- volume_multiplier: 1.5 (range: 1.2-2.0)
