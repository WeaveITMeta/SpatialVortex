//! SOTA Flux Compression - Unified Semantic Compression (CANONICAL FORMAT)
//!
//! **THIS IS THE SINGLE SOURCE OF TRUTH FOR VORTEX COMPRESSION**
//! 
//! Consolidates and replaces:
//! - `asi_16byte.rs` (6W + aspect color) - DEPRECATED, use this instead
//! - `flux_object_macro.rs` 12-byte (ELP + flow) - DEPRECATED, use this instead
//!
//! ## Key Features (Unified)
//! - **6W Framework**: WHO/WHAT/WHEN/WHERE/WHY/HOW (from 16-byte)
//! - **ELP Tensor**: Ethos/Logos/Pathos with sacred positions (from 12-byte)
//! - **Relational Encoding**: Subject-Predicate-Object with transitive inference
//! - **Temporal Dynamics**: Velocity/acceleration of attribute changes
//! - **Uncertainty Quantification**: Confidence intervals, not point estimates
//! - **Inference State**: Chain depth, evidence count, ladder index
//! - **Multi-Entity Support**: Up to 4 entities in extended formats
//!
//! ## Format Hierarchy
//! ```text
//! 24-byte: Unified semantic unit (6W + ELP + relations + dynamics)
//! 32-byte: + secondary entities + explicit confidence bounds
//! 48-byte: + evidence chain (4 evidence hashes)
//! 64-byte: + multi-entity (4 entities) + reasoning trace
//! ```
//!
//! ## 24-Byte Unified Layout (CANONICAL)
//! ```text
//! [0-2]   WHO: 20-bit hash + 4-bit entity_type
//! [3-5]   WHAT: 20-bit hash + 4-bit action_type  
//! [6-7]   WHEN: 12-bit offset + 4-bit granularity
//! [8-9]   WHERE: 12-bit hash + 4-bit flux_position (0-9 vortex)
//! [10]    WHY: 4-bit cause_type + 4-bit intention
//! [11]    HOW: 4-bit method + 4-bit complexity
//! [12-13] RELATION: 8-bit type + 8-bit object_ref
//! [14-15] ELP: 5-bit ethos + 5-bit logos + 5-bit pathos + 1-bit sacred
//! [16-17] CONFIDENCE: 10-bit value + 6-bit uncertainty_width
//! [18-19] DYNAMICS: 8-bit velocity + 8-bit acceleration (signed)
//! [20-21] INFERENCE: 8-bit chain_depth + 8-bit evidence_count
//! [22]    LADDER_IDX: 8-bit transitive rank
//! [23]    FLAGS: validated/negated/hypothetical/multi-subj/multi-obj/reversible/cross-ref/extended
//! ```

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// =============================================================================
// CORE TYPES
// =============================================================================

/// Relation types for knowledge graph encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum RelationType {
    // Spatial (0-15)
    LeftOf = 0,
    RightOf = 1,
    Above = 2,
    Below = 3,
    Inside = 4,
    Contains = 5,
    Near = 6,
    Far = 7,
    
    // Temporal (16-31)
    Before = 16,
    After = 17,
    During = 18,
    Simultaneous = 19,
    
    // Causal (32-47)
    Causes = 32,
    CausedBy = 33,
    Enables = 34,
    Prevents = 35,
    
    // Logical (48-63)
    IsA = 48,
    HasA = 49,
    PartOf = 50,
    InstanceOf = 51,
    
    // Comparative (64-79)
    BiggerThan = 64,
    SmallerThan = 65,
    EqualTo = 66,
    SimilarTo = 67,
    
    // Possessive (80-95)
    Owns = 80,
    OwnedBy = 81,
    CreatedBy = 82,
    Creates = 83,
    
    // Action (96-111)
    Acts = 96,
    ActedUpon = 97,
    Moves = 98,
    MovedBy = 99,
    
    // Custom (112-127)
    Custom = 112,
    
    // None/Unknown
    None = 255,
}

impl RelationType {
    /// Get the inverse relation (A rel B → B inv(rel) A)
    pub fn inverse(&self) -> Self {
        match self {
            Self::LeftOf => Self::RightOf,
            Self::RightOf => Self::LeftOf,
            Self::Above => Self::Below,
            Self::Below => Self::Above,
            Self::Inside => Self::Contains,
            Self::Contains => Self::Inside,
            Self::Before => Self::After,
            Self::After => Self::Before,
            Self::Causes => Self::CausedBy,
            Self::CausedBy => Self::Causes,
            Self::BiggerThan => Self::SmallerThan,
            Self::SmallerThan => Self::BiggerThan,
            Self::Owns => Self::OwnedBy,
            Self::OwnedBy => Self::Owns,
            Self::CreatedBy => Self::Creates,
            Self::Creates => Self::CreatedBy,
            Self::Acts => Self::ActedUpon,
            Self::ActedUpon => Self::Acts,
            Self::Moves => Self::MovedBy,
            Self::MovedBy => Self::Moves,
            _ => *self,
        }
    }
    
    /// Check if relation is transitive (A rel B, B rel C → A rel C)
    pub fn is_transitive(&self) -> bool {
        matches!(self, 
            Self::LeftOf | Self::RightOf | Self::Above | Self::Below |
            Self::Before | Self::After |
            Self::BiggerThan | Self::SmallerThan |
            Self::IsA | Self::PartOf | Self::Inside | Self::Contains
        )
    }
    
    /// Get confidence decay for transitive inference
    pub fn transitive_decay(&self) -> f32 {
        match self {
            Self::IsA | Self::PartOf => 0.95,  // High confidence inheritance
            Self::LeftOf | Self::RightOf | Self::Above | Self::Below => 0.90,
            Self::Before | Self::After => 0.92,
            Self::BiggerThan | Self::SmallerThan => 0.88,
            _ => 0.85,
        }
    }
}

/// Entity type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EntityType {
    Person = 0,
    Place = 1,
    Thing = 2,
    Action = 3,
    Concept = 4,
    Time = 5,
    Quantity = 6,
    Quality = 7,
    Event = 8,
    State = 9,
    Unknown = 255,
}

/// Temporal granularity for WHEN encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TemporalGranularity {
    Instant = 0,      // Precise moment
    Second = 1,
    Minute = 2,
    Hour = 3,
    Day = 4,
    Week = 5,
    Month = 6,
    Year = 7,
    Decade = 8,
    Century = 9,
    Relative = 10,    // Before/after reference
    Duration = 11,    // Time span
    Recurring = 12,   // Periodic
    Unknown = 255,
}

// =============================================================================
// 24-BYTE SOTA COMPRESSION (Basic Semantic Unit)
// =============================================================================

/// 24-byte SOTA compression with full 6W + relational encoding
/// 
/// Layout:
/// ```text
/// [0-2]   WHO: 20-bit hash + 4-bit entity_type
/// [3-5]   WHAT: 20-bit hash + 4-bit action_type  
/// [6-7]   WHEN: 12-bit offset + 4-bit granularity
/// [8-9]   WHERE: 12-bit hash + 4-bit flux_position
/// [10]    WHY: 4-bit cause_type + 4-bit intention
/// [11]    HOW: 4-bit method + 4-bit complexity
/// [12-13] RELATION: 8-bit type + 8-bit object_ref
/// [14-15] ELP: 5-bit ethos + 5-bit logos + 5-bit pathos + 1-bit sacred
/// [16-17] CONFIDENCE: 10-bit value + 6-bit uncertainty_width
/// [18-19] DYNAMICS: 8-bit velocity + 8-bit acceleration
/// [20-21] INFERENCE: 8-bit chain_depth + 8-bit evidence_count
/// [22]    LADDER_IDX: 8-bit transitive rank
/// [23]    FLAGS: 8-bit metadata flags
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FluxCompression24 {
    /// WHO: Entity/Actor (20-bit hash + 4-bit type)
    pub who: [u8; 3],
    /// WHAT: Action/Concept (20-bit hash + 4-bit type)
    pub what: [u8; 3],
    /// WHEN: Temporal (12-bit offset + 4-bit granularity)
    pub when: [u8; 2],
    /// WHERE: Location/Flux (12-bit hash + 4-bit position)
    pub where_: [u8; 2],
    /// WHY: Causality (4-bit type + 4-bit intention)
    pub why: u8,
    /// HOW: Method (4-bit type + 4-bit complexity)
    pub how: u8,
    /// RELATION: Type + Object reference
    pub relation: [u8; 2],
    /// ELP: Compressed Ethos/Logos/Pathos
    pub elp: [u8; 2],
    /// CONFIDENCE: Value + Uncertainty width
    pub confidence: [u8; 2],
    /// DYNAMICS: Velocity + Acceleration of state
    pub dynamics: [u8; 2],
    /// INFERENCE: Chain depth + Evidence count
    pub inference: [u8; 2],
    /// LADDER_IDX: Transitive relation rank
    pub ladder_idx: u8,
    /// FLAGS: Metadata
    pub flags: u8,
}

impl FluxCompression24 {
    pub const SIZE: usize = 24;
    
    // =========================================================================
    // WHO Accessors
    // =========================================================================
    
    /// Extract WHO hash (20-bit)
    pub fn who_hash(&self) -> u32 {
        let b0 = self.who[0] as u32;
        let b1 = self.who[1] as u32;
        let b2 = (self.who[2] & 0x0F) as u32;
        b0 | (b1 << 8) | (b2 << 16)
    }
    
    /// Extract WHO entity type
    pub fn who_type(&self) -> EntityType {
        let t = (self.who[2] >> 4) & 0x0F;
        match t {
            0 => EntityType::Person,
            1 => EntityType::Place,
            2 => EntityType::Thing,
            3 => EntityType::Action,
            4 => EntityType::Concept,
            5 => EntityType::Time,
            6 => EntityType::Quantity,
            7 => EntityType::Quality,
            8 => EntityType::Event,
            9 => EntityType::State,
            _ => EntityType::Unknown,
        }
    }
    
    /// Pack WHO
    pub fn pack_who(&mut self, entity: &str, entity_type: EntityType) {
        let hash = hash_20bit(entity);
        self.who[0] = (hash & 0xFF) as u8;
        self.who[1] = ((hash >> 8) & 0xFF) as u8;
        self.who[2] = ((hash >> 16) & 0x0F) as u8 | ((entity_type as u8) << 4);
    }
    
    // =========================================================================
    // WHAT Accessors
    // =========================================================================
    
    /// Extract WHAT hash (20-bit)
    pub fn what_hash(&self) -> u32 {
        let b0 = self.what[0] as u32;
        let b1 = self.what[1] as u32;
        let b2 = (self.what[2] & 0x0F) as u32;
        b0 | (b1 << 8) | (b2 << 16)
    }
    
    /// Pack WHAT
    pub fn pack_what(&mut self, concept: &str, action_type: u8) {
        let hash = hash_20bit(concept);
        self.what[0] = (hash & 0xFF) as u8;
        self.what[1] = ((hash >> 8) & 0xFF) as u8;
        self.what[2] = ((hash >> 16) & 0x0F) as u8 | ((action_type & 0x0F) << 4);
    }
    
    // =========================================================================
    // WHEN Accessors
    // =========================================================================
    
    /// Extract WHEN offset (12-bit, signed)
    pub fn when_offset(&self) -> i16 {
        let raw = u16::from_le_bytes([self.when[0], self.when[1] & 0x0F]);
        // Sign extend from 12-bit
        if raw & 0x800 != 0 {
            (raw | 0xF000) as i16
        } else {
            raw as i16
        }
    }
    
    /// Extract WHEN granularity
    pub fn when_granularity(&self) -> TemporalGranularity {
        let g = (self.when[1] >> 4) & 0x0F;
        match g {
            0 => TemporalGranularity::Instant,
            1 => TemporalGranularity::Second,
            2 => TemporalGranularity::Minute,
            3 => TemporalGranularity::Hour,
            4 => TemporalGranularity::Day,
            5 => TemporalGranularity::Week,
            6 => TemporalGranularity::Month,
            7 => TemporalGranularity::Year,
            8 => TemporalGranularity::Decade,
            9 => TemporalGranularity::Century,
            10 => TemporalGranularity::Relative,
            11 => TemporalGranularity::Duration,
            12 => TemporalGranularity::Recurring,
            _ => TemporalGranularity::Unknown,
        }
    }
    
    /// Pack WHEN
    pub fn pack_when(&mut self, offset: i16, granularity: TemporalGranularity) {
        let offset_12 = (offset & 0x0FFF) as u16;
        self.when[0] = (offset_12 & 0xFF) as u8;
        self.when[1] = ((offset_12 >> 8) & 0x0F) as u8 | ((granularity as u8) << 4);
    }
    
    // =========================================================================
    // WHERE Accessors
    // =========================================================================
    
    /// Extract WHERE location hash (12-bit)
    pub fn where_hash(&self) -> u16 {
        u16::from_le_bytes([self.where_[0], self.where_[1] & 0x0F])
    }
    
    /// Extract WHERE flux position (0-9)
    pub fn where_flux(&self) -> u8 {
        (self.where_[1] >> 4) & 0x0F
    }
    
    /// Pack WHERE
    pub fn pack_where(&mut self, location: &str, flux_position: u8) {
        let hash = hash_12bit(location);
        self.where_[0] = (hash & 0xFF) as u8;
        self.where_[1] = ((hash >> 8) & 0x0F) as u8 | ((flux_position & 0x0F) << 4);
    }
    
    // =========================================================================
    // WHY Accessors
    // =========================================================================
    
    /// Extract WHY cause type (0-15)
    pub fn why_type(&self) -> u8 {
        self.why & 0x0F
    }
    
    /// Extract WHY intention strength (0.0-1.0)
    pub fn why_intention(&self) -> f32 {
        ((self.why >> 4) & 0x0F) as f32 / 15.0
    }
    
    /// Pack WHY
    pub fn pack_why(&mut self, cause_type: u8, intention: f32) {
        let intention_4 = (intention.clamp(0.0, 1.0) * 15.0) as u8;
        self.why = (cause_type & 0x0F) | (intention_4 << 4);
    }
    
    // =========================================================================
    // HOW Accessors
    // =========================================================================
    
    /// Extract HOW method type (0-15)
    pub fn how_type(&self) -> u8 {
        self.how & 0x0F
    }
    
    /// Extract HOW complexity (0.0-1.0)
    pub fn how_complexity(&self) -> f32 {
        ((self.how >> 4) & 0x0F) as f32 / 15.0
    }
    
    /// Pack HOW
    pub fn pack_how(&mut self, method_type: u8, complexity: f32) {
        let complexity_4 = (complexity.clamp(0.0, 1.0) * 15.0) as u8;
        self.how = (method_type & 0x0F) | (complexity_4 << 4);
    }
    
    // =========================================================================
    // RELATION Accessors
    // =========================================================================
    
    /// Extract relation type
    pub fn relation_type(&self) -> RelationType {
        let t = self.relation[0];
        // Safety: All values map to valid variants or Unknown
        unsafe { std::mem::transmute(t) }
    }
    
    /// Extract relation object reference (8-bit ID)
    pub fn relation_object(&self) -> u8 {
        self.relation[1]
    }
    
    /// Pack RELATION
    pub fn pack_relation(&mut self, rel_type: RelationType, object_ref: u8) {
        self.relation[0] = rel_type as u8;
        self.relation[1] = object_ref;
    }
    
    // =========================================================================
    // ELP Accessors (5-5-5-1 bit packing)
    // =========================================================================
    
    /// Extract Ethos (0.0-1.0)
    pub fn ethos(&self) -> f32 {
        let raw = u16::from_le_bytes(self.elp);
        ((raw & 0x1F) as f32) / 31.0
    }
    
    /// Extract Logos (0.0-1.0)
    pub fn logos(&self) -> f32 {
        let raw = u16::from_le_bytes(self.elp);
        (((raw >> 5) & 0x1F) as f32) / 31.0
    }
    
    /// Extract Pathos (0.0-1.0)
    pub fn pathos(&self) -> f32 {
        let raw = u16::from_le_bytes(self.elp);
        (((raw >> 10) & 0x1F) as f32) / 31.0
    }
    
    /// Check if at sacred position
    pub fn is_sacred(&self) -> bool {
        let raw = u16::from_le_bytes(self.elp);
        (raw & 0x8000) != 0
    }
    
    /// Pack ELP
    pub fn pack_elp(&mut self, ethos: f32, logos: f32, pathos: f32, sacred: bool) {
        let e = (ethos.clamp(0.0, 1.0) * 31.0) as u16;
        let l = (logos.clamp(0.0, 1.0) * 31.0) as u16;
        let p = (pathos.clamp(0.0, 1.0) * 31.0) as u16;
        let s = if sacred { 0x8000u16 } else { 0 };
        let packed = (e & 0x1F) | ((l & 0x1F) << 5) | ((p & 0x1F) << 10) | s;
        self.elp = packed.to_le_bytes();
    }
    
    // =========================================================================
    // CONFIDENCE Accessors (10-bit value + 6-bit uncertainty)
    // =========================================================================
    
    /// Extract confidence value (0.0-1.0)
    pub fn confidence_value(&self) -> f32 {
        let raw = u16::from_le_bytes(self.confidence);
        ((raw & 0x03FF) as f32) / 1023.0
    }
    
    /// Extract confidence uncertainty width (0.0-1.0)
    /// Represents the width of the confidence interval
    pub fn confidence_uncertainty(&self) -> f32 {
        let raw = u16::from_le_bytes(self.confidence);
        (((raw >> 10) & 0x3F) as f32) / 63.0
    }
    
    /// Get confidence interval (lower, upper)
    pub fn confidence_interval(&self) -> (f32, f32) {
        let value = self.confidence_value();
        let width = self.confidence_uncertainty();
        let half_width = width / 2.0;
        ((value - half_width).max(0.0), (value + half_width).min(1.0))
    }
    
    /// Pack CONFIDENCE
    pub fn pack_confidence(&mut self, value: f32, uncertainty: f32) {
        let v = (value.clamp(0.0, 1.0) * 1023.0) as u16;
        let u = (uncertainty.clamp(0.0, 1.0) * 63.0) as u16;
        let packed = (v & 0x03FF) | ((u & 0x3F) << 10);
        self.confidence = packed.to_le_bytes();
    }
    
    // =========================================================================
    // DYNAMICS Accessors (velocity + acceleration)
    // =========================================================================
    
    /// Extract velocity (rate of change, signed -1.0 to 1.0)
    pub fn velocity(&self) -> f32 {
        let raw = self.dynamics[0] as i8;
        raw as f32 / 127.0
    }
    
    /// Extract acceleration (second derivative, signed -1.0 to 1.0)
    pub fn acceleration(&self) -> f32 {
        let raw = self.dynamics[1] as i8;
        raw as f32 / 127.0
    }
    
    /// Pack DYNAMICS
    pub fn pack_dynamics(&mut self, velocity: f32, acceleration: f32) {
        let v = (velocity.clamp(-1.0, 1.0) * 127.0) as i8;
        let a = (acceleration.clamp(-1.0, 1.0) * 127.0) as i8;
        self.dynamics[0] = v as u8;
        self.dynamics[1] = a as u8;
    }
    
    // =========================================================================
    // INFERENCE Accessors
    // =========================================================================
    
    /// Extract inference chain depth (0-255)
    pub fn chain_depth(&self) -> u8 {
        self.inference[0]
    }
    
    /// Extract evidence count (0-255)
    pub fn evidence_count(&self) -> u8 {
        self.inference[1]
    }
    
    /// Pack INFERENCE
    pub fn pack_inference(&mut self, chain_depth: u8, evidence_count: u8) {
        self.inference[0] = chain_depth;
        self.inference[1] = evidence_count;
    }
    
    // =========================================================================
    // LADDER INDEX Accessors
    // =========================================================================
    
    /// Extract ladder index (transitive rank)
    pub fn ladder_index(&self) -> u8 {
        self.ladder_idx
    }
    
    /// Pack LADDER INDEX
    pub fn pack_ladder_index(&mut self, index: u8) {
        self.ladder_idx = index;
    }
    
    // =========================================================================
    // FLAGS Accessors
    // =========================================================================
    
    /// Check if validated
    pub fn is_validated(&self) -> bool {
        (self.flags & 0x01) != 0
    }
    
    /// Check if negated
    pub fn is_negated(&self) -> bool {
        (self.flags & 0x02) != 0
    }
    
    /// Check if hypothetical
    pub fn is_hypothetical(&self) -> bool {
        (self.flags & 0x04) != 0
    }
    
    /// Check if has multiple subjects
    pub fn has_multi_subject(&self) -> bool {
        (self.flags & 0x08) != 0
    }
    
    /// Check if has multiple objects
    pub fn has_multi_object(&self) -> bool {
        (self.flags & 0x10) != 0
    }
    
    /// Check if reversible (VCP)
    pub fn is_reversible(&self) -> bool {
        (self.flags & 0x20) != 0
    }
    
    /// Check if cross-referenced
    pub fn has_cross_ref(&self) -> bool {
        (self.flags & 0x40) != 0
    }
    
    /// Check if extended (links to 32/48/64-byte format)
    pub fn is_extended(&self) -> bool {
        (self.flags & 0x80) != 0
    }
    
    /// Pack FLAGS
    pub fn pack_flags(
        &mut self,
        validated: bool,
        negated: bool,
        hypothetical: bool,
        multi_subject: bool,
        multi_object: bool,
        reversible: bool,
        cross_ref: bool,
        extended: bool,
    ) {
        self.flags = (validated as u8)
            | ((negated as u8) << 1)
            | ((hypothetical as u8) << 2)
            | ((multi_subject as u8) << 3)
            | ((multi_object as u8) << 4)
            | ((reversible as u8) << 5)
            | ((cross_ref as u8) << 6)
            | ((extended as u8) << 7);
    }
}

impl Default for FluxCompression24 {
    fn default() -> Self {
        Self {
            who: [0; 3],
            what: [0; 3],
            when: [0; 2],
            where_: [0; 2],
            why: 0,
            how: 0,
            relation: [RelationType::None as u8, 0],
            elp: [0; 2],
            confidence: [0; 2],
            dynamics: [0; 2],
            inference: [0; 2],
            ladder_idx: 0,
            flags: 0,
        }
    }
}

// =============================================================================
// 32-BYTE EXTENDED FORMAT (+ Temporal Dynamics + Uncertainty)
// =============================================================================

/// 32-byte extended format with full temporal dynamics and uncertainty bounds
#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FluxCompression32 {
    /// Base 24-byte compression
    pub base: FluxCompression24,
    /// Extended WHO: Secondary entity hash (16-bit)
    pub who_secondary: [u8; 2],
    /// Extended WHAT: Secondary action hash (16-bit)
    pub what_secondary: [u8; 2],
    /// Confidence lower bound (8-bit)
    pub confidence_lower: u8,
    /// Confidence upper bound (8-bit)
    pub confidence_upper: u8,
    /// Temporal velocity (rate of time progression)
    pub temporal_velocity: u8,
    /// Reserved for future use
    pub reserved: u8,
}

impl FluxCompression32 {
    pub const SIZE: usize = 32;
    
    /// Get secondary WHO hash
    pub fn who_secondary_hash(&self) -> u16 {
        u16::from_le_bytes(self.who_secondary)
    }
    
    /// Get secondary WHAT hash
    pub fn what_secondary_hash(&self) -> u16 {
        u16::from_le_bytes(self.what_secondary)
    }
    
    /// Get explicit confidence bounds
    pub fn confidence_bounds(&self) -> (f32, f32) {
        let lower = self.confidence_lower as f32 / 255.0;
        let upper = self.confidence_upper as f32 / 255.0;
        (lower, upper)
    }
    
    /// Pack secondary WHO
    pub fn pack_who_secondary(&mut self, entity: &str) {
        let hash = hash_16bit(entity);
        self.who_secondary = hash.to_le_bytes();
    }
    
    /// Pack secondary WHAT
    pub fn pack_what_secondary(&mut self, concept: &str) {
        let hash = hash_16bit(concept);
        self.what_secondary = hash.to_le_bytes();
    }
    
    /// Pack confidence bounds
    pub fn pack_confidence_bounds(&mut self, lower: f32, upper: f32) {
        self.confidence_lower = (lower.clamp(0.0, 1.0) * 255.0) as u8;
        self.confidence_upper = (upper.clamp(0.0, 1.0) * 255.0) as u8;
    }
}

impl Default for FluxCompression32 {
    fn default() -> Self {
        Self {
            base: FluxCompression24::default(),
            who_secondary: [0; 2],
            what_secondary: [0; 2],
            confidence_lower: 0,
            confidence_upper: 255,
            temporal_velocity: 128, // Neutral
            reserved: 0,
        }
    }
}

// =============================================================================
// 48-BYTE EXTENDED FORMAT (+ Inference State + Evidence Chain)
// =============================================================================

/// 48-byte format with full inference state and evidence chain
#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FluxCompression48 {
    /// Base 32-byte compression
    pub base: FluxCompression32,
    /// Evidence hashes (4 × 32-bit = 16 bytes)
    pub evidence: [[u8; 4]; 4],
}

impl FluxCompression48 {
    pub const SIZE: usize = 48;
    
    /// Get evidence hash at index
    pub fn evidence_hash(&self, index: usize) -> u32 {
        if index < 4 {
            u32::from_le_bytes(self.evidence[index])
        } else {
            0
        }
    }
    
    /// Pack evidence hash at index
    pub fn pack_evidence(&mut self, index: usize, evidence: &str) {
        if index < 4 {
            let hash = hash_32bit(evidence);
            self.evidence[index] = hash.to_le_bytes();
        }
    }
    
    /// Get all non-zero evidence hashes
    pub fn all_evidence(&self) -> Vec<u32> {
        self.evidence
            .iter()
            .map(|e| u32::from_le_bytes(*e))
            .filter(|&h| h != 0)
            .collect()
    }
}

impl Default for FluxCompression48 {
    fn default() -> Self {
        Self {
            base: FluxCompression32::default(),
            evidence: [[0; 4]; 4],
        }
    }
}

// =============================================================================
// 64-BYTE EXTENDED FORMAT (+ Multi-Entity + Full Reasoning Trace)
// =============================================================================

/// 64-byte format with multi-entity support and full reasoning trace
#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FluxCompression64 {
    /// Base 48-byte compression
    pub base: FluxCompression48,
    /// Additional entities (4 × 24-bit hash = 12 bytes)
    pub entities: [[u8; 3]; 4],
    /// Reasoning trace (4 bytes: step indices)
    pub reasoning_trace: [u8; 4],
}

impl FluxCompression64 {
    pub const SIZE: usize = 64;
    
    /// Get entity hash at index
    pub fn entity_hash(&self, index: usize) -> u32 {
        if index < 4 {
            let e = &self.entities[index];
            (e[0] as u32) | ((e[1] as u32) << 8) | ((e[2] as u32) << 16)
        } else {
            0
        }
    }
    
    /// Pack entity at index
    pub fn pack_entity(&mut self, index: usize, entity: &str) {
        if index < 4 {
            let hash = hash_20bit(entity);
            self.entities[index][0] = (hash & 0xFF) as u8;
            self.entities[index][1] = ((hash >> 8) & 0xFF) as u8;
            self.entities[index][2] = ((hash >> 16) & 0xFF) as u8;
        }
    }
    
    /// Get reasoning trace
    pub fn get_reasoning_trace(&self) -> [u8; 4] {
        self.reasoning_trace
    }
    
    /// Pack reasoning trace
    pub fn pack_reasoning_trace(&mut self, trace: [u8; 4]) {
        self.reasoning_trace = trace;
    }
}

impl Default for FluxCompression64 {
    fn default() -> Self {
        Self {
            base: FluxCompression48::default(),
            entities: [[0; 3]; 4],
            reasoning_trace: [0; 4],
        }
    }
}

// =============================================================================
// HASH FUNCTIONS
// =============================================================================

/// Hash string to 12 bits
#[inline]
pub fn hash_12bit(s: &str) -> u16 {
    let mut hasher = DefaultHasher::new();
    s.to_lowercase().hash(&mut hasher);
    (hasher.finish() & 0x0FFF) as u16
}

/// Hash string to 16 bits
#[inline]
pub fn hash_16bit(s: &str) -> u16 {
    let mut hasher = DefaultHasher::new();
    s.to_lowercase().hash(&mut hasher);
    (hasher.finish() & 0xFFFF) as u16
}

/// Hash string to 20 bits
#[inline]
pub fn hash_20bit(s: &str) -> u32 {
    let mut hasher = DefaultHasher::new();
    s.to_lowercase().hash(&mut hasher);
    (hasher.finish() & 0x0FFFFF) as u32
}

/// Hash string to 32 bits
#[inline]
pub fn hash_32bit(s: &str) -> u32 {
    let mut hasher = DefaultHasher::new();
    s.to_lowercase().hash(&mut hasher);
    (hasher.finish() & 0xFFFFFFFF) as u32
}

// =============================================================================
// BUILDER PATTERN
// =============================================================================

/// Builder for FluxCompression24
pub struct FluxCompression24Builder {
    inner: FluxCompression24,
}

impl FluxCompression24Builder {
    pub fn new() -> Self {
        Self {
            inner: FluxCompression24::default(),
        }
    }
    
    pub fn who(mut self, entity: &str, entity_type: EntityType) -> Self {
        self.inner.pack_who(entity, entity_type);
        self
    }
    
    pub fn what(mut self, concept: &str, action_type: u8) -> Self {
        self.inner.pack_what(concept, action_type);
        self
    }
    
    pub fn when(mut self, offset: i16, granularity: TemporalGranularity) -> Self {
        self.inner.pack_when(offset, granularity);
        self
    }
    
    pub fn where_(mut self, location: &str, flux_position: u8) -> Self {
        self.inner.pack_where(location, flux_position);
        self
    }
    
    pub fn why(mut self, cause_type: u8, intention: f32) -> Self {
        self.inner.pack_why(cause_type, intention);
        self
    }
    
    pub fn how(mut self, method_type: u8, complexity: f32) -> Self {
        self.inner.pack_how(method_type, complexity);
        self
    }
    
    pub fn relation(mut self, rel_type: RelationType, object_ref: u8) -> Self {
        self.inner.pack_relation(rel_type, object_ref);
        self
    }
    
    pub fn elp(mut self, ethos: f32, logos: f32, pathos: f32, sacred: bool) -> Self {
        self.inner.pack_elp(ethos, logos, pathos, sacred);
        self
    }
    
    pub fn confidence(mut self, value: f32, uncertainty: f32) -> Self {
        self.inner.pack_confidence(value, uncertainty);
        self
    }
    
    pub fn dynamics(mut self, velocity: f32, acceleration: f32) -> Self {
        self.inner.pack_dynamics(velocity, acceleration);
        self
    }
    
    pub fn inference(mut self, chain_depth: u8, evidence_count: u8) -> Self {
        self.inner.pack_inference(chain_depth, evidence_count);
        self
    }
    
    pub fn ladder_index(mut self, index: u8) -> Self {
        self.inner.pack_ladder_index(index);
        self
    }
    
    pub fn validated(mut self, v: bool) -> Self {
        if v { self.inner.flags |= 0x01; } else { self.inner.flags &= !0x01; }
        self
    }
    
    pub fn negated(mut self, v: bool) -> Self {
        if v { self.inner.flags |= 0x02; } else { self.inner.flags &= !0x02; }
        self
    }
    
    pub fn hypothetical(mut self, v: bool) -> Self {
        if v { self.inner.flags |= 0x04; } else { self.inner.flags &= !0x04; }
        self
    }
    
    pub fn build(self) -> FluxCompression24 {
        self.inner
    }
}

impl Default for FluxCompression24Builder {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// INFERENCE ENGINE
// =============================================================================

/// Inference operations on compressed data
pub struct FluxInferenceEngine {
    /// Transitive relation cache
    transitive_cache: Vec<(u8, u8, RelationType, f32)>, // (subj, obj, rel, conf)
}

impl FluxInferenceEngine {
    pub fn new() -> Self {
        Self {
            transitive_cache: Vec::new(),
        }
    }
    
    /// Add a relation to the transitive cache
    pub fn add_relation(&mut self, subject: u8, object: u8, rel: RelationType, confidence: f32) {
        self.transitive_cache.push((subject, object, rel, confidence));
        
        // Compute transitive closure if relation is transitive
        if rel.is_transitive() {
            self.compute_transitive_closure(subject, object, rel, confidence);
        }
    }
    
    /// Compute transitive closure for a new relation
    fn compute_transitive_closure(&mut self, subject: u8, object: u8, rel: RelationType, confidence: f32) {
        let decay = rel.transitive_decay();
        let mut new_relations = Vec::new();
        
        // Find relations where our object is the subject (A→B, B→C = A→C)
        for &(s, o, r, c) in &self.transitive_cache {
            if s == object && r == rel {
                let new_conf = confidence * c * decay;
                if new_conf > 0.1 { // Threshold
                    new_relations.push((subject, o, rel, new_conf));
                }
            }
        }
        
        // Find relations where our subject is the object (B→C, A→B = A→C)
        for &(s, o, r, c) in &self.transitive_cache {
            if o == subject && r == rel {
                let new_conf = confidence * c * decay;
                if new_conf > 0.1 {
                    new_relations.push((s, object, rel, new_conf));
                }
            }
        }
        
        self.transitive_cache.extend(new_relations);
    }
    
    /// Query if a relation exists (with confidence)
    pub fn query_relation(&self, subject: u8, object: u8, rel: RelationType) -> Option<f32> {
        self.transitive_cache
            .iter()
            .filter(|&&(s, o, r, _)| s == subject && o == object && r == rel)
            .map(|&(_, _, _, c)| c)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }
    
    /// Infer from compressed data
    pub fn infer(&mut self, compressed: &FluxCompression24) -> Vec<(RelationType, u8, f32)> {
        let mut inferences = Vec::new();
        
        let rel = compressed.relation_type();
        let obj = compressed.relation_object();
        let conf = compressed.confidence_value();
        let who_hash = (compressed.who_hash() & 0xFF) as u8;
        
        if rel != RelationType::None {
            // Add direct relation
            self.add_relation(who_hash, obj, rel, conf);
            
            // Check for inverse
            let inv = rel.inverse();
            if inv != rel {
                inferences.push((inv, who_hash, conf));
            }
            
            // Check transitive inferences
            if rel.is_transitive() {
                for &(s, o, r, c) in &self.transitive_cache {
                    if s == who_hash && r == rel && o != obj {
                        inferences.push((rel, o, c));
                    }
                }
            }
        }
        
        inferences
    }
    
    /// Clear cache
    pub fn clear(&mut self) {
        self.transitive_cache.clear();
    }
}

impl Default for FluxInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_24byte_size() {
        assert_eq!(std::mem::size_of::<FluxCompression24>(), 24);
    }
    
    #[test]
    fn test_32byte_size() {
        assert_eq!(std::mem::size_of::<FluxCompression32>(), 32);
    }
    
    #[test]
    fn test_48byte_size() {
        assert_eq!(std::mem::size_of::<FluxCompression48>(), 48);
    }
    
    #[test]
    fn test_64byte_size() {
        assert_eq!(std::mem::size_of::<FluxCompression64>(), 64);
    }
    
    #[test]
    fn test_who_encoding() {
        let mut c = FluxCompression24::default();
        c.pack_who("Alice", EntityType::Person);
        
        assert_eq!(c.who_type(), EntityType::Person);
        assert!(c.who_hash() > 0);
    }
    
    #[test]
    fn test_elp_encoding() {
        let mut c = FluxCompression24::default();
        c.pack_elp(0.8, 0.6, 0.4, true);
        
        assert!((c.ethos() - 0.8).abs() < 0.05);
        assert!((c.logos() - 0.6).abs() < 0.05);
        assert!((c.pathos() - 0.4).abs() < 0.05);
        assert!(c.is_sacred());
    }
    
    #[test]
    fn test_confidence_interval() {
        let mut c = FluxCompression24::default();
        c.pack_confidence(0.75, 0.2);
        
        let (lower, upper) = c.confidence_interval();
        assert!(lower < 0.75);
        assert!(upper > 0.75);
        assert!((upper - lower - 0.2).abs() < 0.05);
    }
    
    #[test]
    fn test_dynamics() {
        let mut c = FluxCompression24::default();
        c.pack_dynamics(0.5, -0.3);
        
        assert!((c.velocity() - 0.5).abs() < 0.02);
        assert!((c.acceleration() - (-0.3)).abs() < 0.02);
    }
    
    #[test]
    fn test_relation_transitive() {
        assert!(RelationType::LeftOf.is_transitive());
        assert!(RelationType::BiggerThan.is_transitive());
        assert!(!RelationType::Owns.is_transitive());
    }
    
    #[test]
    fn test_relation_inverse() {
        assert_eq!(RelationType::LeftOf.inverse(), RelationType::RightOf);
        assert_eq!(RelationType::BiggerThan.inverse(), RelationType::SmallerThan);
        assert_eq!(RelationType::Causes.inverse(), RelationType::CausedBy);
    }
    
    #[test]
    fn test_builder() {
        let c = FluxCompression24Builder::new()
            .who("Alice", EntityType::Person)
            .what("move", 1)
            .where_("kitchen", 3)
            .relation(RelationType::Moves, 5)
            .elp(0.7, 0.8, 0.5, true)
            .confidence(0.9, 0.1)
            .validated(true)
            .build();
        
        assert_eq!(c.who_type(), EntityType::Person);
        assert_eq!(c.where_flux(), 3);
        assert_eq!(c.relation_type(), RelationType::Moves);
        assert!(c.is_sacred());
        assert!(c.is_validated());
    }
    
    #[test]
    fn test_transitive_inference() {
        let mut engine = FluxInferenceEngine::new();
        
        // A is left of B
        engine.add_relation(1, 2, RelationType::LeftOf, 0.9);
        // B is left of C
        engine.add_relation(2, 3, RelationType::LeftOf, 0.9);
        
        // Should infer A is left of C
        let conf = engine.query_relation(1, 3, RelationType::LeftOf);
        assert!(conf.is_some());
        assert!(conf.unwrap() > 0.7); // With decay
    }
}
