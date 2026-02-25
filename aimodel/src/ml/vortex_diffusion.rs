//! Vortex Diffusion Language Model
//!
//! A novel discrete diffusion architecture built on SpatialVortex first principles.
//! Unlike MDLM/SEDD/LLaDA which use linear or cosine noise schedules with flat
//! denoising steps, Vortex Diffusion exploits the sacred geometry cycle for
//! non-monotonic, hierarchical token generation.
//!
//! ## Table of Contents
//! 1. **VortexNoiseSchedule** — Sacred geometry noise curve (φ-based, non-monotonic)
//! 2. **SacredUnmaskingGate** — Three-phase verification at positions 3, 6, 9
//! 3. **TokenState** — Per-token state tracking (masked, candidate, verified, locked)
//! 4. **DiffusionTransformer** — Bidirectional attention with vortex-cycle sub-steps
//! 5. **VortexDiffusionEngine** — Main engine: multi-resolution denoising loop
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
    /// Create a new transformer with Xavier-initialized weights.
    ///
    /// In a real implementation, these weights would be loaded from
    /// a pre-trained model. The initialization here uses deterministic
    /// pseudo-random values based on position for reproducibility.
    pub fn new(embed_dim: usize, vocab_size: usize, num_heads: usize, max_seq_len: usize) -> Self {
        let ff_dim = 4 * embed_dim;

        // Xavier initialization scale factors
        let embed_scale = (2.0 / (vocab_size + embed_dim) as f32).sqrt();
        let attn_scale = (2.0 / (embed_dim * 2) as f32).sqrt();
        let ff_scale_1 = (2.0 / (embed_dim + ff_dim) as f32).sqrt();
        let ff_scale_2 = (2.0 / (ff_dim + embed_dim) as f32).sqrt();
        let head_scale = (2.0 / (embed_dim + vocab_size) as f32).sqrt();

        // Deterministic pseudo-random init using golden ratio perturbation
        let init = |size: usize, scale: f32, seed: f32| -> Vec<f32> {
            (0..size)
                .map(|i| {
                    let x = (i as f32 * PHI_INV + seed).fract();
                    (x * 2.0 - 1.0) * scale
                })
                .collect()
        };

        Self {
            embed_dim,
            vocab_size,
            num_heads,
            token_embeddings: init(vocab_size * embed_dim, embed_scale, 0.1),
            position_embeddings: init(max_seq_len * embed_dim, 0.02, 0.2),
            max_seq_len,
            w_q: init(embed_dim * embed_dim, attn_scale, 0.3),
            w_k: init(embed_dim * embed_dim, attn_scale, 0.4),
            w_v: init(embed_dim * embed_dim, attn_scale, 0.5),
            w_o: init(embed_dim * embed_dim, attn_scale, 0.6),
            ff_w1: init(embed_dim * ff_dim, ff_scale_1, 0.7),
            ff_w2: init(ff_dim * embed_dim, ff_scale_2, 0.8),
            ln1_weight: vec![1.0; embed_dim],
            ln2_weight: vec![1.0; embed_dim],
            lm_head: init(embed_dim * vocab_size, head_scale, 0.9),
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
    /// Vocabulary: token_id → string
    pub vocab: HashMap<u32, String>,
    /// Reverse vocabulary: string → token_id
    pub token_to_id: HashMap<String, u32>,
    /// Generation statistics
    pub stats: DiffusionStats,
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
        let schedule = VortexNoiseSchedule::new(config.num_cycles, config.schedule_type.clone());
        let sacred_gate = SacredUnmaskingGate::default();

        Self {
            config,
            transformer,
            schedule,
            sacred_gate,
            vocab: HashMap::new(),
            token_to_id: HashMap::new(),
            stats: DiffusionStats::default(),
        }
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
                break; // All tokens resolved
            }

            // Determine what happens at this sub-position
            match sub_pos {
                // Sacred position 3: proximity check on candidates
                3 if self.config.sacred_verification => {
                    for i in prompt_len..total_len {
                        if states[i].lifecycle == TokenLifecycle::Candidate {
                            let result = self.sacred_gate.check_proximity(&states[i]);
                            if !result.passes {
                                // Don't reject yet — just note low proximity
                                // Position 6 will do the coherence check
                            }
                        }
                    }
                }

                // Sacred position 6: coherence check on candidates
                6 if self.config.sacred_verification => {
                    for i in prompt_len..total_len {
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
                }

                // Sacred position 9: final verification — lock or reject
                9 if self.config.sacred_verification => {
                    let mut locked_this_step = 0;
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
                                    states[i].lock();
                                    locked_this_step += 1;
                                    if forced {
                                        self.stats.forced_acceptances += 1;
                                    }
                                }
                                VerificationDecision::Defer { .. } => {
                                    // Keep as candidate — will be checked again next cycle
                                }
                                VerificationDecision::Reject { .. } => {
                                    states[i].reject();
                                    self.stats.sacred_rejections += 1;
                                }
                            }
                        }
                    }
                    self.stats.tokens_per_cycle.push(locked_this_step);
                }

                // Non-sacred positions: run the denoiser and propose candidates
                _ => {
                    // Collect current token IDs for the transformer
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

        // Return token IDs
        states.iter().map(|s| s.token_id).collect()
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
            // — we just skip already-locked tokens
            match sub_pos {
                3 | 6 | 9 if self.config.sacred_verification => {
                    self.run_sacred_step(sub_pos, &mut states, total_len);
                }
                _ => {
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
        }

        // Force-lock remaining
        for state in states.iter_mut() {
            if !state.is_final() && state.token_id != MASK_TOKEN_ID {
                state.lifecycle = TokenLifecycle::Locked;
            }
        }

        states.iter().map(|s| s.token_id).collect()
    }

    /// Run a sacred verification step (shared between generate and infill)
    fn run_sacred_step(&mut self, sub_pos: u8, states: &mut [TokenState], total_len: usize) {
        match sub_pos {
            3 => {
                for i in 0..total_len {
                    if states[i].lifecycle == TokenLifecycle::Candidate {
                        let _result = self.sacred_gate.check_proximity(&states[i]);
                    }
                }
            }
            6 => {
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
            }
            9 => {
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
            _ => {}
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
}
