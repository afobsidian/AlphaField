-- Phase 12.1: Strategy Library Expansion - Strategies Table
-- Migration: 001_strategies.sql
-- Description: Create the strategies table to store strategy metadata

CREATE TABLE IF NOT EXISTS strategies (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    category VARCHAR(50) NOT NULL,
    sub_type VARCHAR(50),
    description TEXT,
    hypothesis_path TEXT NOT NULL,
    required_indicators JSONB,
    expected_regimes JSONB,
    risk_profile JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE,
    UNIQUE(name)
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_strategies_name ON strategies(name);
CREATE INDEX IF NOT EXISTS idx_strategies_category ON strategies(category);
CREATE INDEX IF NOT EXISTS idx_strategies_is_active ON strategies(is_active);
CREATE INDEX IF NOT EXISTS idx_strategies_created_at ON strategies(created_at);

-- Create a trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_strategies_updated_at BEFORE UPDATE ON strategies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE strategies IS 'Stores metadata for all trading strategies';
COMMENT ON COLUMN strategies.name IS 'Unique strategy name';
COMMENT ON COLUMN strategies.category IS 'Strategy category (TrendFollowing, MeanReversion, etc.)';
COMMENT ON COLUMN strategies.sub_type IS 'Optional sub-type for finer classification';
COMMENT ON COLUMN strategies.description IS 'Human-readable strategy description';
COMMENT ON COLUMN strategies.hypothesis_path IS 'Path to hypothesis documentation file';
COMMENT ON COLUMN strategies.required_indicators IS 'JSON array of required indicator names';
COMMENT ON COLUMN strategies.expected_regimes IS 'JSON array of expected market regimes';
COMMENT ON COLUMN strategies.risk_profile IS 'JSON object with risk profile details';
COMMENT ON COLUMN strategies.is_active IS 'Whether the strategy is currently active';
