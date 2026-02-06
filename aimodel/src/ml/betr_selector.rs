//! BETR-Style Benchmark-Targeted Data Selection
//!
//! Implements "Benchmark-Targeted Ranking" (BETR) from arXiv:2507.12466

use std::collections::{HashMap, HashSet};

/// Stub type for benchmark questions
#[derive(Debug, Clone)]
pub struct BenchmarkQuestion {
    pub question: String,
    pub choices: Vec<String>,
    pub correct_answer: usize,
    pub category: String,
}

/// Load MMLU questions (stub)
fn load_mmlu(_data_dir: &str, _subset: Option<&str>) -> Result<Vec<BenchmarkQuestion>, String> {
    // Stub: return empty vector
    Ok(Vec::new())
}

/// Load ARC questions (stub)
fn load_arc(_data_dir: &str, _challenge: bool) -> Result<Vec<BenchmarkQuestion>, String> {
    // Stub: return empty vector
    Ok(Vec::new())
}

/// Load HellaSwag questions (stub)
fn load_hellaswag(_data_dir: &str) -> Result<Vec<BenchmarkQuestion>, String> {
    // Stub: return empty vector
    Ok(Vec::new())
}

/// Configuration for BETR data selection
#[derive(Debug, Clone)]
pub struct BETRConfig {
    /// Sample size for direct scoring (default: 10M documents, scaled for our use case: 100K)
    pub sample_size: usize,
    /// Embedding dimension (default: 384 for all-MiniLM-L6-v2)
    pub embedding_dim: usize,
    /// Percentage of tokens to retain after filtering (default: 10% = 0.1)
    pub retention_rate: f32,
    /// Max aggregation vs mean aggregation for scoring (BETR uses max)
    pub use_max_aggregation: bool,
    /// Value function: "log" (log2(1/r)) or "inverse" (1/r)
    pub value_function: String,
}

impl Default for BETRConfig {
    fn default() -> Self {
        Self {
            sample_size: 100_000,
            embedding_dim: 384,
            retention_rate: 0.1,  // Top 10% like BETR
            use_max_aggregation: true,  // BETR uses max
            value_function: "log".to_string(),
        }
    }
}

/// Embedded benchmark example
#[derive(Debug, Clone)]
pub struct BenchmarkEmbedding {
    /// Benchmark name (e.g., "MMLU", "ARC-Challenge")
    pub benchmark: String,
    /// Task within benchmark
    pub task: String,
    /// Original question text
    pub question: String,
    /// Correct answer
    pub answer: String,
    /// Embedding vector
    pub embedding: Vec<f32>,
}

/// Document with benchmark similarity score
#[derive(Debug, Clone)]
pub struct ScoredDocument {
    /// Document text
    pub text: String,
    /// Source identifier
    pub source: String,
    /// Max similarity to any benchmark example
    pub max_similarity: f32,
    /// Which benchmark contributed max score
    pub best_matching_benchmark: String,
    /// Normalized rank score (0-1)
    pub rank_score: f32,
}

/// Store for benchmark embeddings
pub struct BenchmarkEmbeddingStore {
    /// All benchmark embeddings
    embeddings: Vec<BenchmarkEmbedding>,
    /// Embedding dimension
    dim: usize,
    /// Benchmark names loaded
    benchmarks: HashSet<String>,
}

impl BenchmarkEmbeddingStore {
    pub fn new(dim: usize) -> Self {
        Self {
            embeddings: Vec::new(),
            dim,
            benchmarks: HashSet::new(),
        }
    }

    /// Load benchmarks from real benchmark questions
    pub fn load_benchmarks(&mut self, data_dir: &str) -> Result<(), String> {
        // Load MMLU examples
        if let Ok(questions) = load_mmlu(data_dir, None) {
            let questions: Vec<BenchmarkQuestion> = questions;
            for q in questions.iter().take(5000) {  // Sample 5K per benchmark
                let text = format!("{} {}", q.question, q.choices.join(" "));
                let embedding = self.compute_embedding(&text);
                self.embeddings.push(BenchmarkEmbedding {
                    benchmark: "MMLU".to_string(),
                    task: q.category.clone(),
                    question: q.question.clone(),
                    answer: q.choices.get(q.correct_answer).cloned().unwrap_or_default(),
                    embedding,
                });
            }
            self.benchmarks.insert("MMLU".to_string());
        }

        // Load ARC Challenge examples
        if let Ok(questions) = load_arc(data_dir, true) {
            let questions: Vec<BenchmarkQuestion> = questions;
            for q in questions.iter().take(5000) {
                let text = format!("{} {}", q.question, q.choices.join(" "));
                let embedding = self.compute_embedding(&text);
                self.embeddings.push(BenchmarkEmbedding {
                    benchmark: "ARC-Challenge".to_string(),
                    task: q.category.clone(),
                    question: q.question.clone(),
                    answer: q.choices.get(q.correct_answer).cloned().unwrap_or_default(),
                    embedding,
                });
            }
            self.benchmarks.insert("ARC-Challenge".to_string());
        }

        // Load HellaSwag examples (if available)
        if let Ok(questions) = load_hellaswag(data_dir) {
            let questions: Vec<BenchmarkQuestion> = questions;
            for q in questions.iter().take(5000) {
                let text = format!("{} {}", q.question, q.choices.join(" "));
                let embedding = self.compute_embedding(&text);
                self.embeddings.push(BenchmarkEmbedding {
                    benchmark: "HellaSwag".to_string(),
                    task: "commonsense".to_string(),
                    question: q.question.clone(),
                    answer: q.choices.get(q.correct_answer).cloned().unwrap_or_default(),
                    embedding,
                });
            }
            self.benchmarks.insert("HellaSwag".to_string());
        }

        println!("[BETR] Loaded {} benchmark embeddings from {:?}", 
                 self.embeddings.len(), self.benchmarks);
        Ok(())
    }

    /// Compute simple embedding (word-level with TF-IDF-like weighting)
    fn compute_embedding(&self, text: &str) -> Vec<f32> {
        // Simple embedding: character n-gram frequency vector
        // In production, this would use embedvec or a proper embedding model
        let mut embedding = vec![0.0f32; self.dim];
        let text_lower = text.to_lowercase();
        
        // Use character trigrams for embedding
        let chars: Vec<char> = text_lower.chars().collect();
        for i in 0..chars.len().saturating_sub(2) {
            let trigram = format!("{}{}{}", chars[i], chars[i+1], chars[i+2]);
            let hash = self.hash_trigram(&trigram);
            embedding[hash % self.dim] += 1.0;
        }
        
        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            embedding.iter_mut().for_each(|x| *x /= norm);
        }
        
        embedding
    }

    fn hash_trigram(&self, trigram: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        trigram.hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Get all benchmark embeddings
    pub fn get_embeddings(&self) -> &[BenchmarkEmbedding] {
        &self.embeddings
    }

    /// Count benchmarks loaded
    pub fn benchmark_count(&self) -> usize {
        self.benchmarks.len()
    }
}

/// Document scorer using BETR methodology
pub struct BETRDocumentScorer {
    store: BenchmarkEmbeddingStore,
    config: BETRConfig,
}

impl BETRDocumentScorer {
    pub fn new(store: BenchmarkEmbeddingStore, config: BETRConfig) -> Self {
        Self { store, config }
    }

    /// Score a document by max similarity to any benchmark
    pub fn score_document(&self, text: &str) -> ScoredDocument {
        let doc_embedding = self.compute_embedding(text);
        
        let mut max_sim = 0.0f32;
        let mut best_benchmark = "none".to_string();
        let mut similarities = Vec::new();
        
        for benchmark in self.store.get_embeddings() {
            let sim = cosine_similarity(&doc_embedding, &benchmark.embedding);
            similarities.push(sim);
            
            if sim > max_sim {
                max_sim = sim;
                best_benchmark = benchmark.benchmark.clone();
            }
        }
        
        // Compute rank score using BETR's value function
        let rank_score = if self.config.use_max_aggregation {
            // Max aggregation: score by best rank
            self.compute_max_rank_score(&similarities)
        } else {
            // Mean aggregation
            self.compute_mean_rank_score(&similarities)
        };
        
        ScoredDocument {
            text: text.to_string(),
            source: "training".to_string(),
            max_similarity: max_sim,
            best_matching_benchmark: best_benchmark,
            rank_score,
        }
    }

    /// Score multiple documents in batch
    pub fn score_documents(&self, texts: &[String]) -> Vec<ScoredDocument> {
        texts.iter()
            .map(|text| self.score_document(text))
            .collect()
    }

    /// Compute max rank score (BETR default)
    fn compute_max_rank_score(&self, similarities: &[f32]) -> f32 {
        // Find rank of highest similarity
        let max_sim = similarities.iter().cloned().fold(0.0f32, f32::max);
        
        // Convert to value using log function
        // BETR uses v(r) = log2(1/r) where r is rank percentile
        // We approximate rank from similarity percentile
        let percentile = max_sim;  // Similarity is proxy for rank
        
        match self.config.value_function.as_str() {
            "log" => (1.0 + percentile).log2(),
            "inverse" => 1.0 / (1.0 - percentile + 0.01),
            _ => percentile,
        }
    }

    /// Compute mean rank score
    fn compute_mean_rank_score(&self, similarities: &[f32]) -> f32 {
        if similarities.is_empty() {
            return 0.0;
        }
        let mean: f32 = similarities.iter().sum::<f32>() / similarities.len() as f32;
        mean
    }

    fn compute_embedding(&self, text: &str) -> Vec<f32> {
        // Same embedding method as BenchmarkEmbeddingStore
        let mut embedding = vec![0.0f32; self.config.embedding_dim];
        let text_lower = text.to_lowercase();
        
        let chars: Vec<char> = text_lower.chars().collect();
        for i in 0..chars.len().saturating_sub(2) {
            let trigram = format!("{}{}{}", chars[i], chars[i+1], chars[i+2]);
            let hash = self.hash_trigram(&trigram);
            embedding[hash % self.config.embedding_dim] += 1.0;
        }
        
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            embedding.iter_mut().for_each(|x| *x /= norm);
        }
        
        embedding
    }

    fn hash_trigram(&self, trigram: &str) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        trigram.hash(&mut hasher);
        hasher.finish() as usize
    }
}

/// BETR data selector for CALM/EBRM training
pub struct BETRDataSelector {
    scorer: BETRDocumentScorer,
    config: BETRConfig,
    /// Cached scores for reuse
    score_cache: HashMap<String, f32>,
}

impl Default for BETRDataSelector {
    fn default() -> Self {
        let store = BenchmarkEmbeddingStore::new(384);
        let config = BETRConfig::default();
        let scorer = BETRDocumentScorer::new(store, config.clone());
        Self::new(scorer, config)
    }
}

impl BETRDataSelector {
    pub fn new(scorer: BETRDocumentScorer, config: BETRConfig) -> Self {
        Self {
            scorer,
            config,
            score_cache: HashMap::new(),
        }
    }

    /// Select top-K% documents by benchmark similarity
    pub fn select_training_data(&mut self, documents: &[String]) -> Vec<String> {
        // Score all documents
        let mut scored: Vec<ScoredDocument> = documents
            .iter()
            .map(|doc| {
                // Check cache
                if let Some(&score) = self.score_cache.get(doc) {
                    ScoredDocument {
                        text: doc.clone(),
                        source: "cached".to_string(),
                        max_similarity: score,
                        best_matching_benchmark: "cached".to_string(),
                        rank_score: score,
                    }
                } else {
                    let scored = self.scorer.score_document(doc);
                    self.score_cache.insert(doc.clone(), scored.rank_score);
                    scored
                }
            })
            .collect();
        
        // Sort by rank score (descending)
        scored.sort_by(|a, b| b.rank_score.partial_cmp(&a.rank_score).unwrap());
        
        // Select top retention_rate%
        let retain_count = (documents.len() as f32 * self.config.retention_rate) as usize;
        let retain_count = retain_count.max(1).min(scored.len());
        
        println!("[BETR] Selected {}/{} documents (top {:.1}%)", 
                 retain_count, documents.len(), 
                 self.config.retention_rate * 100.0);
        
        // Print distribution across benchmarks
        let mut benchmark_counts: HashMap<String, usize> = HashMap::new();
        for doc in scored.iter().take(retain_count) {
            *benchmark_counts.entry(doc.best_matching_benchmark.clone()).or_insert(0) += 1;
        }
        println!("[BETR] Selection by benchmark: {:?}", benchmark_counts);
        
        scored.into_iter()
            .take(retain_count)
            .map(|d| d.text)
            .collect()
    }

    /// Quick filter using cached scores (for efficiency)
    pub fn filter_fast(&self, documents: &[String]) -> Vec<String> {
        let mut scored: Vec<(String, f32)> = documents
            .iter()
            .filter_map(|doc| {
                self.score_cache.get(doc).map(|&score| (doc.clone(), score))
            })
            .collect();
        
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        let retain_count = (scored.len() as f32 * self.config.retention_rate) as usize;
        scored.into_iter()
            .take(retain_count)
            .map(|(text, _)| text)
            .collect()
    }

    /// Update cache with new documents
    pub fn warm_cache(&mut self, documents: &[String]) {
        for doc in documents {
            if !self.score_cache.contains_key(doc) {
                let scored = self.scorer.score_document(doc);
                self.score_cache.insert(doc.clone(), scored.rank_score);
            }
        }
    }

    /// Get cache hit rate statistics
    pub fn get_cache_stats(&self, documents: &[String]) -> (usize, usize, f32) {
        let total = documents.len();
        let cached = documents.iter()
            .filter(|doc| self.score_cache.contains_key(*doc))
            .count();
        let hit_rate = if total > 0 { cached as f32 / total as f32 } else { 0.0 };
        (cached, total, hit_rate)
    }

    /// Score context relevance for contextual learning
    pub fn score_context_relevance(&self, example: &crate::ml::stacked_flux::TrainingExample, _embedding: &crate::ml::stacked_flux::ContextualEmbedding) -> f32 {
        // Simple implementation - score based on input text similarity to cached scores
        if let Some(&score) = self.score_cache.get(&example.input) {
            score * example.priority
        } else {
            example.priority
        }
    }
}

/// Cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot / (norm_a * norm_b)
}

/// Factory function to create BETR selector from data directory
pub fn create_betr_selector(data_dir: &str, retention_rate: f32) -> Result<BETRDataSelector, String> {
    let mut store = BenchmarkEmbeddingStore::new(384);
    store.load_benchmarks(data_dir)?;
    
    let config = BETRConfig {
        retention_rate,
        ..BETRConfig::default()
    };
    
    let scorer = BETRDocumentScorer::new(store, config.clone());
    Ok(BETRDataSelector::new(scorer, config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
        
        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 1e-6);
    }

    #[test]
    fn test_document_scorer() {
        let store = BenchmarkEmbeddingStore::new(384);
        let config = BETRConfig::default();
        let scorer = BETRDocumentScorer::new(store, config);
        
        let scored = scorer.score_document("This is a test document about science.");
        assert!(scored.rank_score >= 0.0);
        assert!(scored.max_similarity >= 0.0);
    }
}
