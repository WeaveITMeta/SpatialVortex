//! Goal inference engine for ARC-AGI-3.
//!
//! Infers what the agent should achieve based on object perception, behavioral
//! evidence, and level completion analysis. Dual inference strategy:
//! 1. Object-based: Agent + GoalMarker → ReachPosition; Collectibles → CollectAll
//! 2. Grid-diff fallback: correlate changes with actions, track disappearances
//!
//! Goal proximity replaces the old structural_improvement_score, feeding into
//! submit timing, MCTS evaluation, and flux matrix sacred gates.

use eustress_vortex_grid2d::Grid2D;
use crate::object_tracker::{ObjectFrame, ObjectEvent, ObjectRole, StableObjectId};

// ─── Goal Types ──────────────────────────────────────────────────────────────

/// What the agent believes the goal of the current level is.
#[derive(Clone, Debug)]
pub enum GoalType {
    /// Navigate agent to a specific position.
    ReachPosition { target_row: usize, target_col: usize },
    /// Collect all objects of a given role.
    CollectAll {
        target_role: ObjectRole,
        total: usize,
        remaining: usize,
    },
    /// Transform the grid to match a target pattern.
    TransformGrid {
        target_pattern: Option<Grid2D>,
        similarity: f64,
    },
    /// Goal inferred from score/progress correlation.
    ScoreCorrelated { description: String },
    /// Unknown — exploring to discover.
    Unknown,
}

/// Evidence supporting a goal hypothesis.
#[derive(Clone, Debug)]
pub enum GoalEvidence {
    /// An object with this role exists.
    ObjectPresence { role: ObjectRole, count: usize },
    /// An object disappeared when agent reached its position.
    ObjectDisappearedOnContact { object_id: StableObjectId, at_step: u32 },
    /// Level advanced after this condition was met.
    LevelAdvancedAfter { condition: String },
    /// Cross-level transfer: same pattern worked before.
    PriorLevelEvidence { description: String, confidence: f32 },
    /// Agent moved toward position and progress was made.
    ProximityCorrelation { target: (usize, usize), correlation: f32 },
}

/// A ranked hypothesis about the current goal.
#[derive(Clone, Debug)]
pub struct GoalHypothesis {
    pub goal_type: GoalType,
    pub confidence: f32,
    pub proximity_score: f32,
    pub evidence: Vec<GoalEvidence>,
}

// ─── Goal Inference Engine ───────────────────────────────────────────────────

/// Infers goals from object perception and environmental feedback.
pub struct GoalInferenceEngine {
    /// Ranked goal hypotheses, highest confidence first.
    pub active_goals: Vec<GoalHypothesis>,
    /// Hysteresis: new hypothesis must exceed current by this margin to take over.
    hysteresis: f32,
    /// Count of collectibles observed at the start of the level.
    initial_collectible_count: Option<usize>,
    /// Disappeared objects this level (potential collectibles).
    disappeared_objects: Vec<StableObjectId>,
    /// Step counter for evidence timing.
    step: u32,
}

impl GoalInferenceEngine {
    pub fn new() -> Self {
        Self {
            active_goals: Vec::new(),
            hysteresis: 0.15,
            initial_collectible_count: None,
            disappeared_objects: Vec::new(),
            step: 0,
        }
    }

    /// Reset for a new level (but may keep cross-level Laws).
    pub fn reset_for_level(&mut self) {
        self.active_goals.clear();
        self.initial_collectible_count = None;
        self.disappeared_objects.clear();
        self.step = 0;
    }

    /// Full reset (new game).
    pub fn reset(&mut self) {
        self.reset_for_level();
    }

    /// Get the top goal hypothesis, if any.
    pub fn top_goal(&self) -> Option<&GoalHypothesis> {
        self.active_goals.first()
    }

    /// Update goal hypotheses based on current object perception.
    pub fn update(
        &mut self,
        frame: &ObjectFrame,
        events: &[ObjectEvent],
        agent_pos: Option<(f32, f32)>,
    ) {
        self.step += 1;
        let mut hypotheses: Vec<GoalHypothesis> = Vec::new();

        // Track disappeared objects.
        for event in events {
            if let ObjectEvent::Disappeared { id } = event {
                self.disappeared_objects.push(*id);
            }
        }

        // ── Object-based inference ───────────────────────────────────────

        // Note: Object role counts come from ObjectTracker classifications,
        // not from ObjectFrame directly. Goal candidates are set by the tracker.

        // If goal candidates exist: ReachPosition hypothesis.
        if let Some(agent_centroid) = agent_pos {
            for &goal_id in &frame.goal_candidates {
                if let Some(goal_obj) = frame.objects.get(&goal_id) {
                    let (gr, gc) = (goal_obj.centroid.0, goal_obj.centroid.1);
                    let dist = (agent_centroid.0 - gr).abs() + (agent_centroid.1 - gc).abs();
                    let max_dist = 128.0_f32; // max manhattan on 64x64
                    let proximity = 1.0 - (dist / max_dist).min(1.0);

                    hypotheses.push(GoalHypothesis {
                        goal_type: GoalType::ReachPosition {
                            target_row: gr as usize,
                            target_col: gc as usize,
                        },
                        confidence: 0.7,
                        proximity_score: proximity,
                        evidence: vec![
                            GoalEvidence::ObjectPresence {
                                role: ObjectRole::Goal,
                                count: frame.goal_candidates.len(),
                            },
                        ],
                    });
                }
            }
        }

        // If objects have disappeared: CollectAll hypothesis.
        if !self.disappeared_objects.is_empty() {
            let total = self.initial_collectible_count.unwrap_or(
                self.disappeared_objects.len() + frame.objects.len()
            );
            let remaining = frame.objects.len().saturating_sub(1); // minus agent
            let collected = self.disappeared_objects.len();

            if collected > 0 {
                let progress = collected as f32 / total.max(1) as f32;
                hypotheses.push(GoalHypothesis {
                    goal_type: GoalType::CollectAll {
                        target_role: ObjectRole::Collectible,
                        total,
                        remaining,
                    },
                    confidence: 0.5 + progress * 0.3,
                    proximity_score: progress,
                    evidence: vec![
                        GoalEvidence::ObjectDisappearedOnContact {
                            object_id: *self.disappeared_objects.last().unwrap_or(&0),
                            at_step: self.step,
                        },
                    ],
                });
            }
        }

        // Fallback: Unknown goal.
        if hypotheses.is_empty() {
            hypotheses.push(GoalHypothesis {
                goal_type: GoalType::Unknown,
                confidence: 0.1,
                proximity_score: 0.0,
                evidence: vec![],
            });
        }

        // Sort by confidence descending.
        hypotheses.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        // Apply hysteresis: only replace active goals if new top exceeds old by margin.
        if let Some(current_top) = self.active_goals.first() {
            if let Some(new_top) = hypotheses.first() {
                if new_top.confidence < current_top.confidence + self.hysteresis {
                    // Keep current goals but update proximity scores.
                    for goal in &mut self.active_goals {
                        if let Some(matching) = hypotheses.iter().find(|h| {
                            std::mem::discriminant(&h.goal_type) == std::mem::discriminant(&goal.goal_type)
                        }) {
                            goal.proximity_score = matching.proximity_score;
                            goal.evidence = matching.evidence.clone();
                        }
                    }
                    return;
                }
            }
        }

        self.active_goals = hypotheses;

        // Cache initial collectible count on first update.
        if self.initial_collectible_count.is_none() && self.step == 1 {
            self.initial_collectible_count = Some(frame.objects.len().saturating_sub(1));
        }
    }

    /// Goal-proximity score: how close we are to achieving the top goal.
    /// Returns 0.0-1.0. Replaces structural_improvement_score.
    pub fn proximity_score(
        &self,
        _grid: &Grid2D,
        agent_pos: Option<(f32, f32)>,
    ) -> f64 {
        let Some(goal) = self.top_goal() else {
            return 0.0;
        };

        match &goal.goal_type {
            GoalType::ReachPosition { target_row, target_col } => {
                if let Some((ar, ac)) = agent_pos {
                    let dist = (ar - *target_row as f32).abs() + (ac - *target_col as f32).abs();
                    let max_dist = 128.0;
                    (1.0 - (dist as f64 / max_dist as f64)).max(0.0)
                } else {
                    0.0
                }
            }
            GoalType::CollectAll { total, remaining, .. } => {
                if *total > 0 {
                    (*total - *remaining) as f64 / *total as f64
                } else {
                    1.0
                }
            }
            GoalType::TransformGrid { similarity, .. } => *similarity,
            GoalType::ScoreCorrelated { .. } => goal.proximity_score as f64,
            GoalType::Unknown => 0.0,
        }
    }

    /// Should we submit now? Returns true if proximity is very high.
    pub fn should_submit(&self) -> bool {
        self.top_goal()
            .map(|g| g.proximity_score > 0.95)
            .unwrap_or(false)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use eustress_vortex_grid2d::objects::{GridObject, BBox};

    fn make_object(id: u32, color: u8, row: usize, col: usize) -> GridObject {
        GridObject {
            local_id: id,
            color,
            cells: vec![(row, col)],
            bbox: BBox {
                min_row: row,
                min_col: col,
                max_row: row,
                max_col: col,
            },
            centroid: (row as f32, col as f32),
            area: 1,
        }
    }

    #[test]
    fn test_reach_position_goal() {
        let mut engine = GoalInferenceEngine::new();

        let mut frame = ObjectFrame::default();
        frame.agent_id = Some(1);
        frame.goal_candidates = vec![2];
        frame.objects.insert(1, make_object(1, 5, 0, 0)); // agent at (0,0)
        frame.objects.insert(2, make_object(2, 3, 10, 10)); // goal at (10,10)

        engine.update(&frame, &[], Some((0.0, 0.0)));

        let goal = engine.top_goal().unwrap();
        assert!(matches!(goal.goal_type, GoalType::ReachPosition { .. }));
        assert!(goal.confidence >= 0.7);
        assert!(goal.proximity_score < 1.0);
    }

    #[test]
    fn test_collect_all_goal() {
        let mut engine = GoalInferenceEngine::new();

        let mut frame = ObjectFrame::default();
        frame.objects.insert(1, make_object(1, 5, 0, 0));
        frame.objects.insert(2, make_object(2, 3, 5, 5));

        // First update: no disappearances, unknown goal.
        engine.update(&frame, &[], Some((0.0, 0.0)));

        // Object disappears.
        let events = vec![ObjectEvent::Disappeared { id: 2 }];
        frame.objects.remove(&2);
        engine.update(&frame, &events, Some((5.0, 5.0)));

        let goal = engine.top_goal().unwrap();
        assert!(matches!(goal.goal_type, GoalType::CollectAll { .. }));
    }

    #[test]
    fn test_unknown_goal_fallback() {
        let mut engine = GoalInferenceEngine::new();
        let frame = ObjectFrame::default();
        engine.update(&frame, &[], None);

        let goal = engine.top_goal().unwrap();
        assert!(matches!(goal.goal_type, GoalType::Unknown));
    }

    #[test]
    fn test_proximity_score() {
        let mut engine = GoalInferenceEngine::new();

        let mut frame = ObjectFrame::default();
        frame.goal_candidates = vec![2];
        frame.objects.insert(1, make_object(1, 5, 0, 0));
        frame.objects.insert(2, make_object(2, 3, 10, 10));

        engine.update(&frame, &[], Some((0.0, 0.0)));

        let grid = Grid2D::empty(64, 64);
        let score_far = engine.proximity_score(&grid, Some((0.0, 0.0)));
        let score_near = engine.proximity_score(&grid, Some((9.0, 9.0)));
        assert!(score_near > score_far);
    }

    #[test]
    fn test_hysteresis() {
        let mut engine = GoalInferenceEngine::new();

        // Set up strong ReachPosition goal
        let mut frame = ObjectFrame::default();
        frame.goal_candidates = vec![2];
        frame.objects.insert(1, make_object(1, 5, 0, 0));
        frame.objects.insert(2, make_object(2, 3, 10, 10));
        engine.update(&frame, &[], Some((0.0, 0.0)));

        let first_goal_type = std::mem::discriminant(&engine.top_goal().unwrap().goal_type);

        // Weak update: should not change goal (hysteresis)
        frame.goal_candidates.clear();
        engine.update(&frame, &[], Some((0.0, 0.0)));

        let second_goal_type = std::mem::discriminant(&engine.top_goal().unwrap().goal_type);
        assert_eq!(first_goal_type, second_goal_type, "Hysteresis should prevent goal flipping");
    }
}
