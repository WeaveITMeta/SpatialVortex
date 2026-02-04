//! Vector Database for RAG System
//!
//! Stores document embeddings with sacred geometry enhancements
//! for efficient similarity search and retrieval.

use crate::models::ELPTensor;
use crate::ml::inference::OnnxSessionPool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Sacred geometry enhanced embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredEmbedding {
    pub id: String,
    pub doc_id: String,
    pub chunk_id: String,
    pub embedding: Vec<f32>,
    pub elp_tensor: ELPTensor,
    pub flux_position: u8,
    pub confidence: f32,
    pub sacred_boost: f32,
    /// Forward chain weight: boost for manifestation queries (future/create/manifest)
    /// Uses doubling sequence: 1→2→4→8→7→5→1
    #[serde(default = "default_chain_weight")]
    pub forward_chain_weight: f32,
    /// Back propagation weight: boost for reflection queries (past/why/causal)
    /// Uses halving sequence: 1→5→7→8→4→2→1
    #[serde(default = "default_chain_weight")]
    pub back_prop_weight: f32,
    pub metadata: HashMap<String, String>,
}

fn default_chain_weight() -> f32 { 1.0 }

impl SacredEmbedding {
    /// Apply sacred geometry transformation to embedding
    pub fn apply_sacred_transformation(&mut self) {
        // Positions 3, 6, 9 get boosted
        if [3, 6, 9].contains(&self.flux_position) {
            self.sacred_boost = 1.5;
            self.confidence *= 1.15;
            
            // Apply vortex flow pattern to embedding
            for (i, val) in self.embedding.iter_mut().enumerate() {
                let position_in_cycle = i % 6;
                let vortex_multiplier = match position_in_cycle {
                    0 => 1.0,  // Position 1
                    1 => 2.0,  // Position 2
                    2 => 4.0,  // Position 4
                    3 => 8.0,  // Position 8
                    4 => 7.0,  // Position 7
                    5 => 5.0,  // Position 5
                    _ => 1.0,
                };
                *val *= (vortex_multiplier / 8.0) * self.sacred_boost;
            }
        }
    }
    
    /// Calculate cosine similarity with another embedding
    pub fn cosine_similarity(&self, other: &[f32]) -> f32 {
        if self.embedding.len() != other.len() {
            return 0.0;
        }
        
        let dot_product: f32 = self.embedding
            .iter()
            .zip(other.iter())
            .map(|(a, b)| a * b)
            .sum();
        
        let norm_self: f32 = self.embedding
            .iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt();
        
        let norm_other: f32 = other
            .iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt();
        
        if norm_self == 0.0 || norm_other == 0.0 {
            return 0.0;
        }
        
        let similarity = dot_product / (norm_self * norm_other);
        
        // Apply sacred boost to similarity
        similarity * self.sacred_boost
    }
}

/// Vector database implementation
pub struct VectorDatabase {
    embeddings: Arc<RwLock<Vec<SacredEmbedding>>>,
    index: Arc<RwLock<HashMap<String, usize>>>,
    dimension: usize,
    use_sacred_geometry: bool,
}

impl VectorDatabase {
    pub fn new(dimension: usize, use_sacred_geometry: bool) -> Self {
        Self {
            embeddings: Arc::new(RwLock::new(Vec::new())),
            index: Arc::new(RwLock::new(HashMap::new())),
            dimension,
            use_sacred_geometry,
        }
    }
    
    /// Add embedding to the database
    pub async fn add_embedding(&self, mut embedding: SacredEmbedding) -> Result<()> {
        if self.use_sacred_geometry {
            embedding.apply_sacred_transformation();
        }
        
        let mut embeddings = self.embeddings.write().await;
        let mut index = self.index.write().await;
        
        let position = embeddings.len();
        index.insert(embedding.id.clone(), position);
        embeddings.push(embedding);
        
        Ok(())
    }
    
    /// Batch add embeddings
    pub async fn add_embeddings_batch(&self, mut batch: Vec<SacredEmbedding>) -> Result<()> {
        if self.use_sacred_geometry {
            for embedding in &mut batch {
                embedding.apply_sacred_transformation();
            }
        }
        
        let mut embeddings = self.embeddings.write().await;
        let mut index = self.index.write().await;
        
        for embedding in batch {
            let position = embeddings.len();
            index.insert(embedding.id.clone(), position);
            embeddings.push(embedding);
        }
        
        Ok(())
    }
    
    /// Search for k nearest neighbors
    pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<(SacredEmbedding, f32)>> {
        let embeddings = self.embeddings.read().await;
        
        // Calculate similarities
        let mut similarities: Vec<(usize, f32)> = embeddings
            .iter()
            .enumerate()
            .map(|(idx, emb)| {
                let similarity = emb.cosine_similarity(query);
                (idx, similarity)
            })
            .collect();
        
        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top k
        let results: Vec<(SacredEmbedding, f32)> = similarities
            .into_iter()
            .take(k)
            .map(|(idx, sim)| (embeddings[idx].clone(), sim))
            .collect();
        
        Ok(results)
    }
    
    /// Search with sacred geometry filtering
    pub async fn search_sacred(
        &self,
        query: &[f32],
        k: usize,
        min_confidence: f32,
    ) -> Result<Vec<(SacredEmbedding, f32)>> {
        let embeddings = self.embeddings.read().await;
        
        // Filter by signal strength and calculate similarities
        let mut similarities: Vec<(usize, f32)> = embeddings
            .iter()
            .enumerate()
            .filter(|(_, emb)| emb.confidence >= min_confidence)
            .map(|(idx, emb)| {
                let similarity = emb.cosine_similarity(query);
                (idx, similarity)
            })
            .collect();
        
        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top k
        let results: Vec<(SacredEmbedding, f32)> = similarities
            .into_iter()
            .take(k)
            .map(|(idx, sim)| (embeddings[idx].clone(), sim))
            .collect();
        
        Ok(results)
    }
    
    /// Get embedding by ID
    pub async fn get(&self, id: &str) -> Result<Option<SacredEmbedding>> {
        let index = self.index.read().await;
        let embeddings = self.embeddings.read().await;
        
        if let Some(&position) = index.get(id) {
            Ok(embeddings.get(position).cloned())
        } else {
            Ok(None)
        }
    }
    
    /// Remove embedding by ID
    pub async fn remove(&self, id: &str) -> Result<bool> {
        let mut index = self.index.write().await;
        let mut embeddings = self.embeddings.write().await;
        
        if let Some(position) = index.remove(id) {
            embeddings.remove(position);
            
            // Rebuild index after removal
            index.clear();
            for (i, emb) in embeddings.iter().enumerate() {
                index.insert(emb.id.clone(), i);
            }
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Get total number of embeddings
    pub async fn count(&self) -> usize {
        self.embeddings.read().await.len()
    }
    
    /// Clear all embeddings
    pub async fn clear(&self) -> Result<()> {
        let mut embeddings = self.embeddings.write().await;
        let mut index = self.index.write().await;
        
        embeddings.clear();
        index.clear();
        
        Ok(())
    }
    
    /// Get statistics about the database
    pub async fn stats(&self) -> VectorDBStats {
        let embeddings = self.embeddings.read().await;
        
        let sacred_count = embeddings
            .iter()
            .filter(|e| [3, 6, 9].contains(&e.flux_position))
            .count();
        
        let avg_signal = if embeddings.is_empty() {
            0.0
        } else {
            embeddings.iter().map(|e| e.confidence).sum::<f32>() / embeddings.len() as f32
        };
        
        VectorDBStats {
            total_embeddings: embeddings.len(),
            sacred_positions: sacred_count,
            average_confidence: avg_signal,
            dimension: self.dimension,
            use_sacred_geometry: self.use_sacred_geometry,
        }
    }
}

/// Statistics about the vector database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDBStats {
    pub total_embeddings: usize,
    pub sacred_positions: usize,
    pub average_confidence: f32,
    pub dimension: usize,
    pub use_sacred_geometry: bool,
}

/// Main vector store combining database and operations
pub struct VectorStore {
    database: VectorDatabase,
    onnx_pool: Option<Arc<OnnxSessionPool>>,
}

impl VectorStore {
    pub fn new(dimension: usize) -> Self {
        Self {
            database: VectorDatabase::new(dimension, true),
            onnx_pool: None,
        }
    }
    
    pub fn with_onnx_pool(mut self, pool: Arc<OnnxSessionPool>) -> Self {
        self.onnx_pool = Some(pool);
        self
    }
    
    /// Create embedding from text
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if let Some(pool) = &self.onnx_pool {
            let embeddings = pool.embed(text).await?;
            // Extract first embedding from batch
            Ok(embeddings.into_iter().next().unwrap_or_default())
        } else {
            // Fallback to simple embedding (for testing)
            Ok(self.simple_embedding(text))
        }
    }
    
    /// Simple embedding for testing (without ONNX)
    fn simple_embedding(&self, text: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut embedding = vec![0.0; 384]; // Standard embedding size
        
        // Hash-based pseudo-embedding
        for (i, word) in text.split_whitespace().enumerate() {
            let mut hasher = DefaultHasher::new();
            word.hash(&mut hasher);
            let hash = hasher.finish();
            
            let idx = (hash as usize) % 384;
            embedding[idx] += 1.0 / (i + 1) as f32;
        }
        
        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }
        
        embedding
    }
    
    /// Store document chunk with embedding
    pub async fn store_chunk(
        &self,
        doc_id: &str,
        chunk_id: &str,
        text: &str,
        elp: ELPTensor,
        flux_position: u8,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        let embedding_vec = self.embed_text(text).await?;
        
        let embedding = SacredEmbedding {
            id: format!("{}_{}", doc_id, chunk_id),
            doc_id: doc_id.to_string(),
            chunk_id: chunk_id.to_string(),
            forward_chain_weight: 1.0,
            back_prop_weight: 1.0,
            embedding: embedding_vec,
            elp_tensor: elp,
            flux_position,
            confidence: 0.8, // Default, could calculate
            sacred_boost: 1.0,
            metadata,
        };
        
        self.database.add_embedding(embedding).await
    }
    
    /// Search for similar content
    pub async fn search(&self, query: &str, k: usize) -> Result<Vec<(SacredEmbedding, f32)>> {
        let query_embedding = self.embed_text(query).await?;
        self.database.search(&query_embedding, k).await
    }
    
    /// Get database reference
    pub fn database(&self) -> &VectorDatabase {
        &self.database
    }
}
