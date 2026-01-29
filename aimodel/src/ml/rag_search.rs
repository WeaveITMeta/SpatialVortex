//! RAG Search Integration for AI Model
//!
//! Provides external knowledge retrieval for benchmark questions
//! using vector store lookups with MMR diversity.
//!
//! Architecture (inspired by SpatialVortex 3-Stage RAG):
//! 1. Knowledge base retrieval → topK candidates
//! 2. MMR reranking → diverse, relevant results
//! 3. Hierarchical context integration

use std::collections::HashMap;

/// RAG Search Configuration
#[derive(Debug, Clone)]
pub struct RAGSearchConfig {
    /// Maximum results to retrieve
    pub max_results: usize,
    /// Minimum relevance score threshold
    pub min_relevance: f32,
    /// Enable web search fallback
    pub enable_web_search: bool,
    /// Cache search results
    pub enable_cache: bool,
    /// MMR lambda: 0.0 = max diversity, 1.0 = max relevance
    pub mmr_lambda: f32,
    /// High relevance threshold
    pub high_relevance_threshold: f32,
    /// Medium relevance threshold
    pub medium_relevance_threshold: f32,
}

impl Default for RAGSearchConfig {
    fn default() -> Self {
        Self {
            max_results: 5,
            min_relevance: 0.3,
            enable_web_search: true,
            enable_cache: true,
            mmr_lambda: 0.7,  // Balance relevance and diversity
            high_relevance_threshold: 0.6,
            medium_relevance_threshold: 0.4,
        }
    }
}

/// Relevance level for hierarchical context integration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelevanceLevel {
    High,    // > 0.6: Include full content
    Medium,  // > 0.4: Include summary
    Low,     // < 0.4: Include key points only
}

/// Retrieved context from RAG search
#[derive(Debug, Clone)]
pub struct RetrievedContext {
    pub content: String,
    pub source: String,
    pub relevance: f32,
    pub embedding: Vec<f32>,
    pub relevance_level: RelevanceLevel,
}

/// RAG Search Engine for external knowledge retrieval
pub struct RAGSearchEngine {
    config: RAGSearchConfig,
    /// Local knowledge base (word -> facts)
    knowledge_base: HashMap<String, Vec<String>>,
    /// Cached search results
    cache: HashMap<String, Vec<RetrievedContext>>,
    /// Word embeddings for semantic search
    word_embeddings: HashMap<String, Vec<f32>>,
    /// N-gram frequencies learned from training (for IDF weighting)
    ngram_frequencies: HashMap<String, usize>,
}

impl RAGSearchEngine {
    pub fn new(config: RAGSearchConfig) -> Self {
        let mut engine = Self {
            config,
            knowledge_base: HashMap::new(),
            cache: HashMap::new(),
            word_embeddings: HashMap::new(),
            ngram_frequencies: HashMap::new(),
        };
        
        // Initialize with commonsense knowledge
        engine.load_commonsense_knowledge();
        engine
    }
    
    /// Load basic commonsense knowledge for CommonsenseQA-style questions
    fn load_commonsense_knowledge(&mut self) {
        // Location/Place knowledge
        self.add_knowledge("bank", vec![
            "A bank is a financial institution that accepts deposits.",
            "A river bank is the land alongside a river.",
            "Banks are found in towns and cities.",
        ]);
        
        self.add_knowledge("store", vec![
            "A store is a place where goods are sold.",
            "Stores are typically found in shopping centers or on streets.",
            "People go to stores to buy things.",
        ]);
        
        self.add_knowledge("kitchen", vec![
            "A kitchen is a room where food is prepared.",
            "Kitchens contain appliances like stoves and refrigerators.",
            "Cooking happens in the kitchen.",
        ]);
        
        self.add_knowledge("bathroom", vec![
            "A bathroom is a room with a toilet and sink.",
            "People wash and bathe in bathrooms.",
            "Bathrooms are found in homes and buildings.",
        ]);
        
        self.add_knowledge("office", vec![
            "An office is a place where people work.",
            "Offices contain desks and computers.",
            "Business activities happen in offices.",
        ]);
        
        self.add_knowledge("school", vec![
            "A school is a place where students learn.",
            "Schools have classrooms and teachers.",
            "Education happens at schools.",
        ]);
        
        self.add_knowledge("hospital", vec![
            "A hospital is where sick people receive medical care.",
            "Doctors and nurses work in hospitals.",
            "Medical treatments are given at hospitals.",
        ]);
        
        // Action/Activity knowledge
        self.add_knowledge("eat", vec![
            "Eating is consuming food for nutrition.",
            "People eat when they are hungry.",
            "Eating typically happens at mealtimes.",
        ]);
        
        self.add_knowledge("sleep", vec![
            "Sleep is a state of rest for the body and mind.",
            "People sleep at night in beds.",
            "Sleep is necessary for health.",
        ]);
        
        self.add_knowledge("work", vec![
            "Work is activity done to earn money or achieve goals.",
            "People work at jobs or on tasks.",
            "Work requires effort and time.",
        ]);
        
        self.add_knowledge("play", vec![
            "Play is recreational activity for enjoyment.",
            "Children play with toys and games.",
            "Play is important for development.",
        ]);
        
        // Object knowledge
        self.add_knowledge("water", vec![
            "Water is a liquid essential for life.",
            "Water is used for drinking, washing, and cooking.",
            "Water is found in rivers, lakes, and oceans.",
        ]);
        
        self.add_knowledge("food", vec![
            "Food provides nutrition and energy.",
            "Food is eaten to satisfy hunger.",
            "Food is prepared in kitchens.",
        ]);
        
        self.add_knowledge("money", vec![
            "Money is used to buy goods and services.",
            "Money is stored in banks.",
            "People earn money by working.",
        ]);
        
        // Relationship knowledge
        self.add_knowledge("friend", vec![
            "A friend is someone you like and trust.",
            "Friends spend time together.",
            "Friendship involves mutual care.",
        ]);
        
        self.add_knowledge("family", vec![
            "Family includes parents, children, and relatives.",
            "Families live together in homes.",
            "Family members care for each other.",
        ]);
        
        // Emotion knowledge
        self.add_knowledge("happy", vec![
            "Happy is a positive emotional state.",
            "People feel happy when good things happen.",
            "Happiness comes from satisfaction and joy.",
        ]);
        
        self.add_knowledge("sad", vec![
            "Sad is a negative emotional state.",
            "People feel sad when bad things happen.",
            "Sadness comes from loss or disappointment.",
        ]);
        
        self.add_knowledge("angry", vec![
            "Angry is an emotional response to frustration.",
            "People feel angry when wronged.",
            "Anger can lead to conflict.",
        ]);
        
        // Cause-effect knowledge
        self.add_knowledge("rain", vec![
            "Rain is water falling from clouds.",
            "Rain makes things wet.",
            "Rain is needed for plants to grow.",
        ]);
        
        self.add_knowledge("fire", vec![
            "Fire produces heat and light.",
            "Fire can burn things.",
            "Fire is used for cooking and warmth.",
        ]);
        
        self.add_knowledge("cold", vec![
            "Cold is low temperature.",
            "Cold weather requires warm clothing.",
            "Ice forms when water is cold.",
        ]);
        
        self.add_knowledge("hot", vec![
            "Hot is high temperature.",
            "Hot things can burn.",
            "Summer is typically hot.",
        ]);
    }
    
    /// Add knowledge entries for a topic (internal)
    fn add_knowledge(&mut self, topic: &str, facts: Vec<&str>) {
        self.knowledge_base.insert(
            topic.to_lowercase(),
            facts.iter().map(|s| s.to_string()).collect()
        );
    }
    
    /// Add knowledge entries for a topic (public API)
    pub fn add_knowledge_entry(&mut self, topic: &str, fact: &str) {
        self.knowledge_base
            .entry(topic.to_lowercase())
            .or_insert_with(Vec::new)
            .push(fact.to_string());
    }
    
    /// Bulk add knowledge from entity-attribute pairs
    pub fn import_entity_attributes(&mut self, entity_attrs: &std::collections::HashMap<String, std::collections::HashMap<String, f32>>) {
        for (entity, attrs) in entity_attrs {
            let facts: Vec<String> = attrs.iter()
                .filter(|(_, &score)| score > 0.5)  // Only high-confidence attributes
                .map(|(attr, _)| format!("{} is associated with {}", entity, attr))
                .collect();
            
            if !facts.is_empty() {
                let entry = self.knowledge_base.entry(entity.to_lowercase()).or_insert_with(Vec::new);
                for fact in facts {
                    if !entry.contains(&fact) {
                        entry.push(fact);
                    }
                }
            }
        }
    }
    
    /// Bulk add knowledge from causal patterns
    pub fn import_causal_patterns(&mut self, causal: &std::collections::HashMap<String, Vec<(String, f32)>>) {
        for (cause, effects) in causal {
            let facts: Vec<String> = effects.iter()
                .filter(|(_, score)| *score > 0.5)
                .map(|(effect, _)| format!("{} leads to {}", cause, effect))
                .collect();
            
            if !facts.is_empty() {
                let entry = self.knowledge_base.entry(cause.to_lowercase()).or_insert_with(Vec::new);
                for fact in facts {
                    if !entry.contains(&fact) {
                        entry.push(fact);
                    }
                }
            }
        }
    }
    
    /// Bulk add knowledge from Q&A patterns
    pub fn import_qa_patterns(&mut self, qa_patterns: &std::collections::HashMap<String, Vec<String>>) {
        for (pattern, answers) in qa_patterns {
            // Extract key topic from pattern
            let topic_words: Vec<&str> = pattern
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 3)
                .take(2)
                .collect();
            
            if let Some(topic) = topic_words.first() {
                let facts: Vec<String> = answers.iter()
                    .take(3)  // Limit facts per topic
                    .map(|a| format!("Answer: {}", a))
                    .collect();
                
                let entry = self.knowledge_base.entry(topic.to_lowercase()).or_insert_with(Vec::new);
                for fact in facts {
                    if !entry.contains(&fact) {
                        entry.push(fact);
                    }
                }
            }
        }
    }
    
    /// Get knowledge base size for diagnostics
    pub fn knowledge_size(&self) -> (usize, usize) {
        let topics = self.knowledge_base.len();
        let facts: usize = self.knowledge_base.values().map(|v| v.len()).sum();
        (topics, facts)
    }
    
    /// Get all facts from the knowledge base for CALM pretraining
    pub fn get_all_facts(&self) -> Vec<String> {
        self.knowledge_base.values()
            .flat_map(|facts| facts.iter().cloned())
            .collect()
    }
    
    /// Import implications from 369 sacred attention heads
    /// These are dynamic implications extracted by comparing node labels to attributes
    pub fn import_implications(&mut self, implications: &[(String, String, String, f32)]) {
        // implications format: (source_label, attribute_key, implication_type, strength)
        for (source, attr_key, impl_type, strength) in implications {
            if *strength < 0.3 {
                continue; // Skip weak implications
            }
            
            // Create fact from implication
            let fact = match impl_type.as_str() {
                "property" => format!("{} has property {}", source, attr_key),
                "causal" => format!("{} causes or leads to {}", source, attr_key),
                "temporal" => format!("{} occurs before/after {}", source, attr_key),
                "spatial" => format!("{} is located near/at {}", source, attr_key),
                "logical" => format!("{} implies {}", source, attr_key),
                "semantic" => format!("{} is related to {}", source, attr_key),
                "sacred_verification" => format!("{} verified at sacred position with {}", source, attr_key),
                _ => format!("{} is associated with {}", source, attr_key),
            };
            
            // Add to knowledge base with source as topic
            let entry = self.knowledge_base
                .entry(source.to_lowercase())
                .or_insert_with(Vec::new);
            if !entry.contains(&fact) {
                entry.push(fact.clone());
            }
            
            // Also index by attribute key
            let attr_entry = self.knowledge_base
                .entry(attr_key.to_lowercase())
                .or_insert_with(Vec::new);
            if !attr_entry.contains(&fact) {
                attr_entry.push(fact);
            }
        }
        
        // Clear cache since knowledge changed
        self.cache.clear();
    }
    
    /// Score a choice using implications as additional context
    /// This boosts choices that align with extracted implications
    pub fn score_with_implications(
        &self,
        question: &str,
        choice: &str,
        implications: &[(String, String, String, f32)],
    ) -> f32 {
        let question_lower = question.to_lowercase();
        let choice_lower = choice.to_lowercase();
        
        let mut score = 0.0f32;
        
        for (source, attr_key, impl_type, strength) in implications {
            // Check if choice relates to the implication
            let source_match = choice_lower.contains(&source.to_lowercase()) 
                || question_lower.contains(&source.to_lowercase());
            let attr_match = choice_lower.contains(&attr_key.to_lowercase());
            
            if source_match && attr_match {
                // Strong match - choice contains both source and attribute
                let type_weight = match impl_type.as_str() {
                    "causal" => 1.5,      // Causal implications are highly relevant
                    "property" => 1.2,    // Properties help identify correct answers
                    "logical" => 1.3,     // Logical implications support reasoning
                    "sacred_verification" => 1.4, // Verified at sacred positions
                    _ => 1.0,
                };
                score += strength * type_weight * 5.0;
            } else if attr_match {
                // Partial match - choice contains attribute
                score += strength * 2.0;
            }
        }
        
        score
    }
    
    /// Update n-gram frequencies from training data (for IDF weighting in embeddings)
    pub fn update_ngram_frequencies(&mut self, frequencies: &HashMap<String, usize>) {
        for (word, &count) in frequencies {
            *self.ngram_frequencies.entry(word.clone()).or_insert(0) += count;
        }
        // Clear cache since embeddings will change
        self.cache.clear();
    }
    
    /// Search for relevant context given a query
    pub fn search(&mut self, query: &str) -> Vec<RetrievedContext> {
        let query_lower = query.to_lowercase();
        
        // Check cache first
        if self.config.enable_cache {
            if let Some(cached) = self.cache.get(&query_lower) {
                return cached.clone();
            }
        }
        
        let mut results = Vec::new();
        
        // Extract keywords from query
        let keywords: Vec<&str> = query_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .collect();
        
        // Search knowledge base for each keyword
        for keyword in &keywords {
            if let Some(facts) = self.knowledge_base.get(*keyword) {
                for fact in facts {
                    // Direct keyword match gets high base relevance
                    let base_relevance = 0.5;
                    let text_relevance = self.compute_relevance(&query_lower, fact);
                    let relevance = base_relevance + text_relevance * 0.5;
                    
                    let level = self.get_relevance_level(relevance);
                    results.push(RetrievedContext {
                        content: fact.clone(),
                        source: format!("knowledge_base:{}", keyword),
                        relevance,
                        embedding: self.text_to_embedding(fact),
                        relevance_level: level,
                    });
                }
            }
        }
        
        // Also check for partial matches
        for (topic, facts) in &self.knowledge_base {
            if query_lower.contains(topic) {
                for fact in facts {
                    let relevance = self.compute_relevance(&query_lower, fact) * 0.8;
                    if relevance >= self.config.min_relevance {
                        // Avoid duplicates
                        if !results.iter().any(|r| r.content == *fact) {
                            let level = self.get_relevance_level(relevance);
                            results.push(RetrievedContext {
                                content: fact.clone(),
                                source: format!("knowledge_base:{}", topic),
                                relevance,
                                embedding: self.text_to_embedding(fact),
                                relevance_level: level,
                            });
                        }
                    }
                }
            }
        }
        
        // Apply MMR reranking for diversity
        results = self.apply_mmr_rerank(results);
        
        // Limit results
        results.truncate(self.config.max_results);
        
        // Cache results
        if self.config.enable_cache {
            self.cache.insert(query_lower, results.clone());
        }
        
        results
    }
    
    /// Compute relevance score between query and document
    fn compute_relevance(&self, query: &str, document: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let doc_lower = document.to_lowercase();
        
        let query_words: std::collections::HashSet<&str> = query_lower
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();
        
        let doc_words: std::collections::HashSet<&str> = doc_lower
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();
        
        // Jaccard similarity
        let intersection = query_words.intersection(&doc_words).count();
        let union = query_words.len() + doc_words.len() - intersection;
        
        if union > 0 {
            intersection as f32 / union as f32
        } else {
            0.0
        }
    }
    
    /// Get relevance level based on score thresholds
    fn get_relevance_level(&self, relevance: f32) -> RelevanceLevel {
        if relevance >= self.config.high_relevance_threshold {
            RelevanceLevel::High
        } else if relevance >= self.config.medium_relevance_threshold {
            RelevanceLevel::Medium
        } else {
            RelevanceLevel::Low
        }
    }
    
    /// Apply MMR (Max Marginal Relevance) reranking for diversity
    /// Inspired by SpatialVortex rag_engine.rs:299-358
    fn apply_mmr_rerank(&self, mut candidates: Vec<RetrievedContext>) -> Vec<RetrievedContext> {
        if candidates.len() <= 1 {
            return candidates;
        }
        
        let lambda = self.config.mmr_lambda;
        let mut selected: Vec<RetrievedContext> = Vec::new();
        let mut used = vec![false; candidates.len()];
        
        // Select top results using MMR
        let max_select = self.config.max_results.min(candidates.len());
        
        for _ in 0..max_select {
            let mut best_idx = None;
            let mut best_mmr = f32::NEG_INFINITY;
            
            for (i, candidate) in candidates.iter().enumerate() {
                if used[i] {
                    continue;
                }
                
                let relevance = candidate.relevance;
                
                // Compute max similarity to already selected docs
                let max_sim = selected.iter()
                    .map(|sel| self.embedding_similarity(&candidate.embedding, &sel.embedding))
                    .fold(0.0f32, |a, b| a.max(b));
                
                // MMR score: balance relevance and diversity
                let mmr = lambda * relevance - (1.0 - lambda) * max_sim;
                
                if mmr > best_mmr {
                    best_mmr = mmr;
                    best_idx = Some(i);
                }
            }
            
            if let Some(idx) = best_idx {
                used[idx] = true;
                selected.push(candidates[idx].clone());
            } else {
                break;
            }
        }
        
        selected
    }
    
    /// Compute cosine similarity between two embeddings
    fn embedding_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        
        // Dot product (vectors are already normalized)
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }
    
    /// Convert text to embedding using hash-based word tokenization
    /// Inspired by SpatialVortex VectorStore.simple_embedding()
    fn text_to_embedding(&self, text: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut embedding = vec![0.0f32; 384]; // Standard embedding size (matches VectorStore)
        
        // Hash-based word embedding - each word contributes to specific dimensions
        for (i, word) in text.split_whitespace().enumerate() {
            let word_lower = word.to_lowercase();
            let mut hasher = DefaultHasher::new();
            word_lower.hash(&mut hasher);
            let hash = hasher.finish();
            
            // Primary dimension from hash
            let idx = (hash as usize) % 384;
            embedding[idx] += 1.0 / (i + 1) as f32;
            
            // Secondary dimensions for richer representation
            let idx2 = ((hash >> 16) as usize) % 384;
            embedding[idx2] += 0.5 / (i + 1) as f32;
            
            let idx3 = ((hash >> 32) as usize) % 384;
            embedding[idx3] += 0.25 / (i + 1) as f32;
            
            // Check if word has learned embedding from training
            if let Some(&freq) = self.ngram_frequencies.get(&word_lower) {
                // Boost dimensions based on word frequency (IDF-like)
                let idf = 1.0 / (1.0 + (freq as f32).ln());
                embedding[idx] *= 1.0 + idf;
            }
        }
        
        // L2 normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            embedding.iter_mut().for_each(|x| *x /= norm);
        }
        
        embedding
    }
    
    /// Augment a question with retrieved context
    pub fn augment_question(&mut self, question: &str, choices: &[String]) -> String {
        let mut context_parts = Vec::new();
        
        // Search for question context
        let question_results = self.search(question);
        for result in question_results.iter().take(2) {
            context_parts.push(result.content.clone());
        }
        
        // Search for choice-related context
        for choice in choices {
            let choice_results = self.search(choice);
            for result in choice_results.iter().take(1) {
                if !context_parts.contains(&result.content) {
                    context_parts.push(result.content.clone());
                }
            }
        }
        
        if context_parts.is_empty() {
            question.to_string()
        } else {
            format!("Context: {} Question: {}", context_parts.join(" "), question)
        }
    }
    
    /// Get the most relevant context for scoring a choice
    pub fn score_choice_with_context(&mut self, question: &str, choice: &str) -> f32 {
        let combined_query = format!("{} {}", question, choice);
        let results = self.search(&combined_query);
        
        if results.is_empty() {
            return 0.0;
        }
        
        // Return average relevance of top results
        let total: f32 = results.iter().map(|r| r.relevance).sum();
        total / results.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rag_search() {
        let mut engine = RAGSearchEngine::new(RAGSearchConfig::default());
        
        let results = engine.search("Where do people keep money?");
        assert!(!results.is_empty());
        
        // Should find bank-related knowledge
        assert!(results.iter().any(|r| r.content.to_lowercase().contains("bank")));
    }
    
    #[test]
    fn test_augment_question() {
        let mut engine = RAGSearchEngine::new(RAGSearchConfig::default());
        
        let augmented = engine.augment_question(
            "Where do you store money?",
            &["bank".to_string(), "kitchen".to_string()]
        );
        
        assert!(augmented.contains("Context:"));
    }
}
