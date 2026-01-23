//! Processing Layer
//!
//! The third layer - transforms and processes data by modality.
//!
//! ## Responsibilities
//!
//! - **Modality Pipelines**: Text, image, audio, video, 3D, time-series, graph
//! - **Transformations**: Tokenization, embedding, vectorization, normalization
//! - **Preprocessing**: Cleaning, filtering, augmentation
//! - **Feature Extraction**: Extract meaningful features for intelligence layer

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;

use super::data_types::{DataEnvelope, UniversalData, TensorData, ProcessingStage};
use super::inference_layer::ModalityType;
use super::PipelineError;

/// Processing layer configuration
#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    /// Maximum sequence length for text
    pub max_seq_length: usize,
    /// Embedding dimension
    pub embedding_dim: usize,
    /// Enable caching of processed results
    pub enable_cache: bool,
    /// Cache size limit
    pub cache_size: usize,
    /// Enable parallel processing
    pub parallel: bool,
    /// Number of worker threads
    pub num_workers: usize,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            max_seq_length: 512,
            embedding_dim: 384,
            enable_cache: true,
            cache_size: 1000,
            parallel: true,
            num_workers: 4,
        }
    }
}

/// Result of embedding operation
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    /// Embedding vector
    pub embedding: Vec<f32>,
    /// Dimension
    pub dim: usize,
    /// Model used
    pub model: String,
    /// Confidence/quality score
    pub quality: f32,
}

/// Processed data ready for intelligence layer
#[derive(Debug, Clone)]
pub struct ProcessedData {
    /// Original envelope
    pub envelope: DataEnvelope,
    /// Extracted features
    pub features: Features,
    /// Embeddings if computed
    pub embeddings: Option<EmbeddingResult>,
    /// Tokens if text
    pub tokens: Option<Vec<u32>>,
    /// Processing metadata
    pub metadata: ProcessingMetadata,
}

/// Extracted features from data
#[derive(Debug, Clone, Default)]
pub struct Features {
    /// Numeric features
    pub numeric: HashMap<String, f32>,
    /// Categorical features
    pub categorical: HashMap<String, String>,
    /// Vector features
    pub vectors: HashMap<String, Vec<f32>>,
    /// Text features
    pub text: HashMap<String, String>,
    /// Temporal features
    pub temporal: HashMap<String, f64>,
}

impl Features {
    pub fn add_numeric(&mut self, key: impl Into<String>, value: f32) {
        self.numeric.insert(key.into(), value);
    }
    
    pub fn add_categorical(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.categorical.insert(key.into(), value.into());
    }
    
    pub fn add_vector(&mut self, key: impl Into<String>, value: Vec<f32>) {
        self.vectors.insert(key.into(), value);
    }
}

/// Processing metadata
#[derive(Debug, Clone, Default)]
pub struct ProcessingMetadata {
    /// Pipeline used
    pub pipeline: String,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Number of transformations applied
    pub transformations: usize,
    /// Warnings generated
    pub warnings: Vec<String>,
    /// Cache hit
    pub cache_hit: bool,
}

/// Trait for modality-specific pipelines
pub trait ModalityPipeline: Send + Sync {
    /// Get the modality this pipeline handles
    fn modality(&self) -> ModalityType;
    
    /// Process data through the pipeline
    fn process(&self, data: &UniversalData, config: &ProcessingConfig) -> Result<(Features, Option<EmbeddingResult>), PipelineError>;
    
    /// Get pipeline name
    fn name(&self) -> &str;
}

/// Text processing pipeline
pub struct TextPipeline {
    name: String,
}

impl TextPipeline {
    pub fn new() -> Self {
        Self { name: "text_pipeline".to_string() }
    }
    
    /// Simple tokenization (word-based)
    fn tokenize(&self, text: &str, max_len: usize) -> Vec<u32> {
        text.split_whitespace()
            .take(max_len)
            .enumerate()
            .map(|(i, _)| i as u32)
            .collect()
    }
    
    /// Generate simple embedding (placeholder - would use actual model)
    fn embed(&self, text: &str, dim: usize) -> Vec<f32> {
        // Simple hash-based embedding for demonstration
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut embedding = vec![0.0f32; dim];
        
        for (i, word) in text.split_whitespace().enumerate() {
            let mut hasher = DefaultHasher::new();
            word.hash(&mut hasher);
            let hash = hasher.finish();
            
            for j in 0..dim {
                let idx = (hash as usize + i * 7 + j * 13) % dim;
                embedding[idx] += ((hash >> (j % 64)) & 1) as f32 * 0.1;
            }
        }
        
        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }
        
        embedding
    }
}

impl ModalityPipeline for TextPipeline {
    fn modality(&self) -> ModalityType {
        ModalityType::PlainText
    }
    
    fn process(&self, data: &UniversalData, config: &ProcessingConfig) -> Result<(Features, Option<EmbeddingResult>), PipelineError> {
        let text = match data {
            UniversalData::Text(t) => t.clone(),
            _ => return Err(PipelineError::ProcessingError("Expected text data".to_string())),
        };
        
        let mut features = Features::default();
        
        // Extract text features
        features.add_numeric("char_count", text.len() as f32);
        features.add_numeric("word_count", text.split_whitespace().count() as f32);
        features.add_numeric("line_count", text.lines().count() as f32);
        features.add_numeric("avg_word_length", {
            let words: Vec<&str> = text.split_whitespace().collect();
            if words.is_empty() { 0.0 } else {
                words.iter().map(|w| w.len()).sum::<usize>() as f32 / words.len() as f32
            }
        });
        
        // Sentiment indicators (simple heuristic)
        features.add_numeric("question_marks", text.matches('?').count() as f32);
        features.add_numeric("exclamation_marks", text.matches('!').count() as f32);
        features.add_numeric("uppercase_ratio", {
            let upper = text.chars().filter(|c| c.is_uppercase()).count();
            let alpha = text.chars().filter(|c| c.is_alphabetic()).count();
            if alpha == 0 { 0.0 } else { upper as f32 / alpha as f32 }
        });
        
        // Generate embedding
        let embedding = self.embed(&text, config.embedding_dim);
        let embed_result = EmbeddingResult {
            embedding,
            dim: config.embedding_dim,
            model: "simple_hash".to_string(),
            quality: 0.7,
        };
        
        Ok((features, Some(embed_result)))
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Structured data processing pipeline
pub struct StructuredPipeline {
    name: String,
}

impl StructuredPipeline {
    pub fn new() -> Self {
        Self { name: "structured_pipeline".to_string() }
    }
    
    fn extract_json_features(&self, json: &serde_json::Value) -> Features {
        let mut features = Features::default();
        
        match json {
            serde_json::Value::Object(obj) => {
                features.add_numeric("field_count", obj.len() as f32);
                features.add_numeric("depth", self.json_depth(json) as f32);
                
                for (key, value) in obj {
                    match value {
                        serde_json::Value::Number(n) => {
                            if let Some(f) = n.as_f64() {
                                features.add_numeric(key, f as f32);
                            }
                        }
                        serde_json::Value::String(s) => {
                            features.add_categorical(key, s);
                        }
                        serde_json::Value::Bool(b) => {
                            features.add_numeric(key, if *b { 1.0 } else { 0.0 });
                        }
                        _ => {}
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                features.add_numeric("array_length", arr.len() as f32);
            }
            _ => {}
        }
        
        features
    }
    
    fn json_depth(&self, json: &serde_json::Value) -> usize {
        match json {
            serde_json::Value::Object(obj) => {
                1 + obj.values().map(|v| self.json_depth(v)).max().unwrap_or(0)
            }
            serde_json::Value::Array(arr) => {
                1 + arr.iter().map(|v| self.json_depth(v)).max().unwrap_or(0)
            }
            _ => 0,
        }
    }
}

impl ModalityPipeline for StructuredPipeline {
    fn modality(&self) -> ModalityType {
        ModalityType::Json
    }
    
    fn process(&self, data: &UniversalData, _config: &ProcessingConfig) -> Result<(Features, Option<EmbeddingResult>), PipelineError> {
        let json = match data {
            UniversalData::Structured(j) => j.clone(),
            _ => return Err(PipelineError::ProcessingError("Expected structured data".to_string())),
        };
        
        let features = self.extract_json_features(&json);
        
        Ok((features, None))
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Time series processing pipeline
pub struct TimeSeriesPipeline {
    name: String,
}

impl TimeSeriesPipeline {
    pub fn new() -> Self {
        Self { name: "timeseries_pipeline".to_string() }
    }
}

impl ModalityPipeline for TimeSeriesPipeline {
    fn modality(&self) -> ModalityType {
        ModalityType::TimeSeries
    }
    
    fn process(&self, data: &UniversalData, _config: &ProcessingConfig) -> Result<(Features, Option<EmbeddingResult>), PipelineError> {
        let ts = match data {
            UniversalData::TimeSeries(t) => t,
            _ => return Err(PipelineError::ProcessingError("Expected time series data".to_string())),
        };
        
        let mut features = Features::default();
        
        if !ts.values.is_empty() {
            let mean: f64 = ts.values.iter().sum::<f64>() / ts.values.len() as f64;
            let variance: f64 = ts.values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / ts.values.len() as f64;
            let std_dev = variance.sqrt();
            
            features.add_numeric("mean", mean as f32);
            features.add_numeric("std_dev", std_dev as f32);
            features.add_numeric("min", *ts.values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() as f32);
            features.add_numeric("max", *ts.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() as f32);
            features.add_numeric("count", ts.values.len() as f32);
            
            if let Some(freq) = ts.frequency_hz {
                features.add_numeric("frequency_hz", freq as f32);
            }
        }
        
        Ok((features, None))
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Image processing pipeline (placeholder)
pub struct ImagePipeline {
    name: String,
}

impl ImagePipeline {
    pub fn new() -> Self {
        Self { name: "image_pipeline".to_string() }
    }
}

impl ModalityPipeline for ImagePipeline {
    fn modality(&self) -> ModalityType {
        ModalityType::Image
    }
    
    fn process(&self, data: &UniversalData, config: &ProcessingConfig) -> Result<(Features, Option<EmbeddingResult>), PipelineError> {
        let bytes = match data {
            UniversalData::Binary(b) => b,
            _ => return Err(PipelineError::ProcessingError("Expected binary data for image".to_string())),
        };
        
        let mut features = Features::default();
        features.add_numeric("size_bytes", bytes.len() as f32);
        
        // Detect format from magic bytes
        let format = if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            "jpeg"
        } else if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            "png"
        } else if bytes.starts_with(b"GIF8") {
            "gif"
        } else {
            "unknown"
        };
        features.add_categorical("format", format);
        
        // Generate placeholder embedding
        let embedding = vec![0.0f32; config.embedding_dim];
        let embed_result = EmbeddingResult {
            embedding,
            dim: config.embedding_dim,
            model: "placeholder".to_string(),
            quality: 0.5,
        };
        
        Ok((features, Some(embed_result)))
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// The Processing Layer
pub struct ProcessingLayer {
    config: ProcessingConfig,
    /// Registered pipelines by modality
    pipelines: HashMap<String, Arc<dyn ModalityPipeline>>,
    /// Processing cache
    cache: RwLock<HashMap<String, ProcessedData>>,
}

impl ProcessingLayer {
    /// Create new processing layer
    pub fn new(config: ProcessingConfig) -> Self {
        let mut layer = Self {
            config,
            pipelines: HashMap::new(),
            cache: RwLock::new(HashMap::new()),
        };
        
        // Register default pipelines
        layer.register_pipeline(Arc::new(TextPipeline::new()));
        layer.register_pipeline(Arc::new(StructuredPipeline::new()));
        layer.register_pipeline(Arc::new(TimeSeriesPipeline::new()));
        layer.register_pipeline(Arc::new(ImagePipeline::new()));
        
        layer
    }
    
    /// Register a modality pipeline
    pub fn register_pipeline(&mut self, pipeline: Arc<dyn ModalityPipeline>) {
        self.pipelines.insert(pipeline.name().to_string(), pipeline);
    }
    
    /// Transform data through appropriate pipeline
    pub async fn transform(&self, mut envelope: DataEnvelope) -> Result<ProcessedData, PipelineError> {
        let start = std::time::Instant::now();
        
        // Check cache
        if self.config.enable_cache {
            if let Some(cached) = self.cache.read().get(&envelope.id) {
                let mut result = cached.clone();
                result.metadata.cache_hit = true;
                return Ok(result);
            }
        }
        
        // Determine modality from context
        let modality_str = &envelope.context.modality;
        let modality = self.parse_modality(modality_str);
        
        // Select pipeline
        let pipeline = self.select_pipeline(modality);
        
        // Process through pipeline
        let (features, embeddings) = pipeline.process(&envelope.data, &self.config)?;
        
        // Tokenize if text
        let tokens = if modality == ModalityType::PlainText || modality == ModalityType::Markdown || modality == ModalityType::Code {
            if let UniversalData::Text(text) = &envelope.data {
                Some(self.simple_tokenize(text))
            } else {
                None
            }
        } else {
            None
        };
        
        // Update envelope stage
        envelope.advance_stage(ProcessingStage::Processing);
        
        let metadata = ProcessingMetadata {
            pipeline: pipeline.name().to_string(),
            processing_time_ms: start.elapsed().as_millis() as u64,
            transformations: 1,
            warnings: Vec::new(),
            cache_hit: false,
        };
        
        let result = ProcessedData {
            envelope,
            features,
            embeddings,
            tokens,
            metadata,
        };
        
        // Cache result
        if self.config.enable_cache {
            let mut cache = self.cache.write();
            if cache.len() < self.config.cache_size {
                cache.insert(result.envelope.id.clone(), result.clone());
            }
        }
        
        Ok(result)
    }
    
    /// Parse modality string to enum
    fn parse_modality(&self, s: &str) -> ModalityType {
        match s.to_lowercase().as_str() {
            "plaintext" => ModalityType::PlainText,
            "markdown" => ModalityType::Markdown,
            "code" => ModalityType::Code,
            "html" => ModalityType::Html,
            "json" => ModalityType::Json,
            "xml" => ModalityType::Xml,
            "csv" => ModalityType::Csv,
            "tabular" => ModalityType::Tabular,
            "image" => ModalityType::Image,
            "audio" => ModalityType::Audio,
            "video" => ModalityType::Video,
            "timeseries" => ModalityType::TimeSeries,
            "graph" => ModalityType::Graph,
            "pointcloud" => ModalityType::PointCloud,
            _ => ModalityType::Unknown,
        }
    }
    
    /// Select appropriate pipeline for modality
    fn select_pipeline(&self, modality: ModalityType) -> Arc<dyn ModalityPipeline> {
        let pipeline_name = match modality {
            ModalityType::PlainText | ModalityType::Markdown | ModalityType::Code | ModalityType::Html => "text_pipeline",
            ModalityType::Json | ModalityType::Xml | ModalityType::Csv | ModalityType::Tabular => "structured_pipeline",
            ModalityType::TimeSeries | ModalityType::EventStream => "timeseries_pipeline",
            ModalityType::Image => "image_pipeline",
            _ => "text_pipeline", // Default fallback
        };
        
        self.pipelines.get(pipeline_name)
            .cloned()
            .unwrap_or_else(|| Arc::new(TextPipeline::new()))
    }
    
    /// Simple word-based tokenization
    fn simple_tokenize(&self, text: &str) -> Vec<u32> {
        text.split_whitespace()
            .take(self.config.max_seq_length)
            .enumerate()
            .map(|(i, _)| i as u32)
            .collect()
    }
    
    /// Clear processing cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::data_types::TimeSeriesData;
    
    #[test]
    fn test_text_pipeline() {
        let pipeline = TextPipeline::new();
        let config = ProcessingConfig::default();
        
        let data = UniversalData::Text("Hello world! How are you?".to_string());
        let result = pipeline.process(&data, &config);
        
        assert!(result.is_ok());
        let (features, embedding) = result.unwrap();
        
        assert!(features.numeric.contains_key("word_count"));
        assert_eq!(features.numeric.get("word_count"), Some(&5.0));
        assert!(embedding.is_some());
    }
    
    #[test]
    fn test_structured_pipeline() {
        let pipeline = StructuredPipeline::new();
        let config = ProcessingConfig::default();
        
        let json = serde_json::json!({
            "name": "test",
            "value": 42,
            "active": true
        });
        let data = UniversalData::Structured(json);
        let result = pipeline.process(&data, &config);
        
        assert!(result.is_ok());
        let (features, _) = result.unwrap();
        
        assert_eq!(features.numeric.get("field_count"), Some(&3.0));
        assert_eq!(features.numeric.get("value"), Some(&42.0));
    }
    
    #[test]
    fn test_timeseries_pipeline() {
        let pipeline = TimeSeriesPipeline::new();
        let config = ProcessingConfig::default();
        
        let ts = TimeSeriesData {
            timestamps: vec![0.0, 1.0, 2.0, 3.0, 4.0],
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            columns: vec!["value".to_string()],
            frequency_hz: Some(1.0),
        };
        let data = UniversalData::TimeSeries(ts);
        let result = pipeline.process(&data, &config);
        
        assert!(result.is_ok());
        let (features, _) = result.unwrap();
        
        assert_eq!(features.numeric.get("mean"), Some(&3.0));
        assert_eq!(features.numeric.get("min"), Some(&1.0));
        assert_eq!(features.numeric.get("max"), Some(&5.0));
    }
}
