# ATR Trailing Stop Strategy Hypothesis

## Metadata
- **Name**: ATR Trailing Stop Strategy
- **Category**: VolatilityBased
- **Sub-Type**: atr_trailing
- **Author**: AI Agent
- **Date**: 2026-01-15
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/volatility/atr_trailing.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
ATR-based trailing stops adapt to volatility and protect profits while allowing room for normal fluctuations. By using dynamic stops that widen in high volatility and tighten in low volatility, the strategy maximizes profitable trades and minimizes premature exits, generating positive returns with an average return of >6% over the holding period.

**Null Hypothesis**: 
ATR-based trailing stops do not improve performance compared to fixed-percentage stops and perform no better than random exit timing.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Average True Range (ATR) measures market volatility and typical price movement range. Fixed-percentage stops fail to adapt to changing market conditions—too tight in volatile markets (premature exits) and too loose in calm markets (giving back profits). ATR-based stops automatically adjust to current volatility, allowing positions room to breathe during normal fluctuations while protecting against significant reversals.

### 2.2 Market Inefficiency Exploited
The strategy exploits the suboptimal use of fixed stops by most traders. Fixed stops don't account for volatility, causing either premature exits (stops too tight) or excessive drawdowns (stops too loose). ATR-based stops provide volatility-aware risk management that aligns stop placement with current market conditions, capturing more of the trend while limiting downside.

### 2.3 Expected Duration of Edge
This edge persists as long as markets exhibit varying volatility levels and traders continue using suboptimal fixed stops. Since volatility clustering is a persistent market phenomenon and ATR provides a robust volatility measure, this edge should maintain an advantage over fixed-stop approaches.

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: Trailing stops capture extended uptrends while locking in profits during pullbacks
- **Historical Evidence**: Bull markets produce sustained moves where trailing stops excel

### 3.2 Bearish Markets
- **Expected Performance**: Low
- **Rationale**: Strategy is long-only, exits on stop loss or death cross
- **Adaptations**: Quick stop loss (3%) prevents deep drawdowns in bear markets

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Medium
- **Rationale**: Trailing stops may be triggered by normal volatility in ranges, causing whipsaws
- **Filters**: Golden cross entry reduces entries in sideways markets

### 3.4 Volatility Conditions
- **High Volatility**: High - ATR-based stops widen appropriately, preventing premature exits
- **Low Volatility**: Medium - stops tighten appropriately, protecting profits
- **Volatility Filter**: ATR itself serves as the volatility measure for stop adjustment

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 18%
- **Average Drawdown**: 5%
- **Drawdown Duration**: 1-2 weeks
- **Worst Case Scenario**: Rapid volatility spike causes trailing stop to be too wide, giving back significant profits

### 4.2 Failure Modes

#### Failure Mode 1: Premature Exit in Volatile Uptrend
- **Trigger**: Volatility spike causes ATR to increase, but stop also widens, potentially giving back too much
- **Impact**: Larger than expected loss before exit
- **Mitigation**: Cap maximum trailing distance (e.g., 8%), add profit-taking levels
- **Detection**: Average loss > stop loss threshold, large drawdowns before exits

#### Failure Mode 2: Whipsaw in Choppy Market
- **Trigger**: Price oscillates causing trailing stop to be triggered repeatedly
- **Impact**: Multiple small losses from frequent stop-outs
- **Mitigation**: Increase min_trailing_pct (from 1.0% to 1.5%), add trend filter
- **Detection**: Win rate falls below 35%, high trade frequency with low profit per trade

#### Failure Mode 3: Late Exit on Rapid Reversal
- **Trigger**: Price reversals faster than trailing stop adjustment
- **Impact**: Stop not updated quickly enough, giving back significant portion of profits
- **Mitigation**: Reduce atr_multiplier (from 2.0 to 1.5), add tighter initial stop
- **Detection**: Large drawdowns from peak to exit (e.g., >50% of max profit)

### 4.3 Correlation Analysis
- **Correlation with Market**: High (trend-following strategy)
- **Correlation with Other Strategies**: High with other trend-following strategies, Medium with volatility-based strategies
- **Diversification Value**: Low-Moderate - unique stop management approach

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1**: Golden cross entry
   - **Indicator**: Fast SMA crosses above Slow SMA
   - **Parameters**: SMA(fast_period) > SMA(slow_period) with crossover
   - **Confirmation**: Confirms uptrend start
   - **Priority**: Required

2. **Condition 2**: No existing position
   - **Indicator**: Position state
   - **Parameters**: Currently not in position
   - **Confirmation**: Avoids multiple entries
   - **Priority**: Required

### 5.2 Entry Filters
- **Time of Day**: Not applicable (works on any timeframe)
- **Volume Requirements**: None currently
- **Market Regime Filter**: Golden cross serves as trend filter
- **Volatility Filter**: None currently
- **Price Filter**: None currently

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: Golden cross (SMA crossover)
- **Minimum Confirmed**: 1 out of 1 (required)

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP 1**: 10% profit - Close 100%

### 6.2 Stop Loss
- **Initial SL**: 3% below entry
- **Trailing SL**: Dynamic - `stop = price - (ATR × atr_multiplier)`
- **Minimum Trailing**: min_trailing_pct (1.0%) below entry price
- **Breakeven**: None
- **Time-based Exit**: None

### 6.3 Exit Conditions
- **Take Profit Reached**: 10% profit target achieved
- **Stop Loss Triggered**: Price drops below trailing stop or initial 3% SL
- **Death Cross**: Fast SMA crosses below Slow SMA (optional exit)
- **Regime Change**: Not implemented
- **Volatility Drop**: Not implemented
- **Time Limit**: None

## 7. Position Sizing
- **Base Position Size**: 1% of portfolio per signal
- **Volatility Adjustment**: Could reduce size if trailing distance is very wide
- **Conviction Levels**: Could use crossover strength
- **Max Position Size**: 5% of portfolio
- **Risk per Trade**: Max 3% (initial stop loss)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| atr_period | 14 | 7-21 | ATR calculation period | int |
| atr_multiplier | 2.0 | 1.0-3.0 | ATR multiplier for trailing stop | float |
| fast_period | 10 | 5-15 | Fast SMA period for golden cross | int |
| slow_period | 30 | 20-50 | Slow SMA period for golden cross | int |
| min_trailing_pct | 1.0 | 0.5-2.0 | Minimum trailing distance as percentage | float |
| take_profit | 10.0 | 5.0-20.0 | Take profit percentage | float |
| stop_loss | 3.0 | 2.0-5.0 | Initial stop loss percentage | float |

### 8.2 Optimization Notes
- **Parameters to Optimize**: atr_multiplier, min_trailing_pct, fast_period, slow_period
- **Optimization Method**: Walk-forward analysis
- **Optimization Period**: 2 years
- **Expected Overfitting Risk**: Medium (trailing parameters can be curve-fit)
- **Sensitivity Analysis Required**: Yes

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
- **Maximum Max Drawdown**: 18%
- **Minimum Win Rate**: 45%
- **Minimum Profit Factor**: 1.5
- **Minimum Robustness Score**: >60
- **Statistical Significance**: p < 0.05
- **Walk-Forward Stability**: >50

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 20-40%
- **Sharpe Ratio**: 1.0-2.0
- **Max Drawdown**: <18%
- **Win Rate**: 45-55%
- **Profit Factor**: >1.5
- **Expectancy**: >0.03

### 10.2 Comparison to Baselines
- **vs. HODL**: +10-25% risk-adjusted returns
- **vs. Market Average**: +8-20% outperformance
- **vs. Fixed Stop Strategy**: Superior due to volatility adaptation
- **vs. Buy & Hold**: Lower max drawdown, higher win rate

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: ATR, SMA
- **Data Requirements**: OHLCV
- **Latency Sensitivity**: Low
- **Computational Complexity**: Low-Medium (ATR calculation, trailing stop management)
- **Memory Requirements**: Low (ATR history for calculation)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/volatility/atr_trailing.rs
- **Strategy Type**: Volatility-adjusted stop management
- **Dependencies**: alphafield_core, indicators::Atr, indicators::Sma
- **State Management**: Tracks trailing stop level, position, entry price, high watermark

### 11.3 Indicator Calculations
**ATR**: Average True Range over atr_period (14) bars using Wilder's smoothing

**Golden Cross**: Fast SMA (fast_period = 10) crosses above Slow SMA (slow_period = 30)

**Death Cross**: Fast SMA (fast_period = 10) crosses below Slow SMA (slow_period = 30)

**Trailing Stop**: 
- Initial stop: Entry price × (1 - stop_loss/100)
- Trailing stop: Current price - (ATR × atr_multiplier)
- Minimum trailing: Entry price × (1 - min_trailing_pct/100)
- Stop = max(initial_stop, trailing_stop, min_trailing)
- Only raise stop, never lower it (trail upward only)

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (golden cross detection)
- [x] Exit conditions (TP, SL, trailing stop, death cross)
- [x] Edge cases (empty data, insufficient bars, single bar)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)
- [x] ATR calculation
- [x] Trailing stop calculation and adjustment
- [x] Minimum trailing distance enforcement
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

## 13. Research Journal

### 2026-01-15: Initial Implementation
**Observation**: Strategy implemented with ATR-based trailing stop and golden cross entry
**Hypothesis Impact**: Code supports hypothesis - uses dynamic trailing stops that adapt to volatility
**Issues Found**: None during implementation
**Action Taken**: Ready for testing

### [Date]: Initial Backtest Results
**Test Period**: [Date range]
**Symbols Tested**: [List of symbols]
**Results**: [Summary of metrics]
**Observation**: [Are results as expected?]
**Action Taken**: [Proceed to validation or refine strategy?]

### [Date]: ATR Multiplier Analysis
**Observation**: [How does trailing stop performance vary with atr_multiplier?]
**Hypothesis Impact**: [Does optimal atr_multiplier validate the hypothesis?]
**Issues Found**: [Any parameter-specific problems]
**Action Taken**: [Parameter adjustments or range modifications]

### [Date]: Comparison with Fixed Stops
**Observation**: [How does ATR-based stop compare to fixed-percentage stops?]
**Hypothesis Impact**: [Does this support the primary hypothesis?]
**Issues Found**: [Any scenarios where fixed stops outperform?]
**Action Taken**: [Hybrid approach or parameter refinement]

## 14. References

### Academic Sources
- Wilder, J. Welles. "New Concepts in Technical Trading Systems" (1978) - Original ATR development
- Le Beau, C., & Lucas, D. W. "Technical Traders Guide to Computer Analysis of the Futures Market" (1992) - Stop management concepts

### Books
- Kaufman, P. J. "Trading Systems and Methods" - Comprehensive stop and volatility analysis
- Tharp, V. K. "Trade Your Way to Financial Freedom" - Risk management and stop strategies
- Elder, A. "Come into My Trading Room" - Trailing stop techniques

### Online Resources
- ATR-based stop loss research and case studies
- Volatility-adjusted risk management strategies
- Trailing stop optimization techniques
- Crypto market volatility and stop management

### Similar Strategies
- ATR Breakout (similar ATR usage, different entry)
- Fixed Stop Strategy (baseline comparison)
- Golden Cross (same entry, different exit)

### Historical Examples
- 2020-2021 crypto bull market (trailing stops captured extended uptrend)
- 2022 crypto bear market (quick stops limited losses)
- Various trending markets showing trailing stop effectiveness

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-15 | 1.0 | Initial hypothesis | AI Agent |