//! Universal Attributes System
//!
//! Flexible key-value attribute storage with ELP compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dynamic attribute value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttributeValue {
    Float(f32),
    Int(i64),
    String(String),
    Bool(bool),
    FloatArray(Vec<f32>),
}

impl AttributeValue {
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            AttributeValue::Float(v) => Some(*v),
            AttributeValue::Int(v) => Some(*v as f32),
            _ => None,
        }
    }
}

/// Tags for categorization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tags(pub Vec<String>);

impl Tags {
    pub fn new() -> Self { Self(Vec::new()) }
    pub fn add(&mut self, tag: &str) { self.0.push(tag.to_string()); }
    pub fn contains(&self, tag: &str) -> bool { self.0.iter().any(|t| t == tag) }
}

/// Universal attributes container with ELP compatibility
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Attributes {
    values: HashMap<String, AttributeValue>,
    tags: Tags,
}

impl Attributes {
    pub fn new() -> Self { Self::default() }

    pub fn with_elp(ethos: f32, logos: f32, pathos: f32) -> Self {
        let mut attrs = Self::new();
        attrs.set_ethos(ethos);
        attrs.set_logos(logos);
        attrs.set_pathos(pathos);
        attrs
    }

    // ELP accessors
    pub fn ethos(&self) -> f32 { self.get_f32("ethos").unwrap_or(0.0) }
    pub fn logos(&self) -> f32 { self.get_f32("logos").unwrap_or(0.0) }
    pub fn pathos(&self) -> f32 { self.get_f32("pathos").unwrap_or(0.0) }

    pub fn set_ethos(&mut self, v: f32) { self.set_f32("ethos", v); }
    pub fn set_logos(&mut self, v: f32) { self.set_f32("logos", v); }
    pub fn set_pathos(&mut self, v: f32) { self.set_f32("pathos", v); }

    pub fn elp_tensor(&self) -> [f32; 3] {
        [self.ethos(), self.logos(), self.pathos()]
    }

    pub fn set_elp_tensor(&mut self, elp: [f32; 3]) {
        self.set_ethos(elp[0]);
        self.set_logos(elp[1]);
        self.set_pathos(elp[2]);
    }

    pub fn set_digital_root_flux(&mut self, position: u8) {
        self.set_f32("digital_root_flux", position as f32);
    }

    // Generic accessors
    pub fn get(&self, key: &str) -> Option<&AttributeValue> { self.values.get(key) }
    pub fn get_f32(&self, key: &str) -> Option<f32> { self.get(key).and_then(|v| v.as_f32()) }
    pub fn set_f32(&mut self, key: &str, value: f32) {
        self.values.insert(key.to_string(), AttributeValue::Float(value));
    }

    pub fn tags(&self) -> &Tags { &self.tags }
    pub fn tags_mut(&mut self) -> &mut Tags { &mut self.tags }
}

/// Trait for types that have attributes
pub trait AttributeAccessor {
    fn attributes(&self) -> &Attributes;
    fn attributes_mut(&mut self) -> &mut Attributes;

    fn ethos(&self) -> f32 { self.attributes().ethos() }
    fn logos(&self) -> f32 { self.attributes().logos() }
    fn pathos(&self) -> f32 { self.attributes().pathos() }
}
