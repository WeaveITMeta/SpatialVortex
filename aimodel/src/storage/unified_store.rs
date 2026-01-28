//! Unified Storage Architecture for SpatialVortex AI Model
//!
//! Single RocksDB instance with column families for all model data:
//! - weights: Model weights (E8 quantized)
//! - embeddings: Vector embeddings with sacred geometry
//! - flux: Flux states (BeamTensor arrays)
//! - latent: CALM compressed latent states
//! - patterns: Verified patterns from RSI learning
//! - benchmarks: SOTA tracking data
//! - metadata: Config, checksums, versions

use crate::data::models::BeamTensor;
use crate::ml::calm::LatentState;
use crate::cognition::verified_patterning::{VerifiedPattern, BenchmarkResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// =============================================================================
// Configuration
// =============================================================================

/// Model tier configuration for progressive scaling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelTier {
    /// Development/testing: 7B params, ~14GB INT4
    Tier0 { name: String },
    /// Production baseline: 70B params, ~35GB INT4
    Tier1 { name: String },
    /// High performance: 405B params, ~200GB INT4
    Tier2 { name: String },
    /// SOTA target: 1T+ params, ~500GB INT4
    Tier3 { name: String },
}

impl Default for ModelTier {
    fn default() -> Self {
        ModelTier::Tier0 { name: "spatialvortex-7b".to_string() }
    }
}

impl ModelTier {
    pub fn estimated_size_gb(&self) -> f64 {
        match self {
            ModelTier::Tier0 { .. } => 14.0,
            ModelTier::Tier1 { .. } => 35.0,
            ModelTier::Tier2 { .. } => 200.0,
            ModelTier::Tier3 { .. } => 500.0,
        }
    }

    pub fn param_count(&self) -> u64 {
        match self {
            ModelTier::Tier0 { .. } => 7_000_000_000,
            ModelTier::Tier1 { .. } => 70_000_000_000,
            ModelTier::Tier2 { .. } => 405_000_000_000,
            ModelTier::Tier3 { .. } => 1_350_000_000_000,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ModelTier::Tier0 { name } => name,
            ModelTier::Tier1 { name } => name,
            ModelTier::Tier2 { name } => name,
            ModelTier::Tier3 { name } => name,
        }
    }
}

/// Quantization level for compression pipeline
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum QuantizationLevel {
    /// Full precision (training)
    FP32,
    /// Half precision (mixed training)
    BF16,
    /// 8-bit quantization (inference)
    INT8,
    /// 4-bit quantization (edge/mobile)
    INT4,
    /// E8 lattice quantization (~1.25 bits/dim)
    E8 { bits_per_block: u8 },
    /// Full compression: INT4 + E8 + CALM
    FullCompression,
}

impl Default for QuantizationLevel {
    fn default() -> Self {
        QuantizationLevel::INT4
    }
}

impl QuantizationLevel {
    /// Compression ratio vs FP32
    pub fn compression_ratio(&self) -> f64 {
        match self {
            QuantizationLevel::FP32 => 1.0,
            QuantizationLevel::BF16 => 2.0,
            QuantizationLevel::INT8 => 4.0,
            QuantizationLevel::INT4 => 8.0,
            QuantizationLevel::E8 { bits_per_block } => {
                32.0 / (*bits_per_block as f64 / 8.0)
            }
            QuantizationLevel::FullCompression => 300.0,
        }
    }
}

/// Configuration for unified store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedStoreConfig {
    /// Database path
    pub path: PathBuf,
    /// Model tier
    pub tier: ModelTier,
    /// Quantization level
    pub quantization: QuantizationLevel,
    /// Enable LZ4 compression in RocksDB
    pub compression: bool,
    /// Block cache size in MB
    pub cache_size_mb: usize,
    /// Write buffer size in MB
    pub write_buffer_mb: usize,
    /// Enable bloom filters
    pub bloom_filter_bits: u8,
    /// Max open files
    pub max_open_files: i32,
}

impl Default for UnifiedStoreConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./spatialvortex_store"),
            tier: ModelTier::default(),
            quantization: QuantizationLevel::INT4,
            compression: true,
            cache_size_mb: 256,
            write_buffer_mb: 64,
            bloom_filter_bits: 10,
            max_open_files: 1000,
        }
    }
}

// =============================================================================
// Column Family Data Types
// =============================================================================

/// Stored model weight tensor (E8 quantized)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredWeight {
    pub layer_id: String,
    pub tensor_name: String,
    pub shape: Vec<usize>,
    pub quantization: QuantizationLevel,
    /// Quantized data bytes
    pub data: Vec<u8>,
    /// Scale factor for dequantization
    pub scale: f32,
    /// Zero point for asymmetric quantization
    pub zero_point: i32,
    /// Flux position for sacred geometry
    pub flux_position: u8,
    /// Checksum for integrity
    pub checksum: u64,
}

/// Stored embedding with E8 + sacred geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEmbedding {
    pub id: String,
    /// E8 quantized data
    pub data: Vec<u8>,
    /// Original dimension
    pub dimension: usize,
    /// Scale factor
    pub scale: f32,
    /// Flux position (1-9)
    pub flux_position: u8,
    /// Signal strength (0.0-1.0)
    pub signal_strength: f32,
    /// Quality boost for sacred positions
    pub quality_boost: f32,
    /// ELP tensor (ethos, logos, pathos)
    pub elp: [f32; 3],
    /// Timestamp
    pub created_at: i64,
}

/// Stored latent state (CALM compressed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredLatent {
    pub id: String,
    /// Compressed latent vector
    pub latent: Vec<f32>,
    /// Energy level
    pub energy: f32,
    /// Compression ratio achieved
    pub compression_ratio: f32,
    /// Source beam count
    pub source_beam_count: usize,
    /// Timestamp
    pub created_at: i64,
}

/// Stored verified pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPattern {
    pub id: String,
    pub pattern: VerifiedPattern,
    pub reinforcement_count: u64,
    pub last_used: i64,
}

// =============================================================================
// Unified Store Implementation
// =============================================================================

/// Column family names
pub mod cf {
    pub const WEIGHTS: &str = "weights";
    pub const EMBEDDINGS: &str = "embeddings";
    pub const FLUX: &str = "flux";
    pub const LATENT: &str = "latent";
    pub const PATTERNS: &str = "patterns";
    pub const BENCHMARKS: &str = "benchmarks";
    pub const METADATA: &str = "metadata";
}

/// Unified store for all SpatialVortex model data
/// 
/// In-memory implementation for development.
/// Production would use actual RocksDB with column families.
pub struct UnifiedStore {
    config: UnifiedStoreConfig,
    
    // Column family data (in-memory for now)
    weights: HashMap<String, StoredWeight>,
    embeddings: HashMap<String, StoredEmbedding>,
    flux_states: HashMap<String, Vec<BeamTensor>>,
    latent_states: HashMap<String, StoredLatent>,
    patterns: HashMap<String, StoredPattern>,
    benchmarks: HashMap<String, Vec<BenchmarkResult>>,
    metadata: HashMap<String, String>,
    
    // Indexes
    flux_position_index: HashMap<u8, Vec<String>>,
    sacred_index: Vec<String>,
    
    // Stats
    total_bytes: u64,
    write_count: u64,
    read_count: u64,
}

impl UnifiedStore {
    /// Create a new unified store
    pub fn new(config: UnifiedStoreConfig) -> Self {
        Self {
            config,
            weights: HashMap::new(),
            embeddings: HashMap::new(),
            flux_states: HashMap::new(),
            latent_states: HashMap::new(),
            patterns: HashMap::new(),
            benchmarks: HashMap::new(),
            metadata: HashMap::new(),
            flux_position_index: HashMap::new(),
            sacred_index: Vec::new(),
            total_bytes: 0,
            write_count: 0,
            read_count: 0,
        }
    }

    /// Open store from path
    pub fn open(config: UnifiedStoreConfig) -> Result<Self, String> {
        // In production: open RocksDB with column families
        // let cf_opts = Options::default();
        // let cfs = vec![cf::WEIGHTS, cf::EMBEDDINGS, ...];
        // let db = DB::open_cf(&opts, &config.path, cfs)?;
        
        let mut store = Self::new(config);
        store.init_metadata()?;
        Ok(store)
    }

    /// Initialize metadata
    fn init_metadata(&mut self) -> Result<(), String> {
        self.metadata.insert("version".to_string(), "1.0.0".to_string());
        self.metadata.insert("tier".to_string(), self.config.tier.name().to_string());
        self.metadata.insert("created_at".to_string(), 
            chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    // =========================================================================
    // Weight Operations
    // =========================================================================

    /// Store a model weight tensor
    pub fn put_weight(&mut self, weight: StoredWeight) -> Result<(), String> {
        let size = weight.data.len() as u64;
        self.weights.insert(weight.layer_id.clone(), weight);
        self.total_bytes += size;
        self.write_count += 1;
        Ok(())
    }

    /// Get a model weight tensor
    pub fn get_weight(&mut self, layer_id: &str) -> Option<&StoredWeight> {
        self.read_count += 1;
        self.weights.get(layer_id)
    }

    /// Get all weight layer IDs
    pub fn weight_layers(&self) -> Vec<&str> {
        self.weights.keys().map(|s| s.as_str()).collect()
    }

    // =========================================================================
    // Embedding Operations
    // =========================================================================

    /// Store an embedding
    pub fn put_embedding(&mut self, embedding: StoredEmbedding) -> Result<(), String> {
        let id = embedding.id.clone();
        let pos = embedding.flux_position;
        let size = embedding.data.len() as u64;

        // Update position index
        self.flux_position_index.entry(pos).or_default().push(id.clone());

        // Update sacred index
        if matches!(pos, 3 | 6 | 9) {
            self.sacred_index.push(id.clone());
        }

        self.embeddings.insert(id, embedding);
        self.total_bytes += size;
        self.write_count += 1;
        Ok(())
    }

    /// Get an embedding
    pub fn get_embedding(&mut self, id: &str) -> Option<&StoredEmbedding> {
        self.read_count += 1;
        self.embeddings.get(id)
    }

    /// Get embeddings by flux position
    pub fn get_embeddings_by_position(&self, position: u8) -> Vec<&StoredEmbedding> {
        self.flux_position_index.get(&position)
            .map(|ids| ids.iter().filter_map(|id| self.embeddings.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get sacred position embeddings (3, 6, 9)
    pub fn get_sacred_embeddings(&self) -> Vec<&StoredEmbedding> {
        self.sacred_index.iter()
            .filter_map(|id| self.embeddings.get(id))
            .collect()
    }

    // =========================================================================
    // Flux State Operations
    // =========================================================================

    /// Store flux state (beam tensors)
    pub fn put_flux(&mut self, id: &str, beams: Vec<BeamTensor>) -> Result<(), String> {
        let size = beams.len() * std::mem::size_of::<BeamTensor>();
        self.flux_states.insert(id.to_string(), beams);
        self.total_bytes += size as u64;
        self.write_count += 1;
        Ok(())
    }

    /// Get flux state
    pub fn get_flux(&mut self, id: &str) -> Option<&Vec<BeamTensor>> {
        self.read_count += 1;
        self.flux_states.get(id)
    }

    // =========================================================================
    // Latent State Operations (CALM)
    // =========================================================================

    /// Store CALM latent state
    pub fn put_latent(&mut self, latent: StoredLatent) -> Result<(), String> {
        let size = latent.latent.len() * 4;
        self.latent_states.insert(latent.id.clone(), latent);
        self.total_bytes += size as u64;
        self.write_count += 1;
        Ok(())
    }

    /// Get latent state
    pub fn get_latent(&mut self, id: &str) -> Option<&StoredLatent> {
        self.read_count += 1;
        self.latent_states.get(id)
    }

    /// Convert LatentState to StoredLatent
    pub fn store_calm_latent(&mut self, id: &str, latent: &LatentState, compression_ratio: f32, source_count: usize) -> Result<(), String> {
        let stored = StoredLatent {
            id: id.to_string(),
            latent: latent.latent.clone(),
            energy: latent.energy,
            compression_ratio,
            source_beam_count: source_count,
            created_at: chrono::Utc::now().timestamp(),
        };
        self.put_latent(stored)
    }

    // =========================================================================
    // Pattern Operations (Verified Patterning)
    // =========================================================================

    /// Store verified pattern
    pub fn put_pattern(&mut self, id: &str, pattern: VerifiedPattern) -> Result<(), String> {
        let stored = StoredPattern {
            id: id.to_string(),
            pattern,
            reinforcement_count: 0,
            last_used: chrono::Utc::now().timestamp(),
        };
        self.patterns.insert(id.to_string(), stored);
        self.write_count += 1;
        Ok(())
    }

    /// Get pattern
    pub fn get_pattern(&mut self, id: &str) -> Option<&StoredPattern> {
        self.read_count += 1;
        self.patterns.get(id)
    }

    /// Get all patterns
    pub fn all_patterns(&self) -> Vec<&StoredPattern> {
        self.patterns.values().collect()
    }

    /// Reinforce a pattern
    pub fn reinforce_pattern(&mut self, id: &str) -> Result<(), String> {
        if let Some(pattern) = self.patterns.get_mut(id) {
            pattern.reinforcement_count += 1;
            pattern.last_used = chrono::Utc::now().timestamp();
            Ok(())
        } else {
            Err(format!("Pattern not found: {}", id))
        }
    }

    // =========================================================================
    // Benchmark Operations (SOTA Tracking)
    // =========================================================================

    /// Record benchmark result
    pub fn record_benchmark(&mut self, result: BenchmarkResult) -> Result<(), String> {
        self.benchmarks
            .entry(result.name.clone())
            .or_default()
            .push(result);
        self.write_count += 1;
        Ok(())
    }

    /// Get benchmark history
    pub fn get_benchmark_history(&self, name: &str) -> Option<&Vec<BenchmarkResult>> {
        self.benchmarks.get(name)
    }

    /// Get latest benchmark result
    pub fn get_latest_benchmark(&self, name: &str) -> Option<&BenchmarkResult> {
        self.benchmarks.get(name)?.last()
    }

    /// Get all benchmark names
    pub fn benchmark_names(&self) -> Vec<&str> {
        self.benchmarks.keys().map(|s| s.as_str()).collect()
    }

    // =========================================================================
    // Metadata Operations
    // =========================================================================

    /// Set metadata
    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Get store statistics
    pub fn stats(&self) -> StoreStats {
        StoreStats {
            tier: self.config.tier.clone(),
            quantization: self.config.quantization,
            total_bytes: self.total_bytes,
            weight_count: self.weights.len(),
            embedding_count: self.embeddings.len(),
            flux_count: self.flux_states.len(),
            latent_count: self.latent_states.len(),
            pattern_count: self.patterns.len(),
            benchmark_count: self.benchmarks.len(),
            write_count: self.write_count,
            read_count: self.read_count,
        }
    }

    /// Calculate actual size from stored data (NOT hardcoded)
    pub fn estimated_full_size_gb(&self) -> f64 {
        // Calculate actual bytes stored
        let mut actual_bytes: u64 = self.total_bytes;
        
        // Add up actual data sizes
        for w in self.weights.values() {
            actual_bytes += w.data.len() as u64;
        }
        for e in self.embeddings.values() {
            actual_bytes += e.data.len() as u64;
        }
        for beams in self.flux_states.values() {
            // flux_states is HashMap<String, Vec<BeamTensor>>
            actual_bytes += beams.len() as u64 * 9 * 4; // 9 digits * 4 bytes per f32 per beam
        }
        for l in self.latent_states.values() {
            actual_bytes += l.latent.len() as u64 * 4; // 4 bytes per f32
        }
        
        // Convert to GB
        actual_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
    
    /// Get tier-based theoretical max size (for reference only)
    pub fn theoretical_max_size_gb(&self) -> f64 {
        let base_size = self.config.tier.estimated_size_gb();
        let compression = self.config.quantization.compression_ratio();
        base_size / compression * 8.0
    }

    // =========================================================================
    // Persistence
    // =========================================================================

    /// Flush to disk (no-op for in-memory)
    pub fn flush(&self) -> Result<(), String> {
        // In production: self.db.flush()?;
        Ok(())
    }

    /// Compact database
    pub fn compact(&self) -> Result<(), String> {
        // In production: self.db.compact_range(None, None)?;
        Ok(())
    }

    /// Get database path
    pub fn path(&self) -> &PathBuf {
        &self.config.path
    }
}

/// Store statistics
#[derive(Debug, Clone)]
pub struct StoreStats {
    pub tier: ModelTier,
    pub quantization: QuantizationLevel,
    pub total_bytes: u64,
    pub weight_count: usize,
    pub embedding_count: usize,
    pub flux_count: usize,
    pub latent_count: usize,
    pub pattern_count: usize,
    pub benchmark_count: usize,
    pub write_count: u64,
    pub read_count: u64,
}

impl StoreStats {
    pub fn total_bytes_human(&self) -> String {
        let bytes = self.total_bytes as f64;
        if bytes >= 1e12 {
            format!("{:.2} TB", bytes / 1e12)
        } else if bytes >= 1e9 {
            format!("{:.2} GB", bytes / 1e9)
        } else if bytes >= 1e6 {
            format!("{:.2} MB", bytes / 1e6)
        } else if bytes >= 1e3 {
            format!("{:.2} KB", bytes / 1e3)
        } else {
            format!("{} bytes", self.total_bytes)
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_store_basic() {
        let config = UnifiedStoreConfig::default();
        let mut store = UnifiedStore::open(config).unwrap();

        // Test metadata
        assert_eq!(store.get_metadata("version"), Some("1.0.0"));

        // Test flux storage
        let beams = vec![BeamTensor::default()];
        store.put_flux("test_flux", beams).unwrap();
        assert!(store.get_flux("test_flux").is_some());

        let stats = store.stats();
        assert_eq!(stats.flux_count, 1);
    }

    #[test]
    fn test_embedding_with_sacred_positions() {
        let mut store = UnifiedStore::new(UnifiedStoreConfig::default());

        for pos in 1..=9 {
            let embedding = StoredEmbedding {
                id: format!("emb_{}", pos),
                data: vec![0u8; 100],
                dimension: 256,
                scale: 1.0,
                flux_position: pos,
                signal_strength: 0.8,
                quality_boost: if matches!(pos, 3 | 6 | 9) { 1.15 } else { 1.0 },
                elp: [0.33, 0.33, 0.34],
                created_at: 0,
            };
            store.put_embedding(embedding).unwrap();
        }

        let sacred = store.get_sacred_embeddings();
        assert_eq!(sacred.len(), 3);

        let pos_3 = store.get_embeddings_by_position(3);
        assert_eq!(pos_3.len(), 1);
    }

    #[test]
    fn test_model_tiers() {
        let tier0 = ModelTier::Tier0 { name: "test-7b".to_string() };
        assert_eq!(tier0.estimated_size_gb(), 14.0);
        assert_eq!(tier0.param_count(), 7_000_000_000);

        let tier3 = ModelTier::Tier3 { name: "test-1t".to_string() };
        assert_eq!(tier3.estimated_size_gb(), 500.0);
    }

    #[test]
    fn test_quantization_compression() {
        assert_eq!(QuantizationLevel::FP32.compression_ratio(), 1.0);
        assert_eq!(QuantizationLevel::INT4.compression_ratio(), 8.0);
        assert_eq!(QuantizationLevel::FullCompression.compression_ratio(), 300.0);
    }

    #[test]
    fn test_benchmark_tracking() {
        let mut store = UnifiedStore::new(UnifiedStoreConfig::default());

        let result = BenchmarkResult {
            name: "MMLU".to_string(),
            version: "v1".to_string(),
            score: 75.0,
            max_score: 100.0,
            sota_score: 90.0,
            timestamp_ms: 1000,
            config_hash: "test".to_string(),
        };
        store.record_benchmark(result).unwrap();

        let latest = store.get_latest_benchmark("MMLU").unwrap();
        assert_eq!(latest.score, 75.0);
    }
}
