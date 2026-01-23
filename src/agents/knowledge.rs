//! Knowledge management for coding agent
//!
//! Integrates with Confidence Lake and RAG system to:
//! - Store successful code generations
//! - Retrieve similar code examples
//! - Learn from past successes

use crate::agents::error::Result;
use crate::agents::language::Language;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(any(feature = "lake", feature = "rag"))]
use std::sync::Arc;

#[cfg(feature = "lake")]
use crate::storage::ConfidenceLake;

#[cfg(feature = "rag")]
use crate::rag::RAGRetriever;

/// Code example stored in knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub id: String,
    pub task: String,
    pub code: String,
    pub language: Language,
    pub flux_position: Option<u8>,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub attempts: usize,
}

impl CodeExample {
    /// Create new code example
    pub fn new(
        task: String,
        code: String,
        language: Language,
        flux_position: Option<u8>,
        success: bool,
        attempts: usize,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task,
            code,
            language,
            flux_position,
            confidence: Self::calculate_confidence(success, attempts, flux_position),
            timestamp: Utc::now(),
            success,
            attempts,
        }
    }
    
    /// Calculate signal strength based on success metrics
    fn calculate_confidence(success: bool, attempts: usize, flux_position: Option<u8>) -> f32 {
        let mut strength: f32 = if success { 0.8 } else { 0.3 };
        
        // Bonus for first-attempt success
        if attempts == 1 && success {
            strength += 0.1;
        }
        
        // Bonus for sacred positions (3, 6, 9)
        if let Some(pos) = flux_position {
            if pos == 3 || pos == 6 || pos == 9 {
                strength += 0.1;
            }
        }
        
        strength.min(1.0)
    }
}

/// Knowledge base for code examples
pub struct CodeKnowledge {
    #[cfg(feature = "lake")]
    confidence_lake: Option<Arc<ConfidenceLake>>,
    
    #[cfg(feature = "rag")]
    rag_retriever: Option<Arc<RAGRetriever>>,
    
    /// In-memory cache (always available)
    cache: Vec<CodeExample>,
    max_cache_size: usize,
}

impl CodeKnowledge {
    /// Create new knowledge base
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "lake")]
            confidence_lake: None,
            #[cfg(feature = "rag")]
            rag_retriever: None,
            cache: Vec::new(),
            max_cache_size: 1000,
        }
    }
    
    /// Store successful code example
    pub async fn store(&mut self, example: CodeExample) -> Result<()> {
        // Only store high-quality examples (signal >= 0.6)
        if example.confidence < 0.6 {
            return Ok(());
        }
        
        // Store in Confidence Lake if available
        #[cfg(feature = "lake")]
        if let Some(_lake) = &self.confidence_lake {
            // TODO: Convert to StoredFluxMatrix and store
            // lake.store_flux_matrix(...).await?;
        }
        
        // Store in RAG if available
        #[cfg(feature = "rag")]
        if let Some(_rag) = &self.rag_retriever {
            // TODO: Implement RAG ingestion
            // _rag.ingest_code(&example).await?;
            tracing::debug!("RAG ingestion not yet implemented");
        }
        
        // Store in memory cache
        self.cache.push(example);
        
        // Maintain cache size
        if self.cache.len() > self.max_cache_size {
            // Remove lowest signal strength examples
            self.cache.sort_by(|a, b| {
                b.confidence.partial_cmp(&a.confidence).unwrap()
            });
            self.cache.truncate(self.max_cache_size);
        }
        
        Ok(())
    }
    
    /// Retrieve similar code examples
    pub async fn retrieve_similar(
        &self,
        task: &str,
        language: Option<Language>,
        limit: usize,
    ) -> Result<Vec<CodeExample>> {
        let mut results = Vec::new();
        
        // Query RAG if available
        #[cfg(feature = "rag")]
        if let Some(_rag) = &self.rag_retriever {
            // TODO: Implement RAG retrieval
            // results = _rag.retrieve_code(task, language, limit).await?;
            tracing::debug!("RAG retrieval not yet implemented");
        }
        
        // Fallback to in-memory search
        if results.is_empty() {
            results = self.search_cache(task, language, limit);
        }
        
        Ok(results)
    }
    
    /// Search in-memory cache
    fn search_cache(
        &self,
        task: &str,
        language: Option<Language>,
        limit: usize,
    ) -> Vec<CodeExample> {
        let task_lower = task.to_lowercase();
        let mut scored: Vec<(f32, &CodeExample)> = self.cache
            .iter()
            .filter(|ex| {
                // Filter by language if specified
                if let Some(lang) = language {
                    if ex.language != lang {
                        return false;
                    }
                }
                true
            })
            .map(|ex| {
                // Simple keyword matching for similarity
                let ex_task_lower = ex.task.to_lowercase();
                let words: Vec<&str> = task_lower.split_whitespace().collect();
                let matches = words.iter()
                    .filter(|w| ex_task_lower.contains(*w))
                    .count();
                
                let similarity = (matches as f32) / (words.len() as f32);
                let score = similarity * ex.confidence;
                
                (score, ex)
            })
            .filter(|(score, _)| *score > 0.3)
            .collect();
        
        // Sort by score
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        // Take top N
        scored
            .into_iter()
            .take(limit)
            .map(|(_, ex)| ex.clone())
            .collect()
    }
    
    /// Get statistics about stored knowledge
    pub fn stats(&self) -> KnowledgeStats {
        let total = self.cache.len();
        let successful = self.cache.iter().filter(|ex| ex.success).count();
        let avg_signal = if total > 0 {
            self.cache.iter().map(|ex| ex.confidence).sum::<f32>() / total as f32
        } else {
            0.0
        };
        
        let mut by_language = std::collections::HashMap::new();
        for ex in &self.cache {
            *by_language.entry(ex.language.name().to_string()).or_insert(0) += 1;
        }
        
        KnowledgeStats {
            total_examples: total,
            successful_examples: successful,
            average_confidence: avg_signal,
            examples_by_language: by_language,
        }
    }
    
    /// Clear all cached examples
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for CodeKnowledge {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about stored knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeStats {
    pub total_examples: usize,
    pub successful_examples: usize,
    pub average_confidence: f32,
    pub examples_by_language: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_store_example() {
        let mut knowledge = CodeKnowledge::new();
        
        let example = CodeExample::new(
            "Sort a list".to_string(),
            "def sort(x): return sorted(x)".to_string(),
            Language::Python,
            Some(9),
            true,
            1,
        );
        
        knowledge.store(example).await.unwrap();
        assert_eq!(knowledge.cache.len(), 1);
    }
    
    #[tokio::test]
    async fn test_confidence_calculation() {
        // First attempt success with sacred position
        let ex1 = CodeExample::new(
            "test".to_string(),
            "code".to_string(),
            Language::Rust,
            Some(9),
            true,
            1,
        );
        assert!(ex1.confidence >= 0.9);
        
        // Failed attempt
        let ex2 = CodeExample::new(
            "test".to_string(),
            "code".to_string(),
            Language::Rust,
            None,
            false,
            3,
        );
        assert!(ex2.confidence < 0.6);
    }
    
    #[tokio::test]
    async fn test_retrieve_similar() {
        let mut knowledge = CodeKnowledge::new();
        
        // Store some examples
        for i in 0..5 {
            let example = CodeExample::new(
                format!("Sort list {}", i),
                "code".to_string(),
                Language::Python,
                Some(9),
                true,
                1,
            );
            knowledge.store(example).await.unwrap();
        }
        
        let results = knowledge.retrieve_similar("Sort list", Some(Language::Python), 3).await.unwrap();
        assert!(results.len() <= 3);
    }
}
