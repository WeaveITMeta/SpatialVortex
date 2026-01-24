//! Flux Matrix Engine - Core 1→2→4→8→7→5→1 vortex pattern

use crate::error::Result;
use crate::data::models::*;
use crate::data::attributes::Attributes;
use super::pattern_coherence::PatternCoherenceTracker;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Core Flux Matrix implementation
#[derive(Clone, Debug)]
pub struct FluxMatrixEngine {
    pub base_pattern: [u8; 7],
    pub sacred_positions: [u8; 3],
    pub pattern_tracker: PatternCoherenceTracker,
    current_position: u8,
}

impl Default for FluxMatrixEngine {
    fn default() -> Self {
        Self {
            base_pattern: [1, 2, 4, 8, 7, 5, 1],
            sacred_positions: [3, 6, 9],
            pattern_tracker: PatternCoherenceTracker::new(100),
            current_position: 1,
        }
    }
}

impl FluxMatrixEngine {
    pub fn new() -> Self { Self::default() }

    pub fn create_matrix(&self, subject: String) -> Result<FluxMatrix> {
        let mut nodes = HashMap::new();
        let mut sacred_guides = HashMap::new();

        for position in 0..=9 {
            if self.sacred_positions.contains(&position) {
                sacred_guides.insert(position, SacredGuide {
                    position,
                    divine_properties: vec![self.sacred_significance(position).to_string()],
                    geometric_significance: self.sacred_significance(position).to_string(),
                });
            } else {
                nodes.insert(position, FluxNode {
                    position,
                    base_value: position,
                    attributes: Attributes::new(),
                    connections: Vec::new(),
                });
            }
        }

        Ok(FluxMatrix {
            id: Uuid::new_v4(),
            subject,
            nodes,
            sacred_guides,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    fn sacred_significance(&self, position: u8) -> &'static str {
        match position {
            3 => "Unity - Ethos checkpoint",
            6 => "Heart - Pathos checkpoint",
            9 => "Ultimate - Logos checkpoint",
            _ => "Unknown",
        }
    }

    pub fn digital_root(&self, n: u64) -> u8 {
        if n == 0 { return 0; }
        let r = (n % 9) as u8;
        if r == 0 { 9 } else { r }
    }

    pub fn is_sacred(&self, position: u8) -> bool {
        self.sacred_positions.contains(&position)
    }

    pub fn next_vortex_position(&self, current: u8) -> u8 {
        match current {
            1 => 2, 2 => 4, 4 => 8, 8 => 7, 7 => 5, 5 => 1,
            _ => 1,
        }
    }

    pub fn advance(&mut self) -> u8 {
        self.current_position = self.next_vortex_position(self.current_position);
        self.current_position
    }

    pub fn current_position(&self) -> u8 { self.current_position }
    pub fn reset(&mut self) { self.current_position = 1; }
}
