# Volatility Regime Strategy Hypothesis

## Metadata
- **Name**: Volatility Regime Strategy
- **Category**: VolatilityBased
- **Sub-Type**: vol_regime
- **Author**: AI Agent
- **Date**: 2026-01-15
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/volatility/vol_regime.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
Trading strategies should adapt to volatility regimes. By using different entry logic based on volatility percentile ranking—breakout in low volatility, trend-following in medium volatility, and mean reversion in high volatility—the strategy generates positive returns with an average return of >4% over the holding period compared to single-approach strategies.

**Null Hypothesis**: 
Volatility regime adaptation does not improve performance compared to fixed-approach strategies and performs no better than random entry across all regimes.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Market behavior varies significantly across volatility regimes. Low volatility periods are often accumulation phases that precede breakouts. Medium volatility represents normal market conditions where trends can be followed. High volatility indicates overextended markets where mean reversion is more likely than trend continuation. Adapting strategy behavior to current volatility improves alignment with market structure.

### 2.2 Market Inefficiency Exploited
The strategy exploits the failure of single-approach strategies to adapt to changing volatility conditions. Most traders use fixed strategies regardless of volatility regime, causing them to underperform in specific conditions (e.g., trend-following in high volatility, mean reversion in low volatility). Regime-aware adaptation maintains optimal performance across all market environments.

### 2.3 Expected Duration of Edge
This edge persists as long as markets exhibit distinct volatility regimes with different behavioral characteristics. Since volatility clustering is a well-documented market phenomenon and regime shifts occur regularly, the adaptive approach should maintain an advantage over fixed-approach methods.

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: Bull markets exhibit all volatility regimes; regime-aware adaptation captures appropriate behavior
- **Historical Evidence**: Bull markets produce low-vol breakouts, medium-vol trends, and high-vol reversals

### 3.2 Bearish Markets
- **Expected Performance**: Medium-High
- **Rationale**: High volatility regime is common in bear markets; mean reversion entries capture bear rallies
- **Adaptations**: Mean reversion focus in high volatility, quicker exits

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Medium
- **Rationale**: Low volatility regime with breakout entries; can produce false signals
- **Filters**: Regime detection helps avoid entries in inappropriate conditions

### 3.4 Volatility Conditions
- **High Volatility**: High - mean reversion entries capture overextended moves
- **Medium Volatility**: High - trend-following captures sustained moves
- **Low Volatility**: Medium - breakout entries anticipate volatility expansion

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 22%
- **Average Drawdown**: 6%
- **Drawdown Duration**: 1-2 weeks
- **Worst Case Scenario**: Rapid regime shift causing inappropriate strategy application (e.g., breakout in high volatility)

### 4.2 Failure Modes

#### Failure Mode 1: Regime Misclassification
- **Trigger**: Rapid volatility shift causes incorrect classification (e.g., high volatility classified as medium)
- **Impact**: Inappropriate strategy applied (trend-following instead of mean reversion)
- **Mitigation**: Increase percentile_period (from 100 to 150) for smoother classification
- **Detection**: Performance degradation correlated with regime transitions

#### Failure Mode 2: Whipsaw During Regime Transitions
- **Trigger**: Volatility oscillates between regimes causing strategy switches
- **Impact**: Multiple entries/exits as regime classification changes
- **Mitigation**: Add regime persistence requirement (minimum bars in regime)
- **Detection**: High trade frequency clustered around percentile threshold crossings

#### Failure Mode 3: Late Entry in Low Volatility Regime
- **Trigger**: Breakout entry occurs after most of the move has already begun
- **Impact**: Limited upside with same downside risk (asymmetric risk/reward)
- **Mitigation**: Lower low_threshold (from 0.25 to 0.20) for earlier low-vol classification
- **Detection**: Average profit per trade in low-vol regime below expected

### 4.3 Correlation Analysis
- **Correlation with Market**: Medium (varies by regime)
- **Correlation with Other Strategies**: Low-Medium with single-approach strategies
- **Diversification Value**: High - unique adaptive approach differs from static strategies

## 5. Entry Rules

### 5.1 Long Entry Conditions

#### Low Volatility Regime (< 25th percentile)
1. **Condition 1**: Low volatility regime detected
   - **Indicator**: ATR percentile ranking < low_threshold (0.25)
   - **Parameters**: ATR percentile calculated over percentile_period (100)
   - **Confirmation**: Market in low volatility, consolidation phase
   - **Priority**: Required

2. **Condition 2**: Breakout entry
   - **Indicator**: Price breakout above recent high
   - **Parameters**: Price > previous high (from fast_period)
   - **Confirmation**: Indicates start of volatility expansion
   - **Priority**: Required

#### Medium Volatility Regime (25-75th percentile)
1. **Condition 1**: Medium volatility regime detected
   - **Indicator**: ATR percentile between low_threshold (0.25) and high_threshold (0.75)
   - **Parameters**: Normal volatility conditions
   - **Confirmation**: Market in trending regime
   - **Priority**: Required

2. **Condition 2**: Trend-following entry
   - **Indicator**: Fast SMA crosses above Slow SMA
   - **Parameters**: SMA(fast_period) > SMA(slow_period) with crossover
   - **Confirmation**: Confirms uptrend
   - **Priority**: Required

#### High Volatility Regime (> 75th percentile)
1. **Condition 1**: High volatility regime detected
   - **Indicator**: ATR percentile ranking > high_threshold (0.75)
   - **Parameters**: Overextended market conditions
   - **Confirmation**: Market in high volatility, mean reversion regime
   - **Priority**: Required

2. **Condition 2**: Mean reversion entry
   - **Indicator**: RSI becomes oversold
   - **Parameters**: RSI < 30 (oversold threshold)
   - **Confirmation**: Market overextended to downside
   - **Priority**: Required

### 5.2 Entry Filters
- **Time of Day**: Not applicable (works on any timeframe)
- **Volume Requirements**: None currently
- **Market Regime Filter**: Built-in via volatility regime detection
- **Volatility Filter**: Regime classification serves as volatility filter
- **Price Filter**: Varies by regime (breakout, trend, mean reversion)

### 5.3 Entry Confirmation
- **Low Volatility**: Breakout above previous high
- **Medium Volatility**: SMA crossover confirmation
- **High Volatility**: RSI oversold condition
- **Minimum Confirmed**: 1 out of 1 (regime-specific condition)

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP 1**: 8% profit - Close 100%

### 6.2 Stop Loss
- **Initial SL**: 4% below entry
- **Trailing SL**: None
- **Breakeven**: None
- **Time-based Exit**: None

### 6.3 Exit Conditions

#### Low Volatility Regime Exits
- **Take Profit Reached**: 8% profit target achieved
- **Stop Loss Triggered**: 4% loss threshold exceeded
- **Regime Change**: Volatility increases to medium or high

#### Medium Volatility Regime Exits
- **Take Profit Reached**: 8% profit target achieved
- **Stop Loss Triggered**: 4% loss threshold exceeded
- **Trend Reversal**: Fast SMA crosses below Slow SMA
- **Regime Change**: Volatility moves to low or high

#### High Volatility Regime Exits
- **Take Profit Reached**: 8% profit target achieved
- **Stop Loss Triggered**: 4% loss threshold exceeded
- **Mean Reversion Complete**: RSI > 50 (neutral/middle)
- **Regime Change**: Volatility decreases to low or medium

## 7. Position Sizing
- **Base Position Size**: 1% of portfolio per signal
- **Volatility Adjustment**: Could reduce size in high volatility regime
- **Conviction Levels**: Could use signal strength relative to threshold
- **Max Position Size**: 5% of portfolio
- **Risk per Trade**: Max 4% (stop loss)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| atr_period | 14 | 7-21 | ATR calculation period | int |
| percentile_period | 100 | 50-200 | Lookback period for percentile calculation | int |
| low_threshold | 0.25 | 0.15-0.35 | Low volatility threshold (percentile) | float |
| high_threshold | 0.75 | 0.65-0.85 | High volatility threshold (percentile) | float |
| fast_period | 10 | 5-15 | Fast SMA period for trend-following | int |
| slow_period | 30 | 20-50 | Slow SMA period for trend-following | int |
| take_profit | 8.0 | 5.0-15.0 | Take profit percentage | float |
| stop_loss | 4.0 | 2.0-8.0 | Stop loss percentage | float |

### 8.2 Optimization Notes
- **Parameters to Optimize**: low_threshold, high_threshold, fast_period, slow_period
- **Optimization Method**: Walk-forward analysis with regime-aware validation
- **Optimization Period**: 2 years
- **Expected Overfitting Risk**: Medium-High (multiple regime-specific parameters)
- **Sensitivity Analysis Required**: Yes (critical for regime thresholds)

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
- [ ] Regime analysis (low/medium/high volatility performance separately)
- [ ] Cross-asset validation (multiple symbols)
- [ ] Bootstrap validation (resampling)
- [ ] Permutation testing (randomness check)

### 9.3 Success Criteria
- **Minimum Sharpe Ratio**: 1.0
- **Minimum Sortino Ratio**: 1.5
- **Maximum Max Drawdown**: 22%
- **Minimum Win Rate**: 40%
- **Minimum Profit Factor**: 1.4
- **Minimum Robustness Score**: >65
- **Statistical Significance**: p < 0.05
- **Walk-Forward Stability**: >50

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 20-40%
- **Sharpe Ratio**: 1.0-1.8
- **Max Drawdown**: <22%
- **Win Rate**: 40-50%
- **Profit Factor**: >1.4
- **Expectancy**: >0.025

### 10.2 Comparison to Baselines
- **vs. HODL**: +10-25% risk-adjusted returns
- **vs. Market Average**: +8-20% outperformance
- **vs. Single-Approach Strategies**: Superior performance across regimes
- **vs. Buy & Hold**: Lower max drawdown, more consistent returns

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: ATR, SMA, RSI
- **Data Requirements**: OHLCV
- **Latency Sensitivity**: Low
- **Computational Complexity**: High (percentile ranking, regime detection, regime-specific logic)
- **Memory Requirements**: High (percentile_period history for ATR values)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/volatility/vol_regime.rs
- **Strategy Type**: Adaptive regime-aware
- **Dependencies**: alphafield_core, indicators::Atr, indicators::Sma, indicators::Rsi
- **State Management**: Tracks ATR history, percentile ranking, regime classification, position, entry price

### 11.3 Indicator Calculations
**ATR Percentile Ranking**:
- Calculate ATR over atr_period (14) bars
- Store ATR values in history (percentile_period = 100 bars)
- Current percentile = (ATR values < current ATR) / total ATR values

**Regime Classification**:
- **Low Volatility**: Percentile < low_threshold (0.25)
- **Medium Volatility**: low_threshold (0.25) ≤ percentile ≤ high_threshold (0.75)
- **High Volatility**: Percentile > high_threshold (0.75)

**Regime-Specific Entry Logic**:
- **Low Volatility**: Breakout entry (price > previous high from fast_period)
- **Medium Volatility**: Trend-following (SMA crossover: fast crosses above slow)
- **High Volatility**: Mean reversion (RSI < 30, oversold)

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (low/medium/high volatility regime entries)
- [x] Exit conditions (TP, SL, regime-specific exits)
- [x] Edge cases (empty data, insufficient bars, single bar)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)
- [x] ATR percentile calculation
- [x] Regime detection (low/medium/high)
- [x] Regime-specific entry/exit logic
- [x] Display formatting

### 12.2 Integration Tests
- [ ] Backtest execution (runs without errors)
- [ ] Performance calculation (metrics are correct)
- [ ] Dashboard integration (API works)
- [ ] Database integration (saves correctly)

### 12.3 Research Tests
- [ ] Hypothesis validation (regime adaptation outperforms single approach)
- [ ] Statistical significance (p < 0.05)
- [ ] Regime analysis (performance as expected in each regime)
- [ ] Robustness testing (stable across parameters)

## 13. Research Journal

### 2026-01-15: Initial Implementation
**Observation**: Strategy implemented with ATR percentile-based regime detection and regime-specific entry/exit logic
**Hypothesis Impact**: Code supports hypothesis - adapts strategy based on volatility regime (breakout/trend/mean reversion)
**Issues Found**: None during implementation
**Action Taken**: Ready for testing

### [Date]: Initial Backtest Results
**Test Period**: [Date range]
**Symbols Tested**: [List of symbols]
**Results**: [Summary of metrics by regime]
**Observation**: [Does regime adaptation improve performance?]
**Action Taken**: [Proceed to validation or refine regime thresholds?]

### [Date]: Regime Performance Analysis
**Observation**: [How does strategy perform in each volatility regime?]
**Hypothesis Impact**: [Does each regime logic perform as expected?]
**Issues Found**: [Any regime-specific problems]
**Action Taken**: [Threshold adjustments or logic modifications]

### [Date]: Regime Transition Analysis
**Observation**: [How does strategy perform during regime transitions?]
**Hypothesis Impact**: [Are regime misclassifications causing problems?]
**Issues Found**: [Whipsaw or late entry/exit during transitions]
**Action Taken**: [Add regime persistence requirement or adjust thresholds]

## 14. References

### Academic Sources
- Engle, R. F. "Autoregressive Conditional Heteroskedasticity with Estimates of the Variance of United Kingdom Inflation" (1982) - GARCH and volatility clustering
- Bollerslev, T. "Generalized Autoregressive Conditional Heteroskedasticity" (1986) - Volatility modeling

### Books
- Kaufman, P. J. "Trading Systems and Methods" - Volatility regime analysis
- Chan, E. P. "Algorithmic Trading: Winning Strategies and Their Rationale" - Adaptive strategy concepts

### Online Resources
- Volatility regime trading research and case studies
- ATR percentile-based regime detection methods
- Market structure and volatility clustering studies

### Similar Strategies
- Volatility Squeeze (different volatility detection approach)
- ATR Breakout (single-approach comparison)
- Regime-Based Sentiment (similar adaptive concept)

### Historical Examples
- 2020 COVID crash (high volatility, mean reversion effective)
- 2020-2021 crypto bull market (medium volatility, trend-following effective)
- 2021 crypto consolidation (low volatility, breakout effective)

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-15 | 1.0 | Initial hypothesis | AI Agent |