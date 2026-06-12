//! Goal inference engine for ARC-AGI-3.
//!
//! Infers what the agent should achieve based on object perception, behavioral
//! evidence, and level completion analysis. Dual inference strategy:
//! 1. Object-based: Agent + GoalMarker → ReachPosition; Collectibles → CollectAll
//! 2. Grid-diff fallback: correlate changes with actions, track disappearances
//!
//! Goal proximity replaces the old structural_improvement_score, feeding into
//! submit timing, MCTS evaluation, and flux matrix sacred gates.

use std::collections::{HashMap, HashSet};
use eustress_vortex_grid2d::Grid2D;
use eustress_vortex_grid2d::objects::GridObject;
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

// ─── Goal-Cell Inference (movement games) ────────────────────────────────────

/// Which heuristic nominated a suggested goal cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalCellHeuristic {
    /// Rarest non-background color on the grid, covering at most 4 cells,
    /// none of them belonging to the avatar.
    RareColorSingleton,
    /// A non-avatar object sharing the avatar's color (a "twin marker").
    AvatarColorTwin,
    /// Smallest object touching no other object, far from the avatar.
    SmallIsolatedObject,
}

/// A suggested goal cell with the confidence of the nominating heuristic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GoalCellSuggestion {
    /// (row, col); never a border cell, never a background-colored cell.
    pub cell: (usize, usize),
    /// 0.0-1.0, comparable to GoalHypothesis::confidence.
    pub confidence: f32,
    pub heuristic: GoalCellHeuristic,
}

/// Infer the most goal-like cell for movement games, trying ranked heuristics:
/// 1. Unique rare-color cell: the rarest color on the grid, if it covers at
///    most 4 cells and none belong to the avatar (confidence 0.9 down to 0.6).
/// 2. A non-avatar object whose color matches the avatar's (confidence 0.55).
/// 3. The smallest isolated object far from the avatar (confidence 0.4).
/// Border and background (ambient) cells are never returned. Deterministic:
/// all ties break toward smaller coordinates / lower object ids.
pub fn infer_goal_cell(grid: &Grid2D, frame: &ObjectFrame) -> Option<GoalCellSuggestion> {
    if grid.height <= 2 || grid.width <= 2 {
        return None; // no interior cells exist
    }

    let avatar = frame.agent_id.and_then(|id| frame.objects.get(&id));
    let avatar_cells: HashSet<(usize, usize)> = avatar
        .map(|a| a.cells.iter().copied().collect())
        .unwrap_or_default();

    rare_color_cell(grid, frame.background_color, &avatar_cells)
        .or_else(|| avatar_color_twin(grid, frame, avatar))
        .or_else(|| small_isolated_object(grid, frame, avatar))
}

/// Heuristic 1: the rarest interior non-background color, if it covers at
/// most 4 cells (excluding the avatar's own cells).
fn rare_color_cell(
    grid: &Grid2D,
    background: u8,
    avatar_cells: &HashSet<(usize, usize)>,
) -> Option<GoalCellSuggestion> {
    let mut cells_by_color: HashMap<u8, Vec<(usize, usize)>> = HashMap::new();
    for r in 1..grid.height - 1 {
        for c in 1..grid.width - 1 {
            let color = grid.cells[r][c];
            if color == background || avatar_cells.contains(&(r, c)) {
                continue;
            }
            cells_by_color.entry(color).or_default().push((r, c));
        }
    }
    // Keyed on (count, color value) so equally rare colors resolve the same
    // way regardless of HashMap iteration order.
    let (_, cells) = cells_by_color
        .iter()
        .min_by_key(|(color, cells)| (cells.len(), **color))?;
    if cells.len() > 4 {
        return None;
    }
    let cell = central_cell(cells)?;
    Some(GoalCellSuggestion {
        cell,
        confidence: 0.9 - 0.1 * (cells.len() as f32 - 1.0),
        heuristic: GoalCellHeuristic::RareColorSingleton,
    })
}

/// Heuristic 2: a non-avatar object with the avatar's color. The frame carries
/// no velocity, but the tracker resolves the single action-correlated mover to
/// agent_id, so every other same-colored object is static by construction.
fn avatar_color_twin(
    grid: &Grid2D,
    frame: &ObjectFrame,
    avatar: Option<&GridObject>,
) -> Option<GoalCellSuggestion> {
    let avatar = avatar?;
    let agent_id = frame.agent_id?;
    let mut twins: Vec<(StableObjectId, &GridObject)> = frame
        .objects
        .iter()
        .filter(|(id, obj)| **id != agent_id && obj.color == avatar.color)
        .map(|(id, obj)| (*id, obj))
        .collect();
    twins.sort_by_key(|&(id, obj)| (obj.area, id));
    twins.into_iter().find_map(|(_, obj)| {
        object_cell(grid, frame.background_color, obj).map(|cell| GoalCellSuggestion {
            cell,
            confidence: 0.55,
            heuristic: GoalCellHeuristic::AvatarColorTwin,
        })
    })
}

/// Heuristic 3: the smallest object that touches no other object
/// (8-connected) and whose centroid is at least (height + width) / 4
/// Manhattan distance from the avatar's.
fn small_isolated_object(
    grid: &Grid2D,
    frame: &ObjectFrame,
    avatar: Option<&GridObject>,
) -> Option<GoalCellSuggestion> {
    let avatar = avatar?;
    let agent_id = frame.agent_id?;
    let far_threshold = (grid.height + grid.width) as f32 / 4.0;

    let mut owner: HashMap<(usize, usize), StableObjectId> = HashMap::new();
    for (id, obj) in &frame.objects {
        for &cell in &obj.cells {
            owner.insert(cell, *id);
        }
    }

    let mut candidates: Vec<(usize, f32, StableObjectId, &GridObject)> = frame
        .objects
        .iter()
        .filter(|(id, _)| **id != agent_id)
        .filter(|(id, obj)| is_isolated(obj, **id, &owner, grid))
        .filter_map(|(id, obj)| {
            let dist = (obj.centroid.0 - avatar.centroid.0).abs()
                + (obj.centroid.1 - avatar.centroid.1).abs();
            (dist >= far_threshold).then_some((obj.area, dist, *id, obj))
        })
        .collect();
    // Smallest area first, then farthest, then lowest id — total order so the
    // pick is independent of HashMap iteration order.
    candidates.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.2.cmp(&b.2))
    });
    candidates.into_iter().find_map(|(_, _, _, obj)| {
        object_cell(grid, frame.background_color, obj).map(|cell| GoalCellSuggestion {
            cell,
            confidence: 0.4,
            heuristic: GoalCellHeuristic::SmallIsolatedObject,
        })
    })
}

/// True when none of the object's cells has an 8-connected neighbor belonging
/// to a different object.
fn is_isolated(
    obj: &GridObject,
    id: StableObjectId,
    owner: &HashMap<(usize, usize), StableObjectId>,
    grid: &Grid2D,
) -> bool {
    obj.cells.iter().all(|&(r, c)| {
        let (r, c) = (r as i32, c as i32);
        (-1..=1).all(|dr| {
            (-1..=1).all(|dc| {
                let (nr, nc) = (r + dr, c + dc);
                if nr < 0 || nc < 0 || nr >= grid.height as i32 || nc >= grid.width as i32 {
                    return true;
                }
                owner
                    .get(&(nr as usize, nc as usize))
                    .map_or(true, |&owner_id| owner_id == id)
            })
        })
    })
}

/// Representative cell of an object: the interior, non-background cell nearest
/// its centroid. None when the object lies entirely on the border (frames,
/// HUD strips) — those are never valid goal cells.
fn object_cell(grid: &Grid2D, background: u8, obj: &GridObject) -> Option<(usize, usize)> {
    let interior: Vec<(usize, usize)> = obj
        .cells
        .iter()
        .copied()
        .filter(|&(r, c)| r > 0 && c > 0 && r + 1 < grid.height && c + 1 < grid.width)
        .filter(|&(r, c)| grid.get(r, c) != Some(background))
        .collect();
    central_cell(&interior)
}

/// Cell nearest the group centroid; ties break toward smaller (row, col).
fn central_cell(cells: &[(usize, usize)]) -> Option<(usize, usize)> {
    let n = cells.len() as f32;
    let cr = cells.iter().map(|&(r, _)| r as f32).sum::<f32>() / n;
    let cc = cells.iter().map(|&(_, c)| c as f32).sum::<f32>() / n;
    cells.iter().copied().min_by(|a, b| {
        let da = (a.0 as f32 - cr).abs() + (a.1 as f32 - cc).abs();
        let db = (b.0 as f32 - cr).abs() + (b.1 as f32 - cc).abs();
        da.partial_cmp(&db)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(b))
    })
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

    fn make_object_with_cells(id: u32, color: u8, cells: Vec<(usize, usize)>) -> GridObject {
        let area = cells.len();
        let centroid = (
            cells.iter().map(|&(r, _)| r as f32).sum::<f32>() / area as f32,
            cells.iter().map(|&(_, c)| c as f32).sum::<f32>() / area as f32,
        );
        GridObject {
            local_id: id,
            color,
            cells: cells.clone(),
            bbox: BBox {
                min_row: cells.iter().map(|c| c.0).min().unwrap(),
                min_col: cells.iter().map(|c| c.1).min().unwrap(),
                max_row: cells.iter().map(|c| c.0).max().unwrap(),
                max_col: cells.iter().map(|c| c.1).max().unwrap(),
            },
            centroid,
            area,
        }
    }

    fn paint(grid: &mut Grid2D, obj: &GridObject) {
        for &(r, c) in &obj.cells {
            grid.set(r, c, obj.color);
        }
    }

    fn frame_with(agent_id: Option<u32>, objects: &[&GridObject]) -> ObjectFrame {
        let mut frame = ObjectFrame::default();
        frame.agent_id = agent_id;
        for obj in objects {
            frame.objects.insert(obj.local_id, (*obj).clone());
        }
        frame
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

    #[test]
    fn test_infer_goal_cell_rare_singleton() {
        let mut grid = Grid2D::empty(8, 8);
        let avatar = make_object(1, 5, 2, 2);
        // 6 cells of color 3: too common to be the rare-color pick.
        let clutter = make_object_with_cells(
            2, 3, vec![(1, 1), (1, 2), (1, 3), (1, 4), (1, 5), (2, 5)],
        );
        let target = make_object(3, 7, 5, 5);
        for obj in [&avatar, &clutter, &target] {
            paint(&mut grid, obj);
        }
        let frame = frame_with(Some(1), &[&avatar, &clutter, &target]);

        let s = infer_goal_cell(&grid, &frame).unwrap();
        assert_eq!(s.cell, (5, 5));
        assert_eq!(s.heuristic, GoalCellHeuristic::RareColorSingleton);
        assert!((s.confidence - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_infer_goal_cell_excludes_border_and_avatar() {
        let mut grid = Grid2D::empty(8, 8);
        // Unique-color decoy on the border: must never be suggested.
        grid.set(0, 3, 7);
        let avatar = make_object(1, 5, 2, 2); // unique color too, but is the avatar
        let target = make_object(2, 3, 4, 4);
        for obj in [&avatar, &target] {
            paint(&mut grid, obj);
        }
        let frame = frame_with(Some(1), &[&avatar, &target]);

        let s = infer_goal_cell(&grid, &frame).unwrap();
        assert_eq!(s.cell, (4, 4));
        assert_eq!(s.heuristic, GoalCellHeuristic::RareColorSingleton);
    }

    #[test]
    fn test_infer_goal_cell_avatar_color_twin() {
        let mut grid = Grid2D::empty(10, 10);
        let avatar = make_object(1, 5, 2, 2);
        // Twin has 5 cells, so the avatar's color is too common for
        // heuristic 1 (count > 4 after excluding the avatar itself).
        let twin = make_object_with_cells(
            2, 5, vec![(6, 6), (6, 7), (7, 6), (7, 7), (7, 8)],
        );
        for obj in [&avatar, &twin] {
            paint(&mut grid, obj);
        }
        let frame = frame_with(Some(1), &[&avatar, &twin]);

        let s = infer_goal_cell(&grid, &frame).unwrap();
        assert_eq!(s.heuristic, GoalCellHeuristic::AvatarColorTwin);
        assert_eq!(s.cell, (7, 7)); // cell nearest the twin's centroid
        assert!((s.confidence - 0.55).abs() < 1e-6);
    }

    #[test]
    fn test_infer_goal_cell_small_isolated_object() {
        let mut grid = Grid2D::empty(12, 12);
        let avatar = make_object(1, 6, 1, 1);
        // Same color for both blobs (10 cells total) defeats heuristic 1;
        // no color-6 twin defeats heuristic 2.
        let near = make_object_with_cells(
            2, 3, vec![(2, 2), (2, 3), (3, 2), (3, 3), (3, 4)],
        );
        let far = make_object_with_cells(
            3, 3, vec![(8, 8), (8, 9), (9, 8), (9, 9), (9, 10)],
        );
        for obj in [&avatar, &near, &far] {
            paint(&mut grid, obj);
        }
        let frame = frame_with(Some(1), &[&avatar, &near, &far]);

        let s = infer_goal_cell(&grid, &frame).unwrap();
        assert_eq!(s.heuristic, GoalCellHeuristic::SmallIsolatedObject);
        assert_eq!(s.cell, (9, 9)); // cell nearest the far blob's centroid
        assert!((s.confidence - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_infer_goal_cell_deterministic() {
        // Two equal-area twins: the (area, id) tie must resolve identically
        // regardless of HashMap insertion order.
        let mut grid = Grid2D::empty(10, 10);
        let avatar = make_object(1, 5, 2, 2);
        let twin_a = make_object_with_cells(
            2, 5, vec![(6, 2), (6, 3), (6, 4), (7, 2), (7, 3)],
        );
        let twin_b = make_object_with_cells(
            3, 5, vec![(2, 6), (2, 7), (3, 6), (3, 7), (3, 8)],
        );
        for obj in [&avatar, &twin_a, &twin_b] {
            paint(&mut grid, obj);
        }
        let frame_fwd = frame_with(Some(1), &[&avatar, &twin_a, &twin_b]);
        let frame_rev = frame_with(Some(1), &[&twin_b, &twin_a, &avatar]);

        let s1 = infer_goal_cell(&grid, &frame_fwd).unwrap();
        let s2 = infer_goal_cell(&grid, &frame_rev).unwrap();
        assert_eq!(s1, s2);
        assert_eq!(s1.heuristic, GoalCellHeuristic::AvatarColorTwin);
        assert_eq!(s1.cell, (6, 3)); // lower-id twin wins the tie
    }

    #[test]
    fn test_infer_goal_cell_none_when_nothing_qualifies() {
        // All-background grid: no candidate cells at all.
        let grid = Grid2D::empty(8, 8);
        let frame = ObjectFrame::default();
        assert!(infer_goal_cell(&grid, &frame).is_none());

        // Grids with no interior can never suggest a cell.
        let tiny = Grid2D::empty(2, 2);
        assert!(infer_goal_cell(&tiny, &frame).is_none());
    }
}
