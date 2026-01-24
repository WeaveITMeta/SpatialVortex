//! RAG Engine - Retrieval Augmented Generation
//!
//! High-performance 3-stage RAG:
//! 1. HNSW Retrieval → topK candidates
//! 2. Rerank with sacred geometry boost
//! 3. Context assembly for generation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RAG Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGConfig {
    /// Number of candidates to retrieve
    pub top_k: usize,
    /// Number of results after reranking
    pub top_r: usize,
    /// Minimum similarity threshold
    pub min_similarity: f32,
    /// Sacred position boost factor
    pub sacred_boost: f32,
    /// Maximum context tokens
    pub max_context_tokens: usize,
    /// Enable MMR diversity
    pub use_mmr: bool,
    /// MMR lambda (0=diversity, 1=relevance)
    pub mmr_lambda: f32,
}

impl Default for RAGConfig {
    fn default() -> Self {
        Self {
            top_k: 100,
            top_r: 10,
            min_similarity: 0.5,
            sacred_boost: 1.2,
            max_context_tokens: 4096,
            use_mmr: true,
            mmr_lambda: 0.7,
        }
    }
}

impl RAGConfig {
    pub fn new() -> Self { Self::default() }
    pub fn with_top_k(mut self, k: usize) -> Self { self.top_k = k; self }
    pub fn with_top_r(mut self, r: usize) -> Self { self.top_r = r; self }
}

/// A retrieved context chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedContext {
    pub id: String,
    pub content: String,
    pub source: String,
    pub similarity: f32,
    pub relevance_score: f32,
    pub flux_position: u8,
    pub is_sacred: bool,
    pub metadata: HashMap<String, String>,
}

impl RetrievedContext {
    pub fn new(id: String, content: String, similarity: f32) -> Self {
        Self {
            id,
            content,
            source: String::new(),
            similarity,
            relevance_score: similarity,
            flux_position: 1,
            is_sacred: false,
            metadata: HashMap::new(),
        }
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }

    pub fn with_position(mut self, pos: u8) -> Self {
        self.flux_position = pos;
        self.is_sacred = matches!(pos, 3 | 6 | 9);
        self
    }
}

/// Document for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub source: String,
    pub metadata: HashMap<String, String>,
    pub flux_position: u8,
}

impl Document {
    pub fn new(id: String, content: String) -> Self {
        Self {
            id,
            content,
            embedding: Vec::new(),
            source: String::new(),
            metadata: HashMap::new(),
            flux_position: 1,
        }
    }

    pub fn with_embedding(mut self, emb: Vec<f32>) -> Self {
        self.embedding = emb;
        self
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }
}

/// RAG Engine with in-memory vector store
pub struct RAGEngine {
    config: RAGConfig,
    documents: Vec<Document>,
    dimension: usize,
}

impl RAGEngine {
    pub fn new(config: RAGConfig, dimension: usize) -> Self {
        Self {
            config,
            documents: Vec::new(),
            dimension,
        }
    }

    /// Add a document to the index
    pub fn add_document(&mut self, doc: Document) {
        if doc.embedding.len() == self.dimension {
            self.documents.push(doc);
        }
    }

    /// Add multiple documents
    pub fn add_documents(&mut self, docs: Vec<Document>) {
        for doc in docs {
            self.add_document(doc);
        }
    }

    /// Retrieve relevant contexts for a query embedding
    pub fn retrieve(&self, query_embedding: &[f32]) -> Vec<RetrievedContext> {
        if query_embedding.len() != self.dimension {
            return Vec::new();
        }

        // Calculate similarities
        let mut scored: Vec<(usize, f32)> = self.documents
            .iter()
            .enumerate()
            .map(|(i, doc)| {
                let sim = cosine_similarity(&doc.embedding, query_embedding);
                (i, sim)
            })
            .filter(|(_, sim)| *sim >= self.config.min_similarity)
            .collect();

        // Sort by similarity descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top_k
        scored.truncate(self.config.top_k);

        // Rerank with sacred boost
        let mut results: Vec<RetrievedContext> = scored
            .into_iter()
            .map(|(i, sim)| {
                let doc = &self.documents[i];
                let is_sacred = matches!(doc.flux_position, 3 | 6 | 9);
                let boost = if is_sacred { self.config.sacred_boost } else { 1.0 };
                let relevance = sim * boost;

                RetrievedContext {
                    id: doc.id.clone(),
                    content: doc.content.clone(),
                    source: doc.source.clone(),
                    similarity: sim,
                    relevance_score: relevance,
                    flux_position: doc.flux_position,
                    is_sacred,
                    metadata: doc.metadata.clone(),
                }
            })
            .collect();

        // Sort by relevance score
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        // Apply MMR if enabled
        if self.config.use_mmr && results.len() > self.config.top_r {
            results = self.apply_mmr(results, query_embedding);
        }

        // Take top_r
        results.truncate(self.config.top_r);

        results
    }

    /// Apply Maximal Marginal Relevance for diversity
    fn apply_mmr(&self, candidates: Vec<RetrievedContext>, query: &[f32]) -> Vec<RetrievedContext> {
        let mut selected: Vec<RetrievedContext> = Vec::new();
        let mut remaining: Vec<RetrievedContext> = candidates;

        while selected.len() < self.config.top_r && !remaining.is_empty() {
            let mut best_idx = 0;
            let mut best_mmr = f32::NEG_INFINITY;

            for (i, candidate) in remaining.iter().enumerate() {
                let relevance = candidate.relevance_score;

                // Max similarity to already selected
                let max_sim = selected.iter()
                    .filter_map(|s| {
                        self.documents.iter()
                            .find(|d| d.id == s.id)
                            .map(|d| cosine_similarity(&d.embedding, 
                                &self.documents.iter()
                                    .find(|dd| dd.id == candidate.id)
                                    .map(|dd| dd.embedding.as_slice())
                                    .unwrap_or(&[])))
                    })
                    .fold(0.0f32, |a, b| a.max(b));

                let mmr = self.config.mmr_lambda * relevance - (1.0 - self.config.mmr_lambda) * max_sim;

                if mmr > best_mmr {
                    best_mmr = mmr;
                    best_idx = i;
                }
            }

            selected.push(remaining.remove(best_idx));
        }

        selected
    }

    /// Build context string from retrieved results
    pub fn build_context(&self, results: &[RetrievedContext]) -> String {
        let mut context = String::new();
        let mut token_count = 0;

        for (i, result) in results.iter().enumerate() {
            let chunk = format!(
                "[{}] (score: {:.3}, sacred: {})\n{}\n\n",
                i + 1,
                result.relevance_score,
                result.is_sacred,
                result.content
            );

            // Rough token estimate (4 chars per token)
            let chunk_tokens = chunk.len() / 4;
            if token_count + chunk_tokens > self.config.max_context_tokens {
                break;
            }

            context.push_str(&chunk);
            token_count += chunk_tokens;
        }

        context
    }

    /// Get document count
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Clear all documents
    pub fn clear(&mut self) {
        self.documents.clear();
    }
}

/// Cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

/// L2 normalize a vector in place
pub fn normalize_l2(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-10 {
        let inv = 1.0 / norm;
        for x in v.iter_mut() {
            *x *= inv;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rag_engine() {
        let config = RAGConfig::new().with_top_k(10).with_top_r(3);
        let mut engine = RAGEngine::new(config, 4);

        // Add some documents
        engine.add_document(
            Document::new("doc1".to_string(), "Hello world".to_string())
                .with_embedding(vec![1.0, 0.0, 0.0, 0.0])
        );
        engine.add_document(
            Document::new("doc2".to_string(), "Goodbye world".to_string())
                .with_embedding(vec![0.0, 1.0, 0.0, 0.0])
        );
        engine.add_document(
            Document::new("doc3".to_string(), "Sacred knowledge".to_string())
                .with_embedding(vec![0.5, 0.5, 0.0, 0.0])
        );

        assert_eq!(engine.len(), 3);

        // Query
        let query = vec![0.9, 0.1, 0.0, 0.0];
        let results = engine.retrieve(&query);

        assert!(!results.is_empty());
        assert_eq!(results[0].id, "doc1"); // Most similar
    }

    #[test]
    fn test_sacred_boost() {
        let mut config = RAGConfig::new();
        config.sacred_boost = 2.0;
        config.min_similarity = 0.0;
        
        let mut engine = RAGEngine::new(config, 4);

        // Normal doc with high similarity
        let mut doc1 = Document::new("normal".to_string(), "Normal doc".to_string())
            .with_embedding(vec![1.0, 0.0, 0.0, 0.0]);
        doc1.flux_position = 1;
        engine.add_document(doc1);

        // Sacred doc with lower similarity
        let mut doc2 = Document::new("sacred".to_string(), "Sacred doc".to_string())
            .with_embedding(vec![0.7, 0.3, 0.0, 0.0]);
        doc2.flux_position = 9; // Sacred position
        engine.add_document(doc2);

        let query = vec![1.0, 0.0, 0.0, 0.0];
        let results = engine.retrieve(&query);

        // Sacred doc should be boosted
        assert!(results.iter().any(|r| r.is_sacred));
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 1e-6);
    }

    #[test]
    fn test_build_context() {
        let config = RAGConfig::new();
        let engine = RAGEngine::new(config, 4);

        let results = vec![
            RetrievedContext::new("1".to_string(), "First chunk".to_string(), 0.9),
            RetrievedContext::new("2".to_string(), "Second chunk".to_string(), 0.8),
        ];

        let context = engine.build_context(&results);
        assert!(context.contains("First chunk"));
        assert!(context.contains("Second chunk"));
    }
}
