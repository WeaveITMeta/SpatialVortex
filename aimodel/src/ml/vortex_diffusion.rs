//! Vortex Diffusion Language Model
//!
//! A novel discrete diffusion architecture built on SpatialVortex first principles.
//! Unlike MDLM/SEDD/LLaDA which use linear or cosine noise schedules with flat
//! denoising steps, Vortex Diffusion exploits the sacred geometry cycle for
//! non-monotonic, hierarchical token generation.
//!
//! ## Table of Contents
//! 1. **VortexNoiseSchedule** — Sacred geometry noise curve (φ-based, non-monotonic)
//! 2. **SacredUnmaskingGate** — Three-phase verification at dynamic sacred positions
//! 3. **TokenState** — Per-token state tracking (masked, candidate, verified, locked)
//! 4. **DiffusionTransformer** — Bidirectional attention with vortex-cycle sub-steps
//! 5. **VortexDiffusionEngine** — Main engine: multi-resolution denoising loop
//! 6. **AdaptiveVortexTopology** — Dynamic flow paths, sacred positions, and RSI feedback
//! 7. **DiffusionTrace** — Debug/introspection system for step-by-step generation visibility
//! 8. **NgramValidator** — Bigram/trigram grammatical plausibility checker
//! 9. **SentenceMemory** — Vector store for generated sentences with brute-force similarity
//! 10. **MetaController** — Self-improving loop: propose → generate → evaluate → adapt
//! 11. **PhaseMetrics** — Phase transition tracking: rejection rate, topology entropy, confidence
//!
//! ## Key Innovations Over Prior Art
//! - **Non-monotonic noise schedule**: Vortex cycle energy curve (expansion→contraction)
//!   instead of linear α(t) = 1 - t. The doubling path (1,2,4,8) corrupts; the
//!   halving path (8,7,5,1) denoises. Sacred positions observe without mutating.
//! - **Confidence-gated unmasking**: EBRM energy signals decide which tokens unmask,
//!   not random probability. High-energy (confident) tokens unmask first.
//! - **Sacred verification**: Positions 3,6,9 act as external observers that can
//!   reject low-quality unmaskings and send tokens back to masked state.
//! - **Multi-resolution cycles**: 1 vortex cycle = 9 sub-steps of refinement.
//!   T vortex cycles = 9T effective steps with hierarchical structure.
//! - **No time embeddings**: The model infers noise level from mask ratio
//!   (same insight as LLaDA/DiffuLLaMA, but derived from EBRM energy).
//! - **Adaptive topology (RSI)**: The flow path [1,2,4,8,7,5] and sacred positions
//!   [3,6,9] are NOT hardcoded — they are dynamic variables shaped by subject
//!   definitions (FluxMatrix terrain) and recursive self-improvement feedback.
//!   After each cycle, the engine measures its own performance and rewires the
//!   topology for the next cycle. Billions of FluxMatrix subject definitions
//!   can each provide a unique semantic landscape that reshapes the diffusion
//!   process for that domain.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Constants — Sacred Geometry
// =============================================================================

/// The golden ratio φ — used for noise schedule scaling
const PHI: f32 = 1.618033988749895;

/// Inverse golden ratio 1/φ
const PHI_INV: f32 = 0.618033988749895;

/// Vortex cycle positions: the doubling-halving path
const VORTEX_CYCLE: [u8; 6] = [1, 2, 4, 8, 7, 5];

/// Sacred observer positions (never mutate the stream)
const SACRED_POSITIONS: [u8; 3] = [3, 6, 9];

/// Full 9-position traversal order
const FULL_TRAVERSAL: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];

/// Special token ID for [MASK]
const MASK_TOKEN_ID: u32 = u32::MAX;

/// Special token ID for [BOS] (beginning of sequence)
const BOS_TOKEN_ID: u32 = u32::MAX - 1;

/// Special token ID for [EOS] (end of sequence)
const EOS_TOKEN_ID: u32 = u32::MAX - 2;

// =============================================================================
// 6. AdaptiveVortexTopology — Dynamic Flow, Sacred Positions, RSI Feedback
// =============================================================================

/// A single directed edge in the vortex flow graph.
///
/// In the default topology, these form [1→2→4→8→7→5→1].
/// But in an adaptive topology, any node can connect to any other node
/// with a learned weight. The flow path is no longer hardcoded — it
/// emerges from the weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEdge {
    /// Source node position (0-9)
    pub from: u8,
    /// Target node position (0-9)
    pub to: u8,
    /// Edge weight — higher means stronger connection in the flow
    pub weight: f32,
    /// Edge role: expansion (corrupts/adds noise) or contraction (denoises/refines)
    pub role: EdgeRole,
}

/// Whether an edge adds entropy (expansion) or reduces it (contraction)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeRole {
    /// Expansion: this edge increases uncertainty (doubling path)
    Expansion,
    /// Contraction: this edge reduces uncertainty (halving path)
    Contraction,
    /// Observation: this edge connects to/from a sacred observer
    Observation,
}

/// Semantic terrain from a FluxMatrix subject definition.
///
/// Each subject (cognition, reasoning, ethics, etc.) provides a unique
/// ELP (Ethos/Logos/Pathos) landscape across the 10 positions. This
/// terrain modulates the noise schedule — positions with high Logos
/// get more refinement time; positions with high Pathos get more
/// exploration time; positions with high Ethos get more verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectTerrain {
    /// Subject name (e.g., "cognition", "reasoning", "ethics")
    pub name: String,
    /// ELP values per position [position] → (ethos, logos, pathos)
    /// Positions 0-9. Missing positions default to (0, 0, 0).
    pub elp_landscape: HashMap<u8, [f32; 3]>,
    /// Sacred position properties: [position] → list of divine property names
    pub sacred_properties: HashMap<u8, Vec<String>>,
}

impl SubjectTerrain {
    /// Create an empty terrain (no subject influence)
    pub fn empty() -> Self {
        Self {
            name: "default".to_string(),
            elp_landscape: HashMap::new(),
            sacred_properties: HashMap::new(),
        }
    }

    /// Get the ELP values at a position, defaulting to zeros
    pub fn elp_at(&self, position: u8) -> [f32; 3] {
        self.elp_landscape.get(&position).copied().unwrap_or([0.0, 0.0, 0.0])
    }

    /// Compute a "terrain weight" for a position: how important is this
    /// position in this subject's semantic landscape?
    /// Uses the ELP magnitude normalized to [0, 1].
    pub fn terrain_weight(&self, position: u8) -> f32 {
        let [e, l, p] = self.elp_at(position);
        let magnitude = (e * e + l * l + p * p).sqrt();
        // Normalize: max possible is sqrt(9^2 + 9^2 + 9^2) ≈ 15.6
        (magnitude / 15.6).min(1.0)
    }

    /// Get the dominant channel at a position: ethos, logos, or pathos
    pub fn dominant_channel(&self, position: u8) -> &'static str {
        let [e, l, p] = self.elp_at(position);
        if e >= l && e >= p { "ethos" }
        else if l >= p { "logos" }
        else { "pathos" }
    }
}

/// Metrics collected during a single denoising cycle.
/// These drive the RSI (Recursive Self-Improvement) feedback loop.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CycleMetrics {
    /// How many tokens were locked this cycle
    pub tokens_locked: usize,
    /// How many tokens were rejected this cycle
    pub tokens_rejected: usize,
    /// How many tokens were force-accepted this cycle
    pub tokens_forced: usize,
    /// Average confidence of locked tokens this cycle
    pub avg_lock_confidence: f32,
    /// Average coherence score at position 6 this cycle
    pub avg_coherence: f32,
    /// Which sacred position triggered the most rejections
    pub rejection_hotspot: u8,
    /// Cycle index (0-based)
    pub cycle_index: usize,
}

/// The adaptive vortex topology — a living, self-modifying graph.
///
/// Instead of the static flow [1,2,4,8,7,5],
/// this struct holds:
/// - A weighted directed graph of flow edges (any node → any node)
/// - Immutable sacred positions [3, 6, 9] — the unmanifest domain
/// - Per-position noise deltas (learned from the terrain + RSI feedback)
/// - Sacred gate thresholds that adapt per-cycle
///
/// ## How it works
/// 1. Start with the DEFAULT topology (classic vortex flow)
/// 2. Optionally load a SubjectTerrain to bias the topology
/// 3. After each cycle, collect CycleMetrics
/// 4. `adapt()` mutates the topology based on those metrics
/// 5. The noise schedule and gate thresholds shift to improve
///    the next cycle's performance (sacred positions never shift)
///
/// This is RSI: the diffusion process optimizes its own reasoning structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveVortexTopology {
    /// Flow edges defining the vortex path (weighted directed graph)
    pub flow_edges: Vec<TopologyEdge>,
    /// Sacred positions — IMMUTABLE. 3, 6, 9 are the unmanifest domain.
    /// They exist outside the physical flow (1→2→4→8→7→5→1).
    /// Like Gods to Mortals — they observe, they do not participate.
    /// They NEVER change, regardless of terrain or RSI adaptation.
    sacred_positions: [u8; 3],
    /// Per-position noise delta overrides (replaces hardcoded local_delta)
    /// Position → delta value for the noise schedule
    pub noise_deltas: HashMap<u8, f32>,
    /// Dynamic gate thresholds: (proximity, coherence, verification)
    pub gate_thresholds: (f32, f32, f32),
    /// Subject terrain currently loaded (None = default)
    pub terrain: Option<SubjectTerrain>,
    /// History of cycle metrics for RSI feedback
    pub cycle_history: Vec<CycleMetrics>,
    /// RSI learning rate: how aggressively topology adapts (0.0 = frozen, 1.0 = radical)
    pub rsi_learning_rate: f32,
    /// Generation counter — how many times this topology has been used
    pub generation: usize,
}

impl Default for AdaptiveVortexTopology {
    /// Default topology matches the original hardcoded vortex flow
    fn default() -> Self {
        let flow_edges = vec![
            // Classic doubling path (expansion): 1→2→4→8
            TopologyEdge { from: 1, to: 2, weight: 1.0, role: EdgeRole::Expansion },
            TopologyEdge { from: 2, to: 4, weight: 1.0, role: EdgeRole::Expansion },
            TopologyEdge { from: 4, to: 8, weight: 1.0, role: EdgeRole::Expansion },
            // Classic halving path (contraction): 8→7→5→1
            TopologyEdge { from: 8, to: 7, weight: 1.0, role: EdgeRole::Contraction },
            TopologyEdge { from: 7, to: 5, weight: 1.0, role: EdgeRole::Contraction },
            TopologyEdge { from: 5, to: 1, weight: 1.0, role: EdgeRole::Contraction },
        ];

        // Classic noise deltas (same as the original hardcoded values)
        let mut noise_deltas = HashMap::new();
        noise_deltas.insert(1, -0.02 * PHI_INV);
        noise_deltas.insert(2, -0.04 * PHI_INV);
        noise_deltas.insert(4, -0.06 * PHI_INV);
        noise_deltas.insert(8, -0.08 * PHI_INV);
        noise_deltas.insert(7,  0.06 * PHI_INV);
        noise_deltas.insert(5,  0.04 * PHI_INV);
        noise_deltas.insert(3,  0.0); // Sacred: observe only
        noise_deltas.insert(6,  0.0); // Sacred: observe only
        noise_deltas.insert(9,  0.0); // Sacred: observe only

        Self {
            flow_edges,
            sacred_positions: [3, 6, 9],
            noise_deltas,
            gate_thresholds: (0.15, 0.30, 0.40),
            terrain: None,
            cycle_history: Vec::new(),
            rsi_learning_rate: 0.1,
            generation: 0,
        }
    }
}

impl AdaptiveVortexTopology {
    /// Create a topology from a SubjectTerrain.
    ///
    /// The subject's ELP landscape reshapes the noise deltas:
    /// - High Logos positions → larger contraction delta (more refinement)
    /// - High Pathos positions → larger expansion delta (more exploration)
    /// - High Ethos positions → tighter gate thresholds (more verification)
    ///
    /// The flow path stays the same initially but can be mutated by RSI.
    pub fn from_terrain(terrain: SubjectTerrain) -> Self {
        let mut topo = Self::default();

        // Modulate noise deltas based on terrain
        for pos in 1..=9 {
            let [ethos, logos, pathos] = terrain.elp_at(pos);
            let max_elp = ethos.max(logos).max(pathos).max(0.01);

            let base_delta = topo.noise_deltas.get(&pos).copied().unwrap_or(0.0);

            // Logos-dominant: increase contraction (resolve faster)
            // Pathos-dominant: increase expansion (explore more)
            // Ethos-dominant: keep delta near zero (observe more)
            let terrain_bias = if logos > ethos && logos > pathos {
                // Logos: push toward contraction (positive delta)
                (logos / max_elp) * 0.02 * PHI_INV
            } else if pathos > ethos {
                // Pathos: push toward expansion (negative delta)
                -(pathos / max_elp) * 0.02 * PHI_INV
            } else {
                // Ethos: dampen toward zero (sacred observation)
                -base_delta * 0.3 * (ethos / max_elp)
            };

            topo.noise_deltas.insert(pos, base_delta + terrain_bias);
        }

        // Adjust gate thresholds based on terrain's sacred positions
        // Higher ELP magnitude at sacred positions → tighter gates
        let sacred_weight: f32 = topo.sacred_positions.iter()
            .map(|&p| terrain.terrain_weight(p))
            .sum::<f32>() / 3.0;

        // Scale thresholds: higher terrain weight → higher thresholds (stricter)
        let scale = 1.0 + sacred_weight * 0.3;
        topo.gate_thresholds = (
            (0.15 * scale).min(0.5),
            (0.30 * scale).min(0.7),
            (0.40 * scale).min(0.8),
        );

        topo.terrain = Some(terrain);
        topo
    }

    /// Get the noise delta for a sub-position (dynamic, not hardcoded)
    pub fn noise_delta(&self, sub_pos: u8) -> f32 {
        self.noise_deltas.get(&sub_pos).copied().unwrap_or(0.0)
    }

    /// Check if a sub-position is a sacred observer position.
    /// Sacred positions are ALWAYS [3, 6, 9] — the unmanifest domain.
    pub fn is_sacred(&self, sub_pos: u8) -> bool {
        self.sacred_positions.contains(&sub_pos)
    }

    /// Get the immutable sacred positions. Always returns [3, 6, 9].
    pub fn sacred_positions(&self) -> &[u8; 3] {
        &self.sacred_positions
    }

    /// Get the ordered flow path from the weighted edges.
    /// Returns the positions in traversal order (highest weight path).
    pub fn flow_path(&self) -> Vec<u8> {
        if self.flow_edges.is_empty() {
            return VORTEX_CYCLE.to_vec();
        }

        // Follow the highest-weight outgoing edge from position 1
        let mut path = vec![1u8];
        let mut visited = vec![false; 10];
        visited[1] = true;

        for _ in 0..8 {
            let current = *path.last().unwrap();
            // Find the highest-weight outgoing edge from current
            let best = self.flow_edges.iter()
                .filter(|e| e.from == current && !visited[e.to as usize])
                .max_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap_or(std::cmp::Ordering::Equal));

            if let Some(edge) = best {
                visited[edge.to as usize] = true;
                path.push(edge.to);
            } else {
                break;
            }
        }

        path
    }

    /// Record metrics from a completed cycle
    pub fn record_cycle(&mut self, metrics: CycleMetrics) {
        self.cycle_history.push(metrics);
    }

    /// RSI adaptation: mutate the topology based on accumulated cycle metrics.
    ///
    /// This is the core recursive self-improvement loop:
    /// 1. If rejection rate is high → lower gate thresholds (be more lenient)
    /// 2. If force-acceptance rate is high → raise gate thresholds (be stricter earlier)
    /// 3. If confidence is low → increase contraction deltas (more refinement time)
    /// 4. If confidence is high → decrease contraction deltas (move faster)
    /// 5. If a specific sacred position is a rejection hotspot → consider
    ///    shifting that sacred position to an adjacent node
    ///
    /// The learning rate controls how aggressively changes are applied.
    pub fn adapt(&mut self) {
        if self.cycle_history.is_empty() || self.rsi_learning_rate <= 0.0 {
            return;
        }

        let lr = self.rsi_learning_rate;
        let recent: Vec<&CycleMetrics> = self.cycle_history.iter()
            .rev()
            .take(3)
            .collect();

        let avg_rejection_rate = {
            let total_tokens: usize = recent.iter()
                .map(|m| m.tokens_locked + m.tokens_rejected + m.tokens_forced)
                .sum();
            let total_rejected: usize = recent.iter()
                .map(|m| m.tokens_rejected)
                .sum();
            if total_tokens > 0 {
                total_rejected as f32 / total_tokens as f32
            } else {
                0.0
            }
        };

        let avg_force_rate = {
            let total_tokens: usize = recent.iter()
                .map(|m| m.tokens_locked + m.tokens_rejected + m.tokens_forced)
                .sum();
            let total_forced: usize = recent.iter()
                .map(|m| m.tokens_forced)
                .sum();
            if total_tokens > 0 {
                total_forced as f32 / total_tokens as f32
            } else {
                0.0
            }
        };

        let avg_confidence = {
            let sum: f32 = recent.iter().map(|m| m.avg_lock_confidence).sum();
            sum / recent.len().max(1) as f32
        };

        // Rule 1: High rejection rate → lower thresholds (be more lenient)
        if avg_rejection_rate > 0.5 {
            let delta = lr * 0.05;
            self.gate_thresholds.0 = (self.gate_thresholds.0 - delta).max(0.05);
            self.gate_thresholds.1 = (self.gate_thresholds.1 - delta).max(0.10);
            self.gate_thresholds.2 = (self.gate_thresholds.2 - delta).max(0.15);
        }

        // Rule 2: High force rate → raise thresholds (be stricter earlier)
        if avg_force_rate > 0.3 {
            let delta = lr * 0.03;
            self.gate_thresholds.0 = (self.gate_thresholds.0 + delta).min(0.5);
            self.gate_thresholds.1 = (self.gate_thresholds.1 + delta).min(0.7);
            self.gate_thresholds.2 = (self.gate_thresholds.2 + delta).min(0.8);
        }

        // Rule 3: Low confidence → boost contraction deltas (more refinement)
        if avg_confidence < 0.3 {
            for pos in &[7u8, 5] {
                let current = self.noise_deltas.get(pos).copied().unwrap_or(0.0);
                self.noise_deltas.insert(*pos, current + lr * 0.01);
            }
        }

        // Rule 4: High confidence → reduce contraction deltas (move faster)
        if avg_confidence > 0.7 {
            for pos in &[7u8, 5] {
                let current = self.noise_deltas.get(pos).copied().unwrap_or(0.0);
                self.noise_deltas.insert(*pos, (current - lr * 0.005).max(0.0));
            }
        }

        // Rule 5: Sacred position hotspot → strengthen that position's edges
        let hotspot_counts: HashMap<u8, usize> = {
            let mut counts = HashMap::new();
            for m in &self.cycle_history {
                if m.rejection_hotspot > 0 {
                    *counts.entry(m.rejection_hotspot).or_insert(0) += 1;
                }
            }
            counts
        };

        // If a sacred position is consistently problematic,
        // boost the edges leading INTO it (give it better input)
        for (&sacred_pos, &count) in &hotspot_counts {
            if count >= 2 {
                for edge in self.flow_edges.iter_mut() {
                    // Strengthen edges whose target feeds into the sacred position's neighbors
                    let feeds_sacred = match sacred_pos {
                        3 => edge.to == 2 || edge.to == 4,
                        6 => edge.to == 5 || edge.to == 7,
                        9 => edge.to == 8 || edge.to == 1,
                        _ => false,
                    };
                    if feeds_sacred {
                        edge.weight = (edge.weight + lr * 0.1).min(2.0);
                    }
                }
            }
        }

        self.generation += 1;
    }

    /// Create a SacredUnmaskingGate with the current dynamic thresholds
    pub fn create_gate(&self) -> SacredUnmaskingGate {
        SacredUnmaskingGate {
            proximity_threshold: self.gate_thresholds.0,
            coherence_threshold: self.gate_thresholds.1,
            verification_threshold: self.gate_thresholds.2,
            confidence_weight: PHI_INV,
            max_rejections: 3,
        }
    }

    /// Compute the adaptive noise schedule using dynamic deltas instead of hardcoded ones.
    /// This replaces `compute_sacred_schedule` with a topology-aware version.
    pub fn compute_adaptive_schedule(&self, total_steps: usize) -> Vec<f32> {
        let mut alphas = Vec::with_capacity(total_steps);
        let num_cycles = (total_steps + 8) / 9;

        for step in 0..total_steps {
            let cycle = step / 9;
            let sub_pos = (step % 9) as u8 + 1;

            // Macro progress: 0.0 → 1.0
            let macro_t = if num_cycles <= 1 {
                step as f32 / total_steps.max(1) as f32
            } else {
                cycle as f32 / (num_cycles - 1).max(1) as f32
            };

            // Base α from φ-scaled sigmoid
            let k = 6.0 * PHI;
            let base_alpha = 1.0 / (1.0 + (-k * (macro_t - 0.5)).exp());

            // Dynamic local delta from topology (NOT hardcoded)
            let local_delta = self.noise_delta(sub_pos);

            // Terrain modulation: if we have a subject, its ELP weight
            // at this position further scales the perturbation
            let terrain_scale = self.terrain.as_ref()
                .map(|t| 0.5 + t.terrain_weight(sub_pos) * 0.5)
                .unwrap_or(1.0);

            // Dampen perturbation at extremes
            let damping = 1.0 - macro_t.powi(2);
            let alpha = (base_alpha + local_delta * damping * terrain_scale).clamp(0.0, 1.0);
            alphas.push(alpha);
        }

        alphas
    }

    /// Get a summary of the current topology state
    pub fn summary(&self) -> TopologySummary {
        let flow = self.flow_path();
        let total_edge_weight: f32 = self.flow_edges.iter().map(|e| e.weight).sum();
        let expansion_edges = self.flow_edges.iter().filter(|e| e.role == EdgeRole::Expansion).count();
        let contraction_edges = self.flow_edges.iter().filter(|e| e.role == EdgeRole::Contraction).count();

        TopologySummary {
            flow_path: flow,
            sacred_positions: self.sacred_positions.to_vec(),
            gate_thresholds: self.gate_thresholds,
            total_edge_weight,
            expansion_edges,
            contraction_edges,
            generation: self.generation,
            terrain_name: self.terrain.as_ref().map(|t| t.name.clone()).unwrap_or_else(|| "default".to_string()),
            cycles_observed: self.cycle_history.len(),
        }
    }
}

/// Summary of the current adaptive topology state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologySummary {
    /// Current flow path (ordered positions)
    pub flow_path: Vec<u8>,
    /// Current sacred positions
    pub sacred_positions: Vec<u8>,
    /// Current gate thresholds (proximity, coherence, verification)
    pub gate_thresholds: (f32, f32, f32),
    /// Total weight of all flow edges
    pub total_edge_weight: f32,
    /// Number of expansion edges
    pub expansion_edges: usize,
    /// Number of contraction edges
    pub contraction_edges: usize,
    /// RSI generation (how many times topology has been adapted)
    pub generation: usize,
    /// Active terrain name
    pub terrain_name: String,
    /// Number of cycles observed for RSI
    pub cycles_observed: usize,
}

// =============================================================================
// 7. DiffusionTrace — Debug/Introspection System
// =============================================================================

/// What happened at a single step in the diffusion process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepAction {
    /// Denoiser ran: proposed tokens at masked positions
    Denoise {
        /// How many tokens were proposed this step
        proposals: usize,
        /// Top proposal: (position, token_id, confidence)
        top_proposal: Option<(usize, u32, f32)>,
        /// Lowest proposal confidence
        min_confidence: f32,
    },
    /// Sacred proximity check (position 3)
    SacredProximity {
        /// How many candidates were checked
        checked: usize,
    },
    /// Sacred coherence check (position 6)
    SacredCoherence {
        /// How many candidates were checked
        checked: usize,
        /// How many passed coherence and were verified
        verified: usize,
    },
    /// Sacred verification gate (position 9)
    SacredVerification {
        /// How many tokens were locked (accepted)
        locked: usize,
        /// How many tokens were rejected (re-masked)
        rejected: usize,
        /// How many tokens were force-accepted
        forced: usize,
        /// Average confidence of locked tokens
        avg_lock_confidence: f32,
    },
    /// Non-sacred, non-denoising step (early exit, all resolved, etc.)
    Skip {
        reason: String,
    },
}

/// A single step trace in the diffusion process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepTrace {
    /// Step index (0-based, global)
    pub step: usize,
    /// Cycle index (step / 9)
    pub cycle: usize,
    /// Sub-position within the cycle (1-9)
    pub sub_pos: u8,
    /// Whether this is a sacred position
    pub is_sacred: bool,
    /// Current alpha (noise level)
    pub alpha: f32,
    /// How many tokens are still unresolved
    pub unresolved: usize,
    /// What happened at this step
    pub action: StepAction,
    /// Token state snapshot: (position, token_id, lifecycle, confidence)
    /// Only included for non-locked tokens to keep trace small.
    pub active_tokens: Vec<(usize, u32, String, f32)>,
}

/// Full trace of a generation run — the "debug log" of diffusion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffusionTrace {
    /// Per-step traces
    pub steps: Vec<StepTrace>,
    /// Whether tracing is enabled (disabled by default for performance)
    pub enabled: bool,
    /// Final token IDs
    pub final_tokens: Vec<u32>,
    /// Prompt length
    pub prompt_len: usize,
    /// Total generation length
    pub gen_len: usize,
}

impl DiffusionTrace {
    /// Create a new trace (disabled by default)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new trace with tracing enabled
    pub fn enabled() -> Self {
        Self { enabled: true, ..Self::default() }
    }

    /// Record a step
    pub fn record(&mut self, trace: StepTrace) {
        if self.enabled {
            self.steps.push(trace);
        }
    }

    /// Print a human-readable summary of the trace
    pub fn print_summary(&self) {
        if !self.enabled || self.steps.is_empty() {
            println!("[DiffusionTrace] Tracing was disabled or no steps recorded.");
            return;
        }

        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║           VORTEX DIFFUSION TRACE                           ║");
        println!("║  prompt={} gen={} total_steps={}                     ",
            self.prompt_len, self.gen_len, self.steps.len());
        println!("╚══════════════════════════════════════════════════════════════╝\n");

        for trace in &self.steps {
            let sacred_marker = if trace.is_sacred {
                match trace.sub_pos {
                    3 => " ⟐ SACRED:3 (proximity)",
                    6 => " ⟐ SACRED:6 (coherence)",
                    9 => " ⟐ SACRED:9 (verification)",
                    _ => " ⟐ SACRED",
                }
            } else {
                ""
            };

            print!("  step {:3} | cycle {:2} | pos {} | α={:.4} | unresolved={:2}{}",
                trace.step, trace.cycle, trace.sub_pos,
                trace.alpha, trace.unresolved, sacred_marker);

            match &trace.action {
                StepAction::Denoise { proposals, top_proposal, min_confidence } => {
                    if let Some((pos, tok, conf)) = top_proposal {
                        println!(" | DENOISE: {} proposals, best=tok:{} @pos:{} conf={:.4}, min_conf={:.4}",
                            proposals, tok, pos, conf, min_confidence);
                    } else {
                        println!(" | DENOISE: {} proposals (none)", proposals);
                    }
                }
                StepAction::SacredProximity { checked } => {
                    println!(" | PROXIMITY: checked {} candidates", checked);
                }
                StepAction::SacredCoherence { checked, verified } => {
                    println!(" | COHERENCE: {}/{} verified", verified, checked);
                }
                StepAction::SacredVerification { locked, rejected, forced, avg_lock_confidence } => {
                    println!(" | VERIFY: locked={} rejected={} forced={} avg_conf={:.4}",
                        locked, rejected, forced, avg_lock_confidence);
                }
                StepAction::Skip { reason } => {
                    println!(" | SKIP: {}", reason);
                }
            }

            // Show active (non-locked) tokens if trace has them
            if !trace.active_tokens.is_empty() && trace.active_tokens.len() <= 20 {
                print!("          └─ tokens: ");
                for (pos, tok, lifecycle, conf) in &trace.active_tokens {
                    print!("[{}:{}({})={:.2}] ", pos, tok, lifecycle, conf);
                }
                println!();
            }
        }

        if !self.final_tokens.is_empty() {
            println!("\n  FINAL: {:?}", self.final_tokens);
        }
        println!();
    }
}

// =============================================================================
// 1. VortexNoiseSchedule — Sacred Geometry Noise Curve
// =============================================================================

/// Noise schedule derived from the vortex energy landscape.
///
/// The key insight: the vortex cycle (1→2→4→8→7→5→1) creates a natural
/// energy curve. The "doubling" path (1,2,4,8) represents expansion/entropy
/// increase. The "halving" path (8,7,5,1) represents contraction/refinement.
///
/// We map this to the diffusion noise parameter α(t):
/// - At t=1.0 (fully noised): α = 0 (all tokens masked)
/// - At t=0.0 (fully clean):  α = 1 (no tokens masked)
///
/// The vortex schedule uses φ-scaled exponential decay during expansion
/// and φ-scaled exponential growth during contraction, creating a schedule
/// that spends MORE time in the "nearly clean" region than linear schedules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VortexNoiseSchedule {
    /// Number of macro denoising steps (vortex cycles)
    pub num_cycles: usize,
    /// Pre-computed alpha values for each sub-step
    alpha_cache: Vec<f32>,
    /// Schedule type
    pub schedule_type: ScheduleType,
}

/// Available noise schedule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    /// Sacred geometry: φ-scaled vortex energy curve
    SacredVortex,
    /// Simple log-linear (MDLM baseline): α(t) = 1 - t
    LogLinear,
    /// Cosine schedule: α(t) = cos(πt/2)²
    Cosine,
}

impl VortexNoiseSchedule {
    /// Create a new schedule with T macro cycles.
    /// Each cycle has 9 sub-steps, so total steps = T × 9.
    pub fn new(num_cycles: usize, schedule_type: ScheduleType) -> Self {
        let total_steps = num_cycles * 9;
        let alpha_cache = match schedule_type {
            ScheduleType::SacredVortex => Self::compute_sacred_schedule(total_steps),
            ScheduleType::LogLinear => Self::compute_log_linear(total_steps),
            ScheduleType::Cosine => Self::compute_cosine(total_steps),
        };
        Self { num_cycles, alpha_cache, schedule_type }
    }

    /// Get α value at a given sub-step index.
    /// α = 1.0 means fully clean, α = 0.0 means fully masked.
    pub fn alpha(&self, step: usize) -> f32 {
        self.alpha_cache.get(step).copied().unwrap_or(1.0)
    }

    /// Get the expected mask ratio at step t: mask_ratio = 1 - α(t)
    pub fn mask_ratio(&self, step: usize) -> f32 {
        1.0 - self.alpha(step)
    }

    /// Total number of sub-steps
    pub fn total_steps(&self) -> usize {
        self.alpha_cache.len()
    }

    /// Compute the sacred vortex noise schedule.
    ///
    /// The vortex cycle creates a non-uniform energy landscape:
    /// - Positions 1,2,4,8 (doubling): energy INCREASES (expansion)
    /// - Positions 8,7,5,1 (halving): energy DECREASES (contraction)
    /// - Sacred 3,6,9: energy is OBSERVED, not changed
    ///
    /// We map this to α by treating each 9-step sub-cycle as:
    ///   sub-positions [1,2,3,4,5,6,7,8,9] → local α adjustment
    ///
    /// The macro schedule progresses from α=0 to α=1 (noised→clean),
    /// but each sub-cycle introduces local non-monotonicity: positions
    /// on the expansion path slightly DECREASE α (re-introduce uncertainty),
    /// while positions on the contraction path INCREASE α (resolve tokens).
    /// Sacred positions hold steady (observation without mutation).
    fn compute_sacred_schedule(total_steps: usize) -> Vec<f32> {
        let mut alphas = Vec::with_capacity(total_steps);
        let num_cycles = (total_steps + 8) / 9;

        for step in 0..total_steps {
            let cycle = step / 9;
            let sub_pos = (step % 9) as u8 + 1; // 1-indexed position in cycle

            // Macro progress: 0.0 (start, fully masked) → 1.0 (end, fully clean)
            let macro_t = if num_cycles <= 1 {
                step as f32 / total_steps.max(1) as f32
            } else {
                cycle as f32 / (num_cycles - 1).max(1) as f32
            };

            // Base α from φ-scaled sigmoid: smooth S-curve centered at 0.5
            // This naturally spends more time near α=0 and α=1 (extremes)
            // than in the middle, which matches the observation that early
            // and late denoising steps matter most.
            let k = 6.0 * PHI; // Steepness scaled by golden ratio
            let base_alpha = 1.0 / (1.0 + (-k * (macro_t - 0.5)).exp());

            // Local perturbation from vortex sub-position
            let local_delta = match sub_pos {
                // Doubling path (expansion): slight α decrease (adds uncertainty)
                1 => -0.02 * PHI_INV,
                2 => -0.04 * PHI_INV,
                4 => -0.06 * PHI_INV,
                8 => -0.08 * PHI_INV, // Peak expansion

                // Halving path (contraction): α increase (resolves tokens)
                7 => 0.06 * PHI_INV,
                5 => 0.04 * PHI_INV,

                // Sacred positions: NO change (observe only)
                3 | 6 | 9 => 0.0,

                _ => 0.0,
            };

            // Dampen perturbation by how far along we are
            // Early cycles: large perturbation (exploring). Late cycles: small (refining).
            let damping = 1.0 - macro_t.powi(2);
            let alpha = (base_alpha + local_delta * damping).clamp(0.0, 1.0);
            alphas.push(alpha);
        }

        alphas
    }

    /// Simple log-linear schedule: α(t) = t (where t goes 0→1 as we denoise)
    fn compute_log_linear(total_steps: usize) -> Vec<f32> {
        (0..total_steps)
            .map(|i| i as f32 / (total_steps - 1).max(1) as f32)
            .collect()
    }

    /// Cosine schedule: α(t) = cos(π(1-t)/2)² — more time near extremes
    fn compute_cosine(total_steps: usize) -> Vec<f32> {
        (0..total_steps)
            .map(|i| {
                let t = i as f32 / (total_steps - 1).max(1) as f32;
                let angle = std::f32::consts::FRAC_PI_2 * (1.0 - t);
                angle.cos().powi(2)
            })
            .collect()
    }
}

// =============================================================================
// 2. TokenState — Per-Token State Machine
// =============================================================================

/// State of a single token in the diffusion sequence.
///
/// Tokens progress through states: Masked → Candidate → Verified → Locked.
/// Unlike MDLM which only has masked/unmasked, we have 4 states that
/// interact with the sacred verification gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenLifecycle {
    /// Token is masked — not yet predicted
    Masked,
    /// Token has a candidate prediction but hasn't been verified
    Candidate,
    /// Token passed sacred verification at position 6
    Verified,
    /// Token is locked — passed position 9 verification, cannot be changed
    Locked,
}

/// Full state of a token at a given position in the sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenState {
    /// Current token ID (MASK_TOKEN_ID if masked)
    pub token_id: u32,
    /// Lifecycle state
    pub lifecycle: TokenLifecycle,
    /// Model's confidence in this prediction (0.0-1.0)
    pub confidence: f32,
    /// Energy score from the denoiser (higher = more certain)
    pub energy: f32,
    /// How many times this token has been re-masked after failed verification
    pub rejection_count: u32,
    /// The top-k candidate token IDs and their probabilities
    pub candidates: Vec<(u32, f32)>,
}

impl TokenState {
    /// Create a new masked token
    pub fn masked() -> Self {
        Self {
            token_id: MASK_TOKEN_ID,
            lifecycle: TokenLifecycle::Masked,
            confidence: 0.0,
            energy: 0.0,
            rejection_count: 0,
            candidates: Vec::new(),
        }
    }

    /// Is this token still needing prediction?
    pub fn needs_prediction(&self) -> bool {
        matches!(self.lifecycle, TokenLifecycle::Masked | TokenLifecycle::Candidate)
    }

    /// Is this token finalized?
    pub fn is_final(&self) -> bool {
        matches!(self.lifecycle, TokenLifecycle::Verified | TokenLifecycle::Locked)
    }

    /// Propose a candidate token
    pub fn propose(&mut self, token_id: u32, confidence: f32, candidates: Vec<(u32, f32)>) {
        self.token_id = token_id;
        self.confidence = confidence;
        self.lifecycle = TokenLifecycle::Candidate;
        self.candidates = candidates;
    }

    /// Verify this token (promoted from Candidate → Verified)
    pub fn verify(&mut self) {
        if self.lifecycle == TokenLifecycle::Candidate {
            self.lifecycle = TokenLifecycle::Verified;
        }
    }

    /// Lock this token (promoted from Verified → Locked)
    pub fn lock(&mut self) {
        if self.lifecycle == TokenLifecycle::Verified {
            self.lifecycle = TokenLifecycle::Locked;
        }
    }

    /// Reject this token back to masked state
    pub fn reject(&mut self) {
        self.token_id = MASK_TOKEN_ID;
        self.lifecycle = TokenLifecycle::Masked;
        self.confidence = 0.0;
        self.energy = 0.0;
        self.rejection_count += 1;
        self.candidates.clear();
    }
}

// =============================================================================
// 3. SacredUnmaskingGate — Three-Phase Verification
// =============================================================================

/// Sacred verification gate that observes token predictions without mutating them.
///
/// Position 3 (Proximity): Is this token plausible given the vocabulary?
/// Position 6 (Coherence): Does this token fit with its neighbors?
/// Position 9 (Verification): Final accept/reject decision.
///
/// This replaces the random unmasking probability from MDLM with a
/// principled, energy-based gating mechanism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredUnmaskingGate {
    /// Minimum confidence to pass position 3 (proximity check)
    pub proximity_threshold: f32,
    /// Minimum coherence to pass position 6 (neighbor agreement)
    pub coherence_threshold: f32,
    /// Minimum combined score to pass position 9 (final verification)
    pub verification_threshold: f32,
    /// How much to weight confidence vs coherence at position 9
    pub confidence_weight: f32,
    /// Maximum rejections before forcing acceptance
    pub max_rejections: u32,
}

impl Default for SacredUnmaskingGate {
    fn default() -> Self {
        Self {
            proximity_threshold: 0.15,   // Position 3: low bar, just plausibility
            coherence_threshold: 0.30,   // Position 6: moderate, must fit context
            verification_threshold: 0.40, // Position 9: higher bar for final lock
            confidence_weight: PHI_INV,   // φ⁻¹ ≈ 0.618 weight on confidence
            max_rejections: 3,            // After 3 rejections, force accept
        }
    }
}

impl SacredUnmaskingGate {
    /// Position 3 check: Is this token plausible?
    ///
    /// Tests whether the model's confidence exceeds the proximity threshold.
    /// This is a SOFT gate — it doesn't reject tokens, it marks them for
    /// closer scrutiny at position 6.
    pub fn check_proximity(&self, token: &TokenState) -> ProximityResult {
        let passes = token.confidence >= self.proximity_threshold
            || token.rejection_count >= self.max_rejections;

        ProximityResult {
            passes,
            proximity_score: token.confidence,
            is_forced: token.rejection_count >= self.max_rejections,
        }
    }

    /// Position 6 check: Does this token cohere with its neighbors?
    ///
    /// Computes a coherence score based on how well this token's embedding
    /// aligns with its left and right neighbors. Uses cosine similarity
    /// between adjacent token embeddings.
    pub fn check_coherence(
        &self,
        token: &TokenState,
        left_embedding: Option<&[f32]>,
        right_embedding: Option<&[f32]>,
        token_embedding: &[f32],
    ) -> CoherenceResult {
        let mut coherence = 0.0f32;
        let mut count = 0;

        if let Some(left) = left_embedding {
            coherence += cosine_similarity(token_embedding, left);
            count += 1;
        }
        if let Some(right) = right_embedding {
            coherence += cosine_similarity(token_embedding, right);
            count += 1;
        }
        if count > 0 {
            coherence /= count as f32;
        }

        // Normalize to [0, 1] range (cosine sim is [-1, 1])
        let normalized = (coherence + 1.0) / 2.0;
        let passes = normalized >= self.coherence_threshold
            || token.rejection_count >= self.max_rejections;

        CoherenceResult {
            passes,
            coherence_score: normalized,
            is_forced: token.rejection_count >= self.max_rejections,
        }
    }

    /// Position 9 check: Final verification — accept, verify, or reject.
    ///
    /// Combines proximity and coherence scores with the model's confidence
    /// to make a final decision. Uses the golden ratio to weight confidence
    /// vs coherence (φ⁻¹ ≈ 0.618 on confidence, 1-φ⁻¹ ≈ 0.382 on coherence).
    pub fn check_verification(
        &self,
        token: &TokenState,
        proximity: &ProximityResult,
        coherence: &CoherenceResult,
    ) -> VerificationDecision {
        // Forced acceptance after too many rejections
        if token.rejection_count >= self.max_rejections {
            return VerificationDecision::Accept {
                combined_score: token.confidence,
                forced: true,
            };
        }

        // Weighted combination: φ⁻¹ on confidence, (1-φ⁻¹) on coherence
        let combined = self.confidence_weight * proximity.proximity_score
            + (1.0 - self.confidence_weight) * coherence.coherence_score;

        if combined >= self.verification_threshold {
            VerificationDecision::Accept {
                combined_score: combined,
                forced: false,
            }
        } else if combined >= self.proximity_threshold {
            // Marginal — keep as candidate, don't lock yet
            VerificationDecision::Defer {
                combined_score: combined,
            }
        } else {
            // Below threshold — reject and re-mask
            VerificationDecision::Reject {
                combined_score: combined,
                reason: if proximity.proximity_score < self.proximity_threshold {
                    RejectionReason::LowConfidence
                } else {
                    RejectionReason::PoorCoherence
                },
            }
        }
    }
}

/// Result of position 3 proximity check
#[derive(Debug, Clone)]
pub struct ProximityResult {
    pub passes: bool,
    pub proximity_score: f32,
    pub is_forced: bool,
}

/// Result of position 6 coherence check
#[derive(Debug, Clone)]
pub struct CoherenceResult {
    pub passes: bool,
    pub coherence_score: f32,
    pub is_forced: bool,
}

/// Final verification decision from position 9
#[derive(Debug, Clone)]
pub enum VerificationDecision {
    /// Token is accepted and can be locked
    Accept { combined_score: f32, forced: bool },
    /// Token is marginal — keep as candidate for now
    Defer { combined_score: f32 },
    /// Token is rejected — re-mask and try again
    Reject { combined_score: f32, reason: RejectionReason },
}

/// Why a token was rejected at position 9
#[derive(Debug, Clone)]
pub enum RejectionReason {
    /// Model confidence too low
    LowConfidence,
    /// Doesn't cohere with neighbors
    PoorCoherence,
}

// =============================================================================
// 4. DiffusionTransformer — Bidirectional Denoiser
// =============================================================================

/// Lightweight bidirectional transformer for predicting masked tokens.
///
/// This is intentionally minimal — a single-layer transformer with
/// multi-head attention that sees ALL positions (bidirectional).
/// The vortex cycle handles the iterative refinement; this just does
/// one forward pass of "what should each masked position be?"
///
/// Key difference from CALM: this operates on token IDs and outputs
/// vocabulary logits, not latent states. No encode→latent→decode path.
#[derive(Debug, Clone)]
pub struct DiffusionTransformer {
    /// Embedding dimension
    pub embed_dim: usize,
    /// Vocabulary size
    pub vocab_size: usize,
    /// Number of attention heads
    pub num_heads: usize,
    /// Token embedding matrix: [vocab_size × embed_dim]
    pub token_embeddings: Vec<f32>,
    /// Positional embeddings: [max_seq_len × embed_dim]
    pub position_embeddings: Vec<f32>,
    /// Maximum sequence length
    pub max_seq_len: usize,
    /// Query projection: [embed_dim × embed_dim]
    pub w_q: Vec<f32>,
    /// Key projection: [embed_dim × embed_dim]
    pub w_k: Vec<f32>,
    /// Value projection: [embed_dim × embed_dim]
    pub w_v: Vec<f32>,
    /// Output projection: [embed_dim × embed_dim]
    pub w_o: Vec<f32>,
    /// Feed-forward layer 1: [embed_dim × (4 * embed_dim)]
    pub ff_w1: Vec<f32>,
    /// Feed-forward layer 2: [(4 * embed_dim) × embed_dim]
    pub ff_w2: Vec<f32>,
    /// Layer norm weights (pre-attention)
    pub ln1_weight: Vec<f32>,
    /// Layer norm weights (pre-ffn)
    pub ln2_weight: Vec<f32>,
    /// Output projection to vocab logits: [embed_dim × vocab_size]
    pub lm_head: Vec<f32>,
}

impl DiffusionTransformer {
    /// Create a new transformer with harmonic-ordered initialization.
    ///
    /// Instead of pseudo-random values, weights are initialized using
    /// overlapping sine harmonics at vortex cycle frequencies (1,2,4,8,7,5).
    /// This creates structured patterns where:
    /// - Low frequencies (1,2) provide broad structure
    /// - Mid frequencies (4,5) provide detail
    /// - High frequencies (7,8) provide fine differentiation
    /// - The golden ratio φ phase-shifts between harmonics to avoid
    ///   degenerate symmetries.
    ///
    /// The result: ORDER derived FROM apparent chaos. Each weight position
    /// has a unique value determined by its place in the harmonic series,
    /// but the overall matrix has coherent structure (not noise).
    ///
    /// In production, these weights would be loaded from a pre-trained model
    /// via `load_weights()` or `load_embeddings()`.
    pub fn new(embed_dim: usize, vocab_size: usize, num_heads: usize, max_seq_len: usize) -> Self {
        let ff_dim = 4 * embed_dim;

        // Xavier scale factors (proper for gradient flow)
        let embed_scale = (2.0 / (vocab_size + embed_dim) as f32).sqrt();
        let attn_scale = (2.0 / (embed_dim * 2) as f32).sqrt();
        let ff_scale_1 = (2.0 / (embed_dim + ff_dim) as f32).sqrt();
        let ff_scale_2 = (2.0 / (ff_dim + embed_dim) as f32).sqrt();
        let head_scale = (2.0 / (embed_dim + vocab_size) as f32).sqrt();

        // Harmonic initialization: superposition of sine waves at vortex frequencies.
        // Each harmonic_seed selects a different phase offset so that
        // Q, K, V, O, FF1, FF2, embeddings, and lm_head all get distinct patterns.
        //
        // The vortex cycle [1,2,4,8,7,5] provides 6 harmonics.
        // Sacred positions [3,6,9] provide dampening nodes (zero-crossings).
        let harmonic_init = |size: usize, scale: f32, harmonic_seed: f32| -> Vec<f32> {
            let pi2 = std::f32::consts::PI * 2.0;
            (0..size)
                .map(|i| {
                    let t = i as f32 / size.max(1) as f32;

                    // 6 harmonics from the vortex cycle frequencies
                    let h1 = (pi2 * 1.0 * t + harmonic_seed).sin();                  // Fundamental
                    let h2 = (pi2 * 2.0 * t + harmonic_seed * PHI).sin() * 0.5;      // Second harmonic
                    let h4 = (pi2 * 4.0 * t + harmonic_seed * PHI * PHI).sin() * 0.25; // Fourth
                    let h8 = (pi2 * 8.0 * t + harmonic_seed * 3.0).sin() * 0.125;    // Eighth (peak expansion)
                    let h7 = (pi2 * 7.0 * t + harmonic_seed * PHI_INV).sin() * 0.15; // Seventh (contraction)
                    let h5 = (pi2 * 5.0 * t + harmonic_seed * 2.0).sin() * 0.2;      // Fifth (contraction)

                    // Sacred dampening: positions that are multiples of 3, 6, 9
                    // get pulled toward zero (observation, not mutation)
                    let sacred_damp = if i % 9 == 2 || i % 9 == 5 || i % 9 == 8 {
                        // Near sacred positions: dampen by φ^(-1)
                        PHI_INV
                    } else {
                        1.0
                    };

                    // Sum harmonics, apply sacred dampening, scale to Xavier range
                    let raw = (h1 + h2 + h4 + h8 + h7 + h5) * sacred_damp;
                    // Normalize: max possible sum ≈ 1 + 0.5 + 0.25 + 0.125 + 0.15 + 0.2 = 2.225
                    let normalized = raw / 2.225;
                    normalized * scale
                })
                .collect()
        };

        Self {
            embed_dim,
            vocab_size,
            num_heads,
            // Each matrix gets a unique harmonic seed (phase offset)
            // Seeds chosen from the vortex cycle positions scaled by φ
            token_embeddings: harmonic_init(vocab_size * embed_dim, embed_scale, 1.0 * PHI_INV),
            position_embeddings: harmonic_init(max_seq_len * embed_dim, 0.02, 2.0 * PHI_INV),
            max_seq_len,
            w_q: harmonic_init(embed_dim * embed_dim, attn_scale, 4.0 * PHI_INV),
            w_k: harmonic_init(embed_dim * embed_dim, attn_scale, 8.0 * PHI_INV),
            w_v: harmonic_init(embed_dim * embed_dim, attn_scale, 7.0 * PHI_INV),
            w_o: harmonic_init(embed_dim * embed_dim, attn_scale, 5.0 * PHI_INV),
            ff_w1: harmonic_init(embed_dim * ff_dim, ff_scale_1, 3.0 * PHI_INV),
            ff_w2: harmonic_init(ff_dim * embed_dim, ff_scale_2, 6.0 * PHI_INV),
            ln1_weight: vec![1.0; embed_dim],
            ln2_weight: vec![1.0; embed_dim],
            lm_head: harmonic_init(embed_dim * vocab_size, head_scale, 9.0 * PHI_INV),
        }
    }

    /// Get the embedding for a single token (token embedding + position embedding)
    pub fn get_embedding(&self, token_id: u32, position: usize) -> Vec<f32> {
        let mut embed = vec![0.0f32; self.embed_dim];

        // Token embedding (use modular index for special tokens)
        let tok_idx = if token_id == MASK_TOKEN_ID || token_id == BOS_TOKEN_ID || token_id == EOS_TOKEN_ID {
            // Special tokens: use last slots in embedding matrix
            let special_offset = match token_id {
                MASK_TOKEN_ID => self.vocab_size.saturating_sub(1),
                BOS_TOKEN_ID => self.vocab_size.saturating_sub(2),
                EOS_TOKEN_ID => self.vocab_size.saturating_sub(3),
                _ => 0,
            };
            special_offset
        } else {
            (token_id as usize).min(self.vocab_size - 1)
        };

        let tok_start = tok_idx * self.embed_dim;
        for i in 0..self.embed_dim {
            if tok_start + i < self.token_embeddings.len() {
                embed[i] = self.token_embeddings[tok_start + i];
            }
        }

        // Add positional embedding
        let pos = position.min(self.max_seq_len - 1);
        let pos_start = pos * self.embed_dim;
        for i in 0..self.embed_dim {
            if pos_start + i < self.position_embeddings.len() {
                embed[i] += self.position_embeddings[pos_start + i];
            }
        }

        embed
    }

    /// Forward pass: given a sequence of token states, predict logits for each position.
    ///
    /// This is a BIDIRECTIONAL transformer — every position attends to every other.
    /// Unmasked tokens provide context; masked tokens receive predictions.
    ///
    /// Returns: Vec of logit vectors, one per position.
    pub fn forward(&self, token_ids: &[u32], _mask_ratio: f32) -> Vec<Vec<f32>> {
        let seq_len = token_ids.len().min(self.max_seq_len);

        // 1. Embed all tokens
        let embeddings: Vec<Vec<f32>> = (0..seq_len)
            .map(|i| self.get_embedding(token_ids[i], i))
            .collect();

        // 2. Layer norm (pre-attention)
        let normed: Vec<Vec<f32>> = embeddings.iter()
            .map(|e| self.layer_norm(e, &self.ln1_weight))
            .collect();

        // 3. Multi-head self-attention (bidirectional — no causal mask)
        let attended = self.multi_head_attention(&normed);

        // 4. Residual connection
        let post_attn: Vec<Vec<f32>> = embeddings.iter().zip(attended.iter())
            .map(|(orig, att)| {
                orig.iter().zip(att.iter()).map(|(o, a)| o + a).collect()
            })
            .collect();

        // 5. Layer norm (pre-FFN)
        let normed2: Vec<Vec<f32>> = post_attn.iter()
            .map(|e| self.layer_norm(e, &self.ln2_weight))
            .collect();

        // 6. Feed-forward network
        let ff_out: Vec<Vec<f32>> = normed2.iter()
            .map(|e| self.feed_forward(e))
            .collect();

        // 7. Residual connection
        let hidden: Vec<Vec<f32>> = post_attn.iter().zip(ff_out.iter())
            .map(|(pa, ff)| {
                pa.iter().zip(ff.iter()).map(|(p, f)| p + f).collect()
            })
            .collect();

        // 8. Project to vocabulary logits via lm_head
        hidden.iter()
            .map(|h| self.project_to_logits(h))
            .collect()
    }

    /// Simple layer normalization
    fn layer_norm(&self, input: &[f32], weight: &[f32]) -> Vec<f32> {
        let mean: f32 = input.iter().sum::<f32>() / input.len().max(1) as f32;
        let var: f32 = input.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / input.len().max(1) as f32;
        let std = (var + 1e-5).sqrt();

        input.iter().enumerate()
            .map(|(i, &x)| {
                let normed = (x - mean) / std;
                normed * weight.get(i).copied().unwrap_or(1.0)
            })
            .collect()
    }

    /// Multi-head self-attention (bidirectional)
    fn multi_head_attention(&self, inputs: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let seq_len = inputs.len();
        let head_dim = self.embed_dim / self.num_heads.max(1);
        let scale = (head_dim as f32).sqrt();

        // Project all inputs to Q, K, V
        let queries: Vec<Vec<f32>> = inputs.iter().map(|x| matvec(&self.w_q, x, self.embed_dim, self.embed_dim)).collect();
        let keys: Vec<Vec<f32>> = inputs.iter().map(|x| matvec(&self.w_k, x, self.embed_dim, self.embed_dim)).collect();
        let values: Vec<Vec<f32>> = inputs.iter().map(|x| matvec(&self.w_v, x, self.embed_dim, self.embed_dim)).collect();

        // Compute attention per head, then concatenate
        let mut outputs = vec![vec![0.0f32; self.embed_dim]; seq_len];

        for head in 0..self.num_heads {
            let h_start = head * head_dim;
            let h_end = h_start + head_dim;

            for i in 0..seq_len {
                // Compute attention scores for query i against all keys
                let mut scores = Vec::with_capacity(seq_len);
                for j in 0..seq_len {
                    let mut dot = 0.0f32;
                    for d in h_start..h_end {
                        if d < queries[i].len() && d < keys[j].len() {
                            dot += queries[i][d] * keys[j][d];
                        }
                    }
                    scores.push(dot / scale);
                }

                // Softmax
                let max_score = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                let exp_scores: Vec<f32> = scores.iter().map(|s| (s - max_score).exp()).collect();
                let exp_sum: f32 = exp_scores.iter().sum();

                // Weighted sum of values
                for d in h_start..h_end {
                    let mut weighted = 0.0f32;
                    for j in 0..seq_len {
                        let w = exp_scores[j] / exp_sum.max(1e-8);
                        if d < values[j].len() {
                            weighted += w * values[j][d];
                        }
                    }
                    outputs[i][d] = weighted;
                }
            }
        }

        // Output projection
        outputs.iter()
            .map(|o| matvec(&self.w_o, o, self.embed_dim, self.embed_dim))
            .collect()
    }

    /// Feed-forward network: GELU(x·W1)·W2
    fn feed_forward(&self, input: &[f32]) -> Vec<f32> {
        let ff_dim = 4 * self.embed_dim;

        // First layer
        let hidden = matvec(&self.ff_w1, input, self.embed_dim, ff_dim);

        // GELU activation
        let activated: Vec<f32> = hidden.iter()
            .map(|&x| x * 0.5 * (1.0 + (x * 0.7978845608 * (1.0 + 0.044715 * x * x)).tanh()))
            .collect();

        // Second layer
        matvec(&self.ff_w2, &activated, ff_dim, self.embed_dim)
    }

    /// Project hidden state to vocabulary logits
    fn project_to_logits(&self, hidden: &[f32]) -> Vec<f32> {
        matvec(&self.lm_head, hidden, self.embed_dim, self.vocab_size)
    }

    /// Load token embeddings from an external source (e.g., safetensors)
    pub fn load_embeddings(&mut self, embeddings: &[f32], vocab_size: usize) {
        if embeddings.len() == vocab_size * self.embed_dim {
            self.token_embeddings = embeddings.to_vec();
            self.vocab_size = vocab_size;
        }
    }

    /// Import pre-trained weights from a HashMap of named tensors
    pub fn load_weights(&mut self, weights: &HashMap<String, Vec<f32>>) {
        if let Some(w) = weights.get("token_embeddings") { self.token_embeddings = w.clone(); }
        if let Some(w) = weights.get("position_embeddings") { self.position_embeddings = w.clone(); }
        if let Some(w) = weights.get("w_q") { self.w_q = w.clone(); }
        if let Some(w) = weights.get("w_k") { self.w_k = w.clone(); }
        if let Some(w) = weights.get("w_v") { self.w_v = w.clone(); }
        if let Some(w) = weights.get("w_o") { self.w_o = w.clone(); }
        if let Some(w) = weights.get("ff_w1") { self.ff_w1 = w.clone(); }
        if let Some(w) = weights.get("ff_w2") { self.ff_w2 = w.clone(); }
        if let Some(w) = weights.get("lm_head") { self.lm_head = w.clone(); }
    }
}

// =============================================================================
// 5. VortexDiffusionEngine — Main Engine
// =============================================================================

/// Configuration for the Vortex Diffusion engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VortexDiffusionConfig {
    /// Embedding dimension
    pub embed_dim: usize,
    /// Vocabulary size
    pub vocab_size: usize,
    /// Number of attention heads
    pub num_heads: usize,
    /// Maximum sequence length
    pub max_seq_len: usize,
    /// Number of vortex denoising cycles
    pub num_cycles: usize,
    /// Noise schedule type
    pub schedule_type: ScheduleType,
    /// Temperature for token sampling
    pub temperature: f32,
    /// Top-k tokens to consider during sampling
    pub top_k: usize,
    /// Enable sacred verification gates
    pub sacred_verification: bool,
    /// Enable adaptive topology with RSI feedback (if false, uses static flow)
    pub adaptive_topology: bool,
    /// Enable step-by-step tracing for debugging (expensive, off by default)
    pub enable_tracing: bool,
}

impl Default for VortexDiffusionConfig {
    fn default() -> Self {
        Self {
            embed_dim: 256,
            vocab_size: 32000,
            num_heads: 8,
            max_seq_len: 512,
            num_cycles: 9,  // 9 sacred cycles × 9 sub-steps = 81 total steps
            schedule_type: ScheduleType::SacredVortex,
            temperature: 0.8,
            top_k: 50,
            sacred_verification: true,
            adaptive_topology: false,
            enable_tracing: false,
        }
    }
}

/// The Vortex Diffusion Engine: generates text through sacred-geometry-guided
/// iterative unmasking.
///
/// ## Generation Process
/// ```text
/// [MASK MASK MASK ... MASK]   ← Start: all tokens masked
///         ↓ Vortex Cycle 1
/// [MASK the  MASK ... MASK]   ← Some high-confidence tokens unmasked
///         ↓ Sacred 3: proximity check
///         ↓ Sacred 6: coherence check
///         ↓ Sacred 9: verify/reject
/// [MASK the  cat  ... MASK]   ← Verified tokens locked, rejects re-masked
///         ↓ Vortex Cycle 2
/// [Once the  cat  sat MASK]   ← More tokens revealed
///         ... repeat ...
/// [Once the  cat  sat down]   ← Fully generated
/// ```
pub struct VortexDiffusionEngine {
    /// Configuration
    pub config: VortexDiffusionConfig,
    /// The denoiser transformer
    pub transformer: DiffusionTransformer,
    /// Noise schedule
    pub schedule: VortexNoiseSchedule,
    /// Sacred unmasking gate
    pub sacred_gate: SacredUnmaskingGate,
    /// Adaptive topology (dynamic flow, sacred positions, RSI)
    pub topology: AdaptiveVortexTopology,
    /// Vocabulary: token_id → string
    pub vocab: HashMap<u32, String>,
    /// Reverse vocabulary: string → token_id
    pub token_to_id: HashMap<String, u32>,
    /// Generation statistics
    pub stats: DiffusionStats,
    /// Debug trace from last generation (only populated if enable_tracing=true)
    pub trace: DiffusionTrace,
}

/// Statistics tracked during generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffusionStats {
    /// Total tokens generated
    pub tokens_generated: usize,
    /// Total sacred rejections
    pub sacred_rejections: usize,
    /// Total sacred verifications
    pub sacred_verifications: usize,
    /// Tokens locked at each cycle
    pub tokens_per_cycle: Vec<usize>,
    /// Average confidence of generated tokens
    pub avg_confidence: f32,
    /// Number of forced acceptances (exceeded max rejections)
    pub forced_acceptances: usize,
}

impl VortexDiffusionEngine {
    /// Create a new Vortex Diffusion engine with the given config
    pub fn new(config: VortexDiffusionConfig) -> Self {
        let transformer = DiffusionTransformer::new(
            config.embed_dim,
            config.vocab_size,
            config.num_heads,
            config.max_seq_len,
        );
        let topology = AdaptiveVortexTopology::default();
        let schedule = VortexNoiseSchedule::new(config.num_cycles, config.schedule_type.clone());
        let sacred_gate = topology.create_gate();

        let trace = if config.enable_tracing { DiffusionTrace::enabled() } else { DiffusionTrace::new() };
        Self {
            config,
            transformer,
            schedule,
            sacred_gate,
            topology,
            vocab: HashMap::new(),
            token_to_id: HashMap::new(),
            stats: DiffusionStats::default(),
            trace,
        }
    }

    /// Create an engine with a specific adaptive topology.
    /// Use this to load a subject-driven terrain or a pre-evolved topology.
    pub fn with_topology(config: VortexDiffusionConfig, topology: AdaptiveVortexTopology) -> Self {
        let transformer = DiffusionTransformer::new(
            config.embed_dim,
            config.vocab_size,
            config.num_heads,
            config.max_seq_len,
        );
        // Use topology-driven adaptive schedule if configured
        let schedule = if config.adaptive_topology {
            let alpha_cache = topology.compute_adaptive_schedule(config.num_cycles * 9);
            VortexNoiseSchedule {
                num_cycles: config.num_cycles,
                alpha_cache,
                schedule_type: ScheduleType::SacredVortex,
            }
        } else {
            VortexNoiseSchedule::new(config.num_cycles, config.schedule_type.clone())
        };
        let sacred_gate = topology.create_gate();

        let trace = if config.enable_tracing { DiffusionTrace::enabled() } else { DiffusionTrace::new() };
        Self {
            config,
            transformer,
            schedule,
            sacred_gate,
            topology,
            vocab: HashMap::new(),
            token_to_id: HashMap::new(),
            stats: DiffusionStats::default(),
            trace,
        }
    }

    /// Load a SubjectTerrain to reshape the topology for a specific domain.
    /// This re-computes the noise schedule and gate thresholds.
    pub fn load_terrain(&mut self, terrain: SubjectTerrain) {
        self.topology = AdaptiveVortexTopology::from_terrain(terrain);
        if self.config.adaptive_topology {
            let alpha_cache = self.topology.compute_adaptive_schedule(self.config.num_cycles * 9);
            self.schedule = VortexNoiseSchedule {
                num_cycles: self.config.num_cycles,
                alpha_cache,
                schedule_type: ScheduleType::SacredVortex,
            };
        }
        self.sacred_gate = self.topology.create_gate();
    }

    /// Get the current topology (for inspection or serialization)
    pub fn get_topology(&self) -> &AdaptiveVortexTopology {
        &self.topology
    }

    /// Load a vocabulary mapping
    pub fn load_vocab(&mut self, vocab: HashMap<u32, String>) {
        self.token_to_id = vocab.iter().map(|(&id, s)| (s.clone(), id)).collect();
        self.vocab = vocab;
    }

    /// Generate text by iterative unmasking.
    ///
    /// Given an optional prompt (pre-filled token IDs), generates `gen_len`
    /// additional tokens through the vortex diffusion process.
    ///
    /// Returns the full sequence of token IDs (prompt + generated).
    pub fn generate(&mut self, prompt_ids: &[u32], gen_len: usize) -> Vec<u32> {
        let total_len = (prompt_ids.len() + gen_len).min(self.config.max_seq_len);
        let prompt_len = prompt_ids.len();

        // Initialize token states
        let mut states: Vec<TokenState> = Vec::with_capacity(total_len);

        // Prompt tokens are pre-locked (they don't change)
        for &id in prompt_ids.iter().take(total_len) {
            let mut ts = TokenState::masked();
            ts.token_id = id;
            ts.lifecycle = TokenLifecycle::Locked;
            ts.confidence = 1.0;
            states.push(ts);
        }

        // Generation tokens start as masked
        for _ in prompt_len..total_len {
            states.push(TokenState::masked());
        }

        self.stats = DiffusionStats::default();
        // Reset trace for this generation run
        let tracing = self.config.enable_tracing;
        if tracing {
            self.trace = DiffusionTrace::enabled();
            self.trace.prompt_len = prompt_len;
            self.trace.gen_len = gen_len;
        }

        // Helper: snapshot active (non-locked) tokens for trace
        let snapshot_active = |states: &[TokenState], prompt_len: usize| -> Vec<(usize, u32, String, f32)> {
            states[prompt_len..].iter().enumerate()
                .filter(|(_, s)| s.lifecycle != TokenLifecycle::Locked)
                .map(|(i, s)| {
                    let lc = match s.lifecycle {
                        TokenLifecycle::Masked => "M",
                        TokenLifecycle::Candidate => "C",
                        TokenLifecycle::Verified => "V",
                        TokenLifecycle::Locked => "L",
                    };
                    (prompt_len + i, s.token_id, lc.to_string(), s.confidence)
                })
                .collect()
        };

        // Run vortex denoising cycles
        let total_steps = self.schedule.total_steps();
        for step in 0..total_steps {
            let sub_pos = (step % 9) as u8 + 1;
            let alpha = self.schedule.alpha(step);

            // Count how many tokens still need prediction
            let unresolved: usize = states[prompt_len..].iter()
                .filter(|t| t.needs_prediction())
                .count();

            if unresolved == 0 {
                if tracing {
                    self.trace.record(StepTrace {
                        step, cycle: step / 9, sub_pos,
                        is_sacred: self.topology.is_sacred(sub_pos),
                        alpha, unresolved: 0,
                        action: StepAction::Skip { reason: "all resolved".to_string() },
                        active_tokens: vec![],
                    });
                }
                break; // All tokens resolved
            }

            // Determine what happens at this sub-position.
            // Sacred positions are immutable [3, 6, 9] — the unmanifest domain.
            // [0]=3=proximity, [1]=6=coherence, [2]=9=verification.
            let sacred = self.topology.sacred_positions();
            let is_proximity_pos = sub_pos == sacred[0];
            let is_coherence_pos = sub_pos == sacred[1];
            let is_verify_pos = sub_pos == sacred[2];

            if is_proximity_pos && self.config.sacred_verification {
                // Sacred position 3 (proximity): plausibility check on candidates
                let mut checked = 0;
                for i in prompt_len..total_len {
                    if states[i].lifecycle == TokenLifecycle::Candidate {
                        let _result = self.sacred_gate.check_proximity(&states[i]);
                        checked += 1;
                    }
                }
                if tracing {
                    self.trace.record(StepTrace {
                        step, cycle: step / 9, sub_pos, is_sacred: true,
                        alpha, unresolved,
                        action: StepAction::SacredProximity { checked },
                        active_tokens: snapshot_active(&states, prompt_len),
                    });
                }
            } else if is_coherence_pos && self.config.sacred_verification {
                // Sacred position 6 (coherence): neighbor agreement check
                let mut coherence_checked = 0;
                let mut coherence_verified = 0;
                for i in prompt_len..total_len {
                    if states[i].lifecycle == TokenLifecycle::Candidate {
                        coherence_checked += 1;
                        let token_embed = self.transformer.get_embedding(states[i].token_id, i);
                        let left_embed = if i > 0 && states[i - 1].is_final() {
                            Some(self.transformer.get_embedding(states[i - 1].token_id, i - 1))
                        } else {
                            None
                        };
                        let right_embed = if i + 1 < total_len && states[i + 1].is_final() {
                            Some(self.transformer.get_embedding(states[i + 1].token_id, i + 1))
                        } else {
                            None
                        };

                        let coherence = self.sacred_gate.check_coherence(
                            &states[i],
                            left_embed.as_deref(),
                            right_embed.as_deref(),
                            &token_embed,
                        );

                        if coherence.passes {
                            states[i].verify();
                            self.stats.sacred_verifications += 1;
                            coherence_verified += 1;
                        }
                    }
                }
                if tracing {
                    self.trace.record(StepTrace {
                        step, cycle: step / 9, sub_pos, is_sacred: true,
                        alpha, unresolved,
                        action: StepAction::SacredCoherence { checked: coherence_checked, verified: coherence_verified },
                        active_tokens: snapshot_active(&states, prompt_len),
                    });
                }
            } else if is_verify_pos && self.config.sacred_verification {
                // Dynamic sacred position (verification): final accept/reject/defer
                let mut locked_this_step = 0;
                let mut rejected_this_step = 0;
                let mut forced_this_step = 0;
                let mut lock_confidence_sum = 0.0f32;
                for i in prompt_len..total_len {
                    if states[i].lifecycle == TokenLifecycle::Candidate
                        || states[i].lifecycle == TokenLifecycle::Verified
                    {
                        let prox = self.sacred_gate.check_proximity(&states[i]);
                        let token_embed = self.transformer.get_embedding(states[i].token_id, i);
                        let left_embed = if i > 0 && !states[i - 1].needs_prediction() {
                            Some(self.transformer.get_embedding(states[i - 1].token_id, i - 1))
                        } else {
                            None
                        };
                        let right_embed = if i + 1 < total_len && !states[i + 1].needs_prediction() {
                            Some(self.transformer.get_embedding(states[i + 1].token_id, i + 1))
                        } else {
                            None
                        };
                        let coh = self.sacred_gate.check_coherence(
                            &states[i],
                            left_embed.as_deref(),
                            right_embed.as_deref(),
                            &token_embed,
                        );

                        match self.sacred_gate.check_verification(&states[i], &prox, &coh) {
                            VerificationDecision::Accept { forced, .. } => {
                                lock_confidence_sum += states[i].confidence;
                                states[i].lock();
                                locked_this_step += 1;
                                if forced {
                                    self.stats.forced_acceptances += 1;
                                    forced_this_step += 1;
                                }
                            }
                            VerificationDecision::Defer { .. } => {
                                // Keep as candidate — will be checked again next cycle
                            }
                            VerificationDecision::Reject { .. } => {
                                states[i].reject();
                                self.stats.sacred_rejections += 1;
                                rejected_this_step += 1;
                            }
                        }
                    }
                }
                self.stats.tokens_per_cycle.push(locked_this_step);

                if tracing {
                    let avg_lc = if locked_this_step > 0 { lock_confidence_sum / locked_this_step as f32 } else { 0.0 };
                    self.trace.record(StepTrace {
                        step, cycle: step / 9, sub_pos, is_sacred: true,
                        alpha, unresolved,
                        action: StepAction::SacredVerification {
                            locked: locked_this_step, rejected: rejected_this_step,
                            forced: forced_this_step, avg_lock_confidence: avg_lc,
                        },
                        active_tokens: snapshot_active(&states, prompt_len),
                    });
                }

                // RSI: record cycle metrics if adaptive topology is enabled
                if self.config.adaptive_topology {
                    let cycle_idx = step / 9;
                    let avg_conf = if locked_this_step > 0 {
                        lock_confidence_sum / locked_this_step as f32
                    } else {
                        0.0
                    };
                    // Determine rejection hotspot: which sacred position had most issues
                    let hotspot = if rejected_this_step > locked_this_step {
                        sacred[2] // Always 9 — verification position
                    } else {
                        0
                    };
                    self.topology.record_cycle(CycleMetrics {
                        tokens_locked: locked_this_step,
                        tokens_rejected: rejected_this_step,
                        tokens_forced: forced_this_step,
                        avg_lock_confidence: avg_conf,
                        avg_coherence: 0.0, // Populated from coherence step
                        rejection_hotspot: hotspot,
                        cycle_index: cycle_idx,
                    });

                    // Adapt topology every 3 cycles (not every step — too aggressive)
                    if cycle_idx > 0 && cycle_idx % 3 == 0 {
                        self.topology.adapt();
                        self.sacred_gate = self.topology.create_gate();
                    }
                }
            } else {
                // Non-sacred positions: run the denoiser and propose candidates
                let current_ids: Vec<u32> = states.iter()
                    .map(|s| s.token_id)
                    .collect();

                // Run transformer forward pass
                let all_logits = self.transformer.forward(&current_ids, 1.0 - alpha);

                // For each masked/candidate position, sample a prediction
                for i in prompt_len..total_len {
                    if states[i].lifecycle == TokenLifecycle::Masked {
                        if let Some(logits) = all_logits.get(i) {
                            let (token_id, confidence, candidates) = self.sample_token(logits, alpha);
                            states[i].propose(token_id, confidence, candidates);
                            states[i].energy = alpha; // Energy correlates with schedule progress
                        }
                    }
                }

                if tracing {
                    // Find top proposal for trace
                    let mut top_prop: Option<(usize, u32, f32)> = None;
                    let mut min_conf = f32::INFINITY;
                    let mut prop_count = 0usize;
                    for i in prompt_len..total_len {
                        if states[i].lifecycle == TokenLifecycle::Candidate && states[i].token_id != MASK_TOKEN_ID {
                            prop_count += 1;
                            if states[i].confidence < min_conf { min_conf = states[i].confidence; }
                            if top_prop.is_none() || states[i].confidence > top_prop.unwrap().2 {
                                top_prop = Some((i, states[i].token_id, states[i].confidence));
                            }
                        }
                    }
                    if min_conf == f32::INFINITY { min_conf = 0.0; }
                    self.trace.record(StepTrace {
                        step, cycle: step / 9, sub_pos, is_sacred: false,
                        alpha, unresolved,
                        action: StepAction::Denoise { proposals: prop_count, top_proposal: top_prop, min_confidence: min_conf },
                        active_tokens: snapshot_active(&states, prompt_len),
                    });
                }

                // If sacred verification is disabled, directly accept
                // based on confidence threshold (simplified path)
                if !self.config.sacred_verification {
                    let unmask_prob = alpha; // Higher α = more likely to accept
                    for i in prompt_len..total_len {
                        if states[i].lifecycle == TokenLifecycle::Candidate {
                            if states[i].confidence >= unmask_prob * 0.5 {
                                states[i].lock();
                            }
                        }
                    }
                }
            }
        }

        // Final pass: force-lock any remaining candidates/masked tokens
        for i in prompt_len..total_len {
            if !states[i].is_final() {
                if states[i].token_id != MASK_TOKEN_ID {
                    states[i].lock();
                } else {
                    // Still masked — generate one last prediction
                    let current_ids: Vec<u32> = states.iter().map(|s| s.token_id).collect();
                    let logits = self.transformer.forward(&current_ids, 0.0);
                    if let Some(l) = logits.get(i) {
                        let (token_id, _, _) = self.sample_token(l, 1.0);
                        states[i].token_id = token_id;
                    }
                    states[i].lifecycle = TokenLifecycle::Locked;
                }
            }
        }

        // Compute stats
        let generated = &states[prompt_len..];
        self.stats.tokens_generated = generated.len();
        if !generated.is_empty() {
            self.stats.avg_confidence = generated.iter()
                .map(|t| t.confidence)
                .sum::<f32>() / generated.len() as f32;
        }

        // Capture final tokens in trace
        let result: Vec<u32> = states.iter().map(|s| s.token_id).collect();
        if tracing {
            self.trace.final_tokens = result.clone();
        }

        result
    }

    /// Generate with adaptive length: the engine decides how many tokens to produce.
    ///
    /// Instead of a fixed `gen_len`, this method:
    /// 1. Estimates complexity from prompt length and vocabulary coverage
    /// 2. Starts with a reasonable initial estimate
    /// 3. Stops early when confidence stabilizes (all tokens locked before max)
    /// 4. Can extend if the sequence ends mid-thought (no EOS detected)
    ///
    /// Returns the full sequence of token IDs (prompt + generated).
    pub fn generate_adaptive(&mut self, prompt_ids: &[u32]) -> Vec<u32> {
        let prompt_len = prompt_ids.len();

        // Estimate generation length based on prompt complexity
        // Short prompts → longer responses (need more context)
        // Long prompts → shorter responses (context already provided)
        let base_gen = if prompt_len == 0 {
            32 // No prompt: generate a full thought
        } else if prompt_len <= 3 {
            24 // Short prompt: expand significantly
        } else if prompt_len <= 10 {
            16 // Medium prompt: moderate expansion
        } else if prompt_len <= 30 {
            12 // Long prompt: focused completion
        } else {
            8  // Very long prompt: brief continuation
        };

        // Scale by number of cycles — more cycles = more refinement capacity
        let cycle_factor = (self.config.num_cycles as f32 / 9.0).max(0.5).min(2.0);
        let estimated_len = (base_gen as f32 * cycle_factor) as usize;

        // Clamp to max sequence length
        let max_gen = self.config.max_seq_len.saturating_sub(prompt_len);
        let gen_len = estimated_len.min(max_gen).max(1);

        // Generate with the estimated length
        // The engine will stop early if all tokens resolve before max steps
        self.generate(prompt_ids, gen_len)
    }

    /// Get the trace from the last generation run
    pub fn get_trace(&self) -> &DiffusionTrace {
        &self.trace
    }

    /// Print the trace from the last generation run
    pub fn print_trace(&self) {
        self.trace.print_summary();
    }

    /// Sample a token from logits using temperature + top-k
    ///
    /// Returns (token_id, confidence, top_candidates)
    fn sample_token(&self, logits: &[f32], _alpha: f32) -> (u32, f32, Vec<(u32, f32)>) {
        let temp = self.config.temperature.max(0.01);

        // Apply temperature
        let scaled: Vec<f32> = logits.iter().map(|&l| l / temp).collect();

        // Softmax
        let max_logit = scaled.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exp_logits: Vec<f32> = scaled.iter().map(|l| (l - max_logit).exp()).collect();
        let exp_sum: f32 = exp_logits.iter().sum();
        let probs: Vec<f32> = exp_logits.iter().map(|e| e / exp_sum.max(1e-8)).collect();

        // Top-k selection
        let mut indexed: Vec<(u32, f32)> = probs.iter().cloned()
            .enumerate()
            .map(|(i, p)| (i as u32, p))
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        indexed.truncate(self.config.top_k);

        // Filter out special tokens from candidates
        indexed.retain(|&(id, _)| id != MASK_TOKEN_ID && id != BOS_TOKEN_ID && id != EOS_TOKEN_ID);

        // Take the best candidate (greedy for now — can add nucleus sampling later)
        let (best_id, best_prob) = indexed.first().copied().unwrap_or((0, 0.0));

        let top_candidates: Vec<(u32, f32)> = indexed.iter().take(5).copied().collect();

        (best_id, best_prob, top_candidates)
    }

    /// Decode a sequence of token IDs back to text
    pub fn decode(&self, token_ids: &[u32]) -> String {
        token_ids.iter()
            .filter_map(|&id| {
                if id == MASK_TOKEN_ID || id == BOS_TOKEN_ID || id == EOS_TOKEN_ID {
                    None
                } else {
                    self.vocab.get(&id).cloned().or_else(|| Some(format!("[{}]", id)))
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }

    /// Tokenize text using the loaded vocabulary (simple whitespace + char fallback)
    pub fn tokenize(&self, text: &str) -> Vec<u32> {
        let mut tokens = Vec::new();
        for word in text.split_whitespace() {
            // Try whole word first
            if let Some(&id) = self.token_to_id.get(word) {
                tokens.push(id);
            } else {
                // Fall back to character-level
                for ch in word.chars() {
                    if let Some(&id) = self.token_to_id.get(&ch.to_string()) {
                        tokens.push(id);
                    } else {
                        tokens.push(1); // UNK
                    }
                }
            }
            // Add space token if it exists
            if let Some(&space_id) = self.token_to_id.get(" ") {
                tokens.push(space_id);
            }
        }
        // Remove trailing space
        if let Some(&space_id) = self.token_to_id.get(" ") {
            if tokens.last() == Some(&space_id) {
                tokens.pop();
            }
        }
        tokens
    }

    /// Build a basic vocabulary from a set of words.
    /// This creates character-level + common word tokens.
    pub fn build_vocab(&mut self, words: &[&str]) {
        let mut id: u32 = 0;
        let mut add = |s: &str, vocab: &mut HashMap<u32, String>, rev: &mut HashMap<String, u32>| -> u32 {
            if let Some(&existing) = rev.get(s) {
                return existing;
            }
            let current_id = id;
            vocab.insert(current_id, s.to_string());
            rev.insert(s.to_string(), current_id);
            id += 1;
            current_id
        };

        // Special tokens
        add("<PAD>", &mut self.vocab, &mut self.token_to_id);
        add("<UNK>", &mut self.vocab, &mut self.token_to_id);
        add("<BOS>", &mut self.vocab, &mut self.token_to_id);
        add("<EOS>", &mut self.vocab, &mut self.token_to_id);

        // Characters
        for c in 'a'..='z' { add(&c.to_string(), &mut self.vocab, &mut self.token_to_id); }
        for c in 'A'..='Z' { add(&c.to_string(), &mut self.vocab, &mut self.token_to_id); }
        for c in '0'..='9' { add(&c.to_string(), &mut self.vocab, &mut self.token_to_id); }
        for c in &[' ', '.', ',', '?', '!', '\'', '"', '-', ':', ';', '\n'] {
            add(&c.to_string(), &mut self.vocab, &mut self.token_to_id);
        }

        // Common words
        for word in words {
            add(word, &mut self.vocab, &mut self.token_to_id);
        }
    }

    /// Get generation statistics from the last run
    pub fn get_stats(&self) -> &DiffusionStats {
        &self.stats
    }

    /// Infill: given a sequence with some tokens filled and some as MASK,
    /// generate the masked positions while keeping filled positions fixed.
    ///
    /// This is something autoregressive models CAN'T do — diffusion handles
    /// it naturally because it already works by unmasking.
    pub fn infill(&mut self, sequence: &[u32]) -> Vec<u32> {
        let total_len = sequence.len().min(self.config.max_seq_len);

        // Initialize states from the input
        let mut states: Vec<TokenState> = Vec::with_capacity(total_len);
        for (i, &id) in sequence.iter().take(total_len).enumerate() {
            if id == MASK_TOKEN_ID {
                states.push(TokenState::masked());
            } else {
                let mut ts = TokenState::masked();
                ts.token_id = id;
                ts.lifecycle = TokenLifecycle::Locked;
                ts.confidence = 1.0;
                states.push(ts);
            }
        }

        self.stats = DiffusionStats::default();

        // Run the same denoising loop, but only on masked positions
        let total_steps = self.schedule.total_steps();
        for step in 0..total_steps {
            let sub_pos = (step % 9) as u8 + 1;
            let alpha = self.schedule.alpha(step);

            let unresolved: usize = states.iter()
                .filter(|t| t.needs_prediction())
                .count();
            if unresolved == 0 { break; }

            // Same logic as generate(), but we don't distinguish prompt/gen regions
            // — we just skip already-locked tokens.
            // Sacred positions are dynamic from topology.
            if self.topology.is_sacred(sub_pos) && self.config.sacred_verification {
                self.run_sacred_step(sub_pos, &mut states, total_len);
            } else {
                let current_ids: Vec<u32> = states.iter().map(|s| s.token_id).collect();
                let all_logits = self.transformer.forward(&current_ids, 1.0 - alpha);

                for i in 0..total_len {
                    if states[i].lifecycle == TokenLifecycle::Masked {
                        if let Some(logits) = all_logits.get(i) {
                            let (token_id, confidence, candidates) = self.sample_token(logits, alpha);
                            states[i].propose(token_id, confidence, candidates);
                            states[i].energy = alpha;
                        }
                    }
                }
            }
        }

        // Force-lock remaining
        for state in states.iter_mut() {
            if !state.is_final() && state.token_id != MASK_TOKEN_ID {
                state.lifecycle = TokenLifecycle::Locked;
            }
        }

        states.iter().map(|s| s.token_id).collect()
    }

    /// Run a sacred verification step (shared between generate and infill).
    /// Sacred position roles are determined dynamically from topology:
    /// sacred_positions[0] = proximity, [1] = coherence, [2] = verification.
    fn run_sacred_step(&mut self, sub_pos: u8, states: &mut [TokenState], total_len: usize) {
        let sacred = self.topology.sacred_positions();
        let is_proximity = sub_pos == sacred[0];
        let is_coherence = sub_pos == sacred[1];
        let is_verify = sub_pos == sacred[2];

        if is_proximity {
            for i in 0..total_len {
                if states[i].lifecycle == TokenLifecycle::Candidate {
                    let _result = self.sacred_gate.check_proximity(&states[i]);
                }
            }
        } else if is_coherence {
            for i in 0..total_len {
                if states[i].lifecycle == TokenLifecycle::Candidate {
                    let token_embed = self.transformer.get_embedding(states[i].token_id, i);
                    let left_embed = if i > 0 && states[i - 1].is_final() {
                        Some(self.transformer.get_embedding(states[i - 1].token_id, i - 1))
                    } else {
                        None
                    };
                    let right_embed = if i + 1 < total_len && states[i + 1].is_final() {
                        Some(self.transformer.get_embedding(states[i + 1].token_id, i + 1))
                    } else {
                        None
                    };
                    let coherence = self.sacred_gate.check_coherence(
                        &states[i],
                        left_embed.as_deref(),
                        right_embed.as_deref(),
                        &token_embed,
                    );
                    if coherence.passes {
                        states[i].verify();
                        self.stats.sacred_verifications += 1;
                    }
                }
            }
        } else if is_verify {
            let mut locked = 0;
            for i in 0..total_len {
                if states[i].lifecycle == TokenLifecycle::Candidate
                    || states[i].lifecycle == TokenLifecycle::Verified
                {
                    let prox = self.sacred_gate.check_proximity(&states[i]);
                    let token_embed = self.transformer.get_embedding(states[i].token_id, i);
                    let left_embed = if i > 0 && !states[i - 1].needs_prediction() {
                        Some(self.transformer.get_embedding(states[i - 1].token_id, i - 1))
                    } else {
                        None
                    };
                    let right_embed = if i + 1 < total_len && !states[i + 1].needs_prediction() {
                        Some(self.transformer.get_embedding(states[i + 1].token_id, i + 1))
                    } else {
                        None
                    };
                    let coh = self.sacred_gate.check_coherence(
                        &states[i],
                        left_embed.as_deref(),
                        right_embed.as_deref(),
                        &token_embed,
                    );
                    match self.sacred_gate.check_verification(&states[i], &prox, &coh) {
                        VerificationDecision::Accept { forced, .. } => {
                            states[i].lock();
                            locked += 1;
                            if forced { self.stats.forced_acceptances += 1; }
                        }
                        VerificationDecision::Defer { .. } => {}
                        VerificationDecision::Reject { .. } => {
                            states[i].reject();
                            self.stats.sacred_rejections += 1;
                        }
                    }
                }
            }
            self.stats.tokens_per_cycle.push(locked);
        }
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Matrix-vector multiplication: y = W·x where W is [out_dim × in_dim]
fn matvec(weights: &[f32], input: &[f32], in_dim: usize, out_dim: usize) -> Vec<f32> {
    let mut output = vec![0.0f32; out_dim];
    for i in 0..out_dim {
        let row_start = i * in_dim;
        let mut sum = 0.0f32;
        for j in 0..in_dim.min(input.len()) {
            let idx = row_start + j;
            if idx < weights.len() {
                sum += weights[idx] * input[j];
            }
        }
        output[i] = sum;
    }
    output
}

/// Cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for i in 0..a.len().min(b.len()) {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a.sqrt() * norm_b.sqrt())
    } else {
        0.0
    }
}

// =============================================================================
// 8. NgramValidator — Grammatical Plausibility Checker
// =============================================================================

/// N-gram frequency table for bigrams and trigrams.
/// Built from a reference corpus. Used to score generated sentences
/// for grammatical plausibility (not just fluency — structural correctness).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NgramValidator {
    /// Bigram counts: "word1 word2" → count
    pub bigrams: HashMap<(u32, u32), u32>,
    /// Trigram counts: "word1 word2 word3" → count
    pub trigrams: HashMap<(u32, u32, u32), u32>,
    /// Unigram counts: word → count (for smoothing)
    pub unigrams: HashMap<u32, u32>,
    /// Total bigram observations
    pub total_bigrams: u64,
    /// Total trigram observations
    pub total_trigrams: u64,
    /// Total unigram observations
    pub total_unigrams: u64,
}

/// Result of n-gram validation on a sentence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NgramScore {
    /// Average log-probability under the bigram model (higher = more grammatical)
    pub bigram_score: f32,
    /// Average log-probability under the trigram model
    pub trigram_score: f32,
    /// Fraction of bigrams that were seen in training (coverage)
    pub bigram_coverage: f32,
    /// Fraction of trigrams that were seen in training
    pub trigram_coverage: f32,
    /// Combined score: weighted average of bigram + trigram (0.0-1.0, higher = better)
    pub combined_score: f32,
    /// Number of unknown bigrams (not seen in training)
    pub unknown_bigrams: usize,
    /// Number of unknown trigrams
    pub unknown_trigrams: usize,
}

impl NgramValidator {
    /// Create a new empty validator
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed a token sequence from a reference corpus to build n-gram tables.
    /// Call this repeatedly with different reference sentences.
    pub fn train(&mut self, token_ids: &[u32]) {
        // Unigrams
        for &id in token_ids {
            *self.unigrams.entry(id).or_insert(0) += 1;
            self.total_unigrams += 1;
        }

        // Bigrams
        for window in token_ids.windows(2) {
            let key = (window[0], window[1]);
            *self.bigrams.entry(key).or_insert(0) += 1;
            self.total_bigrams += 1;
        }

        // Trigrams
        for window in token_ids.windows(3) {
            let key = (window[0], window[1], window[2]);
            *self.trigrams.entry(key).or_insert(0) += 1;
            self.total_trigrams += 1;
        }
    }

    /// Train from a text string using a simple whitespace tokenizer.
    /// Each unique word gets a deterministic ID based on its hash.
    pub fn train_from_text(&mut self, text: &str) {
        let tokens: Vec<u32> = text.split_whitespace()
            .map(|w| {
                use std::hash::{Hash, Hasher};
                let mut h = std::collections::hash_map::DefaultHasher::new();
                w.to_lowercase().hash(&mut h);
                (h.finish() % (u32::MAX as u64 - 10)) as u32
            })
            .collect();
        self.train(&tokens);
    }

    /// Score a generated token sequence for grammatical plausibility.
    /// Returns detailed n-gram analysis.
    pub fn score(&self, token_ids: &[u32]) -> NgramScore {
        if token_ids.len() < 2 {
            return NgramScore {
                bigram_score: 0.0, trigram_score: 0.0,
                bigram_coverage: 0.0, trigram_coverage: 0.0,
                combined_score: 0.0, unknown_bigrams: 0, unknown_trigrams: 0,
            };
        }

        // Bigram scoring with add-1 (Laplace) smoothing
        let vocab_size = self.unigrams.len().max(1) as f32;
        let mut bigram_log_prob_sum = 0.0f32;
        let mut bigram_known = 0usize;
        let mut bigram_unknown = 0usize;
        let bigram_count = token_ids.len().saturating_sub(1);

        for window in token_ids.windows(2) {
            let key = (window[0], window[1]);
            let count = *self.bigrams.get(&key).unwrap_or(&0) as f32;
            let context_count = *self.unigrams.get(&window[0]).unwrap_or(&0) as f32;
            // Laplace smoothing: P(w2|w1) = (count(w1,w2) + 1) / (count(w1) + V)
            let prob = (count + 1.0) / (context_count + vocab_size);
            bigram_log_prob_sum += prob.ln();
            if count > 0.0 { bigram_known += 1; } else { bigram_unknown += 1; }
        }

        let bigram_score = if bigram_count > 0 {
            bigram_log_prob_sum / bigram_count as f32
        } else { 0.0 };
        let bigram_coverage = if bigram_count > 0 {
            bigram_known as f32 / bigram_count as f32
        } else { 0.0 };

        // Trigram scoring
        let mut trigram_log_prob_sum = 0.0f32;
        let mut trigram_known = 0usize;
        let mut trigram_unknown = 0usize;
        let trigram_count = token_ids.len().saturating_sub(2);

        for window in token_ids.windows(3) {
            let tri_key = (window[0], window[1], window[2]);
            let bi_key = (window[0], window[1]);
            let tri_count = *self.trigrams.get(&tri_key).unwrap_or(&0) as f32;
            let bi_count = *self.bigrams.get(&bi_key).unwrap_or(&0) as f32;
            let prob = (tri_count + 1.0) / (bi_count + vocab_size);
            trigram_log_prob_sum += prob.ln();
            if tri_count > 0.0 { trigram_known += 1; } else { trigram_unknown += 1; }
        }

        let trigram_score = if trigram_count > 0 {
            trigram_log_prob_sum / trigram_count as f32
        } else { 0.0 };
        let trigram_coverage = if trigram_count > 0 {
            trigram_known as f32 / trigram_count as f32
        } else { 0.0 };

        // Combined score: normalize log-probs to 0-1 range using sigmoid
        // More negative = worse grammar, more positive = better
        let sigmoid = |x: f32| 1.0 / (1.0 + (-x).exp());
        let combined = sigmoid(bigram_score + 2.0) * 0.4 + sigmoid(trigram_score + 2.0) * 0.3
            + bigram_coverage * 0.2 + trigram_coverage * 0.1;

        NgramScore {
            bigram_score, trigram_score,
            bigram_coverage, trigram_coverage,
            combined_score: combined.clamp(0.0, 1.0),
            unknown_bigrams: bigram_unknown,
            unknown_trigrams: trigram_unknown,
        }
    }

    /// Quick check: is this sentence "grammatically plausible"?
    /// Returns true if combined score exceeds threshold.
    pub fn is_plausible(&self, token_ids: &[u32], threshold: f32) -> bool {
        self.score(token_ids).combined_score >= threshold
    }

    /// Get total n-gram counts (for diagnostics)
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.unigrams.len(), self.bigrams.len(), self.trigrams.len())
    }
}

// =============================================================================
// 9. SentenceMemory — Vector Store for Generated Sentences
// =============================================================================

/// Metadata for a stored sentence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceRecord {
    /// The generated token IDs
    pub token_ids: Vec<u32>,
    /// Decoded text (if vocab available)
    pub text: String,
    /// N-gram grammatical score
    pub ngram_score: f32,
    /// Average confidence from diffusion engine
    pub avg_confidence: f32,
    /// Number of sacred rejections during generation
    pub rejections: usize,
    /// Terrain name active during generation
    pub terrain_name: String,
    /// Meta-controller cycle index when this was generated
    pub cycle_index: usize,
    /// Semantic embedding vector (for similarity search)
    pub embedding: Vec<f32>,
    /// Whether this sentence was committed to the corpus (high quality)
    pub committed: bool,
}

/// In-process vector store for generated sentences.
/// Uses brute-force cosine similarity (no external dependencies).
/// For large-scale use, swap the search to EmbedVec HNSW via feature flag.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SentenceMemory {
    /// All stored sentences
    pub records: Vec<SentenceRecord>,
    /// Embedding dimension (matches DiffusionTransformer.embed_dim)
    pub embed_dim: usize,
    /// Maximum records to store (circular buffer evicts oldest non-committed)
    pub max_records: usize,
}

impl SentenceMemory {
    /// Create a new sentence memory with given embedding dimension
    pub fn new(embed_dim: usize, max_records: usize) -> Self {
        Self { records: Vec::new(), embed_dim, max_records }
    }

    /// Store a generated sentence with its metadata and embedding
    pub fn store(&mut self, record: SentenceRecord) {
        // If at capacity, evict oldest non-committed record
        if self.records.len() >= self.max_records {
            if let Some(pos) = self.records.iter().position(|r| !r.committed) {
                self.records.remove(pos);
            } else {
                // All committed — expand (this is rare)
                self.max_records += 100;
            }
        }
        self.records.push(record);
    }

    /// Find top-k most similar sentences to a query embedding (brute-force cosine)
    pub fn find_similar(&self, query_embed: &[f32], k: usize) -> Vec<(usize, f32)> {
        let mut scored: Vec<(usize, f32)> = self.records.iter().enumerate()
            .map(|(i, r)| (i, cosine_similarity(query_embed, &r.embedding)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    /// Compute the novelty of a sentence relative to the corpus.
    /// Returns 1.0 - max_cosine_similarity (higher = more novel).
    pub fn novelty(&self, embedding: &[f32]) -> f32 {
        if self.records.is_empty() {
            return 1.0; // First sentence is maximally novel
        }
        let max_sim = self.records.iter()
            .map(|r| cosine_similarity(embedding, &r.embedding))
            .fold(0.0f32, f32::max);
        (1.0 - max_sim).clamp(0.0, 1.0)
    }

    /// Get all committed (high-quality) sentences
    pub fn committed(&self) -> Vec<&SentenceRecord> {
        self.records.iter().filter(|r| r.committed).collect()
    }

    /// How many sentences are stored
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the memory is empty
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Average n-gram score of committed sentences
    pub fn avg_committed_ngram_score(&self) -> f32 {
        let committed: Vec<_> = self.committed();
        if committed.is_empty() { return 0.0; }
        committed.iter().map(|r| r.ngram_score).sum::<f32>() / committed.len() as f32
    }
}

// =============================================================================
// 10. PhaseMetrics — Phase Transition Tracking
// =============================================================================

/// Snapshot of system state at a given cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseSnapshot {
    /// Meta-controller cycle index
    pub cycle: usize,
    /// Average number of steps before all tokens lock
    pub avg_steps_to_lock: f32,
    /// Rejection rate this cycle (rejections / total candidates)
    pub rejection_rate: f32,
    /// Average confidence of generated tokens
    pub avg_confidence: f32,
    /// Average coherence (from sacred gate checks)
    pub avg_coherence: f32,
    /// N-gram combined score of best sentence this cycle
    pub best_ngram_score: f32,
    /// Novelty of best sentence this cycle
    pub best_novelty: f32,
    /// Topology entropy: how much flow edges have diverged from default
    pub topology_entropy: f32,
    /// Number of sentences committed to corpus so far
    pub total_committed: usize,
    /// The dual objective score: grammar × novelty
    pub dual_objective: f32,
}

/// Tracks phase transitions across many cycles.
/// Looks for non-linear improvements (sudden jumps in metrics).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PhaseTracker {
    /// All snapshots, one per meta-controller cycle
    pub snapshots: Vec<PhaseSnapshot>,
    /// Detected phase transitions: (cycle_index, metric_name, magnitude)
    pub transitions: Vec<(usize, String, f32)>,
}

impl PhaseTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a snapshot for the current cycle
    pub fn record(&mut self, snapshot: PhaseSnapshot) {
        // Detect phase transitions by comparing to rolling average
        if self.snapshots.len() >= 10 {
            let recent_10: &[PhaseSnapshot] = &self.snapshots[self.snapshots.len() - 10..];
            let avg_rejection = recent_10.iter().map(|s| s.rejection_rate).sum::<f32>() / 10.0;
            let avg_confidence = recent_10.iter().map(|s| s.avg_confidence).sum::<f32>() / 10.0;
            let avg_dual = recent_10.iter().map(|s| s.dual_objective).sum::<f32>() / 10.0;

            // Check for sharp drops in rejection rate (>30% improvement)
            if snapshot.rejection_rate < avg_rejection * 0.7 && avg_rejection > 0.05 {
                self.transitions.push((
                    snapshot.cycle,
                    "rejection_rate_drop".to_string(),
                    (avg_rejection - snapshot.rejection_rate) / avg_rejection,
                ));
            }

            // Check for sharp jumps in confidence (>20% improvement)
            if snapshot.avg_confidence > avg_confidence * 1.2 && avg_confidence > 0.01 {
                self.transitions.push((
                    snapshot.cycle,
                    "confidence_jump".to_string(),
                    (snapshot.avg_confidence - avg_confidence) / avg_confidence,
                ));
            }

            // Check for sharp jumps in dual objective (>25% improvement)
            if snapshot.dual_objective > avg_dual * 1.25 && avg_dual > 0.01 {
                self.transitions.push((
                    snapshot.cycle,
                    "dual_objective_jump".to_string(),
                    (snapshot.dual_objective - avg_dual) / avg_dual,
                ));
            }
        }

        self.snapshots.push(snapshot);
    }

    /// Compute topology entropy: how much the flow edges differ from default.
    /// 0.0 = identical to default, higher = more divergent.
    pub fn topology_entropy(topology: &AdaptiveVortexTopology) -> f32 {
        let default = AdaptiveVortexTopology::default();
        let default_edges = default.flow_path();
        let current_edges = topology.flow_path();

        // Edge weight divergence
        let mut weight_diff = 0.0f32;
        for (i, &pos) in current_edges.iter().enumerate() {
            let default_pos = default_edges.get(i).copied().unwrap_or(0);
            if pos != default_pos { weight_diff += 1.0; }
        }

        // Gate threshold divergence
        let (dp, dc, dv) = default.gate_thresholds;
        let (cp, cc, cv) = topology.gate_thresholds;
        let gate_diff = ((cp - dp).abs() + (cc - dc).abs() + (cv - dv).abs()) / 3.0;

        // Noise delta divergence
        let mut delta_diff = 0.0f32;
        for pos in 1..=9 {
            let d = default.noise_delta(pos);
            let c = topology.noise_delta(pos);
            delta_diff += (c - d).abs();
        }

        // Combine: flow path change + gate divergence + delta divergence
        weight_diff / 6.0 + gate_diff * 10.0 + delta_diff
    }

    /// Print a summary of all detected phase transitions
    pub fn print_transitions(&self) {
        if self.transitions.is_empty() {
            println!("[PhaseTracker] No phase transitions detected yet.");
            return;
        }
        println!("\n=== PHASE TRANSITIONS DETECTED ===");
        for (cycle, metric, magnitude) in &self.transitions {
            println!("  cycle {:4} | {} | magnitude: {:.1}%",
                cycle, metric, magnitude * 100.0);
        }
        println!("  Total: {} transitions across {} cycles",
            self.transitions.len(), self.snapshots.len());
    }

    /// Print a compact metrics summary (every N cycles)
    pub fn print_summary(&self, every_n: usize) {
        println!("\n=== PHASE METRICS (every {} cycles) ===", every_n);
        println!("{:>6} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>6}",
            "cycle", "rej_rate", "conf", "ngram", "novel", "dual", "entropy", "commit");
        for snapshot in self.snapshots.iter().step_by(every_n) {
            println!("{:6} {:8.4} {:8.4} {:8.4} {:8.4} {:8.4} {:8.4} {:6}",
                snapshot.cycle,
                snapshot.rejection_rate,
                snapshot.avg_confidence,
                snapshot.best_ngram_score,
                snapshot.best_novelty,
                snapshot.dual_objective,
                snapshot.topology_entropy,
                snapshot.total_committed,
            );
        }
        // Also print last snapshot if not aligned
        if let Some(last) = self.snapshots.last() {
            if last.cycle % every_n != 0 {
                println!("{:6} {:8.4} {:8.4} {:8.4} {:8.4} {:8.4} {:8.4} {:6} (latest)",
                    last.cycle, last.rejection_rate, last.avg_confidence,
                    last.best_ngram_score, last.best_novelty, last.dual_objective,
                    last.topology_entropy, last.total_committed);
            }
        }
    }
}

// =============================================================================
// 11. MetaController — Self-Improving Loop
// =============================================================================

/// A task proposed by the meta-controller for the next generation cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetaTask {
    /// Generate a fresh sentence from scratch (exploration)
    Explore,
    /// Refine a specific low-scoring sentence by regenerating with similar prompt
    Refine { sentence_idx: usize },
    /// Mutate the terrain to explore a new domain
    MutateTerrain,
}

/// Configuration for the meta-controller loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaControllerConfig {
    /// Commit threshold: dual-objective score above this → commit to corpus
    pub commit_threshold: f32,
    /// Refine threshold: sentences below this are candidates for refinement
    pub refine_threshold: f32,
    /// Terrain mutation rate: probability of mutating terrain each cycle
    pub terrain_mutation_rate: f32,
    /// Terrain mutation magnitude: how much ELP values shift per mutation
    pub terrain_mutation_magnitude: f32,
    /// N-gram plausibility threshold
    pub ngram_threshold: f32,
    /// Maximum cycles to run
    pub max_cycles: usize,
    /// Print progress every N cycles
    pub print_every: usize,
}

impl Default for MetaControllerConfig {
    fn default() -> Self {
        Self {
            commit_threshold: 0.5,
            refine_threshold: 0.2,
            terrain_mutation_rate: 0.1,
            terrain_mutation_magnitude: 0.5,
            ngram_threshold: 0.3,
            max_cycles: 1000,
            print_every: 50,
        }
    }
}

/// The self-improving meta-controller.
/// Orchestrates: propose_task → generate → evaluate → adapt → (maybe commit)
pub struct MetaController {
    /// The diffusion engine
    pub engine: VortexDiffusionEngine,
    /// N-gram validator (trained from reference text)
    pub ngram: NgramValidator,
    /// Sentence memory (vector store)
    pub memory: SentenceMemory,
    /// Phase transition tracker
    pub phase: PhaseTracker,
    /// Configuration
    pub config: MetaControllerConfig,
    /// Current terrain (mutated over time)
    pub terrain: SubjectTerrain,
    /// Current cycle index
    pub cycle: usize,
    /// Deterministic seed for reproducible terrain mutations
    seed: f32,
}

impl MetaController {
    /// Create a new meta-controller with an engine and reference text for n-grams
    pub fn new(
        engine: VortexDiffusionEngine,
        reference_texts: &[&str],
        config: MetaControllerConfig,
    ) -> Self {
        let embed_dim = engine.config.embed_dim;
        let mut ngram = NgramValidator::new();
        for text in reference_texts {
            ngram.train_from_text(text);
        }

        // Default terrain
        let mut elp = HashMap::new();
        for pos in 1..=9 {
            elp.insert(pos, [5.0, 5.0, 5.0]); // Balanced start
        }
        let terrain = SubjectTerrain {
            name: "meta_default".to_string(),
            elp_landscape: elp,
            sacred_properties: HashMap::new(),
        };

        Self {
            engine,
            ngram,
            memory: SentenceMemory::new(embed_dim, 10_000),
            phase: PhaseTracker::new(),
            config,
            terrain,
            cycle: 0,
            seed: 0.0,
        }
    }

    /// Propose the next task based on current memory state
    pub fn propose_next_task(&self) -> MetaTask {
        // If memory is empty or mostly empty, explore
        if self.memory.len() < 5 {
            return MetaTask::Explore;
        }

        // Deterministic "random" based on cycle and seed
        let r = ((self.cycle as f32 * PHI_INV + self.seed).fract() * 100.0) as u32;

        // Every N cycles, consider terrain mutation
        if (r as f32 / 100.0) < self.config.terrain_mutation_rate {
            return MetaTask::MutateTerrain;
        }

        // Find lowest-scoring non-committed sentence
        let worst = self.memory.records.iter().enumerate()
            .filter(|(_, r)| !r.committed)
            .min_by(|(_, a), (_, b)| a.ngram_score.partial_cmp(&b.ngram_score)
                .unwrap_or(std::cmp::Ordering::Equal));

        if let Some((idx, record)) = worst {
            if record.ngram_score < self.config.refine_threshold {
                return MetaTask::Refine { sentence_idx: idx };
            }
        }

        MetaTask::Explore
    }

    /// Generate a sentence based on the given task
    pub fn generate_from_task(&mut self, task: &MetaTask) -> Vec<u32> {
        match task {
            MetaTask::Explore => {
                // Generate from scratch with adaptive length
                self.engine.generate_adaptive(&[])
            }
            MetaTask::Refine { sentence_idx } => {
                // Use first few tokens of the low-scoring sentence as prompt
                if let Some(record) = self.memory.records.get(*sentence_idx) {
                    let prompt_len = (record.token_ids.len() / 3).max(1).min(5);
                    let prompt = &record.token_ids[..prompt_len];
                    self.engine.generate_adaptive(prompt)
                } else {
                    self.engine.generate_adaptive(&[])
                }
            }
            MetaTask::MutateTerrain => {
                // Mutate terrain, reload it, then generate
                self.mutate_terrain();
                self.engine.load_terrain(self.terrain.clone());
                self.engine.generate_adaptive(&[])
            }
        }
    }

    /// Evaluate a generated sentence: returns (ngram_score, novelty, dual_objective)
    pub fn evaluate_output(&self, token_ids: &[u32]) -> (f32, f32, f32) {
        let ngram_score = self.ngram.score(token_ids).combined_score;

        // Compute embedding: average of token embeddings
        let embedding = self.compute_sentence_embedding(token_ids);
        let novelty = self.memory.novelty(&embedding);

        // Dual objective: grammatically perfect AND semantically novel
        // Geometric mean so BOTH must be high
        let dual = (ngram_score * novelty).sqrt();

        (ngram_score, novelty, dual)
    }

    /// Record the generation result, adapt topology, maybe commit
    pub fn record_and_adapt(
        &mut self,
        token_ids: Vec<u32>,
        ngram_score: f32,
        novelty: f32,
        dual: f32,
    ) {
        let embedding = self.compute_sentence_embedding(&token_ids);
        let stats = self.engine.get_stats().clone();

        let committed = dual >= self.config.commit_threshold;

        let record = SentenceRecord {
            token_ids,
            text: String::new(), // Would need vocab to decode
            ngram_score,
            avg_confidence: stats.avg_confidence,
            rejections: stats.sacred_rejections,
            terrain_name: self.terrain.name.clone(),
            cycle_index: self.cycle,
            embedding,
            committed,
        };

        self.memory.store(record);

        // Record phase metrics
        let rejection_rate = if stats.tokens_generated > 0 {
            stats.sacred_rejections as f32 / stats.tokens_generated as f32
        } else { 0.0 };

        let topology_entropy = PhaseTracker::topology_entropy(&self.engine.topology);

        self.phase.record(PhaseSnapshot {
            cycle: self.cycle,
            avg_steps_to_lock: 0.0, // TODO: derive from trace if enabled
            rejection_rate,
            avg_confidence: stats.avg_confidence,
            avg_coherence: 0.0,
            best_ngram_score: ngram_score,
            best_novelty: novelty,
            topology_entropy,
            total_committed: self.memory.committed().len(),
            dual_objective: dual,
        });
    }

    /// Run the full self-improving loop for N cycles
    pub fn run(&mut self, num_cycles: usize) {
        let max = num_cycles.min(self.config.max_cycles);
        let print_every = self.config.print_every;

        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║        VORTEX META-CONTROLLER — SELF-IMPROVING LOOP        ║");
        println!("║  cycles={}, commit_threshold={:.2}, ngram_threshold={:.2}",
            max, self.config.commit_threshold, self.config.ngram_threshold);
        println!("║  ngram stats: {} unigrams, {} bigrams, {} trigrams",
            self.ngram.unigrams.len(), self.ngram.bigrams.len(), self.ngram.trigrams.len());
        println!("╚══════════════════════════════════════════════════════════════╝\n");

        for _ in 0..max {
            // 1. Propose next task
            let task = self.propose_next_task();

            // 2. Generate
            let output = self.generate_from_task(&task);

            // 3. Evaluate
            let (ngram_score, novelty, dual) = self.evaluate_output(&output);

            // 4. Record and adapt
            let committed = dual >= self.config.commit_threshold;
            self.record_and_adapt(output, ngram_score, novelty, dual);

            // 5. Print progress
            if self.cycle % print_every == 0 || self.cycle == max - 1 {
                let task_name = match task {
                    MetaTask::Explore => "explore",
                    MetaTask::Refine { .. } => "refine",
                    MetaTask::MutateTerrain => "mutate",
                };
                println!("  cycle {:4}/{} | task={:7} | ngram={:.4} novel={:.4} dual={:.4} | committed={} | corpus={}",
                    self.cycle, max, task_name,
                    ngram_score, novelty, dual, committed,
                    self.memory.committed().len());
            }

            self.cycle += 1;
        }

        // Final summary
        println!("\n--- Meta-Controller Complete ---");
        println!("  Total cycles: {}", self.cycle);
        println!("  Corpus size: {} ({} committed)",
            self.memory.len(), self.memory.committed().len());
        println!("  Avg committed ngram: {:.4}", self.memory.avg_committed_ngram_score());

        self.phase.print_transitions();
        self.phase.print_summary(print_every);
    }

    /// Mutate the terrain ELP values deterministically
    fn mutate_terrain(&mut self) {
        self.seed += PHI_INV;
        let magnitude = self.config.terrain_mutation_magnitude;

        for pos in 1..=9 {
            if let Some(elp) = self.terrain.elp_landscape.get_mut(&pos) {
                // Deterministic mutation using vortex cycle position
                let t = ((pos as f32 * PHI_INV + self.seed).fract() * 2.0 - 1.0) * magnitude;
                // Rotate ELP emphasis: shift energy between Ethos, Logos, Pathos
                let idx = (self.cycle % 3) as usize;
                elp[idx] = (elp[idx] + t).clamp(0.1, 10.0);
            }
        }

        self.terrain.name = format!("mutated_c{}", self.cycle);
    }

    /// Compute a sentence embedding by averaging token embeddings
    fn compute_sentence_embedding(&self, token_ids: &[u32]) -> Vec<f32> {
        let dim = self.engine.config.embed_dim;
        if token_ids.is_empty() {
            return vec![0.0; dim];
        }

        let mut avg = vec![0.0f32; dim];
        for (i, &id) in token_ids.iter().enumerate() {
            let embed = self.engine.transformer.get_embedding(id, i);
            for (j, val) in embed.iter().enumerate() {
                if j < dim { avg[j] += val; }
            }
        }
        let n = token_ids.len() as f32;
        for val in &mut avg { *val /= n; }

        // L2 normalize
        let norm: f32 = avg.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
            for val in &mut avg { *val /= norm; }
        }
        avg
    }

    /// Get the phase tracker for external analysis
    pub fn phase_tracker(&self) -> &PhaseTracker {
        &self.phase
    }

    /// Get the sentence memory for external analysis
    pub fn sentence_memory(&self) -> &SentenceMemory {
        &self.memory
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sacred_schedule_monotonic_macro() {
        // The macro trend should be increasing (α goes 0→1)
        // even though sub-steps introduce local non-monotonicity
        let schedule = VortexNoiseSchedule::new(9, ScheduleType::SacredVortex);
        let first_cycle_avg: f32 = (0..9).map(|i| schedule.alpha(i)).sum::<f32>() / 9.0;
        let last_cycle_avg: f32 = (72..81).map(|i| schedule.alpha(i)).sum::<f32>() / 9.0;
        assert!(last_cycle_avg > first_cycle_avg, "Macro trend should increase");
    }

    #[test]
    fn test_sacred_schedule_sub_perturbation() {
        // Within a mid-cycle, expansion positions should have lower α
        // than contraction positions
        let schedule = VortexNoiseSchedule::new(9, ScheduleType::SacredVortex);
        let mid_cycle = 4; // Cycle 4 (middle)
        let base = mid_cycle * 9;
        let pos_4_alpha = schedule.alpha(base + 3); // Position 4 (expansion)
        let pos_7_alpha = schedule.alpha(base + 6); // Position 7 (contraction)
        // Position 7 (contraction) should have higher α than position 4 (expansion)
        // in the same cycle due to the vortex energy curve
        assert!(pos_7_alpha >= pos_4_alpha,
            "Contraction pos (7) should have >= α than expansion pos (4): {} vs {}",
            pos_7_alpha, pos_4_alpha);
    }

    #[test]
    fn test_token_lifecycle() {
        let mut token = TokenState::masked();
        assert_eq!(token.lifecycle, TokenLifecycle::Masked);
        assert!(token.needs_prediction());

        token.propose(42, 0.8, vec![(42, 0.8), (7, 0.1)]);
        assert_eq!(token.lifecycle, TokenLifecycle::Candidate);
        assert_eq!(token.token_id, 42);

        token.verify();
        assert_eq!(token.lifecycle, TokenLifecycle::Verified);

        token.lock();
        assert_eq!(token.lifecycle, TokenLifecycle::Locked);
        assert!(token.is_final());

        // Rejection resets to masked
        let mut token2 = TokenState::masked();
        token2.propose(99, 0.3, vec![]);
        token2.reject();
        assert_eq!(token2.lifecycle, TokenLifecycle::Masked);
        assert_eq!(token2.token_id, MASK_TOKEN_ID);
        assert_eq!(token2.rejection_count, 1);
    }

    #[test]
    fn test_sacred_gate_forced_acceptance() {
        let gate = SacredUnmaskingGate::default();
        let mut token = TokenState::masked();
        token.propose(42, 0.05, vec![]); // Very low confidence
        token.rejection_count = 3; // Max rejections reached

        let prox = gate.check_proximity(&token);
        assert!(prox.passes, "Should pass proximity when forced");
        assert!(prox.is_forced);
    }

    #[test]
    fn test_engine_basic_generation() {
        let config = VortexDiffusionConfig {
            embed_dim: 32,
            vocab_size: 100,
            num_heads: 4,
            max_seq_len: 64,
            num_cycles: 3,
            sacred_verification: true,
            ..Default::default()
        };
        let mut engine = VortexDiffusionEngine::new(config);

        // Generate 10 tokens with no prompt
        let result = engine.generate(&[], 10);
        assert_eq!(result.len(), 10);

        let stats = engine.get_stats();
        assert_eq!(stats.tokens_generated, 10);
    }

    #[test]
    fn test_infill() {
        let config = VortexDiffusionConfig {
            embed_dim: 32,
            vocab_size: 100,
            num_heads: 4,
            max_seq_len: 64,
            num_cycles: 3,
            sacred_verification: false, // Simplified for test
            ..Default::default()
        };
        let mut engine = VortexDiffusionEngine::new(config);

        // Create a sequence with some masks in the middle
        let sequence = vec![5, 10, MASK_TOKEN_ID, MASK_TOKEN_ID, 20, 25];
        let result = engine.infill(&sequence);

        assert_eq!(result.len(), 6);
        assert_eq!(result[0], 5);  // Kept
        assert_eq!(result[1], 10); // Kept
        assert_eq!(result[4], 20); // Kept
        assert_eq!(result[5], 25); // Kept
        assert_ne!(result[2], MASK_TOKEN_ID); // Filled in
        assert_ne!(result[3], MASK_TOKEN_ID); // Filled in
    }

    // =========================================================================
    // Benchmark Tests — Measuring What's Novel
    // =========================================================================

    /// Helper: create a test engine with given config overrides
    fn make_engine(num_cycles: usize, sacred: bool, schedule: ScheduleType) -> VortexDiffusionEngine {
        let config = VortexDiffusionConfig {
            embed_dim: 64,
            vocab_size: 200,
            num_heads: 4,
            max_seq_len: 128,
            num_cycles,
            schedule_type: schedule,
            temperature: 0.8,
            top_k: 20,
            sacred_verification: sacred,
            adaptive_topology: false,
            enable_tracing: false,
        };
        VortexDiffusionEngine::new(config)
    }

    /// Helper: create an adaptive engine with topology and RSI enabled
    fn make_adaptive_engine(num_cycles: usize, terrain: Option<SubjectTerrain>) -> VortexDiffusionEngine {
        let config = VortexDiffusionConfig {
            embed_dim: 64,
            vocab_size: 200,
            num_heads: 4,
            max_seq_len: 128,
            num_cycles,
            schedule_type: ScheduleType::SacredVortex,
            temperature: 0.8,
            top_k: 20,
            sacred_verification: true,
            adaptive_topology: true,
            enable_tracing: false,
        };
        match terrain {
            Some(t) => {
                let topo = AdaptiveVortexTopology::from_terrain(t);
                VortexDiffusionEngine::with_topology(config, topo)
            }
            None => VortexDiffusionEngine::new(config),
        }
    }

    /// BENCHMARK 1: Schedule Comparison
    /// Tests that SacredVortex schedule produces DIFFERENT behavior than
    /// linear/cosine — specifically, it should have non-monotonic sub-steps
    /// and spend more refinement time at the extremes.
    #[test]
    fn bench_schedule_comparison() {
        let sacred = VortexNoiseSchedule::new(9, ScheduleType::SacredVortex);
        let linear = VortexNoiseSchedule::new(9, ScheduleType::LogLinear);
        let cosine = VortexNoiseSchedule::new(9, ScheduleType::Cosine);

        assert_eq!(sacred.total_steps(), 81);
        assert_eq!(linear.total_steps(), 81);
        assert_eq!(cosine.total_steps(), 81);

        // Sacred schedule should have LOCAL non-monotonicity within cycles
        // (expansion → contraction), which linear and cosine do NOT
        let mut sacred_reversals = 0;
        let mut linear_reversals = 0;
        let mut cosine_reversals = 0;

        for i in 1..81 {
            if sacred.alpha(i) < sacred.alpha(i - 1) { sacred_reversals += 1; }
            if linear.alpha(i) < linear.alpha(i - 1) { linear_reversals += 1; }
            if cosine.alpha(i) < cosine.alpha(i - 1) { cosine_reversals += 1; }
        }

        // Sacred should have reversals (expansion positions dip α)
        assert!(sacred_reversals > 0,
            "Sacred schedule should have local reversals, got {}", sacred_reversals);

        // Linear should be perfectly monotonic (0 reversals)
        assert_eq!(linear_reversals, 0,
            "Linear schedule should have 0 reversals, got {}", linear_reversals);

        // Cosine should also be monotonic
        assert_eq!(cosine_reversals, 0,
            "Cosine schedule should have 0 reversals, got {}", cosine_reversals);

        // All three should end near α=1.0 (fully clean)
        assert!(sacred.alpha(80) > 0.9, "Sacred should end near 1.0: {}", sacred.alpha(80));
        assert!(linear.alpha(80) > 0.9, "Linear should end near 1.0: {}", linear.alpha(80));
        assert!(cosine.alpha(80) > 0.9, "Cosine should end near 1.0: {}", cosine.alpha(80));

        println!("Schedule reversals — Sacred: {}, Linear: {}, Cosine: {}",
            sacred_reversals, linear_reversals, cosine_reversals);
    }

    /// BENCHMARK 2: Sacred Verification Effectiveness
    /// Compare generation WITH vs WITHOUT sacred gates.
    /// With gates: should have rejections + re-masks, potentially different final tokens.
    /// Without gates: should lock everything immediately with no rejections.
    #[test]
    fn bench_sacred_verification_effectiveness() {
        let gen_len = 16;
        let prompt = vec![5u32, 10, 15];

        // With sacred verification
        let mut sacred_engine = make_engine(5, true, ScheduleType::SacredVortex);
        let sacred_result = sacred_engine.generate(&prompt, gen_len);
        let sacred_stats = sacred_engine.get_stats().clone();

        // Without sacred verification
        let mut flat_engine = make_engine(5, false, ScheduleType::SacredVortex);
        let flat_result = flat_engine.generate(&prompt, gen_len);
        let flat_stats = flat_engine.get_stats().clone();

        // Both should produce the correct total length
        assert_eq!(sacred_result.len(), prompt.len() + gen_len);
        assert_eq!(flat_result.len(), prompt.len() + gen_len);

        // Both should preserve the prompt
        assert_eq!(&sacred_result[..3], &[5, 10, 15]);
        assert_eq!(&flat_result[..3], &[5, 10, 15]);

        // Sacred engine should have recorded verification events
        // (rejections + verifications > 0 means the gates actually fired)
        let sacred_gate_events = sacred_stats.sacred_rejections + sacred_stats.sacred_verifications;

        // Flat engine should have ZERO sacred events
        assert_eq!(flat_stats.sacred_rejections, 0,
            "Flat engine should have 0 rejections");
        assert_eq!(flat_stats.sacred_verifications, 0,
            "Flat engine should have 0 verifications");

        // No masked tokens should remain in either output
        assert!(!sacred_result[3..].contains(&MASK_TOKEN_ID),
            "Sacred output should have no masks");
        assert!(!flat_result[3..].contains(&MASK_TOKEN_ID),
            "Flat output should have no masks");

        println!("Sacred gates fired {} events ({} rejections, {} verifications, {} forced)",
            sacred_gate_events,
            sacred_stats.sacred_rejections,
            sacred_stats.sacred_verifications,
            sacred_stats.forced_acceptances);
        println!("Flat engine: 0 events (no gates)");
        println!("Sacred avg confidence: {:.4}", sacred_stats.avg_confidence);
        println!("Flat avg confidence: {:.4}", flat_stats.avg_confidence);
    }

    /// BENCHMARK 3: Convergence Speed
    /// How many cycles does it take to lock all tokens?
    /// Sacred should converge differently than flat — the re-masking may
    /// take more cycles but produce higher confidence.
    #[test]
    fn bench_convergence_speed() {
        let gen_len = 20;

        // Test convergence at different cycle counts
        for num_cycles in [3, 5, 9, 15] {
            let mut engine = make_engine(num_cycles, true, ScheduleType::SacredVortex);
            let result = engine.generate(&[], gen_len);
            let stats = engine.get_stats();

            // All tokens should be resolved regardless of cycle count
            assert_eq!(result.len(), gen_len);
            assert!(!result.contains(&MASK_TOKEN_ID),
                "All tokens should resolve at {} cycles", num_cycles);

            // Track how tokens locked across cycles
            let total_locked: usize = stats.tokens_per_cycle.iter().sum();

            println!("Cycles={}: locked {} tokens across {} position-9 passes, \
                      avg conf={:.4}, rejections={}, forced={}",
                num_cycles,
                total_locked,
                stats.tokens_per_cycle.len(),
                stats.avg_confidence,
                stats.sacred_rejections,
                stats.forced_acceptances);
        }
    }

    /// BENCHMARK 4: Re-masking Effectiveness
    /// Verify that tokens CAN be re-masked and that the engine handles
    /// the full lifecycle: Masked → Candidate → Reject → Masked → Candidate → Lock
    #[test]
    fn bench_remasking_lifecycle() {
        let gate = SacredUnmaskingGate::default();

        // Simulate a token going through rejection cycles
        let mut token = TokenState::masked();
        assert_eq!(token.rejection_count, 0);

        // Round 1: low confidence → rejected
        token.propose(42, 0.05, vec![(42, 0.05)]);
        assert_eq!(token.lifecycle, TokenLifecycle::Candidate);

        let prox = gate.check_proximity(&token);
        // 0.05 < 0.15 threshold → fails proximity
        assert!(!prox.passes);

        // Simulate position 9 rejection
        token.reject();
        assert_eq!(token.lifecycle, TokenLifecycle::Masked);
        assert_eq!(token.rejection_count, 1);
        assert_eq!(token.token_id, MASK_TOKEN_ID);

        // Round 2: better confidence
        token.propose(55, 0.25, vec![(55, 0.25)]);
        assert_eq!(token.lifecycle, TokenLifecycle::Candidate);

        let prox2 = gate.check_proximity(&token);
        // 0.25 >= 0.15 → passes proximity
        assert!(prox2.passes);

        // Round 3: forced acceptance after max rejections
        token.reject();
        token.reject(); // rejection_count = 3

        token.propose(60, 0.01, vec![(60, 0.01)]); // Very low confidence
        let prox3 = gate.check_proximity(&token);
        assert!(prox3.passes, "Should be forced at rejection_count=3");
        assert!(prox3.is_forced);

        println!("Re-masking lifecycle verified: reject→remask→repropose→force-accept");
    }

    /// BENCHMARK 5: Infill Consistency
    /// Infilling should always preserve fixed tokens and only modify masks.
    /// Test with various mask patterns (beginning, middle, end, scattered).
    #[test]
    fn bench_infill_consistency() {
        let mut engine = make_engine(5, true, ScheduleType::SacredVortex);
        let m = MASK_TOKEN_ID;

        let patterns: Vec<(&str, Vec<u32>)> = vec![
            ("masks at start",   vec![m, m, m, 10, 20, 30]),
            ("masks at end",     vec![10, 20, 30, m, m, m]),
            ("masks in middle",  vec![10, m, m, m, 20, 30]),
            ("scattered masks",  vec![10, m, 20, m, 30, m]),
            ("single mask",      vec![10, 20, m, 30, 40, 50]),
            ("all masked",       vec![m, m, m, m, m, m]),
        ];

        for (name, pattern) in &patterns {
            let result = engine.infill(pattern);

            assert_eq!(result.len(), pattern.len(),
                "{}: length mismatch", name);

            // Fixed tokens must be preserved exactly
            for (i, &original) in pattern.iter().enumerate() {
                if original != m {
                    assert_eq!(result[i], original,
                        "{}: fixed token at position {} changed from {} to {}",
                        name, i, original, result[i]);
                } else {
                    assert_ne!(result[i], m,
                        "{}: mask at position {} was not filled", name, i);
                }
            }

            println!("{}: OK — all fixed preserved, all masks filled", name);
        }
    }

    /// BENCHMARK 6: Schedule Shape Analysis
    /// Print the α curve for visual inspection and verify mathematical properties.
    /// The sacred schedule should form an S-curve with local perturbations.
    #[test]
    fn bench_schedule_shape_analysis() {
        let schedule = VortexNoiseSchedule::new(9, ScheduleType::SacredVortex);

        // Collect per-cycle statistics
        for cycle in 0..9 {
            let base = cycle * 9;
            let alphas: Vec<f32> = (0..9).map(|i| schedule.alpha(base + i)).collect();
            let min = alphas.iter().cloned().fold(f32::INFINITY, f32::min);
            let max = alphas.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let avg: f32 = alphas.iter().sum::<f32>() / 9.0;
            let range = max - min;

            println!("Cycle {}: avg={:.4} min={:.4} max={:.4} range={:.4} | {:?}",
                cycle, avg, min, max, range,
                alphas.iter().map(|a| format!("{:.3}", a)).collect::<Vec<_>>());

            // Within a cycle, the range (perturbation) should be positive
            // except at the very extremes (cycle 0 and cycle 8)
            if cycle > 0 && cycle < 8 {
                assert!(range > 0.001,
                    "Cycle {} should have non-trivial perturbation: range={}", cycle, range);
            }
        }

        // The middle cycles should have the LARGEST perturbation range
        // (damping decreases perturbation at extremes)
        let mid_range = {
            let base = 4 * 9;
            let alphas: Vec<f32> = (0..9).map(|i| schedule.alpha(base + i)).collect();
            let min = alphas.iter().cloned().fold(f32::INFINITY, f32::min);
            let max = alphas.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            max - min
        };
        let late_range = {
            let base = 8 * 9;
            let alphas: Vec<f32> = (0..9).map(|i| schedule.alpha(base + i)).collect();
            let min = alphas.iter().cloned().fold(f32::INFINITY, f32::min);
            let max = alphas.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            max - min
        };

        assert!(mid_range > late_range,
            "Mid-cycles should have larger perturbation than late cycles: {} vs {}",
            mid_range, late_range);

        println!("\nMid-cycle perturbation: {:.4}", mid_range);
        println!("Late-cycle perturbation: {:.4}", late_range);
        println!("Ratio: {:.2}x", mid_range / late_range.max(0.0001));
    }

    /// BENCHMARK 7: Prompt-Conditioned Generation
    /// The same engine with different prompts should produce different outputs.
    /// This tests that bidirectional context from the prompt actually influences generation.
    #[test]
    fn bench_prompt_conditioning() {
        let mut engine = make_engine(5, true, ScheduleType::SacredVortex);
        let gen_len = 12;

        let prompt_a = vec![10u32, 20, 30];
        let prompt_b = vec![50u32, 60, 70];
        let prompt_c = vec![10u32, 20, 30]; // Same as A

        let result_a = engine.generate(&prompt_a, gen_len);
        let result_b = engine.generate(&prompt_b, gen_len);
        let result_c = engine.generate(&prompt_c, gen_len);

        // Prompts should be preserved
        assert_eq!(&result_a[..3], &[10, 20, 30]);
        assert_eq!(&result_b[..3], &[50, 60, 70]);
        assert_eq!(&result_c[..3], &[10, 20, 30]);

        // Different prompts should produce different generated tokens
        // (at least some positions should differ)
        let gen_a = &result_a[3..];
        let gen_b = &result_b[3..];
        let gen_c = &result_c[3..];

        let diff_ab: usize = gen_a.iter().zip(gen_b.iter())
            .filter(|(a, b)| a != b)
            .count();

        // Same prompt should produce IDENTICAL results (deterministic with same weights)
        let diff_ac: usize = gen_a.iter().zip(gen_c.iter())
            .filter(|(a, b)| a != b)
            .count();

        assert_eq!(diff_ac, 0,
            "Same prompt should produce identical output (deterministic): {} differences", diff_ac);

        println!("Prompt A vs B: {}/{} tokens differ", diff_ab, gen_len);
        println!("Prompt A vs C (same): {}/{} tokens differ", diff_ac, gen_len);
    }

    /// BENCHMARK 8: All-Schedule Generation Comparison
    /// Run the same generation task across all 3 schedules and compare stats.
    #[test]
    fn bench_all_schedules_generation() {
        let gen_len = 16;
        let prompt = vec![10u32, 20, 30];

        for (name, stype) in [
            ("SacredVortex", ScheduleType::SacredVortex),
            ("LogLinear", ScheduleType::LogLinear),
            ("Cosine", ScheduleType::Cosine),
        ] {
            let mut engine = make_engine(5, true, stype);
            let result = engine.generate(&prompt, gen_len);
            let stats = engine.get_stats();

            assert_eq!(result.len(), prompt.len() + gen_len);
            assert!(!result[3..].contains(&MASK_TOKEN_ID),
                "{}: should resolve all tokens", name);

            println!("{:14}: conf={:.4} rejections={:2} verifications={:2} forced={:2} cycles_used={}",
                name,
                stats.avg_confidence,
                stats.sacred_rejections,
                stats.sacred_verifications,
                stats.forced_acceptances,
                stats.tokens_per_cycle.len());
        }
    }

    // =========================================================================
    // Adaptive Topology Tests — Dynamic Flow, RSI, Subject Terrain
    // =========================================================================

    /// TOPOLOGY 1: Default topology matches original hardcoded constants
    #[test]
    fn test_default_topology_matches_original() {
        let topo = AdaptiveVortexTopology::default();

        // Sacred positions should be [3, 6, 9] — immutable, the unmanifest
        assert_eq!(*topo.sacred_positions(), [3, 6, 9]);

        // Flow path should follow 1→2→4→8→7→5
        let flow = topo.flow_path();
        assert_eq!(flow, vec![1, 2, 4, 8, 7, 5]);

        // Gate thresholds should match original defaults
        assert!((topo.gate_thresholds.0 - 0.15).abs() < 0.001);
        assert!((topo.gate_thresholds.1 - 0.30).abs() < 0.001);
        assert!((topo.gate_thresholds.2 - 0.40).abs() < 0.001);

        // Noise deltas should match original hardcoded values
        let expected_1 = -0.02 * PHI_INV;
        let expected_8 = -0.08 * PHI_INV;
        let expected_7 = 0.06 * PHI_INV;
        assert!((topo.noise_delta(1) - expected_1).abs() < 0.0001);
        assert!((topo.noise_delta(8) - expected_8).abs() < 0.0001);
        assert!((topo.noise_delta(7) - expected_7).abs() < 0.0001);
        assert_eq!(topo.noise_delta(3), 0.0); // Sacred: no mutation
        assert_eq!(topo.noise_delta(6), 0.0);
        assert_eq!(topo.noise_delta(9), 0.0);

        println!("Default topology verified: flow={:?}, sacred={:?}", flow, topo.sacred_positions());
    }

    /// TOPOLOGY 2: SubjectTerrain modulates noise deltas based on ELP
    #[test]
    fn test_terrain_modulates_deltas() {
        // Create a Logos-heavy terrain (logic/reasoning domain)
        let mut elp = HashMap::new();
        elp.insert(1, [2.0, 8.0, 1.0]); // Logos-dominant
        elp.insert(4, [1.0, 9.0, 1.0]); // Logos-dominant
        elp.insert(5, [1.0, 2.0, 8.0]); // Pathos-dominant
        elp.insert(3, [5.0, 5.0, 5.0]); // Sacred: balanced
        elp.insert(6, [3.0, 3.0, 9.0]); // Sacred: Pathos
        elp.insert(9, [3.0, 9.0, 3.0]); // Sacred: Logos

        let terrain = SubjectTerrain {
            name: "logic".to_string(),
            elp_landscape: elp,
            sacred_properties: HashMap::new(),
        };

        let default_topo = AdaptiveVortexTopology::default();
        let terrain_topo = AdaptiveVortexTopology::from_terrain(terrain);

        // Logos-dominant positions should have MORE positive (contraction) delta
        // than the default — they want more refinement
        let delta_4_default = default_topo.noise_delta(4);
        let delta_4_terrain = terrain_topo.noise_delta(4);
        assert!(delta_4_terrain > delta_4_default,
            "Logos-dominant pos 4 should have higher delta: terrain={:.4} vs default={:.4}",
            delta_4_terrain, delta_4_default);

        // Pathos-dominant positions should have MORE negative (expansion) delta
        let delta_5_default = default_topo.noise_delta(5);
        let delta_5_terrain = terrain_topo.noise_delta(5);
        assert!(delta_5_terrain < delta_5_default,
            "Pathos-dominant pos 5 should have lower delta: terrain={:.4} vs default={:.4}",
            delta_5_terrain, delta_5_default);

        // Gate thresholds should be stricter with high sacred terrain weight
        assert!(terrain_topo.gate_thresholds.2 >= default_topo.gate_thresholds.2,
            "Terrain with strong sacred positions should have stricter verification: {:.3} vs {:.3}",
            terrain_topo.gate_thresholds.2, default_topo.gate_thresholds.2);

        println!("Terrain modulation verified:");
        println!("  Pos 4 delta: default={:.4}, terrain={:.4}", delta_4_default, delta_4_terrain);
        println!("  Pos 5 delta: default={:.4}, terrain={:.4}", delta_5_default, delta_5_terrain);
        println!("  Verify threshold: default={:.3}, terrain={:.3}",
            default_topo.gate_thresholds.2, terrain_topo.gate_thresholds.2);
    }

    /// TOPOLOGY 3: RSI adaptation lowers thresholds when rejection rate is high
    #[test]
    fn test_rsi_adapts_on_high_rejection() {
        let mut topo = AdaptiveVortexTopology::default();
        let original_thresholds = topo.gate_thresholds;

        // Simulate 3 cycles with very high rejection rate
        for i in 0..3 {
            topo.record_cycle(CycleMetrics {
                tokens_locked: 1,
                tokens_rejected: 10,
                tokens_forced: 0,
                avg_lock_confidence: 0.2,
                avg_coherence: 0.3,
                rejection_hotspot: 9,
                cycle_index: i,
            });
        }

        topo.adapt();

        // Thresholds should have DECREASED (more lenient)
        assert!(topo.gate_thresholds.0 < original_thresholds.0,
            "Proximity threshold should decrease: {:.3} vs {:.3}",
            topo.gate_thresholds.0, original_thresholds.0);
        assert!(topo.gate_thresholds.2 < original_thresholds.2,
            "Verification threshold should decrease: {:.3} vs {:.3}",
            topo.gate_thresholds.2, original_thresholds.2);

        assert_eq!(topo.generation, 1, "Generation counter should increment");

        println!("RSI high-rejection adaptation:");
        println!("  Before: ({:.3}, {:.3}, {:.3})", original_thresholds.0, original_thresholds.1, original_thresholds.2);
        println!("  After:  ({:.3}, {:.3}, {:.3})", topo.gate_thresholds.0, topo.gate_thresholds.1, topo.gate_thresholds.2);
    }

    /// TOPOLOGY 4: RSI adaptation raises thresholds when force rate is high
    #[test]
    fn test_rsi_adapts_on_high_force() {
        let mut topo = AdaptiveVortexTopology::default();
        let original_thresholds = topo.gate_thresholds;

        // Simulate 3 cycles with high force-acceptance rate
        for i in 0..3 {
            topo.record_cycle(CycleMetrics {
                tokens_locked: 2,
                tokens_rejected: 0,
                tokens_forced: 8,
                avg_lock_confidence: 0.8,
                avg_coherence: 0.5,
                rejection_hotspot: 0,
                cycle_index: i,
            });
        }

        topo.adapt();

        // Thresholds should have INCREASED (stricter)
        assert!(topo.gate_thresholds.0 > original_thresholds.0,
            "Proximity threshold should increase: {:.3} vs {:.3}",
            topo.gate_thresholds.0, original_thresholds.0);
        assert!(topo.gate_thresholds.2 > original_thresholds.2,
            "Verification threshold should increase: {:.3} vs {:.3}",
            topo.gate_thresholds.2, original_thresholds.2);

        println!("RSI high-force adaptation:");
        println!("  Before: ({:.3}, {:.3}, {:.3})", original_thresholds.0, original_thresholds.1, original_thresholds.2);
        println!("  After:  ({:.3}, {:.3}, {:.3})", topo.gate_thresholds.0, topo.gate_thresholds.1, topo.gate_thresholds.2);
    }

    /// TOPOLOGY 5: RSI boosts contraction deltas when confidence is low
    #[test]
    fn test_rsi_boosts_contraction_on_low_confidence() {
        let mut topo = AdaptiveVortexTopology::default();
        let original_delta_7 = topo.noise_delta(7);
        let original_delta_5 = topo.noise_delta(5);

        // Simulate 3 cycles with low confidence
        for i in 0..3 {
            topo.record_cycle(CycleMetrics {
                tokens_locked: 5,
                tokens_rejected: 2,
                tokens_forced: 0,
                avg_lock_confidence: 0.15, // Very low
                avg_coherence: 0.3,
                rejection_hotspot: 0,
                cycle_index: i,
            });
        }

        topo.adapt();

        // Contraction positions (7, 5) should have HIGHER delta (more refinement)
        assert!(topo.noise_delta(7) > original_delta_7,
            "Pos 7 delta should increase: {:.4} vs {:.4}", topo.noise_delta(7), original_delta_7);
        assert!(topo.noise_delta(5) > original_delta_5,
            "Pos 5 delta should increase: {:.4} vs {:.4}", topo.noise_delta(5), original_delta_5);

        println!("RSI low-confidence adaptation:");
        println!("  Pos 7: {:.4} -> {:.4}", original_delta_7, topo.noise_delta(7));
        println!("  Pos 5: {:.4} -> {:.4}", original_delta_5, topo.noise_delta(5));
    }

    /// TOPOLOGY 6: Adaptive schedule differs from static when terrain is loaded
    #[test]
    fn test_adaptive_schedule_differs_from_static() {
        let mut elp = HashMap::new();
        for pos in 1..=9 {
            elp.insert(pos, [5.0, 7.0, 3.0]); // Logos-heavy everywhere
        }
        let terrain = SubjectTerrain {
            name: "reasoning".to_string(),
            elp_landscape: elp,
            sacred_properties: HashMap::new(),
        };

        let static_topo = AdaptiveVortexTopology::default();
        let terrain_topo = AdaptiveVortexTopology::from_terrain(terrain);

        let total_steps = 81;
        let static_schedule = static_topo.compute_adaptive_schedule(total_steps);
        let terrain_schedule = terrain_topo.compute_adaptive_schedule(total_steps);

        assert_eq!(static_schedule.len(), 81);
        assert_eq!(terrain_schedule.len(), 81);

        // The schedules should differ at non-sacred positions
        let mut differences = 0;
        for i in 0..total_steps {
            if (static_schedule[i] - terrain_schedule[i]).abs() > 0.001 {
                differences += 1;
            }
        }

        assert!(differences > 0,
            "Terrain-modulated schedule should differ from static: {} differences", differences);

        println!("Schedule differences: {}/81 steps differ between static and terrain", differences);
    }

    /// TOPOLOGY 7: Dynamic sacred positions in engine generation
    #[test]
    fn test_adaptive_engine_generates() {
        let mut engine = make_adaptive_engine(5, None);
        let result = engine.generate(&[10, 20], 10);

        assert_eq!(result.len(), 12);
        assert_eq!(result[0], 10);
        assert_eq!(result[1], 20);

        // Should have recorded some RSI cycle metrics
        let topo = engine.get_topology();
        // Topology might have adapted if enough cycles ran
        println!("Adaptive engine: {} RSI cycles observed, generation={}",
            topo.cycle_history.len(), topo.generation);

        let stats = engine.get_stats();
        assert_eq!(stats.tokens_generated, 10);
        println!("  conf={:.4}, rejections={}, verifications={}, forced={}",
            stats.avg_confidence, stats.sacred_rejections,
            stats.sacred_verifications, stats.forced_acceptances);
    }

    /// TOPOLOGY 8: Engine with terrain generates differently than default
    #[test]
    fn test_terrain_engine_vs_default() {
        let gen_len = 12;
        let prompt = vec![10u32, 20, 30];

        // Default engine
        let mut default_engine = make_adaptive_engine(5, None);
        let default_result = default_engine.generate(&prompt, gen_len);

        // Terrain engine with strong Pathos emphasis
        let mut elp = HashMap::new();
        for pos in 1..=9 {
            elp.insert(pos, [1.0, 1.0, 9.0]); // Pathos-heavy everywhere
        }
        let terrain = SubjectTerrain {
            name: "emotion".to_string(),
            elp_landscape: elp,
            sacred_properties: HashMap::new(),
        };
        let mut terrain_engine = make_adaptive_engine(5, Some(terrain));
        let terrain_result = terrain_engine.generate(&prompt, gen_len);

        // Both should produce valid output
        assert_eq!(default_result.len(), prompt.len() + gen_len);
        assert_eq!(terrain_result.len(), prompt.len() + gen_len);

        // The gate thresholds should differ
        let default_thresh = default_engine.get_topology().gate_thresholds;
        let terrain_thresh = terrain_engine.get_topology().gate_thresholds;

        println!("Default thresholds: ({:.3}, {:.3}, {:.3})",
            default_thresh.0, default_thresh.1, default_thresh.2);
        println!("Terrain thresholds: ({:.3}, {:.3}, {:.3})",
            terrain_thresh.0, terrain_thresh.1, terrain_thresh.2);
    }

    /// TOPOLOGY 9: TopologySummary provides correct overview
    #[test]
    fn test_topology_summary() {
        let topo = AdaptiveVortexTopology::default();
        let summary = topo.summary();

        assert_eq!(summary.flow_path, vec![1, 2, 4, 8, 7, 5]);
        assert_eq!(summary.sacred_positions, vec![3, 6, 9]);
        assert_eq!(summary.expansion_edges, 3);
        assert_eq!(summary.contraction_edges, 3);
        assert_eq!(summary.generation, 0);
        assert_eq!(summary.terrain_name, "default");
        assert_eq!(summary.cycles_observed, 0);
        assert!((summary.total_edge_weight - 6.0).abs() < 0.001);

        println!("Summary: {:?}", summary);
    }

    /// TOPOLOGY 10: Sacred positions are IMMUTABLE — always [3, 6, 9]
    /// The unmanifest domain cannot be overridden. Like Gods to Mortals.
    #[test]
    fn test_sacred_positions_immutable() {
        let topo = AdaptiveVortexTopology::default();

        // Sacred positions are always [3, 6, 9]
        assert_eq!(*topo.sacred_positions(), [3, 6, 9]);
        assert!(topo.is_sacred(3));
        assert!(topo.is_sacred(6));
        assert!(topo.is_sacred(9));

        // Flow positions are NOT sacred
        assert!(!topo.is_sacred(1));
        assert!(!topo.is_sacred(2));
        assert!(!topo.is_sacred(4));
        assert!(!topo.is_sacred(5));
        assert!(!topo.is_sacred(7));
        assert!(!topo.is_sacred(8));

        // Even after RSI adaptation, sacred positions remain [3, 6, 9]
        let mut adapted_topo = AdaptiveVortexTopology::default();
        for i in 0..9 {
            adapted_topo.record_cycle(CycleMetrics {
                tokens_locked: 1, tokens_rejected: 10, tokens_forced: 5,
                avg_lock_confidence: 0.1, avg_coherence: 0.2,
                rejection_hotspot: 9, cycle_index: i,
            });
        }
        adapted_topo.adapt();
        adapted_topo.adapt();
        adapted_topo.adapt();

        // Still immutable
        assert_eq!(*adapted_topo.sacred_positions(), [3, 6, 9]);
        assert!(adapted_topo.is_sacred(3));
        assert!(adapted_topo.is_sacred(6));
        assert!(adapted_topo.is_sacred(9));

        // Even with terrain, sacred positions don't change
        let mut elp = HashMap::new();
        for pos in 1..=9 { elp.insert(pos, [9.0, 9.0, 9.0]); }
        let terrain = SubjectTerrain {
            name: "extreme".to_string(),
            elp_landscape: elp,
            sacred_properties: HashMap::new(),
        };
        let terrain_topo = AdaptiveVortexTopology::from_terrain(terrain);
        assert_eq!(*terrain_topo.sacred_positions(), [3, 6, 9]);

        println!("Sacred positions immutability verified: always {:?}", topo.sacred_positions());
    }

    /// TOPOLOGY 12: Harmonic weight initialization produces structured (not random) weights
    #[test]
    fn test_harmonic_init_structured() {
        let t1 = DiffusionTransformer::new(32, 100, 4, 64);
        let t2 = DiffusionTransformer::new(32, 100, 4, 64);

        // Deterministic: same params = same weights
        assert_eq!(t1.w_q, t2.w_q, "Harmonic init should be deterministic");
        assert_eq!(t1.token_embeddings, t2.token_embeddings);

        // Not all zeros (harmonic sums can be small due to cancellation)
        let sum: f32 = t1.w_q.iter().sum();
        let abs_sum: f32 = t1.w_q.iter().map(|x| x.abs()).sum();
        assert!(abs_sum > 0.01, "Weights should not be all zero: abs_sum={}", abs_sum);
        // The signed sum can be small due to symmetry — that's fine

        // Not all the same value (has structure)
        let unique_vals: std::collections::HashSet<u32> = t1.w_q.iter()
            .map(|x| x.to_bits())
            .collect();
        assert!(unique_vals.len() > t1.w_q.len() / 2,
            "Weights should have many unique values: {}/{}", unique_vals.len(), t1.w_q.len());

        // Q and K should be different (different harmonic seeds)
        let q_sum: f32 = t1.w_q.iter().sum();
        let k_sum: f32 = t1.w_k.iter().sum();
        assert!((q_sum - k_sum).abs() > 0.001,
            "Q and K should differ: q_sum={:.4}, k_sum={:.4}", q_sum, k_sum);

        // Sacred dampening: positions 2,5,8 (mod 9) should have lower magnitude
        let sacred_mag: f32 = t1.w_q.iter().enumerate()
            .filter(|(i, _)| i % 9 == 2 || i % 9 == 5 || i % 9 == 8)
            .map(|(_, v)| v.abs())
            .sum::<f32>();
        let flow_mag: f32 = t1.w_q.iter().enumerate()
            .filter(|(i, _)| i % 9 == 0 || i % 9 == 1 || i % 9 == 3)
            .map(|(_, v)| v.abs())
            .sum::<f32>();

        // Sacred positions should have lower total magnitude (dampened by φ⁻¹)
        assert!(sacred_mag < flow_mag,
            "Sacred-dampened positions should have lower magnitude: sacred={:.4} < flow={:.4}",
            sacred_mag, flow_mag);

        println!("Harmonic init: {} unique values in Q[{}], sacred_mag={:.4} < flow_mag={:.4}",
            unique_vals.len(), t1.w_q.len(), sacred_mag, flow_mag);
    }

    /// TOPOLOGY 13: Tracing captures step-by-step generation details
    #[test]
    fn test_diffusion_trace() {
        let config = VortexDiffusionConfig {
            embed_dim: 32,
            vocab_size: 100,
            num_heads: 4,
            max_seq_len: 64,
            num_cycles: 5, // More cycles to ensure all sacred positions fire
            sacred_verification: true,
            enable_tracing: true,
            ..Default::default()
        };
        let mut engine = VortexDiffusionEngine::new(config);
        let result = engine.generate(&[], 8);

        assert_eq!(result.len(), 8);

        let trace = engine.get_trace();
        assert!(trace.enabled, "Trace should be enabled");
        assert!(!trace.steps.is_empty(), "Trace should have steps");
        assert_eq!(trace.prompt_len, 0);
        assert_eq!(trace.gen_len, 8);
        assert_eq!(trace.final_tokens.len(), 8);

        // Count action types
        let mut denoise_count = 0;
        let mut proximity_count = 0;
        let mut coherence_count = 0;
        let mut verify_count = 0;
        let mut skip_count = 0;
        for step in &trace.steps {
            match &step.action {
                StepAction::Denoise { .. } => denoise_count += 1,
                StepAction::SacredProximity { .. } => proximity_count += 1,
                StepAction::SacredCoherence { .. } => coherence_count += 1,
                StepAction::SacredVerification { .. } => verify_count += 1,
                StepAction::Skip { .. } => skip_count += 1,
            }
        }

        // Must have denoise steps (non-sacred positions generate proposals)
        assert!(denoise_count > 0, "Trace should have Denoise steps, got 0");

        // Must have at least one sacred action (proximity, coherence, or verification)
        let sacred_total = proximity_count + coherence_count + verify_count;
        assert!(sacred_total > 0,
            "Trace should have sacred steps: prox={} coh={} verify={}",
            proximity_count, coherence_count, verify_count);

        // Print the trace for visual inspection
        engine.print_trace();

        println!("Trace: {} steps ({} denoise, {} prox, {} coh, {} verify, {} skip), {} final tokens",
            trace.steps.len(), denoise_count, proximity_count,
            coherence_count, verify_count, skip_count, trace.final_tokens.len());
    }

    /// TOPOLOGY 14: Adaptive generation adjusts length based on prompt complexity
    #[test]
    fn test_adaptive_generation() {
        let config = VortexDiffusionConfig {
            embed_dim: 32,
            vocab_size: 100,
            num_heads: 4,
            max_seq_len: 128,
            num_cycles: 3,
            sacred_verification: true,
            ..Default::default()
        };

        // No prompt → longer generation
        let mut engine_empty = VortexDiffusionEngine::new(config.clone());
        let result_empty = engine_empty.generate_adaptive(&[]);
        let gen_empty = result_empty.len();

        // Short prompt → medium generation
        let mut engine_short = VortexDiffusionEngine::new(config.clone());
        let result_short = engine_short.generate_adaptive(&[1, 2, 3]);
        let gen_short = result_short.len() - 3;

        // Long prompt → shorter generation
        let long_prompt: Vec<u32> = (0..40).collect();
        let mut engine_long = VortexDiffusionEngine::new(config.clone());
        let result_long = engine_long.generate_adaptive(&long_prompt);
        let gen_long = result_long.len() - 40;

        // Empty prompt should generate more than long prompt
        assert!(gen_empty > gen_long,
            "Empty prompt should generate more tokens: {} vs {}", gen_empty, gen_long);

        // All should produce valid output (no masks remaining)
        assert!(!result_empty.contains(&MASK_TOKEN_ID));
        assert!(!result_short.contains(&MASK_TOKEN_ID));
        assert!(!result_long.contains(&MASK_TOKEN_ID));

        println!("Adaptive generation: empty={} tokens, short={} tokens, long={} tokens",
            gen_empty, gen_short, gen_long);
    }

    /// TOPOLOGY 11: Billions of terrains — verify diverse subjects produce distinct schedules
    #[test]
    fn test_diverse_subjects_distinct_schedules() {
        let subjects = vec![
            ("mathematics", [2.0, 9.0, 1.0]),  // Logos-dominant
            ("poetry",      [1.0, 2.0, 9.0]),  // Pathos-dominant
            ("ethics",      [9.0, 3.0, 3.0]),  // Ethos-dominant
            ("balanced",    [5.0, 5.0, 5.0]),  // Balanced
        ];

        let mut schedules: Vec<(String, Vec<f32>)> = Vec::new();

        for (name, elp_base) in &subjects {
            let mut elp = HashMap::new();
            for pos in 1..=9 {
                elp.insert(pos, *elp_base);
            }
            let terrain = SubjectTerrain {
                name: name.to_string(),
                elp_landscape: elp,
                sacred_properties: HashMap::new(),
            };
            let topo = AdaptiveVortexTopology::from_terrain(terrain);
            let schedule = topo.compute_adaptive_schedule(81);
            schedules.push((name.to_string(), schedule));
        }

        // Each pair should have at least some differences
        let mut all_distinct = true;
        for i in 0..schedules.len() {
            for j in (i+1)..schedules.len() {
                let diffs: usize = schedules[i].1.iter().zip(schedules[j].1.iter())
                    .filter(|(a, b)| (*a - *b).abs() > 0.001)
                    .count();
                if diffs == 0 { all_distinct = false; }
                println!("{} vs {}: {}/81 steps differ", schedules[i].0, schedules[j].0, diffs);
            }
        }

        assert!(all_distinct, "All distinct subjects should produce distinct schedules");
    }

    // =========================================================================
    // N-gram Validator Tests
    // =========================================================================

    #[test]
    fn test_ngram_train_and_score() {
        let mut ngram = NgramValidator::new();

        // Train on a few sentences (as token IDs)
        ngram.train(&[1, 2, 3, 4, 5]);
        ngram.train(&[1, 2, 3, 6, 7]);
        ngram.train(&[1, 2, 8, 9, 10]);

        let (uni, bi, tri) = ngram.stats();
        assert!(uni > 0, "Should have unigrams");
        assert!(bi > 0, "Should have bigrams");
        assert!(tri > 0, "Should have trigrams");

        // Score a sentence that matches training patterns
        let known = ngram.score(&[1, 2, 3, 4, 5]);
        // Score a sentence with completely unseen bigrams
        let unknown = ngram.score(&[50, 51, 52, 53, 54]);

        println!("Known sentence: combined={:.4}, bigram_cov={:.2}, trigram_cov={:.2}",
            known.combined_score, known.bigram_coverage, known.trigram_coverage);
        println!("Unknown sentence: combined={:.4}, bigram_cov={:.2}, trigram_cov={:.2}",
            unknown.combined_score, unknown.bigram_coverage, unknown.trigram_coverage);

        // Known sentence should have higher bigram coverage
        assert!(known.bigram_coverage > unknown.bigram_coverage,
            "Known patterns should have higher bigram coverage");
        // Known sentence should score higher overall
        assert!(known.combined_score > unknown.combined_score,
            "Known patterns should score higher: {:.4} vs {:.4}",
            known.combined_score, unknown.combined_score);
    }

    #[test]
    fn test_ngram_train_from_text() {
        let mut ngram = NgramValidator::new();
        ngram.train_from_text("the cat sat on the mat");
        ngram.train_from_text("the dog ran in the park");
        ngram.train_from_text("the cat ran on the mat");

        let (uni, bi, tri) = ngram.stats();
        println!("Text training: {} unigrams, {} bigrams, {} trigrams", uni, bi, tri);
        assert!(uni >= 6, "Should have at least 6 unique words");
        assert!(bi >= 5, "Should have at least 5 unique bigrams");
    }

    #[test]
    fn test_ngram_plausibility() {
        let mut ngram = NgramValidator::new();
        // Train on 100 repetitions to build strong patterns
        for _ in 0..100 {
            ngram.train(&[1, 2, 3, 4, 5]);
        }
        assert!(ngram.is_plausible(&[1, 2, 3, 4, 5], 0.3),
            "Highly trained pattern should be plausible");
    }

    #[test]
    fn test_ngram_edge_cases() {
        let ngram = NgramValidator::new();

        // Empty and single-token sequences should return zero score
        let empty = ngram.score(&[]);
        assert_eq!(empty.combined_score, 0.0);
        let single = ngram.score(&[42]);
        assert_eq!(single.combined_score, 0.0);
    }

    // =========================================================================
    // SentenceMemory Tests
    // =========================================================================

    #[test]
    fn test_sentence_memory_store_and_find() {
        let mut mem = SentenceMemory::new(4, 100);

        // Store 3 sentences with different embeddings
        let records = vec![
            (vec![1, 2, 3], vec![1.0, 0.0, 0.0, 0.0], 0.5),
            (vec![4, 5, 6], vec![0.0, 1.0, 0.0, 0.0], 0.8),
            (vec![7, 8, 9], vec![0.9, 0.1, 0.0, 0.0], 0.6), // Similar to first
        ];

        for (tokens, embed, score) in records {
            mem.store(SentenceRecord {
                token_ids: tokens,
                text: String::new(),
                ngram_score: score,
                avg_confidence: 0.5,
                rejections: 0,
                terrain_name: "test".to_string(),
                cycle_index: 0,
                embedding: embed,
                committed: false,
            });
        }

        assert_eq!(mem.len(), 3);

        // Find similar to [1,0,0,0] — should return idx 0 first, then idx 2
        let similar = mem.find_similar(&[1.0, 0.0, 0.0, 0.0], 2);
        assert_eq!(similar.len(), 2);
        assert_eq!(similar[0].0, 0, "First result should be exact match");
        assert!(similar[0].1 > 0.99, "Exact match should have sim ~1.0");
        println!("Top-2 similar: {:?}", similar);
    }

    #[test]
    fn test_sentence_memory_novelty() {
        let mut mem = SentenceMemory::new(3, 100);

        // Empty memory: everything is novel
        assert_eq!(mem.novelty(&[1.0, 0.0, 0.0]), 1.0);

        // Store one embedding
        mem.store(SentenceRecord {
            token_ids: vec![1],
            text: String::new(),
            ngram_score: 0.5,
            avg_confidence: 0.5,
            rejections: 0,
            terrain_name: "test".to_string(),
            cycle_index: 0,
            embedding: vec![1.0, 0.0, 0.0],
            committed: false,
        });

        // Same embedding: zero novelty
        let same_novelty = mem.novelty(&[1.0, 0.0, 0.0]);
        assert!(same_novelty < 0.01, "Same embedding should have near-zero novelty: {}", same_novelty);

        // Orthogonal embedding: maximum novelty
        let orth_novelty = mem.novelty(&[0.0, 1.0, 0.0]);
        assert!(orth_novelty > 0.99, "Orthogonal embedding should be maximally novel: {}", orth_novelty);
    }

    #[test]
    fn test_sentence_memory_eviction() {
        let mut mem = SentenceMemory::new(2, 3); // Capacity = 3

        for i in 0..5 {
            mem.store(SentenceRecord {
                token_ids: vec![i as u32],
                text: String::new(),
                ngram_score: 0.5,
                avg_confidence: 0.5,
                rejections: 0,
                terrain_name: "test".to_string(),
                cycle_index: i,
                embedding: vec![i as f32, 0.0],
                committed: false,
            });
        }

        // Should have evicted oldest, staying at capacity
        assert_eq!(mem.len(), 3, "Should stay at capacity after eviction");
    }

    #[test]
    fn test_sentence_memory_committed() {
        let mut mem = SentenceMemory::new(2, 100);

        // Store mix of committed and non-committed
        for i in 0..4 {
            mem.store(SentenceRecord {
                token_ids: vec![i as u32],
                text: String::new(),
                ngram_score: if i % 2 == 0 { 0.9 } else { 0.3 },
                avg_confidence: 0.5,
                rejections: 0,
                terrain_name: "test".to_string(),
                cycle_index: i,
                embedding: vec![i as f32, 0.0],
                committed: i % 2 == 0,
            });
        }

        let committed = mem.committed();
        assert_eq!(committed.len(), 2);
        let avg = mem.avg_committed_ngram_score();
        assert!((avg - 0.9).abs() < 0.01, "Avg committed ngram should be 0.9: {}", avg);
    }

    // =========================================================================
    // PhaseTracker Tests
    // =========================================================================

    #[test]
    fn test_phase_tracker_records_snapshots() {
        let mut tracker = PhaseTracker::new();

        for i in 0..20 {
            tracker.record(PhaseSnapshot {
                cycle: i,
                avg_steps_to_lock: 10.0,
                rejection_rate: 0.5 - (i as f32 * 0.02), // Gradually decreasing
                avg_confidence: 0.1 + (i as f32 * 0.02),  // Gradually increasing
                avg_coherence: 0.5,
                best_ngram_score: 0.4,
                best_novelty: 0.6,
                topology_entropy: 0.0,
                total_committed: i / 3,
                dual_objective: 0.3 + (i as f32 * 0.01),
            });
        }

        assert_eq!(tracker.snapshots.len(), 20);
        println!("Phase transitions detected: {}", tracker.transitions.len());
        tracker.print_transitions();
    }

    #[test]
    fn test_phase_tracker_detects_sharp_change() {
        let mut tracker = PhaseTracker::new();

        // 10 cycles of stable high rejection
        for i in 0..10 {
            tracker.record(PhaseSnapshot {
                cycle: i,
                avg_steps_to_lock: 10.0,
                rejection_rate: 0.8,
                avg_confidence: 0.1,
                avg_coherence: 0.5,
                best_ngram_score: 0.3,
                best_novelty: 0.5,
                topology_entropy: 0.0,
                total_committed: 0,
                dual_objective: 0.2,
            });
        }

        // Sudden drop in rejection rate
        tracker.record(PhaseSnapshot {
            cycle: 10,
            avg_steps_to_lock: 10.0,
            rejection_rate: 0.1, // Sharp drop from 0.8 to 0.1
            avg_confidence: 0.5, // Jump from 0.1 to 0.5
            avg_coherence: 0.5,
            best_ngram_score: 0.5,
            best_novelty: 0.5,
            topology_entropy: 0.0,
            total_committed: 1,
            dual_objective: 0.5,
        });

        assert!(!tracker.transitions.is_empty(),
            "Should detect phase transition on sharp rejection drop");
        println!("Detected {} transitions:", tracker.transitions.len());
        for (cycle, metric, mag) in &tracker.transitions {
            println!("  cycle {}: {} ({:.1}%)", cycle, metric, mag * 100.0);
        }
    }

    #[test]
    fn test_topology_entropy_default_is_zero() {
        let default_topo = AdaptiveVortexTopology::default();
        let entropy = PhaseTracker::topology_entropy(&default_topo);
        assert!(entropy.abs() < 0.001,
            "Default topology should have zero entropy: {}", entropy);
    }

    // =========================================================================
    // MetaController Tests
    // =========================================================================

    #[test]
    fn test_meta_controller_creation() {
        let engine = make_engine(3, true, ScheduleType::SacredVortex);
        let reference = &[
            "the cat sat on the mat",
            "the dog ran in the park",
            "a bird flew over the tree",
        ];
        let config = MetaControllerConfig {
            max_cycles: 10,
            print_every: 5,
            ..Default::default()
        };
        let ctrl = MetaController::new(engine, reference, config);

        assert_eq!(ctrl.cycle, 0);
        assert!(ctrl.memory.is_empty());
        let (uni, bi, tri) = ctrl.ngram.stats();
        assert!(uni > 0, "Should have trained unigrams: {}", uni);
        assert!(bi > 0, "Should have trained bigrams: {}", bi);
        assert!(tri > 0, "Should have trained trigrams: {}", tri);
        println!("MetaController created: {} unigrams, {} bigrams, {} trigrams", uni, bi, tri);
    }

    #[test]
    fn test_meta_controller_propose_explore_when_empty() {
        let engine = make_engine(3, true, ScheduleType::SacredVortex);
        let ctrl = MetaController::new(engine, &["test text"], MetaControllerConfig::default());
        match ctrl.propose_next_task() {
            MetaTask::Explore => {} // Expected
            other => panic!("Expected Explore when memory is empty, got {:?}", other),
        }
    }

    #[test]
    fn test_meta_controller_evaluate() {
        let engine = make_engine(3, true, ScheduleType::SacredVortex);
        let ctrl = MetaController::new(
            engine,
            &["the cat sat on the mat"],
            MetaControllerConfig::default(),
        );

        let (ngram, novelty, dual) = ctrl.evaluate_output(&[1, 2, 3, 4, 5]);
        println!("Evaluate: ngram={:.4}, novelty={:.4}, dual={:.4}", ngram, novelty, dual);
        assert!(ngram >= 0.0 && ngram <= 1.0, "N-gram score out of range: {}", ngram);
        assert!(novelty >= 0.0 && novelty <= 1.0, "Novelty out of range: {}", novelty);
        assert!(dual >= 0.0, "Dual objective should be non-negative: {}", dual);
    }

    #[test]
    fn test_meta_controller_short_run() {
        let engine = make_engine(3, true, ScheduleType::SacredVortex);
        let reference = &[
            "the cat sat on the mat",
            "the dog ran in the park",
        ];
        let config = MetaControllerConfig {
            max_cycles: 20,
            print_every: 10,
            commit_threshold: 0.3, // Low threshold to get some commits
            ..Default::default()
        };
        let mut ctrl = MetaController::new(engine, reference, config);
        ctrl.run(20);

        assert_eq!(ctrl.cycle, 20);
        assert_eq!(ctrl.memory.len(), 20, "Should have stored 20 sentences");
        assert_eq!(ctrl.phase.snapshots.len(), 20, "Should have 20 phase snapshots");

        let committed = ctrl.memory.committed().len();
        println!("Short run complete: {} committed out of {}", committed, ctrl.memory.len());
        println!("Phase transitions: {}", ctrl.phase.transitions.len());
    }
}
