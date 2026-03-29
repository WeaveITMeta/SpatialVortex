//! Grid Encoder — Hybrid CNN-ViT for ARC-AGI-3 grids
//!
//! Processes 2D integer grids (up to 64×64, 10 discrete cell colours) into
//! 256-dim embeddings compatible with the unified latent space.
//!
//! Architecture:
//!   CellEmbedding(10→256) + Sinusoidal2DPosEnc
//!   → Conv2DBlock(3×3) × 2          (local pattern extraction)
//!   → PatchMerge(4×4 → 1)           (spatial dimensionality reduction)
//!   → TransformerBlock × 2           (global spatial attention)
//!   → FluxPositionMapping            (digital_root → vortex 1–9)
//!   → SacredGateVerification(3→6→9) (coherence gating)
//!   → MeanPool → 256-dim output
//!
//! Integrates with:
//!   - VortexDiffusion (spatial transform generation via denoising)
//!   - TransitiveFluxReasoner (spatial_coords + relations)
//!   - UnifiedLatent (256-dim embedding passthrough)

use std::collections::HashMap;

// ─── Constants ───────────────────────────────────────────────────────────────

pub const EMBED_DIM: usize = 256;
pub const NUM_COLORS: usize = 10; // ARC grid colours 0–9
pub const MAX_GRID: usize = 64; // ARC-AGI-3 upper bound
pub const PATCH_SIZE: usize = 4; // merge 4×4 cells into one token
pub const NUM_HEADS: usize = 8;
pub const HEAD_DIM: usize = EMBED_DIM / NUM_HEADS; // 32
pub const FF_DIM: usize = EMBED_DIM * 4; // 1024

const PHI: f32 = 1.618_033_9;
const PHI_INV: f32 = 0.618_034;
const VORTEX_CYCLE: [u8; 6] = [1, 2, 4, 8, 7, 5];
const SACRED_POSITIONS: [u8; 3] = [3, 6, 9];

// ─── Grid Representation ─────────────────────────────────────────────────────

/// Raw ARC grid — 2D array of cell colours (0–9).
#[derive(Debug, Clone)]
pub struct ArcGrid {
    pub cells: Vec<Vec<u8>>,
    pub height: usize,
    pub width: usize,
}

impl ArcGrid {
    /// Parse a grid from JSON (array of arrays of integers).
    pub fn from_json(value: &serde_json::Value) -> Option<Self> {
        let rows = value.as_array()?;
        let cells: Vec<Vec<u8>> = rows
            .iter()
            .map(|row| {
                row.as_array()
                    .map(|r| {
                        r.iter()
                            .map(|v| v.as_u64().unwrap_or(0) as u8)
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .collect();
        let height = cells.len();
        let width = cells.first().map_or(0, |r| r.len());
        Some(Self {
            cells,
            height,
            width,
        })
    }

    /// Flat index → cell value.
    #[inline]
    pub fn at(&self, row: usize, col: usize) -> u8 {
        self.cells.get(row).and_then(|r| r.get(col)).copied().unwrap_or(0)
    }
}

// ─── Encoder Configuration ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GridEncoderConfig {
    pub embed_dim: usize,
    pub num_colors: usize,
    pub patch_size: usize,
    pub num_heads: usize,
    pub num_attn_layers: usize,
    pub num_conv_layers: usize,
    /// Flux-weighted sacred gate thresholds (proximity, coherence, verification).
    pub gate_thresholds: (f32, f32, f32),
}

impl Default for GridEncoderConfig {
    fn default() -> Self {
        Self {
            embed_dim: EMBED_DIM,
            num_colors: NUM_COLORS,
            patch_size: PATCH_SIZE,
            num_heads: NUM_HEADS,
            num_attn_layers: 2,
            num_conv_layers: 2,
            gate_thresholds: (0.15, 0.30, 0.40),
        }
    }
}

// ─── Cell Embedding ──────────────────────────────────────────────────────────

/// Learned embedding table: colour ID (0–9) → 256-dim vector.
/// Initialised with flux-position-aware sinusoidal seeds so that colours
/// mapping to sacred vortex positions start with higher-magnitude features.
#[derive(Debug, Clone)]
pub struct CellEmbedding {
    /// [num_colors × embed_dim]
    pub weights: Vec<f32>,
    pub embed_dim: usize,
    pub num_colors: usize,
}

impl CellEmbedding {
    pub fn new(num_colors: usize, embed_dim: usize) -> Self {
        let mut weights = vec![0.0f32; num_colors * embed_dim];
        for colour in 0..num_colors {
            let vortex_pos = digital_root(colour as u64);
            let sacred_boost = if SACRED_POSITIONS.contains(&vortex_pos) {
                PHI
            } else {
                1.0
            };
            for d in 0..embed_dim {
                let freq = 1.0 / (10000.0f32.powf(d as f32 / embed_dim as f32));
                let val = if d % 2 == 0 {
                    ((colour as f32) * freq).sin()
                } else {
                    ((colour as f32) * freq).cos()
                };
                weights[colour * embed_dim + d] = val * sacred_boost * 0.1;
            }
        }
        Self {
            weights,
            embed_dim,
            num_colors,
        }
    }

    /// Look up embedding for a single colour ID.
    #[inline]
    pub fn embed(&self, colour: u8) -> &[f32] {
        let c = (colour as usize).min(self.num_colors - 1);
        &self.weights[c * self.embed_dim..(c + 1) * self.embed_dim]
    }
}

// ─── Sinusoidal 2D Position Encoding ─────────────────────────────────────────

/// Generates a 256-dim vector for grid position (row, col) using interleaved
/// sine/cosine over row and column dimensions.
pub fn sinusoidal_2d_pos(row: usize, col: usize, embed_dim: usize) -> Vec<f32> {
    let half = embed_dim / 2;
    let mut enc = vec![0.0f32; embed_dim];
    // First half: row encoding
    for i in 0..half {
        let freq = 1.0 / (10000.0f32.powf(i as f32 / half as f32));
        if i % 2 == 0 {
            enc[i] = (row as f32 * freq).sin();
        } else {
            enc[i] = (row as f32 * freq).cos();
        }
    }
    // Second half: column encoding
    for i in 0..half {
        let freq = 1.0 / (10000.0f32.powf(i as f32 / half as f32));
        if i % 2 == 0 {
            enc[half + i] = (col as f32 * freq).sin();
        } else {
            enc[half + i] = (col as f32 * freq).cos();
        }
    }
    enc
}

// ─── Conv2D Block ────────────────────────────────────────────────────────────

/// Single Conv2D layer: 3×3 kernel, stride 1, padding 1 (output same size).
/// Channels: embed_dim → embed_dim.
#[derive(Debug, Clone)]
pub struct Conv2DBlock {
    /// [C_out × C_in × 3 × 3]
    pub kernel: Vec<f32>,
    /// [C_out]
    pub bias: Vec<f32>,
    pub channels: usize,
}

impl Conv2DBlock {
    pub fn new(channels: usize) -> Self {
        // Kaiming-uniform initialisation scaled by vortex PHI
        let k = 1.0 / (channels as f32 * 9.0).sqrt() * PHI_INV;
        let n = channels * channels * 9;
        let mut kernel = Vec::with_capacity(n);
        let mut state: u64 = 0xDEAD_BEEF_CAFE;
        for _ in 0..n {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u = (state >> 33) as f32 / (u32::MAX as f32);
            kernel.push((u * 2.0 - 1.0) * k);
        }
        let bias = vec![0.0f32; channels];
        Self {
            kernel,
            bias,
            channels,
        }
    }

    /// Forward: [H × W × C] → [H × W × C] with zero-padding, GELU activation.
    pub fn forward(&self, input: &[f32], h: usize, w: usize) -> Vec<f32> {
        let c = self.channels;
        let mut output = vec![0.0f32; h * w * c];

        for out_c in 0..c {
            for row in 0..h {
                for col in 0..w {
                    let mut sum = self.bias[out_c];
                    for in_c in 0..c {
                        for kr in 0..3usize {
                            for kc in 0..3usize {
                                let r = row as isize + kr as isize - 1;
                                let cc = col as isize + kc as isize - 1;
                                if r >= 0 && r < h as isize && cc >= 0 && cc < w as isize {
                                    let in_idx =
                                        r as usize * w * c + cc as usize * c + in_c;
                                    let k_idx =
                                        out_c * c * 9 + in_c * 9 + kr * 3 + kc;
                                    sum += input[in_idx] * self.kernel[k_idx];
                                }
                            }
                        }
                    }
                    // GELU activation
                    output[row * w * c + col * c + out_c] = gelu(sum);
                }
            }
        }
        output
    }
}

// ─── Patch Merge ─────────────────────────────────────────────────────────────

/// Merge patch_size × patch_size cells into a single token by concatenating
/// their embeddings and projecting back to embed_dim.
///
/// Input:  [H × W × C]       (C = embed_dim)
/// Output: [(H/ps) × (W/ps)] tokens × embed_dim
#[derive(Debug, Clone)]
pub struct PatchMerge {
    /// [ps*ps*C → C]
    pub projection: Vec<f32>,
    pub bias: Vec<f32>,
    pub patch_size: usize,
    pub embed_dim: usize,
}

impl PatchMerge {
    pub fn new(patch_size: usize, embed_dim: usize) -> Self {
        let fan_in = patch_size * patch_size * embed_dim;
        let scale = 1.0 / (fan_in as f32).sqrt();
        let mut state: u64 = 0xCAFE_BABE;
        let projection: Vec<f32> = (0..fan_in * embed_dim)
            .map(|_| {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let u = (state >> 33) as f32 / (u32::MAX as f32);
                (u * 2.0 - 1.0) * scale
            })
            .collect();
        let bias = vec![0.0f32; embed_dim];
        Self {
            projection,
            bias,
            patch_size,
            embed_dim,
        }
    }

    /// Returns (output_tokens, out_h, out_w).
    pub fn forward(&self, input: &[f32], h: usize, w: usize) -> (Vec<f32>, usize, usize) {
        let ps = self.patch_size;
        let c = self.embed_dim;
        let oh = (h + ps - 1) / ps; // ceil division
        let ow = (w + ps - 1) / ps;
        let fan_in = ps * ps * c;
        let mut output = vec![0.0f32; oh * ow * c];

        for pr in 0..oh {
            for pc in 0..ow {
                // Gather patch into a flat buffer
                let mut patch = vec![0.0f32; fan_in];
                for dr in 0..ps {
                    for dc in 0..ps {
                        let r = pr * ps + dr;
                        let cc = pc * ps + dc;
                        if r < h && cc < w {
                            let src = r * w * c + cc * c;
                            let dst = (dr * ps + dc) * c;
                            patch[dst..dst + c].copy_from_slice(&input[src..src + c]);
                        }
                    }
                }
                // Linear projection: fan_in → embed_dim
                let out_base = (pr * ow + pc) * c;
                for od in 0..c {
                    let mut sum = self.bias[od];
                    for id in 0..fan_in {
                        sum += patch[id] * self.projection[id * c + od];
                    }
                    output[out_base + od] = sum;
                }
            }
        }
        (output, oh, ow)
    }
}

// ─── Self-Attention Block ────────────────────────────────────────────────────

/// Multi-head self-attention + feed-forward, matching DiffusionTransformer style.
/// Pre-norm with layer normalisation.
#[derive(Debug, Clone)]
pub struct AttentionBlock {
    // Attention weights [embed_dim × embed_dim] each
    pub w_q: Vec<f32>,
    pub w_k: Vec<f32>,
    pub w_v: Vec<f32>,
    pub w_o: Vec<f32>,
    // Feed-forward [embed_dim × ff_dim], [ff_dim × embed_dim]
    pub ff_w1: Vec<f32>,
    pub ff_w2: Vec<f32>,
    pub ff_b1: Vec<f32>,
    pub ff_b2: Vec<f32>,
    pub embed_dim: usize,
    pub num_heads: usize,
}

impl AttentionBlock {
    pub fn new(embed_dim: usize, num_heads: usize) -> Self {
        let dim2 = embed_dim * embed_dim;
        let ff_dim = embed_dim * 4;
        let scale_attn = 1.0 / (embed_dim as f32).sqrt();
        let scale_ff = 1.0 / (ff_dim as f32).sqrt();

        let mut state: u64 = 0x1234_5678_ABCD;
        let mut rand_vec = |n: usize, s: f32| -> Vec<f32> {
            (0..n)
                .map(|_| {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let u = (state >> 33) as f32 / (u32::MAX as f32);
                    (u * 2.0 - 1.0) * s
                })
                .collect()
        };

        Self {
            w_q: rand_vec(dim2, scale_attn),
            w_k: rand_vec(dim2, scale_attn),
            w_v: rand_vec(dim2, scale_attn),
            w_o: rand_vec(dim2, scale_attn),
            ff_w1: rand_vec(embed_dim * ff_dim, scale_ff),
            ff_w2: rand_vec(ff_dim * embed_dim, scale_ff),
            ff_b1: vec![0.0; ff_dim],
            ff_b2: vec![0.0; embed_dim],
            embed_dim,
            num_heads,
        }
    }

    /// Forward: [seq_len × embed_dim] → [seq_len × embed_dim].
    pub fn forward(&self, tokens: &[f32], seq_len: usize) -> Vec<f32> {
        let d = self.embed_dim;
        let hd = d / self.num_heads;
        let ff = d * 4;

        // ── Layer norm + multi-head attention ────────────────────────────
        let normed = layer_norm(tokens, seq_len, d);

        // Q, K, V projections
        let q = matmul_2d(&normed, &self.w_q, seq_len, d, d);
        let k = matmul_2d(&normed, &self.w_k, seq_len, d, d);
        let v = matmul_2d(&normed, &self.w_v, seq_len, d, d);

        // Scaled dot-product attention per head
        let scale = 1.0 / (hd as f32).sqrt();
        let mut attn_out = vec![0.0f32; seq_len * d];

        for h in 0..self.num_heads {
            let off = h * hd;
            // Compute attention scores
            let mut scores = vec![0.0f32; seq_len * seq_len];
            for i in 0..seq_len {
                for j in 0..seq_len {
                    let mut dot = 0.0f32;
                    for dd in 0..hd {
                        dot += q[i * d + off + dd] * k[j * d + off + dd];
                    }
                    scores[i * seq_len + j] = dot * scale;
                }
                // Softmax row i
                softmax_inplace(&mut scores[i * seq_len..(i + 1) * seq_len]);
            }
            // Weighted sum of V
            for i in 0..seq_len {
                for dd in 0..hd {
                    let mut sum = 0.0f32;
                    for j in 0..seq_len {
                        sum += scores[i * seq_len + j] * v[j * d + off + dd];
                    }
                    attn_out[i * d + off + dd] = sum;
                }
            }
        }

        // Output projection
        let projected = matmul_2d(&attn_out, &self.w_o, seq_len, d, d);

        // Residual connection
        let mut residual: Vec<f32> = tokens
            .iter()
            .zip(projected.iter())
            .map(|(a, b)| a + b)
            .collect();

        // ── Layer norm + feed-forward ────────────────────────────────────
        let normed2 = layer_norm(&residual, seq_len, d);

        // FF layer 1: d → ff with GELU
        let mut hidden = vec![0.0f32; seq_len * ff];
        for i in 0..seq_len {
            for f in 0..ff {
                let mut sum = self.ff_b1[f];
                for dd in 0..d {
                    sum += normed2[i * d + dd] * self.ff_w1[dd * ff + f];
                }
                hidden[i * ff + f] = gelu(sum);
            }
        }

        // FF layer 2: ff → d
        for i in 0..seq_len {
            for dd in 0..d {
                let mut sum = self.ff_b2[dd];
                for f in 0..ff {
                    sum += hidden[i * ff + f] * self.ff_w2[f * d + dd];
                }
                residual[i * d + dd] += sum;
            }
        }

        residual
    }
}

// ─── Flux Position Mapping ───────────────────────────────────────────────────

/// Annotates each patch token with its vortex position (1–9) derived from
/// the digital root of (patch_row * grid_width + patch_col + 1).
///
/// Sacred positions (3, 6, 9) receive boosted weights.
#[derive(Debug, Clone)]
pub struct FluxPositionMap {
    /// Per-token vortex position (1–9).
    pub positions: Vec<u8>,
    /// Per-token sacred boost factor (PHI for sacred, 1.0 otherwise).
    pub boosts: Vec<f32>,
}

impl FluxPositionMap {
    pub fn build(num_patches_h: usize, num_patches_w: usize) -> Self {
        let n = num_patches_h * num_patches_w;
        let mut positions = Vec::with_capacity(n);
        let mut boosts = Vec::with_capacity(n);
        for pr in 0..num_patches_h {
            for pc in 0..num_patches_w {
                let linear = pr * num_patches_w + pc + 1; // 1-based
                let vp = digital_root(linear as u64);
                let boost = match vp {
                    3 => 1.15, // Unity
                    6 => 1.10, // Heart
                    9 => 1.20, // Ultimate
                    _ => 1.0,
                };
                positions.push(vp);
                boosts.push(boost);
            }
        }
        Self { positions, boosts }
    }

    /// Apply sacred weighting to token embeddings in-place.
    pub fn apply_boosts(&self, tokens: &mut [f32], embed_dim: usize) {
        for (i, &boost) in self.boosts.iter().enumerate() {
            if (boost - 1.0).abs() > f32::EPSILON {
                let base = i * embed_dim;
                for d in 0..embed_dim {
                    if base + d < tokens.len() {
                        tokens[base + d] *= boost;
                    }
                }
            }
        }
    }
}

// ─── Sacred Gate Verification ────────────────────────────────────────────────

/// Three-phase gate matching VortexDiffusion's SacredUnmaskingGate:
///   Position 3: Proximity check  (is the embedding well-formed?)
///   Position 6: Coherence check  (does it cohere with neighbours?)
///   Position 9: Final verdict    (φ-weighted composite)
#[derive(Debug, Clone)]
pub struct SacredGateResult {
    pub proximity_score: f32,
    pub coherence_score: f32,
    pub composite_score: f32,
    pub accepted: bool,
}

pub fn sacred_gate_verify(
    embedding: &[f32],
    context_embeddings: &[&[f32]],
    thresholds: (f32, f32, f32),
) -> SacredGateResult {
    let d = embedding.len();

    // ── Position 3: Proximity (magnitude check) ─────────────────────────
    let magnitude = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    let proximity_score = (magnitude / (d as f32).sqrt()).min(1.0);
    let prox_pass = proximity_score >= thresholds.0;

    // ── Position 6: Coherence (avg cosine with context) ─────────────────
    let coherence_score = if context_embeddings.is_empty() {
        1.0 // no context → assume coherent
    } else {
        let sum: f32 = context_embeddings
            .iter()
            .map(|ctx| cosine_sim(embedding, ctx))
            .sum();
        (sum / context_embeddings.len() as f32 + 1.0) / 2.0 // normalise to [0,1]
    };
    let coher_pass = coherence_score >= thresholds.1;

    // ── Position 9: φ-weighted verdict ──────────────────────────────────
    let composite = PHI_INV * proximity_score + (1.0 - PHI_INV) * coherence_score;
    let accepted = composite >= thresholds.2 || (prox_pass && coher_pass);

    SacredGateResult {
        proximity_score,
        coherence_score,
        composite_score: composite,
        accepted,
    }
}

// ─── Spatial Relation Extraction ─────────────────────────────────────────────

/// Extract spatial relations between salient patches (non-background colour)
/// for feeding into TransitiveFluxReasoner.
pub fn extract_spatial_relations(
    grid: &ArcGrid,
    patch_size: usize,
) -> Vec<(String, String, String)> {
    // Identify non-zero patch centres
    let ph = (grid.height + patch_size - 1) / patch_size;
    let pw = (grid.width + patch_size - 1) / patch_size;

    let mut salient: Vec<(usize, usize, u8)> = Vec::new(); // (pr, pc, dominant_colour)
    for pr in 0..ph {
        for pc in 0..pw {
            let mut counts = [0u32; NUM_COLORS];
            for dr in 0..patch_size {
                for dc in 0..patch_size {
                    let r = pr * patch_size + dr;
                    let c = pc * patch_size + dc;
                    if r < grid.height && c < grid.width {
                        let v = (grid.at(r, c) as usize).min(NUM_COLORS - 1);
                        counts[v] += 1;
                    }
                }
            }
            // Find dominant non-zero colour
            let dominant = counts
                .iter()
                .enumerate()
                .skip(1) // skip colour 0 (background)
                .max_by_key(|(_, &ct)| ct)
                .filter(|(_, &ct)| ct > 0)
                .map(|(c, _)| c as u8);
            if let Some(colour) = dominant {
                salient.push((pr, pc, colour));
            }
        }
    }

    // Generate spatial relation triples
    let mut relations = Vec::new();
    for (i, &(r1, c1, col1)) in salient.iter().enumerate() {
        let ent_a = format!("patch_{r1}_{c1}_c{col1}");
        for &(r2, c2, col2) in &salient[i + 1..] {
            let ent_b = format!("patch_{r2}_{c2}_c{col2}");
            if c1 < c2 {
                relations.push((ent_a.clone(), "left_of".into(), ent_b.clone()));
            } else if c1 > c2 {
                relations.push((ent_a.clone(), "right_of".into(), ent_b.clone()));
            }
            if r1 < r2 {
                relations.push((ent_a.clone(), "above".into(), ent_b.clone()));
            } else if r1 > r2 {
                relations.push((ent_a.clone(), "below".into(), ent_b.clone()));
            }
            if col1 == col2 {
                relations.push((ent_a.clone(), "same_colour".into(), ent_b.clone()));
            }
        }
    }
    relations
}

// ─── Main Grid Encoder ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GridEncoder {
    pub config: GridEncoderConfig,
    pub cell_embed: CellEmbedding,
    pub conv_layers: Vec<Conv2DBlock>,
    pub patch_merge: PatchMerge,
    pub attn_layers: Vec<AttentionBlock>,
}

/// Encoded grid output — ready for unified latent or policy input.
#[derive(Debug, Clone)]
pub struct EncodedGrid {
    /// Global 256-dim embedding (mean-pooled, sacred-gated).
    pub embedding: Vec<f32>,
    /// Per-patch 256-dim embeddings, shape [num_patches × embed_dim].
    pub patch_embeddings: Vec<f32>,
    pub num_patches_h: usize,
    pub num_patches_w: usize,
    /// Vortex position per patch (1–9).
    pub flux_positions: Vec<u8>,
    /// Extracted spatial relations: (entity_a, relation, entity_b).
    pub spatial_relations: Vec<(String, String, String)>,
    /// Sacred gate result for the pooled embedding.
    pub gate_result: SacredGateResult,
}

impl GridEncoder {
    pub fn new(config: GridEncoderConfig) -> Self {
        let cell_embed = CellEmbedding::new(config.num_colors, config.embed_dim);
        let conv_layers = (0..config.num_conv_layers)
            .map(|_| Conv2DBlock::new(config.embed_dim))
            .collect();
        let patch_merge = PatchMerge::new(config.patch_size, config.embed_dim);
        let attn_layers = (0..config.num_attn_layers)
            .map(|_| AttentionBlock::new(config.embed_dim, config.num_heads))
            .collect();
        Self {
            config,
            cell_embed,
            conv_layers,
            patch_merge,
            attn_layers,
        }
    }

    /// Encode an ARC grid into the unified latent space.
    pub fn encode(&self, grid: &ArcGrid) -> EncodedGrid {
        let h = grid.height;
        let w = grid.width;
        let d = self.config.embed_dim;

        // ── Step 1: Cell embedding + 2D positional encoding ─────────────
        let mut features = vec![0.0f32; h * w * d];
        for r in 0..h {
            for c in 0..w {
                let colour = grid.at(r, c);
                let cell_vec = self.cell_embed.embed(colour);
                let pos_vec = sinusoidal_2d_pos(r, c, d);
                let base = (r * w + c) * d;
                for dd in 0..d {
                    features[base + dd] = cell_vec[dd] + pos_vec[dd];
                }
            }
        }

        // ── Step 2: Conv2D blocks (local pattern extraction) ────────────
        let mut conv_out = features;
        for conv in &self.conv_layers {
            conv_out = conv.forward(&conv_out, h, w);
        }

        // ── Step 3: Patch merge (spatial reduction) ─────────────────────
        let (merged, ph, pw) = self.patch_merge.forward(&conv_out, h, w);
        let num_patches = ph * pw;

        // ── Step 4: Self-attention layers (global reasoning) ────────────
        let mut tokens = merged;
        for attn in &self.attn_layers {
            tokens = attn.forward(&tokens, num_patches);
        }

        // ── Step 5: Flux position mapping + sacred boost ────────────────
        let flux_map = FluxPositionMap::build(ph, pw);
        flux_map.apply_boosts(&mut tokens, d);

        // ── Step 6: Mean pool → global embedding ────────────────────────
        let mut pooled = vec![0.0f32; d];
        if num_patches > 0 {
            for i in 0..num_patches {
                for dd in 0..d {
                    pooled[dd] += tokens[i * d + dd];
                }
            }
            let inv = 1.0 / num_patches as f32;
            for dd in 0..d {
                pooled[dd] *= inv;
            }
        }

        // ── Step 7: Sacred gate verification (3→6→9) ───────────────────
        // Use a sample of patch embeddings as context
        let context_refs: Vec<&[f32]> = (0..num_patches.min(9))
            .map(|i| &tokens[i * d..(i + 1) * d])
            .collect();
        let gate_result =
            sacred_gate_verify(&pooled, &context_refs, self.config.gate_thresholds);

        // If gate rejects, dampen embedding (signal low confidence)
        if !gate_result.accepted {
            for dd in 0..d {
                pooled[dd] *= gate_result.composite_score;
            }
        }

        // ── Step 8: Extract spatial relations ───────────────────────────
        let spatial_relations =
            extract_spatial_relations(grid, self.config.patch_size);

        EncodedGrid {
            embedding: pooled,
            patch_embeddings: tokens,
            num_patches_h: ph,
            num_patches_w: pw,
            flux_positions: flux_map.positions,
            spatial_relations,
            gate_result,
        }
    }

    /// Fast encode: skip conv/attention layers, use colour histogram + FNV hash
    /// for a lightweight 256-dim embedding. ~1000× faster than full encode.
    /// Suitable for interactive ARC-AGI-3 where speed matters more than fidelity.
    pub fn encode_fast(&self, grid: &ArcGrid) -> EncodedGrid {
        let h = grid.height;
        let w = grid.width;
        let d = self.config.embed_dim;

        // ── Colour histogram (normalized) ────────────────────────────────
        let mut hist = [0u32; NUM_COLORS];
        let total = (h * w) as f32;
        for r in 0..h {
            for c in 0..w {
                let v = (grid.at(r, c) as usize).min(NUM_COLORS - 1);
                hist[v] += 1;
            }
        }

        // ── Build embedding from histogram + spatial hash ────────────────
        let mut embedding = vec![0.0f32; d];

        // First NUM_COLORS dims: normalized histogram
        for i in 0..NUM_COLORS.min(d) {
            embedding[i] = hist[i] as f32 / total;
        }

        // Next dims: row/col marginals (spatial structure fingerprint)
        let marginal_start = NUM_COLORS;
        for r in 0..h.min((d - marginal_start) / 2) {
            let row_sum: u32 = (0..w).map(|c| grid.at(r, c) as u32).sum();
            embedding[marginal_start + r] = row_sum as f32 / (w as f32 * 9.0);
        }
        let col_start = marginal_start + h.min((d - marginal_start) / 2);
        for c in 0..w.min(d.saturating_sub(col_start)) {
            let col_sum: u32 = (0..h).map(|r| grid.at(r, c) as u32).sum();
            embedding[col_start + c] = col_sum as f32 / (h as f32 * 9.0);
        }

        // Fill remaining dims with FNV-1a hash-derived features
        let mut fnv: u64 = 0xcbf29ce484222325;
        for r in 0..h {
            for c in 0..w {
                fnv ^= grid.at(r, c) as u64;
                fnv = fnv.wrapping_mul(0x100000001b3);
            }
        }
        for i in (col_start + w.min(d.saturating_sub(col_start)))..d {
            fnv ^= i as u64;
            fnv = fnv.wrapping_mul(0x100000001b3);
            embedding[i] = ((fnv >> 32) as f32 / u32::MAX as f32) * 0.1;
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-9 {
            for v in &mut embedding { *v /= norm; }
        }

        // ── Lightweight spatial relations (4x4 patch grid, no conv) ──────
        let spatial_relations =
            extract_spatial_relations(grid, self.config.patch_size);

        // Minimal gate result (always accept for fast path)
        let gate_result = SacredGateResult {
            proximity_score: 1.0,
            coherence_score: 1.0,
            composite_score: 1.0,
            accepted: true,
        };

        EncodedGrid {
            embedding,
            patch_embeddings: Vec::new(),
            num_patches_h: 0,
            num_patches_w: 0,
            flux_positions: Vec::new(),
            spatial_relations,
            gate_result,
        }
    }

    /// Encode a pair of grids (input → output) and produce a transformation
    /// embedding suitable for the VortexDiffusion spatial transform pipeline.
    ///
    /// The delta embedding captures *what changed* between two grids — the
    /// core signal for ARC pattern induction.
    pub fn encode_transform(&self, input: &ArcGrid, output: &ArcGrid) -> TransformEncoding {
        let enc_in = self.encode(input);
        let enc_out = self.encode(output);
        let d = self.config.embed_dim;

        // Delta embedding: pointwise difference (captures the transformation)
        let mut delta = vec![0.0f32; d];
        for dd in 0..d {
            delta[dd] = enc_out.embedding[dd] - enc_in.embedding[dd];
        }

        // Vortex cycle the delta through one full expansion-contraction
        let cycled = vortex_cycle_embedding(&delta);

        TransformEncoding {
            input_encoding: enc_in,
            output_encoding: enc_out,
            delta_embedding: delta,
            cycled_delta: cycled,
        }
    }
}

/// Encoding of a grid transformation (input → output pair).
#[derive(Debug, Clone)]
pub struct TransformEncoding {
    pub input_encoding: EncodedGrid,
    pub output_encoding: EncodedGrid,
    /// Raw difference between output and input embeddings.
    pub delta_embedding: Vec<f32>,
    /// Delta after one full vortex expansion-contraction cycle.
    pub cycled_delta: Vec<f32>,
}

// ─── Vortex Cycle Embedding ──────────────────────────────────────────────────

/// Run an embedding through one full vortex expansion-contraction cycle
/// (1→2→4→8→7→5) with position-dependent noise scaling, matching the
/// VortexDiffusion noise schedule pattern.
fn vortex_cycle_embedding(embedding: &[f32]) -> Vec<f32> {
    let d = embedding.len();
    let mut state = embedding.to_vec();

    // Noise deltas per vortex position (from AdaptiveVortexTopology defaults)
    let deltas: HashMap<u8, f32> = [
        (1, -0.02 * PHI_INV),
        (2, -0.04 * PHI_INV),
        (4, -0.06 * PHI_INV),
        (8, -0.08 * PHI_INV),
        (7, 0.06 * PHI_INV),
        (5, 0.04 * PHI_INV),
    ]
    .into_iter()
    .collect();

    for &pos in &VORTEX_CYCLE {
        let delta = deltas.get(&pos).copied().unwrap_or(0.0);
        for dd in 0..d {
            let freq = 1.0 / (10000.0f32.powf(dd as f32 / d as f32));
            let perturbation = (pos as f32 * freq).sin() * delta;
            state[dd] += perturbation;
        }
    }
    state
}

// ─── Utility Functions ───────────────────────────────────────────────────────

/// Digital root: maps any positive integer to 1–9.
#[inline]
pub fn digital_root(n: u64) -> u8 {
    if n == 0 {
        return 9;
    }
    let r = (n % 9) as u8;
    if r == 0 { 9 } else { r }
}

/// GELU activation (exact form).
#[inline]
fn gelu(x: f32) -> f32 {
    0.5 * x * (1.0 + ((2.0 / std::f32::consts::PI).sqrt() * (x + 0.044715 * x * x * x)).tanh())
}

/// Layer normalisation over the last dimension.
fn layer_norm(input: &[f32], seq_len: usize, dim: usize) -> Vec<f32> {
    let mut output = input.to_vec();
    let eps = 1e-5f32;
    for i in 0..seq_len {
        let base = i * dim;
        let mean: f32 = (0..dim).map(|d| input[base + d]).sum::<f32>() / dim as f32;
        let var: f32 =
            (0..dim).map(|d| (input[base + d] - mean).powi(2)).sum::<f32>() / dim as f32;
        let inv_std = 1.0 / (var + eps).sqrt();
        for d in 0..dim {
            output[base + d] = (input[base + d] - mean) * inv_std;
        }
    }
    output
}

/// Matrix multiply: [M × K] × [K × N] → [M × N].
fn matmul_2d(a: &[f32], b: &[f32], m: usize, k: usize, n: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; m * n];
    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0f32;
            for kk in 0..k {
                sum += a[i * k + kk] * b[kk * n + j];
            }
            out[i * n + j] = sum;
        }
    }
    out
}

/// Cosine similarity between two vectors.
fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na * nb < f32::EPSILON {
        0.0
    } else {
        dot / (na * nb)
    }
}

/// In-place softmax over a slice.
fn softmax_inplace(logits: &mut [f32]) {
    let max = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;
    for v in logits.iter_mut() {
        *v = (*v - max).exp();
        sum += *v;
    }
    if sum > 0.0 {
        for v in logits.iter_mut() {
            *v /= sum;
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_grid_3x3() -> ArcGrid {
        ArcGrid {
            cells: vec![vec![0, 1, 2], vec![3, 0, 5], vec![6, 7, 0]],
            height: 3,
            width: 3,
        }
    }

    #[test]
    fn test_digital_root() {
        assert_eq!(digital_root(1), 1);
        assert_eq!(digital_root(9), 9);
        assert_eq!(digital_root(10), 1);
        assert_eq!(digital_root(18), 9);
        assert_eq!(digital_root(0), 9);
    }

    #[test]
    fn test_cell_embedding_shape() {
        let emb = CellEmbedding::new(10, 256);
        assert_eq!(emb.embed(0).len(), 256);
        assert_eq!(emb.embed(9).len(), 256);
    }

    #[test]
    fn test_sinusoidal_2d_shape() {
        let enc = sinusoidal_2d_pos(5, 10, 256);
        assert_eq!(enc.len(), 256);
    }

    #[test]
    fn test_grid_from_json() {
        let json = serde_json::json!([[0, 1], [2, 3]]);
        let grid = ArcGrid::from_json(&json).unwrap();
        assert_eq!(grid.height, 2);
        assert_eq!(grid.width, 2);
        assert_eq!(grid.at(1, 1), 3);
    }

    #[test]
    fn test_encode_produces_256_dim() {
        let encoder = GridEncoder::new(GridEncoderConfig::default());
        let grid = test_grid_3x3();
        let encoded = encoder.encode(&grid);
        assert_eq!(encoded.embedding.len(), 256);
        assert!(!encoded.flux_positions.is_empty());
    }

    #[test]
    fn test_transform_encoding() {
        let encoder = GridEncoder::new(GridEncoderConfig::default());
        let input = test_grid_3x3();
        let output = ArcGrid {
            cells: vec![vec![2, 1, 0], vec![5, 0, 3], vec![0, 7, 6]],
            height: 3,
            width: 3,
        };
        let tf = encoder.encode_transform(&input, &output);
        assert_eq!(tf.delta_embedding.len(), 256);
        assert_eq!(tf.cycled_delta.len(), 256);
        // Delta should be non-zero for different grids
        assert!(tf.delta_embedding.iter().any(|&v| v.abs() > 1e-10));
    }

    #[test]
    fn test_spatial_relations_extracted() {
        let grid = ArcGrid {
            cells: vec![
                vec![0, 1, 0, 0],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 2],
                vec![0, 0, 0, 0],
            ],
            height: 4,
            width: 4,
        };
        let rels = extract_spatial_relations(&grid, 4);
        // Colour 1 at patch (0,0) and colour 2 at patch (0,0) — merged to same patch
        // With patch_size 2 we'd get separate patches
        let rels2 = extract_spatial_relations(&grid, 2);
        assert!(
            rels2.iter().any(|(_, rel, _)| rel == "left_of" || rel == "above"),
            "Should find spatial relations between non-background patches"
        );
    }

    #[test]
    fn test_sacred_gate_accepts_well_formed() {
        let emb = vec![0.1f32; 256];
        let ctx1 = vec![0.1f32; 256];
        let ctx2 = vec![0.12f32; 256];
        let result = sacred_gate_verify(&emb, &[&ctx1, &ctx2], (0.15, 0.30, 0.40));
        assert!(
            result.accepted,
            "Well-formed embedding coherent with context should pass"
        );
    }
}
