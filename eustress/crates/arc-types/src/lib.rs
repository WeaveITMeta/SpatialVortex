use serde::{Deserialize, Serialize};

/// One step of ARC-AGI-3 environment interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcStep {
    pub task_id: String,
    pub step: u32,
    /// Grid, goal, available_actions, reward — raw JSON from the ARC environment.
    pub observation: serde_json::Value,
    pub action_taken: Option<String>,
    pub terminated: bool,
    pub score: f32,
}

impl ArcStep {
    /// Extract the available action strings from the observation payload.
    pub fn available_actions(&self) -> Vec<String> {
        self.observation
            .get("available_actions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract available action IDs as u32 (from the ARC environment).
    /// Handles both integer IDs `[1, 2, 3]` and string names `["ACTION1", "ACTION2"]`.
    pub fn available_action_ids(&self) -> Vec<u32> {
        self.observation
            .get("available_actions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        // Integer ID
                        if let Some(n) = v.as_u64() {
                            return Some(n as u32);
                        }
                        // String name like "ACTION1" → extract trailing digits
                        if let Some(s) = v.as_str() {
                            let digits: String = s.chars().rev().take_while(|c| c.is_ascii_digit()).collect::<String>().chars().rev().collect();
                            return digits.parse().ok();
                        }
                        None
                    })
                    .collect()
            })
            .unwrap_or_else(|| vec![1, 2, 3, 4, 5])
    }

    /// Extract the 64×64 rendered game frame as Vec<Vec<u8>>.
    /// The ARC environment sends `frame` as `[layer][row][col]`.
    /// We take layer 0 (the primary render).
    pub fn frame_grid(&self) -> Option<Vec<Vec<u8>>> {
        let frame = self.observation.get("frame")?;
        // frame is [layers][rows][cols] — take layer 0
        let layer0 = frame.as_array()?.first()?;
        let rows: Vec<Vec<u8>> = layer0
            .as_array()?
            .iter()
            .filter_map(|row| {
                row.as_array().map(|cols| {
                    cols.iter()
                        .map(|c| c.as_u64().unwrap_or(0) as u8)
                        .collect()
                })
            })
            .collect();
        if rows.is_empty() {
            None
        } else {
            Some(rows)
        }
    }

    /// Extract game_id from the observation.
    pub fn game_id(&self) -> Option<&str> {
        self.observation.get("game_id").and_then(|v| v.as_str())
    }

    /// Extract levels_completed from the observation.
    pub fn levels_completed(&self) -> u32 {
        self.observation
            .get("levels_completed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32
    }

    /// Extract game state string.
    pub fn state(&self) -> &str {
        self.observation
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("PLAYING")
    }
}

/// Policy decision: given a full episode history, what action to take next.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// Must match ARC action space exactly.
    pub action: String,
    /// Confidence in [0.0, 1.0].
    pub confidence: f32,
    /// Free text for audit trail.
    pub reasoning: String,
}

/// Summary of one complete episode for convergence tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeSummary {
    pub task_id: String,
    pub steps: u32,
    pub goal_reached: bool,
    /// steps / human_baseline
    pub efficiency_ratio: f32,
    pub score: f32,
}
