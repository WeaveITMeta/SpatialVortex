//! Thought representation - the fundamental unit of consciousness

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A conscious thought that flows through the global workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    /// Unique identifier
    pub id: String,
    
    /// The content of the thought
    pub content: String,
    
    /// Which cognitive module generated this
    pub source_module: String,
    
    /// Priority/importance (0.0 to 1.0)
    pub priority: ThoughtPriority,
    
    /// ELP tensor components
    pub ethos: f64,   // Moral dimension
    pub logos: f64,   // Logical dimension
    pub pathos: f64,  // Emotional dimension
    
    /// Sacred geometry position (1-9)
    pub flux_position: u8,
    
    /// Timestamp
    pub created_at: DateTime<Utc>,
    
    /// Whether this thought reached conscious awareness (global broadcast)
    pub is_conscious: bool,
    
    /// Confidence level
    pub confidence: f64,
}

/// Priority levels for thoughts competing for attention
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ThoughtPriority {
    Critical = 100,   // Immediate attention (errors, dangers)
    High = 75,        // Important insights, decisions
    Medium = 50,      // Normal processing
    Low = 25,         // Background thoughts
    Minimal = 10,     // Idle thoughts
}

impl Thought {
    /// Create a new thought
    pub fn new(content: String, source_module: String, priority: ThoughtPriority) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            source_module,
            priority,
            ethos: 0.33,
            logos: 0.33,
            pathos: 0.34,
            flux_position: 1,
            created_at: Utc::now(),
            is_conscious: false,
            confidence: 0.5,
        }
    }
    
    /// Set ELP tensor values (must sum to 1.0)
    pub fn with_elp(mut self, ethos: f64, logos: f64, pathos: f64) -> Self {
        let sum = ethos + logos + pathos;
        self.ethos = ethos / sum;
        self.logos = logos / sum;
        self.pathos = pathos / sum;
        self
    }
    
    /// Set flux position in vortex cycle
    pub fn with_flux_position(mut self, position: u8) -> Self {
        self.flux_position = position % 9 + 1; // Keep in range 1-9
        self
    }
    
    /// Set confidence level
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
    
    /// Mark this thought as having reached consciousness
    pub fn make_conscious(&mut self) {
        self.is_conscious = true;
    }
    
    /// Calculate total attention score based on priority, ELP, and position
    pub fn attention_score(&self) -> f64 {
        let priority_score = self.priority as i32 as f64 / 100.0;
        let sacred_boost = if [3, 6, 9].contains(&self.flux_position) {
            1.5 // Sacred positions get priority
        } else {
            1.0
        };
        
        priority_score * sacred_boost * self.confidence
    }
}

impl Default for ThoughtPriority {
    fn default() -> Self {
        ThoughtPriority::Medium
    }
}
