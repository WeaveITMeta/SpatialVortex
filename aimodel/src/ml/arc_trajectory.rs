//! Vortex-Aware Trajectory Buffer for ARC-AGI-3
//!
//! Every component orbits the same flux matrix: grid observations, actions,
//! rewards, and loop detection all carry vortex position metadata so the
//! policy head (a VortexDiffusion pass) receives geometrically coherent input.
//!
//! # Data flow
//!
//! ```text
//! env.step(action)
//!   → push_step(obs, action, reward, next_obs)
//!       ├─ hash grid (FNV-1a, deterministic)
//!       ├─ update state_graph (DiGraph<GridHash, ActionEdge>)
//!       ├─ detect loops → entropy spike for sacred gates
//!       ├─ JEPA record_state → straightening regularisation
//!       └─ trajectory_ctx (256-dim, recency-biased mean)
//!   → get_trajectory_ctx()     // feeds PolicyHead position 8/7/5
//!   → get_novelty_signal()     // feeds sacred gate entropy
//!   → get_entropy_for_gates()  // direct gate intervention signal
//! ```

use std::collections::HashMap;

use super::jepa::TemporalStraighteningEngine;
use super::flux_object_macro::{
    FLUX_VORTEX_CYCLE, FLUX_SACRED_POSITIONS, next_vortex_position,
};
use super::sacred_moe::{PHI, PHI_INV};

// ─── Constants ───────────────────────────────────────────────────────────────

pub const EMBED_DIM: usize = 256;
/// JEPA sliding window for trajectory straightening.
const JEPA_WINDOW: usize = 16;
/// Regularisation strength for temporal straightening.
const JEPA_LAMBDA: f32 = 0.1;
/// Confidence decay per vortex cycle step.
const CONFIDENCE_DECAY: f32 = 0.9;
/// Entropy spike magnitude when a loop is detected.
const LOOP_ENTROPY_SPIKE: f32 = 0.6;
/// Base entropy (no loops, high novelty).
const BASE_ENTROPY: f32 = 0.05;
/// Maximum steps before forced flow reversal signal.
const MAX_STEPS_BEFORE_REVERSAL: usize = 50;

// ─── Grid Hash (FNV-1a, deterministic) ──────────────────────────────────────

/// 64-bit FNV-1a hash of flattened grid bytes. Deterministic within and across
/// runs (unlike std DefaultHasher which may be randomised).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridHash(pub u64);

impl GridHash {
    /// Hash a 2D grid of cell values (0–9). Row-major, width-prefixed.
    pub fn from_grid(cells: &[Vec<u8>]) -> Self {
        const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
        const FNV_PRIME: u64 = 0x0100_0000_01b3;

        let mut h = FNV_OFFSET;
        // Include dimensions so that differently-shaped grids with the same
        // flattened bytes produce distinct hashes.
        let height = cells.len() as u64;
        let width = cells.first().map_or(0, |r| r.len()) as u64;
        for &b in &height.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(FNV_PRIME);
        }
        for &b in &width.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(FNV_PRIME);
        }
        for row in cells {
            for &cell in row {
                h ^= cell as u64;
                h = h.wrapping_mul(FNV_PRIME);
            }
        }
        Self(h)
    }
}

// ─── Vortex Position ─────────────────────────────────────────────────────────

/// 10-slot flux matrix position. Positions 1–9 are active; Pos0 is the
/// quiescent / unassigned origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VortexPosition {
    Pos0 = 0,
    Pos1 = 1,
    Pos2 = 2,
    Pos3 = 3,
    Pos4 = 4,
    Pos5 = 5,
    Pos6 = 6,
    Pos7 = 7,
    Pos8 = 8,
    Pos9 = 9,
}

impl VortexPosition {
    pub fn from_cycle_step(step: usize) -> Self {
        let cycle_pos = FLUX_VORTEX_CYCLE[step % FLUX_VORTEX_CYCLE.len()];
        Self::from_u8(cycle_pos)
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Pos0,
            1 => Self::Pos1,
            2 => Self::Pos2,
            3 => Self::Pos3,
            4 => Self::Pos4,
            5 => Self::Pos5,
            6 => Self::Pos6,
            7 => Self::Pos7,
            8 => Self::Pos8,
            9 => Self::Pos9,
            _ => Self::Pos0,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Is this a sacred gate position (3, 6, or 9)?
    pub fn is_sacred(self) -> bool {
        FLUX_SACRED_POSITIONS.contains(&self.as_u8())
    }

    /// Sacred gate weight (3→1.15 Unity, 6→1.10 Heart, 9→1.20 Ultimate).
    pub fn sacred_boost(self) -> f32 {
        match self.as_u8() {
            3 => 1.15,
            6 => 1.10,
            9 => 1.20,
            _ => 1.0,
        }
    }
}

// ─── Flow Direction ──────────────────────────────────────────────────────────

/// Vortex flow direction. Forward follows the cycle 1→2→4→8→7→5;
/// Reversed backtracks on high entropy / detected loops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowDir {
    Forward,
    Reversed,
}

// ─── Flux Embedding ──────────────────────────────────────────────────────────

/// 10-position flux matrix state. Each position holds a 256-dim embedding
/// slice; sacred positions carry verification metadata.
#[derive(Debug, Clone)]
pub struct FluxEmbedding {
    /// Embeddings at each flux matrix position [10 × EMBED_DIM].
    pub positions: Vec<Vec<f32>>,
    /// Per-position entropy (computed at sacred gates).
    pub entropy: [f32; 10],
    /// Current flow direction.
    pub flow_direction: FlowDir,
    /// Which vortex cycle step we're at (index into FLUX_VORTEX_CYCLE).
    pub cycle_index: usize,
}

impl FluxEmbedding {
    /// Create a new flux embedding seeded from a grid encoding.
    ///
    /// Distribution strategy:
    /// - Positions 1/2/4: spatial expansion (grid features, split by frequency band)
    /// - Positions 8/7/5: contraction slots (start empty, filled by trajectory ctx)
    /// - Positions 3/6/9: sacred verification (seeded with rule hypothesis)
    /// - Position 0: quiescent origin (zero)
    pub fn from_grid_encoding(grid_embed: &[f32; EMBED_DIM]) -> Self {
        let mut positions = vec![vec![0.0f32; EMBED_DIM]; 10];

        // Expansion: distribute grid encoding across positions 1, 2, 4
        // Each gets the full embedding scaled by vortex doubling pattern
        let scales = [(1, 1.0f32), (2, PHI_INV), (4, PHI_INV * PHI_INV)];
        for &(pos, scale) in &scales {
            for d in 0..EMBED_DIM {
                positions[pos][d] = grid_embed[d] * scale;
            }
        }
        // Position 8: dimmed copy (start of contraction)
        for d in 0..EMBED_DIM {
            positions[8][d] = grid_embed[d] * 0.1;
        }

        Self {
            positions,
            entropy: [BASE_ENTROPY; 10],
            flow_direction: FlowDir::Forward,
            cycle_index: 0,
        }
    }

    /// Seed contraction slots (8/7/5) with trajectory context.
    pub fn seed_trajectory(&mut self, trajectory_ctx: &[f32; EMBED_DIM]) {
        let contraction = [(8, 1.0f32), (7, PHI_INV), (5, PHI_INV * PHI_INV)];
        for &(pos, scale) in &contraction {
            for d in 0..EMBED_DIM {
                self.positions[pos][d] = trajectory_ctx[d] * scale;
            }
        }
    }

    /// Seed sacred gates (3/6/9) with rule hypothesis embedding.
    pub fn seed_rule_hypothesis(&mut self, rule_embed: &[f32; EMBED_DIM]) {
        for &pos in &[3usize, 6, 9] {
            let boost = VortexPosition::from_u8(pos as u8).sacred_boost();
            for d in 0..EMBED_DIM {
                self.positions[pos][d] = rule_embed[d] * boost;
            }
        }
    }

    /// Inject entropy at sacred gates (from loop detection / novelty signal).
    pub fn inject_entropy(&mut self, entropy: f32) {
        for &pos in &[3usize, 6, 9] {
            self.entropy[pos] = entropy.clamp(0.0, 1.0);
        }
        // High entropy triggers flow reversal
        if entropy > 0.5 {
            self.flow_direction = FlowDir::Reversed;
        }
    }

    /// Advance one step through the vortex cycle, propagating embeddings.
    /// Returns the current vortex position after advancement.
    pub fn advance_cycle(&mut self) -> VortexPosition {
        let current_pos = FLUX_VORTEX_CYCLE[self.cycle_index % FLUX_VORTEX_CYCLE.len()];

        let next = match self.flow_direction {
            FlowDir::Forward => {
                self.cycle_index = (self.cycle_index + 1) % FLUX_VORTEX_CYCLE.len();
                next_vortex_position(current_pos).unwrap_or(1)
            }
            FlowDir::Reversed => {
                // Reverse: walk cycle backwards
                if self.cycle_index == 0 {
                    self.cycle_index = FLUX_VORTEX_CYCLE.len() - 1;
                } else {
                    self.cycle_index -= 1;
                }
                FLUX_VORTEX_CYCLE[self.cycle_index]
            }
        };

        // Propagate: blend current position into next with confidence decay
        let next_idx = next as usize;
        let curr_idx = current_pos as usize;
        if next_idx < 10 && curr_idx < 10 {
            for d in 0..EMBED_DIM {
                self.positions[next_idx][d] = self.positions[next_idx][d] * CONFIDENCE_DECAY
                    + self.positions[curr_idx][d] * (1.0 - CONFIDENCE_DECAY);
            }
        }

        VortexPosition::from_u8(next)
    }

    /// Read the pooled embedding from position 1 (manifest slot) after a
    /// full vortex cycle.
    pub fn manifest_embedding(&self) -> [f32; EMBED_DIM] {
        let mut out = [0.0f32; EMBED_DIM];
        out.copy_from_slice(&self.positions[1][..EMBED_DIM]);
        out
    }
}

// ─── Arc Grid State ──────────────────────────────────────────────────────────

/// Grid observation with vortex metadata.
#[derive(Debug, Clone)]
pub struct ArcGridState {
    /// Deterministic hash for exact loop detection.
    pub grid_hash: GridHash,
    /// 256-dim embedding from GridEncoder (Conv2D + attention + flux mapping).
    pub embedding: [f32; EMBED_DIM],
    /// Which vortex position this state was observed at.
    pub vortex_pos: VortexPosition,
}

impl ArcGridState {
    pub fn new(cells: &[Vec<u8>], embedding: [f32; EMBED_DIM], vortex_pos: VortexPosition) -> Self {
        Self {
            grid_hash: GridHash::from_grid(cells),
            embedding,
            vortex_pos,
        }
    }
}

// ─── Action Edge (for state graph) ──────────────────────────────────────────

/// Discrete action identifier. Wraps the arcengine GameAction as a u32 ID
/// plus optional grid coordinates for click/place actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionId(pub u32);

/// Edge in the state transition graph: action taken + outcome metadata.
#[derive(Debug, Clone)]
pub struct ActionEdge {
    pub action: ActionId,
    pub reward: f32,
    /// Did the vortex cycle reverse during this transition?
    pub flow_reversed: bool,
    /// Vortex position at which the action was selected.
    pub vortex_pos: VortexPosition,
}

// ─── Trajectory Step ─────────────────────────────────────────────────────────

/// Single step in an episode, carrying full vortex context.
#[derive(Debug, Clone)]
pub struct TrajectoryStep {
    pub grid_state: ArcGridState,
    pub action: ActionId,
    pub reward: f32,
    pub next_state: Option<ArcGridState>,
    /// Which step in the vortex cycle this action was selected at.
    pub cycle_step: usize,
    /// Flow direction at decision time.
    pub flow_dir: FlowDir,
    /// Entropy at sacred gates when the action was chosen.
    pub gate_entropy: f32,
}

// ─── State Graph (lightweight directed graph) ────────────────────────────────

/// Directed graph where nodes are grid hashes and edges are actions taken.
/// Used for loop detection and planning.
#[derive(Debug, Clone, Default)]
pub struct StateGraph {
    /// Adjacency list: source_hash → Vec<(target_hash, edge)>.
    pub edges: HashMap<GridHash, Vec<(GridHash, ActionEdge)>>,
    /// Reverse adjacency for backtracking.
    pub reverse: HashMap<GridHash, Vec<(GridHash, ActionEdge)>>,
    /// Number of nodes.
    pub node_count: usize,
    nodes: HashMap<GridHash, bool>, // hash → seen
}

impl StateGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a transition. Returns `true` if the target node was already visited
    /// (i.e., this transition creates a loop).
    pub fn add_edge(
        &mut self,
        from: GridHash,
        to: GridHash,
        edge: ActionEdge,
    ) -> bool {
        // Register nodes
        if !self.nodes.contains_key(&from) {
            self.nodes.insert(from, true);
            self.node_count += 1;
        }
        let is_revisit = self.nodes.contains_key(&to);
        if !is_revisit {
            self.nodes.insert(to, true);
            self.node_count += 1;
        }

        // Forward edge
        self.edges.entry(from).or_default().push((to, edge.clone()));
        // Reverse edge
        self.reverse.entry(to).or_default().push((from, edge));

        is_revisit
    }

    /// Check if a grid state has been visited before.
    pub fn is_visited(&self, hash: &GridHash) -> bool {
        self.nodes.contains_key(hash)
    }

    /// Count how many distinct actions have been tried from a given state.
    pub fn actions_tried_from(&self, hash: &GridHash) -> usize {
        self.edges.get(hash).map_or(0, |v| v.len())
    }

    /// Find all states reachable from a given state (BFS, max depth).
    pub fn reachable_from(&self, start: GridHash, max_depth: usize) -> Vec<(GridHash, usize)> {
        let mut visited = HashMap::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start, 0usize));
        visited.insert(start, 0);

        while let Some((node, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            if let Some(neighbours) = self.edges.get(&node) {
                for (target, _) in neighbours {
                    if !visited.contains_key(target) {
                        visited.insert(*target, depth + 1);
                        queue.push_back((*target, depth + 1));
                    }
                }
            }
        }
        visited.into_iter().collect()
    }

    pub fn clear(&mut self) {
        self.edges.clear();
        self.reverse.clear();
        self.nodes.clear();
        self.node_count = 0;
    }
}

// ─── Completed Episode Summary ───────────────────────────────────────────────

/// Meta-patterns extracted from a completed episode for cross-episode learning.
#[derive(Debug, Clone)]
pub struct CompletedEpisode {
    pub task_id: String,
    pub total_steps: u32,
    pub goal_reached: bool,
    pub final_score: f32,
    /// Fraction of steps where flow was reversed (exploration pressure).
    pub reversal_ratio: f32,
    /// Which sacred gate triggered the most reversals.
    pub dominant_gate: VortexPosition,
    /// Actions that appeared in the successful trajectory (if goal_reached).
    pub success_actions: Vec<ActionId>,
    /// Average trajectory curvature (from JEPA straightening).
    pub avg_curvature: f32,
    /// Number of loop detections during the episode.
    pub loop_count: u32,
}

// ─── Current Episode ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct CurrentEpisode {
    steps: Vec<TrajectoryStep>,
    visit_counts: HashMap<GridHash, u32>,
    reversal_count: u32,
    loop_count: u32,
    /// Task ID for this episode.
    task_id: String,
}

impl CurrentEpisode {
    fn new(task_id: String) -> Self {
        Self {
            steps: Vec::with_capacity(128),
            visit_counts: HashMap::new(),
            reversal_count: 0,
            loop_count: 0,
            task_id,
        }
    }

    fn record_visit(&mut self, hash: GridHash) -> u32 {
        let count = self.visit_counts.entry(hash).or_insert(0);
        *count += 1;
        *count
    }

    fn clear(&mut self) {
        self.steps.clear();
        self.visit_counts.clear();
        self.reversal_count = 0;
        self.loop_count = 0;
        self.task_id.clear();
    }
}

// ─── Trajectory Buffer ───────────────────────────────────────────────────────

/// Vortex-aware trajectory buffer. Connects GridEncoder observations, JEPA
/// temporal straightening, and the state transition graph into a single
/// structure that feeds the PolicyHead and sacred gates.
#[derive(Debug)]
pub struct TrajectoryBuffer {
    current: CurrentEpisode,
    /// Cross-episode memory: meta-patterns only (no raw trajectories).
    episode_memory: Vec<CompletedEpisode>,
    /// Maximum completed episodes to retain.
    max_memory: usize,
    /// JEPA temporal straightening for trajectory context extraction.
    jepa: TemporalStraighteningEngine,
    /// Cumulative JEPA curvature for the current episode.
    curvature_sum: f64,
    curvature_count: usize,
    /// Sliding window of recent embeddings (parallel to JEPA, for ctx extraction).
    embedding_window: Vec<[f32; EMBED_DIM]>,
    /// State transition graph for the current episode.
    state_graph: StateGraph,
    /// Current entropy level (accumulated from loop detections).
    current_entropy: f32,
    /// Current vortex cycle index.
    cycle_index: usize,
}

impl TrajectoryBuffer {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            current: CurrentEpisode::new(task_id.into()),
            episode_memory: Vec::new(),
            max_memory: 64,
            jepa: TemporalStraighteningEngine::new(JEPA_WINDOW, JEPA_LAMBDA),
            curvature_sum: 0.0,
            curvature_count: 0,
            embedding_window: Vec::with_capacity(JEPA_WINDOW),
            state_graph: StateGraph::new(),
            current_entropy: BASE_ENTROPY,
            cycle_index: 0,
        }
    }

    // ── Core API ─────────────────────────────────────────────────────────

    /// Record one environment step. This is the main entry point called after
    /// `env.step(action)` returns.
    ///
    /// Returns `(is_loop, entropy)` — the loop flag and current entropy level
    /// for sacred gate injection.
    pub fn push_step(
        &mut self,
        obs_cells: &[Vec<u8>],
        obs_embedding: [f32; EMBED_DIM],
        action: ActionId,
        reward: f32,
        next_cells: Option<&[Vec<u8>]>,
        next_embedding: Option<[f32; EMBED_DIM]>,
    ) -> (bool, f32) {
        let vortex_pos = VortexPosition::from_cycle_step(self.cycle_index);

        // ── Hash & state ────────────────────────────────────────────────
        let grid_hash = GridHash::from_grid(obs_cells);
        let grid_state = ArcGridState {
            grid_hash,
            embedding: obs_embedding,
            vortex_pos,
        };

        let next_state = next_cells.map(|nc| {
            let nh = GridHash::from_grid(nc);
            ArcGridState {
                grid_hash: nh,
                embedding: next_embedding.unwrap_or([0.0; EMBED_DIM]),
                vortex_pos: VortexPosition::from_cycle_step(self.cycle_index + 1),
            }
        });

        // ── Loop detection via state graph ──────────────────────────────
        let is_loop = if let Some(ref ns) = next_state {
            let edge = ActionEdge {
                action,
                reward,
                flow_reversed: self.current_entropy > 0.5,
                vortex_pos,
            };
            self.state_graph.add_edge(grid_hash, ns.grid_hash, edge)
        } else {
            false
        };

        // ── Visit counting ──────────────────────────────────────────────
        let visit_count = self.current.record_visit(grid_hash);

        // ── Entropy update ──────────────────────────────────────────────
        if is_loop {
            self.current.loop_count += 1;
            // Entropy spike: scaled by visit count (more revisits = higher spike)
            let spike = LOOP_ENTROPY_SPIKE * (1.0 + (visit_count as f32 - 2.0).max(0.0) * 0.2);
            self.current_entropy = (self.current_entropy + spike).clamp(0.0, 1.0);
        } else {
            // Gradual entropy decay toward baseline when exploring novel states
            self.current_entropy = self.current_entropy * 0.95 + BASE_ENTROPY * 0.05;
        }

        // Step-count pressure: entropy rises as episode gets long
        let step_pressure = (self.current.steps.len() as f32 / MAX_STEPS_BEFORE_REVERSAL as f32)
            .clamp(0.0, 0.5);
        self.current_entropy = (self.current_entropy + step_pressure * 0.01).clamp(0.0, 1.0);

        // ── Flow direction ──────────────────────────────────────────────
        let flow_dir = if self.current_entropy > 0.5 {
            self.current.reversal_count += 1;
            FlowDir::Reversed
        } else {
            FlowDir::Forward
        };

        // ── JEPA temporal straightening ─────────────────────────────────
        if let Some(curvature) = self.jepa.record_state(&obs_embedding) {
            self.curvature_sum += curvature as f64;
            self.curvature_count += 1;
        }

        // ── Embedding window (for ctx extraction) ───────────────────────
        self.embedding_window.push(obs_embedding);
        if self.embedding_window.len() > JEPA_WINDOW {
            self.embedding_window.remove(0);
        }

        // ── Record step ─────────────────────────────────────────────────
        self.current.steps.push(TrajectoryStep {
            grid_state,
            action,
            reward,
            next_state,
            cycle_step: self.cycle_index,
            flow_dir,
            gate_entropy: self.current_entropy,
        });

        // Advance vortex cycle
        self.cycle_index = (self.cycle_index + 1) % FLUX_VORTEX_CYCLE.len();

        (is_loop, self.current_entropy)
    }

    /// Extract trajectory context: 256-dim embedding suitable for seeding the
    /// PolicyHead contraction slots (positions 8/7/5).
    ///
    /// Uses recency-biased weighted mean over the JEPA-straightened window.
    /// More recent states contribute more (exponential decay).
    pub fn get_trajectory_ctx(&self) -> [f32; EMBED_DIM] {
        if self.embedding_window.is_empty() {
            return [0.0; EMBED_DIM];
        }

        let n = self.embedding_window.len();
        let mut ctx = [0.0f32; EMBED_DIM];
        let mut weight_sum = 0.0f32;

        for (i, emb) in self.embedding_window.iter().enumerate() {
            // Exponential recency: latest state gets weight ≈ 1.0, oldest ≈ 0.1
            let recency = ((i as f32 + 1.0) / n as f32).powf(PHI);
            for d in 0..EMBED_DIM {
                ctx[d] += emb[d] * recency;
            }
            weight_sum += recency;
        }

        if weight_sum > 0.0 {
            let inv = 1.0 / weight_sum;
            for d in 0..EMBED_DIM {
                ctx[d] *= inv;
            }
        }
        ctx
    }

    /// Novelty signal: 0.0 = fully explored / looping, 1.0 = completely novel.
    /// Used for reward shaping and sacred gate modulation.
    pub fn get_novelty_signal(&self) -> f32 {
        if self.current.steps.is_empty() {
            return 1.0;
        }

        // Ratio of unique states to total steps
        let unique = self.state_graph.node_count as f32;
        let total = self.current.steps.len() as f32;
        let visit_ratio = (unique / total).clamp(0.0, 1.0);

        // Inverse entropy (high entropy = low novelty)
        let inv_entropy = 1.0 - self.current_entropy;

        // φ-weighted blend
        PHI_INV * visit_ratio + (1.0 - PHI_INV) * inv_entropy
    }

    /// Entropy level for direct injection into sacred gates.
    /// High entropy → gates become strict → flow reversal → explore novel actions.
    pub fn get_entropy_for_gates(&self) -> f32 {
        self.current_entropy
    }

    /// Current vortex position in the cycle.
    pub fn current_vortex_position(&self) -> VortexPosition {
        VortexPosition::from_cycle_step(self.cycle_index)
    }

    /// Number of steps in the current episode.
    pub fn step_count(&self) -> usize {
        self.current.steps.len()
    }

    /// Has the current grid state been visited before?
    pub fn is_state_visited(&self, cells: &[Vec<u8>]) -> bool {
        let hash = GridHash::from_grid(cells);
        self.state_graph.is_visited(&hash)
    }

    /// How many distinct actions have been tried from the current state?
    pub fn actions_tried_from_current(&self) -> usize {
        self.current.steps.last().map_or(0, |step| {
            self.state_graph.actions_tried_from(&step.grid_state.grid_hash)
        })
    }

    /// Build a FluxEmbedding seeded with current grid + trajectory + rule hypothesis.
    /// This is the full input to the PolicyHead vortex diffusion pass.
    pub fn build_flux_embedding(
        &self,
        grid_embed: &[f32; EMBED_DIM],
        rule_hypothesis: Option<&[f32; EMBED_DIM]>,
    ) -> FluxEmbedding {
        let mut flux = FluxEmbedding::from_grid_encoding(grid_embed);

        // Seed contraction slots with trajectory context
        let traj_ctx = self.get_trajectory_ctx();
        flux.seed_trajectory(&traj_ctx);

        // Seed sacred gates with rule hypothesis (if available)
        if let Some(rule) = rule_hypothesis {
            flux.seed_rule_hypothesis(rule);
        }

        // Inject loop/novelty entropy
        flux.inject_entropy(self.current_entropy);

        flux
    }

    // ── Episode lifecycle ────────────────────────────────────────────────

    /// Complete the current episode and extract meta-patterns into memory.
    /// Returns the episode summary.
    pub fn complete_episode(
        &mut self,
        goal_reached: bool,
        final_score: f32,
    ) -> CompletedEpisode {
        let total_steps = self.current.steps.len() as u32;

        // Determine which gate caused the most reversals
        let dominant_gate = self.find_dominant_reversal_gate();

        // Extract success actions
        let success_actions = if goal_reached {
            self.current.steps.iter().map(|s| s.action).collect()
        } else {
            Vec::new()
        };

        // Average curvature
        let avg_curvature = if self.curvature_count > 0 {
            (self.curvature_sum / self.curvature_count as f64) as f32
        } else {
            0.0
        };

        let reversal_ratio = if total_steps > 0 {
            self.current.reversal_count as f32 / total_steps as f32
        } else {
            0.0
        };

        let summary = CompletedEpisode {
            task_id: self.current.task_id.clone(),
            total_steps,
            goal_reached,
            final_score,
            reversal_ratio,
            dominant_gate,
            success_actions,
            avg_curvature,
            loop_count: self.current.loop_count,
        };

        // Store in memory (evict oldest if full)
        if self.episode_memory.len() >= self.max_memory {
            self.episode_memory.remove(0);
        }
        self.episode_memory.push(summary.clone());

        // Reset for next episode
        self.reset_episode();

        summary
    }

    /// Start a new episode (reset all per-episode state).
    pub fn reset_episode(&mut self) {
        self.current.clear();
        self.jepa.reset_trajectory();
        self.jepa.reset_stats();
        self.curvature_sum = 0.0;
        self.curvature_count = 0;
        self.embedding_window.clear();
        self.state_graph.clear();
        self.current_entropy = BASE_ENTROPY;
        self.cycle_index = 0;
    }

    /// Start a fresh episode with a specific task ID.
    pub fn start_episode(&mut self, task_id: impl Into<String>) {
        self.reset_episode();
        self.current.task_id = task_id.into();
    }

    // ── Cross-episode queries ────────────────────────────────────────────

    /// Success rate for a specific task across all remembered episodes.
    pub fn task_success_rate(&self, task_id: &str) -> f32 {
        let episodes: Vec<_> = self
            .episode_memory
            .iter()
            .filter(|e| e.task_id == task_id)
            .collect();
        if episodes.is_empty() {
            return 0.0;
        }
        let successes = episodes.iter().filter(|e| e.goal_reached).count();
        successes as f32 / episodes.len() as f32
    }

    /// Actions that appeared most frequently in successful episodes for a task.
    /// Returns (action_id, frequency) pairs sorted by frequency.
    pub fn success_action_frequencies(&self, task_id: &str) -> Vec<(ActionId, u32)> {
        let mut counts: HashMap<ActionId, u32> = HashMap::new();
        for ep in &self.episode_memory {
            if ep.task_id == task_id && ep.goal_reached {
                for &action in &ep.success_actions {
                    *counts.entry(action).or_insert(0) += 1;
                }
            }
        }
        let mut freqs: Vec<_> = counts.into_iter().collect();
        freqs.sort_by(|a, b| b.1.cmp(&a.1));
        freqs
    }

    /// Average reversal ratio across all episodes (meta-pattern: how much
    /// exploration pressure is typical).
    pub fn avg_reversal_ratio(&self) -> f32 {
        if self.episode_memory.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.episode_memory.iter().map(|e| e.reversal_ratio).sum();
        sum / self.episode_memory.len() as f32
    }

    /// Number of completed episodes in memory.
    pub fn memory_size(&self) -> usize {
        self.episode_memory.len()
    }

    // ── Internal helpers ─────────────────────────────────────────────────

    /// Find which sacred gate position (3, 6, or 9) was most associated with
    /// flow reversals in the current episode.
    fn find_dominant_reversal_gate(&self) -> VortexPosition {
        let mut gate_reversals = [0u32; 10];
        for step in &self.current.steps {
            if step.flow_dir == FlowDir::Reversed {
                let pos = step.grid_state.vortex_pos.as_u8() as usize;
                // Attribute to the nearest sacred gate
                let nearest_sacred = match pos {
                    0..=3 => 3,
                    4..=6 => 6,
                    _ => 9,
                };
                gate_reversals[nearest_sacred] += 1;
            }
        }
        // Find max among sacred positions
        let dominant = FLUX_SACRED_POSITIONS
            .iter()
            .max_by_key(|&&p| gate_reversals[p as usize])
            .copied()
            .unwrap_or(3);
        VortexPosition::from_u8(dominant)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_a() -> Vec<Vec<u8>> {
        vec![vec![0, 1, 2], vec![3, 0, 5], vec![6, 7, 0]]
    }
    fn grid_b() -> Vec<Vec<u8>> {
        vec![vec![1, 1, 1], vec![0, 0, 0], vec![2, 2, 2]]
    }
    fn embed_from_seed(seed: f32) -> [f32; EMBED_DIM] {
        let mut e = [0.0f32; EMBED_DIM];
        for d in 0..EMBED_DIM {
            e[d] = ((d as f32 + seed) * 0.01).sin();
        }
        e
    }

    #[test]
    fn test_grid_hash_deterministic() {
        let h1 = GridHash::from_grid(&grid_a());
        let h2 = GridHash::from_grid(&grid_a());
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_grid_hash_distinct() {
        let h1 = GridHash::from_grid(&grid_a());
        let h2 = GridHash::from_grid(&grid_b());
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_vortex_position_cycle() {
        let positions: Vec<u8> = (0..7)
            .map(|i| VortexPosition::from_cycle_step(i).as_u8())
            .collect();
        assert_eq!(positions, vec![1, 2, 4, 8, 7, 5, 1]);
    }

    #[test]
    fn test_sacred_detection() {
        assert!(VortexPosition::Pos3.is_sacred());
        assert!(VortexPosition::Pos6.is_sacred());
        assert!(VortexPosition::Pos9.is_sacred());
        assert!(!VortexPosition::Pos1.is_sacred());
        assert!(!VortexPosition::Pos4.is_sacred());
    }

    #[test]
    fn test_push_step_no_loop() {
        let mut buf = TrajectoryBuffer::new("test_task");
        let emb_a = embed_from_seed(1.0);
        let emb_b = embed_from_seed(2.0);
        let (is_loop, _entropy) = buf.push_step(
            &grid_a(), emb_a, ActionId(1), 0.0,
            Some(&grid_b()), Some(emb_b),
        );
        assert!(!is_loop);
        assert_eq!(buf.step_count(), 1);
    }

    #[test]
    fn test_loop_detection() {
        let mut buf = TrajectoryBuffer::new("test_task");
        let emb_a = embed_from_seed(1.0);
        let emb_b = embed_from_seed(2.0);

        // Step 1: A → B (no loop)
        let (loop1, _) = buf.push_step(
            &grid_a(), emb_a, ActionId(1), 0.0,
            Some(&grid_b()), Some(emb_b),
        );
        assert!(!loop1);

        // Step 2: B → A (loop! A was already visited)
        let (loop2, entropy) = buf.push_step(
            &grid_b(), emb_b, ActionId(2), 0.0,
            Some(&grid_a()), Some(emb_a),
        );
        assert!(loop2);
        assert!(entropy > BASE_ENTROPY, "Entropy should spike on loop");
    }

    #[test]
    fn test_trajectory_ctx_shape() {
        let mut buf = TrajectoryBuffer::new("test_task");
        for i in 0..5 {
            let cells = vec![vec![i as u8; 3]; 3];
            let emb = embed_from_seed(i as f32);
            buf.push_step(&cells, emb, ActionId(i), 0.0, None, None);
        }
        let ctx = buf.get_trajectory_ctx();
        assert_eq!(ctx.len(), EMBED_DIM);
        assert!(ctx.iter().any(|&v| v != 0.0), "Context should be non-zero");
    }

    #[test]
    fn test_novelty_signal_degrades_on_loops() {
        let mut buf = TrajectoryBuffer::new("test_task");
        let emb_a = embed_from_seed(1.0);
        let emb_b = embed_from_seed(2.0);

        // Novel exploration
        buf.push_step(&grid_a(), emb_a, ActionId(1), 0.0, Some(&grid_b()), Some(emb_b));
        let novelty_before = buf.get_novelty_signal();

        // Loop back
        buf.push_step(&grid_b(), emb_b, ActionId(2), 0.0, Some(&grid_a()), Some(emb_a));
        let novelty_after = buf.get_novelty_signal();

        assert!(
            novelty_after < novelty_before,
            "Novelty should decrease on loop: {novelty_before} → {novelty_after}"
        );
    }

    #[test]
    fn test_complete_episode() {
        let mut buf = TrajectoryBuffer::new("task_1");
        let emb = embed_from_seed(1.0);
        buf.push_step(&grid_a(), emb, ActionId(1), 0.5, None, None);
        buf.push_step(&grid_b(), embed_from_seed(2.0), ActionId(2), 1.0, None, None);

        let summary = buf.complete_episode(true, 0.85);
        assert_eq!(summary.task_id, "task_1");
        assert_eq!(summary.total_steps, 2);
        assert!(summary.goal_reached);
        assert_eq!(summary.success_actions.len(), 2);
        assert_eq!(buf.memory_size(), 1);
        assert_eq!(buf.step_count(), 0); // reset after completion
    }

    #[test]
    fn test_success_action_frequencies() {
        let mut buf = TrajectoryBuffer::new("t1");
        buf.push_step(&grid_a(), embed_from_seed(1.0), ActionId(5), 1.0, None, None);
        buf.push_step(&grid_b(), embed_from_seed(2.0), ActionId(5), 1.0, None, None);
        buf.complete_episode(true, 1.0);

        buf.start_episode("t1");
        buf.push_step(&grid_a(), embed_from_seed(1.0), ActionId(5), 1.0, None, None);
        buf.push_step(&grid_b(), embed_from_seed(2.0), ActionId(3), 1.0, None, None);
        buf.complete_episode(true, 1.0);

        let freqs = buf.success_action_frequencies("t1");
        // ActionId(5) appeared 3 times, ActionId(3) appeared 1 time
        assert_eq!(freqs[0].0, ActionId(5));
        assert_eq!(freqs[0].1, 3);
    }

    #[test]
    fn test_flux_embedding_seeding() {
        let mut buf = TrajectoryBuffer::new("t");
        let emb = embed_from_seed(1.0);
        buf.push_step(&grid_a(), emb, ActionId(1), 0.0, None, None);

        let rule = embed_from_seed(42.0);
        let flux = buf.build_flux_embedding(&emb, Some(&rule));

        // Expansion positions should be non-zero
        assert!(flux.positions[1].iter().any(|&v| v != 0.0));
        assert!(flux.positions[2].iter().any(|&v| v != 0.0));
        assert!(flux.positions[4].iter().any(|&v| v != 0.0));
        // Sacred gates should be seeded with rule hypothesis
        assert!(flux.positions[3].iter().any(|&v| v != 0.0));
        assert!(flux.positions[6].iter().any(|&v| v != 0.0));
        assert!(flux.positions[9].iter().any(|&v| v != 0.0));
    }

    #[test]
    fn test_entropy_triggers_reversal() {
        let mut buf = TrajectoryBuffer::new("t");
        let emb_a = embed_from_seed(1.0);
        let emb_b = embed_from_seed(2.0);

        // Force multiple loops to spike entropy above 0.5
        for _ in 0..5 {
            buf.push_step(&grid_a(), emb_a, ActionId(1), 0.0, Some(&grid_b()), Some(emb_b));
            buf.push_step(&grid_b(), emb_b, ActionId(2), 0.0, Some(&grid_a()), Some(emb_a));
        }

        let flux = buf.build_flux_embedding(&emb_a, None);
        assert_eq!(flux.flow_direction, FlowDir::Reversed,
            "High entropy should trigger flow reversal");
    }

    #[test]
    fn test_state_graph_reachability() {
        let mut graph = StateGraph::new();
        let h1 = GridHash(1);
        let h2 = GridHash(2);
        let h3 = GridHash(3);
        let edge = ActionEdge {
            action: ActionId(0),
            reward: 0.0,
            flow_reversed: false,
            vortex_pos: VortexPosition::Pos1,
        };
        graph.add_edge(h1, h2, edge.clone());
        graph.add_edge(h2, h3, edge);

        let reachable = graph.reachable_from(h1, 5);
        assert!(reachable.len() >= 3); // h1, h2, h3
    }
}
