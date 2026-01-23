//! Sacred Pattern Coherence Tracking
//!
//! Real-time 3-6-9 recurrence tracking and pattern coherence measurement.
//! This is the mathematical foundation, not heuristics.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Pattern coherence tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCoherenceTracker {
    /// Recent positions (for pattern analysis)
    position_history: VecDeque<u8>,
    
    /// History size
    max_history: usize,
    
    /// Current coherence score
    current_coherence: f32,
    
    /// 3-6-9 recurrence frequency
    sacred_frequency: f32,
    
    /// Digital root coherence
    digital_root_coherence: f32,
    
    /// Vortex cycle coherence
    vortex_cycle_coherence: f32,
}

impl PatternCoherenceTracker {
    /// Create new pattern coherence tracker
    pub fn new(max_history: usize) -> Self {
        Self {
            position_history: VecDeque::with_capacity(max_history),
            max_history,
            current_coherence: 1.0,
            sacred_frequency: 0.333, // Expected 3/9
            digital_root_coherence: 1.0,
            vortex_cycle_coherence: 1.0,
        }
    }
    
    /// Record a position and update coherence
    pub fn record_position(&mut self, position: u8) {
        // Add to history
        if self.position_history.len() >= self.max_history {
            self.position_history.pop_front();
        }
        self.position_history.push_back(position);
        
        // Recalculate coherence
        self.update_coherence();
    }
    
    /// Update all coherence metrics
    fn update_coherence(&mut self) {
        if self.position_history.is_empty() {
            return;
        }
        
        // Calculate 3-6-9 recurrence frequency
        self.sacred_frequency = self.calculate_sacred_frequency();
        
        // Calculate digital root coherence
        self.digital_root_coherence = self.calculate_digital_root_coherence();
        
        // Calculate vortex cycle coherence
        self.vortex_cycle_coherence = self.calculate_vortex_cycle_coherence();
        
        // Overall coherence is weighted average
        self.current_coherence = 
            self.sacred_frequency * 0.4 +
            self.digital_root_coherence * 0.3 +
            self.vortex_cycle_coherence * 0.3;
    }
    
    /// Calculate 3-6-9 recurrence frequency
    fn calculate_sacred_frequency(&self) -> f32 {
        let sacred_count = self.position_history.iter()
            .filter(|&&pos| [3, 6, 9].contains(&pos))
            .count();
        
        let actual_frequency = sacred_count as f32 / self.position_history.len() as f32;
        let expected_frequency = 3.0 / 9.0; // 33.3%
        
        // Coherence is inverse of deviation from expected
        let deviation = (actual_frequency - expected_frequency).abs();
        let coherence = 1.0 - (deviation / expected_frequency).min(1.0);
        
        coherence
    }
    
    /// Calculate digital root coherence
    fn calculate_digital_root_coherence(&self) -> f32 {
        if self.position_history.len() < 2 {
            return 1.0;
        }
        
        let mut coherent_transitions = 0;
        let mut total_transitions = 0;
        
        for window in self.position_history.iter().collect::<Vec<_>>().windows(2) {
            let curr = self.digital_root(*window[0] as usize);
            let next = self.digital_root(*window[1] as usize);
            
            // Check if follows vortex pattern (1→2→4→8→7→5→1)
            let expected_next = match curr {
                1 => 2,
                2 => 4,
                4 => 8,
                8 => 7,
                7 => 5,
                5 => 1,
                _ => curr, // 3, 6, 9 map to themselves
            };
            
            if next == expected_next || [3, 6, 9].contains(&curr) {
                coherent_transitions += 1;
            }
            total_transitions += 1;
        }
        
        if total_transitions > 0 {
            coherent_transitions as f32 / total_transitions as f32
        } else {
            1.0
        }
    }
    
    /// Calculate vortex cycle coherence
    fn calculate_vortex_cycle_coherence(&self) -> f32 {
        if self.position_history.len() < 6 {
            return 1.0;
        }
        
        // Check if we see complete vortex cycles (1→2→4→8→7→5→1)
        let vortex_pattern = [1, 2, 4, 8, 7, 5];
        let mut cycle_matches = 0;
        let mut possible_cycles = 0;
        
        for window in self.position_history.iter().collect::<Vec<_>>().windows(6) {
            let matches_pattern = window.iter()
                .zip(vortex_pattern.iter())
                .filter(|(&a, &b)| *a == b)
                .count();
            
            if matches_pattern >= 4 { // At least 4/6 match
                cycle_matches += 1;
            }
            possible_cycles += 1;
        }
        
        if possible_cycles > 0 {
            cycle_matches as f32 / possible_cycles as f32
        } else {
            1.0
        }
    }
    
    /// Calculate digital root
    fn digital_root(&self, mut n: usize) -> u8 {
        while n >= 10 {
            n = n.to_string().chars()
                .filter_map(|c| c.to_digit(10))
                .sum::<u32>() as usize;
        }
        if n == 0 { 9 } else { n as u8 }
    }
    
    /// Get current coherence score
    pub fn get_coherence(&self) -> f32 {
        self.current_coherence
    }
    
    /// Get sacred frequency
    pub fn get_sacred_frequency(&self) -> f32 {
        self.sacred_frequency
    }
    
    /// Get digital root coherence
    pub fn get_digital_root_coherence(&self) -> f32 {
        self.digital_root_coherence
    }
    
    /// Get vortex cycle coherence
    pub fn get_vortex_cycle_coherence(&self) -> f32 {
        self.vortex_cycle_coherence
    }
    
    /// Check if pattern is degrading
    pub fn is_degrading(&self) -> bool {
        self.current_coherence < 0.7
    }
    
    /// Get degradation severity (0-1, higher is worse)
    pub fn degradation_severity(&self) -> f32 {
        if self.current_coherence >= 0.7 {
            0.0
        } else {
            (0.7 - self.current_coherence) / 0.7
        }
    }
    
    /// Export coherence metrics for performance tracking
    pub fn export_metrics(&self) -> CoherenceMetrics {
        CoherenceMetrics {
            overall_coherence: self.current_coherence,
            sacred_frequency: self.sacred_frequency,
            digital_root_coherence: self.digital_root_coherence,
            vortex_cycle_coherence: self.vortex_cycle_coherence,
            is_degrading: self.is_degrading(),
            degradation_severity: self.degradation_severity(),
        }
    }
    
    /// Clear history and reset
    pub fn reset(&mut self) {
        self.position_history.clear();
        self.current_coherence = 1.0;
        self.sacred_frequency = 0.333;
        self.digital_root_coherence = 1.0;
        self.vortex_cycle_coherence = 1.0;
    }
}

/// Coherence metrics for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceMetrics {
    /// Overall pattern coherence (0-1)
    pub overall_coherence: f32,
    
    /// 3-6-9 recurrence frequency
    pub sacred_frequency: f32,
    
    /// Digital root coherence
    pub digital_root_coherence: f32,
    
    /// Vortex cycle coherence
    pub vortex_cycle_coherence: f32,
    
    /// Is pattern degrading
    pub is_degrading: bool,
    
    /// Degradation severity (0-1)
    pub degradation_severity: f32,
}

impl Default for PatternCoherenceTracker {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_perfect_vortex_cycle() {
        let mut tracker = PatternCoherenceTracker::new(10);
        
        // Perfect vortex cycle
        let cycle = [1, 2, 4, 8, 7, 5, 1, 2, 4, 8];
        for &pos in &cycle {
            tracker.record_position(pos);
        }
        
        // Should have high coherence
        assert!(tracker.get_coherence() > 0.8);
        assert!(tracker.get_vortex_cycle_coherence() > 0.8);
    }
    
    #[test]
    fn test_sacred_positions() {
        let mut tracker = PatternCoherenceTracker::new(9);
        
        // Exactly 3 sacred positions out of 9
        let positions = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        for &pos in &positions {
            tracker.record_position(pos);
        }
        
        // Should have perfect sacred frequency
        let freq = tracker.get_sacred_frequency();
        assert!((freq - 1.0).abs() < 0.1); // Close to 1.0 (perfect)
    }
    
    #[test]
    fn test_degradation_detection() {
        let mut tracker = PatternCoherenceTracker::new(10);
        
        // Random positions (should degrade pattern)
        let random = [1, 5, 2, 8, 3, 7, 4, 6, 9, 1];
        for &pos in &random {
            tracker.record_position(pos);
        }
        
        // Should detect degradation
        assert!(tracker.get_coherence() < 0.9);
    }
    
    #[test]
    fn test_digital_root() {
        let tracker = PatternCoherenceTracker::new(10);
        
        assert_eq!(tracker.digital_root(0), 9);
        assert_eq!(tracker.digital_root(9), 9);
        assert_eq!(tracker.digital_root(18), 9);
        assert_eq!(tracker.digital_root(27), 9);
        assert_eq!(tracker.digital_root(12), 3);
        assert_eq!(tracker.digital_root(15), 6);
    }
}
