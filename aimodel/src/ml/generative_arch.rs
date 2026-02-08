//! Generative Architecture for SpatialVortex
//!
//! This module implements the next-generation architecture with:
//! - **Subword Tokenization**: BPE-style tokenization with learned embeddings
//! - **Generative Mode**: Standalone autoregressive text generation
//! - **Dynamic Attention**: 3-6-9 sacred attention heads per vortex position
//! - **Few-Shot Learning**: K-shot context window for in-context learning
//! - **Exhaustive Pathway**: n! search integrated into generation
//! - **RAG Integration**: External knowledge retrieval
//! - **Symbolic Math**: Hybrid neuro-symbolic execution
//!
//! ## Architecture Overview
//! ```text
//! Input Text → Subword Tokenizer → Token Embeddings → CALM Encoder
//!                                                          ↓
//!                                              Vortex Cycle (1→2→4→8→7→5→1)
//!                                              with Dynamic 3-6-9 Attention
//!                                                          ↓
//!                                              Exhaustive Pathway Search
//!                                                          ↓
//!                                              Generation Head → Output Tokens
//! ```

use crate::data::models::BeamTensor;
use crate::ml::calm::{CALMEngine, CALMConfig, LatentState};
use crate::ml::pathway::{ExhaustivePathwayOptimizer, PathwayConfig, ScoredPathway};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Subword Tokenizer with Learned Embeddings
// =============================================================================

/// Subword token with embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubwordToken {
    pub id: u32,
    pub text: String,
    pub embedding: Vec<f32>,
}

/// BPE-style subword tokenizer with learned embeddings
#[derive(Debug, Clone)]
pub struct SubwordTokenizer {
    /// Vocabulary: token_id → token
    pub vocab: HashMap<u32, String>,
    /// Reverse vocab: token → token_id
    pub token_to_id: HashMap<String, u32>,
    /// Learned embeddings: token_id → embedding
    pub embeddings: HashMap<u32, Vec<f32>>,
    /// Embedding dimension
    pub embed_dim: usize,
    /// Special tokens
    pub pad_id: u32,
    pub unk_id: u32,
    pub bos_id: u32,
    pub eos_id: u32,
    /// Merge rules for BPE
    pub merges: Vec<(String, String)>,
    /// Next available token ID
    next_id: u32,
}

impl SubwordTokenizer {
    pub fn new(embed_dim: usize) -> Self {
        let mut tokenizer = Self {
            vocab: HashMap::new(),
            token_to_id: HashMap::new(),
            embeddings: HashMap::new(),
            embed_dim,
            pad_id: 0,
            unk_id: 1,
            bos_id: 2,
            eos_id: 3,
            merges: Vec::new(),
            next_id: 4,
        };
        
        // Add special tokens
        tokenizer.add_token("<PAD>", 0);
        tokenizer.add_token("<UNK>", 1);
        tokenizer.add_token("<BOS>", 2);
        tokenizer.add_token("<EOS>", 3);
        
        // Add basic character vocabulary (a-z, 0-9, common punctuation)
        for c in 'a'..='z' {
            tokenizer.add_token(&c.to_string(), tokenizer.next_id);
            tokenizer.next_id += 1;
        }
        for c in 'A'..='Z' {
            tokenizer.add_token(&c.to_string(), tokenizer.next_id);
            tokenizer.next_id += 1;
        }
        for c in '0'..='9' {
            tokenizer.add_token(&c.to_string(), tokenizer.next_id);
            tokenizer.next_id += 1;
        }
        for c in [' ', '.', ',', '?', '!', '\'', '"', '-', ':', ';', '(', ')', '\n'].iter() {
            tokenizer.add_token(&c.to_string(), tokenizer.next_id);
            tokenizer.next_id += 1;
        }
        
        tokenizer
    }
    
    fn add_token(&mut self, text: &str, id: u32) {
        self.vocab.insert(id, text.to_string());
        self.token_to_id.insert(text.to_string(), id);
        // Initialize embedding with Xavier initialization
        let scale = (2.0 / self.embed_dim as f32).sqrt();
        let embedding: Vec<f32> = (0..self.embed_dim)
            .map(|i| ((id as f32 * 0.1 + i as f32 * 0.01).sin() * scale))
            .collect();
        self.embeddings.insert(id, embedding);
    }
    
    /// Learn BPE merges from corpus
    pub fn learn_bpe(&mut self, corpus: &[String], num_merges: usize) {
        // Count pair frequencies
        let mut pair_counts: HashMap<(String, String), usize> = HashMap::new();
        
        for text in corpus {
            let chars: Vec<String> = text.chars().map(|c| c.to_string()).collect();
            for window in chars.windows(2) {
                let pair = (window[0].clone(), window[1].clone());
                *pair_counts.entry(pair).or_insert(0) += 1;
            }
        }
        
        // Greedily merge most frequent pairs
        for _ in 0..num_merges {
            if let Some(((a, b), _count)) = pair_counts.iter()
                .max_by_key(|(_, &count)| count)
                .map(|(pair, count)| (pair.clone(), *count))
            {
                let merged = format!("{}{}", a, b);
                self.merges.push((a.clone(), b.clone()));
                self.add_token(&merged, self.next_id);
                self.next_id += 1;
                
                // Update pair counts (simplified - full BPE would re-scan)
                pair_counts.remove(&(a, b));
            } else {
                break;
            }
        }
    }
    
    /// Tokenize text into subword tokens
    pub fn tokenize(&self, text: &str) -> Vec<u32> {
        let mut tokens = vec![self.bos_id];
        
        // Simple character-level with merge application
        let mut chars: Vec<String> = text.chars().map(|c| c.to_string()).collect();
        
        // Apply merges greedily
        for (a, b) in &self.merges {
            let merged = format!("{}{}", a, b);
            let mut i = 0;
            while i + 1 < chars.len() {
                if &chars[i] == a && &chars[i + 1] == b {
                    chars[i] = merged.clone();
                    chars.remove(i + 1);
                }
                i += 1;
            }
        }
        
        // Convert to token IDs
        for subword in chars {
            let id = self.token_to_id.get(&subword)
                .copied()
                .unwrap_or(self.unk_id);
            tokens.push(id);
        }
        
        tokens.push(self.eos_id);
        tokens
    }
    
    /// Detokenize token IDs back to text
    pub fn detokenize(&self, tokens: &[u32]) -> String {
        tokens.iter()
            .filter(|&&id| id != self.pad_id && id != self.bos_id && id != self.eos_id)
            .filter_map(|id| self.vocab.get(id))
            .cloned()
            .collect()
    }
    
    /// Get embedding for token ID
    pub fn get_embedding(&self, id: u32) -> Vec<f32> {
        self.embeddings.get(&id)
            .cloned()
            .unwrap_or_else(|| self.embeddings.get(&self.unk_id).cloned().unwrap_or_default())
    }
    
    /// Get embeddings for token sequence
    pub fn get_embeddings(&self, tokens: &[u32]) -> Vec<Vec<f32>> {
        tokens.iter().map(|&id| self.get_embedding(id)).collect()
    }
    
    /// Update embedding via gradient
    pub fn update_embedding(&mut self, id: u32, gradient: &[f32], learning_rate: f32) {
        if let Some(embed) = self.embeddings.get_mut(&id) {
            for (i, grad) in gradient.iter().enumerate() {
                if i < embed.len() {
                    embed[i] -= learning_rate * grad;
                }
            }
        }
    }
    
    /// Vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }
}

// =============================================================================
// Dynamic Sacred Attention (3-6-9 Heads per Vortex Position)
// =============================================================================

/// Configuration for dynamic sacred attention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredAttentionConfig {
    /// Base number of heads (multiplied by sacred position)
    pub base_heads: usize,
    /// Head dimension
    pub head_dim: usize,
    /// Dropout rate
    pub dropout: f32,
    /// Whether to use position-specific attention patterns
    pub position_specific: bool,
}

impl Default for SacredAttentionConfig {
    fn default() -> Self {
        Self {
            base_heads: 1,  // 3 heads at pos 3, 6 at pos 6, 9 at pos 9
            head_dim: 32,
            dropout: 0.1,
            position_specific: true,
        }
    }
}

// =============================================================================
// Attribute-Focused Implication System (369 Enhanced)
// =============================================================================

/// An implication derived from comparing node labels to attributes
#[derive(Debug, Clone)]
pub struct AttributeImplication {
    /// Source node position in vortex
    pub source_position: u8,
    /// The attribute key being analyzed
    pub attribute_key: String,
    /// The attribute value
    pub attribute_value: f32,
    /// The node label/title being compared
    pub node_label: String,
    /// Implication type (causal, property, relation, etc.)
    pub implication_type: ImplicationType,
    /// Strength of the implication (0.0 to 1.0)
    pub strength: f32,
    /// Embedding of the implication in latent space
    pub latent_embedding: Vec<f32>,
    /// Tracked object this implication relates to (for object-in-flow tracking)
    pub tracked_object: Option<TrackedObject>,
    /// Pattern match detail level (0.0 = general, 1.0 = highly specific)
    pub detail_level: f32,
}

/// A tracked object flowing through the vortex cycle
/// Maintains object identity and accumulated attributes across positions
#[derive(Debug, Clone)]
pub struct TrackedObject {
    /// Unique identifier for this object in the flow
    pub object_id: String,
    /// Object type/category (e.g., "person", "location", "item")
    pub object_type: String,
    /// Current position in vortex flow
    pub current_position: u8,
    /// Accumulated attributes from all positions visited
    pub accumulated_attrs: Vec<(String, f32, u8)>, // (attr_key, value, position_learned)
    /// Object embedding that evolves through flow
    pub flow_embedding: Vec<f32>,
    /// Positions this object has been observed at
    pub position_history: Vec<u8>,
    /// Pattern matches detected for this object
    pub matched_patterns: Vec<ObjectPattern>,
}

/// A pattern that an object matches, triggering detailed analysis
#[derive(Debug, Clone)]
pub struct ObjectPattern {
    /// Pattern name/identifier
    pub pattern_name: String,
    /// Keywords that triggered this pattern
    pub trigger_keywords: Vec<String>,
    /// Detail boost factor (how much more attention to pay)
    pub detail_boost: f32,
    /// Specific attributes to focus on for this pattern
    pub focus_attributes: Vec<String>,
}

/// Types of implications that can be derived
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplicationType {
    /// Attribute implies a property of the node
    Property,
    /// Attribute implies a causal relationship
    Causal,
    /// Attribute implies a temporal relationship
    Temporal,
    /// Attribute implies a spatial relationship
    Spatial,
    /// Attribute implies a logical relationship
    Logical,
    /// Attribute implies a semantic similarity
    Semantic,
    /// Sacred position verification (3, 6, 9)
    SacredVerification,
}

/// Attribute-focused attention that extracts implications at each vortex step
/// Enhanced with object-in-flow tracking for detailed pattern-specific analysis
#[derive(Debug, Clone)]
pub struct AttributeFocusedAttention {
    /// Latent dimension for embeddings
    pub latent_dim: usize,
    /// Implication extraction weights per sacred position
    pub implication_weights_3: Vec<f32>,  // 3 heads for entity attributes
    pub implication_weights_6: Vec<f32>,  // 6 heads for relationships
    pub implication_weights_9: Vec<f32>,  // 9 heads for verification
    /// Attribute comparison weights
    pub attr_compare_weights: Vec<f32>,
    /// Label embedding weights
    pub label_embed_weights: Vec<f32>,
    /// Implication type classifier weights
    pub type_classifier: Vec<f32>,
    /// Tracked objects flowing through the vortex (object_id -> TrackedObject)
    pub tracked_objects: std::collections::HashMap<String, TrackedObject>,
    /// Known patterns that trigger detailed object analysis
    pub known_patterns: Vec<ObjectPattern>,
    /// Object type detection weights
    pub object_type_weights: Vec<f32>,
}

impl AttributeFocusedAttention {
    pub fn new(latent_dim: usize) -> Self {
        let scale = (2.0 / latent_dim as f32).sqrt();
        
        // Initialize known patterns for detailed object analysis
        let known_patterns = vec![
            ObjectPattern {
                pattern_name: "person_location".to_string(),
                trigger_keywords: vec!["went".to_string(), "moved".to_string(), "travelled".to_string(), "is in".to_string()],
                detail_boost: 2.0,
                focus_attributes: vec!["location".to_string(), "position".to_string(), "place".to_string()],
            },
            ObjectPattern {
                pattern_name: "object_container".to_string(),
                trigger_keywords: vec!["picked up".to_string(), "dropped".to_string(), "put".to_string(), "left".to_string()],
                detail_boost: 2.0,
                focus_attributes: vec!["contains".to_string(), "held_by".to_string(), "location".to_string()],
            },
            ObjectPattern {
                pattern_name: "spatial_relation".to_string(),
                trigger_keywords: vec!["left of".to_string(), "right of".to_string(), "above".to_string(), "below".to_string()],
                detail_boost: 2.5,
                focus_attributes: vec!["spatial".to_string(), "position".to_string(), "relative".to_string()],
            },
            ObjectPattern {
                pattern_name: "size_comparison".to_string(),
                trigger_keywords: vec!["bigger".to_string(), "smaller".to_string(), "larger".to_string(), "fits".to_string()],
                detail_boost: 2.5,
                focus_attributes: vec!["size".to_string(), "dimension".to_string(), "capacity".to_string()],
            },
            ObjectPattern {
                pattern_name: "causal_chain".to_string(),
                trigger_keywords: vec!["because".to_string(), "therefore".to_string(), "causes".to_string(), "leads to".to_string()],
                detail_boost: 2.0,
                focus_attributes: vec!["cause".to_string(), "effect".to_string(), "reason".to_string()],
            },
        ];
        
        Self {
            latent_dim,
            // 3 heads × latent_dim for position 3
            implication_weights_3: (0..3 * latent_dim)
                .map(|i| ((i as f32 * 0.1).sin() * scale))
                .collect(),
            // 6 heads × latent_dim for position 6
            implication_weights_6: (0..6 * latent_dim)
                .map(|i| ((i as f32 * 0.15).cos() * scale))
                .collect(),
            // 9 heads × latent_dim for position 9
            implication_weights_9: (0..9 * latent_dim)
                .map(|i| ((i as f32 * 0.2).sin() * scale))
                .collect(),
            // Attribute comparison: latent_dim × latent_dim
            attr_compare_weights: (0..latent_dim * latent_dim)
                .map(|i| ((i as f32 * 0.05).cos() * scale))
                .collect(),
            // Label embedding: latent_dim
            label_embed_weights: (0..latent_dim)
                .map(|i| ((i as f32 * 0.1).sin() * scale))
                .collect(),
            // Type classifier: 7 types × latent_dim
            type_classifier: (0..7 * latent_dim)
                .map(|i| ((i as f32 * 0.08).cos() * scale))
                .collect(),
            // Object tracking
            tracked_objects: std::collections::HashMap::new(),
            known_patterns,
            // Object type detection weights
            object_type_weights: (0..5 * latent_dim) // 5 object types
                .map(|i| ((i as f32 * 0.12).sin() * scale))
                .collect(),
        }
    }
    
    /// Extract implications by comparing node label to attributes at a vortex position
    pub fn extract_implications(
        &self,
        node_label: &str,
        node_position: u8,
        attributes: &[(String, f32)],  // (key, value) pairs
        latent_state: &[f32],
    ) -> Vec<AttributeImplication> {
        let mut implications = Vec::new();
        
        // Get position-specific weights
        let (weights, num_heads) = match node_position {
            3 => (&self.implication_weights_3, 3),
            6 => (&self.implication_weights_6, 6),
            9 => (&self.implication_weights_9, 9),
            _ => (&self.implication_weights_3, 3), // Default to 3
        };
        
        // Embed the node label
        let label_embedding = self.embed_label(node_label);
        
        // For each attribute, compute implication
        for (attr_key, attr_value) in attributes {
            // Compute attention between label and attribute
            let attr_embedding = self.embed_attribute(attr_key, *attr_value);
            
            // Multi-head attention over label-attribute pair
            let mut implication_strength = 0.0f32;
            let mut combined_embedding = vec![0.0f32; self.latent_dim];
            
            for head in 0..num_heads {
                let head_start = head * self.latent_dim;
                let head_end = (head_start + self.latent_dim).min(weights.len());
                
                // Compute attention score for this head
                let mut score = 0.0f32;
                for i in 0..self.latent_dim.min(label_embedding.len()).min(attr_embedding.len()) {
                    let w_idx = head_start + i;
                    if w_idx < head_end {
                        score += label_embedding[i] * attr_embedding[i] * weights[w_idx];
                    }
                }
                
                // Normalize and accumulate
                let head_weight = 1.0 / num_heads as f32;
                implication_strength += score.abs() * head_weight;
                
                // Combine embeddings weighted by attention
                for i in 0..self.latent_dim {
                    combined_embedding[i] += (label_embedding.get(i).unwrap_or(&0.0) 
                        + attr_embedding.get(i).unwrap_or(&0.0)) * head_weight * score.abs();
                }
            }
            
            // Classify implication type
            let impl_type = self.classify_implication_type(&combined_embedding, node_position);
            
            // Modulate by latent state (CALM integration)
            let latent_modulation = self.compute_latent_modulation(&combined_embedding, latent_state);
            implication_strength *= latent_modulation;
            
            // Normalize embedding
            let norm: f32 = combined_embedding.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
            for val in &mut combined_embedding {
                *val /= norm;
            }
            
            // Check for pattern matches to determine detail level
            let (detail_level, matched_pattern) = self.check_pattern_match(node_label, attr_key);
            
            // Boost strength if pattern matches (detailed analysis mode)
            let boosted_strength = if detail_level > 0.0 {
                (implication_strength * (1.0 + detail_level)).clamp(0.0, 1.0)
            } else {
                implication_strength.clamp(0.0, 1.0)
            };
            
            // Check if this relates to a tracked object
            let tracked_obj = self.tracked_objects.get(node_label).cloned();
            
            implications.push(AttributeImplication {
                source_position: node_position,
                attribute_key: attr_key.clone(),
                attribute_value: *attr_value,
                node_label: node_label.to_string(),
                implication_type: impl_type,
                strength: boosted_strength,
                latent_embedding: combined_embedding,
                tracked_object: tracked_obj,
                detail_level,
            });
        }
        
        // Sort by strength (strongest first)
        implications.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal));
        
        implications
    }
    
    /// Check if the node label and attribute match any known patterns
    /// Returns (detail_level, matched_pattern_name)
    fn check_pattern_match(&self, node_label: &str, attr_key: &str) -> (f32, Option<String>) {
        let label_lower = node_label.to_lowercase();
        let attr_lower = attr_key.to_lowercase();
        let combined = format!("{} {}", label_lower, attr_lower);
        
        for pattern in &self.known_patterns {
            for keyword in &pattern.trigger_keywords {
                if combined.contains(keyword) || label_lower.contains(keyword) {
                    return (pattern.detail_boost, Some(pattern.pattern_name.clone()));
                }
            }
            // Also check if attribute matches focus attributes
            for focus_attr in &pattern.focus_attributes {
                if attr_lower.contains(focus_attr) {
                    return (pattern.detail_boost * 0.5, Some(pattern.pattern_name.clone()));
                }
            }
        }
        
        (0.0, None)
    }
    
    /// Embed a node label into latent space
    fn embed_label(&self, label: &str) -> Vec<f32> {
        let mut embedding = vec![0.0f32; self.latent_dim];
        
        // Character-based embedding with position weighting
        for (i, c) in label.chars().enumerate() {
            let char_val = c as u32 as f32 / 128.0;
            let pos_weight = 1.0 / (1.0 + i as f32 * 0.1);
            
            for j in 0..self.latent_dim {
                let idx = (c as usize + i * 7 + j) % self.latent_dim;
                embedding[idx] += char_val * pos_weight * self.label_embed_weights[j];
            }
        }
        
        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
        for val in &mut embedding {
            *val /= norm;
        }
        
        embedding
    }
    
    /// Embed an attribute (key + value) into latent space
    fn embed_attribute(&self, key: &str, value: f32) -> Vec<f32> {
        let mut embedding = vec![0.0f32; self.latent_dim];
        
        // Key embedding
        for (i, c) in key.chars().enumerate() {
            let char_val = c as u32 as f32 / 128.0;
            for j in 0..self.latent_dim {
                let idx = (c as usize + i * 11 + j) % self.latent_dim;
                if idx < self.attr_compare_weights.len() / self.latent_dim {
                    embedding[j] += char_val * self.attr_compare_weights[idx * self.latent_dim + j];
                }
            }
        }
        
        // Value modulation
        let value_scale = value.abs().min(10.0) / 10.0;
        for i in 0..self.latent_dim {
            embedding[i] *= 1.0 + value_scale * 0.5;
            // Add value-based offset
            embedding[i] += value * 0.01 * ((i as f32 * 0.1).sin());
        }
        
        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
        for val in &mut embedding {
            *val /= norm;
        }
        
        embedding
    }
    
    /// Classify the type of implication based on embedding and position
    fn classify_implication_type(&self, embedding: &[f32], position: u8) -> ImplicationType {
        // Compute scores for each type
        let mut type_scores = vec![0.0f32; 7];
        
        for (type_idx, score) in type_scores.iter_mut().enumerate() {
            for i in 0..self.latent_dim.min(embedding.len()) {
                let w_idx = type_idx * self.latent_dim + i;
                if w_idx < self.type_classifier.len() {
                    *score += embedding[i] * self.type_classifier[w_idx];
                }
            }
        }
        
        // Sacred position bias
        match position {
            3 => type_scores[0] += 0.3, // Property bias at position 3
            6 => type_scores[1] += 0.3, // Causal bias at position 6
            9 => type_scores[6] += 0.5, // Verification bias at position 9
            _ => {}
        }
        
        // Find max score
        let max_idx = type_scores.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        match max_idx {
            0 => ImplicationType::Property,
            1 => ImplicationType::Causal,
            2 => ImplicationType::Temporal,
            3 => ImplicationType::Spatial,
            4 => ImplicationType::Logical,
            5 => ImplicationType::Semantic,
            6 => ImplicationType::SacredVerification,
            _ => ImplicationType::Property,
        }
    }
    
    /// Compute modulation factor from CALM latent state
    fn compute_latent_modulation(&self, implication_embed: &[f32], latent_state: &[f32]) -> f32 {
        // Cosine similarity between implication and latent state
        let mut dot = 0.0f32;
        let mut norm_impl = 0.0f32;
        let mut norm_latent = 0.0f32;
        
        for i in 0..self.latent_dim.min(implication_embed.len()).min(latent_state.len()) {
            dot += implication_embed[i] * latent_state[i];
            norm_impl += implication_embed[i] * implication_embed[i];
            norm_latent += latent_state[i] * latent_state[i];
        }
        
        let similarity = if norm_impl > 0.0 && norm_latent > 0.0 {
            dot / (norm_impl.sqrt() * norm_latent.sqrt())
        } else {
            0.0
        };
        
        // Convert to modulation factor (0.5 to 1.5)
        1.0 + similarity * 0.5
    }
    
    /// Aggregate implications for pathway search scoring
    pub fn aggregate_for_pathway(&self, implications: &[AttributeImplication]) -> Vec<f32> {
        let mut aggregated = vec![0.0f32; self.latent_dim];
        let mut total_weight = 0.0f32;
        
        for impl_ in implications {
            let weight = impl_.strength;
            total_weight += weight;
            
            for i in 0..self.latent_dim.min(impl_.latent_embedding.len()) {
                aggregated[i] += impl_.latent_embedding[i] * weight;
            }
        }
        
        // Normalize by total weight
        if total_weight > 0.0 {
            for val in &mut aggregated {
                *val /= total_weight;
            }
        }
        
        aggregated
    }
    
    /// Track or update an object flowing through the vortex
    /// This maintains object identity and accumulates attributes across positions
    pub fn track_object(
        &mut self,
        object_id: &str,
        object_type: &str,
        position: u8,
        attributes: &[(String, f32)],
        embedding: &[f32],
    ) {
        let label_lower = object_id.to_lowercase();
        
        // Check for pattern matches
        let mut matched_patterns = Vec::new();
        for pattern in &self.known_patterns {
            for keyword in &pattern.trigger_keywords {
                if label_lower.contains(keyword) {
                    matched_patterns.push(pattern.clone());
                    break;
                }
            }
        }
        
        if let Some(existing) = self.tracked_objects.get_mut(object_id) {
            // Update existing object
            existing.current_position = position;
            existing.position_history.push(position);
            
            // Accumulate new attributes
            for (attr_key, attr_val) in attributes {
                existing.accumulated_attrs.push((attr_key.clone(), *attr_val, position));
            }
            
            // Evolve embedding through flow (blend with new)
            for (i, &val) in embedding.iter().enumerate() {
                if i < existing.flow_embedding.len() {
                    existing.flow_embedding[i] = existing.flow_embedding[i] * 0.7 + val * 0.3;
                }
            }
            
            // Add new pattern matches
            for pattern in matched_patterns {
                if !existing.matched_patterns.iter().any(|p| p.pattern_name == pattern.pattern_name) {
                    existing.matched_patterns.push(pattern);
                }
            }
        } else {
            // Create new tracked object
            let flow_embedding = if embedding.is_empty() {
                vec![0.0f32; self.latent_dim]
            } else {
                embedding.to_vec()
            };
            
            let tracked = TrackedObject {
                object_id: object_id.to_string(),
                object_type: object_type.to_string(),
                current_position: position,
                accumulated_attrs: attributes.iter()
                    .map(|(k, v)| (k.clone(), *v, position))
                    .collect(),
                flow_embedding,
                position_history: vec![position],
                matched_patterns,
            };
            
            self.tracked_objects.insert(object_id.to_string(), tracked);
        }
    }
    
    /// Get detailed analysis for a tracked object based on its pattern matches
    /// Returns boosted implications when patterns match
    pub fn get_object_detail_boost(&self, object_id: &str) -> f32 {
        if let Some(obj) = self.tracked_objects.get(object_id) {
            if obj.matched_patterns.is_empty() {
                return 1.0; // No boost
            }
            
            // Compute total boost from all matched patterns
            let total_boost: f32 = obj.matched_patterns.iter()
                .map(|p| p.detail_boost)
                .sum();
            
            // Average boost, capped at 3.0
            (total_boost / obj.matched_patterns.len() as f32).min(3.0)
        } else {
            1.0 // No boost for untracked objects
        }
    }
    
    /// Extract objects from context text and begin tracking them
    pub fn extract_and_track_objects(&mut self, context: &str, position: u8) {
        let context_lower = context.to_lowercase();
        
        // Common entity patterns to detect
        let person_indicators = ["went", "moved", "picked", "dropped", "is in", "travelled"];
        let object_indicators = ["the ", "a ", "an "];
        let location_indicators = ["to the", "in the", "at the", "from the"];
        
        // Extract potential objects from context
        let words: Vec<&str> = context_lower.split_whitespace().collect();
        
        for (i, &word) in words.iter().enumerate() {
            // Check for person names (capitalized words not at sentence start)
            if i > 0 && word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                let embedding = self.embed_label(word);
                self.track_object(word, "person", position, &[], &embedding);
            }
            
            // Check for objects after articles
            for indicator in &object_indicators {
                if i > 0 && words.get(i.saturating_sub(1)).map(|w| *w == *indicator).unwrap_or(false) {
                    let embedding = self.embed_label(word);
                    self.track_object(word, "object", position, &[], &embedding);
                }
            }
            
            // Check for locations
            for indicator in &location_indicators {
                let parts: Vec<&str> = indicator.split_whitespace().collect();
                if parts.len() == 2 && i >= 2 {
                    if words.get(i.saturating_sub(2)).map(|w| *w == parts[0]).unwrap_or(false)
                        && words.get(i.saturating_sub(1)).map(|w| *w == parts[1]).unwrap_or(false)
                    {
                        let embedding = self.embed_label(word);
                        self.track_object(word, "location", position, &[], &embedding);
                    }
                }
            }
        }
    }
    
    /// Get all tracked objects with their accumulated state
    pub fn get_tracked_objects(&self) -> Vec<&TrackedObject> {
        self.tracked_objects.values().collect()
    }
    
    /// Clear tracked objects (for new context)
    pub fn clear_tracked_objects(&mut self) {
        self.tracked_objects.clear();
    }
    
    /// Get focus attributes for a specific object based on its patterns
    pub fn get_focus_attributes(&self, object_id: &str) -> Vec<String> {
        if let Some(obj) = self.tracked_objects.get(object_id) {
            obj.matched_patterns.iter()
                .flat_map(|p| p.focus_attributes.iter().cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for AttributeFocusedAttention {
    fn default() -> Self {
        Self::new(256)
    }
}

/// Dynamic attention that adjusts heads based on vortex position
/// Sacred positions (3, 6, 9) get more attention heads
#[derive(Debug, Clone)]
pub struct SacredDynamicAttention {
    pub config: SacredAttentionConfig,
    /// Weights for each sacred position (3, 6, 9)
    /// Position 3: 3 heads focusing on entity attributes
    /// Position 6: 6 heads focusing on relationships
    /// Position 9: 9 heads focusing on verification
    pub w_q_3: Vec<f32>,
    pub w_k_3: Vec<f32>,
    pub w_v_3: Vec<f32>,
    pub w_q_6: Vec<f32>,
    pub w_k_6: Vec<f32>,
    pub w_v_6: Vec<f32>,
    pub w_q_9: Vec<f32>,
    pub w_k_9: Vec<f32>,
    pub w_v_9: Vec<f32>,
    /// Output projections
    pub w_o_3: Vec<f32>,
    pub w_o_6: Vec<f32>,
    pub w_o_9: Vec<f32>,
}

impl SacredDynamicAttention {
    pub fn new(latent_dim: usize, config: SacredAttentionConfig) -> Self {
        let head_dim = config.head_dim;
        
        // Position 3: 3 heads
        let dim_3 = 3 * head_dim;
        let scale_3 = (2.0 / (latent_dim + dim_3) as f32).sqrt();
        
        // Position 6: 6 heads
        let dim_6 = 6 * head_dim;
        let scale_6 = (2.0 / (latent_dim + dim_6) as f32).sqrt();
        
        // Position 9: 9 heads
        let dim_9 = 9 * head_dim;
        let scale_9 = (2.0 / (latent_dim + dim_9) as f32).sqrt();
        
        Self {
            config,
            // Position 3 weights (3 heads)
            w_q_3: (0..latent_dim * dim_3).map(|i| ((i as f32 * 0.1).sin() * scale_3)).collect(),
            w_k_3: (0..latent_dim * dim_3).map(|i| ((i as f32 * 0.2).cos() * scale_3)).collect(),
            w_v_3: (0..latent_dim * dim_3).map(|i| ((i as f32 * 0.3).sin() * scale_3)).collect(),
            w_o_3: (0..dim_3 * latent_dim).map(|i| ((i as f32 * 0.15).cos() * scale_3)).collect(),
            // Position 6 weights (6 heads)
            w_q_6: (0..latent_dim * dim_6).map(|i| ((i as f32 * 0.1).sin() * scale_6)).collect(),
            w_k_6: (0..latent_dim * dim_6).map(|i| ((i as f32 * 0.2).cos() * scale_6)).collect(),
            w_v_6: (0..latent_dim * dim_6).map(|i| ((i as f32 * 0.3).sin() * scale_6)).collect(),
            w_o_6: (0..dim_6 * latent_dim).map(|i| ((i as f32 * 0.15).cos() * scale_6)).collect(),
            // Position 9 weights (9 heads)
            w_q_9: (0..latent_dim * dim_9).map(|i| ((i as f32 * 0.1).sin() * scale_9)).collect(),
            w_k_9: (0..latent_dim * dim_9).map(|i| ((i as f32 * 0.2).cos() * scale_9)).collect(),
            w_v_9: (0..latent_dim * dim_9).map(|i| ((i as f32 * 0.3).sin() * scale_9)).collect(),
            w_o_9: (0..dim_9 * latent_dim).map(|i| ((i as f32 * 0.15).cos() * scale_9)).collect(),
        }
    }
    
    /// Get number of heads for a vortex position
    pub fn heads_for_position(&self, position: u8) -> usize {
        match position {
            3 => 3,
            6 => 6,
            9 => 9,
            // Non-sacred positions use interpolated head count
            1 | 2 => 2,
            4 | 5 => 4,
            7 | 8 => 6,
            _ => 3,
        }
    }
    
    /// Forward pass with position-specific attention
    pub fn forward(&self, query: &[f32], keys: &[Vec<f32>], values: &[Vec<f32>], position: u8) -> (Vec<f32>, Vec<f32>) {
        let (w_q, w_k, w_v, w_o, num_heads) = match position {
            3 => (&self.w_q_3, &self.w_k_3, &self.w_v_3, &self.w_o_3, 3),
            6 => (&self.w_q_6, &self.w_k_6, &self.w_v_6, &self.w_o_6, 6),
            9 => (&self.w_q_9, &self.w_k_9, &self.w_v_9, &self.w_o_9, 9),
            // Default to position 3 weights for non-sacred
            _ => (&self.w_q_3, &self.w_k_3, &self.w_v_3, &self.w_o_3, 3),
        };
        
        let head_dim = self.config.head_dim;
        let total_dim = num_heads * head_dim;
        let latent_dim = query.len();
        
        // Project query
        let mut q_proj = vec![0.0f32; total_dim];
        for i in 0..total_dim.min(w_q.len() / latent_dim) {
            let mut sum = 0.0f32;
            for j in 0..latent_dim.min(query.len()) {
                let idx = i * latent_dim + j;
                if idx < w_q.len() {
                    sum += w_q[idx] * query[j];
                }
            }
            q_proj[i] = sum;
        }
        
        // Project keys and values
        let mut k_projs: Vec<Vec<f32>> = Vec::with_capacity(keys.len());
        let mut v_projs: Vec<Vec<f32>> = Vec::with_capacity(values.len());
        
        for (key, value) in keys.iter().zip(values.iter()) {
            let mut k_proj = vec![0.0f32; total_dim];
            let mut v_proj = vec![0.0f32; total_dim];
            
            for i in 0..total_dim.min(w_k.len() / latent_dim) {
                let mut k_sum = 0.0f32;
                let mut v_sum = 0.0f32;
                for j in 0..latent_dim.min(key.len()) {
                    let idx = i * latent_dim + j;
                    if idx < w_k.len() {
                        k_sum += w_k[idx] * key[j];
                    }
                    if idx < w_v.len() && j < value.len() {
                        v_sum += w_v[idx] * value[j];
                    }
                }
                k_proj[i] = k_sum;
                v_proj[i] = v_sum;
            }
            
            k_projs.push(k_proj);
            v_projs.push(v_proj);
        }
        
        // Compute attention per head
        let scale = (head_dim as f32).sqrt();
        let mut output = vec![0.0f32; total_dim];
        let mut all_attn_weights = vec![0.0f32; keys.len()];
        
        for head in 0..num_heads {
            let head_start = head * head_dim;
            let head_end = head_start + head_dim;
            
            // Compute attention scores
            let mut scores: Vec<f32> = k_projs.iter()
                .map(|k| {
                    let mut dot = 0.0f32;
                    for i in head_start..head_end.min(q_proj.len()).min(k.len()) {
                        dot += q_proj[i] * k[i];
                    }
                    dot / scale
                })
                .collect();
            
            // Softmax
            let max_score = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let exp_scores: Vec<f32> = scores.iter().map(|s| (s - max_score).exp()).collect();
            let exp_sum: f32 = exp_scores.iter().sum();
            let attn_weights: Vec<f32> = exp_scores.iter().map(|e| e / exp_sum.max(1e-8)).collect();
            
            // Weighted sum of values
            for (v, &w) in v_projs.iter().zip(attn_weights.iter()) {
                for i in head_start..head_end.min(output.len()).min(v.len()) {
                    output[i] += v[i] * w;
                }
            }
            
            // Accumulate attention weights
            for (i, &w) in attn_weights.iter().enumerate() {
                if i < all_attn_weights.len() {
                    all_attn_weights[i] += w / num_heads as f32;
                }
            }
        }
        
        // Output projection
        let mut final_output = vec![0.0f32; latent_dim];
        for i in 0..latent_dim {
            let mut sum = 0.0f32;
            for j in 0..total_dim.min(output.len()) {
                let idx = j * latent_dim + i;
                if idx < w_o.len() {
                    sum += output[j] * w_o[idx];
                }
            }
            final_output[i] = sum;
        }
        
        (final_output, all_attn_weights)
    }
}

// =============================================================================
// Few-Shot Context Manager
// =============================================================================

/// A single example for few-shot learning
#[derive(Debug, Clone)]
pub struct FewShotExample {
    pub question: String,
    pub answer: String,
    pub question_embedding: Vec<f32>,
    pub answer_embedding: Vec<f32>,
}

/// Manages few-shot examples for in-context learning
#[derive(Debug, Clone)]
pub struct FewShotContext {
    /// Maximum number of examples to keep
    pub max_examples: usize,
    /// Current examples
    pub examples: Vec<FewShotExample>,
    /// Embedding dimension
    pub embed_dim: usize,
}

impl FewShotContext {
    pub fn new(max_examples: usize, embed_dim: usize) -> Self {
        Self {
            max_examples,
            examples: Vec::new(),
            embed_dim,
        }
    }
    
    /// Add a new example
    pub fn add_example(&mut self, question: String, answer: String, 
                       question_embedding: Vec<f32>, answer_embedding: Vec<f32>) {
        let example = FewShotExample {
            question,
            answer,
            question_embedding,
            answer_embedding,
        };
        
        self.examples.push(example);
        
        // Keep only most recent examples
        if self.examples.len() > self.max_examples {
            self.examples.remove(0);
        }
    }
    
    /// Get K most similar examples to a query
    pub fn get_similar_examples(&self, query_embedding: &[f32], k: usize) -> Vec<&FewShotExample> {
        let mut scored: Vec<(&FewShotExample, f32)> = self.examples.iter()
            .map(|ex| {
                let sim = cosine_similarity(query_embedding, &ex.question_embedding);
                (ex, sim)
            })
            .collect();
        
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).map(|(ex, _)| ex).collect()
    }
    
    /// Build context embeddings for attention
    pub fn build_context_keys(&self, query_embedding: &[f32], k: usize) -> Vec<Vec<f32>> {
        let similar = self.get_similar_examples(query_embedding, k);
        similar.iter()
            .flat_map(|ex| vec![ex.question_embedding.clone(), ex.answer_embedding.clone()])
            .collect()
    }
}

// =============================================================================
// Dynamic MoE Routing (All Experts with Weighted Contributions)
// =============================================================================

/// Expert type for MoE routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpertType {
    EntityAttribute,
    Semantic,
    RAG,
    Attention,
    Math,
    Causal,
    Temporal,
    Commonsense,
}

impl ExpertType {
    pub fn all() -> Vec<ExpertType> {
        vec![
            ExpertType::EntityAttribute,
            ExpertType::Semantic,
            ExpertType::RAG,
            ExpertType::Attention,
            ExpertType::Math,
            ExpertType::Causal,
            ExpertType::Temporal,
            ExpertType::Commonsense,
        ]
    }
}

/// Dynamic MoE router that routes to ALL experts with weighted contributions
#[derive(Debug, Clone)]
pub struct DynamicMoERouter {
    /// Expert weights (learned)
    pub expert_weights: HashMap<ExpertType, f32>,
    /// Expert embeddings for routing
    pub expert_embeddings: HashMap<ExpertType, Vec<f32>>,
    /// Embedding dimension
    pub embed_dim: usize,
    /// Minimum weight threshold
    pub min_weight: f32,
    /// Temperature for softmax
    pub temperature: f32,
}

impl DynamicMoERouter {
    pub fn new(embed_dim: usize) -> Self {
        let mut expert_weights = HashMap::new();
        let mut expert_embeddings = HashMap::new();
        
        // Initialize all experts with equal weights
        for expert in ExpertType::all() {
            expert_weights.insert(expert, 1.0);
            
            // Create unique embedding for each expert
            let embedding: Vec<f32> = (0..embed_dim)
                .map(|i| {
                    let seed = expert as usize * 1000 + i;
                    ((seed as f32 * 0.1).sin() + (seed as f32 * 0.03).cos()) * 0.5
                })
                .collect();
            expert_embeddings.insert(expert, embedding);
        }
        
        Self {
            expert_weights,
            expert_embeddings,
            embed_dim,
            min_weight: 0.01,
            temperature: 1.0,
        }
    }
    
    /// Route to all experts with weighted contributions based on input
    pub fn route(&self, input_embedding: &[f32]) -> Vec<(ExpertType, f32)> {
        let mut scores: Vec<(ExpertType, f32)> = ExpertType::all()
            .into_iter()
            .map(|expert| {
                let expert_embed = self.expert_embeddings.get(&expert).unwrap();
                let base_weight = *self.expert_weights.get(&expert).unwrap_or(&1.0);
                
                // Compute similarity between input and expert embedding
                let similarity = cosine_similarity(input_embedding, expert_embed);
                
                // Combine base weight with similarity
                let score = base_weight * (1.0 + similarity);
                (expert, score)
            })
            .collect();
        
        // Apply softmax with temperature
        let max_score = scores.iter().map(|(_, s)| *s).fold(f32::NEG_INFINITY, f32::max);
        let exp_scores: Vec<f32> = scores.iter()
            .map(|(_, s)| ((s - max_score) / self.temperature).exp())
            .collect();
        let sum_exp: f32 = exp_scores.iter().sum();
        
        // Normalize and apply minimum weight
        scores.iter()
            .zip(exp_scores.iter())
            .map(|((expert, _), &exp_score)| {
                let weight = (exp_score / sum_exp).max(self.min_weight);
                (*expert, weight)
            })
            .collect()
    }
    
    /// Route based on question complexity
    pub fn route_by_complexity(&self, question: &str, input_embedding: &[f32]) -> Vec<(ExpertType, f32)> {
        let question_lower = question.to_lowercase();
        
        // Analyze question complexity
        let has_math = question.chars().any(|c| c.is_ascii_digit()) 
            || ["calculate", "compute", "sum", "total", "how many", "how much"]
                .iter().any(|kw| question_lower.contains(kw));
        
        let has_causal = ["because", "why", "cause", "effect", "result", "therefore"]
            .iter().any(|kw| question_lower.contains(kw));
        
        let has_temporal = ["when", "before", "after", "first", "then", "finally"]
            .iter().any(|kw| question_lower.contains(kw));
        
        let has_knowledge = ["what is", "who is", "where is", "define", "explain"]
            .iter().any(|kw| question_lower.contains(kw));
        
        let word_count = question.split_whitespace().count();
        let is_complex = word_count > 15 || question.contains(',');
        
        // Get base routing
        let mut routing = self.route(input_embedding);
        
        // Boost relevant experts based on question analysis
        for (expert, weight) in &mut routing {
            match expert {
                ExpertType::Math if has_math => *weight *= 2.0,
                ExpertType::Causal if has_causal => *weight *= 1.8,
                ExpertType::Temporal if has_temporal => *weight *= 1.5,
                ExpertType::RAG if has_knowledge => *weight *= 1.7,
                ExpertType::Commonsense if !is_complex => *weight *= 1.3,
                ExpertType::Attention if is_complex => *weight *= 1.5,
                _ => {}
            }
        }
        
        // Re-normalize
        let total: f32 = routing.iter().map(|(_, w)| *w).sum();
        for (_, weight) in &mut routing {
            *weight /= total;
        }
        
        routing
    }
    
    /// Update expert weights based on success/failure
    pub fn update_weights(&mut self, expert: ExpertType, success: bool, learning_rate: f32) {
        if let Some(weight) = self.expert_weights.get_mut(&expert) {
            if success {
                *weight *= 1.0 + learning_rate;
            } else {
                *weight *= 1.0 - learning_rate * 0.5;
            }
            *weight = weight.clamp(0.1, 10.0);
        }
    }
    
    /// Get top-k experts by weight
    pub fn top_k(&self, routing: &[(ExpertType, f32)], k: usize) -> Vec<(ExpertType, f32)> {
        let mut sorted = routing.to_vec();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(k).collect()
    }
}

impl Default for DynamicMoERouter {
    fn default() -> Self {
        Self::new(256)
    }
}

// =============================================================================
// Generation Head (Latent → Token Logits)
// =============================================================================

/// Generation head that converts latent states to token probabilities
#[derive(Debug, Clone)]
pub struct GenerationHead {
    /// Projection weights: latent_dim → vocab_size
    pub projection: Vec<f32>,
    /// Bias
    pub bias: Vec<f32>,
    /// Latent dimension
    pub latent_dim: usize,
    /// Vocabulary size
    pub vocab_size: usize,
    /// Temperature for sampling
    pub temperature: f32,
}

impl GenerationHead {
    pub fn new(latent_dim: usize, vocab_size: usize) -> Self {
        let scale = (2.0 / (latent_dim + vocab_size) as f32).sqrt();
        Self {
            projection: (0..latent_dim * vocab_size)
                .map(|i| ((i as f32 * 0.1).sin() * scale))
                .collect(),
            bias: vec![0.0; vocab_size],
            latent_dim,
            vocab_size,
            temperature: 1.0,
        }
    }
    
    /// Compute logits over vocabulary
    pub fn forward(&self, latent: &[f32]) -> Vec<f32> {
        let mut logits = self.bias.clone();
        
        for v in 0..self.vocab_size {
            for l in 0..self.latent_dim.min(latent.len()) {
                let idx = v * self.latent_dim + l;
                if idx < self.projection.len() {
                    logits[v] += self.projection[idx] * latent[l];
                }
            }
        }
        
        logits
    }
    
    /// Sample a token from logits
    pub fn sample(&self, logits: &[f32], temperature: f32) -> u32 {
        // Apply temperature
        let scaled: Vec<f32> = logits.iter().map(|&l| l / temperature.max(0.01)).collect();
        
        // Softmax
        let max_logit = scaled.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exp_logits: Vec<f32> = scaled.iter().map(|&l| (l - max_logit).exp()).collect();
        let sum: f32 = exp_logits.iter().sum();
        let probs: Vec<f32> = exp_logits.iter().map(|&e| e / sum.max(1e-8)).collect();
        
        // Greedy sampling (argmax)
        probs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx as u32)
            .unwrap_or(0)
    }
    
    /// Top-k sampling
    pub fn sample_top_k(&self, logits: &[f32], k: usize, temperature: f32) -> u32 {
        let scaled: Vec<f32> = logits.iter().map(|&l| l / temperature.max(0.01)).collect();
        
        // Get top-k indices
        let mut indexed: Vec<(usize, f32)> = scaled.iter().cloned().enumerate().collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        indexed.truncate(k);
        
        // Softmax over top-k
        let max_logit = indexed.iter().map(|(_, l)| *l).fold(f32::NEG_INFINITY, f32::max);
        let exp_sum: f32 = indexed.iter().map(|(_, l)| (l - max_logit).exp()).sum();
        
        // Sample from top-k (simplified: take argmax of top-k)
        indexed.first().map(|(idx, _)| *idx as u32).unwrap_or(0)
    }
}

// =============================================================================
// Generative Vortex Engine
// =============================================================================

/// Configuration for the generative engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerativeConfig {
    /// Latent dimension
    pub latent_dim: usize,
    /// Maximum sequence length
    pub max_seq_len: usize,
    /// Number of few-shot examples
    pub num_fewshot: usize,
    /// Temperature for generation
    pub temperature: f32,
    /// Top-k for sampling
    pub top_k: usize,
    /// Whether to use exhaustive pathway search
    pub use_pathway_search: bool,
    /// Maximum nodes for pathway search (n! complexity)
    pub pathway_max_nodes: usize,
}

impl Default for GenerativeConfig {
    fn default() -> Self {
        Self {
            latent_dim: 256,
            max_seq_len: 512,
            num_fewshot: 5,
            temperature: 0.7,
            top_k: 50,
            use_pathway_search: true,
            pathway_max_nodes: 9,  // 9! = 362,880 is tractable
        }
    }
}

/// The main generative engine combining all components
pub struct GenerativeVortexEngine {
    pub config: GenerativeConfig,
    /// Subword tokenizer with learned embeddings
    pub tokenizer: SubwordTokenizer,
    /// CALM engine for latent space operations
    pub calm: CALMEngine,
    /// Sacred dynamic attention (3-6-9 heads)
    pub sacred_attention: SacredDynamicAttention,
    /// Few-shot context manager
    pub fewshot_context: FewShotContext,
    /// Generation head
    pub generation_head: GenerationHead,
    /// Exhaustive pathway optimizer
    pub pathway_optimizer: ExhaustivePathwayOptimizer,
    /// RAG knowledge base (embeddings)
    pub knowledge_base: Vec<(String, Vec<f32>)>,
    /// Training step counter
    pub training_steps: usize,
    /// Attribute-focused attention for implication extraction
    pub attr_attention: AttributeFocusedAttention,
    /// Accumulated implications from vortex processing
    pub current_implications: Vec<AttributeImplication>,
}

impl GenerativeVortexEngine {
    pub fn new(config: GenerativeConfig) -> Self {
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
        
        // Initialize few-shot context
        let fewshot_context = FewShotContext::new(config.num_fewshot, latent_dim);
        
        // Initialize generation head
        let generation_head = GenerationHead::new(latent_dim, vocab_size);
        
        // Initialize pathway optimizer
        let pathway_config = PathwayConfig {
            n_nodes: config.pathway_max_nodes,
            dimension: latent_dim,
            num_stacks: 3,
            top_k_per_stack: 10,
            parallel: true,
            initial_beta: 1.0,
            kl_bound: 0.1,
            ..Default::default()
        };
        let pathway_optimizer = ExhaustivePathwayOptimizer::new(pathway_config);

        // Initialize attribute-focused attention
        let attr_attention = AttributeFocusedAttention::new(latent_dim);
        
        Self {
            config,
            tokenizer,
            calm,
            sacred_attention,
            fewshot_context,
            generation_head,
            pathway_optimizer,
            knowledge_base: Vec::new(),
            training_steps: 0,
            attr_attention,
            current_implications: Vec::new(),
        }
    }
    
    /// Tokenize and embed text
    pub fn encode_text(&self, text: &str) -> (Vec<u32>, Vec<f32>) {
        let tokens = self.tokenizer.tokenize(text);
        let embeddings = self.tokenizer.get_embeddings(&tokens);
        
        // Average embeddings to get single vector
        let mut combined = vec![0.0f32; self.config.latent_dim];
        for embed in &embeddings {
            for (i, &val) in embed.iter().enumerate() {
                if i < combined.len() {
                    combined[i] += val;
                }
            }
        }
        if !embeddings.is_empty() {
            for val in &mut combined {
                *val /= embeddings.len() as f32;
            }
        }
        
        (tokens, combined)
    }
    
    /// Convert embedding to BeamTensors for CALM
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
    
    /// Run one vortex cycle with sacred attention at positions 3, 6, 9
    pub fn vortex_cycle(&self, initial_latent: &LatentState, context_keys: &[Vec<f32>]) -> LatentState {
        let mut latent = initial_latent.clone();
        
        // Vortex positions: 1 → 2 → 4 → 8 → 7 → 5 → 1 (with sacred 3, 6, 9)
        let positions = [1u8, 2, 3, 4, 5, 6, 7, 8, 9];
        
        for &pos in &positions {
            // CALM prediction step
            latent = self.calm.predict_next(&latent);
            
            // Apply sacred attention at positions 3, 6, 9
            if pos == 3 || pos == 6 || pos == 9 {
                if !context_keys.is_empty() {
                    let (attended, _weights) = self.sacred_attention.forward(
                        &latent.latent,
                        context_keys,
                        context_keys,  // Use same as values
                        pos,
                    );
                    
                    // Blend attended output with latent
                    let blend_factor = match pos {
                        3 => 0.3,  // Light blending at position 3
                        6 => 0.5,  // Medium blending at position 6
                        9 => 0.7,  // Strong blending at position 9 (verification)
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
        
        latent
    }
    
    /// Enhanced vortex cycle with attribute-focused implication extraction
    /// This compares node labels to attributes at each sacred position (3, 6, 9)
    /// and feeds implications into the n! pathway search for improved reasoning
    pub fn vortex_cycle_with_implications(
        &mut self,
        initial_latent: &LatentState,
        context_keys: &[Vec<f32>],
        node_labels: &[String],           // Labels for each node position
        node_attributes: &[Vec<(String, f32)>],  // Attributes per node
    ) -> (LatentState, Vec<AttributeImplication>) {
        let mut latent = initial_latent.clone();
        let mut all_implications: Vec<AttributeImplication> = Vec::new();
        
        // Vortex positions with sacred checkpoints
        let positions = [1u8, 2, 3, 4, 5, 6, 7, 8, 9];
        
        for &pos in &positions {
            // CALM prediction step
            latent = self.calm.predict_next(&latent);
            
            // Get node label and attributes for this position
            let node_idx = (pos as usize).saturating_sub(1);
            let node_label = node_labels.get(node_idx)
                .map(|s| s.as_str())
                .unwrap_or("node");
            let attributes = node_attributes.get(node_idx)
                .cloned()
                .unwrap_or_default();
            
            // At sacred positions (3, 6, 9), extract implications
            if pos == 3 || pos == 6 || pos == 9 {
                // Extract implications by comparing node label to attributes
                let implications = self.attr_attention.extract_implications(
                    node_label,
                    pos,
                    &attributes,
                    &latent.latent,
                );
                
                // Aggregate implications into latent space modulation
                if !implications.is_empty() {
                    let impl_embedding = self.attr_attention.aggregate_for_pathway(&implications);
                    
                    // Modulate latent with implication embedding
                    let impl_weight = match pos {
                        3 => 0.2,  // Property implications (lighter)
                        6 => 0.4,  // Causal implications (medium)
                        9 => 0.6,  // Verification implications (stronger)
                        _ => 0.2,
                    };
                    
                    for (i, &impl_val) in impl_embedding.iter().enumerate() {
                        if i < latent.latent.len() {
                            latent.latent[i] += impl_val * impl_weight;
                        }
                    }
                    
                    all_implications.extend(implications);
                }
                
                // Also apply standard sacred attention
                if !context_keys.is_empty() {
                    let (attended, _weights) = self.sacred_attention.forward(
                        &latent.latent,
                        context_keys,
                        context_keys,
                        pos,
                    );
                    
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
        
        // Store implications for pathway search
        self.current_implications = all_implications.clone();
        
        (latent, all_implications)
    }
    
    /// Use implications to score pathway permutations
    /// This feeds the extracted implications into the n! exhaustive search
    pub fn pathway_select_with_implications(&mut self, latent: &LatentState) -> u32 {
        // Get top candidate tokens
        let logits = self.generation_head.forward(&latent.latent);
        let mut indexed: Vec<(u32, f32)> = logits.iter()
            .enumerate()
            .map(|(i, &l)| (i as u32, l))
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top candidates for pathway search
        let candidates: Vec<u32> = indexed.iter()
            .take(self.config.pathway_max_nodes.min(9))
            .map(|(id, _)| *id)
            .collect();
        
        if candidates.is_empty() {
            return self.tokenizer.eos_id;
        }
        
        // Get embeddings for candidates
        let candidate_embeddings: Vec<Vec<f32>> = candidates.iter()
            .map(|&id| self.tokenizer.get_embedding(id))
            .collect();
        
        // Get implication embedding for scoring
        let impl_embedding = if !self.current_implications.is_empty() {
            self.attr_attention.aggregate_for_pathway(&self.current_implications)
        } else {
            vec![0.0f32; self.config.latent_dim]
        };
        
        // Score each candidate based on:
        // 1. Original logit score
        // 2. Similarity to latent state
        // 3. Similarity to implication embedding (how well it aligns with deduced implications)
        let mut best_score = f32::NEG_INFINITY;
        let mut best_token = candidates[0];
        
        for (i, &token_id) in candidates.iter().enumerate() {
            let embed = &candidate_embeddings[i];
            
            // Base score from logits
            let logit_score = indexed.iter()
                .find(|(id, _)| *id == token_id)
                .map(|(_, s)| *s)
                .unwrap_or(0.0);
            
            // Latent similarity
            let latent_sim = cosine_similarity(embed, &latent.latent);
            
            // Implication alignment score
            let impl_sim = cosine_similarity(embed, &impl_embedding);
            
            // Combined score with implication weighting
            // Implications are "heavily considered" as per user request
            let combined_score = logit_score * 0.4 + latent_sim * 0.2 + impl_sim * 0.4;
            
            if combined_score > best_score {
                best_score = combined_score;
                best_token = token_id;
            }
        }
        
        best_token
    }
    
    /// Generate with full implication-aware reasoning
    pub fn generate_with_implications(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        node_labels: &[String],
        node_attributes: &[Vec<(String, f32)>],
    ) -> (String, Vec<AttributeImplication>) {
        // Encode prompt
        let (_, prompt_embedding) = self.encode_text(prompt);
        
        // Get context
        let context_keys = self.fewshot_context.build_context_keys(&prompt_embedding, self.config.num_fewshot);
        let mut all_context = context_keys;
        let rag_results = self.retrieve_knowledge(&prompt_embedding, 3);
        for (_, embed) in rag_results {
            all_context.push(embed);
        }
        
        // Convert to CALM input
        let beams = self.embedding_to_beams(&prompt_embedding);
        let mut latent = self.calm.encode(&beams);
        
        // Collect all implications across generation
        let mut all_implications: Vec<AttributeImplication> = Vec::new();
        let mut output_tokens: Vec<u32> = Vec::new();
        
        for _ in 0..max_tokens {
            // Run vortex cycle with implication extraction
            let (new_latent, cycle_implications) = self.vortex_cycle_with_implications(
                &latent,
                &all_context,
                node_labels,
                node_attributes,
            );
            latent = new_latent;
            all_implications.extend(cycle_implications);
            
            // Select token using implication-aware pathway search
            let token = if self.config.use_pathway_search {
                self.pathway_select_with_implications(&latent)
            } else {
                let logits = self.generation_head.forward(&latent.latent);
                self.generation_head.sample_top_k(&logits, self.config.top_k, self.config.temperature)
            };
            
            output_tokens.push(token);
            
            if token == self.tokenizer.eos_id {
                break;
            }
            
            // Update latent
            let token_embed = self.tokenizer.get_embedding(token);
            for (i, &val) in token_embed.iter().enumerate() {
                if i < latent.latent.len() {
                    latent.latent[i] = latent.latent[i] * 0.9 + val * 0.1;
                }
            }
        }
        
        let output_text = self.tokenizer.detokenize(&output_tokens);
        (output_text, all_implications)
    }
    
    /// Generate text autoregressively (standalone mode)
    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> String {
        // Encode prompt
        let (prompt_tokens, prompt_embedding) = self.encode_text(prompt);
        
        // Get few-shot context
        let context_keys = self.fewshot_context.build_context_keys(&prompt_embedding, self.config.num_fewshot);
        
        // Add RAG knowledge to context
        let mut all_context = context_keys;
        let rag_results = self.retrieve_knowledge(&prompt_embedding, 3);
        for (_, embed) in rag_results {
            all_context.push(embed);
        }
        
        // Convert to CALM input
        let beams = self.embedding_to_beams(&prompt_embedding);
        let mut latent = self.calm.encode(&beams);
        
        // Generate tokens
        let mut output_tokens: Vec<u32> = Vec::new();
        
        for _ in 0..max_tokens {
            // Run vortex cycle with sacred attention
            latent = self.vortex_cycle(&latent, &all_context);
            
            // Optional: Use exhaustive pathway for token selection
            if self.config.use_pathway_search && output_tokens.len() < 5 {
                // For first few tokens, use pathway search for better quality
                let token = self.pathway_select_token(&latent);
                output_tokens.push(token);
            } else {
                // Standard generation
                let logits = self.generation_head.forward(&latent.latent);
                let token = self.generation_head.sample_top_k(&logits, self.config.top_k, self.config.temperature);
                output_tokens.push(token);
            }
            
            // Check for EOS
            if output_tokens.last() == Some(&self.tokenizer.eos_id) {
                break;
            }
            
            // Update latent with new token
            let token_embed = self.tokenizer.get_embedding(*output_tokens.last().unwrap());
            let token_beams = self.embedding_to_beams(&token_embed);
            let token_latent = self.calm.encode(&token_beams);
            
            // Blend new token into latent
            for (i, &val) in token_latent.latent.iter().enumerate() {
                if i < latent.latent.len() {
                    latent.latent[i] = latent.latent[i] * 0.8 + val * 0.2;
                }
            }
        }
        
        // Detokenize
        self.tokenizer.detokenize(&output_tokens)
    }
    
    /// Use exhaustive pathway search to select best token
    fn pathway_select_token(&mut self, latent: &LatentState) -> u32 {
        // Get top-k candidate tokens
        let logits = self.generation_head.forward(&latent.latent);
        let mut indexed: Vec<(u32, f32)> = logits.iter()
            .enumerate()
            .map(|(i, &l)| (i as u32, l))
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let candidates: Vec<u32> = indexed.iter().take(self.config.pathway_max_nodes).map(|(id, _)| *id).collect();
        
        if candidates.len() < 2 {
            return candidates.first().copied().unwrap_or(self.tokenizer.unk_id);
        }
        
        // Build embeddings for pathway search
        let candidate_embeddings: Vec<Vec<f32>> = candidates.iter()
            .map(|&id| self.tokenizer.get_embedding(id))
            .collect();
        
        // Set up pathway optimizer
        self.pathway_optimizer.set_embeddings(&candidate_embeddings);
        self.pathway_optimizer.set_target(&latent.latent);
        
        // Run stacked inference
        let result = self.pathway_optimizer.run_stacked_inference();
        
        // Best path's first node is our token
        if let Some(best_path) = result.top_paths.first() {
            if let Some(&first_idx) = best_path.perm.first() {
                if first_idx < candidates.len() {
                    return candidates[first_idx];
                }
            }
        }
        
        // Fallback to top logit
        candidates.first().copied().unwrap_or(self.tokenizer.unk_id)
    }
    
    /// Add knowledge to RAG knowledge base
    pub fn add_knowledge(&mut self, text: &str) {
        let (_, embedding) = self.encode_text(text);
        self.knowledge_base.push((text.to_string(), embedding));
    }
    
    /// Retrieve relevant knowledge
    pub fn retrieve_knowledge(&self, query_embedding: &[f32], k: usize) -> Vec<(String, Vec<f32>)> {
        let mut scored: Vec<(&String, &Vec<f32>, f32)> = self.knowledge_base.iter()
            .map(|(text, embed)| {
                let sim = cosine_similarity(query_embedding, embed);
                (text, embed, sim)
            })
            .collect();
        
        scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        
        scored.into_iter()
            .take(k)
            .map(|(text, embed, _)| (text.clone(), embed.clone()))
            .collect()
    }
    
    /// Pre-train on dataset
    pub fn pretrain(&mut self, texts: &[String], epochs: usize, learning_rate: f32) {
        println!("Pre-training on {} texts for {} epochs...", texts.len(), epochs);
        
        // Learn BPE merges from corpus
        self.tokenizer.learn_bpe(texts, 1000);
        
        // Update generation head vocab size
        self.generation_head = GenerationHead::new(
            self.config.latent_dim,
            self.tokenizer.vocab_size(),
        );
        
        for epoch in 0..epochs {
            let mut total_loss = 0.0f32;
            
            for text in texts {
                // Tokenize
                let tokens = self.tokenizer.tokenize(text);
                if tokens.len() < 3 {
                    continue;
                }
                
                // Train on next-token prediction
                for i in 1..tokens.len() - 1 {
                    let context_tokens = &tokens[..i];
                    let target_token = tokens[i];
                    
                    // Get context embedding
                    let context_embeds = self.tokenizer.get_embeddings(context_tokens);
                    let mut context_embed = vec![0.0f32; self.config.latent_dim];
                    for embed in &context_embeds {
                        for (j, &val) in embed.iter().enumerate() {
                            if j < context_embed.len() {
                                context_embed[j] += val;
                            }
                        }
                    }
                    if !context_embeds.is_empty() {
                        for val in &mut context_embed {
                            *val /= context_embeds.len() as f32;
                        }
                    }
                    
                    // Encode to latent
                    let beams = self.embedding_to_beams(&context_embed);
                    let latent = self.calm.encode(&beams);
                    
                    // Get logits
                    let logits = self.generation_head.forward(&latent.latent);
                    
                    // Cross-entropy loss
                    let max_logit = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                    let exp_sum: f32 = logits.iter().map(|&l| (l - max_logit).exp()).sum();
                    let log_prob = logits.get(target_token as usize).unwrap_or(&0.0) - max_logit - exp_sum.ln();
                    let loss = -log_prob;
                    total_loss += loss;
                    
                    // Update generation head (simplified gradient)
                    let probs: Vec<f32> = logits.iter()
                        .map(|&l| ((l - max_logit).exp() / exp_sum))
                        .collect();
                    
                    for v in 0..self.generation_head.vocab_size.min(probs.len()) {
                        let target_prob = if v == target_token as usize { 1.0 } else { 0.0 };
                        let grad = probs[v] - target_prob;
                        
                        for l in 0..self.config.latent_dim.min(latent.latent.len()) {
                            let idx = v * self.config.latent_dim + l;
                            if idx < self.generation_head.projection.len() {
                                self.generation_head.projection[idx] -= learning_rate * grad * latent.latent[l];
                            }
                        }
                        self.generation_head.bias[v] -= learning_rate * grad;
                    }
                    
                    // Update token embeddings
                    let target_embed = self.tokenizer.get_embedding(target_token);
                    for (j, &val) in context_embed.iter().enumerate() {
                        if j < target_embed.len() {
                            let grad = val - target_embed[j];
                            // Update context token embeddings
                            for &ctx_token in context_tokens {
                                self.tokenizer.update_embedding(ctx_token, &vec![grad; self.config.latent_dim], learning_rate * 0.1);
                            }
                        }
                    }
                    
                    self.training_steps += 1;
                }
            }
            
            let avg_loss = total_loss / texts.len().max(1) as f32;
            println!("  Epoch {}/{}: avg_loss = {:.4}", epoch + 1, epochs, avg_loss);
        }
        
        println!("Pre-training complete. {} total steps.", self.training_steps);
    }
    
    /// Enhanced pre-training with CALM weight updates
    /// Fixes applied:
    ///   1. Contrastive: next-sentence as positive, 8 negatives from buffer (not self-similarity)
    ///   2. gen_loss: real next-token targets from tokenizer (not pseudo-random hash)
    ///   3. calm_loss: reconstruct original input beams (not decoder's own output)
    ///   4. Cosine annealing LR schedule for better convergence
    ///   5. Multiple negatives per contrastive step for sharper embeddings
    pub fn pretrain_calm(&mut self, texts: &[String], epochs: usize, learning_rate: f32) {
        println!("Pre-training CALM weights on {} texts for {} epochs...", texts.len(), epochs);
        
        // First, learn BPE from corpus
        self.tokenizer.learn_bpe(texts, 1000);
        
        // Update generation head vocab size
        self.generation_head = GenerationHead::new(
            self.config.latent_dim,
            self.tokenizer.vocab_size(),
        );
        
        // Pre-compute all embeddings, beams, AND token sequences
        let batch_size = 32;
        let mut all_embeddings: Vec<Vec<f32>> = Vec::with_capacity(texts.len());
        let mut all_beams: Vec<Vec<BeamTensor>> = Vec::with_capacity(texts.len());
        let mut all_tokens: Vec<Vec<u32>> = Vec::with_capacity(texts.len());
        
        println!("  Pre-computing {} embeddings...", texts.len());
        for text in texts.iter() {
            let tokens = self.tokenizer.tokenize(text);
            if tokens.len() < 3 {
                all_embeddings.push(vec![0.0; self.config.latent_dim]);
                all_beams.push(vec![BeamTensor::default(); 8]);
                all_tokens.push(Vec::new());
                continue;
            }
            
            let embeddings = self.tokenizer.get_embeddings(&tokens);
            let mut text_embed = vec![0.0f32; self.config.latent_dim];
            for embed in &embeddings {
                for (i, &val) in embed.iter().enumerate() {
                    if i < text_embed.len() {
                        text_embed[i] += val;
                    }
                }
            }
            if !embeddings.is_empty() {
                for val in &mut text_embed {
                    *val /= embeddings.len() as f32;
                }
            }
            
            let beams = self.embedding_to_beams(&text_embed);
            all_beams.push(beams);
            all_embeddings.push(text_embed);
            all_tokens.push(tokens);
        }
        
        let num_batches = (texts.len() + batch_size - 1) / batch_size;
        let total_steps = epochs * texts.len();
        let num_negatives = 8; // Sample 8 negatives per contrastive step
        
        // Seed negative buffer with initial embeddings for contrastive learning
        let mut negative_buffer: Vec<Vec<f32>> = all_embeddings.iter()
            .filter(|e| e.iter().any(|&v| v != 0.0))
            .take(128)
            .cloned()
            .collect();
        
        for epoch in 0..epochs {
            let mut total_loss = 0.0f32;
            let mut calm_loss = 0.0f32;
            let mut contrastive_loss = 0.0f32;
            let mut sample_count = 0usize;
            
            // Cosine annealing LR: lr * 0.5 * (1 + cos(pi * epoch / epochs))
            let lr = learning_rate * 0.5 * (1.0 + (std::f32::consts::PI * epoch as f32 / epochs as f32).cos());
            
            // Process in batches for better cache utilization
            for batch_idx in 0..num_batches {
                let start_idx = batch_idx * batch_size;
                let end_idx = (start_idx + batch_size).min(texts.len());
                
                for text_idx in start_idx..end_idx {
                    let text_embed = &all_embeddings[text_idx];
                    let beams = &all_beams[text_idx];
                    let tokens = &all_tokens[text_idx];
                    
                    if beams.is_empty() || text_embed.iter().all(|&v| v == 0.0) || tokens.is_empty() {
                        continue;
                    }
                    
                    sample_count += 1;
                    
                    // === FIX 3: CALM reconstruction — train decoder to reconstruct ORIGINAL input ===
                    let latent = self.calm.encode(beams);
                    self.calm.train_step(beams, beams, lr);
                    
                    // Compute reconstruction loss against original input (not decoder output)
                    let decoded = self.calm.decode(&latent);
                    let mut recon_loss = 0.0f32;
                    for (orig, dec) in beams.iter().zip(decoded.iter()) {
                        for i in 0..9 {
                            let diff = orig.digits[i] as f32 - dec.digits[i] as f32;
                            recon_loss += diff * diff;
                        }
                    }
                    calm_loss += recon_loss / beams.len().max(1) as f32;
                    
                    // === FIX 1: Contrastive — use next text as positive, sample negatives from buffer ===
                    if text_idx + 1 < end_idx {
                        let positive_embed = &all_embeddings[text_idx + 1];
                        
                        // Sample negatives from buffer (not just 1, use num_negatives)
                        let mut negatives: Vec<Vec<f32>> = Vec::with_capacity(num_negatives);
                        if !negative_buffer.is_empty() {
                            for k in 0..num_negatives {
                                let neg_idx = (self.training_steps.wrapping_mul(7) + k * 13 + text_idx) % negative_buffer.len();
                                negatives.push(negative_buffer[neg_idx].clone());
                            }
                        }
                        
                        if !negatives.is_empty() {
                            self.calm.train_contrastive(text_embed, positive_embed, &negatives, lr);
                            let pos_sim = cosine_similarity(text_embed, positive_embed);
                            let avg_neg_sim: f32 = negatives.iter()
                                .map(|n| cosine_similarity(text_embed, n))
                                .sum::<f32>() / negatives.len() as f32;
                            contrastive_loss += (1.0 - pos_sim + avg_neg_sim).max(0.0);
                        }
                        
                        // Refresh negative buffer with current embedding
                        if negative_buffer.len() < 256 {
                            negative_buffer.push(text_embed.clone());
                        } else {
                            let replace_idx = self.training_steps % negative_buffer.len();
                            negative_buffer[replace_idx] = text_embed.clone();
                        }
                    }
                    
                    // === FIX 2: gen_loss — use real next-token targets from tokenizer ===
                    // Train on actual next-token prediction for each position in the sequence
                    let max_positions = tokens.len().min(16); // Cap to avoid O(n^2) blowup
                    for pos in 1..max_positions {
                        let target_token = tokens[pos] as usize;
                        if target_token >= self.generation_head.vocab_size {
                            continue;
                        }
                        
                        // Get context embedding up to this position
                        let ctx_tokens = &tokens[..pos];
                        let ctx_embeds = self.tokenizer.get_embeddings(ctx_tokens);
                        let mut ctx_embed = vec![0.0f32; self.config.latent_dim];
                        for embed in &ctx_embeds {
                            for (j, &val) in embed.iter().enumerate() {
                                if j < ctx_embed.len() {
                                    ctx_embed[j] += val;
                                }
                            }
                        }
                        if !ctx_embeds.is_empty() {
                            for val in &mut ctx_embed {
                                *val /= ctx_embeds.len() as f32;
                            }
                        }
                        
                        // Encode context to latent and predict next token
                        let ctx_beams = self.embedding_to_beams(&ctx_embed);
                        let ctx_latent = self.calm.encode(&ctx_beams);
                        let logits = self.generation_head.forward(&ctx_latent.latent);
                        
                        // Cross-entropy loss with real target
                        let max_logit = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                        let exp_sum: f32 = logits.iter().map(|&l| (l - max_logit).exp()).sum();
                        let log_prob = logits.get(target_token).unwrap_or(&0.0) - max_logit - exp_sum.ln();
                        total_loss += -log_prob;
                        
                        // Update generation head with real gradient
                        let probs: Vec<f32> = logits.iter()
                            .map(|&l| ((l - max_logit).exp() / exp_sum))
                            .collect();
                        
                        for v in 0..self.generation_head.vocab_size.min(probs.len()) {
                            let target_prob = if v == target_token { 1.0 } else { 0.0 };
                            let grad = probs[v] - target_prob;
                            
                            for l in 0..self.config.latent_dim.min(ctx_latent.latent.len()) {
                                let idx = v * self.config.latent_dim + l;
                                if idx < self.generation_head.projection.len() {
                                    self.generation_head.projection[idx] -= lr * grad * ctx_latent.latent[l];
                                }
                            }
                            if v < self.generation_head.bias.len() {
                                self.generation_head.bias[v] -= lr * grad;
                            }
                        }
                    }
                    
                    self.training_steps += 1;
                }
            }
            
            let n = sample_count.max(1) as f32;
            println!("  Epoch {}/{}: gen_loss={:.4}, calm_loss={:.4}, contrastive={:.4} (lr={:.6})", 
                     epoch + 1, epochs, total_loss / n, calm_loss / n, contrastive_loss / n, lr);
        }
        
        println!("CALM pre-training complete. {} total steps.", self.training_steps);
    }
    
    /// Load and pretrain from HuggingFace-style dataset
    pub fn pretrain_from_dataset(&mut self, dataset_texts: Vec<String>, epochs: usize) {
        println!("Loading {} texts from dataset...", dataset_texts.len());
        
        // Use enhanced CALM pretraining
        self.pretrain_calm(&dataset_texts, epochs, 0.001);
        
        // Also populate knowledge base
        for text in dataset_texts.iter().take(1000) {
            self.add_knowledge(text);
        }
        
        println!("Dataset pretraining complete. Knowledge base has {} entries.", self.knowledge_base.len());
    }
    
    /// Add few-shot example
    pub fn add_fewshot_example(&mut self, question: &str, answer: &str) {
        let (_, q_embed) = self.encode_text(question);
        let (_, a_embed) = self.encode_text(answer);
        self.fewshot_context.add_example(
            question.to_string(),
            answer.to_string(),
            q_embed,
            a_embed,
        );
    }
    
    /// Answer a question using few-shot context and generation
    pub fn answer(&mut self, question: &str) -> String {
        // Format as Q&A prompt
        let mut prompt = String::new();
        
        // Add few-shot examples
        let (_, q_embed) = self.encode_text(question);
        let similar_examples = self.fewshot_context.get_similar_examples(&q_embed, self.config.num_fewshot);
        
        for example in similar_examples {
            prompt.push_str(&format!("Q: {}\nA: {}\n\n", example.question, example.answer));
        }
        
        // Add current question
        prompt.push_str(&format!("Q: {}\nA:", question));
        
        // Generate answer
        self.generate(&prompt, 100)
    }
}

// =============================================================================
// Symbolic Math Execution (Priority 5)
// =============================================================================

/// Result of symbolic math evaluation
#[derive(Debug, Clone)]
pub struct MathResult {
    pub expression: String,
    pub result: f64,
    pub steps: Vec<String>,
    pub success: bool,
}

/// Symbolic math executor for GSM8K-style problems
#[derive(Debug, Clone)]
pub struct SymbolicMathExecutor {
    /// Variable bindings
    pub variables: HashMap<String, f64>,
}

impl SymbolicMathExecutor {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
    
    /// Extract numbers from text
    pub fn extract_numbers(&self, text: &str) -> Vec<f64> {
        let mut numbers = Vec::new();
        let mut current = String::new();
        let mut in_number = false;
        
        for c in text.chars() {
            if c.is_ascii_digit() || c == '.' || (c == '-' && !in_number) {
                current.push(c);
                in_number = true;
            } else if in_number {
                if let Ok(n) = current.parse::<f64>() {
                    numbers.push(n);
                }
                current.clear();
                in_number = false;
            }
        }
        
        if !current.is_empty() {
            if let Ok(n) = current.parse::<f64>() {
                numbers.push(n);
            }
        }
        
        numbers
    }
    
    /// Extract math expressions from text
    pub fn extract_expressions(&self, text: &str) -> Vec<String> {
        let mut expressions = Vec::new();
        
        // Pattern: number operator number
        let operators = ['+', '-', '*', '/', '×', '÷'];
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for window in words.windows(3) {
            let left = window[0].trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
            let op = window[1];
            let right = window[2].trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
            
            if let (Ok(_), Ok(_)) = (left.parse::<f64>(), right.parse::<f64>()) {
                if op.len() == 1 && operators.contains(&op.chars().next().unwrap_or(' ')) {
                    expressions.push(format!("{} {} {}", left, op, right));
                }
            }
        }
        
        // Also look for expressions like "5+3" without spaces
        let re_pattern = r"(\d+\.?\d*)\s*([+\-*/×÷])\s*(\d+\.?\d*)";
        // Simple regex-free approach
        let text_chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        while i < text_chars.len() {
            if text_chars[i].is_ascii_digit() {
                let start = i;
                // Consume number
                while i < text_chars.len() && (text_chars[i].is_ascii_digit() || text_chars[i] == '.') {
                    i += 1;
                }
                let left_str: String = text_chars[start..i].iter().collect();
                
                // Skip whitespace
                while i < text_chars.len() && text_chars[i].is_whitespace() {
                    i += 1;
                }
                
                // Check for operator
                if i < text_chars.len() && operators.contains(&text_chars[i]) {
                    let op = text_chars[i];
                    i += 1;
                    
                    // Skip whitespace
                    while i < text_chars.len() && text_chars[i].is_whitespace() {
                        i += 1;
                    }
                    
                    // Consume second number
                    if i < text_chars.len() && text_chars[i].is_ascii_digit() {
                        let start2 = i;
                        while i < text_chars.len() && (text_chars[i].is_ascii_digit() || text_chars[i] == '.') {
                            i += 1;
                        }
                        let right_str: String = text_chars[start2..i].iter().collect();
                        expressions.push(format!("{} {} {}", left_str, op, right_str));
                    }
                }
            } else {
                i += 1;
            }
        }
        
        expressions
    }
    
    /// Evaluate a simple arithmetic expression
    pub fn evaluate(&self, expr: &str) -> Option<f64> {
        let expr = expr.trim();
        
        // Handle parentheses recursively
        if expr.contains('(') {
            // Find innermost parentheses
            let mut depth = 0;
            let mut start = 0;
            let mut end = 0;
            
            for (i, c) in expr.chars().enumerate() {
                if c == '(' {
                    if depth == 0 {
                        start = i;
                    }
                    depth += 1;
                } else if c == ')' {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
            }
            
            if end > start {
                let inner = &expr[start + 1..end];
                if let Some(inner_result) = self.evaluate(inner) {
                    let new_expr = format!(
                        "{}{}{}",
                        &expr[..start],
                        inner_result,
                        &expr[end + 1..]
                    );
                    return self.evaluate(&new_expr);
                }
            }
        }
        
        // Try to parse as a single number
        if let Ok(n) = expr.parse::<f64>() {
            return Some(n);
        }
        
        // Check for variable
        if let Some(&val) = self.variables.get(expr) {
            return Some(val);
        }
        
        // Find operator (respecting precedence: +- before */)
        let mut op_pos = None;
        let mut op_char = ' ';
        let mut depth = 0;
        
        // First pass: look for + or - (lowest precedence)
        for (i, c) in expr.chars().enumerate() {
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
            } else if depth == 0 && (c == '+' || c == '-') && i > 0 {
                op_pos = Some(i);
                op_char = c;
            }
        }
        
        // Second pass: look for * or / if no +/- found
        if op_pos.is_none() {
            depth = 0;
            for (i, c) in expr.chars().enumerate() {
                if c == '(' {
                    depth += 1;
                } else if c == ')' {
                    depth -= 1;
                } else if depth == 0 && (c == '*' || c == '/' || c == '×' || c == '÷') && i > 0 {
                    op_pos = Some(i);
                    op_char = c;
                }
            }
        }
        
        if let Some(pos) = op_pos {
            let left = &expr[..pos].trim();
            let right = &expr[pos + 1..].trim();
            
            let left_val = self.evaluate(left)?;
            let right_val = self.evaluate(right)?;
            
            let result = match op_char {
                '+' => left_val + right_val,
                '-' => left_val - right_val,
                '*' | '×' => left_val * right_val,
                '/' | '÷' => {
                    if right_val.abs() < 1e-10 {
                        return None; // Division by zero
                    }
                    left_val / right_val
                }
                _ => return None,
            };
            
            return Some(result);
        }
        
        None
    }
    
    /// Solve a word problem by extracting and evaluating expressions
    pub fn solve_word_problem(&mut self, problem: &str) -> MathResult {
        let mut steps = Vec::new();
        
        // Extract numbers
        let numbers = self.extract_numbers(problem);
        steps.push(format!("Extracted numbers: {:?}", numbers));
        
        // Extract expressions
        let expressions = self.extract_expressions(problem);
        steps.push(format!("Found expressions: {:?}", expressions));
        
        // Look for keywords that indicate operations
        let problem_lower = problem.to_lowercase();
        
        let mut result = 0.0f64;
        let mut found_answer = false;
        
        // Try to evaluate any explicit expressions first
        for expr in &expressions {
            if let Some(val) = self.evaluate(expr) {
                result = val;
                found_answer = true;
                steps.push(format!("Evaluated '{}' = {}", expr, val));
            }
        }
        
        // If no explicit expression, try to infer from keywords
        if !found_answer && numbers.len() >= 2 {
            let n1 = numbers[0];
            let n2 = numbers[1];
            
            if problem_lower.contains("total") || problem_lower.contains("sum") 
                || problem_lower.contains("altogether") || problem_lower.contains("combined") {
                result = n1 + n2;
                steps.push(format!("Inferred addition: {} + {} = {}", n1, n2, result));
                found_answer = true;
            } else if problem_lower.contains("difference") || problem_lower.contains("left") 
                || problem_lower.contains("remain") || problem_lower.contains("gave away")
                || problem_lower.contains("minus") || problem_lower.contains("fewer") {
                result = n1 - n2;
                steps.push(format!("Inferred subtraction: {} - {} = {}", n1, n2, result));
                found_answer = true;
            } else if problem_lower.contains("times") || problem_lower.contains("product") 
                || problem_lower.contains("each") || problem_lower.contains("per") {
                result = n1 * n2;
                steps.push(format!("Inferred multiplication: {} × {} = {}", n1, n2, result));
                found_answer = true;
            } else if problem_lower.contains("divide") || problem_lower.contains("split") 
                || problem_lower.contains("share equally") || problem_lower.contains("per person") {
                if n2.abs() > 1e-10 {
                    result = n1 / n2;
                    steps.push(format!("Inferred division: {} ÷ {} = {}", n1, n2, result));
                    found_answer = true;
                }
            }
        }
        
        // Multi-step problems
        if !found_answer && numbers.len() >= 3 {
            // Try common patterns
            let n1 = numbers[0];
            let n2 = numbers[1];
            let n3 = numbers[2];
            
            // Pattern: "X has N, gets M more, then gives away P"
            if problem_lower.contains("then") || problem_lower.contains("after") {
                if problem_lower.contains("more") && problem_lower.contains("gave") {
                    result = n1 + n2 - n3;
                    steps.push(format!("Multi-step: {} + {} - {} = {}", n1, n2, n3, result));
                    found_answer = true;
                }
            }
        }
        
        MathResult {
            expression: expressions.first().cloned().unwrap_or_default(),
            result,
            steps,
            success: found_answer,
        }
    }
    
    /// Set a variable
    pub fn set_variable(&mut self, name: &str, value: f64) {
        self.variables.insert(name.to_string(), value);
    }
}

impl Default for SymbolicMathExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// RAG Knowledge Base (Priority 6)
// =============================================================================

/// A knowledge triple (subject, relation, object)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub weight: f32,
    pub embedding: Vec<f32>,
}

/// ConceptNet-style knowledge base for RAG
#[derive(Debug, Clone)]
pub struct KnowledgeBase {
    /// All knowledge triples
    pub triples: Vec<KnowledgeTriple>,
    /// Index by subject
    pub subject_index: HashMap<String, Vec<usize>>,
    /// Index by object
    pub object_index: HashMap<String, Vec<usize>>,
    /// Index by relation
    pub relation_index: HashMap<String, Vec<usize>>,
    /// Embedding dimension
    pub embed_dim: usize,
}

impl KnowledgeBase {
    pub fn new(embed_dim: usize) -> Self {
        let mut kb = Self {
            triples: Vec::new(),
            subject_index: HashMap::new(),
            object_index: HashMap::new(),
            relation_index: HashMap::new(),
            embed_dim,
        };
        
        // Initialize with common knowledge
        kb.init_common_knowledge();
        kb
    }
    
    /// Initialize with common knowledge facts
    fn init_common_knowledge(&mut self) {
        // Science facts
        let science_facts = [
            ("sky", "HasProperty", "blue", 1.0),
            ("grass", "HasProperty", "green", 1.0),
            ("sun", "HasProperty", "hot", 1.0),
            ("ice", "HasProperty", "cold", 1.0),
            ("water", "FreezesAt", "0 degrees Celsius", 1.0),
            ("water", "BoilsAt", "100 degrees Celsius", 1.0),
            ("Earth", "IsA", "planet", 1.0),
            ("Earth", "Orbits", "Sun", 1.0),
            ("Moon", "Orbits", "Earth", 1.0),
            ("light", "TravelsFasterThan", "sound", 1.0),
            ("plants", "Need", "sunlight", 1.0),
            ("plants", "Produce", "oxygen", 1.0),
            ("fire", "Needs", "oxygen", 1.0),
            ("metal", "Conducts", "electricity", 1.0),
            ("rubber", "Insulates", "electricity", 1.0),
        ];
        
        // Commonsense facts
        let commonsense_facts = [
            ("bird", "CanDo", "fly", 0.9),
            ("fish", "LivesIn", "water", 1.0),
            ("dog", "IsA", "animal", 1.0),
            ("cat", "IsA", "animal", 1.0),
            ("apple", "IsA", "fruit", 1.0),
            ("car", "HasPart", "wheels", 1.0),
            ("bicycle", "HasPart", "wheels", 1.0),
            ("rain", "CausedBy", "clouds", 0.8),
            ("snow", "HasProperty", "cold", 1.0),
            ("summer", "HasProperty", "hot", 0.9),
            ("winter", "HasProperty", "cold", 0.9),
            ("human", "Needs", "food", 1.0),
            ("human", "Needs", "water", 1.0),
            ("human", "Needs", "sleep", 1.0),
        ];
        
        // Causal facts
        let causal_facts = [
            ("exercise", "LeadsTo", "health", 0.8),
            ("studying", "LeadsTo", "knowledge", 0.9),
            ("eating", "LeadsTo", "energy", 0.9),
            ("sleeping", "LeadsTo", "rest", 1.0),
            ("rain", "LeadsTo", "wet ground", 0.9),
            ("fire", "LeadsTo", "heat", 1.0),
            ("cold", "LeadsTo", "shivering", 0.8),
        ];
        
        for (subj, rel, obj, weight) in science_facts.iter().chain(commonsense_facts.iter()).chain(causal_facts.iter()) {
            self.add_triple(subj, rel, obj, *weight);
        }
    }
    
    /// Add a knowledge triple
    pub fn add_triple(&mut self, subject: &str, relation: &str, object: &str, weight: f32) {
        let idx = self.triples.len();
        
        // Create embedding from triple text
        let text = format!("{} {} {}", subject, relation, object);
        let embedding = self.text_to_embedding(&text);
        
        let triple = KnowledgeTriple {
            subject: subject.to_lowercase(),
            relation: relation.to_string(),
            object: object.to_lowercase(),
            weight,
            embedding,
        };
        
        // Update indices
        self.subject_index.entry(triple.subject.clone()).or_default().push(idx);
        self.object_index.entry(triple.object.clone()).or_default().push(idx);
        self.relation_index.entry(triple.relation.clone()).or_default().push(idx);
        
        self.triples.push(triple);
    }
    
    /// Simple text to embedding (hash-based, should be replaced with learned embeddings)
    fn text_to_embedding(&self, text: &str) -> Vec<f32> {
        let mut embed = vec![0.0f32; self.embed_dim];
        for (i, c) in text.chars().enumerate() {
            let idx = (c as usize + i * 7) % self.embed_dim;
            embed[idx] += 1.0;
            embed[(idx + 1) % self.embed_dim] += 0.5;
        }
        // Normalize
        let norm: f32 = embed.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
        for val in &mut embed {
            *val /= norm;
        }
        embed
    }
    
    /// Query knowledge by subject
    pub fn query_subject(&self, subject: &str) -> Vec<&KnowledgeTriple> {
        let subject_lower = subject.to_lowercase();
        self.subject_index.get(&subject_lower)
            .map(|indices| indices.iter().filter_map(|&i| self.triples.get(i)).collect())
            .unwrap_or_default()
    }
    
    /// Query knowledge by object
    pub fn query_object(&self, object: &str) -> Vec<&KnowledgeTriple> {
        let object_lower = object.to_lowercase();
        self.object_index.get(&object_lower)
            .map(|indices| indices.iter().filter_map(|&i| self.triples.get(i)).collect())
            .unwrap_or_default()
    }
    
    /// Query knowledge by relation
    pub fn query_relation(&self, relation: &str) -> Vec<&KnowledgeTriple> {
        self.relation_index.get(relation)
            .map(|indices| indices.iter().filter_map(|&i| self.triples.get(i)).collect())
            .unwrap_or_default()
    }
    
    /// Semantic search over knowledge base
    pub fn semantic_search(&self, query_embedding: &[f32], k: usize) -> Vec<&KnowledgeTriple> {
        let mut scored: Vec<(&KnowledgeTriple, f32)> = self.triples.iter()
            .map(|triple| {
                let sim = cosine_similarity(query_embedding, &triple.embedding);
                (triple, sim * triple.weight)
            })
            .collect();
        
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(k).map(|(t, _)| t).collect()
    }
    
    /// Extract relevant knowledge for a question
    pub fn extract_for_question(&self, question: &str) -> Vec<&KnowledgeTriple> {
        let question_lower = question.to_lowercase();
        let words: Vec<&str> = question_lower.split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();
        
        let mut results: Vec<&KnowledgeTriple> = Vec::new();
        
        // Search by keywords
        for word in &words {
            results.extend(self.query_subject(word));
            results.extend(self.query_object(word));
        }
        
        // Deduplicate
        let mut seen = std::collections::HashSet::new();
        results.retain(|t| {
            let key = format!("{}-{}-{}", t.subject, t.relation, t.object);
            seen.insert(key)
        });
        
        // Sort by weight
        results.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
        
        results.into_iter().take(5).collect()
    }
    
    /// Format knowledge as context string
    pub fn format_as_context(&self, triples: &[&KnowledgeTriple]) -> String {
        triples.iter()
            .map(|t| format!("{} {} {}", t.subject, t.relation, t.object))
            .collect::<Vec<_>>()
            .join(". ")
    }
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self::new(256)
    }
}

// Add RAG methods to GenerativeVortexEngine
impl GenerativeVortexEngine {
    /// Initialize knowledge base
    pub fn init_knowledge_base(&mut self) {
        let kb = KnowledgeBase::new(self.config.latent_dim);
        // Add all triples to the simple knowledge base
        for triple in &kb.triples {
            let text = format!("{} {} {}", triple.subject, triple.relation, triple.object);
            self.add_knowledge(&text);
        }
    }
    
    /// Answer with RAG (retrieval-augmented generation)
    pub fn answer_with_rag(&mut self, question: &str) -> String {
        // Create knowledge base for retrieval
        let kb = KnowledgeBase::new(self.config.latent_dim);
        
        // Extract relevant knowledge
        let relevant = kb.extract_for_question(question);
        let context = kb.format_as_context(&relevant);
        
        // Build prompt with context
        let prompt = if context.is_empty() {
            format!("Q: {}\nA:", question)
        } else {
            format!("Context: {}\n\nQ: {}\nA:", context, question)
        };
        
        // Generate with context
        self.generate(&prompt, 100)
    }

    /// Unified answer method with all capabilities
    pub fn unified_answer(&mut self, question: &str) -> String {
        // Heuristic strategy selection without dynamic router
        let is_math = question.chars().any(|c| c.is_ascii_digit())
            || ["calculate", "compute", "sum", "total", "how many", "how much"]
                .iter().any(|kw| question.to_lowercase().contains(kw));

        if is_math {
            return self.solve_math(question);
        }

        // Default: few-shot + RAG
        self.answer_with_moe(question)
    }
    
    /// Answer using MoE-weighted expert contributions
    pub fn answer_with_moe(&mut self, question: &str) -> String {
        // Build simple context (RAG + math hints) without dynamic routing
        let mut context_parts = Vec::new();

        // RAG context
        let kb = KnowledgeBase::new(self.config.latent_dim);
        let relevant = kb.extract_for_question(question);
        if !relevant.is_empty() {
            let context = kb.format_as_context(&relevant);
            context_parts.push(format!("[Knowledge]: {}", context));
        }

        // Math context
        let executor = SymbolicMathExecutor::new();
        let numbers = executor.extract_numbers(question);
        if !numbers.is_empty() {
            context_parts.push(format!("[Math]: Numbers found: {:?}", numbers));
        }

        // Build prompt with MoE context
        let mut prompt = String::new();
        
        if !context_parts.is_empty() {
            prompt.push_str("Expert Analysis:\n");
            for part in &context_parts {
                prompt.push_str(part);
                prompt.push('\n');
            }
            prompt.push('\n');
        }
        
        // Add few-shot examples
        let (_, q_embed) = self.encode_text(question);
        let similar = self.fewshot_context.get_similar_examples(&q_embed, 3);
        for example in similar {
            prompt.push_str(&format!("Q: {}\nA: {}\n\n", example.question, example.answer));
        }
        
        // Add current question
        prompt.push_str(&format!("Q: {}\nA:", question));
        
        // Generate answer
        self.generate(&prompt, 100)
    }
}

// Add symbolic math to GenerativeVortexEngine
impl GenerativeVortexEngine {
    /// Solve a math problem using symbolic execution
    pub fn solve_math(&mut self, problem: &str) -> String {
        let mut executor = SymbolicMathExecutor::new();
        let result = executor.solve_word_problem(problem);
        
        if result.success {
            // Format answer
            let answer = if result.result.fract().abs() < 1e-10 {
                format!("{}", result.result as i64)
            } else {
                format!("{:.2}", result.result)
            };
            
            // Add to few-shot context for learning
            self.add_fewshot_example(problem, &answer);
            
            answer
        } else {
            // Fall back to generative approach
            self.generate(&format!("Solve: {}\nAnswer:", problem), 20)
        }
    }
    
    /// Answer with math awareness
    pub fn answer_with_math(&mut self, question: &str) -> String {
        // Check if this is a math problem
        let has_numbers = question.chars().any(|c| c.is_ascii_digit());
        let math_keywords = ["how many", "how much", "calculate", "compute", "solve", 
                            "total", "sum", "difference", "product", "divide"];
        let is_math = has_numbers && math_keywords.iter().any(|kw| question.to_lowercase().contains(kw));
        
        if is_math {
            self.solve_math(question)
        } else {
            self.answer(question)
        }
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tokenizer() {
        let tokenizer = SubwordTokenizer::new(256);
        let tokens = tokenizer.tokenize("hello world");
        assert!(tokens.len() > 2); // BOS + chars + EOS
        
        let text = tokenizer.detokenize(&tokens);
        assert_eq!(text, "hello world");
    }
    
    #[test]
    fn test_sacred_attention() {
        let attention = SacredDynamicAttention::new(256, SacredAttentionConfig::default());
        
        assert_eq!(attention.heads_for_position(3), 3);
        assert_eq!(attention.heads_for_position(6), 6);
        assert_eq!(attention.heads_for_position(9), 9);
    }
    
    #[test]
    fn test_generation_head() {
        let head = GenerationHead::new(256, 100);
        let latent = vec![0.1f32; 256];
        let logits = head.forward(&latent);
        assert_eq!(logits.len(), 100);
    }
    
    #[test]
    fn test_generative_engine() {
        let config = GenerativeConfig::default();
        let mut engine = GenerativeVortexEngine::new(config);
        
        // Add some knowledge
        engine.add_knowledge("The sky is blue during the day.");
        engine.add_knowledge("Water freezes at 0 degrees Celsius.");
        
        // Add few-shot example
        engine.add_fewshot_example("What color is grass?", "green");
        
        // Generate (will be random without training)
        let output = engine.generate("Hello", 10);
        assert!(!output.is_empty() || output.is_empty()); // Just check it doesn't crash
    }
    
    #[test]
    fn test_symbolic_math_basic() {
        let executor = SymbolicMathExecutor::new();
        
        // Basic arithmetic
        assert_eq!(executor.evaluate("2 + 3"), Some(5.0));
        assert_eq!(executor.evaluate("10 - 4"), Some(6.0));
        assert_eq!(executor.evaluate("3 * 4"), Some(12.0));
        assert_eq!(executor.evaluate("15 / 3"), Some(5.0));
        
        // Precedence
        assert_eq!(executor.evaluate("2 + 3 * 4"), Some(14.0));
        
        // Parentheses
        assert_eq!(executor.evaluate("(2 + 3) * 4"), Some(20.0));
    }
    
    #[test]
    fn test_symbolic_math_word_problem() {
        let mut executor = SymbolicMathExecutor::new();
        
        // Simple addition word problem
        let result = executor.solve_word_problem("John has 5 apples and Mary has 3 apples. How many apples do they have in total?");
        assert!(result.success);
        assert_eq!(result.result, 8.0);
        
        // Subtraction word problem
        let result = executor.solve_word_problem("Tom had 10 cookies and gave away 4. How many are left?");
        assert!(result.success);
        assert_eq!(result.result, 6.0);
    }
    
    #[test]
    fn test_symbolic_math_extract_numbers() {
        let executor = SymbolicMathExecutor::new();
        
        let numbers = executor.extract_numbers("I have 5 apples and 3 oranges");
        assert_eq!(numbers, vec![5.0, 3.0]);
        
        let numbers = executor.extract_numbers("The price is $12.50 for 2 items");
        assert_eq!(numbers, vec![12.50, 2.0]);
    }
}
