# Trading Modes Guide

## Overview

AlphaField supports two trading modes designed to accommodate different strategy types and risk preferences:

- **Spot Mode** (Default): Long-only trading with cash-based settlement
- **Margin Mode** (Opt-in): Long and short trading with margin-based settlement

This dual-mode architecture ensures **backward compatibility** while enabling advanced trading strategies like market neutral, pairs trading, and mean reversion.

### Key Differences

| Aspect | Spot Mode | Margin Mode |
|--------|-----------|-------------|
| **Position Types** | Long only | Long and Short |
| **Settlement** | Cash-based | Margin-based |
| **Borrowing** | Not available | Required for shorts |
| **Default Behavior** | ✅ Yes | ❌ Requires configuration |
| **Risk Profile** | Lower (loss limited to capital) | Higher (unlimited short losses) |
| **Complexity** | Simple | Advanced |
| **Ideal For** | Trend following, momentum | Market neutral, pairs trading, arbitrage |

### When to Use Each Mode

**Use Spot Mode when:**
- You're new to algorithmic trading
- You prefer simple, long-only strategies
- Risk tolerance is moderate
- You want to avoid leverage and borrowing costs

**Use Margin Mode when:**
- You need both long and short positions
- Implementing market neutral strategies
- Doing pairs trading or statistical arbitrage
- Want to profit from declining markets
- Have sufficient experience managing short position risks

---

## Spot Mode

Spot Mode is the **default** trading mode in AlphaField. It restricts trading to long-only positions with cash-based settlement, making it ideal for trend-following and momentum strategies.

### Characteristics

**Position Behavior:**
- **Long Only**: Can only hold positive quantities
- **Buy Signal**: Opens or adds to long position
- **Sell Signal**: Closes or reduces long position
- **Flat Position**: Can only enter via Buy signal
- **No Shorting**: Sell signals when flat are rejected

**Risk Profile:**
- **Loss Limit**: Maximum loss equals initial capital
- **No Margin Requirements**: 100% cash position sizing
- **No Borrowing Costs**: No interest or funding payments
- **Simpler Risk Management**: One-directional risk exposure

**Settlement:**
- Cash-based (no leverage)
- Immediate trade settlement
- No margin calls
- No liquidation risk

### Configuration

Spot Mode is the default, so **no explicit configuration is needed**. However, you can explicitly set it:

```rust
use alphafield_core::TradingMode;

// Method 1: Explicit (default value)
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95)
    .with_trading_mode(TradingMode::Spot);

// Method 2: Implicit (same result)
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95);
```

**Portfolio Configuration:**

```rust
let portfolio = Portfolio::new(10000.0)
    .with_trading_mode(TradingMode::Spot);
```

**BacktestEngine Configuration:**

```rust
let engine = BacktestEngine::new(10000.0, 0.001)
    .with_trading_mode(TradingMode::Spot);
```

### Best Practices

1. **Start Simple**: Begin with Spot Mode to understand strategy behavior
2. **Monitor Cash Usage**: Track available cash vs position value
3. **Use Position Limits**: Set `MaxPositionValue` risk check to control exposure
4. **Focus on Trend**: Spot Mode works best with directional (trend-following) strategies
5. **Avoid Overfitting**: Simple strategies with Spot Mode often generalize better
6. **Test Thoroughly**: Validate with walk-forward and Monte Carlo before deployment

### Example Usage

**Simple Trend Following Strategy:**

```rust
use alphafield_core::{Strategy, Signal, TradingMode};
use alphafield_strategy::indicators::SMA;
use alphafield_backtest::adapter::StrategyAdapter;

struct SimpleTrend {
    short_sma: SMA,
    long_sma: SMA,
}

impl Strategy for SimpleTrend {
    fn name(&self) -> &str {
        "Simple Trend"
    }
    
    fn on_bar(&mut self, bar: &Bar) -> Option<Signal> {
        self.short_sma.update(bar.close);
        self.long_sma.update(bar.close);
        
        if self.short_sma.value() > self.long_sma.value() {
            Some(Signal::new_buy(bar.close, 1.0))
        } else {
            Some(Signal::new_sell(bar.close, 1.0))
        }
    }
}

// In Spot mode, this will only go long
// When flat, Sell signals are rejected
let strategy = SimpleTrend::new();
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95)
    .with_trading_mode(TradingMode::Spot);
```

---

## Margin Mode

Margin Mode is an **opt-in** feature that enables both long and short positions with margin-based settlement. This is essential for market neutral strategies, pairs trading, and other advanced approaches.

### Characteristics

**Position Behavior:**
- **Long and Short**: Can hold positive or negative quantities
- **Buy Signal**: Opens long or closes short
- **Sell Signal**: Opens short or closes long
- **Flat Position**: Can enter via Buy (long) or Sell (short)
- **Position Reversal**: Can go directly from long to short

**Risk Profile:**
- **Unlimited Loss**: Short positions have theoretically unlimited losses
- **Margin Requirements**: Must maintain collateral for short positions
- **Borrowing Costs**: Pay interest/funding for borrowed assets
- **Short Squeeze Risk**: Rapid price increases can force liquidation
- **Complex Risk Management**: Requires multiple risk checks

**Settlement:**
- Margin-based (leverage available)
- Borrowing required for short positions
- Subject to margin calls
- Potential liquidation on short squeezes

### Configuration

Margin Mode requires **explicit configuration** across multiple components:

```rust
use alphafield_core::TradingMode;

// Strategy Adapter
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95)
    .with_trading_mode(TradingMode::Margin);

// Portfolio
let portfolio = Portfolio::new(10000.0)
    .with_trading_mode(TradingMode::Margin);

// Backtest Engine
let engine = BacktestEngine::new(10000.0, 0.001)
    .with_trading_mode(TradingMode::Margin);
```

**Dashboard API Configuration:**

```json
{
  "symbol": "BTCUSDT",
  "strategy_name": "MeanReversion",
  "strategy_params": {
    "rsi_period": 14,
    "oversold": 30,
    "overbought": 70
  },
  "start_date": "2023-01-01",
  "end_date": "2023-12-31",
  "trading_mode": "Margin",  // Enable Margin mode
  "fee_rate": 0.001,
  "slippage_pct": 0.0005
}
```

### Risk Management

Margin Mode requires **additional risk checks** to protect against short position risks:

**1. Max Short Position Limit:**

```rust
use alphafield_execution::risk_checks::MaxShortPosition;

// Limit short position to 50% of portfolio value
let max_short = MaxShortPosition::new(0.5);
risk_manager.add_check(max_short);
```

**2. NoShorts Check (Conditional):**

```rust
use alphafield_execution::risk_checks::NoShorts;

// In Margin mode, NoShorts is typically disabled
// But you can enable it selectively:
let no_shorts = NoShorts::new(TradingMode::Spot);
risk_manager.add_check(no_shorts);
```

**3. Short Squeeze Detection:**

```rust
use alphafield_execution::risk_checks::ShortSqueezeDetection;

// Detect when price moves rapidly against shorts
let squeeze_detect = ShortSqueezeDetection::new(0.1, 3);  // 10% jump in 3 bars
risk_manager.add_check(squeeze_detect);
```

**4. Margin Requirement Monitoring:**

```rust
use alphafield_execution::risk_checks::MarginRequirement;

// Ensure sufficient collateral for shorts
let margin_req = MarginRequirement::new(1.5);  // 150% margin requirement
risk_manager.add_check(margin_req);
```

### Best Practices

1. **Start Small**: Begin with small position sizes to understand short behavior
2. **Use Position Limits**: Always set `MaxShortPosition` to control exposure
3. **Monitor Margin**: Track margin usage and maintain buffer above requirements
4. **Borrowing Costs**: Factor in borrowing costs when calculating profitability
5. **Diversify**: Spread risk across multiple assets to reduce concentration
6. **Test Extensively**: Use walk-forward and Monte Carlo to validate strategy
7. **Watch for Squeezes**: Implement short squeeze detection and auto-close
8. **Paper Trade First**: Test Margin mode strategies with paper trading before live
9. **Regular Rebalancing**: Monitor and adjust positions to stay within risk limits
10. **Keep Cash Buffer**: Maintain extra cash for margin calls and short covering

### Example Usage

**Mean Reversion with Shorts:**

```rust
use alphafield_core::{Strategy, Signal, TradingMode};
use alphafield_strategy::indicators::RSI;
use alphafield_backtest::adapter::StrategyAdapter;

struct MeanReversion {
    rsi: RSI,
    oversold: f64,
    overbought: f64,
}

impl Strategy for MeanReversion {
    fn name(&self) -> &str {
        "Mean Reversion"
    }
    
    fn on_bar(&mut self, bar: &Bar) -> Option<Signal> {
        self.rsi.update(bar.close);
        
        let rsi_val = self.rsi.value();
        
        if rsi_val < self.oversold {
            // RSI oversold - price likely to rebound
            Some(Signal::new_buy(bar.close, 1.0))
        } else if rsi_val > self.overbought {
            // RSI overbought - price likely to decline
            Some(Signal::new_sell(bar.close, 1.0))
        } else {
            Some(Signal::new_hold())
        }
    }
}

// In Margin mode, this will go long on oversold, short on overbought
let strategy = MeanReversion::new(14, 30.0, 70.0);
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95)
    .with_trading_mode(TradingMode::Margin);
```

---

## Short Position Risks

Short positions introduce unique risks that must be carefully managed in Margin Mode.

### 1. Unlimited Loss Potential

**Risk:** Unlike long positions (where loss is limited to initial capital), short positions have theoretically unlimited losses because asset prices can rise arbitrarily high.

**Mitigation Strategies:**
- **Position Limits**: Set maximum short position as percentage of portfolio
- **Stop Losses**: Always use stop-loss orders on short positions
- **Diversification**: Spread shorts across uncorrelated assets
- **Volatility Scaling**: Reduce position size during high volatility

**Example:**

```rust
use alphafield_execution::risk_checks::MaxShortPosition;

// Limit short exposure to 30% of portfolio value
let max_short = MaxShortPosition::new(0.3);
risk_manager.add_check(max_short);
```

### 2. Short Squeeze Risk

**Risk:** A short squeeze occurs when rapid buying pressure forces short sellers to cover their positions, driving prices even higher. This can cause massive losses in a short time.

**Mitigation Strategies:**
- **Squeeze Detection**: Monitor for rapid price increases
- **Auto-Close**: Automatically close shorts when squeeze detected
- **Liquidity Avoidance**: Avoid shorting assets with low liquidity
- **News Monitoring**: Watch for positive catalysts that could trigger squeezes

**Example:**

```rust
use alphafield_execution::risk_checks::ShortSqueezeDetection;

// Detect 10% price increase in 3 consecutive bars
let squeeze_detect = ShortSqueezeDetection::new(0.1, 3);
risk_manager.add_check(squeeze_detect);
```

### 3. Margin Requirements

**Risk:** Short positions require borrowing assets, which requires maintaining sufficient collateral (margin). If the position moves against you, you may receive a margin call or face liquidation.

**Mitigation Strategies:**
- **Margin Buffer**: Maintain extra margin above minimum requirements
- **Monitoring**: Track margin usage in real-time
- **Size Control**: Keep positions small relative to available margin
- **Collateral Management**: Maintain adequate cash reserves

**Example:**

```rust
use alphafield_execution::risk_checks::MarginRequirement;

// Require 150% of short position value as collateral
let margin_req = MarginRequirement::new(1.5);
risk_manager.add_check(margin_req);
```

### 4. Borrow Costs

**Risk:** When you short an asset, you're borrowing it from someone else and must pay interest (borrow rate) on the borrowed amount. These costs can significantly erode profits.

**Mitigation Strategies:**
- **Rate Monitoring**: Track borrow rates and avoid high-cost assets
- **Short Duration**: Keep short positions shorter when borrow costs are high
- **Cost Calculation**: Factor borrowing costs into strategy profitability
- **Rate Hedging**: Consider hedging borrow costs when possible

**Example in Trade Tracking:**

```rust
impl Trade {
    pub fn total_pnl(&self) -> f64 {
        let gross_pnl = self.entry_price - self.exit_price;
        let costs = self.borrow_cost + self.funding_cost;
        gross_pnl - costs
    }
}
```

---

## Example Strategies

### Mean Reversion with Shorts

**Hypothesis:** Asset prices tend to revert to their mean value over time. When price is significantly above mean (overbought), short it; when below mean (oversold), go long.

**Implementation:**

```rust
use alphafield_core::{Strategy, Signal, TradingMode};
use alphafield_strategy::indicators::{RSI, BollingerBands};
use alphafield_backtest::adapter::StrategyAdapter;

struct MeanReversion {
    rsi: RSI,
    bb: BollingerBands,
    oversold_threshold: f64,
    overbought_threshold: f64,
}

impl Strategy for MeanReversion {
    fn name(&self) -> &str {
        "Mean Reversion with Shorts"
    }
    
    fn on_bar(&mut self, bar: &Bar) -> Option<Signal> {
        self.rsi.update(bar.close);
        self.bb.update(bar.close);
        
        let rsi_val = self.rsi.value();
        let (bb_upper, bb_lower) = (self.bb.upper(), self.bb.lower());
        
        // Multiple confluence signals
        if rsi_val < self.oversold_threshold && bar.close < bb_lower {
            // Strong oversold signal - go long
            Some(Signal::new_buy(bar.close, 1.5))  // Larger position
        } else if rsi_val > self.overbought_threshold && bar.close > bb_upper {
            // Strong overbought signal - go short
            Some(Signal::new_sell(bar.close, 1.5))
        } else {
            Some(Signal::new_hold())
        }
    }
}

// Configure for Margin mode
let strategy = MeanReversion::new(
    RSI::new(14),
    BollingerBands::new(20, 2.0),
    30.0,
    70.0
);
let adapter = StrategyAdapter::new(strategy, "ETHUSDT", 10000.0, 0.90)
    .with_trading_mode(TradingMode::Margin);
```

### Pairs Trading

**Hypothesis:** Two correlated assets will maintain a mean spread. When spread widens significantly, short the overperformer and long the underperformer, profiting when spread converges.

**Implementation:**

```rust
use alphafield_core::{Strategy, Signal, TradingMode};
use alphafield_strategy::indicators::SMA;
use alphafield_backtest::adapter::StrategyAdapter;

struct PairsTrading {
    symbol_a: String,
    symbol_b: String,
    hedge_ratio: f64,
    entry_threshold: f64,
    exit_threshold: f64,
    spread_sma: SMA,
}

impl Strategy for PairsTrading {
    fn name(&self) -> &str {
        "Pairs Trading"
    }
    
    fn on_bar(&mut self, bar: &Bar) -> Option<Signal> {
        // Note: This is simplified - real implementation needs two bar streams
        let price_a = bar.close;  // In practice, track both assets
        let price_b = bar.close * self.hedge_ratio;
        
        let spread = price_a - price_b;
        self.spread_sma.update(spread);
        let spread_mean = self.spread_sma.value();
        let spread_z = (spread - spread_mean) / spread_mean;
        
        if spread_z > self.entry_threshold {
            // A is overperforming B - short A, long B
            Some(Signal::new_sell(bar.close, 1.0))
        } else if spread_z < -self.entry_threshold {
            // A is underperforming B - long A, short B
            Some(Signal::new_buy(bar.close, 1.0))
        } else if spread_z.abs() < self.exit_threshold {
            // Spread converged - close positions
            Some(Signal::new_hold())  // In practice, close both legs
        } else {
            Some(Signal::new_hold())
        }
    }
}

// Pairs trading typically requires Margin mode
let strategy = PairsTrading::new(
    "BTCUSDT".to_string(),
    "ETHUSDT".to_string(),
    15.0,   // hedge ratio
    2.0,    // entry threshold (2 standard deviations)
    0.5     // exit threshold
);
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95)
    .with_trading_mode(TradingMode::Margin);
```

### Market Neutral

**Hypothesis:** By taking both long and short positions, overall portfolio exposure to market direction is minimized, focusing on relative performance.

**Implementation:**

```rust
use alphafield_core::{Strategy, Signal, TradingMode};
use alphafield_strategy::indicators::{RSI, ATR};
use alphafield_backtest::adapter::StrategyAdapter;

struct MarketNeutral {
    long_threshold: f64,
    short_threshold: f64,
    atr: ATR,
    position_size: f64,
}

impl Strategy for MarketNeutral {
    fn name(&self) -> &str {
        "Market Neutral"
    }
    
    fn on_bar(&mut self, bar: &Bar) -> Option<Signal> {
        self.atr.update(bar.high, bar.low, bar.close);
        let volatility = self.atr.value();
        
        // Dynamic thresholds based on volatility
        let long_entry = self.long_threshold * volatility;
        let short_entry = self.short_threshold * volatility;
        
        // Simplified logic - use actual strategy signals in production
        if bar.close_change() > long_entry {
            Some(Signal::new_buy(bar.close, self.position_size))
        } else if bar.close_change() < -short_entry {
            Some(Signal::new_sell(bar.close, self.position_size))
        } else {
            Some(Signal::new_hold())
        }
    }
}

// Market neutral requires Margin mode
let strategy = MarketNeutral::new(2.0, -2.0, ATR::new(14), 0.5);
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95)
    .with_trading_mode(TradingMode::Margin);
```

### Trend Following with Shorts

**Hypothesis:** Markets trend in both directions. Identify downward trends and profit from them by shorting, just as upward trends are profited from by going long.

**Implementation:**

```rust
use alphafield_core::{Strategy, Signal, TradingMode};
use alphafield_strategy::indicators::{SMA, EMA, ATR};
use alphafield_backtest::adapter::StrategyAdapter;

struct DualTrend {
    fast_ema: EMA,
    slow_sma: SMA,
    atr: ATR,
    trend_strength: f64,
}

impl Strategy for DualTrend {
    fn name(&self) -> &str {
        "Dual Trend Following"
    }
    
    fn on_bar(&mut self, bar: &Bar) -> Option<Signal> {
        self.fast_ema.update(bar.close);
        self.slow_sma.update(bar.close);
        self.atr.update(bar.high, bar.low, bar.close);
        
        let fast = self.fast_ema.value();
        let slow = self.slow_sma.value();
        let trend = (fast - slow) / slow;
        let volatility = self.atr.value() / bar.close;
        
        // Normalize trend by volatility
        let signal_strength = trend / volatility;
        
        if signal_strength > self.trend_strength {
            // Strong upward trend - go long
            Some(Signal::new_buy(bar.close, 1.0))
        } else if signal_strength < -self.trend_strength {
            // Strong downward trend - go short
            Some(Signal::new_sell(bar.close, 1.0))
        } else {
            // No clear trend - hold current position
            Some(Signal::new_hold())
        }
    }
}

// Dual trend requires Margin mode to short
let strategy = DualTrend::new(EMA::new(9), SMA::new(21), ATR::new(14), 1.5);
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95)
    .with_trading_mode(TradingMode::Margin);
```

---

## Migration Guide

### From Spot to Margin

Converting an existing Spot mode strategy to Margin mode is straightforward but requires careful consideration of risk.

**Step 1: Enable Margin Mode**

```rust
// Before (Spot mode - default)
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95);

// After (Margin mode)
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95)
    .with_trading_mode(TradingMode::Margin);
```

**Step 2: Update Portfolio and BacktestEngine**

```rust
// Portfolio
let portfolio = Portfolio::new(10000.0)
    .with_trading_mode(TradingMode::Margin);

// BacktestEngine
let engine = BacktestEngine::new(10000.0, 0.001)
    .with_trading_mode(TradingMode::Margin);
```

**Step 3: Adjust Risk Management**

```rust
use alphafield_execution::risk_checks::{NoShorts, MaxShortPosition, ShortSqueezeDetection};

// Remove or disable NoShorts check
// Add new risk checks specific to shorts
let max_short = MaxShortPosition::new(0.3);  // 30% max short position
let squeeze_detect = ShortSqueezeDetection::new(0.1, 3);

risk_manager.add_check(max_short);
risk_manager.add_check(squeeze_detect);
```

**Step 4: Test and Validate**

1. Run backtests with Margin mode
2. Compare performance to Spot mode
3. Check for unexpected short positions
4. Validate risk check effectiveness
5. Run walk-forward analysis
6. Perform Monte Carlo simulation

### Common Pitfalls

**1. Forgetting to Set TradingMode**

```rust
// ❌ WRONG - Will use Spot mode by default
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95);

// ✅ CORRECT - Explicitly set Margin mode
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95)
    .with_trading_mode(TradingMode::Margin);
```

**2. Inconsistent Configuration Across Components**

```rust
// ❌ WRONG - Adapter in Margin, Portfolio in Spot
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95)
    .with_trading_mode(TradingMode::Margin);
let portfolio = Portfolio::new(10000.0);  // Defaults to Spot!

// ✅ CORRECT - All components use Margin mode
let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95)
    .with_trading_mode(TradingMode::Margin);
let portfolio = Portfolio::new(10000.0)
    .with_trading_mode(TradingMode::Margin);
```

**3. NoShorts Check Still Active**

```rust
// ❌ WRONG - NoShorts will reject all short orders
let no_shorts = NoShorts::new(TradingMode::Spot);
risk_manager.add_check(no_shorts);
// ... set Margin mode ...

// ✅ CORRECT - NoShorts respects Margin mode
let no_shorts = NoShorts::new(TradingMode::Margin);
risk_manager.add_check(no_shorts);
// OR don't add NoShorts at all in Margin mode
```

**4. Missing Position Size Limits**

```rust
// ❌ WRONG - No limits on short position size
let risk_manager = RiskManager::new(service, checks, TradingMode::Margin);

// ✅ CORRECT - Add MaxShortPosition check
let max_short = MaxShortPosition::new(0.3);
risk_manager.add_check(max_short);
```

**5. Ignoring Borrowing Costs**

```rust
// ❌ WRONG - Not accounting for borrow costs in profitability
let profit = trade.entry_price - trade.exit_price;

// ✅ CORRECT - Subtract borrowing costs
let profit = trade.entry_price - trade.exit_price - trade.borrow_cost - trade.funding_cost;
```

**6. Not Handling Flat State Transitions**

```rust
// ❌ WRONG - Assuming strategy always handles transitions
// Strategy may generate Sell signal when already flat

// ✅ CORRECT - StrategyAdapter handles state transitions
// Flat → Long (Buy), Long → Flat (Sell)
// Flat → Short (Sell), Short → Flat (Buy)
// Long → Short (Sell + Sell)
```

### WebSocket Updates

When using Margin mode, position updates sent via WebSocket include the position side:

```json
{
  "type": "position_update",
  "symbol": "BTCUSDT",
  "side": "short",  // New field: "long" or "short"
  "quantity": 0.5,
  "entry_price": 45000.0,
  "current_price": 44000.0,
  "unrealized_pnl": 500.0
}
```

**Frontend Display:**

```javascript
function handlePositionUpdate(update) {
  const sideColor = update.side === 'long' ? 'green' : 'red';
  const sideIcon = update.side === 'long' ? '▲' : '▼';
  
  console.log(`${sideIcon} ${update.symbol}: ${update.quantity} @ ${update.entry_price}`);
  console.log(`P&L: ${update.unrealized_pnl.toFixed(2)} (${sideColor})`);
}
```

### Risk Management for Short Positions

**Essential Risk Checks:**

```rust
use alphafield_execution::risk_checks::*;

// 1. Maximum short position size (as % of portfolio)
let max_short = MaxShortPosition::new(0.3);

// 2. Short squeeze detection
let squeeze_detect = ShortSqueezeDetection::new(0.1, 3);

// 3. Margin requirement monitoring
let margin_req = MarginRequirement::new(1.5);

// 4. Daily loss limit (applies to shorts too)
let daily_loss = MaxDailyLoss::new(1000.0);

// Add all to risk manager
risk_manager.add_check(max_short);
risk_manager.add_check(squeeze_detect);
risk_manager.add_check(margin_req);
risk_manager.add_check(daily_loss);
```

### Testing Your Strategy

**1. Unit Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_margin_mode_allows_short() {
        let strategy = MeanReversion::new();
        let adapter = StrategyAdapter::new(strategy, "BTCUSDT", 10000.0, 0.95)
            .with_trading_mode(TradingMode::Margin);
        
        // Simulate overbought condition
        let bar = create_overbought_bar();
        let signal = adapter.on_bar(&bar).unwrap();
        
        assert_eq!(signal.side, TradeSide::Sell);
    }
    
    #[test]
    fn test_max_short_position_enforced() {
        let mut portfolio = Portfolio::new(10000.0)
            .with_trading_mode(TradingMode::Margin);
        
        // Try to short more than allowed
        let short_order = Order::new_market_sell("BTCUSDT", 1.0, 50000.0);
        let result = portfolio.apply_order(&short_order);
        
        assert!(result.is_err());  // Should fail
    }
}
```

**2. Integration Tests:**

```rust
#[test]
fn test_full_backtest_short_positions() {
    let strategy = MeanReversion::new();
    let engine = BacktestEngine::new(10000.0, 0.001)
        .with_trading_mode(TradingMode::Margin);
    
    let bars = load_test_data();
    let result = engine.run(&strategy, &bars);
    
    assert!(result.short_trades_count > 0);
    assert!(result.short_win_rate > 0.0);
}
```

**3. Performance Benchmarks:**

```rust
#[bench]
fn bench_backtest_margin_mode(b: &mut Bencher) {
    let strategy = MeanReversion::new();
    let engine = BacktestEngine::new(10000.0, 0.001)
        .with_trading_mode(TradingMode::Margin);
    let bars = load_large_dataset();
    
    b.iter(|| {
        engine.run(&strategy, &bars)
    });
}
```

---

## Common Patterns

### Mean Reversion

**Concept:** Buy when price is significantly below mean, sell when significantly above mean.

**Key Indicators:** RSI, Bollinger Bands, Moving Averages, Standard Deviation

**Best for:** Range-bound markets, assets with established trading ranges

**Example:** See "Mean Reversion with Shorts" in Example Strategies section

```rust
if rsi < oversold && price < lower_bb {
    Signal::new_buy(price, size)
} else if rsi > overbought && price > upper_bb {
    Signal::new_sell(price, size)
} else {
    Signal::new_hold()
}
```

### Pairs Trading

**Concept:** Short the overperforming asset, long the underperforming asset in a correlated pair.

**Key Indicators:** Correlation analysis, spread tracking, cointegration tests, z-score

**Best for:** Highly correlated assets (e.g., BTC/ETH, GLD/SLV)

**Example:** See "Pairs Trading" in Example Strategies section

```rust
let spread = price_a - price_b * hedge_ratio;
let z_score = (spread - spread_mean) / spread_std;

if z_score > entry_threshold {
    // A is overpriced - short A, long B
    Signal::new_sell(price_a, size_a)
} else if z_score < -entry_threshold {
    // A is underpriced - long A, short B
    Signal::new_buy(price_a, size_a)
}
```

### Market Neutral

**Concept:** Maintain balanced long and short exposures to minimize market direction risk.

**Key Indicators:** Beta hedging, factor neutralization, portfolio beta calculation

**Best for:** Volatile markets, when you want to profit from relative performance

**Example:** See "Market Neutral" in Example Strategies section

```rust
// Calculate portfolio beta
let long_beta = sum(position.beta * position.value for long_positions);
let short_beta = sum(position.beta * position.value for short_positions);
let net_beta = (long_beta - short_beta) / total_portfolio_value;

// Adjust to target beta (e.g., 0 for neutral)
if net_beta > target_beta {
    // Reduce net exposure - add shorts or reduce longs
    Signal::new_sell(overweight_asset, adjustment_size)
} else if net_beta < target_beta {
    // Increase net exposure - add longs or reduce shorts
    Signal::new_buy(underweight_asset, adjustment_size)
}
```

### Trend Following with Shorts

**Concept:** Profit from downward trends by shorting, just as upward trends are profited from by going long.

**Key Indicators:** Moving averages, trend strength, ADX, MACD

**Best for:** Trending markets, volatile assets, during bear markets

**Example:** See "Trend Following with Shorts" in Example Strategies section

```rust
let fast_ma = ema9;
let slow_ma = sma21;
let trend = (fast_ma - slow_ma) / slow_ma;

if trend > strong_up_threshold {
    Signal::new_buy(price, size)
} else if trend < strong_down_threshold {
    Signal::new_sell(price, size)
} else {
    Signal::new_hold()
}
```

---

## Best Practices

### General Principles

1. **Start Simple, Then Scale**
   - Begin with Spot mode strategies
   - Master long-only trading first
   - Only use Margin mode when strategy requires it

2. **Understand Your Strategy**
   - Know when your strategy goes long vs short
   - Understand the risk profile of each position type
   - Document strategy hypotheses clearly

3. **Risk Management is Paramount**
   - Always use position size limits
   - Implement stop-losses for short positions
   - Monitor margin usage in real-time
   - Diversify across assets

4. **Test Thoroughly**
   - Use walk-forward analysis
   - Perform Monte Carlo simulations
   - Test across different market conditions
   - Validate backtest results against paper trading

5. **Keep Learning**
   - Analyze trade outcomes regularly
   - Learn from mistakes
   - Adjust strategy parameters based on data
   - Stay updated on market conditions

### Spot Mode Best Practices

1. **Focus on Directional Trends**
   - Spot mode excels with trend-following
   - Use momentum indicators for entry signals
   - Implement trailing stops to protect profits

2. **Monitor Cash Utilization**
   - Track available cash vs position value
   - Avoid over-concentration in single assets
   - Maintain cash buffer for new opportunities

3. **Use Simple Risk Checks**
   - `MaxPositionValue` to control exposure
   - `MaxDailyLoss` to limit drawdowns
   - `NoShorts` check (already default)

4. **Optimize for Sharpe Ratio**
   - Focus on risk-adjusted returns
   - Lower volatility often leads to better compounding
   - Consider transaction costs in optimization

### Margin Mode Best Practices

1. **Always Use Position Limits**
   ```rust
   let max_short = MaxShortPosition::new(0.3);  // Never exceed 30%
   risk_manager.add_check(max_short);
   ```

2. **Implement Short Squeeze Protection**
   ```rust
   let squeeze_detect = ShortSqueezeDetection::new(0.1, 3);
   risk_manager.add_check(squeeze_detect);
   ```

3. **Monitor Borrowing Costs**
   - Track borrow rates regularly
   - Factor costs into profitability calculations
   - Avoid assets with extremely high borrow costs

4. **Maintain Margin Buffers**
   ```rust
   let margin_req = MarginRequirement::new(1.5);  // 150% buffer
   risk_manager.add_check(margin_req);
   ```

5. **Diversify Short Positions**
   - Don't concentrate shorts in correlated assets
   - Spread risk across sectors/asset classes
   - Consider inverse ETFs for market hedges

6. **Paper Trade First**
   - Test Margin mode strategies with paper trading
   - Validate risk check effectiveness
   - Build confidence before using real capital

7. **Use Conservative Position Sizing**
   - Start with smaller positions than Spot mode
   - Scale up gradually as confidence builds
   - Adjust size based on volatility

8. **Regular Rebalancing**
   - Monitor position drift daily
   - Rebalance to maintain target weights
   - Close losing shorts before they become catastrophic

### Development Best Practices

1. **Write Comprehensive Tests**
   - Unit tests for individual components
   - Integration tests for full workflows
   - Regression tests to ensure backward compatibility

2. **Document Everything**
   - Strategy hypotheses and rationale
   - Risk management rules
   - Configuration parameters
   - Performance metrics and expectations

3. **Version Control**
   - Track all strategy changes
   - Document why changes were made
   - Maintain rollback capability

4. **Monitoring and Alerting**
   - Set up alerts for risk breaches
   - Monitor strategy performance in real-time
   - Track margin and borrowing costs

5. **Continuous Improvement**
   - Regular strategy reviews
   - Parameter optimization
   - Adapt to changing market conditions

### Performance Optimization

1. **Reduce Unnecessary Computations**
   - Cache indicator values
   - Avoid redundant calculations
   - Use efficient data structures

2. **Optimize Database Queries**
   - Use TimescaleDB hypertables efficiently
   - Query only needed time ranges
   - Leverage compression policies

3. **Parallelize Where Possible**
   - Use Rust's async capabilities
   - Parallelize independent computations
   - Consider rayon for CPU-bound tasks

4. **Profile Regularly**
   - Identify bottlenecks
   - Optimize hot paths
   - Measure impact of optimizations

---

## Conclusion

AlphaField's Trading Modes feature provides flexibility to implement a wide range of strategies:

- **Spot Mode**: Simple, long-only trading ideal for beginners and trend-following strategies
- **Margin Mode**: Advanced long/short trading for market neutral, pairs trading, and other sophisticated approaches

**Key Takeaways:**

1. **Start with Spot Mode** - It's safer, simpler, and works for most strategies
2. **Understand the Risks** - Short positions have unique risks that must be managed
3. **Use Proper Risk Checks** - Position limits, squeeze detection, margin monitoring
4. **Test Thoroughly** - Walk-forward, Monte Carlo, and paper trading are essential
5. **Document Everything** - Strategy hypotheses, risk rules, and performance metrics

**Next Steps:**

1. Review your strategy's hypothesis - does it require short positions?
2. If yes, carefully evaluate the risks and benefits
3. Implement appropriate risk management checks
4. Test extensively with paper trading
5. Start small and scale gradually

For more information, see:
- [Architecture Document](architecture.md) - System design and component interactions
- [API Documentation](api.md) - REST API endpoints and usage
- [ML Guide](ml.md) - Machine learning features and validation
- [Roadmap](roadmap.md) - Future enhancements and features

Happy trading! 🚀