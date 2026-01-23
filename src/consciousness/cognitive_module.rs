//! Cognitive Module - Specialized processors that compete for attention

use async_trait::async_trait;
use anyhow::Result;
use super::thought::Thought;

/// Response from a cognitive module
#[derive(Debug, Clone)]
pub struct ModuleResponse {
    pub thoughts: Vec<Thought>,
    pub attention_request: AttentionScore,
    pub module_state: String,
}

/// How urgently a module wants attention
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AttentionScore(pub f64);

impl AttentionScore {
    pub fn urgent() -> Self { Self(1.0) }
    pub fn high() -> Self { Self(0.75) }
    pub fn medium() -> Self { Self(0.5) }
    pub fn low() -> Self { Self(0.25) }
    pub fn none() -> Self { Self(0.0) }
}

/// Trait for all cognitive modules
#[async_trait]
pub trait CognitiveModule: Send + Sync {
    /// Name of this module
    fn name(&self) -> &str;
    
    /// Specialty/domain of this module
    fn specialty(&self) -> ModuleSpecialty;
    
    /// Process incoming stimulus and generate thoughts
    async fn process(&self, input: &str) -> Result<ModuleResponse>;
    
    /// Compete for attention in the global workspace
    fn compete_for_attention(&self) -> AttentionScore;
    
    /// Receive a broadcast thought from global workspace
    async fn receive_broadcast(&mut self, thought: &Thought) -> Result<()>;
    
    /// Get current internal state
    fn get_state(&self) -> String;
}

/// Different specialties of cognitive modules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleSpecialty {
    /// Moral and ethical reasoning
    Ethics,
    
    /// Logical analysis and reasoning
    Logic,
    
    /// Emotional processing and empathy
    Emotion,
    
    /// Creative synthesis and novel ideas
    Creativity,
    
    /// Critical thinking and skepticism
    Criticism,
    
    /// Memory retrieval and association
    Memory,
    
    /// Pattern recognition
    Pattern,
    
    /// Planning and strategy
    Planning,
    
    /// Self-monitoring and meta-cognition
    SelfMonitoring,
}

impl ModuleSpecialty {
    /// Default ELP weights for this specialty
    pub fn default_elp(&self) -> (f64, f64, f64) {
        match self {
            ModuleSpecialty::Ethics => (0.7, 0.2, 0.1),      // High ethos
            ModuleSpecialty::Logic => (0.1, 0.8, 0.1),       // High logos
            ModuleSpecialty::Emotion => (0.1, 0.1, 0.8),     // High pathos
            ModuleSpecialty::Creativity => (0.2, 0.3, 0.5),  // Balanced, slight pathos
            ModuleSpecialty::Criticism => (0.3, 0.6, 0.1),   // Logic-focused
            ModuleSpecialty::Memory => (0.33, 0.33, 0.34),   // Balanced
            ModuleSpecialty::Pattern => (0.2, 0.7, 0.1),     // Logic-heavy
            ModuleSpecialty::Planning => (0.3, 0.5, 0.2),    // Logic-focused
            ModuleSpecialty::SelfMonitoring => (0.4, 0.4, 0.2), // Ethics + Logic
        }
    }
}
