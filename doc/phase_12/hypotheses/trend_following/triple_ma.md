# Triple MA Strategy Hypothesis

## Metadata
- **Name**: Triple MA
- **Category**: TrendFollowing
- **Sub-Type**: Triple Moving Average Alignment
- **Author**: AI Agent
- **Date**: 2026-01-11
- **Status**: Proposed
- **Code Location**: `crates/strategy/src/strategies/trend_following/triple_ma.rs`

---

## 1. Hypothesis Statement

**Primary Hypothesis**:  
When three moving averages are aligned in bullish order (**Fast MA > Medium MA > Slow MA**) and the alignment forms after previously not being aligned, the market is in a strong, sustained uptrend. Entering long at alignment formation and exiting when the alignment breaks (Fast MA < Medium MA) will outperform naïve long exposure in trending regimes, with improved drawdown control via ATR-based risk management.

**Null Hypothesis**:  
Triple moving average alignment does not provide predictive information about future returns. Any observed performance is due to chance, and the strategy does not outperform simple baselines after fees/slippage, especially in sideways regimes where whipsaws dominate.

---

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Moving averages are lagging filters that summarize price action over different horizons:
- **Fast MA** captures short-term momentum.
- **Medium MA** captures intermediate trend.
- **Slow MA** represents the primary trend direction.

A triple alignment requires agreement across horizons. This reduces false positives compared to a simple two-MA crossover, because it demands broader trend confirmation.

### 2.2 Market Inefficiency Exploited
This strategy targets:
- **Trend persistence / continuation**: once multiple horizons align, trend-followers systematically add exposure.
- **Delayed reaction**: participants often underreact to gradual trend emergence; full consensus takes time to manifest across horizons.
- **Signal confirmation advantage**: requiring alignment filters out early, noisier signals that frequently reverse.

### 2.3 Expected Duration of Edge
The edge is expected to be most pronounced during **multi-week to multi-month** trending phases. It is expected to degrade in:
- low-volatility ranges,
- choppy mean-reverting markets,
- regime transitions with rapid reversals.

---

## 3. Market Regime Analysis

### 3.1 Bullish Markets
Expected strong performance when a bullish market transitions into a trending phase. Triple alignment prevents early entry but improves signal quality.

### 3.2 Bearish Markets
Spot-only constraint implies no shorting. Strategy is expected to:
- produce fewer or no entries,
- remain flat more often,
- primarily act as a trend participation filter rather than a hedge.

### 3.3 Sideways/Ranging Markets
Expected underperformance due to whipsaws:
- alignment forms and breaks frequently,
- ATR stops may trigger, increasing churn.

Mitigation: optional **ADX filter** (trend strength) and **RSI overbought filter** (avoid late entries).

### 3.4 Volatility Conditions
- **High volatility**: may increase false alignment breaks; ATR-based trailing/stop logic should reduce tail losses.
- **Low volatility**: fewer opportunities; performance depends on capturing rare but persistent trends.

---

## 4. Risk Profile

### 4.1 Drawdown Expectations
- Moderate drawdowns are possible during regime transitions.
- ATR-based stops are expected to cap worst-case losses per trade, but gaps/slippage can increase realized losses.

### 4.2 Failure Modes
**Failure Mode 1: Range whipsaw**  
Frequent small losses as MAs cross repeatedly in sideways markets.

**Failure Mode 2: Late entry / trend exhaustion**  
Triple alignment may form late; entry occurs near local maxima, causing stop-outs or poor expectancy.

### 4.3 Correlation Analysis
Trend-following strategies tend to be positively correlated with broad market direction in spot-only. This strategy is expected to have:
- **High correlation** in strong bull runs,
- **Lower correlation** when the system stays flat due to lack of alignment.

---

## 5. Entry Rules

### 5.1 Long Entry Conditions
A long entry is generated when:
1. **Bullish alignment forms**: `Fast MA > Medium MA > Slow MA`
2. Alignment transition: previously not aligned, now aligned (formation event)
3. (Optional) Confirmation filters pass:
   - RSI filter: `RSI < threshold` (avoid overbought entries)
   - ADX filter: `ADX >= threshold` (ensure trend strength)
   - Volume filter: `volume >= avg_volume * multiplier`

### 5.2 Entry Filters
- **ADX filter**: reduces entries in non-trending markets.
- **RSI filter**: avoids entering after extended acceleration.
- **Volume filter**: requires participation confirmation.
- **MA-type**: SMA/EMA choice impacts responsiveness and lag.

### 5.3 Entry Confirmation
The alignment itself is the primary confirmation. Additional confirmation is optional via filters above.

---

## 6. Exit Rules

### 6.1 Take Profit Levels
- Partial take profit may trigger based on configured thresholds (if implemented).
- Otherwise profit-taking primarily occurs through trailing behavior and alignment break exit.

### 6.2 Stop Loss
- Primary protective stop is expected to be **ATR-based**, optionally combined with trailing components.
- Stop loss objective: limit adverse excursion during trend failure or sudden reversal.

### 6.3 Exit Conditions
Exit (LongExit) is triggered when:
1. **Alignment breaks**: `Fast MA < Medium MA` (trend weakening)
2. **Trailing/ATR stop triggers**
3. Optional time-based exit / max-days in position (if configured)

---

## 7. Position Sizing
Default sizing is full position (spot-only, no leverage). Optional rules may reduce size on partial exits. Strategy does not short.

---

## 8. Parameters

### 8.1 Core Parameters (defaults)
- `fast_period`: 5
- `medium_period`: 15
- `slow_period`: 30
- `ma_type`: SMA or EMA
- `take_profit`: 5.0 (%)
- `stop_loss`: 3.0 (%)
- Optional filters:
  - ADX: period ~14, threshold ~25
  - RSI: period ~14, threshold ~70
  - Volume multiplier: ~1.2
- ATR-based stop:
  - `atr_period`: 14
  - `atr_multiplier`: ~2.0

### 8.2 Optimization Notes
- Periods should be optimized using time-series aware methods:
  - walk-forward analysis
  - parameter sensitivity tests
- Avoid overfitting: constrain search space and validate on multiple assets/regimes.

---

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- Use realistic fees/slippage.
- Use spot-only constraints.
- Test across multiple assets and market regimes.

### 9.2 Validation Techniques
- Walk-forward validation (rolling windows).
- Monte Carlo resampling (trade order / return perturbations where applicable).
- Regime-sliced reporting (Bull/Bear/Sideways; Trending/Ranging).

### 9.3 Success Criteria
- Positive expectancy in Trending regimes.
- Max drawdown within acceptable limits relative to risk profile.
- Robustness across reasonable parameter ranges (no single-point tuning).

---

## 10. Expected Results

### 10.1 Performance Targets
- Designed to meet phase targets when combined across strategies:
  - Sharpe > 1.0 (in favorable regimes)
  - Max DD < 25% (portfolio-level target)

### 10.2 Comparison to Baselines
- Should outperform HODL on risk-adjusted basis in trending windows.
- May underperform in prolonged flat/sideways markets.

---

## 11. Implementation Requirements

### 11.1 Technical Requirements
- Must implement `Strategy` and `MetadataStrategy`.
- Must validate configuration via `StrategyConfig::validate`.
- Must remain thread-safe (`Send + Sync` constraints respected by indicator usage).

### 11.2 Code Structure
- File: `crates/strategy/src/strategies/trend_following/triple_ma.rs`
- Module export: `crates/strategy/src/strategies/trend_following/mod.rs`

### 11.3 Indicator Calculations
- Moving averages updated per bar close.
- Entry should only occur once per alignment transition to avoid repeated signals.

---

## 12. Testing Plan

### 12.1 Unit Tests
- Verify configuration validation rejects invalid periods.
- Verify buy signal occurs on bullish alignment formation.
- Verify sell signal occurs on alignment break.
- Verify reset clears state.

### 12.2 Integration Tests
- Strategy registry includes Triple MA in TrendFollowing category.
- API lists strategy and returns metadata.

### 12.3 Research Tests
- Parameter sensitivity on periods and filter toggles.
- Regime dependence analysis.

---

## 13. Research Journal
- **2026-01-11**: Hypothesis document created to align implementation with Phase 12.2 requirements.

---

## 14. References
- Kaufman, Perry J. *Trading Systems and Methods* (trend-following and MA systems concepts)
- General technical analysis literature on multi-horizon trend confirmation

---

## 15. Revision History
- **2026-01-11**: Initial draft