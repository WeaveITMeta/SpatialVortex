//! Intelligent Retrieval System for RAG
//!
//! Retrieves relevant context using sacred geometry-enhanced search
//! with multi-stage retrieval and re-ranking.

use crate::rag::vector_store::VectorStore;
use crate::models::ELPTensor;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Configuration for retrieval
#[derive(Debug, Clone)]
pub struct RetrievalConfig {
    pub top_k: usize,                    // Initial retrieval count
    pub rerank_top_n: usize,            // Final results after reranking
    pub min_similarity: f32,             // Minimum similarity threshold
    pub min_confidence: f32,       // Minimum confidence (0.6 default)
    pub use_sacred_filtering: bool,     // Filter by sacred positions
    pub diversity_factor: f32,          // Encourage diverse results (0.0-1.0)
    pub context_window: usize,          // Max context tokens
    /// Data-driven sacred weighting applied during rerank (default 1.0 = no boost).
    pub sacred_weight: f32,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            top_k: 20,
            rerank_top_n: 5,
            min_similarity: 0.5,
            min_confidence: 0.6,
            use_sacred_filtering: true,
            diversity_factor: 0.3,
            context_window: 2048,
            sacred_weight: 1.0,
        }
    }
}

/// A retrieved result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub doc_id: String,
    pub chunk_id: String,
    pub content: String,
    pub similarity: f32,
    pub confidence: f32,
    pub elp_tensor: ELPTensor,
    pub flux_position: u8,
    pub is_sacred: bool,
    pub relevance_score: f32,  // Combined score after reranking
    pub context: Option<String>, // Additional context if available
}

/// Main RAG retriever
pub struct RAGRetriever {
    vector_store: Arc<VectorStore>,
    config: RetrievalConfig,
}

impl RAGRetriever {
    pub fn new(vector_store: Arc<VectorStore>, config: RetrievalConfig) -> Self {
        Self {
            vector_store,
            config,
        }
    }
    
    /// Retrieve relevant context for a query
    pub async fn retrieve(&self, query: &str) -> Result<Vec<RetrievalResult>> {
        // Stage 1: Initial retrieval
        let initial_results = self.initial_retrieval(query).await?;
        
        // Stage 2: Re-ranking with diversity
        let reranked = self.rerank_with_diversity(initial_results).await?;
        
        // Stage 3: Context expansion
        let expanded = self.expand_context(reranked).await?;
        
        // Stage 4: Final filtering
        self.final_filtering(expanded).await
    }
    
    /// Initial retrieval stage
    async fn initial_retrieval(&self, query: &str) -> Result<Vec<RetrievalResult>> {
        let search_results = if self.config.use_sacred_filtering {
            self.vector_store
                .database()
                .search_sacred(
                    &self.vector_store.embed_text(query).await?,
                    self.config.top_k,
                    self.config.min_confidence,
                )
                .await?
        } else {
            self.vector_store.search(query, self.config.top_k).await?
        };
        
        let mut results = Vec::new();
        
        for (embedding, similarity) in search_results {
            if similarity < self.config.min_similarity {
                continue;
            }
            
            let is_sacred = [3, 6, 9].contains(&embedding.flux_position);
            
            results.push(RetrievalResult {
                doc_id: embedding.doc_id,
                chunk_id: embedding.chunk_id,
                content: embedding.metadata.get("content")
                    .cloned()
                    .unwrap_or_default(),
                similarity,
                confidence: embedding.confidence,
                elp_tensor: embedding.elp_tensor.clone(),
                flux_position: embedding.flux_position,
                is_sacred,
                relevance_score: similarity * embedding.confidence,
                context: None,
            });
        }
        
        Ok(results)
    }
    
    /// Re-rank results with diversity consideration
    async fn rerank_with_diversity(
        &self,
        mut results: Vec<RetrievalResult>,
    ) -> Result<Vec<RetrievalResult>> {
        // Calculate relevance scores with sacred boost
        for result in &mut results {
            let sacred_boost = if result.is_sacred {
                self.config.sacred_weight
            } else {
                1.0
            };
            let confidence_boost = result.confidence;
            
            result.relevance_score = result.similarity * confidence_boost * sacred_boost;
        }
        
        // Sort by relevance
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Apply MMR (Maximal Marginal Relevance) for diversity
        let mut selected = Vec::new();
        let mut remaining = results;
        
        if let Some(first) = remaining.first() {
            selected.push(first.clone());
            remaining.remove(0);
        }
        
        while selected.len() < self.config.rerank_top_n && !remaining.is_empty() {
            let mut best_idx = 0;
            let mut best_score = f32::MIN;
            
            for (idx, candidate) in remaining.iter().enumerate() {
                // Calculate similarity to already selected items
                let max_sim_to_selected = selected
                    .iter()
                    .map(|s| self.calculate_content_similarity(&candidate.content, &s.content))
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap_or(0.0);
                
                // MMR score
                let mmr_score = self.config.diversity_factor * candidate.relevance_score
                    - (1.0 - self.config.diversity_factor) * max_sim_to_selected;
                
                if mmr_score > best_score {
                    best_score = mmr_score;
                    best_idx = idx;
                }
            }
            
            selected.push(remaining.remove(best_idx));
        }
        
        Ok(selected)
    }
    
    /// Expand context by retrieving neighboring chunks
    async fn expand_context(
        &self,
        mut results: Vec<RetrievalResult>,
    ) -> Result<Vec<RetrievalResult>> {
        for result in &mut results {
            // Try to get previous and next chunks for context
            let prev_id = format!("{}_prev", result.chunk_id);
            let next_id = format!("{}_next", result.chunk_id);
            
            let mut context_parts = Vec::new();
            
            // Get previous chunk if available
            if let Ok(Some(prev)) = self.vector_store.database().get(&prev_id).await {
                if let Some(content) = prev.metadata.get("content") {
                    context_parts.push(content.clone());
                }
            }
            
            // Add current content
            context_parts.push(result.content.clone());
            
            // Get next chunk if available
            if let Ok(Some(next)) = self.vector_store.database().get(&next_id).await {
                if let Some(content) = next.metadata.get("content") {
                    context_parts.push(content.clone());
                }
            }
            
            if context_parts.len() > 1 {
                result.context = Some(context_parts.join("\n\n"));
            }
        }
        
        Ok(results)
    }
    
    /// Final filtering and token limit enforcement
    async fn final_filtering(&self, results: Vec<RetrievalResult>) -> Result<Vec<RetrievalResult>> {
        let mut filtered = Vec::new();
        let mut total_tokens = 0;
        
        for result in results {
            // Estimate tokens (rough approximation)
            let content_tokens = result.context.as_ref()
                .unwrap_or(&result.content)
                .split_whitespace()
                .count() * 2;
            
            if total_tokens + content_tokens <= self.config.context_window {
                total_tokens += content_tokens;
                filtered.push(result);
            } else {
                // Stop adding if we exceed context window
                break;
            }
        }
        
        Ok(filtered)
    }
    
    /// Calculate similarity between two text contents
    fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f32 {
        // Simple Jaccard similarity for demonstration
        let content1_lower = content1.to_lowercase();
        let content2_lower = content2.to_lowercase();
        
        let words1: std::collections::HashSet<_> = content1_lower
            .split_whitespace()
            .collect();
        
        let words2: std::collections::HashSet<_> = content2_lower
            .split_whitespace()
            .collect();
        
        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }
        
        let intersection = words1.intersection(&words2).count() as f32;
        let union = words1.union(&words2).count() as f32;
        
        intersection / union
    }
    
    /// Retrieve with sacred geometry focus
    pub async fn retrieve_sacred(&self, query: &str) -> Result<Vec<RetrievalResult>> {
        let mut config = self.config.clone();
        config.use_sacred_filtering = true;
        config.min_confidence = 0.7; // Higher threshold for sacred
        
        let retriever = RAGRetriever::new(self.vector_store.clone(), config);
        retriever.retrieve(query).await
    }
    
    /// Hybrid retrieval combining multiple strategies
    pub async fn hybrid_retrieve(&self, query: &str) -> Result<Vec<RetrievalResult>> {
        // Get regular results
        let regular = self.retrieve(query).await?;
        
        // Get sacred-focused results
        let sacred = self.retrieve_sacred(query).await?;
        
        // Combine and deduplicate
        let mut combined = regular;
        for sacred_result in sacred {
            if !combined.iter().any(|r| r.chunk_id == sacred_result.chunk_id) {
                combined.push(sacred_result);
            }
        }
        
        // Re-sort by relevance
        combined.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take top results
        combined.truncate(self.config.rerank_top_n);
        
        Ok(combined)
    }
}

/// Batch retrieval for multiple queries
pub struct BatchRetriever {
    retriever: RAGRetriever,
}

impl BatchRetriever {
    pub fn new(retriever: RAGRetriever) -> Self {
        Self { retriever }
    }
    
    /// Retrieve for multiple queries in parallel
    pub async fn batch_retrieve(&self, queries: Vec<String>) -> Result<Vec<Vec<RetrievalResult>>> {
        use futures::future::join_all;
        
        let futures: Vec<_> = queries
            .iter()
            .map(|q| self.retriever.retrieve(q))
            .collect();
        
        let results = join_all(futures).await;
        
        results.into_iter().collect()
    }
}
