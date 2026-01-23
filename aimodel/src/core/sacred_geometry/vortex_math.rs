//! 🌀 Vortex Mathematics - Full Implementation
//!
//! This module implements the complete vortex mathematics system for positioning
//! semantic content in the flux matrix using the doubling sequence.
//!
//! # Mathematical Foundation
//!
//! **Doubling Sequence** (Forward Flow):
//! ```text
//! 1 → 2 → 4 → 8 → 7 → 5 → 1 (cycles)
//! ```
//!
//! **Sacred Positions** (Never in doubling):
//! ```text
//! 3, 6, 9 (stable attractors, checkpoints)
//! ```
//!
//! **Divine Source**:
//! ```text
//! 0 (origin, neutral balance)
//! ```

use std::f32::consts::PI;

/// Position in the Flux Matrix (0-9)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FluxPosition(pub u8);

impl FluxPosition {
    /// Create a new flux position (0-9)
    pub fn new(pos: u8) -> Self {
        assert!(pos <= 9, "FluxPosition must be 0-9");
        Self(pos)
    }

    /// Check if this is a sacred position (3, 6, 9)
    pub fn is_sacred(&self) -> bool {
        matches!(self.0, 3 | 6 | 9)
    }

    /// Check if this is in the vortex flow (1, 2, 4, 8, 7, 5)
    pub fn is_in_vortex_flow(&self) -> bool {
        matches!(self.0, 1 | 2 | 4 | 8 | 7 | 5)
    }

    /// Check if this is the divine source (0)
    pub fn is_divine_source(&self) -> bool {
        self.0 == 0
    }

    /// Get the next position in the vortex flow
    ///
    /// # Flow Pattern
    /// 1 → 2 → 4 → 8 → 7 → 5 → 1 (cycles)
    pub fn next_in_flow(&self) -> Option<FluxPosition> {
        match self.0 {
            1 => Some(FluxPosition(2)),
            2 => Some(FluxPosition(4)),
            4 => Some(FluxPosition(8)),
            8 => Some(FluxPosition(7)),
            7 => Some(FluxPosition(5)),
            5 => Some(FluxPosition(1)),  // Cycle back
            _ => None,  // Sacred positions and 0 don't flow
        }
    }

    /// Get position name/meaning
    pub fn name(&self) -> &'static str {
        match self.0 {
            0 => "Divine Source / Neutral Balance",
            1 => "New Beginnings / Unity",
            2 => "Duality / Partnership",
            3 => "Sacred Triangle: Ethos / Good",
            4 => "Foundation / Stability",
            5 => "Change / Transformation",
            6 => "Sacred Triangle: Pathos / Emotion",
            7 => "Spiritual Completion / Wisdom",
            8 => "Infinite Potential / Power",
            9 => "Sacred Triangle: Logos / Divine",
            _ => "Unknown",
        }
    }

    /// Get position archetype
    pub fn archetype(&self) -> PositionArchetype {
        match self.0 {
            0 => PositionArchetype::Source,
            1 => PositionArchetype::Flow,
            2 => PositionArchetype::Flow,
            3 => PositionArchetype::Sacred,
            4 => PositionArchetype::Flow,
            5 => PositionArchetype::Flow,
            6 => PositionArchetype::Sacred,
            7 => PositionArchetype::Flow,
            8 => PositionArchetype::Flow,
            9 => PositionArchetype::Sacred,
            _ => unreachable!(),
        }
    }
}

/// Position archetype
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionArchetype {
    /// Divine source (0)
    Source,
    /// Sacred checkpoint (3, 6, 9)
    Sacred,
    /// Vortex flow (1, 2, 4, 8, 7, 5)
    Flow,
}

/// Advanced positioning calculator using full vortex mathematics
#[derive(Debug, Clone)]
pub struct VortexPositioningEngine {
    /// Use gradient positioning (true) or simple dominance (false)
    use_gradient: bool,
}

impl Default for VortexPositioningEngine {
    fn default() -> Self {
        Self {
            use_gradient: true,
        }
    }
}

impl VortexPositioningEngine {
    /// Create a new vortex positioning engine
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable/disable gradient positioning
    pub fn with_gradient(mut self, use_gradient: bool) -> Self {
        self.use_gradient = use_gradient;
        self
    }

    /// 🌀 Calculate flux position from ELP channels
    ///
    /// Uses advanced vortex mathematics to determine optimal position
    /// based on ELP balance and signal strength.
    ///
    /// # Algorithm
    ///
    /// 1. **Check for balance** → Position 0 (Divine Source)
    /// 2. **Identify dominant sacred position** → 3, 6, or 9
    /// 3. **Use gradient positioning** → Flow positions based on ratios
    /// 4. **Apply signal strength** → Modulates between positions
    ///
    /// # Arguments
    /// * `ethos` - Ethos channel (0.0-1.0)
    /// * `logos` - Logos channel (0.0-1.0)
    /// * `pathos` - Pathos channel (0.0-1.0)
    /// * `confidence` - Signal strength (0.0-1.0)
    ///
    /// # Returns
    /// * FluxPosition (0-9)
    pub fn calculate_position(
        &self,
        ethos: f32,
        logos: f32,
        pathos: f32,
        confidence: f32,
    ) -> FluxPosition {
        // Step 1: Check for perfect balance → Position 0 (Divine Source)
        if self.is_balanced(ethos, logos, pathos) {
            return FluxPosition(0);
        }

        // Step 2: Determine dominant channel
        let max_channel = ethos.max(logos).max(pathos);
        
        if !self.use_gradient {
            // Simple dominance-based (Day 3 approach)
            return self.simple_position(ethos, logos, pathos);
        }

        // Step 3: Advanced gradient positioning
        let base_position = if ethos == max_channel {
            // Ethos-dominant → Sacred position 3
            self.position_from_ethos(ethos, logos, pathos, confidence)
        } else if logos == max_channel {
            // Logos-dominant → Sacred position 9
            self.position_from_logos(ethos, logos, pathos, confidence)
        } else {
            // Pathos-dominant → Sacred position 6
            self.position_from_pathos(ethos, logos, pathos, confidence)
        };

        base_position
    }

    /// Check if ELP channels are balanced
    fn is_balanced(&self, ethos: f32, logos: f32, pathos: f32) -> bool {
        let ideal = 1.0 / 3.0;  // Perfect balance
        let threshold = 0.05;    // 5% tolerance
        
        (ethos - ideal).abs() < threshold &&
        (logos - ideal).abs() < threshold &&
        (pathos - ideal).abs() < threshold
    }

    /// Simple dominance-based positioning (fallback)
    fn simple_position(&self, ethos: f32, logos: f32, pathos: f32) -> FluxPosition {
        if ethos > logos && ethos > pathos {
            FluxPosition(3)  // Ethos
        } else if logos > pathos {
            FluxPosition(9)  // Logos
        } else {
            FluxPosition(6)  // Pathos
        }
    }

    /// Position from ethos dominance
    ///
    /// Ethos range: Position 1 → 2 → 3 → 4
    fn position_from_ethos(&self, ethos: f32, logos: f32, pathos: f32, signal: f32) -> FluxPosition {
        // Pure ethos → 3 (sacred)
        if ethos > 0.7 {
            return FluxPosition(3);
        }

        // Ethos + Logos → 1 or 2 (new beginnings, duality)
        if logos > pathos {
            if signal > 0.6 {
                FluxPosition(2)  // Strong signal → Duality/Partnership
            } else {
                FluxPosition(1)  // Weak signal → New Beginnings
            }
        } else {
            // Ethos + Pathos → 2 or 4 (balance, foundation)
            if signal > 0.6 {
                FluxPosition(4)  // Strong signal → Foundation
            } else {
                FluxPosition(2)  // Weak signal → Duality
            }
        }
    }

    /// Position from logos dominance
    ///
    /// Logos range: Position 7 → 8 → 9
    fn position_from_logos(&self, ethos: f32, logos: f32, pathos: f32, _signal: f32) -> FluxPosition {
        // Pure logos → 9 (sacred)
        if logos > 0.7 {
            return FluxPosition(9);
        }

        // Logos + Ethos → 8 (infinite potential)
        if ethos > pathos {
            FluxPosition(8)
        } else {
            // Logos + Pathos → 7 (spiritual completion)
            FluxPosition(7)
        }
    }

    /// Position from pathos dominance
    ///
    /// Pathos range: Position 5 → 6 → 7
    fn position_from_pathos(&self, ethos: f32, logos: f32, pathos: f32, _signal: f32) -> FluxPosition {
        // Pure pathos → 6 (sacred)
        if pathos > 0.7 {
            return FluxPosition(6);
        }

        // Pathos + Logos → 7 (spiritual completion)
        if logos > ethos {
            FluxPosition(7)
        } else {
            // Pathos + Ethos → 5 (change/transformation)
            FluxPosition(5)
        }
    }

    /// Calculate position transition path
    ///
    /// Returns the vortex flow path from current to target position
    pub fn transition_path(&self, from: FluxPosition, to: FluxPosition) -> Vec<FluxPosition> {
        if from == to {
            return vec![from];
        }

        let mut path = vec![from];
        let mut current = from;

        // Follow vortex flow to reach target
        for _ in 0..10 {  // Max 10 steps to prevent infinite loop
            if let Some(next) = current.next_in_flow() {
                path.push(next);
                if next == to {
                    break;
                }
                current = next;
            } else {
                // Current is sacred or source, can't flow
                break;
            }
        }

        path
    }

    /// Calculate angular position on the sacred circle (0-360 degrees)
    ///
    /// Maps flux positions to angles for geometric visualization
    pub fn position_angle(&self, position: FluxPosition) -> f32 {
        match position.0 {
            0 => 0.0,      // Top (North)
            1 => 36.0,     // 1/10 circle
            2 => 72.0,     // 2/10 circle
            3 => 108.0,    // 3/10 circle (Sacred)
            4 => 144.0,    // 4/10 circle
            5 => 180.0,    // Bottom (South)
            6 => 216.0,    // 6/10 circle (Sacred)
            7 => 252.0,    // 7/10 circle
            8 => 288.0,    // 8/10 circle
            9 => 324.0,    // 9/10 circle (Sacred)
            _ => 0.0,
        }
    }

    /// Get Cartesian coordinates for position (for visualization)
    pub fn position_coords(&self, position: FluxPosition) -> (f32, f32) {
        let angle = self.position_angle(position) * PI / 180.0;
        let radius = 1.0;
        (radius * angle.cos(), radius * angle.sin())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sacred_positions() {
        assert!(FluxPosition(3).is_sacred());
        assert!(FluxPosition(6).is_sacred());
        assert!(FluxPosition(9).is_sacred());
        assert!(!FluxPosition(1).is_sacred());
    }

    #[test]
    fn test_vortex_flow() {
        let p1 = FluxPosition(1);
        assert_eq!(p1.next_in_flow(), Some(FluxPosition(2)));
        
        let p5 = FluxPosition(5);
        assert_eq!(p5.next_in_flow(), Some(FluxPosition(1)));  // Cycle
        
        let p3 = FluxPosition(3);
        assert_eq!(p3.next_in_flow(), None);  // Sacred doesn't flow
    }

    #[test]
    fn test_balanced_position() {
        let engine = VortexPositioningEngine::new();
        let pos = engine.calculate_position(0.33, 0.33, 0.34, 0.8);
        assert_eq!(pos, FluxPosition(0));  // Balanced → Divine Source
    }

    #[test]
    fn test_ethos_dominant() {
        let engine = VortexPositioningEngine::new();
        let pos = engine.calculate_position(0.8, 0.1, 0.1, 0.9);
        assert_eq!(pos, FluxPosition(3));  // Strong ethos → Sacred 3
    }

    #[test]
    fn test_logos_dominant() {
        let engine = VortexPositioningEngine::new();
        let pos = engine.calculate_position(0.1, 0.8, 0.1, 0.9);
        assert_eq!(pos, FluxPosition(9));  // Strong logos → Sacred 9
    }

    #[test]
    fn test_pathos_dominant() {
        let engine = VortexPositioningEngine::new();
        let pos = engine.calculate_position(0.1, 0.1, 0.8, 0.9);
        assert_eq!(pos, FluxPosition(6));  // Strong pathos → Sacred 6
    }

    #[test]
    fn test_gradient_positioning() {
        let engine = VortexPositioningEngine::new();
        
        // Moderate ethos with logos mix
        let pos = engine.calculate_position(0.5, 0.3, 0.2, 0.7);
        assert!(pos.0 <= 4);  // Should be in ethos range
        
        // Moderate logos with ethos mix
        let pos = engine.calculate_position(0.2, 0.6, 0.2, 0.7);
        assert!(pos.0 >= 7 && pos.0 <= 9);  // Should be in logos range
    }
}
