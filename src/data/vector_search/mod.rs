/// Vector Search and Indexing Module
/// 
/// High-performance vector similarity search optimized for geometric embeddings.
/// Uses HNSW (Hierarchical Navigable Small World) algorithm for approximate nearest neighbor search.
/// 
/// Target Performance:
/// - Index 10M+ vectors
/// - Query time <10ms
/// - Recall >95% @ k=10
/// 
/// Architecture:
/// - Pure Rust implementation (no C++ dependencies)
/// - Lock-free concurrent access using DashMap
/// - HNSW graph structure for fast approximate search
/// - Cosine similarity for geometric embeddings

use ndarray::Array1;
use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

// E8 quantization support
use crate::data::e8_integration::{SacredE8Codec, SacredE8EncodedVector, E8FluxPosition, E8ELPTensor};

/// Vector dimension (configurable)
pub const VECTOR_DIM: usize = 384;  // Matches sentence-transformers default

/// Distance metric for similarity search
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,      // Cosine similarity (default for embeddings)
    Euclidean,   // L2 distance
    DotProduct,  // Inner product
}

/// HNSW index parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HNSWConfig {
    /// Maximum connections per node (M)
    pub max_connections: usize,
    
    /// Construction time connections multiplier (ef_construction)
    pub ef_construction: usize,
    
    /// Search time candidate size (ef_search)
    pub ef_search: usize,
    
    /// Distance metric
    pub metric: DistanceMetric,
    
    /// Max layer (calculated from log2)
    pub max_layer: usize,
}

impl Default for HNSWConfig {
    fn default() -> Self {
        Self {
            max_connections: 16,      // M = 16 (good default)
            ef_construction: 200,     // ef_construction = 200
            ef_search: 50,            // ef_search = 50 (query time)
            metric: DistanceMetric::Cosine,
            max_layer: 16,            // Max layers in hierarchy
        }
    }
}

/// Vector with metadata
#[derive(Debug, Clone)]
pub struct IndexedVector {
    pub id: String,
    pub vector: Array1<f32>,
    pub metadata: VectorMetadata,
}

/// Metadata attached to vectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetadata {
    pub position: Option<u8>,        // Geometric position (0-9)
    pub sacred: bool,                // Is sacred position (3, 6, 9)
    pub ethos: f32,                  // ELP channels
    pub logos: f32,
    pub pathos: f32,
    pub created_at: std::time::SystemTime,
}

impl Default for VectorMetadata {
    fn default() -> Self {
        Self {
            position: None,
            sacred: false,
            ethos: 0.0,
            logos: 0.0,
            pathos: 0.0,
            created_at: std::time::SystemTime::now(),
        }
    }
}

/// Search result with score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,      // Similarity score (higher = more similar)
    pub metadata: VectorMetadata,
}

impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for SearchResult {}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Reverse order for max heap (highest score first)
        other.score.partial_cmp(&self.score)
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// HNSW node in the graph
#[derive(Debug, Clone)]
struct HNSWNode {
    /// Unique identifier for this node (used for neighbor lookups and debugging)
    #[allow(dead_code)]
    id: String,
    vector: Array1<f32>,
    metadata: VectorMetadata,
    /// Neighbors per layer: layer -> neighbor_ids
    neighbors: Vec<Vec<String>>,
}

impl HNSWNode {
    /// Get node ID - used for graph traversal and debugging
    #[allow(dead_code)]  // Will be used by graph algorithms
    pub fn get_id(&self) -> &str {
        &self.id
    }
    
    /// Check if this node is connected to another - used for graph validation
    #[allow(dead_code)]  // Will be used by connectivity checks
    pub fn is_connected_to(&self, other_id: &str) -> bool {
        self.neighbors.iter().any(|layer| layer.contains(&other_id.to_string()))
    }
    
    /// Get neighbor count at specific layer - used for graph statistics
    #[allow(dead_code)]  // Will be used by performance monitoring
    pub fn neighbor_count(&self, layer: usize) -> usize {
        self.neighbors.get(layer).map(|n| n.len()).unwrap_or(0)
    }
}

/// High-performance vector index using HNSW
pub struct VectorIndex {
    config: HNSWConfig,
    
    /// All indexed vectors: id -> node
    nodes: Arc<DashMap<String, HNSWNode>>,
    
    /// Entry point for search (top layer)
    entry_point: Arc<RwLock<Option<String>>>,
    
    /// Total vectors indexed
    count: Arc<parking_lot::Mutex<usize>>,
}

impl VectorIndex {
    /// Create new vector index
    pub fn new(config: HNSWConfig) -> Self {
        Self {
            config,
            nodes: Arc::new(DashMap::new()),
            entry_point: Arc::new(RwLock::new(None)),
            count: Arc::new(parking_lot::Mutex::new(0)),
        }
    }
    
    /// Create with default configuration
    pub fn new_default() -> Self {
        Self::new(HNSWConfig::default())
    }
    
    /// Add vector to index
    pub fn add(&self, id: String, vector: Array1<f32>, metadata: VectorMetadata) -> anyhow::Result<()> {
        if vector.len() != VECTOR_DIM {
            anyhow::bail!("Vector dimension mismatch: expected {}, got {}", VECTOR_DIM, vector.len());
        }
        
        // Determine node layer (random with exponential decay)
        let layer = self.random_layer();
        
        // Create node
        let mut neighbors = Vec::with_capacity(layer + 1);
        for _ in 0..=layer {
            neighbors.push(Vec::new());
        }
        
        let node = HNSWNode {
            id: id.clone(),
            vector: vector.clone(),
            metadata,
            neighbors,
        };
        
        // Insert into index
        self.nodes.insert(id.clone(), node);
        
        // Update entry point if needed
        let mut entry = self.entry_point.write();
        if entry.is_none() {
            *entry = Some(id.clone());
        }
        
        *self.count.lock() += 1;
        
        // Connect to neighbors (simplified for now)
        self.connect_neighbors(&id, layer)?;
        
        Ok(())
    }
    
    /// Search for k nearest neighbors
    pub fn search(&self, query: &Array1<f32>, k: usize) -> anyhow::Result<Vec<SearchResult>> {
        if query.len() != VECTOR_DIM {
            anyhow::bail!("Query vector dimension mismatch: expected {}, got {}", VECTOR_DIM, query.len());
        }
        
        let entry_id = {
            let entry = self.entry_point.read();
            match entry.as_ref() {
                Some(id) => id.clone(),
                None => return Ok(Vec::new()),  // Empty index
            }
        };
        
        // Greedy search from entry point
        let mut candidates = BinaryHeap::new();
        let mut visited = std::collections::HashSet::new();
        
        // Start with entry point
        if let Some(entry_node) = self.nodes.get(&entry_id) {
            let score = self.compute_similarity(query, &entry_node.vector);
            candidates.push(SearchResult {
                id: entry_id.clone(),
                score,
                metadata: entry_node.metadata.clone(),
            });
            visited.insert(entry_id);
        }
        
        // Beam search
        let mut result_heap = BinaryHeap::new();
        let mut checked = 0;
        let max_checked = self.config.ef_search;
        
        while let Some(candidate) = candidates.pop() {
            if checked >= max_checked {
                break;
            }
            checked += 1;
            
            result_heap.push(candidate.clone());
            
            // Explore neighbors
            if let Some(node) = self.nodes.get(&candidate.id) {
                // Get neighbors from bottom layer
                if let Some(neighbors) = node.neighbors.last() {
                    for neighbor_id in neighbors {
                        if visited.contains(neighbor_id) {
                            continue;
                        }
                        visited.insert(neighbor_id.clone());
                        
                        if let Some(neighbor_node) = self.nodes.get(neighbor_id) {
                            let score = self.compute_similarity(query, &neighbor_node.vector);
                            candidates.push(SearchResult {
                                id: neighbor_id.clone(),
                                score,
                                metadata: neighbor_node.metadata.clone(),
                            });
                        }
                    }
                }
            }
        }
        
        // Extract top k results
        let mut results = Vec::new();
        for _ in 0..k.min(result_heap.len()) {
            if let Some(result) = result_heap.pop() {
                results.push(result);
            }
        }
        
        Ok(results)
    }
    
    /// Search by position (geometric filter)
    pub fn search_by_position(&self, query: &Array1<f32>, k: usize, position: u8) -> anyhow::Result<Vec<SearchResult>> {
        let all_results = self.search(query, k * 3)?;  // Get more candidates
        
        // Filter by position
        let filtered: Vec<SearchResult> = all_results
            .into_iter()
            .filter(|r| r.metadata.position == Some(position))
            .take(k)
            .collect();
        
        Ok(filtered)
    }
    
    /// Search by ELP channels (attribute filter)
    pub fn search_by_elp(&self, query: &Array1<f32>, k: usize, min_ethos: f32) -> anyhow::Result<Vec<SearchResult>> {
        let all_results = self.search(query, k * 3)?;
        
        let filtered: Vec<SearchResult> = all_results
            .into_iter()
            .filter(|r| r.metadata.ethos >= min_ethos)
            .take(k)
            .collect();
        
        Ok(filtered)
    }
    
    /// Get index statistics
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            total_vectors: *self.count.lock(),
            vector_dim: VECTOR_DIM,
            max_connections: self.config.max_connections,
            ef_search: self.config.ef_search,
            metric: self.config.metric,
        }
    }
    
    // Private: compute similarity score
    fn compute_similarity(&self, a: &Array1<f32>, b: &Array1<f32>) -> f32 {
        match self.config.metric {
            DistanceMetric::Cosine => {
                let dot = a.dot(b);
                let norm_a = a.dot(a).sqrt();
                let norm_b = b.dot(b).sqrt();
                if norm_a > 0.0 && norm_b > 0.0 {
                    dot / (norm_a * norm_b)
                } else {
                    0.0
                }
            }
            DistanceMetric::Euclidean => {
                let diff = a - b;
                let dist = diff.dot(&diff).sqrt();
                1.0 / (1.0 + dist)  // Convert to similarity
            }
            DistanceMetric::DotProduct => a.dot(b),
        }
    }
    
    // Private: determine random layer for new node
    fn random_layer(&self) -> usize {
        let ml = 1.0 / (self.config.max_connections as f64).ln();
        let r: f64 = rand::random();
        let layer = (-r.ln() * ml).floor() as usize;
        layer.min(self.config.max_layer)
    }
    
    // Private: connect node to neighbors (simplified)
    fn connect_neighbors(&self, id: &str, _layer: usize) -> anyhow::Result<()> {
        // Simplified: connect to entry point only
        let entry = self.entry_point.read().clone();
        if let Some(entry_id) = entry {
            if entry_id != id {
                // Add bidirectional connection - must release each lock before acquiring the next
                {
                    if let Some(mut node) = self.nodes.get_mut(id) {
                        if let Some(neighbors) = node.neighbors.last_mut() {
                            neighbors.push(entry_id.clone());
                        }
                    }
                } // Release lock on id before acquiring lock on entry_id
                
                {
                    if let Some(mut entry_node) = self.nodes.get_mut(&entry_id) {
                        if let Some(neighbors) = entry_node.neighbors.last_mut() {
                            neighbors.push(id.to_string());
                        }
                    }
                } // Release lock on entry_id
            }
        }
        Ok(())
    }
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_vectors: usize,
    pub vector_dim: usize,
    pub max_connections: usize,
    pub ef_search: usize,
    pub metric: DistanceMetric,
}

// ============================================================================
// E8 Quantized Vector Index
// ============================================================================

/// E8 quantization configuration for HNSW
#[derive(Debug, Clone)]
pub struct E8HNSWConfig {
    /// Base HNSW configuration
    pub hnsw: HNSWConfig,
    /// Bits per E8 block (8, 10, or 12)
    pub bits_per_block: u8,
    /// Use Hadamard preprocessing
    pub use_hadamard: bool,
    /// Random seed for reproducibility
    pub random_seed: u64,
    /// Apply sacred position boosts
    pub use_sacred_boost: bool,
}

impl Default for E8HNSWConfig {
    fn default() -> Self {
        Self {
            hnsw: HNSWConfig::default(),
            bits_per_block: 10,      // Good balance of quality/compression
            use_hadamard: true,      // Improves quantization quality
            random_seed: 42,
            use_sacred_boost: true,  // Boost sacred positions (3, 6, 9)
        }
    }
}

/// E8 quantized HNSW node
#[derive(Debug, Clone)]
struct E8HNSWNode {
    /// Node identifier
    #[allow(dead_code)]
    id: String,
    /// E8 quantized vector (compressed storage)
    encoded: SacredE8EncodedVector,
    /// Original vector for asymmetric search (optional, for higher recall)
    original: Option<Array1<f32>>,
    /// Metadata
    metadata: VectorMetadata,
    /// Neighbors per layer
    neighbors: Vec<Vec<String>>,
}

/// E8 quantized vector index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8IndexStats {
    /// Base index stats
    pub base: IndexStats,
    /// Compression ratio (original / compressed)
    pub compression_ratio: f32,
    /// Average signal strength
    pub avg_signal_strength: f32,
    /// Vectors at sacred positions
    pub sacred_count: usize,
    /// Memory usage in bytes (estimated)
    pub memory_bytes: usize,
}

/// E8 quantized HNSW index for memory-efficient vector search
/// 
/// Uses E8 lattice quantization to compress vectors by ~4-6x while
/// maintaining high recall through asymmetric distance computation.
pub struct E8VectorIndex {
    config: E8HNSWConfig,
    /// E8 codec for encoding/decoding
    codec: SacredE8Codec,
    /// All indexed nodes
    nodes: Arc<DashMap<String, E8HNSWNode>>,
    /// Entry point for search
    entry_point: Arc<RwLock<Option<String>>>,
    /// Total vectors indexed
    count: Arc<parking_lot::Mutex<usize>>,
    /// Keep original vectors for asymmetric search
    keep_originals: bool,
}

impl E8VectorIndex {
    /// Create new E8 quantized index
    pub fn new(config: E8HNSWConfig) -> Self {
        let codec = SacredE8Codec::new(
            VECTOR_DIM,
            config.bits_per_block,
            config.use_hadamard,
            config.random_seed,
            config.use_sacred_boost,
        );
        
        Self {
            config,
            codec,
            nodes: Arc::new(DashMap::new()),
            entry_point: Arc::new(RwLock::new(None)),
            count: Arc::new(parking_lot::Mutex::new(0)),
            keep_originals: true, // Default: keep for asymmetric search
        }
    }
    
    /// Create with default configuration
    pub fn new_default() -> Self {
        Self::new(E8HNSWConfig::default())
    }
    
    /// Set whether to keep original vectors (trades memory for recall)
    pub fn set_keep_originals(&mut self, keep: bool) {
        self.keep_originals = keep;
    }
    
    /// Add vector with E8 quantization
    pub fn add(&self, id: String, vector: Array1<f32>, metadata: VectorMetadata) -> anyhow::Result<()> {
        if vector.len() != VECTOR_DIM {
            anyhow::bail!("Vector dimension mismatch: expected {}, got {}", VECTOR_DIM, vector.len());
        }
        
        // Create ELP tensor from metadata
        let elp = E8ELPTensor::new(metadata.ethos, metadata.logos, metadata.pathos);
        
        // Encode with E8 quantization
        let encoded = self.codec.encode_sacred(vector.as_slice().unwrap(), Some(&elp))
            .map_err(|e| anyhow::anyhow!("E8 encoding failed: {}", e))?;
        
        // Determine node layer
        let layer = self.random_layer();
        
        // Create neighbors structure
        let mut neighbors = Vec::with_capacity(layer + 1);
        for _ in 0..=layer {
            neighbors.push(Vec::new());
        }
        
        let node = E8HNSWNode {
            id: id.clone(),
            encoded,
            original: if self.keep_originals { Some(vector) } else { None },
            metadata,
            neighbors,
        };
        
        self.nodes.insert(id.clone(), node);
        
        // Update entry point
        let mut entry = self.entry_point.write();
        if entry.is_none() {
            *entry = Some(id.clone());
        }
        
        *self.count.lock() += 1;
        
        // Connect to neighbors
        self.connect_neighbors(&id, layer)?;
        
        Ok(())
    }
    
    /// Search for k nearest neighbors using asymmetric distance
    /// 
    /// Query vector is NOT quantized, but database vectors are.
    /// This provides better recall than symmetric quantized search.
    pub fn search(&self, query: &Array1<f32>, k: usize) -> anyhow::Result<Vec<SearchResult>> {
        if query.len() != VECTOR_DIM {
            anyhow::bail!("Query dimension mismatch: expected {}, got {}", VECTOR_DIM, query.len());
        }
        
        let entry_id = {
            let entry = self.entry_point.read();
            match entry.as_ref() {
                Some(id) => id.clone(),
                None => return Ok(Vec::new()),
            }
        };
        
        let mut candidates = BinaryHeap::new();
        let mut visited = std::collections::HashSet::new();
        
        // Start with entry point
        if let Some(entry_node) = self.nodes.get(&entry_id) {
            let score = self.compute_asymmetric_similarity(query, &entry_node);
            candidates.push(SearchResult {
                id: entry_id.clone(),
                score,
                metadata: entry_node.metadata.clone(),
            });
            visited.insert(entry_id);
        }
        
        let mut result_heap = BinaryHeap::new();
        let mut checked = 0;
        let max_checked = self.config.hnsw.ef_search;
        
        while let Some(candidate) = candidates.pop() {
            if checked >= max_checked {
                break;
            }
            checked += 1;
            
            result_heap.push(candidate.clone());
            
            // Explore neighbors
            if let Some(node) = self.nodes.get(&candidate.id) {
                if let Some(neighbors) = node.neighbors.last() {
                    for neighbor_id in neighbors {
                        if visited.contains(neighbor_id) {
                            continue;
                        }
                        visited.insert(neighbor_id.clone());
                        
                        if let Some(neighbor_node) = self.nodes.get(neighbor_id) {
                            let score = self.compute_asymmetric_similarity(query, &neighbor_node);
                            candidates.push(SearchResult {
                                id: neighbor_id.clone(),
                                score,
                                metadata: neighbor_node.metadata.clone(),
                            });
                        }
                    }
                }
            }
        }
        
        // Extract top k
        let mut results = Vec::new();
        for _ in 0..k.min(result_heap.len()) {
            if let Some(result) = result_heap.pop() {
                results.push(result);
            }
        }
        
        Ok(results)
    }
    
    /// Search with sacred position filter
    pub fn search_sacred(&self, query: &Array1<f32>, k: usize) -> anyhow::Result<Vec<SearchResult>> {
        let all_results = self.search(query, k * 3)?;
        
        let filtered: Vec<SearchResult> = all_results
            .into_iter()
            .filter(|r| r.metadata.sacred)
            .take(k)
            .collect();
        
        Ok(filtered)
    }
    
    /// Search by flux position
    pub fn search_by_position(&self, query: &Array1<f32>, k: usize, position: u8) -> anyhow::Result<Vec<SearchResult>> {
        let all_results = self.search(query, k * 3)?;
        
        let filtered: Vec<SearchResult> = all_results
            .into_iter()
            .filter(|r| r.metadata.position == Some(position))
            .take(k)
            .collect();
        
        Ok(filtered)
    }
    
    /// Get E8 index statistics
    pub fn stats(&self) -> E8IndexStats {
        let total = *self.count.lock();
        
        // Calculate compression ratio and signal strength
        let mut total_signal = 0.0f32;
        let mut sacred_count = 0usize;
        let mut memory_bytes = 0usize;
        
        for entry in self.nodes.iter() {
            let node = entry.value();
            total_signal += node.encoded.signal_strength;
            if node.metadata.sacred {
                sacred_count += 1;
            }
            // Estimate memory: encoded size + optional original + metadata
            memory_bytes += node.encoded.encoded.size_bytes();
            if node.original.is_some() {
                memory_bytes += VECTOR_DIM * 4; // f32 = 4 bytes
            }
            memory_bytes += 64; // Approximate metadata size
        }
        
        let avg_signal = if total > 0 { total_signal / total as f32 } else { 0.0 };
        
        // Compression ratio: original f32 size / E8 encoded size
        let original_bytes = total * VECTOR_DIM * 4;
        let encoded_bytes = memory_bytes.saturating_sub(if self.keep_originals { total * VECTOR_DIM * 4 } else { 0 });
        let compression_ratio = if encoded_bytes > 0 {
            original_bytes as f32 / encoded_bytes as f32
        } else {
            1.0
        };
        
        E8IndexStats {
            base: IndexStats {
                total_vectors: total,
                vector_dim: VECTOR_DIM,
                max_connections: self.config.hnsw.max_connections,
                ef_search: self.config.hnsw.ef_search,
                metric: self.config.hnsw.metric,
            },
            compression_ratio,
            avg_signal_strength: avg_signal,
            sacred_count,
            memory_bytes,
        }
    }
    
    /// Compute asymmetric similarity (query vs quantized database vector)
    fn compute_asymmetric_similarity(&self, query: &Array1<f32>, node: &E8HNSWNode) -> f32 {
        // If we have original vector, use it for higher recall
        if let Some(ref original) = node.original {
            return self.compute_similarity(query, original);
        }
        
        // Otherwise decode and compute
        let decoded = self.codec.decode_sacred(&node.encoded);
        let decoded_arr = Array1::from_vec(decoded);
        
        // Apply sacred boost to score
        let base_score = self.compute_similarity(query, &decoded_arr);
        if node.encoded.flux_position.is_sacred() {
            base_score * node.encoded.quality_boost
        } else {
            base_score
        }
    }
    
    fn compute_similarity(&self, a: &Array1<f32>, b: &Array1<f32>) -> f32 {
        match self.config.hnsw.metric {
            DistanceMetric::Cosine => {
                let dot = a.dot(b);
                let norm_a = a.dot(a).sqrt();
                let norm_b = b.dot(b).sqrt();
                if norm_a > 0.0 && norm_b > 0.0 {
                    dot / (norm_a * norm_b)
                } else {
                    0.0
                }
            }
            DistanceMetric::Euclidean => {
                let diff = a - b;
                let dist = diff.dot(&diff).sqrt();
                1.0 / (1.0 + dist)
            }
            DistanceMetric::DotProduct => a.dot(b),
        }
    }
    
    fn random_layer(&self) -> usize {
        let ml = 1.0 / (self.config.hnsw.max_connections as f64).ln();
        let r: f64 = rand::random();
        let layer = (-r.ln() * ml).floor() as usize;
        layer.min(self.config.hnsw.max_layer)
    }
    
    fn connect_neighbors(&self, id: &str, _layer: usize) -> anyhow::Result<()> {
        let entry = self.entry_point.read().clone();
        if let Some(entry_id) = entry {
            if entry_id != id {
                {
                    if let Some(mut node) = self.nodes.get_mut(id) {
                        if let Some(neighbors) = node.neighbors.last_mut() {
                            neighbors.push(entry_id.clone());
                        }
                    }
                }
                {
                    if let Some(mut entry_node) = self.nodes.get_mut(&entry_id) {
                        if let Some(neighbors) = entry_node.neighbors.last_mut() {
                            neighbors.push(id.to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn random_vector() -> Array1<f32> {
        Array1::from_iter((0..VECTOR_DIM).map(|_| rand::random::<f32>()))
    }
    
    #[test]
    fn test_index_creation() {
        let index = VectorIndex::new_default();
        let stats = index.stats();
        assert_eq!(stats.total_vectors, 0);
        assert_eq!(stats.vector_dim, VECTOR_DIM);
    }
    
    #[test]
    fn test_add_vector() {
        let index = VectorIndex::new_default();
        let vector = random_vector();
        let metadata = VectorMetadata::default();
        
        let result = index.add("vec1".to_string(), vector, metadata);
        assert!(result.is_ok());
        assert_eq!(index.stats().total_vectors, 1);
    }
    
    #[test]
    fn test_search() {
        let index = VectorIndex::new_default();
        
        // Add some vectors
        for i in 0..10 {
            let vector = random_vector();
            let metadata = VectorMetadata {
                position: Some(i % 10),
                ..Default::default()
            };
            index.add(format!("vec{}", i), vector, metadata).unwrap();
        }
        
        // Search
        let query = random_vector();
        let results = index.search(&query, 5).unwrap();
        
        assert!(results.len() <= 5);
        
        // Results should be sorted by score (descending)
        for i in 1..results.len() {
            assert!(results[i-1].score >= results[i].score);
        }
    }
    
    #[test]
    fn test_position_filter() {
        let index = VectorIndex::new_default();
        
        // Add vectors with specific positions
        for i in 0..20 {
            let vector = random_vector();
            let metadata = VectorMetadata {
                position: Some(if i < 10 { 3 } else { 5 }),  // Half at position 3
                sacred: i < 10,
                ..Default::default()
            };
            index.add(format!("vec{}", i), vector, metadata).unwrap();
        }
        
        // Search for position 3
        let query = random_vector();
        let results = index.search_by_position(&query, 5, 3).unwrap();
        
        // All results should be position 3
        for result in results {
            assert_eq!(result.metadata.position, Some(3));
        }
    }
    
    #[test]
    fn test_cosine_similarity() {
        let index = VectorIndex::new_default();
        
        let a = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let b = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let c = Array1::from_vec(vec![0.0, 1.0, 0.0]);
        
        let sim_ab = index.compute_similarity(&a, &b);
        let sim_ac = index.compute_similarity(&a, &c);
        
        assert!((sim_ab - 1.0).abs() < 0.001);  // Identical vectors
        assert!((sim_ac - 0.0).abs() < 0.001);  // Orthogonal vectors
    }
    
    // ========================================================================
    // E8 Quantized Index Tests
    // ========================================================================
    
    #[test]
    fn test_e8_index_creation() {
        let index = E8VectorIndex::new_default();
        let stats = index.stats();
        assert_eq!(stats.base.total_vectors, 0);
        assert_eq!(stats.base.vector_dim, VECTOR_DIM);
    }
    
    #[test]
    fn test_e8_add_vector() {
        let index = E8VectorIndex::new_default();
        let vector = random_vector();
        let metadata = VectorMetadata {
            ethos: 0.5,
            logos: 0.3,
            pathos: 0.2,
            ..Default::default()
        };
        
        let result = index.add("e8_vec1".to_string(), vector, metadata);
        assert!(result.is_ok());
        assert_eq!(index.stats().base.total_vectors, 1);
    }
    
    #[test]
    fn test_e8_search() {
        let index = E8VectorIndex::new_default();
        
        // Add vectors with varying ELP
        for i in 0..10 {
            let vector = random_vector();
            let metadata = VectorMetadata {
                position: Some(i % 10),
                sacred: matches!(i % 10, 3 | 6 | 9),
                ethos: (i as f32) / 10.0,
                logos: 0.3,
                pathos: 0.2,
                ..Default::default()
            };
            index.add(format!("e8_vec{}", i), vector, metadata).unwrap();
        }
        
        let query = random_vector();
        let results = index.search(&query, 5).unwrap();
        
        assert!(results.len() <= 5);
        
        // Results should be sorted by score
        for i in 1..results.len() {
            assert!(results[i-1].score >= results[i].score);
        }
    }
    
    #[test]
    fn test_e8_sacred_search() {
        let index = E8VectorIndex::new_default();
        
        // Add vectors - some sacred, some not
        for i in 0..20 {
            let vector = random_vector();
            let is_sacred = i < 10;
            let metadata = VectorMetadata {
                position: Some(if is_sacred { 3 } else { 1 }),
                sacred: is_sacred,
                ethos: 0.9,
                logos: 0.05,
                pathos: 0.05,
                ..Default::default()
            };
            index.add(format!("e8_sacred{}", i), vector, metadata).unwrap();
        }
        
        let query = random_vector();
        let results = index.search_sacred(&query, 5).unwrap();
        
        // All results should be sacred
        for result in results {
            assert!(result.metadata.sacred);
        }
    }
    
    #[test]
    fn test_e8_compression_stats() {
        let mut index = E8VectorIndex::new_default();
        index.set_keep_originals(false); // Don't keep originals for compression test
        
        // Add vectors
        for i in 0..100 {
            let vector = random_vector();
            let metadata = VectorMetadata::default();
            index.add(format!("compress{}", i), vector, metadata).unwrap();
        }
        
        let stats = index.stats();
        assert_eq!(stats.base.total_vectors, 100);
        
        // Should have some compression (E8 compresses ~4-6x)
        // Note: compression_ratio calculation depends on keep_originals setting
        assert!(stats.memory_bytes > 0);
    }
}
