# Daily Reconciliation Runbook

## Overview

Daily reconciliation ensures that internal records match exchange balances and all trades are properly accounted for.

---

## Schedule

**Time**: Daily at 00:30 UTC (after midnight PnL reset)

---

## Pre-Requisites

- Access to exchange dashboard
- Access to AlphaField database
- API credentials configured

---

## Reconciliation Steps

### 1. Fetch Exchange Balances

```bash
# Get current balances from Binance
curl -X GET -H "X-MBX-APIKEY: $BINANCE_API_KEY" \
  "https://api.binance.com/api/v3/account" \
  --data-urlencode "signature=$(echo -n "timestamp=$(date +%s000)" | openssl dgst -sha256 -hmac $BINANCE_SECRET_KEY | awk '{print $2}')"
```

Or via dashboard API:
```bash
curl http://localhost:8080/api/positions
```

### 2. Fetch Internal Database Balances

```bash
psql $DATABASE_URL -c "
SELECT 
  symbol,
  SUM(quantity) as position,
  SUM(quantity * entry_price) as notional
FROM positions
WHERE status = 'open'
GROUP BY symbol;
"
```

### 3. Compare Positions

| Symbol | Exchange | Internal | Difference |
|--------|----------|----------|------------|
| BTC | ? | ? | ? |
| ETH | ? | ? | ? |

**Acceptable Variance**: < 0.1%

### 4. Reconcile Cash/USDT

```bash
# Internal cash balance
psql $DATABASE_URL -c "SELECT cash_balance FROM portfolio_state ORDER BY timestamp DESC LIMIT 1;"

# Exchange cash (stablecoin balance)
# Compare with exchange account balance
```

### 5. Trade Count Verification

```bash
# Count trades in DB for last 24h
psql $DATABASE_URL -c "
SELECT COUNT(*) as trade_count
FROM trades
WHERE timestamp > NOW() - INTERVAL '24 hours';
"

# Compare with exchange trade history count
```

---

## Discrepancy Resolution

### Minor Discrepancy (< 0.1%)

1. Log the discrepancy
2. Check for pending fills
3. Usually resolves after market data update

### Major Discrepancy (> 0.1%)

1. **STOP** new order submission
2. Manually verify each position
3. Check for:
   - Failed order acknowledgments
   - Duplicate fills
   - Missing cancel events

### Reconciliation Adjustment

```bash
# If adjustment needed, document and apply:
psql $DATABASE_URL -c "
INSERT INTO reconciliation_adjustments (
  timestamp, symbol, adjustment_qty, reason
) VALUES (
  NOW(), 'BTCUSDT', 0.001, 'Exchange sync discrepancy - manual adjustment'
);
"
```

---

## Daily Report Template

```
=== AlphaField Daily Reconciliation ===
Date: YYYY-MM-DD
Time: HH:MM UTC

Positions Reconciled: [✓ / ✗]
Cash Balance Reconciled: [✓ / ✗]
Trade Count Match: [✓ / ✗]

Discrepancies Found: [None / List]

PnL Summary:
  Realized: $X.XX
  Unrealized: $X.XX
  Total: $X.XX

Notes:
[Any observations or issues]

Verified By: [Name]
```

---

## Automation

The reconciliation can be automated via a scheduled job:

```bash
# Add to crontab
30 0 * * * /path/to/scripts/daily_reconciliation.sh >> /var/log/reconciliation.log 2>&1
```

---

## Audit Trail

All reconciliations should be:
1. Logged to `logs/reconciliation/`
2. Retained for 90 days minimum
3. Available for compliance review
