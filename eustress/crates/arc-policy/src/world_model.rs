//! VortexWorldModel — Phase 3 world model for ARC-AGI-3 interactive games.
//!
//! Uses the full Eustress vortex-core infrastructure:
//! - Grid2D (WorldState) for frame representation
//! - CausalGraph for learning game mechanics (Rules → Laws)
//! - HypothesisTree for multi-step planning with branching
//! - solve() loop: OBSERVE → HYPOTHESIZE → SIMULATE → EVALUATE → INTERNALIZE
//! - IRA (Internal Representation Accuracy) for prediction quality tracking
//!
//! The model learns what each game action does to the grid by observing
//! frame deltas, builds causal rules, and uses the solve loop to plan
//! actions that produce desired structural transformations.

use crate::scene_mirror::{ArcSceneDelta, ArcSceneMirror, ArcMirrorConfig};
use crate::symbolic_decomposer::SymbolicActionDecomposer;
use crate::object_tracker::{
    ObjectTracker, ObjectEvent, ObjectFrame, ObjectActionModel, StableObjectId,
};
use crate::exploration::{UndoExplorer, GridHash};
use crate::goal_inference::GoalInferenceEngine;
use crate::level_analysis::LevelCompletionAnalyzer;
use crate::mcts::{MctsTree, MctsBudget};
use eustress_arc_types::{ArcStep, PolicyDecision};
use eustress_vortex_core::{
    CausalGraph, DSLOp, Delta, Domain, EpisodeRecord,
    GoalPredicate, Property, PropertyValue, Score, SymbolResolver, WorldState,
    causal_graph::CausalNode,
    hypothesis_tree::SimulationBudget,
};
use eustress_vortex_grid2d::{Grid2D, GridAnalyzer, ObjectExtractor, Connectivity};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tracing::info;

// ─── IRA: Internal Representation Accuracy ──────────────────────────────────

/// Tracks prediction accuracy for a single game action.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionModel {
    /// Which game action this models (1-7).
    pub action_id: u32,
    /// Observed grid deltas: (row, col, old_value, new_value).
    pub observed_patches: Vec<Vec<(usize, usize, u8, u8)>>,
    /// Running IRA score for this action.
    pub ira_score: f32,
    /// Total predictions made.
    pub prediction_count: u32,
    /// Correct predictions.
    pub correct_count: u32,
    /// Is this action deterministic (same effect every time)?
    pub is_deterministic: bool,
    /// Consecutive identical-effect streak.
    pub determinism_streak: u32,
    /// Whether this action produced any grid change.
    pub has_effect: bool,
}

impl ActionModel {
    pub fn new(action_id: u32) -> Self {
        Self {
            action_id,
            observed_patches: Vec::new(),
            ira_score: 0.0,
            prediction_count: 0,
            correct_count: 0,
            is_deterministic: false,
            determinism_streak: 0,
            has_effect: false,
        }
    }

    /// Record an observed effect of this action.
    pub fn observe(&mut self, before: &Grid2D, after: &Grid2D) {
        let mut patch = Vec::new();
        let rows = before.height.min(after.height);
        let cols = before.width.min(after.width);
        for r in 0..rows {
            for c in 0..cols {
                let old = before.cells[r][c];
                let new = after.cells[r][c];
                if old != new {
                    patch.push((r, c, old, new));
                }
            }
        }
        self.has_effect = !patch.is_empty();

        // Check determinism
        if let Some(last) = self.observed_patches.last() {
            if *last == patch {
                self.determinism_streak += 1;
                if self.determinism_streak >= 2 {
                    self.is_deterministic = true;
                }
            } else {
                self.determinism_streak = 0;
                self.is_deterministic = false;
            }
        }

        self.observed_patches.push(patch);
    }

    /// Predict what the grid will look like after this action.
    /// Returns None if we don't have enough observations or IRA reliability is too low.
    ///
    /// Uses IRA-informed prediction:
    /// - Deterministic actions: use the consensus patch (high confidence)
    /// - Non-deterministic with good IRA: use most recent patch
    /// - Low IRA: return None (don't pretend we know)
    pub fn predict(&self, current: &Grid2D) -> Option<Grid2D> {
        if self.observed_patches.is_empty() {
            return None;
        }

        // If we have enough data and IRA is too low, don't make unreliable predictions
        // This prevents the solve loop from planning with bad information
        if self.prediction_count >= 3 && self.ira_score < 0.3 {
            return None; // IRA says our predictions are unreliable
        }

        // For deterministic actions: use the consensus patch
        // (the patch that appears most frequently in observations)
        if self.is_deterministic && self.observed_patches.len() >= 2 {
            let consensus = &self.observed_patches[self.observed_patches.len() - 1];
            let mut predicted = current.clone();
            for &(r, c, _old, new) in consensus {
                if r < predicted.height && c < predicted.width {
                    predicted.cells[r][c] = new;
                }
            }
            return Some(predicted);
        }

        // For non-deterministic: use "delta-relative" prediction
        // Instead of absolute (r,c,old,new), apply the color *change* pattern
        // This handles cases where the action shifts content rather than setting fixed cells
        let patch = self.observed_patches.last()?;
        let mut predicted = current.clone();

        // Check if patch changes match current grid state (context-sensitive)
        let mut applicable = 0;
        let mut total = 0;
        for &(r, c, expected_old, _new) in patch {
            if r < current.height && c < current.width {
                total += 1;
                if current.cells[r][c] == expected_old {
                    applicable += 1;
                }
            }
        }

        // Only apply if the patch context matches (>50% of expected old values match)
        if total > 0 && applicable * 2 >= total {
            for &(r, c, expected_old, new) in patch {
                if r < predicted.height && c < predicted.width {
                    // Apply only where old value matches expected
                    if predicted.cells[r][c] == expected_old {
                        predicted.cells[r][c] = new;
                    }
                }
            }
            Some(predicted)
        } else if self.ira_score > 0.5 {
            // High IRA but context doesn't match — apply unconditionally
            // (action may have position-independent effects)
            for &(r, c, _old, new) in patch {
                if r < predicted.height && c < predicted.width {
                    predicted.cells[r][c] = new;
                }
            }
            Some(predicted)
        } else {
            None // Can't reliably predict
        }
    }

    /// IRA reliability: how trustworthy are predictions from this model.
    pub fn reliability(&self) -> f32 {
        if self.prediction_count < 2 {
            return 0.0;
        }
        self.correct_count as f32 / self.prediction_count as f32
    }

    /// Score IRA prediction against actual observation.
    pub fn score_prediction(&mut self, predicted: &Grid2D, actual: &Grid2D) -> f32 {
        let accuracy = predicted.cell_accuracy(actual) as f32;
        self.prediction_count += 1;
        if accuracy > 0.95 {
            self.correct_count += 1;
        }
        self.ira_score = self.ira_score * 0.8 + accuracy * 0.2;
        accuracy
    }
}

// ─── Win Recipe + Plan Buffer ────────────────────────────────────────────────

/// A recorded action sequence that caused a level advance.
/// Captured when `levels_completed` increments, replayed on subsequent levels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WinRecipe {
    /// The action sequence that produced the level advance.
    pub actions: Vec<String>,
    /// Grid fingerprint at start of the sequence (for matching).
    pub start_fingerprint: u64,
    /// How many steps the recipe took.
    pub length: usize,
    /// Number of times this recipe has been successfully replayed.
    pub replay_successes: u32,
    /// Game ID where this was first discovered.
    pub origin_game: String,
}

/// Buffered multi-step plan from the solve loop.
/// Instead of re-planning every step, commit to a sequence and detect deviation.
#[derive(Clone, Debug)]
struct PlanBuffer {
    /// Remaining DSL ops to execute (front = next).
    actions: Vec<DSLOp>,
    /// Predicted grid state after each remaining action (for deviation check).
    predicted_states: Vec<Grid2D>,
    /// Score that the solve loop assigned to this plan.
    plan_score: f64,
    /// Step at which this plan was computed.
    computed_at_step: u32,
}

impl Default for PlanBuffer {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            predicted_states: Vec::new(),
            plan_score: 0.0,
            computed_at_step: 0,
        }
    }
}

impl PlanBuffer {
    fn is_empty(&self) -> bool { self.actions.is_empty() }
    fn clear(&mut self) {
        self.actions.clear();
        self.predicted_states.clear();
        self.plan_score = 0.0;
    }
}

/// Hash grid dimensions + color histogram for fast recipe matching.
fn grid_fingerprint(grid: &Grid2D) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    let mut h = DefaultHasher::new();
    (grid.height, grid.width).hash(&mut h);
    grid.color_histogram().hash(&mut h);
    h.finish()
}

// ─── Game-Aware WorldState ──────────────────────────────────────────────────

/// Wraps Grid2D with learned game action models.
/// This allows the solve loop to simulate game actions using learned effects.
#[derive(Clone, Debug)]
pub struct ArcGameState {
    pub grid: Grid2D,
    pub levels_completed: u32,
    /// Learned action effects — shared reference via cloning.
    action_models: HashMap<u32, ActionModel>,
    /// Available game action IDs.
    available_action_ids: Vec<u32>,
}

impl ArcGameState {
    pub fn new(
        grid: Grid2D,
        levels_completed: u32,
        action_models: HashMap<u32, ActionModel>,
        available_action_ids: Vec<u32>,
    ) -> Self {
        Self { grid, levels_completed, action_models, available_action_ids }
    }

    /// Convert a game action ID to a DSLOp.
    fn action_to_dsl(action_id: u32) -> DSLOp {
        DSLOp {
            name: format!("game_action_{}", action_id),
            domain: Domain::GameState,
            parameters: vec![],
        }
    }

    /// Extract the game action ID from a DSLOp name.
    fn dsl_to_action(op: &DSLOp) -> Option<u32> {
        op.name.strip_prefix("game_action_").and_then(|s| s.parse().ok())
    }
}

impl WorldState for ArcGameState {
    fn analyze(&self) -> Vec<Property> {
        let mut props = self.grid.analyze();
        // Add game-level properties
        props.push(Property {
            name: "levels_completed".into(),
            domain: Domain::GameState,
            value: PropertyValue::Int(self.levels_completed as i64),
        });
        // Add IRA quality properties for each known action
        for (id, model) in &self.action_models {
            if model.prediction_count > 0 {
                props.push(Property {
                    name: format!("action_{}_ira", id),
                    domain: Domain::GameState,
                    value: PropertyValue::Float(model.ira_score as f64),
                });
                props.push(Property {
                    name: format!("action_{}_deterministic", id),
                    domain: Domain::GameState,
                    value: PropertyValue::Bool(model.is_deterministic),
                });
            }
        }
        props
    }

    fn available_actions(&self) -> Vec<DSLOp> {
        self.available_action_ids
            .iter()
            .map(|&id| Self::action_to_dsl(id))
            .collect()
    }

    fn apply(&self, action: &DSLOp) -> Self {
        let action_id = Self::dsl_to_action(action).unwrap_or(0);

        // IRA-gated prediction: only trust models with sufficient reliability
        let model = self.action_models.get(&action_id);
        let predicted_grid = match model {
            Some(m) if m.has_effect => {
                // Use IRA reliability to decide how much to trust the prediction
                match m.predict(&self.grid) {
                    Some(predicted) => predicted,
                    None => self.grid.clone(), // IRA too low to predict
                }
            }
            Some(_) => {
                // Action has no effect — grid unchanged
                self.grid.clone()
            }
            None => {
                // Unknown action — can't predict, assume no change
                self.grid.clone()
            }
        };

        ArcGameState {
            grid: predicted_grid,
            levels_completed: self.levels_completed,
            action_models: self.action_models.clone(),
            available_action_ids: self.available_action_ids.clone(),
        }
    }

    fn score_against(&self, goal: &Self) -> Score {
        // For interactive games: score combines level progress + structural quality + IRA confidence
        let structural = structural_improvement_score(&self.grid);

        let level_score = if self.levels_completed > goal.levels_completed {
            1.0 // We advanced — perfect
        } else if self.levels_completed == goal.levels_completed {
            structural
        } else {
            0.0
        };

        // IRA confidence bonus: predictions we can trust score higher
        // This makes the solve loop prefer action paths with reliable models
        let ira_confidence: f64 = self.action_models.values()
            .filter(|m| m.has_effect && m.prediction_count > 0)
            .map(|m| m.ira_score as f64)
            .sum::<f64>()
            / self.action_models.len().max(1) as f64;

        // Combined score: 70% structural + 30% IRA confidence
        let combined = level_score * 0.7 + ira_confidence * 0.3;

        Score {
            exact_match: level_score >= 1.0,
            accuracy: combined.min(0.99), // Cap below 1.0 unless level advance
            details: serde_json::json!({
                "levels": self.levels_completed,
                "structural_score": structural,
                "ira_confidence": ira_confidence,
            }),
        }
    }

    fn diff(&self, other: &Self) -> Vec<Delta> {
        let mut deltas = self.grid.diff(&other.grid);
        if self.levels_completed != other.levels_completed {
            deltas.push(Delta {
                kind: "level_change".into(),
                description: format!(
                    "levels {} → {}",
                    self.levels_completed, other.levels_completed
                ),
                magnitude: (other.levels_completed as f64) - (self.levels_completed as f64),
            });
        }
        deltas
    }

    fn summary(&self) -> String {
        format!(
            "ArcGame({}x{}, lvl={}, {} action models)",
            self.grid.height,
            self.grid.width,
            self.levels_completed,
            self.action_models.len()
        )
    }
}

/// Score structural "progress" of a grid (used when we don't know the goal).
/// Higher = looks more like a "solved" ARC grid.
fn structural_improvement_score(grid: &Grid2D) -> f64 {
    let props = GridAnalyzer::analyze(grid);
    let mut score: f64 = 0.0;

    for p in &props {
        match p.name.as_str() {
            "symmetric_horizontal" if p.value.as_bool() => score += 0.15,
            "symmetric_vertical" if p.value.as_bool() => score += 0.15,
            "symmetric_rotational_180" if p.value.as_bool() => score += 0.10,
            "has_border" if p.value.as_bool() => score += 0.05,
            "content_centered" if p.value.as_bool() => score += 0.05,
            "has_horizontal_period" if p.value.as_bool() => score += 0.05,
            "has_vertical_period" if p.value.as_bool() => score += 0.05,
            "has_unsupported_objects" if p.value.as_bool() => score -= 0.10,
            "single_object" if p.value.as_bool() => score += 0.05,
            _ => {}
        }
    }

    score.clamp(0.0, 0.95) // Never 1.0 — only level advance is "perfect"
}

// ─── KnowledgeSnapshot — cross-game persistent state ─────────────────────────

/// Snapshot of persistent knowledge that transfers between games.
/// Serializable to JSON for storage in the knowledge/ directory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeSnapshot {
    pub causal_graph: CausalGraph,
    pub decomposer: SymbolicActionDecomposer,
    pub global_ira: f32,
    pub games_played: u32,
    /// Cross-game recipe library — action sequences that solved levels.
    #[serde(default)]
    pub recipe_library: Vec<WinRecipe>,
    /// Per-game learned mechanics (control maps + IRA) — revisits skip the
    /// probe phase, so knowledge genuinely persists across sessions.
    #[serde(default)]
    pub game_profiles: HashMap<String, crate::forward_model::GameProfile>,
    /// Cross-game Q-table over abstract states — the transferable prior
    /// for games never seen before.
    #[serde(default)]
    pub q_table: crate::qlearn::QTable,
    /// Causal claims with epistemic status — the agent's verified science.
    #[serde(default)]
    pub ledger: crate::epistemic::EpistemicLedger,
}

// ─── Click Mechanic Learning ─────────────────────────────────────────────────

/// Observed effect of a single click action.
#[derive(Clone, Debug)]
struct ClickObservation {
    _click_x: u32,
    _click_y: u32,
    /// Cell changes: (row, col, old_value, new_value)
    changes: Vec<(usize, usize, u8, u8)>,
}

/// Learned click mechanic for click-only games.
#[derive(Clone, Debug)]
#[allow(dead_code)]
enum ClickMechanic {
    /// Clicking at (x,y) toggles cell (x,y) between two colors.
    Toggle { color_a: u8, color_b: u8 },
    /// Clicking at (x,y) paints cell to a fixed color.
    Paint(u8),
    /// Clicking cycles through colors in order.
    Cycle(Vec<u8>),
}

// ─── VortexWorldModel — the Phase 3 brain ───────────────────────────────────

/// Persistent world model that learns game mechanics across steps and episodes.
///
/// Uses the full Eustress infrastructure:
/// - CausalGraph: learns Rules (Z) from observations, unifies into Laws (C)
/// - HypothesisTree: branches simulation paths, prunes low-scoring ones
/// - Grid2D + GridDSL: structural analysis and transformation simulation
/// - IRA: tracks prediction accuracy per action to build trust
pub struct VortexWorldModel {
    /// Causal knowledge graph — persists across episodes for cross-game learning.
    pub causal_graph: CausalGraph,
    /// Per-action learned effect models.
    pub action_models: HashMap<u32, ActionModel>,
    /// Symbolic decomposer — algebraic action effect analysis with cross-game learning.
    pub decomposer: SymbolicActionDecomposer,
    /// Scene mirror — live EustressEngine entity state for Explorer rendering.
    pub scene_mirror: ArcSceneMirror,
    /// Previous frame for delta computation.
    prev_grid: Option<Grid2D>,
    /// Last action taken (to attribute observed deltas).
    last_action_id: Option<u32>,
    /// Pending IRA prediction for comparison on next observation.
    pending_prediction: Option<(u32, Grid2D)>,
    /// Global IRA score (exponential moving average across all actions).
    pub global_ira: f32,
    /// IRA history for monitoring convergence.
    pub ira_history: Vec<(u32, f32)>,
    /// Steps taken in current game.
    step_in_game: u32,
    /// Current game ID.
    current_game: String,
    /// Previous levels_completed for detecting advances.
    prev_levels: u32,
    /// Actions that have been tried at least once in current game.
    tried_actions: Vec<u32>,
    /// Click targets cycled through for ACTION6.
    click_targets: Vec<(u32, u32)>,
    click_count: usize,
    /// Scene deltas from the last `decide()` call — available for Iggy publishing.
    pub last_scene_deltas: Vec<ArcSceneDelta>,
    // ─── Click Mechanic Learning ─────────────────────────────────────
    /// Observed click effects: (click_x, click_y, cell_changes)
    click_observations: Vec<ClickObservation>,
    /// Last click position for correlating with observed grid changes.
    last_click_pos: Option<(u32, u32)>,
    /// Learned click mechanic (toggle, paint, etc.).
    click_mechanic: Option<ClickMechanic>,
    /// Best structural score seen in current level (for submit timing).
    best_structural_score: f64,
    /// Clicks since last submit attempt.
    clicks_since_submit: u32,
    /// Whether we're in a click-only game (only actions 6 and 7).
    click_only_game: bool,
    // ─── Action Diversity + Burst Mode ───────────────────────────────
    /// Per-action structural improvement tracking: action_id → (times_improved, times_used).
    action_structural_scores: HashMap<u32, (u32, u32)>,
    /// Round-robin index for action diversity when no clear winner.
    diversity_index: usize,
    /// Steps since last level advance (for detecting total stagnation).
    steps_since_level_advance: u32,
    /// Actions since last submit.
    actions_since_submit: u32,
    // ─── Win Recipe + Plan Buffer fields ─────────────────────────────
    /// Actions taken since last level advance (for recipe capture).
    current_level_actions: Vec<String>,
    /// Grid fingerprint at the start of the current level.
    level_start_fingerprint: u64,
    /// Recipes that worked for levels in THIS game.
    win_recipes: Vec<WinRecipe>,
    /// Cross-game recipe library (persisted via KnowledgeSnapshot).
    recipe_library: Vec<WinRecipe>,
    /// Active recipe replay cursor: (recipe_index, step_within_recipe).
    replay_cursor: Option<(usize, usize)>,
    /// Buffered multi-step plan from solve().
    plan_buffer: PlanBuffer,
    /// Recent action history for stuck-action detection (last N actions + whether they changed the grid).
    recent_action_effects: VecDeque<(u32, bool)>,
    // ─── Object Perception (Pillar 1) ───────────────────────────────
    /// Object identity tracker — maintains stable IDs across frames.
    object_tracker: ObjectTracker,
    /// Current object frame snapshot.
    object_frame: ObjectFrame,
    /// Per-action object-level effect models.
    object_action_models: HashMap<u32, ObjectActionModel>,
    /// Identified agent object ID.
    agent_object_id: Option<StableObjectId>,
    // ─── Undo-Safe Exploration (Pillar 2) ────────────────────────────
    /// Undo-based state-graph explorer.
    undo_explorer: UndoExplorer,
    /// Whether we're currently in the undo exploration phase.
    in_exploration: bool,
    // ─── Goal Inference (Pillar 3) ───────────────────────────────────
    /// Goal inference engine.
    goal_engine: GoalInferenceEngine,
    /// Grid at the start of current level (for level completion analysis).
    level_start_grid: Option<Grid2D>,
    /// Object frame at the start of current level.
    level_start_frame: Option<ObjectFrame>,
    /// Level completion analyzer.
    level_analyzer: LevelCompletionAnalyzer,
    // ─── MCTS (Pillar 4) ────────────────────────────────────────────
    /// MCTS search budget.
    mcts_budget: MctsBudget,
    /// Persistent rule→law unifier. Its cache remembers attempted pairs,
    /// keeping the per-step scan incremental — recreating it each step
    /// made unification O(rules²) per step and minted duplicate laws.
    symbol_resolver: SymbolResolver,
    /// Cells whose clicks changed nothing this level — never re-clicked.
    dead_clicks: std::collections::HashSet<(u32, u32)>,
    /// Cells whose clicks visibly changed the frame this level.
    responsive_clicks: std::collections::HashSet<(u32, u32)>,
    /// Cells already clicked this level (responsive or not) — exploration
    /// prefers cells it has never tried.
    clicked_this_level: std::collections::HashSet<(u32, u32)>,
    /// Per-cell change counts across ALL actions — cells changing on most
    /// steps are ambient dynamics (timers, counters), not action effects.
    ambient_change_counts: HashMap<(usize, usize), u32>,
    /// Total observations contributing to ambient_change_counts.
    ambient_observations: u32,
    /// Cross-game Q-table over abstracted states — transfers to unseen
    /// games because abstract situations recur across games.
    q_table: crate::qlearn::QTable,
    /// Competing win-condition interpretations for the current level,
    /// posterior-weighted — reward shaping follows what we currently
    /// believe winning means, updated at every completion.
    predicate_ensemble: Option<crate::predicates::PredicateEnsemble>,
    /// Epistemic ledger: causal claims with the scientific lifecycle
    /// Observed -> Correlated -> Verified -> Law. Interventions with
    /// declared predictions are the only road to Law status.
    ledger: crate::epistemic::EpistemicLedger,
    /// Progress metric at the previous step (targets remaining / goal
    /// distance), for shaped-reward deltas.
    prev_progress: f32,
    /// Online object-level forward model: control mapping, per-action IRA,
    /// and goal-directed stepping for keyboard games.
    forward_model: crate::forward_model::ObjectForwardModel,
    /// Per-game learned-mechanics profiles (persisted in the knowledge
    /// snapshot) — a revisited game skips the probe phase entirely.
    game_profiles: HashMap<String, crate::forward_model::GameProfile>,
}

impl VortexWorldModel {
    pub fn new() -> Self {
        Self {
            causal_graph: CausalGraph::with_physics_laws(),
            action_models: HashMap::new(),
            decomposer: SymbolicActionDecomposer::new(),
            scene_mirror: ArcSceneMirror::new("", ArcMirrorConfig::default()),
            prev_grid: None,
            last_action_id: None,
            pending_prediction: None,
            global_ira: 0.0,
            ira_history: Vec::new(),
            step_in_game: 0,
            current_game: String::new(),
            prev_levels: 0,
            tried_actions: Vec::new(),
            click_targets: Vec::new(),
            click_count: 0,
            last_scene_deltas: Vec::new(),
            click_observations: Vec::new(),
            last_click_pos: None,
            click_mechanic: None,
            best_structural_score: 0.0,
            clicks_since_submit: 0,
            click_only_game: false,
            action_structural_scores: HashMap::new(),
            diversity_index: 0,
            steps_since_level_advance: 0,
            actions_since_submit: 0,
            current_level_actions: Vec::new(),
            level_start_fingerprint: 0,
            win_recipes: Vec::new(),
            recipe_library: Vec::new(),
            replay_cursor: None,
            plan_buffer: PlanBuffer::default(),
            recent_action_effects: VecDeque::new(),
            // Object perception
            object_tracker: ObjectTracker::new(),
            object_frame: ObjectFrame::default(),
            object_action_models: HashMap::new(),
            agent_object_id: None,
            // Undo exploration
            undo_explorer: UndoExplorer::with_default_budget(),
            in_exploration: false,
            // Goal inference
            goal_engine: GoalInferenceEngine::new(),
            level_start_grid: None,
            level_start_frame: None,
            level_analyzer: LevelCompletionAnalyzer::new(),
            // MCTS
            mcts_budget: MctsBudget::default(),
            symbol_resolver: SymbolResolver::new(),
            dead_clicks: std::collections::HashSet::new(),
            responsive_clicks: std::collections::HashSet::new(),
            clicked_this_level: std::collections::HashSet::new(),
            ambient_change_counts: HashMap::new(),
            ambient_observations: 0,
            q_table: crate::qlearn::QTable::default(),
            predicate_ensemble: None,
            ledger: crate::epistemic::EpistemicLedger::default(),
            prev_progress: 0.0,
            forward_model: crate::forward_model::ObjectForwardModel::new(),
            game_profiles: HashMap::new(),
        }
    }

    /// Current abstract state for the Q-layer plus a scalar progress metric
    /// (higher = closer to done) for shaped rewards.
    fn abstract_state(&self, grid: &Grid2D) -> (crate::qlearn::AbstractState, f32) {
        if self.click_only_game || self.click_mechanic.is_some() {
            let remaining = self.click_targets.len();
            // Progress blends structural order (uniformity/symmetry — the
            // board approaching a solved configuration) with targets
            // remaining: a dense, win-correlated gradient instead of a
            // sparse one. This is the forward-looking win signal the
            // post-hoc LevelCompletionAnalyzer can't give us.
            let structural = structural_improvement_score(grid) as f32;
            // Posterior-weighted win-predicate progress dominates when the
            // ensemble exists; structural order remains as a stabilizer.
            let predicate_progress = self
                .predicate_ensemble
                .as_ref()
                .map(|e| e.progress(grid))
                .unwrap_or(0.0);
            (
                crate::qlearn::AbstractState::click(remaining, self.click_mechanic.is_some()),
                predicate_progress * 6.0 + structural * 2.0 - remaining as f32 * 0.1,
            )
        } else {
            let avatar = self.object_tracker.agent().map(|a| a.current.centroid);
            let goal = self
                .object_frame
                .goal_candidates
                .first()
                .and_then(|&id| self.object_tracker.get(id))
                .map(|g| g.current.centroid);
            let blocked = self
                .recent_action_effects
                .back()
                .map(|(_, had)| !had)
                .unwrap_or(false);
            match (avatar, goal) {
                (Some(a), Some(g)) => {
                    let (dr, dc) = (g.0 - a.0, g.1 - a.1);
                    (
                        crate::qlearn::AbstractState::movement(dr, dc, blocked),
                        -(dr.abs() + dc.abs()),
                    )
                }
                _ => (
                    crate::qlearn::AbstractState(format!("explore|b{}", blocked as u8)),
                    0.0,
                ),
            }
        }
    }

    /// A cell is ambient when it changed on >40% of observed steps (with at
    /// least 5 observations) — environment dynamics, not action effects.
    fn is_ambient_cell(&self, r: usize, c: usize) -> bool {
        if self.ambient_observations < 5 {
            return false;
        }
        let count = self.ambient_change_counts.get(&(r, c)).copied().unwrap_or(0);
        (count as f32) / (self.ambient_observations as f32) > 0.4
    }

    /// Reset state for a new game episode.
    /// Preserves cross-game knowledge: CausalGraph, decomposer archetypes.
    pub fn new_episode(&mut self, game_id: &str) {
        // Bank what the forward model learned in the previous game, then
        // reset it — adopting a stored profile if we've played this game
        // before (probe phase skipped on revisits).
        if !self.current_game.is_empty() {
            let mut profile = self.forward_model.to_profile().unwrap_or_default();
            // Bank what we believe winning means here, even when no
            // control map was learned (click games have none).
            profile.predicates = self.predicate_ensemble.clone();
            if !profile.control_map.is_empty() || profile.predicates.is_some() {
                info!(
                    "  PROFILE saved for {}: {} controls, predicates={}, ira={:.2}",
                    self.current_game,
                    profile.control_map.len(),
                    profile.predicates.is_some(),
                    profile.ira
                );
                self.game_profiles.insert(self.current_game.clone(), profile);
            }
        }
        self.forward_model.reset();
        self.q_table.reset_pending();
        self.ledger.reset_pending();
        self.prev_progress = 0.0;
        if let Some(profile) = self.game_profiles.get(game_id) {
            info!(
                "  PROFILE adopted for {}: {} controls known, probing skipped",
                game_id, profile.control_map.len()
            );
            self.forward_model.adopt_profile(&profile.clone());
        }
        self.prev_grid = None;
        self.last_action_id = None;
        self.pending_prediction = None;
        self.step_in_game = 0;
        self.current_game = game_id.to_string();
        self.prev_levels = 0;
        self.tried_actions.clear();
        self.click_targets.clear();
        self.click_count = 0;
        self.click_observations.clear();
        self.last_click_pos = None;
        self.click_mechanic = None;
        self.best_structural_score = 0.0;
        self.clicks_since_submit = 0;
        self.click_only_game = false;
        self.action_structural_scores.clear();
        self.diversity_index = 0;
        self.steps_since_level_advance = 0;
        self.actions_since_submit = 0;
        // Reset scene mirror for new game (clear entities)
        self.scene_mirror = ArcSceneMirror::new(game_id, ArcMirrorConfig::default());
        // Keep causal_graph, action_models, decomposer — they learn across episodes
        // Reset per-game recipe/plan state (recipe_library persists cross-game)
        self.current_level_actions.clear();
        self.level_start_fingerprint = 0;
        self.win_recipes.clear();
        self.replay_cursor = None;
        self.plan_buffer.clear();
        self.recent_action_effects.clear();
        // Click memory is per-level state
        self.dead_clicks.clear();
        self.responsive_clicks.clear();
        self.clicked_this_level.clear();
        self.ambient_change_counts.clear();
        self.ambient_observations = 0;
        // Object perception reset
        self.object_tracker.reset();
        self.object_frame = ObjectFrame::default();
        self.object_action_models.clear();
        self.agent_object_id = None;
        // Undo exploration reset
        self.undo_explorer.reset();
        self.in_exploration = false;
        // Goal inference reset
        self.goal_engine.reset();
        self.level_start_grid = None;
        self.level_start_frame = None;
        self.level_analyzer.reset();
    }

    // ─── Cross-game persistence ─────────────────────────────────────────

    /// Export cross-game knowledge for persistence.
    pub fn export_knowledge(&self) -> String {
        // Include the live game's profile even before its episode ends
        let mut game_profiles = self.game_profiles.clone();
        if !self.current_game.is_empty() {
            if let Some(profile) = self.forward_model.to_profile() {
                game_profiles.insert(self.current_game.clone(), profile);
            }
        }
        let snapshot = KnowledgeSnapshot {
            causal_graph: self.causal_graph.clone(),
            decomposer: self.decomposer.clone(),
            global_ira: self.global_ira,
            games_played: self.decomposer.action_history.len() as u32,
            recipe_library: self.recipe_library.clone(),
            game_profiles,
            q_table: self.q_table.clone(),
            ledger: self.ledger.clone(),
        };
        serde_json::to_string_pretty(&snapshot).unwrap_or_default()
    }

    /// Import cross-game knowledge from a previous session.
    pub fn import_knowledge(&mut self, json: &str) -> Result<(), String> {
        // Self-healing: knowledge files written before the June 2026
        // dedupe fixes can carry millions of duplicate laws/edges (one
        // poisoned file reached 2GB / 11M edges and made every decision
        // take 9+ seconds). A healthy graph is a few thousand nodes; if
        // the snapshot is implausibly large, start clean instead of
        // inheriting the bloat.
        const MAX_SANE_BYTES: usize = 50 * 1024 * 1024;
        if json.len() > MAX_SANE_BYTES {
            return Err(format!(
                "knowledge snapshot is {:.1}MB (> {}MB sanity bound) — \
                 refusing to import bloated pre-dedupe state, starting clean",
                json.len() as f64 / 1e6,
                MAX_SANE_BYTES / 1024 / 1024,
            ));
        }

        let snapshot: KnowledgeSnapshot = serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize knowledge: {}", e))?;

        const MAX_SANE_EDGES: usize = 100_000;
        if snapshot.causal_graph.edge_count() > MAX_SANE_EDGES {
            return Err(format!(
                "knowledge snapshot has {} edges (> {} sanity bound) — \
                 refusing to import bloated pre-dedupe state, starting clean",
                snapshot.causal_graph.edge_count(), MAX_SANE_EDGES,
            ));
        }

        self.causal_graph = snapshot.causal_graph;
        self.decomposer = snapshot.decomposer;
        self.global_ira = snapshot.global_ira;
        self.recipe_library = snapshot.recipe_library;
        self.game_profiles = snapshot.game_profiles;
        self.q_table = snapshot.q_table;
        self.ledger = snapshot.ledger;

        info!(
            "Imported knowledge: {} nodes, {} edges, {} archetypes, {} recipes, global_ira={:.3}",
            self.causal_graph.node_count(),
            self.causal_graph.edge_count(),
            self.decomposer.archetype_summary().len(),
            self.recipe_library.len(),
            self.global_ira,
        );
        Ok(())
    }

    /// Core decision function. Called once per game step.
    ///
    /// Implements the Eustress cycle:
    /// 1. OBSERVE: Parse frame → Grid2D, compute delta from previous
    /// 2. INTERNALIZE: Score IRA prediction, feed delta into CausalGraph
    /// 3. HYPOTHESIZE: CausalGraph suggests hypotheses from learned Laws
    /// 4. SIMULATE: HypothesisTree explores action sequences
    /// 5. EVALUATE: Pick best action from completed branches
    pub fn decide(&mut self, step: &ArcStep) -> PolicyDecision {
        let mut available = step.available_action_ids();
        if available.is_empty() {
            available = vec![1, 2, 3, 4, 5]; // Default ARC action set
        }
        let levels = step.levels_completed();
        let game_id = step.game_id().unwrap_or("unknown");

        // Detect new game
        if game_id != self.current_game {
            self.new_episode(game_id);
        }

        // Detect click-only game (only actions 6 and 7 available)
        if !self.click_only_game && self.step_in_game == 0 {
            let non_click = available.iter().any(|&a| a != 6 && a != 7);
            self.click_only_game = !non_click && available.contains(&6);
            if self.click_only_game {
                info!("CLICK-ONLY GAME detected: available={:?}", available);
            }
        }

        // ── 1. OBSERVE ──────────────────────────────────────────────────
        let grid = match step.frame_grid() {
            Some(cells) => Grid2D::new(cells),
            None => {
                return PolicyDecision {
                    action: available.first().map(|a| a.to_string()).unwrap_or_default(),
                    confidence: 0.0,
                    reasoning: "no frame data available".into(),
                };
            }
        };

        let props = grid.analyze();

        // ── OBJECT EXTRACTION + TRACKING ──────────────────────────────────
        let object_map = ObjectExtractor::extract(&grid, Connectivity::Four);
        let object_events = self.object_tracker.update(&object_map, self.last_action_id);
        self.object_frame = self.object_tracker.current_frame();
        self.agent_object_id = self.object_tracker.agent_id();

        info!(
            "OBSERVE: {}x{} grid, {} colors, {} objects, {} obj_events, step {}",
            grid.height, grid.width, grid.distinct_colors(),
            object_map.object_count(), object_events.len(), self.step_in_game,
        );

        // ── Q-LAYER: score the pending transition ─────────────────────────
        // Reward = true reward (level advance) + progress shaping + step
        // cost + first-visit curiosity, all from signals already computed.
        {
            let (state_now, progress_now) = self.abstract_state(&grid);
            let first_visit = !self.q_table.states.contains_key(&state_now.0);
            let reward = crate::qlearn::shaped_reward(
                levels > self.prev_levels,
                progress_now - self.prev_progress,
                first_visit,
            );
            self.q_table.observe(reward, &state_now);
            self.prev_progress = progress_now;
            // Dyna-Q: replay stored real transitions while the API
            // round-trip is in flight — free planning between actions.
            self.q_table.dyna_replay(50);
        }

        // ── FORWARD MODEL: fold the last action's outcome in ──────────────
        // Updates the per-action displacement stats and scores the pending
        // avatar-position forecast (per-action Representation Accuracy).
        if let Some(last_action) = self.last_action_id {
            let avatar_now = self
                .object_tracker
                .agent()
                .map(|a| a.current.centroid);
            self.forward_model.observe(
                last_action,
                &object_events,
                self.agent_object_id,
                avatar_now,
            );
        }

        // Save level start grid on first observation of a new level
        if self.level_start_grid.is_none() {
            self.level_start_grid = Some(grid.clone());
            self.level_start_frame = Some(self.object_frame.clone());
            // Fresh win-condition hypotheses for the new level, seeded by
            // this game's stored posteriors when we've played it before —
            // interpretation itself transfers.
            let mut ensemble = crate::predicates::PredicateEnsemble::init_from_grid(&grid);
            if let Some(stored) = self
                .game_profiles
                .get(game_id)
                .and_then(|p| p.predicates.as_ref())
            {
                for sp in &stored.predicates {
                    if let Some(p) = ensemble.predicates.iter_mut().find(|p| p.kind == sp.kind) {
                        p.posterior = (p.posterior + sp.posterior * 2.0) / 3.0;
                        p.confirmations = sp.confirmations;
                    }
                }
            }
            self.predicate_ensemble = Some(ensemble);
        }

        // ── SCENE MIRROR: update live entity state ──────────────────────
        let scene_deltas = self.scene_mirror.apply_frame(&grid, self.step_in_game, levels);
        self.scene_mirror.properties.global_ira = self.global_ira;
        self.scene_mirror.properties.total_actions = self.step_in_game;

        // Project behavioral object roles onto the EEP tag/attribute system:
        // every cell of a tracked object carries role_* and object_* tags in
        // the 3D scene (avatar/goal/barrier/... — queryable, visible).
        {
            use crate::object_tracker::ObjectRole;
            let mut cell_roles: HashMap<(usize, usize), (u32, &'static str)> = HashMap::new();
            for t in self.object_tracker.tracked_objects() {
                let role: &'static str = if Some(t.stable_id) == self.agent_object_id {
                    "avatar"
                } else if self.object_frame.goal_candidates.contains(&t.stable_id) {
                    "goal"
                } else {
                    match t.classification.role {
                        // Only the tracker-selected avatar (checked above)
                        // earns the avatar tag — other action-correlated
                        // movers are pushed blocks / scrolling, i.e. movable
                        ObjectRole::Agent => "movable",
                        ObjectRole::StaticBarrier => "barrier",
                        ObjectRole::Movable => "movable",
                        ObjectRole::Collectible => "collectible",
                        ObjectRole::Goal => "goal",
                        ObjectRole::Decoration => "decoration",
                        ObjectRole::Unknown => continue,
                    }
                };
                for &(r, c) in &t.current.cells {
                    cell_roles.insert((r, c), (t.stable_id, role));
                }
            }
            if !cell_roles.is_empty() {
                self.scene_mirror.apply_object_roles(grid.width, &cell_roles);
            }
        }

        // Update symmetry property from grid analysis
        let h_sym = props.iter().any(|p| p.name == "symmetric_horizontal" && p.value.as_bool());
        let v_sym = props.iter().any(|p| p.name == "symmetric_vertical" && p.value.as_bool());
        let symmetry = match (h_sym, v_sym) {
            (true, true) => "both",
            (true, false) => "horizontal",
            (false, true) => "vertical",
            (false, false) => "none",
        };
        self.scene_mirror.set_symmetry(symmetry);

        // Store deltas for external consumers (Iggy publishing, TOML materializer)
        self.last_scene_deltas = scene_deltas;
        if !self.last_scene_deltas.is_empty() {
            info!("  SCENE: {} entity deltas for step {}", self.last_scene_deltas.len(), self.step_in_game);
        }

        // ── 2. INTERNALIZE (learn from previous action's effect) ────────
        if let (Some(prev), Some(action_id)) = (&self.prev_grid, self.last_action_id) {
            let model = self.action_models
                .entry(action_id)
                .or_insert_with(|| ActionModel::new(action_id));
            model.observe(prev, &grid);

            let changed = prev.cells_changed(&grid);
            let had_effect = changed > 0;
            info!(
                "  action_{} produced {} cell changes (effect={})",
                action_id, changed, model.has_effect
            );

            // Track recent action effects for stuck-action detection
            self.recent_action_effects.push_back((action_id, had_effect));
            if self.recent_action_effects.len() > 10 {
                self.recent_action_effects.pop_front();
            }

            // ── STRUCTURAL IMPROVEMENT TRACKING ──────────────────────────
            // Track whether each action improves structural score (for burst/diversity)
            if had_effect {
                let prev_structural = structural_improvement_score(prev);
                let new_structural = structural_improvement_score(&grid);
                let entry = self.action_structural_scores.entry(action_id).or_insert((0, 0));
                entry.1 += 1; // times_used
                if new_structural > prev_structural + 0.01 {
                    entry.0 += 1; // times_improved
                    info!(
                        "  STRUCTURAL: action_{} improved {:.3} → {:.3}",
                        action_id, prev_structural, new_structural
                    );
                } else if new_structural < prev_structural - 0.01 {
                    info!(
                        "  STRUCTURAL: action_{} degraded {:.3} → {:.3}",
                        action_id, prev_structural, new_structural
                    );
                }
            }

            self.steps_since_level_advance += 1;
            self.actions_since_submit += 1;

            // ── AMBIENT DYNAMICS TRACKING ────────────────────────────────
            // Cells that change on (nearly) every action regardless of what
            // we did are environment state — move counters, timers,
            // animations — not consequences of our action. vc33's energy
            // bar drains one cell per click, which made EVERY click look
            // "responsive" and defeated dead-click learning entirely.
            {
                let rows = prev.height.min(grid.height);
                let cols = prev.width.min(grid.width);
                self.ambient_observations += 1;
                for r in 0..rows {
                    for c in 0..cols {
                        if prev.cells[r][c] != grid.cells[r][c] {
                            *self.ambient_change_counts.entry((r, c)).or_insert(0) += 1;
                        }
                    }
                }
            }

            // ── CLICK MECHANIC LEARNING ──────────────────────────────────
            if action_id == 6 {
                if let Some((cx, cy)) = self.last_click_pos.take() {
                    let mut changes = Vec::new();
                    let rows = prev.height.min(grid.height);
                    let cols = prev.width.min(grid.width);
                    // Field-precise ambient check (a method call here would
                    // conflict with the live &mut into action_models)
                    let ambient_obs = self.ambient_observations;
                    for r in 0..rows {
                        for c in 0..cols {
                            let old = prev.cells[r][c];
                            let new_val = grid.cells[r][c];
                            if old == new_val {
                                continue;
                            }
                            let is_ambient = ambient_obs >= 5 && {
                                let count = self
                                    .ambient_change_counts
                                    .get(&(r, c))
                                    .copied()
                                    .unwrap_or(0);
                                (count as f32) / (ambient_obs as f32) > 0.4
                            };
                            if !is_ambient {
                                changes.push((r, c, old, new_val));
                            }
                        }
                    }
                    // Effect signature for the epistemic ledger: the first
                    // non-ambient change plus magnitude — compact, comparable.
                    let effect_sig = match changes.first() {
                        Some((r, c, old, newv)) => format!("d({},{}:{}->{})x{}", r, c, old, newv, changes.len()),
                        None => "none".to_string(),
                    };
                    let claim_id = format!("click({},{})", cx, cy);
                    self.ledger.observe_cooccurrence(&format!("{}->{}", claim_id, effect_sig));
                    if let Some((intent, held)) = self.ledger.resolve_intent(&effect_sig) {
                        info!(
                            "  LEDGER: intent {} expected {} got {} — {}",
                            intent.claim_id, intent.expected_effect, effect_sig,
                            if held { "VERIFIED" } else { "CONTRADICTED" }
                        );
                    }

                    if !changes.is_empty() {
                        info!(
                            "  CLICK LEARN: click({},{}) → {} cell changes: {:?}",
                            cx, cy, changes.len(),
                            changes.iter().take(3).collect::<Vec<_>>()
                        );
                        // A responsive cell — remember it as live
                        self.responsive_clicks.insert((cx, cy));
                        self.click_observations.push(ClickObservation {
                            _click_x: cx, _click_y: cy, changes,
                        });
                        // Try to infer mechanic after 2+ observations
                        if self.click_observations.len() >= 2 && self.click_mechanic.is_none() {
                            self.click_mechanic = infer_click_mechanic(&self.click_observations);
                            if let Some(ref mech) = self.click_mechanic {
                                info!("  CLICK MECHANIC learned: {:?}", mech);
                            }
                        }
                    } else {
                        // Real-time adaptation: a click that changed nothing is a
                        // dead cell for this level — never spend another action
                        // on it. (Cleared on level advance / new game.)
                        self.dead_clicks.insert((cx, cy));
                        if self.dead_clicks.len() % 25 == 0 {
                            info!(
                                "  CLICK LEARN: {} dead cells excluded, {} responsive",
                                self.dead_clicks.len(), self.responsive_clicks.len()
                            );
                        }
                    }
                    // Invalidate static click targets so they're rebuilt with new knowledge
                    self.click_targets.clear();
                }
            }

            // Score IRA prediction if we had one
            if let Some((pred_action, pred_grid)) = self.pending_prediction.take() {
                if pred_action == action_id {
                    let accuracy = model.score_prediction(&pred_grid, &grid);
                    self.global_ira = self.global_ira * 0.85 + accuracy * 0.15;
                    self.ira_history.push((self.step_in_game, accuracy));
                    info!(
                        "  IRA: prediction accuracy={:.3}, global={:.3}, reliability={:.3}",
                        accuracy, self.global_ira, model.reliability()
                    );
                }
            }

            // ── SYMBOLIC DECOMPOSITION ──────────────────────────────────
            // Decompose the action effect into algebraic operations and
            // check for cross-game matches via the EquivalenceCache.
            let level_advanced = levels > self.prev_levels;
            let (decomp, cross_matches) = self.decomposer.observe_and_learn(
                game_id, action_id, prev, &grid, level_advanced,
            );

            // Log decomposition results
            for (op, coverage) in &decomp.ops {
                if op.is_structural() {
                    info!(
                        "  SYMBOLIC: action_{} → {} (coverage={:.0}%)",
                        action_id, op.formula(), coverage * 100.0
                    );
                }
            }
            for m in &cross_matches {
                info!(
                    "  CROSS-GAME: {} ≡ {} ({}, conf={:.2})",
                    m.local_formula, m.matched_formula, m.category, m.confidence
                );
            }

            // Feed structural decomposition ops into CausalGraph as richer Rules.
            // Each symbolic op becomes a Rule with the formula as its symbolic_form,
            // which the SymbolResolver can then unify into Laws across games.
            //
            // Rule identity is (category, action, game) — NOT the step. The
            // old step-suffixed names minted a structurally identical node
            // every step, which fed the SymbolResolver a quadratically
            // growing pair set (the 6s/decision, 3GB-RSS regression).
            // Re-observing the same behavior strengthens the one rule.
            for (op, coverage) in &decomp.ops {
                if op.is_structural() && *coverage > 0.5 {
                    let rule_name = format!(
                        "symbolic_{}_a{}_{}",
                        op.category(), action_id, game_id
                    );
                    let conditions: Vec<String> = props.iter()
                        .filter(|p| p.value.as_bool())
                        .map(|p| p.name.clone())
                        .take(5)
                        .collect();

                    let (merged_confidence, merged_evidence) = match self.causal_graph.get_node(&rule_name) {
                        Some(CausalNode::Rule { confidence, evidence_count, .. }) => (
                            (confidence * 0.8 + *coverage as f32 * 0.2).clamp(0.0, 1.0),
                            evidence_count + 1,
                        ),
                        _ => (*coverage as f32, 1),
                    };

                    self.causal_graph.add_node(CausalNode::Rule {
                        name: rule_name.clone(),
                        conditions: conditions.clone(),
                        program: vec![ArcGameState::action_to_dsl(action_id)],
                        confidence: merged_confidence,
                        evidence_count: merged_evidence,
                    });

                    // Also create a StateVariable for the symbolic effect type
                    let effect_name = format!("effect_{}", op.category());
                    self.causal_graph.add_node(CausalNode::StateVariable {
                        name: effect_name.clone(),
                        domain: Domain::GameState,
                        value_type: eustress_vortex_core::ValueType::Float,
                    });
                }
            }

            // ── OBJECT-LEVEL INTERNALIZE ──────────────────────────────────
            // Update object action model with events from this step.
            {
                let obj_model = self.object_action_models
                    .entry(action_id)
                    .or_insert_with(|| ObjectActionModel::new(action_id));
                obj_model.observe(&object_events, self.agent_object_id);
            }

            // Log object-level events for debugging.
            for event in &object_events {
                match event {
                    ObjectEvent::Moved { id, delta_row, delta_col } => {
                        let role = self.object_tracker.get(*id)
                            .map(|t| format!("{:?}", t.classification.role))
                            .unwrap_or_else(|| "?".into());
                        info!(
                            "  OBJ: {} (id={}) moved ({:+.1}, {:+.1}) on action_{}",
                            role, id, delta_row, delta_col, action_id
                        );
                    }
                    ObjectEvent::Disappeared { id } => {
                        info!("  OBJ: id={} disappeared on action_{}", id, action_id);
                    }
                    ObjectEvent::Appeared { id } => {
                        info!("  OBJ: id={} appeared after action_{}", id, action_id);
                    }
                    _ => {}
                }
            }

            // Update goal inference with object events.
            let agent_pos = self.object_tracker.agent()
                .map(|a| a.current.centroid);
            self.goal_engine.update(&self.object_frame, &object_events, agent_pos);
            if let Some(goal) = self.goal_engine.top_goal() {
                info!(
                    "  GOAL: {:?} (conf={:.2}, prox={:.2})",
                    std::mem::discriminant(&goal.goal_type),
                    goal.confidence,
                    goal.proximity_score
                );
            }

            // Update undo explorer with the result of the action.
            if self.in_exploration {
                self.undo_explorer.observe_result(&grid, action_id);
            }

            // Feed observation into CausalGraph as an episode
            let deltas = prev.diff(&grid);
            if !deltas.is_empty() {
                let episode = EpisodeRecord {
                    episode_id: format!("{}_step_{}", game_id, self.step_in_game),
                    task_id: game_id.to_string(),
                    observed_properties: props.clone(),
                    actions_taken: vec![ArcGameState::action_to_dsl(action_id)],
                    state_deltas: deltas,
                    final_score: Score::from_accuracy(
                        if level_advanced { 1.0 } else { 0.1 }
                    ),
                    success: level_advanced,
                    duration_ms: 0,
                };
                self.causal_graph.integrate_episode(&episode);

                // Run SymbolResolver to generalize Rules into Laws.
                // The resolver lives on the model so its attempted-pair
                // cache persists — each call only unifies NEW rule pairs.
                let new_laws = self.symbol_resolver.scan_and_unify(&mut self.causal_graph);
                if new_laws > 0 {
                    info!("  SymbolResolver unified {} new Laws", new_laws);
                }
            }

            // Detect level advance → capture win recipe
            if level_advanced {
                info!("  LEVEL ADVANCE: {} → {}", self.prev_levels, levels);

                // Bayesian update: the previous frame is the winning state —
                // predicates satisfied there gain posterior, others lose it.
                if let Some(ref mut ensemble) = self.predicate_ensemble {
                    ensemble.on_level_complete(prev);
                    if let Some(best) = ensemble.best() {
                        info!(
                            "  PREDICATE: best win-interpretation now {:?} (p={:.2}, conf={})",
                            best.kind, best.posterior, best.confirmations
                        );
                    }
                }
                // New level gets fresh hypotheses (seeded on next observe)
                self.predicate_ensemble = None;

                // New level, new board: click responsiveness doesn't carry over
                self.dead_clicks.clear();
                self.responsive_clicks.clear();
                self.clicked_this_level.clear();
                self.ambient_change_counts.clear();
                self.ambient_observations = 0;
                // Control physics usually persist across levels — keep the
                // map, drop only the outstanding forecast.
                self.forward_model.on_level_advance();

                // Capture the action sequence that produced this level advance
                if !self.current_level_actions.is_empty() {
                    let recipe = WinRecipe {
                        actions: self.current_level_actions.clone(),
                        start_fingerprint: self.level_start_fingerprint,
                        length: self.current_level_actions.len(),
                        replay_successes: 0,
                        origin_game: game_id.to_string(),
                    };
                    info!(
                        "  RECIPE captured: {} actions [{}]",
                        recipe.length,
                        recipe.actions.join(", "),
                    );
                    self.win_recipes.push(recipe.clone());

                    // Add to cross-game library (dedup by fingerprint, max 50)
                    if !self.recipe_library.iter().any(|r| {
                        r.start_fingerprint == recipe.start_fingerprint
                            && r.actions == recipe.actions
                    }) {
                        self.recipe_library.push(recipe);
                        if self.recipe_library.len() > 50 {
                            self.recipe_library
                                .sort_by(|a, b| b.replay_successes.cmp(&a.replay_successes));
                            self.recipe_library.truncate(50);
                        }
                    }
                }

                // ── Level Completion Analysis ──────────────────────────────
                if let Some(ref start_grid) = self.level_start_grid {
                    let record = self.level_analyzer.analyze_completion(
                        self.prev_levels,
                        start_grid,
                        &grid,
                        self.current_level_actions.clone(),
                        self.level_start_frame.as_ref(),
                        Some(&self.object_frame),
                        self.goal_engine.top_goal(),
                        game_id,
                    );
                    info!(
                        "  LEVEL ANALYSIS: {} rules, {} diffs, {} steps",
                        record.transformation_rules.len(),
                        record.object_diffs.len(),
                        record.step_count,
                    );
                }

                // Mark level advance in object tracker and goal engine
                self.object_tracker.mark_level_advance();
                self.goal_engine.reset_for_level();
                self.undo_explorer.reset_for_level();

                // Reset for next level
                self.current_level_actions.clear();
                self.level_start_fingerprint = grid_fingerprint(&grid);
                self.level_start_grid = Some(grid.clone());
                self.level_start_frame = Some(self.object_frame.clone());
                self.plan_buffer.clear();
                self.replay_cursor = None;
                self.steps_since_level_advance = 0;
                self.best_structural_score = 0.0;
                self.actions_since_submit = 0;
            }
        }

        self.prev_levels = levels;

        // Set fingerprint on first observation (level 0 start)
        if self.level_start_fingerprint == 0 {
            self.level_start_fingerprint = grid_fingerprint(&grid);
        }

        // Track structural score for submit timing
        let current_structural = structural_improvement_score(&grid);
        if current_structural > self.best_structural_score {
            self.best_structural_score = current_structural;
        }

        // ── 3. DECIDE ─────────────────────────────────────────────────────
        //
        // Priority chain (vortex-native):
        //   0. Undo-safe exploration (characterize actions via undo probes)
        //   1. Submit (goal proximity > 0.9 OR structural + IRA indicate readiness)
        //   2. Recipe replay (repeat a known winning sequence)
        //   3. Plan buffer (continue a multi-step plan from solve loop)
        //   4. MCTS (when state graph + goals available and solve() weak)
        //   5. Explore (try untried actions to build IRA models)
        //   6. Exploit (solve loop simulation → best predicted action)
        //   7. Diversity fallback (round-robin through effective actions)

        let decision = if self.undo_explorer.should_explore(self.step_in_game) && !self.click_only_game {
            // Undo-safe exploration: probe actions via undo to build state graph
            if let Some(action) = self.undo_explorer.next_action(&available, &grid) {
                self.in_exploration = true;
                let d = PolicyDecision {
                    action: action.to_string(),
                    confidence: 0.3,
                    reasoning: format!(
                        "undo_explore: action={}, steps_used={}/{}",
                        action,
                        self.undo_explorer.steps_used(),
                        12, // default budget
                    ),
                };
                info!(
                    "EXPLORE_UNDO: action={}, reversible={:?}, irreversible={:?}",
                    action,
                    self.undo_explorer.reversible_actions,
                    self.undo_explorer.irreversible_actions,
                );
                // Don't add exploration actions to current_level_actions (they're probes)
                d
            } else {
                self.in_exploration = false;
                self.undo_explorer.finished = true;
                // Fall through to normal decision chain
                self.decide_normal(&available, &grid, &props, levels, current_structural)
            }
        } else {
            self.in_exploration = false;
            self.decide_normal(&available, &grid, &props, levels, current_structural)
        };

        // ── 4. IRA PREDICTION: predict next state for comparison ────────
        // Parse action_id from "6:x,y" or plain "1"
        let action_id: u32 = decision.action
            .split(':').next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        if let Some(model) = self.action_models.get(&action_id) {
            if let Some(predicted) = model.predict(&grid) {
                self.pending_prediction = Some((action_id, predicted));
            }
        }

        // Update scene mirror with the action being taken
        self.scene_mirror.set_last_action(
            &decision.action,
            &decision.reasoning,
        );

        // Track click position for mechanic learning
        if action_id == 6 {
            // Parse "6:x,y" to extract click coordinates
            if let Some(rest) = decision.action.strip_prefix("6:") {
                let parts: Vec<&str> = rest.split(',').collect();
                if let (Some(Ok(x)), Some(Ok(y))) = (
                    parts.first().map(|s| s.parse::<u32>()),
                    parts.get(1).map(|s| s.parse::<u32>()),
                ) {
                    self.last_click_pos = Some((x, y));
                }
            }
            self.clicks_since_submit += 1;
        }

        // Save state for next step
        self.prev_grid = Some(grid);
        self.last_action_id = Some(action_id);
        self.step_in_game += 1;

        decision
    }

    /// Normal decision chain (after undo exploration phase is done).
    /// Detect if agent is stuck repeating the same ineffective action.
    fn is_stuck(&self) -> bool {
        if self.recent_action_effects.len() < 4 { return false; }
        let last: Vec<_> = self.recent_action_effects.iter().rev().take(4).collect();
        let same_action = last.iter().all(|(a, _)| *a == last[0].0);
        let no_effect = last.iter().all(|(_, had_effect)| !had_effect);
        same_action && no_effect
    }

    fn decide_normal(
        &mut self,
        available: &[u32],
        grid: &Grid2D,
        props: &[Property],
        levels: u32,
        current_structural: f64,
    ) -> PolicyDecision {
        // Goal-informed submit: use goal proximity when available
        let goal_ready = self.goal_engine.should_submit();

        if available.contains(&7) && (goal_ready || self.should_submit_now(grid)) {
            info!(
                "SUBMIT: structural={:.3}, goal_ready={}, actions_since={}, step={}",
                current_structural, goal_ready, self.actions_since_submit, self.step_in_game,
            );
            self.clicks_since_submit = 0;
            self.actions_since_submit = 0;
            let d = PolicyDecision {
                action: "7".to_string(),
                confidence: 0.6,
                reasoning: format!(
                    "submit: structural={:.3}, goal_ready={}, step={}",
                    current_structural, goal_ready, self.step_in_game
                ),
            };
            self.current_level_actions.push(d.action.clone());
            return d;
        }

        // STUCK ESCAPE: if repeating same no-effect action, force rotation
        if self.is_stuck() {
            let stuck_action = self.recent_action_effects.back().map(|(a, _)| *a).unwrap_or(0);
            let alternatives: Vec<u32> = available.iter()
                .filter(|&&a| a != stuck_action && a != 7) // exclude submit + stuck action
                .copied()
                .collect();
            if !alternatives.is_empty() {
                let idx = self.diversity_index % alternatives.len();
                self.diversity_index += 1;
                let action = alternatives[idx];
                info!("STUCK ESCAPE: was repeating action_{}, trying action_{}", stuck_action, action);
                let d = self.make_decision(action, 0.25, grid,
                    format!("stuck_escape: breaking action_{} loop, trying action_{}", stuck_action, action));
                self.current_level_actions.push(d.action.clone());
                return d;
            }
        }

        if let Some(d) = self.try_recipe_replay(available, grid) {
            return d;
        }

        if let Some(d) = self.try_plan_buffer(available, grid) {
            return d;
        }

        // FORWARD MODEL: probe controls, then walk toward goals (keyboard games)
        if let Some(d) = self.try_forward_model_move(available, grid) {
            self.current_level_actions.push(d.action.clone());
            return d;
        }

        // Q-LAYER: cross-game value estimates over abstract states,
        // arbitrating over OPTIONS (macro-actions) as well as primitives —
        // hierarchical RL with UCB-style optimism in choose(). Goal-stepping
        // is not an option here because the forward-model stage above
        // already owns that path.
        {
            let (state, _) = self.abstract_state(grid);
            let mut menu: Vec<u32> = available.to_vec();
            if available.contains(&6) && !self.click_targets.is_empty() {
                menu.push(crate::qlearn::OPT_CLICK_TARGETS);
            }
            if !self.recipe_library.is_empty() || !self.win_recipes.is_empty() {
                menu.push(crate::qlearn::OPT_RECIPE);
            }
            if let Some((choice, q, conf)) = self.q_table.choose(&state, &menu) {
                if conf >= 0.3 && q > 0.2 {
                    let decision = match choice {
                        crate::qlearn::OPT_CLICK_TARGETS => Some(self.make_decision(
                            6,
                            (0.4 + conf * 0.4).min(0.85),
                            grid,
                            format!("q_option: {} → click_targets (q={:.2})", state.0, q),
                        )),
                        crate::qlearn::OPT_RECIPE => self.try_recipe_replay(available, grid),
                        a if available.contains(&a) => Some(self.make_decision(
                            a,
                            (0.4 + conf * 0.4).min(0.85),
                            grid,
                            format!("q_layer: {} → action_{} (q={:.2}, conf={:.2})", state.0, a, q, conf),
                        )),
                        _ => None,
                    };
                    if let Some(d) = decision {
                        self.q_table.act(&state, choice);
                        self.current_level_actions.push(d.action.clone());
                        return d;
                    }
                }
            }
        }

        // MCTS: try when we have any exploration data
        let mcts_ready = self.undo_explorer.graph.state_count() > 0;
        if mcts_ready {
            if let Some(d) = self.try_mcts(available, grid) {
                self.current_level_actions.push(d.action.clone());
                return d;
            }
        }

        if self.should_explore(available) {
            let d = self.explore_action(available, grid);
            self.current_level_actions.push(d.action.clone());
            return d;
        }

        let d = self.exploit_action(available, grid, props, levels);
        self.current_level_actions.push(d.action.clone());
        d
    }

    /// Try MCTS search using the exploration graph and IRA priors.
    /// The Representation Accuracy loop for keyboard games.
    ///
    /// Phase 1 (PROBE): while the control map is unknown, spend actions
    /// measuring what each control does to the avatar — the most valuable
    /// action under uncertainty is the one that reduces it fastest.
    /// Phase 2 (PLAN): with a confident control map, a tracked avatar, and a
    /// goal candidate, take the IRA-weighted step that closes the distance.
    /// Every emitted action registers a position forecast, so the model's
    /// trust level is continuously re-measured against reality.
    fn try_forward_model_move(
        &mut self,
        available: &[u32],
        grid: &Grid2D,
    ) -> Option<PolicyDecision> {
        // Keyboard games only: at least one movement action available
        let has_movement = crate::forward_model::MOVEMENT_ACTIONS
            .iter()
            .any(|a| available.contains(a));
        if !has_movement {
            return None;
        }

        self.forward_model.seed_probes(available);

        let avatar = self.object_tracker.agent().map(|a| a.current.centroid);

        // Phase 1: control probing
        if self.forward_model.probing() {
            if let Some(action) = self.forward_model.next_probe() {
                if available.contains(&action) {
                    if let Some(av) = avatar {
                        self.forward_model.predict(action, av);
                    }
                    let d = self.make_decision(
                        action,
                        0.4,
                        grid,
                        format!("fm_probe: measuring action_{} effect on avatar", action),
                    );
                    return Some(d);
                }
            }
        }

        // Phase 2: goal-directed stepping (IRA-gated)
        let avatar = avatar?;
        let goal_id = self.object_frame.goal_candidates.first().copied()?;
        let goal = self.object_tracker.get(goal_id).map(|g| g.current.centroid)?;

        // A* first: route around barrier-classified cells instead of
        // greedily walking into them (the ACTION5-fixation failure mode).
        {
            use crate::object_tracker::ObjectRole;
            let control_map = self.forward_model.control_map();
            if !control_map.is_empty() {
                let blocked: std::collections::HashSet<(i32, i32)> = self
                    .object_tracker
                    .tracked_objects()
                    .iter()
                    .filter(|t| t.classification.role == ObjectRole::StaticBarrier)
                    .flat_map(|t| t.current.cells.iter().map(|&(r, c)| (r as i32, c as i32)))
                    .collect();
                let start = (avatar.0.round() as i32, avatar.1.round() as i32);
                let goal_cell = (goal.0.round() as i32, goal.1.round() as i32);
                if let Some(action) = crate::forward_model::astar_step(
                    &control_map,
                    start,
                    goal_cell,
                    &|r, c| blocked.contains(&(r, c)),
                    (grid.height as i32, grid.width as i32),
                ) {
                    if available.contains(&action) {
                        let ira = self.forward_model.action_ira.get(&action).copied().unwrap_or(0.5);
                        if ira >= 0.35 {
                            self.forward_model.predict(action, avatar);
                            let d = self.make_decision(
                                action,
                                (0.5 + ira * 0.4).min(0.9),
                                grid,
                                format!(
                                    "fm_astar: action_{} pathing to goal obj_{} (ira={:.2})",
                                    action, goal_id, ira
                                ),
                            );
                            return Some(d);
                        }
                    }
                }
            }
        }

        let (action, progress, ira) = self.forward_model.step_toward(avatar, goal)?;
        if !available.contains(&action) || ira < 0.35 {
            return None;
        }
        self.forward_model.predict(action, avatar);
        let d = self.make_decision(
            action,
            (0.5 + ira * 0.4).min(0.9),
            grid,
            format!(
                "fm_goal_step: action_{} toward goal obj_{} (progress={:.2}, ira={:.2})",
                action, goal_id, progress, ira
            ),
        );
        Some(d)
    }

    fn try_mcts(&self, available: &[u32], grid: &Grid2D) -> Option<PolicyDecision> {
        let current_hash = GridHash::from_grid(grid);

        // Build IRA scores from action models
        let mut ira_scores = HashMap::new();
        for (&action_id, model) in &self.action_models {
            ira_scores.insert(action_id, model.ira_score);
        }

        // Build goal scores from goal engine
        let mut goal_scores = HashMap::new();
        let agent_pos = self.object_tracker.agent().map(|a| a.current.centroid);
        let current_prox = self.goal_engine.proximity_score(grid, agent_pos);
        goal_scores.insert(current_hash, current_prox);

        // Add goal scores for known states in the graph
        // (We can't easily compute proximity for states we don't have grids for,
        //  but known transitions tell us which direction moves toward goals)
        if let Some(edges) = self.undo_explorer.graph.edges.get(&current_hash) {
            for edge in edges {
                // States reached by reversible actions are slightly valued
                let base_score = if edge.is_reversible { 0.3 } else { 0.2 };
                goal_scores.entry(edge.target).or_insert(base_score);
            }
        }

        let mut tree = MctsTree::new(current_hash);
        let result = tree.search(
            &self.undo_explorer.graph,
            available,
            &ira_scores,
            &goal_scores,
            &self.mcts_budget,
        )?;

        // Only use MCTS result if it's confident enough
        if result.confidence < 0.1 || result.visit_count < 2 {
            return None;
        }

        Some(PolicyDecision {
            action: result.action.to_string(),
            confidence: result.confidence,
            reasoning: format!(
                "mcts: action={}, visits={}, value={:.3}, iters={}, nodes={}",
                result.action, result.visit_count, result.value, result.iterations,
                tree.node_count()
            ),
        })
    }

    /// Should we explore (try untried actions) or exploit (use learned model)?
    fn should_explore(&self, available: &[u32]) -> bool {
        // Never explore if we're replaying a recipe or executing a plan
        if self.replay_cursor.is_some() || !self.plan_buffer.is_empty() {
            return false;
        }

        let untried: Vec<u32> = available
            .iter()
            .filter(|a| !self.tried_actions.contains(a))
            // Skip actions we already know have no effect from prior observations
            .filter(|&&a| {
                self.action_models.get(&a).map_or(true, |m| {
                    m.observed_patches.len() < 2 || m.has_effect
                })
            })
            .copied()
            .collect();

        // Tighter budget: try each untried action once (not 2×)
        if !untried.is_empty() && self.step_in_game < available.len() as u32 {
            return true;
        }

        // Re-explore only if IRA is very low and we're very early
        if self.global_ira < 0.15 && self.step_in_game < 10 {
            return true;
        }

        false
    }

    /// Exploration phase: try each action to learn what it does.
    /// Prioritizes click actions (ACTION6) and skips known no-effect actions.
    fn explore_action(&mut self, available: &[u32], grid: &Grid2D) -> PolicyDecision {
        let untried: Vec<u32> = available
            .iter()
            .filter(|a| !self.tried_actions.contains(a))
            .filter(|&&a| {
                self.action_models.get(&a).map_or(true, |m| {
                    m.observed_patches.len() < 2 || m.has_effect
                })
            })
            .copied()
            .collect();

        // Priority: ACTION6 (click) first — highest information value for interactive games
        let action_id = if let Some(&a) = untried.iter().find(|&&a| a == 6) {
            a
        } else if let Some(&a) = untried.first() {
            a
        } else {
            // All effective actions tried — pick lowest observation count
            *available
                .iter()
                .filter(|&&a| {
                    self.action_models.get(&a).map_or(true, |m| m.has_effect)
                })
                .min_by_key(|&&a| {
                    self.action_models
                        .get(&a)
                        .map_or(0, |m| m.observed_patches.len())
                })
                .unwrap_or(&available[0])
        };

        self.tried_actions.push(action_id);
        info!(
            "EXPLORE: trying action_{} ({}/{} tried, {} untried effective)",
            action_id,
            self.tried_actions.len(),
            available.len(),
            untried.len().saturating_sub(1),
        );

        self.make_decision(action_id, 0.3, grid, format!(
            "explore: action_{} ({} untried remaining)",
            action_id, untried.len().saturating_sub(1)
        ))
    }

    /// Exploitation phase: use Eustress solve loop + symbolic archetypes to plan.
    fn exploit_action(
        &mut self,
        available: &[u32],
        grid: &Grid2D,
        props: &[Property],
        levels: u32,
    ) -> PolicyDecision {
        // ── SYMBOLIC ARCHETYPE SUGGESTION ───────────────────────────────
        // Before running the full solve loop, check if cross-game archetypes
        // suggest a specific action based on observed properties.
        let property_hints = self.property_to_desired_effect(props);
        let mut archetype_suggestion: Option<(u32, f32, String)> = None;

        for (desired_effect, hint_conf) in &property_hints {
            if let Some((action_id, arch_conf)) = self.decomposer.suggest_action_for_effect(
                &self.current_game,
                desired_effect,
            ) {
                if available.contains(&action_id) {
                    let combined_conf = hint_conf * arch_conf;
                    if archetype_suggestion.as_ref().map_or(true, |(_, c, _)| combined_conf > *c) {
                        archetype_suggestion = Some((action_id, combined_conf, desired_effect.clone()));
                    }
                }
            }
        }

        if let Some((action_id, conf, effect)) = &archetype_suggestion {
            if *conf > 0.6 {
                info!(
                    "ARCHETYPE: cross-game suggests action_{} for '{}' (conf={:.2})",
                    action_id, effect, conf
                );
                return self.make_decision(
                    *action_id,
                    *conf,
                    grid,
                    format!("archetype: action_{} for {} (cross-game conf={:.2})", action_id, effect, conf),
                );
            }
        }

        // ── HYPOTHESIZE: ask CausalGraph for suggestions ────────────────
        let hypotheses = self.causal_graph.suggest_hypotheses(
            props,
            &GoalPredicate::ScoreThreshold(0.5),
            16,
        );

        info!(
            "HYPOTHESIZE: CausalGraph suggests {} hypotheses from {} nodes, {} edges",
            hypotheses.len(),
            self.causal_graph.node_count(),
            self.causal_graph.edge_count(),
        );

        // Log archetype summary for debugging
        let archetypes = self.decomposer.archetype_summary();
        if !archetypes.is_empty() {
            info!(
                "  ARCHETYPES: {} known ({} cross-game)",
                archetypes.len(),
                archetypes.iter().filter(|(_, gc, _)| *gc > 1).count()
            );
        }

        // ── SIMULATE: use HypothesisTree via solve loop ─────────────────
        let game_state = ArcGameState::new(
            grid.clone(),
            levels,
            self.action_models.clone(),
            available.to_vec(),
        );

        let goal_state = ArcGameState::new(
            grid.clone(),
            levels + 1,
            self.action_models.clone(),
            available.to_vec(),
        );

        let budget = SimulationBudget {
            max_total_steps: 500,
            max_branch_depth: 5,
            max_active_branches: 16,
            prune_threshold: 0.05,
            time_limit: Some(std::time::Duration::from_millis(200)),
        };

        let result = eustress_vortex_core::solve(&game_state, &goal_state, &mut self.causal_graph, &budget);

        info!(
            "SIMULATE: solve returned {} ops, score={:.3}, solved={}",
            result.program.len(),
            result.score.accuracy,
            result.solved,
        );

        // ── EVALUATE: extract best action, weighted by IRA reliability ────
        // The solve loop's score is only as good as the ActionModel predictions.
        // Weight confidence by average IRA of the actions in the plan.
        if let Some(first_op) = result.program.first() {
            if let Some(action_id) = ArcGameState::dsl_to_action(first_op) {
                if available.contains(&action_id) {
                    // Weight solve score by IRA reliability of involved actions
                    let plan_ira: f32 = result.program.iter()
                        .filter_map(|op| ArcGameState::dsl_to_action(op))
                        .filter_map(|aid| self.action_models.get(&aid))
                        .map(|m| if m.prediction_count > 0 { m.ira_score } else { 0.3 })
                        .sum::<f32>()
                        / result.program.len().max(1) as f32;
                    let confidence = result.score.accuracy as f32 * plan_ira.max(0.1);

                    info!(
                        "  IRA-weighted: raw_score={:.3}, plan_ira={:.3}, confidence={:.3}",
                        result.score.accuracy, plan_ira, confidence
                    );

                    // Buffer remaining plan steps if multi-step and confident enough
                    if result.program.len() > 1 && confidence > 0.3 {
                        let mut predicted_states = Vec::new();
                        let mut sim_state = game_state.clone();
                        // Simulate first action
                        sim_state = sim_state.apply(first_op);
                        // Simulate remaining actions to get predicted states
                        for op in &result.program[1..] {
                            sim_state = sim_state.apply(op);
                            predicted_states.push(sim_state.grid.clone());
                        }
                        self.plan_buffer = PlanBuffer {
                            actions: result.program[1..].to_vec(),
                            predicted_states,
                            plan_score: result.score.accuracy,
                            computed_at_step: self.step_in_game,
                        };
                        info!(
                            "PLAN BUFFER: stored {} remaining steps (score={:.3})",
                            self.plan_buffer.actions.len(), result.score.accuracy
                        );
                    }

                    info!(
                        "EVALUATE: solve suggests action_{} (confidence={:.3})",
                        action_id, confidence
                    );
                    return self.make_decision(
                        action_id,
                        confidence,
                        grid,
                        format!(
                            "vortex solve: {} ({} ops planned, score={:.3})",
                            first_op.name,
                            result.program.len(),
                            result.score.accuracy
                        ),
                    );
                }
            }
        }

        // ── MCTS FALLBACK: try MCTS before blind fallback ────────────────
        if self.undo_explorer.graph.state_count() > 0 {
            if let Some(d) = self.try_mcts(available, grid) {
                info!("MCTS FALLBACK: action={} (after solve failed)", d.action);
                return d;
            }
        }

        // ── ARCHETYPE FALLBACK: use lower-confidence archetype if available
        if let Some((action_id, conf, effect)) = archetype_suggestion {
            info!("ARCHETYPE FALLBACK: action_{} for '{}' (conf={:.2})", action_id, effect, conf);
            return self.make_decision(
                action_id,
                conf,
                grid,
                format!("archetype fallback: action_{} for {} (conf={:.2})", action_id, effect, conf),
            );
        }

        // ── FALLBACK: use best known effective action ───────────────────
        let best_action = self.best_effective_action(available, grid);
        info!("FALLBACK: using best effective action_{}", best_action);

        self.make_decision(
            best_action,
            0.2,
            grid,
            format!("fallback: best effective action_{}", best_action),
        )
    }

    /// Map observed grid properties to desired symbolic effect categories.
    /// This is the bridge between GridAnalyzer properties and SymbolicOp categories.
    fn property_to_desired_effect(&self, props: &[Property]) -> Vec<(String, f32)> {
        let mut effects = Vec::new();
        for p in props {
            match p.name.as_str() {
                "has_unsupported_objects" if p.value.as_bool() => {
                    effects.push(("gravitational_settling".into(), 0.8));
                }
                "symmetric_horizontal" if !p.value.as_bool() => {
                    effects.push(("spatial_reflection".into(), 0.5));
                }
                "symmetric_vertical" if !p.value.as_bool() => {
                    effects.push(("spatial_reflection".into(), 0.5));
                }
                "symmetric_rotational_180" if !p.value.as_bool() => {
                    effects.push(("spatial_rotation".into(), 0.4));
                }
                "has_border" if !p.value.as_bool() => {
                    effects.push(("border_op".into(), 0.3));
                }
                _ => {}
            }
        }
        effects
    }

    /// Pick the best known effective action using IRA models + structural tracking.
    ///
    /// Scoring factors:
    /// - IRA score (prediction reliability for this action)
    /// - Has observable effect
    /// - Structural improvement ratio (times improved / times used)
    /// - Property-guided boost (floating objects → directional actions)
    /// - Stuck-action penalty (recent ineffective uses)
    /// - Diversity bonus (actions not tried recently get a boost to prevent stagnation)
    fn best_effective_action(&mut self, available: &[u32], grid: &Grid2D) -> u32 {
        let props = grid.analyze();
        let has_floating = props.iter().any(|p| p.name == "has_unsupported_objects" && p.value.as_bool());

        // Count recent ineffective uses per action
        let mut recent_fails: HashMap<u32, u32> = HashMap::new();
        let mut recent_uses: HashMap<u32, u32> = HashMap::new();
        for &(act, had_effect) in &self.recent_action_effects {
            *recent_uses.entry(act).or_insert(0) += 1;
            if !had_effect {
                *recent_fails.entry(act).or_insert(0) += 1;
            }
        }

        // Filter to non-submit actions for scoring
        let non_submit: Vec<u32> = available.iter().copied().filter(|&a| a != 7).collect();
        if non_submit.is_empty() {
            return *available.first().unwrap_or(&1);
        }

        // Score each available action
        let mut scores: Vec<(u32, f32)> = non_submit
            .iter()
            .map(|&a| {
                let mut s = 0.0f32;

                // IRA model score
                if let Some(m) = self.action_models.get(&a) {
                    if m.has_effect { s += 2.0; }
                    if m.is_deterministic { s += 0.5; }
                    s += m.ira_score;
                } else {
                    s += 1.0; // Unknown actions get exploration priority
                }

                // Structural improvement ratio (actions that improve structure score higher)
                if let Some(&(improved, used)) = self.action_structural_scores.get(&a) {
                    if used > 0 {
                        let ratio = improved as f32 / used as f32;
                        s += ratio * 3.0; // Strong boost for structurally improving actions
                    }
                }

                // Property-guided boost
                if has_floating && (a == 1 || a == 2) { s += 1.5; }

                // Penalize recent ineffective uses
                let fails = *recent_fails.get(&a).unwrap_or(&0);
                s -= fails as f32 * 1.5;

                // Diversity bonus: actions used less recently get a boost
                let uses = *recent_uses.get(&a).unwrap_or(&0);
                if uses == 0 {
                    s += 1.0; // Haven't used this recently — try it
                }

                (a, s)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // If all scores are very similar (stagnation), use round-robin diversity
        if scores.len() >= 2 {
            let top = scores[0].1;
            let second = scores[1].1;
            if (top - second).abs() < 0.5 && self.steps_since_level_advance > 10 {
                // Round-robin through ALL non-submit actions (not just known-effective)
                // Exclude only actions proven 3+ times to have no effect
                let candidates: Vec<u32> = non_submit.iter().copied()
                    .filter(|&a| {
                        self.action_models.get(&a).map_or(true, |m| {
                            m.has_effect || m.observed_patches.len() < 3
                        })
                    })
                    .collect();
                if !candidates.is_empty() {
                    let idx = self.diversity_index % candidates.len();
                    self.diversity_index += 1;
                    let action = candidates[idx];
                    info!(
                        "DIVERSITY: round-robin action_{} (idx={}/{}, stagnant {} steps)",
                        action, idx, candidates.len(), self.steps_since_level_advance
                    );
                    return action;
                }
            }
        }

        scores.first().map(|&(a, _)| a).unwrap_or(1)
    }

    // ─── Win Recipe Replay ─────────────────────────────────────────────

    /// Try to continue or start a recipe replay. Returns None if no recipe applies.
    fn try_recipe_replay(&mut self, available: &[u32], grid: &Grid2D) -> Option<PolicyDecision> {
        // Continue an active replay
        if let Some((recipe_idx, step_idx)) = self.replay_cursor {
            if let Some(recipe) = self.win_recipes.get(recipe_idx).cloned() {
                if step_idx < recipe.actions.len() {
                    // Verify the last replay step had some effect (on-track check)
                    let on_track = self.prev_grid.as_ref().map_or(true, |prev| {
                        prev.cells_changed(grid) > 0
                    });

                    if on_track {
                        let action = recipe.actions[step_idx].clone();
                        self.replay_cursor = Some((recipe_idx, step_idx + 1));
                        info!(
                            "RECIPE REPLAY: step {}/{} → action {}",
                            step_idx + 1, recipe.length, action
                        );
                        self.current_level_actions.push(action.clone());

                        let action_id: u32 = action.split(':').next()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        if !available.contains(&action_id) {
                            info!("RECIPE REPLAY: action {} not available, abandoning", action_id);
                            self.replay_cursor = None;
                            return None;
                        }

                        return Some(PolicyDecision {
                            action,
                            confidence: 0.8,
                            reasoning: format!(
                                "recipe replay: step {}/{} (from level {})",
                                step_idx + 1, recipe.length, recipe_idx
                            ),
                        });
                    } else {
                        info!("RECIPE REPLAY: no grid change at step {}, abandoning", step_idx);
                        self.replay_cursor = None;
                    }
                } else {
                    // Recipe exhausted
                    info!("RECIPE REPLAY: completed {} steps", recipe.length);
                    // Mark success if we're about to see a level advance
                    if let Some(r) = self.win_recipes.get_mut(recipe_idx) {
                        r.replay_successes += 1;
                    }
                    self.replay_cursor = None;
                }
            } else {
                self.replay_cursor = None;
            }
        }

        // Try to START a new replay if we have recipes
        if self.replay_cursor.is_none() && !self.win_recipes.is_empty() {
            let current_fp = grid_fingerprint(grid);

            // Exact fingerprint match
            if let Some(idx) = self.win_recipes.iter().position(|r| r.start_fingerprint == current_fp) {
                info!(
                    "RECIPE MATCH (fingerprint): starting replay of recipe {} ({} actions)",
                    idx, self.win_recipes[idx].length
                );
                self.replay_cursor = Some((idx, 0));
                return self.try_recipe_replay(available, grid); // recurse to take first step
            }

            // Same-game heuristic: after level 0, try the most recent recipe
            // (levels in the same game often have the same solution pattern)
            if self.prev_levels > 0 {
                let idx = self.win_recipes.len() - 1;
                info!(
                    "RECIPE MATCH (same-game heuristic): trying recipe {} ({} actions)",
                    idx, self.win_recipes[idx].length
                );
                self.replay_cursor = Some((idx, 0));
                return self.try_recipe_replay(available, grid);
            }
        }

        // Cross-game: try recipe_library for fingerprint match with prior successes
        if self.replay_cursor.is_none() && self.win_recipes.is_empty() {
            let current_fp = grid_fingerprint(grid);
            if let Some(lib_recipe) = self.recipe_library.iter()
                .find(|r| r.start_fingerprint == current_fp && r.replay_successes > 0)
                .cloned()
            {
                info!(
                    "RECIPE MATCH (cross-game): {} ({} actions, {} prior successes)",
                    lib_recipe.origin_game, lib_recipe.length, lib_recipe.replay_successes
                );
                self.win_recipes.push(lib_recipe);
                let idx = self.win_recipes.len() - 1;
                self.replay_cursor = Some((idx, 0));
                return self.try_recipe_replay(available, grid);
            }
        }

        None
    }

    // ─── Plan Buffer Execution ───────────────────────────────────────────

    /// Try to execute the next step of a buffered plan. Returns None if no plan active.
    fn try_plan_buffer(&mut self, available: &[u32], grid: &Grid2D) -> Option<PolicyDecision> {
        if self.plan_buffer.is_empty() {
            return None;
        }

        // Stale plan check: abandon if >8 steps since computed
        if self.step_in_game > self.plan_buffer.computed_at_step + 8 {
            info!("PLAN BUFFER: stale (computed {} steps ago), abandoning",
                  self.step_in_game - self.plan_buffer.computed_at_step);
            self.plan_buffer.clear();
            return None;
        }

        // Deviation check: compare actual grid to predicted
        if let Some(expected) = self.plan_buffer.predicted_states.first() {
            let accuracy = grid.cell_accuracy(expected) as f32;
            if accuracy < 0.85 {
                info!(
                    "PLAN BUFFER: deviation detected (accuracy={:.3}), abandoning {} remaining steps",
                    accuracy, self.plan_buffer.actions.len()
                );
                self.plan_buffer.clear();
                return None;
            }
        }

        // Pop next action
        let next_op = self.plan_buffer.actions.remove(0);
        if !self.plan_buffer.predicted_states.is_empty() {
            self.plan_buffer.predicted_states.remove(0);
        }

        if let Some(action_id) = ArcGameState::dsl_to_action(&next_op) {
            if available.contains(&action_id) {
                let remaining = self.plan_buffer.actions.len();
                info!(
                    "PLAN BUFFER: executing {} ({} remaining, score={:.3})",
                    next_op.name, remaining, self.plan_buffer.plan_score
                );
                let decision = self.make_decision(
                    action_id,
                    self.plan_buffer.plan_score as f32,
                    grid,
                    format!("plan buffer: {} ({} remaining)", next_op.name, remaining),
                );
                self.current_level_actions.push(decision.action.clone());
                return Some(decision);
            }
        }

        // Action not available — clear buffer
        info!("PLAN BUFFER: action {} not available, clearing", next_op.name);
        self.plan_buffer.clear();
        None
    }

    /// Build a PolicyDecision with click coordinates if needed.
    fn make_decision(
        &mut self,
        action_id: u32,
        confidence: f32,
        grid: &Grid2D,
        reasoning: String,
    ) -> PolicyDecision {
        // For ACTION6 (click), compute click coordinates
        let action_str = if action_id == 6 {
            let (x, y) = self.compute_click_target(grid);
            format!("6:{},{}", x, y)
        } else {
            action_id.to_string()
        };

        PolicyDecision {
            action: action_str,
            confidence,
            reasoning,
        }
    }

    /// Compute click target using learned mechanics and goal inference.
    /// Rebuilt every step for click-only games (reactive targeting).
    fn compute_click_target(&mut self, grid: &Grid2D) -> (u32, u32) {
        // For click-only games: always rebuild targets based on current grid state
        if self.click_only_game || self.click_targets.is_empty() {
            self.build_smart_click_targets(grid);
        }

        // Real-time adaptation: drop cells already proven dead this level,
        // never click ambient cells (move counters, timers — clicking the
        // energy bar costs a move and does nothing), and explore
        // never-clicked cells before re-trying clicked ones. Responsive
        // cells stay eligible (re-clicking is how toggle/cycle mechanics
        // make progress).
        let ambient_counts = &self.ambient_change_counts;
        let ambient_obs = self.ambient_observations;
        let dead = &self.dead_clicks;
        self.click_targets.retain(|t| {
            if dead.contains(t) {
                return false;
            }
            if ambient_obs < 5 {
                return true;
            }
            let count = ambient_counts
                .get(&(t.1 as usize, t.0 as usize))
                .copied()
                .unwrap_or(0);
            (count as f32) / (ambient_obs as f32) <= 0.4
        });
        self.click_targets.sort_by_key(|t| {
            if !self.clicked_this_level.contains(t) {
                0 // never tried — highest priority
            } else if self.responsive_clicks.contains(t) {
                1 // known responsive — worth re-trying
            } else {
                2
            }
        });

        if self.click_targets.is_empty() {
            return (grid.width as u32 / 2, grid.height as u32 / 2);
        }

        // Alternate fresh exploration with re-activation of the most
        // responsive cell: confirm-button mechanics (tn36's winning run
        // pressed (0,0) sixteen times between target clicks) never fire
        // under a pure unclicked-first sweep. Every 4th click re-presses
        // the strongest known-responsive cell.
        self.click_count += 1;
        // Intervention scheduling: periodically test the scientific
        // frontier — re-click a correlated-but-unverified claim's cell with
        // a DECLARED prediction, so verification (not just correlation)
        // accumulates. This is the ledger driving behavior.
        if self.click_count % 8 == 0 {
            if let Some(claim) = self.ledger.frontier(1).first() {
                // claim ids look like "click(x,y)->effect"
                if let Some(rest) = claim.strip_prefix("click(") {
                    if let Some(end) = rest.find(')') {
                        let coords: Vec<u32> = rest[..end]
                            .split(',')
                            .filter_map(|v| v.parse().ok())
                            .collect();
                        if coords.len() == 2 {
                            let expected = claim.split("->").nth(1).unwrap_or("none").to_string();
                            self.ledger.declare_intent(claim, &expected);
                            return (coords[0], coords[1]);
                        }
                    }
                }
            }
        }
        if self.click_count % 4 == 0 {
            if let Some(&confirm) = self.responsive_clicks.iter().next() {
                if !self.dead_clicks.contains(&confirm) {
                    return confirm;
                }
            }
        }
        let target = self.click_targets[0];
        self.clicked_this_level.insert(target);
        target
    }

    /// Build click targets using IRA-informed simulation.
    ///
    /// Strategy:
    /// 1. If we've learned the click mechanic, simulate clicking each candidate
    ///    cell and score the resulting grid via `structural_improvement_score`.
    ///    Pick cells whose simulated click produces the highest improvement.
    /// 2. Use GridAnalyzer properties to identify "defect" cells — cells whose
    ///    local context suggests they are anomalous (via property analysis).
    /// 3. Fallback: structural analysis targets (centroids, bbox, non-bg cells).
    fn build_smart_click_targets(&mut self, grid: &Grid2D) {
        let mut targets: Vec<(u32, u32)> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // ── Strategy 0: object-first targeting ──────────────────────────
        // Click games are about objects, not pixels: try one representative
        // cell per tracked object before any cell-level heuristic. Smaller
        // objects first — buttons and toggles are small; backdrops are big.
        // Combined with dead-click learning this sweeps the interactive
        // surface in ~object-count actions instead of ~cell-count.
        {
            use crate::object_tracker::ObjectRole;
            let mut objs: Vec<(usize, (u32, u32))> = self
                .object_tracker
                .tracked_objects()
                .iter()
                .filter(|t| t.current.area >= 1 && t.current.area < (grid.width * grid.height) / 4)
                .filter_map(|t| {
                    // Representative cell: the member cell nearest the centroid
                    // (a centroid of an L-shape can fall outside the object).
                    let (cy, cx) = t.current.centroid;
                    // Goal-classified objects outrank everything (r11l — the
                    // first game to score — scored because its goals were
                    // identified and clicked); then smallest-first.
                    let rank = if t.classification.role == ObjectRole::Goal {
                        t.current.area
                    } else {
                        t.current.area + 100_000
                    };
                    t.current.cells.iter()
                        .min_by_key(|(r, c)| {
                            let dr = *r as f32 - cy;
                            let dc = *c as f32 - cx;
                            ((dr * dr + dc * dc) * 100.0) as i64
                        })
                        .map(|(r, c)| (rank, (*c as u32, *r as u32)))
                })
                .collect();
            objs.sort_by_key(|(rank, _)| *rank);
            for (_, pt) in objs {
                if !self.dead_clicks.contains(&pt) && seen.insert(pt) {
                    targets.push(pt);
                }
            }
            if !targets.is_empty() {
                info!("  CLICK STRATEGY: object-first → {} object targets", targets.len());
            }
        }

        // ── Strategy 1: IRA-simulated click scoring ─────────────────────
        // If we know the click mechanic (toggle/paint), simulate clicking each
        // candidate cell and rank by structural improvement.
        if let Some(ref mechanic) = self.click_mechanic.clone() {
            let current_score = structural_improvement_score(grid);
            let mut scored_targets: Vec<(u32, u32, f64)> = Vec::new();

            // Generate candidate cells to simulate
            let _bg = grid.background_color();
            for r in 0..grid.height {
                for c in 0..grid.width {
                    // Simulate what clicking this cell would do
                    let mut simulated = grid.clone();
                    let old_val = simulated.cells[r][c];
                    let new_val = match mechanic {
                        ClickMechanic::Toggle { color_a, color_b } => {
                            if old_val == *color_a { *color_b }
                            else if old_val == *color_b { *color_a }
                            else { continue; } // cell not part of toggle pair
                        }
                        ClickMechanic::Paint(target) => {
                            if old_val == *target { continue; } // already painted
                            *target
                        }
                        ClickMechanic::Cycle(colors) => {
                            if let Some(pos) = colors.iter().position(|&v| v == old_val) {
                                colors[(pos + 1) % colors.len()]
                            } else {
                                continue;
                            }
                        }
                    };

                    simulated.cells[r][c] = new_val;
                    let sim_score = structural_improvement_score(&simulated);
                    if sim_score > current_score {
                        scored_targets.push((c as u32, r as u32, sim_score));
                    }
                }
            }

            // Sort by improvement (highest first)
            scored_targets.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            for (x, y, _score) in scored_targets.iter().take(32) {
                if seen.insert((*x, *y)) {
                    targets.push((*x, *y));
                }
            }
            if !targets.is_empty() {
                info!(
                    "  CLICK STRATEGY: IRA-simulated → {} improving targets (best Δ={:.3})",
                    targets.len(),
                    scored_targets.first().map(|t| t.2 - current_score).unwrap_or(0.0),
                );
            }
        }

        // ── Strategy 2: Property-guided defect detection ────────────────
        // Use GridAnalyzer to find cells that look anomalous in their local context.
        // This is data-driven (via grid properties) not hardcoded pattern matching.
        if targets.is_empty() {
            let bg = grid.background_color();
            // Find cells that differ from their majority-neighbor color
            for r in 1..grid.height.saturating_sub(1) {
                for c in 1..grid.width.saturating_sub(1) {
                    let v = grid.cells[r][c];
                    if v == bg { continue; }
                    // Count neighbor colors
                    let mut color_counts: HashMap<u8, u32> = HashMap::new();
                    for &(dr, dc) in &[(-1i32, 0), (1, 0), (0, -1i32), (0, 1)] {
                        let nr = (r as i32 + dr) as usize;
                        let nc = (c as i32 + dc) as usize;
                        if nr < grid.height && nc < grid.width {
                            let nv = grid.cells[nr][nc];
                            if nv != bg {
                                *color_counts.entry(nv).or_insert(0) += 1;
                            }
                        }
                    }
                    // If this cell differs from ALL non-bg neighbors, it's a defect
                    if let Some((&majority_color, &count)) = color_counts.iter().max_by_key(|e| e.1) {
                        if v != majority_color && count >= 3 {
                            let pt = (c as u32, r as u32);
                            if seen.insert(pt) { targets.push(pt); }
                        }
                    }
                }
            }
            if !targets.is_empty() {
                info!(
                    "  CLICK STRATEGY: neighbor-defect analysis → {} targets",
                    targets.len()
                );
            }
        }

        // ── Strategy 3: Grid region difference detection ────────────────
        // Detect if grid has two halves (divider row/col) and target differences
        if targets.is_empty() {
            self.detect_region_differences(grid, &mut targets, &mut seen);
        }

        // ── Fallback: structural analysis targets ───────────────────────
        if targets.is_empty() {
            self.build_fallback_click_targets(grid, &mut targets, &mut seen);
        }

        // ── Coverage extension ───────────────────────────────────────────
        // The strategies above propose a fixed head of candidates; once
        // those are all clicked (and possibly dead), exploration must move
        // on to the rest of the board. Append every untried non-background
        // cell so the level's clickable surface is eventually covered.
        // PLUS all border cells regardless of color: the tn36 coverage
        // diff showed 67 of 96 winning cells excluded, clustered on
        // borders/corners — switches live where bg-filters and 1..h-1
        // loops never look. Hidden affordances have no signifier.
        let bg = grid.background_color();
        for r in 0..grid.height {
            for c in 0..grid.width {
                let is_border = r == 0 || c == 0 || r + 1 == grid.height || c + 1 == grid.width;
                if grid.cells[r][c] == bg && !is_border {
                    continue;
                }
                let pt = (c as u32, r as u32);
                if !self.clicked_this_level.contains(&pt)
                    && !self.dead_clicks.contains(&pt)
                    && seen.insert(pt)
                {
                    targets.push(pt);
                }
            }
        }

        // Cap targets to bound per-step sorting work
        targets.truncate(512);
        self.click_targets = targets;
    }

    /// Detect grid regions separated by divider rows/columns and target differences.
    fn detect_region_differences(
        &self,
        grid: &Grid2D,
        targets: &mut Vec<(u32, u32)>,
        seen: &mut std::collections::HashSet<(u32, u32)>,
    ) {
        // Check for horizontal divider (uniform color row)
        if grid.height >= 4 {
            let mid_r = grid.height / 2;
            let divider = (mid_r.saturating_sub(1)..=(mid_r + 1).min(grid.height - 1))
                .find(|&r| {
                    let first = grid.cells[r][0];
                    grid.cells[r].iter().all(|&v| v == first)
                });
            if let Some(div_r) = divider {
                for r in (div_r + 1)..grid.height {
                    let src_r = r - div_r - 1;
                    if src_r < div_r {
                        for c in 0..grid.width {
                            if grid.cells[r][c] != grid.cells[src_r][c] {
                                let pt = (c as u32, r as u32);
                                if seen.insert(pt) { targets.push(pt); }
                            }
                        }
                    }
                }
                if !targets.is_empty() {
                    info!("  CLICK STRATEGY: region-diff (divider row {}) → {} targets", div_r, targets.len());
                    return;
                }
            }
        }

        // Check for vertical divider (uniform color column)
        if grid.width >= 4 {
            let mid_c = grid.width / 2;
            let divider = (mid_c.saturating_sub(1)..=(mid_c + 1).min(grid.width - 1))
                .find(|&c| {
                    let first = grid.cells[0][c];
                    (0..grid.height).all(|r| grid.cells[r][c] == first)
                });
            if let Some(div_c) = divider {
                for r in 0..grid.height {
                    for c in (div_c + 1)..grid.width {
                        let src_c = c - div_c - 1;
                        if src_c < div_c && grid.cells[r][c] != grid.cells[r][src_c] {
                            let pt = (c as u32, r as u32);
                            if seen.insert(pt) { targets.push(pt); }
                        }
                    }
                }
                if !targets.is_empty() {
                    info!("  CLICK STRATEGY: region-diff (divider col {}) → {} targets", div_c, targets.len());
                }
            }
        }
    }

    /// Fallback click targeting: color centroids, bbox, grid scan.
    fn build_fallback_click_targets(
        &self,
        grid: &Grid2D,
        targets: &mut Vec<(u32, u32)>,
        seen: &mut std::collections::HashSet<(u32, u32)>,
    ) {
        let bg = grid.background_color();

        // Color region centroids
        let mut color_groups: HashMap<u8, Vec<(u32, u32)>> = HashMap::new();
        for r in 0..grid.height {
            for c in 0..grid.width {
                let v = grid.cells[r][c];
                if v != bg {
                    color_groups.entry(v).or_default().push((c as u32, r as u32));
                }
            }
        }
        for (_color, cells) in &color_groups {
            if cells.is_empty() { continue; }
            let cx: u32 = cells.iter().map(|&(x, _)| x).sum::<u32>() / cells.len() as u32;
            let cy: u32 = cells.iter().map(|&(_, y)| y).sum::<u32>() / cells.len() as u32;
            if seen.insert((cx, cy)) { targets.push((cx, cy)); }
        }

        // Content bbox center + corners
        let (min_r, max_r, min_c, max_c) = content_bbox(grid, bg);
        let center_x = (min_c + max_c) / 2;
        let center_y = (min_r + max_r) / 2;
        for &(x, y) in &[
            (center_x, center_y),
            (min_c, min_r), (max_c, min_r), (min_c, max_r), (max_c, max_r),
        ] {
            if seen.insert((x as u32, y as u32)) { targets.push((x as u32, y as u32)); }
        }

        // Non-background cells
        for r in 0..grid.height {
            for c in 0..grid.width {
                if grid.cells[r][c] != bg && targets.len() < 64 {
                    let pt = (c as u32, r as u32);
                    if seen.insert(pt) { targets.push(pt); }
                }
            }
        }

        info!("  CLICK STRATEGY: fallback → {} targets", targets.len());
    }

    /// Determine if we should submit now (action 7).
    /// Uses IRA reliability + structural score to decide — no hardcoded patterns.
    fn should_submit_now(&self, grid: &Grid2D) -> bool {
        // Don't submit before taking any actions
        if self.actions_since_submit == 0 {
            return false;
        }

        // Don't submit during exploration phase (still learning action effects)
        if self.step_in_game < 3 {
            return false;
        }

        let structural = structural_improvement_score(grid);

        // ── IRA-informed submit: high prediction accuracy means we understand the game ──
        // If global IRA is high AND structural score is good, we likely have the answer
        if self.global_ira > 0.7 && structural > 0.4 && self.actions_since_submit >= 3 {
            info!("  SUBMIT CHECK: high IRA ({:.3}) + structural ({:.3})",
                  self.global_ira, structural);
            return true;
        }

        // ── Structural peak: if score was improving and plateaued, submit ──
        if structural >= self.best_structural_score && structural > 0.3
            && self.actions_since_submit >= 4
        {
            // Check if recent actions aren't improving anymore (plateau)
            let recent_no_improvement = self.recent_action_effects.iter()
                .rev()
                .take(3)
                .filter(|(_, had_effect)| !had_effect)
                .count();
            if recent_no_improvement >= 2 {
                info!("  SUBMIT CHECK: structural plateau at {:.3}", structural);
                return true;
            }
        }

        // ── Periodic submit for feedback: every N actions ──
        // Provides IRA feedback even when we're uncertain
        let submit_interval = if self.click_only_game { 8 } else { 12 };
        if self.actions_since_submit >= submit_interval {
            info!("  SUBMIT CHECK: periodic after {} actions (structural={:.3})",
                  self.actions_since_submit, structural);
            return true;
        }

        // ── Click-only: no more targets means we may be done ──
        if self.click_only_game && self.click_targets.is_empty() && self.clicks_since_submit >= 2 {
            info!("  SUBMIT CHECK: no more click targets");
            return true;
        }

        false
    }
}

/// Infer click mechanic from observed click effects.
fn infer_click_mechanic(observations: &[ClickObservation]) -> Option<ClickMechanic> {
    if observations.len() < 2 {
        return None;
    }

    // Collect all (old_value, new_value) transitions across all clicks
    let mut transitions: HashMap<(u8, u8), u32> = HashMap::new();
    for obs in observations {
        for &(_r, _c, old, new) in &obs.changes {
            *transitions.entry((old, new)).or_insert(0) += 1;
        }
    }

    if transitions.is_empty() {
        return None;
    }

    // Check for toggle: if we see A→B and B→A with similar frequency
    let mut toggle_pairs: Vec<(u8, u8, u32)> = Vec::new();
    for (&(a, b), &count_ab) in &transitions {
        if let Some(&count_ba) = transitions.get(&(b, a)) {
            if a < b { // avoid duplicates
                toggle_pairs.push((a, b, count_ab + count_ba));
            }
        }
    }

    if let Some(&(a, b, _)) = toggle_pairs.iter().max_by_key(|t| t.2) {
        return Some(ClickMechanic::Toggle { color_a: a, color_b: b });
    }

    // Check for paint: all transitions go to the same target color
    let target_colors: std::collections::HashSet<u8> = transitions.keys().map(|&(_, to)| to).collect();
    if target_colors.len() == 1 {
        let target = *target_colors.iter().next().unwrap();
        return Some(ClickMechanic::Paint(target));
    }

    // Check for cycle: A→B, B→C, C→A
    let from_colors: Vec<u8> = transitions.keys().map(|&(from, _)| from).collect();
    if !from_colors.is_empty() {
        let mut cycle = vec![from_colors[0]];
        for _ in 0..10 {
            let last = *cycle.last().unwrap();
            if let Some(&(_, to)) = transitions.keys().find(|&&(from, _)| from == last) {
                if to == cycle[0] && cycle.len() >= 2 {
                    return Some(ClickMechanic::Cycle(cycle));
                }
                if cycle.contains(&to) { break; }
                cycle.push(to);
            } else {
                break;
            }
        }
    }

    None
}

/// Compute bounding box of non-background content.
fn content_bbox(grid: &Grid2D, bg: u8) -> (usize, usize, usize, usize) {
    let mut min_r = grid.height;
    let mut max_r = 0;
    let mut min_c = grid.width;
    let mut max_c = 0;
    for r in 0..grid.height {
        for c in 0..grid.width {
            if grid.cells[r][c] != bg {
                min_r = min_r.min(r);
                max_r = max_r.max(r);
                min_c = min_c.min(c);
                max_c = max_c.max(c);
            }
        }
    }
    if min_r > max_r {
        (0, grid.height.saturating_sub(1), 0, grid.width.saturating_sub(1))
    } else {
        (min_r, max_r, min_c, max_c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_model_observe() {
        let before = Grid2D::new(vec![vec![0, 1], vec![2, 3]]);
        let after = Grid2D::new(vec![vec![0, 1], vec![2, 5]]);
        let mut model = ActionModel::new(1);
        model.observe(&before, &after);
        assert!(model.has_effect);
        assert_eq!(model.observed_patches.len(), 1);
        assert_eq!(model.observed_patches[0], vec![(1, 1, 3, 5)]);
    }

    #[test]
    fn test_action_model_predict() {
        let before = Grid2D::new(vec![vec![0, 1], vec![2, 3]]);
        let after = Grid2D::new(vec![vec![0, 1], vec![2, 5]]);
        let mut model = ActionModel::new(1);
        model.observe(&before, &after);

        let current = Grid2D::new(vec![vec![0, 1], vec![2, 3]]);
        let predicted = model.predict(&current).unwrap();
        assert_eq!(predicted.cells[1][1], 5);
    }

    #[test]
    fn test_arc_game_state_worldstate() {
        let grid = Grid2D::new(vec![vec![0, 1, 2], vec![3, 4, 5]]);
        let state = ArcGameState::new(grid, 0, HashMap::new(), vec![1, 2, 6]);
        let props = state.analyze();
        assert!(!props.is_empty());
        let actions = state.available_actions();
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].name, "game_action_1");
    }
}
