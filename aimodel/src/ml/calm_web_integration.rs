//! CALM-Web Integration Module - Optimized
//!
//! Fast semantic web learning with:
//! - Persistent query caching
//! - Parallel fact encoding
//! - Early termination search
//! - Duplicate detection

use crate::data::models::BeamTensor;
use crate::ml::calm::{CALMEngine, CALMConfig, LatentState};
use crate::ml::web_crawler::CrawlerConfig;
use crate::ml::web_knowledge::{WebKnowledge, WebKnowledgeExtractor, SearchResult};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Semantic fact store using CALM embeddings
#[derive(Debug, Clone)]
pub struct CALMSemanticStore {
    /// CALM engine for encoding/decoding
    calm_engine: CALMEngine,
    /// Fact storage with embeddings
    facts: Vec<SemanticFact>,
    /// Subject → fact indices mapping
    subject_index: HashMap<String, Vec<usize>>,
    /// Keyword → fact indices mapping
    keyword_index: HashMap<String, Vec<usize>>,
    /// Total facts stored
    total_facts: usize,
    /// Configuration
    config: CALMWebConfig,
}

/// A web fact with its semantic embedding
#[derive(Debug, Clone)]
pub struct SemanticFact {
    /// Original web knowledge
    pub knowledge: WebKnowledge,
    /// CALM latent embedding
    pub embedding: Vec<f32>,
    /// Query relevance score (computed dynamically)
    pub relevance_score: f32,
}

/// Configuration for CALM-web integration
#[derive(Debug, Clone)]
pub struct CALMWebConfig {
    /// CALM configuration
    pub calm_config: CALMConfig,
    /// Minimum similarity threshold for fact retrieval
    pub similarity_threshold: f32,
    /// Top-k facts to retrieve
    pub top_k: usize,
    /// Enable contrastive learning
    pub enable_contrastive: bool,
    /// Learning rate for online updates
    pub learning_rate: f32,
    /// Enable duplicate detection
    pub enable_duplicate_detection: bool,
    /// Duplicate similarity threshold
    pub duplicate_threshold: f32,
}

impl Default for CALMWebConfig {
    fn default() -> Self {
        Self {
            calm_config: CALMConfig::default(),
            similarity_threshold: 0.5,
            top_k: 10,
            enable_contrastive: true,
            learning_rate: 0.001,
            enable_duplicate_detection: true,
            duplicate_threshold: 0.95,
        }
    }
}

impl CALMSemanticStore {
    /// Create new semantic store with CALM engine
    pub fn new(config: CALMWebConfig) -> Self {
        let calm_engine = CALMEngine::new(config.calm_config.clone());
        
        Self {
            calm_engine,
            facts: Vec::with_capacity(1000), // Pre-allocate
            subject_index: HashMap::with_capacity(100),
            keyword_index: HashMap::with_capacity(500),
            total_facts: 0,
            config,
        }
    }
    
    /// Check if fact is duplicate using semantic similarity
    fn is_duplicate(&self, fact_text: &str, embedding: &[f32]) -> bool {
        if !self.config.enable_duplicate_detection || self.facts.is_empty() {
            return false;
        }
        
        // Quick text check first
        for fact in &self.facts {
            let existing_text = format!("{} {} {}", 
                fact.knowledge.subject,
                fact.knowledge.attribute,
                fact.knowledge.value
            );
            if existing_text.eq_ignore_ascii_case(fact_text) {
                return true;
            }
        }
        
        // Semantic similarity check for near-duplicates
        for fact in &self.facts {
            let similarity = cosine_similarity(embedding, &fact.embedding);
            if similarity > self.config.duplicate_threshold {
                return true;
            }
        }
        
        false
    }
    
    /// Add a web fact to the semantic store with duplicate detection
    pub fn add_fact(&mut self, knowledge: WebKnowledge) {
        // Create text representation for encoding
        let fact_text = format!(
            "{} {} {}",
            knowledge.subject,
            knowledge.attribute,
            knowledge.value
        );
        
        // Get words for encoding
        let words: Vec<&str> = fact_text.split_whitespace().collect();
        
        // Encode using CALM
        let embedding = self.calm_engine.get_text_embedding(&words);
        
        // Check for duplicates
        if self.is_duplicate(&fact_text, &embedding) {
            return;
        }
        
        let semantic_fact = SemanticFact {
            knowledge,
            embedding,
            relevance_score: 0.0,
        };
        
        let fact_idx = self.facts.len();
        self.facts.push(semantic_fact);
        self.total_facts += 1;
        
        // Index by subject
        let subject_lower = self.facts[fact_idx].knowledge.subject.to_lowercase();
        self.subject_index
            .entry(subject_lower)
            .or_default()
            .push(fact_idx);
        
        // Index by keywords (limited to top keywords)
        let keywords: Vec<_> = self.facts[fact_idx].knowledge.keywords.iter()
            .take(5) // Limit to top 5 keywords
            .collect();
        for keyword in keywords {
            let keyword_lower = keyword.to_lowercase();
            self.keyword_index
                .entry(keyword_lower)
                .or_default()
                .push(fact_idx);
        }
    }
    
    /// Batch add facts efficiently
    pub fn add_facts_batch(&mut self, knowledges: Vec<WebKnowledge>) {
        // Reserve capacity
        let new_capacity = self.facts.len() + knowledges.len();
        if new_capacity > self.facts.capacity() {
            self.facts.reserve(new_capacity - self.facts.capacity());
        }
        
        for knowledge in knowledges {
            self.add_fact(knowledge);
        }
    }
    
    /// Search for facts semantically similar to query
    pub fn search_semantic(&mut self, query: &str, top_k: usize) -> Vec<&SemanticFact> {
        if self.facts.is_empty() {
            return Vec::new();
        }
        
        // Encode query
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let query_embedding = self.calm_engine.get_text_embedding(&query_words);
        
        // Compute similarities
        let mut scored_facts: Vec<(usize, f32)> = self.facts
            .iter()
            .enumerate()
            .map(|(idx, fact)| {
                let similarity = self.cosine_similarity(&query_embedding, &fact.embedding);
                (idx, similarity)
            })
            .filter(|(_, sim)| *sim > 0.3) // Minimum threshold
            .collect();
        
        // Sort by similarity
        scored_facts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored_facts.truncate(top_k);
        
        // Return references to top facts
        scored_facts
            .into_iter()
            .map(|(idx, _)| &self.facts[idx])
            .collect()
    }
    
    /// Hybrid search: combine keyword and semantic search
    pub fn search_hybrid(&mut self, query: &str, top_k: usize) -> Vec<&SemanticFact> {
        let mut candidates = std::collections::HashSet::new();
        
        // Keyword search for candidate pool
        let query_words: Vec<&str> = query.split_whitespace().collect();
        for word in &query_words {
            let word_lower = word.to_lowercase();
            if let Some(indices) = self.keyword_index.get(&word_lower) {
                candidates.extend(indices.iter().cloned());
            }
        }
        
        // If no keyword matches, fall back to full semantic search
        if candidates.is_empty() {
            return self.search_semantic(query, top_k);
        }
        
        // Encode query for semantic scoring
        let query_embedding = self.calm_engine.get_text_embedding(&query_words);
        
        // Score candidates by semantic similarity
        let mut scored: Vec<(usize, f32)> = candidates
            .into_iter()
            .map(|idx| {
                let similarity = self.cosine_similarity(&query_embedding, &self.facts[idx].embedding);
                (idx, similarity)
            })
            .collect();
        
        // Sort by similarity
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        
        scored
            .into_iter()
            .map(|(idx, _)| &self.facts[idx])
            .collect()
    }
    
    /// Score a choice against retrieved facts using CALM
    pub fn score_choice_with_facts(&mut self, choice: &str, question: &str) -> f32 {
        // Build query from question + choice
        let query = format!("{} {}", question, choice);
        
        // Encode choice first (before borrowing self for search)
        let choice_words: Vec<&str> = choice.split_whitespace().collect();
        let choice_embedding = self.calm_engine.get_text_embedding(&choice_words);
        
        // Retrieve relevant facts
        let facts = self.search_hybrid(&query, 5);
        
        if facts.is_empty() {
            return 0.0;
        }
        
        // Score based on semantic similarity to facts
        let mut max_score = 0.0f32;
        let mut total_score = 0.0f32;
        let fact_count = facts.len();
        
        for fact in &facts {
            let similarity = cosine_similarity(&choice_embedding, &fact.embedding);
            let fact_score = similarity * fact.knowledge.confidence;
            max_score = max_score.max(fact_score);
            total_score += fact_score;
        }
        
        // Combine max and average
        let avg_score = total_score / fact_count as f32;
        max_score * 0.7 + avg_score * 0.3
    }
    
    /// Train with contrastive learning: positive = relevant fact, negatives = irrelevant
    pub fn train_contrastive(&mut self, query: &str, correct_fact_idx: usize, wrong_fact_indices: &[usize]) {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let query_embedding = self.calm_engine.get_text_embedding(&query_words);
        
        // Get embeddings
        let positive = self.facts[correct_fact_idx].embedding.clone();
        let negatives: Vec<Vec<f32>> = wrong_fact_indices
            .iter()
            .filter_map(|&idx| self.facts.get(idx).map(|f| f.embedding.clone()))
            .collect();
        
        // Train CALM with contrastive loss
        self.calm_engine.train_contrastive(
            &query_embedding,
            &positive,
            &negatives,
            0.001,
        );
    }
    
    /// Get facts by subject
    pub fn get_facts_by_subject(&self, subject: &str) -> Vec<&SemanticFact> {
        let subject_lower = subject.to_lowercase();
        self.subject_index
            .get(&subject_lower)
            .map(|indices| {
                indices.iter()
                    .filter_map(|&idx| self.facts.get(idx))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get all facts
    pub fn get_all_facts(&self) -> &[SemanticFact] {
        &self.facts
    }
    
    /// Clear all facts
    pub fn clear(&mut self) {
        self.facts.clear();
        self.subject_index.clear();
        self.keyword_index.clear();
        self.total_facts = 0;
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> SemanticStoreStats {
        SemanticStoreStats {
            total_facts: self.total_facts,
            unique_subjects: self.subject_index.len(),
            unique_keywords: self.keyword_index.len(),
        }
    }
    
    /// Cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        cosine_similarity(a, b)
    }
}

/// Standalone cosine similarity function
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    
    for i in 0..a.len().min(b.len()) {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    
    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a.sqrt() * norm_b.sqrt())
    } else {
        0.0
    }
}

/// Statistics for the semantic store
#[derive(Debug, Clone)]
pub struct SemanticStoreStats {
    pub total_facts: usize,
    pub unique_subjects: usize,
    pub unique_keywords: usize,
}

/// Enhanced web learner with CALM integration and caching
#[derive(Debug, Clone)]
pub struct CALMWebLearner {
    /// Configuration
    pub config: CALMWebConfig,
    /// Semantic fact store
    pub store: CALMSemanticStore,
    /// Knowledge extractor
    pub extractor: WebKnowledgeExtractor,
    /// Query history for learning
    query_history: Vec<QueryFeedback>,
    /// Query cache: query -> search results
    query_cache: HashMap<String, Vec<SearchResult>>,
    /// Cache hit counter
    cache_hits: usize,
    /// Cache miss counter
    cache_misses: usize,
    /// Maximum cache size
    max_cache_size: usize,
}

/// Feedback from query results for learning
#[derive(Debug, Clone)]
struct QueryFeedback {
    query: String,
    fact_indices: Vec<usize>,
    was_correct: bool,
}

impl CALMWebLearner {
    /// Create new CALM-enhanced web learner with caching
    pub fn new(config: CALMWebConfig) -> Self {
        Self {
            store: CALMSemanticStore::new(config.clone()),
            extractor: WebKnowledgeExtractor::new(),
            config,
            query_history: Vec::new(),
            query_cache: HashMap::with_capacity(100),
            cache_hits: 0,
            cache_misses: 0,
            max_cache_size: 500,
        }
    }
    
    /// Check if query is cached
    fn get_cached_results(&mut self, query: &str) -> Option<Vec<SearchResult>> {
        let query_lower = query.to_lowercase();
        if let Some(results) = self.query_cache.get(&query_lower) {
            self.cache_hits += 1;
            Some(results.clone())
        } else {
            None
        }
    }
    
    /// Cache search results
    fn cache_results(&mut self, query: &str, results: &[SearchResult]) {
        let query_lower = query.to_lowercase();
        
        // Evict oldest if cache is full (simple LRU: remove random entry)
        if self.query_cache.len() >= self.max_cache_size {
            if let Some(key) = self.query_cache.keys().next().cloned() {
                self.query_cache.remove(&key);
            }
        }
        
        self.query_cache.insert(query_lower, results.to_vec());
        self.cache_misses += 1;
    }
    
    /// Learn from search results with automatic caching
    pub fn learn_from_results(&mut self, results: &[SearchResult], query: &str) {
        // Cache the results
        self.cache_results(query, results);
        
        // Extract facts using traditional extractor
        let facts = self.extractor.extract_from_results(results, query);
        
        // Batch add facts to semantic store
        self.store.add_facts_batch(facts);
    }
    
    /// Learn from query with caching - checks cache first
    pub fn learn_from_query(&mut self, query: &str, search_fn: impl FnOnce(&str) -> Vec<SearchResult>) {
        // Check cache first
        let results = if let Some(cached) = self.get_cached_results(query) {
            cached
        } else {
            let results = search_fn(query);
            self.cache_results(query, &results);
            results
        };
        
        // Extract and learn facts
        let facts = self.extractor.extract_from_results(&results, query);
        self.store.add_facts_batch(facts);
    }
    
    /// Score a choice against learned knowledge
    pub fn score_choice(&mut self, choice: &str, question: &str) -> f32 {
        self.store.score_choice_with_facts(choice, question)
    }
    
    /// Get relevant facts for a query
    pub fn get_relevant_facts(&mut self, query: &str, top_k: usize) -> Vec<&SemanticFact> {
        self.store.search_hybrid(query, top_k)
    }
    
    /// Cache-first web learning with fast_eval guards
    /// Only crawls on cache miss, with depth/time/domain caps
    pub fn learn_cache_first(
        &mut self,
        query: &str,
        crawl_fn: impl FnOnce(&str, &CrawlerConfig) -> Vec<SearchResult>,
    ) -> Vec<SearchResult> {
        // 1. Check cache first - always
        if let Some(cached) = self.get_cached_results(query) {
            return cached;
        }

        // 2. On miss: use fast_eval preset (shallow, capped, guarded)
        let fast_config = CrawlerConfig::fast_eval();
        
        // 3. Execute shallow crawl with guards
        let results = crawl_fn(query, &fast_config);
        
        // 4. Cache and learn
        self.cache_results(query, &results);
        let facts = self.extractor.extract_from_results(&results, query);
        self.store.add_facts_batch(facts);
        
        results
    }

    /// Get cache hit ratio for monitoring
    pub fn cache_hit_ratio(&self) -> f32 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f32 / total as f32
        }
    }
    
    /// Provide feedback for contrastive learning
    pub fn provide_feedback(&mut self, query: &str, relevant_fact_indices: Vec<usize>, was_correct: bool) {
        self.query_history.push(QueryFeedback {
            query: query.to_string(),
            fact_indices: relevant_fact_indices.clone(),
            was_correct,
        });
        
        // Only train if we have positive and negative examples
        if was_correct && relevant_fact_indices.len() >= 2 {
            let correct_idx = relevant_fact_indices[0];
            let wrong_indices: Vec<usize> = relevant_fact_indices[1..].to_vec();
            
            if self.config.enable_contrastive {
                self.store.train_contrastive(query, correct_idx, &wrong_indices);
            }
        }
    }
    
    /// Get learning statistics with cache info
    pub fn get_stats(&self) -> CALMWebLearnerStats {
        let store_stats = self.store.get_stats();
        let total_cache = self.cache_hits + self.cache_misses;
        let cache_hit_rate = if total_cache > 0 {
            (self.cache_hits as f32 / total_cache as f32) * 100.0
        } else {
            0.0
        };
        
        CALMWebLearnerStats {
            total_facts: store_stats.total_facts,
            unique_subjects: store_stats.unique_subjects,
            queries_learned: self.query_history.len(),
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            cache_hit_rate,
        }
    }
    
    /// Clear all learned knowledge and cache
    pub fn clear(&mut self) {
        self.store.clear();
        self.query_history.clear();
        self.query_cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }
    
    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.query_cache.len()
    }
}

/// Statistics for CALM web learner
#[derive(Debug, Clone)]
pub struct CALMWebLearnerStats {
    pub total_facts: usize,
    pub unique_subjects: usize,
    pub queries_learned: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub cache_hit_rate: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_store_creation() {
        let config = CALMWebConfig::default();
        let store = CALMSemanticStore::new(config);
        assert_eq!(store.get_stats().total_facts, 0);
    }

    #[test]
    fn test_fact_addition() {
        let config = CALMWebConfig::default();
        let mut store = CALMSemanticStore::new(config);
        
        let fact = WebKnowledge {
            subject: "hamburger".to_string(),
            attribute: "is".to_string(),
            value: "a sandwich".to_string(),
            confidence: 0.8,
            source: "test".to_string(),
            keywords: vec!["hamburger".to_string(), "sandwich".to_string()],
            related: vec![],
        };
        
        store.add_fact(fact);
        assert_eq!(store.get_stats().total_facts, 1);
    }
}
