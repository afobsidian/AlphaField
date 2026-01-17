# Trend+Mean Reversion Hybrid Strategy Hypothesis

## Metadata
- **Name**: Trend+Mean Reversion Hybrid
- **Category**: MultiIndicator
- **Sub-Type**: hybrid
- **Author**: AI Agent
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/multi_indicator/trend_mean_rev.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
During established uptrends (fast EMA > slow EMA), temporary pullbacks to oversold RSI levels represent buying opportunities rather than trend reversals. By entering long on RSI oversold conditions while maintaining uptrend context, the strategy captures bounces back to the trend direction, generating positive returns with higher win rates than pure trend following or pure mean reversion approaches. Expected average return of 3-6% per trade with 50-60% win rate.

**Null Hypothesis**: 
Combining trend filters with mean reversion entries does not provide statistically significant improvement over either approach alone. Pullbacks in uptrends are random noise, and RSI oversold signals during uptrends do not predict subsequent price rebounds better than random.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Trends in crypto markets exhibit momentum persistence but are not smooth. They include periods of retracement or consolidation as profit-taking occurs and new buyers enter at lower prices. RSI measures price velocity and identifies when price has moved rapidly downward relative to recent history (oversold). In an established uptrend, these oversold conditions often represent temporary imbalances that correct as trend-following buyers re-enter. By combining trend direction (EMA alignment) with mean reversion signal (RSI oversold), we filter counter-trend mean reversion trades (which fail in downtrends) while capturing the rebound potential of pullbacks in uptrends.

### 2.2 Market Inefficiency Exploited
The strategy exploits the tendency of markets to overreact to short-term news or sentiment (creating pullbacks) while maintaining longer-term trend direction (institutional positioning, adoption trends). Retail traders often panic-sell during pullbacks in uptrends, while algorithmic trend-following systems may be slower to react. Mean reversion at RSI oversold catches the panic before trend-following re-establishes, providing early entries at favorable prices.

### 2.3 Expected Duration of Edge
Pullback mean reversion in uptrends is a fundamental market property driven by behavioral biases (fear of missing out, profit-taking cycles) and liquidity dynamics. As markets become more algorithmically traded, pullbacks may become shallower or shorter, but the basic dynamic should persist. Edge expected to degrade slowly but remain relevant as long as human traders participate and trends exist.

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: Bull markets have frequent uptrends with regular pullbacks. EMA crossover identifies trend direction, RSI catches oversold pullbacks. High frequency of valid setups with favorable risk/reward (buying dips in uptrend).
- **Historical Evidence**: "Buy the dip" strategy has historically worked in crypto bull markets (2017, 2020-2021) when combined with trend confirmation.

### 3.2 Bearish Markets
- **Expected Performance**: Low
- **Rationale**: Bear markets have downtrends. EMA crossover keeps strategy flat or in downtrend mode. When trend temporarily reverses (bear rallies), RSI oversold entries may trigger but are likely to be false signals. Strategy's trend filter prevents counter-trend trades, but misses bear market bounces.
- **Adaptations**: Consider adding short-selling capability for bear market mean reversion (buying oversold in downtrend for short-covering bounces), or switch to cash-only mode in confirmed downtrends.

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Medium
- **Rationale**: Ranging markets lack clear trend direction. EMA crossover may flip frequently, causing whipsaws. RSI oversold signals may be followed by small bounces that don't reach TP. However, mean reversion works well in ranges if trend filter can be relaxed.
- **Filters**: Consider requiring minimum trend strength (EMA separation) or using volatility filter to avoid choppy ranging periods. Alternatively, disable trend filter in low-volatility regimes.

### 3.4 Volatility Conditions
- **High Volatility**: Higher performance potential but higher risk. Oversold RSI signals are more meaningful in high volatility (stronger mean reversion). However, SL triggers more frequently. Consider wider SL in high-volatility environments.
- **Low Volatility**: Lower performance. Oversold RSI signals may be weak, pullbacks shallow. Difficult to reach TP before trend reversals. Consider scaling position size or TP targets based on volatility.
- **Volatility Filter**: ATR calculated but not used as filter. Consider adding minimum ATR threshold to avoid low-volatility ranging.

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 20%
- **Average Drawdown**: 5-10%
- **Drawdown Duration**: 3-5 days (short due to 4% SL)
- **Worst Case Scenario**: Trend reversal shortly after RSI oversold entry (false bottom), causing SL hit. Multiple consecutive false signals during trend transition could produce 10-15% drawdown.

### 4.2 Failure Modes

#### Failure Mode 1: Trend Reversal (False Bottom)
- **Trigger**: Uptrend reverses to downtrend, RSI becomes oversold as price accelerates downward, but strategy interprets as pullback and enters long
- **Impact**: Entry at local high of downtrend, immediate continued decline, SL hit within 1-3 bars. Loss of 4% plus slippage. May occur multiple times during trend transition
- **Mitigation**: Require minimum EMA separation (trend strength) before accepting RSI signals, or add confirmation (price bounces off support, volume spike)
- **Detection**: High percentage of SL hits, low win rate (<45%), multiple trades hitting SL within short time windows

#### Failure Mode 2: Failed Pullback Recovery
- **Trigger**: Pullback is deeper than expected (news-driven selloff), RSI reaches oversold but price continues lower instead of rebounding
- **Impact**: Entry in middle of decline, SL hit as trend continues. Loss of 4%. May happen when trend is weakening but EMAs haven't crossed yet
- **Mitigation**: Use wider SL based on ATR (e.g., 2x ATR instead of fixed 4%), or require confirmation (price stabilization) before entry
- **Detection**: Many trades hitting SL at similar levels, deep drawdowns after entries

#### Failure Mode 3: Choppy/Sideways Whipsaws
- **Trigger**: Price oscillates in range, EMAs cross frequently, RSI moves between overbought and oversold without clear direction
- **Impact**: Series of small losses and small gains, high trade frequency, slippage and fees accumulate. Low profit factor despite reasonable win rate
- **Mitigation**: Add volatility filter (ATR > threshold) to avoid low-volatility chop, or require minimum time between trades
- **Detection**: High trade count, low average profit per trade, low Sharpe ratio

### 4.3 Correlation Analysis
- **Correlation with Market**: Medium. Strategy is long-only during uptrends, will participate in market upswings but may miss market rallies during downtrend phases.
- **Correlation with Other Strategies**: Medium-high with other trend-following strategies (shares uptrend detection). Medium with mean reversion strategies (similar entry signals but with trend filter). Low with sentiment-based strategies.
- **Diversification Value**: Moderate. Adds mean reversion component to trend-following portfolio, reducing correlation with pure momentum. However, still fundamentally trend-dependent (only buys in uptrends). Best combined with counter-trend or bear-market strategies for true diversification.

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1: Uptrend Confirmation**
   - **Indicator**: EMA crossover position
   - **Parameters**: Fast EMA(20) > Slow EMA(50)
   - **Confirmation**: Must be in uptrend (not just crossed recently)
   - **Priority**: Required (primary filter)

2. **Condition 2: RSI Oversold**
   - **Indicator**: RSI(14) value
   - **Parameters**: RSI < 30 (oversold threshold)
   - **Confirmation**: Must be below oversold at entry time
   - **Priority**: Required (trigger condition)

3. **Condition 3: Confidence Threshold**
   - **Indicator**: Composite confidence calculation
   - **Parameters**: Confidence > 0.0 (derived from EMA separation and RSI value)
   - **Confirmation**: Ensures signals have minimum quality
   - **Priority**: Required (internal calculation)

### 5.2 Entry Filters
- **Time of Day**: Not applicable (operates on any timeframe)
- **Volume Requirements**: No explicit volume filter
- **Market Regime Filter**: Uptrend filter (EMA fast > EMA slow) acts as regime filter
- **Volatility Filter**: ATR calculated but not used as filter currently
- **Price Filter**: No minimum price requirement

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: RSI turning up (not currently implemented)
- **Confirmation Indicator 2**: Volume spike on pullback (not currently implemented)
- **Minimum Confirmed**: 1 out of 2 (currently only RSI oversold required)

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP**: 4% profit (configurable, default 4%)
- **Trailing**: No trailing stop
- **Scaling**: Close full position at TP

### 6.2 Stop Loss
- **Initial SL**: 4% below entry (configurable, default 4%)
- **Trailing SL**: No trailing stop
- **Breakeven**: No breakeven movement
- **Time-based Exit**: None

### 6.3 Exit Conditions
- **Reversal Signal**: Fast EMA crosses below Slow EMA (trend reversal)
- **RSI Overbought Exit**: RSI crosses above 75 (overbought threshold)
- **Confidence Drop**: Not implemented
- **Trend Exhaustion**: RSI becomes extremely overbought (>80) - not currently implemented

## 7. Position Sizing

- **Base Position Size**: Not specified in strategy (handled externally)
- **Volatility Adjustment**: No internal volatility adjustment
- **Conviction Levels**: Signal strength (0.0-1.0) can be used for position sizing
- **Max Position Size**: Not specified (external)
- **Risk per Trade**: 4% risk via SL (assuming full position)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| ema_fast | 20 | 10-30 | Fast EMA period for trend detection | usize |
| ema_slow | 50 | 30-80 | Slow EMA period for trend detection | usize |
| rsi_period | 14 | 10-20 | RSI calculation period | usize |
| rsi_overbought | 75.0 | 70.0-80.0 | RSI overbought threshold for exits | f64 |
| rsi_oversold | 30.0 | 20.0-35.0 | RSI oversold threshold for entries | f64 |
| take_profit | 4.0 | 2.0-8.0 | Take profit percentage | f64 |
| stop_loss | 4.0 | 2.0-6.0 | Stop loss percentage | f64 |

### 8.2 Optimization Notes
- **Parameters to Optimize**: ema_fast, ema_slow (primary), rsi_oversold, rsi_overbought (secondary), take_profit, stop_loss (risk management)
- **Optimization Method**: Grid search for TP/SL ratio, genetic algorithm for EMA periods
- **Optimization Period**: 2 years of data minimum, walk-forward validation
- **Expected Overfitting Risk**: Medium-High (many parameters, trend-mean reversion interaction complex)
- **Sensitivity Analysis Required**: Yes, especially for EMA periods and RSI thresholds (affects trade frequency and signal quality)

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- **Timeframes**: Daily (primary), 4H (validation), 1H (intraday analysis)
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
- **Minimum Sharpe Ratio**: 1.0
- **Minimum Sortino Ratio**: 1.2
- **Maximum Max Drawdown**: 20%
- **Minimum Win Rate**: 45% (mean reversion with trend filter should have higher win rate)
- **Minimum Profit Factor**: 1.5
- **Minimum Robustness Score**: >65 (from walk-forward analysis)
- **Statistical Significance**: p < 0.05 for outperformance vs baseline
- **Walk-Forward Stability**: >55 (consistent performance across windows)

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 18-30% (depends on market cycle and number of pullbacks)
- **Sharpe Ratio**: 1.0-1.5
- **Max Drawdown**: 15-20% (during trend transitions)
- **Win Rate**: 50-60% (higher than pure trend following due to mean reversion entries)
- **Profit Factor**: 1.4-1.8
- **Expectancy**: 0.02-0.04 (2-4% per trade)
- **Average Trade Duration**: 3-7 days (shorter than pure trend following due to mean reversion exits)

### 10.2 Comparison to Baselines
- **vs. HODL**: Expected to outperform during choppy periods and trend transitions (avoids full drawdowns), but may underperform during straight-line uptrends where pullbacks are rare. Better risk-adjusted returns expected.
- **vs. Market Average**: Should generate alpha in trending markets with pullbacks due to superior entry timing (buying dips). Neutral in ranging markets.
- **vs. Pure Trend Following**: Higher win rate and lower drawdowns (entries at pullbacks vs chasing breakouts), but may miss some trend continuation moves. Expect 10-20% improvement in Sharpe ratio.
- **vs. Pure Mean Reversion**: Better performance in trending markets (avoids counter-trend trades), worse in ranging markets (misses mean reversion opportunities). Better overall due to trend filter.

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: EMA (fast and slow), RSI
- **Data Requirements**: OHLCV (open, high, low, close, volume)
- **Latency Sensitivity**: Low-Medium (RSI oversold can be detected on close, but early entries may be missed if waiting for close)
- **Computational Complexity**: Low (simple indicator calculations)
- **Memory Requirements**: Minimal (tracks last EMA values, last position state)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/multi_indicator/trend_mean_rev.rs
- **Strategy Type**: Multi-indicator (hybrid approach)
- **Dependencies**: alphafield_core (Bar, Signal), alphafield_strategy (indicators module)
- **State Management**: Tracks last position, entry price, last EMA values

### 11.3 Indicator Calculations
- **Fast EMA**: 20-period Exponential Moving Average of close price
- **Slow EMA**: 50-period Exponential Moving Average of close price
- **RSI**: 14-period Relative Strength Index using Wilder's smoothing
- **Confidence Calculation**: Combines EMA trend strength (normalized separation) and RSI position (oversold confidence)

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (all signals generated correctly)
- [x] Exit conditions (all exits triggered correctly)
- [x] Edge cases (empty data, single bar, etc.)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)
- [x] Confidence calculation (produces correct values)

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
**Observation**: Strategy implements trend-following (EMA crossover) with mean reversion entries (RSI oversold). Good hybrid approach combining trend filter with pullback entries.
**Hypothesis Impact**: Code structure supports hypothesis well. RSI oversold in uptrend should capture pullback reversals effectively.
**Issues Found**: None during implementation. Parameter validation is comprehensive.
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
- Lane, G. (1980s). RSI development and application (original RSI methodology)
- Murphy, J. (1999). "Technical Analysis of the Financial Markets" - EMA and RSI applications
- Brock, W., Lakonishok, J., & LeBaron, B. (1992). "Simple Technical Trading Rules and the Stochastic Properties of Stock Returns" - Evidence of mean reversion value

### Books
- Murphy, J. (1999). "Technical Analysis of the Financial Markets" - EMA and RSI guide
- Pring, M. (2002). "Technical Analysis Explained" - Trend + mean reversion combinations
- Kaufman, P. (2013). "Trading Systems and Methods" - Hybrid strategy approaches

### Online Resources
- Investopedia: RSI guide and EMA guide
- TradingView: RSI and EMA indicator documentation
- QuantConnect: Mean reversion in trends research articles

### Similar Strategies
- MACD+RSI Combo (multi_indicator/macd_rsi_combo.md) - Different indicator pair for similar concept
- EMA+RSI Strategy - Alternative trend+mean reversion combination
- Stochastic+EMA Strategy - Different oscillator with trend filter

### Historical Examples
- Bitcoin 2020-2021: Pullbacks to RSI oversold during uptrends consistently bounced higher
- Ethereum 2021: Multiple profitable dip-buying opportunities during major uptrend
- General observation: "Buy the dip" has been a profitable strategy in crypto during uptrend phases

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-11 | 1.0 | Initial hypothesis document | AI Agent |