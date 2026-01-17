# Regime-Switching Strategy Hypothesis

## Metadata
- **Name**: Regime-Switching
- **Category**: MultiIndicator
- **Sub-Type**: regime_aware
- **Author**: AI Agent
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/multi_indicator/regime_switching.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
Markets exhibit distinct regimes (Bull, Bear, Sideways) where different trading approaches perform optimally. By dynamically detecting current regime and switching between trend-following (Bull), mean reversion (Sideways), and defensive positioning (Bear), the strategy achieves superior risk-adjusted returns compared to any single approach. Expected annual returns of 25-40%, Sharpe ratios of 1.5-2.2, and maximum drawdowns of <15%.

**Null Hypothesis**: 
Regime detection and switching does not provide statistically significant improvement over using a single static approach. Observed regime changes are random noise, and the strategy underperforms due to regime classification errors, transition costs, or overfitting to historical regime patterns.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Markets are not homogeneous - they transition between periods of sustained trends (bull/bear) and oscillating ranges (sideways). Trend-following strategies excel in directional moves but fail in choppy markets. Mean reversion strategies capture bounces in ranges but miss sustained trends. By detecting which regime is currently active and deploying the appropriate approach, the strategy avoids tail-risk events (using trend-following in ranges, mean reversion in strong trends). Regime detection uses EMA position and trend strength (percentage separation) to identify directional vs. oscillating markets, while ATR measures volatility to confirm regime stability.

### 2.2 Market Inefficiency Exploited
The strategy exploits multiple inefficiencies: (1) Momentum persistence in trends (trend-following advantage), (2) Overreaction at extremes in ranges (mean reversion advantage), and (3) Slow participant adjustment to regime changes. Retail traders often use single approach regardless of market conditions. Institutional traders may have complex multi-strategy frameworks but rarely implement explicit regime switching. By actively monitoring and switching regimes, the strategy captures inefficiency where market participants are slow to adapt their approach to current market structure.

### 2.3 Expected Duration of Edge
Regime-based trading is a fundamental concept that should persist as long as (1) markets exhibit distinct regimes, and (2) regimes can be reliably detected using technical indicators. As markets become more efficient, the clarity of regime boundaries may degrade (more transitions, more ambiguity), but the basic bull/bear/sideways classification should remain relevant. Edge expected to degrade slowly but remain viable for several years.

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: Bull markets have sustained uptrends. Regime detection identifies bull regime (trend strength > 2%, EMA fast > EMA slow). Strategy switches to trend-following approach (EMA crossover entries), which excels in directional markets. Captures multi-day uptrends effectively.
- **Historical Evidence**: Trend-following strategies have historically performed well in crypto bull markets (2017, 2020-2021). Regime switching would have correctly identified bull regime and deployed optimal approach.

### 3.2 Bearish Markets
- **Expected Performance**: Neutral (cash-like)
- **Rationale**: Bear markets have downtrends. Regime detection identifies bear regime (trend strength > 2%, EMA fast < EMA slow). Strategy enters defensive mode - no new entries, exits any existing positions immediately. This preserves capital by avoiding failing trend entries and poor mean reversion bounces in downtrends. Performance approaches cash (no trading), avoiding drawdowns.
- **Adaptations**: Consider adding short-selling capability for bear market mean reversion (shorting rallies at RSI overbought), or implementing partial short positions in bear regimes to profit from continued downtrend.

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Medium
- **Rationale**: Ranging markets lack clear direction. Regime detection identifies sideways regime (trend strength < 2%). Strategy switches to mean reversion approach (RSI oversold entries), which excels in oscillating markets by capturing bounces at range extremes. Avoids whipsaws that plague trend-following in ranges.
- **Filters**: Consider adding minimum regime duration (e.g., maintain regime for N bars) to avoid frequent switching, or volatility confirmation (ATR in specific range) for sideways classification.

### 3.4 Volatility Conditions
- **High Volatility**: Mixed performance. High volatility can create regime detection challenges (frequent regime changes). However, strong volatility also creates larger opportunities in whichever regime is active. Consider requiring minimum regime duration or smoothing regime transitions to avoid switch-hopping.
- **Low Volatility**: Lower performance. All approaches struggle with small price moves. Trend-following rarely signals, mean reversion signals are weak and don't reach TP. Price may stagnate. Consider scaling out of low-volatility environments or implementing different exit criteria (e.g., time-based instead of percentage-based).
- **Volatility Filter**: ATR calculated and can be used to confirm regime or adjust TP/SL thresholds. Not currently used as filter but could be added.

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 15%
- **Average Drawdown**: 5-8%
- **Drawdown Duration**: 3-6 days (short due to 5% SL and defensive positioning in bears)
- **Worst Case Scenario**: Rapid regime change detection failure - identifies bull market while actually transitioning to bear, enters trend-following trade, then market accelerates downward, SL hit. Multiple consecutive regime misclassifications during volatile transition could produce 12-18% drawdown.

### 4.2 Failure Modes

#### Failure Mode 1: Regime Misclassification
- **Trigger**: Trend strength temporarily exceeds threshold (e.g., during volatile bear rally) leading to bull regime detection, but downtrend resumes quickly
- **Impact**: Strategy enters trend-following trade in bear market, immediate reversal, SL hit within 1-3 bars. Loss of 5% plus slippage. May occur during regime transitions, especially in volatile markets.
- **Mitigation**: Increase trend strength threshold (e.g., to 3%), require minimum regime duration (e.g., maintain for 5 bars before switching), or add confirmation (e.g., ATR pattern, volume) for regime changes
- **Detection**: High percentage of SL hits immediately after regime switches to bull, losses concentrated in what turn out to be bear markets

#### Failure Mode 2: Frequent Regime Switching
- **Trigger**: Market oscillates near regime boundary (trend strength fluctuates around 2%), causing strategy to switch between bull/sideways regimes repeatedly
- **Impact**: Strategy exits positions or switches approach prematurely, missing moves. Frequent approach changes without trades create inconsistency and reduce confidence in signals. Whipsaw-like behavior even without actual trades.
- **Mitigation**: Implement hysteresis (higher threshold to exit regime than to enter), require minimum regime duration before switch, or smooth regime detection signal (e.g., exponential moving average of trend strength)
- **Detection**: High number of regime transitions (e.g., >10 per 100 bars), inconsistent strategy behavior, periods with no clear approach

#### Failure Mode 3: Late Regime Detection
- **Trigger**: Regime has clearly changed (e.g., bull to bear), but detection lags due to EMA smoothing or trend strength still above threshold
- **Impact**: Strategy continues using wrong approach for multiple bars. In bull-to-bear transition, trend-following generates entries that fail. In sideways-to-bull, strategy misses early trend entry phase. Reduces profitability and increases drawdown during transition.
- **Mitigation**: Use faster regime detection indicators (shorter EMAs, ATR acceleration), add secondary confirmation (e.g., price momentum), or reduce EMA smoothing factor
- **Detection**: Drawdowns occurring immediately after actual regime change (visible in price), strategy still using old approach, delayed switch to correct approach

### 4.3 Correlation Analysis
- **Correlation with Market**: Variable - depends on which regime is active. High in bull (trend-following), Low in bear (defensive/cash), Medium in sideways (mean reversion). Overall correlation lower than single-strategy approaches due to regime switching.
- **Correlation with Other Strategies**: Varies dynamically. High with trend-following strategies during bull regimes, high with mean reversion during sideways regimes. Low-moderate with momentum strategies (shares some characteristics across regimes). Low with sentiment-based strategies.
- **Diversification Value**: High. Regime switching provides true diversification across market conditions - not just diversification across signals but across approaches. Different approaches reduce correlation with any single static strategy, providing better portfolio diversification value. Particularly effective when combined with other multi-regime strategies or independent signal sources.

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1: Bull Regime Detection**
   - **Indicator**: Regime classification
   - **Parameters**: DetectedRegime::Bull (trend_strength > 2%, EMA_fast > EMA_slow)
   - **Confirmation**: Sustained trend with clear directional movement
   - **Priority**: Required (primary filter - determines which approach to use)

2. **Condition 2: Sideways Regime Detection**
   - **Indicator**: Regime classification
   - **Parameters**: DetectedRegime::Sideways (trend_strength < 2%)
   - **Confirmation**: Oscillating price without clear direction
   - **Priority**: Required (primary filter - determines which approach to use)

3. **Condition 3: Approach-Specific Entry**
   - **Bull Regime**: TrendFollowingStrategy entry (EMA fast crosses above EMA slow, golden cross)
   - **Sideways Regime**: MeanReversionStrategy entry (RSI < 30 oversold)
   - **Bear Regime**: No entries (defensive positioning)
   - **Priority**: Required (actual entry trigger based on active regime)

### 5.2 Entry Filters
- **Time of Day**: Not applicable (operates on any timeframe)
- **Volume Requirements**: No explicit volume filter
- **Market Regime Filter**: Explicit regime detection and switching (core feature)
- **Volatility Filter**: ATR calculated but not used as filter currently
- **Price Filter**: No minimum price requirement

### 5.3 Entry Confirmation
- **Confirmation Indicator 1 (Bull)**: EMA crossover confirmed (fast crosses above slow)
- **Confirmation Indicator 2 (Sideways)**: RSI turning up from oversold (not currently required)
- **Minimum Confirmed**: 1 out of 2 for each regime (bull requires crossover, sideways requires RSI oversold)

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
- **Bull Regime Exits**: 
  - Trend reversal: Fast EMA crosses below Slow EMA (death cross)
  - RSI overbought (> 70) with trend-following approach
- **Sideways Regime Exits**:
  - RSI overbought (> 70) during mean reversion trade
  - Mean reversion profit target reached (implicit in TP)
- **Bear Regime Exits**:
  - Immediate exit of any position when regime switches to Bear
  - All positions closed regardless of profit/loss
- **Regime Change Exits**: Immediate exit when regime switches (from bull to bear/sideways, or from sideways to bear)

## 7. Position Sizing

- **Base Position Size**: Not specified in strategy (handled externally)
- **Volatility Adjustment**: No internal volatility adjustment
- **Conviction Levels**: Not directly provided (but regime confidence could be used)
- **Max Position Size**: Not specified (external)
- **Risk per Trade**: 5% risk via SL (assuming full position), except in bear regime (no positions)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| ema_fast | 20 | 10-30 | Fast EMA period for trend detection | usize |
| ema_slow | 50 | 30-80 | Slow EMA period for trend detection | usize |
| atr_period | 14 | 10-20 | ATR period for volatility measurement | usize |
| rsi_period | 14 | 10-20 | RSI calculation period | usize |
| trend_threshold | 0.02 | 0.01-0.05 | Trend strength threshold for regime classification (e.g., 2%) | f64 |
| rsi_oversold | 30.0 | 20.0-35.0 | RSI oversold threshold for sideways regime entries | f64 |
| rsi_overbought | 70.0 | 65.0-80.0 | RSI overbought threshold for exits | f64 |
| take_profit | 5.0 | 3.0-10.0 | Take profit percentage | f64 |
| stop_loss | 5.0 | 3.0-8.0 | Stop loss percentage | f64 |

### 8.2 Optimization Notes
- **Parameters to Optimize**: trend_threshold (primary - critical for regime detection), ema_fast, ema_slow (secondary - trend detection accuracy), rsi_oversold, rsi_overbought (secondary - mean reversion timing), take_profit, stop_loss (risk management)
- **Optimization Method**: Grid search for trend_threshold and TP/SL ratio, genetic algorithm for EMA periods
- **Optimization Period**: 2 years of data minimum, walk-forward validation to ensure robustness across regime transitions
- **Expected Overfitting Risk**: Medium (regime detection is somewhat subjective, but market structure stable)
- **Sensitivity Analysis Required**: Yes, especially for trend_threshold (affects regime classification accuracy) and ema periods (affects detection speed vs. stability)

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- **Timeframes**: Daily, 4H (primary), 1H (validation)
- **Test Period**: 2019-2025 (6 years), covering multiple market cycles and regime transitions
- **Assets**: BTC, ETH, SOL, and 10+ other top crypto assets (minimum 15 assets for cross-asset validation)
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
- **Minimum Sharpe Ratio**: 1.5
- **Minimum Sortino Ratio**: 1.7
- **Maximum Max Drawdown**: 15%
- **Minimum Win Rate**: 45% (regime switching should improve consistency)
- **Minimum Profit Factor**: 1.6
- **Minimum Robustness Score**: >70 (from walk-forward analysis, critical for regime-switching)
- **Statistical Significance**: p < 0.05 for outperformance vs baseline
- **Walk-Forward Stability**: >65 (consistent performance across windows, including regime transitions)

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 25-40% (depends on market cycle and regime detection accuracy)
- **Sharpe Ratio**: 1.5-2.2
- **Max Drawdown**: 12-15% (lower than trend-following due to defensive positioning)
- **Win Rate**: 45-55% (improved vs single approaches through regime adaptation)
- **Profit Factor**: 1.6-2.0
- **Expectancy**: 0.03-0.05 (3-5% per trade)
- **Average Trade Duration**: 5-10 days

### 10.2 Comparison to Baselines
- **vs. HODL**: Expected to significantly outperform HODL during bear markets (defensive positioning preserves capital) and choppy periods (mean reversion avoids whipsaws), but may slightly underperform during extended strong bull runs if trend-following doesn't capture full upside. Overall risk-adjusted outperformance expected with lower drawdowns.
- **vs. Market Average**: Should generate strong alpha across regimes by using optimal approach for each phase. Expected to outperform static strategies by 25-40% in Sharpe ratio due to regime adaptation.
- **vs. Pure Trend Following**: Better performance in bear markets (avoids whipsaws) and ranging markets (switches to mean reversion), similar performance in bull markets (same approach). Expected improvement in Sharpe ratio of 20-30%.
- **vs. Pure Mean Reversion**: Better performance in bull and bear markets (uses trend-following, defensive), worse in ranging markets (same approach). Better overall due to avoiding bad regimes for mean reversion.
- **vs. Buy & Hold**: Better risk-adjusted returns due to drawdown control and regime adaptation, with capital preservation in bears.

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: EMA (fast and slow), ATR, RSI
- **Data Requirements**: OHLCV (open, high, low, close, volume)
- **Latency Sensitivity**: Medium (regime detection requires multiple bars for EMA convergence, but entry signals can be detected on close)
- **Computational Complexity**: Medium (regime detection + two sub-strategies with their own indicators)
- **Memory Requirements**: Moderate (tracks regime state, last EMA values, sub-strategy states)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/multi_indicator/regime_switching.rs
- **Strategy Type**: Multi-indicator (regime-aware with sub-strategies)
- **Dependencies**: alphafield_core (Bar, Signal), alphafield_strategy (indicators module)
- **State Management**: Tracks current regime (Bull/Sideways/Bear), TrendFollowingStrategy state, MeanReversionStrategy state, last position, entry price

### 11.3 Indicator Calculations
- **Fast EMA**: 20-period Exponential Moving Average of close price
- **Slow EMA**: 50-period Exponential Moving Average of close price
- **Trend Strength**: (Fast EMA - Slow EMA) / Slow EMA (percentage difference)
- **ATR**: 14-period Average True Range (for volatility, though not primary detection method)
- **RSI**: 14-period Relative Strength Index using Wilder's smoothing
- **Regime Detection**: 
  - If trend_strength > trend_threshold:
    - If Fast EMA > Slow EMA: Bull regime
    - If Fast EMA < Slow EMA: Bear regime
  - Else: Sideways regime

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (all signals generated correctly for each regime)
- [x] Exit conditions (all exits triggered correctly)
- [x] Edge cases (empty data, single bar, regime transitions)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)
- [x] Regime detection (correctly classifies bull/bear/sideways)
- [x] Sub-strategy isolation (trend and mean reversion work independently)

### 12.2 Integration Tests
- [x] Backtest execution (runs without errors)
- [x] Performance calculation (metrics are correct)
- [x] Dashboard integration (API works)
- [x] Database integration (saves correctly)

### 12.3 Research Tests
- [ ] Hypothesis validation (regime switching outperforms single strategies)
- [ ] Statistical significance (p < 0.05)
- [ ] Regime analysis (performance across regimes as expected)
- [ ] Robustness testing (stable across parameters, especially trend_threshold)
- [ ] Regime transition accuracy analysis (how often are regimes correctly identified?)

## 13. Research Journal

### 2026-01-11: Initial Implementation
**Observation**: Strategy implements comprehensive regime detection using EMA position and trend strength. Three sub-strategies (TrendFollowing, MeanReversion) for different regimes. Defensive positioning in bear regime (no entries). Clean separation of concerns.
**Hypothesis Impact**: Code structure strongly supports hypothesis. Regime switching should automatically select optimal approach for current market conditions.
**Issues Found**: None during implementation. Regime detection is straightforward. Sub-strategy isolation prevents interference. Parameter validation is comprehensive.
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
**Regime Transition Performance**: TBD
**Decision**: TBD

### [Date]: Monte Carlo Simulation
**Number of Simulations**: TBD
**95% Confidence Interval**: TBD
**Best Case**: TBD
**Worst Case**: TBD
**Observation**: TBD

### [Date]: Regime Detection Accuracy Analysis
**Detection Accuracy**: TBD (how often are regimes correctly identified?)
**False Positive Rate**: TBD (how often does bull detection occur in bears?)
**False Negative Rate**: TBD (how often is sideways missed?)
**Transition Lag**: TBD (how many bars before correct regime identification?)
**Action Taken**: TBD

### [Date]: Final Decision
**Final Verdict**: TBD
**Reasoning**: TBD
**Deployment**: TBD
**Monitoring**: TBD

## 14. References

### Academic Sources
- Kaufman, P. (2013). "Trading Systems and Methods" - Regime detection and switching concepts
- Chan, E. (2013). "Algorithmic Trading: Winning Strategies and Their Rationale" - Market regime concepts
- Hamilton, J. (1989). "A New Approach to the Economic Analysis of Nonstationary Time Series and the Business Cycle" - Regime-switching models
- Ang, A., & Bekaert, G. (2002). "International Asset Allocation with Regime Shifts" - Regime-based investment

### Books
- Kaufman, P. (2013). "Trading Systems and Methods" - Adaptive and regime-based trading
- Chan, E. (2013). "Algorithmic Trading" - Multi-regime strategies
- Pring, M. (2002). "Technical Analysis Explained" - Regime analysis in technical trading

### Online Resources
- Investopedia: Market regimes guide, trend analysis, mean reversion concepts
- TradingView: Regime detection indicators and strategies
- QuantConnect: Regime-switching research articles
- SSRN: Academic papers on Markov regime-switching models

### Similar Strategies
- Adaptive Combo (multi_indicator/adaptive_combo.md) - Performance-based weighting vs. explicit regime detection
- Confidence-Weighted (multi_indicator/confidence_weighted.md) - Static confidence vs. regime switching
- Trend+Mean Rev (multi_indicator/trend_mean_rev.md) - Hybrid approach within single framework

### Historical Examples
- Bitcoin 2020-2021: Clear bull regime, trend-following would dominate, regime switching would deploy optimal approach
- Bitcoin 2022: Ranging/choppy markets, mean reversion would work better, regime switching would adapt
- General observation: Crypto markets exhibit distinct bull/bear/sideways phases that last months to years, supporting regime-based approach

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-11 | 1.0 | Initial hypothesis document | AI Agent |