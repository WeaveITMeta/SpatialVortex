//! Inference Layer
//!
//! The second layer - routes, maps, and contextualizes data instantly.
//!
//! ## Responsibilities
//!
//! - **Intelligent Routing**: Direct data to appropriate processing pipelines
//! - **Field Mapping**: Map source fields to target properties
//! - **Modality Detection**: Identify text, image, video, 3D, time-series, etc.
//! - **Context Injection**: Add tags, attributes, and domain context
//! - **Schema Validation**: Validate against expected schemas

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;

use super::data_types::{DataEnvelope, UniversalData, ContextInfo, ProcessingStage};
use super::PipelineError;

/// Detected modality types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModalityType {
    #[default]
    Unknown,
    // Text modalities
    PlainText,
    Markdown,
    Code,
    Html,
    // Structured data
    Json,
    Xml,
    Csv,
    Tabular,
    // Media
    Image,
    Audio,
    Video,
    // Spatial
    PointCloud,
    Mesh3D,
    Voxel,
    // Time-based
    TimeSeries,
    EventStream,
    // Graph
    Graph,
    Tree,
    // Composite
    MultiModal,
    Document,
}

impl ModalityType {
    /// Get the primary processing category
    pub fn category(&self) -> &'static str {
        match self {
            ModalityType::PlainText | ModalityType::Markdown | 
            ModalityType::Code | ModalityType::Html => "text",
            ModalityType::Json | ModalityType::Xml | 
            ModalityType::Csv | ModalityType::Tabular => "structured",
            ModalityType::Image => "vision",
            ModalityType::Audio => "audio",
            ModalityType::Video => "video",
            ModalityType::PointCloud | ModalityType::Mesh3D | 
            ModalityType::Voxel => "spatial",
            ModalityType::TimeSeries | ModalityType::EventStream => "temporal",
            ModalityType::Graph | ModalityType::Tree => "graph",
            ModalityType::MultiModal | ModalityType::Document => "composite",
            ModalityType::Unknown => "unknown",
        }
    }
    
    /// Check if this modality requires embedding
    pub fn requires_embedding(&self) -> bool {
        matches!(self, 
            ModalityType::PlainText | ModalityType::Markdown | ModalityType::Code |
            ModalityType::Image | ModalityType::Audio
        )
    }
}

/// Field mapping from source to target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Source field path (e.g., "patient.name.given[0]")
    pub source_path: String,
    /// Target property name
    pub target_property: String,
    /// Target type (Attribute, Color, Position, etc.)
    pub target_type: MappingTargetType,
    /// Optional transform expression
    pub transform: Option<String>,
    /// Default value if source missing
    pub default_value: Option<String>,
    /// Whether this mapping is required
    pub required: bool,
}

impl FieldMapping {
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source_path: source.into(),
            target_property: target.into(),
            target_type: MappingTargetType::Attribute,
            transform: None,
            default_value: None,
            required: false,
        }
    }
    
    pub fn with_transform(mut self, transform: impl Into<String>) -> Self {
        self.transform = Some(transform.into());
        self
    }
    
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Target types for field mappings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MappingTargetType {
    #[default]
    Attribute,
    Color,
    Material,
    Transparency,
    Size,
    Position,
    Rotation,
    Name,
    Visible,
    Tag,
    ELP,  // Ethos, Logos, Pathos
}

/// Routing decision made by the inference layer
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// Detected modality
    pub modality: ModalityType,
    /// Target pipeline(s)
    pub pipelines: Vec<String>,
    /// Priority (higher = process first)
    pub priority: i32,
    /// Confidence in the routing decision (0.0 - 1.0)
    pub confidence: f32,
    /// Reasoning for the decision
    pub reasoning: String,
    /// Suggested processing parameters
    pub parameters: HashMap<String, String>,
}

impl Default for RoutingDecision {
    fn default() -> Self {
        Self {
            modality: ModalityType::Unknown,
            pipelines: vec!["default".to_string()],
            priority: 0,
            confidence: 0.5,
            reasoning: String::new(),
            parameters: HashMap::new(),
        }
    }
}

/// Contextual metadata added during inference
#[derive(Debug, Clone, Default)]
pub struct ContextualMetadata {
    /// Domain classification
    pub domain: String,
    /// Tags for filtering and routing
    pub tags: Vec<String>,
    /// Key-value attributes
    pub attributes: HashMap<String, String>,
    /// Detected language (for text)
    pub language: Option<String>,
    /// Detected schema/format
    pub schema: Option<String>,
    /// ELP tensor estimate [ethos, logos, pathos]
    pub elp_estimate: Option<[f32; 3]>,
    /// Signal strength estimate
    pub confidence: f32,
}

/// Inference layer configuration
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// Enable automatic modality detection
    pub auto_detect_modality: bool,
    /// Enable ELP estimation
    pub estimate_elp: bool,
    /// Default domain if not detected
    pub default_domain: String,
    /// Minimum confidence for routing
    pub min_routing_confidence: f32,
    /// Enable schema validation
    pub validate_schema: bool,
    /// Maximum field mappings per envelope
    pub max_mappings: usize,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            auto_detect_modality: true,
            estimate_elp: true,
            default_domain: "general".to_string(),
            min_routing_confidence: 0.3,
            validate_schema: true,
            max_mappings: 100,
        }
    }
}

/// Domain configuration for routing
#[derive(Debug, Clone)]
pub struct DomainConfig {
    /// Domain name
    pub name: String,
    /// Description
    pub description: String,
    /// Default field mappings
    pub default_mappings: Vec<FieldMapping>,
    /// Preferred pipelines
    pub pipelines: Vec<String>,
    /// Keywords for detection
    pub keywords: Vec<String>,
    /// ELP bias [ethos, logos, pathos]
    pub elp_bias: [f32; 3],
}

impl Default for DomainConfig {
    fn default() -> Self {
        Self {
            name: "general".to_string(),
            description: "General purpose domain".to_string(),
            default_mappings: Vec::new(),
            pipelines: vec!["default".to_string()],
            keywords: Vec::new(),
            elp_bias: [0.33, 0.34, 0.33],
        }
    }
}

/// The Inference Layer
pub struct InferenceLayer {
    config: InferenceConfig,
    /// Registered domains
    domains: RwLock<HashMap<String, DomainConfig>>,
    /// Field mapping registry
    mappings: RwLock<HashMap<String, Vec<FieldMapping>>>,
    /// Routing rules
    routing_rules: RwLock<Vec<RoutingRule>>,
}

/// Routing rule for intelligent dispatch
#[derive(Debug, Clone)]
pub struct RoutingRule {
    /// Rule name
    pub name: String,
    /// Condition to match
    pub condition: RoutingCondition,
    /// Target pipelines if matched
    pub pipelines: Vec<String>,
    /// Priority boost
    pub priority_boost: i32,
}

/// Conditions for routing rules
#[derive(Debug, Clone)]
pub enum RoutingCondition {
    /// Match by modality
    Modality(ModalityType),
    /// Match by source type
    SourceType(String),
    /// Match by domain
    Domain(String),
    /// Match by tag
    HasTag(String),
    /// Match by content pattern (regex)
    ContentPattern(String),
    /// Match by attribute value
    Attribute { key: String, value: String },
    /// Combine conditions with AND
    And(Vec<RoutingCondition>),
    /// Combine conditions with OR
    Or(Vec<RoutingCondition>),
    /// Always match
    Always,
}

impl InferenceLayer {
    /// Create new inference layer
    pub fn new(config: InferenceConfig) -> Self {
        let layer = Self {
            config,
            domains: RwLock::new(HashMap::new()),
            mappings: RwLock::new(HashMap::new()),
            routing_rules: RwLock::new(Vec::new()),
        };
        
        // Register default domains
        layer.register_default_domains();
        
        layer
    }
    
    /// Register default domains
    fn register_default_domains(&self) {
        let domains = vec![
            DomainConfig {
                name: "healthcare".to_string(),
                description: "Healthcare and medical data".to_string(),
                keywords: vec!["patient".into(), "fhir".into(), "hl7".into(), "diagnosis".into()],
                elp_bias: [0.5, 0.3, 0.2], // Ethics-heavy
                ..Default::default()
            },
            DomainConfig {
                name: "iot".to_string(),
                description: "IoT and sensor data".to_string(),
                keywords: vec!["sensor".into(), "device".into(), "telemetry".into(), "mqtt".into()],
                elp_bias: [0.2, 0.6, 0.2], // Logic-heavy
                ..Default::default()
            },
            DomainConfig {
                name: "finance".to_string(),
                description: "Financial data".to_string(),
                keywords: vec!["transaction".into(), "payment".into(), "account".into(), "balance".into()],
                elp_bias: [0.4, 0.5, 0.1], // Ethics + Logic
                ..Default::default()
            },
            DomainConfig {
                name: "social".to_string(),
                description: "Social and communication data".to_string(),
                keywords: vec!["message".into(), "post".into(), "comment".into(), "user".into()],
                elp_bias: [0.2, 0.2, 0.6], // Pathos-heavy
                ..Default::default()
            },
        ];
        
        let mut domain_map = self.domains.write();
        for domain in domains {
            domain_map.insert(domain.name.clone(), domain);
        }
    }
    
    /// Register a domain
    pub fn register_domain(&self, domain: DomainConfig) {
        self.domains.write().insert(domain.name.clone(), domain);
    }
    
    /// Register field mappings for a source
    pub fn register_mappings(&self, source_id: impl Into<String>, mappings: Vec<FieldMapping>) {
        self.mappings.write().insert(source_id.into(), mappings);
    }
    
    /// Add a routing rule
    pub fn add_routing_rule(&self, rule: RoutingRule) {
        self.routing_rules.write().push(rule);
    }
    
    /// Route and contextualize data
    pub async fn route(&self, mut envelope: DataEnvelope) -> Result<DataEnvelope, PipelineError> {
        // Detect modality
        let modality = self.detect_modality(&envelope.data);
        
        // Detect domain
        let domain = self.detect_domain(&envelope);
        
        // Estimate ELP if enabled
        let elp = if self.config.estimate_elp {
            self.estimate_elp(&envelope, &domain)
        } else {
            None
        };
        
        // Make routing decision
        let routing = self.make_routing_decision(&envelope, modality, &domain);
        
        // Apply field mappings
        let mappings = self.apply_mappings(&envelope);
        
        // Update envelope context
        envelope.context = ContextInfo {
            modality: format!("{:?}", modality),
            domain: domain.clone(),
            tags: self.generate_tags(&envelope, &domain),
            attributes: self.extract_attributes(&envelope),
            mappings,
            routing: Some(routing.pipelines.join(",")),
        };
        
        envelope.elp = elp;
        envelope.confidence = routing.confidence;
        envelope.priority = routing.priority;
        envelope.advance_stage(ProcessingStage::Inference);
        
        Ok(envelope)
    }
    
    /// Detect modality from data
    fn detect_modality(&self, data: &UniversalData) -> ModalityType {
        match data {
            UniversalData::Text(text) => {
                if text.contains("```") || text.contains("fn ") || text.contains("def ") {
                    ModalityType::Code
                } else if text.starts_with('#') || text.contains("\n## ") {
                    ModalityType::Markdown
                } else if text.starts_with("<!DOCTYPE") || text.starts_with("<html") {
                    ModalityType::Html
                } else {
                    ModalityType::PlainText
                }
            }
            UniversalData::Binary(bytes) => {
                // Check magic bytes
                if bytes.len() >= 4 {
                    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) || 
                       bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                        return ModalityType::Image;
                    }
                    if bytes.starts_with(b"ID3") || bytes.starts_with(b"RIFF") {
                        return ModalityType::Audio;
                    }
                }
                ModalityType::Unknown
            }
            UniversalData::Structured(json) => {
                if json.is_array() {
                    ModalityType::Tabular
                } else {
                    ModalityType::Json
                }
            }
            UniversalData::Tensor(_) => ModalityType::Unknown,
            UniversalData::TimeSeries(_) => ModalityType::TimeSeries,
            UniversalData::Graph(_) => ModalityType::Graph,
            UniversalData::Spatial3D(_) => ModalityType::PointCloud,
            UniversalData::Composite(_) => ModalityType::MultiModal,
            UniversalData::Empty => ModalityType::Unknown,
        }
    }
    
    /// Detect domain from envelope
    fn detect_domain(&self, envelope: &DataEnvelope) -> String {
        let domains = self.domains.read();
        
        // Check source type hints
        let source_type = &envelope.source.source_type.to_lowercase();
        if source_type.contains("fhir") || source_type.contains("hl7") {
            return "healthcare".to_string();
        }
        if source_type.contains("mqtt") || source_type.contains("opcua") {
            return "iot".to_string();
        }
        
        // Check content for keywords
        let content_str = match &envelope.data {
            UniversalData::Text(t) => t.to_lowercase(),
            UniversalData::Structured(j) => j.to_string().to_lowercase(),
            _ => String::new(),
        };
        
        for (name, config) in domains.iter() {
            for keyword in &config.keywords {
                if content_str.contains(&keyword.to_lowercase()) {
                    return name.clone();
                }
            }
        }
        
        self.config.default_domain.clone()
    }
    
    /// Estimate ELP tensor
    fn estimate_elp(&self, envelope: &DataEnvelope, domain: &str) -> Option<[f32; 3]> {
        let domains = self.domains.read();
        
        // Start with domain bias
        let mut elp = domains.get(domain)
            .map(|d| d.elp_bias)
            .unwrap_or([0.33, 0.34, 0.33]);
        
        // Adjust based on content analysis
        if let UniversalData::Text(text) = &envelope.data {
            // Simple heuristic: question marks increase logos, exclamation increases pathos
            let question_ratio = text.matches('?').count() as f32 / (text.len() as f32 + 1.0);
            let exclaim_ratio = text.matches('!').count() as f32 / (text.len() as f32 + 1.0);
            
            elp[1] += question_ratio * 10.0; // Logos boost for questions
            elp[2] += exclaim_ratio * 10.0;  // Pathos boost for exclamations
            
            // Normalize
            let sum = elp[0] + elp[1] + elp[2];
            if sum > 0.0 {
                elp[0] /= sum;
                elp[1] /= sum;
                elp[2] /= sum;
            }
        }
        
        Some(elp)
    }
    
    /// Make routing decision
    fn make_routing_decision(&self, envelope: &DataEnvelope, modality: ModalityType, domain: &str) -> RoutingDecision {
        let mut decision = RoutingDecision {
            modality,
            pipelines: vec![modality.category().to_string()],
            confidence: 0.7,
            reasoning: format!("Detected modality {:?} in domain {}", modality, domain),
            ..Default::default()
        };
        
        // Apply routing rules
        let rules = self.routing_rules.read();
        for rule in rules.iter() {
            if self.matches_condition(&rule.condition, envelope, modality, domain) {
                decision.pipelines.extend(rule.pipelines.clone());
                decision.priority += rule.priority_boost;
                decision.confidence = (decision.confidence + 0.1).min(1.0);
            }
        }
        
        decision
    }
    
    /// Check if routing condition matches
    fn matches_condition(&self, condition: &RoutingCondition, envelope: &DataEnvelope, modality: ModalityType, domain: &str) -> bool {
        match condition {
            RoutingCondition::Modality(m) => *m == modality,
            RoutingCondition::SourceType(s) => envelope.source.source_type.to_lowercase().contains(&s.to_lowercase()),
            RoutingCondition::Domain(d) => domain == d,
            RoutingCondition::HasTag(t) => envelope.context.tags.contains(t),
            RoutingCondition::ContentPattern(pattern) => {
                if let UniversalData::Text(text) = &envelope.data {
                    text.contains(pattern)
                } else {
                    false
                }
            }
            RoutingCondition::Attribute { key, value } => {
                envelope.context.attributes.get(key).map(|v| v == value).unwrap_or(false)
            }
            RoutingCondition::And(conditions) => {
                conditions.iter().all(|c| self.matches_condition(c, envelope, modality, domain))
            }
            RoutingCondition::Or(conditions) => {
                conditions.iter().any(|c| self.matches_condition(c, envelope, modality, domain))
            }
            RoutingCondition::Always => true,
        }
    }
    
    /// Apply field mappings
    fn apply_mappings(&self, envelope: &DataEnvelope) -> Vec<(String, String)> {
        let mappings = self.mappings.read();
        let mut result = Vec::new();
        
        if let Some(source_mappings) = mappings.get(&envelope.source.resource_id) {
            for mapping in source_mappings {
                // Extract value from source path
                if let Some(value) = self.extract_value(&envelope.data, &mapping.source_path) {
                    result.push((mapping.source_path.clone(), mapping.target_property.clone()));
                }
            }
        }
        
        result
    }
    
    /// Extract value from data using path
    fn extract_value(&self, data: &UniversalData, path: &str) -> Option<String> {
        match data {
            UniversalData::Structured(json) => {
                // Simple JSON path extraction
                let parts: Vec<&str> = path.split('.').collect();
                let mut current = json;
                
                for part in parts {
                    if let Some(obj) = current.as_object() {
                        current = obj.get(part)?;
                    } else if let Some(arr) = current.as_array() {
                        let idx: usize = part.parse().ok()?;
                        current = arr.get(idx)?;
                    } else {
                        return None;
                    }
                }
                
                Some(current.to_string())
            }
            _ => None,
        }
    }
    
    /// Generate tags for envelope
    fn generate_tags(&self, envelope: &DataEnvelope, domain: &str) -> Vec<String> {
        let mut tags = vec![domain.to_string()];
        
        // Add source type tag
        if !envelope.source.source_type.is_empty() {
            tags.push(format!("source:{}", envelope.source.source_type.to_lowercase()));
        }
        
        // Add format tag
        if !envelope.source.format.is_empty() {
            tags.push(format!("format:{}", envelope.source.format));
        }
        
        tags
    }
    
    /// Extract attributes from envelope
    fn extract_attributes(&self, envelope: &DataEnvelope) -> HashMap<String, String> {
        let mut attrs = HashMap::new();
        
        attrs.insert("resource_id".to_string(), envelope.source.resource_id.clone());
        attrs.insert("data_size".to_string(), envelope.data.size_bytes().to_string());
        
        if let Some(version) = &envelope.source.schema_version {
            attrs.insert("schema_version".to_string(), version.clone());
        }
        
        attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_modality_detection() {
        let layer = InferenceLayer::new(InferenceConfig::default());
        
        let text = UniversalData::Text("Hello world".to_string());
        assert_eq!(layer.detect_modality(&text), ModalityType::PlainText);
        
        let markdown = UniversalData::Text("# Heading\n\nParagraph".to_string());
        assert_eq!(layer.detect_modality(&markdown), ModalityType::Markdown);
        
        let code = UniversalData::Text("fn main() {\n    println!(\"Hello\");\n}".to_string());
        assert_eq!(layer.detect_modality(&code), ModalityType::Code);
    }
    
    #[test]
    fn test_domain_detection() {
        let layer = InferenceLayer::new(InferenceConfig::default());
        
        let mut envelope = DataEnvelope::new("test", UniversalData::Text("patient diagnosis".to_string()));
        envelope.source.source_type = "REST".to_string();
        
        let domain = layer.detect_domain(&envelope);
        assert_eq!(domain, "healthcare");
    }
    
    #[test]
    fn test_routing_condition() {
        let layer = InferenceLayer::new(InferenceConfig::default());
        let envelope = DataEnvelope::new("test", UniversalData::Text("test".to_string()));
        
        assert!(layer.matches_condition(&RoutingCondition::Always, &envelope, ModalityType::PlainText, "general"));
        assert!(layer.matches_condition(&RoutingCondition::Modality(ModalityType::PlainText), &envelope, ModalityType::PlainText, "general"));
        assert!(!layer.matches_condition(&RoutingCondition::Modality(ModalityType::Image), &envelope, ModalityType::PlainText, "general"));
    }
}
