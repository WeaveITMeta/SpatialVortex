//! Input Data Layer
//!
//! The first layer of the universal pipeline - accepts any data source and normalizes it.
//!
//! ## Responsibilities
//!
//! - **Universal Connectors**: REST, GraphQL, Kafka, WebSocket, databases, files, etc.
//! - **Format Detection**: Auto-detect JSON, CSV, Parquet, images, audio, video
//! - **Authentication**: Bearer, OAuth2, API keys, certificates, cloud credentials
//! - **Streaming**: Real-time data ingestion with backpressure
//! - **Anonymization**: Hash, redact, synthetic data generation
//!
//! ## Supported Sources (aligned with EustressEngine Parameters)
//!
//! - General: REST, GraphQL, CSV, JSON, XML, Parquet, Excel, gRPC
//! - Messaging: MQTT, Kafka, AMQP, WebSocket, WebTransport, SSE
//! - Databases: PostgreSQL, MySQL, MongoDB, Redis, SQLite, Snowflake, BigQuery
//! - Cloud: S3, AzureBlob, GCS, Firebase, Supabase
//! - Industrial/IoT: OPCUA, Modbus, BACnet, CoAP, LwM2M
//! - Healthcare: FHIR, HL7v2, HL7v3, CDA, DICOM, X12, NCPDP

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;

use super::data_types::{DataEnvelope, UniversalData, SourceInfo, ProcessingStage};
use super::PipelineError;

/// Supported data source types (mirrors EustressEngine DataSourceType)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DataSourceType {
    #[default]
    None,
    // General Data Formats
    REST, GraphQL, CSV, JSON, XML, Parquet, Excel, GRPC,
    // Messaging & Streaming
    MQTT, Kafka, AMQP, WebSocket, WebTransport, SSE,
    // Databases
    PostgreSQL, MySQL, MongoDB, Redis, SQLite, Snowflake, BigQuery,
    // Cloud Services
    S3, AzureBlob, GCS, Firebase, Supabase, Oracle, DigitalOcean,
    // Industrial & IoT
    OPCUA, Modbus, BACnet, CoAP, LwM2M,
    // Healthcare Standards
    FHIR, HL7v2, HL7v3, CDA, DICOM, X12, NCPDP, IHE, OpenEHR, OMOP,
    // Specialty
    LDAP, SFTP, FTP, Email, RSS,
}

impl DataSourceType {
    /// Get category for routing
    pub fn category(&self) -> &'static str {
        match self {
            DataSourceType::None => "none",
            DataSourceType::REST | DataSourceType::GraphQL | DataSourceType::GRPC => "api",
            DataSourceType::CSV | DataSourceType::JSON | DataSourceType::XML |
            DataSourceType::Parquet | DataSourceType::Excel => "file",
            DataSourceType::MQTT | DataSourceType::Kafka | DataSourceType::AMQP |
            DataSourceType::WebSocket | DataSourceType::WebTransport | DataSourceType::SSE => "streaming",
            DataSourceType::PostgreSQL | DataSourceType::MySQL | DataSourceType::MongoDB |
            DataSourceType::Redis | DataSourceType::SQLite | DataSourceType::Snowflake |
            DataSourceType::BigQuery => "database",
            DataSourceType::S3 | DataSourceType::AzureBlob | DataSourceType::GCS |
            DataSourceType::Firebase | DataSourceType::Supabase | DataSourceType::Oracle |
            DataSourceType::DigitalOcean => "cloud",
            DataSourceType::OPCUA | DataSourceType::Modbus | DataSourceType::BACnet |
            DataSourceType::CoAP | DataSourceType::LwM2M => "iot",
            DataSourceType::FHIR | DataSourceType::HL7v2 | DataSourceType::HL7v3 |
            DataSourceType::CDA | DataSourceType::DICOM | DataSourceType::X12 |
            DataSourceType::NCPDP | DataSourceType::IHE | DataSourceType::OpenEHR |
            DataSourceType::OMOP => "healthcare",
            DataSourceType::LDAP | DataSourceType::SFTP | DataSourceType::FTP |
            DataSourceType::Email | DataSourceType::RSS => "specialty",
        }
    }
    
    /// Check if this is a streaming source
    pub fn is_streaming(&self) -> bool {
        matches!(self, 
            DataSourceType::MQTT | DataSourceType::Kafka | DataSourceType::AMQP |
            DataSourceType::WebSocket | DataSourceType::WebTransport | DataSourceType::SSE
        )
    }
}

/// Authentication types (mirrors EustressEngine AuthType)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AuthType {
    #[default]
    None,
    Basic,
    Bearer,
    APIKey,
    OAuth2,
    Certificate,
    AWSSignature,
    AzureAD,
    GoogleServiceAccount,
    SAML,
    Custom,
}

/// Update mode for data refresh
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum UpdateMode {
    #[default]
    Manual,
    Polling { interval_seconds: f32 },
    Webhook { endpoint_url: String },
    FileWatch { path: String },
    Streaming,
    EventDriven { event_pattern: String },
}

/// Anonymization mode for sensitive data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnonymizationMode {
    #[default]
    None,
    Hash,
    Synthetic,
    Redact,
    KAnonymity,
    DifferentialPrivacy,
}

/// Raw input from any source
#[derive(Debug, Clone)]
pub struct RawInput {
    /// Source type
    pub source_type: DataSourceType,
    /// Raw bytes or text
    pub content: RawContent,
    /// Source endpoint/path
    pub endpoint: String,
    /// Resource identifier
    pub resource_id: String,
    /// Content type hint (MIME type)
    pub content_type: Option<String>,
    /// Additional headers/metadata
    pub metadata: HashMap<String, String>,
    /// Authentication info
    pub auth: AuthType,
    /// Anonymization to apply
    pub anonymization: AnonymizationMode,
}

/// Raw content variants
#[derive(Debug, Clone)]
pub enum RawContent {
    /// UTF-8 text
    Text(String),
    /// Binary data
    Bytes(Vec<u8>),
    /// Already parsed JSON
    Json(serde_json::Value),
    /// Stream handle (for streaming sources)
    StreamHandle(String),
}

impl RawInput {
    /// Create from text
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            source_type: DataSourceType::None,
            content: RawContent::Text(text.into()),
            endpoint: String::new(),
            resource_id: uuid::Uuid::new_v4().to_string(),
            content_type: Some("text/plain".to_string()),
            metadata: HashMap::new(),
            auth: AuthType::None,
            anonymization: AnonymizationMode::None,
        }
    }
    
    /// Create from JSON
    pub fn from_json(json: serde_json::Value) -> Self {
        Self {
            source_type: DataSourceType::JSON,
            content: RawContent::Json(json),
            endpoint: String::new(),
            resource_id: uuid::Uuid::new_v4().to_string(),
            content_type: Some("application/json".to_string()),
            metadata: HashMap::new(),
            auth: AuthType::None,
            anonymization: AnonymizationMode::None,
        }
    }
    
    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            source_type: DataSourceType::None,
            content: RawContent::Bytes(bytes),
            endpoint: String::new(),
            resource_id: uuid::Uuid::new_v4().to_string(),
            content_type: None,
            metadata: HashMap::new(),
            auth: AuthType::None,
            anonymization: AnonymizationMode::None,
        }
    }
    
    /// Set source type
    pub fn with_source_type(mut self, source_type: DataSourceType) -> Self {
        self.source_type = source_type;
        self
    }
    
    /// Set endpoint
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }
    
    /// Set resource ID
    pub fn with_resource_id(mut self, id: impl Into<String>) -> Self {
        self.resource_id = id.into();
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Data source configuration
#[derive(Debug, Clone)]
pub struct DataSource {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Source type
    pub source_type: DataSourceType,
    /// Base endpoint
    pub endpoint: String,
    /// Authentication
    pub auth: AuthType,
    /// Auth token reference (env var or secret name)
    pub auth_token_ref: Option<String>,
    /// Update mode
    pub update_mode: UpdateMode,
    /// Anonymization
    pub anonymization: AnonymizationMode,
    /// Whether source is enabled
    pub enabled: bool,
    /// Rate limit (requests per second)
    pub rate_limit: Option<f32>,
    /// Timeout in seconds
    pub timeout_seconds: f32,
    /// Retry count on failure
    pub retry_count: u32,
}

impl Default for DataSource {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            source_type: DataSourceType::None,
            endpoint: String::new(),
            auth: AuthType::None,
            auth_token_ref: None,
            update_mode: UpdateMode::Manual,
            anonymization: AnonymizationMode::None,
            enabled: true,
            rate_limit: None,
            timeout_seconds: 30.0,
            retry_count: 3,
        }
    }
}

/// Input layer configuration
#[derive(Debug, Clone)]
pub struct InputConfig {
    /// Maximum content size in bytes
    pub max_content_size: usize,
    /// Default timeout in seconds
    pub default_timeout: f32,
    /// Enable content type auto-detection
    pub auto_detect_content_type: bool,
    /// Enable streaming for large content
    pub enable_streaming: bool,
    /// Buffer size for streaming
    pub stream_buffer_size: usize,
    /// Enable input validation
    pub validate_input: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            max_content_size: 100 * 1024 * 1024, // 100MB
            default_timeout: 30.0,
            auto_detect_content_type: true,
            enable_streaming: true,
            stream_buffer_size: 64 * 1024, // 64KB
            validate_input: true,
        }
    }
}

/// The Input Data Layer
pub struct InputLayer {
    config: InputConfig,
    /// Registered data sources
    sources: RwLock<HashMap<String, DataSource>>,
    /// Format detectors
    format_detectors: Vec<Box<dyn FormatDetector + Send + Sync>>,
}

impl InputLayer {
    /// Create new input layer
    pub fn new(config: InputConfig) -> Self {
        Self {
            config,
            sources: RwLock::new(HashMap::new()),
            format_detectors: vec![
                Box::new(JsonDetector),
                Box::new(XmlDetector),
                Box::new(CsvDetector),
                Box::new(ImageDetector),
                Box::new(AudioDetector),
            ],
        }
    }
    
    /// Register a data source
    pub fn register_source(&self, source: DataSource) {
        self.sources.write().insert(source.id.clone(), source);
    }
    
    /// Get a registered source
    pub fn get_source(&self, id: &str) -> Option<DataSource> {
        self.sources.read().get(id).cloned()
    }
    
    /// Ingest raw input and create data envelope
    pub async fn ingest(&self, input: RawInput) -> Result<DataEnvelope, PipelineError> {
        // Validate input size
        let content_size = match &input.content {
            RawContent::Text(t) => t.len(),
            RawContent::Bytes(b) => b.len(),
            RawContent::Json(j) => j.to_string().len(),
            RawContent::StreamHandle(_) => 0,
        };
        
        if content_size > self.config.max_content_size {
            return Err(PipelineError::InputError(format!(
                "Content size {} exceeds maximum {}",
                content_size, self.config.max_content_size
            )));
        }
        
        // Detect content type if not provided
        let content_type = input.content_type.clone().unwrap_or_else(|| {
            self.detect_content_type(&input.content)
        });
        
        // Convert to universal data
        let data = self.convert_to_universal(&input, &content_type)?;
        
        // Apply anonymization if needed
        let data = self.apply_anonymization(data, input.anonymization)?;
        
        // Create envelope
        let mut envelope = DataEnvelope::new(&input.resource_id, data);
        envelope.source = SourceInfo {
            source_type: format!("{:?}", input.source_type),
            endpoint: input.endpoint.clone(),
            resource_id: input.resource_id.clone(),
            auth_type: format!("{:?}", input.auth),
            format: content_type,
            schema_version: input.metadata.get("schema_version").cloned(),
        };
        envelope.stage = ProcessingStage::Input;
        
        Ok(envelope)
    }
    
    /// Detect content type from raw content
    fn detect_content_type(&self, content: &RawContent) -> String {
        for detector in &self.format_detectors {
            if let Some(mime) = detector.detect(content) {
                return mime;
            }
        }
        "application/octet-stream".to_string()
    }
    
    /// Convert raw content to universal data
    fn convert_to_universal(&self, input: &RawInput, content_type: &str) -> Result<UniversalData, PipelineError> {
        match &input.content {
            RawContent::Text(text) => {
                if content_type.contains("json") {
                    match serde_json::from_str(text) {
                        Ok(json) => Ok(UniversalData::Structured(json)),
                        Err(_) => Ok(UniversalData::Text(text.clone())),
                    }
                } else {
                    Ok(UniversalData::Text(text.clone()))
                }
            }
            RawContent::Bytes(bytes) => {
                if content_type.starts_with("text/") {
                    match String::from_utf8(bytes.clone()) {
                        Ok(text) => Ok(UniversalData::Text(text)),
                        Err(_) => Ok(UniversalData::Binary(bytes.clone())),
                    }
                } else {
                    Ok(UniversalData::Binary(bytes.clone()))
                }
            }
            RawContent::Json(json) => Ok(UniversalData::Structured(json.clone())),
            RawContent::StreamHandle(handle) => {
                // For streaming, return empty and handle separately
                Ok(UniversalData::Text(format!("stream:{}", handle)))
            }
        }
    }
    
    /// Apply anonymization to data
    fn apply_anonymization(&self, data: UniversalData, mode: AnonymizationMode) -> Result<UniversalData, PipelineError> {
        match mode {
            AnonymizationMode::None => Ok(data),
            AnonymizationMode::Hash => {
                // Hash sensitive fields
                match data {
                    UniversalData::Text(text) => {
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        text.hash(&mut hasher);
                        Ok(UniversalData::Text(format!("hashed:{:x}", hasher.finish())))
                    }
                    other => Ok(other),
                }
            }
            AnonymizationMode::Redact => {
                match data {
                    UniversalData::Text(_) => Ok(UniversalData::Text("[REDACTED]".to_string())),
                    other => Ok(other),
                }
            }
            _ => Ok(data), // Other modes would need more complex implementation
        }
    }
}

/// Trait for format detection
pub trait FormatDetector {
    fn detect(&self, content: &RawContent) -> Option<String>;
}

struct JsonDetector;
impl FormatDetector for JsonDetector {
    fn detect(&self, content: &RawContent) -> Option<String> {
        match content {
            RawContent::Text(t) => {
                let trimmed = t.trim();
                if (trimmed.starts_with('{') && trimmed.ends_with('}')) ||
                   (trimmed.starts_with('[') && trimmed.ends_with(']')) {
                    Some("application/json".to_string())
                } else {
                    None
                }
            }
            RawContent::Json(_) => Some("application/json".to_string()),
            _ => None,
        }
    }
}

struct XmlDetector;
impl FormatDetector for XmlDetector {
    fn detect(&self, content: &RawContent) -> Option<String> {
        match content {
            RawContent::Text(t) => {
                if t.trim().starts_with("<?xml") || t.trim().starts_with('<') {
                    Some("application/xml".to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

struct CsvDetector;
impl FormatDetector for CsvDetector {
    fn detect(&self, content: &RawContent) -> Option<String> {
        match content {
            RawContent::Text(t) => {
                let lines: Vec<&str> = t.lines().take(3).collect();
                if lines.len() >= 2 {
                    let comma_count: Vec<usize> = lines.iter()
                        .map(|l| l.matches(',').count())
                        .collect();
                    if comma_count.iter().all(|&c| c > 0 && c == comma_count[0]) {
                        return Some("text/csv".to_string());
                    }
                }
                None
            }
            _ => None,
        }
    }
}

struct ImageDetector;
impl FormatDetector for ImageDetector {
    fn detect(&self, content: &RawContent) -> Option<String> {
        match content {
            RawContent::Bytes(b) if b.len() >= 4 => {
                // Check magic bytes
                if b.starts_with(&[0xFF, 0xD8, 0xFF]) {
                    Some("image/jpeg".to_string())
                } else if b.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                    Some("image/png".to_string())
                } else if b.starts_with(b"GIF8") {
                    Some("image/gif".to_string())
                } else if b.starts_with(b"RIFF") && b.len() > 12 && &b[8..12] == b"WEBP" {
                    Some("image/webp".to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

struct AudioDetector;
impl FormatDetector for AudioDetector {
    fn detect(&self, content: &RawContent) -> Option<String> {
        match content {
            RawContent::Bytes(b) if b.len() >= 4 => {
                if b.starts_with(b"ID3") || (b.len() >= 2 && b[0] == 0xFF && (b[1] & 0xE0) == 0xE0) {
                    Some("audio/mpeg".to_string())
                } else if b.starts_with(b"RIFF") && b.len() > 12 && &b[8..12] == b"WAVE" {
                    Some("audio/wav".to_string())
                } else if b.starts_with(b"OggS") {
                    Some("audio/ogg".to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_source_type_category() {
        assert_eq!(DataSourceType::REST.category(), "api");
        assert_eq!(DataSourceType::Kafka.category(), "streaming");
        assert_eq!(DataSourceType::PostgreSQL.category(), "database");
        assert_eq!(DataSourceType::FHIR.category(), "healthcare");
    }
    
    #[test]
    fn test_raw_input_creation() {
        let input = RawInput::from_text("hello world")
            .with_source_type(DataSourceType::REST)
            .with_endpoint("https://api.example.com");
        
        assert_eq!(input.source_type, DataSourceType::REST);
        assert_eq!(input.endpoint, "https://api.example.com");
    }
    
    #[test]
    fn test_json_detection() {
        let detector = JsonDetector;
        
        let json_content = RawContent::Text(r#"{"key": "value"}"#.to_string());
        assert_eq!(detector.detect(&json_content), Some("application/json".to_string()));
        
        let text_content = RawContent::Text("plain text".to_string());
        assert_eq!(detector.detect(&text_content), None);
    }
    
    #[tokio::test]
    async fn test_input_layer_ingest() {
        let layer = InputLayer::new(InputConfig::default());
        
        let input = RawInput::from_json(serde_json::json!({"test": "data"}));
        let result = layer.ingest(input).await;
        
        assert!(result.is_ok());
        let envelope = result.unwrap();
        assert_eq!(envelope.stage, ProcessingStage::Input);
    }
}
