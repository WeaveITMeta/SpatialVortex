//! Bridge: ArcSceneDelta (serde, arc-policy) → SceneDelta (rkyv, eustress-common)
//!
//! ArcSceneDelta uses serde JSON — no rkyv or Bevy dependency.
//! SceneDelta uses rkyv zero-copy serialization for Iggy wire format.
//! This module converts between them for Iggy publishing.

#[cfg(feature = "iggy-streaming")]
pub mod iggy_bridge {
    use eustress_arc_policy::scene_mirror::{ArcDeltaKind, ArcSceneDelta};
    use eustress_common::iggy_delta::{
        DeltaKind, NamePayload, PartPayload, SceneDelta, TransformPayload,
    };

    /// Convert a batch of ArcSceneDeltas to Iggy-compatible SceneDeltas.
    pub fn convert_deltas(deltas: &[ArcSceneDelta]) -> Vec<SceneDelta> {
        let ts = now_ms();
        deltas.iter().map(|d| convert_one(d, ts)).collect()
    }

    fn convert_one(arc: &ArcSceneDelta, timestamp_ms: u64) -> SceneDelta {
        let kind = match arc.kind {
            ArcDeltaKind::PartAdded => DeltaKind::PartAdded,
            ArcDeltaKind::PartRemoved => DeltaKind::PartRemoved,
            ArcDeltaKind::TransformChanged => DeltaKind::TransformChanged,
            ArcDeltaKind::PropertiesChanged => DeltaKind::PartPropertiesChanged,
            ArcDeltaKind::Renamed => DeltaKind::Renamed,
        };

        let transform = arc.position.map(|pos| TransformPayload {
            position: pos,
            rotation: arc.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]),
            scale: arc.scale.unwrap_or([1.0, 1.0, 1.0]),
        });

        let part = if arc.color.is_some() || arc.anchored.is_some() {
            Some(PartPayload {
                color: arc.color,
                material: None,
                size: None,
                name: None,
                anchored: arc.anchored,
                can_collide: None,
                transparency: None,
                reflectance: None,
            })
        } else {
            None
        };

        let name = arc.name.as_ref().map(|n| NamePayload { name: n.clone() });

        SceneDelta {
            entity: arc.entity,
            kind,
            seq: arc.seq,
            timestamp_ms,
            transform,
            part,
            name,
            new_parent: None,
        }
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}
