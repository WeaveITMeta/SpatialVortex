//! Test-Time Benchmark-Weighted Retrieval (BETR Phase 4)
//!
//! Implements benchmark-aware retrieval that weights documents by similarity
//! to the current benchmark type during inference.
//!
//! ## Key Innovation
//! At test time, the system knows which benchmark task it's evaluating on.
//! This module uses that information to boost retrieval of documents that
//! are similar to the target benchmark examples.
//!
//! ## Method
//! 1. Detect current benchmark context (MMLU, ARC, etc.)
//! 2. Compute query embedding
//! 3. Retrieve candidate documents
//! 4. Re-rank by combined score: semantic_similarity * benchmark_similarity
//! 5. Return top-k re-ranked documents

use std::collections::HashMap;

/// Benchmark context for retrieval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BenchmarkContext {
    MMLU,
    ARCChallenge,
    HellaSwag,
    GSM8K,
    TruthfulQA,
    CommonsenseQA,
    SQuAD,
    Winogrande,
    PIQA,
    General,
}

impl BenchmarkContext {
    /// Detect benchmark from task name
    pub fn from_task(task: &str) -> Self {
        let lower = task.to_lowercase();
        if lower.contains("mmlu") {
            BenchmarkContext::MMLU
        } else if lower.contains("arc") {
            BenchmarkContext::ARCChallenge
        } else if lower.contains("hellaswag") {
            BenchmarkContext::HellaSwag
        } else if lower.contains("gsm8k") || lower.contains("math") {
            BenchmarkContext::GSM8K
        } else if lower.contains("truthfulqa") {
            BenchmarkContext::TruthfulQA
        } else if lower.contains("commonsenseqa") {
            BenchmarkContext::CommonsenseQA
        } else if lower.contains("squad") {
            BenchmarkContext::SQuAD
        } else if lower.contains("winogrande") {
            BenchmarkContext::Winogrande
        } else if lower.contains("piqa") {
            BenchmarkContext::PIQA
        } else {
            BenchmarkContext::General
        }
    }

    /// Get representative keywords for this benchmark
    pub fn keywords(&self) -> Vec<&str> {
        match self {
            BenchmarkContext::MMLU => vec!["question", "answer", "knowledge", "subject", "exam"],
            BenchmarkContext::ARCChallenge => vec!["science", "reasoning", "question", "choice", "correct"],
            BenchmarkContext::HellaSwag => vec!["commonsense", "completion", "sentence", "context"],
            BenchmarkContext::GSM8K => vec!["math", "problem", "solution", "step", "calculate"],
            BenchmarkContext::TruthfulQA => vec!["truth", "fact", "misconception", "question", "answer"],
            BenchmarkContext::CommonsenseQA => vec!["commonsense", "question", "answer", "reasoning"],
            BenchmarkContext::SQuAD => vec!["reading", "comprehension", "passage", "question", "context"],
            BenchmarkContext::Winogrande => vec!["pronoun", "resolution", "coreference", "sentence"],
            BenchmarkContext::PIQA => vec!["physical", "commonsense", "interaction", "question"],
            BenchmarkContext::General => vec!["text", "information", "content"],
        }
    }
}

/// Configuration for benchmark-weighted retrieval
#[derive(Debug, Clone)]
pub struct BenchmarkWeightedRetrievalConfig {
    /// Embedding dimension
    pub embedding_dim: usize,
    /// Weight for benchmark similarity (0-1)
    pub benchmark_weight: f32,
    /// Weight for semantic similarity (0-1)
    pub semantic_weight: f32,
    /// Number of candidates to retrieve before re-ranking
    pub num_candidates: usize,
    /// Final number of documents to return
    pub top_k: usize,
    /// Minimum benchmark similarity to apply boost
    pub min_benchmark_sim: f32,
}

impl Default for BenchmarkWeightedRetrievalConfig {
    fn default() -> Self {
        Self {
            embedding_dim: 384,
            benchmark_weight: 0.3,      // 30% benchmark, 70% semantic
            semantic_weight: 0.7,
            num_candidates: 50,         // Retrieve 50, re-rank to top_k
            top_k: 10,
            min_benchmark_sim: 0.2,
        }
    }
}

/// Retrieved document with scores
#[derive(Debug, Clone)]
pub struct BenchmarkWeightedDocument {
    /// Document text
    pub text: String,
    /// Document ID or source
    pub source: String,
    /// Semantic similarity to query
    pub semantic_score: f32,
    /// Similarity to benchmark context
    pub benchmark_score: f32,
    /// Combined weighted score
    pub combined_score: f32,
    /// Which benchmark contributed most
    pub matched_benchmark: BenchmarkContext,
}

/// Benchmark-weighted retrieval engine
pub struct BenchmarkWeightedRetriever {
    config: BenchmarkWeightedRetrievalConfig,
    /// Benchmark context centroids (embeddings)
    benchmark_centroids: HashMap<BenchmarkContext, Vec<f32>>,
    /// Document store (simplified)
    documents: Vec<(String, String, Vec<f32>)>, // (text, source, embedding)
}

impl BenchmarkWeightedRetriever {
    pub fn new(config: BenchmarkWeightedRetrievalConfig) -> Self {
        Self {
            config,
            benchmark_centroids: HashMap::new(),
            documents: Vec::new(),
        }
    }

    /// Initialize benchmark centroids from example texts
    pub fn initialize_benchmarks(&mut self, examples: HashMap<BenchmarkContext, Vec<String>>) {
        for (context, texts) in examples {
            if texts.is_empty() {
                continue;
            }
            
            // Compute centroid embedding
            let mut centroid = vec![0.0f32; self.config.embedding_dim];
            for text in &texts {
                let embedding = self.embed(text);
                for (i, &v) in embedding.iter().enumerate() {
                    if i < centroid.len() {
                        centroid[i] += v;
                    }
                }
            }
            
            // Average and normalize
            let n = texts.len() as f32;
            centroid.iter_mut().for_each(|v| *v /= n);
            let norm: f32 = centroid.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                centroid.iter_mut().for_each(|x| *x /= norm);
            }
            
            self.benchmark_centroids.insert(context, centroid);
        }
        
        println!("[BW-Retrieval] Initialized {} benchmark centroids", 
                 self.benchmark_centroids.len());
    }

    /// Add documents to the retrieval store
    pub fn add_documents(&mut self, documents: Vec<(String, String)>) {
        for (text, source) in documents {
            let embedding = self.embed(&text);
            self.documents.push((text, source, embedding));
        }
    }

    /// Compute embedding for text
    fn embed(&self, text: &str) -> Vec<f32> {
        let mut embedding = vec![0.0f32; self.config.embedding_dim];
        let text_lower = text.to_lowercase();
        
        // Character trigram embedding
        let chars: Vec<char> = text_lower.chars().collect();
        for i in 0..chars.len().saturating_sub(2) {
            let trigram = format!("{}{}{}", chars[i], chars[i+1], chars[i+2]);
            let hash = self.hash_trigram(&trigram);
            embedding[hash % self.config.embedding_dim] += 1.0;
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

    /// Retrieve documents weighted by benchmark context
    pub fn retrieve(
        &self,
        query: &str,
        context: BenchmarkContext,
    ) -> Vec<BenchmarkWeightedDocument> {
        let query_embedding = self.embed(query);
        
        // Get benchmark centroid for this context
        let benchmark_centroid = self.benchmark_centroids.get(&context);
        
        // Score all documents
        let mut scored: Vec<BenchmarkWeightedDocument> = self.documents.iter()
            .map(|(text, source, doc_embedding)| {
                // Semantic similarity
                let semantic_score = cosine_similarity(&query_embedding, doc_embedding);
                
                // Benchmark similarity
                let (benchmark_score, matched) = if let Some(centroid) = benchmark_centroid {
                    let sim = cosine_similarity(doc_embedding, centroid);
                    (sim, context)
                } else {
                    // Find best matching benchmark
                    let mut best_sim = 0.0f32;
                    let mut best_ctx = BenchmarkContext::General;
                    for (ctx, centroid) in &self.benchmark_centroids {
                        let sim = cosine_similarity(doc_embedding, centroid);
                        if sim > best_sim {
                            best_sim = sim;
                            best_ctx = *ctx;
                        }
                    }
                    (best_sim, best_ctx)
                };
                
                // Compute combined score
                let boost = if benchmark_score > self.config.min_benchmark_sim {
                    1.0 + (benchmark_score * self.config.benchmark_weight)
                } else {
                    1.0
                };
                let combined = semantic_score * boost * self.config.semantic_weight +
                               benchmark_score * self.config.benchmark_weight;
                
                BenchmarkWeightedDocument {
                    text: text.clone(),
                    source: source.clone(),
                    semantic_score,
                    benchmark_score,
                    combined_score: combined,
                    matched_benchmark: matched,
                }
            })
            .collect();
        
        // Sort by combined score descending
        scored.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());
        
        // Return top-k
        scored.into_iter().take(self.config.top_k).collect()
    }

    /// Fast retrieval with pre-filtering by benchmark
    pub fn retrieve_fast(
        &self,
        query: &str,
        context: BenchmarkContext,
    ) -> Vec<BenchmarkWeightedDocument> {
        let query_embedding = self.embed(query);
        let benchmark_centroid = self.benchmark_centroids.get(&context);
        
        // First pass: get candidates by semantic similarity
        let mut candidates: Vec<(usize, f32)> = self.documents.iter()
            .enumerate()
            .map(|(idx, (_, _, embedding))| {
                let sim = cosine_similarity(&query_embedding, embedding);
                (idx, sim)
            })
            .collect();
        
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Second pass: re-rank top candidates with benchmark weighting
        let candidate_indices: Vec<usize> = candidates.iter()
            .take(self.config.num_candidates)
            .map(|(idx, _)| *idx)
            .collect();
        
        let mut scored: Vec<BenchmarkWeightedDocument> = candidate_indices.iter()
            .filter_map(|&idx| {
                self.documents.get(idx).map(|(text, source, embedding)| {
                    let semantic_score = cosine_similarity(&query_embedding, embedding);
                    
                    let (benchmark_score, matched) = if let Some(centroid) = benchmark_centroid {
                        let sim = cosine_similarity(embedding, centroid);
                        (sim, context)
                    } else {
                        (0.0, BenchmarkContext::General)
                    };
                    
                    let boost = if benchmark_score > self.config.min_benchmark_sim {
                        1.0 + (benchmark_score * self.config.benchmark_weight)
                    } else {
                        1.0
                    };
                    let combined = semantic_score * boost;
                    
                    BenchmarkWeightedDocument {
                        text: text.clone(),
                        source: source.clone(),
                        semantic_score,
                        benchmark_score,
                        combined_score: combined,
                        matched_benchmark: matched,
                    }
                })
            })
            .collect();
        
        scored.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());
        scored.into_iter().take(self.config.top_k).collect()
    }

    /// Get retrieval statistics
    pub fn get_stats(&self) -> RetrievalStats {
        RetrievalStats {
            total_documents: self.documents.len(),
            benchmark_centroids: self.benchmark_centroids.len(),
            embedding_dim: self.config.embedding_dim,
        }
    }
}

/// Retrieval statistics
#[derive(Debug, Clone)]
pub struct RetrievalStats {
    pub total_documents: usize,
    pub benchmark_centroids: usize,
    pub embedding_dim: usize,
}

/// Cosine similarity
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

/// Integration wrapper for existing RAG systems
pub struct BenchmarkAwareRAG {
    retriever: BenchmarkWeightedRetriever,
    /// Current benchmark context
    current_context: BenchmarkContext,
}

impl BenchmarkAwareRAG {
    pub fn new(retriever: BenchmarkWeightedRetriever) -> Self {
        Self {
            retriever,
            current_context: BenchmarkContext::General,
        }
    }

    /// Set current benchmark context
    pub fn set_context(&mut self, task: &str) {
        self.current_context = BenchmarkContext::from_task(task);
    }

    /// Retrieve with benchmark weighting
    pub fn retrieve(&self, query: &str) -> Vec<BenchmarkWeightedDocument> {
        self.retriever.retrieve(query, self.current_context)
    }

    /// Get current context
    pub fn current_context(&self) -> BenchmarkContext {
        self.current_context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_context_detection() {
        assert_eq!(BenchmarkContext::from_task("mmlu_physics"), BenchmarkContext::MMLU);
        assert_eq!(BenchmarkContext::from_task("arc_challenge"), BenchmarkContext::ARCChallenge);
        assert_eq!(BenchmarkContext::from_task("hellaswag"), BenchmarkContext::HellaSwag);
    }

    #[test]
    fn test_retriever_initialization() {
        let mut retriever = BenchmarkWeightedRetriever::new(
            BenchmarkWeightedRetrievalConfig::default()
        );
        
        let mut examples = HashMap::new();
        examples.insert(BenchmarkContext::MMLU, vec![
            "What is the capital of France? Paris".to_string(),
            "Who wrote Hamlet? Shakespeare".to_string(),
        ]);
        
        retriever.initialize_benchmarks(examples);
        assert!(retriever.benchmark_centroids.contains_key(&BenchmarkContext::MMLU));
    }

    #[test]
    fn test_document_retrieval() {
        let config = BenchmarkWeightedRetrievalConfig::default();
        let mut retriever = BenchmarkWeightedRetriever::new(config);
        
        // Initialize benchmark
        let mut examples = HashMap::new();
        examples.insert(BenchmarkContext::MMLU, vec![
            "Science question about physics".to_string(),
        ]);
        retriever.initialize_benchmarks(examples);
        
        // Add documents
        retriever.add_documents(vec![
            ("Physics is the study of matter".to_string(), "wiki".to_string()),
            ("Cooking recipes for beginners".to_string(), "recipes".to_string()),
        ]);
        
        // Retrieve
        let results = retriever.retrieve("What is physics?", BenchmarkContext::MMLU);
        assert!(!results.is_empty());
        assert!(results[0].combined_score > 0.0);
    }
}
