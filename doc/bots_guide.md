# 🤖 Automated Trading Bots Guide

> **Phase 14 Feature**: Hands-off trading automation for AlphaField

AlphaField includes three types of automated trading bots that execute predefined strategies without manual intervention. This guide covers configuration, usage, and best practices for each bot type.

---

## 📊 Overview

### Bot Types

| Bot Type | Use Case | Difficulty | Risk Level |
|----------|----------|------------|------------|
| **DCA Bot** | Regular accumulation at fixed intervals | Easy | Low |
| **Grid Bot** | Range-bound market profits | Medium | Medium |
| **Trailing Orders** | Dynamic exit strategy protection | Easy | Low-Medium |

### Common Features

All bots share these capabilities:

- **Lifecycle Management**: Start, pause, resume, and stop operations
- **Real-time Monitoring**: Track performance, orders, and PnL
- **Risk Controls**: Built-in safeguards prevent runaway behavior
- **Persistence**: State saved for seamless restarts
- **REST API**: Full programmatic control

---

## 💰 DCA Bot (Dollar Cost Averaging)

### Overview

The DCA bot executes recurring buy orders at configurable intervals, implementing a dollar-cost averaging strategy to smooth out price volatility.

### Use Cases

- **Long-term Accumulation**: Build a position gradually over time
- **Reduce Timing Risk**: Avoid trying to "time the market"
- **Budget Management**: Invest fixed amounts on a schedule

### Configuration

```json
{
  "symbol": "BTCUSDT",
  "amount_type": {
    "FixedAmount": 100.0  // Or: {"PercentOfBalance": 10.0}
  },
  "frequency": "Daily",  // Options: Minutes(n), Hours(n), Daily, Weekly, Monthly
  "max_price": 50000.0,  // Optional: stop if price exceeds this
  "total_budget": 10000.0  // Optional: stop when total spent reaches this
}
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `symbol` | String | Trading pair (e.g., "BTCUSDT") |
| `amount_type` | Enum | `FixedAmount(f64)` or `PercentOfBalance(f64)` |
| `frequency` | Enum | Buy interval: Minutes/Hours/Daily/Weekly/Monthly |
| `max_price` | Option<f64> | Stop buying if price exceeds threshold |
| `total_budget` | Option<f64> | Total capital limit for the bot |

### Example Usage

**Create a DCA bot:**
```bash
curl -X POST http://localhost:8080/api/bots/dca \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "amount_type": {"FixedAmount": 100.0},
    "frequency": "Daily",
    "max_price": null,
    "total_budget": 10000.0
  }'
```

**Start the bot:**
```bash
curl -X POST http://localhost:8080/api/bots/dca/{id}/start
```

**Monitor status:**
```bash
curl http://localhost:8080/api/bots/dca
```

### Best Practices

1. **Set Realistic Budgets**: Ensure you have sufficient balance for all scheduled buys
2. **Use Price Thresholds**: Set `max_price` to avoid buying at extreme highs
3. **Start Small**: Test with small amounts before scaling up
4. **Monitor Regularly**: Check bot status weekly to ensure proper operation

---

## 📊 Grid Bot

### Overview

Grid bots place multiple limit orders at predefined price levels, automatically buying low and selling high within a specified range.

### Use Cases

- **Range-Bound Markets**: Profit from sideways price action
- **Volatility Harvesting**: Capture small price oscillations
- **Market Making**: Provide liquidity while earning spreads

### Configuration

```json
{
  "symbol": "BTCUSDT",
  "lower_price": 40000.0,
  "upper_price": 60000.0,
  "grid_levels": 10,
  "total_capital": 10000.0,
  "min_profit_percent": 1.0
}
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `symbol` | String | Trading pair |
| `lower_price` | f64 | Bottom of the price range |
| `upper_price` | f64 | Top of the price range |
| `grid_levels` | u32 | Number of price levels in the grid |
| `total_capital` | f64 | Total capital allocated to the grid |
| `min_profit_percent` | f64 | Minimum profit per trade (%) |

### How It Works

1. **Grid Initialization**: Bot calculates evenly spaced price levels
2. **Order Placement**: Buy orders placed below current price
3. **Buy Fill**: When buy order fills, corresponding sell order is placed above
4. **Sell Fill**: When sell order fills, profit is recorded and buy order is re-placed
5. **Continuous Cycling**: Process repeats indefinitely within the range

### Grid Calculation Example

For a grid with:
- Lower: $40,000
- Upper: $60,000
- Levels: 5

Grid prices will be: **$40,000**, **$45,000**, **$50,000**, **$55,000**, **$60,000**

### Example Usage

**Create a Grid bot:**
```bash
curl -X POST http://localhost:8080/api/bots/grid \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "ETHUSDT",
    "lower_price": 3000.0,
    "upper_price": 4000.0,
    "grid_levels": 10,
    "total_capital": 5000.0,
    "min_profit_percent": 0.5
  }'
```

**Initialize and start:**
```bash
curl -X POST http://localhost:8080/api/bots/grid/{id}/start
```

**View grid levels and profit:**
```bash
curl http://localhost:8080/api/bots/grid
```

### Best Practices

1. **Choose the Right Range**: Analyze historical support/resistance levels
2. **More Levels = Smaller Profits**: Balance grid density with profit targets
3. **Account for Fees**: Ensure `min_profit_percent` covers trading fees (typically 0.1%)
4. **Range Breakouts**: Monitor for price breaking out of range; stop bot if needed
5. **Rebalance Periodically**: Adjust range if market conditions change

### Risk Considerations

- **Directional Risk**: If price trends strongly up/down, you may accumulate one-sided position
- **Capital Lockup**: Grid capital is tied up for the duration
- **Fees Impact**: Frequent trading incurs fees; ensure profitability

---

## 🎯 Trailing Orders

### Overview

Trailing orders dynamically adjust stop-loss or take-profit levels as price moves favorably, locking in profits while allowing for further gains.

### Types

| Type | Behavior | Best For |
|------|----------|----------|
| **Trailing Stop-Loss** | Follows price up, triggers on drop | Protecting profits on long positions |
| **Trailing Take-Profit** | Follows price down, triggers on rise | Ensuring profit capture after pullback |

### Configuration

```json
{
  "symbol": "BTCUSDT",
  "trailing_type": "StopLoss",  // or "TakeProfit"
  "side": "Sell",
  "quantity": 0.1,
  "trailing_percent": 5.0,
  "activation_price": 55000.0,  // Optional: price to activate trailing
  "limit_price": null  // Optional: hard stop price
}
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `symbol` | String | Trading pair |
| `trailing_type` | Enum | `StopLoss` or `TakeProfit` |
| `side` | Enum | Order side when triggered (typically `Sell`) |
| `quantity` | f64 | Amount to trade |
| `trailing_percent` | f64 | Distance from extreme price (%) |
| `activation_price` | Option<f64> | Price level to activate trailing |
| `limit_price` | Option<f64> | Absolute price limit |

### How Trailing Stop-Loss Works

1. **Entry**: You buy BTC at $50,000
2. **Activation**: Price reaches $55,000 (activation price)
3. **Tracking**: Bot tracks highest price (extreme)
4. **Trigger**: Sell order placed 5% below highest price
5. **Example Flow**:
   - Price → $60,000 → Trigger at $57,000
   - Price → $62,000 → Trigger now at $58,900
   - Price drops to $58,900 → **Sell order executed**

### Example Usage

**Create a Trailing Stop-Loss:**
```bash
curl -X POST http://localhost:8080/api/bots/trailing \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "trailing_type": "StopLoss",
    "side": "Sell",
    "quantity": 0.5,
    "trailing_percent": 3.0,
    "activation_price": 52000.0,
    "limit_price": null
  }'
```

**Start tracking:**
```bash
curl -X POST http://localhost:8080/api/bots/trailing/{id}/start
```

**Monitor trigger price:**
```bash
curl http://localhost:8080/api/bots/trailing
```

### Best Practices

1. **Reasonable Trailing %**: Too tight (< 2%) = premature exit; Too wide (> 10%) = give back profits
2. **Set Activation Price**: Ensure you're already in profit before trailing activates
3. **Volatility Consideration**: Increase trailing % for volatile assets
4. **Never Delete Active Orders**: Always stop before deleting
5. **Test with Small Positions**: Validate behavior before full position sizing

---

## 🔧 API Reference

### Common Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/bots/status` | GET | Overview of all active bots |
| `/api/bots/{type}` | GET | List all bots of a specific type |
| `/api/bots/{type}` | POST | Create a new bot |
| `/api/bots/{type}/:id/start` | POST | Start a bot |
| `/api/bots/{type}/:id/pause` | POST | Pause a bot (DCA only) |
| `/api/bots/{type}/:id/stop` | POST | Stop a bot |
| `/api/bots/{type}/:id` | DELETE | Delete a stopped bot |

Replace `{type}` with: `dca`, `grid`, or `trailing`

### Bot Status Response

```json
{
  "id": "uuid",
  "status": "Active",  // Active, Paused, Completed, Stopped, Error
  "stats": {
    "orders_executed": 42,
    "total_volume": 5000.0,
    "total_fees": 5.0,
    "realized_pnl": 250.0,
    "started_at": "2026-01-01T00:00:00Z",
    "last_execution": "2026-01-03T12:00:00Z"
  }
}
```

---

## ⚠️ Risk Management

### Safety Features

1. **No Leverage**: All bots operate in spot-only mode
2. **Capital Limits**: Bots cannot exceed allocated capital
3. **Price Validation**: Orders rejected if prices are invalid
4. **Status Tracking**: All state changes logged

### Monitoring Checklist

- [ ] **Daily**: Check bot status and recent orders
- [ ] **Weekly**: Review PnL and adjust parameters if needed
- [ ] **Monthly**: Evaluate bot performance vs. objectives

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Bot stopped unexpectedly | Budget/price limit reached | Increase limits or adjust params |
| No orders executed | Price outside range | Adjust price range |
| High fees, low profit | Too many small trades | Increase min profit % or grid spacing |

---

## 🎓 Strategy Examples

### Conservative DCA

```json
{
  "symbol": "BTCUSDT",
  "amount_type": {"FixedAmount": 50.0},
  "frequency": "Weekly",
  "max_price": null,
  "total_budget": 2600.0  // 1 year at $50/week
}
```

### Aggressive Grid Trading

```json
{
  "symbol": "SOLUSDT",
  "lower_price": 100.0,
  "upper_price": 150.0,
  "grid_levels": 20,
  "total_capital": 3000.0,
  "min_profit_percent": 0.3
}
```

### Profit Protection

```json
{
  "symbol": "ETHUSDT",
  "trailing_type": "StopLoss",
  "side": "Sell",
  "quantity": 5.0,
  "trailing_percent": 4.0,
  "activation_price": 3500.0  // Entry was at 3000
}
```

---

## 📚 Further Reading

- [AlphaField API Documentation](./api.md)
- [Risk Management Guide](./detailed_design.md#risk-management)
- [Backtesting Bots](./optimization_workflow.md) (simulate bot behavior)

---

*Last updated: January 2026*
