# Confidence-Weighted Strategy Hypothesis

## Metadata
- **Name**: Confidence-Weighted
- **Category**: MultiIndicator
- **Sub-Type**: confidence_sizing
- **Author**: AI Agent
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/multi_indicator/confidence_weighted.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
Weighting trade entries based on composite confidence (combining trend strength, MACD momentum, and RSI position) improves risk-adjusted returns compared to unfiltered entries. By only entering when multiple indicators align and confidence exceeds a threshold, the strategy filters out low-quality signals while capturing high-conviction setups, leading to higher win rates (45-55%), smaller drawdowns (15-20%), and improved Sharpe ratios (1.2-1.8).

**Null Hypothesis**: 
Confidence-based filtering does not improve risk-adjusted returns. The additional filtering reduces trade frequency without improving signal quality, leading to similar or worse performance than unfiltered indicator combinations.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Technical indicators provide probabilistic information about market direction. When multiple indicators agree (e.g., uptrend + MACD bullish + RSI neutral), the combined probability of continuation is higher than when indicators disagree. Confidence weighting quantifies this alignment as a score (0.0-1.0), allowing the strategy to distinguish between strong confluence (high confidence) and weak/mixed signals (low confidence). By requiring minimum confidence for entry, we reduce low-probability trades that contribute to whipsaw losses. This is analogous to betting more heavily when odds are favorable.

### 2.2 Market Inefficiency Exploited
The strategy exploits the fact that markets are not perfectly efficient - multiple technical indicators sometimes provide redundant confirmation of the same directional move. Retail and amateur traders often act on single indicator signals without confirmation, creating inefficiencies that informed traders can capture by requiring multi-indicator confluence. Additionally, many market participants ignore the strength of signals, treating weak and strong setups equally; confidence weighting exploits this by sizing or filtering based on signal quality.

### 2.3 Expected Duration of Edge
Multi-indicator confluence is a robust concept that should persist as long as markets exhibit trends and momentum. However, as algorithmic trading increases, pure technical signals may become less profitable as more participants exploit similar patterns. Confidence filtering's advantage of reducing low-quality trades should remain valuable even as individual edges degrade. Edge expected to gradually weaken but remain viable for several years.

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: Bull markets have strong trend alignment (EMA fast > EMA slow), frequent MACD bullish crossovers, and RSI often in neutral-to-overbought zones. High-confidence setups occur regularly when all three indicators align. Strategy captures strong uptrend entries with good risk/reward.
- **Historical Evidence**: Multi-indicator strategies have historically performed well in crypto bull markets when filters reduce whipsaws.

### 3.2 Bearish Markets
- **Expected Performance**: Medium
- **Rationale**: Bear markets have downtrends and weak bear rallies. Trend filter (fast EMA > slow EMA) prevents most entries. Occasional entries occur during bear market rallies when temporary trend alignment happens, but these have lower confidence and may fail. Strategy's position sizing (if using confidence) or filtering helps limit risk.
- **Adaptations**: Consider reducing min_confidence threshold in bear markets to capture more rally entries, or switch to cash-only mode in confirmed bear regimes.

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Low
- **Rationale**: Ranging markets lack clear trend direction. EMA alignment fluctuates, MACD oscillates around signal line, RSI moves between overbought and oversold. High-confidence setups are rare (indicators rarely all align). When they do occur, they may be false signals in choppy conditions.
- **Filters**: Consider adding volatility filter (ATR > threshold) to avoid low-volatility ranging, or requiring minimum time between trades to reduce whipsaw frequency.

### 3.4 Volatility Conditions
- **High Volatility**: Mixed performance. High volatility creates more indicator alignment opportunities but also faster reversals. Confidence calculations may fluctuate rapidly. Consider longer lookback periods or wider TP/SL in high volatility.
- **Low Volatility**: Lower performance. Indicators move slowly, reducing high-confidence setup frequency. Price moves may be too small to reach TP before reversal. Consider scaling out of low-volatility environments.
- **Volatility Filter**: ATR calculated but not used as filter currently. Consider adding minimum ATR threshold.

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 22%
- **Average Drawdown**: 6-10%
- **Drawdown Duration**: 4-8 days (typically short due to 5% SL and confidence-based exits)
- **Worst Case Scenario**: Regime change shortly after high-confidence entry (trend reversal), causing SL hit. Multiple false high-confidence signals during transition could produce 15-20% drawdown.

### 4.2 Failure Modes

#### Failure Mode 1: False High-Confidence Signal in Range
- **Trigger**: Indicators temporarily align in choppy range (e.g., EMA crosses, MACD crosses, RSI neutral), creating high confidence score, but no sustained trend
- **Impact**: Entry at local extremum, immediate reversal or sideways movement, SL hit within 1-3 bars. Loss of 5% plus slippage. May occur when volatility is low and indicators oscillate around thresholds.
- **Mitigation**: Add minimum volatility filter (ATR > threshold) or require minimum trend duration (e.g., EMAs aligned for N bars)
- **Detection**: Low win rate on high-confidence trades, high SL hit rate, high trade frequency in ranging markets

#### Failure Mode 2: Confidence Drop Exit Too Early
- **Trigger**: Entry with high confidence (e.g., 0.8), but confidence quickly drops (e.g., to 0.4) as trend weakens or RSI moves, triggering exit before TP
- **Impact**: Small gains lost or converted to small losses if exit triggered after small move. Reduces profit per trade, potentially missing multi-day uptrends.
- **Mitigation**: Increase confidence drop threshold (e.g., exit only if drops to 50% or less), or use absolute confidence threshold instead of percentage drop
- **Detection**: High exit rate, low average profit per winning trade, many trades hitting TP shortly after exit

#### Failure Mode 3: Trend Reversal Before Exit Signal
- **Trigger**: Trend reverses (fast EMA crosses below slow EMA or MACD crosses below signal) before TP or SL hit, but exit signal doesn't trigger in time
- **Impact**: Position held through reversal, larger loss than expected SL. May turn small gain into loss. Depends on how quickly reversal signal is detected.
- **Mitigation**: Prioritize trend reversal exits over confidence-based exits, or implement faster trend detection (shorter EMAs)
- **Detection**: Large losses exceeding SL threshold, high drawdowns after periods of gains

### 4.3 Correlation Analysis
- **Correlation with Market**: Medium-High. Strategy is long-only trend-following with momentum confirmation, will generally move with market but may underperform during choppy periods due to filtering.
- **Correlation with Other Strategies**: High with other multi-indicator strategies (uses similar indicators). High with momentum strategies (MACD-based). Medium with mean reversion strategies (different entry logic). Low with sentiment-based strategies.
- **Diversification Value**: Limited as it's fundamentally trend-following. Confidence filtering provides some differentiation (reduces trade frequency), but not independent signal source. Best combined with counter-trend, mean reversion, or sentiment strategies for true diversification.

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1: Trend Alignment**
   - **Indicator**: EMA position
   - **Parameters**: Fast EMA(20) > Slow EMA(50)
   - **Confirmation**: Must be in uptrend
   - **Priority**: Required (primary filter)

2. **Condition 2: MACD Bullish Crossover**
   - **Indicator**: MACD crossover
   - **Parameters**: MACD line crosses above signal line
   - **Confirmation**: Must be a true crossover (not just above)
   - **Priority**: Required (trigger condition)

3. **Condition 3: Minimum Confidence**
   - **Indicator**: Composite confidence score
   - **Parameters**: Confidence >= 0.5 (min_confidence threshold)
   - **Confirmation**: Ensures high-quality signal
   - **Priority**: Required (quality filter)

### 5.2 Entry Filters
- **Time of Day**: Not applicable (operates on any timeframe)
- **Volume Requirements**: No explicit volume filter
- **Market Regime Filter**: Uptrend filter (EMA fast > EMA slow) acts as regime filter
- **Volatility Filter**: ATR calculated but not used as filter currently
- **Price Filter**: No minimum price requirement

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: MACD histogram positive (already in crossover condition)
- **Confirmation Indicator 2**: RSI not overbought (not currently required for entry)
- **Minimum Confirmed**: 1 out of 2 (currently only MACD crossover + confidence required)

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP**: 5% profit (configurable, default 5%)
- **Trailing**: No trailing stop
- **Scaling**: Close full position at TP

### 6.2 Stop Loss
- **Initial SL**: 5% below entry (configurable, default 5%)
- **Trailing SL**: No trailing stop
- **Breakeven**: No breakeven movement
- **Time-based Exit**: None

### 6.3 Exit Conditions
- **Reversal Signal 1**: Fast EMA crosses below Slow EMA (trend reversal)
- **Reversal Signal 2**: MACD line crosses below signal line
- **Confidence Drop**: Current confidence < 60% of entry confidence (adaptive exit)
- **RSI Overbought**: Not implemented as primary exit (could be added)
- **Regime Change**: Not explicitly implemented (implicit via trend reversal exits)

## 7. Position Sizing

- **Base Position Size**: Not specified in strategy (handled externally)
- **Volatility Adjustment**: No internal volatility adjustment
- **Conviction Levels**: Confidence score (0.0-1.0) can be used for position sizing (higher confidence = larger position)
- **Max Position Size**: Not specified (external)
- **Risk per Trade**: 5% risk via SL (assuming full position)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| ema_fast | 20 | 10-30 | Fast EMA period for trend detection | usize |
| ema_slow | 50 | 30-80 | Slow EMA period for trend detection | usize |
| macd_fast | 12 | 8-20 | MACD fast EMA period | usize |
| macd_slow | 26 | 20-40 | MACD slow EMA period | usize |
| macd_signal | 9 | 7-12 | MACD signal line period | usize |
| rsi_period | 14 | 10-20 | RSI calculation period | usize |
| min_confidence | 0.5 | 0.3-0.7 | Minimum confidence score for entry | f64 |
| take_profit | 5.0 | 3.0-10.0 | Take profit percentage | f64 |
| stop_loss | 5.0 | 3.0-8.0 | Stop loss percentage | f64 |

### 8.2 Optimization Notes
- **Parameters to Optimize**: min_confidence (primary - affects trade quality/frequency balance), ema_fast, ema_slow (secondary - trend detection), take_profit, stop_loss (risk management)
- **Optimization Method**: Grid search for min_confidence and TP/SL ratio, Bayesian optimization for EMA periods
- **Optimization Period**: 2 years of data minimum, walk-forward validation
- **Expected Overfitting Risk**: Medium (confidence calculation is somewhat subjective, market structure stable)
- **Sensitivity Analysis Required**: Yes, especially for min_confidence threshold (critical parameter affecting trade frequency and quality)

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- **Timeframes**: Daily, 4H (primary), 1H (validation)
- **Test Period**: 2019-2025 (6 years), covering multiple market cycles
- **Assets**: BTC, ETH, SOL, and 10+ other top crypto assets (minimum 15 assets)
- **Minimum Trades**: 50 trades per asset for statistical significance
- **Slippage**: 0.1% per trade (typical crypto spot)
- **Commission**: 0.1% per trade (typical exchange fees)
- **Slippage Model**: Implementation uses actual bar prices (close), no additional slippage model

### 9.2 Validation Techniques
- [x] Walk-forward analysis (rolling window) - Implemented in backtest module
- [x] Monte Carlo simulation (trade sequence randomization) - Implemented in backtest module
- [ ] Parameter sweep (sensitivity analysis) - To be performed
- [x] Regime analysis (bull/bear/sideways) - Implemented in backtest module
- [x] Cross-asset validation (multiple symbols) - Required validation
- [ ] Bootstrap validation (resampling) - Optional for additional robustness
- [ ] Permutation testing (randomness check) - Optional for hypothesis testing

### 9.3 Success Criteria
- **Minimum Sharpe Ratio**: 1.2
- **Minimum Sortino Ratio**: 1.5
- **Maximum Max Drawdown**: 22%
- **Minimum Win Rate**: 45% (confidence filtering should improve vs unfiltered)
- **Minimum Profit Factor**: 1.5
- **Minimum Robustness Score**: >65 (from walk-forward analysis)
- **Statistical Significance**: p < 0.05 for outperformance vs baseline
- **Walk-Forward Stability**: >60 (consistent performance across windows)

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 22-35% (depends on market cycle and confidence threshold)
- **Sharpe Ratio**: 1.2-1.8
- **Max Drawdown**: 18-22% (during bear markets)
- **Win Rate**: 45-55% (confidence filtering should improve vs unfiltered)
- **Profit Factor**: 1.5-2.0
- **Expectancy**: 0.025-0.045 (2.5-4.5% per trade)
- **Average Trade Duration**: 7-12 days

### 10.2 Comparison to Baselines
- **vs. HODL**: Expected to outperform HODL during bear markets (capital preservation) and choppy periods (confidence filtering avoids poor entries), but may underperform during strong bull runs if high-confidence setups are rare. Overall risk-adjusted outperformance expected.
- **vs. Market Average**: Should generate alpha in trending markets due to timing improvements (MACD crossover trigger). Neutral in ranging markets.
- **vs. Unfiltered Indicators**: Confidence filtering should reduce whipsaw losses and improve win rate, though may reduce total trade count. Expected improvement in Sharpe ratio of 15-25%.
- **vs. Buy & Hold**: Better risk-adjusted returns due to drawdown control and quality filtering, but may miss some gains during strongest trends if confidence threshold is too strict.

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: EMA (fast and slow), MACD (fast EMA, slow EMA, signal line), RSI
- **Data Requirements**: OHLCV (open, high, low, close, volume)
- **Latency Sensitivity**: Medium (confidence score requires multiple indicators, MACD crossover can be detected on close)
- **Computational Complexity**: Low-Medium (multiple indicator calculations + confidence weighting)
- **Memory Requirements**: Moderate (tracks last EMA values, last MACD/signal values, entry confidence)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/multi_indicator/confidence_weighted.rs
- **Strategy Type**: Multi-indicator (confidence-based filtering)
- **Dependencies**: alphafield_core (Bar, Signal), alphafield_strategy (indicators module)
- **State Management**: Tracks last position, entry price, entry confidence, last EMA/MACD/signal values

### 11.3 Indicator Calculations
- **Fast EMA**: 20-period Exponential Moving Average of close price
- **Slow EMA**: 50-period Exponential Moving Average of close price
- **MACD**: 12-period EMA of price minus 26-period EMA of price
- **MACD Signal**: 9-period EMA of MACD line
- **MACD Histogram**: MACD line minus MACD signal line
- **RSI**: 14-period Relative Strength Index using Wilder's smoothing
- **Trend Confidence**: Normalized EMA spread (0-1 range based on min/max historical)
- **RSI Confidence**: Inverted distance from extreme (higher at 50, lower at 0/100)
- **Composite Confidence**: Weighted average of trend, MACD, and RSI confidences

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (all signals generated correctly)
- [x] Exit conditions (all exits triggered correctly)
- [x] Edge cases (empty data, single bar, etc.)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)
- [x] Confidence calculation (produces correct values for various scenarios)

### 12.2 Integration Tests
- [x] Backtest execution (runs without errors)
- [x] Performance calculation (metrics are correct)
- [x] Dashboard integration (API works)
- [x] Database integration (saves correctly)

### 12.3 Research Tests
- [ ] Hypothesis validation (supports primary hypothesis)
- [ ] Statistical significance (p < 0.05)
- [ ] Regime analysis (performance as expected)
- [ ] Robustness testing (stable across parameters)

## 13. Research Journal

### 2026-01-11: Initial Implementation
**Observation**: Strategy implements comprehensive confidence calculation combining trend (EMA), momentum (MACD), and mean reversion (RSI). Min confidence threshold provides quality filtering. Multiple exit conditions (TP, SL, confidence drop, reversals).
**Hypothesis Impact**: Code structure supports hypothesis well. Confidence weighting should improve signal quality and reduce low-probability trades.
**Issues Found**: None during implementation. Parameter validation is comprehensive. Confidence drop exit is adaptive and sophisticated.
**Action Taken**: Implementation complete, ready for backtesting validation.

### [Date]: Initial Backtest Results
**Test Period**: TBD
**Symbols Tested**: TBD
**Results**: TBD
**Observation**: TBD
**Action Taken**: Proceed to validation or refine strategy?

### [Date]: Parameter Optimization
**Optimization Method**: TBD
**Best Parameters**: TBD
**Optimization Score**: TBD
**Overfitting Check**: TBD
**Action Taken**: TBD

### [Date]: Walk-Forward Validation
**Configuration**: TBD
**Results**: TBD
**Stability Score**: TBD
**Decision**: TBD

### [Date]: Monte Carlo Simulation
**Number of Simulations**: TBD
**95% Confidence Interval**: TBD
**Best Case**: TBD
**Worst Case**: TBD
**Observation**: TBD

### [Date]: Final Decision
**Final Verdict**: TBD
**Reasoning**: TBD
**Deployment**: TBD
**Monitoring**: TBD

## 14. References

### Academic Sources
- Appel, G. (1979). "The Moving Average Convergence-Divergence Trading Method" - Original MACD methodology
- Wilder, J. (1978). "New Concepts in Technical Trading Systems" - RSI development
- Kaufman, P. (2013). "Trading Systems and Methods" - Multi-indicator confidence weighting concepts

### Books
- Murphy, J. (1999). "Technical Analysis of the Financial Markets" - MACD, RSI, and EMA applications
- Pring, M. (2002). "Technical Analysis Explained" - Multi-indicator confluence strategies
- Kaufman, P. (2013). "Trading Systems and Methods" - Confidence-based trading approaches

### Online Resources
- Investopedia: MACD guide, RSI guide, EMA guide
- TradingView: Multi-indicator strategies documentation and community examples
- QuantConnect: Confidence weighting and signal quality research articles

### Similar Strategies
- MACD+RSI Combo (multi_indicator/macd_rsi_combo.md) - Similar indicator combination without confidence weighting
- Trend+Mean Rev (multi_indicator/trend_mean_rev.md) - Different hybrid approach
- Adaptive Combo (multi_indicator/adaptive_combo.md) - Performance-based weighting instead of static confidence

### Historical Examples
- Bitcoin 2020-2021: High-confidence setups (uptrend + MACD bullish + RSI neutral) frequently preceded strong rallies
- Ethereum 2018-2019: Confidence filtering would have reduced whipsaws in bear market rallies
- General observation: Multi-indicator confluence has historically provided higher win rates than single indicators when properly thresholded

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-11 | 1.0 | Initial hypothesis document | AI Agent |