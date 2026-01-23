//! Vortex Cycle Runtime Engine
//! 
//! Implements the sacred doubling/halving sequence for neural network propagation.
//! Objects float through the cycle pattern, interpolating between nodes with exponential timing.
//! 
//! ## Cycle Patterns
//! - **Forward (Doubling)**: 1 → 2 → 4 → 8 → 7 → 5 → 1 (expansion, inference)
//! - **Backward (Halving)**: 1 → 5 → 7 → 8 → 4 → 2 → 1 (contraction, training)
//! - **Sacred Anchors**: 3, 6, 9 (bend and guide tensor paths)
//! 
//! ## Runtime Behavior
//! Each object in the flux matrix propagates through positions:
//! - Position advances exponentially: 2^n steps
//! - Interpolation between nodes based on cycle phase
//! - Sacred geometry guides determine tensor curvature

use crate::models::ELPTensor;
use crate::error::{Result, SpatialVortexError};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Forward doubling sequence (1→2→4→8→7→5→1)
pub const FORWARD_SEQUENCE: [u8; 7] = [1, 2, 4, 8, 7, 5, 1];

/// Backward halving sequence (1→5→7→8→4→2→1)  
pub const BACKWARD_SEQUENCE: [u8; 7] = [1, 5, 7, 8, 4, 2, 1];

/// Sacred anchor positions (excluded from cycle)
pub const SACRED_ANCHORS: [u8; 3] = [3, 6, 9];

/// Direction of cycle propagation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CycleDirection {
    Forward,  // Doubling sequence (inference)
    Backward, // Halving sequence (training)
}

/// Current position in the vortex cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CyclePosition {
    /// Current position (0-9)
    pub position: u8,
    
    /// Index in sequence (0-6 for forward/backward)
    pub sequence_index: usize,
    
    /// Direction of propagation
    pub direction: CycleDirection,
    
    /// Number of complete cycles
    pub cycle_count: u64,
    
    /// Interpolation factor (0.0-1.0) between current and next position
    pub interpolation: f64,
    
    /// Exponential runtime step (2^n)
    pub exponential_step: u64,
    
    /// Start time of current cycle (not serialized)
    #[serde(skip)]
    pub cycle_start: Instant,
}

impl Default for CyclePosition {
    fn default() -> Self {
        Self::new_forward()
    }
}

impl CyclePosition {
    /// Create new position at start of forward sequence
    pub fn new_forward() -> Self {
        Self {
            position: 1,
            sequence_index: 0,
            direction: CycleDirection::Forward,
            cycle_count: 0,
            interpolation: 0.0,
            exponential_step: 1,
            cycle_start: Instant::now(),
        }
    }
    
    /// Create new position at start of backward sequence
    pub fn new_backward() -> Self {
        Self {
            position: 1,
            sequence_index: 0,
            direction: CycleDirection::Backward,
            cycle_count: 0,
            interpolation: 0.0,
            exponential_step: 1,
            cycle_start: Instant::now(),
        }
    }
    
    /// Get current sequence being used
    #[inline]
    pub fn sequence(&self) -> &[u8; 7] {
        match self.direction {
            CycleDirection::Forward => &FORWARD_SEQUENCE,
            CycleDirection::Backward => &BACKWARD_SEQUENCE,
        }
    }
    
    /// Get next position in sequence
    pub fn next_position(&self) -> u8 {
        let seq = self.sequence();
        let next_idx = (self.sequence_index + 1) % seq.len();
        seq[next_idx]
    }
    
    /// Advance to next position in cycle
    pub fn advance(&mut self) {
        let seq_len = self.sequence().len();
        self.sequence_index = (self.sequence_index + 1) % seq_len;
        let seq = self.sequence();
        self.position = seq[self.sequence_index];
        
        // Reset interpolation
        self.interpolation = 0.0;
        
        // Exponential step growth: 2^n
        self.exponential_step = self.exponential_step.saturating_mul(2);
        
        // Cycle completed?
        if self.sequence_index == 0 {
            self.cycle_count += 1;
            self.cycle_start = Instant::now();
        }
    }
    
    /// Update interpolation based on progress (0.0-1.0)
    pub fn update_interpolation(&mut self, progress: f64) {
        self.interpolation = progress.clamp(0.0, 1.0);
    }
    
    /// Get interpolated position between current and next
    pub fn interpolated_position(&self) -> f64 {
        let current = self.position as f64;
        let next = self.next_position() as f64;
        
        // Handle wrap-around (1→1 at end of cycle)
        if (next - current).abs() > 5.0 {
            // Wrap around through 0
            current + self.interpolation * (10.0 - current)
        } else {
            current + self.interpolation * (next - current)
        }
    }
    
    /// Check if at sacred anchor (3, 6, or 9)
    pub fn is_at_sacred_anchor(&self) -> bool {
        SACRED_ANCHORS.contains(&self.position)
    }
    
    /// Get nearest sacred anchor for bending guidance
    pub fn nearest_sacred_anchor(&self) -> u8 {
        SACRED_ANCHORS
            .iter()
            .min_by_key(|&&anchor| {
                let diff = (anchor as i32 - self.position as i32).abs();
                diff.min(10 - diff) // Wrap around
            })
            .copied()
            .unwrap_or(3)
    }
    
    /// Calculate bend factor from sacred anchor (0.0-1.0)
    pub fn sacred_bend_factor(&self) -> f64 {
        let nearest = self.nearest_sacred_anchor();
        let distance = (self.position as i32 - nearest as i32).abs();
        let wrapped_distance = distance.min(10 - distance);
        
        // Closer to sacred anchor = stronger bend
        1.0 - (wrapped_distance as f64 / 5.0)
    }
}

/// Propagating object in the vortex cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CycleObject {
    /// Unique identifier
    pub id: String,
    
    /// Current cycle position
    pub position: CyclePosition,
    
    /// ELP tensor at current position
    pub tensor: ELPTensor,
    
    /// Target position (where object wants to go)
    pub target_position: Option<u8>,
    
    /// Velocity through cycle
    pub velocity: f64,
    
    /// Last update time (not serialized)
    #[serde(skip)]
    pub last_update: Instant,
}

impl Default for CycleObject {
    fn default() -> Self {
        Self::new("default".to_string(), ELPTensor::new(0.5, 0.5, 0.5), CycleDirection::Forward)
    }
}

impl CycleObject {
    /// Create new object at starting position
    pub fn new(id: String, tensor: ELPTensor, direction: CycleDirection) -> Self {
        let position = match direction {
            CycleDirection::Forward => CyclePosition::new_forward(),
            CycleDirection::Backward => CyclePosition::new_backward(),
        };
        
        Self {
            id,
            position,
            tensor,
            target_position: None,
            velocity: 1.0,
            last_update: Instant::now(),
        }
    }
    
    /// Update object position based on elapsed time
    pub fn update(&mut self, delta_time: Duration) {
        let dt = delta_time.as_secs_f64();
        
        // Progress based on velocity and exponential step
        let base_progress = dt * self.velocity;
        let exponential_factor = 1.0 / (self.position.exponential_step as f64).sqrt();
        let progress = base_progress * exponential_factor;
        
        // Update interpolation
        let new_interpolation = self.position.interpolation + progress;
        
        if new_interpolation >= 1.0 {
            // Advance to next position
            self.position.advance();
            self.position.interpolation = new_interpolation - 1.0;
        } else {
            self.position.interpolation = new_interpolation;
        }
        
        self.last_update = Instant::now();
    }
    
    /// Apply sacred geometry bend to tensor
    pub fn apply_sacred_bend(&mut self, anchor_tensors: &[ELPTensor; 3]) {
        let nearest = self.position.nearest_sacred_anchor();
        let bend_factor = self.position.sacred_bend_factor();
        
        // Get anchor tensor (3→index 0, 6→index 1, 9→index 2)
        let anchor_idx = match nearest {
            3 => 0,
            6 => 1,
            9 => 2,
            _ => 0,
        };
        let anchor_tensor = anchor_tensors[anchor_idx];
        
        // Interpolate tensor toward anchor based on bend factor
        self.tensor = ELPTensor::new(
            self.tensor.ethos * (1.0 - bend_factor) + anchor_tensor.ethos * bend_factor,
            self.tensor.logos * (1.0 - bend_factor) + anchor_tensor.logos * bend_factor,
            self.tensor.pathos * (1.0 - bend_factor) + anchor_tensor.pathos * bend_factor,
        );
    }
    
    /// Check if object has reached target
    pub fn reached_target(&self) -> bool {
        if let Some(target) = self.target_position {
            self.position.position == target && self.position.interpolation > 0.9
        } else {
            false
        }
    }
}

/// Vortex cycle engine managing multiple objects
pub struct VortexCycleEngine {
    /// Objects propagating through cycle
    objects: Arc<RwLock<Vec<CycleObject>>>,
    
    /// Sacred anchor tensors (3, 6, 9)
    sacred_anchors: [ELPTensor; 3],
    
    /// Engine running state
    running: Arc<RwLock<bool>>,
    
    /// Update rate (Hz)
    update_rate: f64,
}

impl VortexCycleEngine {
    /// Create new cycle engine
    pub fn new(update_rate: f64) -> Self {
        Self {
            objects: Arc::new(RwLock::new(Vec::new())),
            sacred_anchors: [
                ELPTensor::new(1.0, 0.0, 0.0), // Position 3: Pure Ethos
                ELPTensor::new(0.0, 0.0, 1.0), // Position 6: Pure Pathos
                ELPTensor::new(0.0, 1.0, 0.0), // Position 9: Pure Logos
            ],
            running: Arc::new(RwLock::new(false)),
            update_rate,
        }
    }
    
    /// Set sacred anchor tensors
    pub fn set_sacred_anchors(&mut self, anchors: [ELPTensor; 3]) {
        self.sacred_anchors = anchors;
    }
    
    /// Add object to cycle
    pub async fn add_object(&self, object: CycleObject) {
        let mut objects = self.objects.write().await;
        objects.push(object);
    }
    
    /// Remove object from cycle
    pub async fn remove_object(&self, id: &str) {
        let mut objects = self.objects.write().await;
        objects.retain(|obj| obj.id != id);
    }
    
    /// Get all objects (for visualization)
    pub async fn get_objects(&self) -> Vec<CycleObject> {
        let objects = self.objects.read().await;
        objects.clone()
    }
    
    /// Start the cycle engine
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(SpatialVortexError::InvalidFluxMatrix("Engine already running".to_string()));
        }
        *running = true;
        
        // Spawn update loop
        let objects = Arc::clone(&self.objects);
        let running_flag = Arc::clone(&self.running);
        let update_interval = Duration::from_secs_f64(1.0 / self.update_rate);
        let sacred_anchors = self.sacred_anchors;
        
        tokio::spawn(async move {
            let mut last_update = Instant::now();
            
            while *running_flag.read().await {
                let now = Instant::now();
                let delta = now - last_update;
                last_update = now;
                
                // Update all objects
                let mut objects_lock = objects.write().await;
                for object in objects_lock.iter_mut() {
                    object.update(delta);
                    object.apply_sacred_bend(&sacred_anchors);
                }
                drop(objects_lock);
                
                // Sleep until next update
                tokio::time::sleep(update_interval).await;
            }
        });
        
        Ok(())
    }
    
    /// Stop the cycle engine
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }
    
    /// Check if engine is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
    
    /// Get object by ID
    pub async fn get_object(&self, id: &str) -> Option<CycleObject> {
        let objects = self.objects.read().await;
        objects.iter().find(|obj| obj.id == id).cloned()
    }
    
    /// Get objects at specific position
    pub async fn get_objects_at_position(&self, position: u8) -> Vec<CycleObject> {
        let objects = self.objects.read().await;
        objects
            .iter()
            .filter(|obj| obj.position.position == position)
            .cloned()
            .collect()
    }
    
    /// Get statistics
    pub async fn stats(&self) -> CycleStats {
        let objects = self.objects.read().await;
        
        let total_objects = objects.len();
        let forward_count = objects
            .iter()
            .filter(|obj| obj.position.direction == CycleDirection::Forward)
            .count();
        let backward_count = total_objects - forward_count;
        
        let avg_cycle_count = if !objects.is_empty() {
            objects.iter().map(|obj| obj.position.cycle_count).sum::<u64>() as f64
                / total_objects as f64
        } else {
            0.0
        };
        
        let avg_velocity = if !objects.is_empty() {
            objects.iter().map(|obj| obj.velocity).sum::<f64>() / total_objects as f64
        } else {
            0.0
        };
        
        CycleStats {
            total_objects,
            forward_count,
            backward_count,
            avg_cycle_count,
            avg_velocity,
            update_rate: self.update_rate,
        }
    }
}

/// Cycle engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleStats {
    pub total_objects: usize,
    pub forward_count: usize,
    pub backward_count: usize,
    pub avg_cycle_count: f64,
    pub avg_velocity: f64,
    pub update_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_sequence() {
        let mut pos = CyclePosition::new_forward();
        
        // Verify sequence: 1→2→4→8→7→5→1
        assert_eq!(pos.position, 1);
        pos.advance();
        assert_eq!(pos.position, 2);
        pos.advance();
        assert_eq!(pos.position, 4);
        pos.advance();
        assert_eq!(pos.position, 8);
        pos.advance();
        assert_eq!(pos.position, 7);
        pos.advance();
        assert_eq!(pos.position, 5);
        pos.advance();
        assert_eq!(pos.position, 1);
        
        // Verify cycle count
        assert_eq!(pos.cycle_count, 1);
    }

    #[test]
    fn test_backward_sequence() {
        let mut pos = CyclePosition::new_backward();
        
        // Verify sequence: 1→5→7→8→4→2→1
        assert_eq!(pos.position, 1);
        pos.advance();
        assert_eq!(pos.position, 5);
        pos.advance();
        assert_eq!(pos.position, 7);
        pos.advance();
        assert_eq!(pos.position, 8);
        pos.advance();
        assert_eq!(pos.position, 4);
        pos.advance();
        assert_eq!(pos.position, 2);
        pos.advance();
        assert_eq!(pos.position, 1);
        
        assert_eq!(pos.cycle_count, 1);
    }

    #[test]
    fn test_exponential_steps() {
        let mut pos = CyclePosition::new_forward();
        
        assert_eq!(pos.exponential_step, 1);
        pos.advance();
        assert_eq!(pos.exponential_step, 2);
        pos.advance();
        assert_eq!(pos.exponential_step, 4);
        pos.advance();
        assert_eq!(pos.exponential_step, 8);
    }

    #[test]
    fn test_sacred_anchors_excluded() {
        // Verify 3, 6, 9 never appear in sequences
        for &pos in &FORWARD_SEQUENCE {
            assert!(!SACRED_ANCHORS.contains(&pos));
        }
        for &pos in &BACKWARD_SEQUENCE {
            assert!(!SACRED_ANCHORS.contains(&pos));
        }
    }

    #[test]
    fn test_nearest_sacred_anchor() {
        let mut pos = CyclePosition::new_forward();
        
        pos.position = 1;
        assert_eq!(pos.nearest_sacred_anchor(), 3);
        
        pos.position = 5;
        assert_eq!(pos.nearest_sacred_anchor(), 6);
        
        pos.position = 8;
        assert_eq!(pos.nearest_sacred_anchor(), 9);
    }

    #[tokio::test]
    async fn test_cycle_engine() {
        let engine = VortexCycleEngine::new(60.0); // 60 Hz
        
        let obj = CycleObject::new(
            "test1".to_string(),
            ELPTensor::new(0.5, 0.5, 0.5),
            CycleDirection::Forward,
        );
        
        engine.add_object(obj).await;
        
        let objects = engine.get_objects().await;
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].id, "test1");
    }
}
