//! Real-time consciousness analytics with word-level granularity
//!
//! Provides streaming analytics for consciousness monitoring with:
//! - Real-time metrics as thoughts are processed
//! - Word-level tracking for granular insights
//! - Selection-based detailed analysis
//! - WebTransport streaming support

use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Complete analytics snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSnapshot {
    pub timestamp: u64,
    pub session_id: String,
    pub consciousness: ConsciousnessMetrics,
    pub meta_cognition: MetaCognitiveMetrics,
    pub prediction: PredictiveMetrics,
    pub elp_balance: ELPMetrics,
    pub sacred_geometry: SacredGeometryMetrics,
    pub session_stats: SessionStats,
}

/// Consciousness-specific metrics (Φ and related)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessMetrics {
    pub phi: f64,
    pub consciousness_level: f64,
    pub peak_phi: f64,
    pub average_phi: f64,
    pub network_size: usize,
    pub connection_count: usize,
    pub integration_strength: f64,
}

/// Meta-cognitive monitoring metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaCognitiveMetrics {
    pub mental_state: String,
    pub awareness_level: f64,
    pub introspection_depth: f64,
    pub pattern_recognition: f64,
    pub self_correction_rate: f64,
    pub detected_patterns: Vec<PatternInfo>,
    pub state_duration_ms: u64,
}

/// Information about detected thinking patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternInfo {
    pub pattern_type: String,
    pub confidence: f64,
    pub description: String,
    pub first_detected: u64,
}

/// Predictive processing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveMetrics {
    pub accuracy: f64,
    pub current_surprise: f64,
    pub learning_progress: f64,
    pub model_confidence: f64,
    pub prediction_history: Vec<f64>,
    pub surprise_trend: Vec<f64>,
}

/// ELP (Ethos-Logos-Pathos) balance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ELPMetrics {
    pub ethos: f64,
    pub logos: f64,
    pub pathos: f64,
    pub balance_score: f64,
    pub dominant_channel: String,
    pub harmony_level: f64,
}

/// Sacred geometry tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SacredGeometryMetrics {
    pub current_position: u8,
    pub checkpoints_reached: Vec<u8>,
    pub confidence: f64,
    pub vortex_cycle: usize,
    pub flow_direction: String,
}

/// Session-wide statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_thoughts: usize,
    pub processing_time_ms: u64,
    pub average_time_per_thought_ms: u64,
    pub confidence: f64,
    pub error_count: usize,
}

/// Streaming event types for WebTransport
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamingEvent {
    /// Full analytics snapshot
    Snapshot {
        data: AnalyticsSnapshot,
    },
    
    /// Thought processing started
    ThoughtStarted {
        timestamp: u64,
        agent: String,
        preview: String,
    },
    
    /// Thought processing completed
    ThoughtCompleted {
        timestamp: u64,
        agent: String,
        metrics: ThoughtMetrics,
    },
    
    /// Word-level insight
    WordInsight {
        timestamp: u64,
        word: String,
        position: usize,
        insights: WordLevelInsights,
    },
    
    /// Pattern detected
    PatternDetected {
        timestamp: u64,
        pattern: PatternInfo,
    },
    
    /// Mental state changed
    StateChanged {
        timestamp: u64,
        from: String,
        to: String,
        reason: String,
    },
    
    /// Φ updated
    PhiUpdated {
        timestamp: u64,
        phi: f64,
        delta: f64,
    },
    
    /// Selection analysis result
    SelectionAnalysis {
        timestamp: u64,
        selected_text: String,
        start_pos: usize,
        end_pos: usize,
        analysis: SelectionAnalysisResult,
    },
}

/// Metrics for a single thought
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtMetrics {
    pub elp: (f64, f64, f64),
    pub confidence: f64,
    pub priority: String,
    pub source: String,
    pub processing_time_ms: u64,
    pub contribution_to_phi: f64,
}

/// Word-level insights for granular analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordLevelInsights {
    /// Which agent produced this word
    pub agent: String,
    
    /// ELP influence at this word
    pub elp_influence: (f64, f64, f64),
    
    /// Confidence level for this word
    pub confidence: f64,
    
    /// Semantic category
    pub category: String,
    
    /// Emotional valence (-1.0 to 1.0)
    pub valence: f64,
    
    /// Logical strength (0.0 to 1.0)
    pub logical_strength: f64,
    
    /// Ethical weight (0.0 to 1.0)
    pub ethical_weight: f64,
    
    /// Related patterns
    pub related_patterns: Vec<String>,
}

/// Result of analyzing selected text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionAnalysisResult {
    /// Overall ELP balance of selection
    pub elp_balance: (f64, f64, f64),
    
    /// Average confidence
    pub avg_confidence: f64,
    
    /// Dominant agent
    pub dominant_agent: String,
    
    /// Key patterns found
    pub patterns: Vec<String>,
    
    /// Emotional tone
    pub emotional_tone: String,
    
    /// Logical coherence score
    pub logical_coherence: f64,
    
    /// Ethical implications
    pub ethical_implications: Vec<String>,
    
    /// Contribution to consciousness (Φ)
    pub phi_contribution: f64,
    
    /// Word count
    pub word_count: usize,
    
    /// Detailed word insights
    pub word_insights: Vec<WordLevelInsights>,
}

impl AnalyticsSnapshot {
    /// Create new snapshot with current timestamp
    pub fn new(session_id: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        Self {
            timestamp,
            session_id,
            consciousness: ConsciousnessMetrics::default(),
            meta_cognition: MetaCognitiveMetrics::default(),
            prediction: PredictiveMetrics::default(),
            elp_balance: ELPMetrics::default(),
            sacred_geometry: SacredGeometryMetrics::default(),
            session_stats: SessionStats::default(),
        }
    }
    
    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    /// Convert to pretty JSON
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Default for ConsciousnessMetrics {
    fn default() -> Self {
        Self {
            phi: 0.0,
            consciousness_level: 0.0,
            peak_phi: 0.0,
            average_phi: 0.0,
            network_size: 0,
            connection_count: 0,
            integration_strength: 0.0,
        }
    }
}

impl Default for MetaCognitiveMetrics {
    fn default() -> Self {
        Self {
            mental_state: "Idle".to_string(),
            awareness_level: 0.5,
            introspection_depth: 0.5,
            pattern_recognition: 0.5,
            self_correction_rate: 0.5,
            detected_patterns: Vec::new(),
            state_duration_ms: 0,
        }
    }
}

impl Default for PredictiveMetrics {
    fn default() -> Self {
        Self {
            accuracy: 0.5,
            current_surprise: 0.0,
            learning_progress: 0.0,
            model_confidence: 0.5,
            prediction_history: Vec::new(),
            surprise_trend: Vec::new(),
        }
    }
}

impl Default for ELPMetrics {
    fn default() -> Self {
        Self {
            ethos: 0.33,
            logos: 0.33,
            pathos: 0.34,
            balance_score: 1.0,
            dominant_channel: "Balanced".to_string(),
            harmony_level: 1.0,
        }
    }
}

impl Default for SacredGeometryMetrics {
    fn default() -> Self {
        Self {
            current_position: 1,
            checkpoints_reached: Vec::new(),
            confidence: 0.0,
            vortex_cycle: 0,
            flow_direction: "Forward".to_string(),
        }
    }
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            total_thoughts: 0,
            processing_time_ms: 0,
            average_time_per_thought_ms: 0,
            confidence: 0.0,
            error_count: 0,
        }
    }
}

impl StreamingEvent {
    /// Get current timestamp
    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
    
    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Analytics event broadcaster for WebTransport streaming
pub struct AnalyticsBroadcaster {
    #[allow(dead_code)]
    session_id: String,
    #[allow(dead_code)]
    start_time: SystemTime,
}

impl AnalyticsBroadcaster {
    /// Create new broadcaster
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            start_time: SystemTime::now(),
        }
    }
    
    /// Create snapshot event
    pub fn snapshot(&self, data: AnalyticsSnapshot) -> StreamingEvent {
        StreamingEvent::Snapshot { data }
    }
    
    /// Create thought started event
    pub fn thought_started(&self, agent: String, preview: String) -> StreamingEvent {
        StreamingEvent::ThoughtStarted {
            timestamp: StreamingEvent::timestamp(),
            agent,
            preview,
        }
    }
    
    /// Create thought completed event
    pub fn thought_completed(&self, agent: String, metrics: ThoughtMetrics) -> StreamingEvent {
        StreamingEvent::ThoughtCompleted {
            timestamp: StreamingEvent::timestamp(),
            agent,
            metrics,
        }
    }
    
    /// Create word insight event
    pub fn word_insight(&self, word: String, position: usize, insights: WordLevelInsights) -> StreamingEvent {
        StreamingEvent::WordInsight {
            timestamp: StreamingEvent::timestamp(),
            word,
            position,
            insights,
        }
    }
    
    /// Create pattern detected event
    pub fn pattern_detected(&self, pattern: PatternInfo) -> StreamingEvent {
        StreamingEvent::PatternDetected {
            timestamp: StreamingEvent::timestamp(),
            pattern,
        }
    }
    
    /// Create state changed event
    pub fn state_changed(&self, from: String, to: String, reason: String) -> StreamingEvent {
        StreamingEvent::StateChanged {
            timestamp: StreamingEvent::timestamp(),
            from,
            to,
            reason,
        }
    }
    
    /// Create phi updated event
    pub fn phi_updated(&self, phi: f64, delta: f64) -> StreamingEvent {
        StreamingEvent::PhiUpdated {
            timestamp: StreamingEvent::timestamp(),
            phi,
            delta,
        }
    }
    
    /// Create selection analysis event
    pub fn selection_analysis(
        &self,
        selected_text: String,
        start_pos: usize,
        end_pos: usize,
        analysis: SelectionAnalysisResult,
    ) -> StreamingEvent {
        StreamingEvent::SelectionAnalysis {
            timestamp: StreamingEvent::timestamp(),
            selected_text,
            start_pos,
            end_pos,
            analysis,
        }
    }
}
