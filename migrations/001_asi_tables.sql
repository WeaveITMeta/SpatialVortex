-- Migration 001: ASI Infrastructure Tables
-- Created: October 27, 2025
-- Purpose: Add tables for ONNX models, ASI inference, and sacred geometry tracking

-- ============================================================
-- 1. ONNX Model Registry
-- ============================================================
CREATE TABLE IF NOT EXISTS onnx_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT UNIQUE NOT NULL,
    model_path TEXT NOT NULL,
    tokenizer_path TEXT NOT NULL,
    embedding_dim INTEGER NOT NULL DEFAULT 384,
    status TEXT NOT NULL DEFAULT 'ready' CHECK (status IN ('ready', 'loading', 'error', 'disabled')),
    loaded_at TIMESTAMP WITH TIME ZONE,
    performance_metrics JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_onnx_models_status ON onnx_models(status);
CREATE INDEX IF NOT EXISTS idx_onnx_models_name ON onnx_models(name);

COMMENT ON TABLE onnx_models IS 'Registry of ONNX models for ASI inference with performance tracking';
COMMENT ON COLUMN onnx_models.performance_metrics IS 'JSON: {avg_inference_ms, accuracy, total_runs}';

-- ============================================================
-- 2. BeamTensor Storage
-- ============================================================
CREATE TABLE IF NOT EXISTS beam_tensors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    digits REAL[9] NOT NULL,
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    elp_channels JSONB NOT NULL,
    flux_position INTEGER NOT NULL CHECK (flux_position >= 0 AND flux_position <= 9),
    is_sacred BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_beam_tensors_signal ON beam_tensors(confidence DESC);
CREATE INDEX IF NOT EXISTS idx_beam_tensors_sacred ON beam_tensors(is_sacred) WHERE is_sacred = TRUE;
CREATE INDEX IF NOT EXISTS idx_beam_tensors_position ON beam_tensors(flux_position);

COMMENT ON TABLE beam_tensors IS 'Storage for BeamTensor objects with sacred geometry metadata';
COMMENT ON COLUMN beam_tensors.elp_channels IS 'JSON: {ethos: float, logos: float, pathos: float}';

-- ============================================================
-- 3. ASI Inference History
-- ============================================================
CREATE TABLE IF NOT EXISTS asi_inferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    input_text TEXT NOT NULL,
    beam_tensor_id UUID REFERENCES beam_tensors(id) ON DELETE SET NULL,
    onnx_model_id UUID REFERENCES onnx_models(id) ON DELETE SET NULL,
    semantic_embedding REAL[],
    flux_position INTEGER NOT NULL CHECK (flux_position >= 0 AND flux_position <= 9),
    archetype TEXT,
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    hallucination_detected BOOLEAN NOT NULL DEFAULT FALSE,
    vortex_intervention BOOLEAN NOT NULL DEFAULT FALSE,
    lake_worthy BOOLEAN NOT NULL DEFAULT FALSE,
    processing_time_ms INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_asi_inferences_signal ON asi_inferences(confidence DESC);
CREATE INDEX IF NOT EXISTS idx_asi_inferences_hallucination ON asi_inferences(hallucination_detected);
CREATE INDEX IF NOT EXISTS idx_asi_inferences_lake_worthy ON asi_inferences(lake_worthy) WHERE lake_worthy = TRUE;
CREATE INDEX IF NOT EXISTS idx_asi_inferences_model ON asi_inferences(onnx_model_id);
CREATE INDEX IF NOT EXISTS idx_asi_inferences_created ON asi_inferences(created_at DESC);

COMMENT ON TABLE asi_inferences IS 'Complete history of ASI inference operations with hallucination detection';
COMMENT ON COLUMN asi_inferences.lake_worthy IS 'Whether this inference qualifies for Confidence Lake storage (signal >= 0.6)';

-- ============================================================
-- 4. Sacred Position Interventions
-- ============================================================
CREATE TABLE IF NOT EXISTS sacred_interventions (
    id SERIAL PRIMARY KEY,
    inference_id UUID NOT NULL REFERENCES asi_inferences(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position IN (3, 6, 9)),
    signal_before REAL NOT NULL,
    signal_after REAL NOT NULL,
    confidence_boost REAL NOT NULL,
    intervention_type TEXT NOT NULL CHECK (intervention_type IN ('magnification', 'reset', 'stabilization')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sacred_interventions_inference ON sacred_interventions(inference_id);
CREATE INDEX IF NOT EXISTS idx_sacred_interventions_position ON sacred_interventions(position);
CREATE INDEX IF NOT EXISTS idx_sacred_interventions_created ON sacred_interventions(created_at DESC);

COMMENT ON TABLE sacred_interventions IS 'Track sacred position interventions (3-6-9 triangle) and their effectiveness';
COMMENT ON COLUMN sacred_interventions.confidence_boost IS 'Typically 0.15 (15% boost) at sacred positions';

-- ============================================================
-- 5. Context Preservation Metrics
-- ============================================================
CREATE TABLE IF NOT EXISTS context_metrics (
    id SERIAL PRIMARY KEY,
    inference_id UUID NOT NULL REFERENCES asi_inferences(id) ON DELETE CASCADE,
    token_count INTEGER NOT NULL,
    context_window_size INTEGER NOT NULL DEFAULT 4096,
    retention_rate REAL NOT NULL CHECK (retention_rate >= 0.0 AND retention_rate <= 1.0),
    sacred_checkpoints INTEGER[] NOT NULL DEFAULT '{}',
    vortex_cycle_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_context_metrics_inference ON context_metrics(inference_id);
CREATE INDEX IF NOT EXISTS idx_context_metrics_retention ON context_metrics(retention_rate DESC);

COMMENT ON TABLE context_metrics IS 'Context preservation metrics for Vortex Context Preserver (VCP)';
COMMENT ON COLUMN context_metrics.retention_rate IS 'Percentage of context preserved vs linear transformer baseline';

-- ============================================================
-- 6. Training Samples
-- ============================================================
CREATE TABLE IF NOT EXISTS training_samples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    input_text TEXT NOT NULL,
    expected_position INTEGER NOT NULL CHECK (expected_position >= 0 AND expected_position <= 9),
    expected_elp JSONB NOT NULL,
    actual_position INTEGER,
    actual_elp JSONB,
    loss REAL,
    epoch INTEGER,
    model_id UUID REFERENCES onnx_models(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_training_samples_model ON training_samples(model_id);
CREATE INDEX IF NOT EXISTS idx_training_samples_epoch ON training_samples(epoch);

COMMENT ON TABLE training_samples IS 'Training data for fine-tuning ASI models with expected vs actual outputs';

-- ============================================================
-- 7. Materialized View: ASI Performance Summary
-- ============================================================
CREATE MATERIALIZED VIEW IF NOT EXISTS asi_performance_summary AS
SELECT 
    DATE_TRUNC('hour', created_at) as hour,
    COUNT(*) as total_inferences,
    AVG(confidence) as avg_confidence,
    AVG(confidence) as avg_confidence,
    SUM(CASE WHEN hallucination_detected THEN 1 ELSE 0 END) as hallucination_count,
    SUM(CASE WHEN vortex_intervention THEN 1 ELSE 0 END) as intervention_count,
    SUM(CASE WHEN lake_worthy THEN 1 ELSE 0 END) as lake_worthy_count,
    AVG(processing_time_ms) as avg_processing_time_ms,
    COUNT(DISTINCT onnx_model_id) as models_used
FROM asi_inferences
GROUP BY DATE_TRUNC('hour', created_at)
ORDER BY hour DESC;

CREATE UNIQUE INDEX IF NOT EXISTS idx_asi_perf_summary_hour ON asi_performance_summary(hour);

COMMENT ON MATERIALIZED VIEW asi_performance_summary IS 'Hourly rollup of ASI performance metrics for dashboards';

-- ============================================================
-- 8. Functions: Auto-update timestamps
-- ============================================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_onnx_models_updated_at
    BEFORE UPDATE ON onnx_models
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================
-- 9. Function: Refresh materialized view
-- ============================================================
CREATE OR REPLACE FUNCTION refresh_asi_performance_summary()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY asi_performance_summary;
END;
$$ LANGUAGE plpgsql;

-- Create refresh schedule (run every 15 minutes)
-- Note: Requires pg_cron extension
-- SELECT cron.schedule('refresh-asi-perf', '*/15 * * * *', 'SELECT refresh_asi_performance_summary()');

-- ============================================================
-- 10. Seed Data: Default ONNX Model
-- ============================================================
INSERT INTO onnx_models (name, model_path, tokenizer_path, embedding_dim, status)
VALUES (
    'default_embeddings',
    'models/all-MiniLM-L6-v2.onnx',
    'models/tokenizer.json',
    384,
    'ready'
) ON CONFLICT (name) DO NOTHING;

-- ============================================================
-- Migration Complete
-- ============================================================
