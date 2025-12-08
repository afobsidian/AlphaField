# Incident Response Runbook

## Overview

This runbook covers common incidents and their resolution procedures for the AlphaField trading system.

---

## Severity Levels

| Level | Description | Response Time |
|-------|-------------|---------------|
| **P1 - Critical** | Trading halted, funds at risk | Immediate |
| **P2 - High** | Degraded trading, data issues | < 15 min |
| **P3 - Medium** | Minor issues, non-blocking | < 1 hour |
| **P4 - Low** | Cosmetic, logging issues | Next business day |

---

## Common Incidents

### 1. Exchange API Outage

**Symptoms**: API calls failing, orders not submitting

**Immediate Actions**:
1. Check exchange status page (e.g., https://www.binance.com/en/support/status)
2. Review logs: `tail -f logs/alphafield.log | grep -i error`
3. If widespread, halt all order submission

**Resolution**:
```bash
# Check API connectivity
curl -s https://api.binance.com/api/v3/ping

# If exchange is down, enable paper trading mode
export TRADING_MODE=paper

# Monitor for recovery
watch -n 60 'curl -s https://api.binance.com/api/v3/ping'
```

**Post-Incident**:
- Review any orders that were mid-flight
- Reconcile positions with exchange

---

### 2. Database Connection Lost

**Symptoms**: Data fetch failures, dashboard unresponsive

**Immediate Actions**:
1. Check database health: `docker ps | grep alphafield-db`
2. Check logs: `docker logs alphafield-db --tail 100`

**Resolution**:
```bash
# Restart database
docker restart alphafield-db

# Verify connection
psql $DATABASE_URL -c "SELECT 1;"

# Restart API server
docker restart alphafield-api
```

---

### 3. Circuit Breaker Triggered (Max Daily Loss)

**Symptoms**: All orders rejected with "circuit breaker active"

**Immediate Actions**:
1. Verify PnL in dashboard
2. Check logs for loss events

**Resolution**:
- Circuit breaker resets at midnight UTC
- If manual reset needed, restart the API server:
```bash
docker restart alphafield-api
```

**Post-Incident**:
- Review trades that caused the loss
- Adjust strategy parameters if needed

---

### 4. High Slippage / Position Drift Alert

**Symptoms**: `PositionDrift` warning in logs

**Immediate Actions**:
1. Check current market conditions (volatility spike?)
2. Review recent fills vs expected prices

**Resolution**:
```bash
# Check recent fills
tail -100 logs/alphafield.log | grep "Order filled"

# Temporarily increase drift threshold if needed (config change)
```

---

### 5. Rate Limit Exceeded

**Symptoms**: 429 errors from exchange API

**Immediate Actions**:
1. Reduce order frequency
2. Check for runaway processes

**Resolution**:
```bash
# View current rate limits
curl -s https://api.binance.com/api/v3/exchangeInfo | jq '.rateLimits'

# Implement exponential backoff (already in code)
```

---

## Emergency Procedures

### Panic Stop All Trading

```bash
# 1. Flatten all positions immediately
# (via exchange UI or emergency endpoint)

# 2. Stop the trading service
docker stop alphafield-api

# 3. Review situation before restarting
```

### Contact Information

| Role | Contact |
|------|---------|
| On-call Engineer | [TODO: Add contact] |
| Exchange Support | https://www.binance.com/en/support |

---

## Escalation Path

1. **First Response**: Check logs, attempt standard fixes
2. **15 min no resolution**: Escalate to senior engineer
3. **30 min no resolution**: Consider emergency stop
4. **Post-resolution**: Document in incident log
