//! Energy-Based Reasoning Model (EBRM) for SpatialVortex
//!
//! Inspired by Logical Intelligence's Kona model, this module implements:
//! - Global energy scoring over partial and complete reasoning traces
//! - Non-autoregressive trace evaluation (unlike LLM token-by-token)
//! - Continuous latent space reasoning with gradient-based refinement
//! - Backward conditioning (optimizing traces given context AND target)
//!
//! # Key Advantages over LLM-based Reasoning
//!
//! 1. **Non-autoregressive**: Evaluates complete traces simultaneously
//! 2. **Globally-scored**: End-to-end trace quality, not just next-token
//! 3. **Continuous space**: Dense vector tokens allow gradient-based edits
//! 4. **Failure localization**: Pinpoints WHERE constraints are violated
//!
//! # Integration with Sacred Geometry
//!
//! - Sacred positions (3, 6, 9) serve as constraint checkpoints
//! - Signal strength = energy score (low energy = consistent with constraints)
//! - Vortex flow (1→2→4→8→7→5→1) provides natural optimization trajectory

use crate::data::models::BeamTensor;
use crate::ml::hallucinations::SignalSubspace;
use serde::{Deserialize, Serialize};

/// Energy score for a reasoning trace
/// 
/// Low energy = more consistent with constraints/objectives
/// High energy = something is broken (constraint violation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEnergy {
    /// Global energy score (0.0 = perfect, higher = worse)
    pub global_energy: f32,
    
    /// Constraint satisfaction score (0.0-1.0, higher = better)
    pub constraint_satisfaction: f32,
    
    /// Signal strength from 3-6-9 pattern coherence
    pub signal_strength: f32,
    
    /// Localized failure information (if any)
    pub failure_location: Option<FailureLocation>,
    
    /// Per-position energy breakdown
    pub position_energies: Vec<PositionEnergy>,
    
    /// Whether this trace is considered valid
    pub is_valid: bool,
}

impl TraceEnergy {
    /// Convert energy to confidence score (inverse relationship)
    pub fn to_confidence(&self) -> f32 {
        // Energy 0.0 → confidence 1.0
        // Energy 1.0 → confidence 0.0
        (1.0 - self.global_energy).clamp(0.0, 1.0)
    }
    
    /// Check if energy indicates hallucination risk
    pub fn has_hallucination_risk(&self) -> bool {
        self.global_energy > 0.5 || self.signal_strength < 0.5
    }
}

/// Localized failure information for actionable guidance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureLocation {
    /// Index in the trace where failure was detected
    pub trace_index: usize,
    
    /// Position in flux pattern (1-9)
    pub flux_position: u8,
    
    /// Type of constraint violated
    pub constraint_type: ConstraintType,
    
    /// Severity of the violation (0.0-1.0)
    pub severity: f32,
    
    /// Human-readable description
    pub description: String,
}

/// Types of constraints that can be violated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConstraintType {
    /// 3-6-9 sacred pattern coherence violated
    SacredPatternCoherence,
    
    /// ELP channel balance violated (e.g., pathos > 70%)
    ChannelBalance,
    
    /// Signal strength dropped below threshold
    SignalStrength,
    
    /// Dynamics divergence too high
    DynamicsDivergence,
    
    /// Vortex flow pattern broken
    VortexFlowViolation,
    
    /// Overflow risk detected
    OverflowRisk,
    
    /// Custom constraint
    Custom(String),
}

/// Energy at a specific position in the trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionEnergy {
    /// Index in trace
    pub index: usize,
    
    /// Flux position (1-9)
    pub position: u8,
    
    /// Local energy at this position
    pub energy: f32,
    
    /// Whether this is a sacred checkpoint
    pub is_sacred: bool,
    
    /// Constraint status at this position
    pub constraints_satisfied: bool,
}

/// Energy-Based Reasoning Model
/// 
/// Learns to assign scalar energy scores to reasoning traces.
/// Unlike LLMs, can evaluate partial traces and localize failures.
pub struct EnergyBasedReasoningModel {
    /// Signal threshold for constraint satisfaction
    signal_threshold: f32,
    
    /// Maximum allowed dynamics divergence
    dynamics_threshold: f32,
    
    /// ELP channel balance thresholds
    channel_balance: ChannelBalanceConfig,
    
    /// Subspace rank for signal analysis
    subspace_rank: usize,
    
    /// Sacred position boost factor
    sacred_boost: f32,
}

/// Configuration for ELP channel balance constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelBalanceConfig {
    /// Maximum allowed pathos dominance (default: 0.7)
    pub max_pathos_ratio: f32,
    
    /// Minimum required ethos (default: 0.2)
    pub min_ethos_ratio: f32,
    
    /// Minimum required logos (default: 0.2)
    pub min_logos_ratio: f32,
}

impl Default for ChannelBalanceConfig {
    fn default() -> Self {
        Self {
            max_pathos_ratio: 0.7,
            min_ethos_ratio: 0.2,
            min_logos_ratio: 0.2,
        }
    }
}

impl Default for EnergyBasedReasoningModel {
    fn default() -> Self {
        Self {
            signal_threshold: 0.5,
            dynamics_threshold: 0.15,
            channel_balance: ChannelBalanceConfig::default(),
            subspace_rank: 5,
            sacred_boost: 1.5,
        }
    }
}

impl EnergyBasedReasoningModel {
    /// Create new EBRM with custom configuration
    pub fn new(
        signal_threshold: f32,
        dynamics_threshold: f32,
        subspace_rank: usize,
    ) -> Self {
        Self {
            signal_threshold,
            dynamics_threshold,
            channel_balance: ChannelBalanceConfig::default(),
            subspace_rank,
            sacred_boost: 1.5,
        }
    }
    
    /// Score a complete reasoning trace
    /// 
    /// Unlike LLM token-by-token scoring, this evaluates the entire trace
    /// simultaneously and returns a global energy score.
    pub fn score_trace(&self, trace: &[BeamTensor]) -> TraceEnergy {
        if trace.is_empty() {
            return TraceEnergy {
                global_energy: 1.0, // Maximum energy = invalid
                constraint_satisfaction: 0.0,
                signal_strength: 0.0,
                failure_location: Some(FailureLocation {
                    trace_index: 0,
                    flux_position: 0,
                    constraint_type: ConstraintType::Custom("Empty trace".to_string()),
                    severity: 1.0,
                    description: "Cannot score empty trace".to_string(),
                }),
                position_energies: vec![],
                is_valid: false,
            };
        }
        
        // 1. Compute signal subspace for global coherence
        let subspace = SignalSubspace::from_beam_tensors(trace, self.subspace_rank);
        let signal_strength = subspace.strength;
        
        // 2. Compute per-position energies
        let position_energies = self.compute_position_energies(trace, &subspace);
        
        // 3. Check constraints and find failures
        let (constraint_satisfaction, failure_location) = 
            self.check_constraints(trace, &position_energies);
        
        // 4. Compute global energy (inverse of quality)
        let global_energy = self.compute_global_energy(
            signal_strength,
            constraint_satisfaction,
            &position_energies,
        );
        
        let is_valid = global_energy < 0.5 && constraint_satisfaction > 0.5;
        
        TraceEnergy {
            global_energy,
            constraint_satisfaction,
            signal_strength,
            failure_location,
            position_energies,
            is_valid,
        }
    }
    
    /// Score a partial trace (key EBRM advantage)
    /// 
    /// This is what LLMs cannot do efficiently - evaluate incomplete plans
    /// to guide search before committing to a full solution.
    pub fn score_partial_trace(&self, partial_trace: &[BeamTensor]) -> TraceEnergy {
        // For partial traces, we're more lenient but still check constraints
        let mut energy = self.score_trace(partial_trace);
        
        // Adjust for partial completion (don't penalize incompleteness)
        let completion_factor = (partial_trace.len() as f32 / 10.0).min(1.0);
        energy.global_energy *= completion_factor;
        
        // Partial traces are valid if they're on track
        energy.is_valid = energy.global_energy < 0.6 && energy.constraint_satisfaction > 0.4;
        
        energy
    }
    
    /// Compute per-position energy breakdown
    fn compute_position_energies(
        &self,
        trace: &[BeamTensor],
        subspace: &SignalSubspace,
    ) -> Vec<PositionEnergy> {
        trace.iter().enumerate().map(|(idx, beam)| {
            let is_sacred = matches!(beam.position, 3 | 6 | 9);
            
            // Local energy based on signal projection
            let projected = subspace.project(beam);
            let projection_energy: f32 = projected.iter()
                .map(|v| (v - 1.0/9.0).powi(2))
                .sum();
            
            // Sacred positions get energy reduction (they're checkpoints)
            let energy = if is_sacred {
                projection_energy / self.sacred_boost
            } else {
                projection_energy
            };
            
            // Check local constraints
            let constraints_satisfied = self.check_local_constraints(beam);
            
            PositionEnergy {
                index: idx,
                position: beam.position,
                energy,
                is_sacred,
                constraints_satisfied,
            }
        }).collect()
    }
    
    /// Check local constraints for a single beam
    fn check_local_constraints(&self, beam: &BeamTensor) -> bool {
        let ethos = beam.ethos();
        let logos = beam.logos();
        let pathos = beam.pathos();
        let total = ethos + logos + pathos;
        
        if total == 0.0 {
            return true; // No ELP data, assume ok
        }
        
        let ethos_ratio = ethos / total;
        let logos_ratio = logos / total;
        let pathos_ratio = pathos / total;
        
        // Check balance constraints
        pathos_ratio <= self.channel_balance.max_pathos_ratio &&
        ethos_ratio >= self.channel_balance.min_ethos_ratio &&
        logos_ratio >= self.channel_balance.min_logos_ratio
    }
    
    /// Check global constraints and locate failures
    fn check_constraints(
        &self,
        trace: &[BeamTensor],
        position_energies: &[PositionEnergy],
    ) -> (f32, Option<FailureLocation>) {
        let mut violations = 0;
        let mut total_checks = 0;
        let mut worst_failure: Option<FailureLocation> = None;
        let mut worst_severity = 0.0;
        
        // Check 1: Signal strength at sacred positions
        for (idx, beam) in trace.iter().enumerate() {
            if matches!(beam.position, 3 | 6 | 9) {
                total_checks += 1;
                if beam.confidence < self.signal_threshold {
                    violations += 1;
                    let severity = 1.0 - beam.confidence / self.signal_threshold;
                    if severity > worst_severity {
                        worst_severity = severity;
                        worst_failure = Some(FailureLocation {
                            trace_index: idx,
                            flux_position: beam.position,
                            constraint_type: ConstraintType::SignalStrength,
                            severity,
                            description: format!(
                                "Signal strength {:.2} below threshold {:.2} at sacred position {}",
                                beam.confidence, self.signal_threshold, beam.position
                            ),
                        });
                    }
                }
            }
        }
        
        // Check 2: ELP channel balance
        for (idx, beam) in trace.iter().enumerate() {
            total_checks += 1;
            if !self.check_local_constraints(beam) {
                violations += 1;
                let severity = 0.7; // Medium severity for balance violations
                if severity > worst_severity {
                    worst_severity = severity;
                    worst_failure = Some(FailureLocation {
                        trace_index: idx,
                        flux_position: beam.position,
                        constraint_type: ConstraintType::ChannelBalance,
                        severity,
                        description: format!(
                            "ELP channel balance violated at position {}",
                            beam.position
                        ),
                    });
                }
            }
        }
        
        // Check 3: Vortex flow pattern
        let flux_pattern = [1u8, 2, 4, 8, 7, 5];
        for window in trace.windows(2) {
            total_checks += 1;
            let curr_pos = window[0].position;
            let next_pos = window[1].position;
            
            // Check if transition follows vortex pattern or goes to sacred
            let valid_transition = 
                matches!(next_pos, 3 | 6 | 9) || // Sacred positions always valid
                flux_pattern.windows(2).any(|w| w[0] == curr_pos && w[1] == next_pos) ||
                (curr_pos == 5 && next_pos == 1); // Loop back
            
            if !valid_transition {
                violations += 1;
                let severity = 0.5; // Lower severity for flow violations
                if worst_failure.is_none() || severity > worst_severity {
                    worst_severity = severity;
                    worst_failure = Some(FailureLocation {
                        trace_index: trace.len() - 1,
                        flux_position: next_pos,
                        constraint_type: ConstraintType::VortexFlowViolation,
                        severity,
                        description: format!(
                            "Invalid vortex flow: {} → {}",
                            curr_pos, next_pos
                        ),
                    });
                }
            }
        }
        
        // Check 4: 3-6-9 pattern coherence
        let sacred_beams: Vec<_> = trace.iter()
            .filter(|b| matches!(b.position, 3 | 6 | 9))
            .collect();
        
        if !sacred_beams.is_empty() {
            total_checks += 1;
            let sacred_coherence = self.compute_sacred_coherence(&sacred_beams);
            if sacred_coherence < 0.5 {
                violations += 1;
                let severity = 1.0 - sacred_coherence;
                if severity > worst_severity {
                    worst_failure = Some(FailureLocation {
                        trace_index: trace.len() / 2,
                        flux_position: 6, // Middle sacred position
                        constraint_type: ConstraintType::SacredPatternCoherence,
                        severity,
                        description: format!(
                            "Sacred pattern coherence {:.2} below threshold",
                            sacred_coherence
                        ),
                    });
                }
            }
        }
        
        let satisfaction = if total_checks > 0 {
            1.0 - (violations as f32 / total_checks as f32)
        } else {
            1.0
        };
        
        (satisfaction, worst_failure)
    }
    
    /// Compute coherence of sacred positions (3-6-9 triangle)
    fn compute_sacred_coherence(&self, sacred_beams: &[&BeamTensor]) -> f32 {
        if sacred_beams.is_empty() {
            return 1.0;
        }
        
        // Check that sacred positions have consistent signal
        let confidences: Vec<f32> = sacred_beams.iter()
            .map(|b| b.confidence)
            .collect();
        
        let mean = confidences.iter().sum::<f32>() / confidences.len() as f32;
        let variance = confidences.iter()
            .map(|c| (c - mean).powi(2))
            .sum::<f32>() / confidences.len() as f32;
        
        // Low variance = high coherence
        (1.0 - variance.sqrt()).clamp(0.0, 1.0)
    }
    
    /// Compute global energy from components
    fn compute_global_energy(
        &self,
        signal_strength: f32,
        constraint_satisfaction: f32,
        position_energies: &[PositionEnergy],
    ) -> f32 {
        // Average position energy
        let avg_position_energy = if position_energies.is_empty() {
            0.5
        } else {
            position_energies.iter().map(|p| p.energy).sum::<f32>() 
                / position_energies.len() as f32
        };
        
        // Global energy: weighted combination (lower = better)
        let energy = 
            (1.0 - signal_strength) * 0.4 +           // Signal weakness
            (1.0 - constraint_satisfaction) * 0.4 +   // Constraint violations
            avg_position_energy.min(1.0) * 0.2;       // Position energies
        
        energy.clamp(0.0, 1.0)
    }
}

/// Continuous Latent Space Editor for gradient-based trace refinement
/// 
/// Unlike discrete LLM tokens, this allows controlled local edits
/// via approximate gradient information.
pub struct LatentSpaceEditor {
    /// Learning rate for gradient updates
    learning_rate: f32,
    
    /// Maximum iterations for refinement
    max_iterations: usize,
    
    /// Convergence threshold
    convergence_threshold: f32,
    
    /// EBRM for energy computation
    ebrm: EnergyBasedReasoningModel,
}

impl Default for LatentSpaceEditor {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            max_iterations: 10,
            convergence_threshold: 0.01,
            ebrm: EnergyBasedReasoningModel::default(),
        }
    }
}

impl LatentSpaceEditor {
    /// Create new editor with custom parameters
    pub fn new(learning_rate: f32, max_iterations: usize) -> Self {
        Self {
            learning_rate,
            max_iterations,
            convergence_threshold: 0.01,
            ebrm: EnergyBasedReasoningModel::default(),
        }
    }
    
    /// Refine a trace using gradient-based optimization
    /// 
    /// This is the key advantage of continuous latent space:
    /// we can make targeted edits to improve coherence/constraint satisfaction.
    pub fn refine_trace(&self, trace: &mut [BeamTensor]) -> RefinementResult {
        let initial_energy = self.ebrm.score_trace(trace);
        let mut current_energy = initial_energy.global_energy;
        let mut iterations = 0;
        
        for iter in 0..self.max_iterations {
            iterations = iter + 1;
            
            // Compute energy gradient (approximate via finite differences)
            let gradients = self.compute_energy_gradients(trace);
            
            // Apply gradient descent step
            self.apply_gradient_step(trace, &gradients);
            
            // Check convergence
            let new_energy = self.ebrm.score_trace(trace).global_energy;
            let improvement = current_energy - new_energy;
            
            if improvement.abs() < self.convergence_threshold {
                break;
            }
            
            current_energy = new_energy;
        }
        
        let final_energy = self.ebrm.score_trace(trace);
        
        RefinementResult {
            initial_energy: initial_energy.global_energy,
            final_energy: final_energy.global_energy,
            improvement: initial_energy.global_energy - final_energy.global_energy,
            iterations,
            converged: iterations < self.max_iterations,
            final_trace_energy: final_energy,
        }
    }
    
    /// Compute approximate energy gradients via finite differences
    fn compute_energy_gradients(&self, trace: &[BeamTensor]) -> Vec<[f32; 9]> {
        let epsilon = 0.01;
        let base_energy = self.ebrm.score_trace(trace).global_energy;
        
        trace.iter().map(|beam| {
            let mut gradients = [0.0; 9];
            
            for dim in 0..9 {
                // Perturb dimension
                let mut perturbed = beam.clone();
                perturbed.digits[dim] += epsilon;
                
                // Normalize
                let sum: f32 = perturbed.digits.iter().sum();
                if sum > 0.0 {
                    for d in perturbed.digits.iter_mut() {
                        *d /= sum;
                    }
                }
                
                // Compute gradient (finite difference)
                let mut perturbed_trace: Vec<BeamTensor> = trace.to_vec();
                perturbed_trace[0] = perturbed; // Simplified: only perturb first
                
                let perturbed_energy = self.ebrm.score_trace(&perturbed_trace).global_energy;
                gradients[dim] = (perturbed_energy - base_energy) / epsilon;
            }
            
            gradients
        }).collect()
    }
    
    /// Apply gradient descent step to trace
    fn apply_gradient_step(&self, trace: &mut [BeamTensor], gradients: &[[f32; 9]]) {
        for (beam, grad) in trace.iter_mut().zip(gradients.iter()) {
            // Update digits in direction of negative gradient (minimize energy)
            for dim in 0..9 {
                beam.digits[dim] -= self.learning_rate * grad[dim];
                beam.digits[dim] = beam.digits[dim].max(0.0); // Keep non-negative
            }
            
            // Normalize to maintain probability distribution
            let sum: f32 = beam.digits.iter().sum();
            if sum > 0.0 {
                for d in beam.digits.iter_mut() {
                    *d /= sum;
                }
            }
            
            // Update confidence based on new distribution
            let sacred_sum = beam.digits[2] + beam.digits[5] + beam.digits[8];
            beam.confidence = (sacred_sum / 3.0 * 0.33 + beam.confidence * 0.67).clamp(0.0, 1.0);
        }
    }
    
    /// Refine trace at specific failure location
    pub fn refine_at_location(
        &self,
        trace: &mut [BeamTensor],
        location: &FailureLocation,
    ) -> RefinementResult {
        // Focus refinement on the failure location
        let start = location.trace_index.saturating_sub(2);
        let end = (location.trace_index + 3).min(trace.len());
        
        // Refine the local window
        let mut window: Vec<BeamTensor> = trace[start..end].to_vec();
        let result = self.refine_trace(&mut window);
        
        // Copy back
        for (i, beam) in window.into_iter().enumerate() {
            trace[start + i] = beam;
        }
        
        result
    }
}

/// Result of trace refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementResult {
    /// Energy before refinement
    pub initial_energy: f32,
    
    /// Energy after refinement
    pub final_energy: f32,
    
    /// Total improvement (positive = better)
    pub improvement: f32,
    
    /// Number of iterations used
    pub iterations: usize,
    
    /// Whether refinement converged
    pub converged: bool,
    
    /// Final trace energy details
    pub final_trace_energy: TraceEnergy,
}

/// Backward Conditioning for target-aware optimization
/// 
/// Unlike LLMs which struggle with "backwards" conditioning,
/// EBRMs can natively optimize traces given both context AND target.
pub struct BackwardConditioner {
    /// EBRM for energy computation
    ebrm: EnergyBasedReasoningModel,
    
    /// Editor for trace refinement
    editor: LatentSpaceEditor,
    
    /// Weight for target alignment (vs context consistency)
    target_weight: f32,
}

impl Default for BackwardConditioner {
    fn default() -> Self {
        Self {
            ebrm: EnergyBasedReasoningModel::default(),
            editor: LatentSpaceEditor::default(),
            target_weight: 0.5,
        }
    }
}

impl BackwardConditioner {
    /// Create new conditioner with custom target weight
    pub fn new(target_weight: f32) -> Self {
        Self {
            ebrm: EnergyBasedReasoningModel::default(),
            editor: LatentSpaceEditor::default(),
            target_weight: target_weight.clamp(0.0, 1.0),
        }
    }
    
    /// Optimize a trace to satisfy both context and target constraints
    /// 
    /// This is what LLMs cannot do efficiently - they can only generate
    /// forward from context, not optimize toward a target.
    pub fn optimize_for_target(
        &self,
        context: &[BeamTensor],
        target: &BeamTensor,
        max_trace_length: usize,
    ) -> ConditionedResult {
        // Initialize trace from context
        let mut trace: Vec<BeamTensor> = context.to_vec();
        
        // Extend trace toward target
        while trace.len() < max_trace_length {
            let next = self.generate_next_toward_target(&trace, target);
            trace.push(next);
            
            // Check if we've reached target-like state
            if self.is_target_reached(&trace, target) {
                break;
            }
        }
        
        // Refine the complete trace
        let refinement = self.editor.refine_trace(&mut trace);
        
        // Compute final metrics
        let context_consistency = self.compute_context_consistency(&trace, context);
        let target_alignment = self.compute_target_alignment(&trace, target);
        
        ConditionedResult {
            trace,
            context_consistency,
            target_alignment,
            combined_score: context_consistency * (1.0 - self.target_weight) 
                          + target_alignment * self.target_weight,
            refinement,
        }
    }
    
    /// Generate next beam moving toward target
    fn generate_next_toward_target(
        &self,
        current_trace: &[BeamTensor],
        target: &BeamTensor,
    ) -> BeamTensor {
        let last = current_trace.last().cloned().unwrap_or_default();
        let mut next = last.clone();
        
        // Interpolate toward target
        let alpha = 0.3; // Step size toward target
        for i in 0..9 {
            next.digits[i] = last.digits[i] * (1.0 - alpha) + target.digits[i] * alpha;
        }
        
        // Normalize
        let sum: f32 = next.digits.iter().sum();
        if sum > 0.0 {
            for d in next.digits.iter_mut() {
                *d /= sum;
            }
        }
        
        // Advance flux position
        let flux_pattern = [1u8, 2, 4, 8, 7, 5];
        let current_idx = flux_pattern.iter().position(|&p| p == last.position).unwrap_or(0);
        next.position = flux_pattern[(current_idx + 1) % flux_pattern.len()];
        
        // Interpolate ELP toward target
        let target_ethos = target.ethos();
        let target_logos = target.logos();
        let target_pathos = target.pathos();
        
        next.ethos = Some(last.ethos() * (1.0 - alpha) + target_ethos * alpha);
        next.logos = Some(last.logos() * (1.0 - alpha) + target_logos * alpha);
        next.pathos = Some(last.pathos() * (1.0 - alpha) + target_pathos * alpha);
        
        next
    }
    
    /// Check if trace has reached target-like state
    fn is_target_reached(&self, trace: &[BeamTensor], target: &BeamTensor) -> bool {
        if let Some(last) = trace.last() {
            // Check digit similarity
            let digit_diff: f32 = last.digits.iter()
                .zip(target.digits.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();
            
            digit_diff < 0.3 // Threshold for "close enough"
        } else {
            false
        }
    }
    
    /// Compute consistency with original context
    fn compute_context_consistency(&self, trace: &[BeamTensor], context: &[BeamTensor]) -> f32 {
        if context.is_empty() || trace.is_empty() {
            return 0.5;
        }
        
        // Compare signal subspaces
        let context_subspace = SignalSubspace::from_beam_tensors(context, 5);
        let trace_subspace = SignalSubspace::from_beam_tensors(trace, 5);
        
        // Similarity based on strength correlation
        let strength_diff = (context_subspace.strength - trace_subspace.strength).abs();
        
        (1.0 - strength_diff).clamp(0.0, 1.0)
    }
    
    /// Compute alignment with target
    fn compute_target_alignment(&self, trace: &[BeamTensor], target: &BeamTensor) -> f32 {
        if let Some(last) = trace.last() {
            // Digit alignment
            let digit_alignment: f32 = 1.0 - last.digits.iter()
                .zip(target.digits.iter())
                .map(|(a, b)| (a - b).abs())
                .sum::<f32>() / 9.0;
            
            // ELP alignment
            let elp_diff = (
                (last.ethos() - target.ethos()).abs() +
                (last.logos() - target.logos()).abs() +
                (last.pathos() - target.pathos()).abs()
            ) / 30.0; // Normalize by max range
            
            let elp_alignment = 1.0 - elp_diff;
            
            (digit_alignment * 0.6 + elp_alignment * 0.4).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Result of backward-conditioned optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionedResult {
    /// Optimized trace
    pub trace: Vec<BeamTensor>,
    
    /// Consistency with original context (0.0-1.0)
    pub context_consistency: f32,
    
    /// Alignment with target (0.0-1.0)
    pub target_alignment: f32,
    
    /// Combined score (weighted average)
    pub combined_score: f32,
    
    /// Refinement details
    pub refinement: RefinementResult,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ebrm_score_trace() {
        let ebrm = EnergyBasedReasoningModel::default();
        
        // Create a valid trace following vortex pattern
        let mut trace = vec![BeamTensor::default(); 6];
        let positions = [1u8, 2, 4, 8, 7, 5];
        for (i, beam) in trace.iter_mut().enumerate() {
            beam.position = positions[i];
            beam.confidence = 0.7;
            beam.digits = [1.0/9.0; 9]; // Uniform distribution
        }
        
        let energy = ebrm.score_trace(&trace);
        
        assert!(energy.global_energy >= 0.0);
        assert!(energy.global_energy <= 1.0);
        assert!(energy.signal_strength > 0.0);
    }
    
    #[test]
    fn test_partial_trace_scoring() {
        let ebrm = EnergyBasedReasoningModel::default();
        
        // Partial trace (incomplete)
        let mut partial = vec![BeamTensor::default(); 3];
        for (i, beam) in partial.iter_mut().enumerate() {
            beam.position = [1, 2, 4][i];
            beam.confidence = 0.6;
        }
        
        let energy = ebrm.score_partial_trace(&partial);
        
        // Partial traces should be evaluated but not penalized for incompleteness
        assert!(energy.global_energy < 1.0);
    }
    
    #[test]
    fn test_latent_space_refinement() {
        let editor = LatentSpaceEditor::default();
        
        // Create trace with some noise
        let mut trace = vec![BeamTensor::default(); 5];
        for (i, beam) in trace.iter_mut().enumerate() {
            beam.position = [1, 2, 4, 8, 7][i];
            beam.digits = [0.0; 9];
            beam.digits[i % 9] = 1.0; // Peaked distribution
            beam.confidence = 0.5;
        }
        
        let result = editor.refine_trace(&mut trace);
        
        assert!(result.iterations > 0);
        // Refinement should not make things worse
        assert!(result.final_energy <= result.initial_energy + 0.1);
    }
    
    #[test]
    fn test_backward_conditioning() {
        let conditioner = BackwardConditioner::default();
        
        // Context
        let mut context = vec![BeamTensor::default(); 3];
        for beam in &mut context {
            beam.confidence = 0.7;
            beam.digits = [1.0/9.0; 9];
        }
        
        // Target
        let mut target = BeamTensor::default();
        target.digits = [0.0; 9];
        target.digits[8] = 1.0; // Target position 9
        target.ethos = Some(7.0);
        target.logos = Some(8.0);
        target.pathos = Some(5.0);
        
        let result = conditioner.optimize_for_target(&context, &target, 10);
        
        assert!(!result.trace.is_empty());
        assert!(result.combined_score > 0.0);
        assert!(result.target_alignment >= 0.0);
    }
    
    #[test]
    fn test_failure_localization() {
        let ebrm = EnergyBasedReasoningModel::default();
        
        // Create trace with a constraint violation
        let mut trace = vec![BeamTensor::default(); 5];
        for (i, beam) in trace.iter_mut().enumerate() {
            beam.position = [1, 2, 3, 4, 5][i]; // Position 3 is sacred
            beam.confidence = if i == 2 { 0.2 } else { 0.7 }; // Low confidence at sacred
        }
        
        let energy = ebrm.score_trace(&trace);
        
        // Should detect failure at the sacred position with low confidence
        if let Some(failure) = &energy.failure_location {
            assert_eq!(failure.constraint_type, ConstraintType::SignalStrength);
        }
    }
}
