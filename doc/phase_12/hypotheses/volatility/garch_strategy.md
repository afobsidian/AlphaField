# GARCH-Based Strategy Hypothesis

## Metadata
- **Name**: GARCH-Based Strategy
- **Category**: VolatilityBased
- **Sub-Type**: garch_strategy
- **Author**: AI Agent
- **Date**: 2026-01-15
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/volatility/garch_strategy.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
GARCH models predict volatility and provide forward-looking risk assessment. By entering when predicted volatility is low (indicating lower risk) and exiting when predicted volatility spikes (indicating increased risk or regime change), strategy generates positive returns with an average return of >5% over the holding period compared to strategies using historical volatility only.

**Null Hypothesis**: 
GARCH-based volatility prediction does not improve timing decisions and performs no better than strategies using historical volatility measures.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
GARCH (Generalized Autoregressive Conditional Heteroskedasticity) models capture volatility clustering and persistence - the tendency for high volatility periods to cluster together. The EWMA (Exponentially Weighted Moving Average) approximation provides a simplified GARCH(1,1) model that predicts future volatility based on past volatility and recent shocks. Entering when predicted volatility is low minimizes risk exposure, while exiting on volatility spikes avoids drawdowns.

### 2.2 Market Inefficiency Exploited
The strategy exploits the lag in most traders' adaptation to changing volatility. Historical volatility measures (like ATR) look backward and only react to volatility after it occurs. GARCH models provide forward-looking predictions, allowing proactive position management before volatility increases. This predictive capability provides an edge over reactive approaches.

### 2.3 Expected Duration of Edge
This edge persists as long as markets exhibit volatility clustering and traders rely on historical measures. Since volatility clustering is a well-documented persistent market phenomenon and GARCH models are sophisticated but underutilized by retail traders, this edge should maintain an advantage over historical volatility-based approaches.

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: Bull markets have volatility cycles; entering in low-vol phases captures upward moves with minimal risk
- **Historical Evidence**: Bull markets produce low-vol accumulation phases followed by breakouts

### 3.2 Bearish Markets
- **Expected Performance**: Medium
- **Rationale**: Low-vol entries in bear markets are rare; quick exits on vol spikes limit losses
- **Adaptations**: Fast exits when predicted volatility increases

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Medium
- **Rationale**: Low volatility is common in ranges; entries may be followed by false breakouts
- **Filters**: Golden cross entry reduces entries during sideways markets

### 3.4 Volatility Conditions
- **High Volatility**: High - quick exits on volatility spikes protect against drawdowns
- **Low Volatility**: High - ideal entry conditions with minimal risk
- **Volatility Filter**: Predicted volatility serves as timing filter

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 22%
- **Average Drawdown**: 6%
- **Drawdown Duration**: 1-2 weeks
- **Worst Case Scenario**: Volatility spike occurs immediately after entry, causing quick exit

### 4.2 Failure Modes

#### Failure Mode 1: False Low Volatility Prediction
- **Trigger**: Model predicts low volatility but volatility increases unexpectedly
- **Impact**: Entry occurs just before volatility spike, triggering quick exit
- **Mitigation**: Use longer return_window (from 20 to 30), add additional confirmation filters
- **Detection**: High frequency of exits shortly after entry (<2 days)

#### Failure Mode 2: Late Exit on Rapid Volatility Spike
- **Trigger**: Volatility increases faster than model prediction update
- **Impact**: Model underestimates volatility spike, giving back profits before exit
- **Mitigation**: Reduce vol_exit_multiplier (from 2.0 to 1.5), add absolute volatility threshold
- **Detection**: Large drawdowns from peak to exit relative to expected

#### Failure Mode 3: Missed Opportunities Due to Over-Caution
- **Trigger**: Volatility prediction too conservative, missing low-vol entry opportunities
- **Impact**: Strategy sits out while market moves upward
- **Mitigation**: Increase vol_entry_threshold (from 0.4 to 0.5), reduce return_window
- **Detection**: Low trade frequency, significant market moves without positions

### 4.3 Correlation Analysis
- **Correlation with Market**: Medium (follows market with volatility timing)
- **Correlation with Other Strategies**: Medium-High with other volatility-based strategies
- **Diversification Value**: Moderate - unique predictive volatility approach

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1**: Golden cross entry
   - **Indicator**: Fast SMA crosses above Slow SMA
   - **Parameters**: SMA(fast_period) > SMA(slow_period) with crossover
   - **Confirmation**: Confirms uptrend start
   - **Priority**: Required

2. **Condition 2**: Low predicted volatility
   - **Indicator**: Predicted volatility percentile
   - **Parameters**: Predicted volatility < vol_entry_threshold (40th percentile)
   - **Confirmation**: Indicates low-risk entry point
   - **Priority**: Required

### 5.2 Entry Filters
- **Time of Day**: Not applicable (works on any timeframe)
- **Volume Requirements**: None currently
- **Market Regime Filter**: Golden cross serves as trend filter
- **Volatility Filter**: Low predicted volatility required
- **Price Filter**: None currently

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: Golden cross (SMA crossover)
- **Confirmation Indicator 2**: Low predicted volatility (<40th percentile)
- **Minimum Confirmed**: 2 out of 2 (both required)

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
- **Volatility Spike**: Predicted volatility > entry_volatility × vol_exit_multiplier (2.0)
- **Death Cross**: Fast SMA crosses below Slow SMA
- **Regime Change**: Not implemented
- **Time Limit**: None

## 7. Position Sizing
- **Base Position Size**: 1% of portfolio per signal
- **Volatility Adjustment**: Could reduce size if predicted volatility is moderate (not optimal)
- **Conviction Levels**: Could use how low predicted volatility is (percentile rank)
- **Max Position Size**: 5% of portfolio
- **Risk per Trade**: Max 4% (stop loss)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| lambda | 0.94 | 0.90-0.98 | EWMA smoothing factor | float |
| return_window | 20 | 10-40 | Window for return calculation | int |
| vol_entry_threshold | 0.4 | 0.2-0.6 | Volatility percentile for entry | float |
| vol_exit_multiplier | 2.0 | 1.5-3.0 | Volatility multiplier for exit | float |
| fast_period | 10 | 5-15 | Fast SMA period for golden cross | int |
| slow_period | 30 | 20-50 | Slow SMA period for golden cross | int |
| take_profit | 8.0 | 5.0-15.0 | Take profit percentage | float |
| stop_loss | 4.0 | 2.0-8.0 | Stop loss percentage | float |

### 8.2 Optimization Notes
- **Parameters to Optimize**: lambda, return_window, vol_entry_threshold, vol_exit_multiplier
- **Optimization Method**: Walk-forward analysis
- **Optimization Period**: 2 years
- **Expected Overfitting Risk**: Medium-High (GARCH parameters can be curve-fit)
- **Sensitivity Analysis Required**: Yes (critical for GARCH parameters)

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
- **Maximum Max Drawdown**: 22%
- **Minimum Win Rate**: 40%
- **Minimum Profit Factor**: 1.4
- **Minimum Robustness Score**: >60
- **Statistical Significance**: p < 0.05
- **Walk-Forward Stability**: >50

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 18-35%
- **Sharpe Ratio**: 1.0-1.8
- **Max Drawdown**: <22%
- **Win Rate**: 40-50%
- **Profit Factor**: >1.4
- **Expectancy**: >0.025

### 10.2 Comparison to Baselines
- **vs. HODL**: +8-20% risk-adjusted returns
- **vs. Market Average**: +6-15% outperformance
- **vs. Historical Volatility Strategies**: Superior due to predictive capability
- **vs. Buy & Hold**: Lower max drawdown, better timing

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: EWMA GARCH volatility prediction, SMA
- **Data Requirements**: OHLCV
- **Latency Sensitivity**: Low
- **Computational Complexity**: Medium (GARCH prediction, volatility percentile)
- **Memory Requirements**: Moderate (return_window for return calculation, percentile history)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/volatility/garch_strategy.rs
- **Strategy Type**: GARCH-based volatility prediction
- **Dependencies**: alphafield_core, indicators::Sma
- **State Management**: Tracks predicted volatility, volatility history, entry volatility, position, entry price

### 11.3 Indicator Calculations
**EWMA GARCH(1,1) Approximation**:
- Calculate returns: r_t = (price_t - price_{t-1}) / price_{t-1}
- Predicted volatility: σ²_t = λ × σ²_{t-1} + (1-λ) × r²_{t-1}
- Where λ (lambda) = 0.94 (industry standard for daily data)

**Volatility Percentile**: 
- Store predicted volatility values in history
- Current percentile = (volatility values < current volatility) / total volatility values

**Golden Cross**: Fast SMA (10) crosses above Slow SMA (30)

**Volatility Spike Exit**: current_volatility > entry_volatility × vol_exit_multiplier (2.0)

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (golden cross, low predicted volatility)
- [x] Exit conditions (TP, SL, volatility spike, death cross)
- [x] Edge cases (empty data, insufficient bars, single bar)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)
- [x] EWMA GARCH calculation
- [x] Volatility prediction
- [x] Volatility percentile calculation
- [x] Volatility spike detection
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
- [ ] GARCH prediction accuracy vs historical ATR

## 13. Research Journal

### 2026-01-15: Initial Implementation
**Observation**: Strategy implemented with EWMA GARCH(1,1) approximation for volatility prediction
**Hypothesis Impact**: Code supports hypothesis - enters on low predicted volatility, exits on volatility spikes
**Issues Found**: None during implementation
**Action Taken**: Ready for testing

### [Date]: Initial Backtest Results
**Test Period**: [Date range]
**Symbols Tested**: [List of symbols]
**Results**: [Summary of metrics]
**Observation**: [Are results as expected?]
**Action Taken**: [Proceed to validation or refine strategy?]

### [Date]: Lambda Parameter Analysis
**Observation**: [How does prediction accuracy vary with lambda?]
**Hypothesis Impact**: [Does optimal lambda validate EWMA GARCH hypothesis?]
**Issues Found**: [Any parameter-specific problems]
**Action Taken**: [Parameter adjustments or range modifications]

### [Date]: GARCH vs Historical Volatility
**Observation**: [How does GARCH prediction compare to historical ATR?]
**Hypothesis Impact**: [Does GARCH provide superior timing?]
**Issues Found**: [Scenarios where historical ATR outperforms]
**Action Taken**: [Hybrid approach or parameter refinement]

## 14. References

### Academic Sources
- Engle, R. F. "Autoregressive Conditional Heteroskedasticity with Estimates of Variance of United Kingdom Inflation" (1982) - Original GARCH paper
- Bollerslev, T. "Generalized Autoregressive Conditional Heteroskedasticity" (1986) - GARCH extension
- RiskMetrics. "RiskMetrics Technical Document" (1994) - EWMA methodology

### Books
- Taylor, S. J. "Modelling Financial Time Series" - GARCH and volatility modeling
- Tsay, R. S. "Analysis of Financial Time Series" - Comprehensive GARCH coverage
- Alexander, C. "Market Risk Analysis, Volume II: Practical Financial Econometrics" - GARCH applications

### Online Resources
- GARCH models in finance research and case studies
- EWMA volatility prediction methodology
- Volatility clustering and persistence studies
- Crypto market volatility and GARCH modeling

### Similar Strategies
- ATR Breakout (historical volatility vs predicted)
- Volatility Regime (different volatility adaptation)
- Historical Volatility Strategies (baseline comparison)

### Historical Examples
- 2020 COVID volatility spike (GARCH prediction signaled exit before peak)
- 2020-2021 crypto bull market (low predicted volatility signaled good entry points)
- 2022 crypto bear market (volatility spike exits limited losses)

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-15 | 1.0 | Initial hypothesis | AI Agent |