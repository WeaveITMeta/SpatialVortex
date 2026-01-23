//! # Universal Attributes System
//!
//! Unified attribute system compatible with EustressEngine's Roblox-style Attributes.
//! Replaces the legacy ELP (Ethos, Logos, Pathos) system with a more flexible
//! key-value attribute approach while maintaining backward compatibility.
//!
//! ## Table of Contents
//! 1. AttributeValue - Dynamic value types (String, Number, Bool, Vector3, Color, etc.)
//! 2. Attributes - Key-value storage container
//! 3. Tags - CollectionService-style entity tagging
//! 4. ELP Compatibility - Backward compatible ELP tensor access
//! 5. Sacred Geometry Integration - 3-6-9 pattern support

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ============================================================================
// 1. Traits - Property-Based Attribute Access
// ============================================================================

/// Trait for accessing attributes in a property-based manner
/// Similar to EustressEngine's Instance:GetAttribute/SetAttribute pattern
/// 
/// This trait provides generic attribute access. ELP-specific methods are
/// provided as convenience helpers that delegate to the generic methods.
pub trait AttributeAccessor {
    /// Get the underlying Attributes container
    fn attributes(&self) -> &Attributes;
    
    /// Get mutable access to the underlying Attributes container
    fn attributes_mut(&mut self) -> &mut Attributes;
    
    // Generic attribute accessors
    
    /// Get an attribute value by key
    fn get_attribute(&self, key: &str) -> Option<&AttributeValue> {
        self.attributes().get(key)
    }
    
    /// Set an attribute value
    fn set_attribute(&mut self, key: impl Into<String>, value: AttributeValue) {
        self.attributes_mut().set(key, value);
    }
    
    /// Check if an attribute exists
    fn has_attribute(&self, key: &str) -> bool {
        self.attributes().has(key)
    }
    
    /// Remove an attribute
    fn remove_attribute(&mut self, key: &str) -> Option<AttributeValue> {
        self.attributes_mut().remove(key)
    }
    
    /// Get attribute as f32 number
    fn get_f32(&self, key: &str) -> Option<f32> {
        self.attributes().get_f32(key)
    }
    
    /// Get attribute as f64 number
    fn get_number(&self, key: &str) -> Option<f64> {
        self.attributes().get_number(key)
    }
    
    /// Get attribute as string
    fn get_string(&self, key: &str) -> Option<&str> {
        self.attributes().get_string(key)
    }
    
    /// Get attribute as bool
    fn get_bool(&self, key: &str) -> Option<bool> {
        self.attributes().get_bool(key)
    }
    
    /// Get attribute as Vector3
    fn get_vector3(&self, key: &str) -> Option<[f32; 3]> {
        self.attributes().get_vector3(key)
    }
    
    /// Get attribute as Color
    fn get_color(&self, key: &str) -> Option<[f32; 4]> {
        self.attributes().get_color(key)
    }
    
    /// Get attribute as u32
    fn get_u32(&self, key: &str) -> Option<u32> {
        self.attributes().get_u32(key)
    }
}

// Implement AttributeAccessor for Attributes itself (identity implementation)
impl AttributeAccessor for Attributes {
    fn attributes(&self) -> &Attributes {
        self
    }
    
    fn attributes_mut(&mut self) -> &mut Attributes {
        self
    }
}

// ============================================================================
// 2. AttributeValue - Dynamic Value Types
// ============================================================================

/// Dynamic attribute value types (compatible with EustressEngine)
/// Supports all common data types for entity metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttributeValue {
    /// String value
    String(String),
    /// 64-bit floating point number
    Number(f64),
    /// Boolean value
    Bool(bool),
    /// 3D vector [x, y, z]
    Vector3([f32; 3]),
    /// 2D vector [x, y]
    Vector2([f32; 2]),
    /// RGBA color [r, g, b, a]
    Color([f32; 4]),
    /// Integer value
    Int(i64),
    /// Reference to another entity by ID
    EntityRef(u32),
    /// Transform/CFrame as position + rotation quaternion
    Transform {
        position: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    },
    /// NumberRange for constraints
    NumberRange { min: f64, max: f64 },
    /// NumberSequence for gradients/animations
    NumberSequence(Vec<NumberSequenceKeypoint>),
    /// ColorSequence for color gradients
    ColorSequence(Vec<ColorSequenceKeypoint>),
    /// Tensor data (for ML/AI)
    Tensor(Vec<f32>),
}

impl Default for AttributeValue {
    fn default() -> Self {
        AttributeValue::String(String::new())
    }
}

impl AttributeValue {
    /// Get the type name as a string for display
    pub fn type_name(&self) -> &'static str {
        match self {
            AttributeValue::String(_) => "String",
            AttributeValue::Number(_) => "Number",
            AttributeValue::Bool(_) => "Bool",
            AttributeValue::Vector3(_) => "Vector3",
            AttributeValue::Vector2(_) => "Vector2",
            AttributeValue::Color(_) => "Color",
            AttributeValue::Int(_) => "Int",
            AttributeValue::EntityRef(_) => "EntityRef",
            AttributeValue::Transform { .. } => "Transform",
            AttributeValue::NumberRange { .. } => "NumberRange",
            AttributeValue::NumberSequence(_) => "NumberSequence",
            AttributeValue::ColorSequence(_) => "ColorSequence",
            AttributeValue::Tensor(_) => "Tensor",
        }
    }
    
    /// Create a default value for a given type name
    pub fn default_for_type(type_name: &str) -> Option<Self> {
        match type_name {
            "String" => Some(AttributeValue::String(String::new())),
            "Number" => Some(AttributeValue::Number(0.0)),
            "Bool" => Some(AttributeValue::Bool(false)),
            "Vector3" => Some(AttributeValue::Vector3([0.0, 0.0, 0.0])),
            "Vector2" => Some(AttributeValue::Vector2([0.0, 0.0])),
            "Color" => Some(AttributeValue::Color([1.0, 1.0, 1.0, 1.0])),
            "Int" => Some(AttributeValue::Int(0)),
            "EntityRef" => Some(AttributeValue::EntityRef(0)),
            "Transform" => Some(AttributeValue::Transform {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            }),
            "NumberRange" => Some(AttributeValue::NumberRange { min: 0.0, max: 1.0 }),
            "Tensor" => Some(AttributeValue::Tensor(Vec::new())),
            _ => None,
        }
    }
    
    /// List all available type names
    pub fn available_types() -> &'static [&'static str] {
        &[
            "String", "Number", "Bool", "Vector3", "Vector2", "Color", "Int",
            "EntityRef", "Transform", "NumberRange", "NumberSequence", 
            "ColorSequence", "Tensor"
        ]
    }
    
    /// Get a display-friendly string representation of the value
    pub fn display_value(&self) -> String {
        match self {
            AttributeValue::String(s) => format!("\"{}\"", s),
            AttributeValue::Number(n) => format!("{:.4}", n),
            AttributeValue::Bool(b) => b.to_string(),
            AttributeValue::Vector3(v) => format!("({:.2}, {:.2}, {:.2})", v[0], v[1], v[2]),
            AttributeValue::Vector2(v) => format!("({:.2}, {:.2})", v[0], v[1]),
            AttributeValue::Color(c) => format!("#{:02X}{:02X}{:02X}{:02X}", 
                (c[0] * 255.0) as u8, (c[1] * 255.0) as u8, 
                (c[2] * 255.0) as u8, (c[3] * 255.0) as u8),
            AttributeValue::Int(i) => i.to_string(),
            AttributeValue::EntityRef(id) => format!("Entity({})", id),
            AttributeValue::Transform { position, .. } => 
                format!("Transform({:.1}, {:.1}, {:.1})", position[0], position[1], position[2]),
            AttributeValue::NumberRange { min, max } => format!("{:.2} - {:.2}", min, max),
            AttributeValue::NumberSequence(keypoints) => format!("Seq[{}]", keypoints.len()),
            AttributeValue::ColorSequence(keypoints) => format!("ColorSeq[{}]", keypoints.len()),
            AttributeValue::Tensor(data) => format!("Tensor[{}]", data.len()),
        }
    }
    
    /// Try to get as f64
    pub fn as_number(&self) -> Option<f64> {
        match self {
            AttributeValue::Number(n) => Some(*n),
            AttributeValue::Int(i) => Some(*i as f64),
            _ => None,
        }
    }
    
    /// Try to get as f32
    pub fn as_f32(&self) -> Option<f32> {
        self.as_number().map(|n| n as f32)
    }
    
    /// Try to get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            AttributeValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// Try to get as bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AttributeValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    
    /// Try to get as Vector3
    pub fn as_vector3(&self) -> Option<[f32; 3]> {
        match self {
            AttributeValue::Vector3(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Try to get as Color
    pub fn as_color(&self) -> Option<[f32; 4]> {
        match self {
            AttributeValue::Color(c) => Some(*c),
            _ => None,
        }
    }
}

/// Keypoint for NumberSequence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumberSequenceKeypoint {
    pub time: f32,      // 0.0 to 1.0
    pub value: f64,
    pub envelope: f32,  // Variance
}

/// Keypoint for ColorSequence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorSequenceKeypoint {
    pub time: f32,      // 0.0 to 1.0
    pub color: [f32; 4],
}

// ============================================================================
// 2. Attributes - Key-Value Storage
// ============================================================================

/// Attributes container for storing dynamic key-value pairs
/// Compatible with EustressEngine's Instance:SetAttribute/GetAttribute system
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Attributes {
    /// Key-value storage for attributes
    values: HashMap<String, AttributeValue>,
}

impl Attributes {
    /// Create a new empty Attributes container
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
    
    /// Create with a single attribute
    pub fn with_attribute(key: impl Into<String>, value: AttributeValue) -> Self {
        let mut attrs = Self::new();
        attrs.set(key, value);
        attrs
    }
    
    /// Create with ELP-style balance (backward compatibility)
    /// Creates "ethos", "logos", "pathos" attributes
    pub fn with_elp(ethos: f32, logos: f32, pathos: f32) -> Self {
        let mut attrs = Self::new();
        attrs.set("ethos", AttributeValue::Number(ethos as f64));
        attrs.set("logos", AttributeValue::Number(logos as f64));
        attrs.set("pathos", AttributeValue::Number(pathos as f64));
        attrs
    }
    
    /// Create from ELP tensor [ethos, logos, pathos]
    pub fn from_elp_tensor(elp: [f32; 3]) -> Self {
        Self::with_elp(elp[0], elp[1], elp[2])
    }
    
    /// Set an attribute value
    pub fn set(&mut self, key: impl Into<String>, value: AttributeValue) {
        self.values.insert(key.into(), value);
    }
    
    /// Get an attribute value
    pub fn get(&self, key: &str) -> Option<&AttributeValue> {
        self.values.get(key)
    }
    
    /// Get a mutable reference to an attribute value
    pub fn get_mut(&mut self, key: &str) -> Option<&mut AttributeValue> {
        self.values.get_mut(key)
    }
    
    /// Remove an attribute
    pub fn remove(&mut self, key: &str) -> Option<AttributeValue> {
        self.values.remove(key)
    }
    
    /// Check if an attribute exists
    pub fn has(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    /// Get all attribute keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
    
    /// Get all attributes as an iterator
    pub fn iter(&self) -> impl Iterator<Item = (&String, &AttributeValue)> {
        self.values.iter()
    }
    
    /// Get mutable iterator over all attributes
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut AttributeValue)> {
        self.values.iter_mut()
    }
    
    /// Get the number of attributes
    pub fn len(&self) -> usize {
        self.values.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    
    /// Clear all attributes
    pub fn clear(&mut self) {
        self.values.clear();
    }
    
    /// Merge another Attributes into this one (other values override)
    pub fn merge(&mut self, other: &Attributes) {
        for (key, value) in other.iter() {
            self.set(key.clone(), value.clone());
        }
    }
    
    // ========================================================================
    // Convenience getters with type checking
    // ========================================================================
    
    /// Get as string
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_string())
    }
    
    /// Get as number (f64)
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_number())
    }
    
    /// Get as f32
    pub fn get_f32(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(|v| v.as_f32())
    }
    
    /// Get as bool
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }
    
    /// Get as Vector3
    pub fn get_vector3(&self, key: &str) -> Option<[f32; 3]> {
        self.get(key).and_then(|v| v.as_vector3())
    }
    
    /// Get as Color
    pub fn get_color(&self, key: &str) -> Option<[f32; 4]> {
        self.get(key).and_then(|v| v.as_color())
    }
    
    /// Get as u32
    pub fn get_u32(&self, key: &str) -> Option<u32> {
        self.get_number(key).map(|n| n as u32)
    }
    
    // ========================================================================
    // ELP (Ethos, Logos, Pathos) Compatibility Methods
    // ========================================================================
    
    /// Get ethos value (backward compatibility)
    pub fn ethos(&self) -> f32 {
        self.get_f32("ethos").unwrap_or(0.0)
    }
    
    /// Get logos value (backward compatibility)
    pub fn logos(&self) -> f32 {
        self.get_f32("logos").unwrap_or(0.0)
    }
    
    /// Get pathos value (backward compatibility)
    pub fn pathos(&self) -> f32 {
        self.get_f32("pathos").unwrap_or(0.0)
    }
    
    /// Set ethos value
    pub fn set_ethos(&mut self, value: f32) {
        self.set("ethos", AttributeValue::Number(value as f64));
    }
    
    /// Set logos value
    pub fn set_logos(&mut self, value: f32) {
        self.set("logos", AttributeValue::Number(value as f64));
    }
    
    /// Set pathos value
    pub fn set_pathos(&mut self, value: f32) {
        self.set("pathos", AttributeValue::Number(value as f64));
    }
    
    /// Get confidence value
    pub fn confidence(&self) -> f32 {
        self.get_f32("confidence").unwrap_or(0.0)
    }
    
    /// Set confidence value
    pub fn set_confidence(&mut self, value: f32) {
        self.set("confidence", AttributeValue::Number(value as f64));
    }
    
    /// Get ELP as tensor [ethos, logos, pathos]
    pub fn elp_tensor(&self) -> [f32; 3] {
        [self.ethos(), self.logos(), self.pathos()]
    }
    
    /// Get ELP normalized to sum to 1.0
    pub fn elp_normalized(&self) -> [f32; 3] {
        let e = self.ethos();
        let l = self.logos();
        let p = self.pathos();
        let sum = e + l + p;
        if sum > 0.0 {
            [e / sum, l / sum, p / sum]
        } else {
            [0.333, 0.333, 0.334]
        }
    }
    
    /// Set digital root flux position
    pub fn set_digital_root_flux(&mut self, position: u8) {
        self.set("digital_root_flux", AttributeValue::Int(position as i64));
    }
    
    /// Get digital root flux position
    pub fn digital_root_flux(&self) -> Option<u8> {
        self.get_number("digital_root_flux").map(|n| n as u8)
    }
    
    /// Check if at a sacred position (3, 6, or 9)
    pub fn is_sacred_position(&self) -> bool {
        matches!(self.digital_root_flux(), Some(3) | Some(6) | Some(9))
    }
    
    /// Set ELP tensor [ethos, logos, pathos]
    pub fn set_elp_tensor(&mut self, elp: [f32; 3]) {
        self.set_ethos(elp[0]);
        self.set_logos(elp[1]);
        self.set_pathos(elp[2]);
    }
}

// ============================================================================
// 3. Tags - CollectionService-style Tagging
// ============================================================================

/// Tags container for CollectionService-style entity grouping
/// Entities can have multiple tags for categorization and querying
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tags {
    /// Set of tags applied to this entity
    tags: HashSet<String>,
}

impl Tags {
    /// Create a new empty Tags container
    pub fn new() -> Self {
        Self {
            tags: HashSet::new(),
        }
    }
    
    /// Create Tags with initial tags
    pub fn with_tags(tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            tags: tags.into_iter().map(|t| t.into()).collect(),
        }
    }
    
    /// Create Tags with a single tag
    pub fn with_tag(tag: impl Into<String>) -> Self {
        let mut tags = Self::new();
        tags.add(tag);
        tags
    }
    
    /// Add a tag
    pub fn add(&mut self, tag: impl Into<String>) -> bool {
        self.tags.insert(tag.into())
    }
    
    /// Remove a tag
    pub fn remove(&mut self, tag: &str) -> bool {
        self.tags.remove(tag)
    }
    
    /// Check if entity has a tag
    pub fn has(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }
    
    /// Get all tags
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.tags.iter()
    }
    
    /// Get the number of tags
    pub fn len(&self) -> usize {
        self.tags.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }
    
    /// Clear all tags
    pub fn clear(&mut self) {
        self.tags.clear();
    }
    
    /// Get tags as a sorted vector (for consistent display)
    pub fn sorted(&self) -> Vec<&String> {
        let mut tags: Vec<_> = self.tags.iter().collect();
        tags.sort();
        tags
    }
    
    /// Convert to Vec<String>
    pub fn to_vec(&self) -> Vec<String> {
        self.tags.iter().cloned().collect()
    }
}

// ============================================================================
// 4. AttributeValueJson - JSON Transport Format
// ============================================================================

/// JSON-serializable representation of AttributeValue for API transport
/// Compatible with spatial-llm types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum AttributeValueJson {
    String(String),
    Number(f64),
    Bool(bool),
    Vector3([f32; 3]),
    Vector2([f32; 2]),
    Color([f32; 4]),
    Int(i64),
    EntityRef(u32),
}

impl From<&AttributeValue> for AttributeValueJson {
    fn from(value: &AttributeValue) -> Self {
        match value {
            AttributeValue::String(s) => AttributeValueJson::String(s.clone()),
            AttributeValue::Number(n) => AttributeValueJson::Number(*n),
            AttributeValue::Bool(b) => AttributeValueJson::Bool(*b),
            AttributeValue::Vector3(v) => AttributeValueJson::Vector3(*v),
            AttributeValue::Vector2(v) => AttributeValueJson::Vector2(*v),
            AttributeValue::Color(c) => AttributeValueJson::Color(*c),
            AttributeValue::Int(i) => AttributeValueJson::Int(*i),
            AttributeValue::EntityRef(id) => AttributeValueJson::EntityRef(*id),
            // Complex types convert to their primary value
            AttributeValue::Transform { position, .. } => AttributeValueJson::Vector3(*position),
            AttributeValue::NumberRange { min, .. } => AttributeValueJson::Number(*min),
            AttributeValue::NumberSequence(seq) => {
                AttributeValueJson::Number(seq.first().map(|k| k.value).unwrap_or(0.0))
            }
            AttributeValue::ColorSequence(seq) => {
                AttributeValueJson::Color(seq.first().map(|k| k.color).unwrap_or([1.0, 1.0, 1.0, 1.0]))
            }
            AttributeValue::Tensor(data) => {
                AttributeValueJson::Number(data.first().map(|v| *v as f64).unwrap_or(0.0))
            }
        }
    }
}

impl From<AttributeValueJson> for AttributeValue {
    fn from(value: AttributeValueJson) -> Self {
        match value {
            AttributeValueJson::String(s) => AttributeValue::String(s),
            AttributeValueJson::Number(n) => AttributeValue::Number(n),
            AttributeValueJson::Bool(b) => AttributeValue::Bool(b),
            AttributeValueJson::Vector3(v) => AttributeValue::Vector3(v),
            AttributeValueJson::Vector2(v) => AttributeValue::Vector2(v),
            AttributeValueJson::Color(c) => AttributeValue::Color(c),
            AttributeValueJson::Int(i) => AttributeValue::Int(i),
            AttributeValueJson::EntityRef(id) => AttributeValue::EntityRef(id),
        }
    }
}

// ============================================================================
// 5. Conversion Utilities
// ============================================================================

impl Attributes {
    /// Convert to JSON-compatible HashMap
    pub fn to_json_map(&self) -> HashMap<String, AttributeValueJson> {
        self.values.iter()
            .map(|(k, v)| (k.clone(), AttributeValueJson::from(v)))
            .collect()
    }
    
    /// Create from JSON-compatible HashMap
    pub fn from_json_map(map: HashMap<String, AttributeValueJson>) -> Self {
        let mut attrs = Self::new();
        for (k, v) in map {
            attrs.set(k, AttributeValue::from(v));
        }
        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_attributes_basic() {
        let mut attrs = Attributes::new();
        attrs.set("health", AttributeValue::Number(100.0));
        attrs.set("name", AttributeValue::String("Player".to_string()));
        attrs.set("active", AttributeValue::Bool(true));
        
        assert_eq!(attrs.get_number("health"), Some(100.0));
        assert_eq!(attrs.get_string("name"), Some("Player"));
        assert_eq!(attrs.get_bool("active"), Some(true));
        assert_eq!(attrs.len(), 3);
    }
    
    #[test]
    fn test_elp_compatibility() {
        let attrs = Attributes::with_elp(0.5, 0.3, 0.2);
        
        assert!((attrs.ethos() - 0.5).abs() < 0.001);
        assert!((attrs.logos() - 0.3).abs() < 0.001);
        assert!((attrs.pathos() - 0.2).abs() < 0.001);
        
        let tensor = attrs.elp_tensor();
        assert!((tensor[0] - 0.5).abs() < 0.001);
        assert!((tensor[1] - 0.3).abs() < 0.001);
        assert!((tensor[2] - 0.2).abs() < 0.001);
    }
    
    #[test]
    fn test_elp_normalized() {
        let attrs = Attributes::with_elp(0.6, 0.3, 0.1);
        let normalized = attrs.elp_normalized();
        
        let sum = normalized[0] + normalized[1] + normalized[2];
        assert!((sum - 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_tags() {
        let mut tags = Tags::new();
        tags.add("enemy");
        tags.add("boss");
        tags.add("enemy"); // Duplicate
        
        assert!(tags.has("enemy"));
        assert!(tags.has("boss"));
        assert_eq!(tags.len(), 2);
        
        tags.remove("enemy");
        assert!(!tags.has("enemy"));
    }
    
    #[test]
    fn test_sacred_position() {
        let mut attrs = Attributes::new();
        attrs.set_digital_root_flux(3);
        assert!(attrs.is_sacred_position());
        
        attrs.set_digital_root_flux(6);
        assert!(attrs.is_sacred_position());
        
        attrs.set_digital_root_flux(9);
        assert!(attrs.is_sacred_position());
        
        attrs.set_digital_root_flux(5);
        assert!(!attrs.is_sacred_position());
    }
    
    #[test]
    fn test_json_conversion() {
        let mut attrs = Attributes::new();
        attrs.set("value", AttributeValue::Number(42.0));
        attrs.set("name", AttributeValue::String("test".to_string()));
        
        let json_map = attrs.to_json_map();
        let restored = Attributes::from_json_map(json_map);
        
        assert_eq!(restored.get_number("value"), Some(42.0));
        assert_eq!(restored.get_string("name"), Some("test"));
    }
}
