//! Level completion analysis for ARC-AGI-3.
//!
//! When a level completes (levels_completed increments), this module captures
//! a rich record of what happened: starting vs final grid, object-level diffs,
//! inferred transformation rules. These feed into CausalGraph as Laws for
//! cross-level transfer.

use eustress_vortex_grid2d::Grid2D;
use crate::object_tracker::{ObjectFrame, StableObjectId, ObjectRole};
use crate::goal_inference::GoalHypothesis;

// ─── Object Diff ─────────────────────────────────────────────────────────────

/// Object-level change between start and end of a level.
#[derive(Clone, Debug)]
pub enum ObjectDiff {
    /// Object was present at start but gone by the end.
    Disappeared { id: StableObjectId, role: ObjectRole },
    /// Object appeared during the level (not present at start).
    Appeared { id: StableObjectId, role: ObjectRole },
    /// Object moved from one position to another.
    Moved {
        id: StableObjectId,
        from: (f32, f32),
        to: (f32, f32),
    },
    /// Object changed color.
    ColorChanged {
        id: StableObjectId,
        from: u8,
        to: u8,
    },
}

// ─── Transformation Rule ─────────────────────────────────────────────────────

/// Semantic rule extracted from a completed level.
#[derive(Clone, Debug)]
pub struct TransformationRule {
    pub description: String,
    pub rule_type: TransformationRuleType,
    pub confidence: f32,
}

/// Types of transformation rules inferred from level completion.
#[derive(Clone, Debug)]
pub enum TransformationRuleType {
    /// Agent navigated to the goal marker's position.
    AgentReachedGoal {
        agent_final_pos: (usize, usize),
        goal_pos: (usize, usize),
    },
    /// All objects of a given role were collected (disappeared).
    AllCollected {
        role: ObjectRole,
        count: usize,
    },
    /// Grid was transformed to match a pattern.
    GridTransformed {
        cell_accuracy: f64,
    },
    /// Unknown transformation — captured as raw grid diff.
    Unknown {
        cells_changed: usize,
    },
}

// ─── Level Completion Record ─────────────────────────────────────────────────

/// Rich record of a completed level.
#[derive(Clone, Debug)]
pub struct LevelCompletionRecord {
    /// Which level (0-indexed).
    pub level_index: u32,
    /// Grid at the start of this level.
    pub start_grid: Grid2D,
    /// Grid at the moment of completion.
    pub final_grid: Grid2D,
    /// Sequence of actions that solved this level.
    pub action_sequence: Vec<String>,
    /// Number of steps to solve.
    pub step_count: usize,
    /// Object-level diffs between start and end.
    pub object_diffs: Vec<ObjectDiff>,
    /// Goal hypothesis that was active when level completed.
    pub inferred_goal: Option<GoalHypothesis>,
    /// Extracted transformation rules.
    pub transformation_rules: Vec<TransformationRule>,
    /// Game ID for cross-game reference.
    pub game_id: String,
}

// ─── Level Completion Analyzer ───────────────────────────────────────────────

/// Analyzes level completions to extract transformation rules.
pub struct LevelCompletionAnalyzer {
    /// History of completed levels.
    pub history: Vec<LevelCompletionRecord>,
}

impl LevelCompletionAnalyzer {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Reset for new game.
    pub fn reset(&mut self) {
        self.history.clear();
    }

    /// Analyze a level completion and produce a record.
    pub fn analyze_completion(
        &mut self,
        level_index: u32,
        start_grid: &Grid2D,
        final_grid: &Grid2D,
        action_sequence: Vec<String>,
        start_frame: Option<&ObjectFrame>,
        final_frame: Option<&ObjectFrame>,
        active_goal: Option<&GoalHypothesis>,
        game_id: &str,
    ) -> LevelCompletionRecord {
        let step_count = action_sequence.len();

        // Compute object diffs if frames available.
        let object_diffs = if let (Some(start_f), Some(final_f)) = (start_frame, final_frame) {
            compute_object_diffs(start_f, final_f)
        } else {
            Vec::new()
        };

        // Extract transformation rules.
        let transformation_rules = infer_rules(
            start_grid,
            final_grid,
            &object_diffs,
            start_frame,
            final_frame,
        );

        let record = LevelCompletionRecord {
            level_index,
            start_grid: start_grid.clone(),
            final_grid: final_grid.clone(),
            action_sequence,
            step_count,
            object_diffs,
            inferred_goal: active_goal.cloned(),
            transformation_rules,
            game_id: game_id.to_string(),
        };

        self.history.push(record.clone());
        record
    }

    /// Get the most recent completion record.
    pub fn last_completion(&self) -> Option<&LevelCompletionRecord> {
        self.history.last()
    }

    /// Count levels completed.
    pub fn levels_completed(&self) -> usize {
        self.history.len()
    }

    /// Extract transferable patterns across completed levels.
    /// Returns common rule types seen in 2+ levels.
    pub fn common_patterns(&self) -> Vec<String> {
        use std::collections::HashMap;
        let mut type_counts: HashMap<String, usize> = HashMap::new();

        for record in &self.history {
            for rule in &record.transformation_rules {
                let key = match &rule.rule_type {
                    TransformationRuleType::AgentReachedGoal { .. } => "agent_reached_goal",
                    TransformationRuleType::AllCollected { .. } => "all_collected",
                    TransformationRuleType::GridTransformed { .. } => "grid_transformed",
                    TransformationRuleType::Unknown { .. } => "unknown",
                };
                *type_counts.entry(key.to_string()).or_insert(0) += 1;
            }
        }

        type_counts
            .into_iter()
            .filter(|(_, count)| *count >= 2)
            .map(|(pattern, _)| pattern)
            .collect()
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Compute object-level diffs between start and end frames.
fn compute_object_diffs(start: &ObjectFrame, end: &ObjectFrame) -> Vec<ObjectDiff> {
    let mut diffs = Vec::new();

    // Objects in start but not in end → Disappeared.
    for (&id, _obj) in &start.objects {
        if !end.objects.contains_key(&id) {
            diffs.push(ObjectDiff::Disappeared {
                id,
                role: ObjectRole::Unknown, // role from tracker context
            });
        }
    }

    // Objects in end but not in start → Appeared.
    for (&id, _obj) in &end.objects {
        if !start.objects.contains_key(&id) {
            diffs.push(ObjectDiff::Appeared {
                id,
                role: ObjectRole::Unknown,
            });
        }
    }

    // Objects in both → check movement and color change.
    for (&id, start_obj) in &start.objects {
        if let Some(end_obj) = end.objects.get(&id) {
            let (sr, sc) = start_obj.centroid;
            let (er, ec) = end_obj.centroid;
            if (sr - er).abs() > 0.3 || (sc - ec).abs() > 0.3 {
                diffs.push(ObjectDiff::Moved {
                    id,
                    from: (sr, sc),
                    to: (er, ec),
                });
            }
            if start_obj.color != end_obj.color {
                diffs.push(ObjectDiff::ColorChanged {
                    id,
                    from: start_obj.color,
                    to: end_obj.color,
                });
            }
        }
    }

    diffs
}

/// Infer transformation rules from grid and object diffs.
fn infer_rules(
    start_grid: &Grid2D,
    final_grid: &Grid2D,
    object_diffs: &[ObjectDiff],
    _start_frame: Option<&ObjectFrame>,
    final_frame: Option<&ObjectFrame>,
) -> Vec<TransformationRule> {
    let mut rules = Vec::new();

    // Check if agent reached a specific position (goal marker location).
    if let Some(frame) = final_frame {
        if let Some(agent_id) = frame.agent_id {
            if let Some(agent_obj) = frame.objects.get(&agent_id) {
                let agent_pos = (
                    agent_obj.centroid.0 as usize,
                    agent_obj.centroid.1 as usize,
                );

                // Check if agent ended up where a goal candidate was at start.
                // (This is a heuristic — actual goal positions come from tracker.)
                for diff in object_diffs {
                    if let ObjectDiff::Disappeared { id, .. } = diff {
                        // If a disappeared object was near the agent's final position,
                        // the goal might have been "reach that position".
                        // We'd need start frame positions here, but this is a best-effort.
                        rules.push(TransformationRule {
                            description: format!("Agent reached disappeared object {} at {:?}", id, agent_pos),
                            rule_type: TransformationRuleType::AgentReachedGoal {
                                agent_final_pos: agent_pos,
                                goal_pos: agent_pos, // approximation
                            },
                            confidence: 0.5,
                        });
                    }
                }
            }
        }
    }

    // Check if all collectibles disappeared.
    let disappeared_count = object_diffs
        .iter()
        .filter(|d| matches!(d, ObjectDiff::Disappeared { .. }))
        .count();
    if disappeared_count > 0 {
        rules.push(TransformationRule {
            description: format!("{} objects collected/disappeared", disappeared_count),
            rule_type: TransformationRuleType::AllCollected {
                role: ObjectRole::Collectible,
                count: disappeared_count,
            },
            confidence: 0.6,
        });
    }

    // Grid-level transformation analysis.
    let cells_changed = start_grid.cells_changed(final_grid);
    let accuracy = start_grid.cell_accuracy(final_grid);

    if cells_changed > 0 {
        rules.push(TransformationRule {
            description: format!(
                "{} cells changed ({:.1}% grid accuracy)",
                cells_changed,
                accuracy * 100.0
            ),
            rule_type: if accuracy > 0.5 {
                TransformationRuleType::GridTransformed { cell_accuracy: accuracy }
            } else {
                TransformationRuleType::Unknown { cells_changed }
            },
            confidence: if accuracy > 0.8 { 0.7 } else { 0.3 },
        });
    }

    rules
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use eustress_vortex_grid2d::objects::{GridObject, BBox};

    fn make_frame_with_objects(objs: Vec<(StableObjectId, u8, usize, usize)>) -> ObjectFrame {
        let mut frame = ObjectFrame::default();
        for (id, color, row, col) in objs {
            frame.objects.insert(id, GridObject {
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
            });
        }
        frame
    }

    #[test]
    fn test_object_diffs_disappeared() {
        let start = make_frame_with_objects(vec![(1, 5, 0, 0), (2, 3, 5, 5)]);
        let end = make_frame_with_objects(vec![(1, 5, 0, 0)]);

        let diffs = compute_object_diffs(&start, &end);
        assert!(diffs.iter().any(|d| matches!(d, ObjectDiff::Disappeared { id: 2, .. })));
    }

    #[test]
    fn test_object_diffs_moved() {
        let start = make_frame_with_objects(vec![(1, 5, 0, 0)]);
        let end = make_frame_with_objects(vec![(1, 5, 3, 4)]);

        let diffs = compute_object_diffs(&start, &end);
        assert!(diffs.iter().any(|d| matches!(d, ObjectDiff::Moved { id: 1, .. })));
    }

    #[test]
    fn test_object_diffs_color_changed() {
        let start = make_frame_with_objects(vec![(1, 5, 0, 0)]);
        let end = make_frame_with_objects(vec![(1, 8, 0, 0)]);

        let diffs = compute_object_diffs(&start, &end);
        assert!(diffs.iter().any(|d| matches!(d, ObjectDiff::ColorChanged { id: 1, from: 5, to: 8 })));
    }

    #[test]
    fn test_level_completion_analyzer() {
        let mut analyzer = LevelCompletionAnalyzer::new();

        let start = Grid2D::new(vec![vec![0, 1], vec![0, 0]]);
        let final_g = Grid2D::new(vec![vec![0, 0], vec![1, 0]]);

        let record = analyzer.analyze_completion(
            0,
            &start,
            &final_g,
            vec!["2".into(), "3".into()],
            None,
            None,
            None,
            "test_game",
        );

        assert_eq!(record.level_index, 0);
        assert_eq!(record.step_count, 2);
        assert!(!record.transformation_rules.is_empty());
        assert_eq!(analyzer.levels_completed(), 1);
    }

    #[test]
    fn test_common_patterns() {
        let mut analyzer = LevelCompletionAnalyzer::new();

        let g = Grid2D::new(vec![vec![0, 1], vec![0, 0]]);
        let g2 = Grid2D::new(vec![vec![0, 0], vec![0, 0]]);

        // Two levels with similar disappearances
        let start_f = make_frame_with_objects(vec![(1, 5, 0, 0), (2, 3, 1, 1)]);
        let end_f = make_frame_with_objects(vec![(1, 5, 0, 0)]);

        analyzer.analyze_completion(0, &g, &g2, vec![], Some(&start_f), Some(&end_f), None, "g1");
        analyzer.analyze_completion(1, &g, &g2, vec![], Some(&start_f), Some(&end_f), None, "g1");

        let patterns = analyzer.common_patterns();
        assert!(patterns.contains(&"all_collected".to_string()));
    }
}
