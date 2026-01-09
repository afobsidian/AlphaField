-- Phase 12.1: Strategy Library Expansion - Strategy Performance Table
-- Migration: 002_performance.sql
-- Description: Create the strategy_performance table to store backtest results

CREATE TABLE IF NOT EXISTS strategy_performance (
    id SERIAL PRIMARY KEY,
    strategy_id INTEGER NOT NULL REFERENCES strategies(id) ON DELETE CASCADE,
    symbol VARCHAR(20) NOT NULL,
    timeframe VARCHAR(10) NOT NULL,
    test_period_start TIMESTAMPTZ NOT NULL,
    test_period_end TIMESTAMPTZ NOT NULL,

    -- Performance metrics
    total_return DECIMAL(10, 4) NOT NULL,
    sharpe_ratio DECIMAL(10, 4),
    sortino_ratio DECIMAL(10, 4),
    max_drawdown DECIMAL(10, 4) NOT NULL,
    max_drawdown_duration_days INTEGER,
    win_rate DECIMAL(5, 2),
    profit_factor DECIMAL(10, 2),
    expectancy DECIMAL(10, 4),
    sqn DECIMAL(10, 4), -- System Quality Number
    total_trades INTEGER NOT NULL,
    avg_trade_duration_hours DECIMAL(10, 2),
    avg_win DECIMAL(10, 4),
    avg_loss DECIMAL(10, 4),
    best_trade DECIMAL(10, 4),
    worst_trade DECIMAL(10, 4),

    -- Regime performance
    regime_bull_return DECIMAL(10, 4),
    regime_bear_return DECIMAL(10, 4),
    regime_sideways_return DECIMAL(10, 4),
    regime_high_vol_return DECIMAL(10, 4),
    regime_low_vol_return DECIMAL(10, 4),

    -- Validation metrics
    robustness_score DECIMAL(5, 2),
    walk_forward_stability DECIMAL(5, 2),
    overfitting_risk VARCHAR(20),
    monte_carlo_95ci_low DECIMAL(10, 4),
    monte_carlo_95ci_high DECIMAL(10, 4),
    parameter_cv DECIMAL(5, 2), -- Coefficient of variation for parameters

    -- Market correlation
    market_correlation DECIMAL(5, 2),

    -- Comparison to baselines
    vs_hodl_return DECIMAL(10, 4),
    vs_market_avg_return DECIMAL(10, 4),

    -- Additional metadata
    notes TEXT,
    parameters JSONB,

    tested_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(strategy_id, symbol, timeframe, test_period_start, test_period_end)
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_perf_strategy ON strategy_performance(strategy_id);
CREATE INDEX IF NOT EXISTS idx_perf_symbol ON strategy_performance(symbol);
CREATE INDEX IF NOT EXISTS idx_perf_timeframe ON strategy_performance(timeframe);
CREATE INDEX IF NOT EXISTS idx_perf_period ON strategy_performance(test_period_start, test_period_end);
CREATE INDEX IF NOT EXISTS idx_perf_return ON strategy_performance(total_return DESC);
CREATE INDEX IF NOT EXISTS idx_perf_sharpe ON strategy_performance(sharpe_ratio DESC);
CREATE INDEX IF NOT EXISTS idx_perf_robustness ON strategy_performance(robustness_score DESC);
CREATE INDEX IF NOT EXISTS idx_perf_tested_at ON strategy_performance(tested_at DESC);

-- Comment on table and columns
COMMENT ON TABLE strategy_performance IS 'Stores backtest results and performance metrics for strategies';
COMMENT ON COLUMN strategy_performance.strategy_id IS 'Foreign key to strategies table';
COMMENT ON COLUMN strategy_performance.symbol IS 'Trading symbol (e.g., BTC, ETH)';
COMMENT ON COLUMN strategy_performance.timeframe IS 'Bar timeframe (e.g., 1d, 4h, 1h)';
COMMENT ON COLUMN strategy_performance.total_return IS 'Total return percentage over test period';
COMMENT ON COLUMN strategy_performance.sharpe_ratio IS 'Risk-adjusted return metric';
COMMENT ON COLUMN strategy_performance.sortino_ratio IS 'Risk-adjusted return using downside deviation';
COMMENT ON COLUMN strategy_performance.max_drawdown IS 'Maximum peak-to-trough drawdown percentage';
COMMENT ON COLUMN strategy_performance.win_rate IS 'Percentage of winning trades (0-100)';
COMMENT ON COLUMN strategy_performance.profit_factor IS 'Ratio of gross profit to gross loss';
COMMENT ON COLUMN strategy_performance.expectancy IS 'Average profit/loss per trade';
COMMENT ON COLUMN strategy_performance.sqn IS 'System Quality Number from Van Tharp';
COMMENT ON COLUMN strategy_performance.regime_bull_return IS 'Performance during bull market regime';
COMMENT ON COLUMN strategy_performance.regime_bear_return IS 'Performance during bear market regime';
COMMENT ON COLUMN strategy_performance.regime_high_vol_return IS 'Performance during high volatility regime';
COMMENT ON COLUMN strategy_performance.regime_low_vol_return IS 'Performance during low volatility regime';
COMMENT ON COLUMN strategy_performance.robustness_score IS 'Overall robustness score (0-100)';
COMMENT ON COLUMN strategy_performance.walk_forward_stability IS 'Walk-forward analysis stability score (0-100)';
COMMENT ON COLUMN strategy_performance.overfitting_risk IS 'Assessed overfitting risk (low/medium/high)';
COMMENT ON COLUMN strategy_performance.monte_carlo_95ci_low IS 'Lower bound of 95% confidence interval from Monte Carlo';
COMMENT ON COLUMN strategy_performance.monte_carlo_95ci_high IS 'Upper bound of 95% confidence interval from Monte Carlo';
COMMENT ON COLUMN strategy_performance.parameter_cv IS 'Coefficient of variation across parameter optimization';
COMMENT ON COLUMN strategy_performance.market_correlation IS 'Correlation with overall market performance';
COMMENT ON COLUMN strategy_performance.vs_hodl_return IS 'Performance relative to HODL baseline';
COMMENT ON COLUMN strategy_performance.vs_market_avg_return IS 'Performance relative to market average baseline';
