//! 3D-plus-time reasoning queries: bodies with position, velocity and
//! constant acceleration, asked about at a point in time or over an interval.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Body {
    pub name: String,
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    #[serde(default)]
    pub acceleration: [f64; 3],
}

impl Body {
    /// Position at time `t` under constant acceleration.
    pub fn at(&self, t: f64) -> [f64; 3] {
        let mut p = [0.0; 3];
        for i in 0..3 {
            p[i] = self.position[i] + self.velocity[i] * t + 0.5 * self.acceleration[i] * t * t;
        }
        p
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum KinematicsQuery {
    /// Where is `body` at time `t`? Answer: `[x, y, z]`.
    PositionAt { body: Body, t: f64 },
    /// Minimum separation of two constant-velocity bodies over `[0, horizon]`.
    /// Answer: `[t_min, distance]`.
    ClosestApproach { a: Body, b: Body, horizon: f64 },
    /// When does a body under gravity along -Y first reach height `y`
    /// (descending)? Answer: `[t]`.
    TimeToHeight { body: Body, y: f64 },
}
