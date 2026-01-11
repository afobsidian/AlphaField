# Adaptive Moving Average (KAMA) Strategy Hypothesis

## Metadata
- **Name**: Adaptive MA Strategy
- **Category**: TrendFollowing
- **Sub-Type**: Adaptive Moving Average (KAMA)
- **Author**: AI Agent
- **Date**: 2025-01-02
- **Status**: Proposed
- **Code Location**: crates/strategy/src/strategies/trend_following/adaptive_ma.rs

---

## 1. Hypothesis Statement

**Primary Hypothesis**:  
Kaufman’s Adaptive Moving Average (KAMA) can improve trend-following performance versus fixed-parameter moving averages by adapting its smoothing to market conditions. In trending regimes, the strategy should enter earlier and capture more of the move; in ranging regimes, the strategy should reduce false signals by smoothing more aggressively. With disciplined exits (ATR-based trailing stops and optional take-profit/stop-loss), the strategy should achieve improved risk-adjusted returns compared to a basic MA crossover baseline.

**Null Hypothesis**:  
KAMA adaptivity does not provide a meaningful edge. The strategy’s returns are indistinguishable from fixed moving average approaches after fees/slippage, and the adaptive smoothing does not materially reduce whipsaws in ranging markets.

---

## 2. Market Logic & Rationale

### 2.1 Economic/Technical Rationale
Markets alternate between regimes:
- **Trending**: directional persistence, where faster reaction is beneficial.
- **Ranging/Choppy**: mean-reverting/sideways conditions, where slower signals reduce noise.

KAMA is designed to vary its effective smoothing based on how “efficiently” price moves. When price progresses in a more directional way (high efficiency), KAMA becomes more responsive. When price oscillates without progress (low efficiency), KAMA becomes smoother.

This adaptivity aims to balance two competing goals:
1. **Responsiveness** in high-quality trends (reduce lag).
2. **Noise filtering** in choppy markets (reduce whipsaws).

### 2.2 Market Inefficiency Exploited
This strategy attempts to exploit **trend persistence** and **slow re-pricing**:
- Participants often adjust to new trends gradually due to risk constraints and confirmation bias.
- Trend-following signals can capture sustained moves when risk is well-managed.

KAMA’s adaptivity attempts to reduce entry/exit lag during strong trends while avoiding overreaction to noise.

### 2.3 Expected Duration of Edge
If present, the edge should persist as long as markets continue exhibiting regime shifts between trending and ranging behavior and as long as trend persistence remains a structural feature of the traded crypto markets.

---

## 3. Market Regime Analysis

### 3.1 Bullish Markets
Expected to perform well during sustained uptrends. KAMA should tighten smoothing (faster reaction), allowing earlier entry and less lag.

### 3.2 Bearish Markets
This project enforces **spot-only, no shorting**, so the strategy should primarily:
- Avoid prolonged exposure when the market is down.
- Exit promptly when the trend breaks (price crosses below KAMA and/or trailing stop triggers).

### 3.3 Sideways/Ranging Markets
KAMA should smooth more (slower reaction), which may reduce the number of false entries compared to a fixed fast MA. However, ranging markets can still cause whipsaws and small repeated losses.

### 3.4 Volatility Conditions
- **High volatility**: ATR-based trailing stops help scale exits to market conditions; without them, fixed stops can be too tight or too loose.
- **Low volatility**: Strategy may trade less frequently; moves may be smaller and more sensitive to fees/slippage.

---

## 4. Risk Profile

### 4.1 Drawdown Expectations
Moderate. Trend-following strategies can experience clustered losses in sideways regimes and larger drawdowns during sharp reversals.

### 4.2 Failure Modes
#### Failure Mode 1: Persistent chop / whipsaw regime
Price crosses KAMA repeatedly without follow-through. Result: many small losses and underperformance.

#### Failure Mode 2: Fast crash / gap-like movement
In very sharp drops, trailing stop/KAMA cross may exit late vs instantaneous price move. Result: losses exceed expectations.

### 4.3 Correlation Analysis
In crypto, many assets are highly correlated during risk-on/risk-off moves. This strategy’s performance may be highly correlated with the broader market, especially in major drawdowns. Portfolio-level risk management is required when running across many correlated assets.

---

## 5. Entry Rules

### 5.1 Long Entry Conditions
1. Price crosses above KAMA (bullish cross).
2. Optional filters (if enabled in config):
   - RSI filter: avoid entries when RSI indicates overbought conditions.
   - ADX filter: require trend strength.
   - Volume filter: require volume confirmation relative to recent average.

### 5.2 Entry Filters
- **RSI filter** (optional): reduces buying into short-term overextended moves.
- **ADX filter** (optional): attempts to ensure the market is trending.
- **Volume filter** (optional): attempts to confirm breakouts with participation.

### 5.3 Entry Confirmation
The KAMA adaptive smoothing is the primary confirmation mechanism. Filters provide additional confirmation when configured.

---

## 6. Exit Rules

### 6.1 Take Profit Levels
Optional partial/full take profit based on configured thresholds. This can lock in gains in volatile markets but may reduce total trend capture.

### 6.2 Stop Loss
- Configured stop loss (percentage-based) acts as a hard cap on losses.
- ATR-based trailing stop (if enabled) provides volatility-adjusted exit management.

### 6.3 Exit Conditions
1. Price crosses below KAMA.
2. ATR-based trailing stop level triggers.
3. Stop loss triggers.
4. (Optional) partial exit triggers on take-profit rules.

---

## 7. Position Sizing
Default position sizing is constant (1.0 in the current implementation), subject to engine-level risk management. In future, volatility-scaled sizing could be applied at the execution/risk layer rather than strategy logic.

---

## 8. Parameters

### 8.1 Core Parameters
- `fast_period`: KAMA fast smoothing equivalent period (trend responsiveness).
- `slow_period`: KAMA slow smoothing equivalent period (range smoothing).
- `price_period`: lookback for price change / efficiency ratio calculation.
- `take_profit`: profit-taking threshold (%).
- `stop_loss`: stop loss threshold (%).
- Optional:
  - `trailing_stop` / ATR-based trailing stop multiplier and period
  - RSI: period + threshold
  - ADX: period + threshold
  - Volume minimum multiplier

### 8.2 Optimization Notes
- Optimize with **walk-forward** and **Monte Carlo**; avoid overfitting.
- Key sensitivities:
  - Too fast `fast_period` can overtrade in chop.
  - Too slow `slow_period` can increase lag and reduce trend capture.
  - Tight stops increase churn; wide stops increase drawdowns.

---

## 9. Validation Requirements

### 9.1 Backtesting Configuration
- Include realistic fees and slippage.
- Use multiple crypto assets (high cap + mid cap) and include delisted assets if available (survivorship bias prevention).
- Test multiple timeframes if supported (e.g., 1h, 4h, 1d).

### 9.2 Validation Techniques
- Walk-forward analysis (required by project rules).
- Monte Carlo simulation (required by project rules).
- Parameter sensitivity checks for the core KAMA parameters and stop settings.

### 9.3 Success Criteria
- Outperform baseline trend-following strategies in risk-adjusted terms (e.g., Sharpe) or reduce drawdown meaningfully at similar return.
- Demonstrate regime-dependent advantage (better behavior in trend vs chop relative to fixed MA).
- Stable performance under small parameter perturbations (robustness).

---

## 10. Expected Results

### 10.1 Performance Targets
- Sharpe ratio improvement vs comparable fixed MA strategy in trending regimes.
- Max drawdown controlled below strategy library targets when combined with engine-level risk management.

### 10.2 Comparison to Baselines
Compare against:
- HODL baseline.
- Simple MA crossover baseline (same timeframe/assets).
- Golden Cross (trend-following baseline in library).

---

## 11. Implementation Requirements

### 11.1 Technical Requirements
- Use `VecDeque` for rolling price windows.
- Maintain thread safety (`Send + Sync`) for indicator trait usage.
- Validate configuration on construction.

### 11.2 Code Structure
- Strategy implemented at: `crates/strategy/src/strategies/trend_following/adaptive_ma.rs`
- Metadata provided via `MetadataStrategy` trait with:
  - `hypothesis_path`: `hypotheses/trend_following/adaptive_ma.md`

### 11.3 Indicator Calculations
- KAMA efficiency ratio and smoothing constant must be computed correctly.
- ATR and optional RSI/ADX must be updated consistently on each bar.

---

## 12. Testing Plan

### 12.1 Unit Tests
- Config validation tests for invalid parameters.
- Signal generation tests for expected entry/exit behavior under synthetic trending vs ranging sequences.
- Indicator readiness behavior (warmup) verified.

### 12.2 Integration Tests
- Registry and API listing should include this strategy once integrated into module exports/registry initialization.

### 12.3 Research Tests
- Walk-forward + Monte Carlo stability checks.
- Parameter sensitivity sweeps (fast/slow/price periods and trailing stop settings).

---

## 13. Research Journal
- **2025-01-02**: Initial implementation and unit test coverage added.
- **TBD**: Walk-forward validation results.
- **TBD**: Monte Carlo validation results.
- **TBD**: Parameter sweep and robustness assessment.
- **TBD**: Final decision and deployment recommendation.

---

## 14. References
- Perry J. Kaufman, *Trading Systems and Methods* (KAMA concept)
- General trend-following literature and MA filtering concepts

---

## 15. Revision History
- **2025-01-02**: Initial hypothesis document created.