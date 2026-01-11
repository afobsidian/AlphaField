# RSI Momentum Hypothesis

## Metadata
- **Name**: RSI Momentum
- **Category**: Momentum
- **Sub-Type**: rsi_momentum
- **Author**: AI Agent
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/momentum/rsi_momentum.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
RSI momentum signals indicate strong trends when RSI crosses above 50 with increasing momentum. Unlike RSI mean reversion which trades oversold/overbought extremes, RSI momentum identifies sustained directional moves by tracking RSI strength above the neutral level.

**Null Hypothesis**: 
RSI crossing above 50 does not reliably predict continued upward price movement and performs no better than random entry.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
RSI above 50 indicates that recent gains outweigh recent losses, suggesting bullish momentum. When RSI crosses above 50 with strength (>60), it signals that buying pressure is accelerating. This momentum-based approach differs from contrarian RSI strategies that fade extremes.

### 2.2 Market Inefficiency Exploited
The strategy exploits momentum persistence - the tendency for strong moves to continue in the same direction. While mean reversion traders fade RSI extremes, momentum traders ride the wave of sustained buying pressure indicated by elevated RSI levels.

### 2.3 Expected Duration of Edge
This edge persists as long as markets exhibit trending behavior. In highly efficient, range-bound markets, the strategy may underperform as RSI oscillates around 50 without sustained directional moves.

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: RSI momentum excels in uptrends where RSI stays elevated (>50) for extended periods
- **Historical Evidence**: Strong bull markets show persistent RSI > 60

### 3.2 Bearish Markets
- **Expected Performance**: Low
- **Rationale**: Strategy is long-only, exits when RSI < 50
- **Adaptations**: Quick exits prevent deep drawdowns in bear markets

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Low
- **Rationale**: RSI oscillates around 50, generating whipsaw signals
- **Filters**: Need volume or price confirmation to reduce false signals

### 3.4 Volatility Conditions
- **High Volatility**: Medium - momentum can be strong but exits may be frequent
- **Low Volatility**: Low - RSI may not reach meaningful levels
- **Volatility Filter**: Consider ADX or ATR filter for trend strength

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 25%
- **Average Drawdown**: 8%
- **Drawdown Duration**: 1-2 weeks
- **Worst Case Scenario**: Prolonged sideways market with frequent whipsaws

### 4.2 Failure Modes

#### Failure Mode 1: Whipsaw in Ranging Markets
- **Trigger**: RSI oscillates around 50 without sustained momentum
- **Impact**: Frequent small losses from failed breakouts
- **Mitigation**: Add trend filter (price > 200 SMA) or volume confirmation
- **Detection**: Win rate falls below 35%, average trade duration < 2 days

#### Failure Mode 2: Late Exits in Reversals
- **Trigger**: RSI stays > 50 during initial pullback before reversing
- **Impact**: Gives back profits as reversal develops
- **Mitigation**: Tighter stop loss (3%) or faster RSI period (9)
- **Detection**: Average loss > average win, profit factor < 1.2

### 4.3 Correlation Analysis
- **Correlation with Market**: Medium (follows market trends)
- **Correlation with Other Strategies**: High with other momentum strategies
- **Diversification Value**: Moderate - complements mean reversion strategies

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1**: RSI crosses above momentum_threshold (50)
   - **Indicator**: 14-period RSI
   - **Parameters**: Previous RSI ≤ 50, Current RSI > 50
   - **Confirmation**: None required for entry
   - **Priority**: Required

2. **Condition 2**: Strong momentum (RSI ≥ 60)
   - **Signal strength**: 1.0 if RSI ≥ 60, else 0.7
   - **Priority**: Optional (affects signal strength)

### 5.2 Entry Filters
- **Time of Day**: Not applicable (works on any timeframe)
- **Volume Requirements**: Optional - could add volume > 1.2x average
- **Market Regime Filter**: Optional - could filter for ADX > 20 (trending)
- **Volatility Filter**: None currently
- **Price Filter**: None currently

### 5.3 Entry Confirmation
None required - pure RSI crossover signal

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP 1**: 5% profit - Close 100%

### 6.2 Stop Loss
- **Initial SL**: 3% below entry
- **Trailing SL**: None
- **Breakeven**: None
- **Time-based Exit**: None

### 6.3 Exit Conditions
- **Momentum Loss**: RSI crosses below 50
- **Regime Change**: Not implemented
- **Volatility Spike**: Not implemented
- **Time Limit**: None

## 7. Position Sizing
- **Base Position Size**: 1% of portfolio per signal
- **Volatility Adjustment**: None currently
- **Conviction Levels**: Signal strength (0.7 or 1.0) could be used
- **Max Position Size**: 5% of portfolio
- **Risk per Trade**: Max 3% (stop loss)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| rsi_period | 14 | 7-21 | RSI calculation period | int |
| momentum_threshold | 50 | 45-55 | RSI level for momentum signal | float |
| strength_threshold | 60 | 55-70 | RSI level for strong signal | float |
| take_profit | 5.0 | 3.0-10.0 | Take profit percentage | float |
| stop_loss | 3.0 | 2.0-5.0 | Stop loss percentage | float |

### 8.2 Optimization Notes
- **Parameters to Optimize**: rsi_period, momentum_threshold
- **Optimization Method**: Walk-forward analysis
- **Optimization Period**: 2 years
- **Expected Overfitting Risk**: Medium
- **Sensitivity Analysis Required**: Yes

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- **Timeframes**: 1H, 4H, 1D
- **Test Period**: 2019-2025 (6 years)
- **Assets**: BTC, ETH, SOL, 10+ others
- **Minimum Trades**: 100
- **Slippage**: 0.1% per trade
- **Commission**: 0.1% per trade

### 9.2 Validation Techniques
- [x] Walk-forward analysis (rolling window)
- [x] Monte Carlo simulation (trade sequence randomization)
- [x] Parameter sweep (sensitivity analysis)
- [ ] Regime analysis (bull/bear/sideways)
- [ ] Cross-asset validation (multiple symbols)
- [ ] Bootstrap validation (resampling)
- [ ] Permutation testing (randomness check)

### 9.3 Success Criteria
- **Minimum Sharpe Ratio**: 1.0
- **Minimum Sortino Ratio**: 1.5
- **Maximum Max Drawdown**: 25%
- **Minimum Win Rate**: 40%
- **Minimum Profit Factor**: 1.3
- **Minimum Robustness Score**: >60
- **Statistical Significance**: p < 0.05
- **Walk-Forward Stability**: >50

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 15-25%
- **Sharpe Ratio**: 1.0-1.5
- **Max Drawdown**: <25%
- **Win Rate**: 40-50%
- **Profit Factor**: >1.3
- **Expectancy**: >0.01

### 10.2 Comparison to Baselines
- **vs. HODL**: +5-10% risk-adjusted returns
- **vs. Market Average**: +3-7% outperformance
- **vs. RSI Mean Reversion**: Different regime focus (trending vs ranging)
- **vs. Buy & Hold**: Lower max drawdown, more consistent returns

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: RSI (14-period)
- **Data Requirements**: OHLCV
- **Latency Sensitivity**: Low
- **Computational Complexity**: Low
- **Memory Requirements**: Minimal (14 bars for RSI)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/momentum/rsi_momentum.rs
- **Strategy Type**: Simple indicator-based
- **Dependencies**: alphafield_core, indicators::Rsi
- **State Management**: Tracks last RSI, position, entry price

### 11.3 Indicator Calculations
RSI = 100 - (100 / (1 + RS))
where RS = Average Gain / Average Loss over period

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (RSI crossover above 50)
- [x] Exit conditions (RSI crossover below 50, TP, SL)
- [x] Edge cases (empty data, single bar)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)

### 12.2 Integration Tests
- [ ] Backtest execution (runs without errors)
- [ ] Performance calculation (metrics are correct)
- [ ] Dashboard integration (API works)
- [ ] Database integration (saves correctly)

### 12.3 Research Tests
- [ ] Hypothesis validation (supports primary hypothesis)
- [ ] Statistical significance (p < 0.05)
- [ ] Regime analysis (performance as expected)
- [ ] Robustness testing (stable across parameters)

## 13. Research Journal

### 2026-01-11: Initial Implementation
**Observation**: Strategy implemented following momentum template
**Hypothesis Impact**: Code supports hypothesis - enters on RSI momentum, exits on loss of momentum
**Issues Found**: None yet
**Action Taken**: Ready for testing

## 14. References

### Academic Sources
- Wilder, J. Welles. "New Concepts in Technical Trading Systems" (1978) - Original RSI development

### Online Resources
- Momentum vs Mean Reversion RSI strategies comparison studies

### Similar Strategies
- RSI Mean Reversion (opposite approach - fades extremes vs follows momentum)
- MACD Momentum (similar concept using different indicator)

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-11 | 1.0 | Initial hypothesis | AI Agent |
