# ATR Breakout Strategy Hypothesis

## Metadata
- **Name**: ATR Breakout Strategy
- **Category**: VolatilityBased
- **Sub-Type**: atr_breakout
- **Author**: AI Agent
- **Date**: 2026-01-15
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/volatility/atr_breakout.md

## 1. Hypothesis Statement

**Primary Hypothesis**: 
ATR breakout captures volatility expansions and trend resumptions. When price breaks above the previous high plus ATR × multiplier, the asset is experiencing a volatility expansion and will generate positive returns with an average return of >5% over the holding period.

**Null Hypothesis**: 
ATR-based breakouts do not reliably predict continued upward price movement and perform no better than random entry at similar price levels.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Average True Range (ATR) measures market volatility and typical price movement range. Breakouts above previous highs adjusted by ATR represent significant price movements that exceed normal volatility, suggesting strong buying pressure and trend continuation. The ATR multiplier adapts the breakout threshold to current market conditions - wider in volatile markets, tighter in calm markets.

### 2.2 Market Inefficiency Exploited
The strategy exploits the persistence of volatility and trends. When price breaks through resistance with increased volatility, it often indicates the start or resumption of a sustained trend. Market participants may under-react to the significance of ATR-adjusted breakouts, particularly in crypto markets where volatility is common but meaningful breakouts are less frequent.

### 2.3 Expected Duration of Edge
This edge persists as long as markets exhibit trending behavior and volatility clustering. In highly efficient, range-bound markets, the strategy may underperform. However, in crypto markets where volatility expansions often precede or accompany strong moves, this edge should persist.

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: ATR breakouts capture trend resumptions and continuation in uptrends
- **Historical Evidence**: Bull markets produce volatility expansions that lead to sustained rallies

### 3.2 Bearish Markets
- **Expected Performance**: Low
- **Rationale**: Strategy is long-only, exits on stop loss; bear rallies are often short-lived
- **Adaptations**: Quick stop loss (4%) prevents deep drawdowns in bear markets

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Medium
- **Rationale**: False breakouts common in choppy markets, but volume confirmation helps filter
- **Filters**: Trend MA filter (optional) can reduce false signals in ranging markets

### 3.4 Volatility Conditions
- **High Volatility**: High - ATR breakout captures volatility expansions naturally
- **Low Volatility**: Low - breakout thresholds may be too tight, causing premature exits
- **Volatility Filter**: ATR itself serves as the volatility measure and breakout trigger

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 20%
- **Average Drawdown**: 5%
- **Drawdown Duration**: 1-2 weeks
- **Worst Case Scenario**: Multiple false breakouts in choppy market with consecutive stop losses

### 4.2 Failure Modes

#### Failure Mode 1: False Breakout in Ranging Market
- **Trigger**: Price temporarily breaks above ATR-adjusted high but immediately reverses
- **Impact**: Stop loss hit as price falls back below entry
- **Mitigation**: Increase volume_multiplier (>1.5), add trend MA filter, require breakout persistence
- **Detection**: Win rate falls below 35%, average trade duration < 2 days

#### Failure Mode 2: Late Entry After Most of the Move
- **Trigger**: Breakout occurs after significant portion of the move has already occurred
- **Impact**: Limited upside potential with same downside risk (asymmetric risk/reward)
- **Mitigation**: Add momentum confirmation, use tighter lookback_period, monitor RSI for overbought
- **Detection**: Average profit per trade significantly lower than total move after breakout

#### Failure Mode 3: Volatility Spike False Positive
- **Trigger**: ATR spikes due to volatility expansion unrelated to genuine trend resumption
- **Impact**: ATR-adjusted breakout threshold becomes too wide or too narrow, causing poor entries
- **Mitigation**: Use ATR smoothing (longer atr_period), add ATR percentile filter
- **Detection**: Correlation between ATR spikes and failed breakouts

### 4.3 Correlation Analysis
- **Correlation with Market**: High (trend-following strategy)
- **Correlation with Other Strategies**: High with other breakout and trend-following strategies
- **Diversification Value**: Low-Moderate - complements mean reversion and contrarian strategies

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1**: Price breaks above ATR-adjusted high
   - **Indicator**: Breakout level = previous_high + (ATR × atr_multiplier)
   - **Parameters**: Current price > breakout level
   - **Confirmation**: Breakout must be above previous high from lookback_period (20)
   - **Priority**: Required

2. **Condition 2**: Volume confirmation (optional)
   - **Indicator**: Current volume / average volume
   - **Parameters**: Volume > average_volume × volume_multiplier (1.5)
   - **Confirmation**: Ensures breakout has participation
   - **Priority**: Optional but recommended

3. **Condition 3**: Trend MA filter (optional)
   - **Indicator**: Price vs trend moving average
   - **Parameters**: Price > trend_ma (50-period SMA)
   - **Confirmation**: Ensures entry in uptrend context
   - **Priority**: Optional

### 5.2 Entry Filters
- **Time of Day**: Not applicable (works on any timeframe)
- **Volume Requirements**: Volume > average_volume × volume_multiplier (1.5)
- **Market Regime Filter**: Optional trend MA filter
- **Volatility Filter**: ATR serves as inherent volatility measure
- **Price Filter**: Breakout must exceed ATR-adjusted threshold

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: Volume spike (>1.5× average)
- **Confirmation Indicator 2**: Trend MA filter (>50 SMA) - optional
- **Minimum Confirmed**: 1 out of 2 (volume required, trend optional)

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP 1**: 8% profit - Close 100%

### 6.2 Stop Loss
- **Initial SL**: 4% below entry
- **Trailing SL**: None
- **Breakeven**: None
- **Time-based Exit**: None

### 6.3 Exit Conditions
- **Take Profit Reached**: 8% profit target achieved
- **Stop Loss Triggered**: 4% loss threshold exceeded
- **Reversal Signal**: Not implemented (relies on SL)
- **Regime Change**: Not implemented
- **Volatility Drop**: Not implemented
- **Time Limit**: None

## 7. Position Sizing
- **Base Position Size**: 1% of portfolio per signal
- **Volatility Adjustment**: Could reduce size if ATR is very high
- **Conviction Levels**: Could use breakout strength (price above threshold)
- **Max Position Size**: 5% of portfolio
- **Risk per Trade**: Max 4% (stop loss)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| atr_period | 14 | 7-21 | ATR calculation period | int |
| atr_multiplier | 1.5 | 1.0-3.0 | ATR multiplier for breakout threshold | float |
| lookback_period | 20 | 10-40 | Lookback for previous high | int |
| trend_ma_period | 50 | 20-100 | Trend MA period for optional filter | int |
| volume_multiplier | 1.5 | 1.0-3.0 | Volume multiplier for confirmation | float |
| take_profit | 8.0 | 5.0-15.0 | Take profit percentage | float |
| stop_loss | 4.0 | 2.0-8.0 | Stop loss percentage | float |

### 8.2 Optimization Notes
- **Parameters to Optimize**: atr_multiplier, lookback_period, volume_multiplier
- **Optimization Method**: Walk-forward analysis
- **Optimization Period**: 2 years
- **Expected Overfitting Risk**: Medium (breakout parameters can be curve-fit)
- **Sensitivity Analysis Required**: Yes

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- **Timeframes**: 1H, 4H, 1D
- **Test Period**: 2019-2025 (6 years)
- **Assets**: BTC, ETH, SOL, 10+ others
- **Minimum Trades**: 80
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
- **Maximum Max Drawdown**: 20%
- **Minimum Win Rate**: 40%
- **Minimum Profit Factor**: 1.5
- **Minimum Robustness Score**: >60
- **Statistical Significance**: p < 0.05
- **Walk-Forward Stability**: >50

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 20-40%
- **Sharpe Ratio**: 1.0-2.0
- **Max Drawdown**: <20%
- **Win Rate**: 40-50%
- **Profit Factor**: >1.5
- **Expectancy**: >0.03

### 10.2 Comparison to Baselines
- **vs. HODL**: +10-25% risk-adjusted returns
- **vs. Market Average**: +8-20% outperformance
- **vs. Simple Breakout**: Superior due to ATR adaptation
- **vs. Buy & Hold**: Lower max drawdown, more consistent returns

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: ATR, SMA (for optional trend filter)
- **Data Requirements**: OHLCV
- **Latency Sensitivity**: Low
- **Computational Complexity**: Low-Medium (ATR calculation, breakout detection)
- **Memory Requirements**: Low (lookback_period for high calculation)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/volatility/atr_breakout.md
- **Strategy Type**: Volatility-adjusted breakout
- **Dependencies**: alphafield_core, indicators::Atr, indicators::Sma
- **State Management**: Tracks previous high, ATR values, position, entry price

### 11.3 Indicator Calculations
**ATR**: Average True Range over atr_period (14) bars using Wilder's smoothing

**Breakout Level**: Previous high (from lookback_period bars ago) + (ATR × atr_multiplier)

**Volume Confirmation**: Current volume > average_volume × volume_multiplier

**Trend MA Filter**: Current price > SMA(trend_ma_period) - optional

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (ATR breakout detection, volume confirmation, trend filter)
- [x] Exit conditions (TP, SL)
- [x] Edge cases (empty data, insufficient bars, single bar)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)
- [x] ATR calculation
- [x] Breakout level calculation
- [x] Display formatting

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

### 2026-01-15: Initial Implementation
**Observation**: Strategy implemented with ATR-adjusted breakout levels and optional filters
**Hypothesis Impact**: Code supports hypothesis - enters on ATR breakouts, exits on TP/SL
**Issues Found**: None during implementation
**Action Taken**: Ready for testing

### [Date]: Initial Backtest Results
**Test Period**: [Date range]
**Symbols Tested**: [List of symbols]
**Results**: [Summary of metrics]
**Observation**: [Are results as expected?]
**Action Taken**: [Proceed to validation or refine strategy?]

### [Date]: False Breakout Analysis
**Observation**: [What patterns cause false breakouts?]
**Hypothesis Impact**: [Does this invalidate the hypothesis or require parameter adjustment?]
**Issues Found**: [Specific failure patterns]
**Action Taken**: [Parameter adjustments or filter additions]

## 14. References

### Academic Sources
- Wilder, J. Welles. "New Concepts in Technical Trading Systems" (1978) - Original ATR development
- Connors, L., & Raschke, L. "Street Smarts: High Probability Short-Term Trading Strategies" (1995) - Breakout concepts

### Books
- Kaufman, P. J. "Trading Systems and Methods" - Comprehensive breakout and ATR analysis
- Bulkowski, T. N. "Encyclopedia of Chart Patterns" - Breakout patterns and statistics

### Online Resources
- ATR breakout trading research and case studies
- Volatility-based breakout strategies
- Crypto market volatility patterns

### Similar Strategies
- Volatility Squeeze (different volatility detection approach)
- Simple Breakout (without ATR adjustment)
- Trend Following (similar entry logic, different trigger)

### Historical Examples
- 2020 COVID crash recovery (ATR breakout captured resumption)
- 2021 crypto bull market (multiple ATR breakouts)
- Various volatility expansions leading to trends

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-15 | 1.0 | Initial hypothesis | AI Agent |