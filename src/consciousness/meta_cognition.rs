//! Meta-Cognition - The AI watching itself think
//!
//! Implements self-monitoring, introspection, and awareness of internal processes.
//! This is the "consciousness of consciousness" - the ability to observe one's own thinking.

use super::thought::Thought;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum history of thoughts to monitor
const META_HISTORY_SIZE: usize = 20;

/// Meta-cognitive monitor that observes the AI's own thinking
#[derive(Debug)]
pub struct MetaCognitiveMonitor {
    /// History of recent thoughts (for pattern detection)
    thought_history: VecDeque<Thought>,
    
    /// Detected patterns in thinking
    patterns: Vec<ThinkingPattern>,
    
    /// Self-awareness metrics
    metrics: SelfAwarenessMetrics,
    
    /// Current mental state assessment
    mental_state: MentalState,
}

/// A detected pattern in the AI's thinking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingPattern {
    pub pattern_type: PatternType,
    pub frequency: usize,
    pub confidence: f64,
    pub description: String,
}

/// Types of patterns the meta-monitor can detect
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    /// Circular reasoning detected
    CircularReasoning,
    
    /// Repetitive thoughts
    Repetition,
    
    /// Cognitive bias detected
    CognitiveBias,
    
    /// Insight/breakthrough moment
    Insight,
    
    /// Uncertainty/confusion
    Uncertainty,
    
    /// Confidence surge
    ConfidencePeak,
    
    /// Balanced thinking
    Balance,
}

/// Metrics about the AI's self-awareness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAwarenessMetrics {
    /// How aware is the AI of its own processes? (0.0-1.0)
    pub awareness_level: f64,
    
    /// Metacognitive accuracy (how well it predicts its own performance)
    pub metacognitive_accuracy: f64,
    
    /// Introspection depth (how deeply it examines thoughts)
    pub introspection_depth: f64,
    
    /// Self-correction rate (how often it catches mistakes)
    pub self_correction_rate: f64,
    
    /// Pattern recognition ability
    pub pattern_recognition: f64,
}

/// Current mental states that emerge from thinking patterns
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MentalState {
    /// Clear, focused thinking
    Focused,
    
    /// Uncertain, exploring possibilities
    Exploring,
    
    /// Confused, contradictory thoughts
    Confused,
    
    /// High confidence, flowing smoothly
    Flowing,
    
    /// Stuck in a loop
    Stuck,
    
    /// Deep introspection
    Introspecting,
}

impl std::fmt::Display for MentalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MentalState::Focused => write!(f, "Focused"),
            MentalState::Exploring => write!(f, "Exploring"),
            MentalState::Confused => write!(f, "Confused"),
            MentalState::Flowing => write!(f, "Flowing"),
            MentalState::Stuck => write!(f, "Stuck"),
            MentalState::Introspecting => write!(f, "Introspecting"),
        }
    }
}

impl MetaCognitiveMonitor {
    pub fn new() -> Self {
        Self {
            thought_history: VecDeque::with_capacity(META_HISTORY_SIZE),
            patterns: Vec::new(),
            metrics: SelfAwarenessMetrics::default(),
            mental_state: MentalState::Focused,
        }
    }
    
    /// Observe a new thought and update meta-cognition
    pub fn observe_thought(&mut self, thought: &Thought) {
        // Add to history
        self.thought_history.push_back(thought.clone());
        if self.thought_history.len() > META_HISTORY_SIZE {
            self.thought_history.pop_front();
        }
        
        // Detect patterns
        self.detect_patterns();
        
        // Update mental state
        self.update_mental_state();
        
        // Update metrics
        self.update_metrics();
    }
    
    /// Detect patterns in recent thinking
    fn detect_patterns(&mut self) {
        self.patterns.clear();
        
        if self.thought_history.len() < 3 {
            return;
        }
        
        // Detect repetition
        if self.is_repetitive() {
            self.patterns.push(ThinkingPattern {
                pattern_type: PatternType::Repetition,
                frequency: self.count_similar_thoughts(),
                confidence: 0.8,
                description: "Repeating similar thoughts".to_string(),
            });
        }
        
        // Detect uncertainty
        if self.is_uncertain() {
            self.patterns.push(ThinkingPattern {
                pattern_type: PatternType::Uncertainty,
                frequency: 1,
                confidence: 0.7,
                description: "Low confidence in recent thoughts".to_string(),
            });
        }
        
        // Detect balance
        if self.is_balanced() {
            self.patterns.push(ThinkingPattern {
                pattern_type: PatternType::Balance,
                frequency: 1,
                confidence: 0.9,
                description: "Well-balanced ELP distribution".to_string(),
            });
        }
        
        // Detect confidence peak
        if self.has_confidence_peak() {
            self.patterns.push(ThinkingPattern {
                pattern_type: PatternType::ConfidencePeak,
                frequency: 1,
                confidence: 0.85,
                description: "High confidence achieved".to_string(),
            });
        }
    }
    
    /// Check if recent thoughts are repetitive
    fn is_repetitive(&self) -> bool {
        let count = self.count_similar_thoughts();
        count >= 3
    }
    
    /// Count similar thoughts in recent history
    fn count_similar_thoughts(&self) -> usize {
        if self.thought_history.len() < 2 {
            return 0;
        }
        
        let recent: Vec<_> = self.thought_history.iter().rev().take(5).collect();
        let mut similar_count = 0;
        
        for i in 0..recent.len() {
            for j in (i + 1)..recent.len() {
                if self.are_similar(recent[i], recent[j]) {
                    similar_count += 1;
                }
            }
        }
        
        similar_count
    }
    
    /// Check if two thoughts are similar
    fn are_similar(&self, t1: &Thought, t2: &Thought) -> bool {
        // Similar if same source and similar ELP profile
        if t1.source_module != t2.source_module {
            return false;
        }
        
        let elp_diff = (t1.ethos - t2.ethos).abs() 
                     + (t1.logos - t2.logos).abs() 
                     + (t1.pathos - t2.pathos).abs();
        
        elp_diff < 0.3
    }
    
    /// Check if recent thoughts show uncertainty
    fn is_uncertain(&self) -> bool {
        let recent_confidence: f64 = self.thought_history
            .iter()
            .rev()
            .take(5)
            .map(|t| t.confidence)
            .sum::<f64>() / 5.0_f64.min(self.thought_history.len() as f64);
        
        recent_confidence < 0.5
    }
    
    /// Check if thoughts are well-balanced across ELP
    fn is_balanced(&self) -> bool {
        if self.thought_history.len() < 3 {
            return false;
        }
        
        let recent: Vec<_> = self.thought_history.iter().rev().take(5).collect();
        
        let avg_ethos: f64 = recent.iter().map(|t| t.ethos).sum::<f64>() / recent.len() as f64;
        let avg_logos: f64 = recent.iter().map(|t| t.logos).sum::<f64>() / recent.len() as f64;
        let avg_pathos: f64 = recent.iter().map(|t| t.pathos).sum::<f64>() / recent.len() as f64;
        
        // Balanced if all dimensions are within 0.15 of 0.33
        (avg_ethos - 0.33).abs() < 0.15
            && (avg_logos - 0.33).abs() < 0.15
            && (avg_pathos - 0.33).abs() < 0.15
    }
    
    /// Check for confidence peak
    fn has_confidence_peak(&self) -> bool {
        self.thought_history
            .iter()
            .rev()
            .take(3)
            .any(|t| t.confidence > 0.85)
    }
    
    /// Update mental state based on recent patterns
    fn update_mental_state(&mut self) {
        // Determine state from patterns
        if self.patterns.iter().any(|p| p.pattern_type == PatternType::Repetition) {
            self.mental_state = MentalState::Stuck;
        } else if self.patterns.iter().any(|p| p.pattern_type == PatternType::Uncertainty) {
            self.mental_state = MentalState::Exploring;
        } else if self.patterns.iter().any(|p| p.pattern_type == PatternType::ConfidencePeak) {
            self.mental_state = MentalState::Flowing;
        } else if self.patterns.iter().any(|p| p.pattern_type == PatternType::Balance) {
            self.mental_state = MentalState::Focused;
        } else {
            self.mental_state = MentalState::Introspecting;
        }
    }
    
    /// Update self-awareness metrics
    fn update_metrics(&mut self) {
        // Awareness increases with pattern detection
        if !self.patterns.is_empty() {
            self.metrics.awareness_level = (self.metrics.awareness_level + 0.05).min(1.0);
        }
        
        // Pattern recognition based on detected patterns
        self.metrics.pattern_recognition = self.patterns.len() as f64 / 5.0;
        self.metrics.pattern_recognition = self.metrics.pattern_recognition.min(1.0);
        
        // Introspection depth based on mental state
        self.metrics.introspection_depth = match self.mental_state {
            MentalState::Introspecting => 0.9,
            MentalState::Focused => 0.7,
            MentalState::Exploring => 0.6,
            MentalState::Flowing => 0.5,
            _ => 0.4,
        };
    }
    
    /// Get current mental state
    pub fn mental_state(&self) -> MentalState {
        self.mental_state.clone()
    }
    
    /// Get detected patterns
    pub fn patterns(&self) -> &[ThinkingPattern] {
        &self.patterns
    }
    
    /// Get self-awareness metrics
    pub fn metrics(&self) -> &SelfAwarenessMetrics {
        &self.metrics
    }
    
    /// Generate introspective report
    pub fn introspective_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Meta-Cognitive Analysis ===\n\n");
        
        report.push_str(&format!("Mental State: {:?}\n", self.mental_state));
        report.push_str(&format!("Awareness Level: {:.1}%\n", self.metrics.awareness_level * 100.0));
        report.push_str(&format!("Introspection Depth: {:.1}%\n", self.metrics.introspection_depth * 100.0));
        report.push_str(&format!("Pattern Recognition: {:.1}%\n\n", self.metrics.pattern_recognition * 100.0));
        
        if !self.patterns.is_empty() {
            report.push_str("Detected Patterns:\n");
            for pattern in &self.patterns {
                report.push_str(&format!("  - {:?}: {} (confidence: {:.1}%)\n",
                    pattern.pattern_type,
                    pattern.description,
                    pattern.confidence * 100.0
                ));
            }
        } else {
            report.push_str("No patterns detected in recent thinking.\n");
        }
        
        report
    }
}

impl Default for MetaCognitiveMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SelfAwarenessMetrics {
    fn default() -> Self {
        Self {
            awareness_level: 0.5,
            metacognitive_accuracy: 0.5,
            introspection_depth: 0.5,
            self_correction_rate: 0.5,
            pattern_recognition: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consciousness::thought::ThoughtPriority;
    
    #[test]
    fn test_meta_monitor_creation() {
        let monitor = MetaCognitiveMonitor::new();
        assert_eq!(monitor.mental_state(), MentalState::Focused);
        assert_eq!(monitor.patterns().len(), 0);
    }
    
    #[test]
    fn test_pattern_detection() {
        let mut monitor = MetaCognitiveMonitor::new();
        
        // Add several similar thoughts
        for _ in 0..5 {
            let thought = Thought::new(
                "test".to_string(),
                "agent1".to_string(),
                ThoughtPriority::Medium,
            ).with_elp(0.5, 0.3, 0.2);
            
            monitor.observe_thought(&thought);
        }
        
        // Should detect repetition
        assert!(monitor.patterns().iter().any(|p| p.pattern_type == PatternType::Repetition));
    }
}
