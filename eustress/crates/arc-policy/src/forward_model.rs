//! Online object-level forward model — the Representation Accuracy loop.
//!
//! This is the "simulate before you act" spine for keyboard games:
//!
//! 1. PROBE: spend the first few actions measuring what each control does
//!    to the avatar (action → displacement distribution).
//! 2. PREDICT: before each action, forecast the avatar's next position.
//! 3. MEASURE: compare forecast to observation — per-action IRA.
//! 4. PLAN: when IRA is high, walk the avatar toward goal candidates with
//!    greedy distance-minimizing steps; when IRA is low, the most valuable
//!    action is the one that reduces uncertainty fastest (more probes).
//!
//! Everything is learned online, within the episode, in milliseconds — no
//! epochs, no pretrained weights. Profiles persist per game so a revisit
//! skips the probe phase entirely.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::object_tracker::{ObjectEvent, StableObjectId};

/// Movement actions in the ARC action space (ACTION1..ACTION5).
/// ACTION6 is click (handled by the click pipeline), ACTION7 submit.
pub const MOVEMENT_ACTIONS: [u32; 5] = [1, 2, 3, 4, 5];

/// How many times each action is probed before we trust its statistics.
const PROBES_PER_ACTION: u32 = 3;

/// Displacement agreement threshold: a control mapping is "confident" when
/// the mean displacement is consistent (low variance) and non-trivial.
const CONFIDENT_VAR: f32 = 1.5;
const MIN_DISPLACEMENT: f32 = 0.45;

/// Per-action displacement statistics for the avatar object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisplacementStats {
    pub samples: u32,
    pub mean_dr: f32,
    pub mean_dc: f32,
    /// EMA of squared deviation from the running mean (scalar, both axes).
    pub var: f32,
    /// Fraction of observations where the avatar moved at all.
    pub effect_rate: f32,
}

impl DisplacementStats {
    fn update(&mut self, dr: f32, dc: f32) {
        let n = self.samples as f32;
        let moved = (dr.abs() + dc.abs()) > 0.05;
        self.effect_rate = (self.effect_rate * n + if moved { 1.0 } else { 0.0 }) / (n + 1.0);
        // Deviation from current mean BEFORE folding the new sample in
        let dev = (dr - self.mean_dr).powi(2) + (dc - self.mean_dc).powi(2);
        self.var = if self.samples == 0 { 0.0 } else { self.var * 0.7 + dev * 0.3 };
        self.mean_dr = (self.mean_dr * n + dr) / (n + 1.0);
        self.mean_dc = (self.mean_dc * n + dc) / (n + 1.0);
        self.samples += 1;
    }

    /// Confident, consistent unit direction — None if noisy or no-op.
    pub fn unit_direction(&self) -> Option<(i32, i32)> {
        if self.samples < 2 || self.var > CONFIDENT_VAR {
            return None;
        }
        if self.mean_dr.abs() < MIN_DISPLACEMENT && self.mean_dc.abs() < MIN_DISPLACEMENT {
            return None;
        }
        let dr = if self.mean_dr.abs() >= MIN_DISPLACEMENT { self.mean_dr.signum() as i32 } else { 0 };
        let dc = if self.mean_dc.abs() >= MIN_DISPLACEMENT { self.mean_dc.signum() as i32 } else { 0 };
        Some((dr, dc))
    }
}

/// A persisted, per-game summary of learned mechanics. Reloading one on a
/// revisit skips the probe phase — knowledge persists across sessions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameProfile {
    /// action_id → consistent unit displacement of the avatar.
    pub control_map: HashMap<u32, (i32, i32)>,
    /// Mean prediction accuracy achieved with that map (provenance).
    pub ira: f32,
    /// Episodes that contributed to this profile.
    pub episodes: u32,
    /// Win-condition interpretation posteriors learned for this game —
    /// a revisit starts from what we already believe winning means here.
    #[serde(default)]
    pub predicates: Option<crate::predicates::PredicateEnsemble>,
    /// Per-level learning vault (fatal/dead/responsive clicks, predicate
    /// state) — cross-card continuation of the elimination search.
    #[serde(default)]
    pub level_vault: HashMap<u32, crate::world_model::LevelMemory>,
}

/// Online forward model over tracked objects.
#[derive(Debug, Clone, Default)]
pub struct ObjectForwardModel {
    /// Avatar displacement stats per movement action.
    pub avatar_effects: HashMap<u32, DisplacementStats>,
    /// Per-action prediction accuracy (EMA): the Representation Accuracy.
    pub action_ira: HashMap<u32, f32>,
    /// Remaining probe schedule (actions repeated PROBES_PER_ACTION times).
    probe_queue: Vec<u32>,
    /// Outstanding prediction: (action, predicted avatar centroid).
    pending: Option<(u32, (f32, f32))>,
    /// Whether the probe schedule has been seeded for this level.
    seeded: bool,
}

impl ObjectForwardModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed the probe schedule from this game's available movement actions.
    /// Round-robin (1,2,3,4,1,2,3,4,...) so early information spreads across
    /// controls instead of exhausting one before touching the next.
    pub fn seed_probes(&mut self, available: &[u32]) {
        if self.seeded {
            return;
        }
        self.seeded = true;
        let movement: Vec<u32> = MOVEMENT_ACTIONS
            .iter()
            .copied()
            .filter(|a| available.contains(a))
            .collect();
        for _round in 0..PROBES_PER_ACTION {
            for &a in &movement {
                self.probe_queue.push(a);
            }
        }
    }

    /// Adopt a persisted profile: pre-fill stats so probing is skipped.
    pub fn adopt_profile(&mut self, profile: &GameProfile) {
        for (&action, &(dr, dc)) in &profile.control_map {
            let stats = DisplacementStats {
                samples: PROBES_PER_ACTION,
                mean_dr: dr as f32,
                mean_dc: dc as f32,
                var: 0.0,
                effect_rate: 1.0,
            };
            self.avatar_effects.insert(action, stats);
            self.action_ira.insert(action, profile.ira.max(0.5));
        }
        self.seeded = true;
        self.probe_queue.clear();
    }

    /// Export the learned control map as a persistable profile.
    pub fn to_profile(&self) -> Option<GameProfile> {
        let control_map = self.control_map();
        if control_map.is_empty() {
            return None;
        }
        let ira = if self.action_ira.is_empty() {
            0.0
        } else {
            self.action_ira.values().sum::<f32>() / self.action_ira.len() as f32
        };
        Some(GameProfile {
            control_map,
            ira,
            episodes: 1,
            predicates: None,
            level_vault: HashMap::new(),
        })
    }

    /// Next probe action, if the schedule isn't exhausted.
    pub fn next_probe(&mut self) -> Option<u32> {
        self.probe_queue.pop()
    }

    pub fn probing(&self) -> bool {
        !self.probe_queue.is_empty()
    }

    /// The learned control map: action → unit displacement.
    pub fn control_map(&self) -> HashMap<u32, (i32, i32)> {
        self.avatar_effects
            .iter()
            .filter_map(|(&a, s)| s.unit_direction().map(|d| (a, d)))
            .collect()
    }

    /// Forecast the avatar centroid after `action` and remember it for the
    /// IRA comparison on the next observation.
    pub fn predict(&mut self, action: u32, avatar_centroid: (f32, f32)) {
        if let Some(stats) = self.avatar_effects.get(&action) {
            if stats.samples >= 1 {
                let predicted = (
                    avatar_centroid.0 + stats.mean_dr,
                    avatar_centroid.1 + stats.mean_dc,
                );
                self.pending = Some((action, predicted));
                return;
            }
        }
        self.pending = None;
    }

    /// Fold the observed outcome of the last action into the model:
    /// update displacement stats from the avatar's Moved event (or absence),
    /// and score the pending prediction → per-action IRA.
    pub fn observe(
        &mut self,
        action: u32,
        events: &[ObjectEvent],
        avatar: Option<StableObjectId>,
        avatar_centroid_now: Option<(f32, f32)>,
    ) {
        if !MOVEMENT_ACTIONS.contains(&action) {
            return;
        }

        // Avatar displacement this step (0,0 when no Moved event fired)
        let (mut dr, mut dc) = (0.0f32, 0.0f32);
        if let Some(avatar_id) = avatar {
            for e in events {
                if let ObjectEvent::Moved { id, delta_row, delta_col } = e {
                    if *id == avatar_id {
                        dr = *delta_row;
                        dc = *delta_col;
                    }
                }
            }
        }
        self.avatar_effects.entry(action).or_default().update(dr, dc);

        // Score the outstanding forecast — this is Representation Accuracy.
        if let (Some((pred_action, predicted)), Some(actual)) =
            (self.pending.take(), avatar_centroid_now)
        {
            if pred_action == action {
                let err = ((predicted.0 - actual.0).powi(2)
                    + (predicted.1 - actual.1).powi(2))
                .sqrt();
                // 1.0 at perfect, →0 as error grows past a couple of cells
                let accuracy = (1.0 - (err / 3.0)).clamp(0.0, 1.0);
                let ira = self.action_ira.entry(action).or_insert(0.5);
                *ira = *ira * 0.8 + accuracy * 0.2;
            }
        }
    }

    /// Mean IRA across actions with data — the model's overall trust level.
    pub fn mean_ira(&self) -> f32 {
        if self.action_ira.is_empty() {
            return 0.0;
        }
        self.action_ira.values().sum::<f32>() / self.action_ira.len() as f32
    }

    /// Greedy goal-directed step: pick the control whose learned direction
    /// best reduces distance from avatar to goal, weighted by that action's
    /// IRA. Returns (action, expected_progress, ira).
    pub fn step_toward(
        &self,
        avatar: (f32, f32),
        goal: (f32, f32),
    ) -> Option<(u32, f32, f32)> {
        let map = self.control_map();
        if map.is_empty() {
            return None;
        }
        let dist = |p: (f32, f32)| ((p.0 - goal.0).powi(2) + (p.1 - goal.1).powi(2)).sqrt();
        let here = dist(avatar);

        let mut best: Option<(u32, f32, f32)> = None;
        for (&action, &(dr, dc)) in &map {
            let next = (avatar.0 + dr as f32, avatar.1 + dc as f32);
            let progress = here - dist(next);
            let ira = self.action_ira.get(&action).copied().unwrap_or(0.5);
            let score = progress * ira;
            if best.map_or(true, |(_, bp, bi)| score > bp * bi) {
                best = Some((action, progress, ira));
            }
        }
        best.filter(|(_, progress, _)| *progress > 0.0)
    }

    /// Reset per-level state (stats persist — physics rarely change between
    /// levels of one game — but outstanding predictions don't).
    pub fn on_level_advance(&mut self) {
        self.pending = None;
    }

    /// Full reset for a new game.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// First action along the A* path from `start` to `goal` (row, col),
/// moving only in directions the learned control map provides and never
/// entering blocked cells. Greedy stepping walks into walls (one game
/// fixated on a single action oscillating against a barrier); A* routes
/// around them optimally. Returns None when no path exists within budget.
pub fn astar_step(
    control_map: &HashMap<u32, (i32, i32)>,
    start: (i32, i32),
    goal: (i32, i32),
    is_blocked: &dyn Fn(i32, i32) -> bool,
    bounds: (i32, i32),
) -> Option<u32> {
    use std::cmp::Reverse;
    use std::collections::{BinaryHeap, HashMap as Map};

    if control_map.is_empty() || start == goal {
        return None;
    }
    let h = |p: (i32, i32)| (p.0 - goal.0).abs() + (p.1 - goal.1).abs();

    let mut open: BinaryHeap<(Reverse<i32>, (i32, i32))> = BinaryHeap::new();
    let mut g: Map<(i32, i32), i32> = Map::new();
    let mut first_action: Map<(i32, i32), u32> = Map::new();

    open.push((Reverse(h(start)), start));
    g.insert(start, 0);

    let mut explored = 0;
    while let Some((_, cur)) = open.pop() {
        explored += 1;
        if explored > 4096 {
            break;
        }
        if cur == goal || h(cur) <= 1 {
            return first_action.get(&cur).copied();
        }
        let cur_g = *g.get(&cur).unwrap_or(&i32::MAX);
        for (&action, &(dr, dc)) in control_map {
            let next = (cur.0 + dr, cur.1 + dc);
            if next.0 < 0 || next.1 < 0 || next.0 >= bounds.0 || next.1 >= bounds.1 {
                continue;
            }
            if is_blocked(next.0, next.1) {
                continue;
            }
            let tentative = cur_g + 1;
            if tentative < *g.get(&next).unwrap_or(&i32::MAX) {
                g.insert(next, tentative);
                // Propagate which FIRST action started this path
                let fa = first_action.get(&cur).copied().unwrap_or(action);
                first_action.insert(next, fa);
                open.push((Reverse(tentative + h(next)), next));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn astar_routes_around_walls() {
        let mut map = HashMap::new();
        map.insert(1u32, (-1i32, 0i32)); // up
        map.insert(2, (1, 0)); // down
        map.insert(3, (0, -1)); // left
        map.insert(4, (0, 1)); // right
        // Row 5 walled for c>=1 and column 5 walled below row 0: from
        // (5,0) the ONLY viable first move is up; the path crosses via
        // row 0 and descends to the goal at (4,9).
        let blocked = |r: i32, c: i32| (c == 5 && r != 0) || (r == 5 && c >= 1);
        let first = astar_step(&map, (5, 0), (4, 9), &blocked, (10, 10));
        assert_eq!(first, Some(1), "must start by moving up to round the wall");
    }

    #[test]
    fn control_map_learned_from_consistent_motion() {
        let mut m = ObjectForwardModel::new();
        m.seed_probes(&[1, 2, 6]);
        assert!(m.probing());
        // Action 1 consistently moves the avatar up (dr = -1)
        for _ in 0..3 {
            m.avatar_effects.entry(1).or_default().update(-1.0, 0.0);
        }
        let map = m.control_map();
        assert_eq!(map.get(&1), Some(&(-1, 0)));
    }

    #[test]
    fn noisy_action_yields_no_mapping() {
        let mut m = ObjectForwardModel::new();
        let s = m.avatar_effects.entry(3).or_default();
        s.update(1.0, 0.0);
        s.update(-1.0, 1.0);
        s.update(0.0, -1.0);
        s.update(1.0, 1.0);
        assert_eq!(m.control_map().get(&3), None);
    }

    #[test]
    fn step_toward_picks_distance_reducing_action() {
        let mut m = ObjectForwardModel::new();
        for _ in 0..3 {
            m.avatar_effects.entry(1).or_default().update(-1.0, 0.0); // up
            m.avatar_effects.entry(2).or_default().update(1.0, 0.0); // down
            m.avatar_effects.entry(4).or_default().update(0.0, 1.0); // right
        }
        // Goal is to the right of the avatar
        let pick = m.step_toward((5.0, 5.0), (5.0, 20.0)).expect("step");
        assert_eq!(pick.0, 4);
    }

    #[test]
    fn ira_rises_on_correct_predictions() {
        let mut m = ObjectForwardModel::new();
        for _ in 0..3 {
            m.avatar_effects.entry(4).or_default().update(0.0, 1.0);
        }
        for i in 0..5 {
            let here = (5.0, 5.0 + i as f32);
            m.predict(4, here);
            m.observe(
                4,
                &[ObjectEvent::Moved { id: 7, delta_row: 0.0, delta_col: 1.0 }],
                Some(7),
                Some((here.0, here.1 + 1.0)),
            );
        }
        assert!(m.action_ira.get(&4).copied().unwrap_or(0.0) > 0.6);
    }

    #[test]
    fn profile_roundtrip_skips_probing() {
        let mut m = ObjectForwardModel::new();
        for _ in 0..3 {
            m.avatar_effects.entry(1).or_default().update(-1.0, 0.0);
        }
        m.action_ira.insert(1, 0.9);
        let profile = m.to_profile().expect("profile");

        let mut fresh = ObjectForwardModel::new();
        fresh.adopt_profile(&profile);
        assert!(!fresh.probing());
        assert_eq!(fresh.control_map().get(&1), Some(&(-1, 0)));
    }
}
