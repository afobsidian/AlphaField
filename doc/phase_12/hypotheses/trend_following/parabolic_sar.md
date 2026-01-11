# Parabolic SAR Strategy Hypothesis

## Metadata
- **Name**: Parabolic SAR
- **Category**: TrendFollowing
- **Sub-Type**: Parabolic SAR Trailing Stop
- **Author**: AI Agent
- **Date**: 2026-01-11
- **Status**: Proposed
- **Code Location**: `crates/strategy/src/strategies/trend_following/parabolic_sar.rs`

---

## 1. Hypothesis Statement

**Primary Hypothesis**:  
Parabolic SAR (PSAR) provides an effective, volatility-adaptive trailing stop for trend following. Entering long when price crosses above the SAR (indicating SAR has moved below price) and exiting when price crosses below the SAR will capture medium-term uptrends while limiting drawdowns through systematic stop behavior. Adding a simple trend filter (e.g., requiring price > 50 SMA) reduces whipsaws in sideways regimes by only participating when a broader uptrend is present.

**Null Hypothesis**:  
PSAR-based entries and exits do not provide an exploitable edge once fees and slippage are accounted for. The strategy suffers from frequent reversals and whipsaws, producing returns comparable to (or worse than) simple baselines such as buy-and-hold.

---

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Parabolic SAR is designed to:
- **Track trends** by placing a stop level that “accelerates” in the direction of the move.
- **Exit quickly on reversals** by flipping above/below price when the trend breaks.
- **Reduce discretionary decision-making** by enforcing objective exits.

In practice, PSAR acts like a **dynamic trailing stop**: the stop tightens as the trend persists, which aims to lock in gains and reduce exposure when momentum fades.

### 2.2 Market Inefficiency Exploited
This strategy attempts to exploit:
- **Trend persistence**: once a trend begins, returns often exhibit continuation for some horizon.
- **Delayed positioning**: market participants often scale into trends slowly; systematic trailing stops can remain exposed longer than discretionary traders.
- **Behavioral anchoring**: resistance/support shifts and momentum reinforcement can lead to “drift” after trend confirmation.

### 2.3 Expected Duration of Edge
If present, the edge is expected to appear in **sustained trending conditions** (days to weeks). It degrades when:
- volatility or noise increases,
- the market is range-bound,
- trend direction changes frequently.

---

## 3. Market Regime Analysis

### 3.1 Bullish Markets
- Expected to perform well when trends are persistent and reversals are relatively infrequent.
- SAR trailing behavior should allow the position to remain open through minor pullbacks (depending on step settings).

### 3.2 Bearish Markets
- Project constraint: **spot-only** (no shorting/leverage).
- Strategy should generally remain flat more often in prolonged downtrends, especially with the SMA trend filter enabled.
- Risk: bear-market rallies can still generate entries; exits should occur on SAR flip.

### 3.3 Sideways / Ranging Markets
- Highest whipsaw risk.
- The SMA trend filter is intended to reduce exposure during low-conviction regimes by only allowing entries in a broader uptrend context.

### 3.4 Volatility Conditions
- High volatility may cause frequent SAR flips (tightening and reversals).
- Lower volatility can produce fewer flips but may also cause slower trend capture depending on the `step` parameter.

---

## 4. Risk Profile

### 4.1 Drawdown Expectations
- Drawdowns are expected to be **moderate** and primarily driven by:
  - whipsaws,
  - large adverse moves between bars (gap/slippage in backtests),
  - entry near trend exhaustion.
- Risk controls are primarily embedded in the SAR trailing stop itself (systematic exit), plus the optional trend filter to reduce low-quality entries.

### 4.2 Failure Modes

#### Failure Mode 1: Whipsaw / chop regime
- SAR flips frequently, forcing repeated entries/exits with small losses.
- Mitigation: trend filter (price > SMA), parameter tuning (`step`, `max_step`), potentially additional volatility or ADX filters in future.

#### Failure Mode 2: Trend exhaustion / late entry
- Entry occurs after a large move; SAR tightens quickly and exit occurs near the top, reducing expectancy.
- Mitigation: trend filter and conservative SAR parameters; consider requiring a stronger confirmation in future (multi-bar confirm).

#### Failure Mode 3: Sudden drop / slippage
- Large down moves can exceed expected loss due to bar-based evaluation and simulated execution assumptions.
- Mitigation: realistic slippage/fees; consider execution-layer circuit breakers.

### 4.3 Correlation Analysis
- Trend-following strategies in crypto often correlate with market beta, particularly in bull regimes.
- With a trend filter, time-in-market may be reduced in non-trending phases, potentially lowering overall correlation compared to always-long baselines.

---

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Price crosses above SAR** (SAR is below price; bullish flip or bullish condition).
2. **Trend filter (optional)**: price > 50 SMA (default enabled).

### 5.2 Entry Filters
- Trend filter: `price > SMA(trend_sma_period)`.
- Additional filters (not required by Phase 12.2 spec but possible future work):
  - ADX threshold to confirm trending regime,
  - volume confirmation,
  - ATR-based volatility gating.

### 5.3 Entry Confirmation
The primary confirmation is the SAR regime itself (SAR below price). The trend SMA is an additional confirmation against noise.

---

## 6. Exit Rules

### 6.1 Take Profit Levels
No fixed take-profit is required for this strategy; profit is primarily harvested by SAR trailing behavior. (If added later, ensure the strategy remains trend-following and does not clip winners too aggressively.)

### 6.2 Stop Loss
PSAR is used as the trailing stop:
- In an uptrend, SAR rises toward price.
- Exit occurs when price crosses below SAR.

### 6.3 Exit Conditions
1. **Price crosses below SAR** (SAR above price; bearish flip).
2. Strategy exits fully (spot-only; no short reversal entry).

---

## 7. Position Sizing
- Default sizing is constant and spot-only.
- Any volatility scaling or portfolio constraints should be handled by the engine’s execution/risk layer.

---

## 8. Parameters

### 8.1 Core Parameters
- `step`: default 0.02  
- `max_step`: default 0.2  
- `trend_filter`: default true  
- `trend_sma_period`: default 50

### 8.2 Optimization Notes
- Smaller `step` tends to widen the stop early, reducing premature exits but increasing drawdown.
- Larger `step` tightens faster, improving drawdown control but increasing whipsaws.
- `max_step` caps tightening late in trends; too high increases whipsaws near trend ends.
- Use walk-forward validation to tune parameters and avoid overfitting.

---

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- Spot-only, no leverage.
- Include realistic:
  - fees,
  - slippage,
  - latency (if simulated).
- Test across multiple assets and time periods to avoid survivorship bias where possible.

### 9.2 Validation Techniques
- **Walk-forward analysis**: optimize parameters on rolling training windows, evaluate out-of-sample.
- **Monte Carlo simulation**: resample/perturb returns or trade outcomes to estimate robustness.
- **Sensitivity analysis**: vary `step` and `max_step` in a neighborhood to identify brittle tuning.

### 9.3 Success Criteria
- Demonstrates positive expectancy in trending regimes.
- Exhibits lower drawdown than always-long baseline during choppy periods due to reduced exposure (especially with trend filter).
- Robustness: performance should not depend on a single narrow parameter point.

---

## 10. Expected Results

### 10.1 Performance Targets
- Best performance in **Bull/Trending** regimes.
- Reduced time-in-market and improved drawdown control versus HODL during sideways periods (with trend filter enabled).

### 10.2 Comparison to Baselines
Compare against:
- `HoldBaseline` (HODL),
- `GoldenCrossStrategy` (MA-based trend baseline),
- Breakout-based trend strategy (if available) for regime complementarity.

---

## 11. Implementation Requirements

### 11.1 Technical Requirements
- Must remain spot-only (no shorts).
- Deterministic, bar-driven logic (no lookahead).
- Configuration validation must ensure:
  - `step > 0`,
  - `max_step >= step`,
  - `trend_sma_period > 0` if trend filter enabled.

### 11.2 Code Structure
- Strategy: `crates/strategy/src/strategies/trend_following/parabolic_sar.rs`
- Metadata must reference:
  - `hypotheses/trend_following/parabolic_sar.md`

### 11.3 Indicator Calculations
Use standard PSAR update rules:
- `SAR_next = SAR_prev + AF * (EP - SAR_prev)`
- Reverse direction when price crosses SAR.
- EP is highest high (uptrend) / lowest low (downtrend).
- AF increases by `step` on new EP, capped at `max_step`.

---

## 12. Testing Plan

### 12.1 Unit Tests
- Config validation rejects invalid parameters.
- SAR initializes after sufficient bars.
- Entry signal occurs when price crosses above SAR (and trend filter condition satisfied if enabled).
- Exit signal occurs when price crosses below SAR.
- Reset clears state.

### 12.2 Integration Tests
- Strategy is exported in `trend_following::mod`.
- Strategy can be registered in the Strategy Registry.
- API can list and retrieve metadata for the strategy.

### 12.3 Research Tests
- Run WFA and Monte Carlo; compare against GoldenCross and HODL baselines on BTC/ETH and at least one additional asset.

---

## 13. Research Journal
- **2026-01-11**: Hypothesis drafted to match Phase 12.2 requirements and strategy implementation intent.

---

## 14. References
- Wilder, J. Welles. *New Concepts in Technical Trading Systems* (Parabolic SAR)
- General trend-following literature and trailing-stop systems

---

## 15. Revision History
- **2026-01-11**: Initial hypothesis document created.