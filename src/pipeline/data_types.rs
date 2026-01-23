//! Universal Data Types for Pipeline
//!
//! Core data structures that flow through all pipeline layers.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::data::attributes::{Attributes, AttributeValue, Tags};

/// Processing stage tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessingStage {
    /// Raw input received
    Input,
    /// Routed and contextualized
    Inference,
    /// Transformed by modality
    Processing,
    /// Reasoned and analyzed
    Intelligence,
    /// Output generated
    Output,
    /// Knowledge updated
    Feedback,
}

/// Universal data container that can hold any modality
#[derive(Debug, Clone)]
pub enum UniversalData {
    /// Text content
    Text(String),
    /// Binary data (images, audio, video, etc.)
    Binary(Vec<u8>),
    /// Structured JSON-like data
    Structured(serde_json::Value),
    /// Numeric tensor (for ML operations)
    Tensor(TensorData),
    /// Time series data
    TimeSeries(TimeSeriesData),
    /// Graph/network data
    Graph(GraphData),
    /// 3D spatial data
    Spatial3D(Spatial3DData),
    /// Multi-modal composite
    Composite(Vec<UniversalData>),
    /// Empty/null
    Empty,
}

impl UniversalData {
    /// Get the modality type of this data
    pub fn modality(&self) -> &'static str {
        match self {
            UniversalData::Text(_) => "text",
            UniversalData::Binary(_) => "binary",
            UniversalData::Structured(_) => "structured",
            UniversalData::Tensor(_) => "tensor",
            UniversalData::TimeSeries(_) => "timeseries",
            UniversalData::Graph(_) => "graph",
            UniversalData::Spatial3D(_) => "spatial3d",
            UniversalData::Composite(_) => "composite",
            UniversalData::Empty => "empty",
        }
    }
    
    /// Get approximate size in bytes
    pub fn size_bytes(&self) -> usize {
        match self {
            UniversalData::Text(s) => s.len(),
            UniversalData::Binary(b) => b.len(),
            UniversalData::Structured(v) => v.to_string().len(),
            UniversalData::Tensor(t) => t.data.len() * 4,
            UniversalData::TimeSeries(ts) => ts.values.len() * 8,
            UniversalData::Graph(g) => (g.nodes.len() + g.edges.len()) * 32,
            UniversalData::Spatial3D(s) => s.points.len() * 12,
            UniversalData::Composite(c) => c.iter().map(|d| d.size_bytes()).sum(),
            UniversalData::Empty => 0,
        }
    }
    
    /// Check if data is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, UniversalData::Empty)
    }
}

/// Tensor data for ML operations
#[derive(Debug, Clone)]
pub struct TensorData {
    /// Flattened data
    pub data: Vec<f32>,
    /// Shape dimensions
    pub shape: Vec<usize>,
    /// Data type name
    pub dtype: String,
}

impl TensorData {
    pub fn new(data: Vec<f32>, shape: Vec<usize>) -> Self {
        Self {
            data,
            shape,
            dtype: "float32".to_string(),
        }
    }
    
    /// Get total number of elements
    pub fn numel(&self) -> usize {
        self.shape.iter().product()
    }
}

/// Time series data
#[derive(Debug, Clone)]
pub struct TimeSeriesData {
    /// Timestamps (Unix epoch seconds)
    pub timestamps: Vec<f64>,
    /// Values at each timestamp
    pub values: Vec<f64>,
    /// Column names if multi-dimensional
    pub columns: Vec<String>,
    /// Sampling frequency in Hz (if regular)
    pub frequency_hz: Option<f64>,
}

/// Graph/network data
#[derive(Debug, Clone)]
pub struct GraphData {
    /// Node IDs and attributes
    pub nodes: Vec<GraphNode>,
    /// Edges between nodes
    pub edges: Vec<GraphEdge>,
    /// Graph-level attributes
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub weight: f32,
    pub edge_type: Option<String>,
}

/// 3D spatial data
#[derive(Debug, Clone)]
pub struct Spatial3DData {
    /// Point cloud [x, y, z]
    pub points: Vec<[f32; 3]>,
    /// Optional colors per point [r, g, b, a]
    pub colors: Option<Vec<[f32; 4]>>,
    /// Optional normals per point
    pub normals: Option<Vec<[f32; 3]>>,
    /// Mesh faces (indices into points)
    pub faces: Option<Vec<[usize; 3]>>,
    /// Bounding box [min, max]
    pub bounds: Option<([f32; 3], [f32; 3])>,
}

/// Data envelope wrapping universal data with metadata
#[derive(Debug, Clone)]
pub struct DataEnvelope {
    /// Unique identifier for this data unit
    pub id: String,
    /// The actual data
    pub data: UniversalData,
    /// Current processing stage
    pub stage: ProcessingStage,
    /// Source information
    pub source: SourceInfo,
    /// Contextual metadata
    pub context: ContextInfo,
    /// Processing history
    pub history: Vec<ProcessingEvent>,
    /// Universal attributes (EustressEngine compatible)
    /// Includes ELP via attributes.elp_tensor() for backward compatibility
    pub attributes: Attributes,
    /// Tags for categorization (CollectionService-style)
    pub tags: Tags,
    /// ELP tensor (Ethos, Logos, Pathos) - backward compatibility
    /// Prefer using attributes.elp_tensor() for new code
    pub elp: Option<[f32; 3]>,
    /// Signal strength (0.0 - 1.0)
    pub confidence: f32,
    /// Priority (higher = more important)
    pub priority: i32,
    /// Timestamp of creation
    pub created_at: std::time::SystemTime,
}

impl DataEnvelope {
    /// Create a new envelope with raw data
    pub fn new(id: impl Into<String>, data: UniversalData) -> Self {
        Self {
            id: id.into(),
            data,
            stage: ProcessingStage::Input,
            source: SourceInfo::default(),
            context: ContextInfo::default(),
            history: Vec::new(),
            attributes: Attributes::new(),
            tags: Tags::new(),
            elp: None,
            confidence: 1.0,
            priority: 0,
            created_at: std::time::SystemTime::now(),
        }
    }
    
    /// Create with initial attributes
    pub fn with_attributes(id: impl Into<String>, data: UniversalData, attributes: Attributes) -> Self {
        let mut envelope = Self::new(id, data);
        envelope.attributes = attributes;
        envelope
    }
    
    /// Get ELP tensor from attributes (preferred over elp field)
    pub fn get_elp(&self) -> [f32; 3] {
        self.elp.unwrap_or_else(|| self.attributes.elp_tensor())
    }
    
    /// Set ELP tensor in attributes
    pub fn set_elp(&mut self, elp: [f32; 3]) {
        self.attributes.set_elp_tensor(elp);
        self.elp = Some(elp);
    }
    
    /// Get signal strength from attributes or field
    pub fn get_confidence(&self) -> f32 {
        if self.confidence != 1.0 {
            self.confidence
        } else {
            self.attributes.confidence()
        }
    }
    
    /// Set signal strength in both field and attributes
    pub fn set_confidence(&mut self, value: f32) {
        self.confidence = value;
        self.attributes.set_confidence(value);
    }
    
    /// Get attribute value
    pub fn get_attribute(&self, key: &str) -> Option<&AttributeValue> {
        self.attributes.get(key)
    }
    
    /// Set attribute value
    pub fn set_attribute(&mut self, key: impl Into<String>, value: AttributeValue) {
        self.attributes.set(key, value);
    }
    
    /// Check if has tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.has(tag)
    }
    
    /// Add tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.add(tag);
    }
    
    /// Add a processing event to history
    pub fn add_event(&mut self, event: ProcessingEvent) {
        self.history.push(event);
    }
    
    /// Advance to next stage
    pub fn advance_stage(&mut self, new_stage: ProcessingStage) {
        self.add_event(ProcessingEvent {
            stage: self.stage,
            timestamp: std::time::SystemTime::now(),
            duration_ms: 0,
            notes: format!("Advanced from {:?} to {:?}", self.stage, new_stage),
        });
        self.stage = new_stage;
    }
    
    /// Get total processing time
    pub fn total_processing_time(&self) -> Duration {
        self.history.iter()
            .map(|e| Duration::from_millis(e.duration_ms))
            .sum()
    }
}

/// Source information
#[derive(Debug, Clone, Default)]
pub struct SourceInfo {
    /// Source type (REST, Kafka, File, etc.)
    pub source_type: String,
    /// Source endpoint/path
    pub endpoint: String,
    /// Resource identifier
    pub resource_id: String,
    /// Authentication used
    pub auth_type: String,
    /// Original format
    pub format: String,
    /// Schema version
    pub schema_version: Option<String>,
}

/// Contextual information
#[derive(Debug, Clone, Default)]
pub struct ContextInfo {
    /// Detected modality
    pub modality: String,
    /// Domain/category
    pub domain: String,
    /// Tags for filtering
    pub tags: Vec<String>,
    /// Key-value attributes
    pub attributes: HashMap<String, String>,
    /// Field mappings applied
    pub mappings: Vec<(String, String)>,
    /// Routing decision made
    pub routing: Option<String>,
}

/// Processing event for history tracking
#[derive(Debug, Clone)]
pub struct ProcessingEvent {
    /// Stage where event occurred
    pub stage: ProcessingStage,
    /// When it happened
    pub timestamp: std::time::SystemTime,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Notes/description
    pub notes: String,
}

/// Pipeline-wide metrics
#[derive(Debug, Clone, Default)]
pub struct PipelineMetrics {
    /// Total items processed
    pub total_processed: u64,
    /// Total processing time in milliseconds
    pub total_time_ms: u64,
    /// Items by modality
    pub by_modality: HashMap<String, u64>,
    /// Items by source type
    pub by_source: HashMap<String, u64>,
    /// Errors by stage
    pub errors_by_stage: HashMap<String, u64>,
    /// Average processing time per stage
    pub avg_time_by_stage: HashMap<String, f64>,
    /// Knowledge updates performed
    pub knowledge_updates: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
}

impl PipelineMetrics {
    /// Get average processing time per item
    pub fn avg_time_per_item(&self) -> f64 {
        if self.total_processed == 0 {
            0.0
        } else {
            self.total_time_ms as f64 / self.total_processed as f64
        }
    }
    
    /// Get throughput (items per second)
    pub fn throughput(&self) -> f64 {
        if self.total_time_ms == 0 {
            0.0
        } else {
            self.total_processed as f64 / (self.total_time_ms as f64 / 1000.0)
        }
    }
    
    /// Record an error
    pub fn record_error(&mut self, stage: &str) {
        *self.errors_by_stage.entry(stage.to_string()).or_insert(0) += 1;
    }
    
    /// Record modality
    pub fn record_modality(&mut self, modality: &str) {
        *self.by_modality.entry(modality.to_string()).or_insert(0) += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_universal_data_modality() {
        let text = UniversalData::Text("hello".to_string());
        assert_eq!(text.modality(), "text");
        
        let tensor = UniversalData::Tensor(TensorData::new(vec![1.0, 2.0, 3.0], vec![3]));
        assert_eq!(tensor.modality(), "tensor");
    }
    
    #[test]
    fn test_data_envelope() {
        let mut envelope = DataEnvelope::new("test-1", UniversalData::Text("hello".to_string()));
        
        assert_eq!(envelope.stage, ProcessingStage::Input);
        
        envelope.advance_stage(ProcessingStage::Inference);
        assert_eq!(envelope.stage, ProcessingStage::Inference);
        assert_eq!(envelope.history.len(), 1);
    }
    
    #[test]
    fn test_pipeline_metrics() {
        let mut metrics = PipelineMetrics::default();
        metrics.total_processed = 100;
        metrics.total_time_ms = 1000;
        
        assert!((metrics.avg_time_per_item() - 10.0).abs() < 0.01);
        assert!((metrics.throughput() - 100.0).abs() < 0.01);
    }
}
