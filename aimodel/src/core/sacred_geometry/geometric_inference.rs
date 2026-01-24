//! Geometric Inference Engine

use crate::data::attributes::Attributes;

#[derive(Debug, Clone, Default)]
pub struct GeometricInferenceEngine {
    confidence_threshold: f32,
}

impl GeometricInferenceEngine {
    pub fn new() -> Self { Self { confidence_threshold: 0.6 } }

    pub fn infer_from_position(&self, position: u8) -> Attributes {
        let mut attrs = Attributes::new();
        match position {
            1 | 3 => attrs.set_ethos(0.8),
            4 | 9 => attrs.set_logos(0.8),
            5 | 6 => attrs.set_pathos(0.8),
            _ => {
                attrs.set_ethos(0.33);
                attrs.set_logos(0.33);
                attrs.set_pathos(0.34);
            }
        }
        attrs.set_digital_root_flux(position);
        attrs
    }

    pub fn meets_threshold(&self, confidence: f32) -> bool {
        confidence >= self.confidence_threshold
    }
}
