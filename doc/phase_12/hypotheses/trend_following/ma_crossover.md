# MA Crossover Strategy Hypothesis

## Metadata
- **Name**: MA Crossover Strategy
- **Category**: TrendFollowing
- **Sub-Type**: Moving Average Crossover
- **Author**: AI Agent
- **Date**: 2026-01-11
- **Status**: Proposed
- **Code Location**: crates/strategy/src/strategies/trend_following/ma_crossover.rs

---

## 1. Hypothesis Statement

**Primary Hypothesis**:  
When a shorter-term moving average (fast MA) crosses above a longer-term moving average (slow MA), the market is transitioning into a sustained uptrend. Entering long on this crossover—optionally requiring minimum MA separation and confirmation filters (RSI, ADX, volume) and managing risk via ATR-based trailing stops—will produce positive risk-adjusted returns over medium horizons (days to weeks), outperforming a simple buy-and-hold baseline in choppy regimes by reducing drawdowns and avoiding low-conviction entries.

**Null Hypothesis**:  
MA crossovers do not provide an exploitable edge after transaction costs and slippage. Signals occur too late (lag), and/or produce frequent whipsaws in sideways markets, resulting in performance comparable to or worse than buy-and-hold.

---

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Moving averages act as **trend estimators** by smoothing short-term noise to reveal directional bias. A fast MA crossing above a slow MA suggests:
1. **Recent price strength dominates** longer-term mean behavior.
2. **Momentum** is shifting from neutral/negative to positive.
3. **Trend followers** begin accumulating, reinforcing direction.

Crossover signals are imperfect but can be improved with contextual filters that:
- reduce low-quality entries (e.g., RSI overbought avoidance),
- ensure trend presence (e.g., ADX confirmation),
- validate participation (e.g., volume confirmation),
- control risk dynamically (ATR-based stops).

### 2.2 Market Inefficiency Exploited
This strategy targets **trend persistence** and **reaction lag**:
- Market participants and systematic flows often respond with delay to trend formation.
- Breakouts and trend starts can attract incremental demand (momentum funds, quant systems), creating follow-through.

The inefficiency is most present in **trending regimes** where autocorrelation in returns is stronger and noise-to-signal ratio is lower.

### 2.3 Expected Duration of Edge
- Expected edge duration: **several days to multiple weeks** (typical for medium-term trend-following signals).
- The edge degrades during:
  - tight ranges,
  - rapidly mean-reverting markets,
  - high-volatility chop.

---

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- Expected to perform well as crossovers align with rising markets.
- Risk: late entries after large moves; mitigated by MA separation + volume filters.

### 3.2 Bearish Markets
- Spot-only (no shorting). Strategy should most often stay flat.
- Risk: false bullish crossovers during bear rallies; mitigated by ADX + RSI filters and tight risk controls.

### 3.3 Sideways/Ranging Markets
- Highest whipsaw risk. Filters (ADX threshold, MA separation) are intended to reduce exposure.
- Expect reduced trade frequency; performance may be flat to slightly negative absent strong filtering.

### 3.4 Volatility Conditions
- **High volatility** can inflate whipsaws; ATR trailing stops help adapt.
- **Low volatility** may lead to slow trends; EMA-based fast MA can improve responsiveness.

---

## 4. Risk Profile

### 4.1 Drawdown Expectations
- Moderate drawdowns can occur during regime transitions and chop.
- ATR-based stops should cap tail risk per-trade; overall DD depends on trade frequency and filter strictness.

### 4.2 Failure Modes
#### Failure Mode 1: Whipsaw in range-bound markets
- Frequent crossovers with no trend continuation.
- Mitigation: ADX filter, minimum MA separation, volume confirmation.

#### Failure Mode 2: Lag / late entry after large move
- Crossovers occur after a significant portion of the move has happened.
- Mitigation: EMA option, parameter tuning, and avoiding extreme RSI.

### 4.3 Correlation Analysis
- Trend strategies can become correlated during strong market trends (all “risk-on”).
- Expect moderate correlation with other trend-following systems; diversification across strategy families/regimes is recommended.

---

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. Fast MA crosses above Slow MA (bullish crossover).
2. (Optional) Minimum MA separation threshold satisfied (reduces marginal crossovers).
3. (Optional) RSI filter passes (avoid overbought entries).
4. (Optional) ADX filter passes (ensure trending regime).
5. (Optional) Volume confirmation passes (participation confirmation).

### 5.2 Entry Filters (Configurable)
- **MA separation**: require the fast MA to exceed slow MA by a minimum percentage.
- **RSI**: require RSI < threshold to avoid chasing extended moves.
- **ADX**: require ADX >= threshold to confirm trend strength.
- **Volume filter**: require current volume >= average_volume * multiplier.

### 5.3 Entry Confirmation
- Entry is taken on the first bar where conditions become true.
- (Optional) Confirmation can be strengthened by requiring multiple consecutive bars above crossover (not required by current implementation unless explicitly added).

---

## 6. Exit Rules

### 6.1 Take Profit Levels
- Partial take profit may be applied at configured profit thresholds.
- Rationale: harvest gains while still allowing trend continuation with a remaining position.

### 6.2 Stop Loss
- Base stop can be expressed as a percent loss from entry and/or ATR-based trailing stop.
- ATR trailing stop adapts to changing volatility and tends to reduce “stop too tight” issues during expansion.

### 6.3 Exit Conditions
1. Fast MA crosses below Slow MA (bearish crossover) → exit remaining position.
2. Trailing stop hit (ATR-based or percent-based) → exit.
3. Stop-loss threshold breached → exit.

---

## 7. Position Sizing
- Spot-only: long exposure only.
- Default sizing is fixed (e.g., 1.0) with optional scaling:
  - reduce position size in high volatility (ATR scaling),
  - reduce exposure after partial profit-taking.

---

## 8. Parameters

### 8.1 Core Parameters
- `fast_period` (usize): short MA period
- `slow_period` (usize): long MA period
- `ma_type` (string): `"SMA"` or `"EMA"`
- `min_separation_pct` (f64): minimum distance between MAs
- Risk parameters:
  - `take_profit` (f64, %)
  - `stop_loss` (f64, %)
  - `atr_period` (usize)
  - `atr_multiplier` (f64)

### 8.2 Optimization Notes
- Expect sensitivity to MA periods and filters:
  - shorter periods → more trades, more whipsaw
  - longer periods → fewer trades, more lag
- Filters reduce trade count; tune to target an acceptable signal frequency and DD.

---

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- Use unbiased backtesting assumptions:
  - include realistic fees and slippage,
  - no lookahead bias (signals from historical bars only),
  - spot-only (no shorts/leverage).

### 9.2 Validation Techniques
- Walk-forward analysis (WFA):
  - optimize parameters on training windows,
  - evaluate on out-of-sample windows.
- Monte Carlo / bootstrapping:
  - permute trade sequence or resample returns to estimate robustness.
- Sensitivity analysis:
  - vary MA periods and thresholds to detect overfitting.

### 9.3 Success Criteria
- Positive expected return net of costs in trending regimes.
- Sharpe > baseline and Max Drawdown within acceptable bounds for the strategy class.
- Robustness: parameter neighborhoods should perform similarly (avoid razor-thin optimum).

---

## 10. Expected Results

### 10.1 Performance Targets
- Best performance expected during **Bull/Trending** regimes.
- Lower volatility drawdowns than naive buy-and-hold during choppy regimes (depending on filters).

### 10.2 Comparison to Baselines
- Compare to:
  - `HoldBaseline` (HODL),
  - `MarketAverageBaseline` (if applicable),
  - existing Golden Cross (special case of MA crossover with longer periods).

---

## 11. Implementation Requirements

### 11.1 Technical Requirements
- Must remain spot-only (no short signals).
- Must validate configuration inputs.
- Must be thread safe (strategy + indicators compatible with Send/Sync behavior at the trait boundaries).

### 11.2 Code Structure
- Strategy implementation: `crates/strategy/src/strategies/trend_following/ma_crossover.rs`
- Config type: `MACrossoverConfig` in `crates/strategy/src/config.rs`
- Strategy metadata must reference this hypothesis path:
  - `hypotheses/trend_following/ma_crossover.md`

### 11.3 Indicator Calculations
- Moving averages: SMA/EMA
- Optional: RSI, ADX, ATR, volume SMA/mean
- Use rolling windows with `VecDeque` as needed.

---

## 12. Testing Plan

### 12.1 Unit Tests
- Configuration validation:
  - rejects invalid periods, negative thresholds, invalid MA types.
- Signal generation:
  - constructs crossover scenario and verifies Buy signal.
  - constructs cross-down scenario and verifies Sell signal.
- Risk logic:
  - take profit triggers partial exit (if enabled).
  - trailing stop triggers exit (ATR-based).

### 12.2 Integration Tests
- Ensure strategy is exported under trend-following module and can be registered in the Strategy Registry.
- Ensure API listing includes the strategy once integrated.

### 12.3 Research Tests
- Compare SMA vs EMA variants across assets and regimes.
- Verify filter impact (trade count vs DD vs Sharpe) across multiple parameter sets.

---

## 13. Research Journal
- **2026-01-11**: Hypothesis drafted to match implementation structure and Phase 12 validation requirements.

---

## 14. References
- Kaufman, Perry J. *Trading Systems and Methods* (trend-following concepts and MA usage)
- Murphy, John J. *Technical Analysis of the Financial Markets* (moving averages and trend interpretation)

---

## 15. Revision History
- **2026-01-11**: Initial hypothesis drafted.