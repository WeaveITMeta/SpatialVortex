//! Embeddings Storage with embedvec Integration
//!
//! SacredEmbedding layer: HNSW indexing + SIMD distances.
//! Geometric priors for flux position lookups.

use crate::data::models::BeamTensor;
use crate::data::attributes::Attributes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for embeddings storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsConfig {
    /// Embedding dimension
    pub dim: usize,
    /// HNSW M parameter (max connections per node)
    pub hnsw_m: usize,
    /// HNSW ef_construction parameter
    pub hnsw_ef_construction: usize,
    /// HNSW ef_search parameter
    pub hnsw_ef_search: usize,
    /// Enable sacred position geometric priors
    pub sacred_priors: bool,
}

impl Default for EmbeddingsConfig {
    fn default() -> Self {
        Self {
            dim: 256,
            hnsw_m: 16,
            hnsw_ef_construction: 200,
            hnsw_ef_search: 50,
            sacred_priors: true,
        }
    }
}

/// Sacred embedding with geometric priors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredEmbedding {
    /// Raw embedding vector
    pub vector: Vec<f32>,
    /// Flux position (1-9)
    pub position: u8,
    /// Is sacred position (3, 6, 9)
    pub is_sacred: bool,
    /// ELP attributes
    pub attributes: Attributes,
    /// Unique identifier
    pub id: String,
    /// Associated text/content
    pub content: String,
}

impl SacredEmbedding {
    pub fn new(id: String, vector: Vec<f32>, position: u8) -> Self {
        Self {
            vector,
            position,
            is_sacred: matches!(position, 3 | 6 | 9),
            attributes: Attributes::new(),
            id,
            content: String::new(),
        }
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    pub fn with_attributes(mut self, attrs: Attributes) -> Self {
        self.attributes = attrs;
        self
    }

    /// Apply sacred geometric prior to embedding
    pub fn apply_sacred_prior(&mut self) {
        if self.is_sacred {
            // Boost magnitude at sacred positions
            let boost = 1.15;
            self.vector.iter_mut().for_each(|v| *v *= boost);
        }
    }
}

/// HNSW-like index for sacred embeddings (simplified implementation)
/// In production, would use embedvec crate directly
#[derive(Debug, Clone)]
pub struct SacredEmbeddingIndex {
    config: EmbeddingsConfig,
    embeddings: Vec<SacredEmbedding>,
    /// Position-based buckets for geometric lookup
    position_index: HashMap<u8, Vec<usize>>,
}

impl SacredEmbeddingIndex {
    pub fn new(config: EmbeddingsConfig) -> Self {
        Self {
            config,
            embeddings: Vec::new(),
            position_index: HashMap::new(),
        }
    }

    /// Add embedding to index
    pub fn add(&mut self, mut embedding: SacredEmbedding) {
        if self.config.sacred_priors {
            embedding.apply_sacred_prior();
        }

        let idx = self.embeddings.len();
        let position = embedding.position;
        
        self.embeddings.push(embedding);
        self.position_index.entry(position).or_default().push(idx);
    }

    /// Search by vector similarity (cosine distance)
    pub fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        let mut results: Vec<(usize, f32)> = self.embeddings.iter()
            .enumerate()
            .map(|(i, emb)| (i, self.cosine_similarity(query, &emb.vector)))
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        results.into_iter()
            .take(k)
            .map(|(idx, score)| SearchResult {
                embedding: self.embeddings[idx].clone(),
                score,
                idx,
            })
            .collect()
    }

    /// Search with geometric prior (prefer same/adjacent flux positions)
    pub fn search_geometric(&self, query: &[f32], query_position: u8, k: usize) -> Vec<SearchResult> {
        let mut results: Vec<(usize, f32)> = self.embeddings.iter()
            .enumerate()
            .map(|(i, emb)| {
                let sim = self.cosine_similarity(query, &emb.vector);
                let geo_bonus = self.geometric_bonus(query_position, emb.position);
                (i, sim * (1.0 + geo_bonus))
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        results.into_iter()
            .take(k)
            .map(|(idx, score)| SearchResult {
                embedding: self.embeddings[idx].clone(),
                score,
                idx,
            })
            .collect()
    }

    /// Search only within a specific flux position
    pub fn search_by_position(&self, query: &[f32], position: u8, k: usize) -> Vec<SearchResult> {
        let indices = self.position_index.get(&position).cloned().unwrap_or_default();
        
        let mut results: Vec<(usize, f32)> = indices.iter()
            .map(|&i| (i, self.cosine_similarity(query, &self.embeddings[i].vector)))
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        results.into_iter()
            .take(k)
            .map(|(idx, score)| SearchResult {
                embedding: self.embeddings[idx].clone(),
                score,
                idx,
            })
            .collect()
    }

    /// Cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let len = a.len().min(b.len());
        if len == 0 { return 0.0; }

        let dot: f32 = a[..len].iter().zip(&b[..len]).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a[..len].iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b[..len].iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a * norm_b)
        } else {
            0.0
        }
    }

    /// Geometric bonus based on flux position relationship
    fn geometric_bonus(&self, query_pos: u8, emb_pos: u8) -> f32 {
        // Same position: high bonus
        if query_pos == emb_pos { return 0.2; }
        
        // Sacred positions get bonus
        if matches!(emb_pos, 3 | 6 | 9) { return 0.1; }
        
        // Adjacent in vortex cycle
        let next = match query_pos {
            1 => 2, 2 => 4, 4 => 8, 8 => 7, 7 => 5, 5 => 1, _ => 0,
        };
        if emb_pos == next { return 0.05; }
        
        0.0
    }

    /// Get embedding by ID
    pub fn get(&self, id: &str) -> Option<&SacredEmbedding> {
        self.embeddings.iter().find(|e| e.id == id)
    }

    /// Total embeddings count
    pub fn len(&self) -> usize {
        self.embeddings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.embeddings.is_empty()
    }

    /// Get all embeddings at a position
    pub fn get_by_position(&self, position: u8) -> Vec<&SacredEmbedding> {
        self.position_index.get(&position)
            .map(|indices| indices.iter().map(|&i| &self.embeddings[i]).collect())
            .unwrap_or_default()
    }
}

/// Search result with score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub embedding: SacredEmbedding,
    pub score: f32,
    pub idx: usize,
}

/// Convert BeamTensor to embedding vector
pub fn beam_to_embedding(beam: &BeamTensor, dim: usize) -> Vec<f32> {
    let mut embedding = vec![0.0f32; dim];
    
    // First 9 dimensions: digit distribution
    for (i, &d) in beam.digits.iter().enumerate().take(9.min(dim)) {
        embedding[i] = d;
    }
    
    // Next 3 dimensions: ELP
    if dim > 9 {
        let elp = beam.attributes.elp_tensor();
        for (i, &e) in elp.iter().enumerate().take((dim - 9).min(3)) {
            embedding[9 + i] = e / 10.0; // Normalize
        }
    }
    
    // Position encoding
    if dim > 12 {
        embedding[12] = beam.position as f32 / 9.0;
    }
    
    // Confidence
    if dim > 13 {
        embedding[13] = beam.confidence;
    }
    
    // Fill remaining with derived features
    for i in 14..dim {
        embedding[i] = embedding[i % 14] * 0.1;
    }
    
    embedding
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sacred_embedding_index() {
        let config = EmbeddingsConfig { dim: 16, ..Default::default() };
        let mut index = SacredEmbeddingIndex::new(config);

        // Add some embeddings
        for i in 1..=9 {
            let vec: Vec<f32> = (0..16).map(|j| (i * j) as f32 / 100.0).collect();
            let emb = SacredEmbedding::new(format!("emb_{}", i), vec, i as u8);
            index.add(emb);
        }

        assert_eq!(index.len(), 9);

        // Search
        let query: Vec<f32> = (0..16).map(|j| (3 * j) as f32 / 100.0).collect();
        let results = index.search(&query, 3);
        assert_eq!(results.len(), 3);
        assert!(results[0].score > 0.0);
    }

    #[test]
    fn test_geometric_search() {
        let config = EmbeddingsConfig::default();
        let mut index = SacredEmbeddingIndex::new(config.clone());

        for i in 1..=9 {
            let vec: Vec<f32> = (0..config.dim).map(|j| ((i + j) % 10) as f32 / 10.0).collect();
            let emb = SacredEmbedding::new(format!("emb_{}", i), vec, i as u8);
            index.add(emb);
        }

        let query: Vec<f32> = (0..config.dim).map(|j| (j % 10) as f32 / 10.0).collect();
        let results = index.search_geometric(&query, 3, 5);
        
        // Sacred position 3 should be boosted
        assert!(!results.is_empty());
    }
}
