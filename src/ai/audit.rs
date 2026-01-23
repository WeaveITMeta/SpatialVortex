use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::io::Write;
use uuid::Uuid;

/// Structured audit event for tracking ASI Orchestrator behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub event_id: String,
    /// Session identifier
    pub session_id: String,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Type of audit event
    pub event_type: AuditEventType,
    /// Event severity level
    pub severity: AuditSeverity,
    /// Event-specific data
    pub data: AuditEventData,
    /// Optional free-form message
    pub message: Option<String>,
    /// Context hash for integrity verification
    pub context_hash: Option<String>,
    /// Chain of custody for the event
    pub chain: Vec<String>,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Session lifecycle events
    SessionStarted,
    SessionEnded,
    SessionCompressed,
    
    /// Controller events
    ControllerIntervention,
    ControllerPassThrough,
    VCPRiskAssessment,
    CheckpointReached,
    
    /// Generation events
    GenerationStarted,
    GenerationCompleted,
    GenerationFailed,
    GenerationFallback,
    
    /// Context events
    ContextCreated,
    ContextUpdated,
    ContextCompressed,
    ContextRetrieved,
    
    /// Security events
    SecurityViolation,
    AccessDenied,
    UnauthorizedAction,
    
    /// Performance events
    PerformanceThreshold,
    LatencySpike,
    ResourceExhaustion,
    
    /// Error events
    SystemError,
    UserError,
    ValidationError,
    
    /// Custom events
    Custom(String),
}

/// Severity levels for audit events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Event-specific data payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventData {
    /// Generic key-value data
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Performance metrics
    pub performance: Option<PerformanceMetrics>,
    
    /// Controller-specific data
    pub controller: Option<ControllerData>,
    
    /// Context-specific data
    pub context: Option<ContextData>,
    
    /// Security-specific data
    pub security: Option<SecurityData>,
}

/// Performance metrics in audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub latency_ms: u64,
    pub tokens_generated: Option<usize>,
    pub memory_usage_mb: Option<f32>,
    pub cpu_usage_percent: Option<f32>,
    pub confidence_score: Option<f32>,
}

/// Controller-specific audit data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerData {
    pub flux_position: Option<u8>,
    pub checkpoint: bool,
    pub vcp_risk_score: Option<f32>,
    pub vcp_signal_strength: Option<f32>,
    pub intervention_type: Option<String>,
    pub confidence_before: Option<f32>,
    pub confidence_after: Option<f32>,
    pub compression_applied: bool,
}

/// Context-specific audit data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
    pub context_length: usize,
    pub compressed_length: Option<usize>,
    pub compression_ratio: Option<f32>,
    pub context_type: String,
    pub retrieval_count: Option<usize>,
    pub grounding_score: Option<f32>,
}

/// Security-specific audit data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityData {
    pub user_id: Option<String>,
    pub access_level: Option<String>,
    pub violation_type: Option<String>,
    pub blocked_content: Option<String>,
    pub risk_score: Option<f32>,
}

/// Audit stream manager for handling event collection and persistence
pub struct AuditStream {
    session_id: String,
    events: Vec<AuditEvent>,
    max_events: usize,
    enable_persistence: bool,
    persistence_path: Option<std::path::PathBuf>,
}

impl AuditStream {
    /// Create a new audit stream for a session
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            events: Vec::new(),
            max_events: 10000,
            enable_persistence: false,
            persistence_path: None,
        }
    }

    /// Configure persistence settings
    pub fn with_persistence(mut self, enabled: bool, path: Option<std::path::PathBuf>) -> Self {
        self.enable_persistence = enabled;
        self.persistence_path = path;
        self
    }

    /// Set maximum events to keep in memory
    pub fn with_max_events(mut self, max_events: usize) -> Self {
        self.max_events = max_events;
        self
    }

    /// Record a new audit event
    pub fn record_event(&mut self, event_type: AuditEventType, severity: AuditSeverity, data: AuditEventData) -> Result<(), AuditError> {
        let event = AuditEvent {
            event_id: Uuid::new_v4().to_string(),
            session_id: self.session_id.clone(),
            timestamp: Utc::now(),
            event_type,
            severity,
            data,
            message: None,
            context_hash: None,
            chain: Vec::new(),
        };

        self.add_event(event)
    }

    /// Get all events in the stream
    pub fn get_events(&self) -> &[AuditEvent] {
        &self.events
    }

    /// Generate summary statistics
    pub fn generate_summary(&self) -> AuditSummary {
        let mut event_counts = HashMap::new();
        let mut severity_counts = HashMap::new();
        let mut performance_summary = PerformanceSummary::new();

        for event in &self.events {
            *event_counts.entry(format!("{:?}", event.event_type)).or_insert(0) += 1;
            *severity_counts.entry(format!("{:?}", event.severity)).or_insert(0) += 1;

            if let Some(ref perf) = event.data.performance {
                performance_summary.add_sample(perf);
            }
        }

        AuditSummary {
            session_id: self.session_id.clone(),
            total_events: self.events.len(),
            event_counts,
            severity_counts,
            performance_summary,
            time_range: if self.events.is_empty() {
                None
            } else {
                let start = self.events[0].timestamp;
                let end = self.events[self.events.len() - 1].timestamp;
                Some((start, end))
            },
        }
    }

    /// Add an event to the stream
    fn add_event(&mut self, event: AuditEvent) -> Result<(), AuditError> {
        if self.events.len() >= self.max_events {
            let remove_count = self.events.len() - self.max_events + 1;
            self.events.drain(0..remove_count);
        }

        self.events.push(event.clone());

        if self.enable_persistence {
            self.persist_event(&event)?;
        }

        Ok(())
    }

    /// Persist a single event to storage
    fn persist_event(&self, event: &AuditEvent) -> Result<(), AuditError> {
        if let Some(ref path) = self.persistence_path {
            std::fs::create_dir_all(path)?;
            let file_path = path.join(format!("audit_{}.jsonl", self.session_id));
            let event_json = serde_json::to_string(event)?;
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)?
                .write_all(format!("{}\n", event_json).as_bytes())?;
        }

        Ok(())
    }
}

/// Summary statistics for an audit stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub session_id: String,
    pub total_events: usize,
    pub event_counts: HashMap<String, usize>,
    pub severity_counts: HashMap<String, usize>,
    pub performance_summary: PerformanceSummary,
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

/// Performance summary aggregated from audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_latency_ms: u64,
    pub avg_latency_ms: f64,
    pub min_latency_ms: u64,
    pub max_latency_ms: u64,
    pub total_tokens: usize,
    pub avg_confidence: Option<f32>,
    pub sample_count: usize,
}

impl PerformanceSummary {
    pub fn new() -> Self {
        Self {
            total_latency_ms: 0,
            avg_latency_ms: 0.0,
            min_latency_ms: u64::MAX,
            max_latency_ms: 0,
            total_tokens: 0,
            avg_confidence: None,
            sample_count: 0,
        }
    }

    pub fn add_sample(&mut self, metrics: &PerformanceMetrics) {
        self.total_latency_ms += metrics.latency_ms;
        self.min_latency_ms = self.min_latency_ms.min(metrics.latency_ms);
        self.max_latency_ms = self.max_latency_ms.max(metrics.latency_ms);
        
        if let Some(tokens) = metrics.tokens_generated {
            self.total_tokens += tokens;
        }

        if let Some(confidence) = metrics.confidence_score {
            self.avg_confidence = match self.avg_confidence {
                None => Some(confidence),
                Some(avg) => Some((avg + confidence) / 2.0),
            };
        }

        self.sample_count += 1;
        self.avg_latency_ms = self.total_latency_ms as f64 / self.sample_count as f64;
    }
}

/// Audit error types
#[derive(Debug, Clone)]
pub enum AuditError {
    SerializationError(String),
    IoError(String),
    CapacityExceeded,
    InvalidConfiguration,
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            AuditError::IoError(e) => write!(f, "IO error: {}", e),
            AuditError::CapacityExceeded => write!(f, "Audit stream capacity exceeded"),
            AuditError::InvalidConfiguration => write!(f, "Invalid audit configuration"),
        }
    }
}

impl std::error::Error for AuditError {}

impl From<serde_json::Error> for AuditError {
    fn from(error: serde_json::Error) -> Self {
        AuditError::SerializationError(error.to_string())
    }
}

impl From<std::io::Error> for AuditError {
    fn from(error: std::io::Error) -> Self {
        AuditError::IoError(error.to_string())
    }
}

/// Convenience functions for creating common audit events
impl AuditEventData {
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            performance: None,
            controller: None,
            context: None,
            security: None,
        }
    }

    pub fn with_performance(mut self, metrics: PerformanceMetrics) -> Self {
        self.performance = Some(metrics);
        self
    }

    pub fn with_controller(mut self, data: ControllerData) -> Self {
        self.controller = Some(data);
        self
    }

    pub fn with_context(mut self, data: ContextData) -> Self {
        self.context = Some(data);
        self
    }

    pub fn with_security(mut self, data: SecurityData) -> Self {
        self.security = Some(data);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl Default for AuditEventData {
    fn default() -> Self {
        Self::new()
    }
}

/// Global audit manager for handling multiple streams
pub struct AuditManager {
    streams: HashMap<String, AuditStream>,
    default_config: AuditConfig,
}

/// Configuration for audit management
#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub enable_persistence: bool,
    pub persistence_path: Option<std::path::PathBuf>,
    pub max_events_per_stream: usize,
    pub max_streams: usize,
    pub retention_days: Option<u32>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enable_persistence: false,
            persistence_path: None,
            max_events_per_stream: 10000,
            max_streams: 1000,
            retention_days: Some(30),
        }
    }
}

impl AuditManager {
    pub fn new(config: AuditConfig) -> Self {
        Self {
            streams: HashMap::new(),
            default_config: config,
        }
    }

    /// Get or create an audit stream for a session
    pub fn get_stream(&mut self, session_id: &str) -> &mut AuditStream {
        if !self.streams.contains_key(session_id) {
            if self.streams.len() >= self.default_config.max_streams {
                if let Some(oldest_key) = self.streams.keys().next().cloned() {
                    self.streams.remove(&oldest_key);
                }
            }

            let stream = AuditStream::new(session_id.to_string())
                .with_persistence(
                    self.default_config.enable_persistence,
                    self.default_config.persistence_path.clone(),
                )
                .with_max_events(self.default_config.max_events_per_stream);

            self.streams.insert(session_id.to_string(), stream);
        }

        self.streams.get_mut(session_id).unwrap()
    }

    /// Get all stream summaries
    pub fn get_summaries(&self) -> Vec<AuditSummary> {
        self.streams.values()
            .map(|stream| stream.generate_summary())
            .collect()
    }
}

/// Global audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalAuditStats {
    pub total_streams: usize,
    pub total_events: usize,
    pub total_latency_ms: u64,
    pub total_tokens_generated: usize,
    pub avg_latency_per_event: f64,
}