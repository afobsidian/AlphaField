# MACD Trend Strategy Hypothesis

## Metadata
- **Name**: MACD Trend
- **Category**: TrendFollowing
- **Sub-Type**: MACD Crossover + Histogram Confirmation
- **Author**: AI Agent
- **Date**: 2026-01-11
- **Status**: Proposed
- **Code Location**: `crates/strategy/src/strategies/trend_following/macd_trend.rs`

---

## 1. Hypothesis Statement

**Primary Hypothesis**:  
When the MACD line crosses above the signal line *and* the MACD histogram is above a configurable threshold (default `0.0`), the market has sufficient positive momentum to justify entering a long position. Exiting on bearish MACD crossover (and/or histogram turning negative) reduces exposure to trend reversals and cuts whipsaws versus a pure crossover approach.

**Null Hypothesis**:  
MACD crossovers with histogram confirmation do not produce an edge over random entry/exit decisions and do not improve risk-adjusted returns compared to simple baselines (e.g., HODL) once fees and slippage are included.

---

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
MACD (Moving Average Convergence Divergence) captures the relationship between two EMAs (fast and slow). The difference between these EMAs reflects acceleration/deceleration in price movement.

- **MACD line**: `EMA_fast - EMA_slow`
- **Signal line**: EMA of the MACD line
- **Histogram**: `MACD line - Signal line`

A bullish crossover (MACD crossing above the signal) indicates a shift from negative/weak momentum toward positive/strong momentum. Histogram confirmation is intended to reduce false positives by requiring that the crossover is supported by measurable momentum.

### 2.2 Market Inefficiency Exploited
This strategy targets:
- **Trend persistence / momentum continuation**: once momentum shifts positive, price often continues in the same direction for a period.
- **Behavioral anchoring and delayed reactions**: participants react slowly to a momentum regime change, creating drift after crossover.
- **Whipsaw reduction via confirmation**: histogram thresholding can block marginal crossovers that tend to revert quickly.

### 2.3 Expected Duration of Edge
The expected edge duration is **short-to-medium term** (days to weeks), consistent with MACD’s smoothing and common parameterization.

---

## 3. Market Regime Analysis

### 3.1 Bullish Markets
Expected to perform well:
- Catches continuation legs after momentum resumes.
- Histogram confirmation should filter weaker pullback crossovers.

### 3.2 Bearish Markets
Spot-only constraint means:
- Strategy should aim to **avoid entries** in prolonged downtrends via filters (optional) and tighter stop logic.
- Performance is expected to be weaker than in bullish markets, potentially flat-to-negative.

### 3.3 Sideways / Ranging Markets
Most challenging regime:
- Crossovers occur frequently with limited follow-through.
- Histogram or additional trend filters (e.g., SMA trend filter) are expected to reduce excessive churn.

### 3.4 Volatility Conditions
- **High volatility**: crossovers may occur with higher noise; ATR-based stop logic can reduce catastrophic losses, but churn can rise.
- **Low volatility**: fewer signals; trades may be longer and fewer, potentially improving net after costs.

---

## 4. Risk Profile

### 4.1 Drawdown Expectations
- Expected max drawdown: **moderate** (targeted < 25% in phase success metrics), dependent on stop configuration and market conditions.

### 4.2 Failure Modes
1. **Range-bound chop**: repeated crossovers leading to repeated small losses.
2. **Late entries**: MACD lags by design; fast reversals can invalidate trades shortly after entry.
3. **Gap/fast drop risk**: sudden sell-offs can jump past stop logic (backtest slippage assumptions matter).

### 4.3 Correlation Analysis
- Likely **high correlation** with broader market beta in bullish markets.
- Correlation expected to reduce somewhat if filters reduce entries during non-trending periods.

---

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. **Bullish crossover**: MACD line crosses above signal line.
2. **Histogram confirmation**: histogram > `histogram_threshold` (default `0.0`).
3. Optional trend filter (if enabled): price above a long-term SMA (e.g., 200 SMA).

### 5.2 Entry Filters
Optional filters (configuration-dependent):
- RSI filter (avoid overbought entries).
- ADX filter (require trending conditions).
- Volume confirmation (avoid low-quality breakouts).

### 5.3 Entry Confirmation
- Histogram above threshold is the primary confirmation within this strategy definition.

---

## 6. Exit Rules

### 6.1 Take Profit Levels
- Base configuration uses **fixed take-profit %** and/or partial exits (if implemented in strategy config).
- Take-profit is expected to capture trend continuation bursts.

### 6.2 Stop Loss
- ATR-based stop is recommended (per plan: “Stop loss: 3% ATR-based”).
- Trailing stop logic may be used to lock in gains as trend progresses.

### 6.3 Exit Conditions
1. **Bearish crossover**: MACD crosses below signal line.
2. **Histogram turns negative**: histogram < 0 (or below configured threshold for exits).
3. **Stop loss / trailing stop triggers**.

---

## 7. Position Sizing
- Spot-only (no leverage / no shorts).
- Default sizing: 1.0 unit (engine-level risk manager may scale).
- Optional volatility scaling can be applied at execution/risk layer, not strategy layer.

---

## 8. Parameters

### 8.1 Core Parameters
- `fast_period`: 12 (default)
- `slow_period`: 26 (default)
- `signal_period`: 9 (default)
- `histogram_threshold`: 0.0 (default)

### 8.2 Optimization Notes
- Optimize parameters using walk-forward validation to reduce overfitting.
- Consider a small positive histogram threshold to reduce chop entries.
- Consider a higher slow period (e.g., 34–50) for slower regimes, but expect fewer trades.

---

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- Include realistic fees, slippage, and latency consistent with the engine’s simulation settings.
- Evaluate on multiple assets (BTC, ETH, etc.) and include delisted assets where applicable to avoid survivorship bias.

### 9.2 Validation Techniques
- Walk-forward analysis (required in project rules).
- Monte Carlo perturbation of fills/returns (required).
- Sensitivity analysis for periods and histogram threshold.

### 9.3 Success Criteria
- Meets or exceeds phase-level targets where feasible:
  - Sharpe > 1.0 (aspirational, averaged across strategies)
  - Max drawdown < 25% (aspirational)
- Demonstrates robustness across reasonable parameter perturbations.
- Exhibits reduced whipsaw frequency versus crossover-only baseline.

---

## 10. Expected Results

### 10.1 Performance Targets
- Positive risk-adjusted return in trending regimes.
- Reduced churn in sideways regimes relative to crossover-only MACD.

### 10.2 Comparison to Baselines
- Compare against:
  - `HoldBaseline`
  - `GoldenCrossStrategy` (trend baseline)
  - “MACD crossover without histogram” variant (if available) as an ablation.

---

## 11. Implementation Requirements

### 11.1 Technical Requirements
- Must use existing `Macd` indicator implementation from `crate::indicators`.
- Must remain spot-only (no shorting/leverage).
- Must return signals consistent with `alphafield_core::Signal`.

### 11.2 Code Structure
- File: `crates/strategy/src/strategies/trend_following/macd_trend.rs`
- Module export: `crates/strategy/src/strategies/trend_following/mod.rs`
- Must implement `Strategy` + `MetadataStrategy`.

### 11.3 Indicator Calculations
- Confirm MACD update yields `(macd_line, signal_line, histogram)` and use histogram thresholding.
- Ensure warmup behavior is handled (indicator returns `None` until fully initialized).

---

## 12. Testing Plan

### 12.1 Unit Tests
- MACD warmup behavior (no signals until MACD values available).
- Entry only on bullish crossover + histogram > threshold.
- Exit on bearish crossover and/or histogram < 0.
- Stop loss / trailing stop trigger behavior.

### 12.2 Integration Tests
- Registry registration and API listing for the strategy.
- Metadata includes correct hypothesis path: `hypotheses/trend_following/macd_trend.md`.

### 12.3 Research Tests
- Walk-forward validation on multiple assets and time ranges.
- Monte Carlo robustness evaluation.

---

## 13. Research Journal
- **2026-01-11**: Hypothesis drafted to align with Phase 12.2 requirements.

---

## 14. References
- Perry J. Kaufman, *Trading Systems and Methods* (MACD discussion and momentum framing)
- Standard MACD documentation and practitioner references (EMA 12/26, signal 9)

---

## 15. Revision History
- **2026-01-11**: Initial version.