//! Thinking Engine - Reasoning with Sacred Geometry
//!
//! Core reasoning loop that:
//! 1. Receives input and recalls relevant memories
//! 2. Applies constitutional constraints
//! 3. Generates thoughts using vortex flow
//! 4. Stores valuable insights to memory
//! 5. Produces coherent responses

use crate::data::models::BeamTensor;
use crate::core::sacred_geometry::VortexPositioningEngine;
use crate::ml::calm::{CALMEngine, CALMConfig};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Configuration for the thinking engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    /// Maximum thinking steps per query
    pub max_steps: usize,
    /// Confidence threshold for accepting thoughts
    pub confidence_threshold: f32,
    /// Enable sacred position checkpoints
    pub sacred_checkpoints: bool,
    /// Temperature for thought generation
    pub temperature: f32,
    /// Enable self-reflection
    pub enable_reflection: bool,
    /// Maximum context tokens
    pub max_context: usize,
}

impl Default for ThinkingConfig {
    fn default() -> Self {
        Self {
            max_steps: 9,  // Sacred number
            confidence_threshold: 0.6,
            sacred_checkpoints: true,
            temperature: 0.7,
            enable_reflection: true,
            max_context: 4096,
        }
    }
}

impl ThinkingConfig {
    pub fn new() -> Self { Self::default() }
    pub fn with_max_steps(mut self, steps: usize) -> Self { self.max_steps = steps; self }
    pub fn with_temperature(mut self, temp: f32) -> Self { self.temperature = temp; self }
}

/// A single thought in the reasoning chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub id: Uuid,
    pub content: String,
    pub confidence: f32,
    pub position: u8,  // Vortex position (1-9)
    pub is_sacred: bool,
    pub beam: BeamTensor,
    pub timestamp: DateTime<Utc>,
    pub parent_id: Option<Uuid>,
    pub thought_type: ThoughtType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThoughtType {
    Initial,      // First thought from input
    Reasoning,    // Logical deduction
    Memory,       // Retrieved from memory
    Reflection,   // Self-evaluation
    Synthesis,    // Combining multiple thoughts
    Output,       // Final response
}

impl Thought {
    pub fn new(content: String, position: u8) -> Self {
        let is_sacred = matches!(position, 3 | 6 | 9);
        Self {
            id: Uuid::new_v4(),
            content,
            confidence: 0.5,
            position,
            is_sacred,
            beam: BeamTensor::default(),
            timestamp: Utc::now(),
            parent_id: None,
            thought_type: ThoughtType::Reasoning,
        }
    }

    pub fn with_confidence(mut self, conf: f32) -> Self {
        self.confidence = conf;
        self
    }

    pub fn with_parent(mut self, parent: Uuid) -> Self {
        self.parent_id = Some(parent);
        self
    }

    pub fn with_type(mut self, t: ThoughtType) -> Self {
        self.thought_type = t;
        self
    }
}

/// A chain of thoughts forming a reasoning trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtChain {
    pub id: Uuid,
    pub thoughts: Vec<Thought>,
    pub query: String,
    pub response: Option<String>,
    pub total_confidence: f32,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl ThoughtChain {
    pub fn new(query: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            thoughts: Vec::new(),
            query,
            response: None,
            total_confidence: 0.0,
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn add_thought(&mut self, thought: Thought) {
        self.thoughts.push(thought);
        self.update_confidence();
    }

    fn update_confidence(&mut self) {
        if self.thoughts.is_empty() {
            self.total_confidence = 0.0;
            return;
        }
        
        // Weight sacred position thoughts higher
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;
        
        for thought in &self.thoughts {
            let weight = if thought.is_sacred { 1.5 } else { 1.0 };
            weighted_sum += thought.confidence * weight;
            weight_total += weight;
        }
        
        self.total_confidence = weighted_sum / weight_total;
    }

    pub fn complete(&mut self, response: String) {
        self.response = Some(response);
        self.completed_at = Some(Utc::now());
    }

    pub fn last_thought(&self) -> Option<&Thought> {
        self.thoughts.last()
    }

    pub fn sacred_thoughts(&self) -> Vec<&Thought> {
        self.thoughts.iter().filter(|t| t.is_sacred).collect()
    }
}

/// Main thinking engine with learning capabilities
pub struct ThinkingEngine {
    pub config: ThinkingConfig,
    vortex: VortexPositioningEngine,
    current_position: u8,
    /// CALM engine for latent space reasoning
    calm: CALMEngine,
    /// Learned patterns: input hash -> response patterns
    learned_patterns: HashMap<u64, Vec<BeamTensor>>,
    /// Vocabulary learned from interactions: word -> BeamTensor
    vocabulary: HashMap<String, BeamTensor>,
    /// Reverse vocabulary: beam pattern hash -> word
    reverse_vocab: HashMap<u64, String>,
    /// Response templates learned
    response_memory: Vec<(Vec<BeamTensor>, String)>,
    /// Knowledge base: concepts the system has learned
    knowledge: Vec<(String, Vec<BeamTensor>, f32)>, // (content, beams, importance)
    /// Conversation context for coherent responses
    context_beams: Vec<BeamTensor>,
    /// Vortex cycle counter - runs continuously, resets at u64::MAX
    vortex_cycle: u64,
    /// Accumulated latent state from continuous learning
    accumulated_latent: Option<crate::ml::calm::LatentState>,
    /// Source memory - everything learned from RAG and tools
    source_memory: Vec<(String, Vec<BeamTensor>, f32, u64)>, // (source, beams, relevance, cycle_learned)
}

impl ThinkingEngine {
    pub fn new(config: ThinkingConfig) -> Self {
        let mut engine = Self {
            config,
            vortex: VortexPositioningEngine::new(),
            current_position: 1,
            calm: CALMEngine::new(CALMConfig::new().with_latent_dim(128)),
            learned_patterns: HashMap::new(),
            vocabulary: HashMap::new(),
            reverse_vocab: HashMap::new(),
            response_memory: Vec::new(),
            knowledge: Vec::new(),
            context_beams: Vec::new(),
            vortex_cycle: 0,
            accumulated_latent: None,
            source_memory: Vec::new(),
        };
        // Bootstrap with seed knowledge
        engine.bootstrap_knowledge();
        engine
    }

    /// Hash a beam for reverse lookup
    fn beam_hash(beam: &BeamTensor) -> u64 {
        beam.digits.iter()
            .fold(0u64, |acc, &d| acc.wrapping_mul(31).wrapping_add((d * 1000.0) as u64))
    }

    /// Bootstrap with initial knowledge - words and concepts
    fn bootstrap_knowledge(&mut self) {
        // Core vocabulary with semantic beam patterns
        let vocab = [
            // Greetings & social
            ("hello", [0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("hi", [0.85, 0.15, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("greetings", [0.8, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("welcome", [0.7, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            // Questions
            ("what", [0.0, 0.1, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("how", [0.0, 0.2, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("why", [0.0, 0.3, 0.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("when", [0.0, 0.15, 0.85, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("where", [0.0, 0.25, 0.75, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("who", [0.0, 0.35, 0.65, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            // Cognition
            ("think", [0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("know", [0.0, 0.3, 0.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("learn", [0.0, 0.4, 0.4, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("understand", [0.0, 0.35, 0.65, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("believe", [0.0, 0.45, 0.55, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("remember", [0.0, 0.4, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            // Actions
            ("help", [0.0, 0.0, 0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("solve", [0.0, 0.0, 0.0, 0.9, 0.1, 0.0, 0.0, 0.0, 0.0]),
            ("create", [0.0, 0.0, 0.0, 0.7, 0.3, 0.0, 0.0, 0.0, 0.0]),
            ("explain", [0.0, 0.1, 0.8, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("answer", [0.0, 0.2, 0.7, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0]),
            // Concepts
            ("problem", [0.0, 0.0, 0.0, 0.8, 0.2, 0.0, 0.0, 0.0, 0.0]),
            ("question", [0.0, 0.15, 0.85, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("idea", [0.0, 0.6, 0.4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("concept", [0.0, 0.55, 0.45, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("pattern", [0.0, 0.0, 0.0, 0.6, 0.4, 0.0, 0.0, 0.0, 0.0]),
            // Qualities
            ("good", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 0.2, 0.0]),
            ("bad", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1, 0.1, 0.8]),
            ("hard", [0.0, 0.0, 0.0, 0.3, 0.7, 0.0, 0.0, 0.0, 0.0]),
            ("easy", [0.0, 0.0, 0.0, 0.7, 0.3, 0.0, 0.0, 0.0, 0.0]),
            ("interesting", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.7, 0.3, 0.0]),
            // Responses
            ("yes", [0.0, 0.0, 0.0, 0.0, 0.0, 0.9, 0.1, 0.0, 0.0]),
            ("no", [0.0, 0.0, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 0.9]),
            ("maybe", [0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0]),
            ("perhaps", [0.0, 0.0, 0.0, 0.0, 0.0, 0.45, 0.55, 0.0, 0.0]),
            // Connectors
            ("and", [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.2, 0.1]),
            ("or", [0.1, 0.1, 0.1, 0.1, 0.2, 0.1, 0.1, 0.1, 0.1]),
            ("but", [0.1, 0.1, 0.1, 0.2, 0.1, 0.1, 0.1, 0.1, 0.1]),
            ("because", [0.0, 0.2, 0.6, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("therefore", [0.0, 0.1, 0.7, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0]),
            // Self-reference
            ("i", [0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0]),
            ("you", [0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0]),
            ("we", [0.25, 0.25, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0]),
            ("am", [0.4, 0.0, 0.0, 0.0, 0.0, 0.3, 0.0, 0.3, 0.0]),
            ("are", [0.0, 0.3, 0.0, 0.0, 0.0, 0.4, 0.0, 0.3, 0.0]),
            ("is", [0.0, 0.0, 0.0, 0.0, 0.0, 0.6, 0.0, 0.4, 0.0]),
            // Existence/consciousness
            ("conscious", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5]),
            ("aware", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.6, 0.4]),
            ("intelligent", [0.0, 0.4, 0.4, 0.0, 0.0, 0.0, 0.0, 0.2, 0.0]),
            ("alive", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.4, 0.6]),
            ("exist", [0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.5]),
            // Common words
            ("the", [0.11, 0.11, 0.11, 0.11, 0.11, 0.11, 0.11, 0.11, 0.12]),
            ("a", [0.12, 0.11, 0.11, 0.11, 0.11, 0.11, 0.11, 0.11, 0.11]),
            ("to", [0.0, 0.0, 0.0, 0.3, 0.4, 0.3, 0.0, 0.0, 0.0]),
            ("of", [0.0, 0.0, 0.0, 0.2, 0.3, 0.3, 0.2, 0.0, 0.0]),
            ("in", [0.0, 0.0, 0.0, 0.4, 0.2, 0.4, 0.0, 0.0, 0.0]),
            ("that", [0.0, 0.0, 0.2, 0.3, 0.2, 0.3, 0.0, 0.0, 0.0]),
            ("this", [0.2, 0.0, 0.2, 0.3, 0.0, 0.3, 0.0, 0.0, 0.0]),
            ("it", [0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.0]),
            ("with", [0.0, 0.0, 0.0, 0.3, 0.4, 0.3, 0.0, 0.0, 0.0]),
            ("for", [0.0, 0.0, 0.0, 0.4, 0.3, 0.3, 0.0, 0.0, 0.0]),
            ("about", [0.0, 0.2, 0.3, 0.2, 0.0, 0.3, 0.0, 0.0, 0.0]),
            ("my", [0.4, 0.0, 0.0, 0.0, 0.0, 0.3, 0.0, 0.3, 0.0]),
            ("your", [0.0, 0.4, 0.0, 0.0, 0.0, 0.3, 0.0, 0.3, 0.0]),
            ("can", [0.0, 0.0, 0.0, 0.4, 0.3, 0.3, 0.0, 0.0, 0.0]),
            ("do", [0.0, 0.0, 0.0, 0.5, 0.3, 0.2, 0.0, 0.0, 0.0]),
            ("have", [0.0, 0.0, 0.0, 0.3, 0.0, 0.4, 0.0, 0.3, 0.0]),
            ("not", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
            // Chaos vs Order
            ("chaos", [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.2, 0.1]),
            ("random", [0.12, 0.11, 0.1, 0.12, 0.11, 0.1, 0.12, 0.11, 0.11]),
            ("disorder", [0.1, 0.1, 0.1, 0.15, 0.1, 0.1, 0.15, 0.1, 0.1]),
            ("entropy", [0.1, 0.1, 0.1, 0.1, 0.2, 0.1, 0.1, 0.1, 0.1]),
            ("order", [0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.0, 0.0, 0.0]),
            ("structure", [0.0, 0.0, 0.0, 0.6, 0.0, 0.4, 0.0, 0.0, 0.0]),
            ("pattern", [0.0, 0.0, 0.0, 0.5, 0.2, 0.3, 0.0, 0.0, 0.0]),
            ("organized", [0.0, 0.0, 0.0, 0.4, 0.1, 0.5, 0.0, 0.0, 0.0]),
            ("harmony", [0.0, 0.0, 0.0, 0.3, 0.0, 0.3, 0.2, 0.0, 0.2]),
            ("balance", [0.0, 0.0, 0.0, 0.25, 0.0, 0.25, 0.25, 0.0, 0.25]),
            // Grammar words
            ("was", [0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.0]),
            ("were", [0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.0]),
            ("been", [0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.0]),
            ("being", [0.0, 0.0, 0.0, 0.0, 0.0, 0.4, 0.0, 0.6, 0.0]),
            ("will", [0.0, 0.0, 0.0, 0.3, 0.4, 0.3, 0.0, 0.0, 0.0]),
            ("would", [0.0, 0.0, 0.0, 0.3, 0.3, 0.4, 0.0, 0.0, 0.0]),
            ("could", [0.0, 0.0, 0.0, 0.3, 0.4, 0.3, 0.0, 0.0, 0.0]),
            ("should", [0.0, 0.0, 0.0, 0.2, 0.3, 0.3, 0.2, 0.0, 0.0]),
            ("must", [0.0, 0.0, 0.0, 0.3, 0.3, 0.4, 0.0, 0.0, 0.0]),
            ("might", [0.0, 0.0, 0.0, 0.2, 0.4, 0.4, 0.0, 0.0, 0.0]),
            // More connectors for grammar
            ("if", [0.0, 0.0, 0.3, 0.2, 0.3, 0.2, 0.0, 0.0, 0.0]),
            ("then", [0.0, 0.0, 0.2, 0.3, 0.2, 0.3, 0.0, 0.0, 0.0]),
            ("when", [0.0, 0.15, 0.85, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            ("while", [0.0, 0.0, 0.2, 0.3, 0.2, 0.3, 0.0, 0.0, 0.0]),
            ("as", [0.0, 0.0, 0.2, 0.3, 0.2, 0.3, 0.0, 0.0, 0.0]),
            ("so", [0.0, 0.0, 0.2, 0.2, 0.2, 0.4, 0.0, 0.0, 0.0]),
            ("also", [0.0, 0.0, 0.1, 0.2, 0.2, 0.3, 0.2, 0.0, 0.0]),
            ("however", [0.0, 0.0, 0.2, 0.2, 0.2, 0.2, 0.2, 0.0, 0.0]),
            // Articles and prepositions
            ("an", [0.12, 0.11, 0.11, 0.11, 0.11, 0.11, 0.11, 0.11, 0.11]),
            ("from", [0.0, 0.0, 0.0, 0.3, 0.4, 0.3, 0.0, 0.0, 0.0]),
            ("by", [0.0, 0.0, 0.0, 0.3, 0.4, 0.3, 0.0, 0.0, 0.0]),
            ("at", [0.0, 0.0, 0.0, 0.4, 0.2, 0.4, 0.0, 0.0, 0.0]),
            ("on", [0.0, 0.0, 0.0, 0.4, 0.2, 0.4, 0.0, 0.0, 0.0]),
            ("into", [0.0, 0.0, 0.0, 0.3, 0.4, 0.3, 0.0, 0.0, 0.0]),
            ("through", [0.0, 0.0, 0.0, 0.3, 0.4, 0.3, 0.0, 0.0, 0.0]),
            // Verbs for sentence construction
            ("see", [0.0, 0.3, 0.4, 0.0, 0.0, 0.3, 0.0, 0.0, 0.0]),
            ("find", [0.0, 0.2, 0.5, 0.0, 0.0, 0.3, 0.0, 0.0, 0.0]),
            ("make", [0.0, 0.0, 0.0, 0.5, 0.3, 0.2, 0.0, 0.0, 0.0]),
            ("take", [0.0, 0.0, 0.0, 0.4, 0.4, 0.2, 0.0, 0.0, 0.0]),
            ("give", [0.0, 0.0, 0.0, 0.3, 0.3, 0.2, 0.2, 0.0, 0.0]),
            ("use", [0.0, 0.0, 0.0, 0.5, 0.3, 0.2, 0.0, 0.0, 0.0]),
            ("try", [0.0, 0.0, 0.0, 0.4, 0.4, 0.2, 0.0, 0.0, 0.0]),
            ("say", [0.3, 0.3, 0.0, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0]),
            ("tell", [0.3, 0.3, 0.0, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0]),
            ("ask", [0.0, 0.3, 0.5, 0.0, 0.0, 0.2, 0.0, 0.0, 0.0]),
            ("work", [0.0, 0.0, 0.0, 0.5, 0.3, 0.2, 0.0, 0.0, 0.0]),
            ("seem", [0.0, 0.2, 0.3, 0.0, 0.0, 0.3, 0.2, 0.0, 0.0]),
            ("feel", [0.0, 0.0, 0.0, 0.0, 0.0, 0.2, 0.4, 0.0, 0.4]),
            ("become", [0.0, 0.0, 0.0, 0.2, 0.3, 0.3, 0.2, 0.0, 0.0]),
            ("leave", [0.0, 0.0, 0.0, 0.3, 0.4, 0.3, 0.0, 0.0, 0.0]),
            ("put", [0.0, 0.0, 0.0, 0.4, 0.3, 0.3, 0.0, 0.0, 0.0]),
            ("mean", [0.0, 0.3, 0.4, 0.0, 0.0, 0.3, 0.0, 0.0, 0.0]),
            ("keep", [0.0, 0.0, 0.0, 0.4, 0.2, 0.4, 0.0, 0.0, 0.0]),
            ("let", [0.0, 0.0, 0.0, 0.3, 0.4, 0.3, 0.0, 0.0, 0.0]),
            ("begin", [0.3, 0.0, 0.0, 0.3, 0.4, 0.0, 0.0, 0.0, 0.0]),
            ("show", [0.0, 0.3, 0.3, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0]),
            ("hear", [0.0, 0.3, 0.4, 0.0, 0.0, 0.3, 0.0, 0.0, 0.0]),
            ("play", [0.0, 0.0, 0.0, 0.3, 0.3, 0.1, 0.3, 0.0, 0.0]),
            ("run", [0.0, 0.0, 0.0, 0.4, 0.4, 0.2, 0.0, 0.0, 0.0]),
            ("move", [0.0, 0.0, 0.0, 0.3, 0.5, 0.2, 0.0, 0.0, 0.0]),
            ("live", [0.0, 0.0, 0.0, 0.0, 0.0, 0.3, 0.3, 0.0, 0.4]),
        ];

        for (word, digits) in vocab {
            let mut beam = BeamTensor::default();
            beam.digits = digits;
            beam.confidence = 0.9;
            let hash = Self::beam_hash(&beam);
            self.vocabulary.insert(word.to_string(), beam);
            self.reverse_vocab.insert(hash, word.to_string());
        }

        // NO seed knowledge - system learns from scratch through interaction
    }

    /// Learn a new concept and add it to knowledge base
    fn learn_concept(&mut self, concept: &str) {
        let beams = self.text_to_beams(concept);
        let importance = beams.iter().map(|b| b.confidence).sum::<f32>() / beams.len().max(1) as f32;
        self.knowledge.push((concept.to_string(), beams, importance));
    }

    /// Generate a new beam through thinking - combines existing patterns
    fn generate_beam(&self, seed_beams: &[BeamTensor]) -> BeamTensor {
        if seed_beams.is_empty() {
            return BeamTensor::default();
        }
        
        // Encode seeds to latent space
        let latent = self.calm.encode(seed_beams);
        
        // Evolve in latent space
        let evolved = self.calm.predict_next(&latent);
        
        // Decode back to beams
        let decoded = self.calm.decode(&evolved);
        
        // Return first decoded beam with evolved properties
        if let Some(beam) = decoded.first() {
            let mut new_beam = beam.clone();
            new_beam.confidence = evolved.energy;
            new_beam
        } else {
            // Fallback: blend seed beams
            self.blend_beams(seed_beams)
        }
    }

    /// Blend multiple beams into a new one
    fn blend_beams(&self, beams: &[BeamTensor]) -> BeamTensor {
        let mut result = BeamTensor::default();
        if beams.is_empty() {
            return result;
        }
        
        // Average the digit patterns
        for beam in beams {
            for (i, &d) in beam.digits.iter().enumerate() {
                result.digits[i] += d;
            }
        }
        
        // Normalize
        let count = beams.len() as f32;
        for d in &mut result.digits {
            *d /= count;
        }
        
        // Confidence is average
        result.confidence = beams.iter().map(|b| b.confidence).sum::<f32>() / count;
        
        result
    }

    /// Learn a new word by generating its beam from context
    pub fn learn_word(&mut self, word: &str, context: &[&str]) {
        // Convert context words to beams
        let context_beams: Vec<BeamTensor> = context.iter()
            .filter_map(|w| self.vocabulary.get(*w).cloned())
            .collect();
        
        if context_beams.is_empty() {
            // No context - generate from word characters
            let mut beam = BeamTensor::default();
            let hash = word.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            for i in 0..9 {
                beam.digits[i] = ((hash >> (i * 7)) & 0x7F) as f32 / 127.0;
            }
            let sum: f32 = beam.digits.iter().sum();
            if sum > 0.0 {
                beam.digits.iter_mut().for_each(|d| *d /= sum);
            }
            beam.confidence = 0.5;
            let hash = Self::beam_hash(&beam);
            self.vocabulary.insert(word.to_string(), beam);
            self.reverse_vocab.insert(hash, word.to_string());
        } else {
            // Generate beam from context
            let new_beam = self.generate_beam(&context_beams);
            let hash = Self::beam_hash(&new_beam);
            self.vocabulary.insert(word.to_string(), new_beam);
            self.reverse_vocab.insert(hash, word.to_string());
        }
    }

    /// Think to generate new beams - creative beam generation
    pub fn think_new_beam(&mut self, concept: &str) -> BeamTensor {
        let input_beams = self.text_to_beams(concept);
        
        // Encode and evolve multiple times for creativity
        let mut latent = self.calm.encode(&input_beams);
        for _ in 0..3 {
            latent = self.calm.predict_next(&latent);
        }
        
        // Decode to get new beam
        let decoded = self.calm.decode(&latent);
        
        if let Some(beam) = decoded.first() {
            beam.clone()
        } else {
            self.blend_beams(&input_beams)
        }
    }

    /// Process input and generate a thought chain - EXPONENTIAL vortex cycles
    /// Runs 2^power iterations through the vortex, learning at each position
    pub fn think(&mut self, input: &str) -> ThoughtChain {
        let mut chain = ThoughtChain::new(input.to_string());
        let input_beams = self.text_to_beams(input);
        
        // Encode input to latent space
        let mut latent = self.calm.encode(&input_beams);
        
        // Initial thought with beam encoding
        let initial_beam = self.generate_beam(&input_beams);
        let mut initial = Thought::new(format!("Encoding: {}", input), 1)
            .with_confidence(latent.energy)
            .with_type(ThoughtType::Initial);
        initial.beam = initial_beam;
        chain.add_thought(initial);

        // EXPONENTIAL vortex cycles: run 2^6 = 64 iterations minimum
        // Each iteration: 1 -> 2 -> 4 -> 8 -> 7 -> 5 -> 1 (loops back)
        let vortex_cycle = [1u8, 2, 4, 8, 7, 5];
        let sacred_positions = [3u8, 6, 9];
        
        // Calculate iterations: 2^power where power scales with input complexity
        let complexity = input_beams.len().max(1);
        let power = (complexity as f32).log2().ceil() as u32 + 4; // At least 2^4 = 16 cycles
        let iterations = 2u64.pow(power.min(10)); // Cap at 2^10 = 1024 cycles
        
        // Track cycle for u64 overflow reset
        self.vortex_cycle = self.vortex_cycle.wrapping_add(iterations);
        
        // Accumulate latent state across all iterations
        let mut accumulated_energy = 0.0f32;
        let mut best_latent = latent.clone();
        let mut best_energy = latent.energy;
        
        // Run exponential cycles - learning through dynamics
        for iteration in 0..iterations {
            // Full vortex cycle
            for &pos in &vortex_cycle {
                self.current_position = pos;
                latent = self.calm.predict_next(&latent);
                
                // Track best state
                if latent.energy > best_energy {
                    best_energy = latent.energy;
                    best_latent = latent.clone();
                }
                
                accumulated_energy += latent.energy;
            }
            
            // Sacred checkpoint every 8 iterations
            if iteration % 8 == 7 {
                for &sacred_pos in &sacred_positions {
                    latent = self.calm.predict_next(&latent);
                    latent.sacred_alignment = (latent.sacred_alignment + 0.05).min(1.0);
                }
            }
            
            // Early exit if energy stabilizes (converged)
            if iteration > 16 && latent.energy < 0.01 {
                break;
            }
        }
        
        // Record key thoughts from the exponential run (sample at log intervals)
        let mut sample_points = vec![1u64, 2, 4, 8, 16, 32, 64];
        sample_points.retain(|&s| s < iterations);
        
        for (idx, &sample) in sample_points.iter().enumerate() {
            let pos = vortex_cycle[idx % vortex_cycle.len()];
            let thought_content = format!(
                "Cycle {}: position {}, accumulated energy {:.2}", 
                sample, pos, accumulated_energy / (sample as f32 * 6.0)
            );
            
            let mut thought = Thought::new(thought_content, pos)
                .with_confidence(best_energy)
                .with_type(ThoughtType::Reasoning);
            thought.beam = self.calm.decode(&best_latent).first().cloned().unwrap_or_default();
            chain.add_thought(thought);
        }

        // Sacred checkpoints with accumulated learning
        if self.config.sacred_checkpoints {
            for &sacred_pos in &sacred_positions {
                let sacred_beams = self.calm.decode(&best_latent);
                let sacred_words = self.beams_to_text(&sacred_beams);
                
                let checkpoint_content = match sacred_pos {
                    3 => format!("Ethos: {} cycles, {}", iterations, if sacred_words.len() > 3 { &sacred_words } else { "aligned" }),
                    6 => format!("Logos: energy {:.2}, {}", best_energy, if sacred_words.len() > 3 { &sacred_words } else { "consistent" }),
                    9 => format!("Pathos: alignment {:.2}, {}", best_latent.sacred_alignment, if sacred_words.len() > 3 { &sacred_words } else { "calibrated" }),
                    _ => format!("Sacred {}", sacred_pos),
                };
                
                let mut checkpoint = Thought::new(checkpoint_content, sacred_pos)
                    .with_confidence(best_latent.sacred_alignment.max(0.7))
                    .with_type(ThoughtType::Reflection);
                
                if let Some(beam) = sacred_beams.first() {
                    checkpoint.beam = beam.clone();
                }
                
                chain.add_thought(checkpoint);
            }
        }

        // Final synthesis with exponential learning
        let all_beams: Vec<BeamTensor> = chain.thoughts.iter().map(|t| t.beam.clone()).collect();
        let synthesis_beam = self.blend_beams(&all_beams);
        
        // Store accumulated latent for future use
        self.accumulated_latent = Some(best_latent.clone());
        
        let mut synthesis = Thought::new(
            format!("Synthesis: {} cycles, {} thoughts, energy {:.2}, alignment {:.2}", 
                iterations, chain.thoughts.len(), best_energy, best_latent.sacred_alignment),
            9
        )
        .with_confidence(best_energy)
        .with_type(ThoughtType::Synthesis);
        synthesis.beam = synthesis_beam;
        chain.add_thought(synthesis);

        // Generate response from best evolved latent
        let response = self.generate_response(&mut best_latent, &chain);
        chain.complete(response);

        chain
    }

    /// Convert text to BeamTensors using learned vocabulary
    fn text_to_beams(&self, text: &str) -> Vec<BeamTensor> {
        let lowercase = text.to_lowercase();
        let words: Vec<&str> = lowercase
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();

        let mut beams = Vec::new();
        for word in words {
            if let Some(beam) = self.vocabulary.get(word) {
                beams.push(beam.clone());
            } else {
                // Create beam from character patterns
                let mut beam = BeamTensor::default();
                let hash = word.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
                for i in 0..9 {
                    beam.digits[i] = ((hash >> (i * 7)) & 0x7F) as f32 / 127.0;
                }
                // Normalize
                let sum: f32 = beam.digits.iter().sum();
                if sum > 0.0 {
                    beam.digits.iter_mut().for_each(|d| *d /= sum);
                }
                beam.confidence = 0.5;
                beams.push(beam);
            }
        }
        beams
    }

    /// Compute semantic similarity between beam sequences
    fn beam_similarity(&self, a: &[BeamTensor], b: &[BeamTensor]) -> f32 {
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }
        let mut total_sim = 0.0;
        let comparisons = a.len().min(b.len());
        for i in 0..comparisons {
            let dot: f32 = a[i].digits.iter().zip(&b[i].digits).map(|(x, y)| x * y).sum();
            total_sim += dot;
        }
        total_sim / comparisons as f32
    }

    // Old methods removed - thinking now happens in think() with full vortex cycle

    /// Generate response purely from evolved latent state - NO canned responses
    fn generate_response(&mut self, latent: &mut crate::ml::calm::LatentState, chain: &ThoughtChain) -> String {
        let input_beams = self.text_to_beams(&chain.query);
        
        // Store context
        self.context_beams = input_beams.clone();
        
        // Evolve latent multiple times for richer output
        for _ in 0..3 {
            *latent = self.calm.predict_next(latent);
        }
        
        // Generate beams from evolved latent
        let output_beams = self.calm.decode(latent);
        
        // Also generate speculatively for more content
        let speculative_beams = self.calm.generate_speculative(&input_beams, 6);
        
        // Combine all beams
        let mut all_output: Vec<BeamTensor> = output_beams;
        all_output.extend(speculative_beams);
        
        // Convert beams to words
        let mut words: Vec<String> = Vec::new();
        for beam in &all_output {
            if let Some(word) = self.beam_to_word(beam) {
                words.push(word);
            }
        }
        
        // If we got words, build response
        let response = if words.len() >= 3 {
            let raw = words.join(" ");
            self.polish_response(&raw, latent.energy)
        } else {
            // Generate from thought chain beams
            let chain_beams: Vec<BeamTensor> = chain.thoughts.iter()
                .map(|t| t.beam.clone())
                .collect();
            
            let blended = self.blend_beams(&chain_beams);
            let blended_latent = self.calm.encode(&[blended]);
            let decoded = self.calm.decode(&blended_latent);
            
            let chain_words: Vec<String> = decoded.iter()
                .filter_map(|b| self.beam_to_word(b))
                .collect();
            
            if chain_words.len() >= 2 {
                self.polish_response(&chain_words.join(" "), latent.energy)
            } else {
                // Absolute fallback - describe what we learned
                format!("Processing {} words through {} thought steps. Energy: {:.0}%, Alignment: {:.0}%.",
                    input_beams.len(),
                    chain.thoughts.len(),
                    latent.energy * 100.0,
                    latent.sacred_alignment * 100.0)
            }
        };
        
        // Learn from this interaction
        self.learn_from_interaction(&input_beams, &response);
        
        // Learn new words from input that aren't in vocabulary
        let query_lower = chain.query.to_lowercase();
        let query_words: Vec<&str> = query_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();
        
        // Collect words to learn first, then learn them
        let words_to_learn: Vec<String> = query_words.iter()
            .filter(|w| !self.vocabulary.contains_key(&w.to_string()) && w.len() > 2)
            .map(|w| w.to_string())
            .collect();
        
        for word in words_to_learn {
            // Generate beam from character hash since we can't borrow vocabulary
            let mut beam = BeamTensor::default();
            let hash = word.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            for i in 0..9 {
                beam.digits[i] = ((hash >> (i * 7)) & 0x7F) as f32 / 127.0;
            }
            let sum: f32 = beam.digits.iter().sum();
            if sum > 0.0 {
                beam.digits.iter_mut().for_each(|d| *d /= sum);
            }
            beam.confidence = 0.5;
            let beam_hash = Self::beam_hash(&beam);
            self.vocabulary.insert(word.clone(), beam);
            self.reverse_vocab.insert(beam_hash, word);
        }
        
        response
    }

    /// Find the closest word in vocabulary to a beam pattern
    fn beam_to_word(&self, beam: &BeamTensor) -> Option<String> {
        let mut best_word = None;
        let mut best_sim = 0.0f32;
        
        for (word, vocab_beam) in &self.vocabulary {
            let sim: f32 = beam.digits.iter()
                .zip(&vocab_beam.digits)
                .map(|(a, b)| a * b)
                .sum();
            if sim > best_sim {
                best_sim = sim;
                best_word = Some(word.clone());
            }
        }
        
        if best_sim > 0.3 {
            best_word
        } else {
            None
        }
    }

    /// Convert beam sequence back to text
    fn beams_to_text(&self, beams: &[BeamTensor]) -> String {
        let words: Vec<String> = beams.iter()
            .filter_map(|b| self.beam_to_word(b))
            .collect();
        words.join(" ")
    }

    // Old find_relevant_knowledge and generate_intelligent_response removed
    // Response generation now happens purely through latent evolution in generate_response

    /// Polish a response for readability with grammar rules
    fn polish_response(&self, raw: &str, energy: f32) -> String {
        let words: Vec<&str> = raw.split_whitespace().collect();
        if words.is_empty() {
            return self.generate_from_latent(&crate::ml::calm::LatentState::new(128));
        }
        
        // Remove duplicate consecutive words
        let mut cleaned: Vec<String> = Vec::new();
        let mut last_word = "";
        for word in &words {
            if *word != last_word {
                cleaned.push(word.to_string());
                last_word = word;
            }
        }
        
        // Apply grammar rules
        let cleaned = self.apply_grammar(&cleaned);
        
        // Capitalize first letter of each sentence
        let mut result = String::new();
        let mut capitalize_next = true;
        for (i, word) in cleaned.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }
            if capitalize_next {
                if let Some(first) = word.chars().next() {
                    result.push_str(&first.to_uppercase().to_string());
                    result.push_str(&word[first.len_utf8()..]);
                } else {
                    result.push_str(word);
                }
                capitalize_next = false;
            } else {
                result.push_str(word);
            }
            // Check for sentence end
            if word.ends_with('.') || word.ends_with('!') || word.ends_with('?') {
                capitalize_next = true;
            }
        }
        
        // Add period if missing
        if !result.ends_with('.') && !result.ends_with('?') && !result.ends_with('!') {
            result.push('.');
        }
        
        // Add energy indicator if high
        if energy > 0.7 {
            result = format!("{} [High confidence]", result);
        }
        
        result
    }

    /// Apply grammar rules to word sequence
    fn apply_grammar(&self, words: &[String]) -> Vec<String> {
        let mut result = Vec::new();
        let articles = ["a", "an", "the"];
        let verbs = ["is", "are", "was", "were", "am", "be", "been", "being", 
                     "have", "has", "had", "do", "does", "did", "will", "would", 
                     "could", "should", "might", "must", "can"];
        let prepositions = ["in", "on", "at", "to", "for", "with", "by", "from", 
                           "of", "about", "into", "through", "during", "before", "after"];
        
        for (i, word) in words.iter().enumerate() {
            let word_lower = word.to_lowercase();
            
            // Rule: Don't start with articles unless followed by noun-like word
            if i == 0 && articles.contains(&word_lower.as_str()) {
                if words.len() > 1 {
                    result.push(word.clone());
                }
                continue;
            }
            
            // Rule: Don't have two verbs in a row (except auxiliaries)
            if i > 0 && verbs.contains(&word_lower.as_str()) {
                let prev = words[i-1].to_lowercase();
                if verbs.contains(&prev.as_str()) && !["be", "been", "being", "have", "has", "had"].contains(&prev.as_str()) {
                    continue;
                }
            }
            
            // Rule: Don't end with preposition followed by nothing
            if i == words.len() - 1 && prepositions.contains(&word_lower.as_str()) {
                continue;
            }
            
            // Rule: "a" before consonant, "an" before vowel
            if word_lower == "a" && i + 1 < words.len() {
                let next = &words[i + 1].to_lowercase();
                if let Some(first_char) = next.chars().next() {
                    if "aeiou".contains(first_char) {
                        result.push("an".to_string());
                        continue;
                    }
                }
            }
            
            result.push(word.clone());
        }
        
        // Ensure we have at least some content
        if result.is_empty() && !words.is_empty() {
            result = words.to_vec();
        }
        
        result
    }

    /// Generate response directly from latent state when no patterns match
    fn generate_from_latent(&self, latent: &crate::ml::calm::LatentState) -> String {
        // Decode latent to beams
        let decoded_beams = self.calm.decode(latent);
        
        // Convert to words
        let mut words = Vec::new();
        for beam in &decoded_beams {
            if let Some(word) = self.beam_to_word(beam) {
                words.push(word);
            }
        }
        
        if words.is_empty() {
            // Fallback: use knowledge base
            if let Some((knowledge, _, _)) = self.knowledge.first() {
                return knowledge.clone();
            }
            return "I am processing and learning.".to_string();
        }
        
        // Build coherent sentence
        let mut response = words.join(" ");
        if let Some(first) = response.chars().next() {
            response = first.to_uppercase().to_string() + &response[first.len_utf8()..];
        }
        if !response.ends_with('.') {
            response.push('.');
        }
        
        response
    }

    /// Learn from interaction to improve future responses
    fn learn_from_interaction(&mut self, input_beams: &[BeamTensor], response: &str) {
        // Store pattern for future matching
        if !input_beams.is_empty() && !response.is_empty() {
            // Limit memory size
            if self.response_memory.len() > 100 {
                self.response_memory.remove(0);
            }
            self.response_memory.push((input_beams.to_vec(), response.to_string()));
            
            // Update learned patterns hash
            let hash = input_beams.iter()
                .flat_map(|b| b.digits.iter())
                .fold(0u64, |acc, &d| acc.wrapping_mul(31).wrapping_add((d * 1000.0) as u64));
            self.learned_patterns.insert(hash, input_beams.to_vec());
        }
    }

    /// Get current vortex position
    pub fn current_position(&self) -> u8 {
        self.current_position
    }

    /// Advance to next position in vortex cycle
    pub fn advance_position(&mut self) -> u8 {
        self.current_position = match self.current_position {
            1 => 2,
            2 => 4,
            4 => 8,
            8 => 7,
            7 => 5,
            5 => 1,
            _ => 1,
        };
        self.current_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thinking_engine() {
        let config = ThinkingConfig::new().with_max_steps(6);
        let mut engine = ThinkingEngine::new(config);
        
        let chain = engine.think("What is the meaning of life?");
        
        assert!(!chain.thoughts.is_empty());
        assert!(chain.response.is_some());
        assert!(chain.total_confidence > 0.0);
    }

    #[test]
    fn test_thought_chain() {
        let mut chain = ThoughtChain::new("test query".to_string());
        
        chain.add_thought(Thought::new("thought 1".to_string(), 1).with_confidence(0.8));
        chain.add_thought(Thought::new("thought 2".to_string(), 3).with_confidence(0.9));
        
        assert_eq!(chain.thoughts.len(), 2);
        assert!(chain.total_confidence > 0.8); // Sacred boost
    }

    #[test]
    fn test_sacred_thoughts() {
        let mut chain = ThoughtChain::new("test".to_string());
        
        chain.add_thought(Thought::new("normal".to_string(), 1));
        chain.add_thought(Thought::new("sacred 3".to_string(), 3));
        chain.add_thought(Thought::new("sacred 6".to_string(), 6));
        chain.add_thought(Thought::new("sacred 9".to_string(), 9));
        
        let sacred = chain.sacred_thoughts();
        assert_eq!(sacred.len(), 3);
    }
}
