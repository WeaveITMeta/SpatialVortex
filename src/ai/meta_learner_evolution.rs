//! Meta-Learner Evolution
//!
//! Learns new sacred interventions based on coherence recovery success.
//! The system discovers optimal boost factors, orbital radii, and intervention
//! strategies through experience, not hardcoded rules.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::core::sacred_geometry::pattern_coherence::CoherenceMetrics;
use crate::error::Result;

/// Sacred intervention that can be learned and evolved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredIntervention {
    /// Unique ID
    pub id: Uuid,
    
    /// Intervention name
    pub name: String,
    
    /// Sacred position (3, 6, or 9)
    pub position: u8,
    
    /// Boost factor (learned)
    pub boost_factor: f32,
    
    /// Orbital radius (learned)
    pub orbital_radius: f32,
    
    /// Intervention timing (when to apply)
    pub timing: InterventionTiming,
    
    /// Success rate (0-1)
    pub success_rate: f32,
    
    /// Times applied
    pub applications: u64,
    
    /// Times succeeded
    pub successes: u64,
    
    /// Average coherence improvement
    pub avg_coherence_improvement: f32,
    
    /// Created at
    pub created_at: DateTime<Utc>,
    
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

/// When to apply intervention
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterventionTiming {
    /// Always at sacred position
    Always,
    
    /// Only when degradation detected
    OnDegradation,
    
    /// Only when coherence below threshold
    BelowThreshold(u8), // threshold * 100
    
    /// At specific vortex cycle count
    AtCycle(u64),
    
    /// Adaptive (learned timing)
    Adaptive,
}

/// Meta-learner that evolves sacred interventions
pub struct MetaLearnerEvolution {
    /// Known interventions
    interventions: HashMap<Uuid, SacredIntervention>,
    
    /// Intervention history
    history: Vec<InterventionRecord>,
    
    /// Learning rate
    learning_rate: f32,
    
    /// Exploration rate (for trying new interventions)
    exploration_rate: f32,
    
    /// Minimum applications before trusting success rate
    min_applications: u64,
    
    /// Evolution statistics
    stats: EvolutionStats,
}

/// Record of intervention application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionRecord {
    /// Intervention ID
    pub intervention_id: Uuid,
    
    /// Coherence before
    pub coherence_before: f32,
    
    /// Coherence after
    pub coherence_after: f32,
    
    /// Improvement
    pub improvement: f32,
    
    /// Success (improvement > 0)
    pub success: bool,
    
    /// Applied at
    pub applied_at: DateTime<Utc>,
}

/// Evolution statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvolutionStats {
    /// Total interventions created
    pub interventions_created: u64,
    
    /// Total interventions applied
    pub interventions_applied: u64,
    
    /// Total successes
    pub total_successes: u64,
    
    /// Total coherence improvement
    pub total_improvement: f32,
    
    /// Best intervention ID
    pub best_intervention_id: Option<Uuid>,
    
    /// Best improvement
    pub best_improvement: f32,
}

impl MetaLearnerEvolution {
    /// Create new meta-learner evolution
    pub fn new() -> Self {
        let mut interventions = HashMap::new();
        
        // Start with default sacred interventions
        let default_interventions = Self::create_default_interventions();
        for intervention in default_interventions {
            interventions.insert(intervention.id, intervention);
        }
        
        Self {
            interventions,
            history: Vec::new(),
            learning_rate: 0.1,
            exploration_rate: 0.2,
            min_applications: 5,
            stats: EvolutionStats::default(),
        }
    }
    
    /// Create default sacred interventions
    fn create_default_interventions() -> Vec<SacredIntervention> {
        let now = Utc::now();
        
        vec![
            // Position 3: Creative Trinity
            SacredIntervention {
                id: Uuid::new_v4(),
                name: "Position 3 Boost".to_string(),
                position: 3,
                boost_factor: 1.5,
                orbital_radius: 1.0,
                timing: InterventionTiming::Always,
                success_rate: 0.0,
                applications: 0,
                successes: 0,
                avg_coherence_improvement: 0.0,
                created_at: now,
                updated_at: now,
            },
            
            // Position 6: Heart Center
            SacredIntervention {
                id: Uuid::new_v4(),
                name: "Position 6 Boost".to_string(),
                position: 6,
                boost_factor: 1.5,
                orbital_radius: 1.2,
                timing: InterventionTiming::Always,
                success_rate: 0.0,
                applications: 0,
                successes: 0,
                avg_coherence_improvement: 0.0,
                created_at: now,
                updated_at: now,
            },
            
            // Position 9: Divine Completion
            SacredIntervention {
                id: Uuid::new_v4(),
                name: "Position 9 Boost".to_string(),
                position: 9,
                boost_factor: 1.5,
                orbital_radius: 1.5,
                timing: InterventionTiming::Always,
                success_rate: 0.0,
                applications: 0,
                successes: 0,
                avg_coherence_improvement: 0.0,
                created_at: now,
                updated_at: now,
            },
            
            // Adaptive degradation recovery
            SacredIntervention {
                id: Uuid::new_v4(),
                name: "Adaptive Degradation Recovery".to_string(),
                position: 9, // Start at position 9
                boost_factor: 2.0,
                orbital_radius: 2.0,
                timing: InterventionTiming::OnDegradation,
                success_rate: 0.0,
                applications: 0,
                successes: 0,
                avg_coherence_improvement: 0.0,
                created_at: now,
                updated_at: now,
            },
        ]
    }
    
    /// Select intervention to apply
    pub fn select_intervention(
        &self,
        current_position: u8,
        coherence: &CoherenceMetrics,
        cycle_count: u64,
    ) -> Option<SacredIntervention> {
        // Exploration: Try random intervention
        if rand::random::<f32>() < self.exploration_rate {
            return self.select_random_intervention(current_position);
        }
        
        // Exploitation: Select best intervention for current context
        self.select_best_intervention(current_position, coherence, cycle_count)
    }
    
    /// Select random intervention (exploration)
    fn select_random_intervention(&self, position: u8) -> Option<SacredIntervention> {
        let candidates: Vec<_> = self.interventions.values()
            .filter(|i| i.position == position)
            .collect();
        
        if candidates.is_empty() {
            return None;
        }
        
        let idx = (rand::random::<f32>() * candidates.len() as f32) as usize;
        Some(candidates[idx].clone())
    }
    
    /// Select best intervention (exploitation)
    fn select_best_intervention(
        &self,
        position: u8,
        coherence: &CoherenceMetrics,
        cycle_count: u64,
    ) -> Option<SacredIntervention> {
        let mut candidates: Vec<_> = self.interventions.values()
            .filter(|i| {
                // Must match position
                if i.position != position {
                    return false;
                }
                
                // Check timing
                match i.timing {
                    InterventionTiming::Always => true,
                    InterventionTiming::OnDegradation => coherence.is_degrading,
                    InterventionTiming::BelowThreshold(t) => {
                        coherence.overall_coherence < (t as f32 / 100.0)
                    }
                    InterventionTiming::AtCycle(c) => cycle_count % c == 0,
                    InterventionTiming::Adaptive => true,
                }
            })
            .collect();
        
        if candidates.is_empty() {
            return None;
        }
        
        // Sort by success rate (with minimum applications filter)
        candidates.sort_by(|a, b| {
            let a_score = if a.applications >= self.min_applications {
                a.success_rate
            } else {
                0.5 // Neutral score for untested
            };
            
            let b_score = if b.applications >= self.min_applications {
                b.success_rate
            } else {
                0.5
            };
            
            b_score.partial_cmp(&a_score).unwrap()
        });
        
        Some(candidates[0].clone())
    }
    
    /// Record intervention result and learn
    pub fn record_result(
        &mut self,
        intervention_id: Uuid,
        coherence_before: f32,
        coherence_after: f32,
    ) -> Result<()> {
        let improvement = coherence_after - coherence_before;
        let success = improvement > 0.0;
        
        // Record in history
        let record = InterventionRecord {
            intervention_id,
            coherence_before,
            coherence_after,
            improvement,
            success,
            applied_at: Utc::now(),
        };
        self.history.push(record);
        
        // Update intervention statistics
        if let Some(intervention) = self.interventions.get_mut(&intervention_id) {
            intervention.applications += 1;
            if success {
                intervention.successes += 1;
            }
            
            // Update success rate
            intervention.success_rate = intervention.successes as f32 / intervention.applications as f32;
            
            // Update average improvement (exponential moving average)
            intervention.avg_coherence_improvement = 
                intervention.avg_coherence_improvement * (1.0 - self.learning_rate) +
                improvement * self.learning_rate;
            
            intervention.updated_at = Utc::now();
            
            // Learn: Adjust boost factor based on success
            if success {
                // Increase boost if successful
                intervention.boost_factor *= 1.0 + self.learning_rate;
                intervention.boost_factor = intervention.boost_factor.min(3.0);
            } else {
                // Decrease boost if failed
                intervention.boost_factor *= 1.0 - self.learning_rate * 0.5;
                intervention.boost_factor = intervention.boost_factor.max(1.0);
            }
            
            // Learn: Adjust orbital radius based on improvement magnitude
            if improvement.abs() > 0.1 {
                // Large change - adjust radius
                if improvement > 0.0 {
                    intervention.orbital_radius *= 1.0 + self.learning_rate * 0.5;
                } else {
                    intervention.orbital_radius *= 1.0 - self.learning_rate * 0.5;
                }
                intervention.orbital_radius = intervention.orbital_radius.clamp(0.5, 3.0);
            }
        }
        
        // Update global stats
        self.stats.interventions_applied += 1;
        if success {
            self.stats.total_successes += 1;
        }
        self.stats.total_improvement += improvement;
        
        if improvement > self.stats.best_improvement {
            self.stats.best_improvement = improvement;
            self.stats.best_intervention_id = Some(intervention_id);
        }
        
        Ok(())
    }
    
    /// Propose new intervention based on learning
    pub fn propose_new_intervention(
        &mut self,
        position: u8,
        coherence: &CoherenceMetrics,
    ) -> Result<SacredIntervention> {
        // Analyze successful interventions at this position
        let successful_interventions: Vec<_> = self.interventions.values()
            .filter(|i| i.position == position && i.success_rate > 0.6)
            .collect();
        
        let (boost_factor, orbital_radius) = if successful_interventions.is_empty() {
            // No successful interventions - use defaults with variation
            let variation = (rand::random::<f32>() - 0.5) * 0.5;
            (1.5 + variation, 1.0 + variation)
        } else {
            // Average successful interventions
            let avg_boost = successful_interventions.iter()
                .map(|i| i.boost_factor)
                .sum::<f32>() / successful_interventions.len() as f32;
            
            let avg_radius = successful_interventions.iter()
                .map(|i| i.orbital_radius)
                .sum::<f32>() / successful_interventions.len() as f32;
            
            // Add small variation for exploration
            let variation = (rand::random::<f32>() - 0.5) * 0.3;
            (avg_boost + variation, avg_radius + variation)
        };
        
        // Determine timing based on coherence state
        let timing = if coherence.is_degrading {
            InterventionTiming::OnDegradation
        } else if coherence.overall_coherence < 0.8 {
            InterventionTiming::BelowThreshold(80)
        } else {
            InterventionTiming::Adaptive
        };
        
        let now = Utc::now();
        let intervention = SacredIntervention {
            id: Uuid::new_v4(),
            name: format!("Learned Position {} Intervention", position),
            position,
            boost_factor: boost_factor.clamp(1.0, 3.0),
            orbital_radius: orbital_radius.clamp(0.5, 3.0),
            timing,
            success_rate: 0.0,
            applications: 0,
            successes: 0,
            avg_coherence_improvement: 0.0,
            created_at: now,
            updated_at: now,
        };
        
        self.stats.interventions_created += 1;
        
        Ok(intervention)
    }
    
    /// Add intervention to repertoire
    pub fn add_intervention(&mut self, intervention: SacredIntervention) {
        self.interventions.insert(intervention.id, intervention);
    }
    
    /// Get best interventions
    pub fn get_best_interventions(&self, n: usize) -> Vec<SacredIntervention> {
        let mut interventions: Vec<_> = self.interventions.values()
            .filter(|i| i.applications >= self.min_applications)
            .cloned()
            .collect();
        
        interventions.sort_by(|a, b| {
            b.success_rate.partial_cmp(&a.success_rate).unwrap()
        });
        
        interventions.into_iter().take(n).collect()
    }
    
    /// Get evolution statistics
    pub fn get_stats(&self) -> &EvolutionStats {
        &self.stats
    }
    
    /// Get intervention by ID
    pub fn get_intervention(&self, id: &Uuid) -> Option<&SacredIntervention> {
        self.interventions.get(id)
    }
    
    /// Get all interventions
    pub fn get_all_interventions(&self) -> Vec<SacredIntervention> {
        self.interventions.values().cloned().collect()
    }
    
    /// Get intervention history
    pub fn get_history(&self) -> &[InterventionRecord] {
        &self.history
    }
    
    /// Prune poorly performing interventions
    pub fn prune_interventions(&mut self, min_success_rate: f32) {
        let to_remove: Vec<_> = self.interventions.iter()
            .filter(|(_, i)| {
                i.applications >= self.min_applications && i.success_rate < min_success_rate
            })
            .map(|(id, _)| *id)
            .collect();
        
        for id in to_remove {
            self.interventions.remove(&id);
        }
    }
    
    /// Set learning rate
    pub fn set_learning_rate(&mut self, rate: f32) {
        self.learning_rate = rate.clamp(0.01, 0.5);
    }
    
    /// Set exploration rate
    pub fn set_exploration_rate(&mut self, rate: f32) {
        self.exploration_rate = rate.clamp(0.0, 1.0);
    }
}

impl Default for MetaLearnerEvolution {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_default_interventions() {
        let learner = MetaLearnerEvolution::new();
        assert_eq!(learner.interventions.len(), 4);
    }
    
    #[test]
    fn test_record_success() {
        let mut learner = MetaLearnerEvolution::new();
        let intervention_id = learner.interventions.values().next().unwrap().id;
        
        // Record successful intervention
        learner.record_result(intervention_id, 0.6, 0.8).unwrap();
        
        let intervention = learner.get_intervention(&intervention_id).unwrap();
        assert_eq!(intervention.applications, 1);
        assert_eq!(intervention.successes, 1);
        assert_eq!(intervention.success_rate, 1.0);
        assert!(intervention.boost_factor > 1.5); // Should increase
    }
    
    #[test]
    fn test_record_failure() {
        let mut learner = MetaLearnerEvolution::new();
        let intervention_id = learner.interventions.values().next().unwrap().id;
        
        // Record failed intervention
        learner.record_result(intervention_id, 0.8, 0.6).unwrap();
        
        let intervention = learner.get_intervention(&intervention_id).unwrap();
        assert_eq!(intervention.applications, 1);
        assert_eq!(intervention.successes, 0);
        assert_eq!(intervention.success_rate, 0.0);
        assert!(intervention.boost_factor < 1.5); // Should decrease
    }
    
    #[test]
    fn test_propose_new_intervention() {
        let mut learner = MetaLearnerEvolution::new();
        
        let coherence = CoherenceMetrics {
            overall_coherence: 0.6,
            sacred_frequency: 0.3,
            digital_root_coherence: 0.7,
            vortex_cycle_coherence: 0.8,
            is_degrading: true,
            degradation_severity: 0.4,
        };
        
        let new_intervention = learner.propose_new_intervention(3, &coherence).unwrap();
        assert_eq!(new_intervention.position, 3);
        assert!(new_intervention.boost_factor >= 1.0 && new_intervention.boost_factor <= 3.0);
    }
}
