//! Core Data Models for AIModel

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::data::attributes::{Attributes, AttributeAccessor};

/// Overflow risk classification for VCP
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum OverflowRisk {
    #[default]
    Safe,
    Warning,
    Critical,
    Imminent,
}

/// ELP Tensor (DEPRECATED - use Attributes)
#[deprecated(since = "0.3.0", note = "Use Attributes system instead")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct ELPTensor {
    pub ethos: f64,
    pub logos: f64,
    pub pathos: f64,
}

#[allow(deprecated)]
impl ELPTensor {
    pub fn new(ethos: f64, logos: f64, pathos: f64) -> Self {
        Self { ethos, logos, pathos }
    }
    
    pub fn to_attributes(&self) -> Attributes {
        Attributes::with_elp(self.ethos as f32, self.logos as f32, self.pathos as f32)
    }
}

/// Core Flux Matrix pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxMatrix {
    pub id: Uuid,
    pub subject: String,
    pub nodes: HashMap<u8, FluxNode>,
    pub sacred_guides: HashMap<u8, SacredGuide>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual node in the flux matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxNode {
    pub position: u8,
    pub base_value: u8,
    pub attributes: Attributes,
    pub connections: Vec<NodeConnection>,
}

/// Sacred guide nodes (3, 6, 9)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredGuide {
    pub position: u8,
    pub divine_properties: Vec<String>,
    pub geometric_significance: String,
}

/// Connection between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnection {
    pub target_position: u8,
    pub weight: f32,
}

/// BeamTensor: Light-based word representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamTensor {
    pub digits: [f32; 9],
    pub attributes: Attributes,
    pub curviness_signed: f32,
    pub timestamp: f64,
    pub confidence: f32,
    pub word: String,
    pub position: u8,
    pub can_replicate: bool,
    pub mark_for_confidence_lake: bool,
    pub calculation_depth: u64,
    pub overflow_risk: OverflowRisk,
}

pub type BeadTensor = BeamTensor;

impl AttributeAccessor for BeamTensor {
    fn attributes(&self) -> &Attributes { &self.attributes }
    fn attributes_mut(&mut self) -> &mut Attributes { &mut self.attributes }
}

impl Default for BeamTensor {
    fn default() -> Self {
        Self {
            digits: [1.0 / 9.0; 9],
            attributes: Attributes::new(),
            curviness_signed: 0.0,
            timestamp: 0.0,
            confidence: 0.5,
            word: String::new(),
            position: 1,
            can_replicate: false,
            mark_for_confidence_lake: false,
            calculation_depth: 0,
            overflow_risk: OverflowRisk::Safe,
        }
    }
}

impl BeamTensor {
    pub fn dominant_position(&self) -> u8 {
        self.digits.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| (idx + 1) as u8)
            .unwrap_or(1)
    }
    
    pub fn is_diamond_moment(&self) -> bool {
        self.attributes.ethos() >= 8.5 && self.attributes.logos() >= 7.0 && self.curviness_signed < 0.0
    }
}
