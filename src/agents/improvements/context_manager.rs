//! Context Manager - Prevents repetitive introductions
//!
//! Tracks conversation state and adjusts verbosity based on:
//! - Turn count in conversation
//! - Mental state (from consciousness simulator)
//! - User frustration signals


/// Manages conversation context to prevent repetition
pub struct ContextManager {
    /// Number of turns in conversation
    turn_count: usize,
    
    /// Has introduced self this session?
    has_introduced: bool,
    
    /// Recent topics discussed
    recent_topics: Vec<String>,
    
    /// User frustration signals detected
    frustration_signals: usize,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            turn_count: 0,
            has_introduced: false,
            recent_topics: Vec::new(),
            frustration_signals: 0,
        }
    }
    
    /// Increment turn and check if introduction needed
    pub fn should_introduce(&mut self) -> bool {
        self.turn_count += 1;
        
        // Only introduce on first turn
        if self.turn_count == 1 && !self.has_introduced {
            self.has_introduced = true;
            return true;
        }
        
        false
    }
    
    /// Detect if user is frustrated (repeated questions, "still", "again", etc.)
    pub fn detect_frustration(&mut self, query: &str) -> bool {
        let frustration_keywords = [
            "still", "again", "you didn't", "you did not",
            "i told you", "i asked", "answer my question"
        ];
        
        for keyword in &frustration_keywords {
            if query.to_lowercase().contains(keyword) {
                self.frustration_signals += 1;
                return true;
            }
        }
        
        false
    }
    
    /// Check if topic already discussed
    pub fn is_repeat_topic(&mut self, topic: &str) -> bool {
        self.recent_topics.contains(&topic.to_string())
    }
    
    /// Add topic to recent history
    pub fn add_topic(&mut self, topic: String) {
        self.recent_topics.push(topic);
        
        // Keep only last 5 topics
        if self.recent_topics.len() > 5 {
            self.recent_topics.remove(0);
        }
    }
    
    /// Get conversation verbosity level
    pub fn verbosity_level(&self) -> VerbosityLevel {
        if self.frustration_signals > 2 {
            VerbosityLevel::Minimal  // User is frustrated, be concise
        } else if self.turn_count > 10 {
            VerbosityLevel::Moderate  // Long conversation, stay focused
        } else if self.turn_count == 1 {
            VerbosityLevel::Detailed  // First turn, can be more explanatory
        } else {
            VerbosityLevel::Balanced  // Normal conversation flow
        }
    }
}

/// Verbosity levels for response generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbosityLevel {
    /// Minimal: Direct answers only, no fluff
    Minimal,
    
    /// Moderate: Concise with key details
    Moderate,
    
    /// Balanced: Normal conversation flow
    Balanced,
    
    /// Detailed: More explanation (first turn)
    Detailed,
}

impl VerbosityLevel {
    /// Should include introduction section?
    pub fn include_introduction(&self) -> bool {
        matches!(self, VerbosityLevel::Detailed)
    }
    
    /// Should include "How I Work" sections?
    pub fn include_technical_details(&self) -> bool {
        matches!(self, VerbosityLevel::Detailed | VerbosityLevel::Balanced)
    }
    
    /// Should include "Limitations" sections?
    pub fn include_limitations(&self) -> bool {
        matches!(self, VerbosityLevel::Detailed)
    }
    
    /// Maximum paragraph length
    pub fn max_paragraph_sentences(&self) -> usize {
        match self {
            VerbosityLevel::Minimal => 2,
            VerbosityLevel::Moderate => 3,
            VerbosityLevel::Balanced => 4,
            VerbosityLevel::Detailed => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_introduction_once() {
        let mut ctx = ContextManager::new();
        
        assert!(ctx.should_introduce());  // First turn
        assert!(!ctx.should_introduce()); // Second turn
        assert!(!ctx.should_introduce()); // Third turn
    }
    
    #[test]
    fn test_frustration_detection() {
        let mut ctx = ContextManager::new();
        
        assert!(ctx.detect_frustration("You still didn't answer my question"));
        assert!(ctx.detect_frustration("I told you I need web search"));
        assert!(!ctx.detect_frustration("What is consciousness?"));
    }
    
    #[test]
    fn test_verbosity_adjustment() {
        let mut ctx = ContextManager::new();
        
        // First turn: detailed
        ctx.should_introduce();
        assert_eq!(ctx.verbosity_level(), VerbosityLevel::Detailed);
        
        // After frustration: minimal
        ctx.detect_frustration("You didn't answer");
        ctx.detect_frustration("Still waiting");
        ctx.detect_frustration("Again, please");
        assert_eq!(ctx.verbosity_level(), VerbosityLevel::Minimal);
    }
}
