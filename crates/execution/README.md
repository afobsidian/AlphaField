# 🛡️ AlphaField Execution Crate

Responsible for order management and risk controls.

## 🛑 Risk Management

All orders must pass through the `RiskManager` before execution.

### Checks
1. **MaxOrderValue**: Rejects orders exceeding a hard cap.
2. **NoShorts**: blocks short selling (Spot-only mode).
3. **MaxDailyLoss**: Circuit breaker that halts trading if daily P&L hits threshold.
4. **PositionDrift**: Monitors live execution slippage.
5. **VolatilityScaling**: Adjusts position size based on asset volatility (ATR).

## 📡 Execution

- **`ExchangeClient`**: Trait for execution venues.
- **`PaperExchange`**: Simulation adapter for live-testing without real funds.
