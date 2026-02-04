/// Overflow risk classification for Vortex Context Preserver (VCP)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum OverflowRisk {
    Safe,
    Warning,
    Critical,
    Imminent,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::data::attributes::{Attributes, AttributeValue, AttributeAccessor};

/// ELP Tensor for geometric space representation
/// 
/// DEPRECATED: Use Attributes system instead for new code.
/// Kept for backward compatibility with existing code.
/// 
/// Core data structure used across training, federated learning, and visualization.
/// Not gated behind features so it's always available.
#[deprecated(since = "0.3.0", note = "Use Attributes system instead. See docs/ATTRIBUTES_MIGRATION.md")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct ELPTensor {
    /// Ethos (Character/Authority): -13 to +13
    pub ethos: f64,
    /// Logos (Logic/Analytical): -13 to +13
    pub logos: f64,
    /// Pathos (Emotion/Expressive): -13 to +13
    pub pathos: f64,
}

impl ELPTensor {
    /// Creates a new ELP tensor
    pub fn new(ethos: f64, logos: f64, pathos: f64) -> Self {
        ELPTensor { ethos, logos, pathos }
    }
    
    /// Calculate Euclidean distance to another ELP tensor
    #[inline(always)]
    pub fn distance(&self, other: &Self) -> f64 {
        let de = self.ethos - other.ethos;
        let dl = self.logos - other.logos;
        let dp = self.pathos - other.pathos;
        (de * de + dl * dl + dp * dp).sqrt()
    }
    
    /// Calculates tensor magnitude
    #[inline(always)]
    pub fn magnitude(&self) -> f64 {
        (self.ethos.powi(2) + self.logos.powi(2) + self.pathos.powi(2)).sqrt()
    }
    
    /// Returns the dominant channel
    pub fn dominant_channel(&self) -> &str {
        let abs_e = self.ethos.abs();
        let abs_l = self.logos.abs();
        let abs_p = self.pathos.abs();
        
        if abs_e > abs_l && abs_e > abs_p {
            "ethos"
        } else if abs_l > abs_p {
            "logos"
        } else {
            "pathos"
        }
    }
    
    /// Convert to Attributes system
    pub fn to_attributes(&self) -> Attributes {
        Attributes::with_elp(self.ethos as f32, self.logos as f32, self.pathos as f32)
    }
    
    /// Create from Attributes system
    pub fn from_attributes(attrs: &Attributes) -> Self {
        let elp = attrs.elp_tensor();
        Self::new(elp[0] as f64, elp[1] as f64, elp[2] as f64)
    }
}

/// Core Flux Matrix pattern: 1, 2, 4, 8, 7, 5, 1 (with sacred guides 3, 6, 9)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxMatrix {
    pub id: Uuid,
    pub subject: String,
    pub nodes: HashMap<u8, FluxNode>,
    pub sacred_guides: HashMap<u8, SacredGuide>, // 3, 6, 9
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Reference to an object node (can be same subject matrix or cross-subject)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRef {
    /// Subject (matrix domain) where the object node lives
    pub subject: String,
    /// Position of the object node in the subject matrix (0-9)
    pub position: u8,
}

/// Relation triple tying Subject (node) → Relation (predicate) → Object (node)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// Name/label of the relation (e.g., "causes", "is_a", "part_of")
    pub name: String,
    /// Signed index position in the ladder (aligned with SemanticAssociation.index)
    pub index: i16,
    /// Weight for ranking and traversal (0.0 - 1.0)
    pub weight: f32,
    /// Computed rank in the ladder (1..N), lower is higher priority
    pub ladder_rank: u8,
    /// The object this relation targets
    pub object: ObjectRef,
    /// Optional confidence for the relation assertion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    /// Optional context tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Individual node in the 9-position flux matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxNode {
    pub position: u8,   // 0-8 (0 = neutral center)
    pub base_value: u8, // Core flux pattern value (1,2,4,8,7,5,1,2,4)
    pub semantic_index: SemanticIndex,
    pub attributes: NodeAttributes,
    pub connections: Vec<NodeConnection>,
}

/// Semantic indexing with positive/negative reinforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticIndex {
    pub positive_associations: Vec<SemanticAssociation>, // +1 and above (Heaven)
    pub negative_associations: Vec<SemanticAssociation>, // -1 and below (Hell)
    pub neutral_base: String,                            // Core meaning at index 0
    /// Predicates associated with this subject (node): weighted indices/ladder
    #[serde(default)]
    pub predicates: Vec<Predicate>,
    /// Relations (predicate + object) attached to this subject (node)
    #[serde(default)]
    pub relations: Vec<Relation>,
}

/// Individual semantic association with index weight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAssociation {
    pub word: String,
    pub index: i16,
    pub confidence: f64,
    /// Unified attributes - all inferrable
    pub attributes: HashMap<String, f32>,
}

impl SemanticAssociation {
    pub fn new(word: String, index: i16, confidence: f64) -> Self {
        Self {
            word,
            index,
            confidence,
            attributes: HashMap::new(),
        }
    }

    /// Get any attribute by name
    pub fn get_attribute(&self, name: &str) -> Option<f32> {
        self.attributes.get(name).copied()
    }

    /// Set any attribute
    pub fn set_attribute(&mut self, name: String, value: f32) {
        self.attributes.insert(name, value);
    }

    /// Convenience methods for common attributes
    pub fn ethos(&self) -> f32 {
        self.get_attribute("ethos").unwrap_or(0.0)
    }

    pub fn logos(&self) -> f32 {
        self.get_attribute("logos").unwrap_or(0.0)
    }

    pub fn pathos(&self) -> f32 {
        self.get_attribute("pathos").unwrap_or(0.0)
    }
}

/// Predicate represents the relation (verb/connector) attached to a Subject (node)
/// Weighted and ordered as a ladder for traversal and ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predicate {
    /// Name/label of the predicate (e.g., "causes", "is_a", "related_to")
    pub name: String,
    /// Signed index position in the ladder (aligned with SemanticAssociation.index)
    pub index: i16,
    /// Weight for ranking and traversal (0.0 - 1.0)
    pub weight: f32,
    /// Computed rank in the ladder (1..N), lower is higher priority
    pub ladder_rank: u8,
    /// Optional target node position this predicate points to (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_position: Option<u8>,
    /// Optional context tag for the predicate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Sacred guide nodes (3, 6, 9) with special properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredGuide {
    pub position: u8, // 3, 6, or 9
    pub divine_properties: Vec<String>,
    pub intersection_points: Vec<IntersectionPoint>,
    pub geometric_significance: String,
}

/// Node attributes for computational properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAttributes {
    pub properties: HashMap<String, String>,
    pub parameters: HashMap<String, f64>,
    pub state: NodeState,
    pub dynamics: NodeDynamics,
}

/// Connection between nodes in the matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnection {
    pub target_position: u8,
    pub connection_type: ConnectionType,
    pub weight: f32,
    pub bidirectional: bool,
}

/// Intersection points for sacred guides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntersectionPoint {
    pub with_node: u8,
    pub significance: String,
    pub computational_value: f64,
}

/// Node state for dynamic processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub active: bool,
    pub last_accessed: DateTime<Utc>,
    pub usage_count: u64,
    pub context_stack: Vec<String>,
}

/// Position in the vortex loop sequence (1→2→4→8→7→5→1)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VortexPosition {
    Position1,      // Beginning of loop
    Position2,      // Second in sequence
    Position4,      // Power position
    Position8,      // Mastery position
    Position7,      // Wisdom position
    Position5,      // Change position
    LoopComplete,   // Returned to 1, ready for next iteration
}

impl Default for VortexPosition {
    fn default() -> Self {
        VortexPosition::Position1
    }
}

/// Role in the sacred geometry order of operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderRole {
    Center,       // Position 0: Neutral/Balance
    Beginning,    // Position 1: Ethos start
    Expansion,    // Position 2: Growth
    SacredEthos,  // Position 3: Unity checkpoint ✨
    Power,        // Position 4: Logos peak
    Change,       // Position 5: Pathos dynamics
    SacredPathos, // Position 6: Heart checkpoint ✨
    Wisdom,       // Position 7: Understanding
    Mastery,      // Position 8: Excellence
    SacredLogos,  // Position 9: Ultimate checkpoint ✨
}

impl Default for OrderRole {
    fn default() -> Self {
        OrderRole::Center
    }
}

/// Dominant attribute channel for the node
/// Renamed from ELPChannel to AttributeChannel for clarity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttributeChannel {
    Ethos,   // Positions 1, 3 - Character/Identity
    Logos,   // Positions 4, 9 - Logic/Reason
    Pathos,  // Positions 5, 6 - Emotion/Dynamics
    Mixed,   // Positions 2, 7, 8 - Multiple channels
    Neutral, // Position 0 - Balance
}

/// Backward compatibility alias
pub type ELPChannel = AttributeChannel;

impl Default for AttributeChannel {
    fn default() -> Self {
        AttributeChannel::Neutral
    }
}

/// Context about the object being evaluated by this node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectContext {
    pub query: String,
    pub subject: String,
    /// Universal attributes (replaces elp_tensor)
    pub attributes: Attributes,
    pub keywords: Vec<String>,
    pub semantic_matches: u32,
    pub timestamp: DateTime<Utc>,
}

/// Pattern of interaction with other nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionPattern {
    pub with_position: u8,
    pub interaction_type: InteractionType,
    pub frequency: u32,
    pub avg_confidence: f32,
    pub last_interaction: DateTime<Utc>,
}

/// Type of interaction between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    VortexFlow,         // Normal 1→2→4→8→7→5→1 flow
    SacredCheckpoint,   // Flow through 3, 6, or 9
    CrossSubject,       // Reference to another subject
    BackwardCorrection, // Halving sequence correction
}

/// Snapshot of confidence at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub confidence: f32,
    pub object_type: String,
    pub adjustment_applied: Option<String>,
}

/// Enhanced node dynamics with loop and order awareness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDynamics {
    // === VORTEX LOOP AWARENESS ===
    #[serde(default)]
    pub vortex_position: VortexPosition,
    #[serde(default)]
    pub loop_iteration: u32,
    #[serde(default)]
    pub sequence_confidence: f32,
    
    // === ORDER OF OPERATIONS CONTEXT ===
    #[serde(default)]
    pub order_role: OrderRole,
    #[serde(default)]
    pub attribute_channel: AttributeChannel,
    #[serde(default)]
    pub is_sacred: bool,
    #[serde(default = "default_multiplier")]
    pub sacred_multiplier: f32,
    
    // === OBJECT-RELATIVE EVALUATION ===
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_object: Option<ObjectContext>,
    #[serde(default)]
    pub object_confidence: f32,
    #[serde(default)]
    pub object_fit_score: f32,
    #[serde(default)]
    pub last_evaluation: Option<DateTime<Utc>>,
    
    // === DYNAMIC ADJUSTMENTS (existing fields) ===
    #[serde(default = "default_evolution_rate")]
    pub evolution_rate: f32,
    #[serde(default = "default_stability")]
    pub stability_index: f32,
    #[serde(default)]
    pub interaction_patterns: Vec<InteractionPattern>,
    #[serde(default)]
    pub learning_adjustments: Vec<LearningAdjustment>,
    #[serde(default)]
    pub confidence_history: Vec<ConfidenceSnapshot>,
}

// Default value functions for serde
fn default_multiplier() -> f32 { 1.0 }
fn default_evolution_rate() -> f32 { 1.0 }
fn default_stability() -> f32 { 0.5 }

impl Default for NodeDynamics {
    fn default() -> Self {
        Self {
            vortex_position: VortexPosition::default(),
            loop_iteration: 0,
            sequence_confidence: 0.0,
            order_role: OrderRole::default(),
            attribute_channel: AttributeChannel::default(),
            is_sacred: false,
            sacred_multiplier: 1.0,
            current_object: None,
            object_confidence: 0.0,
            object_fit_score: 0.0,
            last_evaluation: None,
            evolution_rate: 1.0,
            stability_index: 0.5,
            interaction_patterns: Vec::new(),
            learning_adjustments: Vec::new(),
            confidence_history: Vec::new(),
        }
    }
}

/// Learning adjustment for RL improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningAdjustment {
    pub timestamp: DateTime<Utc>,
    pub adjustment_type: AdjustmentType,
    pub magnitude: f32,
    pub rationale: String,
}

/// Source of semantic associations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssociationSource {
    UserInput,
    AIGenerated,
    MachineLearning,
    ManualCuration,
    ReinforcementLearning,
    SubspaceAnalysis,  // Derived from signal subspace interventions
}

/// Types of connections between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Sequential, // Following flux pattern
    Sacred,     // Connection to sacred guide
    Semantic,   // Meaning-based connection
    Geometric,  // Spatial relationship
    Temporal,   // Time-based sequence
    Subspace,   // Signal-based connection for context preservation
}

/// Types of learning adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdjustmentType {
    ConfidenceBoost,
    ConfidenceReduction,
    SemanticRefinement,
    ConnectionStrengthening,
    ConnectionWeakening,
    NodeActivation,
    NodeDeactivation,
    SubspaceMagnification,  // For hallucination mitigation at sacred positions
}

/// Input for inference - now supports both legacy seeds and compression hashes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceInput {
    /// Compression hashes (preferred method)
    #[serde(default)]
    pub compression_hashes: Vec<String>,
    /// Legacy seed numbers (for backward compatibility)
    #[serde(default)]
    pub seed_numbers: Vec<u64>,
    pub subject_filter: SubjectFilter,
    pub processing_options: ProcessingOptions,
}

/// Legacy seed number input (deprecated - use InferenceInput with compression_hashes)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(
    since = "0.2.0",
    note = "Use InferenceInput with compression_hashes instead"
)]
pub struct SeedInput {
    pub seed_numbers: Vec<u64>,
    pub subject_filter: SubjectFilter,
    pub processing_options: ProcessingOptions,
}

// Allow deprecated usage in conversion - this IS the migration path
#[allow(deprecated)]
impl From<SeedInput> for InferenceInput {
    fn from(seed_input: SeedInput) -> Self {
        Self {
            compression_hashes: Vec::new(),
            seed_numbers: seed_input.seed_numbers,
            subject_filter: seed_input.subject_filter,
            processing_options: seed_input.processing_options,
        }
    }
}

/// Subject filtering options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubjectFilter {
    Specific(String),
    GeneralIntelligence,
    Category(String),
    All,
}

/// Processing options for inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOptions {
    pub include_synonyms: bool,
    pub include_antonyms: bool,
    pub max_depth: u8,
    pub confidence_threshold: f32,
    pub use_sacred_guides: bool,
}

/// Inference result output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub id: Uuid,
    pub input: InferenceInput,
    pub matched_matrices: Vec<FluxMatrix>,
    pub inferred_meanings: Vec<InferredMeaning>,
    pub confidence_score: f32,
    pub processing_time_ms: u64,
    pub created_at: DateTime<Utc>,
    /// Hash metadata from compression (if using hashes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_metadata: Option<Vec<HashMetadata>>,
}

/// Metadata extracted from compression hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashMetadata {
    pub hash_hex: String,
    pub flux_position: u8,
    /// Universal attributes (replaces elp_channels)
    pub attributes: Attributes,
    pub rgb_color: (u8, u8, u8),
    pub is_sacred: bool,
    pub confidence: f32,
}

impl Default for HashMetadata {
    fn default() -> Self {
        Self {
            hash_hex: String::new(),
            flux_position: 0,
            attributes: Attributes::new(),
            rgb_color: (0, 0, 0),
            is_sacred: false,
            confidence: 0.0,
        }
    }
}

/// ELP channel values
/// DEPRECATED: Use Attributes system instead
#[deprecated(since = "0.3.0", note = "Use Attributes system. Access via attributes.elp_tensor()")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ELPValues {
    pub ethos: f32,
    pub logos: f32,
    pub pathos: f32,
}

impl ELPValues {
    /// Convert to Attributes
    pub fn to_attributes(&self) -> Attributes {
        Attributes::with_elp(self.ethos, self.logos, self.pathos)
    }
    
    /// Create from Attributes
    pub fn from_attributes(attrs: &Attributes) -> Self {
        let elp = attrs.elp_tensor();
        Self { ethos: elp[0], logos: elp[1], pathos: elp[2] }
    }
}

/// Individual inferred meaning from the process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredMeaning {
    pub subject: String,
    pub node_position: u8,
    pub primary_meaning: String,
    pub semantic_associations: Vec<SemanticAssociation>,
    pub contextual_relevance: f32,
    pub moral_alignment: MoralAlignment,
}

/// Moral alignment for ethical AI reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoralAlignment {
    Constructive(f32), // Heaven - positive influence
    Destructive(f32),  // Hell - negative influence
    Neutral,           // Balanced state
}

/// Database model for storing flux matrices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxMatrixRecord {
    pub id: Uuid,
    pub subject: String,
    pub matrix_data: serde_json::Value,
    pub version: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub source: String, // "manual", "ai_generated", "imported"
}

/// Cache entry for subject matrix patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub matrix: FluxMatrix,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Voice-to-Space Pipeline: Tensor Structures
// ============================================================================

/// BeamTensor: Light-based word representation for AGI consciousness
/// Words become beams of colored light flowing through the flux pattern
/// Renamed from BeadTensor to better represent the visual nature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamTensor {
    /// Position distribution across digits 1-9 (softmax probabilities)
    pub digits: [f32; 9],
    
    /// Universal attributes (includes ethos, logos, pathos and more)
    pub attributes: Attributes,
    
    /// Legacy fields for backward compatibility - use attributes instead
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(since = "0.3.0", note = "Use attributes.ethos() instead")]
    pub ethos: Option<f32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(since = "0.3.0", note = "Use attributes.logos() instead")]
    pub logos: Option<f32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(since = "0.3.0", note = "Use attributes.pathos() instead")]
    pub pathos: Option<f32>,
    
    /// Curvature signed: amplitude × sign(pitch_slope) for path bending
    pub curviness_signed: f32,
    
    /// Unix timestamp of capture
    pub timestamp: f64,
    
    /// Confidence/quality score (0.0-1.0)
    /// CONSOLIDATED: Replaces both previous confidence and confidence
    /// Measures trustworthiness, signal preservation, and hallucination resistance
    /// Threshold: ≥0.6 for Confidence Lake storage
    pub confidence: f32,
    
    /// The actual word this beam represents
    pub word: String,
    
    /// Current position in flux pattern (0-9)
    pub position: u8,
    
    /// Can this word replicate at high confidence?
    pub can_replicate: bool,
    
    /// Should be stored in Confidence Lake?
    pub mark_for_confidence_lake: bool,

    /// Accumulated calculation depth to prevent numeric wrapping
    pub calculation_depth: u64,

    /// Current overflow risk assessment
    pub overflow_risk: OverflowRisk,
}

/// Alias for backward compatibility
pub type BeadTensor = BeamTensor;

// Implement AttributeAccessor trait for BeamTensor
impl AttributeAccessor for BeamTensor {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
    
    fn attributes_mut(&mut self) -> &mut Attributes {
        &mut self.attributes
    }
}

impl BeamTensor {
    /// Create a new BeamTensor with default values
    pub fn default() -> Self {
        Self {
            digits: [1.0 / 9.0; 9],  // Uniform distribution
            attributes: Attributes::new(),
            ethos: None,
            logos: None,
            pathos: None,
            curviness_signed: 0.0,
            timestamp: 0.0,
            confidence: 0.5,  // Neutral default (CONSOLIDATED)
            word: String::new(),
            position: 1,  // Default start position
            can_replicate: false,
            mark_for_confidence_lake: false,
            calculation_depth: 0,
            overflow_risk: OverflowRisk::Safe,
        }
    }
    
    /// Fuse from separate ELP channel tensors
    /// Each channel tensor is [f32; 9] representing position distributions
    pub fn fuse_from_channels(
        ethos_tensor: &[f32; 9],
        logos_tensor: &[f32; 9],
        pathos_tensor: &[f32; 9],
        pitch_slope: f32,
        amplitude: f32,
    ) -> Self {
        // Average the three channel distributions for digit fusion
        let mut fused_digits = [0.0; 9];
        for i in 0..9 {
            fused_digits[i] = (ethos_tensor[i] + logos_tensor[i] + pathos_tensor[i]) / 3.0;
        }
        
        // Calculate channel masses (decisiveness via entropy)
        let ethos_mass = Self::calculate_mass(ethos_tensor);
        let logos_mass = Self::calculate_mass(logos_tensor);
        let pathos_mass = Self::calculate_mass(pathos_tensor);
        
        // Curviness: amplitude × sign of pitch slope
        let curviness = amplitude * pitch_slope.signum();
        
        let mut attributes = Attributes::new();
        attributes.set_elp_tensor([ethos_mass * 9.0, logos_mass * 9.0, pathos_mass * 9.0]);
        
        Self {
            digits: fused_digits,
            attributes,
            ethos: Some(ethos_mass * 9.0),
            logos: Some(logos_mass * 9.0),
            pathos: Some(pathos_mass * 9.0),
            curviness_signed: curviness,
            timestamp: chrono::Utc::now().timestamp() as f64,
            confidence: (ethos_mass + logos_mass + pathos_mass) / 3.0,  // CONSOLIDATED metric
            word: String::new(),  // Will be set by caller
            position: 1,  // Default start position
            can_replicate: false,
            mark_for_confidence_lake: false,
            calculation_depth: 0,
            overflow_risk: OverflowRisk::Safe,
        }
    }
    
    /// Calculate mass (decisiveness) from distribution using entropy
    /// mass = 1 - entropy(softmax) / log(9)
    /// Higher mass = more decisive/focused distribution
    fn calculate_mass(distribution: &[f32; 9]) -> f32 {
        let sum: f32 = distribution.iter().sum();
        if sum < 1e-6 {
            return 0.0;
        }
        
        // Normalize to probabilities
        let probs: Vec<f32> = distribution.iter().map(|&x| x / sum).collect();
        
        // Calculate Shannon entropy
        let entropy: f32 = probs.iter()
            .filter(|&&p| p > 1e-6)
            .map(|&p| -p * p.ln())
            .sum();
        
        // Normalize by max entropy (log 9) and invert
        let max_entropy = 9.0_f32.ln();
        1.0 - (entropy / max_entropy)
    }
    
    /// Get the dominant digit position (argmax)
    pub fn dominant_position(&self) -> u8 {
        self.digits.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| (idx + 1) as u8)
            .unwrap_or(1)
    }
    
    /// Check if this is a "diamond moment" worthy of Confidence Lake
    /// Criteria: ethos ≥ 8.5 AND logos ≥ 7.0 AND down-tone (negative curviness)
    pub fn is_diamond_moment(&self) -> bool {
        self.attributes.ethos() >= 8.5 && self.attributes.logos() >= 7.0 && self.curviness_signed < 0.0
    }
    
    /// Get ethos value (convenience method)
    pub fn ethos(&self) -> f32 {
        self.attributes.ethos()
    }
    
    /// Get logos value (convenience method)
    pub fn logos(&self) -> f32 {
        self.attributes.logos()
    }
    
    /// Get pathos value (convenience method)
    pub fn pathos(&self) -> f32 {
        self.attributes.pathos()
    }
    
    /// Set ethos value (convenience method)
    pub fn set_ethos(&mut self, value: f32) {
        self.attributes.set_ethos(value);
        self.ethos = Some(value);
    }
    
    /// Set logos value (convenience method)
    pub fn set_logos(&mut self, value: f32) {
        self.attributes.set_logos(value);
        self.logos = Some(value);
    }
    
    /// Set pathos value (convenience method)
    pub fn set_pathos(&mut self, value: f32) {
        self.attributes.set_pathos(value);
        self.pathos = Some(value);
    }
}

impl Default for BeamTensor {
    fn default() -> Self {
        Self::default()
    }
}

/// StoredFluxMatrix: Rich memory structure for high-confidence moments
/// Stored encrypted in Confidence Lake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFluxMatrix {
    pub id: Uuid,
    
    /// Universal attributes with full channel distributions
    pub attributes: Attributes,
    
    /// Legacy channel distributions for backward compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethos_distribution: Option<[f32; 9]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logos_distribution: Option<[f32; 9]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pathos_distribution: Option<[f32; 9]>,
    
    /// Pitch curve over time (frequency samples)
    pub pitch_curve: Vec<f32>,
    
    /// Transcribed text from STT
    pub text: String,
    
    /// Associated BeadTensor snapshot
    pub tensor: BeadTensor,
    
    /// Model version used for inference
    pub model_version: String,
    
    /// Metadata
    pub created_at: DateTime<Utc>,
    pub context_tags: Vec<String>,
}

/// Alias for backward compatibility (deprecated: use StoredFluxMatrix)
#[deprecated(note = "Use StoredFluxMatrix instead - aligns with Flux Matrix terminology")]
pub type Diamond = StoredFluxMatrix;
