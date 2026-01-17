# Volatility Squeeze Strategy Hypothesis

## Metadata
- **Name**: Volatility Squeeze Strategy
- **Category**: VolatilityBased
- **Sub-Type**: vol_squeeze
- **Author**: AI Agent
- **Date**: 2026-01-15
- **Status**: Testing
- **Code Location**: crates/strategy/src/strategies/volatility/vol_squeeze.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
Volatility squeeze precedes significant price movements. When Bollinger Band width is narrower than Keltner Channel width (indicating low volatility), the market is in a consolidation phase that will be followed by a directional breakout. Entering on the breakout with volume confirmation generates positive returns with an average return of >6% over the holding period.

**Null Hypothesis**: 
Volatility squeeze detection does not reliably predict breakout direction or magnitude and performs no better than random entry at similar price levels.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Volatility squeezes occur when markets enter periods of low volatility and consolidation. The narrowing of price range (Bollinger Bands) relative to volatility (Keltner Channel) indicates that the market is storing energy for a move. When price breaks out of this squeeze with volume, it often leads to a sustained directional move as stored energy is released.

### 2.2 Market Inefficiency Exploited
The strategy exploits the under-reaction to low volatility periods. Most traders focus on high volatility events, missing the opportunity to position before the breakout. The Bollinger Band/Keltner Channel combination provides a robust measure of squeeze that identifies consolidation phases before they become obvious to the broader market.

### 2.3 Expected Duration of Edge
This edge persists as long as markets exhibit periods of consolidation followed by directional moves. Volatility squeezes are a recurring market pattern due to the natural cycles of accumulation/distribution and the tendency of markets to alternate between low and high volatility regimes.

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: Medium-High
- **Rationale**: Squeezes in bull markets often lead to continuation breakouts upward
- **Historical Evidence**: Bull markets produce consolidation periods that break out upward

### 3.2 Bearish Markets
- **Expected Performance**: Low-Medium
- **Rationale**: Squeezes in bear markets can lead to breakdowns; strategy is long-only
- **Adaptations**: Quick exit on false breakout or re-entry into squeeze

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: High
- **Rationale**: Squeeze detection excels in identifying consolidation phases within ranges
- **Filters**: Volume confirmation helps distinguish genuine breakouts from false moves

### 3.4 Volatility Conditions
- **High Volatility**: Low - squeeze detection requires low volatility periods
- **Low Volatility**: High - ideal conditions for squeeze formation and detection
- **Volatility Filter**: The squeeze condition itself is the volatility filter

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 18%
- **Average Drawdown**: 5%
- **Drawdown Duration**: 1-2 weeks
- **Worst Case Scenario**: False breakout followed by immediate reversal, hitting stop loss

### 4.2 Failure Modes

#### Failure Mode 1: False Breakout from Squeeze
- **Trigger**: Price temporarily breaks out of squeeze but immediately reverses and re-enters squeeze
- **Impact**: Stop loss hit as price falls back; multiple false breakouts possible in extended squeezes
- **Mitigation**: Require breakout persistence (2-3 bars), add additional volume confirmation (>2.0×)
- **Detection**: Win rate falls below 35%, frequent re-entries into squeeze after exit

#### Failure Mode 2: Extended Squeeze Without Breakout
- **Trigger**: Market remains in squeeze for extended period without clear directional move
- **Impact**: Opportunity cost from capital tied up waiting for breakout
- **Mitigation**: Add maximum squeeze duration filter (e.g., 50 bars), reduce position size after extended squeeze
- **Detection**: Low trade frequency, squeeze duration > historical average

#### Failure Mode 3: Late Entry After Most of the Breakout
- **Trigger**: Breakout detection occurs after significant portion of the move has already occurred
- **Impact**: Limited upside with same downside risk (asymmetric risk/reward)
- **Mitigation**: Use tighter squeeze_threshold (0.05) for earlier detection, add momentum confirmation
- **Detection**: Average profit per trade significantly lower than breakout move magnitude

### 4.3 Correlation Analysis
- **Correlation with Market**: Medium (breakout-based, but selective on volatility)
- **Correlation with Other Strategies**: Medium-High with other volatility-based strategies, Medium with trend-following
- **Diversification Value**: Moderate - unique volatility approach differs from simple trend following

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1**: Volatility squeeze detected
   - **Indicator**: Bollinger Band width vs Keltner Channel width
   - **Parameters**: BB_width < Keltner_width × squeeze_threshold (0.1)
   - **Confirmation**: Market is in low volatility consolidation phase
   - **Priority**: Required

2. **Condition 2**: Breakout from squeeze
   - **Indicator**: Price breakout above Bollinger Band upper band
   - **Parameters**: Price > BB_upper_band
   - **Confirmation**: Indicates directional move out of consolidation
   - **Priority**: Required

3. **Condition 3**: Volume confirmation
   - **Indicator**: Current volume / average volume
   - **Parameters**: Volume > average_volume × 1.5
   - **Confirmation**: Ensures breakout has market participation
   - **Priority**: Required

### 5.2 Entry Filters
- **Time of Day**: Not applicable (works on any timeframe)
- **Volume Requirements**: Volume > average_volume × 1.5
- **Market Regime Filter**: Squeeze condition serves as regime filter
- **Volatility Filter**: Low volatility (squeeze) required for setup
- **Price Filter**: Price must break above BB_upper_band

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: Squeeze detection
- **Confirmation Indicator 2**: Breakout above BB_upper_band
- **Confirmation Indicator 3**: Volume spike (>1.5× average)
- **Minimum Confirmed**: 3 out of 3 (all required)

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP 1**: 8% profit - Close 100%

### 6.2 Stop Loss
- **Initial SL**: 3% below entry
- **Trailing SL**: None
- **Breakeven**: None
- **Time-based Exit**: None

### 6.3 Exit Conditions
- **Take Profit Reached**: 8% profit target achieved
- **Stop Loss Triggered**: 3% loss threshold exceeded
- **Re-enter Squeeze**: Price re-enters squeeze condition (false breakout detection)
- **Regime Change**: Not implemented
- **Volatility Drop**: Not implemented
- **Time Limit**: None

## 7. Position Sizing
- **Base Position Size**: 1% of portfolio per signal
- **Volatility Adjustment**: Could increase size after extended squeeze (>30 bars)
- **Conviction Levels**: Could use squeeze duration and volume strength
- **Max Position Size**: 5% of portfolio
- **Risk per Trade**: Max 3% (stop loss)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| bb_period | 20 | 10-40 | Bollinger Bands period | int |
| bb_std_dev | 2.0 | 1.5-2.5 | Bollinger Bands standard deviation | float |
| kk_period | 20 | 10-40 | Keltner Channel period | int |
| kk_mult | 1.5 | 1.0-2.0 | Keltner Channel ATR multiplier | float |
| squeeze_threshold | 0.1 | 0.05-0.2 | Squeeze detection threshold | float |
| volume_multiplier | 1.5 | 1.0-3.0 | Volume confirmation multiplier | float |
| take_profit | 8.0 | 5.0-15.0 | Take profit percentage | float |
| stop_loss | 3.0 | 2.0-5.0 | Stop loss percentage | float |

### 8.2 Optimization Notes
- **Parameters to Optimize**: squeeze_threshold, bb_period, kk_period
- **Optimization Method**: Walk-forward analysis
- **Optimization Period**: 2 years
- **Expected Overfitting Risk**: Medium (squeeze parameters can be curve-fit)
- **Sensitivity Analysis Required**: Yes

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- **Timeframes**: 1H, 4H, 1D
- **Test Period**: 2019-2025 (6 years)
- **Assets**: BTC, ETH, SOL, 10+ others
- **Minimum Trades**: 60 (squeezes are less frequent)
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
- **Minimum Win Rate**: 42%
- **Minimum Profit Factor**: 1.6
- **Minimum Robustness Score**: >65
- **Statistical Significance**: p < 0.05
- **Walk-Forward Stability**: >50

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 18-35%
- **Sharpe Ratio**: 1.0-2.0
- **Max Drawdown**: <18%
- **Win Rate**: 42-52%
- **Profit Factor**: >1.6
- **Expectancy**: >0.04

### 10.2 Comparison to Baselines
- **vs. HODL**: +8-20% risk-adjusted returns
- **vs. Market Average**: +6-15% outperformance
- **vs. Simple Breakout**: Superior due to squeeze pre-filter
- **vs. Buy & Hold**: Lower max drawdown, higher win rate

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: Bollinger Bands, Keltner Channel, ATR, EMA, Volume Average
- **Data Requirements**: OHLCV
- **Latency Sensitivity**: Low
- **Computational Complexity**: Medium (BB, Keltner, squeeze detection)
- **Memory Requirements**: Moderate (period bars for indicator calculations)

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/volatility/vol_squeeze.rs
- **Strategy Type**: Volatility squeeze breakout
- **Dependencies**: alphafield_core, indicators::BollingerBands, indicators::Atr, indicators::Ema
- **State Management**: Tracks squeeze state, breakout confirmation, position, entry price

### 11.3 Indicator Calculations
**Bollinger Bands**:
- Middle Band: SMA(price, bb_period)
- Upper Band: Middle + (bb_std_dev × std_dev(price, bb_period))
- Lower Band: Middle - (bb_std_dev × std_dev(price, bb_period))
- BB Width: Upper Band - Lower Band

**Keltner Channel**:
- Middle Band: EMA(price, kk_period)
- Upper Band: Middle + (kk_mult × ATR)
- Lower Band: Middle - (kk_mult × ATR)
- Keltner Width: Upper Band - Lower Band

**Squeeze Detection**: BB Width < Keltner Width × squeeze_threshold

## 12. Testing Plan

### 12.1 Unit Tests
- [x] Entry conditions (squeeze detection, breakout, volume confirmation)
- [x] Exit conditions (TP, SL, re-enter squeeze)
- [x] Edge cases (empty data, insufficient bars, single bar)
- [x] Parameter validation (invalid params rejected)
- [x] State management (reset works correctly)
- [x] Squeeze detection logic
- [x] Breakout detection
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
**Observation**: Strategy implemented with Bollinger Bands/Keltner Channel squeeze detection and breakout entry
**Hypothesis Impact**: Code supports hypothesis - detects squeezes, enters on breakout with volume, exits on TP/SL or re-entry to squeeze
**Issues Found**: None during implementation
**Action Taken**: Ready for testing

### [Date]: Initial Backtest Results
**Test Period**: [Date range]
**Symbols Tested**: [List of symbols]
**Results**: [Summary of metrics]
**Observation**: [Are results as expected?]
**Action Taken**: [Proceed to validation or refine strategy?]

### [Date]: False Breakout Analysis
**Observation**: [What patterns cause false breakouts from squeezes?]
**Hypothesis Impact**: [Does this invalidate the hypothesis or require parameter adjustment?]
**Issues Found**: [Specific failure patterns]
**Action Taken**: [Parameter adjustments or filter additions]

## 14. References

### Academic Sources
- Raschke, L., & Connors, L. "Street Smarts: High Probability Short-Term Trading Strategies" (1995) - Squeeze concepts
- Bollinger, J. "Bollinger on Bollinger Bands" (2002) - Bollinger Bands theory

### Books
- Kaufman, P. J. "Trading Systems and Methods" - Volatility and breakout analysis
- Bulkowski, T. N. "Encyclopedia of Chart Patterns" - Consolidation and breakout patterns

### Online Resources
- Volatility squeeze trading research and case studies
- Bollinger Band/Keltner Channel combination strategies
- Crypto market volatility patterns and squeezes

### Similar Strategies
- ATR Breakout (similar breakout approach without squeeze pre-filter)
- Volatility Regime (different volatility adaptation approach)
- Bollinger Band Breakout (simpler version without Keltner Channel)

### Historical Examples
- 2020 COVID crash recovery (squeeze before major breakout)
- 2021 crypto consolidation periods (multiple squeezes leading to breakouts)
- Various market consolidations showing squeeze patterns

## 15. Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-01-15 | 1.0 | Initial hypothesis | AI Agent |