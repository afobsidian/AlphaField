# [Strategy Name] Hypothesis

## Metadata
- **Name**: [Strategy Name]
- **Category**: [TrendFollowing / MeanReversion / Momentum / VolatilityBased / SentimentBased / MultiIndicator]
- **Sub-Type**: [e.g., Golden Cross, Bollinger Bands, etc.]
- **Author**: AI Agent
- **Date**: [YYYY-MM-DD]
- **Status**: [Proposed / Testing / Validated / Deployed / Rejected]
- **Code Location**: [crates/strategy/src/strategies/[category]/[strategy].rs]

## 1. Hypothesis Statement

**Primary Hypothesis**: 
[One clear, testable statement about market behavior. Must be falsifiable.]

Example: "When the 50-day SMA crosses above the 200-day SMA (Golden Cross), the asset enters a sustainable uptrend and will generate positive returns over the next 30 trading days with an average return of >3%."

**Null Hypothesis**: 
[The opposite of the primary hypothesis - must be testable]

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
[Explain WHY this should work based on market mechanics, behavioral finance, or technical principles]

### 2.2 Market Inefficiency Exploited
[What market inefficiency does this strategy exploit? Why hasn't it been arbitraged away?]

### 2.3 Expected Duration of Edge
[How long will this edge persist? Under what conditions will it degrade?]

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: [High / Medium / Low]
- **Rationale**: [Why it works in bull markets]
- **Historical Evidence**: [Any supporting examples]

### 3.2 Bearish Markets
- **Expected Performance**: [High / Medium / Low]
- **Rationale**: [Why it works/doesn't work in bear markets]
- **Adaptations**: [Any modifications needed for bears]

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: [High / Medium / Low]
- **Rationale**: [Why it works/doesn't work in sideways markets]
- **Filters**: [Any filters to avoid sideways markets]

### 3.4 Volatility Conditions
- **High Volatility**: [Performance and adaptations]
- **Low Volatility**: [Performance and adaptations]
- **Volatility Filter**: [ATR threshold, etc.]

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: [e.g., 15%]
- **Average Drawdown**: [e.g., 5%]
- **Drawdown Duration**: [e.g., 1-3 weeks]
- **Worst Case Scenario**: [Describe worst possible outcome]

### 4.2 Failure Modes

#### Failure Mode 1: [Name]
- **Trigger**: [What causes this failure]
- **Impact**: [How bad is it - drawdown, frequency, etc.]
- **Mitigation**: [How to prevent or minimize]
- **Detection**: [How to detect this failure in real-time]

#### Failure Mode 2: [Name]
- **Trigger**: ...
- **Impact**: ...
- **Mitigation**: ...
- **Detection**: ...

### 4.3 Correlation Analysis
- **Correlation with Market**: [Low / Medium / High]
- **Correlation with Other Strategies**: [Which strategies?]
- **Diversification Value**: [What does this add to a portfolio?]

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1**: [Specific technical condition]
   - **Indicator**: [e.g., SMA crossover]
   - **Parameters**: [e.g., 50-day > 200-day]
   - **Confirmation**: [Any additional requirements]
   - **Priority**: [Required / Optional]

2. **Condition 2**: [Additional conditions as needed]

### 5.2 Entry Filters
- **Time of Day**: [If applicable]
- **Volume Requirements**: [e.g., Volume > 1.5x average]
- **Market Regime Filter**: [e.g., Only in trending markets]
- **Volatility Filter**: [e.g., ATR < 2x average]
- **Price Filter**: [e.g., Price above $X]

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: [e.g., Volume spike]
- **Confirmation Indicator 2**: [e.g., RSI not overbought]
- **Minimum Confirmed**: [e.g., 2 out of 3]

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP 1**: [e.g., 3% profit] - Close [e.g., 50%]
- **TP 2**: [e.g., 5% profit] - Close [e.g., 30%]
- **TP 3**: [e.g., 10% profit] - Close [e.g., 20%]
- **Trailing**: [Trailing stop after TP 1?]

### 6.2 Stop Loss
- **Initial SL**: [e.g., 2% below entry]
- **Trailing SL**: [e.g., 2% trailing after TP 1]
- **Breakeven**: [Move to breakeven after TP 1?]
- **Time-based Exit**: [e.g., Close after 30 days if no TP]

### 6.3 Exit Conditions
- **Reversal Signal**: [e.g., Death cross, opposite signal]
- **Regime Change**: [e.g., Market turns bearish]
- **Volatility Spike**: [e.g., ATR > 2x average]
- **Time Limit**: [e.g., Maximum 60 days in position]

## 7. Position Sizing

- **Base Position Size**: [e.g., 1% of portfolio]
- **Volatility Adjustment**: [e.g., Scale down if ATR > 1.5x avg]
- **Conviction Levels**: [Adjust size based on signal strength]
- **Max Position Size**: [e.g., 5% of portfolio]
- **Risk per Trade**: [e.g., Max 1% risk]

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| [param1] | [value] | [min-max] | [description] | [int/float] |
| [param2] | [value] | [min-max] | [description] | [int/float] |

### 8.2 Optimization Notes
- **Parameters to Optimize**: [Which parameters]
- **Optimization Method**: [Grid search / Walk-forward / Bayesian]
- **Optimization Period**: [Time period for optimization]
- **Expected Overfitting Risk**: [Low / Medium / High]
- **Sensitivity Analysis Required**: [Yes / No]

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- **Timeframes**: [e.g., Daily, 4H, 1H]
- **Test Period**: [e.g., 2019-2025, 6 years]
- **Assets**: [e.g., BTC, ETH, SOL, 10+ others]
- **Minimum Trades**: [For statistical significance]
- **Slippage**: [e.g., 0.1% per trade]
- **Commission**: [e.g., 0.1% per trade]

### 9.2 Validation Techniques
- [ ] Walk-forward analysis (rolling window)
- [ ] Monte Carlo simulation (trade sequence randomization)
- [ ] Parameter sweep (sensitivity analysis)
- [ ] Regime analysis (bull/bear/sideways)
- [ ] Cross-asset validation (multiple symbols)
- [ ] Bootstrap validation (resampling)
- [ ] Permutation testing (randomness check)

### 9.3 Success Criteria
- **Minimum Sharpe Ratio**: [e.g., 1.0]
- **Minimum Sortino Ratio**: [e.g., 1.5]
- **Maximum Max Drawdown**: [e.g., 20%]
- **Minimum Win Rate**: [e.g., 40%]
- **Minimum Profit Factor**: [e.g., 1.3]
- **Minimum Robustness Score**: [e.g., >70]
- **Statistical Significance**: [e.g., p < 0.05]
- **Walk-Forward Stability**: [e.g., >50]

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: [e.g., 20-30%]
- **Sharpe Ratio**: [e.g., 1.5-2.0]
- **Max Drawdown**: [e.g., <15%]
- **Win Rate**: [e.g., 45-55%]
- **Profit Factor**: [e.g., >1.5]
- **Expectancy**: [e.g., >0.02]

### 10.2 Comparison to Baselines
- **vs. HODL**: [Expected outperformance in %]
- **vs. Market Average**: [Expected outperformance in %]
- **vs. Similar Strategies**: [Unique advantages]
- **vs. Buy & Hold**: [Risk-adjusted comparison]

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: [List of indicators]
- **Data Requirements**: [OHLCV, volume, sentiment, etc.]
- **Latency Sensitivity**: [Low / Medium / High]
- **Computational Complexity**: [Low / Medium / High]
- **Memory Requirements**: [Approximate]

### 11.2 Code Structure
- **File Location**: [crates/strategy/src/strategies/[category]/[strategy].rs]
- **Strategy Type**: [Simple / Multi-indicator / ML-based]
- **Dependencies**: [External crates or internal modules]
- **State Management**: [How to track state between bars]

### 11.3 Indicator Calculations
[Describe how each indicator is calculated or reference existing implementations]

## 12. Testing Plan

### 12.1 Unit Tests
- [ ] Entry conditions (all signals generated correctly)
- [ ] Exit conditions (all exits triggered correctly)
- [ ] Edge cases (empty data, single bar, etc.)
- [ ] Parameter validation (invalid params rejected)
- [ ] State management (reset works correctly)

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

### [Date]: Initial Implementation
**Observation**: [What did you notice during implementation?]
**Hypothesis Impact**: [Does code support or contradict hypothesis?]
**Issues Found**: [Any problems or edge cases]
**Action Taken**: [What did you change?]

### [Date]: Initial Backtest Results
**Test Period**: [Date range]
**Symbols Tested**: [List of symbols]
**Results**: [Summary of metrics]
**Observation**: [Are results as expected?]
**Action Taken**: [Proceed to validation or refine strategy?]

### [Date]: Parameter Optimization
**Optimization Method**: [Grid search, etc.]
**Best Parameters**: [Results]
**Optimization Score**: [Performance score]
**Overfitting Check**: [Any concerns about overfitting?]
**Action Taken**: [Accept parameters or continue optimizing?]

### [Date]: Walk-Forward Validation
**Configuration**: [Window sizes, etc.]
**Results**: [Summary of WFA performance]
**Stability Score**: [How stable across windows?]
**Decision**: [Accept, reject, or modify?]

### [Date]: Monte Carlo Simulation
**Number of Simulations**: [e.g., 1000]
**95% Confidence Interval**: [Range]
**Best Case**: [Best outcome]
**Worst Case**: [Worst outcome]
**Observation**: [Is strategy robust to luck?]

### [Date]: Final Decision
**Final Verdict**: [Accept / Reject / Needs More Work]
**Reasoning**: [Why this decision?]
**Deployment**: [If accepted, when to deploy?]
**Monitoring**: [What to monitor in live trading?]

## 14. References

### Academic Sources
- [Citation of relevant academic papers]

### Books
- [Citation of relevant books]

### Online Resources
- [URLs to blog posts, articles, etc.]

### Similar Strategies
- [References to similar strategies and their performance]

### Historical Examples
- [Examples of this strategy working/failing in real markets]

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| [date] | 1.0 | Initial hypothesis | AI Agent |
| [date] | 1.1 | [Changes made] | AI Agent |
| [date] | 1.2 | [Changes made] | AI Agent |