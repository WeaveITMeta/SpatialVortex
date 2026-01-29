//! Unified Inference Engine
//!
//! Replaces the 18+ competing experts with a single generative model.
//! All reasoning flows through one unified latent representation.
//!
//! ## Architecture
//! ```text
//! Context + Question → Tokenize → Encode → Unified Latent
//!                                              ↓
//!                                    ┌─────────┴─────────┐
//!                                    │  Reasoning Layer  │
//!                                    │  - Temporal State │
//!                                    │  - Transitive     │
//!                                    │  - Multi-hop      │
//!                                    └─────────┬─────────┘
//!                                              ↓
//!                                    Vortex Cycle (3-6-9)
//!                                              ↓
//!                                    Generation Head → Answer
//! ```

use crate::data::models::BeamTensor;
use crate::ml::calm::{CALMEngine, CALMConfig, LatentState};
use crate::ml::generative_arch::{
    SubwordTokenizer, SacredDynamicAttention, SacredAttentionConfig,
    GenerationHead, DynamicMoERouter,
};
use crate::ml::reasoning_engine::{
    TemporalStateTracker, MultiHopReasoner,
};
use crate::ml::transitive_flux::TransitiveFluxReasoner;
use crate::ml::recursive_chains::ChainPathwayReasoner;
use crate::ml::conceptual_agglomeration::ConceptualReasoner;
use std::collections::HashMap;

// =============================================================================
// UNIFIED LATENT STATE
// =============================================================================

/// Unified latent state that carries all reasoning information
#[derive(Debug, Clone)]
pub struct UnifiedLatent {
    /// Core latent vector from CALM encoding
    pub latent: Vec<f32>,
    /// Temporal facts extracted from context
    pub temporal_facts: Vec<(String, String, String)>, // (subject, predicate, object)
    /// Transitive relations
    pub relations: Vec<(String, String, String, f32)>, // (source, relation, target, confidence)
    /// Entity locations (for bAbI-style questions)
    pub entity_locations: HashMap<String, String>,
    /// Entity possessions
    pub entity_possessions: HashMap<String, Vec<String>>,
    /// Reasoning confidence
    pub confidence: f32,
    /// Vortex position (1-9)
    pub vortex_position: u8,
}

impl Default for UnifiedLatent {
    fn default() -> Self {
        Self {
            latent: vec![0.0; 256],
            temporal_facts: Vec::new(),
            relations: Vec::new(),
            entity_locations: HashMap::new(),
            entity_possessions: HashMap::new(),
            confidence: 1.0,
            vortex_position: 1,
        }
    }
}

// =============================================================================
// REASONING LAYER
// =============================================================================

/// Reasoning layer that enriches the latent with structured knowledge
pub struct ReasoningLayer {
    /// Temporal state tracker
    pub temporal: TemporalStateTracker,
    /// Transitive reasoner
    pub transitive: TransitiveFluxReasoner,
    /// Multi-hop reasoner
    pub multi_hop: MultiHopReasoner,
    /// SNOAT chain pathway reasoner (depth-9 multi-hop)
    pub chain_reasoner: ChainPathwayReasoner,
    /// Conceptual agglomeration reasoner (language-independent concepts)
    pub conceptual: ConceptualReasoner,
    /// Embedding dimension
    pub embed_dim: usize,
}

impl ReasoningLayer {
    pub fn new(embed_dim: usize) -> Self {
        Self {
            temporal: TemporalStateTracker::new(),
            transitive: TransitiveFluxReasoner::new(embed_dim),
            multi_hop: MultiHopReasoner::new(5),
            chain_reasoner: ChainPathwayReasoner::new(embed_dim),
            conceptual: ConceptualReasoner::new(),
            embed_dim,
        }
    }
    
    /// Process context and extract structured knowledge into latent
    pub fn process(&mut self, context: &str, latent: &mut UnifiedLatent) {
        // Extract temporal facts
        self.temporal.extract_facts(context);
        
        // Query known entities from context to populate latent
        // Parse context to find entity names
        let context_lower = context.to_lowercase();
        let words: Vec<&str> = context_lower.split_whitespace().collect();
        
        // Find potential entity names (words that appear before "went", "picked", etc.)
        let mut entities: Vec<String> = Vec::new();
        for (i, word) in words.iter().enumerate() {
            if ["went", "picked", "dropped", "got", "took", "is", "moved"].contains(word) {
                if i > 0 {
                    let entity = words[i - 1].trim_matches(|c: char| !c.is_alphanumeric()).to_string();
                    if !entity.is_empty() && entity.len() > 1 {
                        entities.push(entity);
                    }
                }
            }
        }
        
        // Query state for each entity using public API
        for entity in &entities {
            // Query location
            if let Some((location, _conf)) = self.temporal.query_state(entity, "is_at") {
                latent.entity_locations.insert(entity.clone(), location.clone());
                latent.temporal_facts.push((entity.clone(), "is_at".to_string(), location));
            }
            
            // Query possessions
            let possessions = self.temporal.query_possessions(entity);
            if !possessions.is_empty() {
                latent.entity_possessions.insert(entity.clone(), possessions.clone());
                for item in possessions {
                    latent.temporal_facts.push((entity.clone(), "has".to_string(), item));
                }
            }
        }
        
        // Extract transitive relations using public API
        self.transitive.extract_relations(context);
        // Note: Relations are stored internally, we query them via scoring methods
        
        // Process with SNOAT chain reasoner for depth-9 multi-hop
        self.chain_reasoner.process_context(context);
        
        // Process with conceptual agglomeration for language-independent reasoning
        self.conceptual.process_context(context);
        
        // Encode reasoning results into latent vector
        self.encode_reasoning_to_latent(latent);
    }
    
    /// Encode structured reasoning into the latent vector
    fn encode_reasoning_to_latent(&self, latent: &mut UnifiedLatent) {
        // Use first 64 dims for entity count encoding
        let entity_count = latent.entity_locations.len();
        if entity_count > 0 && latent.latent.len() > 64 {
            latent.latent[0] = entity_count as f32 / 10.0;
        }
        
        // Use dims 64-128 for relation count encoding
        let relation_count = latent.relations.len();
        if relation_count > 0 && latent.latent.len() > 128 {
            latent.latent[64] = relation_count as f32 / 10.0;
        }
        
        // Use dims 128-192 for possession count encoding
        let total_possessions: usize = latent.entity_possessions.values().map(|v| v.len()).sum();
        if total_possessions > 0 && latent.latent.len() > 192 {
            latent.latent[128] = total_possessions as f32 / 10.0;
        }
        
        // Encode average confidence from relations
        if !latent.relations.is_empty() {
            let avg_conf: f32 = latent.relations.iter().map(|(_, _, _, c)| c).sum::<f32>() 
                / latent.relations.len() as f32;
            latent.confidence = avg_conf;
            if latent.latent.len() > 256 {
                latent.latent[192] = avg_conf;
            }
        }
    }
    
    /// Answer a question using structured reasoning
    pub fn answer_question(&self, question: &str, latent: &UnifiedLatent) -> Option<(String, f32)> {
        let question_lower = question.to_lowercase();
        
        // Location questions: "Where is X?"
        if question_lower.contains("where is ") || question_lower.contains("where's ") {
            let entity = self.extract_entity(&question_lower, &["where is ", "where's "]);
            if let Some(ent) = entity {
                if let Some(location) = latent.entity_locations.get(&ent) {
                    return Some((location.clone(), latent.confidence));
                }
            }
        }
        
        // Counting questions: "How many objects is X carrying?"
        if question_lower.contains("how many") {
            let entity = self.extract_entity(&question_lower, &["is ", "does "]);
            if let Some(ent) = entity {
                if let Some(possessions) = latent.entity_possessions.get(&ent) {
                    let count = possessions.len();
                    let number_words = ["zero", "one", "two", "three", "four", "five", 
                                       "six", "seven", "eight", "nine", "ten"];
                    let answer = if count < number_words.len() {
                        number_words[count].to_string()
                    } else {
                        count.to_string()
                    };
                    return Some((answer, latent.confidence));
                }
            }
        }
        
        // Yes/No questions about location
        if question_lower.starts_with("is ") && question_lower.contains(" in ") {
            if let Some((entity, location)) = self.parse_location_question(&question_lower) {
                if let Some(actual_loc) = latent.entity_locations.get(&entity) {
                    let matches = actual_loc.to_lowercase() == location.to_lowercase();
                    let answer = if matches { "yes" } else { "no" };
                    return Some((answer.to_string(), latent.confidence));
                }
            }
        }
        
        // Try SNOAT chain reasoner for depth-9 multi-hop
        if let Some((answer, conf, _path)) = self.chain_reasoner.chain.answer_question(question) {
            return Some((answer, conf));
        }
        
        // Multi-hop: "Where is the X?" where X is an object
        // Need to find who has X, then where that person is
        if question_lower.contains("where is the ") {
            // Extract the object being asked about
            if let Some(pos) = question_lower.find("where is the ") {
                let after = &question_lower[pos + 13..];
                let object = after.split(|c: char| c == '?' || c.is_whitespace())
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                
                if !object.is_empty() {
                    // First, check if object is directly at a location
                    if let Some(loc) = latent.entity_locations.get(&object) {
                        return Some((loc.clone(), latent.confidence));
                    }
                    
                    // Multi-hop: Find who has the object, then where they are
                    for (entity, possessions) in &latent.entity_possessions {
                        if possessions.iter().any(|p| p.to_lowercase().contains(&object)) {
                            // Found who has it, now find where they are
                            if let Some(loc) = latent.entity_locations.get(entity) {
                                return Some((loc.clone(), latent.confidence * 0.9));
                            }
                        }
                    }
                }
            }
        }
        
        // "What is X carrying?" or "What does X have?"
        if question_lower.contains("carrying") || 
           (question_lower.contains("what") && question_lower.contains("have")) {
            let entity = self.extract_entity(&question_lower, &["is ", "does ", "did "]);
            if let Some(ent) = entity {
                if let Some(possessions) = latent.entity_possessions.get(&ent) {
                    if !possessions.is_empty() {
                        return Some((possessions.join(", "), latent.confidence));
                    }
                }
            }
        }
        
        None
    }
    
    fn extract_entity(&self, question: &str, patterns: &[&str]) -> Option<String> {
        for pattern in patterns {
            if let Some(pos) = question.find(pattern) {
                let after = &question[pos + pattern.len()..];
                let entity = after.split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string();
                
                if !entity.is_empty() && !["the", "a", "an"].contains(&entity.as_str()) {
                    return Some(entity);
                }
            }
        }
        None
    }
    
    fn parse_location_question(&self, question: &str) -> Option<(String, String)> {
        if let Some(pos) = question.find(" in ") {
            let before = &question[..pos];
            let entity = before
                .trim_start_matches("is ")
                .trim_start_matches("the ")
                .trim()
                .to_string();
            
            let after = &question[pos + 4..];
            let location = after
                .trim_end_matches('?')
                .trim_start_matches("the ")
                .trim()
                .to_string();
            
            if !entity.is_empty() && !location.is_empty() {
                return Some((entity, location));
            }
        }
        None
    }
}

// =============================================================================
// UNIFIED INFERENCE ENGINE
// =============================================================================

/// Configuration for unified inference
#[derive(Debug, Clone)]
pub struct UnifiedConfig {
    pub latent_dim: usize,
    pub max_seq_len: usize,
    pub temperature: f32,
    pub use_reasoning_layer: bool,
    pub vortex_cycles: usize,
}

impl Default for UnifiedConfig {
    fn default() -> Self {
        Self {
            latent_dim: 256,
            max_seq_len: 512,
            temperature: 0.7,
            use_reasoning_layer: true,
            vortex_cycles: 1,
        }
    }
}

/// Unified inference engine - single model, no competing experts
pub struct UnifiedInferenceEngine {
    pub config: UnifiedConfig,
    /// Tokenizer
    pub tokenizer: SubwordTokenizer,
    /// CALM encoder/decoder
    pub calm: CALMEngine,
    /// Sacred attention (3-6-9)
    pub sacred_attention: SacredDynamicAttention,
    /// Reasoning layer
    pub reasoning: ReasoningLayer,
    /// Generation head for answer selection
    pub generation_head: GenerationHead,
    /// MoE router (for expert weighting, not voting)
    pub moe_router: DynamicMoERouter,
    /// Word embeddings
    pub word_embeddings: HashMap<String, Vec<f32>>,
}

impl UnifiedInferenceEngine {
    pub fn new(config: UnifiedConfig) -> Self {
        let latent_dim = config.latent_dim;
        
        // Initialize tokenizer
        let tokenizer = SubwordTokenizer::new(latent_dim);
        let vocab_size = tokenizer.vocab_size();
        
        // Initialize CALM
        let calm_config = CALMConfig {
            latent_dim,
            chunk_size: 8,
            compression_ratio: 8,
            energy_threshold: 0.01,
            speculative_decoding: true,
            batch_size: 4,
        };
        let calm = CALMEngine::new(calm_config);
        
        // Initialize sacred attention
        let sacred_attention = SacredDynamicAttention::new(
            latent_dim,
            SacredAttentionConfig::default(),
        );
        
        // Initialize reasoning layer
        let reasoning = ReasoningLayer::new(latent_dim);
        
        // Initialize generation head
        let generation_head = GenerationHead::new(latent_dim, vocab_size);
        
        // Initialize MoE router
        let moe_router = DynamicMoERouter::new(latent_dim);
        
        Self {
            config,
            tokenizer,
            calm,
            sacred_attention,
            reasoning,
            generation_head,
            moe_router,
            word_embeddings: HashMap::new(),
        }
    }
    
    /// Get or create embedding for a word
    pub fn get_embedding(&mut self, word: &str) -> Vec<f32> {
        if let Some(embed) = self.word_embeddings.get(word) {
            return embed.clone();
        }
        
        // Create embedding from hash
        let hash = word.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let embed: Vec<f32> = (0..self.config.latent_dim)
            .map(|i| {
                let seed = hash.wrapping_add(i as u64);
                ((seed as f32 * 0.0001).sin() + (seed as f32 * 0.00003).cos()) * 0.5
            })
            .collect();
        
        self.word_embeddings.insert(word.to_string(), embed.clone());
        embed
    }
    
    /// Encode text to latent
    pub fn encode(&mut self, text: &str) -> UnifiedLatent {
        let words: Vec<&str> = text.split_whitespace().collect();
        
        // Get embeddings and average
        let mut combined = vec![0.0f32; self.config.latent_dim];
        for word in &words {
            let embed = self.get_embedding(word);
            for (i, &val) in embed.iter().enumerate() {
                if i < combined.len() {
                    combined[i] += val;
                }
            }
        }
        if !words.is_empty() {
            for val in &mut combined {
                *val /= words.len() as f32;
            }
        }
        
        // Convert to BeamTensors for CALM
        let beams = self.embedding_to_beams(&combined);
        let calm_latent = self.calm.encode(&beams);
        
        UnifiedLatent {
            latent: calm_latent.latent,
            ..Default::default()
        }
    }
    
    fn embedding_to_beams(&self, embedding: &[f32]) -> Vec<BeamTensor> {
        let chunk_size = 9;
        let mut beams = Vec::new();
        
        for (chunk_idx, chunk) in embedding.chunks(chunk_size).enumerate() {
            let mut beam = BeamTensor::default();
            for (i, &val) in chunk.iter().enumerate() {
                if i < 9 {
                    beam.digits[i] = val;
                }
            }
            beam.position = chunk_idx as u8;
            beam.confidence = 1.0;
            beams.push(beam);
        }
        
        beams
    }
    
    /// Run vortex cycle with sacred attention at 3, 6, 9
    fn vortex_cycle(&self, latent: &mut UnifiedLatent, context_keys: &[Vec<f32>]) {
        let positions = [1u8, 2, 3, 4, 5, 6, 7, 8, 9];
        
        for &pos in &positions {
            latent.vortex_position = pos;
            
            // CALM prediction step
            let calm_state = LatentState {
                latent: latent.latent.clone(),
                energy: 0.0,
                sacred_alignment: 0.0,
                step: pos as usize,
            };
            let next_state = self.calm.predict_next(&calm_state);
            latent.latent = next_state.latent;
            
            // Apply sacred attention at positions 3, 6, 9
            if (pos == 3 || pos == 6 || pos == 9) && !context_keys.is_empty() {
                let (attended, _weights) = self.sacred_attention.forward(
                    &latent.latent,
                    context_keys,
                    context_keys,
                    pos,
                );
                
                // Blend attended output with latent
                let blend_factor = match pos {
                    3 => 0.3,
                    6 => 0.5,
                    9 => 0.7,
                    _ => 0.3,
                };
                
                for (i, &att) in attended.iter().enumerate() {
                    if i < latent.latent.len() {
                        latent.latent[i] = latent.latent[i] * (1.0 - blend_factor) + att * blend_factor;
                    }
                }
            }
        }
    }
    
    /// Main inference: context + question → answer
    pub fn infer(&mut self, context: &str, question: &str, choices: &[String]) -> (usize, f32) {
        // Step 1: Encode context + question into unified latent
        let full_text = format!("{}\n{}", context, question);
        let mut latent = self.encode(&full_text);
        
        // Step 2: Apply reasoning layer to extract structured knowledge
        if self.config.use_reasoning_layer {
            self.reasoning.process(context, &mut latent);
        }
        
        // Step 3: Try to answer directly from reasoning
        if let Some((answer, conf)) = self.reasoning.answer_question(question, &latent) {
            // Find matching choice
            for (idx, choice) in choices.iter().enumerate() {
                let choice_lower = choice.to_lowercase();
                if choice_lower == answer || choice_lower.contains(&answer) || answer.contains(&choice_lower) {
                    return (idx, conf);
                }
            }
        }
        
        // Step 4: Build context keys from choices for attention
        let context_keys: Vec<Vec<f32>> = choices.iter()
            .map(|c| self.get_embedding(c))
            .collect();
        
        // Step 5: Run vortex cycles
        for _ in 0..self.config.vortex_cycles {
            self.vortex_cycle(&mut latent, &context_keys);
        }
        
        // Step 6: Score each choice using cosine similarity to final latent
        let mut best_idx = 0;
        let mut best_score = f32::NEG_INFINITY;
        
        for (idx, choice) in choices.iter().enumerate() {
            let choice_embed = self.get_embedding(choice);
            let score = self.cosine_similarity(&latent.latent, &choice_embed);
            
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }
        
        // Convert score to confidence (0-1)
        let confidence = (best_score + 1.0) / 2.0; // Cosine sim is [-1, 1]
        
        (best_idx, confidence)
    }
    
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a * norm_b)
        } else {
            0.0
        }
    }
    
    /// Generate text (for open-ended questions)
    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> String {
        let mut latent = self.encode(prompt);
        
        // Apply reasoning
        if self.config.use_reasoning_layer {
            self.reasoning.process(prompt, &mut latent);
        }
        
        // Run vortex cycle
        self.vortex_cycle(&mut latent, &[]);
        
        // Generate tokens
        let mut output_tokens: Vec<u32> = Vec::new();
        
        for _ in 0..max_tokens {
            let logits = self.generation_head.forward(&latent.latent);
            let token = self.generation_head.sample_top_k(&logits, 50, self.config.temperature);
            
            if token == self.tokenizer.eos_id {
                break;
            }
            
            output_tokens.push(token);
            
            // Update latent with new token
            let token_embed = self.tokenizer.get_embedding(token);
            for (i, &val) in token_embed.iter().enumerate() {
                if i < latent.latent.len() {
                    latent.latent[i] = latent.latent[i] * 0.9 + val * 0.1;
                }
            }
        }
        
        self.tokenizer.detokenize(&output_tokens)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unified_latent() {
        let latent = UnifiedLatent::default();
        assert_eq!(latent.latent.len(), 256);
        assert_eq!(latent.vortex_position, 1);
    }
    
    #[test]
    fn test_reasoning_layer() {
        let mut reasoning = ReasoningLayer::new(256);
        let mut latent = UnifiedLatent::default();
        
        reasoning.process("John went to the kitchen. Mary went to the garden.", &mut latent);
        
        assert!(latent.entity_locations.contains_key("john"));
        assert_eq!(latent.entity_locations.get("john"), Some(&"kitchen".to_string()));
    }
    
    #[test]
    fn test_unified_inference() {
        let config = UnifiedConfig::default();
        let mut engine = UnifiedInferenceEngine::new(config);
        
        let context = "John went to the kitchen. Mary went to the garden.";
        let question = "Where is John?";
        let choices = vec!["kitchen".to_string(), "garden".to_string(), "bathroom".to_string()];
        
        let (answer_idx, confidence) = engine.infer(context, question, &choices);
        
        // Should select "kitchen" (index 0)
        assert_eq!(answer_idx, 0);
        assert!(confidence > 0.0);
    }
}
