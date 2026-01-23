-- Meta-Learning Pattern Storage Schema
-- Creates tables for storing and retrieving reasoning patterns

CREATE TABLE IF NOT EXISTS reasoning_patterns (
    -- Identity
    pattern_id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Query Signature
    domain TEXT NOT NULL,
    complexity REAL NOT NULL CHECK (complexity >= 0 AND complexity <= 1),
    keywords TEXT[] NOT NULL,
    elp_dominant CHAR(1) NOT NULL CHECK (elp_dominant IN ('E', 'L', 'P')),
    
    -- ELP Profile (Ethos, Logos, Pathos)
    ethos DOUBLE PRECISION NOT NULL CHECK (ethos >= 0),
    logos DOUBLE PRECISION NOT NULL CHECK (logos >= 0),
    pathos DOUBLE PRECISION NOT NULL CHECK (pathos >= 0),
    
    -- Solution Pathway
    entropy_type TEXT NOT NULL,
    vortex_path INTEGER[] NOT NULL, -- Vortex positions visited (1→2→4→8→7→5→1)
    sacred_influences INTEGER[] NOT NULL, -- Trinity positions (3, 6, 9)
    oracle_questions TEXT[] NOT NULL, -- Effective questions asked
    transformations JSONB NOT NULL, -- Key transformations in reasoning
    
    -- Effectiveness Metrics
    success_rate REAL NOT NULL DEFAULT 1.0 CHECK (success_rate >= 0 AND success_rate <= 1),
    avg_steps INTEGER NOT NULL CHECK (avg_steps > 0),
    confidence_achieved REAL NOT NULL CHECK (confidence_achieved >= 0 AND confidence_achieved <= 1),
    reuse_count INTEGER NOT NULL DEFAULT 0 CHECK (reuse_count >= 0),
    
    -- Quality Signals
    confidence REAL NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    efficiency_score REAL NOT NULL CHECK (efficiency_score >= 0)
);

-- Performance Indexes
CREATE INDEX IF NOT EXISTS idx_patterns_domain 
    ON reasoning_patterns(domain);

CREATE INDEX IF NOT EXISTS idx_patterns_success 
    ON reasoning_patterns(success_rate DESC) 
    WHERE success_rate >= 0.5;

CREATE INDEX IF NOT EXISTS idx_patterns_signal 
    ON reasoning_patterns(confidence DESC)
    WHERE confidence >= 0.6;

CREATE INDEX IF NOT EXISTS idx_patterns_elp_dominant 
    ON reasoning_patterns(elp_dominant);

CREATE INDEX IF NOT EXISTS idx_patterns_domain_elp 
    ON reasoning_patterns(domain, elp_dominant);

CREATE INDEX IF NOT EXISTS idx_patterns_reuse 
    ON reasoning_patterns(reuse_count DESC);

CREATE INDEX IF NOT EXISTS idx_patterns_updated 
    ON reasoning_patterns(updated_at DESC);

-- Composite index for common queries
CREATE INDEX IF NOT EXISTS idx_patterns_lookup 
    ON reasoning_patterns(domain, success_rate DESC, confidence DESC)
    WHERE success_rate >= 0.5;

-- GIN index for keyword search
CREATE INDEX IF NOT EXISTS idx_patterns_keywords 
    ON reasoning_patterns USING GIN(keywords);

-- JSONB index for transformation queries
CREATE INDEX IF NOT EXISTS idx_patterns_transformations 
    ON reasoning_patterns USING GIN(transformations);

-- Comments for documentation
COMMENT ON TABLE reasoning_patterns IS 'Stores learned reasoning patterns for query acceleration';
COMMENT ON COLUMN reasoning_patterns.pattern_id IS 'Unique identifier for the pattern';
COMMENT ON COLUMN reasoning_patterns.domain IS 'Domain category (health, math, ethics, etc.)';
COMMENT ON COLUMN reasoning_patterns.complexity IS 'Query complexity score (0.0-1.0)';
COMMENT ON COLUMN reasoning_patterns.elp_dominant IS 'Dominant ELP dimension: E=Ethos, L=Logos, P=Pathos';
COMMENT ON COLUMN reasoning_patterns.vortex_path IS 'Sequence of vortex positions visited during reasoning';
COMMENT ON COLUMN reasoning_patterns.sacred_influences IS 'Sacred trinity positions (3, 6, 9) that influenced reasoning';
COMMENT ON COLUMN reasoning_patterns.confidence IS 'Trinity coherence signal (≥0.6 for Confidence Lake storage)';
COMMENT ON COLUMN reasoning_patterns.efficiency_score IS 'Steps vs baseline efficiency (>1.0 = better than average)';

-- View for high-quality patterns
CREATE OR REPLACE VIEW high_quality_patterns AS
SELECT 
    pattern_id,
    domain,
    success_rate,
    reuse_count,
    confidence,
    avg_steps,
    created_at
FROM reasoning_patterns
WHERE success_rate >= 0.8
  AND confidence >= 0.7
  AND reuse_count > 0
ORDER BY success_rate DESC, reuse_count DESC;

COMMENT ON VIEW high_quality_patterns IS 'Patterns with high success rate, strong signal, and proven reuse';

-- View for pattern statistics by domain
CREATE OR REPLACE VIEW pattern_stats_by_domain AS
SELECT 
    domain,
    COUNT(*) as total_patterns,
    AVG(success_rate) as avg_success,
    AVG(confidence) as avg_signal,
    AVG(avg_steps) as avg_steps,
    SUM(reuse_count) as total_reuses
FROM reasoning_patterns
GROUP BY domain
ORDER BY total_reuses DESC;

COMMENT ON VIEW pattern_stats_by_domain IS 'Aggregate statistics of patterns grouped by domain';
