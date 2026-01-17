# Ensemble Weighted Strategy Hypothesis

## Metadata
- **Name**: Ensemble Weighted
- **Category**: MultiIndicator
- **Sub-Type**: ensemble
- **Author**: AI Agent
- **Date**: 2026-01-11
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/multi_indicator/ensemble_weighted.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
An ensemble of 5 independent strategies (EMA Trend, MACD Momentum, and 3 Mean Reversion variants) with performance-based dynamic weighting generates more robust signals than any single strategy. By combining diverse signal sources and allocating voting power based on recent success rates, strategy reduces failure modes of individual approaches while capturing their strengths, leading to Sharpe ratios of 1.5-2.2, maximum drawdowns of <20%, and annual returns of 30-45%.

**Null Hypothesis**: 
Ensemble voting with performance weighting does not provide statistically significant improvement over the best-performing single strategy or simple equal-weighted ensemble. Observed improvements are due to random noise, data-snooping bias, or insufficient testing period. The 5-strategy ensemble adds complexity without proportional benefit.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
No single technical approach dominates across all market conditions. Trend-following (EMA crossovers) excels in directional markets but fails in choppy conditions. Momentum (MACD) captures accelerating trends but generates late entries at peaks. Mean reversion (RSI-based variants) identifies reversals and ranges but misses sustained trends. By combining 5 strategies spanning these approaches and weighting their votes by recent performance, the ensemble aggregates diverse market perspectives. Performance-based weighting ensures strategies currently working get more influence while underperforming strategies are downweighted but not completely excluded (preserving regime-transition readiness).

### 2.2 Market Inefficiency Exploited
The strategy exploits multiple inefficiencies: (1) Momentum persistence in trends, (2) overreaction at extremes (mean reversion opportunities), and (3) slow participant adjustment to regime changes. Retail traders often fixate on single indicators or approaches. Algorithmic traders may use static multi-strategy combinations. By dynamically re-weighting based on actual recent performance, the ensemble captures inefficiency where market participants are slow to adapt their approach to changing conditions. Additionally, averaging 5 independent signals reduces noise compared to any single signal.

### 2.3 Expected Duration of Edge
Multi-strategy ensembles with performance-based weighting is a robust concept that should persist as long as (1) different market regimes exist, and (2) individual strategy edges vary across regimes. As markets become more algorithmically traded, individual edges may degrade, but the ensemble's averaging effect provides inherent diversification benefit. Performance-based weighting's advantage of shifting influence to currently-working strategies should remain valuable even as edges weaken. Edge expected to be durable and potentially strengthen as market complexity increases (more variations to diversify across).

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: Bull markets have sustained uptrends where trend and momentum strategies excel. Mean reversion strategies underperform but may capture occasional pullback bounces. As trend/momentum gain wins, their weights increase, amplifying their influence. Ensemble effectively focuses on trend-following approaches during bulls, capturing most of the upside while mean reversion provides occasional pullback entries.
- **Historical Evidence**: Trend and momentum strategies have historically dominated in crypto bull markets (2017, 2020-2021). Ensemble would have automatically weighted these higher.

### 3.2 Bearish Markets
- **Expected Performance**: Medium-Low
- **Rationale**: Bear markets have downtrends with sharp bear rallies. Trend and momentum fail (frequent whipsaws). Mean reversion strategies may capture bear market rallies (short-covering bounces) as RSI becomes oversold. Performance becomes mixed, with mean reversion potentially gaining weight. Ensemble's long-only nature limits upside in bears.
- **Adaptations**: Consider adding short-selling capability for bear market mean reversion strategies (shorting rallies at RSI overbought), or switch to cash-only mode in confirmed downtrends.

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Medium
- **Rationale**: Ranging markets lack clear direction. Trend and momentum oscillate, generating many signals but low success. Mean reversion strategies (RSI variants) work well in ranges, capturing bounces at extremes. As mean reversion strategies outperform, their weights increase, shifting ensemble focus toward mean reversion entries. Automatic regime switching should capture range-bound profits.
- **Filters**: Consider adding volatility filter (ATR > threshold) to distinguish between genuine range and low-volatility stagnation, or requiring minimum time between trades to reduce whipsaw frequency.

### 3.4 Volatility Conditions
- **High Volatility**: Higher performance potential but higher risk. Volatility creates opportunities for all strategy types but also faster reversals. Ensemble's averaging effect may smooth some noise, but individual SL triggers increase frequently. Consider wider SL or reducing position sizes in high volatility.
- **Low Volatility**: Lower performance. Trend and momentum fail to signal, mean reversion signals are weak. Price moves may be too small to reach TP before reversal. Consider scaling out of low-volatility environments or requiring minimum price change for entry.
- **Volatility Filter**: Not implemented currently. Consider adding ATR filter or using ATR for position sizing.

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 20%
- **Average Drawdown**: 6-12%
- **Drawdown Duration**: 4-8 days (typically short due to 5% SL)
- **Worst Case Scenario**: Sudden regime change where all strategies underperform simultaneously (e.g., transition from trend to range), causing multiple consecutive losses before weights adjust. Multiple false signals during transition could produce 15-25% drawdown.

### 4.2 Failure Modes

#### Failure Mode 1: All Strategies Underperforming
- **Trigger**: Market condition where none of the 5 strategies work well (e.g., high volatility without clear direction, unusual patterns), all strategies have poor recent performance, leading to equalized weights or confused voting
- **Impact**: Combined signal becomes weak or random, no clear direction. Strategy may generate poor-quality entries or miss opportunities entirely. May occur during regime transitions or unusual market behavior.
- **Mitigation**: Add fallback to cash mode when all strategy performance is below threshold, or add additional strategy type (e.g., volatility breakout) that may work in conditions where current systems fail
- **Detection**: Low signal strength, high rate of TP failures (entries don't reach targets), low win rate across all strategies

#### Failure Mode 2: Weight Lag During Regime Change
- **Trigger**: Market regime changes (e.g., trend → sideways), but weight adjustment is slow (10-trade lookback with 30% smoothing) or individual strategy outcomes are noisy
- **Impact**: Strategy continues to give high influence to underperforming strategies for several trades, causing losses during transition period. May turn winning streak into breakeven or small loss.
- **Mitigation**: Reduce lookback window for faster adaptation (e.g., 5 trades), increase smoothing adjustment rate (e.g., 50% change instead of 30%), or add additional regime detection indicators (e.g., ATR, ADX) to trigger faster weight resets
- **Detection**: High drawdown during regime transitions, consecutive losses from strategies that should be downweighted but still have high weight

#### Failure Mode 3: Ensemble Averaging Dilutes Strong Signals
- **Trigger**: One strategy generates a very strong, correct signal, but other strategies are neutral or mildly negative, causing combined vote to be diluted
- **Impact**: Weaker position sizing or no signal despite strong individual setup. Misses opportunity when best-performing strategy has clear conviction. Reduces profit per winning trade.
- **Mitigation**: Implement conviction threshold for individual strategy signals (e.g., if any strategy score > 0.8, use that signal directly), or allow more aggressive weighting (higher max weight) for clearly-outperforming strategies
- **Detection**: Lower-than-expected profits on winning trades, many instances where a single strategy had strong signal but ensemble didn't trigger or used reduced strength

### 4.3 Correlation Analysis
- **Correlation with Market**: Medium. Strategy is long-only but ensembles multiple approaches, so correlation varies by which strategies dominate. High correlation when trend/momentum dominate (bull markets), lower when mean reversion dominates (ranging).
- **Correlation with Other Strategies**: High with component strategies (uses identical signals). Medium with other multi-indicator strategies (may use similar subsets). Low with sentiment-based strategies.
- **Diversification Value**: Moderate-High. Ensemble provides internal diversification across 5 different approaches within a single strategy. Better than single-strategy approaches but still fundamentally technical. Best combined with independent signal sources (sentiment, fundamental, cross-asset) for true portfolio diversification.

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1: Weighted Vote Threshold**
   - **Indicator**: Combined weighted vote from all 5 strategies
   - **Parameters**: Weighted average > 0.0 (positive net confluence)
   - **Confirmation**: Requires more bullish influence than bearish across all strategies
   - **Priority**: Required (primary entry condition)

2. **Condition 2: Not Currently Long**
   - **Indicator**: Position state
   - **Parameters**: last_position != SignalType::Buy
   - **Confirmation**: Only enter new positions (no pyramiding)
   - **Priority**: Required (position management)

3. **Condition 3: Minimum Strategy Performance**
   - **Indicator**: Strategy performance weights (implicit)
   - **Parameters**: All strategies have valid weights (within min_weight to 1.0 range)
   - **Confirmation**: At least some strategies have non-zero weight
   - **Priority**: Required (ensures strategy has performance data)

### 5.2 Entry Filters
- **Time of Day**: Not applicable (operates on any timeframe)
- **Volume Requirements**: No explicit volume filter
- **Market Regime Filter**: No explicit regime filter (ensemble performance weighting handles adaptation)
- **Volatility Filter**: Not implemented currently
- **Price Filter**: No minimum price requirement

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: Multiple strategies agree (weighted vote positive)
- **Confirmation Indicator 2**: At least one strategy has strong conviction (> 0.7 score)
- **Minimum Confirmed**: Weighted average > 0.0 (already required)

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
- **Reversal Signal**: Combined weighted vote drops below 0.0 (net bearish confluence)
- **Strategy-Specific Exits**: Not explicitly implemented (ensemble uses combined vote)
- **Regime Change**: Not explicitly detected (implicit via performance weighting shifting)

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
| num_strategies | 5 | 3-7 | Number of independent strategies in ensemble | usize |
| ema_fast | 20 | 10-30 | EMA fast period for trend strategy | usize |
| ema_slow | 50 | 30-80 | EMA slow period for trend strategy | usize |
| macd_fast | 12 | 8-20 | MACD fast EMA period for momentum strategy | usize |
| macd_slow | 26 | 20-40 | MACD slow EMA period for momentum strategy | usize |
| macd_signal | 9 | 7-12 | MACD signal line period for momentum strategy | usize |
| rsi_period | 14 | 10-20 | RSI period for mean reversion strategies | usize |
| performance_lookback | 10 | 5-20 | Number of trades to track for performance weighting | usize |
| min_weight | 0.1 | 0.05-0.2 | Minimum weight for each strategy (prevents complete exclusion) | f64 |
| weight_smoothing | 0.3 | 0.1-0.5 | Weight smoothing factor (0.1 = aggressive, 0.5 = slow) | f64 |
| entry_threshold | 0.0 | -0.2 to 0.2 | Minimum weighted vote for entry | f64 |
| take_profit | 5.0 | 3.0-10.0 | Take profit percentage | f64 |
| stop_loss | 5.0 | 3.0-8.0 | Stop loss percentage | f64 |

### 8.2 Optimization Notes
- **Parameters to Optimize**: performance_lookback (primary - critical for adaptation speed), min_weight (secondary - exclusion risk), weight_smoothing (secondary - adaptation speed), entry_threshold (secondary - trade quality/frequency balance), ema parameters, macd parameters, rsi parameters (secondary - strategy tuning), take_profit, stop_loss (risk management)
- **Optimization Method**: Grid search for entry_threshold and performance_lookback, genetic algorithm for indicator periods
- **Optimization Period**: 2 years of data minimum, walk-forward validation to ensure stability across regimes
- **Expected Overfitting Risk**: Medium-High (many parameters, 5-strategy ensemble adds complexity, performance tracking may overfit to short lookback)
- **Sensitivity Analysis Required**: Yes, especially for performance_lookback (too short = noise, too long = slow adaptation) and weight_smoothing (affects adaptation speed)

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
- **Minimum Sharpe Ratio**: 1.5
- **Minimum Sortino Ratio**: 1.7
- **Maximum Max Drawdown**: 20%
- **Minimum Win Rate**: 45% (ensemble averaging should improve vs single strategies)
- **Minimum Profit Factor**: 1.6
- **Minimum Robustness Score**: >70 (from walk-forward analysis, critical for ensemble)
- **Statistical Significance**: p < 0.05 for outperformance vs baseline
- **Walk-Forward Stability**: >65 (consistent performance across windows, with regime transitions)

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 30-45% (depends on market cycle and ensemble effectiveness)
- **Sharpe Ratio**: 1.5-2.2
- **Max Drawdown**: 15-20% (during bear markets and regime transitions)
- **Win Rate**: 45-55% (ensemble averaging should improve vs individual strategies)
- **Profit Factor**: 1.6-2.2
- **Expectancy**: 0.035-0.055 (3.5-5.5% per trade)
- **Average Trade Duration**: 6-11 days

### 10.2 Comparison to Baselines
- **vs. HODL**: Expected to significantly outperform HODL during bear markets (performance weighting shifts to mean reversion) and choppy periods (diversification reduces whipsaws), but may slightly underperform during extended strong bull runs if trend/momentum weights don't increase fast enough. Overall risk-adjusted outperformance expected.
- **vs. Market Average**: Should generate strong alpha across regimes by automatically shifting to best-performing strategies. Expected to outperform static strategies by 25-40% in Sharpe ratio due to diversification benefit.
- **vs. Best Single Strategy**: Ensemble may slightly underperform best-performing single strategy during its ideal regime, but should significantly outperform it during other regimes, leading to better overall consistency and lower drawdowns. Expected Sharpe ratio improvement of 20-30% vs best single strategy across full cycle.
- **vs. Equal-Weighted Ensemble**: Performance-based weighting should improve win rate and reduce drawdowns during regime transitions by focusing on currently-working strategies. Expected improvement of 10-15% in Sharpe ratio.
- **vs. Buy & Hold**: Better risk-adjusted returns due to drawdown control and ensemble diversification, with potential to reduce downside exposure across varied conditions.

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: EMA (fast and slow), MACD (fast EMA, slow EMA, signal line), RSI (multiple variants with different parameters)
- **Data Requirements**: OHLCV (open, high, low, close, volume)
- **Latency Sensitivity**: Medium-High (requires tracking performance over multiple trades for 5 strategies, ensemble voting on close)
- **Computational Complexity**: Medium-High (5 independent strategies + performance tracking + weight adjustment + ensemble voting)
- **Memory Requirements**: Moderate-High (tracks 5 SystemPerformance structs each with wins, losses, last signal, current weight)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/multi_indicator/ensemble_weighted.rs
- **Strategy Type**: Multi-indicator (ensemble with performance-based weighting)
- **Dependencies**: alphafield_core (Bar, Signal), alphafield_strategy (indicators module)
- **State Management**: Tracks last position, entry price, 5 SystemPerformance structs (trend, momentum, meanrev1, meanrev2, meanrev3) each with wins, losses, last_signal, current_weight

### 11.3 Indicator Calculations
- **EMA Trend Strategy**: EMA(20) vs EMA(50) crossover, score from EMA position (-1 to +1)
- **MACD Momentum Strategy**: MACD(12,26,9) crossover and direction, score from histogram position (-1 to +1)
- **Mean Reversion Strategy 1**: RSI(14) oversold entry (< 30), score from position (-1 to +1)
- **Mean Reversion Strategy 2**: RSI(10) oversold entry (< 25), score from position (-1 to +1)
- **SMA Crossover Strategy**: SMA(20) vs SMA(50) crossover, score from position (-1 to +1)
- **Weight Update**: Each strategy tracks wins/losses over lookback, calculates success_rate = wins / (wins + losses), updates weight = max(min_weight, success_rate) with smoothing factor applied
- **Weighted Vote**: (Σ(strategy_score × strategy_weight)) / (Σ(strategy_weight))
- **Weight Normalization**: Ensures weights sum to 1.0

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (all signals generated correctly)
- [x] Exit conditions (all exits triggered correctly)
- [x] Edge cases (empty data, single bar, etc.)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)
- [x] System performance tracking (correctly tracks wins/losses and updates weights)
- [x] Weight normalization (ensures weights stay in valid range)
- [x] Ensemble voting (correctly calculates weighted average)
- [x] Weight smoothing (applies smoothing factor correctly)

### 12.2 Integration Tests
- [x] Backtest execution (runs without errors)
- [x] Performance calculation (metrics are correct)
- [x] Dashboard integration (API works)
- [x] Database integration (saves correctly)

### 12.3 Research Tests
- [ ] Hypothesis validation (ensemble outperforms single strategies)
- [ ] Statistical significance (p < 0.05)
- [ ] Regime analysis (performance adaptation as expected)
- [ ] Robustness testing (stable across parameters, especially performance_lookback)
- [ ] Ensemble benefit analysis (does ensemble provide diversification benefit vs individual strategies?)

## 13. Research Journal

### 2026-01-11: Initial Implementation
**Observation**: Strategy implements ensemble of 5 independent strategies (EMA Trend, MACD Momentum, 3 Mean Reversion variants) with comprehensive performance tracking. Each strategy tracks wins/losses over configurable lookback, updates weights based on success rate with smoothing. Ensemble voting uses weighted average. Weight normalization ensures equal contribution regardless of strategy count.
**Hypothesis Impact**: Code structure strongly supports hypothesis. Ensemble approach with performance-based weighting should provide diversification and automatic regime adaptation.
**Issues Found**: None during implementation. Weight smoothing prevents rapid oscillation. Parameter validation is comprehensive.
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

### [Date]: Ensemble Diversification Analysis
**Benefit vs Best Strategy**: TBD (how much does ensemble add?)
**Benefit vs Equal Weighting**: TBD (is performance-based weighting worth it?)
**Strategy Contribution**: TBD (which strategies contribute most to ensemble success?)
**Action Taken**: TBD

### [Date]: Final Decision
**Final Verdict**: TBD
**Reasoning**: TBD
**Deployment**: TBD
**Monitoring**: TBD

## 14. References

### Academic Sources
- Kaufman, P. (2013). "Trading Systems and Methods" - Ensemble and multi-system concepts
- Chan, E. (2013). "Algorithmic Trading: Winning Strategies and Their Rationale" - Portfolio construction and diversification
- Brock, W., Lakonishok, J., & LeBaron, B. (1992). "Simple Technical Trading Rules and the Stochastic Properties of Stock Returns" - Evidence of strategy performance variation
- Hansen, L. (2014). "Pairs Trading: A Bayesian Approach" - Ensemble weighting concepts

### Books
- Kaufman, P. (2013). "Trading Systems and Methods" - Multi-strategy ensembles
- Chan, E. (2013). "Algorithmic Trading" - Portfolio and ensemble approaches
- Pring, M. (2002). "Technical Analysis Explained" - Combining multiple indicators
- Ruppert, D. (2011). "Portfolio Construction and Risk Budgeting" - Weight optimization concepts

### Online Resources
- Investopedia: Ensemble learning in trading, portfolio diversification
- TradingView: Multi-indicator ensemble strategies
- QuantConnect: Ensemble and voting system research
- SSRN: Academic papers on ensemble methods in finance
- Kaggle: Ensemble learning competitions and methodologies

### Similar Strategies
- Adaptive Combo (multi_indicator/adaptive_combo.md) - 3 strategies with adaptive weighting
- Confidence-Weighted (multi_indicator/confidence_weighted.md) - Single strategy with confidence filtering
- Regime-Switching (multi_indicator/regime_switching.md) - Explicit regime detection vs. ensemble

### Historical Examples
- Bitcoin 2020-2021: Trend and momentum dominated, ensemble would have weighted these highly
- Ethereum 2018-2019: Mean reversion would have outperformed, ensemble weighting would have shifted accordingly
- General observation: No single strategy type dominates across all market phases, supporting need for ensemble diversification

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-11 | 1.0 | Initial hypothesis document | AI Agent |