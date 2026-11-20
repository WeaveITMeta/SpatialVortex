//! The Astra / Spatial Reasoning Lab challenge format
//! (`Documents/Eustress/Astra/Spaces/Spatial Reasoning Lab/DataService/challenge.json`).
//! Field names match that file exactly so the real challenge deserialises
//! directly.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Obstacle {
    pub name: String,
    /// `[x_min, z_min, x_max, z_max]` on the XZ plane.
    pub rect: [f64; 4],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Navigation {
    pub start: [f64; 2],
    pub goal: [f64; 2],
    pub robot_width: f64,
    pub safety_margin: f64,
    pub max_route_length: f64,
    /// `[x_min, z_min, x_max, z_max]`.
    pub bounds: [f64; 4],
    pub obstacles: Vec<Obstacle>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clearance {
    pub portal_gaps: Vec<f64>,
    pub robot_width: f64,
    pub margin_each_side: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Assembly {
    pub targets: BTreeMap<String, [f64; 3]>,
    pub position_tolerance: f64,
    #[serde(default)]
    pub rotation_tolerance: f64,
    #[serde(default)]
    pub sizes: BTreeMap<String, [f64; 3]>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpatialChallenge {
    pub navigation: Navigation,
    pub clearance: Clearance,
    pub assembly: Assembly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct SpatialSubmission {
    pub route: Vec<[f64; 2]>,
    /// 1-based portal ids.
    pub passable_portals: Vec<u32>,
    pub assembly_positions: BTreeMap<String, [f64; 3]>,
}
