# MACD+RSI Combo Strategy Hypothesis

## Metadata
- **Name**: MACD+RSI Combo
- **Category**: MultiIndicator
- **Sub-Type**: indicator_confluence
- **Author**: AI Agent
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/multi_indicator/macd_rsi_combo.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
When MACD crosses above its signal line (bullish momentum shift) while RSI is not overbought (not at peak), the asset is likely entering a sustained uptrend. This confluence of trend confirmation (MACD) and momentum filter (RSI) produces higher-quality signals than either indicator alone, leading to positive risk-adjusted returns over 7-20 day holding periods with average returns of 3-8% per trade.

**Null Hypothesis**: 
The combination of MACD crossovers and RSI filters does not provide statistically significant improvement over MACD crossovers alone. Any observed outperformance is due to random noise or data-snooping bias.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
MACD captures trend momentum changes by comparing short-term and long-term moving averages. Crossovers signal momentum shifts. However, MACD alone can generate false signals in choppy markets or late signals at trend peaks. RSI measures the velocity of price changes and identifies overbought/oversold conditions. By requiring RSI to not be overbought (>70), we filter out late entries that occur when price is extended and likely to reverse. This combination aims to catch early-to-mid trend entries while avoiding exhaustion entries.

### 2.2 Market Inefficiency Exploited
The strategy exploits the tendency of momentum to persist after a confirmed crossover (momentum anomaly) while avoiding the tendency for reversals at extreme RSI levels (mean reversion tendency at extremes). Market participants often chase trends at overextended prices; RSI filter helps avoid this behavioral bias.

### 2.3 Expected Duration of Edge
Momentum effects have historically persisted in crypto markets due to asymmetric information and retail trader behavior. However, as algorithmic trading increases, pure momentum edges may degrade. The RSI filter should provide some durability as mean reversion at extremes is a robust market property. Edge expected to persist as long as crypto markets remain inefficient.

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: Bull markets have strong, sustained uptrends. MACD crossovers occur frequently as momentum builds. RSI filter prevents entering at local tops, improving entry timing. The strategy captures multi-day uptrends effectively.
- **Historical Evidence**: MACD crossovers have historically performed well in crypto bull markets (2017, 2020-2021).

### 3.2 Bearish Markets
- **Expected Performance**: Medium
- **Rationale**: Bear markets have downtrends with sharp bear rallies. MACD crossovers may signal short-term bounces. RSI filter helps avoid entering at local peaks of bear rallies. However, overall market pressure works against long entries.
- **Adaptations**: Consider reducing position sizes or adding additional trend filters (e.g., 200-day SMA below price) in bear markets.

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Low
- **Rationale**: Ranging markets produce frequent MACD crossovers as price oscillates, leading to whipsaw losses. RSI filter doesn't help much as price moves between overbought and oversold without sustained trends.
- **Filters**: Consider adding ATR filter to avoid low-volatility ranging periods, or requiring minimum days since last trade to reduce trade frequency.

### 3.4 Volatility Conditions
- **High Volatility**: Performance may be mixed. MACD crossovers are more significant in high volatility, but SL triggers more frequently. RSI thresholds may need adjustment (wider ranges like 30/70).
- **Low Volatility**: Lower performance due to smaller price moves. SL triggers less but TP also harder to reach. Consider scaling out of low-volatility assets.
- **Volatility Filter**: ATR period of 14 is calculated but not used as filter. Consider adding minimum ATR threshold.

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 25%
- **Average Drawdown**: 8-12%
- **Drawdown Duration**: 3-7 days (typically short due to 5% SL)
- **Worst Case Scenario**: Multiple whipsaws in ranging market could produce 5-10 consecutive small losses, or a gap down through SL causing larger-than-expected loss.

### 4.2 Failure Modes

#### Failure Mode 1: Whipsaw in Ranging Market
- **Trigger**: Price oscillates in range, MACD crosses back and forth, RSI moves between zones
- **Impact**: Series of small losses (3-5% each), potentially 5-10 consecutive losses. Drawdown of 15-25% possible
- **Mitigation**: Add volatility filter (ATR > threshold) or reduce position sizing during low-volatility periods
- **Detection**: Low win rate (<40%), high trade frequency, small average trade duration

#### Failure Mode 2: Late Entry at Trend Peak
- **Trigger**: MACD crossover occurs after trend has been running, RSI nears overbought but not crossed threshold yet
- **Impact**: Entry at local top, immediate reversal, SL hit within 1-2 bars. Loss of 5% plus slippage
- **Mitigation**: Lower RSI overbought threshold to 65, or require price to be below a certain multiple of recent ATR above MA
- **Detection**: High percentage of trades hitting SL vs TP, frequent early exits

#### Failure Mode 3: Failed Breakout/Fakeout
- **Trigger**: MACD crossover on news or short-term impulse, but no sustained follow-through
- **Impact**: Small gains lost to SL, or small loss if price reverses quickly
- **Mitigation**: Add volume confirmation or minimum breakout magnitude (e.g., price must move >1% above EMA)
- **Detection**: Many trades hitting breakeven or small losses, low profit factor

### 4.3 Correlation Analysis
- **Correlation with Market**: Medium-High. Strategy is long-only trend-following, will move with market but may outperform during strong trends due to timing filters
- **Correlation with Other Strategies**: High with other momentum strategies (MACD, EMA crossover). Medium with mean reversion strategies (often opposite signals). Low with sentiment-based strategies
- **Diversification Value**: Limited as it's trend-following. Adds value through signal quality filtering (RSI), but not independent signal source. Best combined with mean reversion for diversification.

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1: MACD Crossover**
   - **Indicator**: MACD line crossing above signal line
   - **Parameters**: MACD(12,26,9) line > MACD(12,26,9) signal, with previous line <= previous signal
   - **Confirmation**: Must be a true crossover (not just above)
   - **Priority**: Required

2. **Condition 2: RSI Not Overbought**
   - **Indicator**: RSI(14) value
   - **Parameters**: RSI < 70 (overbought threshold)
   - **Confirmation**: Must be below overbought at entry time
   - **Priority**: Required

3. **Condition 3: Confidence Threshold**
   - **Indicator**: Composite confidence calculation
   - **Parameters**: Confidence > 0.3 (derived from MACD strength and RSI position)
   - **Confirmation**: Ensures signals have minimum quality
   - **Priority**: Required (internal calculation)

### 5.2 Entry Filters
- **Time of Day**: Not applicable (operates on any timeframe)
- **Volume Requirements**: No explicit volume filter, though higher volume increases signal reliability
- **Market Regime Filter**: No explicit filter, but performs best in trending regimes
- **Volatility Filter**: ATR calculated but not used as filter currently
- **Price Filter**: No minimum price requirement

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: MACD histogram turning positive (optional, already in crossover condition)
- **Confirmation Indicator 2**: RSI rising (not currently implemented)
- **Minimum Confirmed**: 1 out of 2 (currently only MACD crossover required)

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
- **Reversal Signal**: MACD line crosses below signal line
- **RSI Overbought Exit**: RSI crosses above 70 (not currently implemented as primary exit)
- **Confidence Drop**: Entry confidence drops to 60% of initial (not currently implemented)
- **Trend Reversal**: Fast EMA crosses below slow EMA (not currently implemented)

## 7. Position Sizing

- **Base Position Size**: Not specified in strategy (handled externally)
- **Volatility Adjustment**: No internal volatility adjustment
- **Conviction Levels**: Signal strength (0.0-1.0) can be used for position sizing
- **Max Position Size**: Not specified (external)
- **Risk per Trade**: 5% risk via SL (assuming full position)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| macd_fast | 12 | 8-20 | MACD fast EMA period | usize |
| macd_slow | 26 | 20-40 | MACD slow EMA period | usize |
| macd_signal | 9 | 7-12 | MACD signal line period | usize |
| rsi_period | 14 | 10-20 | RSI calculation period | usize |
| rsi_overbought | 70.0 | 65.0-80.0 | RSI overbought threshold | f64 |
| rsi_oversold | 30.0 | 20.0-35.0 | RSI oversold threshold | f64 |
| take_profit | 5.0 | 3.0-10.0 | Take profit percentage | f64 |
| stop_loss | 5.0 | 3.0-8.0 | Stop loss percentage | f64 |

### 8.2 Optimization Notes
- **Parameters to Optimize**: macd_fast, macd_slow, take_profit, stop_loss (primary); rsi_overbought (secondary)
- **Optimization Method**: Grid search for TP/SL, Bayesian optimization for MACD periods
- **Optimization Period**: 2 years of data minimum, walk-forward validation
- **Expected Overfitting Risk**: Medium (many parameters, but market structure relatively stable)
- **Sensitivity Analysis Required**: Yes, especially for TP/SL ratio and RSI thresholds

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
- **Minimum Sharpe Ratio**: 1.0
- **Minimum Sortino Ratio**: 1.5
- **Maximum Max Drawdown**: 25%
- **Minimum Win Rate**: 40% (trend following typically has lower win rate)
- **Minimum Profit Factor**: 1.5
- **Minimum Robustness Score**: >70 (from walk-forward analysis)
- **Statistical Significance**: p < 0.05 for outperformance vs baseline
- **Walk-Forward Stability**: >60 (consistent performance across windows)

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 20-35% (depends on market cycle)
- **Sharpe Ratio**: 1.2-1.8
- **Max Drawdown**: 20-25% (during bear markets)
- **Win Rate**: 42-52% (typical for trend following)
- **Profit Factor**: 1.5-2.0
- **Expectancy**: 0.03-0.05 (3-5% per trade)
- **Average Trade Duration**: 7-15 days

### 10.2 Comparison to Baselines
- **vs. HODL**: Expected to outperform HODL during bear markets (capital preservation) and choppy periods (avoid drawdowns), but may underperform during strong bull runs if RSI filter causes missed entries. Overall risk-adjusted outperformance expected.
- **vs. Market Average**: Should generate alpha in trending markets due to timing improvements. May lag in pure momentum phases.
- **vs. MACD Only**: RSI filter should reduce whipsaw losses and improve win rate, though may reduce total trade count. Expected improvement in Sharpe ratio of 10-20%.
- **vs. Buy & Hold**: Better risk-adjusted returns due to drawdown control, but may miss some gains during strongest trends.

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: MACD (fast EMA, slow EMA, signal line), RSI
- **Data Requirements**: OHLCV (open, high, low, close, volume)
- **Latency Sensitivity**: Medium (MACD crossovers can be detected on close)
- **Computational Complexity**: Low (simple indicator calculations)
- **Memory Requirements**: Minimal (tracks last EMA values, last position state)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/multi_indicator/macd_rsi_combo.rs
- **Strategy Type**: Multi-indicator (confluence-based)
- **Dependencies**: alphafield_core (Bar, Signal), alphafield_strategy (indicators module)
- **State Management**: Tracks last position, entry price, last MACD and signal values

### 11.3 Indicator Calculations
- **MACD**: 12-period EMA of price minus 26-period EMA of price
- **MACD Signal**: 9-period EMA of MACD line
- **MACD Histogram**: MACD line minus MACD signal line
- **RSI**: 14-period Relative Strength Index using Wilder's smoothing
- **Confidence Calculation**: Weighted average of MACD strength (normalized) and RSI-based confidence (1.0 at RSI 50, lower at extremes)

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
**Observation**: Strategy implements MACD crossover with RSI filter. Confidence calculation uses both indicators. Simple but effective signal confluence approach.
**Hypothesis Impact**: Code structure supports hypothesis well. RSI filter should reduce whipsaw losses in ranging markets.
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
- Appel, G. (1979). "The Moving Average Convergence-Divergence Trading Method" - Original MACD methodology
- Wilder, J. (1978). "New Concepts in Technical Trading Systems" - RSI development
- Brock, W., Lakonishok, J., & LeBaron, B. (1992). "Simple Technical Trading Rules and the Stochastic Properties of Stock Returns" - Evidence of technical indicator value

### Books
- Murphy, J. (1999). "Technical Analysis of the Financial Markets" - Comprehensive guide to MACD and RSI
- Pring, M. (2002). "Technical Analysis Explained" - Multi-indicator confluence strategies

### Online Resources
- Investopedia: MACD guide and RSI guide
- TradingView: MACD and RSI indicator documentation and community strategies

### Similar Strategies
- MACD Strategy (momentum/macd_strategy.md) - Single-indicator version
- EMA+RSI Strategy - Alternative trend+momentum combination
- Stochastic+RSI Strategy - Different indicator pair for similar concept

### Historical Examples
- Bitcoin 2020-2021 bull run: MACD crossovers captured significant portions of uptrend
- Ethereum 2018-2019 bear market: RSI filter would have avoided many false MACD signals
- General observation: MACD crossovers preceded major crypto rallies in multiple historical instances

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-11 | 1.0 | Initial hypothesis document | AI Agent |