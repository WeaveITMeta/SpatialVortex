//! Pattern Coherence Tracker

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PatternCoherenceTracker {
    window_size: usize,
    positions: Vec<u8>,
}

impl PatternCoherenceTracker {
    pub fn new(window_size: usize) -> Self {
        Self { window_size, positions: Vec::with_capacity(window_size) }
    }

    pub fn record(&mut self, position: u8) -> f32 {
        self.positions.push(position);
        if self.positions.len() > self.window_size {
            self.positions.remove(0);
        }
        self.calculate_coherence()
    }

    fn calculate_coherence(&self) -> f32 {
        if self.positions.is_empty() { return 0.0; }
        let sacred = self.positions.iter().filter(|&&p| matches!(p, 3 | 6 | 9)).count();
        sacred as f32 / self.positions.len() as f32
    }

    pub fn reset(&mut self) { self.positions.clear(); }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoherenceMetrics {
    pub current_coherence: f32,
    pub sacred_ratio: f32,
    pub sample_count: usize,
}
