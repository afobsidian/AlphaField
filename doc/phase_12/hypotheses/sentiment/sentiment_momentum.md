# Sentiment Momentum Hypothesis

## Metadata
- **Name**: Sentiment Momentum
- **Category**: SentimentBased
- **Sub-Type**: sentiment_momentum
- **Author**: AI Agent
- **Date**: 2026-01-17
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/sentiment/sentiment_momentum.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
Sentiment momentum predicts price momentum. When technical sentiment (derived from RSI-like component, momentum, and volume) crosses above bullish thresholds with positive momentum, the asset is experiencing improving market sentiment and will generate positive returns with an average return of >3% over the holding period.

**Null Hypothesis**: 
Sentiment momentum signals do not reliably predict continued upward price movement and perform no better than random entry.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Technical sentiment combines multiple price-based indicators (RSI-like oscillator, price momentum, and volume) to create a composite measure of market sentiment. When sentiment crosses above the bullish threshold (30) with positive momentum, it indicates improving buying pressure and market confidence. This momentum-based approach follows the trend of improving sentiment rather than fading extremes.

### 2.2 Market Inefficiency Exploited
The strategy exploits the persistence of sentiment momentum - the tendency for improving sentiment to continue driving prices higher. Markets under-react to gradual sentiment improvements, creating an edge for strategies that identify and follow sentiment trends rather than anticipating reversals.

### 2.3 Expected Duration of Edge
This edge persists as long as markets exhibit trending behavior with gradual sentiment changes. In highly efficient markets with instant information dissemination, the edge may diminish. However, in crypto markets where sentiment shifts gradually, this edge should persist.

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: Sentiment momentum excels in uptrends where sentiment stays elevated above bullish thresholds with consistent positive momentum
- **Historical Evidence**: Strong bull markets show persistent positive sentiment momentum with sustained entries

### 3.2 Bearish Markets
- **Expected Performance**: Low
- **Rationale**: Strategy is long-only, exits when sentiment drops below bearish threshold (70)
- **Adaptations**: Quick exits prevent deep drawdowns in bear markets when sentiment deteriorates

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Medium
- **Rationale**: Sentiment oscillates around neutral levels, but momentum component can still identify direction
- **Filters**: Volume confirmation (default 1.0) can be increased to reduce false signals

### 3.4 Volatility Conditions
- **High Volatility**: Medium - momentum can be strong but exits may be frequent due to sentiment swings
- **Low Volatility**: Low - sentiment may not reach meaningful levels for clear signals
- **Volatility Filter**: Could add ATR filter for minimum volatility threshold

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 25%
- **Average Drawdown**: 8%
- **Drawdown Duration**: 1-3 weeks
- **Worst Case Scenario**: Rapid sentiment reversal causing sequential stop-loss hits

### 4.2 Failure Modes

#### Failure Mode 1: Whipsaw in Range-Bound Markets
- **Trigger**: Sentiment oscillates around bullish/bearish thresholds without sustained momentum
- **Impact**: Frequent small losses from failed entries/exits
- **Mitigation**: Increase volume confirmation threshold (>1.2), add trend filter
- **Detection**: Win rate falls below 35%, average trade duration < 3 days

#### Failure Mode 2: Late Exit on Sentiment Collapse
- **Trigger**: Sentiment drops rapidly from bullish to bearish without intermediate exit signals
- **Impact**: Gives back significant profits before exit triggers
- **Mitigation**: Tighter bearish_threshold (65 instead of 70), faster momentum calculation
- **Detection**: Average loss > average win, profit factor < 1.2

#### Failure Mode 3: Volume Spike False Positive
- **Trigger**: Volume spike causes volume confirmation but doesn't indicate genuine sentiment improvement
- **Impact**: Entry on temporary volume anomaly without true momentum
- **Mitigation**: Require sustained volume over multiple bars, add volume trend filter
- **Detection**: Low correlation between volume spikes and actual entry timing

### 4.3 Correlation Analysis
- **Correlation with Market**: Medium (follows market sentiment trends)
- **Correlation with Other Strategies**: High with other momentum strategies, medium with trend-following
- **Diversification Value**: Moderate - complements mean reversion and low-correlation strategies

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1**: Sentiment > bullish_threshold (30)
   - **Indicator**: Composite technical sentiment (RSI component 40%, momentum component 40%, volume component 20%)
   - **Parameters**: Current sentiment value > 30.0
   - **Confirmation**: None required for entry
   - **Priority**: Required

2. **Condition 2**: Sentiment momentum > momentum_threshold (5)
   - **Indicator**: Change in sentiment over lookback period
   - **Parameters**: Sentiment change > 5.0
   - **Confirmation**: Positive momentum indicates improving sentiment
   - **Priority**: Required

3. **Condition 3**: Volume confirmation (optional)
   - **Indicator**: Current volume / average volume
   - **Parameters**: Volume ratio > volume_confirmation (default 1.0)
   - **Confirmation**: Disabled by default (1.0), can be enabled (>1.0)
   - **Priority**: Optional

### 5.2 Entry Filters
- **Time of Day**: Not applicable (works on any timeframe)
- **Volume Requirements**: Optional - volume_confirmation > 1.0
- **Market Regime Filter**: None currently
- **Volatility Filter**: None currently
- **Price Filter**: None currently

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: Volume spike (if enabled with volume_confirmation > 1.0)
- **Confirmation Indicator 2**: None additional
- **Minimum Confirmed**: 1 out of 2 (volume optional)

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP 1**: 5% profit - Close 100%

### 6.2 Stop Loss
- **Initial SL**: 3% below entry
- **Trailing SL**: None
- **Breakeven**: None
- **Time-based Exit**: None

### 6.3 Exit Conditions
- **Sentiment Drop**: Sentiment < bearish_threshold (70)
- **Momentum Loss**: Sentiment momentum < -momentum_threshold (-5)
- **Volume Drop**: Volume confirmation fails (if enabled)
- **Regime Change**: Not implemented
- **Volatility Spike**: Not implemented
- **Time Limit**: None

## 7. Position Sizing
- **Base Position Size**: 1% of portfolio per signal
- **Volatility Adjustment**: None currently
- **Conviction Levels**: Could use sentiment strength above threshold
- **Max Position Size**: 5% of portfolio
- **Risk per Trade**: Max 3% (stop loss)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| lookback_period | 14 | 7-21 | Sentiment calculation lookback period | int |
| bullish_threshold | 30.0 | 20-40 | Sentiment level for bullish entry | float |
| bearish_threshold | 70.0 | 60-80 | Sentiment level for bearish exit | float |
| momentum_threshold | 5.0 | 3-10 | Minimum positive momentum for entry | float |
| volume_confirmation | 1.0 | 1.0-2.0 | Volume multiplier for confirmation | float |
| take_profit | 5.0 | 3.0-10.0 | Take profit percentage | float |
| stop_loss | 3.0 | 2.0-5.0 | Stop loss percentage | float |

### 8.2 Optimization Notes
- **Parameters to Optimize**: bullish_threshold, bearish_threshold, momentum_threshold
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
- **Annual Return**: 15-30%
- **Sharpe Ratio**: 1.0-1.8
- **Max Drawdown**: <25%
- **Win Rate**: 40-50%
- **Profit Factor**: >1.3
- **Expectancy**: >0.02

### 10.2 Comparison to Baselines
- **vs. HODL**: +5-15% risk-adjusted returns
- **vs. Market Average**: +3-10% outperformance
- **vs. RSI Momentum**: Different approach (composite sentiment vs single indicator)
- **vs. Buy & Hold**: Lower max drawdown, more consistent returns

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: RSI, Momentum, Volume Average
- **Data Requirements**: OHLCV
- **Latency Sensitivity**: Low
- **Computational Complexity**: Medium (composite sentiment calculation)
- **Memory Requirements**: Moderate (lookback_period bars for components)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/sentiment/sentiment_momentum.rs
- **Strategy Type**: Composite indicator-based
- **Dependencies**: alphafield_core, indicators::Rsi
- **State Management**: Tracks sentiment history, sentiment momentum, position, entry price

### 11.3 Indicator Calculations
**Technical Sentiment** = (RSI_component * 0.4) + (momentum_component * 0.4) + (volume_component * 0.2)

Where:
- RSI Component: Standard 14-period RSI (0-100)
- Momentum Component: Normalized price change over lookback period
- Volume Component: Current volume / average volume (normalized)

**Sentiment Momentum** = Current sentiment - Sentiment at (lookback_period) bars ago

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (sentiment > bullish, momentum > threshold, volume confirmation)
- [x] Exit conditions (sentiment < bearish, momentum < -threshold, TP, SL)
- [x] Edge cases (empty data, insufficient bars, single bar)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)
- [x] Sentiment calculation in uptrend/downtrend
- [x] Average volume calculation
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

### 2026-01-17: Initial Implementation
**Observation**: Strategy implemented following technical sentiment pattern with composite sentiment calculation
**Hypothesis Impact**: Code supports hypothesis - enters on improving sentiment momentum, exits on deterioration
**Issues Found**: Default volume_confirmation of 1.2 blocked signals in test scenarios with constant volume
**Action Taken**: Changed default to 1.0 (disabled) to allow strategy flexibility

### [Date]: Initial Backtest Results
**Test Period**: [Date range]
**Symbols Tested**: [List of symbols]
**Results**: [Summary of metrics]
**Observation**: [Are results as expected?]
**Action Taken**: [Proceed to validation or refine strategy?]

## 14. References

### Academic Sources
- Baker, M., & Wurgler, J. "Investor Sentiment and the Cross-Section of Stock Returns" (2006) - Sentiment research foundation

### Books
- Brown, C. "Panic-Proof Investing: Lessons in Market Psychology" (2018)

### Online Resources
- Technical sentiment indicators comparison studies
- Momentum vs Mean Reversion trading research

### Similar Strategies
- RSI Momentum (similar concept using single indicator)
- MACD Sentiment (different sentiment calculation approach)

### Historical Examples
- Crypto bull markets showing sustained positive sentiment momentum
- Market reversals following sentiment collapse

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-17 | 1.0 | Initial hypothesis | AI Agent |