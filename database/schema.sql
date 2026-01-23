-- SpatialVortex Database Schema
-- PostgreSQL 12+ required (for JSONB support)

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- FluxMatrix storage table
CREATE TABLE IF NOT EXISTS flux_matrices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subject TEXT NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Index for fast subject lookups
CREATE INDEX IF NOT EXISTS idx_flux_matrices_subject 
ON flux_matrices(subject);

-- Index for JSON data queries
CREATE INDEX IF NOT EXISTS idx_flux_matrices_data_gin 
ON flux_matrices USING GIN (data);

-- Inference logging table
CREATE TABLE IF NOT EXISTS inference_log (
    id SERIAL PRIMARY KEY,
    matrix_id UUID REFERENCES flux_matrices(id) ON DELETE CASCADE,
    input_seeds JSONB NOT NULL,
    output_meanings JSONB NOT NULL,
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    processing_time_ms INTEGER NOT NULL CHECK (processing_time_ms >= 0),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Index for fast matrix_id lookups
CREATE INDEX IF NOT EXISTS idx_inference_log_matrix_id 
ON inference_log(matrix_id);

-- Index for timestamp-based queries
CREATE INDEX IF NOT EXISTS idx_inference_log_created_at 
ON inference_log(created_at DESC);

-- Function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
DROP TRIGGER IF EXISTS update_flux_matrices_updated_at ON flux_matrices;
CREATE TRIGGER update_flux_matrices_updated_at
    BEFORE UPDATE ON flux_matrices
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- View for matrix statistics
CREATE OR REPLACE VIEW matrix_statistics AS
SELECT 
    COUNT(*) as total_matrices,
    COUNT(DISTINCT subject) as unique_subjects,
    AVG(pg_column_size(data)) as avg_matrix_size_bytes,
    MIN(created_at) as oldest_matrix,
    MAX(updated_at) as newest_matrix
FROM flux_matrices;

-- View for inference statistics
CREATE OR REPLACE VIEW inference_statistics AS
SELECT 
    COUNT(*) as total_inferences,
    AVG(confidence) as avg_confidence,
    AVG(processing_time_ms) as avg_processing_time_ms,
    MAX(processing_time_ms) as max_processing_time_ms,
    MIN(processing_time_ms) as min_processing_time_ms
FROM inference_log
WHERE created_at > NOW() - INTERVAL '24 hours';
