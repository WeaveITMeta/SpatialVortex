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
use eustress_arc_types::{ArcStep, PolicyDecision};
use eustress_vortex_core::{
    CausalGraph, DSLOp, Delta, Domain, EpisodeRecord,
    GoalPredicate, Property, PropertyValue, Score, SymbolResolver, WorldState,
    causal_graph::CausalNode,
    hypothesis_tree::SimulationBudget,
};
use eustress_vortex_grid2d::{Grid2D, GridAnalyzer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    /// Returns None if we don't have enough observations.
    pub fn predict(&self, current: &Grid2D) -> Option<Grid2D> {
        if self.observed_patches.is_empty() {
            return None;
        }
        // Use the most recent patch as prediction
        let patch = self.observed_patches.last()?;
        let mut predicted = current.clone();
        for &(r, c, _old, new) in patch {
            if r < predicted.height && c < predicted.width {
                predicted.cells[r][c] = new;
            }
        }
        Some(predicted)
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
        let predicted_grid = self
            .action_models
            .get(&action_id)
            .and_then(|m| m.predict(&self.grid))
            .unwrap_or_else(|| self.grid.clone());

        ArcGameState {
            grid: predicted_grid,
            levels_completed: self.levels_completed,
            action_models: self.action_models.clone(),
            available_action_ids: self.available_action_ids.clone(),
        }
    }

    fn score_against(&self, goal: &Self) -> Score {
        // For interactive games: score = levels completed improvement + structural improvement
        let level_score = if self.levels_completed > goal.levels_completed {
            1.0 // We advanced — perfect
        } else {
            // Structural improvement score from GridAnalyzer
            structural_improvement_score(&self.grid)
        };
        Score {
            exact_match: level_score >= 1.0,
            accuracy: level_score,
            details: serde_json::json!({
                "levels": self.levels_completed,
                "structural_score": level_score,
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
        }
    }

    /// Reset state for a new game episode.
    /// Preserves cross-game knowledge: CausalGraph, decomposer archetypes.
    pub fn new_episode(&mut self, game_id: &str) {
        self.prev_grid = None;
        self.last_action_id = None;
        self.pending_prediction = None;
        self.step_in_game = 0;
        self.current_game = game_id.to_string();
        self.prev_levels = 0;
        self.tried_actions.clear();
        self.click_targets.clear();
        self.click_count = 0;
        // Reset scene mirror for new game (clear entities)
        self.scene_mirror = ArcSceneMirror::new(game_id, ArcMirrorConfig::default());
        // Keep causal_graph, action_models, decomposer — they learn across episodes
    }

    // ─── Cross-game persistence ─────────────────────────────────────────

    /// Export cross-game knowledge for persistence.
    pub fn export_knowledge(&self) -> String {
        let snapshot = KnowledgeSnapshot {
            causal_graph: self.causal_graph.clone(),
            decomposer: self.decomposer.clone(),
            global_ira: self.global_ira,
            games_played: self.decomposer.action_history.len() as u32,
        };
        serde_json::to_string_pretty(&snapshot).unwrap_or_default()
    }

    /// Import cross-game knowledge from a previous session.
    pub fn import_knowledge(&mut self, json: &str) -> Result<(), String> {
        let snapshot: KnowledgeSnapshot = serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize knowledge: {}", e))?;

        self.causal_graph = snapshot.causal_graph;
        self.decomposer = snapshot.decomposer;
        self.global_ira = snapshot.global_ira;

        info!(
            "Imported knowledge: {} nodes, {} edges, {} archetypes, global_ira={:.3}",
            self.causal_graph.node_count(),
            self.causal_graph.edge_count(),
            self.decomposer.archetype_summary().len(),
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
        info!(
            "OBSERVE: {}x{} grid, {} colors, {} props, step {}",
            grid.height, grid.width, grid.distinct_colors(), props.len(), self.step_in_game
        );

        // ── SCENE MIRROR: update live entity state ──────────────────────
        let scene_deltas = self.scene_mirror.apply_frame(&grid, self.step_in_game, levels);
        self.scene_mirror.properties.global_ira = self.global_ira;
        self.scene_mirror.properties.total_actions = self.step_in_game;

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
            info!(
                "  action_{} produced {} cell changes (effect={})",
                action_id, changed, model.has_effect
            );

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
            for (op, coverage) in &decomp.ops {
                if op.is_structural() && *coverage > 0.5 {
                    let rule_name = format!(
                        "symbolic_{}_a{}_{}_s{}",
                        op.category(), action_id, game_id, self.step_in_game
                    );
                    let conditions: Vec<String> = props.iter()
                        .filter(|p| p.value.as_bool())
                        .map(|p| p.name.clone())
                        .take(5)
                        .collect();

                    self.causal_graph.add_node(CausalNode::Rule {
                        name: rule_name.clone(),
                        conditions: conditions.clone(),
                        program: vec![ArcGameState::action_to_dsl(action_id)],
                        confidence: *coverage,
                        evidence_count: 1,
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

                // Run SymbolResolver to generalize Rules into Laws
                let mut resolver = SymbolResolver::new();
                let new_laws = resolver.scan_and_unify(&mut self.causal_graph);
                if new_laws > 0 {
                    info!("  SymbolResolver unified {} new Laws", new_laws);
                }
            }

            // Detect level advance
            if level_advanced {
                info!("  LEVEL ADVANCE: {} → {}", self.prev_levels, levels);
            }
        }

        self.prev_levels = levels;

        // ── 3. DECIDE: exploration vs exploitation ──────────────────────
        let decision = if self.should_explore(&available) {
            self.explore_action(&available, &grid)
        } else {
            self.exploit_action(&available, &grid, &props, levels)
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

        // Save state for next step
        self.prev_grid = Some(grid);
        self.last_action_id = Some(action_id);
        self.step_in_game += 1;

        decision
    }

    /// Should we explore (try untried actions) or exploit (use learned model)?
    fn should_explore(&self, available: &[u32]) -> bool {
        let untried: Vec<u32> = available
            .iter()
            .filter(|a| !self.tried_actions.contains(a))
            .copied()
            .collect();

        // Explore if we have untried actions and haven't spent too many steps exploring
        if !untried.is_empty() && self.step_in_game < (available.len() as u32 * 2) {
            return true;
        }

        // Also explore if global IRA is too low (we don't understand the game yet)
        if self.global_ira < 0.3 && self.step_in_game < 20 {
            return true;
        }

        false
    }

    /// Exploration phase: try each action to learn what it does.
    fn explore_action(&mut self, available: &[u32], grid: &Grid2D) -> PolicyDecision {
        // Priority: untried actions first
        let untried: Vec<u32> = available
            .iter()
            .filter(|a| !self.tried_actions.contains(a))
            .copied()
            .collect();

        let action_id = if let Some(&a) = untried.first() {
            a
        } else {
            // Re-explore: pick the action with lowest observation count
            *available
                .iter()
                .min_by_key(|&&a| {
                    self.action_models
                        .get(&a)
                        .map_or(0, |m| m.observed_patches.len())
                })
                .unwrap_or(&available[0])
        };

        self.tried_actions.push(action_id);
        info!(
            "EXPLORE: trying action_{} ({}/{} tried)",
            action_id,
            self.tried_actions.len(),
            available.len()
        );

        self.make_decision(action_id, 0.3, grid, format!(
            "explore: characterize action_{} ({} untried remaining)",
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

        // ── EVALUATE: extract best action from solve result ─────────────
        if let Some(first_op) = result.program.first() {
            if let Some(action_id) = ArcGameState::dsl_to_action(first_op) {
                if available.contains(&action_id) {
                    let confidence = result.score.accuracy as f32;
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

    /// Pick the best known effective action (one that actually changes the grid).
    fn best_effective_action(&self, available: &[u32], grid: &Grid2D) -> u32 {
        let props = grid.analyze();
        let has_floating = props.iter().any(|p| p.name == "has_unsupported_objects" && p.value.as_bool());

        // Score each available action based on what we've learned
        let mut scores: Vec<(u32, f32)> = available
            .iter()
            .map(|&a| {
                let model_score = self
                    .action_models
                    .get(&a)
                    .map(|m| {
                        let mut s = 0.0f32;
                        if m.has_effect { s += 2.0; }
                        if m.is_deterministic { s += 1.0; }
                        s += m.ira_score;
                        // Property-guided boost
                        if has_floating && (a == 1 || a == 2) { s += 1.5; } // directional actions for gravity
                        s
                    })
                    .unwrap_or(0.5); // Unknown actions get medium priority
                (a, model_score)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.first().map(|&(a, _)| a).unwrap_or(1)
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

    /// Compute click target using Grid2D structural analysis.
    /// Uses color centroids, content bounding box, and systematic grid scan.
    fn compute_click_target(&mut self, grid: &Grid2D) -> (u32, u32) {
        if self.click_targets.is_empty() {
            self.build_click_targets(grid);
        }

        if self.click_targets.is_empty() {
            return (grid.width as u32 / 2, grid.height as u32 / 2);
        }

        let idx = self.click_count % self.click_targets.len();
        self.click_count += 1;
        self.click_targets[idx]
    }

    /// Build click target list from grid analysis.
    fn build_click_targets(&mut self, grid: &Grid2D) {
        let bg = grid.background_color();
        let mut targets = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Phase 1: Color region centroids
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
            if seen.insert((cx, cy)) {
                targets.push((cx, cy));
            }
        }

        // Phase 2: Content bounding box center + corners
        let (min_r, max_r, min_c, max_c) = content_bbox(grid, bg);
        let center_x = (min_c + max_c) / 2;
        let center_y = (min_r + max_r) / 2;
        for &(x, y) in &[
            (center_x, center_y),
            (min_c, min_r), (max_c, min_r), (min_c, max_r), (max_c, max_r),
            (center_x, min_r), (center_x, max_r), (min_c, center_y), (max_c, center_y),
        ] {
            if seen.insert((x as u32, y as u32)) {
                targets.push((x as u32, y as u32));
            }
        }

        // Phase 3: Systematic 8×8 grid scan
        let block_h = (grid.height / 8).max(1);
        let block_w = (grid.width / 8).max(1);
        for by in 0..8 {
            for bx in 0..8 {
                let x = (bx * block_w + block_w / 2) as u32;
                let y = (by * block_h + block_h / 2) as u32;
                if seen.insert((x, y)) {
                    targets.push((x, y));
                }
            }
        }

        // Phase 4: Individual non-background cells (up to 64)
        for r in 0..grid.height {
            for c in 0..grid.width {
                if grid.cells[r][c] != bg && targets.len() < 128 {
                    let pt = (c as u32, r as u32);
                    if seen.insert(pt) {
                        targets.push(pt);
                    }
                }
            }
        }

        self.click_targets = targets;
    }
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
