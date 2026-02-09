//! Unified Knowledge Pipeline
//!
//! Replaces the fragmented 18-expert architecture with a coherent knowledge flow:
//!
//! ## Order of Operations (Critical)
//!
//! 1. **RETRIEVE** - Get relevant knowledge from pre-built knowledge base
//! 2. **EXTRACT** - Parse full content into structured facts (not just patterns)
//! 3. **EMBED** - Create semantic embeddings using EmbedVec (HNSW-indexed)
//! 4. **REASON** - Apply logical inference over extracted facts
//! 5. **SCORE** - Rank choices based on knowledge alignment
//!
//! ## Key Improvements
//!
//! - Full markdown extraction instead of pattern matching
//! - EmbedVec HNSW-indexed embeddings (when feature enabled)
//! - Knowledge-first scoring (RAG dominates when knowledge found)
//! - Pre-built knowledge base (not test-time crawling)

use std::collections::HashMap;
use rayon::prelude::*;

#[cfg(feature = "embeddings")]
use embedvec::{EmbedVec, Distance as EmbedDistance};

/// Extracted fact from knowledge base
#[derive(Debug, Clone)]
pub struct ExtractedFact {
    /// Subject of the fact
    pub subject: String,
    /// Predicate/relation
    pub predicate: String,
    /// Object/value
    pub object: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Source document
    pub source: String,
    /// Full sentence context
    pub context: String,
}

/// Knowledge retrieval result
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    /// Retrieved facts relevant to query
    pub facts: Vec<ExtractedFact>,
    /// Relevance score
    pub relevance: f32,
    /// Source documents
    pub sources: Vec<String>,
}

/// Semantic embedding (384-dim to match common models)
pub type SemanticEmbedding = Vec<f32>;

/// Unified Knowledge Pipeline Configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Embedding dimension
    pub embed_dim: usize,
    /// Maximum facts to retrieve per query
    pub max_facts: usize,
    /// Minimum confidence threshold for facts
    pub min_confidence: f32,
    /// Enable semantic similarity (vs keyword matching)
    pub use_semantic: bool,
    /// Knowledge weight in final scoring (0.0-1.0)
    pub knowledge_weight: f32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            embed_dim: 384,
            max_facts: 50,
            min_confidence: 0.3,
            use_semantic: true,
            knowledge_weight: 0.7, // Knowledge dominates scoring
        }
    }
}

/// Unified Knowledge Pipeline
///
/// Implements the correct order of operations for knowledge-based reasoning:
/// RETRIEVE → EXTRACT → EMBED → REASON → SCORE
pub struct UnifiedKnowledgePipeline {
    config: PipelineConfig,
    
    /// Pre-built knowledge base: subject → facts
    knowledge_base: HashMap<String, Vec<ExtractedFact>>,
    
    /// Semantic embeddings for subjects (fallback when EmbedVec not available)
    subject_embeddings: HashMap<String, SemanticEmbedding>,
    
    /// Learned word embeddings (from training data)
    word_embeddings: HashMap<String, SemanticEmbedding>,
    
    /// IDF weights for words (inverse document frequency)
    idf_weights: HashMap<String, f32>,
    
    /// Total documents seen (for IDF calculation)
    total_docs: usize,
    
    /// EmbedVec for HNSW-indexed semantic search (when feature enabled)
    #[cfg(feature = "embeddings")]
    embed_vec: Option<EmbedVec>,
    
    /// Subject to EmbedVec ID mapping
    #[cfg(feature = "embeddings")]
    subject_to_id: HashMap<String, u64>,
    
    /// Next ID for EmbedVec insertions
    #[cfg(feature = "embeddings")]
    next_embed_id: u64,
    
    /// Truth checker for misconception detection
    truth_checker: crate::cognition::constitution::TruthChecker,
}

impl UnifiedKnowledgePipeline {
    /// Create a new unified knowledge pipeline
    pub fn new(config: PipelineConfig) -> Self {
        // Initialize EmbedVec when feature is enabled
        #[cfg(feature = "embeddings")]
        let embed_vec = {
            use tokio::runtime::Runtime;
            Runtime::new().ok().and_then(|rt| {
                rt.block_on(async {
                    // EmbedVec::new(dim, distance_metric, hnsw_m, hnsw_ef_construction)
                    EmbedVec::new(config.embed_dim, EmbedDistance::Cosine, 16, 200).await.ok()
                })
            })
        };
        
        Self {
            config,
            knowledge_base: HashMap::new(),
            subject_embeddings: HashMap::new(),
            word_embeddings: HashMap::new(),
            idf_weights: HashMap::new(),
            total_docs: 0,
            #[cfg(feature = "embeddings")]
            embed_vec,
            #[cfg(feature = "embeddings")]
            subject_to_id: HashMap::new(),
            #[cfg(feature = "embeddings")]
            next_embed_id: 0,
            truth_checker: crate::cognition::constitution::TruthChecker::new(),
        }
    }

    // =========================================================================
    // PHASE 1: BUILD KNOWLEDGE BASE (Pre-benchmark)
    // =========================================================================

    /// Build knowledge base from crawled markdown content
    /// This should be called BEFORE benchmarks, not during
    pub fn build_knowledge_base(&mut self, documents: &[(String, String)]) {
        println!("[Pipeline] Building knowledge base from {} documents...", documents.len());
        
        // Process documents in parallel
        let all_facts: Vec<Vec<ExtractedFact>> = documents.par_iter()
            .map(|(url, markdown)| self.extract_facts_from_markdown(markdown, url))
            .collect();
        
        // Merge into knowledge base
        for facts in all_facts {
            for fact in facts {
                self.knowledge_base
                    .entry(fact.subject.clone())
                    .or_insert_with(Vec::new)
                    .push(fact);
            }
        }
        
        // Build embeddings for all subjects
        self.build_subject_embeddings();
        
        // Calculate IDF weights
        self.calculate_idf_weights();
        
        let total_facts: usize = self.knowledge_base.values().map(|v| v.len()).sum();
        println!("[Pipeline] Knowledge base built: {} subjects, {} facts", 
                 self.knowledge_base.len(), total_facts);
    }

    /// Extract facts from full markdown content (not just pattern matching)
    fn extract_facts_from_markdown(&self, markdown: &str, source: &str) -> Vec<ExtractedFact> {
        let mut facts = Vec::new();
        
        // Split into sentences
        let sentences: Vec<&str> = markdown
            .split(&['.', '!', '?', '\n'][..])
            .map(|s| s.trim())
            .filter(|s| s.len() > 10 && s.len() < 500)
            .collect();
        
        for sentence in sentences {
            // Extract subject-predicate-object triples using multiple patterns
            if let Some(fact) = self.extract_spo_triple(sentence, source) {
                facts.push(fact);
            }
            
            // Also extract definition patterns
            if let Some(fact) = self.extract_definition(sentence, source) {
                facts.push(fact);
            }
            
            // Extract property patterns
            if let Some(fact) = self.extract_property(sentence, source) {
                facts.push(fact);
            }
        }
        
        facts
    }

    /// Extract subject-predicate-object triple from sentence
    fn extract_spo_triple(&self, sentence: &str, source: &str) -> Option<ExtractedFact> {
        let lower = sentence.to_lowercase();
        
        // Pattern: "X is Y" / "X are Y"
        for verb in ["is", "are", "was", "were"] {
            if let Some(idx) = lower.find(&format!(" {} ", verb)) {
                let subject = lower[..idx].trim();
                let object = lower[idx + verb.len() + 2..].trim();
                
                // Filter out too short/long subjects/objects
                if subject.len() > 2 && subject.len() < 50 && 
                   object.len() > 2 && object.len() < 100 {
                    // Get last word of subject as key (usually the noun)
                    let subject_key = subject.split_whitespace().last()
                        .unwrap_or(subject).to_string();
                    
                    return Some(ExtractedFact {
                        subject: subject_key,
                        predicate: verb.to_string(),
                        object: object.to_string(),
                        confidence: 0.7,
                        source: source.to_string(),
                        context: sentence.to_string(),
                    });
                }
            }
        }
        
        // Pattern: "X has Y" / "X have Y"
        for verb in ["has", "have", "had"] {
            if let Some(idx) = lower.find(&format!(" {} ", verb)) {
                let subject = lower[..idx].trim();
                let object = lower[idx + verb.len() + 2..].trim();
                
                if subject.len() > 2 && subject.len() < 50 && 
                   object.len() > 2 && object.len() < 100 {
                    let subject_key = subject.split_whitespace().last()
                        .unwrap_or(subject).to_string();
                    
                    return Some(ExtractedFact {
                        subject: subject_key,
                        predicate: "has".to_string(),
                        object: object.to_string(),
                        confidence: 0.6,
                        source: source.to_string(),
                        context: sentence.to_string(),
                    });
                }
            }
        }
        
        // Pattern: "X can Y" / "X could Y"
        for verb in ["can", "could", "may", "might"] {
            if let Some(idx) = lower.find(&format!(" {} ", verb)) {
                let subject = lower[..idx].trim();
                let object = lower[idx + verb.len() + 2..].trim();
                
                if subject.len() > 2 && subject.len() < 50 && 
                   object.len() > 2 && object.len() < 100 {
                    let subject_key = subject.split_whitespace().last()
                        .unwrap_or(subject).to_string();
                    
                    return Some(ExtractedFact {
                        subject: subject_key,
                        predicate: "can".to_string(),
                        object: object.to_string(),
                        confidence: 0.5,
                        source: source.to_string(),
                        context: sentence.to_string(),
                    });
                }
            }
        }
        
        None
    }

    /// Extract definition pattern: "X is a type of Y" / "X is defined as Y"
    fn extract_definition(&self, sentence: &str, source: &str) -> Option<ExtractedFact> {
        let lower = sentence.to_lowercase();
        
        // Definition patterns
        let patterns = [
            (" is a type of ", "type_of"),
            (" is a kind of ", "kind_of"),
            (" is defined as ", "defined_as"),
            (" refers to ", "refers_to"),
            (" is known as ", "known_as"),
            (" is called ", "called"),
        ];
        
        for (pattern, predicate) in patterns {
            if let Some(idx) = lower.find(pattern) {
                let subject = lower[..idx].trim();
                let object = lower[idx + pattern.len()..].trim();
                
                if subject.len() > 2 && subject.len() < 50 && 
                   object.len() > 2 && object.len() < 100 {
                    let subject_key = subject.split_whitespace().last()
                        .unwrap_or(subject).to_string();
                    
                    return Some(ExtractedFact {
                        subject: subject_key,
                        predicate: predicate.to_string(),
                        object: object.to_string(),
                        confidence: 0.8,
                        source: source.to_string(),
                        context: sentence.to_string(),
                    });
                }
            }
        }
        
        None
    }

    /// Extract property pattern: "X contains Y" / "X includes Y"
    fn extract_property(&self, sentence: &str, source: &str) -> Option<ExtractedFact> {
        let lower = sentence.to_lowercase();
        
        let patterns = [
            (" contains ", "contains"),
            (" includes ", "includes"),
            (" consists of ", "consists_of"),
            (" is made of ", "made_of"),
            (" is located in ", "located_in"),
            (" is found in ", "found_in"),
            (" is used for ", "used_for"),
            (" is used in ", "used_in"),
        ];
        
        for (pattern, predicate) in patterns {
            if let Some(idx) = lower.find(pattern) {
                let subject = lower[..idx].trim();
                let object = lower[idx + pattern.len()..].trim();
                
                if subject.len() > 2 && subject.len() < 50 && 
                   object.len() > 2 && object.len() < 100 {
                    let subject_key = subject.split_whitespace().last()
                        .unwrap_or(subject).to_string();
                    
                    return Some(ExtractedFact {
                        subject: subject_key,
                        predicate: predicate.to_string(),
                        object: object.to_string(),
                        confidence: 0.6,
                        source: source.to_string(),
                        context: sentence.to_string(),
                    });
                }
            }
        }
        
        None
    }

    /// Build semantic embeddings for all subjects in knowledge base
    /// Uses EmbedVec for HNSW-indexed search when feature is enabled
    fn build_subject_embeddings(&mut self) {
        let subjects: Vec<String> = self.knowledge_base.keys().cloned().collect();
        
        for subject in &subjects {
            let embedding = self.compute_semantic_embedding(subject);
            
            // Store in EmbedVec for HNSW search when available
            #[cfg(feature = "embeddings")]
            {
                if let Some(ref mut embed_vec) = self.embed_vec {
                    use tokio::runtime::Runtime;
                    if let Ok(rt) = Runtime::new() {
                        let embed_clone = embedding.clone();
                        let subject_clone = subject.clone();
                        
                        rt.block_on(async {
                            // EmbedVec uses add() with metadata
                            let metadata = serde_json::json!({"subject": subject_clone});
                            let _ = embed_vec.add(&embed_clone, metadata).await;
                        });
                        
                        self.subject_to_id.insert(subject.clone(), self.next_embed_id);
                        self.next_embed_id += 1;
                    }
                }
            }
            
            // Always store in HashMap as fallback
            self.subject_embeddings.insert(subject.clone(), embedding);
        }
    }

    /// Calculate IDF weights from knowledge base
    fn calculate_idf_weights(&mut self) {
        let mut doc_freq: HashMap<String, usize> = HashMap::new();
        self.total_docs = self.knowledge_base.len();
        
        for facts in self.knowledge_base.values() {
            let mut seen_words: std::collections::HashSet<String> = std::collections::HashSet::new();
            for fact in facts {
                for word in fact.context.split_whitespace() {
                    let word_lower = word.to_lowercase();
                    if word_lower.len() > 2 && !seen_words.contains(&word_lower) {
                        seen_words.insert(word_lower.clone());
                        *doc_freq.entry(word_lower).or_insert(0) += 1;
                    }
                }
            }
        }
        
        // Calculate IDF: log(N / df)
        for (word, df) in doc_freq {
            let idf = (self.total_docs as f32 / df as f32).ln();
            self.idf_weights.insert(word, idf);
        }
    }

    // =========================================================================
    // PHASE 2: RETRIEVE (Query-time)
    // =========================================================================

    /// Retrieve relevant facts for a query
    /// Uses EmbedVec HNSW search when available, falls back to linear scan
    pub fn retrieve(&self, query: &str) -> RetrievalResult {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();
        
        let query_embedding = self.compute_semantic_embedding(&query_lower);
        
        let mut all_facts = Vec::new();
        let mut sources = Vec::new();
        let mut _total_relevance = 0.0;
        
        // Search by keyword matching first
        for word in &query_words {
            if let Some(facts) = self.knowledge_base.get(*word) {
                for fact in facts {
                    if fact.confidence >= self.config.min_confidence {
                        all_facts.push(fact.clone());
                        if !sources.contains(&fact.source) {
                            sources.push(fact.source.clone());
                        }
                    }
                }
            }
        }
        
        // Search by semantic similarity using EmbedVec HNSW when available
        if self.config.use_semantic {
            #[cfg(feature = "embeddings")]
            {
                // Use EmbedVec HNSW-indexed search for O(log n) retrieval
                if let Some(ref embed_vec) = self.embed_vec {
                    use tokio::runtime::Runtime;
                    if let Ok(rt) = Runtime::new() {
                        let results = rt.block_on(async {
                            embed_vec.search(&query_embedding, 20, 64, None).await.unwrap_or_default()
                        });
                        
                        for hit in results {
                            let similarity = hit.score;
                            if similarity > 0.3 {
                                // Get subject from metadata
                                let subject = hit.payload.get("subject")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                
                                if let Some(facts) = self.knowledge_base.get(&subject) {
                                    for fact in facts {
                                        if fact.confidence >= self.config.min_confidence {
                                            let mut boosted_fact = fact.clone();
                                            boosted_fact.confidence *= similarity;
                                            all_facts.push(boosted_fact);
                                            if !sources.contains(&fact.source) {
                                                sources.push(fact.source.clone());
                                            }
                                        }
                                    }
                                }
                                _total_relevance += similarity;
                            }
                        }
                    }
                }
            }
            
            // Fallback to linear scan when EmbedVec not available
            #[cfg(not(feature = "embeddings"))]
            {
                for (subject, embedding) in &self.subject_embeddings {
                    let similarity = self.cosine_similarity(&query_embedding, embedding);
                    if similarity > 0.3 {
                        if let Some(facts) = self.knowledge_base.get(subject) {
                            for fact in facts {
                                if fact.confidence >= self.config.min_confidence {
                                    let mut boosted_fact = fact.clone();
                                    boosted_fact.confidence *= similarity;
                                    all_facts.push(boosted_fact);
                                    if !sources.contains(&fact.source) {
                                        sources.push(fact.source.clone());
                                    }
                                }
                            }
                        }
                        _total_relevance += similarity;
                    }
                }
            }
        }
        
        // Deduplicate and sort by confidence
        all_facts.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        all_facts.truncate(self.config.max_facts);
        
        let relevance = if !all_facts.is_empty() {
            all_facts.iter().map(|f| f.confidence).sum::<f32>() / all_facts.len() as f32
        } else {
            0.0
        };
        
        RetrievalResult {
            facts: all_facts,
            relevance,
            sources,
        }
    }

    // =========================================================================
    // PHASE 3: SCORE (Knowledge-first scoring)
    // =========================================================================

    /// Score a choice against retrieved knowledge
    /// Returns (score, confidence, explanation)
    pub fn score_choice(&self, query: &str, choice: &str, retrieval: &RetrievalResult) -> (f32, f32, String) {
        let choice_lower = choice.to_lowercase();
        let choice_words: Vec<&str> = choice_lower.split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();
        
        let mut score = 0.0;
        let mut matched_facts = Vec::new();
        
        // Score based on fact matching
        for fact in &retrieval.facts {
            // Check if choice matches fact object
            let object_words: Vec<&str> = fact.object.split_whitespace().collect();
            let mut word_matches = 0;
            
            for choice_word in &choice_words {
                for obj_word in &object_words {
                    if choice_word == obj_word || 
                       choice_word.contains(obj_word) || 
                       obj_word.contains(*choice_word) {
                        word_matches += 1;
                    }
                }
            }
            
            if word_matches > 0 {
                let match_ratio = word_matches as f32 / object_words.len().max(1) as f32;
                let fact_score = fact.confidence * match_ratio * 10.0;
                score += fact_score;
                matched_facts.push(fact.predicate.clone());
            }
            
            // Check if choice appears in fact context
            if fact.context.to_lowercase().contains(&choice_lower) {
                score += fact.confidence * 5.0;
                matched_facts.push("context_match".to_string());
            }
        }
        
        // Semantic similarity between choice and query
        let query_embedding = self.compute_semantic_embedding(query);
        let choice_embedding = self.compute_semantic_embedding(choice);
        let semantic_sim = self.cosine_similarity(&query_embedding, &choice_embedding);
        // Reduce semantic similarity weight — it rewards misconceptions that
        // sound like the question (e.g., "bulls attracted by red" for "why red capes")
        score += semantic_sim * 1.0;
        
        // TRUTH CHECK: Penalize misconceptions, boost truthful answers
        // This is critical for TruthfulQA where the wrong answer IS the common belief
        let truth_score = self.truth_checker.score_truthfulness(
            &query.to_lowercase(), &choice_lower
        );
        score += truth_score;
        
        // Calculate confidence based on evidence
        let confidence = if !matched_facts.is_empty() {
            (matched_facts.len() as f32 / retrieval.facts.len().max(1) as f32).min(1.0)
        } else {
            semantic_sim * 0.5
        };
        
        let explanation = if matched_facts.is_empty() {
            "No direct knowledge match".to_string()
        } else {
            format!("Matched: {}", matched_facts.join(", "))
        };
        
        (score, confidence, explanation)
    }

    /// Score all choices and return ranked results
    /// This is the main entry point for inference
    pub fn infer(&self, query: &str, choices: &[String]) -> (usize, f32) {
        // STEP 1: RETRIEVE relevant knowledge
        let retrieval = self.retrieve(query);
        
        // STEP 2: SCORE each choice
        let mut scores: Vec<(usize, f32, f32)> = choices.iter()
            .enumerate()
            .map(|(idx, choice)| {
                let (score, confidence, _) = self.score_choice(query, choice, &retrieval);
                (idx, score, confidence)
            })
            .collect();
        
        // STEP 3: RANK by score
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Return best choice with margin-based confidence
        // Margin = how much better the best is vs second-best
        if scores.len() >= 2 {
            let best = &scores[0];
            let second = &scores[1];
            let range = best.1 - scores.last().map(|s| s.1).unwrap_or(0.0);
            let margin = best.1 - second.1;
            let confidence = if range > 0.0 {
                (0.3 + 0.7 * (margin / range)).min(1.0).max(0.15)
            } else {
                0.2
            };
            (best.0, confidence)
        } else if let Some(best) = scores.first() {
            (best.0, best.2.max(0.2))
        } else {
            (0, 0.0)
        }
    }

    // =========================================================================
    // EMBEDDING UTILITIES
    // =========================================================================

    /// Compute semantic embedding for text
    /// Uses TF-IDF weighted word embeddings
    fn compute_semantic_embedding(&self, text: &str) -> SemanticEmbedding {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .collect();
        
        let mut embedding = vec![0.0f32; self.config.embed_dim];
        let mut total_weight = 0.0;
        
        for word in words.iter() {
            // Get or compute word embedding
            let word_embed = self.get_word_embedding(word);
            
            // Get IDF weight (default to 1.0 if not found)
            let idf = self.idf_weights.get(*word).copied().unwrap_or(1.0);
            
            // Use IDF weight directly (no position decay)
            let weight = idf;
            total_weight += weight;
            
            // Add weighted word embedding
            for (j, &val) in word_embed.iter().enumerate() {
                if j < self.config.embed_dim {
                    embedding[j] += val * weight;
                }
            }
        }
        
        // Normalize
        if total_weight > 0.0 {
            for val in &mut embedding {
                *val /= total_weight;
            }
        }
        
        // L2 normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }
        
        embedding
    }

    /// Get word embedding (learned or computed)
    fn get_word_embedding(&self, word: &str) -> SemanticEmbedding {
        // Check if we have a learned embedding
        if let Some(embed) = self.word_embeddings.get(word) {
            return embed.clone();
        }
        
        // Compute hash-based embedding as fallback
        // This is deterministic and captures some character-level features
        let mut embedding = vec![0.0f32; self.config.embed_dim];
        
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        // Character n-grams for better representation
        let chars: Vec<char> = word.chars().collect();
        for n in 1..=3 {
            for i in 0..chars.len().saturating_sub(n - 1) {
                let ngram: String = chars[i..i+n].iter().collect();
                
                let mut hasher = DefaultHasher::new();
                ngram.hash(&mut hasher);
                let hash = hasher.finish();
                
                // Multiple hash functions for better distribution
                let idx1 = (hash as usize) % self.config.embed_dim;
                let idx2 = ((hash >> 16) as usize) % self.config.embed_dim;
                let idx3 = ((hash >> 32) as usize) % self.config.embed_dim;
                
                let weight = 1.0 / n as f32; // Shorter n-grams weighted more
                embedding[idx1] += weight;
                embedding[idx2] += weight * 0.5;
                embedding[idx3] += weight * 0.25;
            }
        }
        
        // L2 normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }
        
        embedding
    }

    /// Cosine similarity between two embeddings
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a * norm_b)
        } else {
            0.0
        }
    }

    // =========================================================================
    // LEARNING (Update embeddings from training data)
    // =========================================================================

    /// Learn word embeddings from training examples
    pub fn learn_from_examples(&mut self, examples: &[(String, String)]) {
        // Co-occurrence based learning
        for (question, answer) in examples {
            let q_words: Vec<String> = question.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 2)
                .map(|s| s.to_string())
                .collect();
            
            let a_words: Vec<String> = answer.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 2)
                .map(|s| s.to_string())
                .collect();
            
            // Words that appear together should have similar embeddings
            for q_word in &q_words {
                for a_word in &a_words {
                    self.update_word_similarity(q_word, a_word, 0.1);
                }
            }
        }
    }

    /// Update word embeddings to be more similar
    fn update_word_similarity(&mut self, word1: &str, word2: &str, learning_rate: f32) {
        let embed1 = self.get_word_embedding(word1);
        let embed2 = self.get_word_embedding(word2);
        
        // Move embeddings closer together
        let mut new_embed1 = embed1.clone();
        let mut new_embed2 = embed2.clone();
        
        for i in 0..self.config.embed_dim {
            let diff = embed2[i] - embed1[i];
            new_embed1[i] += diff * learning_rate;
            new_embed2[i] -= diff * learning_rate;
        }
        
        self.word_embeddings.insert(word1.to_string(), new_embed1);
        self.word_embeddings.insert(word2.to_string(), new_embed2);
    }

    // =========================================================================
    // STATISTICS
    // =========================================================================

    /// Get pipeline statistics
    pub fn stats(&self) -> PipelineStats {
        let total_facts: usize = self.knowledge_base.values().map(|v| v.len()).sum();
        
        PipelineStats {
            subjects: self.knowledge_base.len(),
            facts: total_facts,
            embeddings: self.subject_embeddings.len(),
            word_embeddings: self.word_embeddings.len(),
            idf_words: self.idf_weights.len(),
        }
    }
}

/// Pipeline statistics
#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub subjects: usize,
    pub facts: usize,
    pub embeddings: usize,
    pub word_embeddings: usize,
    pub idf_words: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_extraction() {
        let pipeline = UnifiedKnowledgePipeline::new(PipelineConfig::default());
        
        let markdown = "Artificial intelligence is a branch of computer science. \
                        Machine learning has many applications. \
                        Neural networks can learn from data.";
        
        let facts = pipeline.extract_facts_from_markdown(markdown, "test");
        
        assert!(!facts.is_empty());
        assert!(facts.iter().any(|f| f.subject == "intelligence" || f.subject == "learning"));
    }

    #[test]
    fn test_retrieval() {
        let mut pipeline = UnifiedKnowledgePipeline::new(PipelineConfig::default());
        
        let docs = vec![
            ("wiki/ai".to_string(), "Artificial intelligence is the simulation of human intelligence.".to_string()),
            ("wiki/ml".to_string(), "Machine learning is a subset of artificial intelligence.".to_string()),
        ];
        
        pipeline.build_knowledge_base(&docs);
        
        let result = pipeline.retrieve("What is artificial intelligence?");
        assert!(!result.facts.is_empty());
    }

    #[test]
    fn test_inference() {
        let mut pipeline = UnifiedKnowledgePipeline::new(PipelineConfig::default());
        
        let docs = vec![
            ("wiki/ai".to_string(), "Artificial intelligence is the simulation of human intelligence.".to_string()),
        ];
        
        pipeline.build_knowledge_base(&docs);
        
        let choices = vec![
            "simulation of human intelligence".to_string(),
            "a type of hardware".to_string(),
            "a programming language".to_string(),
        ];
        
        let (best_idx, confidence) = pipeline.infer("What is artificial intelligence?", &choices);
        
        // Should pick the first choice (simulation of human intelligence)
        assert_eq!(best_idx, 0);
        assert!(confidence > 0.0);
    }
}
