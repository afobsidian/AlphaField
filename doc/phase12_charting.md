# Phase 12: Interactive Charting

> Status: ✅ Complete  
> Date Completed: January 2026

## Overview

Phase 12 adds comprehensive interactive charting capabilities to the AlphaField dashboard, enabling visual analysis of price action, technical indicators, and backtest results all in one place.

## Features Implemented

### Chart Display
- **Multiple Chart Types**: Candlestick, line, and area charts
- **Timeframe Support**: Works with all supported intervals (1m, 5m, 15m, 1h, 4h, 1d)
- **Responsive Design**: Plotly.js-based charts that adapt to screen size

### Technical Indicators
All indicators use the same calculations from the strategy crate, ensuring consistency between backtest logic and visualization:

**Overlay Indicators:**
- SMA (Simple Moving Average) - configurable periods (50, 200)
- EMA (Exponential Moving Average) - configurable period (20)
- Bollinger Bands - upper/middle/lower bands with 2σ standard deviation

**Oscillators (Separate Panels):**
- RSI (Relative Strength Index) - 14-period with overbought (70) and oversold (30) lines
- MACD (Moving Average Convergence Divergence) - MACD line, signal line, and histogram

### Trade Visualization
- **Entry Markers**: Green triangle-up markers showing buy entry points
- **Exit Markers**: Red triangle-down markers showing sell exit points
- **Trade Context**: Hover over markers to see entry/exit prices and P&L

### User Controls
- **Chart Type Selector**: Toggle between candlestick, line, and area displays
- **Indicator Panel**: Collapsible panel with checkboxes to enable/disable indicators
- **Auto-Update**: Chart automatically updates after running a backtest

## Technical Architecture

### Backend API

**Endpoint:** `POST /api/chart/ohlcv`

**Request:**
```json
{
  "symbol": "BTC",
  "interval": "1h",
  "days": 90,
  "indicators": [
    {"type": "sma", "period": 50},
    {"type": "ema", "period": 20},
    {"type": "rsi", "period": 14},
    {"type": "macd", "fast": 12, "slow": 26, "signal": 9},
    {"type": "bb", "period": 20, "std_dev": 2.0}
  ]
}
```

**Response:**
```json
{
  "symbol": "BTC",
  "interval": "1h",
  "bars": [
    {
      "timestamp": 1609459200,
      "open": 29000.0,
      "high": 29500.0,
      "low": 28800.0,
      "close": 29200.0,
      "volume": 1234.56
    }
  ],
  "indicators": {
    "indicator_0": {
      "type": "line",
      "values": [
        {"timestamp": 1609459200, "value": 29100.0}
      ]
    },
    "indicator_2": {
      "type": "oscillator",
      "values": [{"timestamp": 1609459200, "value": 55.2}],
      "upper_bound": 70.0,
      "lower_bound": 30.0
    }
  }
}
```

### Frontend Implementation

**File:** `crates/dashboard/static/app.js`

Key functions:
- `updateChartIndicators()` - Fetches chart data from API
- `renderPriceChart()` - Main price chart renderer
- `renderOscillators()` - Renders RSI/MACD panels
- `switchChartType()` - Toggles between chart types
- `toggleIndicatorPanel()` - Shows/hides indicator controls

**File:** `crates/dashboard/static/index.html`

Added interactive charting section in the Backtest tab with:
- Chart type selector buttons
- Indicator control panel
- Main price chart div
- Separate panels for RSI and MACD

**File:** `crates/dashboard/static/style.css`

Custom styles for:
- Chart type selector buttons
- Indicator control panel
- Oscillator panels

### Code Organization

**New File:** `crates/dashboard/src/chart_api.rs`

Contains:
- `ChartRequest`/`ChartResponse` types
- `IndicatorConfig` enum for different indicator types
- `IndicatorData` enum for different visualization types
- Calculation functions for each indicator
- Comprehensive unit tests

## Usage Guide

### Basic Usage

1. **Run a Backtest**
   - Navigate to Build tab, select strategy and asset category
   - Go to Optimize tab, run Auto-Optimize
   - Go to Backtest tab, select a symbol
   - Click "Run Backtest"

2. **View Interactive Chart**
   - After backtest completes, scroll down to "Interactive Price Chart"
   - Chart automatically loads with default candlestick view
   - Trade entry/exit markers from backtest are overlaid

3. **Add Indicators**
   - Click "⚙️ Indicators" button to open control panel
   - Check boxes for desired indicators (SMA 50, SMA 200, EMA 20, etc.)
   - Indicators are calculated and displayed automatically
   - RSI and MACD appear in separate panels below the main chart

4. **Change Chart Type**
   - Click candlestick/line/area buttons to switch visualization
   - All indicators remain visible across chart types

### Advanced Features

**Multiple Indicators:**
You can enable multiple overlays simultaneously:
- Both SMA 50 and SMA 200 for golden/death cross analysis
- EMA 20 with Bollinger Bands for trend + volatility
- All overlays together for comprehensive technical analysis

**Oscillator Analysis:**
- RSI panel shows momentum with overbought/oversold zones
- MACD panel shows trend strength with histogram
- Panels only appear when respective indicators are enabled

**Trade Analysis:**
- Green entry markers show where strategy initiated positions
- Red exit markers show where strategy closed positions
- Hover over markers to see exact price, quantity, and P&L

## Performance Considerations

- Chart data is fetched separately from backtest data to keep responses lightweight
- Indicators are calculated on-demand based on user selection
- All calculations use the same efficient indicator implementations as backtesting
- Plotly.js handles efficient rendering of large datasets with interactive zoom/pan

## Testing

### Unit Tests

Located in `crates/dashboard/src/chart_api.rs`:

- `test_indicator_config_deserialization` - Validates JSON parsing
- `test_calculate_sma` - Verifies SMA calculation accuracy
- `test_calculate_rsi` - Verifies RSI calculation and bounds

All tests pass with expected values.

### Integration Testing

Manual testing workflow:
1. Start dashboard server: `cargo run --bin dashboard_server`
2. Navigate to `http://localhost:8080`
3. Run backtest with BTC, 1h interval, 90 days
4. Verify chart loads with candlesticks
5. Enable each indicator individually
6. Switch between chart types
7. Verify trade markers appear correctly

## Future Enhancements

Potential additions for future phases:
- **Custom Timeframe Selection**: Let users pick custom date ranges
- **Drawing Tools**: Trendlines, horizontal levels, Fibonacci retracements
- **Volume Bars**: Separate volume panel below price chart
- **More Indicators**: Stochastic, ATR, ADX, Ichimoku Cloud, etc.
- **Chart Annotations**: Manual notes and labels on specific bars
- **Multi-Symbol Comparison**: Overlay multiple assets on same chart
- **Export Functionality**: Save charts as PNG/SVG

## Conclusion

Phase 12 successfully delivers a professional-grade interactive charting experience integrated seamlessly into the AlphaField dashboard. The implementation follows the existing architectural patterns, reuses core indicator logic, and provides an intuitive user interface for visual analysis of trading strategies.

All acceptance criteria from the roadmap have been met:
✅ Multiple timeframes  
✅ Candlestick, line, area chart types  
✅ SMA/EMA overlays  
✅ RSI, MACD in separate panels  
✅ Bollinger Bands visualization  
✅ Custom indicator controls  
✅ Trade entry/exit markers  
✅ P&L annotations  
✅ Backtest integration  

---

*Last Updated: January 2026*
