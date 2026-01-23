//! Attention Mechanism - Decides which thoughts reach consciousness

use super::thought::Thought;
use std::collections::VecDeque;

/// Maximum thoughts that can be conscious at once (working memory limit)
const CONSCIOUSNESS_CAPACITY: usize = 7; // Miller's Law: 7±2 items

/// Manages the "spotlight of attention" in the global workspace
#[derive(Debug)]
pub struct AttentionMechanism {
    /// Currently conscious thoughts (in the spotlight)
    conscious_thoughts: VecDeque<Thought>,
    
    /// Capacity of working memory
    capacity: usize,
    
    /// Sacred geometry boost factor (3-6-9 positions)
    #[allow(dead_code)]
    sacred_boost: f64,
}

impl AttentionMechanism {
    pub fn new() -> Self {
        Self {
            conscious_thoughts: VecDeque::with_capacity(CONSCIOUSNESS_CAPACITY),
            capacity: CONSCIOUSNESS_CAPACITY,
            sacred_boost: 1.5,
        }
    }
    
    /// Select which thoughts deserve conscious attention
    pub fn select_conscious_thoughts(&mut self, candidate_thoughts: Vec<Thought>) -> Vec<Thought> {
        let mut thoughts = candidate_thoughts;
        
        // Sort by attention score (highest first)
        thoughts.sort_by(|a, b| {
            b.attention_score()
                .partial_cmp(&a.attention_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take top N based on capacity
        let selected: Vec<Thought> = thoughts
            .into_iter()
            .take(self.capacity)
            .map(|mut t| {
                t.make_conscious();
                t
            })
            .collect();
        
        // Update conscious thoughts buffer
        self.conscious_thoughts.clear();
        for thought in &selected {
            self.conscious_thoughts.push_back(thought.clone());
        }
        
        // Trim if over capacity
        while self.conscious_thoughts.len() > self.capacity {
            self.conscious_thoughts.pop_front();
        }
        
        selected
    }
    
    /// Get currently conscious thoughts
    pub fn get_conscious_thoughts(&self) -> Vec<Thought> {
        self.conscious_thoughts.iter().cloned().collect()
    }
    
    /// Check if working memory is full
    pub fn is_at_capacity(&self) -> bool {
        self.conscious_thoughts.len() >= self.capacity
    }
    
    /// Get current attention load (0.0 to 1.0)
    pub fn attention_load(&self) -> f64 {
        self.conscious_thoughts.len() as f64 / self.capacity as f64
    }
    
    /// Clear all conscious thoughts (mental reset)
    pub fn clear(&mut self) {
        self.conscious_thoughts.clear();
    }
    
    /// Filter thoughts based on minimum threshold
    pub fn filter_by_threshold(&self, thoughts: Vec<Thought>, threshold: f64) -> Vec<Thought> {
        thoughts
            .into_iter()
            .filter(|t| t.attention_score() >= threshold)
            .collect()
    }
}

impl Default for AttentionMechanism {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consciousness::thought::ThoughtPriority;
    
    #[test]
    fn test_attention_capacity() {
        let mut attention = AttentionMechanism::new();
        
        // Create 10 thoughts
        let thoughts: Vec<Thought> = (0..10)
            .map(|i| {
                Thought::new(
                    format!("Thought {}", i),
                    "test".to_string(),
                    ThoughtPriority::Medium,
                )
            })
            .collect();
        
        let conscious = attention.select_conscious_thoughts(thoughts);
        
        // Should only select up to capacity (7)
        assert!(conscious.len() <= CONSCIOUSNESS_CAPACITY);
        
        // All selected should be marked conscious
        assert!(conscious.iter().all(|t| t.is_conscious));
    }
    
    #[test]
    fn test_sacred_position_priority() {
        let mut attention = AttentionMechanism::new();
        
        let sacred = Thought::new(
            "Sacred thought".to_string(),
            "test".to_string(),
            ThoughtPriority::Low,
        ).with_flux_position(3); // Sacred position
        
        let normal = Thought::new(
            "Normal thought".to_string(),
            "test".to_string(),
            ThoughtPriority::Low,
        ).with_flux_position(2); // Not sacred
        
        // Sacred position should score higher despite same priority
        assert!(sacred.attention_score() > normal.attention_score());
    }
}
