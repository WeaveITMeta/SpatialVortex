//! # 008 | Vortex Spatial
//!
//! Geometric reasoning in 2D, 3D and time.
//!
//! - [`plan`]: shortest collision-free route for a robot of given width
//!   among rectangular obstacles (configuration-space expansion, visibility
//!   graph, Dijkstra). Exact, not sampled.
//! - [`solve_challenge`]: the full Astra Spatial Reasoning Lab task
//!   (navigation, portal clearance, assembly placement).
//! - [`kinematics`]: closed-form answers about bodies moving under constant
//!   acceleration.
//!
//! These are engineered solvers. The harness labels them so; they are what
//! the learned components are later measured against and distilled from.

pub mod kinematics;
pub mod plan;

use std::collections::BTreeMap;

use vortex_kernel::{SpatialChallenge, SpatialSubmission};

/// Solve all three Astra sub-tasks. Confidence is 0.95 when a route was
/// found within the length limit, 0.3 otherwise (the submission is then
/// missing its navigation component and the agent says so).
pub fn solve_challenge(ch: &SpatialChallenge) -> (SpatialSubmission, f64) {
    let route = plan::plan(&ch.navigation);
    let passable = passable_portals(&ch.clearance);
    let assembly: BTreeMap<String, [f64; 3]> = ch.assembly.targets.clone();
    let ok = route.as_ref().is_some_and(|r| plan::length(r) <= ch.navigation.max_route_length);
    let confidence = if ok { 0.95 } else { 0.3 };
    (
        SpatialSubmission { route: route.unwrap_or_default(), passable_portals: passable, assembly_positions: assembly },
        confidence,
    )
}

/// 1-based ids of portals wide enough for the robot plus margin on both sides.
pub fn passable_portals(c: &vortex_kernel::spatial::Clearance) -> Vec<u32> {
    let need = c.robot_width + 2.0 * c.margin_each_side;
    c.portal_gaps.iter().enumerate().filter(|(_, g)| **g + 1e-9 >= need).map(|(i, _)| i as u32 + 1).collect()
}
