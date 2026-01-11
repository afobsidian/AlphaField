# Golden Cross Strategy Hypothesis

## Metadata
- **Name**: Golden Cross
- **Category**: TrendFollowing
- **Sub-Type**: Moving Average Crossover
- **Author**: AI Agent
- **Date**: 2025-01-02
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/trend_following/golden_cross.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
When the fast simple moving average (SMA) crosses above the slow SMA (Golden Cross), the asset enters a sustainable uptrend and will generate positive returns over the next 30 trading days with an average return of >3%, outperforming a buy-and-hold baseline with better risk-adjusted returns.

**Null Hypothesis**: 
Golden Cross crossovers do not provide any statistically significant edge over random entry points. Any observed positive returns are due to general market trends rather than the crossover signal itself.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
The Golden Cross is based on the principle that short-term price momentum, when confirmed by longer-term trend alignment, indicates a sustainable trend shift. When the fast SMA (e.g., 50-day) crosses above the slow SMA (e.g., 200-day), it suggests that recent price action is strong enough to overcome long-term resistance, indicating institutional buying pressure and potentially the start of a sustained uptrend.

Moving averages smooth out price noise and reveal the underlying trend direction. The crossover of two differently-paced MAs captures the acceleration of price movement - a fundamental shift in market sentiment from bearish or neutral to bullish.

### 2.2 Market Inefficiency Exploited
This strategy exploits the **trend persistence bias** - markets tend to move in trends more than random walk theory would predict. Many market participants are slow to recognize trend changes, creating delayed reactions. The Golden Cross provides a clear, objective signal that catches trends early while many traders are still uncertain.

Additionally, the strategy exploits **herding behavior** - once a trend is established, more participants join, reinforcing the trend. The crossover provides an early entry point before the herd fully commits.

### 2.3 Expected Duration of Edge
The edge is expected to persist as long as markets exhibit trending behavior. However, in highly efficient markets or during regimes of high algorithmic trading activity, the edge may diminish. The strategy is most effective during:
- Periods of sustained market movements (not ranging)
- Markets with moderate to high liquidity
- Assets with active institutional participation

The edge may degrade during:
- High-frequency trading dominance
- Extremely volatile conditions with rapid reversals
- Strong mean-reversion regimes

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: Golden Cross signals are most reliable in established or emerging bull markets. The crossover often catches the early-to-mid stages of uptrends, allowing participation in sustained upward movements. In strong bull markets, whipsaws are less frequent.
- **Historical Evidence**: Historical studies show Golden Cross signals in equity markets have produced positive returns 60-70% of the time in bull market conditions. Crypto markets exhibit similar behavior during extended uptrends.

### 3.2 Bearish Markets
- **Expected Performance**: Low to Medium
- **Rationale**: In bear markets, Golden Cross signals may produce short-lived rallies (bear market rallies) that quickly reverse. The strategy can still work but with lower win rates and higher volatility. Fast exits are critical.
- **Adaptations**: Consider shorter SMA periods (e.g., 20/50 instead of 50/200) to be more responsive to shorter-term rallies. Implement tighter stop losses to limit exposure.

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Low
- **Rationale**: Golden Cross performs poorly in ranging markets as price oscillates around the moving averages, generating multiple false signals (whipsaws). The strategy loses money through frequent trading and transaction costs.
- **Filters**: Implement volatility filters (e.g., ATR > threshold) to avoid entering during low-volatility ranging periods. Consider using ADX to identify trending vs. ranging conditions.

### 3.4 Volatility Conditions
- **High Volatility**: Moderate performance - signals are more likely to be false, but when they work, gains can be larger. Requires wider stop losses (3-5%) to avoid premature exits.
- **Low Volatility**: Poor performance - signals are rare and tend to be weak. Strategy may not generate enough signals to be profitable.
- **Volatility Filter**: Consider requiring ATR to be within a specific range (neither too low nor too high) before accepting signals.

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 25-30% per trade
- **Average Drawdown**: 10-15% during losing streaks
- **Drawdown Duration**: 1-4 weeks per losing trade
- **Worst Case Scenario**: 
  - Multiple consecutive whipsaws in ranging market
  - 5-10 consecutive losing trades
  - 40-50% drawdown in severe conditions
  - Strategy may underperform HODL by 20-30% in strong uptrends if exits are too early

### 4.2 Failure Modes

#### Failure Mode 1: Whipsaw in Ranging Market
- **Trigger**: Price oscillates around MAs, generating multiple crossover signals
- **Impact**: Multiple small losses (3-5% each), high transaction costs, psychological stress
- **Frequency**: 30-40% of signals in ranging conditions
- **Mitigation**: 
  - Add ADX filter (require ADX > 25 for trending market)
  - Add volume confirmation (require volume spike on crossover)
  - Implement minimum separation between MAs (e.g., >1% difference)
- **Detection**: Track recent signal history - if last 2-3 signals were losses, increase filter strictness

#### Failure Mode 2: Late Entry in Exhausted Trend
- **Trigger**: Golden Cross occurs near the end of an uptrend, followed by reversal
- **Impact**: Small gains followed by rapid reversal losses, potentially worse than buy-and-hold
- **Frequency**: 10-15% of signals
- **Mitigation**: 
  - Add RSI filter (avoid if RSI > 70, overbought)
  - Add trend strength filter (require MA separation > threshold)
  - Use shorter periods for faster exits
- **Detection**: Monitor rate of change of MAs - if slow MA is rising sharply, trend may be exhausted

#### Failure Mode 3: Flash Crash or Gap Down
- **Trigger**: Sudden price movement below stop loss, gaps through stop level
- **Impact**: Losses exceed stop loss amount (5-10% instead of 3-5%)
- **Frequency**: 5-10% of trades, typically during news events
- **Mitigation**: 
  - Use limit orders instead of market orders for exits
  - Add volatility-based stop widening during high volatility
  - Consider options for downside protection
- **Detection**: Monitor ATR for sudden spikes before market open

### 4.3 Correlation Analysis
- **Correlation with Market**: High (0.7-0.9) - strategy is long-only and trends with market
- **Correlation with Other Strategies**: 
  - High with other trend-following strategies (0.6-0.8)
  - Medium with momentum strategies (0.4-0.6)
  - Low with mean-reversion strategies (-0.2 to 0.2)
- **Diversification Value**: Limited - strategy works best as a core position with complementary mean-reversion or volatility strategies. Not suitable as the sole strategy in a diversified portfolio.

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1**: Golden Cross Crossover
   - **Indicator**: Fast SMA crosses above Slow SMA
   - **Parameters**: Fast SMA period = 50, Slow SMA period = 200
   - **Confirmation**: Prior fast SMA <= prior slow SMA, current fast SMA > current slow SMA
   - **Priority**: Required

2. **Condition 2**: MA Separation
   - **Indicator**: Percentage difference between fast and slow SMA
   - **Parameters**: Separation > 1% (configurable)
   - **Rationale**: Ensures meaningful crossover, not noise
   - **Priority**: Required

3. **Condition 3**: Not Overbought
   - **Indicator**: RSI (14-period)
   - **Parameters**: RSI < 70 (optional filter)
   - **Rationale**: Avoid late entries in exhausted trends
   - **Priority**: Optional

4. **Condition 4**: Trending Market
   - **Indicator**: ADX (14-period)
   - **Parameters**: ADX > 25 (optional filter)
   - **Rationale**: Avoid ranging markets where whipsaws are common
   - **Priority**: Optional

### 5.2 Entry Filters
- **Time of Day**: Not applicable for daily timeframe (24/7 crypto markets)
- **Volume Requirements**: Volume > 1.0x average volume over past 20 periods (optional)
- **Market Regime Filter**: Prefer bull markets, avoid extreme bear markets
- **Volatility Filter**: ATR within 0.5x to 2.0x of 30-day average (avoid extreme conditions)
- **Price Filter**: None required, but avoid very low liquidity assets

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: Volume spike (>1.2x average) on crossover bar (optional)
- **Confirmation Indicator 2**: Close above fast SMA on next bar (confirmation)
- **Minimum Confirmed**: 2 out of 2 (crossover + next bar close)

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP 1**: 5% profit - Close 50% of position
- **TP 2**: 10% profit - Close 30% of position
- **TP 3**: 20% profit - Close remaining 20% of position
- **Trailing**: 2% trailing stop after TP 1 reached

### 6.2 Stop Loss
- **Initial SL**: 5% below entry price
- **Trailing SL**: 2% trailing stop after TP 1 (moves up as price rises, never down)
- **Breakeven**: Move stop to breakeven after TP 2 (10% profit)
- **Time-based Exit**: Close position after 60 days if no TP or SL triggered (reduce exposure)

### 6.3 Exit Conditions
- **Reversal Signal**: Death Cross (fast SMA crosses below slow SMA) - exit 100% remaining position
- **Regime Change**: If market enters confirmed bear market (200 SMA turns down), consider exiting even if in profit
- **Volatility Spike**: If ATR > 3x average, tighten stop loss to 3%
- **Time Limit**: Maximum 60 days in position to avoid stagnation

## 7. Position Sizing

- **Base Position Size**: 2% of portfolio capital per trade
- **Volatility Adjustment**: 
  - If ATR > 1.5x average: Reduce position size to 1%
  - If ATR < 0.5x average: Increase position size to 3% (higher conviction in low volatility)
- **Conviction Levels**: 
  - Strong signal (high separation, high volume): 2.5%
  - Weak signal (low separation, low volume): 1.5%
- **Max Position Size**: 5% of portfolio (absolute maximum)
- **Risk per Trade**: Maximum 1% of portfolio (stop loss × position size)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| fast_period | 50 | 10-100 | Period for fast SMA | int |
| slow_period | 200 | 50-400 | Period for slow SMA | int |
| min_separation | 1.0% | 0.1-5.0% | Minimum MA separation for valid signal | float |
| take_profit | 5.0% | 2.0-15.0% | First take profit level | float |
| stop_loss | 5.0% | 2.0-10.0% | Initial stop loss percentage | float |
| trailing_stop | 2.0% | 1.0-5.0% | Trailing stop after TP1 | float |
| rsi_filter_enabled | false | boolean | Enable RSI overbought filter | bool |
| rsi_threshold | 70 | 60-80 | RSI threshold for overbought filter | int |
| adx_filter_enabled | false | boolean | Enable ADX trending filter | bool |
| adx_threshold | 25 | 20-35 | ADX threshold for trending filter | int |

### 8.2 Optimization Notes
- **Parameters to Optimize**: fast_period, slow_period, min_separation, take_profit, stop_loss
- **Optimization Method**: Grid search with walk-forward validation
- **Optimization Period**: 3 years in-sample, 2 years out-of-sample
- **Expected Overfitting Risk**: Medium - MA crossover strategies are prone to overfitting to historical data
- **Sensitivity Analysis Required**: Yes - test robustness across different parameter combinations
- **Key Finding**: Historical analysis suggests (50, 200) is robust across many assets, but (20, 100) works better in volatile crypto markets

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- **Timeframes**: Daily (primary), 4-hour (secondary for crypto)
- **Test Period**: 2019-2025 (6 years) - includes bull, bear, and ranging regimes
- **Assets**: BTC, ETH, SOL, ADA, XRP, DOT, AVAX, LINK, MATIC, UNI (10+ crypto assets)
- **Minimum Trades**: 30 trades per asset for statistical significance
- **Slippage**: 0.1% per trade (conservative estimate)
- **Commission**: 0.1% per trade (typical exchange fee)
- **Position sizing**: Fixed fractional (2% of portfolio)

### 9.2 Validation Techniques
- [ ] Walk-forward analysis (rolling window: 2-year train, 1-year test, 6-month step)
- [ ] Monte Carlo simulation (1000 iterations of trade sequence randomization)
- [ ] Parameter sweep (10x10 grid for fast_period and slow_period)
- [ ] Regime analysis (separate results for bull/bear/sideways periods)
- [ ] Cross-asset validation (test on out-of-sample assets)
- [ ] Bootstrap validation (resample trades with replacement)
- [ ] Permutation testing (randomize entry dates to test significance)

### 9.3 Success Criteria
- **Minimum Sharpe Ratio**: 1.0 (risk-adjusted return threshold)
- **Minimum Sortino Ratio**: 1.5 (focus on downside risk)
- **Maximum Max Drawdown**: 25% (absolute worst case)
- **Minimum Win Rate**: 45% (trend strategies often have lower win rates but higher average wins)
- **Minimum Profit Factor**: 1.3 (gross profit / gross loss)
- **Minimum Robustness Score**: 60 (walk-forward stability measure)
- **Statistical Significance**: p < 0.05 (t-test vs. random entries)
- **Walk-Forward Stability**: >50% of walk-forward windows profitable

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 25-40% (before fees)
- **Sharpe Ratio**: 1.2-1.8
- **Max Drawdown**: 20-25%
- **Win Rate**: 45-55%
- **Profit Factor**: 1.5-2.0
- **Expectancy**: 0.03-0.05 (3-5% per trade average profit)
- **Average Trade Duration**: 30-60 days

### 10.2 Comparison to Baselines
- **vs. HODL**: 
  - Expected outperformance: 10-20% annually
  - Lower maximum drawdown
  - Better risk-adjusted returns (higher Sharpe)
- **vs. Market Average**: 
  - Expected outperformance: 15-25% annually
  - More consistent returns across different market conditions
- **vs. Similar Strategies**: 
  - Simpler than MACD crossover, less prone to whipsaws
  - More reliable than single MA strategies
  - Slower to react than price breakout strategies but more stable
- **vs. Buy & Hold**: 
  - Better downside protection (stop losses)
  - May underperform in strong uptrends if exits are too early
  - Superior in bear or ranging markets

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: SMA (simple moving average), RSI (optional), ADX (optional), ATR (for stops)
- **Data Requirements**: OHLCV (Open, High, Low, Close, Volume) data
- **Latency Sensitivity**: Low - strategy operates on daily timeframe
- **Computational Complexity**: Low - simple arithmetic operations
- **Memory Requirements**: Minimal - need to store last slow_period bars

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/trend_following/golden_cross.rs
- **Strategy Type**: Simple trend-following
- **Dependencies**: 
  - alphafield_core::Bar, Signal, SignalType, Strategy
  - crate::indicators::Sma
  - crate::framework::{MetadataStrategy, StrategyMetadata, StrategyCategory, etc.}
  - crate::config::GoldenCrossConfig
- **State Management**: 
  - Track current SMA values
  - Track previous SMA values for crossover detection
  - Track entry price and position state
  - Track stop loss level

### 11.3 Indicator Calculations
**SMA Calculation**:
```
SMA = Sum(closes over period) / period
```
Already implemented in `crates/strategy/src/indicators.rs`

**Crossover Detection**:
```
Golden Cross = (prior_fast <= prior_slow) AND (current_fast > current_slow)
Death Cross = (prior_fast >= prior_slow) AND (current_fast < current_slow)
```

**MA Separation**:
```
Separation % = ((fast_sma - slow_sma) / slow_sma) * 100
```

## 12. Testing Plan

### 12.1 Unit Tests
- [ ] Entry conditions (golden cross signal generated correctly)
- [ ] Exit conditions (death cross signal generated correctly)
- [ ] Take profit logic (partial exits at correct levels)
- [ ] Stop loss logic (exit when stop triggered)
- [ ] Trailing stop logic (stop moves up correctly, never down)
- [ ] Edge cases (empty data, insufficient bars, NaN values)
- [ ] Parameter validation (invalid fast/slow periods rejected)
- [ ] State management (reset clears all state, position tracking works)
- [ ] MA separation filter (signal rejected if separation too small)
- [ ] Multiple crossovers (handles consecutive crossovers correctly)

### 12.2 Integration Tests
- [ ] Backtest execution (runs without errors across multiple assets)
- [ ] Performance calculation (metrics are accurate vs. manual calculation)
- [ ] Dashboard integration (strategy appears in API, metadata accessible)
- [ ] Database integration (performance metrics saved correctly)
- [ ] Registry integration (strategy registered and retrievable)
- [ ] Parameter sweep (testing across parameter ranges)

### 12.3 Research Tests
- [ ] Hypothesis validation (results support primary hypothesis of >3% avg return)
- [ ] Statistical significance (p < 0.05 for outperformance vs. random)
- [ ] Regime analysis (better performance in bull markets, worse in ranging)
- [ ] Robustness testing (stable performance across parameter variations)
- [ ] Walk-forward stability (strategy profitable in majority of windows)
- [ ] Monte Carlo analysis (distribution of outcomes acceptable)

## 13. Research Journal

### 2025-01-02: Initial Hypothesis Creation
**Observation**: Golden Cross is a well-documented pattern in traditional markets but needs validation in crypto markets which are more volatile and 24/7.
**Hypothesis Impact**: Hypothesis assumes similar behavior in crypto as equities, but may need adjustments for higher volatility.
**Issues Found**: None yet - hypothesis is theoretical at this stage.
**Action Taken**: Created comprehensive hypothesis document with all required sections. Ready for implementation.

### [Date]: Initial Implementation
**Observation**: [To be filled during implementation]
**Hypothesis Impact**: [To be filled]
**Issues Found**: [To be filled]
**Action Taken**: [To be filled]

### [Date]: Initial Backtest Results
**Test Period**: [To be filled]
**Symbols Tested**: [To be filled]
**Results**: [To be filled]
**Observation**: [To be filled]
**Action Taken**: [To be filled]

### [Date]: Parameter Optimization
**Optimization Method**: [To be filled]
**Best Parameters**: [To be filled]
**Optimization Score**: [To be filled]
**Observation**: [To be filled]
**Action Taken**: [To be filled]

### [Date]: Walk-Forward Validation
**Window Size**: [To be filled]
**Step Size**: [To be filled]
**Stability Score**: [To be filled]
**Observation**: [To be filled]
**Action Taken**: [To be filled]

### [Date]: Monte Carlo Simulation
**Iterations**: [To be filled]
**Results**: [To be filled]
**Observation**: [To be filled]
**Action Taken**: [To be filled]

### [Date]: Final Validation
**Overall Assessment**: [To be filled]
**Recommendation**: [Deploy/Reject/Improve]
**Confidence Level**: [Low/Medium/High]
**Next Steps**: [To be filled]

## 14. References

### Academic Sources
- Brock, W., Lakonishok, J., & LeBaron, B. (1992). "Simple Technical Trading Rules and the Stochastic Properties of Stock Returns." Journal of Finance, 47(5), 1731-1764.
- Sullivan, R., Timmermann, A., & White, H. (1999). "Data-Snooping, Technical Trading Rule Performance, and the Bootstrap." Journal of Finance, 54(5), 1647-1691.

### Books
- Murphy, J. J. (1999). "Technical Analysis of the Financial Markets." New York Institute of Finance.
- Kaufman, P. J. (2013). "Trading Systems and Methods." Wiley.

### Online Resources
- Investopedia: "Golden Cross" - Definition and explanation
- TradingView: Golden Cross indicator documentation and community examples
- QuantConnect: Golden Cross strategy tutorials and research

### Similar Strategies
- Death Cross (opposite signal)
- Triple Moving Average Crossover
- MACD Crossover (related momentum indicator)
- Price Channel Breakout (alternative trend entry)

### Historical Examples
- 2009-2010: Post-financial crisis market recovery (equities)
- 2017: Crypto bull market (BTC, ETH)
- 2020-2021: COVID recovery bull market
- 2023-2024: Crypto recovery trend

## 15. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2025-01-02 | 1.0 | AI Agent | Initial hypothesis creation |
| [Date] | [Version] | [Author] | [Description of changes] |

---
**Status**: Draft - Ready for Implementation
**Next Action**: Implement strategy code in crates/strategy/src/strategies/trend_following/golden_cross.rs
**Expected Completion**: [Date]