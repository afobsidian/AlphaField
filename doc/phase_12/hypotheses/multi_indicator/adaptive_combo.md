# Adaptive Combination Strategy Hypothesis

## Metadata
- **Name**: Adaptive Combination
- **Category**: MultiIndicator
- **Sub-Type**: adaptive_weighting
- **Author**: AI Agent
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/multi_indicator/adaptive_combo.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
Dynamically adjusting the weight of three independent trading systems (trend-following, momentum, and mean-reversion) based on their recent performance (over a 10-trade lookback) improves risk-adjusted returns compared to static weighting. By allocating more influence to systems that are currently working and reducing weight on underperforming systems, the strategy adapts to changing market conditions, leading to Sharpe ratios of 1.3-2.0, maximum drawdowns of <20%, and annual returns of 25-40%.

**Null Hypothesis**: 
Performance-based dynamic weighting does not improve risk-adjusted returns over static or equal weighting. Observed improvements are due to random noise, data-snooping bias, or insufficient testing period. The short lookback window (10 trades) captures noise rather than true regime changes.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Markets exhibit regime-dependent behavior - different approaches (trend, momentum, mean reversion) perform better in different conditions. No single system dominates across all market phases. By tracking each system's success rate over a rolling window and adjusting weights accordingly, the strategy automatically allocates influence to the most effective approach for current market conditions. Trend-following (EMA crossover) works best in sustained directional moves, momentum (MACD) excels in accelerating trends, and mean reversion (RSI) captures reversals in choppy markets. Adaptive weighting exploits the fact that market regimes persist long enough for performance tracking to be meaningful, yet change frequently enough that static weighting is suboptimal.

### 2.2 Market Inefficiency Exploited
The strategy exploits the tendency of market regimes to persist (momentum in regime persistence) and the slow adjustment of many market participants to regime changes. Retail traders often stick to their favorite approach regardless of performance. Institutional traders may use static multi-system combinations. By dynamically re-weighting based on actual recent performance, the strategy captures inefficiency where market participants are slow to adapt their approach to changing conditions.

### 2.3 Expected Duration of Edge
Adaptive performance-based weighting is a robust concept that should persist as long as (1) different market regimes exist, and (2) regime changes occur at a pace where a 10-trade lookback can capture the shift. As markets become more efficient, individual system edges may degrade, but the adaptive mechanism should preserve value by shifting to whatever approach currently works. Edge expected to be durable and potentially even strengthen as market complexity increases (more regimes to adapt to).

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: Bull markets have sustained uptrends where trend-following and momentum systems excel. As these systems perform well, their weights increase, amplifying their influence. Mean reversion system performs poorly and is downweighted. Strategy effectively focuses on trend systems during bulls, capturing most of the upside.
- **Historical Evidence**: Trend and momentum strategies have historically dominated in crypto bull markets (2017, 2020-2021). Adaptive mechanism would have automatically focused on these approaches.

### 3.2 Bearish Markets
- **Expected Performance**: Medium
- **Rationale**: Bear markets have downtrends with sharp bear rallies. Trend-following fails (frequent whipsaws), momentum may capture bear rallies (short-term), mean reversion may work if rallies get overextended. Performance becomes mixed - no single system dominates, weights may fluctuate. Strategy's long-only nature limits upside in bears.
- **Adaptations**: Consider adding short-selling capability for bear market mean reversion (shorting rallies at RSI overbought), or switch to cash-only mode in confirmed downtrends.

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Medium
- **Rationale**: Ranging markets lack directional trends. Trend-following fails, momentum oscillates, mean reversion works well. As mean reversion system outperforms, its weight increases, shifting strategy focus toward mean reversion entries. This automatic regime switching should capture range-bound profits while avoiding whipsaws from trend/momentum systems.
- **Filters**: Consider adding volatility filter (ATR > threshold) to identify low-volatility ranges vs. high-volatility chop, or requiring minimum regime duration (consecutive wins for a system) before increasing weight.

### 3.4 Volatility Conditions
- **High Volatility**: Higher performance potential but higher risk. Volatility creates regime changes (alternating between trend/momentum and mean reversion dominance). Adaptive mechanism should quickly shift to current dominant system, but frequent weight changes may reduce stability. Consider smoothing weight updates or requiring minimum lookback before major weight shifts.
- **Low Volatility**: Lower performance. All systems struggle with small price moves. Trend and momentum fail to signal, mean reversion signals are weak. Consider scaling out of low-volatility environments or requiring minimum price change threshold for any entry.
- **Volatility Filter**: Not implemented currently. Consider adding ATR filter or using ATR for position sizing.

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 20%
- **Average Drawdown**: 6-10%
- **Drawdown Duration**: 4-9 days (typically short due to 5% SL)
- **Worst Case Scenario**: Regime change occurs during trade - system that generated entry (e.g., trend) becomes underperforming, market reverses, SL hit. Multiple consecutive regime transitions with lagging weight adjustment could produce 15-25% drawdown.

### 4.2 Failure Modes

#### Failure Mode 1: Weight Lag During Regime Change
- **Trigger**: Market regime changes (e.g., trend → sideways), but weight adjustment is slow (10-trade lookback) or individual trade outcomes are noisy
- **Impact**: Strategy continues to use underperforming system for several trades, causing losses during transition period. May turn winning streak into breakeven or small loss.
- **Mitigation**: Reduce lookback window for faster adaptation (e.g., 5 trades), or add additional regime detection indicators (e.g., ATR, ADX) to trigger faster weight resets
- **Detection**: High drawdown during regime transitions, consecutive losses from a particular system while it still has high weight

#### Failure Mode 2: Overfitting to Recent Noise
- **Trigger**: Short-term performance run is due to luck rather than true regime, but weights adjust aggressively based on small sample size
- **Impact**: System gets upweighted just before its performance reverts to mean, leading to poor entries. Creates instability as weights oscillate based on noise.
- **Mitigation**: Increase minimum lookback for weight updates, smooth weight changes (e.g., only change by 20% of difference per update), or require minimum statistical significance (e.g., 3 wins in 5 trades) before major weight increase
- **Detection**: High weight volatility, frequent major weight swings, inconsistent performance across similar market conditions

#### Failure Mode 3: All Systems Underperforming
- **Trigger**: Market condition where none of the three systems work well (e.g., high volatility without clear direction, unusual patterns), all systems have poor recent performance
- **Impact**: Weights may equalize (all low), combined signal becomes weak or random, no clear direction. Strategy may generate poor-quality entries or miss opportunities entirely.
- **Mitigation**: Add fallback to cash mode when all system performance is below threshold, or add additional system type (e.g., volatility breakout) that may work in conditions where current systems fail
- **Detection**: Low signal strength, high rate of TP failures (entries don't reach targets), low win rate across all systems

### 4.3 Correlation Analysis
- **Correlation with Market**: Medium. Strategy is long-only but adapts approach based on market conditions, so correlation varies by regime. High correlation when trend systems dominate (bull markets), lower when mean reversion dominates (ranging).
- **Correlation with Other Strategies**: Medium-high with component strategies (trend, momentum, mean reversion). Correlation varies dynamically as weights shift. Medium with other multi-indicator strategies (may use similar systems). Low with sentiment-based strategies.
- **Diversification Value**: Moderate. Adaptive mechanism provides some regime diversification (not always trend-following or mean reversion), but fundamentally limited to the three system types. Best combined with independent signal sources (sentiment, fundamental, cross-asset) for true diversification.

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1: Composite Score Threshold**
   - **Indicator**: Combined weighted score from all three systems
   - **Parameters**: Combined score > 0.3 (entry threshold)
   - **Confirmation**: Requires moderate-to-high positive confluence from weighted systems
   - **Priority**: Required (primary entry condition)

2. **Condition 2: Not Currently Long**
   - **Indicator**: Position state
   - **Parameters**: last_position != SignalType::Buy
   - **Confirmation**: Only enter new positions (no pyramiding)
   - **Priority**: Required (position management)

3. **Condition 3: Minimum System Performance**
   - **Indicator**: System performance weights (implicit)
   - **Parameters**: Weights within valid range (min_weight to 1.0)
   - **Confirmation**: At least some systems have non-zero weight
   - **Priority**: Required (ensures strategy has data)

### 5.2 Entry Filters
- **Time of Day**: Not applicable (operates on any timeframe)
- **Volume Requirements**: No explicit volume filter
- **Market Regime Filter**: No explicit regime filter (adaptive weighting handles regime adaptation)
- **Volatility Filter**: ATR calculated but not used as filter currently
- **Price Filter**: No minimum price requirement

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: Trend system score positive (already in composite calculation)
- **Confirmation Indicator 2**: Momentum system score positive (already in composite calculation)
- **Confirmation Indicator 3**: Mean reversion system score positive (already in composite calculation)
- **Minimum Confirmed**: Weighted average > 0.3 threshold (already required)

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
- **Reversal Signal**: Combined score drops below -0.5 (strong negative confluence from weighted systems)
- **Trend Reversal**: Fast EMA crosses below Slow EMA (trend system reversal - implicitly in combined score)
- **Performance-Based Exit**: Not explicitly implemented (weight updates affect next entries, not current position)
- **Regime Change**: Not explicitly detected (implicit via score threshold)

## 7. Position Sizing

- **Base Position Size**: Not specified in strategy (handled externally)
- **Volatility Adjustment**: No internal volatility adjustment
- **Conviction Levels**: Combined signal strength (0.0-1.0) can be used for position sizing
- **Max Position Size**: Not specified (external)
- **Risk per Trade**: 5% risk via SL (assuming full position)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| ema_fast | 20 | 10-30 | Fast EMA period for trend system | usize |
| ema_slow | 50 | 30-80 | Slow EMA period for trend system | usize |
| macd_fast | 12 | 8-20 | MACD fast EMA period for momentum system | usize |
| macd_slow | 26 | 20-40 | MACD slow EMA period for momentum system | usize |
| macd_signal | 9 | 7-12 | MACD signal line period for momentum system | usize |
| rsi_period | 14 | 10-20 | RSI period for mean reversion system | usize |
| performance_lookback | 10 | 5-20 | Number of trades to track for performance weighting | usize |
| min_weight | 0.1 | 0.05-0.2 | Minimum weight for each system (prevents complete exclusion) | f64 |
| entry_threshold | 0.3 | 0.1-0.5 | Minimum combined score for entry | f64 |
| take_profit | 5.0 | 3.0-10.0 | Take profit percentage | f64 |
| stop_loss | 5.0 | 3.0-8.0 | Stop loss percentage | f64 |

### 8.2 Optimization Notes
- **Parameters to Optimize**: performance_lookback (primary - critical for adaptation speed), entry_threshold (primary - affects trade quality/frequency), ema_fast, ema_slow (secondary - trend system tuning), macd parameters (secondary - momentum system tuning), rsi_period (secondary - mean reversion tuning), take_profit, stop_loss (risk management)
- **Optimization Method**: Grid search for entry_threshold and performance_lookback, genetic algorithm for indicator periods
- **Optimization Period**: 2 years of data minimum, walk-forward validation to ensure stability across regimes
- **Expected Overfitting Risk**: Medium-High (many parameters, adaptive mechanism adds complexity)
- **Sensitivity Analysis Required**: Yes, especially for performance_lookback (too short = noise, too long = slow adaptation) and entry_threshold (affects trade frequency)

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- **Timeframes**: Daily, 4H (primary), 1H (validation)
- **Test Period**: 2019-2025 (6 years), covering multiple market cycles and regime changes
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
- **Minimum Sharpe Ratio**: 1.3
- **Minimum Sortino Ratio**: 1.6
- **Maximum Max Drawdown**: 20%
- **Minimum Win Rate**: 45% (adaptive mechanism should improve win rate vs static systems)
- **Minimum Profit Factor**: 1.6
- **Minimum Robustness Score**: >70 (from walk-forward analysis, critical for adaptive strategy)
- **Statistical Significance**: p < 0.05 for outperformance vs baseline
- **Walk-Forward Stability**: >65 (consistent performance across windows, with regime transitions)

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 25-40% (depends on market cycle and number of regime changes)
- **Sharpe Ratio**: 1.3-2.0
- **Max Drawdown**: 15-20% (during bear markets and regime transitions)
- **Win Rate**: 45-55% (improved vs static systems through adaptation)
- **Profit Factor**: 1.6-2.2
- **Expectancy**: 0.03-0.05 (3-5% per trade)
- **Average Trade Duration**: 6-12 days

### 10.2 Comparison to Baselines
- **vs. HODL**: Expected to significantly outperform HODL during bear markets (adapts to avoid failing trend systems) and choppy periods (shifts to mean reversion), but may slightly underperform during extended strong bull runs if weight transitions are slow. Overall risk-adjusted outperformance expected.
- **vs. Market Average**: Should generate strong alpha across regimes by automatically adapting to best-performing approach. Expected to outperform static strategies by 20-30% in Sharpe ratio.
- **vs. Static Equal Weighting**: Adaptive mechanism should improve win rate and reduce drawdowns during regime transitions. Expected Sharpe ratio improvement of 15-25%.
- **vs. Best Static System**: Adaptive may slightly underperform the best-performing single system during its ideal regime, but should significantly outperform it during other regimes, leading to better overall consistency.
- **vs. Buy & Hold**: Better risk-adjusted returns due to drawdown control and regime adaptation, with potential to reduce downside exposure during bears.

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: EMA (fast and slow), MACD (fast EMA, slow EMA, signal line), RSI
- **Data Requirements**: OHLCV (open, high, low, close, volume)
- **Latency Sensitivity**: Medium (requires tracking performance over multiple trades, signals can be detected on close)
- **Computational Complexity**: Medium (multiple indicator calculations + performance tracking + weight adjustment)
- **Memory Requirements**: Moderate (tracks last EMA values, system performance metrics, weight history)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/multi_indicator/adaptive_combo.rs
- **Strategy Type**: Multi-indicator (adaptive performance-based weighting)
- **Dependencies**: alphafield_core (Bar, Signal), alphafield_strategy (indicators module)
- **State Management**: Tracks last position, entry price, three SystemPerformance structs (trend, momentum, mean_reversion) each with wins, losses, and current weight

### 11.3 Indicator Calculations
- **Trend System**: EMA(20) vs EMA(50) crossover, score from EMA spread (-1 to +1)
- **Momentum System**: MACD(12,26,9) crossover and direction, score from histogram position (-1 to +1)
- **Mean Reversion System**: RSI(14) position, score from distance from 50 (positive when <50, negative when >50)
- **Weight Update**: Each system tracks wins/losses over lookback, calculates success_rate = wins / (wins + losses), updates weight = max(min_weight, success_rate)
- **Combined Score**: (trend_score × trend_weight + momentum_score × momentum_weight + meanrev_score × meanrev_weight) / (trend_weight + momentum_weight + meanrev_weight)

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (all signals generated correctly)
- [x] Exit conditions (all exits triggered correctly)
- [x] Edge cases (empty data, single bar, etc.)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)
- [x] System performance tracking (correctly tracks wins/losses and updates weights)
- [x] Weight normalization (ensures weights stay in valid range)

### 12.2 Integration Tests
- [x] Backtest execution (runs without errors)
- [x] Performance calculation (metrics are correct)
- [x] Dashboard integration (API works)
- [x] Database integration (saves correctly)

### 12.3 Research Tests
- [ ] Hypothesis validation (adaptive weighting outperforms static)
- [ ] Statistical significance (p < 0.05)
- [ ] Regime analysis (adaptive behavior as expected)
- [ ] Robustness testing (stable across parameters, especially performance_lookback)
- [ ] Weight stability analysis (weights don't oscillate excessively)

## 13. Research Journal

### 2026-01-11: Initial Implementation
**Observation**: Strategy implements three independent trading systems (trend, momentum, mean reversion) with performance tracking and dynamic weight adjustment. SystemPerformance struct tracks wins/losses over configurable lookback and updates weights based on success rate. Combined score uses weighted average.
**Hypothesis Impact**: Code structure strongly supports hypothesis. Adaptive mechanism should automatically shift focus to best-performing systems for current market regime.
**Issues Found**: None during implementation. Weight normalization ensures no system is completely excluded (min_weight). Parameter validation is comprehensive.
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
**Weight Adaptation Analysis**: TBD
**Decision**: TBD

### [Date]: Monte Carlo Simulation
**Number of Simulations**: TBD
**95% Confidence Interval**: TBD
**Best Case**: TBD
**Worst Case**: TBD
**Observation**: TBD

### [Date]: Adaptive Mechanism Analysis
**Weight Adaptation Speed**: TBD (how fast do weights shift after regime change?)
**Regime Detection Accuracy**: TBD (does strategy correctly identify dominant system?)
**Overfitting to Noise**: TBD (do weights oscillate excessively?)
**Action Taken**: TBD

### [Date]: Final Decision
**Final Verdict**: TBD
**Reasoning**: TBD
**Deployment**: TBD
**Monitoring**: TBD

## 14. References

### Academic Sources
- Kaufman, P. (2013). "Trading Systems and Methods" - Adaptive systems and performance-based weighting concepts
- Chan, E. (2013). "Algorithmic Trading: Winning Strategies and Their Rationale" - Multi-system combination approaches
- Brock, W., Lakonishok, J., & LeBaron, B. (1992). "Simple Technical Trading Rules and the Stochastic Properties of Stock Returns" - Evidence of strategy performance variation

### Books
- Kaufman, P. (2013). "Trading Systems and Methods" - Adaptive trading systems
- Chan, E. (2013). "Algorithmic Trading" - Multi-strategy combination
- Pring, M. (2002). "Technical Analysis Explained" - Combining multiple indicators

### Online Resources
- Investopedia: EMA guide, MACD guide, RSI guide
- TradingView: Multi-indicator adaptive strategies
- QuantConnect: Performance-based weighting research
- SSRN: Academic papers on adaptive portfolio construction

### Similar Strategies
- Confidence-Weighted (multi_indicator/confidence_weighted.md) - Static confidence weighting instead of performance-based
- Ensemble Weighted (multi_indicator/ensemble_weighted.md) - More systems with different weighting approach
- Regime-Switching (multi_indicator/regime_switching.md) - Explicit regime detection instead of adaptive weighting

### Historical Examples
- Bitcoin 2020-2021: Trend and momentum systems dominated, adaptive weighting would have automatically focused on them
- Ethereum 2018-2019: Mean reversion would have outperformed, adaptive mechanism would have shifted weight accordingly
- General observation: No single strategy type dominates across all market phases, supporting need for adaptive combination

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-11 | 1.0 | Initial hypothesis document | AI Agent |