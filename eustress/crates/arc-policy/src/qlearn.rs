//! Cross-game Q-learning over abstracted world-model state.
//!
//! Vanilla Q-learning over raw 64×64 grids cannot work here: the state
//! space is astronomical and the reward (a level advance) arrives once per
//! hundreds of actions. This layer learns over the world model's
//! ABSTRACTION instead — a few hundred recurring situations like "click
//! game, many targets left" or "avatar north-west of goal, path blocked" —
//! which makes the table small, dense, and most importantly TRANSFERABLE:
//! an unseen game maps onto abstract states already explored in other
//! games, inheriting their Q-values as a prior (the answer to "what about
//! games we have never seen?").
//!
//! Reward shaping uses signals the world model already computes: level
//! advances (the true reward), progress proxies (target count dropping,
//! goal distance shrinking), a small step cost, and first-visit curiosity
//! so the table fills before the first win ever happens.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Compact abstract state. String-keyed so the table serializes naturally
/// into the knowledge snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbstractState(pub String);

impl AbstractState {
    /// Click-game state: how much is left to do, and do we understand the
    /// mechanic yet. Buckets keep the space tiny.
    pub fn click(targets_remaining: usize, mechanic_known: bool) -> Self {
        let bucket = match targets_remaining {
            0 => "0",
            1..=2 => "1-2",
            3..=5 => "3-5",
            6..=10 => "6-10",
            _ => "many",
        };
        AbstractState(format!("click|t{}|m{}", bucket, mechanic_known as u8))
    }

    /// Keyboard-game state: where the goal is relative to the avatar
    /// (sign-bucketed), and whether the last move was blocked.
    pub fn movement(goal_dr: f32, goal_dc: f32, blocked: bool) -> Self {
        let sgn = |v: f32| {
            if v > 1.5 { "+" } else if v < -1.5 { "-" } else { "0" }
        };
        AbstractState(format!(
            "move|r{}|c{}|b{}",
            sgn(goal_dr), sgn(goal_dc), blocked as u8
        ))
    }
}

/// One state's action values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateValues {
    /// action_id → Q value.
    pub q: HashMap<u32, f32>,
    /// Visit count (drives exploration bonus decay).
    pub visits: u32,
}

/// Tabular Q-learner shared across games and sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QTable {
    pub states: HashMap<String, StateValues>,
    pub alpha: f32,
    pub gamma: f32,
    /// Pending transition: (state, action) awaiting its outcome.
    #[serde(skip)]
    pending: Option<(String, u32)>,
}

impl Default for QTable {
    fn default() -> Self {
        Self {
            states: HashMap::new(),
            alpha: 0.3,
            gamma: 0.9,
            pending: None,
        }
    }
}

impl QTable {
    /// Choose an action by optimistic greedy selection: unseen actions get
    /// an optimism bonus so every action gets tried, decaying with visits.
    /// Returns (action, q_value, confidence) — confidence grows with visits.
    pub fn choose(&mut self, state: &AbstractState, available: &[u32]) -> Option<(u32, f32, f32)> {
        if available.is_empty() {
            return None;
        }
        let entry = self.states.entry(state.0.clone()).or_default();
        let optimism = 0.5 / (1.0 + entry.visits as f32 * 0.1);
        let (best, q) = available
            .iter()
            .map(|&a| (a, entry.q.get(&a).copied().unwrap_or(0.0) + optimism * ((a as f32 * 7.13).sin().abs())))
            .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal))?;
        let confidence = (entry.visits as f32 / 20.0).min(0.9);
        Some((best, q, confidence))
    }

    /// Register that `action` was taken from `state` (outcome pending).
    pub fn act(&mut self, state: &AbstractState, action: u32) {
        if let Some(e) = self.states.get_mut(&state.0) {
            e.visits += 1;
        }
        self.pending = Some((state.0.clone(), action));
    }

    /// TD update for the pending transition given the observed reward and
    /// the successor state.
    pub fn observe(&mut self, reward: f32, next_state: &AbstractState) {
        let Some((s, a)) = self.pending.take() else { return };
        let next_max = self
            .states
            .get(&next_state.0)
            .map(|e| e.q.values().copied().fold(0.0f32, f32::max))
            .unwrap_or(0.0);
        let entry = self.states.entry(s).or_default();
        let q = entry.q.entry(a).or_insert(0.0);
        *q += self.alpha * (reward + self.gamma * next_max - *q);
    }

    /// Drop any pending transition (game/level boundary).
    pub fn reset_pending(&mut self) {
        self.pending = None;
    }
}

/// Shaped reward from world-model signals.
pub fn shaped_reward(
    level_advanced: bool,
    progress_delta: f32,
    first_visit: bool,
) -> f32 {
    let mut r = -0.05; // step cost: efficiency matters on the leaderboard
    if level_advanced {
        r += 10.0;
    }
    r += progress_delta.clamp(-1.0, 1.0) * 0.5;
    if first_visit {
        r += 0.2;
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q_learns_rewarding_action() {
        let mut q = QTable::default();
        let s = AbstractState::click(5, true);
        let s_next = AbstractState::click(4, true);
        for _ in 0..30 {
            q.act(&s, 6);
            q.observe(shaped_reward(false, 1.0, false), &s_next);
        }
        // Action 6 should now dominate a fresh alternative
        let (best, _, conf) = q.choose(&s, &[1, 6]).unwrap();
        assert_eq!(best, 6);
        assert!(conf > 0.5);
    }

    #[test]
    fn level_advance_dominates_shaping() {
        assert!(shaped_reward(true, -1.0, false) > 5.0);
        assert!(shaped_reward(false, 1.0, true) < 1.0);
    }

    #[test]
    fn abstract_states_bucket() {
        assert_eq!(AbstractState::click(4, true).0, "click|t3-5|m1");
        assert_eq!(AbstractState::movement(-3.0, 0.5, false).0, "move|r-|c0|b0");
    }

    #[test]
    fn table_roundtrips_serde() {
        let mut q = QTable::default();
        let s = AbstractState::click(1, false);
        q.act(&s, 6);
        q.observe(1.0, &s);
        let json = serde_json::to_string(&q).unwrap();
        let q2: QTable = serde_json::from_str(&json).unwrap();
        assert!(q2.states.contains_key(&s.0));
    }
}
