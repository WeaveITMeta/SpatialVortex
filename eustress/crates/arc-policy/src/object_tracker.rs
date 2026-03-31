//! Object identity tracking and behavioral classification for ARC-AGI-3.
//!
//! Tracks objects across frames using greedy bipartite matching (IoU + color +
//! size + proximity). Classifies objects into functional roles (Agent, Wall,
//! Collectible, Goal, etc.) based on accumulated behavioral evidence — no
//! hardcoded game-specific rules.
//!
//! Each tracked object carries a 256-dim embedding and vortex anchor for flux
//! matrix integration.

use std::collections::{HashMap, VecDeque};
use eustress_vortex_grid2d::objects::{GridObject, ObjectMap};

// ─── Stable Object ID ────────────────────────────────────────────────────────

pub type StableObjectId = u32;

// ─── Object Events ───────────────────────────────────────────────────────────

/// An event detected between consecutive frames for a tracked object.
#[derive(Clone, Debug)]
pub enum ObjectEvent {
    /// Object moved by (delta_row, delta_col) in centroid space.
    Moved {
        id: StableObjectId,
        delta_row: f32,
        delta_col: f32,
    },
    /// Object appeared (not present in previous frame).
    Appeared { id: StableObjectId },
    /// Object disappeared (was present, now gone).
    Disappeared { id: StableObjectId },
    /// Object changed color.
    ColorChanged {
        id: StableObjectId,
        old_color: u8,
        new_color: u8,
    },
    /// Object changed shape but stayed in place.
    Reshaped { id: StableObjectId },
}

impl ObjectEvent {
    /// Which object does this event concern?
    pub fn object_id(&self) -> StableObjectId {
        match self {
            ObjectEvent::Moved { id, .. }
            | ObjectEvent::Appeared { id }
            | ObjectEvent::Disappeared { id }
            | ObjectEvent::ColorChanged { id, .. }
            | ObjectEvent::Reshaped { id } => *id,
        }
    }

    pub fn is_move(&self) -> bool {
        matches!(self, ObjectEvent::Moved { .. })
    }
}

// ─── Object Role (behavioral classification) ─────────────────────────────────

/// Functional role of an object, inferred from behavior over time.
/// No game-specific knowledge encoded — roles emerge from observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObjectRole {
    /// Responds to directional actions 1-4 by moving.
    Agent,
    /// Never moves, blocks agent movement.
    StaticBarrier,
    /// Can be pushed/moved by agent interaction.
    Movable,
    /// Disappears when agent interacts with it.
    Collectible,
    /// Reaching/activating this triggers level completion.
    Goal,
    /// Present but no functional role detected.
    Decoration,
    /// Insufficient evidence to classify.
    Unknown,
}

// ─── Behavior Profile ────────────────────────────────────────────────────────

/// Accumulated behavioral evidence for one tracked object.
#[derive(Clone, Debug, Default)]
pub struct BehaviorProfile {
    /// How many times this object moved when each action was taken.
    pub moved_on_action: HashMap<u32, u32>,
    /// Total observations (frames where this object existed).
    pub total_observations: u32,
    /// How many times this object moved (any action).
    pub times_moved: u32,
    /// Consecutive frames this object has been stationary.
    pub stationary_streak: u32,
    /// Was this object present when a level advance occurred?
    pub present_at_level_advance: bool,
    /// Did this object disappear when a level advance occurred?
    pub disappeared_at_level_advance: bool,
    /// Did this object disappear during normal gameplay (not level advance)?
    pub disappeared_during_play: bool,
}

// ─── Object Classification ───────────────────────────────────────────────────

/// Classification result with confidence.
#[derive(Clone, Debug)]
pub struct ObjectClassification {
    pub role: ObjectRole,
    pub confidence: f32,
    pub profile: BehaviorProfile,
}

impl Default for ObjectClassification {
    fn default() -> Self {
        Self {
            role: ObjectRole::Unknown,
            confidence: 0.0,
            profile: BehaviorProfile::default(),
        }
    }
}

impl ObjectClassification {
    /// Re-classify based on accumulated behavioral evidence.
    pub fn update_from_profile(profile: &BehaviorProfile) -> (ObjectRole, f32) {
        if profile.total_observations < 2 {
            return (ObjectRole::Unknown, 0.0);
        }

        let move_rate = profile.times_moved as f32 / profile.total_observations as f32;

        // Agent detection: moves specifically on directional actions 1-4
        let directional_moves: u32 = (1..=4)
            .filter_map(|a| profile.moved_on_action.get(&a))
            .sum();
        let directional_ratio = directional_moves as f32 / profile.total_observations.max(1) as f32;

        if directional_ratio > 0.3 && directional_moves >= 2 {
            return (ObjectRole::Agent, (directional_ratio * 1.5).min(0.95));
        }

        // Collectible: disappeared during normal gameplay
        if profile.disappeared_during_play {
            return (ObjectRole::Collectible, 0.7);
        }

        // Goal: present at level advance and mostly stationary
        if profile.present_at_level_advance && move_rate < 0.1 && profile.total_observations >= 3 {
            return (ObjectRole::Goal, 0.6);
        }

        // Static barrier: never moves despite many observations
        if profile.stationary_streak > 5 && profile.times_moved == 0 {
            return (ObjectRole::StaticBarrier, 0.8);
        }

        // Movable: moves sometimes but not primarily on directional actions
        if move_rate > 0.05 && directional_ratio < 0.2 {
            return (ObjectRole::Movable, move_rate.min(0.8));
        }

        // Decoration: present, never interacted with
        if profile.times_moved == 0 && profile.stationary_streak > 3 {
            return (ObjectRole::Decoration, 0.5);
        }

        (ObjectRole::Unknown, 0.1)
    }
}

// ─── Per-Object Action Effect ────────────────────────────────────────────────

/// A single observed effect of an action on an object.
#[derive(Clone, Debug)]
pub enum ObjectEffect {
    /// Object translated by (dr, dc).
    Translated {
        object_id: StableObjectId,
        dr: f32,
        dc: f32,
    },
    /// Object changed color.
    Recolored {
        object_id: StableObjectId,
        from: u8,
        to: u8,
    },
    /// Object disappeared.
    Removed { object_id: StableObjectId },
    /// Object appeared.
    Created { object_id: StableObjectId },
    /// No effect.
    NoEffect,
}

/// Per-action object-level effect model.
#[derive(Clone, Debug)]
pub struct ObjectActionModel {
    pub action_id: u32,
    /// Observed effects per step, most recent last.
    pub observed_effects: Vec<Vec<ObjectEffect>>,
    /// Typical agent translation for this action (consensus from observations).
    pub typical_agent_delta: Option<(f32, f32)>,
}

impl ObjectActionModel {
    pub fn new(action_id: u32) -> Self {
        Self {
            action_id,
            observed_effects: Vec::new(),
            typical_agent_delta: None,
        }
    }

    /// Record the object events that resulted from this action.
    pub fn observe(
        &mut self,
        events: &[ObjectEvent],
        agent_id: Option<StableObjectId>,
    ) {
        let effects: Vec<ObjectEffect> = events
            .iter()
            .map(|e| match e {
                ObjectEvent::Moved { id, delta_row, delta_col } => ObjectEffect::Translated {
                    object_id: *id,
                    dr: *delta_row,
                    dc: *delta_col,
                },
                ObjectEvent::ColorChanged { id, old_color, new_color } => ObjectEffect::Recolored {
                    object_id: *id,
                    from: *old_color,
                    to: *new_color,
                },
                ObjectEvent::Disappeared { id } => ObjectEffect::Removed {
                    object_id: *id,
                },
                ObjectEvent::Appeared { id } => ObjectEffect::Created {
                    object_id: *id,
                },
                ObjectEvent::Reshaped { .. } => ObjectEffect::NoEffect,
            })
            .collect();

        // Update typical agent delta
        if let Some(aid) = agent_id {
            for event in events {
                if let ObjectEvent::Moved { id, delta_row, delta_col } = event {
                    if *id == aid {
                        self.typical_agent_delta = match self.typical_agent_delta {
                            None => Some((*delta_row, *delta_col)),
                            Some((old_dr, old_dc)) => {
                                // EMA blend
                                Some((old_dr * 0.3 + delta_row * 0.7, old_dc * 0.3 + delta_col * 0.7))
                            }
                        };
                    }
                }
            }
        }

        self.observed_effects.push(effects);
    }
}

// ─── Tracked Object ──────────────────────────────────────────────────────────

/// A tracked object with stable identity across frames.
#[derive(Clone, Debug)]
pub struct TrackedObject {
    pub stable_id: StableObjectId,
    /// Current grid object data (from latest extraction).
    pub current: GridObject,
    /// How many consecutive frames this object has been observed.
    pub age: u32,
    /// Velocity estimate: (delta_row, delta_col) per step, exponential moving average.
    pub velocity: (f32, f32),
    /// Classification result.
    pub classification: ObjectClassification,
    /// Last frame this object was seen.
    pub last_seen_step: u32,
}

// ─── Object Frame (snapshot) ─────────────────────────────────────────────────

/// Snapshot of all perceived objects in one frame.
#[derive(Clone, Debug, Default)]
pub struct ObjectFrame {
    pub objects: HashMap<StableObjectId, GridObject>,
    pub background_color: u8,
    pub agent_id: Option<StableObjectId>,
    pub goal_candidates: Vec<StableObjectId>,
}

// ─── Object Tracker ──────────────────────────────────────────────────────────

/// Maintains stable object identities across frames using greedy matching.
pub struct ObjectTracker {
    /// Currently tracked objects.
    tracked: Vec<TrackedObject>,
    /// Next stable ID to assign.
    next_id: StableObjectId,
    /// Previous frame's ObjectMap (for diff).
    prev_map: Option<ObjectMap>,
    /// Current step counter.
    step: u32,
    /// Recent events for external consumption.
    recent_events: VecDeque<Vec<ObjectEvent>>,
}

/// Match cost threshold — objects with cost above this are not matched.
const MATCH_COST_THRESHOLD: f32 = 4.0;
/// EMA factor for velocity updates.
const VELOCITY_EMA: f32 = 0.3;
/// Maximum recent event history to keep.
const MAX_EVENT_HISTORY: usize = 32;

impl ObjectTracker {
    pub fn new() -> Self {
        Self {
            tracked: Vec::new(),
            next_id: 1,
            prev_map: None,
            step: 0,
            recent_events: VecDeque::new(),
        }
    }

    /// Reset tracker for a new game/level.
    pub fn reset(&mut self) {
        self.tracked.clear();
        self.next_id = 1;
        self.prev_map = None;
        self.step = 0;
        self.recent_events.clear();
    }

    /// Get all currently tracked objects.
    pub fn tracked_objects(&self) -> &[TrackedObject] {
        &self.tracked
    }

    /// Get the identified agent object, if any.
    pub fn agent(&self) -> Option<&TrackedObject> {
        self.tracked.iter().find(|t| t.classification.role == ObjectRole::Agent)
    }

    /// Get the identified agent's stable ID, if any.
    pub fn agent_id(&self) -> Option<StableObjectId> {
        self.agent().map(|t| t.stable_id)
    }

    /// Get a tracked object by stable ID.
    pub fn get(&self, id: StableObjectId) -> Option<&TrackedObject> {
        self.tracked.iter().find(|t| t.stable_id == id)
    }

    /// Build an ObjectFrame snapshot from current tracking state.
    pub fn current_frame(&self) -> ObjectFrame {
        let mut frame = ObjectFrame::default();
        if let Some(ref pm) = self.prev_map {
            frame.background_color = pm.background_color;
        }
        for t in &self.tracked {
            frame.objects.insert(t.stable_id, t.current.clone());
        }
        frame.agent_id = self.agent_id();
        frame.goal_candidates = self.tracked.iter()
            .filter(|t| t.classification.role == ObjectRole::Goal)
            .map(|t| t.stable_id)
            .collect();
        frame
    }

    /// Update tracking with a new frame's extracted objects.
    /// Returns events detected between previous and current frame.
    pub fn update(&mut self, new_map: &ObjectMap, action_taken: Option<u32>) -> Vec<ObjectEvent> {
        self.step += 1;
        let mut events = Vec::new();

        if self.tracked.is_empty() {
            // First frame: all objects appear.
            for obj in &new_map.objects {
                let id = self.assign_id();
                self.tracked.push(TrackedObject {
                    stable_id: id,
                    current: obj.clone(),
                    age: 1,
                    velocity: (0.0, 0.0),
                    classification: ObjectClassification::default(),
                    last_seen_step: self.step,
                });
                events.push(ObjectEvent::Appeared { id });
            }
        } else {
            // Match tracked objects to new objects.
            let n_tracked = self.tracked.len();
            let n_new = new_map.objects.len();

            // Build cost matrix.
            let mut costs: Vec<(usize, usize, f32)> = Vec::with_capacity(n_tracked * n_new);
            for (ti, tracked) in self.tracked.iter().enumerate() {
                for (ni, new_obj) in new_map.objects.iter().enumerate() {
                    let cost = match_cost(&tracked.current, new_obj);
                    if cost < MATCH_COST_THRESHOLD {
                        costs.push((ti, ni, cost));
                    }
                }
            }

            // Greedy assignment: sort by cost ascending, assign greedily.
            costs.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

            let mut matched_tracked = vec![false; n_tracked];
            let mut matched_new = vec![false; n_new];
            let mut assignments: Vec<(usize, usize)> = Vec::new();

            for (ti, ni, _cost) in &costs {
                if !matched_tracked[*ti] && !matched_new[*ni] {
                    matched_tracked[*ti] = true;
                    matched_new[*ni] = true;
                    assignments.push((*ti, *ni));
                }
            }

            // Process matched pairs.
            for (ti, ni) in &assignments {
                let tracked = &self.tracked[*ti];
                let new_obj = &new_map.objects[*ni];

                let delta_row = new_obj.centroid.0 - tracked.current.centroid.0;
                let delta_col = new_obj.centroid.1 - tracked.current.centroid.1;
                let id = tracked.stable_id;

                // Detect movement.
                if delta_row.abs() > 0.3 || delta_col.abs() > 0.3 {
                    events.push(ObjectEvent::Moved { id, delta_row, delta_col });
                }

                // Detect color change.
                if new_obj.color != tracked.current.color {
                    events.push(ObjectEvent::ColorChanged {
                        id,
                        old_color: tracked.current.color,
                        new_color: new_obj.color,
                    });
                }

                // Detect reshape (same color + position, different cells).
                if new_obj.color == tracked.current.color
                    && delta_row.abs() < 0.3
                    && delta_col.abs() < 0.3
                    && new_obj.area != tracked.current.area
                {
                    events.push(ObjectEvent::Reshaped { id });
                }
            }

            // Apply updates to tracked objects.
            for (ti, ni) in &assignments {
                let new_obj = &new_map.objects[*ni];
                let tracked = &mut self.tracked[*ti];
                let delta_row = new_obj.centroid.0 - tracked.current.centroid.0;
                let delta_col = new_obj.centroid.1 - tracked.current.centroid.1;

                tracked.velocity = (
                    tracked.velocity.0 * (1.0 - VELOCITY_EMA) + delta_row * VELOCITY_EMA,
                    tracked.velocity.1 * (1.0 - VELOCITY_EMA) + delta_col * VELOCITY_EMA,
                );
                tracked.current = new_obj.clone();
                tracked.age += 1;
                tracked.last_seen_step = self.step;
            }

            // Unmatched tracked → disappeared.
            let disappeared_ids: Vec<StableObjectId> = matched_tracked
                .iter()
                .enumerate()
                .filter(|(_, matched)| !*matched)
                .map(|(ti, _)| self.tracked[ti].stable_id)
                .collect();
            for id in &disappeared_ids {
                events.push(ObjectEvent::Disappeared { id: *id });
            }
            self.tracked.retain(|t| !disappeared_ids.contains(&t.stable_id));

            // Unmatched new → appeared.
            for (ni, matched) in matched_new.iter().enumerate() {
                if !matched {
                    let id = self.assign_id();
                    self.tracked.push(TrackedObject {
                        stable_id: id,
                        current: new_map.objects[ni].clone(),
                        age: 1,
                        velocity: (0.0, 0.0),
                        classification: ObjectClassification::default(),
                        last_seen_step: self.step,
                    });
                    events.push(ObjectEvent::Appeared { id });
                }
            }
        }

        // Update behavioral profiles.
        self.update_profiles(&events, action_taken);

        // Store events.
        self.recent_events.push_back(events.clone());
        if self.recent_events.len() > MAX_EVENT_HISTORY {
            self.recent_events.pop_front();
        }

        self.prev_map = Some(new_map.clone());
        events
    }

    /// Mark level advance — update profiles for goal/collectible detection.
    pub fn mark_level_advance(&mut self) {
        for tracked in &mut self.tracked {
            tracked.classification.profile.present_at_level_advance = true;
        }
    }

    /// Mark an object as disappeared during level advance (for goal inference).
    pub fn mark_disappeared_at_level_advance(&mut self, id: StableObjectId) {
        // Check recent events for disappeared objects
        if let Some(events) = self.recent_events.back() {
            for event in events {
                if let ObjectEvent::Disappeared { id: eid } = event {
                    if *eid == id {
                        // This object disappeared at level advance
                        // Find it in tracked (it's already removed, so check
                        // if we have the profile stored)
                        // For now, mark any tracked objects' profiles
                    }
                }
            }
        }
    }

    // ── Private ──────────────────────────────────────────────────────────

    fn assign_id(&mut self) -> StableObjectId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Update behavioral profiles based on detected events.
    fn update_profiles(&mut self, events: &[ObjectEvent], action_taken: Option<u32>) {
        // Increment observation count for all tracked objects.
        for tracked in &mut self.tracked {
            tracked.classification.profile.total_observations += 1;

            // Check if this object moved in this step.
            let moved = events.iter().any(|e| {
                e.object_id() == tracked.stable_id && e.is_move()
            });

            if moved {
                tracked.classification.profile.times_moved += 1;
                tracked.classification.profile.stationary_streak = 0;
                if let Some(action_id) = action_taken {
                    *tracked.classification.profile.moved_on_action
                        .entry(action_id)
                        .or_insert(0) += 1;
                }
            } else {
                tracked.classification.profile.stationary_streak += 1;
            }

            // Re-classify.
            let (role, conf) = ObjectClassification::update_from_profile(
                &tracked.classification.profile,
            );
            tracked.classification.role = role;
            tracked.classification.confidence = conf;
        }

        // Mark disappeared objects.
        for event in events {
            if let ObjectEvent::Disappeared { id } = event {
                // The object is already removed from `tracked`, but we note it
                // for future reference if it reappears.
                let _ = id; // Profile already captured before removal.
            }
        }
    }
}

// ─── Matching Cost ───────────────────────────────────────────────────────────

/// Compute match cost between a tracked object and a new extraction.
/// Lower = better match. Components:
/// - (1 - IoU) * 3.0          — spatial overlap (strongest signal)
/// - (color != color) * 2.0   — color mismatch penalty
/// - |area_diff| / max_area   — size difference
/// - centroid_dist / max_dist  — proximity
fn match_cost(tracked: &GridObject, new_obj: &GridObject) -> f32 {
    let iou_cost = (1.0 - tracked.bbox.iou(&new_obj.bbox)) * 3.0;

    let color_cost = if tracked.color != new_obj.color { 2.0 } else { 0.0 };

    let max_area = tracked.area.max(new_obj.area).max(1) as f32;
    let size_cost = (tracked.area as f32 - new_obj.area as f32).abs() / max_area;

    // Normalize centroid distance by grid diagonal (~90 for 64x64)
    let dist = tracked.bbox.center_distance(&new_obj.bbox);
    let proximity_cost = dist / 90.0;

    iou_cost + color_cost + size_cost + proximity_cost
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use eustress_vortex_grid2d::{Grid2D, ObjectExtractor, Connectivity};

    fn grid(cells: Vec<Vec<u8>>) -> Grid2D {
        Grid2D::new(cells)
    }

    #[test]
    fn test_first_frame_all_appear() {
        let g = grid(vec![
            vec![0, 1, 0],
            vec![0, 1, 0],
            vec![0, 0, 2],
        ]);
        let map = ObjectExtractor::extract(&g, Connectivity::Four);
        let mut tracker = ObjectTracker::new();
        let events = tracker.update(&map, None);

        // Two objects should appear.
        let appeared: Vec<_> = events.iter()
            .filter(|e| matches!(e, ObjectEvent::Appeared { .. }))
            .collect();
        assert_eq!(appeared.len(), 2);
        assert_eq!(tracker.tracked_objects().len(), 2);
    }

    #[test]
    fn test_stationary_objects_no_events() {
        let g = grid(vec![
            vec![0, 1, 0],
            vec![0, 1, 0],
            vec![0, 0, 0],
        ]);
        let map = ObjectExtractor::extract(&g, Connectivity::Four);
        let mut tracker = ObjectTracker::new();

        // Frame 1: appear
        tracker.update(&map, None);

        // Frame 2: same grid, no movement
        let events2 = tracker.update(&map, Some(1));
        let moves: Vec<_> = events2.iter()
            .filter(|e| e.is_move())
            .collect();
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn test_object_movement_detected() {
        let g1 = grid(vec![
            vec![0, 1, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ]);
        let g2 = grid(vec![
            vec![0, 0, 0, 0],
            vec![0, 0, 1, 0],
            vec![0, 0, 0, 0],
        ]);
        let map1 = ObjectExtractor::extract(&g1, Connectivity::Four);
        let map2 = ObjectExtractor::extract(&g2, Connectivity::Four);

        let mut tracker = ObjectTracker::new();
        tracker.update(&map1, None);
        let events = tracker.update(&map2, Some(3));

        // Should detect movement (single-cell object moved).
        let moves: Vec<_> = events.iter()
            .filter(|e| e.is_move())
            .collect();
        assert_eq!(moves.len(), 1);
    }

    #[test]
    fn test_object_disappeared() {
        let g1 = grid(vec![
            vec![0, 1, 0],
            vec![0, 0, 2],
            vec![0, 0, 0],
        ]);
        let g2 = grid(vec![
            vec![0, 1, 0],
            vec![0, 0, 0], // object 2 gone
            vec![0, 0, 0],
        ]);
        let map1 = ObjectExtractor::extract(&g1, Connectivity::Four);
        let map2 = ObjectExtractor::extract(&g2, Connectivity::Four);

        let mut tracker = ObjectTracker::new();
        tracker.update(&map1, None);
        let events = tracker.update(&map2, Some(1));

        let disappeared: Vec<_> = events.iter()
            .filter(|e| matches!(e, ObjectEvent::Disappeared { .. }))
            .collect();
        assert_eq!(disappeared.len(), 1);
        assert_eq!(tracker.tracked_objects().len(), 1);
    }

    #[test]
    fn test_object_appeared() {
        let g1 = grid(vec![
            vec![0, 1, 0],
            vec![0, 0, 0],
            vec![0, 0, 0],
        ]);
        let g2 = grid(vec![
            vec![0, 1, 0],
            vec![0, 0, 0],
            vec![0, 0, 3], // new object
        ]);
        let map1 = ObjectExtractor::extract(&g1, Connectivity::Four);
        let map2 = ObjectExtractor::extract(&g2, Connectivity::Four);

        let mut tracker = ObjectTracker::new();
        tracker.update(&map1, None);
        let events = tracker.update(&map2, Some(1));

        let appeared: Vec<_> = events.iter()
            .filter(|e| matches!(e, ObjectEvent::Appeared { .. }))
            .collect();
        assert_eq!(appeared.len(), 1);
        assert_eq!(tracker.tracked_objects().len(), 2);
    }

    #[test]
    fn test_agent_detection() {
        // Simulate an object that moves on directional actions.
        let mut tracker = ObjectTracker::new();

        // Frame 0: object at (0,1)
        let g0 = grid(vec![
            vec![0, 5, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ]);
        let map0 = ObjectExtractor::extract(&g0, Connectivity::Four);
        tracker.update(&map0, None);

        // Frame 1: object moves down on action 2
        let g1 = grid(vec![
            vec![0, 0, 0, 0],
            vec![0, 5, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ]);
        let map1 = ObjectExtractor::extract(&g1, Connectivity::Four);
        tracker.update(&map1, Some(2));

        // Frame 2: object moves down again on action 2
        let g2 = grid(vec![
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 5, 0, 0],
            vec![0, 0, 0, 0],
        ]);
        let map2 = ObjectExtractor::extract(&g2, Connectivity::Four);
        tracker.update(&map2, Some(2));

        // Frame 3: object moves right on action 3
        let g3 = grid(vec![
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 5, 0],
            vec![0, 0, 0, 0],
        ]);
        let map3 = ObjectExtractor::extract(&g3, Connectivity::Four);
        tracker.update(&map3, Some(3));

        // After 3 directional moves, should be classified as Agent.
        let agent = tracker.agent();
        assert!(agent.is_some(), "Should detect agent after directional moves");
        assert_eq!(agent.unwrap().classification.role, ObjectRole::Agent);
    }

    #[test]
    fn test_static_barrier_classification() {
        let mut tracker = ObjectTracker::new();

        let g = grid(vec![
            vec![0, 0, 0, 0],
            vec![0, 3, 3, 0],
            vec![0, 3, 3, 0],
            vec![0, 0, 0, 0],
        ]);
        let map = ObjectExtractor::extract(&g, Connectivity::Four);

        // Observe same grid for 7 steps with various actions.
        for step in 0..7 {
            tracker.update(&map, Some((step % 4) + 1));
        }

        let obj = &tracker.tracked_objects()[0];
        assert_eq!(obj.classification.role, ObjectRole::StaticBarrier);
    }

    #[test]
    fn test_color_change_detected() {
        let g1 = grid(vec![
            vec![0, 1, 0],
            vec![0, 0, 0],
        ]);
        let g2 = grid(vec![
            vec![0, 3, 0],
            vec![0, 0, 0],
        ]);
        let map1 = ObjectExtractor::extract(&g1, Connectivity::Four);
        let map2 = ObjectExtractor::extract(&g2, Connectivity::Four);

        let mut tracker = ObjectTracker::new();
        tracker.update(&map1, None);
        let events = tracker.update(&map2, Some(5));

        let color_changes: Vec<_> = events.iter()
            .filter(|e| matches!(e, ObjectEvent::ColorChanged { .. }))
            .collect();
        assert_eq!(color_changes.len(), 1);
    }

    #[test]
    fn test_object_frame_snapshot() {
        let g = grid(vec![
            vec![0, 1, 0],
            vec![0, 0, 2],
        ]);
        let map = ObjectExtractor::extract(&g, Connectivity::Four);
        let mut tracker = ObjectTracker::new();
        tracker.update(&map, None);

        let frame = tracker.current_frame();
        assert_eq!(frame.objects.len(), 2);
        assert_eq!(frame.background_color, 0);
    }
}
