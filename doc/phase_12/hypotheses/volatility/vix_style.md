## VIX-Style Strategy Hypothesis

## Metadata
- **Name**: VIX-Style Strategy
- **Category**: VolatilityBased
- **Sub-Type**: vix_style
- **Author**: AI Agent
- **Date**: 2026-01-15
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/volatility/vix_style.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
Crypto volatility index (like VIX) predicts market stress and reversals. By using high VIX as a contrarian signal—buying when fear is extreme (VIX > 85) and exiting when greed becomes extreme (VIX < 15)—the strategy generates positive returns with an average return of >6% over the holding period compared to momentum or trend-following strategies.

**Null Hypothesis**: 
VIX-style volatility index does not reliably predict market reversals and performs no better than random contrarian entry at similar volatility levels.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
The VIX (CBOE Volatility Index) measures expected market volatility and is a well-known contrarian indicator in traditional markets. High VIX indicates extreme fear and potential oversold conditions, often preceding market reversals. Low VIX indicates extreme greed and complacency, often preceding corrections. ATR percentile ranking creates a similar 0-100 volatility index for crypto markets, enabling the same contrarian approach.

### 2.2 Market Inefficiency Exploited
The strategy exploits emotional market extremes where participants panic sell at bottoms (high VIX/fear) and chase rallies at tops (low VIX/greed). Sentiment often reaches extremes just before reversals, creating opportunities for contrarian traders who buy when others are selling and sell when others are buying. The VIX-style index provides an objective measure of these sentiment extremes.

### 2.3 Expected Duration of Edge
This edge persists as long as market participants exhibit emotional extremes and sentiment-driven behavior. Since fear and greed are fundamental human psychological traits and markets will always cycle between panic and euphoria, this contrarian edge should maintain an advantage over trend-following or momentum strategies.

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: Medium
- **Rationale**: Bull markets have periods of extreme greed followed by corrections; VIX helps avoid chasing tops
- **Historical Evidence**: Bull markets produce low VIX readings before corrections

### 3.2 Bearish Markets
- **Expected Performance**: High
- **Rationale**: Bear markets produce extreme fear readings that signal potential bounces and reversals
- **Adaptations**: Contrarian entries in bear markets can be profitable but require quick exits

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Low-Medium
- **Rationale**: Sideways markets oscillate between fear and greed without clear trends
- **Filters**: Volume confirmation helps avoid entries in choppy ranges

### 3.4 Volatility Conditions
- **High Volatility**: High - extreme fear readings create excellent contrarian entry opportunities
- **Low Volatility**: Low - extreme greed readings indicate potential tops but fewer reversals
- **Volatility Filter**: The VIX index itself serves as the volatility/sentiment filter

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 25%
- **Average Drawdown**: 7%
- **Drawdown Duration**: 1-3 weeks
- **Worst Case Scenario**: Extreme fear persists (VIX remains high), indicating genuine market collapse rather than temporary oversold condition

### 4.2 Failure Modes

#### Failure Mode 1: Premature Entry in Sustained Downtrend
- **Trigger**: VIX reaches extreme fear (>85) but downtrend continues
- **Impact**: Stop loss hit as market continues falling instead of reversing
- **Mitigation**: Increase extreme_fear_threshold (from 0.85 to 0.90), add trend filter (require price below 200 SMA)
- **Detection**: Consecutive stop losses with sustained high VIX readings

#### Failure Mode 2: Late Exit on Volatility Expansion
- **Trigger**: VIX remains low (greed) while market starts reversing
- **Impact**: Gives back significant portion of gains before exit triggers
- **Mitigation**: Use faster lookback_period (from 100 to 75) for quicker VIX response, add price-based stop loss
- **Detection**: Large drawdowns from peak to exit relative to VIX readings

#### Failure Mode 3: Whipsaw in Volatile Market
- **Trigger**: VIX oscillates between fear and greed without clear direction
- **Impact**: Multiple entries/exits as sentiment swings back and forth
- **Mitigation**: Increase volume_multiplier (>1.5), require minimum bars at extreme (>3 bars)
- **Detection**: High trade frequency with low profit/loss per trade

### 4.3 Correlation Analysis
- **Correlation with Market**: Negative (contrarian strategy)
- **Correlation with Other Strategies**: Low with momentum and trend-following, Medium with mean reversion
- **Diversification Value**: High - contrarian approach provides diversification from trend-following strategies

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1**: Extreme fear detected
   - **Indicator**: VIX-style volatility index
   - **Parameters**: VIX > extreme_fear_threshold (85)
   - **Confirmation**: Market in extreme fear/oversold condition
   - **Priority**: Required

2. **Condition 2**: Volume confirmation
   - **Indicator**: Current volume / average volume
   - **Parameters**: Volume > average_volume × volume_multiplier (1.2)
   - **Confirmation**: Ensures market participation at extreme levels
   - **Priority**: Required

3. **Condition 3**: Sufficient data for VIX calculation
   - **Indicator**: Data length check
   - **Parameters**: Minimum lookback_period (100) bars for percentile calculation
   - **Confirmation**: VIX percentile ranking is meaningful
   - **Priority**: Required

### 5.2 Entry Filters
- **Time of Day**: Not applicable (works on any timeframe)
- **Volume Requirements**: Volume > average_volume × 1.2
- **Market Regime Filter**: Could filter for bearish or ranging markets only
- **Volatility Filter**: High VIX (extreme fear) required for entry
- **Price Filter**: Could filter for price below long-term average (contrarian focus)

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: Extreme fear (VIX > 85)
- **Confirmation Indicator 2**: Volume confirmation (>1.2× average)
- **Minimum Confirmed**: 2 out of 2 (both required)

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP 1**: 10% profit - Close 100%

### 6.2 Stop Loss
- **Initial SL**: 5% below entry
- **Trailing SL**: None
- **Breakeven**: None
- **Time-based Exit**: None

### 6.3 Exit Conditions
- **Take Profit Reached**: 10% profit target achieved
- **Stop Loss Triggered**: 5% loss threshold exceeded
- **Extreme Greed**: VIX < extreme_greed_threshold (15)
- **Volatility Drop**: VIX returns to neutral range (45-55)
- **Regime Change**: Not implemented
- **Time Limit**: None

## 7. Position Sizing
- **Base Position Size**: 1% of portfolio per signal
- **Volatility Adjustment**: Could increase size when VIX is extremely high (>90)
- **Conviction Levels**: Could use how far above extreme_fear_threshold VIX is
- **Max Position Size**: 5% of portfolio
- **Risk per Trade**: Max 5% (stop loss)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| atr_period | 14 | 7-21 | ATR calculation period | int |
| lookback_period | 100 | 50-200 | Lookback period for VIX percentile calculation | int |
| extreme_fear_threshold | 0.85 | 0.75-0.95 | Extreme fear threshold (VIX percentile) | float |
| extreme_greed_threshold | 0.15 | 0.05-0.25 | Extreme greed threshold (VIX percentile) | float |
| volume_multiplier | 1.2 | 1.0-2.0 | Volume confirmation multiplier | float |
| take_profit | 10.0 | 5.0-20.0 | Take profit percentage | float |
| stop_loss | 5.0 | 3.0-8.0 | Stop loss percentage | float |

### 8.2 Optimization Notes
- **Parameters to Optimize**: extreme_fear_threshold, extreme_greed_threshold, lookback_period
- **Optimization Method**: Walk-forward analysis
- **Optimization Period**: 2 years
- **Expected Overfitting Risk**: Medium (threshold parameters can be curve-fit)
- **Sensitivity Analysis Required**: Yes

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- **Timeframes**: 1H, 4H, 1D
- **Test Period**: 2019-2025 (6 years)
- **Assets**: BTC, ETH, SOL, 10+ others
- **Minimum Trades**: 50 (extreme conditions are less frequent)
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
- **Minimum Sharpe Ratio**: 0.8 (lower due to lower trade frequency)
- **Minimum Sortino Ratio**: 1.2
- **Maximum Max Drawdown**: 25%
- **Minimum Win Rate**: 45%
- **Minimum Profit Factor**: 1.6 (higher due to contrarian nature)
- **Minimum Robustness Score**: >60
- **Statistical Significance**: p < 0.05
- **Walk-Forward Stability**: >45

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 15-30%
- **Sharpe Ratio**: 0.8-1.5
- **Max Drawdown**: <25%
- **Win Rate**: 45-55%
- **Profit Factor**: >1.6
- **Expectancy**: >0.04

### 10.2 Comparison to Baselines
- **vs. HODL**: +5-15% risk-adjusted returns
- **vs. Market Average**: +3-10% outperformance
- **vs. Momentum Strategies**: Different approach (contrarian vs trend)
- **vs. Buy & Hold**: Similar returns but with lower drawdown during bear markets

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: ATR, Volume Average
- **Data Requirements**: OHLCV
- **Latency Sensitivity**: Low
- **Computational Complexity**: Medium (ATR calculation, percentile ranking, VIX index)
- **Memory Requirements**: High (lookback_period for percentile calculation)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/volatility/vix_style.rs
- **Strategy Type**: VIX-style contrarian
- **Dependencies**: alphafield_core, indicators::Atr
- **State Management**: Tracks ATR history, VIX index, sentiment classification, position, entry price

### 11.3 Indicator Calculations
**ATR**: Average True Range over atr_period (14) bars using Wilder's smoothing

**VIX-Style Index (0-100)**:
- Calculate ATR over atr_period (14) bars
- Store ATR values in history (lookback_period = 100 bars)
- VIX = (ATR values < current ATR) / total ATR values × 100

**Sentiment Classification**:
- **Extreme Greed**: 0-25 (VIX < extreme_greed_threshold = 15)
- **Greed**: 25-45
- **Neutral**: 45-55
- **Fear**: 55-75
- **Extreme Fear**: 75-100 (VIX > extreme_fear_threshold = 85)

**Volume Confirmation**: Current volume > average_volume × volume_multiplier (1.2)

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (extreme fear detection, volume confirmation)
- [x] Exit conditions (TP, SL, extreme greed, volatility drop)
- [x] Edge cases (empty data, insufficient bars, single bar)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)
- [x] ATR calculation
- [x] VIX index calculation (ATR percentile)
- [x] Sentiment classification (fear/greed bands)
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
- [ ] VIX index correlation with actual market reversals

## 13. Research Journal

### 2026-01-15: Initial Implementation
**Observation**: Strategy implemented with VIX-style index using ATR percentile ranking
**Hypothesis Impact**: Code supports hypothesis - enters on extreme fear (VIX > 85), exits on extreme greed (VIX < 15)
**Issues Found**: None during implementation
**Action Taken**: Ready for testing

### [Date]: Initial Backtest Results
**Test Period**: [Date range]
**Symbols Tested**: [List of symbols]
**Results**: [Summary of metrics]
**Observation**: [Are results as expected?]
**Action Taken**: [Proceed to validation or refine strategy?]

### [Date]: Extreme Fear Threshold Analysis
**Observation**: [How does performance vary with extreme_fear_threshold?]
**Hypothesis Impact**: [Does optimal threshold validate contrarian hypothesis?]
**Issues Found**: [Any parameter-specific problems]
**Action Taken**: [Parameter adjustments or range modifications]

### [Date]: VIX vs Actual Market Reversals
**Observation**: [How well does VIX predict actual market reversals?]
**Hypothesis Impact**: [Does VIX provide reliable contrarian signals?]
**Issues Found**: [Scenarios where VIX fails to predict reversals]
**Action Taken**: [Hybrid approach or additional filters]

## 14. References

### Academic Sources
- Whaley, R. E. "The Investor Fear Gauge" (2000) - Original VIX research
- CBOE. "CBOE Volatility Index VIX White Paper" (2003) - VIX methodology
- Giot, P. "Relationships Between Implied Volatility Indexes and Stock Index Returns" (2005) - VIX predictive power

### Books
- Connors, L., & Alvarez, C. "Short Term Trading Strategies That Work" - VIX trading strategies
- Covel, M. W. "Trend Following" - Contrarian approaches vs trend following
- Taleb, N. N. "The Black Swan" - Fat tails and extreme events

### Online Resources
- VIX trading research and case studies
- Crypto volatility indices and sentiment measures
- Contrarian trading methodology papers
- Fear and greed indicators in financial markets

### Similar Strategies
- Mean Reversion (similar contrarian concept)
- RSI Mean Reversion (indicator-based contrarian)
- Volatility Regime (different volatility interpretation)
- Momentum Strategies (opposite approach)

### Historical Examples
- 2020 COVID crash (extreme fear VIX preceded recovery)
- 2021 crypto market tops (extreme greed VIX preceded corrections)
- 2022 crypto bear market bottoms (extreme fear VIX preceded rallies)
- Various market reversals showing VIX predictive power

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-15 | 1.0 | Initial hypothesis | AI Agent |
