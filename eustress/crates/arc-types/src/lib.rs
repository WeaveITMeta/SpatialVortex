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
