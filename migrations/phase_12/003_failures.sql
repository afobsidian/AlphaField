-- Phase 12.1: Strategy Library Expansion - Strategy Failures Table
-- Migration: 003_failures.sql
-- Description: Create the strategy_failures table to track failure modes and mitigation strategies

CREATE TABLE IF NOT EXISTS strategy_failures (
    id SERIAL PRIMARY KEY,
    strategy_id INTEGER NOT NULL REFERENCES strategies(id) ON DELETE CASCADE,
    failure_type VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    trigger_conditions TEXT,
    mitigation_strategy TEXT,
    severity VARCHAR(20) DEFAULT 'medium',
    documented_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_failures_strategy ON strategy_failures(strategy_id);
CREATE INDEX IF NOT EXISTS idx_failures_severity ON strategy_failures(severity);
CREATE INDEX IF NOT EXISTS idx_failures_type ON strategy_failures(failure_type);
CREATE INDEX IF NOT EXISTS idx_failures_documented_at ON strategy_failures(documented_at DESC);

-- Check constraint for severity values
ALTER TABLE strategy_failures
ADD CONSTRAINT check_failure_severity
CHECK (severity IN ('low', 'medium', 'high', 'critical'));

-- Comment on table and columns
COMMENT ON TABLE strategy_failures IS 'Tracks documented failure modes and mitigation strategies for trading strategies';
COMMENT ON COLUMN strategy_failures.strategy_id IS 'Foreign key to strategies table';
COMMENT ON COLUMN strategy_failures.failure_type IS 'Type of failure (e.g., whipsaw, regime_change, signal_quality)';
COMMENT ON COLUMN strategy_failures.description IS 'Detailed description of the failure mode';
COMMENT ON COLUMN strategy_failures.trigger_conditions IS 'Conditions that trigger this failure mode';
COMMENT ON COLUMN strategy_failures.mitigation_strategy IS 'Strategies to prevent or minimize impact of this failure';
COMMENT ON COLUMN strategy_failures.severity IS 'Severity level (low, medium, high, critical)';
COMMENT ON COLUMN strategy_failures.documented_at IS 'When this failure mode was documented';

-- Create enum type for failure categories if not exists
CREATE TYPE IF NOT EXISTS failure_category AS ENUM (
    'signal_generation',
    'market_regime',
    'volatility',
    'liquidity',
    'technical',
    'external',
    'other'
);

-- Add failure_category column for better organization
ALTER TABLE strategy_failures
ADD COLUMN IF NOT EXISTS failure_category failure_category DEFAULT 'other';

-- Update index to include failure_category
DROP INDEX IF EXISTS idx_failures_type;
CREATE INDEX idx_failures_type ON strategy_failures(failure_type);
CREATE INDEX idx_failures_category ON strategy_failures(failure_category);

COMMENT ON COLUMN strategy_failures.failure_category IS 'Category of failure mode for organization and filtering';

-- Create function to automatically add common failure modes
CREATE OR REPLACE FUNCTION document_failure_mode(
    p_strategy_id INTEGER,
    p_failure_category VARCHAR,
    p_failure_type VARCHAR,
    p_description TEXT,
    p_trigger_conditions TEXT DEFAULT NULL,
    p_mitigation_strategy TEXT DEFAULT NULL,
    p_severity VARCHAR DEFAULT 'medium'
)
RETURNS INTEGER AS $$
DECLARE
    new_failure_id INTEGER;
BEGIN
    INSERT INTO strategy_failures (
        strategy_id,
        failure_category,
        failure_type,
        description,
        trigger_conditions,
        mitigation_strategy,
        severity
    )
    VALUES (
        p_strategy_id,
        p_failure_category::failure_category,
        p_failure_type,
        p_description,
        p_trigger_conditions,
        p_mitigation_strategy,
        p_severity
    )
    RETURNING id INTO new_failure_id;

    RETURN new_failure_id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION document_failure_mode IS 'Helper function to document a new failure mode for a strategy';

-- Create view for strategy failures with strategy name
CREATE OR REPLACE VIEW v_strategy_failures AS
SELECT
    sf.id,
    s.name AS strategy_name,
    sf.strategy_id,
    sf.failure_category,
    sf.failure_type,
    sf.description,
    sf.trigger_conditions,
    sf.mitigation_strategy,
    sf.severity,
    sf.documented_at
FROM strategy_failures sf
JOIN strategies s ON sf.strategy_id = s.id
ORDER BY sf.severity DESC, sf.documented_at DESC;

COMMENT ON VIEW v_strategy_failures IS 'View of strategy failures with strategy name for easier querying';
