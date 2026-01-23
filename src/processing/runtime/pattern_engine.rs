//! Pattern Engine - Flexible Vortex Traversal Patterns
//! 
//! Supports multiple traversal patterns while preserving the sacred doubling sequence
//! as the optimal default. Additional patterns for experimentation and visualization.
//! 
//! ## Core Patterns
//! - **Sacred Doubling** (1→2→4→8→7→5→1): PERFECT, optimal energy flow
//! - **Linear Ascending** (1→2→3→4→5→6→7→8→9→1): Simple progression
//! - **Linear Descending** (9→8→7→6→5→4→3→2→1→9): Reverse progression
//! - **Custom Steps**: Variable step sizes per segment
//! 
//! ## Design Principle
//! The sacred doubling pattern remains the default and recommended pattern.
//! Other patterns exist for comparative analysis and specialized use cases.

use serde::{Deserialize, Serialize};

/// Pattern type for vortex traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// Sacred doubling sequence - OPTIMAL (1→2→4→8→7→5→1)
    SacredDoubling,
    
    /// Sacred halving sequence - for backpropagation (1→5→7→8→4→2→1)
    SacredHalving,
    
    /// Linear ascending (1→2→3→4→5→6→7→8→9→1)
    LinearAscending,
    
    /// Linear descending (9→8→7→6→5→4→3→2→1→9)
    LinearDescending,
    
    /// Custom step pattern (user-defined)
    Custom,
}

/// Vortex traversal pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VortexPattern {
    /// Pattern type
    pub pattern_type: PatternType,
    
    /// Sequence of positions (0-9)
    pub sequence: Vec<u8>,
    
    /// Human-readable name
    pub name: String,
    
    /// Description
    pub description: String,
    
    /// Whether this pattern is sacred (mathematically proven optimal)
    pub is_sacred: bool,
}

impl VortexPattern {
    /// Create the sacred doubling pattern (DEFAULT, OPTIMAL)
    pub fn sacred_doubling() -> Self {
        Self {
            pattern_type: PatternType::SacredDoubling,
            sequence: vec![1, 2, 4, 8, 7, 5, 1],
            name: "Sacred Doubling".to_string(),
            description: "Perfect energy flow pattern via doubling modulo 9. Mathematically optimal.".to_string(),
            is_sacred: true,
        }
    }
    
    /// Create the sacred halving pattern (for backpropagation)
    pub fn sacred_halving() -> Self {
        Self {
            pattern_type: PatternType::SacredHalving,
            sequence: vec![1, 5, 7, 8, 4, 2, 1],
            name: "Sacred Halving".to_string(),
            description: "Perfect gradient flow pattern via halving modulo 9. Optimal for learning.".to_string(),
            is_sacred: true,
        }
    }
    
    /// Create linear ascending pattern (experimental)
    pub fn linear_ascending() -> Self {
        Self {
            pattern_type: PatternType::LinearAscending,
            sequence: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 1],
            name: "Linear Ascending".to_string(),
            description: "Simple linear progression. Experimental - not optimal.".to_string(),
            is_sacred: false,
        }
    }
    
    /// Create linear descending pattern (experimental)
    pub fn linear_descending() -> Self {
        Self {
            pattern_type: PatternType::LinearDescending,
            sequence: vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 9],
            name: "Linear Descending".to_string(),
            description: "Reverse linear progression. Experimental - not optimal.".to_string(),
            is_sacred: false,
        }
    }
    
    /// Create custom pattern with variable steps
    /// 
    /// # Example
    /// ```rust
    /// // Step by 2: 1→3→5→7→9→2→4→6→8→1
    /// let pattern = VortexPattern::custom_steps(1, 2, "Step by 2");
    /// ```
    pub fn custom_steps(start: u8, step_size: u8, name: &str) -> Self {
        let mut sequence = Vec::new();
        let mut current = start % 10;
        let start_pos = current;
        
        loop {
            sequence.push(current);
            current = (current + step_size) % 10;
            
            // Avoid infinite loops
            if current == start_pos || sequence.len() > 20 {
                break;
            }
        }
        
        // Close the loop
        if sequence.last() != Some(&start_pos) {
            sequence.push(start_pos);
        }
        
        Self {
            pattern_type: PatternType::Custom,
            sequence,
            name: name.to_string(),
            description: format!("Custom pattern: start={}, step={}", start, step_size),
            is_sacred: false,
        }
    }
    
    /// Create custom pattern from explicit sequence
    pub fn from_sequence(sequence: Vec<u8>, name: &str, description: &str) -> Self {
        Self {
            pattern_type: PatternType::Custom,
            sequence,
            name: name.to_string(),
            description: description.to_string(),
            is_sacred: false,
        }
    }
    
    /// Get next position in sequence
    pub fn next_position(&self, current_index: usize) -> (u8, usize) {
        let next_idx = (current_index + 1) % self.sequence.len();
        let next_pos = self.sequence[next_idx];
        (next_pos, next_idx)
    }
    
    /// Get previous position in sequence
    pub fn prev_position(&self, current_index: usize) -> (u8, usize) {
        let prev_idx = if current_index == 0 {
            self.sequence.len() - 1
        } else {
            current_index - 1
        };
        let prev_pos = self.sequence[prev_idx];
        (prev_pos, prev_idx)
    }
    
    /// Check if position is in this pattern
    pub fn contains_position(&self, position: u8) -> bool {
        self.sequence.contains(&position)
    }
    
    /// Get index of position in sequence
    pub fn position_index(&self, position: u8) -> Option<usize> {
        self.sequence.iter().position(|&p| p == position)
    }
    
    /// Check if this pattern visits sacred anchors (3, 6, 9)
    pub fn visits_sacred_anchors(&self) -> bool {
        [3, 6, 9].iter().any(|&anchor| self.sequence.contains(&anchor))
    }
    
    /// Get pattern efficiency score (sacred patterns = 1.0)
    pub fn efficiency_score(&self) -> f64 {
        if self.is_sacred {
            1.0
        } else {
            // Score based on:
            // - Length (shorter is better)
            // - Coverage (more positions is better)
            // - Sacred anchor avoidance (good, they're checkpoints)
            
            let length_score = 1.0 - (self.sequence.len() as f64 / 20.0).min(1.0);
            let coverage_score = self.unique_positions() as f64 / 10.0;
            let sacred_penalty = if self.visits_sacred_anchors() { 0.8 } else { 1.0 };
            
            (length_score * 0.3 + coverage_score * 0.7) * sacred_penalty
        }
    }
    
    /// Count unique positions in sequence
    pub fn unique_positions(&self) -> usize {
        let mut positions = self.sequence.clone();
        positions.sort_unstable();
        positions.dedup();
        positions.len()
    }
    
    /// Validate pattern
    pub fn is_valid(&self) -> bool {
        if self.sequence.is_empty() {
            return false;
        }
        
        // All positions must be 0-9
        if self.sequence.iter().any(|&p| p > 9) {
            return false;
        }
        
        // Pattern should loop back to start
        if self.sequence.first() != self.sequence.last() {
            return false;
        }
        
        true
    }
}

impl Default for VortexPattern {
    fn default() -> Self {
        // Default to sacred doubling - the OPTIMAL pattern
        Self::sacred_doubling()
    }
}

/// Pattern comparator for analysis
pub struct PatternComparator {
    patterns: Vec<VortexPattern>,
}

impl PatternComparator {
    /// Create new comparator
    pub fn new() -> Self {
        Self {
            patterns: vec![
                VortexPattern::sacred_doubling(),
                VortexPattern::sacred_halving(),
                VortexPattern::linear_ascending(),
                VortexPattern::linear_descending(),
            ],
        }
    }
    
    /// Add custom pattern for comparison
    pub fn add_pattern(&mut self, pattern: VortexPattern) {
        self.patterns.push(pattern);
    }
    
    /// Compare all patterns and generate report
    pub fn compare_all(&self) -> Vec<PatternAnalysis> {
        self.patterns.iter().map(|p| {
            PatternAnalysis {
                name: p.name.clone(),
                pattern_type: p.pattern_type,
                length: p.sequence.len(),
                unique_positions: p.unique_positions(),
                visits_sacred: p.visits_sacred_anchors(),
                efficiency: p.efficiency_score(),
                is_sacred: p.is_sacred,
            }
        }).collect()
    }
    
    /// Get best pattern (always returns sacred doubling)
    pub fn get_optimal(&self) -> &VortexPattern {
        self.patterns.iter()
            .find(|p| p.is_sacred && p.pattern_type == PatternType::SacredDoubling)
            .expect("Sacred doubling pattern must exist")
    }
}

impl Default for PatternComparator {
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysis {
    pub name: String,
    pub pattern_type: PatternType,
    pub length: usize,
    pub unique_positions: usize,
    pub visits_sacred: bool,
    pub efficiency: f64,
    pub is_sacred: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sacred_doubling_is_perfect() {
        let pattern = VortexPattern::sacred_doubling();
        assert_eq!(pattern.efficiency_score(), 1.0);
        assert!(pattern.is_sacred);
        assert_eq!(pattern.sequence, vec![1, 2, 4, 8, 7, 5, 1]);
    }

    #[test]
    fn test_linear_ascending() {
        let pattern = VortexPattern::linear_ascending();
        assert!(!pattern.is_sacred);
        assert_eq!(pattern.sequence, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 1]);
        assert!(pattern.visits_sacred_anchors());
    }

    #[test]
    fn test_custom_steps() {
        let pattern = VortexPattern::custom_steps(1, 2, "Step by 2");
        assert!(pattern.is_valid());
        assert!(!pattern.is_sacred);
    }

    #[test]
    fn test_pattern_comparison() {
        let comparator = PatternComparator::new();
        let analyses = comparator.compare_all();
        
        // Sacred doubling should have highest efficiency
        let sacred = analyses.iter().find(|a| a.is_sacred && a.pattern_type == PatternType::SacredDoubling).unwrap();
        assert_eq!(sacred.efficiency, 1.0);
        
        // All other patterns should have lower efficiency
        for analysis in &analyses {
            if !analysis.is_sacred {
                assert!(analysis.efficiency < 1.0);
            }
        }
    }

    #[test]
    fn test_optimal_pattern() {
        let comparator = PatternComparator::new();
        let optimal = comparator.get_optimal();
        assert_eq!(optimal.pattern_type, PatternType::SacredDoubling);
    }
}
