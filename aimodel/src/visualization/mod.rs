//! Visualization Module
//!
//! Bevy 3D rendering for sacred geometry:
//! - Live flux orbits
//! - Ethical color deflections
//! - Discovery adaptation visualization

#[cfg(feature = "bevy_viz")]
pub mod bevy_3d;

#[cfg(feature = "bevy_viz")]
pub use bevy_3d::*;
