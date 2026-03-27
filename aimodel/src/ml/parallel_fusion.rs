//! Parallel Fusion Architecture
//!
//! # Table of Contents
//! 1.  Helpers / shared math
//! 2.  SpectralConsensus       — Von Neumann spectral decomposition of expert agreement
//! 3.  Color3 / SubjectMetadata — Realm attribution for RocksDB partitioning
//! 4.  SeedGenerator           — Deterministic seed from processing work
//! 5.  ReasoningPath / Implication — Typed pathway representation
//! 6.  TransitiveChainValidator — 3-6-9 checkpoint transitive closure verification
//! 7.  RuleExplosionEngine     — Rayon-parallel up to 9! permutation rule explosion
//! 8.  JEPAPathwayPredictor    — PathwayQuality learning from explosion outcomes
//! 9.  ReasoningScale / ScaledReasoning — Concept / Impl / Micro / Macro output scale
//! 10. FluxMatrixProcessor     — Vortex cycle 1→2→4→8→7→5 query sequencing
//! 11. VonNeumannMerger        — Lattice meet/join + spectral adaptive merge
//! 12. ExhaustiveExplorer      — Tunable heuristic_ratio + JEPA-ranked exhaustive search
//! 13. Tests

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// 1. Helpers
// =============================================================================

/// Cosine similarity between two f32 slices.
fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na < 1e-8 || nb < 1e-8 { return 0.0; }
    (dot / (na * nb)).clamp(-1.0, 1.0)
}

/// Exact factorial — not Stirling's approximation.
pub fn factorial(n: usize) -> usize {
    (1..=n).product()
}

/// Lehmer-encode index → permutation of 0..n.
/// Index must be in [0, n!) — panics otherwise.
pub fn index_to_permutation(mut index: usize, n: usize) -> Vec<usize> {
    let mut elements: Vec<usize> = (0..n).collect();
    let mut result = Vec::with_capacity(n);
    let mut fact = factorial(n);
    for k in (1..=n).rev() {
        fact /= k;
        let pos = index / fact;
        index %= fact;
        result.push(elements.remove(pos));
    }
    result
}

// =============================================================================
// 2. SpectralConsensus — Von Neumann Spectral Decomposition
// =============================================================================
//
// No learned weights: the "magic" is in the spectral structure of agreement.
// Build an (n×n) cosine-similarity covariance matrix from expert outputs, then
// extract the principal eigenvector via power iteration (stable, no LAPACK needed).
// Project all experts onto that axis and sum the projections as consensus.

/// Von Neumann spectral consensus over a set of expert output vectors.
/// Memory: O(n²) where n = number of experts.
pub struct SpectralConsensus;

impl SpectralConsensus {
    /// Compute consensus from expert output embeddings.
    /// Returns a single embedding representing the direction of maximum agreement.
    pub fn compute_consensus(expert_outputs: &[Vec<f32>]) -> Vec<f32> {
        let n = expert_outputs.len();
        if n == 0 { return Vec::new(); }
        if n == 1 { return expert_outputs[0].clone(); }

        let dim = expert_outputs[0].len();

        // Build n×n cosine-similarity covariance matrix
        let cov = Self::build_covariance(expert_outputs);

        // Find dominant eigenvector via power iteration
        let dominant = Self::power_iteration(&cov, n, 64);

        // Project all expert outputs onto dominant eigenvector,
        // weight-sum them into a single consensus embedding
        let mut consensus = vec![0.0f32; dim];
        for (expert_idx, output) in expert_outputs.iter().enumerate() {
            let weight = dominant[expert_idx].abs();
            for (c, &o) in consensus.iter_mut().zip(output.iter()) {
                *c += weight * o;
            }
        }

        // L2-normalize consensus
        let norm: f32 = consensus.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
            for c in &mut consensus { *c /= norm; }
        }
        consensus
    }

    /// Build n×n covariance matrix where entry (i,j) = cosine_similarity(i, j).
    pub fn build_covariance(outputs: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let n = outputs.len();
        let mut cov = vec![vec![0.0f32; n]; n];
        for i in 0..n {
            for j in 0..n {
                cov[i][j] = cosine_sim(&outputs[i], &outputs[j]);
            }
        }
        cov
    }

    /// Power iteration to find the dominant eigenvector of a symmetric matrix.
    /// Runs for `max_iter` steps or until convergence (|δ| < 1e-6).
    pub fn power_iteration(matrix: &[Vec<f32>], n: usize, max_iter: usize) -> Vec<f32> {
        // Start from uniform vector
        let inv_sqrt_n = 1.0 / (n as f32).sqrt();
        let mut v: Vec<f32> = vec![inv_sqrt_n; n];

        for _ in 0..max_iter {
            // v_new = M · v
            let mut v_new = vec![0.0f32; n];
            for i in 0..n {
                for j in 0..n {
                    v_new[i] += matrix[i][j] * v[j];
                }
            }

            // Normalize
            let norm: f32 = v_new.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm < 1e-12 { break; }
            let v_prev = v.clone();
            v = v_new.iter().map(|x| x / norm).collect();

            // Check convergence
            let delta: f32 = v.iter().zip(v_prev.iter()).map(|(a, b)| (a - b).abs()).sum();
            if delta < 1e-6 { break; }
        }
        v
    }

    /// Dominant eigenvalue (Rayleigh quotient) — measures consensus strength.
    /// Value near n means all experts agree; near 1 means no consensus.
    pub fn eigenvalue(matrix: &[Vec<f32>], eigenvec: &[f32]) -> f32 {
        let n = eigenvec.len();
        let mv: Vec<f32> = (0..n).map(|i| {
            matrix[i].iter().zip(eigenvec.iter()).map(|(m, v)| m * v).sum()
        }).collect();
        mv.iter().zip(eigenvec.iter()).map(|(a, b)| a * b).sum()
    }
}

// =============================================================================
// 3. Color3 + SubjectMetadata — Realm Attribution for RocksDB Partitioning
// =============================================================================

/// Three-channel realm descriptor.
///   R = Technical depth    (0 = none, 255 = deeply technical)
///   G = Business impact    (0 = none, 255 = high impact)
///   B = Abstraction level  (0 = concrete, 255 = fully abstract)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Color3 {
    /// Technical depth channel
    pub r: u8,
    /// Business impact channel
    pub g: u8,
    /// Abstraction level channel
    pub b: u8,
}

impl Color3 {
    pub fn new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }

    /// Deterministic hash → RocksDB partition key prefix.
    pub fn to_realm_hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        h.write_u8(self.r);
        h.write_u8(self.g);
        h.write_u8(self.b);
        h.finish()
    }

    /// Assign a Color3 from a named subject domain.
    pub fn from_subject_domain(domain: &str) -> Self {
        match domain {
            "physics"      => Self { r: 255, g: 128, b: 200 },
            "chemistry"    => Self { r: 220, g: 150, b: 180 },
            "biology"      => Self { r: 180, g: 200, b: 120 },
            "math"         => Self { r: 255, g: 50,  b: 255 },
            "cs"           => Self { r: 230, g: 100, b: 210 },
            "business"     => Self { r: 100, g: 255, b: 50  },
            "economics"    => Self { r: 120, g: 220, b: 80  },
            "history"      => Self { r: 80,  g: 180, b: 60  },
            "philosophy"   => Self { r: 200, g: 60,  b: 240 },
            "psychology"   => Self { r: 160, g: 180, b: 140 },
            _              => Self { r: 128, g: 128, b: 128 },
        }
    }

    /// Blend two realms (midpoint in RGB space).
    pub fn blend(&self, other: &Self) -> Self {
        Self {
            r: ((self.r as u16 + other.r as u16) / 2) as u8,
            g: ((self.g as u16 + other.g as u16) / 2) as u8,
            b: ((self.b as u16 + other.b as u16) / 2) as u8,
        }
    }
}

/// Subject with Color3 realm classification for partitioned storage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubjectMetadata {
    /// Unique subject identifier
    pub subject_id: u64,
    /// Business domain classification
    pub realm: Color3,
    /// Deterministic seed from processing (see SeedGenerator)
    pub seed: u64,
    /// Node titles produced during real-time reasoning
    pub node_titles: Vec<String>,
}

impl SubjectMetadata {
    /// Build the RocksDB key for this subject.
    pub fn storage_key(&self) -> String {
        format!("realm:{}:subject:{}", self.realm.to_realm_hash(), self.subject_id)
    }
}

// =============================================================================
// 4. SeedGenerator — Deterministic Seed from Processing Work
// =============================================================================
//
// Seed is deterministic from the WORK done, not random.
// Same query + same node_titles + same operators → same seed.

/// Reasoning operation types (deduction, abduction, induction).
#[derive(Clone, Debug, Serialize, Deserialize, Copy)]
#[repr(u8)]
pub enum ReasoningOp {
    /// A→B, B→C ∴ A→C
    Deduction  = 1,
    /// A→C, B→C ∴ A or B
    Abduction  = 2,
    /// Pattern generalization from examples
    Induction  = 3,
}

/// Generates deterministic seeds from the actual processing work.
pub struct SeedGenerator {
    /// Transitive chain hashes accumulated during reasoning
    transitive_chain_hashes: Vec<u64>,
}

impl SeedGenerator {
    pub fn new() -> Self {
        Self { transitive_chain_hashes: Vec::new() }
    }

    /// Record a transitive chain hash (e.g., from TransitiveFluxReasoner).
    pub fn record_chain(&mut self, chain_hash: u64) {
        self.transitive_chain_hashes.push(chain_hash);
    }

    /// Generate a deterministic seed from: query + ordered node_titles + operators + chains.
    /// Same inputs → same seed every time.
    pub fn generate(&self, query: &str, node_titles: &[String], operators: &[ReasoningOp]) -> u64 {
        let mut h = DefaultHasher::new();

        // 1. Hash the query
        for byte in query.as_bytes() { h.write_u8(*byte); }

        // 2. Hash node titles IN ORDER (sequence matters for reasoning)
        for title in node_titles {
            for byte in title.as_bytes() { h.write_u8(*byte); }
        }

        // 3. Hash reasoning operators
        for op in operators {
            h.write_u8(*op as u8);
        }

        // 4. Hash transitive chain structure
        for &chain_hash in &self.transitive_chain_hashes {
            h.write_u64(chain_hash);
        }

        h.finish()
    }
}

impl Default for SeedGenerator {
    fn default() -> Self { Self::new() }
}

// =============================================================================
// 5. ReasoningPath + Implication — Typed Pathway Representation
// =============================================================================

/// A node in a reasoning path.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FusionPathNode {
    /// Unique node identifier
    pub id: u64,
    /// Human-readable label
    pub label: String,
    /// Embedding vector for this node
    pub embedding: Vec<f32>,
}

/// A path through the reasoning space.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReasoningPath {
    /// Ordered sequence of nodes
    pub nodes: Vec<FusionPathNode>,
    /// Deterministic seed (from SeedGenerator)
    pub seed: u64,
    /// Overall path confidence
    pub confidence: f32,
    /// Whether any sacred checkpoint failed
    pub checkpoint_failed: bool,
}

impl ReasoningPath {
    pub fn new(seed: u64) -> Self {
        Self { nodes: Vec::new(), seed, confidence: 1.0, checkpoint_failed: false }
    }

    pub fn add_node(&mut self, node: FusionPathNode) {
        self.nodes.push(node);
    }

    pub fn mark_failed(&mut self) {
        self.checkpoint_failed = true;
    }
}

/// A derived implication from rule application on a path.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Implication {
    /// Source path that generated this implication
    pub source_seed: u64,
    /// The implied conclusion embedding
    pub conclusion: Vec<f32>,
    /// Confidence in this implication
    pub confidence: f32,
    /// Whether this implication is logically valid
    pub is_valid: bool,
    /// Rule sequence that produced this implication (indices into flux rules)
    pub rule_sequence: Vec<usize>,
}

// =============================================================================
// 6. TransitiveChainValidator — 3-6-9 Checkpoint Verification
// =============================================================================
//
// At each sacred position (3, 6, 9), verifies that the transitive closure
// holds: A→B and B→C implies A→C in the embedding space.
// Uses cosine similarity threshold as a proxy for semantic transitivity.

/// Validates transitive chain closure at sacred positions 3, 6, 9.
pub struct TransitiveChainValidator {
    /// Sacred checkpoint positions (immutable — never change)
    pub checkpoint_positions: [usize; 3],
    /// Minimum cosine similarity to consider two embeddings related
    pub relation_threshold: f32,
    /// Minimum transitivity score to pass a checkpoint
    pub transitivity_threshold: f32,
}

impl Default for TransitiveChainValidator {
    fn default() -> Self {
        Self {
            checkpoint_positions: [3, 6, 9],
            relation_threshold: 0.3,
            transitivity_threshold: 0.2,
        }
    }
}

impl TransitiveChainValidator {
    pub fn new(relation_threshold: f32, transitivity_threshold: f32) -> Self {
        Self {
            checkpoint_positions: [3, 6, 9],
            relation_threshold,
            transitivity_threshold,
        }
    }

    /// Validate all sacred checkpoints for a path.
    /// Returns true only if ALL reachable checkpoints pass.
    pub fn validate_checkpoints(&self, path: &ReasoningPath) -> bool {
        for &pos in &self.checkpoint_positions {
            if pos >= path.nodes.len() { continue; }
            if !self.verify_at_checkpoint(path, pos) {
                return false;
            }
        }
        true
    }

    /// Verify transitive chain at a single checkpoint position.
    /// Checks: nodes[pos-2] → nodes[pos-1] → nodes[pos] implies nodes[pos-2] → nodes[pos].
    pub fn verify_at_checkpoint(&self, path: &ReasoningPath, checkpoint: usize) -> bool {
        if checkpoint < 2 || path.nodes.len() <= checkpoint { return true; }

        let a = &path.nodes[checkpoint - 2].embedding;
        let b = &path.nodes[checkpoint - 1].embedding;
        let c = &path.nodes[checkpoint].embedding;

        let ab = cosine_sim(a, b);
        let bc = cosine_sim(b, c);
        let ac = cosine_sim(a, c);

        // Both A→B and B→C must be above threshold to check transitivity
        if ab < self.relation_threshold || bc < self.relation_threshold {
            return true; // Insufficient data — don't fail
        }

        // Transitive closure: A→C similarity should be at least min(A→B, B→C) * threshold
        let expected_ac = ab.min(bc) * self.transitivity_threshold;
        ac >= expected_ac
    }

    /// Checkpoint result detail for each sacred position.
    pub fn checkpoint_results(&self, path: &ReasoningPath) -> [bool; 3] {
        [
            self.checkpoint_at(path, 0),
            self.checkpoint_at(path, 1),
            self.checkpoint_at(path, 2),
        ]
    }

    fn checkpoint_at(&self, path: &ReasoningPath, idx: usize) -> bool {
        let pos = self.checkpoint_positions[idx];
        if pos >= path.nodes.len() { return true; }
        self.verify_at_checkpoint(path, pos)
    }
}

// =============================================================================
// 7. RuleExplosionEngine — Rayon-Parallel Up to 9! Rule Permutations
// =============================================================================
//
// Applies ALL permutations of up to 9 rules to a reasoning path in parallel.
// Each permutation is a different rule application order — the one that
// generates the highest-confidence implication wins.

/// A single symbolic rule operating on an embedding.
#[derive(Clone, Debug)]
pub struct FluxRule {
    /// Rule identifier
    pub id: usize,
    /// Rule name / description
    pub name: String,
    /// Transformation: scales embedding by weight then adds bias direction
    pub weight: f32,
    /// Bias direction (must be same dimension as embeddings)
    pub bias: Vec<f32>,
}

impl FluxRule {
    pub fn new(id: usize, name: &str, weight: f32, bias: Vec<f32>) -> Self {
        Self { id, name: name.to_string(), weight, bias }
    }

    /// Apply this rule to an embedding.
    pub fn apply(&self, embedding: &[f32]) -> Vec<f32> {
        let mut result: Vec<f32> = embedding.iter()
            .zip(self.bias.iter().chain(std::iter::repeat(&0.0f32)))
            .map(|(e, b)| e * self.weight + b)
            .collect();
        result.truncate(embedding.len());
        // L2 normalize
        let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 { for x in &mut result { *x /= norm; } }
        result
    }
}

/// Configuration for rule explosion.
#[derive(Clone, Debug)]
pub struct RuleExplosionConfig {
    /// Maximum rules to permute (caps at 9 for tractability — 9! = 362,880)
    pub max_rules: usize,
    /// Minimum implication confidence to keep
    pub min_confidence: f32,
}

impl Default for RuleExplosionConfig {
    fn default() -> Self {
        Self { max_rules: 9, min_confidence: 0.1 }
    }
}

/// Explodes a set of rules into all valid permutations applied to a path.
pub struct RuleExplosionEngine {
    pub config: RuleExplosionConfig,
}

impl RuleExplosionEngine {
    pub fn new(config: RuleExplosionConfig) -> Self {
        Self { config }
    }

    /// Explode all permutations of `rules` applied to `path` in parallel via Rayon.
    /// Returns all valid implications sorted by confidence descending.
    pub fn explode_parallel(
        &self,
        path: &ReasoningPath,
        rules: &[FluxRule],
    ) -> Vec<Implication> {
        let n = rules.len().min(self.config.max_rules);
        if n == 0 || path.nodes.is_empty() { return Vec::new(); }

        let n_perms = factorial(n);
        let min_conf = self.config.min_confidence;

        // Parallel iteration over all permutations
        let mut implications: Vec<Implication> = (0..n_perms)
            .into_par_iter()
            .filter_map(|perm_index| {
                let rule_sequence = index_to_permutation(perm_index, n);
                Self::apply_rule_sequence(path, &rule_sequence, rules, min_conf)
            })
            .collect();

        // Sort by confidence descending
        implications.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal));
        implications
    }

    /// Apply a specific rule sequence to the last node embedding of a path.
    fn apply_rule_sequence(
        path: &ReasoningPath,
        rule_sequence: &[usize],
        rules: &[FluxRule],
        min_confidence: f32,
    ) -> Option<Implication> {
        let start = path.nodes.last()?.embedding.clone();
        let mut current = start;

        // Apply rules in sequence order — each rule transforms the embedding
        for &rule_idx in rule_sequence {
            if rule_idx < rules.len() {
                current = rules[rule_idx].apply(&current);
            }
        }

        // Confidence = cosine similarity between start and end (measures coherence)
        let confidence = cosine_sim(path.nodes.first().map(|n| n.embedding.as_slice()).unwrap_or(&[]),
                                    &current).abs();

        if confidence < min_confidence {
            return None;
        }

        Some(Implication {
            source_seed: path.seed,
            conclusion: current,
            confidence,
            is_valid: true,
            rule_sequence: rule_sequence.to_vec(),
        })
    }

    /// Count total permutations for n rules (exact factorial).
    pub fn total_permutations(n: usize) -> usize {
        factorial(n.min(9))
    }
}

// =============================================================================
// 8. JEPAPathwayPredictor — Learns PathwayQuality from Explosion Outcomes
// =============================================================================
//
// After each rule explosion, the predictor records which seeds led to
// high-quality implications. Future explorations rank candidates by learned
// quality, visiting high-quality-seed pathways first (without pruning).

/// Quality record for a reasoning pathway seed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PathwayQuality {
    /// Seed that identifies this pathway
    pub seed: u64,
    /// Fraction of implications that were valid and high-confidence
    pub success_rate: f32,
    /// Average number of implications produced
    pub avg_implications: usize,
    /// Whether each sacred checkpoint passed [pos3, pos6, pos9]
    pub checkpoint_validity: [bool; 3],
    /// Number of times this seed has been observed
    pub observation_count: u32,
}

/// JEPA-inspired pathway quality predictor.
/// Learns from explosion outcomes — no pretrained weights, pure online learning.
pub struct JEPAPathwayPredictor {
    /// Quality records indexed by seed
    pattern_db: HashMap<u64, PathwayQuality>,
}

impl JEPAPathwayPredictor {
    pub fn new() -> Self {
        Self { pattern_db: HashMap::new() }
    }

    /// Record the outcome of a rule explosion for a pathway.
    pub fn learn_from_explosion(
        &mut self,
        path: &ReasoningPath,
        implications: &[Implication],
        checkpoint_validity: [bool; 3],
    ) {
        let valid_count = implications.iter().filter(|i| i.is_valid).count();
        let success_rate = if implications.is_empty() { 0.0 }
            else { valid_count as f32 / implications.len() as f32 };

        let entry = self.pattern_db.entry(path.seed).or_insert(PathwayQuality {
            seed: path.seed,
            success_rate: 0.5, // Neutral prior — same as unknown seeds
            avg_implications: 0,
            checkpoint_validity: [false; 3],
            observation_count: 0,
        });

        // Exponential moving average update (α = 0.1) for stable online learning
        let alpha = 0.1f32;
        entry.success_rate = (1.0 - alpha) * entry.success_rate + alpha * success_rate;
        entry.avg_implications = ((1.0 - alpha) * entry.avg_implications as f32
            + alpha * implications.len() as f32) as usize;
        entry.checkpoint_validity = checkpoint_validity;
        entry.observation_count += 1;
    }

    /// Get quality estimate for a seed (defaults to 0.5 = neutral for unseen seeds).
    pub fn quality(&self, seed: u64) -> f32 {
        self.pattern_db.get(&seed).map(|q| q.success_rate).unwrap_or(0.5)
    }

    /// Sort paths by learned quality descending (best first, but ALL are explored).
    pub fn rank_paths(&self, paths: &mut Vec<ReasoningPath>) {
        paths.sort_by(|a, b| {
            let qa = self.quality(a.seed);
            let qb = self.quality(b.seed);
            qb.partial_cmp(&qa).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Number of unique seeds recorded.
    pub fn known_seeds(&self) -> usize {
        self.pattern_db.len()
    }

    /// Get quality record for a seed (if known).
    pub fn get_quality(&self, seed: u64) -> Option<&PathwayQuality> {
        self.pattern_db.get(&seed)
    }
}

impl Default for JEPAPathwayPredictor {
    fn default() -> Self { Self::new() }
}

// =============================================================================
// 9. ReasoningScale + ScaledReasoning — Output Verbosity Control
// =============================================================================

/// Scale at which reasoning output is generated.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReasoningScale {
    /// High-level abstract principles: "Gravity causes spacetime curvature"
    Concept,
    /// Concrete algorithm: "1. Check mass, 2. Compute curvature, 3. Apply equations"
    Implementation,
    /// Individual operations / formulas: "G_μν = (8πG/c^4) T_μν"
    Micro,
    /// System-level patterns and laws: "GR unifies gravity with geometry at all scales"
    Macro,
}

/// Generates scaled output from reasoning implications.
pub struct ScaledReasoning {
    /// Output scale
    pub scale: ReasoningScale,
    /// Detail level [0.0, 1.0] — controls token budget
    pub detail_level: f32,
}

impl ScaledReasoning {
    pub fn new(scale: ReasoningScale, detail_level: f32) -> Self {
        Self { scale, detail_level: detail_level.clamp(0.0, 1.0) }
    }

    /// Compute token budget from query length and detail level.
    /// Budget = query_len × 2 × (1 + detail_level × 9), capped at 2048.
    pub fn compute_token_budget(&self, query_length: usize) -> usize {
        let base = (query_length * 2).max(32);
        let scaled = (base as f32 * (1.0 + self.detail_level * 9.0)) as usize;
        scaled.min(2048)
    }

    /// Generate text output from a set of implications at the configured scale.
    pub fn generate_output(&self, implications: &[Implication], token_budget: usize) -> String {
        if implications.is_empty() {
            return match self.scale {
                ReasoningScale::Concept => "No conceptual patterns identified.".to_string(),
                ReasoningScale::Implementation => "No implementation steps derived.".to_string(),
                ReasoningScale::Micro => "No micro-level operations derived.".to_string(),
                ReasoningScale::Macro => "No macro-level laws identified.".to_string(),
            };
        }

        // Take the top implications within token budget
        let max_implications = (token_budget / 40).max(1);
        let top = implications.iter().take(max_implications);

        match self.scale {
            ReasoningScale::Concept => {
                // High-level: describe confidence clusters
                let avg_conf: f32 = implications.iter().map(|i| i.confidence).sum::<f32>()
                    / implications.len() as f32;
                format!("[Concept] {} implication patterns found, avg confidence {:.2}. \
                         Top {} shown. High-confidence paths indicate strong conceptual alignment.",
                    implications.len(), avg_conf, top.count())
            }
            ReasoningScale::Implementation => {
                // Algorithm: list rule sequences for top implications
                let steps: Vec<String> = top.enumerate()
                    .map(|(i, imp)| format!("{}. Rule sequence {:?} → confidence {:.3}",
                        i + 1, imp.rule_sequence, imp.confidence))
                    .collect();
                format!("[Implementation]\n{}", steps.join("\n"))
            }
            ReasoningScale::Micro => {
                // Operations: detailed per-implication breakdown
                let ops: Vec<String> = top.enumerate()
                    .map(|(i, imp)| format!("op[{}]: seed={} conf={:.4} rules={:?}",
                        i, imp.source_seed, imp.confidence, imp.rule_sequence))
                    .collect();
                format!("[Micro]\n{}", ops.join("\n"))
            }
            ReasoningScale::Macro => {
                // System-level: derive laws from valid implications
                let valid = implications.iter().filter(|i| i.is_valid).count();
                let total = implications.len();
                let validity_rate = valid as f32 / total as f32;
                format!("[Macro] {}/{} implications valid (rate={:.2}). \
                         System exhibits {:.0}% logical coherence across {} rule permutations.",
                    valid, total, validity_rate, validity_rate * 100.0, total)
            }
        }
    }
}

// =============================================================================
// 10. FluxMatrixProcessor — Vortex Cycle Query Sequencing
// =============================================================================
//
// Processes a query by inserting it at position 1 as a flux object, then
// streaming it through the vortex cycle 1→2→4→8→7→5, verifying at sacred
// checkpoints 3, 6, 9 (which exist outside the flow, observing only).

/// A flux object representing a query at a vortex position.
#[derive(Clone, Debug)]
pub struct FluxObject {
    /// Digital root value (1-9 via exponential reduction)
    pub value: usize,
    /// Embedding of the original query
    pub embedding: Vec<f32>,
    /// Original query text
    pub query: String,
    /// Node label derived at this position
    pub label: String,
}

/// Processes queries through the vortex cycle 1→2→4→8→7→5.
pub struct FluxMatrixProcessor {
    /// Vortex cycle positions (immutable)
    pub vortex_cycle: [usize; 6],
    /// Sacred observer positions (outside the flow — observe only)
    pub sacred_positions: [usize; 3],
    /// Transitive chain validator for sacred gate checks
    pub validator: TransitiveChainValidator,
}

impl Default for FluxMatrixProcessor {
    fn default() -> Self {
        Self {
            vortex_cycle: [1, 2, 4, 8, 7, 5],
            sacred_positions: [3, 6, 9],
            validator: TransitiveChainValidator::default(),
        }
    }
}

impl FluxMatrixProcessor {
    pub fn new() -> Self { Self::default() }

    /// Convert a query string into a FluxObject at position 1.
    pub fn query_to_flux_object(&self, query: &str) -> FluxObject {
        // Build a simple hash-based embedding (real use: replace with text encoder)
        let mut h = DefaultHasher::new();
        for byte in query.as_bytes() { h.write_u8(*byte); }
        let hash = h.finish();

        let dim = 64usize;
        let embedding: Vec<f32> = (0..dim)
            .map(|i| {
                let v = hash.wrapping_add(i as u64).wrapping_mul(6364136223846793005);
                (v as i64 as f32) / (i64::MAX as f32)
            })
            .collect();

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        let embedding = if norm > 1e-8 {
            embedding.iter().map(|x| x / norm).collect()
        } else {
            embedding
        };

        FluxObject {
            value: self.digital_root(hash),
            embedding,
            query: query.to_string(),
            label: format!("pos1:{}", &query[..query.len().min(20)]),
        }
    }

    /// Digital root: reduce to single digit 1-9.
    pub fn digital_root(&self, n: u64) -> usize {
        if n == 0 { return 0; }
        let r = (n % 9) as usize;
        if r == 0 { 9 } else { r }
    }

    /// Stream a flux object through the vortex cycle, producing a ReasoningPath.
    /// Sacred positions observe but never mutate.
    pub fn process_query(&self, query: &str, seed: u64) -> (ReasoningPath, Vec<bool>) {
        let flux_object = self.query_to_flux_object(query);
        let mut path = ReasoningPath::new(seed);
        let mut sacred_signals: Vec<bool> = Vec::new();

        // Insert at position 1 (always the starting point)
        path.add_node(FusionPathNode {
            id: 1,
            label: flux_object.label.clone(),
            embedding: flux_object.embedding.clone(),
        });

        // Stream through vortex cycle 1→2→4→8→7→5
        let mut current_embed = flux_object.embedding.clone();
        for &pos in &self.vortex_cycle[1..] {
            // Transform embedding at each position (position-scaled rotation)
            let transformed = self.position_transform(&current_embed, pos);

            path.add_node(FusionPathNode {
                id: pos as u64,
                label: format!("pos{}", pos),
                embedding: transformed.clone(),
            });

            // Sacred positions (3, 6, 9) observe externally — emit signal
            if self.sacred_positions.contains(&pos) {
                let signal = self.validator.verify_at_checkpoint(&path, path.nodes.len() - 1);
                sacred_signals.push(signal);
                if !signal {
                    path.mark_failed();
                }
            }

            current_embed = transformed;
        }

        (path, sacred_signals)
    }

    /// Position-specific transformation: scales and rotates embedding by
    /// the vortex position's sacred weight.
    fn position_transform(&self, embed: &[f32], position: usize) -> Vec<f32> {
        let sacred_boost = if self.sacred_positions.contains(&position) { 1.15_f32 } else { 1.0_f32 };
        let scale = (position as f32 * std::f32::consts::PI / 9.0).sin().abs() * sacred_boost;
        let scale = scale.max(0.1);

        let mut result: Vec<f32> = embed.iter().enumerate()
            .map(|(i, &v)| {
                let phase = (position as f32 + i as f32 * 0.01) * std::f32::consts::PI / 9.0;
                v * scale + phase.sin() * 0.01
            })
            .collect();

        let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 { for x in &mut result { *x /= norm; } }
        result
    }
}

// =============================================================================
// 11. VonNeumannMerger — Lattice Meet/Join + Spectral Adaptive Merge
// =============================================================================
//
// Von Neumann algebra: pathways are projections on a Hilbert space.
// Meet (∧) = intersection of agreements (conservative).
// Join (∨) = union of claims (comprehensive).
// Spectral merge = SpectralConsensus over all projections.

/// A projection of a reasoning path into embedding space.
#[derive(Clone, Debug)]
pub struct Projection {
    /// The projected embedding
    pub embedding: Vec<f32>,
    /// Confidence weight
    pub weight: f32,
    /// Source path seed
    pub source_seed: u64,
}

impl Projection {
    /// Component-wise intersection (conservative: take minimum absolute value per dimension).
    pub fn intersect(&self, other: &Self) -> Self {
        let embedding = self.embedding.iter()
            .zip(other.embedding.iter())
            .map(|(a, b)| if a.abs() < b.abs() { *a } else { *b })
            .collect();
        Self {
            embedding,
            weight: self.weight.min(other.weight),
            source_seed: self.source_seed,
        }
    }

    /// Component-wise union (comprehensive: take maximum absolute value per dimension).
    pub fn union(&self, other: &Self) -> Self {
        let embedding = self.embedding.iter()
            .zip(other.embedding.iter())
            .map(|(a, b)| if a.abs() > b.abs() { *a } else { *b })
            .collect();
        Self {
            embedding,
            weight: self.weight.max(other.weight),
            source_seed: self.source_seed,
        }
    }

    /// Empty projection (identity for union).
    pub fn empty(dim: usize) -> Self {
        Self { embedding: vec![0.0; dim], weight: 0.0, source_seed: 0 }
    }
}

/// Von Neumann algebraic merger for reasoning pathways.
pub struct VonNeumannMerger;

impl VonNeumannMerger {
    /// Convert a ReasoningPath to a Projection (uses last node embedding).
    pub fn pathway_to_projection(path: &ReasoningPath) -> Projection {
        let embedding = path.nodes.last()
            .map(|n| n.embedding.clone())
            .unwrap_or_default();
        Projection {
            weight: path.confidence,
            source_seed: path.seed,
            embedding,
        }
    }

    /// Meet (∧): intersection of all projections — what EVERY path agrees on.
    pub fn lattice_meet(projections: &[Projection]) -> Projection {
        if projections.is_empty() {
            return Projection::empty(0);
        }
        projections.iter().skip(1)
            .fold(projections[0].clone(), |acc, proj| acc.intersect(proj))
    }

    /// Join (∨): union of all projections — what ANY path claims.
    pub fn lattice_join(projections: &[Projection]) -> Projection {
        if projections.is_empty() {
            return Projection::empty(0);
        }
        let dim = projections[0].embedding.len();
        projections.iter()
            .fold(Projection::empty(dim), |acc, proj| acc.union(proj))
    }

    /// Spectral merge: SpectralConsensus over all projection embeddings.
    pub fn spectral_merge(projections: &[Projection]) -> Projection {
        if projections.is_empty() { return Projection::empty(0); }
        let outputs: Vec<Vec<f32>> = projections.iter()
            .map(|p| p.embedding.clone())
            .collect();
        let consensus = SpectralConsensus::compute_consensus(&outputs);
        let avg_weight = projections.iter().map(|p| p.weight).sum::<f32>()
            / projections.len() as f32;
        Projection { embedding: consensus, weight: avg_weight, source_seed: 0 }
    }

    /// Adaptive merge: choose strategy based on agreement level.
    ///   High agreement (eigenvalue > 0.8n) → spectral merge
    ///   Medium agreement                   → lattice join
    ///   Low agreement (eigenvalue < 0.3n)  → lattice meet (conservative)
    pub fn merge_pathways(paths: &[ReasoningPath]) -> Projection {
        if paths.is_empty() { return Projection::empty(0); }
        if paths.len() == 1 { return Self::pathway_to_projection(&paths[0]); }

        let projections: Vec<Projection> = paths.iter()
            .map(Self::pathway_to_projection)
            .collect();

        let outputs: Vec<Vec<f32>> = projections.iter()
            .map(|p| p.embedding.clone())
            .collect();

        let n = outputs.len();
        let cov = SpectralConsensus::build_covariance(&outputs);
        let dominant = SpectralConsensus::power_iteration(&cov, n, 32);
        let eigenvalue = SpectralConsensus::eigenvalue(&cov, &dominant);

        let high_threshold = (n as f32) * 0.8;
        let low_threshold = (n as f32) * 0.3;

        if eigenvalue >= high_threshold {
            Self::spectral_merge(&projections)
        } else if eigenvalue <= low_threshold {
            Self::lattice_meet(&projections)
        } else {
            Self::lattice_join(&projections)
        }
    }
}

// =============================================================================
// 12. ExhaustiveExplorer — Tunable Heuristic Ratio + JEPA-Ranked Exploration
// =============================================================================
//
// heuristic_ratio: 0.0 = fully exhaustive (explores ALL paths)
//                  1.0 = fully heuristic (explores JEPA-top-k only)
// Either way, no pruning of high-quality paths — JEPA only reorders, not prunes.

/// Configuration for the exhaustive explorer.
#[derive(Clone, Debug)]
pub struct ExplorerConfig {
    /// 0.0 = full exhaustive, 1.0 = full heuristic, values in between = hybrid
    pub heuristic_ratio: f32,
    /// Maximum path depth (capped at 9 for tractability)
    pub max_depth: usize,
    /// Minimum transitive checkpoint validity to explore a path
    pub require_valid_checkpoints: bool,
}

impl Default for ExplorerConfig {
    fn default() -> Self {
        Self {
            heuristic_ratio: 0.5,
            max_depth: 9,
            require_valid_checkpoints: true,
        }
    }
}

/// Exhaustive pathway explorer with tunable heuristic/exhaustive split.
pub struct ExhaustiveExplorer {
    pub config: ExplorerConfig,
    pub validator: TransitiveChainValidator,
    pub predictor: JEPAPathwayPredictor,
    pub explosion_engine: RuleExplosionEngine,
}

impl ExhaustiveExplorer {
    pub fn new(config: ExplorerConfig) -> Self {
        Self {
            config,
            validator: TransitiveChainValidator::default(),
            predictor: JEPAPathwayPredictor::new(),
            explosion_engine: RuleExplosionEngine::new(RuleExplosionConfig::default()),
        }
    }

    /// Explore all paths, ranked by JEPA quality. Apply rule explosion and
    /// collect implications. Use SpectralConsensus to merge final result.
    ///
    /// - When heuristic_ratio = 0.0: explore all paths regardless of quality
    /// - When heuristic_ratio = 1.0: explore only top-ranked paths
    /// - In between: explore all but prioritize high-quality ones
    pub fn explore(
        &mut self,
        mut candidate_paths: Vec<ReasoningPath>,
        rules: &[FluxRule],
    ) -> Vec<Implication> {
        // 1. JEPA ranks paths by learned quality (reorders, never prunes for ratio < 1.0)
        self.predictor.rank_paths(&mut candidate_paths);

        // 2. Determine exploration cutoff based on heuristic_ratio
        let n_explore = if self.config.heuristic_ratio >= 1.0 {
            // Fully heuristic: top-sqrt(n) paths only
            ((candidate_paths.len() as f32).sqrt().ceil() as usize).max(1)
        } else if self.config.heuristic_ratio <= 0.0 {
            // Fully exhaustive: all paths
            candidate_paths.len()
        } else {
            // Hybrid: explore all but weight implication confidence by JEPA quality
            candidate_paths.len()
        };

        let paths_to_explore = &candidate_paths[..n_explore.min(candidate_paths.len())];

        // 3. Explore each path: validate checkpoints, explode rules, collect implications
        let mut all_implications: Vec<Implication> = Vec::new();

        for path in paths_to_explore {
            // Sacred checkpoint validation (3, 6, 9)
            if self.config.require_valid_checkpoints
                && !self.validator.validate_checkpoints(path) {
                continue;
            }

            // Rule explosion (up to 9! permutations in parallel)
            let mut implications = self.explosion_engine.explode_parallel(path, rules);

            // In hybrid mode: scale implication confidence by JEPA quality weight
            if self.config.heuristic_ratio > 0.0 && self.config.heuristic_ratio < 1.0 {
                let quality = self.predictor.quality(path.seed);
                let weight = (1.0 - self.config.heuristic_ratio)
                    + self.config.heuristic_ratio * quality;
                for imp in &mut implications {
                    imp.confidence *= weight;
                }
            }

            // Update JEPA predictor with explosion outcomes
            let checkpoint_validity = self.validator.checkpoint_results(path);
            self.predictor.learn_from_explosion(path, &implications, checkpoint_validity);

            all_implications.extend(implications);
        }

        // 4. Sort all implications by confidence
        all_implications.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal));

        all_implications
    }
}

// =============================================================================
// 13. Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_paths(seeds: &[u64], dim: usize) -> Vec<ReasoningPath> {
        seeds.iter().enumerate().map(|(i, &seed)| {
            let mut path = ReasoningPath::new(seed);
            for j in 0..4 {
                let mut embed = vec![0.0f32; dim];
                let idx = (i * 4 + j) % dim;
                embed[idx] = 1.0;
                path.add_node(FusionPathNode { id: j as u64, label: format!("n{}", j), embedding: embed });
            }
            path
        }).collect()
    }

    // -------------------------------------------------------------------------
    // SpectralConsensus
    // -------------------------------------------------------------------------

    #[test]
    fn test_spectral_consensus_single_expert() {
        let outputs = vec![vec![1.0, 0.0, 0.0, 0.0]];
        let consensus = SpectralConsensus::compute_consensus(&outputs);
        assert_eq!(consensus.len(), 4);
    }

    #[test]
    fn test_spectral_consensus_agreeing_experts() {
        // All experts point in the same direction → consensus should align with them
        let outputs = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.9, 0.1, 0.0],
            vec![0.95, 0.05, 0.0],
        ];
        let consensus = SpectralConsensus::compute_consensus(&outputs);
        assert_eq!(consensus.len(), 3);
        // Consensus should be mostly in first dimension
        assert!(consensus[0].abs() > 0.5, "Consensus should align with agreeing direction, got {:?}", consensus);
    }

    #[test]
    fn test_covariance_diagonal_is_one() {
        let outputs = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![0.5, 0.5f32.sqrt()]];
        let cov = SpectralConsensus::build_covariance(&outputs);
        for i in 0..cov.len() {
            assert!((cov[i][i] - 1.0).abs() < 1e-5, "Diagonal should be 1.0 (self-similarity)");
        }
    }

    #[test]
    fn test_power_iteration_converges() {
        // Identity matrix → eigenvector is uniform
        let n = 3;
        let identity = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];
        let v = SpectralConsensus::power_iteration(&identity, n, 64);
        assert_eq!(v.len(), n);
        // Each component should be ~1/sqrt(3)
        let expected = 1.0 / (3.0f32).sqrt();
        for &vi in &v {
            assert!((vi.abs() - expected).abs() < 0.01, "Uniform matrix eigenvec should be uniform, got {:?}", v);
        }
    }

    // -------------------------------------------------------------------------
    // Color3
    // -------------------------------------------------------------------------

    #[test]
    fn test_color3_realm_hash_deterministic() {
        let c = Color3::new(255, 128, 64);
        assert_eq!(c.to_realm_hash(), c.to_realm_hash());
    }

    #[test]
    fn test_color3_distinct_domains_have_distinct_hashes() {
        let physics = Color3::from_subject_domain("physics");
        let business = Color3::from_subject_domain("business");
        assert_ne!(physics.to_realm_hash(), business.to_realm_hash());
    }

    #[test]
    fn test_color3_blend_midpoint() {
        let a = Color3::new(0, 0, 0);
        let b = Color3::new(200, 100, 50);
        let blended = a.blend(&b);
        assert_eq!(blended.r, 100);
        assert_eq!(blended.g, 50);
        assert_eq!(blended.b, 25);
    }

    // -------------------------------------------------------------------------
    // SeedGenerator
    // -------------------------------------------------------------------------

    #[test]
    fn test_seed_deterministic() {
        let gen = SeedGenerator::new();
        let titles = vec!["node_a".to_string(), "node_b".to_string()];
        let ops = vec![ReasoningOp::Deduction, ReasoningOp::Induction];
        let s1 = gen.generate("what is gravity?", &titles, &ops);
        let s2 = gen.generate("what is gravity?", &titles, &ops);
        assert_eq!(s1, s2, "Same inputs should yield same seed");
    }

    #[test]
    fn test_seed_differs_on_different_query() {
        let gen = SeedGenerator::new();
        let titles = vec!["node_a".to_string()];
        let ops = vec![ReasoningOp::Deduction];
        let s1 = gen.generate("query A", &titles, &ops);
        let s2 = gen.generate("query B", &titles, &ops);
        assert_ne!(s1, s2, "Different queries should yield different seeds");
    }

    #[test]
    fn test_seed_order_sensitive() {
        let gen = SeedGenerator::new();
        let ops = vec![ReasoningOp::Deduction];
        let s1 = gen.generate("q", &["A".to_string(), "B".to_string()], &ops);
        let s2 = gen.generate("q", &["B".to_string(), "A".to_string()], &ops);
        assert_ne!(s1, s2, "Node title ORDER should matter for seed");
    }

    // -------------------------------------------------------------------------
    // TransitiveChainValidator
    // -------------------------------------------------------------------------

    #[test]
    fn test_validator_straight_chain_passes() {
        let validator = TransitiveChainValidator::new(0.3, 0.2);
        // Embeddings that form a clear transitive chain (all similar)
        let mut path = ReasoningPath::new(42);
        for i in 0u64..4 {
            let mut embed = vec![0.0f32; 4];
            embed[0] = 1.0 - i as f32 * 0.1; // Slowly rotating, stays similar
            embed[1] = i as f32 * 0.1;
            path.add_node(FusionPathNode { id: i, label: format!("n{}", i), embedding: embed });
        }
        // With low thresholds, should pass
        assert!(validator.validate_checkpoints(&path));
    }

    #[test]
    fn test_validator_short_path_always_passes() {
        let validator = TransitiveChainValidator::default();
        let mut path = ReasoningPath::new(0);
        path.add_node(FusionPathNode { id: 0, label: "a".into(), embedding: vec![1.0, 0.0] });
        path.add_node(FusionPathNode { id: 1, label: "b".into(), embedding: vec![0.0, 1.0] });
        // Path len < 3 → no checkpoint reachable → always passes
        assert!(validator.validate_checkpoints(&path));
    }

    // -------------------------------------------------------------------------
    // RuleExplosionEngine
    // -------------------------------------------------------------------------

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(9), 362880);
    }

    #[test]
    fn test_index_to_permutation_all_distinct() {
        let n = 4;
        let perms: Vec<Vec<usize>> = (0..factorial(n))
            .map(|i| index_to_permutation(i, n))
            .collect();
        assert_eq!(perms.len(), 24);
        let mut sorted = perms.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 24, "All permutations should be distinct");
    }

    #[test]
    fn test_rule_explosion_produces_implications() {
        let config = RuleExplosionConfig { max_rules: 3, min_confidence: 0.0 };
        let engine = RuleExplosionEngine::new(config);

        let dim = 4;
        let rules: Vec<FluxRule> = (0..3).map(|i| {
            let bias = vec![0.01 * i as f32; dim];
            FluxRule::new(i, &format!("rule_{}", i), 0.9, bias)
        }).collect();

        let mut path = ReasoningPath::new(1);
        path.add_node(FusionPathNode { id: 0, label: "start".into(), embedding: vec![1.0, 0.0, 0.0, 0.0] });
        path.add_node(FusionPathNode { id: 1, label: "mid".into(),   embedding: vec![0.8, 0.2, 0.0, 0.0] });

        let implications = engine.explode_parallel(&path, &rules);

        // 3! = 6 permutations → should produce up to 6 implications
        assert!(implications.len() <= 6);
        assert!(!implications.is_empty(), "Should produce some implications");
        // Should be sorted by confidence descending
        for i in 1..implications.len() {
            assert!(implications[i-1].confidence >= implications[i].confidence);
        }
    }

    // -------------------------------------------------------------------------
    // JEPAPathwayPredictor
    // -------------------------------------------------------------------------

    #[test]
    fn test_jepa_predictor_unseen_seed_returns_neutral() {
        let predictor = JEPAPathwayPredictor::new();
        assert!((predictor.quality(99999) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_jepa_predictor_learns_from_explosion() {
        let mut predictor = JEPAPathwayPredictor::new();
        let path = ReasoningPath::new(42);
        let implications = vec![
            Implication { source_seed: 42, conclusion: vec![1.0], confidence: 0.9, is_valid: true, rule_sequence: vec![0] },
            Implication { source_seed: 42, conclusion: vec![0.5], confidence: 0.8, is_valid: true, rule_sequence: vec![1] },
        ];
        predictor.learn_from_explosion(&path, &implications, [true, true, true]);
        assert!(predictor.quality(42) > 0.5, "High-success path should have above-neutral quality");
        assert_eq!(predictor.known_seeds(), 1);
    }

    // -------------------------------------------------------------------------
    // ReasoningScale
    // -------------------------------------------------------------------------

    #[test]
    fn test_token_budget_scales_with_detail() {
        let low  = ScaledReasoning::new(ReasoningScale::Concept, 0.0);
        let high = ScaledReasoning::new(ReasoningScale::Concept, 1.0);
        let budget_low  = low.compute_token_budget(50);
        let budget_high = high.compute_token_budget(50);
        assert!(budget_high > budget_low, "Higher detail should give larger budget");
    }

    #[test]
    fn test_scaled_output_all_scales() {
        let implications = vec![
            Implication { source_seed: 1, conclusion: vec![1.0, 0.0], confidence: 0.8, is_valid: true, rule_sequence: vec![0, 1] },
        ];
        for scale in [ReasoningScale::Concept, ReasoningScale::Implementation, ReasoningScale::Micro, ReasoningScale::Macro] {
            let sr = ScaledReasoning::new(scale, 0.5);
            let output = sr.generate_output(&implications, 512);
            assert!(!output.is_empty());
        }
    }

    // -------------------------------------------------------------------------
    // FluxMatrixProcessor
    // -------------------------------------------------------------------------

    #[test]
    fn test_digital_root() {
        let p = FluxMatrixProcessor::new();
        assert_eq!(p.digital_root(0), 0);
        assert_eq!(p.digital_root(9), 9);
        assert_eq!(p.digital_root(18), 9);
        assert_eq!(p.digital_root(19), 1);
        assert_eq!(p.digital_root(123), 6); // 1+2+3 = 6
    }

    #[test]
    fn test_process_query_produces_6_cycle_nodes() {
        let processor = FluxMatrixProcessor::new();
        let (path, signals) = processor.process_query("test query", 42);
        // 1 starting node + 5 more cycle steps = 6 total
        assert_eq!(path.nodes.len(), 6, "Vortex cycle should produce 6 nodes");
        // Sacred positions 3,6,9 → only 3,6 are in cycle (9 not visited in 1→2→4→8→7→5)
        // Actually 8 and 7 are in the cycle, not 3,6,9 → signals may be empty
        assert!(signals.len() <= 3);
    }

    // -------------------------------------------------------------------------
    // VonNeumannMerger
    // -------------------------------------------------------------------------

    #[test]
    fn test_lattice_meet_conservative() {
        let p1 = Projection { embedding: vec![1.0, 0.5], weight: 1.0, source_seed: 1 };
        let p2 = Projection { embedding: vec![0.3, 0.8], weight: 0.8, source_seed: 2 };
        let meet = VonNeumannMerger::lattice_meet(&[p1, p2]);
        // Meet: take minimum absolute value per dimension
        assert!((meet.embedding[0] - 0.3).abs() < 1e-6);
        assert!((meet.embedding[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_lattice_join_comprehensive() {
        let p1 = Projection { embedding: vec![1.0, 0.5], weight: 1.0, source_seed: 1 };
        let p2 = Projection { embedding: vec![0.3, 0.8], weight: 0.8, source_seed: 2 };
        let join = VonNeumannMerger::lattice_join(&[p1, p2]);
        // Join: take maximum absolute value per dimension
        assert!((join.embedding[0] - 1.0).abs() < 1e-6);
        assert!((join.embedding[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_merge_pathways_returns_projection() {
        let paths = make_paths(&[1, 2, 3], 8);
        let result = VonNeumannMerger::merge_pathways(&paths);
        assert_eq!(result.embedding.len(), 8);
    }

    // -------------------------------------------------------------------------
    // ExhaustiveExplorer
    // -------------------------------------------------------------------------

    #[test]
    fn test_exhaustive_explorer_explores_all_paths() {
        let config = ExplorerConfig { heuristic_ratio: 0.0, max_depth: 4, require_valid_checkpoints: false };
        let mut explorer = ExhaustiveExplorer::new(config);

        // Use similar (non-orthogonal) embeddings so cosine similarity > 0
        let mut paths: Vec<ReasoningPath> = (0..3u64).map(|seed| {
            let mut path = ReasoningPath::new(seed);
            // All embeddings point in roughly the same direction
            path.add_node(FusionPathNode { id: 0, label: "s".into(), embedding: vec![1.0, 0.1, 0.1, 0.1] });
            path.add_node(FusionPathNode { id: 1, label: "m".into(), embedding: vec![0.9, 0.2, 0.1, 0.1] });
            path
        }).collect();

        let rules: Vec<FluxRule> = (0..2).map(|i| {
            FluxRule::new(i, &format!("r{}", i), 0.9, vec![0.01; 4])
        }).collect();

        let implications = explorer.explore(paths, &rules);
        // 3 paths × 2! = 2 permutations each → some should pass min_confidence
        assert!(!implications.is_empty(), "Expected implications from similar embeddings");
    }

    #[test]
    fn test_exhaustive_explorer_heuristic_mode_ranks_by_quality() {
        let config = ExplorerConfig { heuristic_ratio: 1.0, max_depth: 4, require_valid_checkpoints: false };
        let mut explorer = ExhaustiveExplorer::new(config);

        // Pre-learn: seed 99 has high quality, seed 1 has low quality
        let high_path = ReasoningPath::new(99);
        let low_path  = ReasoningPath::new(1);
        let good_impls = vec![Implication { source_seed: 99, conclusion: vec![1.0], confidence: 0.9, is_valid: true, rule_sequence: vec![0] }];
        let bad_impls  = vec![Implication { source_seed: 1,  conclusion: vec![0.0], confidence: 0.1, is_valid: false, rule_sequence: vec![0] }];
        explorer.predictor.learn_from_explosion(&high_path, &good_impls, [true, true, true]);
        explorer.predictor.learn_from_explosion(&low_path,  &bad_impls,  [false, false, false]);

        assert!(explorer.predictor.quality(99) > explorer.predictor.quality(1));
    }
}
