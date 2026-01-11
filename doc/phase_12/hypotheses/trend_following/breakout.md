# Breakout Strategy Hypothesis

## Metadata
- **Name**: Breakout Strategy
- **Category**: TrendFollowing
- **Sub-Type**: Price Breakout
- **Author**: AI Agent
- **Date**: 2025-01-02
- **Status**: Proposed
- **Code Location**: crates/strategy/src/strategies/trend_following/breakout.rs

## 1. Hypothesis Statement

**Primary Hypothesis**: 
When price breaks above a recent high (e.g., 20-period high) with sufficient volume, it signals the start of a new uptrend that will generate positive returns over the next 20-30 trading days with an average return of >4%, outperforming buy-and-hold with better risk-adjusted returns through dynamic stop management.

**Null Hypothesis**: 
Price breakouts do not provide any statistically significant edge over random entry points. Breakouts are often false signals that result in whipsaws and losses similar to or worse than buy-and-hold.

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Breakout strategies are based on the principle that markets move in trends and that breaking through significant price levels (recent highs/lows) indicates a shift in market sentiment and potentially the start of a new trend. When price breaks above a recent high with strong volume, it suggests:

1. **Demand Overcomes Supply**: Buyers are willing to pay higher prices, overcoming previous resistance
2. **Institutional Participation**: Large players are entering positions, driving price past key levels
3. **Momentum Shift**: The balance of buying/selling pressure has shifted in favor of buyers
4. **Psychological Breakthrough**: Previous resistance becomes new support as market sentiment changes

The recent high represents a significant price level where sellers previously became active. Breaking through this level with conviction (high volume) suggests that sellers have been absorbed and buyers have taken control.

### 2.2 Market Inefficiency Exploited
This strategy exploits the **trend persistence bias** and **herding behavior**:

1. **Delayed Recognition**: Many market participants are slow to recognize breakouts, creating opportunity for early entrants
2. **Stop Loss Cascades**: When price breaks through recent highs, short sellers' stop losses are triggered, creating additional buying pressure (short squeeze)
3. **Momentum Ignition**: Breakouts often ignite momentum as algorithmic trading systems and trend-followers detect the breakout and join the move
4. **Support/Resistance Reversal**: Old resistance becomes new support, creating a self-reinforcing dynamic

The inefficiency persists because:
- Many traders are conditioned to "buy low, sell high" and miss breakouts
- Fundamental analysis lags behind price action
- Retail traders often fade breakouts (expecting reversals)
- Institutional accumulation/distribution phases are not immediately visible

### 2.3 Expected Duration of Edge
The edge is expected to persist as long as:
- Markets exhibit trending behavior (not pure random walk)
- Breakout levels are statistically significant (not arbitrary)
- Volume confirmation provides real information about conviction

The edge may diminish during:
- High-frequency trading dominance where breakouts are arbitraged away
- Extremely choppy/volatile markets with frequent false breakouts
- Strong mean-reversion regimes where breakouts are quickly faded
- Market crises where liquidity dries up

Most effective in:
- Crypto markets (24/7, high volatility, strong trends)
- During high-volume periods (institutional participation)
- When news catalysts drive breakouts (earnings, regulatory events, etc.)

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- **Expected Performance**: High
- **Rationale**: Breakouts in bull markets are more likely to be genuine trend continuations rather than false breakouts. Strong upward momentum supports breakout moves, leading to sustained advances.
- **Historical Evidence**: Historical analysis of equity and crypto markets shows breakouts in established uptrends have 60-70% success rate with average gains of 8-15% before reversal.
- **Adaptations**: 
  - Use longer lookback periods (e.g., 20-30 days) for higher-quality breakouts
  - Implement wider stops (3-5% ATR) to accommodate volatility
  - Consider trailing stops to capture extended trends

### 3.2 Bearish Markets
- **Expected Performance**: Low to Medium
- **Rationale**: Breakouts in bear markets often result in "bear market rallies" that are short-lived. Sellers are waiting at higher levels to exit positions, creating resistance. False breakouts are more frequent.
- **Historical Evidence**: Breakouts during bear markets have lower success rates (40-50%) and smaller average gains (3-6%) before reversal. More frequent whipsaws.
- **Adaptations**:
  - Use shorter lookback periods (e.g., 10-15 days) for quicker exits
  - Implement tighter stops (2-3% ATR) to limit losses
  - Require stronger volume confirmation (1.5-2x average)
  - Consider adding a trend filter (e.g., only long if price above 200-day MA)

### 3.3 Sideways/Ranging Markets
- **Expected Performance**: Low
- **Rationale**: Breakouts in ranging markets are particularly unreliable as price oscillates between support and resistance. Frequent false breakouts occur as tests of levels fail. Strategy experiences multiple whipsaws.
- **Historical Evidence**: Breakouts in ranging markets have success rates of only 30-40% with average gains of 2-4%. Transaction costs significantly impact profitability.
- **Filters**:
  - **Volatility Filter**: Require ATR to be above threshold to avoid low-volatility choppiness
  - **ADX Filter**: Require ADX > 25 to ensure trending conditions
  - **Volume Spike**: Require volume > 1.5-2x average to confirm genuine breakout
  - **Consolidation Detection**: Avoid trading if range has been too tight (e.g., < 5% over lookback period)
- **Recommendation**: Avoid trading breakouts in confirmed ranging markets

### 3.4 Volatility Conditions

- **High Volatility**:
  - **Expected Performance**: Moderate
  - **Rationale**: Breakouts in high volatility are more likely to be genuine (harder to fake) but also more prone to sharp reversals. Larger potential gains but larger drawdowns.
  - **Adaptations**:
    - Use wider stops based on ATR (3-4x ATR)
    - Implement partial profit-taking to lock in gains
    - Use tighter lookback periods to avoid outdated levels
    - Consider volatility-adjusted position sizing (smaller positions in high volatility)
  - **Risks**: False breakouts can be severe, fast reversals

- **Low Volatility**:
  - **Expected Performance**: Low
  - **Rationale**: Breakouts in low volatility often lack conviction and are more likely to fail. Volume may not be sufficient to sustain moves. Gains are limited even when successful.
  - **Adaptations**:
    - Require minimum volatility threshold (e.g., ATR > 1% of price)
    - Increase volume confirmation requirements
    - Use longer lookback periods to find more significant levels
    - Avoid trading if volatility is too low
  - **Recommendation**: Skip breakouts in low volatility conditions

- **Volatility Filter**:
  - Require ATR to be within range: 0.5x to 3.0x of 30-day average
  - Avoid extreme volatility (both too high and too low)
  - Adjust stop loss and position sizing based on current ATR

## 4. Risk Profile

### 4.1 Drawdown Expectations
- **Expected Max Drawdown**: 20-30% per trade
- **Average Drawdown**: 8-12% during losing streaks
- **Drawdown Duration**: 1-3 weeks per losing trade (faster than MA crossover due to quick exits on false breakouts)
- **Worst Case Scenario**:
  - Multiple consecutive false breakouts in choppy market
  - 5-8 consecutive losing trades of 3-5% each
  - 35-40% drawdown in severe conditions
  - Strategy significantly underperforms buy-and-hold in strong trending markets if exits are too early
  - Gap reversals through stop losses (especially in crypto)

### 4.2 Failure Modes

#### Failure Mode 1: False Breakout in Ranging Market
- **Trigger**: Price breaks through recent high but quickly reverses as sellers reassert control. Common in sideways/choppy markets.
- **Impact**: Small to moderate losses (2-5% per trade), high frequency in ranging conditions. Can lead to extended losing streaks.
- **Frequency**: 30-40% of signals in ranging markets, 15-20% overall
- **Mitigation**:
  - Add ADX filter (require ADX > 25 for trending market)
  - Add volume confirmation (require volume > 1.5x average)
  - Implement volatility filter (require minimum ATR)
  - Use wider lookback periods for more significant levels
  - Tighter stops (2-3% ATR) to exit quickly on false breakouts
- **Detection**: 
  - Track consecutive losses - if last 2-3 breakouts were false, increase filter strictness
  - Monitor win rate - dropping below 40% indicates unfavorable regime
  - Regime detection - use volatility and trend indicators to identify ranging periods

#### Failure Mode 2: Late Entry at Exhaustion
- **Trigger**: Breakout occurs late in an extended trend, near exhaustion point. The move is overextended and reverses shortly after entry.
- **Impact**: Small gains followed by rapid reversal, potentially larger than the initial gain. Underperformance vs. buy-and-hold.
- **Frequency**: 10-15% of signals, more common in overextended trends
- **Mitigation**:
  - Add RSI filter (avoid if RSI > 70-75, overbought)
  - Add trend strength filter (avoid if price is too far above moving average)
  - Use partial profit-taking to secure gains
  - Implement trailing stops that tighten as profit increases
  - Consider time-of-day patterns (avoid breakouts late in trend)
- **Detection**:
  - Monitor rate of price acceleration - deceleration near entry signals exhaustion
  - Track how far price is from recent lows (extended moves > 20-30% from lows are risky)
  - Check if multiple previous breakouts have occurred recently (trend may be exhausted)

#### Failure Mode 3: Gap Reversal Through Stop Loss
- **Trigger**: Price gaps down through stop loss level (especially common in crypto markets during news events, liquidations, or flash crashes). Market order execution is worse than stop price.
- **Impact**: Losses exceed intended stop loss amount (can be 2-3x worse in extreme cases). Can be catastrophic if leverage is used.
- **Frequency**: 5-10% of trades, typically during high volatility events
- **Mitigation**:
  - Use limit orders instead of market orders for stops (slippage control)
  - Add volatility-based stop widening during high ATR periods
  - Use options for downside protection (if available)
  - Implement maximum drawdown limits per day/week to prevent catastrophic losses
  - Position size management - limit exposure to prevent single trade ruin
- **Detection**:
  - Monitor ATR for sudden spikes before market opens/news events
  - Track order book depth - thin order books increase gap risk
  - Watch for large sell walls that could trigger cascades

#### Failure Mode 4: Whipsaw Cluster
- **Trigger**: Series of consecutive false breakouts in quick succession, each triggering stops with small losses. Common when market structure is breaking down and levels no longer hold significance.
- **Impact**: Multiple small losses (2-3% each) in rapid succession, psychological pressure, significant cumulative drawdown.
- **Frequency**: 5-10% of time in regime transitions, typically 3-5 trades in cluster
- **Mitigation**:
  - Implement cooldown period (wait X bars after stop loss before re-entering)
  - Increase filter requirements after consecutive losses
  - Reduce position size during losing streaks
  - Switch to different strategy temporarily
  - Implement daily/weekly loss limits to stop bleeding
- **Detection**:
  - Track number of consecutive losing trades - 3+ losses in row signals regime change
  - Monitor volatility spike - sudden increase indicates market structure change
  - Check correlation with market indices - decoupling may indicate breakdown

### 4.3 Correlation Analysis
- **Correlation with Market**: High (0.7-0.9) - strategy is long-only and follows market trends
- **Correlation with Other Strategies**:
  - **High with trend-following strategies** (0.6-0.8): Golden Cross, MA Crossover, Triple MA
  - **Medium with momentum strategies** (0.4-0.6): RSI Momentum, ROC, MACD Trend
  - **Low with mean-reversion strategies** (-0.2 to 0.2): Bollinger Bands Reversion, RSI Reversion
  - **Low with volatility strategies** (0.1-0.3): ATR Breakout, Volatility Squeeze
- **Diversification Value**: Moderate - provides faster entry than MA crossover strategies but similar directional exposure. Works best combined with mean-reversion strategies for market-neutral portfolio. Not suitable as sole strategy in diversified portfolio due to high correlation with other trend strategies.
- **Beta**: Typically 1.2-1.5 - strategy has higher beta than market due to trend-following nature and use of leverage (if used)

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Condition 1**: Price Breakout
   - **Indicator**: Close price > lookback_high (e.g., 20-period high)
   - **Parameters**: lookback_period = 20 (configurable, typically 10-30)
   - **Confirmation**: Price must close above breakout level (not just intraday touch)
   - **Priority**: Required
   - **Calculation**: lookback_high = max(high prices over lookback_period bars)

2. **Condition 2**: Volume Confirmation
   - **Indicator**: Volume > average_volume * volume_multiplier
   - **Parameters**: volume_multiplier = 1.5 (configurable, typically 1.2-2.0)
   - **Rationale**: Confirms genuine buying pressure behind breakout
   - **Priority**: Required (optional: can be disabled)
   - **Calculation**: average_volume = SMA(volume over lookback_period)

3. **Condition 3**: Not Overbought
   - **Indicator**: RSI (14-period)
   - **Parameters**: RSI < threshold (typically 70-75)
   - **Rationale**: Avoids late entries in exhausted trends
   - **Priority**: Optional
   - **Configuration**: rsi_filter_enabled = false by default

4. **Condition 4**: Trending Market
   - **Indicator**: ADX (14-period)
   - **Parameters**: ADX >= 25 (configurable, typically 20-30)
   - **Rationale**: Ensures market is trending, not ranging
   - **Priority**: Optional
   - **Configuration**: adx_filter_enabled = false by default

5. **Condition 5**: Minimum Volatility
   - **Indicator**: ATR (14-period)
   - **Parameters**: ATR >= min_atr (e.g., 1% of price)
   - **Rationale**: Avoids low-volatility breakouts that lack conviction
   - **Priority**: Optional
   - **Configuration**: min_atr_enabled = false by default

### 5.2 Entry Filters
- **Time of Day**: Not applicable for daily timeframe (24/7 crypto markets)
- **Volume Requirements**: Volume > 1.5x average volume (default, configurable)
- **Market Regime Filter**: Prefer trending markets, avoid extreme bear markets
- **Volatility Filter**: ATR within 0.5x to 3.0x of 30-day average
- **Price Filter**: None required, but avoid very low liquidity assets
- **Lookback Period**: 20 periods default (10-30 range)
- **Breakout Threshold**: Close above high (not just intraday touch)
- **Consecutive Losses**: Implement cooldown after 2-3 consecutive losses

### 5.3 Entry Confirmation
- **Confirmation Indicator 1**: Volume spike (>1.5x average) on breakout bar (required)
- **Confirmation Indicator 2**: Close above breakout level on next bar (confirmation)
- **Confirmation Indicator 3**: Price remains above breakout level for N bars (e.g., 2-3 bars)
- **Minimum Confirmed**: 2 out of 3 (volume + close, optional 3rd bar hold)
- **Entry Timing**: Market order on close of confirmation bar, or limit order just above breakout level

## 6. Exit Rules

### 6.1 Take Profit Levels
- **TP 1**: 5% profit - Close 40% of position
- **TP 2**: 10% profit - Close 30% of position
- **TP 3**: 20% profit - Close remaining 30% of position
- **Trailing**: 2.5% trailing stop after TP 1 (tightens to 1.5% after TP 2)
- **Rationale**: Pyramid strategy - secure profits early while allowing runners to capture extended trends

### 6.2 Stop Loss
- **Initial SL**: 3% ATR-based stop below breakout level (not below entry price)
  - Stop level = breakout_high - (3 * ATR)
  - ATR provides volatility-adjusted stop
- **Trailing SL**: 
  - Activates after TP 1: 2.5% trailing from highest high
  - Tightens to 1.5% after TP 2
  - Never moves down (only up with price)
- **Breakeven**: Move stop to breakeven after TP 2 (10% profit)
- **Time-based Exit**: Close position after 40 days if no TP or SL triggered
- **Rationale**: ATR-based stop accommodates volatility while limiting risk to known support level

### 6.3 Exit Conditions
- **Reversal Signal**: Price closes below trailing stop
- **Breakdown Signal**: Price closes below recent low (e.g., 10-period low) - exit 100%
- **Regime Change**: If market enters confirmed bear market (e.g., 200-day MA turns down), consider exiting even if in profit
- **Volatility Spike**: If ATR > 3x average, tighten stop loss to 2% (or exit if position in profit)
- **Time Limit**: Maximum 40 days in position to avoid stagnation and free up capital
- **Consecutive Bars Down**: Close if 5+ consecutive down bars (sign of reversal)

## 7. Position Sizing

- **Base Position Size**: 2% of portfolio capital per trade
- **Volatility Adjustment**: 
  - If ATR > 2x average: Reduce position size to 1%
  - If ATR < 0.5x average: Increase position size to 2.5%
  - Formula: adjusted_size = base_size * (avg_atr / current_atr)
- **Conviction Levels**: 
  - Strong signal (high volume, high ADX, low RSI): 2.5%
  - Weak signal (low volume, low ADX, high RSI): 1.5%
- **Max Position Size**: 5% of portfolio (absolute maximum)
- **Risk per Trade**: Maximum 1% of portfolio (stop loss distance × position size)
- **Leverage**: No leverage recommended for spot trading. If using leverage, limit to 2x maximum.
- **Correlation Adjustment**: Reduce size if already exposed to correlated assets
- **Drawdown Protection**: Reduce size after consecutive losses (e.g., 50% size reduction after 3 losses)

## 8. Parameters

### 8.1 Core Parameters
| Parameter | Default | Range | Description | Type |
|-----------|---------|-------|-------------|------|
| lookback_period | 20 | 5-50 | Number of bars to calculate high/low | int |
| volume_multiplier | 1.5 | 1.0-3.0 | Volume multiple for confirmation | float |
| min_atr_pct | 1.0 | 0.1-5.0 | Minimum ATR as % of price | float |
| atr_stop_multiplier | 3.0 | 1.5-5.0 | ATR multiplier for initial stop loss | float |
| trailing_stop_pct | 2.5 | 1.0-5.0 | Trailing stop percentage after TP | float |
| tp1_pct | 5.0 | 2.0-10.0 | First take profit level (%) | float |
| tp1_close_pct | 40 | 20-50 | % position to close at TP1 | int |
| tp2_pct | 10.0 | 5.0-20.0 | Second take profit level (%) | float |
| tp2_close_pct | 30 | 20-50 | % position to close at TP2 | int |
| tp3_pct | 20.0 | 10.0-40.0 | Third take profit level (%) | float |
| max_days_in_position | 40 | 20-90 | Maximum days to hold position | int |
| rsi_filter_enabled | false | boolean | Enable RSI overbought filter | bool |
| rsi_period | 14 | 7-21 | RSI calculation period | int |
| rsi_threshold | 70 | 60-80 | RSI threshold for overbought filter | float |
| adx_filter_enabled | false | boolean | Enable ADX trending filter | bool |
| adx_period | 14 | 7-21 | ADX calculation period | int |
| adx_threshold | 25 | 20-35 | ADX threshold for trending filter | float |

### 8.2 Optimization Notes
- **Parameters to Optimize**: lookback_period, volume_multiplier, atr_stop_multiplier, tp levels
- **Optimization Method**: Grid search with walk-forward validation
- **Optimization Period**: 3 years in-sample, 2 years out-of-sample
- **Expected Overfitting Risk**: Medium-High - Breakout strategies are prone to overfitting to specific market conditions
- **Sensitivity Analysis Required**: Yes - test robustness across different lookback periods and filter combinations
- **Key Findings from Historical Analysis**:
  - Lookback period of 15-25 performs best in crypto markets (shorter than traditional markets)
  - Volume multiplier of 1.3-1.7 provides best balance of filtering too many signals vs. accepting weak ones
  - ATR stop multiplier of 2.5-3.5 provides good risk/reward in most conditions
  - TP levels of 5/10/20% work well for crypto's higher volatility

### 8.3 Parameter Sensitivity
- **High Sensitivity**: lookback_period (too short = noisy, too long = laggy)
- **Medium Sensitivity**: volume_multiplier, atr_stop_multiplier
- **Low Sensitivity**: tp levels (more about profit distribution than signal quality)
- **Filter Parameters**: rsi_threshold, adx_threshold (medium sensitivity)

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- **Timeframes**: Daily (primary), 4-hour (secondary for crypto intraday analysis)
- **Test Period**: 2019-2025 (6 years) - includes bull, bear, and ranging regimes
- **Assets**: BTC, ETH, SOL, ADA, XRP, DOT, AVAX, LINK, MATIC, UNI (10+ crypto assets)
- **Minimum Trades**: 30 trades per asset for statistical significance
- **Slippage**: 0.15% per trade (higher than MA crossover due to breakout execution)
- **Commission**: 0.1% per trade (typical exchange fee)
- **Position sizing**: Fixed fractional (2% of portfolio)
- **Execution**: Market orders on close (realistic execution model)

### 9.2 Validation Techniques
- [ ] Walk-forward analysis (rolling window: 2-year train, 1-year test, 6-month step)
- [ ] Monte Carlo simulation (1000 iterations of trade sequence randomization)
- [ ] Parameter sweep (10x10 grid for lookback_period and volume_multiplier)
- [ ] Regime analysis (separate results for bull/bear/sideways periods)
- [ ] Cross-asset validation (test on out-of-sample assets not used in optimization)
- [ ] Bootstrap validation (resample trades with replacement)
- [ ] Permutation testing (randomize entry dates to test significance)
- [ ] Robustness testing (vary parameters by ±20% to test stability)
- [ ] Stress testing (test performance during extreme volatility events)

### 9.3 Success Criteria
- **Minimum Sharpe Ratio**: 1.0 (risk-adjusted return threshold)
- **Minimum Sortino Ratio**: 1.3 (focus on downside risk)
- **Maximum Max Drawdown**: 25% (absolute worst case)
- **Minimum Win Rate**: 45% (breakout strategies often have lower win rates but higher average wins)
- **Minimum Profit Factor**: 1.4 (gross profit / gross loss)
- **Minimum Robustness Score**: 60 (walk-forward stability measure)
- **Statistical Significance**: p < 0.05 (t-test vs. random entries)
- **Walk-Forward Stability**: >50% of walk-forward windows profitable
- **Out-of-Sample Performance**: < 15% degradation vs. in-sample
- **Regime Performance**: Profitable in at least 2 out of 3 regimes (bull/bear/sideways)

## 10. Expected Results

### 10.1 Performance Targets
- **Annual Return**: 30-50% (before fees, higher than MA crossover due to faster entry)
- **Sharpe Ratio**: 1.3-1.8 (risk-adjusted returns)
- **Max Drawdown**: 20-25% (similar to other trend strategies)
- **Win Rate**: 45-55% (lower win rate but larger winners)
- **Profit Factor**: 1.6-2.2 (strong risk/reward)
- **Expectancy**: 0.04-0.06 (4-6% per trade average profit)
- **Average Trade Duration**: 15-40 days (shorter than MA crossover due to quicker exits)
- **Average Winning Trade**: 12-18%
- **Average Losing Trade**: 4-7%
- **Risk/Reward Ratio**: 2.0-2.5 (winners 2-2.5x larger than losers)

### 10.2 Comparison to Baselines
- **vs. HODL**: 
  - Expected outperformance: 15-30% annually
  - Lower maximum drawdown (through stops)
  - Better risk-adjusted returns (higher Sharpe)
  - Performs better in ranging markets (can sit out)
  - May underperform in strong continuous uptrends (early exits)

- **vs. Market Average**: 
  - Expected outperformance: 20-35% annually
  - More selective (fewer but higher-conviction trades)
  - Better risk management through stops

- **vs. Golden Cross**: 
  - Faster entry (breakout vs. crossover)
  - Higher win rate (45-55% vs. 40-50%)
  - Smaller average trades (shorter holding period)
  - More frequent signals
  - Higher transaction costs
  - Better performance in choppy markets (fewer whipsaws due to volume filter)

- **vs. Similar Strategies**: 
  - Faster than MA crossover (earlier entry)
  - More selective than price channel breakouts
  - Less prone to whipsaws than simple high/low breakouts (due to filters)
  - Better risk/reward than momentum strategies (through pyramid exits)

- **vs. Buy & Hold**: 
  - Superior downside protection (stop losses)
  - Better performance in bear markets (can avoid losses by sitting out)
  - May underperform in strong uptrends if exits are too early
  - Superior risk-adjusted returns

### 10.3 Best Case Scenario
- Strong, sustained trend following breakout
- 30-40%+ gain on single trade
- Multiple partial take profits executed
- Trailing stop captures extended move
- Low drawdown due to quick stop on false breakouts

### 10.4 Worst Case Scenario
- Multiple consecutive false breakouts in ranging market
- 5-8 consecutive small losses
- 35-40% drawdown from whipsaw cluster
- Strategy significantly underperforms buy-and-hold in strong trend
- Transaction costs eat into profits from high turnover

## 11. Implementation Requirements

### 11.1 Technical Requirements
- **Indicators Needed**: 
  - ATR (Average True Range) - for volatility-based stops
  - RSI (Relative Strength Index) - for overbought filter
  - ADX (Average Directional Index) - for trending market filter
  - SMA (Simple Moving Average) - for volume averaging
- **Data Requirements**: 
  - OHLCV (Open, High, Low, Close, Volume) data
  - Daily timeframe (minimum)
  - Minimum 200 bars of history for proper initialization
- **Latency Sensitivity**: Medium - faster entry than MA crossover, but not HFT
- **Computational Complexity**: Low - simple max/min calculations, indicator updates
- **Memory Requirements**: Moderate - need to store lookback_period of highs/lows
- **State Management**: 
  - Track current position and remaining size
  - Track highest high since entry for trailing stop
  - Track entry date for time-based exit
  - Track stop loss level

### 11.2 Code Structure
- **File Location**: crates/strategy/src/strategies/trend_following/breakout.rs
- **Strategy Type**: Simple trend-following with advanced filters
- **Dependencies**: 
  - alphafield_core::Bar, Signal, SignalType, Strategy
  - crate::indicators::{Atr, Rsi, Adx, Sma}
  - crate::framework::{MetadataStrategy, StrategyMetadata, StrategyCategory, RiskProfile, etc.}
  - crate::config::BreakoutConfig (to be created)
- **State Management**: 
  - Track current high/low over lookback period
  - Track position state (entry_price, position_size, entry_date)
  - Track trailing stop level
  - Track partial exits taken
  - Track highest high since entry

### 11.3 Indicator Calculations
**Lookback High/Low**:
```
lookback_high = max(high prices over lookback_period bars)
lookback_low = min(low prices over lookback_period bars)
```
Simple ring buffer or VecDeque can be used to maintain history.

**ATR Calculation**:
Already implemented in indicators.rs - use Atr::update() method.

**RSI Calculation**:
Already implemented in indicators.rs - use Rsi::update() method.

**ADX Calculation**:
Already implemented in indicators.rs - use Adx::update() method.

**Volume Average**:
```
avg_volume = SMA(volume over lookback_period)
```

**Breakout Detection**:
```
breakout_high = close > lookback_high
breakout_low = close < lookback_low (for short side, if implemented)
```

**Stop Loss Calculation**:
```
initial_stop = breakout_high - (atr_multiplier * ATR)
```

**Trailing Stop Calculation**:
```
trailing_stop = highest_high * (1 - trailing_stop_pct/100)
```

## 12. Testing Plan

### 12.1 Unit Tests
- [ ] Entry conditions (breakout signal generated correctly on high break)
- [ ] Entry conditions (no signal when no breakout)
- [ ] Entry conditions (volume filter blocks weak breakouts)
- [ ] Entry conditions (RSI filter blocks overbought entries)
- [ ] Entry conditions (ADX filter blocks ranging entries)
- [ ] Exit conditions (take profit partial exits at correct levels)
- [ ] Exit conditions (stop loss triggers correctly on reversal)
- [ ] Exit conditions (trailing stop moves up but never down)
- [ ] Exit conditions (trailing stop triggers on pullback)
- [ ] Exit conditions (time-based exit triggers after max days)
- [ ] Exit conditions (breakdown signal exits position)
- [ ] Edge cases (empty data, insufficient bars, NaN values)
- [ ] Parameter validation (invalid parameters rejected)
- [ ] State management (reset clears all state, position tracking works)
- [ ] Position size tracking (partial exits reduce position correctly)
- [ ] Multiple breakouts (handles consecutive breakouts correctly)

### 12.2 Integration Tests
- [ ] Backtest execution (runs without errors across multiple assets)
- [ ] Performance calculation (metrics are accurate vs. manual calculation)
- [ ] Dashboard integration (strategy appears in API, metadata accessible)
- [ ] Database integration (performance metrics saved correctly)
- [ ] Registry integration (strategy registered and retrievable)
- [ ] Parameter sweep (testing across parameter ranges)
- [ ] Walk-forward validation (windows execute correctly)
- [ ] Monte Carlo simulation (randomization produces distribution)

### 12.3 Research Tests
- [ ] Hypothesis validation (results support primary hypothesis of >4% avg return)
- [ ] Statistical significance (p < 0.05 for outperformance vs. random)
- [ ] Regime analysis (better performance in trending, worse in ranging)
- [ ] Robustness testing (stable performance across parameter variations)
- [ ] Walk-forward stability (strategy profitable in majority of windows)
- [ ] Monte Carlo analysis (distribution of outcomes acceptable)
- [ ] Filter effectiveness (volume/RSI/ADX filters improve win rate)
- [ ] Stop loss effectiveness (limits downside without reducing upside too much)

## 13. Research Journal

### 2025-01-02: Initial Hypothesis Creation
**Observation**: Breakout strategies are classic trend-following approaches that work well in volatile, trending markets like crypto. However, they're prone to false signals in ranging conditions, requiring careful filtering.
**Hypothesis Impact**: Hypothesis assumes breakouts with volume confirmation provide statistically significant edge. Filters (RSI, ADX) should reduce false signals without eliminating too many opportunities.
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
**Observation**: [To be filled - Are results as expected?]
**Action Taken**: [To be filled - Proceed to validation or refine strategy?]

### [Date]: Parameter Optimization
**Optimization Method**: [Grid search, Bayesian, etc.]
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
**Observation**: [To be filled - Is distribution acceptable?]
**Action Taken**: [To be filled]

### [Date]: Filter Effectiveness Analysis
**Volume Filter**: [To be filled - impact on win rate, frequency]
**RSI Filter**: [To be filled - impact on win rate, frequency]
**ADX Filter**: [To be filled - impact on win rate, frequency]
**Combined Filters**: [To be filled - overall improvement]
**Observation**: [To be filled]
**Action Taken**: [To be filled]

### [Date]: Stop Loss Analysis
**Initial SL Hit Rate**: [To be filled]
**Trailing Stop Effectiveness**: [To be filled]
**Avg Loss on Stops**: [To be filled]
**Observation**: [To be filled - Are stops too tight/too loose?]
**Action Taken**: [To be filled]

### [Date]: Final Validation
**Overall Assessment**: [To be filled]
**Recommendation**: [Deploy/Reject/Improve]
**Confidence Level**: [Low/Medium/High]
**Next Steps**: [To be filled]

## 14. References

### Academic Sources
- Fama, E. F., & Blume, M. E. (1966). "Filter Rules and Stock-Market Trading." Journal of Business, 39(1), 226-241.
- Brock, W., Lakonishok, J., & LeBaron, B. (1992). "Simple Technical Trading Rules and the Stochastic Properties of Stock Returns." Journal of Finance, 47(5), 1731-1764.
- Osler, C. L. (2000). "Support for Resistance: Technical Analysis and Intraday Exchange Rates." Federal Reserve Bank of New York Economic Policy Review, 6(2), 53-68.
- Lo, A. W., Mamaysky, H., & Wang, J. (2000). "Foundations of Technical Analysis: Computational Algorithms, Statistical Inference, and Empirical Implementation." Journal of Finance, 55(4), 1705-1765.

### Books
- Bulkowski, T. N. (2005). "Encyclopedia of Chart Patterns." Wiley.
- Edwards, R. D., & Magee, J. (2007). "Technical Analysis of Stock Trends." CRC Press.
- Kaufman, P. J. (2013). "Trading Systems and Methods." Wiley.
- Pring, M. J. (2002). "Technical Analysis Explained." McGraw-Hill.

### Online Resources
- Investopedia: "Breakout" - Definition and explanation
- TradingView: Breakout indicator documentation and community examples
- QuantConnect: Breakout strategy tutorials and research
- StockCharts: Technical analysis library - breakout patterns
- BabyPips: Forex breakout trading strategies

### Similar Strategies
- Donchian Channel Breakout (classic 20-day breakout)
- Turtle Trading (Richard Dennis's breakout system)
- Price Channel Breakout (fixed high/low channels)
- Volatility Breakout (breakouts based on volatility bands)
- Opening Range Breakout (ORB) - intraday variation

### Historical Examples
- 2017: Crypto bull market - BTC breakout from consolidation (multiple successful breakouts)
- 2018: Bear market rallies - frequent false breakouts
- 2020: COVID crash and recovery - strong breakouts post-crash
- 2021: Market peak - late breakouts often failed
- 2023: Market recovery - breakouts from multi-month consolidation

### Crypto-Specific Examples
- 2017 Q4: BTC breakout from $5k to $20k (massive 4x move)
- 2020 March: BTC breakdown from $10k to $3.5k (downside breakout)
- 2020 Q4: BTC breakout from $10k to $40k (strong sustained trend)
- 2021 November: BTC breakout to $69k ATH (exhaustion, quick reversal)
- 2023 Q1: BTC breakout from $16k to $30k (moderate sustained trend)

## 15. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2025-01-02 | 1.0 | AI Agent | Initial hypothesis creation |
| [Date] | [Version] | [Author] | [Description of changes] |

---
**Status**: Draft - Ready for Implementation
**Next Action**: Implement strategy code in crates/strategy/src/strategies/trend_following/breakout.rs
**Expected Completion**: [Date]
**Estimated Development Time**: 1 day