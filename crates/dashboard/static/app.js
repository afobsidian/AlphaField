/**
 * AlphaField Dashboard v2.0
 * Workflow-Centric State Management
 */

const API_BASE = "/api";

// ===========================
// Global Strategy State
// ===========================

const AppState = {
  strategy: "GoldenCross",
  symbol: "",
  params: {},
  backtestDays: 90,
  backtestInterval: "1h",
  backtestResults: null,
  optimizeResults: null,
  isTrading: false,
  currentTab: "build",
  // ML Enhancement State
  mlEnabled: false,
  mlModelType: "linear",
  mlConfidence: 0.6,
  mlTrainingResults: null,
  mlPredictions: null,
};

// ===========================
// Strategies (dynamic)
// ===========================

// Cached list of strategies returned by /api/strategies
let AVAILABLE_STRATEGIES = [];

// Param schemas keyed by backend strategy name (e.g. "GoldenCross")
let STRATEGY_PARAMS = {};

// Friendly names/descriptions for UI.
// Friendly names/descriptions for UI.
// IMPORTANT: The optimization/backtest APIs expect the backend strategy key (e.g. "GoldenCross"),
// so we keep the radio `value` as the backend key and only change the displayed label/description.
const STRATEGY_UI_OVERRIDES = {
  // Baseline Strategies
  HODL_Baseline: {
    displayName: "HODL Baseline",
    description: "Buy and hold benchmark",
  },
  Market_Average_Baseline: {
    displayName: "Market Average",
    description: "Equal-weighted BTC/ETH/SOL basket",
  },

  // Trend Following Strategies
  GoldenCross: {
    displayName: "Golden Cross",
    description: "SMA 50/200 crossover trend following",
  },
  Breakout: {
    displayName: "Price Breakout",
    description: "20-period Donchian channel breakout",
  },
  MACrossover: {
    displayName: "MA Crossover",
    description: "Fast/Slow SMA crossover trend following",
  },
  AdaptiveMA: {
    displayName: "Adaptive MA (KAMA)",
    description: "Kaufman adaptive moving average trend following",
  },
  TripleMA: {
    displayName: "Triple MA",
    description: "Fast/medium/slow MA alignment trend following",
  },
  MacdTrend: {
    displayName: "MACD Trend",
    description: "MACD trend-following crossover strategy",
  },
  ParabolicSAR: {
    displayName: "Parabolic SAR",
    description: "Parabolic SAR trend + trailing stop",
  },

  // Mean Reversion Strategies (Phase 12.3)
  BollingerBands: {
    displayName: "Bollinger Bands",
    description: "BB reversion with RSI confirmation",
  },
  RSIReversion: {
    displayName: "RSI Reversion",
    description: "RSI oversold/overbought mean reversion",
  },
  StochReversion: {
    displayName: "Stochastic Reversion",
    description: "Stochastic oscillator mean reversion",
  },
  ZScoreReversion: {
    displayName: "Z-Score Reversion",
    description: "Statistical z-score mean reversion",
  },
  PriceChannel: {
    displayName: "Price Channel (Donchian)",
    description: "Donchian channel mean reversion",
  },
  KeltnerReversion: {
    displayName: "Keltner Channel",
    description: "Keltner channel reversion with volume",
  },
  StatArb: {
    displayName: "Statistical Arbitrage",
    description: "Z-score based statistical arbitrage",
  },

  // Momentum Strategies (Phase 12.4)
  RsiMomentumStrategy: {
    displayName: "RSI Momentum",
    description: "RSI > 50 momentum following strategy",
  },
  MACDStrategy: {
    displayName: "MACD Momentum",
    description: "MACD crossover with trend filter",
  },
  RocStrategy: {
    displayName: "Rate of Change (ROC)",
    description: "Price momentum rate of change",
  },
  AdxTrendStrategy: {
    displayName: "ADX Trend",
    description: "ADX strength-based trend following",
  },
  MomentumFactorStrategy: {
    displayName: "Momentum Factor",
    description: "Multi-factor momentum combination",
  },
  VolumeMomentumStrategy: {
    displayName: "Volume Momentum",
    description: "Volume-confirmed price momentum",
  },
  MultiTfMomentumStrategy: {
    displayName: "Multi-Timeframe Momentum",
    description: "Multi-EMA alignment momentum",
  },

  // Sentiment-Based Strategies (Phase 12.6)
  SentimentMomentumStrategy: {
    displayName: "Sentiment Momentum",
    description:
      "Follows sentiment trends using RSI, momentum, and volume indicators",
  },
  DivergenceStrategy: {
    displayName: "Divergence",
    description: "Price-sentiment divergence detection for reversal signals",
  },
  RegimeSentimentStrategy: {
    displayName: "Regime Sentiment",
    description: "Regime-aware sentiment adaptation (bull/bear/sideways)",
  },

  // Volatility-Based Strategies (Phase 12.5)
  ATRBreakoutStrategy: {
    displayName: "ATR Breakout",
    description: "Volatility breakout using ATR",
  },
  ATRTrailingStrategy: {
    displayName: "ATR Trailing Stop",
    description: "ATR-based trailing stop strategy",
  },
  GarchStrategy: {
    displayName: "GARCH Volatility",
    description: "GARCH model-based volatility trading",
  },
  VIXStyleStrategy: {
    displayName: "VIX-Style",
    description: "VIX-style volatility contrarian strategy",
  },
  VolRegimeStrategy: {
    displayName: "Volatility Regime",
    description: "Regime-based volatility adaptation",
  },
  VolSizingStrategy: {
    displayName: "Volatility Sizing",
    description: "Volatility-adjusted position sizing",
  },
  VolatilitySqueeze: {
    displayName: "Volatility Squeeze",
    description: "Volatility squeeze breakout strategy",
  },

  // Multi-Indicator Strategies (Phase 12.8)
  MACDRSIComboStrategy: {
    displayName: "MACD + RSI Combo",
    description: "Combines MACD trend and RSI reversal signals",
  },
  TrendMeanRevStrategy: {
    displayName: "Trend + Mean Reversion Hybrid",
    description: "Combines trend following with mean reversion",
  },
  ConfidenceWeightedStrategy: {
    displayName: "Confidence-Weighted",
    description: "Confidence-weighted multi-indicator strategy",
  },
  AdaptiveComboStrategy: {
    displayName: "Adaptive Combination",
    description: "Adaptive combination of multiple indicators",
  },
  EnsembleWeightedStrategy: {
    displayName: "Ensemble Weighted",
    description: "Ensemble of multiple strategies with weights",
  },
  RegimeSwitchingStrategy: {
    displayName: "Regime Switching",
    description: "Regime-aware strategy switching",
  },
  MLEnhancedStrategy: {
    displayName: "ML-Enhanced",
    description: "Machine learning enhanced multi-indicator strategy",
  },
};

// ===========================
// Utility Functions
// ===========================

/**
 * Display Monte Carlo simulation results in the UI
 * @param {Object} monteCarloData - The monte_carlo object from API response
 */
function displayMonteCarloResults(monteCarloData) {
  const mcStatus = document.getElementById("mc-status");
  const mcEmpty = document.getElementById("mc-empty");
  const mcResults = document.getElementById("mc-results");

  if (monteCarloData && monteCarloData.num_simulations > 0) {
    // Monte Carlo ran successfully
    mcStatus.textContent = `(Ran ${monteCarloData.num_simulations} simulations)`;
    mcStatus.style.color = "#10b981";
    mcEmpty.style.display = "none";
    mcResults.style.display = "grid";

    // Update Monte Carlo metrics
    document.getElementById("mc-prob-profit").textContent =
      (monteCarloData.probability_of_profit * 100).toFixed(1) + "%";
    document.getElementById("mc-equity-5th").textContent =
      monteCarloData.equity_5th.toFixed(2);
    document.getElementById("mc-equity-50th").textContent =
      monteCarloData.equity_50th.toFixed(2);
    document.getElementById("mc-equity-95th").textContent =
      monteCarloData.equity_95th.toFixed(2);
    document.getElementById("mc-return-5th").textContent =
      (monteCarloData.return_5th * 100).toFixed(2) + "%";
    document.getElementById("mc-return-50th").textContent =
      (monteCarloData.return_50th * 100).toFixed(2) + "%";
    document.getElementById("mc-return-95th").textContent =
      (monteCarloData.return_95th * 100).toFixed(2) + "%";
    document.getElementById("mc-dd-95th").textContent =
      (monteCarloData.drawdown_95th * 100).toFixed(2) + "%";
    document.getElementById("mc-simulations").textContent =
      monteCarloData.num_simulations;
  } else if (monteCarloData && monteCarloData.num_simulations === 0) {
    // Monte Carlo ran but no simulations (edge case)
    mcStatus.textContent = "(No simulations)";
    mcStatus.style.color = "#f59e0b";
    mcEmpty.style.display = "block";
    mcResults.style.display = "none";
  } else {
    // Monte Carlo not run
    mcStatus.textContent = "(Not Run)";
    mcStatus.style.color = "#888";
    mcEmpty.style.display = "block";
    mcResults.style.display = "none";
  }
}

// Best-effort param schema generator based on common parameter names used by StrategyFactory on the backend.
// For strategies that aren’t mapped here, we’ll show no params (and send an empty params object). The backend
// should either use defaults or reject invalid/missing parameters depending on the endpoint.
function buildDefaultParamSchema(strategyKey) {
  switch (strategyKey) {
    case "GoldenCross":
      return [
        {
          name: "fast_period",
          label: "Fast Period",
          default: 10,
          min: 5,
          max: 50,
          step: 5,
        },
        {
          name: "slow_period",
          label: "Slow Period",
          default: 30,
          min: 20,
          max: 120,
          step: 10,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 1.0,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "Rsi":
      return [
        {
          name: "period",
          label: "RSI Period",
          default: 14,
          min: 5,
          max: 30,
          step: 1,
        },
        {
          name: "lower_bound",
          label: "Oversold Level",
          default: 30,
          min: 10,
          max: 40,
          step: 5,
        },
        {
          name: "upper_bound",
          label: "Overbought Level",
          default: 70,
          min: 60,
          max: 90,
          step: 5,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 3.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "MeanReversion":
      return [
        {
          name: "period",
          label: "BB Period",
          default: 20,
          min: 10,
          max: 50,
          step: 5,
        },
        {
          name: "std_dev",
          label: "Std Deviations",
          default: 2.0,
          min: 1.0,
          max: 3.0,
          step: 0.5,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 3.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "Momentum":
      return [
        {
          name: "ema_period",
          label: "EMA Period",
          default: 50,
          min: 20,
          max: 100,
          step: 10,
        },
        {
          name: "macd_fast",
          label: "MACD Fast",
          default: 12,
          min: 5,
          max: 20,
          step: 1,
        },
        {
          name: "macd_slow",
          label: "MACD Slow",
          default: 26,
          min: 20,
          max: 40,
          step: 1,
        },
        {
          name: "macd_signal",
          label: "Signal Line",
          default: 9,
          min: 5,
          max: 15,
          step: 1,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "Breakout":
      return [
        {
          name: "lookback",
          label: "Lookback (bars)",
          default: 20,
          min: 5,
          max: 200,
          step: 5,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "MACrossover":
      return [
        {
          name: "fast_period",
          label: "Fast Period",
          default: 10,
          min: 5,
          max: 50,
          step: 5,
        },
        {
          name: "slow_period",
          label: "Slow Period",
          default: 30,
          min: 10,
          max: 200,
          step: 10,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "AdaptiveMA":
      return [
        {
          name: "fast_period",
          label: "Fast Period",
          default: 10,
          min: 2,
          max: 50,
          step: 1,
        },
        {
          name: "slow_period",
          label: "Slow Period",
          default: 30,
          min: 5,
          max: 200,
          step: 5,
        },
        {
          name: "price_period",
          label: "Price Period",
          default: 10,
          min: 2,
          max: 100,
          step: 1,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "TripleMA":
      return [
        {
          name: "fast_period",
          label: "Fast Period",
          default: 5,
          min: 2,
          max: 50,
          step: 1,
        },
        {
          name: "medium_period",
          label: "Medium Period",
          default: 15,
          min: 5,
          max: 100,
          step: 5,
        },
        {
          name: "slow_period",
          label: "Slow Period",
          default: 30,
          min: 10,
          max: 200,
          step: 10,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "MacdTrend":
      return [
        {
          name: "fast_period",
          label: "MACD Fast",
          default: 12,
          min: 2,
          max: 30,
          step: 1,
        },
        {
          name: "slow_period",
          label: "MACD Slow",
          default: 26,
          min: 5,
          max: 60,
          step: 1,
        },
        {
          name: "signal_period",
          label: "Signal Line",
          default: 9,
          min: 2,
          max: 30,
          step: 1,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "ParabolicSAR":
      return [
        {
          name: "af_step",
          label: "AF Step",
          default: 0.02,
          min: 0.01,
          max: 0.2,
          step: 0.01,
        },
        {
          name: "af_max",
          label: "AF Max",
          default: 0.2,
          min: 0.05,
          max: 0.5,
          step: 0.05,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "RSIReversionStrategy":
      return [
        {
          name: "rsi_period",
          label: "RSI Period",
          default: 14,
          min: 5,
          max: 30,
          step: 1,
        },
        {
          name: "oversold_threshold",
          label: "Oversold Threshold",
          default: 30.0,
          min: 10.0,
          max: 40.0,
          step: 5.0,
        },
        {
          name: "overbought_threshold",
          label: "Overbought Threshold",
          default: 70.0,
          min: 60.0,
          max: 90.0,
          step: 5.0,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "StochReversionStrategy":
      return [
        {
          name: "k_period",
          label: "%K Period",
          default: 14,
          min: 5,
          max: 30,
          step: 1,
        },
        {
          name: "d_period",
          label: "%D Period",
          default: 3,
          min: 1,
          max: 10,
          step: 1,
        },
        {
          name: "oversold",
          label: "Oversold Threshold",
          default: 20.0,
          min: 10.0,
          max: 30.0,
          step: 5.0,
        },
        {
          name: "overbought",
          label: "Overbought Threshold",
          default: 80.0,
          min: 70.0,
          max: 90.0,
          step: 5.0,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "ZScoreReversionStrategy":
      return [
        {
          name: "lookback_period",
          label: "Lookback Period",
          default: 20,
          min: 5,
          max: 100,
          step: 5,
        },
        {
          name: "entry_zscore",
          label: "Entry Z-Score",
          default: -2.0,
          min: -4.0,
          max: -1.0,
          step: 0.5,
        },
        {
          name: "exit_zscore",
          label: "Exit Z-Score",
          default: 0.0,
          min: 0.0,
          max: 2.0,
          step: 0.5,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "PriceChannelStrategy":
      return [
        {
          name: "lookback_period",
          label: "Lookback Period",
          default: 20,
          min: 5,
          max: 200,
          step: 5,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "KeltnerReversionStrategy":
      return [
        {
          name: "ema_period",
          label: "EMA Period",
          default: 20,
          min: 5,
          max: 50,
          step: 5,
        },
        {
          name: "atr_period",
          label: "ATR Period",
          default: 10,
          min: 5,
          max: 30,
          step: 5,
        },
        {
          name: "atr_multiplier",
          label: "ATR Multiplier",
          default: 2.0,
          min: 1.0,
          max: 3.0,
          step: 0.5,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "StatArbStrategy":
      return [
        {
          name: "lookback_period",
          label: "Lookback Period",
          default: 30,
          min: 10,
          max: 100,
          step: 5,
        },
        {
          name: "entry_zscore",
          label: "Entry Z-Score",
          default: -2.0,
          min: -4.0,
          max: -1.0,
          step: 0.5,
        },
        {
          name: "exit_zscore",
          label: "Exit Z-Score",
          default: 0.0,
          min: 0.0,
          max: 2.0,
          step: 0.5,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "RsiMomentumStrategy":
      return [
        {
          name: "rsi_period",
          label: "RSI Period",
          default: 14,
          min: 5,
          max: 30,
          step: 1,
        },
        {
          name: "momentum_threshold",
          label: "Momentum Threshold",
          default: 50.0,
          min: 40.0,
          max: 60.0,
          step: 5.0,
        },
        {
          name: "strength_threshold",
          label: "Strength Threshold",
          default: 60.0,
          min: 50.0,
          max: 80.0,
          step: 5.0,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 3.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "MACDStrategy":
      return [
        {
          name: "ema_period",
          label: "Trend EMA Period",
          default: 50,
          min: 20,
          max: 100,
          step: 10,
        },
        {
          name: "macd_fast",
          label: "MACD Fast",
          default: 12,
          min: 5,
          max: 20,
          step: 1,
        },
        {
          name: "macd_slow",
          label: "MACD Slow",
          default: 26,
          min: 20,
          max: 40,
          step: 1,
        },
        {
          name: "macd_signal",
          label: "MACD Signal",
          default: 9,
          min: 5,
          max: 15,
          step: 1,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 5.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "RocStrategy":
      return [
        {
          name: "roc_period",
          label: "ROC Period",
          default: 10,
          min: 5,
          max: 30,
          step: 1,
        },
        {
          name: "entry_threshold",
          label: "Entry Threshold (%)",
          default: 2.0,
          min: 1.0,
          max: 5.0,
          step: 0.5,
        },
        {
          name: "exit_threshold",
          label: "Exit Threshold (%)",
          default: -1.0,
          min: -3.0,
          max: 0.0,
          step: 0.5,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 3.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "AdxTrendStrategy":
      return [
        {
          name: "adx_period",
          label: "ADX Period",
          default: 14,
          min: 5,
          max: 30,
          step: 1,
        },
        {
          name: "strong_trend_threshold",
          label: "Strong Trend Threshold",
          default: 25.0,
          min: 20.0,
          max: 40.0,
          step: 5.0,
        },
        {
          name: "weak_trend_threshold",
          label: "Weak Trend Threshold",
          default: 20.0,
          min: 15.0,
          max: 25.0,
          step: 5.0,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 3.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "MomentumFactorStrategy":
      return [
        {
          name: "lookback_period",
          label: "Lookback Period",
          default: 20,
          min: 5,
          max: 50,
          step: 5,
        },
        {
          name: "rsi_period",
          label: "RSI Period",
          default: 14,
          min: 5,
          max: 30,
          step: 1,
        },
        {
          name: "min_factors",
          label: "Min Positive Factors",
          default: 2,
          min: 1,
          max: 3,
          step: 1,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 3.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "VolumeMomentumStrategy":
      return [
        {
          name: "price_ema_period",
          label: "Price EMA Period",
          default: 20,
          min: 5,
          max: 50,
          step: 5,
        },
        {
          name: "volume_period",
          label: "Volume EMA Period",
          default: 20,
          min: 5,
          max: 50,
          step: 5,
        },
        {
          name: "volume_multiplier",
          label: "Volume Multiplier",
          default: 1.5,
          min: 1.0,
          max: 3.0,
          step: 0.1,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 3.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    case "MultiTfMomentumStrategy":
      return [
        {
          name: "fast_ema_period",
          label: "Fast EMA Period",
          default: 20,
          min: 5,
          max: 50,
          step: 5,
        },
        {
          name: "slow_ema_period",
          label: "Slow EMA Period",
          default: 50,
          min: 20,
          max: 200,
          step: 10,
        },
        {
          name: "take_profit",
          label: "Take Profit (%)",
          default: 5.0,
          min: 1.0,
          max: 50.0,
          step: 0.5,
        },
        {
          name: "stop_loss",
          label: "Stop Loss (%)",
          default: 3.0,
          min: 1.0,
          max: 20.0,
          step: 0.5,
        },
      ];
    default:
      return [];
  }
}

async function loadStrategies() {
  const container = document.getElementById("strategy-categories");
  if (container) {
    container.innerHTML = `<div class="strategy-loading">Loading strategies…</div>`;
  }

  try {
    const resp = await fetch(`${API_BASE}/strategies`);
    if (!resp.ok) {
      throw new Error(`Failed to load strategies: HTTP ${resp.status}`);
    }

    const strategies = await resp.json();
    if (!Array.isArray(strategies) || strategies.length === 0) {
      throw new Error("No strategies returned from API");
    }

    AVAILABLE_STRATEGIES = strategies;

    // Build param schemas
    STRATEGY_PARAMS = {};
    strategies.forEach((s) => {
      const key = s && s.name ? s.name : null;
      if (!key) return;
      STRATEGY_PARAMS[key] = buildDefaultParamSchema(key);
    });

    renderStrategyOptions(strategies);

    // Ensure selected strategy exists; otherwise default to first
    const selectedExists = strategies.some(
      (s) => s && s.name === AppState.strategy,
    );
    if (!selectedExists) {
      AppState.strategy = strategies[0].name;
    }

    // Initialize default params for selected strategy
    initParamsForStrategy(AppState.strategy);

    // Refresh parameter UI and context
    updateParamsUI();
    updateContextBar();
  } catch (err) {
    console.error(err);
    if (container) {
      container.innerHTML = `<div class="strategy-no-results">
        <div class="strategy-no-results-icon">⚠️</div>
        <div>Failed to load strategies. Ensure the backend is running and /api/strategies is reachable.</div>
        <div style="font-size: 11px; margin-top: 4px; opacity: 0.7;">${err.message}</div>
      </div>`;
    }
  }
}

// Category display names and icons
const CATEGORY_CONFIG = {
  TrendFollowing: { name: "Trend Following", icon: "📈" },
  MeanReversion: { name: "Mean Reversion", icon: "🔄" },
  Momentum: { name: "Momentum", icon: "⚡" },
  VolatilityBased: { name: "Volatility Based", icon: "🌊" },
  SentimentBased: { name: "Sentiment Based", icon: "💭" },
  MultiIndicator: { name: "Multi Indicator", icon: "🎯" },
  Baseline: { name: "Baseline", icon: "📊" },
};

function getCategoryDisplayName(category) {
  return CATEGORY_CONFIG[category]?.name || category;
}

function getCategoryIcon(category) {
  return CATEGORY_CONFIG[category]?.icon || "📋";
}

// Group strategies by category
function groupStrategiesByCategory(strategies) {
  const grouped = {};
  strategies.forEach((s) => {
    const category = s.category || "Baseline";
    if (!grouped[category]) {
      grouped[category] = [];
    }
    grouped[category].push(s);
  });
  return grouped;
}

// Filter strategies based on search query
function filterStrategies(query) {
  const container = document.getElementById("strategy-categories");
  if (!container) return;

  const lowercaseQuery = query.toLowerCase().trim();

  if (!lowercaseQuery) {
    // Show all categories and strategies
    document.querySelectorAll(".strategy-category").forEach((cat) => {
      cat.classList.remove("hidden");
      cat.classList.remove("open");
      cat.querySelectorAll(".strategy-compact-item").forEach((item) => {
        item.classList.remove("hidden");
      });
    });
    return;
  }

  // Filter strategies and toggle categories
  document.querySelectorAll(".strategy-category").forEach((cat) => {
    const categoryName = cat.dataset.category;
    const catConfig = CATEGORY_CONFIG[categoryName];
    const catMatches =
      catConfig && catConfig.name.toLowerCase().includes(lowercaseQuery);

    let hasVisibleStrategies = false;
    let strategies = cat.querySelectorAll(".strategy-compact-item");

    strategies.forEach((item) => {
      const name = item.dataset.name?.toLowerCase() || "";
      const desc = item.dataset.description?.toLowerCase() || "";
      const matches =
        name.includes(lowercaseQuery) || desc.includes(lowercaseQuery);

      if (catMatches || matches) {
        item.classList.remove("hidden");
        hasVisibleStrategies = true;
      } else {
        item.classList.add("hidden");
      }
    });

    if (hasVisibleStrategies) {
      cat.classList.remove("hidden");
      cat.classList.add("open"); // Auto-expand matching categories
    } else {
      cat.classList.add("hidden");
      cat.classList.remove("open");
    }
  });

  // Show/hide no results message
  const anyVisible =
    document.querySelectorAll(".strategy-category:not(.hidden)").length > 0;
  let noResults = document.getElementById("strategy-no-results");
  if (!anyVisible) {
    if (!noResults) {
      noResults = document.createElement("div");
      noResults.id = "strategy-no-results";
      noResults.className = "strategy-no-results";
      noResults.innerHTML = `
        <div class="strategy-no-results-icon">🔍</div>
        <div>No strategies found matching "${query}"</div>
      `;
      container.appendChild(noResults);
    }
    noResults.style.display = "block";
  } else if (noResults) {
    noResults.style.display = "none";
  }
}

function renderStrategyOptions(strategies) {
  const container = document.getElementById("strategy-categories");
  if (!container) return;

  container.innerHTML = "";

  // Group strategies by category
  const grouped = groupStrategiesByCategory(strategies);
  const categories = Object.keys(grouped).sort((a, b) => {
    // Sort categories: put TrendFollowing and MeanReversion first
    const priority = { TrendFollowing: 0, MeanReversion: 1 };
    const prioA = priority[a] !== undefined ? priority[a] : 100;
    const prioB = priority[b] !== undefined ? priority[b] : 100;
    return prioA - prioB;
  });

  // Render each category
  categories.forEach((category, catIdx) => {
    const categoryStrategies = grouped[category];
    const config = CATEGORY_CONFIG[category] || {
      name: category,
      icon: "📋",
    };

    // Create category accordion
    const categoryDiv = document.createElement("div");
    categoryDiv.className = "strategy-category";
    categoryDiv.dataset.category = category;

    // Category header
    const header = document.createElement("div");
    header.className = "strategy-category-header";
    header.innerHTML = `
      <span class="strategy-category-name">
        <span class="strategy-category-icon">${config.icon}</span>
        ${config.name}
        <span class="strategy-category-count">${categoryStrategies.length}</span>
      </span>
      <span class="strategy-category-chevron">▼</span>
    `;

    header.addEventListener("click", () => {
      const isOpen = categoryDiv.classList.contains("open");
      // Close all categories
      document.querySelectorAll(".strategy-category").forEach((cat) => {
        cat.classList.remove("open");
        cat
          .querySelector(".strategy-category-header")
          ?.classList.remove("active");
      });
      // Open this category if it wasn't open
      if (!isOpen) {
        categoryDiv.classList.add("open");
        header.classList.add("active");
      }
    });

    // Category content
    const content = document.createElement("div");
    content.className = "strategy-category-content";

    const strategiesList = document.createElement("div");
    strategiesList.className = "strategy-category-strategies";

    // Render strategies in this category
    categoryStrategies.forEach((s) => {
      const key = s && s.name ? s.name : null;
      if (!key) return;

      const overrides = STRATEGY_UI_OVERRIDES[key] || {};
      const displayName = overrides.displayName || s.name || key;
      const description = overrides.description || s.description || "";

      const item = document.createElement("label");
      item.className = "strategy-compact-item";
      item.dataset.name = displayName;
      item.dataset.description = description;

      const input = document.createElement("input");
      input.type = "radio";
      input.name = "strategy";
      input.value = key;

      input.addEventListener("change", (e) => {
        AppState.strategy = e.target.value;
        initParamsForStrategy(AppState.strategy);
        updateParamsUI();
        updateContextBar();

        // Update selection visuals
        document.querySelectorAll(".strategy-compact-item").forEach((i) => {
          i.classList.remove("selected");
        });
        item.classList.add("selected");
      });

      const info = document.createElement("div");
      info.className = "strategy-compact-info";

      const nameSpan = document.createElement("span");
      nameSpan.className = "strategy-compact-name";
      nameSpan.textContent = displayName;

      const descSpan = document.createElement("span");
      descSpan.className = "strategy-compact-desc";
      descSpan.textContent = description;

      info.appendChild(nameSpan);
      info.appendChild(descSpan);

      const checkIcon = document.createElement("span");
      checkIcon.className = "strategy-compact-selected-icon";
      checkIcon.textContent = "✓";

      item.appendChild(input);
      item.appendChild(info);
      item.appendChild(checkIcon);

      strategiesList.appendChild(item);
    });

    content.appendChild(strategiesList);
    categoryDiv.appendChild(header);
    categoryDiv.appendChild(content);
    container.appendChild(categoryDiv);

    // Open first category by default
    if (catIdx === 0) {
      categoryDiv.classList.add("open");
      header.classList.add("active");
    }
  });

  // Set initial selection state
  let selectedExists = false;
  document
    .querySelectorAll('.strategy-compact-item input[name="strategy"]')
    .forEach((r) => {
      if (r.value === AppState.strategy) {
        r.checked = true;
        r.closest(".strategy-compact-item").classList.add("selected");
        // Open the category containing the selected strategy
        const category = r.closest(".strategy-category");
        if (category) {
          category.classList.add("open");
          category
            .querySelector(".strategy-category-header")
            ?.classList.add("active");
        }
        selectedExists = true;
      }
    });

  if (!selectedExists) {
    const firstInput = document.querySelector(
      '.strategy-compact-item input[name="strategy"]',
    );
    if (firstInput) {
      firstInput.checked = true;
      AppState.strategy = firstInput.value;
      firstInput.closest(".strategy-compact-item").classList.add("selected");
    }
  }
}

function initParamsForStrategy(strategyKey) {
  const strategyParams = STRATEGY_PARAMS[strategyKey] || [];
  strategyParams.forEach((param) => {
    if (
      AppState.params[param.name] === undefined ||
      AppState.params[param.name] === null
    ) {
      AppState.params[param.name] = param.default;
    }
  });
}

// ===========================
// Initialization
// ===========================

document.addEventListener("DOMContentLoaded", () => {
  console.log("AlphaField Dashboard v2.0 initializing...");

  // Initialize tabs
  initTabNavigation();

  // Initialize strategy selection (dynamic)
  // We still wire up change listeners, but the list itself is populated from /api/strategies
  initStrategySelection();
  loadStrategies();

  // Load available symbols
  loadSymbols();

  // Initialize backtest period buttons
  initPeriodSelector();

  // Initialize interval selector
  initIntervalSelector();

  // Initialize analysis mode toggle
  initAnalysisModeToggle();

  // Check database status
  checkDbStatus();

  // Initialize WebSocket for live trading
  initWebSocket();

  // Load sentiment data
  loadSentiment();

  console.log("Dashboard initialized");
});

// ===========================
// Tab Navigation
// ===========================

function initTabNavigation() {
  // Set initial tab from URL hash or default to 'build'
  const hash = window.location.hash.slice(1);
  if (["build", "backtest", "optimize", "deploy"].includes(hash)) {
    switchTab(hash);
  }
}

function switchTab(tabName) {
  AppState.currentTab = tabName;

  // Update tab buttons
  document.querySelectorAll(".workflow-tab").forEach((tab) => {
    tab.classList.remove("active");
    if (tab.dataset.tab === tabName) {
      tab.classList.add("active");
    }
  });

  // Update view sections
  document.querySelectorAll(".view-section").forEach((section) => {
    section.classList.remove("active");
  });
  document.getElementById(`${tabName}-view`).classList.add("active");

  // Update URL hash
  window.location.hash = tabName;

  // Refresh tab-specific data
  onTabEnter(tabName);
}

function onTabEnter(tabName) {
  switch (tabName) {
    case "build":
      updateParamsUI();
      break;
    case "backtest":
      updateBacktestSummary();
      // Initialize backtest symbol selector when entering backtest tab
      setTimeout(() => initBacktestSymbolSelect(), 100);
      break;
    case "optimize":
      updateSensitivityParams();
      updateOptimizeMLStatus();
      break;
    case "deploy":
      updateDeploySummary();
      loadSentiment();
      break;
  }
}

// Update ML status display in optimize tab
function updateOptimizeMLStatus() {
  const mlDisplay = document.getElementById("opt-display-ml");
  const mlPipelineItem = document.getElementById("ml-pipeline-item");

  if (AppState.mlEnabled) {
    const modelNames = {
      linear: "Linear Regression",
      logistic: "Logistic (Direction)",
      rf: "Random Forest",
    };
    mlDisplay.innerHTML = `<span style="color: #a78bfa;">🤖 ${modelNames[AppState.mlModelType] || AppState.mlModelType}</span>`;
    if (mlPipelineItem) mlPipelineItem.classList.remove("hidden");
  } else {
    mlDisplay.textContent = "Disabled";
    mlDisplay.style.color = "";
    if (mlPipelineItem) mlPipelineItem.classList.add("hidden");
  }
}

// ===========================
// Strategy Selection & Params
// ===========================

function initStrategySelection() {
  // The strategy radios are now rendered dynamically in renderStrategyOptions().
  // This function keeps the existing initialization flow safe if the DOM has
  // not yet been populated.
  const container = document.getElementById("strategy-options");
  if (!container) {
    // Fall back to legacy behavior if container isn’t present
    document.querySelectorAll('input[name="strategy"]').forEach((radio) => {
      radio.addEventListener("change", (e) => {
        AppState.strategy = e.target.value;
        initParamsForStrategy(AppState.strategy);
        updateParamsUI();
        updateContextBar();
      });
    });
  }

  // Initial params setup
  initParamsForStrategy(AppState.strategy);
  updateParamsUI();
}

function updateParamsUI() {
  const container = document.getElementById("build-params");

  // build-params element no longer exists since parameters are set through optimization
  // This function is kept for backward compatibility but does nothing if element doesn't exist
  if (!container) {
    // Still initialize params with defaults even if UI doesn't exist
    const strategyParams = STRATEGY_PARAMS[AppState.strategy] || [];
    strategyParams.forEach((param) => {
      if (!AppState.params[param.name]) {
        AppState.params[param.name] = param.default;
      }
    });
    return;
  }

  const strategyParams = STRATEGY_PARAMS[AppState.strategy] || [];

  if (strategyParams.length === 0) {
    container.innerHTML =
      '<p class="text-muted">No configurable parameters</p>';
    return;
  }

  let html = '<h4>Strategy Parameters</h4><div class="param-row">';

  strategyParams.forEach((param) => {
    const value = AppState.params[param.name] || param.default;
    html += `
            <div class="form-group">
                <label>${param.label}</label>
                <input type="number"
                       class="form-input"
                       id="param-${param.name}"
                       value="${value}"
                       min="${param.min}"
                       max="${param.max}"
                       step="${param.step}"
                       onchange="updateParam('${param.name}', this.value)">
            </div>
        `;
  });

  html += "</div>";
  container.innerHTML = html;

  // Initialize params with defaults
  strategyParams.forEach((param) => {
    if (!AppState.params[param.name]) {
      AppState.params[param.name] = param.default;
    }
  });
}

function updateParam(name, value) {
  AppState.params[name] = parseFloat(value);
}

function getParams() {
  // Return current strategy params
  const strategyParams = STRATEGY_PARAMS[AppState.strategy] || [];
  const params = {};

  strategyParams.forEach((param) => {
    const input = document.getElementById(`param-${param.name}`);
    if (input) {
      params[param.name] = parseFloat(input.value);
    } else {
      params[param.name] = AppState.params[param.name] || param.default;
    }
  });

  return params;
}

// ===========================
// Context Bar
// ===========================

function updateContextBar() {
  // Strategy with ML indicator
  const backendKey = AppState.strategy;
  const override = STRATEGY_UI_OVERRIDES[backendKey];
  const friendly = (override && override.displayName) || backendKey;

  let strategyText = `${friendly} (${backendKey})`;
  if (AppState.mlEnabled) {
    strategyText += " + 🤖ML";
  }
  document.getElementById("ctx-strategy").textContent = strategyText;
  document.getElementById("ctx-symbol").textContent = AppState.symbol || "—";

  let status = "Configure strategy to begin";
  if (AppState.strategy && AppState.symbol) {
    status = "Ready to test";
  }
  if (AppState.backtestResults) {
    const returnVal =
      AppState.backtestResults.total_return ||
      (AppState.backtestResults.metrics &&
        AppState.backtestResults.metrics.total_return) ||
      0;
    status = `Tested: ${(returnVal * 100).toFixed(1)}% return`;
  }
  if (AppState.isTrading) {
    status = "🟢 Trading Active";
  }
  document.getElementById("ctx-status").textContent = status;
}

// ===========================
// Symbol Loading
// ===========================

// ===========================
// Symbol Loading & Custom Select
// ===========================

// State for symbol data
let allSymbols = [];

async function loadSymbols() {
  try {
    const res = await fetch(`${API_BASE}/data/pairs`);
    const data = await res.json();

    // Handle inconsistent API responses (array of strings vs object with pairs array)
    if (Array.isArray(data)) {
      allSymbols = data;
    } else if (data.pairs && Array.isArray(data.pairs)) {
      allSymbols = data.pairs.map((p) =>
        typeof p === "string" ? p : p.symbol,
      );
    } else {
      // Fallback for unexpected structure
      allSymbols = [];
      console.warn("Unexpected symbol data structure:", data);
    }

    // Note: Symbol selection is now done through asset categories in Build tab
    // The old Build tab and Optimize tab symbol selectors have been removed
    // Symbol selection for backtest is initialized when entering the Backtest tab
    console.log("Loaded symbols:", allSymbols.length);
  } catch (e) {
    console.error("Failed to load symbols:", e);
    // Fallback symbols
    allSymbols = ["BTCUSDT", "ETHUSDT"];
  }
}

function initCustomSelect() {
  // This function is deprecated - Build tab no longer has individual symbol selection
  // Symbol selection is now done through asset categories
  const wrapper = document.getElementById("symbol-select-wrapper");
  const trigger = document.getElementById("symbol-trigger");
  const optionsList = document.getElementById("symbol-options-list");
  const searchInput = document.getElementById("symbol-search-input");
  const hiddenInput = document.getElementById("build-symbol");
  const selectedText = document.getElementById("selected-symbol-text");

  // Early return if elements don't exist (they were removed in workflow restructuring)
  if (
    !wrapper ||
    !trigger ||
    !optionsList ||
    !searchInput ||
    !hiddenInput ||
    !selectedText
  ) {
    console.log(
      "Build tab symbol selector elements not found (expected - using asset categories now)",
    );
    return;
  }

  // Toggle dropdown
  trigger.addEventListener("click", () => {
    wrapper.classList.toggle("open");
    if (wrapper.classList.contains("open")) {
      searchInput.focus();
    }
  });

  // Close when clicking outside
  document.addEventListener("click", (e) => {
    if (!wrapper.contains(e.target)) {
      wrapper.classList.remove("open");
    }
  });

  // Filter function
  function filterSymbols(query) {
    const q = query.toLowerCase();
    const POPULAR = [
      "BTC",
      "ETH",
      "SOL",
      "XRP",
      "ADA",
      "DOGE",
      "AVAX",
      "DOT",
      "LINK",
      "MATIC",
    ];

    let filtered = allSymbols.filter((s) => s.toLowerCase().includes(q));

    // Sort: Popular first, then alphabetical
    filtered.sort((a, b) => {
      const aPop = POPULAR.indexOf(a);
      const bPop = POPULAR.indexOf(b);

      if (aPop !== -1 && bPop !== -1) return aPop - bPop;
      if (aPop !== -1) return -1;
      if (bPop !== -1) return 1;
      return a.localeCompare(b);
    });

    renderOptions(filtered);
  }

  // Render options to the list
  function renderOptions(symbols) {
    optionsList.innerHTML = "";

    if (symbols.length === 0) {
      optionsList.innerHTML =
        '<div class="option" style="cursor: default; opacity: 0.5;">No matches found</div>';
      return;
    }

    // Limit rendering for performance if list is huge
    const displaySymbols = symbols.slice(0, 100);

    displaySymbols.forEach((symbol) => {
      const div = document.createElement("div");
      div.className = "option";
      if (symbol === AppState.symbol) div.classList.add("selected");

      div.innerHTML = `
                <span class="option-ticker">${symbol}</span>
                <span class="option-name">USDT</span>
            `;

      div.addEventListener("click", () => {
        selectSymbol(symbol);
      });

      optionsList.appendChild(div);
    });

    if (symbols.length > 100) {
      const more = document.createElement("div");
      more.className = "option";
      more.style.opacity = "0.5";
      more.style.fontStyle = "italic";
      more.textContent = `...and ${symbols.length - 100} more`;
      optionsList.appendChild(more);
    }
  }

  // Selection handler
  function selectSymbol(symbol) {
    AppState.symbol = symbol;
    hiddenInput.value = symbol;
    selectedText.textContent = symbol;
    wrapper.classList.remove("open");

    // Update UI state
    updateContextBar();

    // Re-render to update selected styling
    filterSymbols(searchInput.value);
  }

  // Search input handler
  searchInput.addEventListener("input", (e) => {
    filterSymbols(e.target.value);
  });

  // Initial render
  filterSymbols("");

  // Select default or first available
  if (!AppState.symbol && allSymbols.length > 0) {
    // Default to BTC if available, or first item
    const defaultSym = allSymbols.includes("BTC") ? "BTC" : allSymbols[0];
    selectSymbol(defaultSym);
  } else if (AppState.symbol) {
    selectSymbol(AppState.symbol);
  }
}

// ===========================
// Workflow Navigation
// ===========================

function goToBacktest() {
  AppState.params = getParams();
  updateContextBar();
  switchTab("backtest");
}

function goToOptimize() {
  switchTab("optimize");
}

function goToDeploy() {
  switchTab("deploy");
}

// ===========================
// ML Enhancement Controls
// ===========================

function toggleMLMode() {
  const checkbox = document.getElementById("ml-enabled");
  const optionsDiv = document.getElementById("ml-options");

  AppState.mlEnabled = checkbox.checked;

  if (checkbox.checked) {
    optionsDiv.classList.remove("hidden");
  } else {
    optionsDiv.classList.add("hidden");
  }

  updateContextBar();
}

function updateMLModelType(value) {
  AppState.mlModelType = value;
}

function updateMLConfidence(value) {
  AppState.mlConfidence = parseInt(value) / 100;
  document.getElementById("ml-confidence-value").textContent = `${value}%`;
}

// Train ML model during optimization (multi-symbol with random subsets)
async function trainMLModel(symbols, interval, days) {
  if (!AppState.mlEnabled) return null;

  // Ensure symbols is an array
  const symbolList = Array.isArray(symbols) ? symbols : [symbols];

  console.log("Training ML model on multiple symbols...", {
    symbols: symbolList,
    interval,
    days,
  });

  try {
    const res = await fetch(`${API_BASE}/ml/train/multi`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        model_type: AppState.mlModelType,
        symbols: symbolList,
        interval: interval,
        days: days,
        samples_per_symbol: 200,
        prediction_horizon: 1,
      }),
    });

    const data = await res.json();

    if (!data.success) {
      console.warn("ML training failed:", data.error);
      return null;
    }

    AppState.mlTrainingResults = data;
    console.log("ML multi-symbol training complete:", data);
    return data;
  } catch (e) {
    console.error("ML training error:", e);
    return null;
  }
}

// Validate ML model with walk-forward
async function validateMLModel(symbol, interval, days) {
  if (!AppState.mlEnabled || !AppState.mlTrainingResults) return null;

  try {
    const res = await fetch(`${API_BASE}/ml/validate`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        symbol: symbol,
        interval: interval,
        days: days,
        model_type: AppState.mlModelType,
        train_window_days: 100,
        test_window_days: 30,
      }),
    });

    const data = await res.json();
    return data.success ? data : null;
  } catch (e) {
    console.error("ML validation error:", e);
    return null;
  }
}

// ===========================
// Backtest
// ===========================

function initPeriodSelector() {
  document.querySelectorAll(".period-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      document
        .querySelectorAll(".period-btn")
        .forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      AppState.backtestDays = parseInt(btn.dataset.days);
    });
  });
}

function initIntervalSelector() {
  document.querySelectorAll(".interval-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      document
        .querySelectorAll(".interval-btn")
        .forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      AppState.backtestInterval = btn.dataset.interval;
    });
  });
}

function updateBacktestSummary() {
  document.getElementById("bt-summary-strategy").textContent =
    AppState.strategy;
  document.getElementById("bt-summary-symbol").textContent =
    AppState.symbol || "—";

  const params = getParams();
  const paramsStr = Object.entries(params)
    .map(([k, v]) => `${k}=${v}`)
    .join(", ");
  document.getElementById("bt-summary-params").textContent = paramsStr || "—";
}

async function runBacktest() {
  const btn = document.getElementById("btn-run-backtest");
  btn.disabled = true;
  btn.innerHTML = "⏳ Running...";

  const placeholderEl = document.getElementById("backtest-results-placeholder");
  const contentEl = document.getElementById("backtest-results-content");

  try {
    // Get symbol from backtest tab selector
    const backtestSymbol = document.getElementById("backtest-symbol").value;
    if (!backtestSymbol) {
      throw new Error("Please select a symbol to backtest");
    }

    // Update AppState with selected symbol
    AppState.symbol = backtestSymbol;

    const params = getParams();

    const res = await fetch(`${API_BASE}/backtest/run`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        strategy: AppState.strategy,
        symbol: backtestSymbol,
        interval: AppState.backtestInterval,
        days: AppState.backtestDays,
        params: params,
        include_benchmark: true,
      }),
    });

    const data = await res.json();

    if (data.error) throw new Error(data.error);
    if (!data.metrics) throw new Error("No metrics returned");

    // Store results
    AppState.backtestResults = data.metrics;

    // Update reports data
    currentTrades = data.trades || [];
    currentEquity = data.equity_curve
      ? data.equity_curve.map((p) => [
          new Date(p.timestamp).getTime(),
          p.equity,
        ])
      : [];

    // Auto-update report if tab is active
    if (AppState.currentTab === "reports") {
      updatePerformanceReport();
      updateJournal(currentTrades);
    }

    // Update metrics display
    updateMetric("bt-return", data.metrics.total_return * 100, "%");
    document.getElementById("bt-sharpe").textContent =
      data.metrics.sharpe_ratio.toFixed(2);
    updateMetric("bt-drawdown", -data.metrics.max_drawdown * 100, "%", true);
    document.getElementById("bt-winrate").textContent =
      `${(data.metrics.win_rate * 100).toFixed(0)}%`;
    document.getElementById("bt-trades").textContent =
      data.metrics.total_trades;

    // Alpha calculation
    // Fix: Benchmark metrics are nested in data.benchmark.comparison and use benchmark_return
    if (
      data.benchmark &&
      data.benchmark.comparison &&
      data.benchmark.comparison.benchmark_return !== undefined
    ) {
      const alpha =
        (data.metrics.total_return -
          data.benchmark.comparison.benchmark_return) *
        100;
      if (isNaN(alpha)) {
        document.getElementById("bt-alpha").textContent = "—";
      } else {
        updateMetric("bt-alpha", alpha, "%", false);
      }
    } else {
      console.warn(
        "Benchmark data missing or invalid structure",
        data.benchmark,
      );
      document.getElementById("bt-alpha").textContent = "—";
    }

    // Render equity chart with trade markers
    renderEquityChart(data.equity_curve, data.benchmark?.curve, data.trades);

    // Store backtest results for trade markers on price chart
    AppState.backtestResults = data;

    // Update interactive price chart
    updateChartIndicators();

    // Show results
    placeholderEl.classList.add("hidden");
    contentEl.classList.remove("hidden");

    updateContextBar();
  } catch (e) {
    alert("Backtest failed: " + e.message);
    console.error(e);
  }

  btn.disabled = false;
  btn.innerHTML = "▶️ Run Backtest";
}
// ===========================
// Parameter Optimization
// ===========================

// Store optimization results for visualization
let lastOptimizeResults = null;

async function optimizeParams() {
  const btn = document.getElementById("btn-auto-optimize");
  const runBtn = document.getElementById("btn-run-backtest");
  btn.disabled = true;
  runBtn.disabled = true;
  btn.innerHTML = "⏳ Optimizing...";

  try {
    const res = await fetch(`${API_BASE}/backtest/optimize`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        strategy: AppState.strategy,
        symbol: AppState.symbol,
        interval: AppState.backtestInterval,
        days: AppState.backtestDays,
      }),
    });

    const data = await res.json();

    if (!data.success) {
      throw new Error(data.error || "Optimization failed");
    }

    // Store results for later reference
    lastOptimizeResults = data;

    // Apply optimized params to AppState
    AppState.params = data.optimized_params;

    // Update UI inputs with optimized values
    const strategyParams = STRATEGY_PARAMS[AppState.strategy] || [];
    strategyParams.forEach((param) => {
      const value = data.optimized_params[param.name];
      if (value !== undefined) {
        const input = document.getElementById(`param-${param.name}`);
        if (input) {
          input.value = value;
        }
      }
    });

    // Update backtest summary
    updateBacktestSummary();

    // Show detailed results with visualization
    const returnPct = (data.best_return * 100).toFixed(2);
    const sharpe = data.best_sharpe.toFixed(2);
    const winRate = (data.best_win_rate * 100).toFixed(0);
    const drawdown = (data.best_max_drawdown * 100).toFixed(2);
    const score = data.best_score.toFixed(3);

    alert(`✨ Optimization Complete!

📊 Best Composite Score: ${score}
   - Sharpe Ratio: ${sharpe}
   - Return: ${returnPct}%
   - Win Rate: ${winRate}%
   - Max Drawdown: ${drawdown}%

🔢 Tested ${data.iterations} combinations in ${data.elapsed_ms}ms

Parameters have been applied.
Click "Run Backtest" to see full results, or go to "Optimize" tab to view all sweep results.`);

    // Navigate to Optimize tab and show results
    switchTab("optimize");
    renderOptimizationSweepChart(data);
  } catch (e) {
    alert("Optimization failed: " + e.message);
    console.error(e);
  }

  btn.disabled = false;
  runBtn.disabled = false;
  btn.innerHTML = "🎯 Auto-Optimize";
}

// Render optimization sweep results as a scatter chart
function renderOptimizationSweepChart(data) {
  if (!data.sweep_results || data.sweep_results.length === 0) {
    console.warn("No sweep results to visualize");
    return;
  }

  // Sort by score for color gradient
  const results = [...data.sweep_results].sort((a, b) => a.score - b.score);

  // Create scatter plot: Sharpe vs Return, colored by score
  const trace = {
    x: results.map((r) => r.sharpe),
    y: results.map((r) => r.total_return * 100),
    mode: "markers",
    type: "scatter",
    marker: {
      size: 10,
      color: results.map((r) => r.score),
      colorscale: "Viridis",
      colorbar: {
        title: "Score",
        titleside: "right",
      },
      line: { color: "white", width: 1 },
    },
    text: results.map((r) => {
      const params = Object.entries(r.params)
        .map(([k, v]) => `${k}: ${v}`)
        .join("<br>");
      return `Score: ${r.score.toFixed(3)}<br>Sharpe: ${r.sharpe.toFixed(2)}<br>Return: ${(r.total_return * 100).toFixed(2)}%<br>Win Rate: ${(r.win_rate * 100).toFixed(0)}%<br>Drawdown: ${(r.max_drawdown * 100).toFixed(2)}%<br>Trades: ${r.total_trades}<br><br>${params}`;
    }),
    hoverinfo: "text",
  };

  // Mark best result
  const best = results[results.length - 1]; // Highest score
  const bestTrace = {
    x: [best.sharpe],
    y: [best.total_return * 100],
    mode: "markers",
    type: "scatter",
    name: "Best",
    marker: {
      size: 18,
      color: "#10b981",
      symbol: "star",
      line: { color: "white", width: 2 },
    },
    hoverinfo: "skip",
  };

  const layout = {
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor: "rgba(0,0,0,0)",
    font: { color: "#94a3b8" },
    margin: { t: 40, r: 80, b: 50, l: 60 },
    xaxis: {
      title: "Sharpe Ratio",
      gridcolor: "rgba(255, 255, 255, 0.05)",
      zeroline: true,
      zerolinecolor: "rgba(255, 255, 255, 0.2)",
    },
    yaxis: {
      title: "Total Return (%)",
      gridcolor: "rgba(255, 255, 255, 0.05)",
      zeroline: true,
      zerolinecolor: "rgba(255, 255, 255, 0.2)",
    },
    showlegend: false,
    title: {
      text: `Parameter Sweep Results (${data.iterations} combinations)`,
      font: { color: "#e2e8f0", size: 14 },
    },
  };

  Plotly.newPlot("optimize-chart", [trace, bestTrace], layout, {
    responsive: true,
  });

  // Update title
  document.getElementById("optimize-chart-title").textContent =
    "Parameter Sweep Analysis";

  // Show sensitivity results block
  document
    .getElementById("optimize-results-placeholder")
    .classList.add("hidden");
  document.getElementById("wfa-results").classList.add("hidden");
  document.getElementById("sensitivity-results").classList.remove("hidden");

  // Update metrics - format params nicely
  const paramsText = Object.entries(data.optimized_params)
    .map(([k, v]) => `${k}=${Number(v).toFixed(1)}`)
    .join(", ");
  document.getElementById("sens-best-param").textContent = paramsText;
  document.getElementById("sens-max-sharpe").textContent =
    data.best_sharpe.toFixed(2);
}

// Store best params from last optimization
function applyBestParams() {
  if (lastOptimizeResults && lastOptimizeResults.optimized_params) {
    AppState.params = lastOptimizeResults.optimized_params;
    updateParamsUI();
    updateBacktestSummary();
    switchTab("backtest");
  } else {
    alert("No optimization results available. Run Auto-Optimize first.");
  }
}

function updateMetric(id, value, suffix = "", invert = false) {
  const el = document.getElementById(id);
  if (!el) return;

  // Handle NaN or undefined values
  if (isNaN(value) || value === undefined || value === null) {
    el.textContent = "—";
    el.className = "metric-value";
    return;
  }

  el.textContent = `${value >= 0 ? "+" : ""}${value.toFixed(2)}${suffix}`;
  el.className = `metric-value ${invert ? (value <= 0 ? "positive" : "negative") : value >= 0 ? "positive" : "negative"}`;
}

function renderEquityChart(equityCurve, benchmarkCurve, trades = []) {
  if (!equityCurve || equityCurve.length === 0) return;

  const timestamps = equityCurve.map((p) => new Date(p.timestamp));
  const equity = equityCurve.map((p) => p.equity);

  const traces = [
    {
      x: timestamps,
      y: equity,
      type: "scatter",
      mode: "lines",
      name: "Strategy",
      line: { color: "#3b82f6", width: 2 },
      hoverlabel: { bgcolor: "#1e293b" },
    },
  ];

  if (benchmarkCurve && benchmarkCurve.length > 0) {
    traces.push({
      x: benchmarkCurve.map((p) => new Date(p.timestamp)),
      y: benchmarkCurve.map((p) => p.equity),
      type: "scatter",
      mode: "lines",
      name: "Buy & Hold",
      line: { color: "#6b7280", width: 1, dash: "dot" },
      hoverlabel: { bgcolor: "#1e293b" },
    });
  }

  // Add trade entry markers (green triangles pointing up)
  if (trades && trades.length > 0) {
    const entryX = [];
    const entryY = [];
    const entryText = [];
    const exitX = [];
    const exitY = [];
    const exitText = [];

    trades.forEach((trade) => {
      // Find closest equity point for entry
      const entryTime = new Date(trade.entry_time);
      const exitTime = new Date(trade.exit_time);

      // Find equity value at entry time
      let entryEquity = null;
      for (let i = 0; i < equityCurve.length; i++) {
        if (new Date(equityCurve[i].timestamp) >= entryTime) {
          entryEquity = equityCurve[i].equity;
          break;
        }
      }
      if (entryEquity === null && equityCurve.length > 0) {
        entryEquity = equityCurve[0].equity;
      }

      // Find equity value at exit time
      let exitEquity = null;
      for (let i = 0; i < equityCurve.length; i++) {
        if (new Date(equityCurve[i].timestamp) >= exitTime) {
          exitEquity = equityCurve[i].equity;
          break;
        }
      }
      if (exitEquity === null && equityCurve.length > 0) {
        exitEquity = equityCurve[equityCurve.length - 1].equity;
      }

      entryX.push(entryTime);
      entryY.push(entryEquity);
      entryText.push(
        `Entry: $${trade.entry_price?.toFixed(2) || "?"}<br>Qty: ${trade.quantity?.toFixed(4) || "?"}`,
      );

      exitX.push(exitTime);
      exitY.push(exitEquity);
      const pnlColor = (trade.pnl || 0) >= 0 ? "#10b981" : "#ef4444";
      exitText.push(
        `Exit: $${trade.exit_price?.toFixed(2) || "?"}<br>Reason: ${trade.exit_reason || "Unknown"}<br>PnL: <span style="color:${pnlColor}">$${trade.pnl?.toFixed(2) || "?"}</span>`,
      );
    });

    // Entry markers
    traces.push({
      x: entryX,
      y: entryY,
      type: "scatter",
      mode: "markers",
      name: "Entry",
      marker: {
        color: "#10b981",
        size: 12,
        symbol: "triangle-up",
        line: { color: "white", width: 1 },
      },
      text: entryText,
      hoverinfo: "text+x",
      hoverlabel: { bgcolor: "#1e293b", bordercolor: "#10b981" },
    });

    // Exit markers
    traces.push({
      x: exitX,
      y: exitY,
      type: "scatter",
      mode: "markers",
      name: "Exit",
      marker: {
        color: "#ef4444",
        size: 12,
        symbol: "triangle-down",
        line: { color: "white", width: 1 },
      },
      text: exitText,
      hoverinfo: "text+x",
      hoverlabel: { bgcolor: "#1e293b", bordercolor: "#ef4444" },
    });
  }

  Plotly.newPlot(
    "equity-chart",
    traces,
    {
      paper_bgcolor: "rgba(0,0,0,0)",
      plot_bgcolor: "rgba(0,0,0,0)",
      font: { color: "#94a3b8" },
      margin: { t: 20, r: 20, b: 40, l: 60 },
      xaxis: { gridcolor: "rgba(255, 255, 255, 0.05)" },
      yaxis: {
        gridcolor: "rgba(255, 255, 255, 0.05)",
        title: "Equity ($)",
      },
      legend: { x: 0, y: 1, orientation: "h" },
    },
    { responsive: true },
  );
}

// ===========================
// Optimize & Tune
// ===========================

function initAnalysisModeToggle() {
  document.querySelectorAll('input[name="analysis-mode"]').forEach((radio) => {
    radio.addEventListener("change", (e) => {
      const mode = e.target.value;

      // Update mode option styles
      document.querySelectorAll(".mode-option").forEach((opt) => {
        opt.classList.remove("active");
      });
      e.target.closest(".mode-option").classList.add("active");

      // Show/hide config sections
      document
        .getElementById("comprehensive-config")
        .classList.toggle("hidden", mode !== "comprehensive");
      document
        .getElementById("wfa-config")
        .classList.toggle("hidden", mode !== "walkforward");
      document
        .getElementById("sensitivity-config")
        .classList.toggle("hidden", mode !== "sensitivity");

      // Hide results and show placeholder
      document.getElementById("comprehensive-results").classList.add("hidden");
      document.getElementById("wfa-results").classList.add("hidden");
      document.getElementById("sensitivity-results").classList.add("hidden");
      document
        .getElementById("optimize-results-placeholder")
        .classList.remove("hidden");
    });
  });
}

function updateSensitivityParams() {
  const strategyParams = STRATEGY_PARAMS[AppState.strategy] || [];
  const select1 = document.getElementById("sens-param-1");
  const select2 = document.getElementById("sens-param-2");

  // Return early if elements don't exist (removed in new workflow)
  if (!select1 || !select2) {
    console.log(
      "Sensitivity parameter selectors not found (expected in new workflow)",
    );
    return;
  }

  select1.innerHTML = "";
  select2.innerHTML = '<option value="">None (1D Sweep)</option>';

  strategyParams.forEach((param) => {
    select1.innerHTML += `<option value="${param.name}"
            data-min="${param.min}" data-max="${param.max}" data-step="${param.step}">
            ${param.label}
        </option>`;
    select2.innerHTML += `<option value="${param.name}"
            data-min="${param.min}" data-max="${param.max}" data-step="${param.step}">
            ${param.label}
        </option>`;
  });
}

async function runWalkForward() {
  const chartContainer = document.getElementById("optimize-chart");
  const resultsPlaceholder = document.getElementById(
    "optimize-results-placeholder",
  );
  const originalContent = chartContainer.innerHTML;
  const originalPlaceholder = resultsPlaceholder.innerHTML;

  // Show loading state with more detail in Chart
  chartContainer.innerHTML = `
        <div class="placeholder-content">
            <span class="placeholder-icon">⏳</span>
            <span>Running Walk-Forward Analysis...</span>
            <span style="font-size: 12px; margin-top: 8px; opacity: 0.7;">Fetching 4 years of data and simulating. This may take 10-20 seconds.</span>
        </div>`;

  // Update Results Placeholder to match
  resultsPlaceholder.innerHTML = `
        <span class="placeholder-icon">⏳</span>
        <span>Calculating Metrics...</span>
    `;

  // Disable button to prevent double-submit
  const btn = document.querySelector('button[onclick="runWalkForward()"]');
  if (btn) btn.disabled = true;

  try {
    // Get symbol from optimize tab
    const optimizeSymbol = document.getElementById("optimize-symbol").value;
    if (!optimizeSymbol) {
      throw new Error("Please select a trading symbol first");
    }

    // Update AppState
    AppState.symbol = optimizeSymbol;

    const res = await fetch(`${API_BASE}/walk-forward`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        strategy: AppState.strategy,
        symbol: optimizeSymbol,
        interval: "1h",
        params: getParams(),
      }),
    });

    const data = await res.json();

    if (!data.success) {
      throw new Error(data.error || "Analysis failed");
    }

    const result = data.result;

    // Update metrics
    document.getElementById("wfa-stability").textContent = (
      result.stability_score || 0
    ).toFixed(2);
    updateMetric(
      "wfa-avg-return",
      (result.aggregate_oos?.mean_return || 0) * 100,
      "%",
    );
    document.getElementById("wfa-win-rate").textContent =
      `${((result.aggregate_oos?.win_rate || 0) * 100).toFixed(0)}%`;
    document.getElementById("wfa-oos-sharpe").textContent = (
      result.aggregate_oos?.mean_sharpe || 0
    ).toFixed(2);

    // Show results block
    resultsPlaceholder.classList.add("hidden");
    document.getElementById("wfa-results").classList.remove("hidden");
    document.getElementById("sensitivity-results").classList.add("hidden");

    // Render chart
    renderWFAChart(result.windows);

    document.getElementById("optimize-chart-title").textContent =
      "Walk-Forward Window Returns";
  } catch (e) {
    console.error("WFA Error:", e);
    // Show error in the chart container
    chartContainer.innerHTML = `
            <div class="placeholder-content" style="color: #ef4444;">
                <span class="placeholder-icon">⚠️</span>
                <span>Analysis Failed</span>
                <span style="font-size: 14px; margin-top: 8px; max-width: 80%; text-align: center;">${e.message}</span>
                <button class="btn-primary" style="margin-top: 16px;" onclick="runWalkForward()">Try Again</button>
            </div>`;

    // Show error in results placeholder too
    resultsPlaceholder.innerHTML = `
            <span class="placeholder-icon" style="color: #ef4444;">⚠️</span>
            <span style="color: #ef4444;">Analysis Failed</span>
        `;
    resultsPlaceholder.classList.remove("hidden"); // Ensure it's visible if we came from success state
    document.getElementById("wfa-results").classList.add("hidden");
  } finally {
    if (btn) btn.disabled = false;
  }
}

function renderWFAChart(windows) {
  const container = document.getElementById("optimize-chart");
  container.innerHTML = ""; // Clear loading state explicitly

  if (!windows || windows.length === 0) {
    container.innerHTML =
      '<div class="placeholder-content"><span class="placeholder-icon">📊</span><span>No window data available</span></div>';
    return;
  }

  const windowLabels = windows.map((w, i) => `Window ${i + 1}`);
  const testReturns = windows.map(
    (w) => (w.test_metrics?.total_return || 0) * 100,
  );
  const trainReturns = windows.map(
    (w) => (w.train_metrics?.total_return || 0) * 100,
  );
  const barColors = testReturns.map((r) => (r >= 0 ? "#10b981" : "#ef4444"));

  Plotly.newPlot(
    "optimize-chart",
    [
      {
        x: windowLabels,
        y: trainReturns,
        type: "bar",
        name: "Train Return",
        marker: { color: "rgba(59, 130, 246, 0.6)" },
      },
      {
        x: windowLabels,
        y: testReturns,
        type: "bar",
        name: "Test Return (OOS)",
        marker: { color: barColors },
      },
    ],
    {
      paper_bgcolor: "rgba(0,0,0,0)",
      plot_bgcolor: "rgba(0,0,0,0)",
      font: { color: "#94a3b8" },
      margin: { t: 40, r: 20, b: 60, l: 60 },
      xaxis: { gridcolor: "rgba(255, 255, 255, 0.05)", tickangle: -45 },
      yaxis: {
        gridcolor: "rgba(255, 255, 255, 0.05)",
        title: "Return (%)",
      },
      barmode: "group",
      legend: { x: 0, y: 1.15, orientation: "h" },
    },
    { responsive: true },
  );
}

async function runSensitivity() {
  const chartContainer = document.getElementById("optimize-chart");
  chartContainer.innerHTML =
    '<div class="placeholder-content"><span class="placeholder-icon">⏳</span><span>Running Sensitivity Analysis...</span></div>';

  const p1Select = document.getElementById("sens-param-1");
  const p2Select = document.getElementById("sens-param-2");

  const p1Opt = p1Select.options[p1Select.selectedIndex];
  const param = {
    name: p1Select.value,
    min: parseFloat(p1Opt.dataset.min),
    max: parseFloat(p1Opt.dataset.max),
    step: parseFloat(p1Opt.dataset.step),
  };

  let param_y = null;
  if (p2Select.value) {
    const p2Opt = p2Select.options[p2Select.selectedIndex];
    param_y = {
      name: p2Select.value,
      min: parseFloat(p2Opt.dataset.min),
      max: parseFloat(p2Opt.dataset.max),
      step: parseFloat(p2Opt.dataset.step),
    };
  }

  const baseParams = getParams();
  const fixed_params = { ...baseParams };
  delete fixed_params[param.name];
  if (param_y) delete fixed_params[param_y.name];

  try {
    // Get symbol from optimize tab
    const optimizeSymbol = document.getElementById("optimize-symbol").value;
    if (!optimizeSymbol) {
      throw new Error("Please select a trading symbol first");
    }

    // Update AppState
    AppState.symbol = optimizeSymbol;

    const res = await fetch(`${API_BASE}/sensitivity`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        strategy: AppState.strategy,
        symbol: optimizeSymbol,
        interval: "1h",
        days: 180,
        param,
        param_y,
        fixed_params,
      }),
    });

    const data = await res.json();
    if (!data.success) throw new Error(data.error);

    // Update metrics
    if (data.results && data.results.length > 0) {
      const bestResult = data.results.reduce(
        (best, r) => (r.sharpe_ratio > best.sharpe_ratio ? r : best),
        data.results[0],
      );

      let bestText = Object.entries(bestResult.params)
        .map(([k, v]) => `${k}: ${v}`)
        .join(", ");
      document.getElementById("sens-best-param").textContent = bestText;
      document.getElementById("sens-max-sharpe").textContent =
        bestResult.sharpe_ratio.toFixed(3);

      // Store best params for "Apply" button
      AppState.bestSensitivityParams = bestResult.params;
    }

    // Show results block
    document
      .getElementById("optimize-results-placeholder")
      .classList.add("hidden");
    document.getElementById("sensitivity-results").classList.remove("hidden");
    document.getElementById("wfa-results").classList.add("hidden");

    // Render chart
    if (data.heatmap) {
      renderHeatmap(data.heatmap);
      document.getElementById("optimize-chart-title").textContent =
        `Sharpe Ratio: ${data.heatmap.x_param} vs ${data.heatmap.y_param}`;
    } else if (data.results && data.results.length > 0) {
      renderSensitivityLineChart(data.results, param.name);
      document.getElementById("optimize-chart-title").textContent =
        `Parameter Sensitivity: ${param.name}`;
    }
  } catch (e) {
    alert("Sensitivity Analysis failed: " + e.message);
    console.error(e);
  }
}

function renderSensitivityLineChart(results, paramName) {
  document.getElementById("optimize-chart").innerHTML = ""; // Clear loading state

  const sorted = [...results].sort(
    (a, b) => a.params[paramName] - b.params[paramName],
  );

  const xValues = sorted.map((r) => r.params[paramName]);
  const sharpeValues = sorted.map((r) => r.sharpe_ratio);
  const returnValues = sorted.map((r) => r.total_return * 100);

  Plotly.newPlot(
    "optimize-chart",
    [
      {
        x: xValues,
        y: sharpeValues,
        type: "scatter",
        mode: "lines+markers",
        name: "Sharpe Ratio",
        line: { color: "#3b82f6", width: 2 },
        marker: { size: 8 },
        yaxis: "y",
      },
      {
        x: xValues,
        y: returnValues,
        type: "scatter",
        mode: "lines+markers",
        name: "Return (%)",
        line: { color: "#10b981", width: 2 },
        marker: { size: 8 },
        yaxis: "y2",
      },
    ],
    {
      paper_bgcolor: "rgba(0,0,0,0)",
      plot_bgcolor: "rgba(0,0,0,0)",
      font: { color: "#94a3b8" },
      margin: { t: 50, r: 80, b: 60, l: 60 },
      xaxis: { title: paramName, gridcolor: "rgba(255, 255, 255, 0.05)" },
      yaxis: {
        title: "Sharpe Ratio",
        gridcolor: "rgba(255, 255, 255, 0.05)",
        side: "left",
      },
      yaxis2: {
        title: "Return (%)",
        overlaying: "y",
        side: "right",
        gridcolor: "rgba(255, 255, 255, 0.05)",
      },
      legend: { x: 0, y: 1.15, orientation: "h" },
    },
    { responsive: true },
  );
}

function renderHeatmap(heatmapData) {
  if (
    !heatmapData ||
    !heatmapData.sharpe_matrix ||
    !heatmapData.x_values ||
    !heatmapData.y_values
  ) {
    console.warn("Heatmap data missing required fields:", heatmapData);
    return;
  }

  const zData = heatmapData.sharpe_matrix;
  const xValues = heatmapData.x_values;
  const yValues = heatmapData.y_values;

  document.getElementById("optimize-chart").innerHTML = ""; // Clear loading state

  Plotly.newPlot(
    "optimize-chart",
    [
      {
        z: zData,
        x: xValues,
        y: yValues,
        type: "heatmap",
        texttemplate: "%{z:.3f}",
        textfont: {
          family: "Inter, sans-serif",
          size: 11,
          color: "white",
          weight: 600, // Plotly uses integer weights or 'bold' string in newer versions, but font object usually takes styling
        },
        colorscale: [
          [0, "#ef4444"],
          [0.5, "#fbbf24"],
          [1, "#10b981"],
        ],
        colorbar: { title: "Sharpe" },
        hoverongaps: false,
      },
    ],
    {
      paper_bgcolor: "rgba(0,0,0,0)",
      plot_bgcolor: "rgba(0,0,0,0)",
      font: { color: "#94a3b8" },
      xaxis: {
        title: heatmapData.x_param,
        tickformat: ".1f",
        tickmode: "array",
        tickvals: xValues,
      },
      yaxis: {
        title: heatmapData.y_param,
        tickformat: ".1f",
        tickmode: "array",
        tickvals: yValues,
      },
      margin: { t: 50, r: 100, b: 60, l: 70 },
    },
    { responsive: true },
  );
}

// Old sensitivity applyBestParams removed - now using the one in Parameter Optimization section

// ===========================
// Deploy & Live Trading
// ===========================

function updateDeploySummary() {
  document.getElementById("deploy-strategy").textContent = AppState.strategy;
  document.getElementById("deploy-symbol").textContent = AppState.symbol || "—";

  if (AppState.backtestResults) {
    updateMetric(
      "deploy-return",
      AppState.backtestResults.total_return * 100,
      "%",
    );
    document.getElementById("deploy-sharpe").textContent =
      AppState.backtestResults.sharpe_ratio?.toFixed(2) || "—";
  } else {
    document.getElementById("deploy-return").textContent = "—";
    document.getElementById("deploy-sharpe").textContent = "—";
  }
}

async function loadSentiment() {
  const badge = document.getElementById("detailed-sentiment-badge");
  badge.textContent = "Loading...";
  badge.className = "sentiment-badge";

  try {
    // Fetch 365 days history for chart + stats
    const res = await fetch(`${API_BASE}/sentiment/history?days=365`);
    const data = await res.json();

    if (data.success && data.data && data.data.length > 0) {
      const history = data.data;
      // Backend returns data.stats with pre-calculated metrics
      const stats = data.stats || {};

      // 1. Current Value (First item is usually latest from API, but let's verify sort)
      // Backend api.rs uses data.first() as current.
      const current = history[0];
      const value = current.value;

      // Update Badge
      badge.textContent = current.classification;
      badge.className = `sentiment-badge ${getSentimentClass(value)}`;

      // Update Large Gauge
      document.getElementById("gauge-large-value").textContent = value;
      document.getElementById("gauge-large-label").textContent =
        current.classification;
      document.getElementById("gauge-large-fill").style.width = `${value}%`;

      // 2. Key Stats
      // Daily Change (Current - Prev)
      let change = 0;
      if (history.length > 1) {
        change = value - history[1].value;
      }
      updateStatPill("sent-change", change, "", true);

      // 7-Day SMA
      const sma7 =
        stats.sma_7 !== undefined ? stats.sma_7 : calculateSMA(history, 7);
      updateStatPill("sent-sma7", sma7, "");

      // Momentum (Current - 7 Days Ago)
      const momentum =
        stats.momentum_7 !== undefined
          ? stats.momentum_7
          : calculateMomentum(history, 7);
      updateStatPill("sent-momentum", momentum, "", true);

      // Dominance (Fear vs Greed days)
      const fearDays = stats.days_in_fear || 0;
      const greedDays = stats.days_in_greed || 0;
      const total = fearDays + greedDays + (stats.days_neutral || 0);
      let dominanceText = "Neutral";
      if (fearDays > greedDays)
        dominanceText = `${((fearDays / total) * 100).toFixed(0)}% Fear`;
      if (greedDays > fearDays)
        dominanceText = `${((greedDays / total) * 100).toFixed(0)}% Greed`;
      document.getElementById("sent-dominance").textContent = dominanceText;

      // 3. Render Chart
      renderSentimentChart(history);
    } else {
      console.warn("Sentiment data unavailable");
      badge.textContent = "Unavailable";
    }
  } catch (e) {
    console.error("Failed to load sentiment:", e);
    badge.textContent = "Error";
  }
}

function updateStatPill(id, value, suffix, colorize = false) {
  const el = document.getElementById(id);
  if (!el) return;

  if (value === undefined || value === null || isNaN(value)) {
    el.textContent = "--";
    return;
  }

  const formatted = Math.abs(value) < 10 ? value.toFixed(1) : value.toFixed(0);
  const sign = value > 0 ? "+" : "";
  el.textContent = `${sign}${formatted}${suffix}`;

  if (colorize) {
    el.style.color =
      value > 0 ? "#10b981" : value < 0 ? "#ef4444" : "var(--text-primary)";
  }
}

function calculateSMA(data, period) {
  if (data.length < period) return null;
  const slice = data.slice(0, period);
  const sum = slice.reduce((acc, curr) => acc + curr.value, 0);
  return sum / period;
}

function calculateMomentum(data, period) {
  if (data.length <= period) return null;
  return data[0].value - data[period].value;
}

function getSentimentClass(value) {
  if (value <= 25) return "text-danger"; // Extreme Fear
  if (value <= 45) return "text-warning"; // Fear
  if (value <= 55) return "text-muted"; // Neutral
  if (value <= 75) return "text-success"; // Greed
  return "text-success"; // Extreme Greed
}

function renderSentimentChart(history) {
  // History needs to be reversed for chart (oldest to newest)
  const sorted = [...history].sort((a, b) => a.timestamp - b.timestamp);

  const x = sorted.map((d) => new Date(d.timestamp * 1000));
  const y = sorted.map((d) => d.value);
  const colors = y.map((v) => {
    if (v <= 25) return "#ef4444";
    if (v <= 45) return "#f97316";
    if (v <= 55) return "#94a3b8";
    if (v <= 75) return "#84cc16";
    return "#10b981";
  });

  const trace = {
    x: x,
    y: y,
    type: "scatter",
    mode: "lines+markers",
    line: { color: "#64748b", width: 2 },
    marker: {
      color: colors,
      size: 8,
      line: { color: "white", width: 1 },
    },
    hovertemplate: "%{x|%b %d}<br>Score: %{y}<br>%{text}<extra></extra>",
    text: sorted.map((d) => d.classification),
  };

  const layout = {
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor: "rgba(0,0,0,0)",
    height: 300,
    margin: { t: 20, r: 20, b: 40, l: 40 },
    font: { color: "#94a3b8" },
    xaxis: {
      gridcolor: "rgba(255, 255, 255, 0.05)",
      showgrid: false,
    },
    yaxis: {
      range: [0, 100],
      gridcolor: "rgba(255, 255, 255, 0.05)",
      dtick: 25,
    },
    shapes: [
      // Zones
      {
        type: "rect",
        xref: "paper",
        yref: "y",
        x0: 0,
        x1: 1,
        y0: 0,
        y1: 25,
        fillcolor: "rgba(239, 68, 68, 0.1)",
        line: { width: 0 },
      },
      {
        type: "rect",
        xref: "paper",
        yref: "y",
        x0: 0,
        x1: 1,
        y0: 75,
        y1: 100,
        fillcolor: "rgba(16, 185, 129, 0.1)",
        line: { width: 0 },
      },
    ],
  };

  Plotly.newPlot("sentiment-history-chart", [trace], layout, {
    responsive: true,
    displayModeBar: false,
  });
}

let ws = null;
let equityChart = null;

function initWebSocket() {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  ws = new WebSocket(`${protocol}//${window.location.host}/api/ws`);

  ws.onopen = () => {
    console.log("WebSocket connected");
    updateWsStatus(true);
  };

  ws.onclose = () => {
    console.log("WebSocket disconnected");
    updateWsStatus(false);
    // Reconnect after 5 seconds
    setTimeout(initWebSocket, 5000);
  };

  ws.onerror = (err) => {
    console.error("WebSocket error:", err);
  };

  ws.onmessage = (event) => {
    try {
      const msg = JSON.parse(event.data);
      handleWsMessage(msg);
    } catch (e) {
      console.error("Failed to parse WebSocket message:", e);
    }
  };
}

function updateWsStatus(connected) {
  const el = document.getElementById("ws-status");
  if (connected) {
    el.className = "status-badge connected";
    el.innerHTML = '<span class="status-dot"></span><span>Live</span>';
  } else {
    el.className = "status-badge disconnected";
    el.innerHTML = '<span class="status-dot"></span><span>Live</span>';
  }
}

function handleWsMessage(msg) {
  switch (msg.type) {
    case "portfolio_update":
      updatePortfolioDisplay(msg.data);
      break;
    case "trade":
      addTradeToHistory(msg.data);
      break;
    case "log":
      addLogEntry(msg.data);
      break;
  }
}

function updatePortfolioDisplay(data) {
  document.getElementById("live-total-value").textContent = formatCurrency(
    data.total_value,
  );
  document.getElementById("live-cash").textContent = formatCurrency(data.cash);
  document.getElementById("live-positions-value").textContent = formatCurrency(
    data.positions_value,
  );
  document.getElementById("live-pnl").textContent = formatCurrency(data.pnl);
}

function formatCurrency(value) {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
  }).format(value);
}

function addLogEntry(entry) {
  const console = document.getElementById("log-console");
  const time = new Date().toLocaleTimeString();
  const div = document.createElement("div");
  div.className = `log-entry log-${entry.level || "info"}`;
  div.innerHTML = `<span class="log-time">${time}</span><span class="log-msg">${entry.message}</span>`;
  console.appendChild(div);
  console.scrollTop = console.scrollHeight;
}

function startTrading() {
  if (!AppState.strategy || !AppState.symbol) {
    alert("Please configure a strategy and symbol first");
    return;
  }

  AppState.isTrading = true;

  document.getElementById("btn-start").disabled = true;
  document.getElementById("btn-stop").disabled = false;
  document.getElementById("engine-status").textContent = "Running";
  document.getElementById("engine-status").className = "engine-status running";

  ws.send(
    JSON.stringify({
      type: "start_trading",
      strategy: AppState.strategy,
      symbol: AppState.symbol,
      params: getParams(),
      position_size: parseFloat(document.getElementById("position-size").value),
    }),
  );

  addLogEntry({
    message: `Started ${AppState.strategy} on ${AppState.symbol}`,
    level: "success",
  });
  updateContextBar();
}

function stopTrading() {
  AppState.isTrading = false;

  document.getElementById("btn-start").disabled = false;
  document.getElementById("btn-stop").disabled = true;
  document.getElementById("engine-status").textContent = "Stopped";
  document.getElementById("engine-status").className = "engine-status stopped";

  ws.send(JSON.stringify({ type: "stop_trading" }));

  addLogEntry({ message: "Trading stopped", level: "info" });
  updateContextBar();
}

function panicClose() {
  if (!confirm("Close all positions immediately?")) return;

  ws.send(JSON.stringify({ type: "panic_close" }));
  addLogEntry({
    message: "⚠️ PANIC CLOSE - All positions closed",
    level: "error",
  });
  stopTrading();
}

// ===========================
// Database Status
// ===========================

async function checkDbStatus() {
  try {
    const res = await fetch(`${API_BASE}/health`);
    const data = await res.json();

    const el = document.getElementById("db-status");
    if (data.database === "connected") {
      el.className = "status-badge connected";
      el.innerHTML = '<span class="status-dot"></span><span>DB</span>';
    } else {
      el.className = "status-badge disconnected";
      el.innerHTML = '<span class="status-dot"></span><span>DB</span>';
    }
  } catch (e) {
    document.getElementById("db-status").className =
      "status-badge disconnected";
  }
}

// ===========================
// Comprehensive Optimization Workflow
// ===========================

let comprehensiveWorkflowResults = null;

async function runComprehensiveWorkflow() {
  const chartContainer = document.getElementById("optimize-chart");
  const resultsPlaceholder = document.getElementById(
    "optimize-results-placeholder",
  );

  // Show loading state
  chartContainer.innerHTML = `
        <div class="placeholder-content">
            <span class="placeholder-icon">⏳</span>
            <span>Running Comprehensive Optimization Workflow...</span>
            <span style="font-size: 12px; margin-top: 8px; opacity: 0.7;">
                Grid search + parameter dispersion + walk-forward + sensitivity analysis.
                This may take 30-60 seconds.
            </span>
        </div>`;

  resultsPlaceholder.innerHTML = `
        <span class="placeholder-icon">⏳</span>
        <span>Analyzing...</span>
    `;

  // Disable button
  const btn = document.querySelector(
    'button[onclick="runComprehensiveWorkflow()"]',
  );
  if (btn) btn.disabled = true;

  try {
    const include3D = document.getElementById("include-3d-sensitivity").checked;

    // Get symbol from optimize tab
    const optimizeSymbol = document.getElementById("optimize-symbol").value;
    if (!optimizeSymbol) {
      throw new Error("Please select a trading symbol first");
    }

    // Update AppState with the selected symbol
    AppState.symbol = optimizeSymbol;

    const res = await fetch(`${API_BASE}/backtest/workflow`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        strategy: AppState.strategy,
        symbol: optimizeSymbol,
        interval: AppState.backtestInterval,
        days: 730, // Use 2 years for comprehensive analysis
        include_3d_sensitivity: include3D,
        train_window_days: 252,
        test_window_days: 63,
      }),
    });

    const data = await res.json();

    if (!data.success) {
      throw new Error(data.error || "Workflow failed");
    }

    // Store results
    comprehensiveWorkflowResults = data;

    // Update metrics
    const robustnessScore = data.robustness_score;
    const robustnessEl = document.getElementById("comp-robustness");
    robustnessEl.textContent = robustnessScore.toFixed(1);

    // Color code the robustness score
    if (robustnessScore >= 70) {
      robustnessEl.style.color = "#10b981"; // Green
    } else if (robustnessScore >= 50) {
      robustnessEl.style.color = "#f59e0b"; // Orange
    } else {
      robustnessEl.style.color = "#ef4444"; // Red
    }

    document.getElementById("comp-sharpe").textContent =
      data.best_sharpe.toFixed(2);
    document.getElementById("comp-wf-stability").textContent =
      data.walk_forward_stability_score.toFixed(2);
    document.getElementById("comp-param-cv").textContent =
      data.parameter_dispersion.sharpe_cv.toFixed(2);
    document.getElementById("comp-positive-pct").textContent =
      data.parameter_dispersion.positive_sharpe_pct.toFixed(0) + "%";
    document.getElementById("comp-iterations").textContent =
      data.sweep_results.length;

    // Display Monte Carlo results
    displayMonteCarloResults(data.monte_carlo);

    // Show optimized parameters
    const paramSummary = document.getElementById("comp-param-summary");
    let paramsHtml =
      '<div style="margin-top: 12px; padding: 12px; background: rgba(139, 92, 246, 0.1); border-radius: 8px;">';
    paramsHtml += "<strong>Optimized Parameters:</strong><br>";
    for (const [key, value] of Object.entries(data.optimized_params)) {
      paramsHtml += `<span style=\"display: inline-block; margin: 4px 8px 4px 0;\">${key}: <strong>${value.toFixed(2)}</strong></span>`;
    }
    paramsHtml += "</div>";
    paramSummary.innerHTML = paramsHtml;

    // Show results block
    resultsPlaceholder.classList.add("hidden");
    document.getElementById("comprehensive-results").classList.remove("hidden");
    document.getElementById("wfa-results").classList.add("hidden");
    document.getElementById("sensitivity-results").classList.add("hidden");

    // Render visualization
    if (data.sensitivity_heatmap && include3D) {
      renderSensitivityHeatmap(data.sensitivity_heatmap);
      document.getElementById("optimize-chart-title").textContent =
        "3D Parameter Sensitivity Heatmap";
    } else {
      renderParameterSweep(data.sweep_results);
      document.getElementById("optimize-chart-title").textContent =
        "Parameter Sweep Results";
    }
  } catch (e) {
    console.error("Comprehensive Workflow Error:", e);
    chartContainer.innerHTML = `
            <div class="placeholder-content" style="color: #ef4444;">
                <span class="placeholder-icon">⚠️</span>
                <span>Workflow Failed</span>
                <span style="font-size: 14px; margin-top: 8px; max-width: 80%; text-align: center;">${e.message}</span>
                <button class="btn-primary" style="margin-top: 16px;" onclick="runComprehensiveWorkflow()">Try Again</button>
            </div>`;

    resultsPlaceholder.innerHTML = `
            <span class="placeholder-icon" style="color: #ef4444;">⚠️</span>
            <span style="color: #ef4444;">Workflow Failed</span>
        `;
  } finally {
    if (btn) btn.disabled = false;
  }
}

function renderSensitivityHeatmap(heatmap) {
  const data = [
    {
      z: heatmap.sharpe_matrix,
      x: heatmap.x_values,
      y: heatmap.y_values,
      type: "heatmap",
      colorscale: "RdYlGn",
      colorbar: {
        title: "Sharpe Ratio",
      },
    },
  ];

  const layout = {
    title: `${heatmap.x_param} vs ${heatmap.y_param}`,
    xaxis: { title: heatmap.x_param },
    yaxis: { title: heatmap.y_param },
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor: "rgba(0,0,0,0)",
    font: { color: "#e5e7eb" },
  };

  Plotly.newPlot("optimize-chart", data, layout, { responsive: true });
}

function renderParameterSweep(sweepResults) {
  // Show parameter sweep as scatter plot
  const data = [
    {
      x: sweepResults.map((r) => r.sharpe),
      y: sweepResults.map((r) => r.total_return * 100),
      mode: "markers",
      type: "scatter",
      marker: {
        size: 8,
        color: sweepResults.map((r) => r.score),
        colorscale: "Viridis",
        colorbar: {
          title: "Composite Score",
        },
      },
      text: sweepResults.map(
        (r) => `Score: ${r.score.toFixed(2)}<br>Trades: ${r.total_trades}`,
      ),
      hovertemplate: "%{text}<extra></extra>",
    },
  ];

  const layout = {
    title: "Parameter Combinations (Sharpe vs Return)",
    xaxis: { title: "Sharpe Ratio" },
    yaxis: { title: "Total Return (%)" },
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor: "rgba(0,0,0,0)",
    font: { color: "#e5e7eb" },
  };

  Plotly.newPlot("optimize-chart", data, layout, { responsive: true });
}

function applyOptimizedParams() {
  if (!comprehensiveWorkflowResults) {
    alert("No optimization results available");
    return;
  }

  // Apply params to AppState
  AppState.params = comprehensiveWorkflowResults.optimized_params;

  // Update UI inputs
  const strategyParams = STRATEGY_PARAMS[AppState.strategy] || [];
  strategyParams.forEach((param) => {
    const value = comprehensiveWorkflowResults.optimized_params[param.name];
    if (value !== undefined) {
      const input = document.getElementById(`param-${param.name}`);
      if (input) {
        input.value = value;
      }
    }
  });

  // Update backtest summary
  updateBacktestSummary();

  // Navigate to backtest tab
  switchTab("backtest");

  alert(
    `✓ Optimized parameters applied!\n\nRobustness Score: ${comprehensiveWorkflowResults.robustness_score.toFixed(1)}/100\n\nClick "Run Backtest" to see full results with these parameters.`,
  );
}

function goToBacktestWithOptimizedParams() {
  if (!comprehensiveWorkflowResults) {
    alert("No optimization results available");
    return;
  }

  // Apply params to AppState
  AppState.params = comprehensiveWorkflowResults.optimized_params;

  // Symbol will be selected in the backtest tab from the asset category
  // No need to set it here as user selects from backtest-symbol dropdown

  // Update context bar and backtest summary
  updateContextBar();
  updateBacktestSummary();

  // Navigate to backtest tab
  switchTab("backtest");

  // Show helpful message
  alert(
    `✓ Optimized parameters applied!\n\nRobustness Score: ${comprehensiveWorkflowResults.robustness_score.toFixed(1)}/100\n\nSelect a symbol from the "${AppState.assetCategory}" category and click "Run Backtest" to see results.`,
  );
}

// Initialize symbol selector for optimize tab
function initOptimizeSymbolSelect() {
  // This function is deprecated - Optimize tab no longer has individual symbol selection
  // Optimization now trains across all symbols in the selected asset category
  const wrapper = document.getElementById("optimize-symbol-select-wrapper");
  const trigger = document.getElementById("optimize-symbol-trigger");
  const optionsList = document.getElementById("optimize-symbol-options-list");
  const searchInput = document.getElementById("optimize-symbol-search-input");
  const hiddenInput = document.getElementById("optimize-symbol");
  const selectedText = document.getElementById("optimize-selected-symbol-text");

  // Exit if elements don't exist (they were removed in workflow restructuring)
  if (
    !wrapper ||
    !trigger ||
    !optionsList ||
    !searchInput ||
    !hiddenInput ||
    !selectedText
  ) {
    console.log(
      "Optimize tab symbol selector elements not found (expected - using asset category optimization now)",
    );
    return;
  }

  // Toggle dropdown
  trigger.addEventListener("click", () => {
    wrapper.classList.toggle("open");
    if (wrapper.classList.contains("open")) {
      searchInput.focus();
    }
  });

  // Close when clicking outside
  document.addEventListener("click", (e) => {
    if (!wrapper.contains(e.target)) {
      wrapper.classList.remove("open");
    }
  });

  // Filter function
  function filterSymbols(query) {
    const q = query.toLowerCase();
    const POPULAR = [
      "BTC",
      "ETH",
      "SOL",
      "XRP",
      "ADA",
      "DOGE",
      "AVAX",
      "DOT",
      "LINK",
      "MATIC",
    ];

    let filtered = allSymbols.filter((s) => s.toLowerCase().includes(q));

    // Sort: Popular first, then alphabetical
    filtered.sort((a, b) => {
      const aPop = POPULAR.indexOf(a);
      const bPop = POPULAR.indexOf(b);

      if (aPop !== -1 && bPop !== -1) return aPop - bPop;
      if (aPop !== -1) return -1;
      if (bPop !== -1) return 1;
      return a.localeCompare(b);
    });

    renderOptions(filtered);
  }

  // Render options to the list
  function renderOptions(symbols) {
    optionsList.innerHTML = "";

    if (symbols.length === 0) {
      optionsList.innerHTML =
        '<div class="option" style="cursor: default; opacity: 0.5;">No matches found</div>';
      return;
    }

    // Limit rendering for performance if list is huge
    const displaySymbols = symbols.slice(0, 100);

    displaySymbols.forEach((symbol) => {
      const div = document.createElement("div");
      div.className = "option";
      if (symbol === AppState.symbol) div.classList.add("selected");

      div.innerHTML = `
                <span class="option-ticker">${symbol}</span>
                <span class="option-name">USDT</span>
            `;

      div.addEventListener("click", () => {
        AppState.symbol = symbol;
        hiddenInput.value = symbol;
        selectedText.textContent = symbol;
        wrapper.classList.remove("open");
        updateContextBar();
      });

      optionsList.appendChild(div);
    });
  }

  // Search as you type
  searchInput.addEventListener("input", (e) => {
    filterSymbols(e.target.value);
  });

  // Initial render
  filterSymbols("");
}

// ===========================
// Auto-Optimize Function (Simplified UI)
// ===========================

async function runAutoOptimize() {
  const resultsPlaceholder = document.getElementById(
    "optimize-results-placeholder",
  );
  const allResults = document.getElementById("all-results");

  // Show loading state
  resultsPlaceholder.innerHTML = `
        <span class="placeholder-icon">⏳</span>
        <span>Running comprehensive optimization...</span>
        <span style="font-size: 12px; margin-top: 8px; opacity: 0.7;">
            This includes parameter sweep, sensitivity analysis, and walk-forward validation.
            May take 30-60 seconds.
        </span>
    `;

  // Disable button with more robust selector
  const btn = document.querySelector('button[onclick*="runAutoOptimize"]');
  if (!btn) {
    console.error("Auto-Optimize button not found");
    return;
  }
  btn.disabled = true;
  btn.innerHTML = "⏳ Optimizing...";

  try {
    // Get symbols from selected asset category
    const category = ASSET_CATEGORIES[AppState.assetCategory];
    if (!category || !category.symbols || category.symbols.length === 0) {
      throw new Error("Invalid asset category selected");
    }

    const symbols = category.symbols;

    // Use multi-symbol optimization endpoint
    resultsPlaceholder.innerHTML = `
            <span class="placeholder-icon">⏳</span>
            <span>Running multi-symbol optimization...</span>
            <span style="font-size: 12px; margin-top: 8px; opacity: 0.7;">
                Optimizing across ${category.name} (${symbols.length} symbols)<br>
                Aggregating results for robust parameter selection.<br>
                This may take 1-2 minutes.
            </span>
        `;

    const res = await fetch(`${API_BASE}/backtest/workflow/multi`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        strategy: AppState.strategy,
        symbols: symbols, // Send all symbols in category
        interval: AppState.backtestInterval,
        days: 730, // Use 2 years for comprehensive analysis
        include_3d_sensitivity: false, // Disable for multi-symbol (too slow)
        train_window_days: 252,
        test_window_days: 63,
      }),
    });

    const data = await res.json();

    if (!data.success) {
      throw new Error(data.error || "Workflow failed");
    }

    // Store results globally
    comprehensiveWorkflowResults = data;

    // Automatically apply optimized parameters to AppState
    AppState.params = data.optimized_params;
    updateContextBar();

    // Train ML model if enabled
    let mlTrainResult = null;
    if (AppState.mlEnabled) {
      resultsPlaceholder.innerHTML = `
                <span class="placeholder-icon">🤖</span>
                <span>Training ML model on ${symbols.length} symbols...</span>
                <span style="font-size: 12px; margin-top: 8px; opacity: 0.7;">
                    Model Type: ${AppState.mlModelType}<br>
                    Training with random subsets from each symbol
                </span>
            `;

      // Pass ALL symbols for multi-symbol training with random subsets
      mlTrainResult = await trainMLModel(
        symbols,
        AppState.backtestInterval,
        365,
      );

      if (mlTrainResult) {
        // Use first symbol for validation (already includes multi-symbol data)
        const validateSymbol = symbols[0];
        const mlValidateResult = await validateMLModel(
          validateSymbol,
          AppState.backtestInterval,
          365,
        );
        if (mlValidateResult) {
          mlTrainResult.validation = mlValidateResult;
        }
      }
    }

    // Hide placeholder, show all results
    resultsPlaceholder.classList.add("hidden");
    allResults.classList.remove("hidden");

    // Update summary metrics (handle both single and multi-symbol response formats)
    const robustnessScore = data.robustness_score || 0;
    const robustnessEl = document.getElementById("comp-robustness");
    robustnessEl.textContent = robustnessScore.toFixed(1);

    // Color code the robustness score
    if (robustnessScore >= 70) {
      robustnessEl.style.color = "#10b981"; // Green
    } else if (robustnessScore >= 50) {
      robustnessEl.style.color = "#f59e0b"; // Orange
    } else {
      robustnessEl.style.color = "#ef4444"; // Red
    }

    // Handle multi-symbol format (avg_sharpe) vs single-symbol format (best_sharpe)
    const sharpeValue = data.best_sharpe ?? data.avg_sharpe ?? 0;
    document.getElementById("comp-sharpe").textContent = sharpeValue.toFixed(2);

    // Walk-forward stability (may not be present in multi-symbol response)
    const wfStability = data.walk_forward_stability_score ?? 0;
    document.getElementById("comp-wf-stability").textContent =
      wfStability.toFixed(2);

    // Display Monte Carlo results
    displayMonteCarloResults(data.monte_carlo);

    // Parameter dispersion
    const disp = data.parameter_dispersion || {};
    document.getElementById("comp-param-cv").textContent = (
      disp.sharpe_cv ?? 0
    ).toFixed(2);
    document.getElementById("comp-positive-pct").textContent =
      (disp.positive_sharpe_pct ?? 0).toFixed(0) + "%";

    // Iterations (sweep_results for single-symbol, symbols_processed for multi-symbol)
    const iterations =
      data.sweep_results?.length ?? data.symbols_processed ?? 0;
    document.getElementById("comp-iterations").textContent = iterations;

    // Show optimized parameters
    const paramSummary = document.getElementById("comp-param-summary");
    let paramsHtml =
      '<div style="margin-top: 12px; padding: 12px; background: rgba(16, 185, 129, 0.1); border-radius: 8px;">';
    paramsHtml +=
      '<strong style="color: #10b981;">✓ Optimized Parameters (Auto-Applied):</strong><br>';
    for (const [key, value] of Object.entries(data.optimized_params || {})) {
      paramsHtml += `<span style="display: inline-block; margin: 4px 8px 4px 0;">${key}: <strong>${value.toFixed(2)}</strong></span>`;
    }
    paramsHtml += "</div>";

    // Show multi-symbol results if available
    if (data.symbol_results && data.symbol_results.length > 0) {
      paramsHtml +=
        '<div style="margin-top: 12px; padding: 12px; background: rgba(96, 165, 250, 0.1); border-radius: 8px;">';
      paramsHtml += `<strong style="color: #60a5fa;">📊 Multi-Symbol Results (${data.symbols_processed} symbols):</strong><br>`;
      paramsHtml += `<span>Avg Sharpe: <strong>${(data.avg_sharpe || 0).toFixed(2)}</strong></span> | `;
      paramsHtml += `<span>Avg Return: <strong>${((data.avg_return || 0) * 100).toFixed(2)}%</strong></span> | `;
      paramsHtml += `<span>Worst DD: <strong>${((data.worst_drawdown || 0) * 100).toFixed(2)}%</strong></span>`;
      paramsHtml += "</div>";
    }

    // Show ML Training Results if available
    if (mlTrainResult) {
      paramsHtml +=
        '<div style="margin-top: 12px; padding: 12px; background: rgba(139, 92, 246, 0.15); border-radius: 8px; border: 1px solid rgba(139, 92, 246, 0.3);">';
      paramsHtml +=
        '<strong style="color: #a78bfa;">🤖 ML Model Trained (Multi-Symbol):</strong><br>';

      // Show symbols used
      const symbolsUsed = mlTrainResult.symbols_used || [];
      paramsHtml += `<span>Symbols: <strong>${symbolsUsed.length > 3 ? symbolsUsed.slice(0, 3).join(", ") + "..." : symbolsUsed.join(", ")}</strong></span> | `;
      paramsHtml += `<span>Total Samples: <strong>${mlTrainResult.total_samples || "N/A"}</strong></span><br>`;

      // Show training metrics
      paramsHtml += `<span style="margin-top: 8px; display: block;">`;
      paramsHtml += `Train R²: <strong>${(mlTrainResult.train_r_squared || 0).toFixed(3)}</strong> | `;
      paramsHtml += `Test R²: <strong>${(mlTrainResult.test_r_squared || 0).toFixed(3)}</strong> | `;
      paramsHtml += `Stability: <strong>${(mlTrainResult.stability_score || 0).toFixed(1)}%</strong>`;
      paramsHtml += `</span>`;

      if (mlTrainResult.validation && mlTrainResult.validation.result) {
        const val = mlTrainResult.validation.result;
        paramsHtml += `<span style="margin-top: 4px; display: block; color: #60a5fa;">`;
        paramsHtml += `WF Stability: <strong>${(val.stability_score || 0).toFixed(1)}</strong> | `;
        paramsHtml += `Avg Test R²: <strong>${(val.avg_test_r_squared || 0).toFixed(3)}</strong>`;
        if (val.is_overfit !== undefined) {
          const overfit = val.is_overfit ? "⚠️ Risk" : "✓ OK";
          paramsHtml += ` | Overfit: <strong>${overfit}</strong>`;
        }
        paramsHtml += `</span>`;
      }
      paramsHtml += "</div>";
    }

    paramSummary.innerHTML = paramsHtml;

    // Walk-Forward inline metrics (may not be present in multi-symbol response)
    const wfResult = data.walk_forward_validation || {};
    const wfAgg = wfResult.aggregate_oos || {};
    document.getElementById("wfa-stability-inline").textContent =
      wfStability.toFixed(2);
    document.getElementById("wfa-avg-return-inline").textContent =
      ((wfAgg.mean_return || data.avg_return || 0) * 100).toFixed(2) + "%";
    document.getElementById("wfa-win-rate-inline").textContent =
      ((wfAgg.win_rate || 0) * 100).toFixed(0) + "%";
    document.getElementById("wfa-oos-sharpe-inline").textContent = (
      wfAgg.mean_sharpe ||
      data.avg_sharpe ||
      0
    ).toFixed(2);

    // Parameter dispersion details
    document.getElementById("disp-sharpe-std").textContent = (
      disp.sharpe_std ?? 0
    ).toFixed(3);
    document.getElementById("disp-return-std").textContent =
      ((disp.return_std ?? 0) * 100).toFixed(2) + "%";
    document.getElementById("disp-sharpe-range").textContent = (
      disp.sharpe_range ?? 0
    ).toFixed(2);
    document.getElementById("disp-return-range").textContent =
      ((disp.return_range ?? 0) * 100).toFixed(2) + "%";

    // Render Parameter Sweep Chart (only if sweep_results available)
    if (data.sweep_results && data.sweep_results.length > 0) {
      renderParameterSweepChart(data.sweep_results);
    }

    // Render 3D Sensitivity Heatmap
    if (data.sensitivity_heatmap) {
      renderSensitivityHeatmapChart(data.sensitivity_heatmap);
    }

    // Render Walk-Forward Chart (only if windows available)
    console.log("Walk-forward validation data:", wfResult);
    if (wfResult && wfResult.windows && wfResult.windows.length > 0) {
      console.log(
        "Rendering walk-forward chart with",
        wfResult.windows.length,
        "windows",
      );
      renderWalkForwardChart(wfResult.windows);
    } else {
      console.log("Multi-symbol mode: walk-forward chart not available");
    }
  } catch (e) {
    console.error("Auto-Optimize Error:", e);
    resultsPlaceholder.innerHTML = `
            <div class="placeholder-content" style="color: #ef4444;">
                <span class="placeholder-icon">⚠️</span>
                <span>Optimization Failed</span>
                <span style="font-size: 14px; margin-top: 8px; max-width: 80%; text-align: center;">${e.message}</span>
                <button class="btn-primary" style="margin-top: 16px;" onclick="runAutoOptimize()">Try Again</button>
            </div>`;
  } finally {
    if (btn) {
      btn.disabled = false;
      btn.innerHTML = "🎯 Auto-Optimize";
    }
  }
}

function renderParameterSweepChart(sweepResults) {
  // Hide placeholder content
  const chartDiv = document.getElementById("param-sweep-chart");
  const placeholder = chartDiv.querySelector(".placeholder-content");
  if (placeholder) {
    placeholder.style.display = "none";
  }

  const data = [
    {
      x: sweepResults.map((r) => r.sharpe),
      y: sweepResults.map((r) => r.total_return * 100),
      mode: "markers",
      type: "scatter",
      marker: {
        size: 8,
        color: sweepResults.map((r) => r.score),
        colorscale: "Viridis",
        colorbar: {
          title: "Score",
        },
      },
      text: sweepResults.map(
        (r) =>
          `Sharpe: ${r.sharpe.toFixed(2)}<br>Return: ${(r.total_return * 100).toFixed(2)}%<br>Trades: ${r.total_trades}`,
      ),
      hovertemplate: "%{text}<extra></extra>",
    },
  ];

  const layout = {
    title: "",
    xaxis: { title: "Sharpe Ratio" },
    yaxis: { title: "Total Return (%)" },
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor: "rgba(0,0,0,0)",
    font: { color: "#e5e7eb" },
  };

  Plotly.newPlot("param-sweep-chart", data, layout, { responsive: true });
}

function renderSensitivityHeatmapChart(heatmap) {
  if (!heatmap) return;

  // Hide placeholder content
  const chartDiv = document.getElementById("sensitivity-chart");
  const placeholder = chartDiv.querySelector(".placeholder-content");
  if (placeholder) {
    placeholder.style.display = "none";
  }

  const data = [
    {
      z: heatmap.sharpe_matrix,
      x: heatmap.x_values,
      y: heatmap.y_values,
      type: "heatmap",
      colorscale: "RdYlGn",
      colorbar: {
        title: "Sharpe",
      },
    },
  ];

  const layout = {
    title: "",
    xaxis: { title: heatmap.x_param },
    yaxis: { title: heatmap.y_param },
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor: "rgba(0,0,0,0)",
    font: { color: "#e5e7eb" },
  };

  Plotly.newPlot("sensitivity-chart", data, layout, { responsive: true });
}

function renderWalkForwardChart(windows) {
  console.log("renderWalkForwardChart called with:", windows);

  if (!windows || windows.length === 0) {
    console.warn("No windows data to render");
    return;
  }

  try {
    // Hide placeholder content
    const chartDiv = document.getElementById("walk-forward-chart");
    const placeholder = chartDiv.querySelector(".placeholder-content");
    if (placeholder) {
      placeholder.style.display = "none";
    }

    const data = [
      {
        x: windows.map((w, i) => `Window ${i + 1}`),
        y: windows.map(
          (w) => ((w.test_metrics && w.test_metrics.total_return) || 0) * 100,
        ),
        name: "OOS Return",
        type: "bar",
        marker: { color: "#60a5fa" },
      },
      {
        x: windows.map((w, i) => `Window ${i + 1}`),
        y: windows.map(
          (w) => (w.test_metrics && w.test_metrics.sharpe_ratio) || 0,
        ),
        name: "OOS Sharpe",
        type: "bar",
        marker: { color: "#34d399" },
        yaxis: "y2",
      },
    ];

    const layout = {
      title: "",
      xaxis: { title: "Walk-Forward Window" },
      yaxis: { title: "Return (%)", side: "left" },
      yaxis2: { title: "Sharpe Ratio", overlaying: "y", side: "right" },
      paper_bgcolor: "rgba(0,0,0,0)",
      plot_bgcolor: "rgba(0,0,0,0)",
      font: { color: "#e5e7eb" },
      barmode: "group",
    };

    console.log("Creating Plotly chart with data:", data);
    Plotly.newPlot("walk-forward-chart", data, layout, {
      responsive: true,
    });
    console.log("Walk-forward chart rendered successfully");
  } catch (error) {
    console.error("Error rendering walk-forward chart:", error);
  }
}

// ===========================
// Asset Category Definitions
// ===========================

const ASSET_CATEGORIES = {
  market: {
    name: "Market (Top 10)",
    symbols: [
      "BTC",
      "ETH",
      "BNB",
      "XRP",
      "ADA",
      "SOL",
      "DOT",
      "DOGE",
      "AVAX",
      "MATIC",
    ],
  },
  "large-cap": {
    name: "Large Cap (Rank 1-30)",
    symbols: [
      "BTC",
      "ETH",
      "BNB",
      "XRP",
      "ADA",
      "SOL",
      "DOT",
      "DOGE",
      "AVAX",
      "MATIC",
      "LINK",
      "UNI",
      "ATOM",
      "LTC",
      "ETC",
      "XLM",
      "ALGO",
      "VET",
      "ICP",
      "FIL",
    ],
  },
  "mid-cap": {
    name: "Mid Cap (Rank 31-100)",
    symbols: [
      "AAVE",
      "GRT",
      "SAND",
      "MANA",
      "AXS",
      "ENJ",
      "CHZ",
      "THETA",
      "FTM",
      "ONE",
      "HBAR",
      "FLOW",
      "EGLD",
      "XTZ",
      "ZIL",
      "QTUM",
      "WAVES",
      "ICX",
      "OMG",
      "ZRX",
    ],
  },
  "small-cap": {
    name: "Small Cap (Rank 101-300)",
    symbols: [
      "ANKR",
      "CRV",
      "BAL",
      "COMP",
      "YFI",
      "SNX",
      "SUSHI",
      "CAKE",
      "1INCH",
      "UMA",
      "REN",
      "KNC",
      "LRC",
      "BAND",
      "RSR",
      "NMR",
      "OCEAN",
      "CELR",
      "STORJ",
      "AUDIO",
    ],
  },
  defi: {
    name: "DeFi Protocols",
    symbols: [
      "AAVE",
      "UNI",
      "LINK",
      "COMP",
      "MKR",
      "SNX",
      "CRV",
      "SUSHI",
      "YFI",
      "BAL",
      "1INCH",
      "CAKE",
      "UMA",
      "BNT",
      "REN",
      "KNC",
      "LRC",
      "BAND",
      "RSR",
      "NMR",
    ],
  },
};

// ===========================
// Update AppState for Asset Category
// ===========================

// Add asset category to AppState
if (!AppState.assetCategory) {
  AppState.assetCategory = "market";
}

// Update goToOptimize to capture asset category
const originalGoToOptimize = goToOptimize;
goToOptimize = function () {
  // Get selected asset category
  const categoryInput = document.querySelector(
    'input[name="asset-category"]:checked',
  );
  if (categoryInput) {
    AppState.assetCategory = categoryInput.value;
  }

  // Update optimize tab display
  updateOptimizeDisplay();

  // Call original function
  originalGoToOptimize();
};

function updateOptimizeDisplay() {
  const strategyEl = document.getElementById("opt-display-strategy");
  const categoryEl = document.getElementById("opt-display-category");

  if (strategyEl) {
    strategyEl.textContent = AppState.strategy || "—";
  }

  if (categoryEl && AppState.assetCategory) {
    const category = ASSET_CATEGORIES[AppState.assetCategory];
    categoryEl.textContent = category ? category.name : AppState.assetCategory;
  }
}

// Initialize display on page load
document.addEventListener("DOMContentLoaded", function () {
  // Listen for asset category changes
  document.querySelectorAll('input[name="asset-category"]').forEach((input) => {
    input.addEventListener("change", function () {
      AppState.assetCategory = this.value;
    });
  });
});

// ===========================
// Backtest Symbol Selector
// ===========================

function initBacktestSymbolSelect() {
  const wrapper = document.getElementById("backtest-symbol-select-wrapper");
  const trigger = document.getElementById("backtest-symbol-trigger");
  const optionsList = document.getElementById("backtest-symbol-options-list");
  const searchInput = document.getElementById("backtest-symbol-search-input");
  const hiddenInput = document.getElementById("backtest-symbol");

  if (!wrapper || !trigger) {
    console.warn("Backtest symbol selector elements not found");
    return;
  }

  console.log(
    "Initializing backtest symbol selector with asset category:",
    AppState.assetCategory,
  );

  // Remove any existing listeners to avoid duplicates
  const newTrigger = trigger.cloneNode(true);
  trigger.parentNode.replaceChild(newTrigger, trigger);

  // Get fresh reference to selectedText after trigger replacement
  const selectedText = document.getElementById("backtest-selected-symbol-text");

  // Toggle dropdown
  newTrigger.addEventListener("click", () => {
    wrapper.classList.toggle("open");
    if (wrapper.classList.contains("open")) {
      searchInput.focus();
      renderBacktestSymbolOptions("");
    }
  });

  // Close when clicking outside
  document.addEventListener("click", (e) => {
    if (!wrapper.contains(e.target)) {
      wrapper.classList.remove("open");
    }
  });

  // Search as you type
  searchInput.addEventListener("input", (e) => {
    renderBacktestSymbolOptions(e.target.value);
  });

  // Render options from selected category
  function renderBacktestSymbolOptions(query) {
    optionsList.innerHTML = "";

    // Ensure asset category is set
    if (!AppState.assetCategory) {
      optionsList.innerHTML =
        '<div class="option" style="cursor: default; opacity: 0.5;">Please select an asset category in Build tab</div>';
      return;
    }

    // Get symbols from selected asset category
    const category = ASSET_CATEGORIES[AppState.assetCategory];
    if (!category || !category.symbols) {
      optionsList.innerHTML =
        '<div class="option" style="cursor: default; opacity: 0.5;">No symbols available for this category</div>';
      return;
    }

    let symbols = category.symbols;

    // Filter by search query
    if (query) {
      const q = query.toLowerCase();
      symbols = symbols.filter((s) => s.toLowerCase().includes(q));
    }

    if (symbols.length === 0) {
      optionsList.innerHTML =
        '<div class="option" style="cursor: default; opacity: 0.5;">No matches found</div>';
      return;
    }

    symbols.forEach((symbol) => {
      const div = document.createElement("div");
      div.className = "option";
      if (symbol === AppState.symbol) div.classList.add("selected");

      div.innerHTML = `
                <span class="option-ticker">${symbol}</span>
                <span class="option-name">USDT</span>
            `;

      div.addEventListener("click", () => {
        // Get fresh reference each time in case of re-initialization
        const currentSelectedText = document.getElementById(
          "backtest-selected-symbol-text",
        );
        AppState.symbol = symbol;
        hiddenInput.value = symbol;
        if (currentSelectedText) {
          currentSelectedText.textContent = symbol;
        }
        wrapper.classList.remove("open");
        updateBacktestSummary();
      });

      optionsList.appendChild(div);
    });
  }

  // Initialize with primary symbol if available
  if (AppState.symbol && selectedText) {
    selectedText.textContent = AppState.symbol;
    hiddenInput.value = AppState.symbol;
  } else if (AppState.assetCategory) {
    // Auto-select first symbol from category
    const category = ASSET_CATEGORIES[AppState.assetCategory];
    if (category && category.symbols && category.symbols.length > 0) {
      AppState.symbol = category.symbols[0];
      selectedText.textContent = AppState.symbol;
      hiddenInput.value = AppState.symbol;
    }
  }
}

// Update goToBacktestWithOptimizedParams to initialize symbol selector
const originalGoToBacktestWithOptimizedParams = goToBacktestWithOptimizedParams;
goToBacktestWithOptimizedParams = function () {
  originalGoToBacktestWithOptimizedParams();

  // Initialize the backtest symbol selector with category symbols
  setTimeout(() => {
    initBacktestSymbolSelect();

    // Auto-select the primary symbol if not already selected
    const backtestSymbol = document.getElementById("backtest-symbol");
    const selectedText = document.getElementById(
      "backtest-selected-symbol-text",
    );
    if (!backtestSymbol.value && AppState.symbol) {
      backtestSymbol.value = AppState.symbol;
      selectedText.textContent = AppState.symbol;
    }
  }, 100);
};

// Also update the backtest summary display
function updateBacktestSummary() {
  const strategyEl = document.getElementById("bt-summary-strategy");
  const categoryEl = document.getElementById("bt-summary-category");
  const paramsEl = document.getElementById("bt-summary-params");

  if (strategyEl) {
    strategyEl.textContent = AppState.strategy || "—";
  }

  if (categoryEl && AppState.assetCategory) {
    const category = ASSET_CATEGORIES[AppState.assetCategory];
    categoryEl.textContent = category ? category.name : AppState.assetCategory;
  }

  if (paramsEl && AppState.params) {
    let paramsText = "";
    for (const [key, value] of Object.entries(AppState.params)) {
      paramsText += `${key}=${typeof value === "number" ? value.toFixed(2) : value} `;
    }
    paramsEl.textContent = paramsText || "—";
  }
}

// ===========================
// Interactive Chart Functions
// ===========================

let chartData = null;
let currentChartType = "candlestick";

function toggleIndicatorPanel() {
  const panel = document.getElementById("indicator-panel");
  if (panel.style.display === "none") {
    panel.style.display = "block";
  } else {
    panel.style.display = "none";
  }
}

function switchChartType(type) {
  currentChartType = type;

  // Update button states
  document.querySelectorAll(".chart-type-btn").forEach((btn) => {
    btn.classList.remove("active");
    if (btn.dataset.type === type) {
      btn.classList.add("active");
    }
  });

  // Re-render chart with new type
  if (chartData) {
    renderPriceChart(chartData);
  }
}

async function updateChartIndicators() {
  // Check if we have a symbol and data
  if (
    !AppState.symbol ||
    !AppState.backtestInterval ||
    !AppState.backtestDays
  ) {
    console.log("No symbol, interval, or days selected for chart");
    return;
  }

  // Collect selected indicators
  const indicators = [];

  if (document.getElementById("ind-sma-50")?.checked) {
    indicators.push({ type: "sma", period: 50 });
  }
  if (document.getElementById("ind-sma-200")?.checked) {
    indicators.push({ type: "sma", period: 200 });
  }
  if (document.getElementById("ind-ema-20")?.checked) {
    indicators.push({ type: "ema", period: 20 });
  }
  if (document.getElementById("ind-bb")?.checked) {
    indicators.push({ type: "bb", period: 20, std_dev: 2.0 });
  }
  if (document.getElementById("ind-rsi")?.checked) {
    indicators.push({ type: "rsi", period: 14 });
  }
  if (document.getElementById("ind-macd")?.checked) {
    indicators.push({ type: "macd", fast: 12, slow: 26, signal: 9 });
  }

  // Fetch chart data with indicators
  try {
    const response = await fetch(`${API_BASE}/chart/ohlcv`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        symbol: AppState.symbol,
        interval: AppState.backtestInterval,
        days: AppState.backtestDays,
        indicators: indicators,
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    chartData = await response.json();
    console.log("Chart data received:", chartData);

    renderPriceChart(chartData);
  } catch (error) {
    console.error("Failed to fetch chart data:", error);
  }
}

function renderPriceChart(data) {
  if (!data || !data.bars || data.bars.length === 0) {
    console.log("No chart data to render");
    return;
  }

  const timestamps = data.bars.map((b) => new Date(b.timestamp * 1000));

  const traces = [];

  // Main price trace
  if (currentChartType === "candlestick") {
    traces.push({
      type: "candlestick",
      x: timestamps,
      open: data.bars.map((b) => b.open),
      high: data.bars.map((b) => b.high),
      low: data.bars.map((b) => b.low),
      close: data.bars.map((b) => b.close),
      name: data.symbol,
      increasing: { line: { color: "#10b981" } },
      decreasing: { line: { color: "#ef4444" } },
      hoverlabel: { bgcolor: "#1e293b" },
    });
  } else if (currentChartType === "line") {
    traces.push({
      type: "scatter",
      mode: "lines",
      x: timestamps,
      y: data.bars.map((b) => b.close),
      name: "Close",
      line: { color: "#3b82f6", width: 2 },
      hoverlabel: { bgcolor: "#1e293b" },
    });
  } else if (currentChartType === "area") {
    traces.push({
      type: "scatter",
      mode: "lines",
      x: timestamps,
      y: data.bars.map((b) => b.close),
      name: "Close",
      fill: "tozeroy",
      fillcolor: "rgba(59, 130, 246, 0.2)",
      line: { color: "#3b82f6", width: 2 },
      hoverlabel: { bgcolor: "#1e293b" },
    });
  }

  // Add indicators
  if (data.indicators) {
    Object.keys(data.indicators).forEach((key, idx) => {
      const indicator = data.indicators[key];

      if (indicator.type === "line") {
        // SMA/EMA overlay
        const values = indicator.values.map((v) => v.value);
        const ts = indicator.values.map((v) => new Date(v.timestamp * 1000));
        traces.push({
          type: "scatter",
          mode: "lines",
          x: ts,
          y: values,
          name: `Indicator ${idx}`,
          line: { width: 1.5 },
          hoverlabel: { bgcolor: "#1e293b" },
        });
      } else if (indicator.type === "bands") {
        // Bollinger Bands
        const upperTs = indicator.upper.map(
          (v) => new Date(v.timestamp * 1000),
        );
        const middleTs = indicator.middle.map(
          (v) => new Date(v.timestamp * 1000),
        );
        const lowerTs = indicator.lower.map(
          (v) => new Date(v.timestamp * 1000),
        );

        traces.push({
          type: "scatter",
          mode: "lines",
          x: upperTs,
          y: indicator.upper.map((v) => v.value),
          name: "BB Upper",
          line: { color: "#8b5cf6", width: 1, dash: "dot" },
          hoverlabel: { bgcolor: "#1e293b" },
        });
        traces.push({
          type: "scatter",
          mode: "lines",
          x: middleTs,
          y: indicator.middle.map((v) => v.value),
          name: "BB Middle",
          line: { color: "#8b5cf6", width: 1 },
          hoverlabel: { bgcolor: "#1e293b" },
        });
        traces.push({
          type: "scatter",
          mode: "lines",
          x: lowerTs,
          y: indicator.lower.map((v) => v.value),
          name: "BB Lower",
          line: { color: "#8b5cf6", width: 1, dash: "dot" },
          fill: "tonexty",
          fillcolor: "rgba(139, 92, 246, 0.1)",
          hoverlabel: { bgcolor: "#1e293b" },
        });
      }
    });
  }

  // Add trade markers if available from backtest
  if (AppState.backtestResults?.trades) {
    const entryX = [];
    const entryY = [];
    const exitX = [];
    const exitY = [];

    AppState.backtestResults.trades.forEach((trade) => {
      entryX.push(new Date(trade.entry_time));
      entryY.push(trade.entry_price);
      exitX.push(new Date(trade.exit_time));
      exitY.push(trade.exit_price);
    });

    traces.push({
      type: "scatter",
      mode: "markers",
      x: entryX,
      y: entryY,
      name: "Entry",
      marker: {
        color: "#10b981",
        size: 10,
        symbol: "triangle-up",
        line: { color: "white", width: 1 },
      },
      hoverlabel: { bgcolor: "#1e293b" },
    });

    traces.push({
      type: "scatter",
      mode: "markers",
      x: exitX,
      y: exitY,
      name: "Exit",
      marker: {
        color: "#ef4444",
        size: 10,
        symbol: "triangle-down",
        line: { color: "white", width: 1 },
      },
      hoverlabel: { bgcolor: "#1e293b" },
    });
  }

  const layout = {
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor: "rgba(0,0,0,0)",
    font: { color: "#94a3b8" },
    margin: { t: 20, r: 20, b: 40, l: 60 },
    xaxis: {
      gridcolor: "rgba(148, 163, 184, 0.1)",
      showgrid: true,
      type: "date",
    },
    yaxis: {
      gridcolor: "rgba(148, 163, 184, 0.1)",
      showgrid: true,
      title: "Price",
    },
    hovermode: "x unified",
    showlegend: true,
    legend: {
      x: 0,
      y: 1,
      bgcolor: "rgba(30, 41, 59, 0.8)",
    },
  };

  Plotly.newPlot("price-chart", traces, layout, { responsive: true });

  // Render oscillators in separate panels
  renderOscillators(data);
}

function renderOscillators(data) {
  if (!data.indicators) return;

  let hasRsi = false;
  let hasMacd = false;

  Object.keys(data.indicators).forEach((key) => {
    const indicator = data.indicators[key];

    if (indicator.type === "oscillator") {
      hasRsi = true;
      renderRsiChart(indicator);
    } else if (indicator.type === "macd") {
      hasMacd = true;
      renderMacdChart(indicator);
    }
  });

  // Show/hide panels
  document.getElementById("rsi-panel").style.display = hasRsi
    ? "block"
    : "none";
  document.getElementById("macd-panel").style.display = hasMacd
    ? "block"
    : "none";
}

function renderRsiChart(indicator) {
  const timestamps = indicator.values.map((v) => new Date(v.timestamp * 1000));
  const values = indicator.values.map((v) => v.value);

  const traces = [
    {
      type: "scatter",
      mode: "lines",
      x: timestamps,
      y: values,
      name: "RSI",
      line: { color: "#f59e0b", width: 2 },
      hoverlabel: { bgcolor: "#1e293b" },
    },
  ];

  // Add overbought/oversold lines
  if (indicator.upper_bound) {
    traces.push({
      type: "scatter",
      mode: "lines",
      x: timestamps,
      y: Array(timestamps.length).fill(indicator.upper_bound),
      name: "Overbought",
      line: { color: "#ef4444", width: 1, dash: "dash" },
      hoverlabel: { bgcolor: "#1e293b" },
    });
  }
  if (indicator.lower_bound) {
    traces.push({
      type: "scatter",
      mode: "lines",
      x: timestamps,
      y: Array(timestamps.length).fill(indicator.lower_bound),
      name: "Oversold",
      line: { color: "#10b981", width: 1, dash: "dash" },
      hoverlabel: { bgcolor: "#1e293b" },
    });
  }

  const layout = {
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor: "rgba(0,0,0,0)",
    font: { color: "#94a3b8" },
    margin: { t: 10, r: 20, b: 40, l: 60 },
    xaxis: {
      gridcolor: "rgba(148, 163, 184, 0.1)",
      showgrid: true,
      type: "date",
    },
    yaxis: {
      gridcolor: "rgba(148, 163, 184, 0.1)",
      showgrid: true,
      range: [0, 100],
    },
    hovermode: "x unified",
    showlegend: false,
  };

  Plotly.newPlot("rsi-chart", traces, layout, { responsive: true });
}

function renderMacdChart(indicator) {
  const timestamps = indicator.macd.map((v) => new Date(v.timestamp * 1000));

  const traces = [
    {
      type: "scatter",
      mode: "lines",
      x: timestamps,
      y: indicator.macd.map((v) => v.value),
      name: "MACD",
      line: { color: "#3b82f6", width: 2 },
      hoverlabel: { bgcolor: "#1e293b" },
    },
    {
      type: "scatter",
      mode: "lines",
      x: timestamps,
      y: indicator.signal.map((v) => v.value),
      name: "Signal",
      line: { color: "#f59e0b", width: 2 },
      hoverlabel: { bgcolor: "#1e293b" },
    },
    {
      type: "bar",
      x: timestamps,
      y: indicator.histogram.map((v) => v.value),
      name: "Histogram",
      marker: {
        color: indicator.histogram.map((v) =>
          v.value >= 0 ? "#10b981" : "#ef4444",
        ),
      },
      hoverlabel: { bgcolor: "#1e293b" },
    },
  ];

  const layout = {
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor: "rgba(0,0,0,0)",
    font: { color: "#94a3b8" },
    margin: { t: 10, r: 20, b: 40, l: 60 },
    xaxis: {
      gridcolor: "rgba(148, 163, 184, 0.1)",
      showgrid: true,
      type: "date",
    },
    yaxis: {
      gridcolor: "rgba(148, 163, 184, 0.1)",
      showgrid: true,
    },
    hovermode: "x unified",
    showlegend: true,
    legend: {
      x: 0,
      y: 1,
      bgcolor: "rgba(30, 41, 59, 0.8)",
    },
  };

  Plotly.newPlot("macd-chart", traces, layout, { responsive: true });
}

// ===========================
// Reports Logic
// ===========================

let currentTrades = []; // Store trades from last backtest
let currentEquity = []; // Store equity history

async function updatePerformanceReport() {
  if (!currentTrades || currentTrades.length === 0) {
    return;
  }

  const period = document.getElementById("report-period").value;
  const placeholder = document.getElementById("report-placeholder");
  const metricsDiv = document.getElementById("report-metrics");

  placeholder.style.display = "none";
  metricsDiv.style.display = "block";

  try {
    const response = await fetch(`${API_BASE}/reports/summary`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        trades: currentTrades,
        period: period,
        equity_history: currentEquity,
      }),
    });

    const report = await response.json();
    renderPerformanceReport(report);

    const timestamp = new Date().toLocaleTimeString();
    document.getElementById("report-last-generated").textContent =
      `Generated: ${timestamp}`;
  } catch (error) {
    console.error("Error generating report:", error);
    showToast("Error generating report", "error");
  }
}

function renderPerformanceReport(report) {
  // Overall Stats
  document.getElementById("report-total-pnl").textContent = formatCurrency(
    report.overall.total_pnl,
  );
  document.getElementById("report-total-pnl").className =
    `metric-value big ${report.overall.total_pnl >= 0 ? "positive" : "negative"}`;

  document.getElementById("report-win-rate").textContent = formatPercent(
    report.overall.win_rate,
  );
  document.getElementById("report-profit-factor").textContent =
    report.overall.profit_factor.toFixed(2);
  document.getElementById("report-total-trades").textContent =
    report.overall.trade_count;

  // Period Table
  const tbody = document.querySelector("#report-period-table tbody");
  tbody.innerHTML = "";

  const pnlData = {
    x: [],
    y: [],
    type: "bar",
    marker: { color: [] },
  };

  report.summaries.forEach((s) => {
    const row = document.createElement("tr");
    row.innerHTML = `
            <td>${s.period_key}</td>
            <td class="${s.total_pnl >= 0 ? "text-success" : "text-danger"}">${formatCurrency(s.total_pnl)}</td>
            <td>${s.trade_count}</td>
            <td>${formatPercent(s.win_rate)}</td>
            <td class="text-success">${formatCurrency(s.best_trade)}</td>
            <td class="text-danger">${formatCurrency(s.worst_trade)}</td>
        `;
    tbody.appendChild(row);

    // Chart Data
    pnlData.x.push(s.period_key);
    pnlData.y.push(s.total_pnl);
    pnlData.marker.color.push(s.total_pnl >= 0 ? "#2ecc71" : "#e74c3c");
  });

  // Render Chart
  const layout = {
    title: "P&L By Period",
    autosize: true,
    height: 300,
    margin: { t: 30, r: 10, b: 40, l: 60 },
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor: "rgba(0,0,0,0)",
    xaxis: { title: "Period", color: "#888" },
    yaxis: { title: "P&L ($)", color: "#888", gridcolor: "#333" },
    font: { color: "#ccc" },
  };

  Plotly.newPlot("report-pnl-chart", [pnlData], layout, {
    responsive: true,
    displayModeBar: false,
  });
}

async function generateTaxReport() {
  if (!currentTrades || currentTrades.length === 0) {
    showToast("Run a backtest first to generate trades", "warning");
    return;
  }

  const method = document.getElementById("tax-method").value;
  const placeholder = document.getElementById("tax-placeholder");
  const resultsDiv = document.getElementById("tax-results");
  const exportBtn = document.getElementById("btn-export-tax");

  try {
    const response = await fetch(`${API_BASE}/tax/calculate`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        trades: currentTrades,
        method: method,
      }),
    });

    const summary = await response.json();

    placeholder.style.display = "none";
    resultsDiv.style.display = "block";
    exportBtn.disabled = false;

    renderTaxReport(summary);
  } catch (error) {
    console.error("Error calculating tax:", error);
    showToast("Error calculating tax", "error");
  }
}

function renderTaxReport(summary) {
  // Yearly Summary
  const yearlyBody = document.querySelector("#tax-yearly-table tbody");
  yearlyBody.innerHTML = "";

  summary.yearly_summaries.forEach((s) => {
    const row = document.createElement("tr");
    row.innerHTML = `
            <td>${s.year}</td>
            <td class="text-success">${formatCurrency(s.net_short_term)}</td>
            <td class="text-success">${formatCurrency(s.net_long_term)}</td>
            <td class="${s.total_net >= 0 ? "text-success fw-bold" : "text-danger fw-bold"}">${formatCurrency(s.total_net)}</td>
        `;
    yearlyBody.appendChild(row);
  });

  // Events Table
  const eventsBody = document.querySelector("#tax-events-table tbody");
  eventsBody.innerHTML = "";

  summary.events.forEach((e) => {
    const row = document.createElement("tr");
    const termClass = e.is_short_term
      ? "badge bg-warning text-dark"
      : "badge bg-info text-dark";
    const termLabel = e.is_short_term ? "Short" : "Long";

    row.innerHTML = `
            <td>${formatDate(e.sale_date)}</td>
            <td>${e.symbol}</td>
            <td>${e.quantity.toFixed(4)}</td>
            <td>${formatCurrency(e.proceeds)}</td>
            <td>${formatCurrency(e.cost_basis)}</td>
            <td class="${e.gain_loss >= 0 ? "text-success" : "text-danger"}">${formatCurrency(e.gain_loss)}</td>
            <td><span class="${termClass}">${termLabel}</span></td>
        `;
    eventsBody.appendChild(row);
  });
}

async function exportReportCSV() {
  if (!currentTrades || currentTrades.length === 0) return;

  const period = document.getElementById("report-period").value;

  try {
    const response = await fetch(`${API_BASE}/reports/export`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        trades: currentTrades,
        period: period,
      }),
    });

    const blob = await response.blob();
    downloadBlob(blob, `performance_report_${period.toLowerCase()}.csv`);
  } catch (error) {
    console.error("Export failed:", error);
    showToast("CSV export failed", "error");
  }
}

async function exportTaxCSV() {
  if (!currentTrades || currentTrades.length === 0) return;

  const method = document.getElementById("tax-method").value;

  try {
    const response = await fetch(`${API_BASE}/tax/export`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        trades: currentTrades,
        method: method,
      }),
    });

    const blob = await response.blob();
    downloadBlob(blob, `tax_report_${method}.csv`);
  } catch (error) {
    console.error("Export failed:", error);
    showToast("CSV export failed", "error");
  }
}

function updateJournal(trades) {
  const tbody = document.querySelector("#journal-table tbody");
  const emptyMsg = document.getElementById("journal-empty");

  tbody.innerHTML = "";

  if (!trades || trades.length === 0) {
    if (emptyMsg) emptyMsg.style.display = "block";
    return;
  }

  if (emptyMsg) emptyMsg.style.display = "none";

  // Sort by exit time desc
  const sortedTrades = [...trades].sort(
    (a, b) => new Date(b.exit_time) - new Date(a.exit_time),
  );

  sortedTrades.forEach((t) => {
    const row = document.createElement("tr");
    const pnlClass = t.pnl >= 0 ? "text-success" : "text-danger";
    const sideClass =
      t.side === "Long" ? "badge bg-success" : "badge bg-danger";

    row.innerHTML = `
            <td>${formatDate(t.exit_time)}</td>
            <td>${t.symbol}</td>
            <td><span class="${sideClass}">${t.side}</span></td>
            <td class="${pnlClass}">${formatCurrency(t.pnl)}</td>
            <td>
                <span class="badge bg-secondary">#setup</span>
            </td>
            <td>
                <input type="text" class="form-control form-control-sm bg-dark text-light border-secondary" placeholder="Add notes..."
                       onchange="saveJournalNote('${t.symbol}_${new Date(t.entry_time).getTime()}', this.value)">
            </td>
            <td>
                <button class="btn btn-sm btn-outline-light" onclick="editJournalTags('${t.symbol}_${new Date(t.entry_time).getTime()}')">
                    <i class="fas fa-tag"></i>
                </button>
            </td>
        `;
    tbody.appendChild(row);
  });
}

function saveJournalNote(id, note) {
  console.log(`Saving note for ${id}: ${note}`);
  // In a real app we would call the API
  showToast("Note saved", "success");
}

function editJournalTags(id) {
  // Placeholder for tag editor
  const tag = prompt("Add a tag:");
  if (tag) {
    // API call would go here
    showToast(`Tag added: ${tag}`, "success");
  }
}

function downloadBlob(blob, filename) {
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  window.URL.revokeObjectURL(url);
  document.body.removeChild(a);
}

// Utility formatting
function formatCurrency(val) {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
  }).format(val);
}

function formatPercent(val) {
  return (val * 100).toFixed(1) + "%";
}

function formatDate(isoString) {
  return new Date(isoString).toLocaleDateString();
}

// ==================== Advanced Order Management Functions ====================

// Order Queue Management
async function refreshOrders() {
  try {
    const response = await fetch("/api/orders/pending");
    const orders = await response.json();
    updatePendingOrdersTable(orders);
    logMessage("Orders refreshed", "info");
  } catch (error) {
    logMessage(`Failed to refresh orders: ${error.message}`, "error");
  }
}

async function cancelAllOrders() {
  if (!confirm("Are you sure you want to cancel all pending orders?")) {
    return;
  }

  try {
    const response = await fetch("/api/orders/cancel-all/BTCUSDT", {
      method: "POST",
    });
    if (response.ok) {
      logMessage("All orders canceled", "info");
      refreshOrders();
    }
  } catch (error) {
    logMessage(`Failed to cancel orders: ${error.message}`, "error");
  }
}

function updatePendingOrdersTable(orders) {
  const tbody = document.getElementById("pending-orders-table");
  if (!tbody) return;

  if (orders.length === 0) {
    tbody.innerHTML =
      '<tr><td colspan="8" class="text-center text-muted">No pending orders</td></tr>';
    return;
  }

  tbody.innerHTML = orders
    .map(
      (order) => `
        <tr>
            <td>${order.id.substring(0, 8)}</td>
            <td>${order.symbol}</td>
            <td><span class="badge ${order.side === "Buy" ? "badge-success" : "badge-danger"}">${order.side}</span></td>
            <td>${order.order_type}</td>
            <td>${order.quantity}</td>
            <td>${order.price ? "$" + order.price : "Market"}</td>
            <td><span class="badge badge-secondary">${order.status}</span></td>
            <td>
                <button class="btn btn-sm btn-danger" onclick="cancelOrder('${order.id}')">Cancel</button>
            </td>
        </tr>
    `,
    )
    .join("");

  // Update queue summary
  document.getElementById("queue-total-count").textContent = orders.length;
  const totalQty = orders.reduce((sum, o) => sum + o.quantity, 0);
  document.getElementById("queue-total-qty").textContent = totalQty.toFixed(2);
  const uniqueSymbols = new Set(orders.map((o) => o.symbol)).size;
  document.getElementById("queue-symbol-count").textContent = uniqueSymbols;
}

// OCO Order Management
function addOCOOrderRow() {
  const container = document.getElementById("oco-orders-list");
  const row = document.createElement("div");
  row.className = "oco-order-row";
  row.innerHTML = `
        <div class="form-row">
            <div class="form-group">
                <label>Side</label>
                <select class="form-select oco-side">
                    <option value="Buy">Buy</option>
                    <option value="Sell">Sell</option>
                </select>
            </div>
            <div class="form-group">
                <label>Type</label>
                <select class="form-select oco-type" onchange="toggleOCOPrice(this)">
                    <option value="Limit">Limit</option>
                    <option value="Market">Market</option>
                </select>
            </div>
            <div class="form-group oco-price-group">
                <label>Price</label>
                <input type="number" step="0.01" class="form-input oco-price" placeholder="0.00">
            </div>
            <div class="form-group">
                <label>Quantity</label>
                <input type="number" step="0.01" class="form-input oco-qty" placeholder="0.00">
            </div>
            <button type="button" class="btn btn-danger btn-sm" onclick="this.closest('.oco-order-row').remove()">✖</button>
        </div>
    `;
  container.appendChild(row);
}

function toggleOCOPrice(select) {
  const priceGroup = select
    .closest(".form-row")
    .querySelector(".oco-price-group");
  priceGroup.style.display = select.value === "Limit" ? "block" : "none";
}

async function createOCOOrder(event) {
  event.preventDefault();

  const symbol = document.getElementById("oco-symbol").value;
  const rows = document.querySelectorAll(".oco-order-row");

  if (rows.length < 2) {
    alert("OCO requires at least 2 orders");
    return;
  }

  const orders = Array.from(rows).map((row) => {
    const type = row.querySelector(".oco-type").value;
    return {
      id: "",
      symbol,
      side: row.querySelector(".oco-side").value,
      order_type: type,
      quantity: parseFloat(row.querySelector(".oco-qty").value),
      price:
        type === "Limit"
          ? parseFloat(row.querySelector(".oco-price").value)
          : null,
      status: "New",
      timestamp: new Date().toISOString(),
    };
  });

  try {
    const response = await fetch("/api/orders/oco", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ orders }),
    });

    if (response.ok) {
      const result = await response.json();
      logMessage(`OCO order created: ${result.group_id}`, "info");
      refreshOrders();
      event.target.reset();
    } else {
      throw new Error("Failed to create OCO order");
    }
  } catch (error) {
    logMessage(`OCO order error: ${error.message}`, "error");
  }
}

// Bracket Order Management
function toggleBracketEntryPrice() {
  const type = document.getElementById("bracket-entry-type").value;
  document.getElementById("bracket-entry-price-group").style.display =
    type === "Limit" ? "block" : "none";
}

async function createBracketOrder(event) {
  event.preventDefault();

  const entryType = document.getElementById("bracket-entry-type").value;
  const bracketOrder = {
    entry: {
      id: "",
      symbol: document.getElementById("bracket-symbol").value,
      side: "Buy",
      order_type: entryType,
      quantity: parseFloat(document.getElementById("bracket-qty").value),
      price:
        entryType === "Limit"
          ? parseFloat(document.getElementById("bracket-entry-price").value)
          : null,
      status: "New",
      timestamp: new Date().toISOString(),
    },
    stop_loss: parseFloat(document.getElementById("bracket-sl").value),
    stop_loss_qty: document.getElementById("bracket-sl-qty").value
      ? parseFloat(document.getElementById("bracket-sl-qty").value)
      : undefined,
    take_profit: parseFloat(document.getElementById("bracket-tp").value),
    take_profit_qty: document.getElementById("bracket-tp-qty").value
      ? parseFloat(document.getElementById("bracket-tp-qty").value)
      : undefined,
    symbol: document.getElementById("bracket-symbol").value,
  };

  try {
    const response = await fetch("/api/orders/bracket", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(bracketOrder),
    });

    if (response.ok) {
      const result = await response.json();
      logMessage(`Bracket order created: ${result.bracket_id}`, "info");
      refreshOrders();
      event.target.reset();
    } else {
      throw new Error("Failed to create bracket order");
    }
  } catch (error) {
    logMessage(`Bracket order error: ${error.message}`, "error");
  }
}

// Iceberg Order Management
document
  .getElementById("iceberg-visible-qty")
  .addEventListener("input", updateIcebergSlices);
document
  .getElementById("iceberg-total-qty")
  .addEventListener("input", updateIcebergSlices);

function updateIcebergSlices() {
  const visible =
    parseFloat(document.getElementById("iceberg-visible-qty").value) || 0;
  const total =
    parseFloat(document.getElementById("iceberg-total-qty").value) || 0;
  const slices = visible > 0 ? Math.ceil(total / visible) : 0;
  document.getElementById("iceberg-slices").textContent = slices;
}

async function createIcebergOrder(event) {
  event.preventDefault();

  const icebergOrder = {
    symbol: document.getElementById("iceberg-symbol").value,
    side: document.getElementById("iceberg-side").value,
    total_quantity: parseFloat(
      document.getElementById("iceberg-total-qty").value,
    ),
    visible_quantity: parseFloat(
      document.getElementById("iceberg-visible-qty").value,
    ),
    price: parseFloat(document.getElementById("iceberg-price").value),
  };

  try {
    const response = await fetch("/api/orders/iceberg", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(icebergOrder),
    });

    if (response.ok) {
      const result = await response.json();
      logMessage(`Iceberg order created: ${result.iceberg_id}`, "info");
      refreshOrders();
      event.target.reset();
    } else {
      throw new Error("Failed to create iceberg order");
    }
  } catch (error) {
    logMessage(`Iceberg order error: ${error.message}`, "error");
  }
}

// Limit Chase Order Management
async function createLimitChaseOrder(event) {
  event.preventDefault();

  const chaseOrder = {
    order: {
      id: "",
      symbol: document.getElementById("chase-symbol").value,
      side: document.getElementById("chase-side").value,
      order_type: "Limit",
      quantity: parseFloat(document.getElementById("chase-qty").value),
      price: parseFloat(document.getElementById("chase-price").value),
      status: "New",
      timestamp: new Date().toISOString(),
    },
    chase_amount: parseFloat(document.getElementById("chase-amount").value),
    is_percentage: document.getElementById("chase-type").value === "percentage",
    max_adjustments: parseInt(document.getElementById("chase-max-adj").value),
  };

  try {
    const response = await fetch("/api/orders/limit-chase", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(chaseOrder),
    });

    if (response.ok) {
      const result = await response.json();
      logMessage(`Limit chase order created: ${result.chase_id}`, "info");
      refreshOrders();
      event.target.reset();
    } else {
      throw new Error("Failed to create limit chase order");
    }
  } catch (error) {
    logMessage(`Limit chase order error: ${error.message}`, "error");
  }
}

// Position Management
async function scalePosition(event) {
  event.preventDefault();

  const scaleRequest = {
    symbol: document.getElementById("scale-symbol").value,
    quantity: parseFloat(document.getElementById("scale-qty").value),
    price: document.getElementById("scale-price").value
      ? parseFloat(document.getElementById("scale-price").value)
      : undefined,
  };

  const endpoint =
    scaleRequest.quantity > 0
      ? "/api/positions/scale-in"
      : "/api/positions/scale-out";

  try {
    const response = await fetch(endpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(scaleRequest),
    });

    if (response.ok) {
      logMessage(`Position scaled for ${scaleRequest.symbol}`, "info");
      refreshOrders();
      event.target.reset();
    }
  } catch (error) {
    logMessage(`Scale position error: ${error.message}`, "error");
  }
}

async function partialTakeProfit(event) {
  event.preventDefault();

  const tpRequest = {
    symbol: document.getElementById("tp-symbol").value,
    close_qty: parseFloat(document.getElementById("tp-close-qty").value),
    target_price: parseFloat(document.getElementById("tp-price").value),
    remaining_qty: parseFloat(document.getElementById("tp-remaining").value),
  };

  try {
    const response = await fetch("/api/positions/partial-tp", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(tpRequest),
    });

    if (response.ok) {
      logMessage(`Partial TP submitted for ${tpRequest.symbol}`, "info");
      event.target.reset();
    }
  } catch (error) {
    logMessage(`Partial TP error: ${error.message}`, "error");
  }
}

async function moveBreakEvenStop(event) {
  event.preventDefault();

  const beRequest = {
    symbol: document.getElementById("be-symbol").value,
    entry_price: parseFloat(document.getElementById("be-entry").value),
    stop_offset: parseFloat(document.getElementById("be-offset").value),
  };

  try {
    const response = await fetch("/api/positions/break-even", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(beRequest),
    });

    if (response.ok) {
      logMessage(`Break-even stop set for ${beRequest.symbol}`, "info");
      event.target.reset();
    }
  } catch (error) {
    logMessage(`Break-even stop error: ${error.message}`, "error");
  }
}

// Load advanced orders on page load
async function loadActiveAdvancedOrders() {
  try {
    // Load OCO orders
    const ocoResponse = await fetch("/api/orders/oco");
    const ocoOrders = await ocoResponse.json();
    updateActiveOCOOrders(ocoOrders);

    // Load bracket orders
    const bracketResponse = await fetch("/api/orders/bracket");
    const bracketOrders = await bracketResponse.json();
    updateActiveBracketOrders(bracketOrders);

    // Load iceberg orders
    const icebergResponse = await fetch("/api/orders/iceberg");
    const icebergOrders = await icebergResponse.json();
    updateActiveIcebergOrders(icebergOrders);

    // Load limit chase orders
    const chaseResponse = await fetch("/api/orders/limit-chase");
    const chaseOrders = await chaseResponse.json();
    updateActiveLimitChaseOrders(chaseOrders);
  } catch (error) {
    console.error("Failed to load advanced orders:", error);
  }
}

function updateActiveOCOOrders(orders) {
  const container = document.getElementById("active-oco-orders");
  if (!container) return;

  if (orders.length === 0) {
    container.innerHTML =
      '<div class="empty-section">No active OCO orders</div>';
    return;
  }

  container.innerHTML = orders
    .map(
      (oco) => `
        <div class="advanced-order-card">
            <div class="order-header">
                <strong>${oco.group_id.substring(0, 8)}</strong>
                <span class="badge ${oco.active ? "badge-success" : "badge-secondary"}">
                    ${oco.active ? "Active" : "Inactive"}
                </span>
            </div>
            <div class="order-details">
                <p><strong>Orders:</strong> ${oco.orders.length}</p>
                <p><strong>Created:</strong> ${new Date(oco.timestamp).toLocaleString()}</p>
                ${oco.filled_order_id ? `<p><strong>Filled:</strong> ${oco.filled_order_id.substring(0, 8)}</p>` : ""}
            </div>
            <button class="btn btn-sm btn-danger" onclick="cancelOCOOrder('${oco.group_id}')">Cancel</button>
        </div>
    `,
    )
    .join("");
}

function updateActiveBracketOrders(orders) {
  const container = document.getElementById("active-bracket-orders");
  if (!container) return;

  if (orders.length === 0) {
    container.innerHTML =
      '<div class="empty-section">No active bracket orders</div>';
    return;
  }

  container.innerHTML = orders
    .map(
      (bracket) => `
        <div class="advanced-order-card">
            <div class="order-header">
                <strong>${bracket.bracket_id.substring(0, 8)}</strong>
                <span class="badge badge-info">${bracket.state}</span>
            </div>
            <div class="order-details">
                <p><strong>Entry:</strong> ${bracket.entry_order.price ? "$" + bracket.entry_order.price : "Market"}</p>
                <p><strong>Stop Loss:</strong> $${bracket.stop_loss_order.price}</p>
                <p><strong>Take Profit:</strong> $${bracket.take_profit_order.price}</p>
            </div>
            <button class="btn btn-sm btn-danger" onclick="cancelBracketOrder('${bracket.bracket_id}')">Cancel</button>
        </div>
    `,
    )
    .join("");
}

function updateActiveIcebergOrders(orders) {
  const container = document.getElementById("active-iceberg-orders");
  if (!container) return;

  if (orders.length === 0) {
    container.innerHTML =
      '<div class="empty-section">No active iceberg orders</div>';
    return;
  }

  container.innerHTML = orders
    .map(
      (iceberg) => `
        <div class="advanced-order-card">
            <div class="order-header">
                <strong>${iceberg.iceberg_id.substring(0, 8)}</strong>
                <span class="badge ${iceberg.active ? "badge-success" : "badge-secondary"}">
                    ${iceberg.active ? "Active" : "Inactive"}
                </span>
            </div>
            <div class="order-details">
                <p><strong>Symbol:</strong> ${iceberg.symbol}</p>
                <p><strong>Total:</strong> ${iceberg.total_quantity}</p>
                <p><strong>Visible:</strong> ${iceberg.visible_quantity}</p>
                <p><strong>Remaining:</strong> ${iceberg.total_quantity - (iceberg.filled_slices?.reduce((s, [_, qty]) => s + qty, 0) || 0)}</p>
            </div>
            <button class="btn btn-sm btn-danger" onclick="cancelIcebergOrder('${iceberg.iceberg_id}')">Cancel</button>
        </div>
    `,
    )
    .join("");
}

function updateActiveLimitChaseOrders(orders) {
  const container = document.getElementById("active-chase-orders");
  if (!container) return;

  if (orders.length === 0) {
    container.innerHTML =
      '<div class="empty-section">No active limit chase orders</div>';
    return;
  }

  container.innerHTML = orders
    .map(
      (chase) => `
        <div class="advanced-order-card">
            <div class="order-header">
                <strong>${chase.chase_id.substring(0, 8)}</strong>
                <span class="badge ${chase.active ? "badge-success" : "badge-secondary"}">
                    ${chase.active ? "Active" : "Inactive"}
                </span>
            </div>
            <div class="order-details">
                <p><strong>Current Limit:</strong> $${chase.order.price}</p>
                <p><strong>Chase:</strong> ${chase.chase_amount}${chase.is_percentage ? "%" : ""}</p>
                <p><strong>Adjustments:</strong> ${chase.adjustments}/${chase.max_adjustments}</p>
            </div>
            <button class="btn btn-sm btn-danger" onclick="cancelLimitChaseOrder('${chase.chase_id}')">Cancel</button>
        </div>
    `,
    )
    .join("");
}

// Cancel advanced orders
async function cancelOCOOrder(groupId) {
  try {
    const response = await fetch(`/api/orders/oco/${groupId}/cancel`, {
      method: "POST",
    });
    if (response.ok) {
      logMessage(`OCO order ${groupId.substring(0, 8)} canceled`, "info");
      loadActiveAdvancedOrders();
    }
  } catch (error) {
    logMessage(`Failed to cancel OCO: ${error.message}`, "error");
  }
}

async function cancelBracketOrder(bracketId) {
  try {
    const response = await fetch(`/api/orders/bracket/${bracketId}/cancel`, {
      method: "POST",
    });
    if (response.ok) {
      logMessage(`Bracket order ${bracketId.substring(0, 8)} canceled`, "info");
      loadActiveAdvancedOrders();
    }
  } catch (error) {
    logMessage(`Failed to cancel bracket: ${error.message}`, "error");
  }
}

async function cancelIcebergOrder(icebergId) {
  try {
    const response = await fetch(`/api/orders/iceberg/${icebergId}/cancel`, {
      method: "POST",
    });
    if (response.ok) {
      logMessage(`Iceberg order ${icebergId.substring(0, 8)} canceled`, "info");
      loadActiveAdvancedOrders();
    }
  } catch (error) {
    logMessage(`Failed to cancel iceberg: ${error.message}`, "error");
  }
}

async function cancelLimitChaseOrder(chaseId) {
  try {
    const response = await fetch(`/api/orders/limit-chase/${chaseId}/cancel`, {
      method: "POST",
    });
    if (response.ok) {
      logMessage(`Limit chase ${chaseId.substring(0, 8)} canceled`, "info");
      loadActiveAdvancedOrders();
    }
  } catch (error) {
    logMessage(`Failed to cancel limit chase: ${error.message}`, "error");
  }
}

// Initialize orders tab when switching
document.addEventListener("DOMContentLoaded", function () {
  // Initialize order forms
  const ordersTab = document.querySelector('[data-tab="orders"]');
  if (ordersTab) {
    ordersTab.addEventListener("click", function () {
      loadActiveAdvancedOrders();
      refreshOrders();
    });
  }
});
