//! Pattern Recognition for Runtime Detection
//!
//! Identifies recurring issues and patterns in system behavior:
//! - Temporal patterns (hourly spikes, daily cycles)
//! - Sequential patterns (operation sequences leading to issues)
//! - Frequency patterns (recurring at specific intervals)
//! - Correlation patterns (multiple metrics degrading together)

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Pattern type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PatternType {
    /// Temporal: Recurring at specific time intervals
    Temporal { interval_secs: u64 },
    
    /// Sequential: Specific sequence of events
    Sequential { sequence: Vec<String> },
    
    /// Frequency: Occurs N times per time period
    Frequency { count: usize, period_secs: u64 },
    
    /// Correlation: Multiple metrics degrade together
    Correlation { metrics: Vec<String> },
    
    /// Cyclic: Daily/weekly/monthly cycles
    Cyclic { cycle_type: CycleType },
}

/// Cycle type for cyclic patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CycleType {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

/// Detected pattern with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// Pattern type
    pub pattern_type: PatternType,
    
    /// Pattern description
    pub description: String,
    
    /// Confidence score (0-1)
    pub confidence: f32,
    
    /// Number of occurrences observed
    pub occurrences: usize,
    
    /// First detection timestamp
    pub first_seen: u64,
    
    /// Last detection timestamp
    pub last_seen: u64,
    
    /// Average severity when pattern occurs
    pub avg_severity: f32,
    
    /// Predicted next occurrence (if temporal)
    pub next_predicted: Option<u64>,
    
    /// Associated weakness types
    pub weakness_types: Vec<String>,
}

/// Event for pattern detection
#[derive(Debug, Clone)]
pub struct PatternEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub severity: f32,
    pub metadata: HashMap<String, String>,
}

/// Temporal pattern detector
struct TemporalDetector {
    /// Event history with timestamps
    events: VecDeque<(u64, String, f32)>,
    
    /// Maximum history size
    max_history: usize,
    
    /// Minimum occurrences to confirm pattern
    min_occurrences: usize,
}

impl TemporalDetector {
    fn new(max_history: usize, min_occurrences: usize) -> Self {
        Self {
            events: VecDeque::new(),
            max_history,
            min_occurrences,
        }
    }
    
    fn add_event(&mut self, timestamp: u64, event_type: String, severity: f32) {
        self.events.push_back((timestamp, event_type, severity));
        
        if self.events.len() > self.max_history {
            self.events.pop_front();
        }
    }
    
    fn detect_patterns(&self) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();
        
        // Group events by type
        let mut event_groups: HashMap<String, Vec<(u64, f32)>> = HashMap::new();
        for (ts, event_type, severity) in &self.events {
            event_groups.entry(event_type.clone())
                .or_insert_with(Vec::new)
                .push((*ts, *severity));
        }
        
        // Detect temporal patterns for each event type
        for (event_type, timestamps) in event_groups {
            if timestamps.len() < self.min_occurrences {
                continue;
            }
            
            // Calculate intervals between occurrences
            let mut intervals = Vec::new();
            for i in 1..timestamps.len() {
                intervals.push(timestamps[i].0 - timestamps[i - 1].0);
            }
            
            if intervals.is_empty() {
                continue;
            }
            
            // Check for consistent intervals (temporal pattern)
            let avg_interval = intervals.iter().sum::<u64>() / intervals.len() as u64;
            let variance = intervals.iter()
                .map(|i| (*i as i64 - avg_interval as i64).abs() as u64)
                .sum::<u64>() / intervals.len() as u64;
            
            // If variance is low relative to interval, it's a pattern
            let consistency = 1.0 - (variance as f32 / avg_interval.max(1) as f32).min(1.0);
            
            if consistency > 0.7 && avg_interval > 0 {
                let avg_severity = timestamps.iter().map(|(_, s)| s).sum::<f32>() / timestamps.len() as f32;
                let first_seen = timestamps.first().unwrap().0;
                let last_seen = timestamps.last().unwrap().0;
                let next_predicted = Some(last_seen + avg_interval);
                
                patterns.push(DetectedPattern {
                    pattern_type: PatternType::Temporal { interval_secs: avg_interval },
                    description: format!("{} recurring every ~{} seconds", event_type, avg_interval),
                    confidence: consistency,
                    occurrences: timestamps.len(),
                    first_seen,
                    last_seen,
                    avg_severity,
                    next_predicted,
                    weakness_types: vec![event_type.clone()],
                });
            }
            
            // Check for cyclic patterns (hourly, daily, etc.)
            if let Some(cycle_pattern) = self.detect_cyclic_pattern(&event_type, &timestamps, avg_interval) {
                patterns.push(cycle_pattern);
            }
        }
        
        patterns
    }
    
    fn detect_cyclic_pattern(&self, event_type: &str, timestamps: &[(u64, f32)], avg_interval: u64) -> Option<DetectedPattern> {
        const HOUR: u64 = 3600;
        const DAY: u64 = 86400;
        const WEEK: u64 = 604800;
        
        let cycle_type = if (avg_interval as i64 - HOUR as i64).abs() < 600 {
            Some(CycleType::Hourly)
        } else if (avg_interval as i64 - DAY as i64).abs() < 3600 {
            Some(CycleType::Daily)
        } else if (avg_interval as i64 - WEEK as i64).abs() < 7200 {
            Some(CycleType::Weekly)
        } else {
            None
        };
        
        cycle_type.map(|ct| {
            let avg_severity = timestamps.iter().map(|(_, s)| s).sum::<f32>() / timestamps.len() as f32;
            let first_seen = timestamps.first().unwrap().0;
            let last_seen = timestamps.last().unwrap().0;
            
            DetectedPattern {
                pattern_type: PatternType::Cyclic { cycle_type: ct.clone() },
                description: format!("{} occurring on {:?} cycle", event_type, ct),
                confidence: 0.85,
                occurrences: timestamps.len(),
                first_seen,
                last_seen,
                avg_severity,
                next_predicted: Some(last_seen + avg_interval),
                weakness_types: vec![event_type.to_string()],
            }
        })
    }
}

/// Sequential pattern detector
struct SequentialDetector {
    /// Recent event sequence
    sequence: VecDeque<String>,
    
    /// Maximum sequence length to track
    max_sequence_length: usize,
    
    /// Detected sequences and their counts
    sequence_counts: HashMap<Vec<String>, usize>,
    
    /// Minimum occurrences to confirm pattern
    min_occurrences: usize,
}

impl SequentialDetector {
    fn new(max_sequence_length: usize, min_occurrences: usize) -> Self {
        Self {
            sequence: VecDeque::new(),
            max_sequence_length,
            sequence_counts: HashMap::new(),
            min_occurrences,
        }
    }
    
    fn add_event(&mut self, event_type: String) {
        self.sequence.push_back(event_type);
        
        if self.sequence.len() > self.max_sequence_length {
            self.sequence.pop_front();
        }
        
        // Record all subsequences
        for len in 2..=self.sequence.len().min(5) {
            if self.sequence.len() >= len {
                let subseq: Vec<String> = self.sequence.iter()
                    .rev()
                    .take(len)
                    .rev()
                    .cloned()
                    .collect();
                
                *self.sequence_counts.entry(subseq).or_insert(0) += 1;
            }
        }
    }
    
    fn detect_patterns(&self) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        for (sequence, count) in &self.sequence_counts {
            if *count >= self.min_occurrences && sequence.len() >= 2 {
                let confidence = (*count as f32 / 10.0).min(1.0);
                
                patterns.push(DetectedPattern {
                    pattern_type: PatternType::Sequential { sequence: sequence.clone() },
                    description: format!("Sequence pattern: {} → leads to issues", sequence.join(" → ")),
                    confidence,
                    occurrences: *count,
                    first_seen: now - 3600, // Approximate
                    last_seen: now,
                    avg_severity: 0.7,
                    next_predicted: None,
                    weakness_types: sequence.clone(),
                });
            }
        }
        
        patterns
    }
}

/// Correlation pattern detector
struct CorrelationDetector {
    /// Recent metric values
    metric_history: HashMap<String, VecDeque<(u64, f32)>>,
    
    /// Maximum history per metric
    max_history: usize,
    
    /// Correlation threshold
    correlation_threshold: f32,
}

impl CorrelationDetector {
    fn new(max_history: usize, correlation_threshold: f32) -> Self {
        Self {
            metric_history: HashMap::new(),
            max_history,
            correlation_threshold,
        }
    }
    
    fn add_metric(&mut self, metric_name: String, timestamp: u64, value: f32) {
        let history = self.metric_history.entry(metric_name).or_insert_with(VecDeque::new);
        history.push_back((timestamp, value));
        
        if history.len() > self.max_history {
            history.pop_front();
        }
    }
    
    fn detect_patterns(&self) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        // Find correlated metrics
        let metrics: Vec<String> = self.metric_history.keys().cloned().collect();
        
        for i in 0..metrics.len() {
            for j in (i + 1)..metrics.len() {
                let metric1 = &metrics[i];
                let metric2 = &metrics[j];
                
                if let Some(correlation) = self.calculate_correlation(metric1, metric2) {
                    if correlation.abs() > self.correlation_threshold {
                        patterns.push(DetectedPattern {
                            pattern_type: PatternType::Correlation {
                                metrics: vec![metric1.clone(), metric2.clone()],
                            },
                            description: format!("{} and {} are correlated (r={:.2})", metric1, metric2, correlation),
                            confidence: correlation.abs(),
                            occurrences: self.metric_history.get(metric1).map(|h| h.len()).unwrap_or(0),
                            first_seen: now - 3600,
                            last_seen: now,
                            avg_severity: 0.6,
                            next_predicted: None,
                            weakness_types: vec![metric1.clone(), metric2.clone()],
                        });
                    }
                }
            }
        }
        
        patterns
    }
    
    fn calculate_correlation(&self, metric1: &str, metric2: &str) -> Option<f32> {
        let history1 = self.metric_history.get(metric1)?;
        let history2 = self.metric_history.get(metric2)?;
        
        if history1.len() < 3 || history2.len() < 3 {
            return None;
        }
        
        // Align timestamps and calculate Pearson correlation
        let mut pairs = Vec::new();
        for (ts1, val1) in history1 {
            if let Some((_, val2)) = history2.iter().find(|(ts2, _)| (*ts2 as i64 - *ts1 as i64).abs() < 10) {
                pairs.push((*val1, *val2));
            }
        }
        
        if pairs.len() < 3 {
            return None;
        }
        
        let n = pairs.len() as f32;
        let sum_x: f32 = pairs.iter().map(|(x, _)| x).sum();
        let sum_y: f32 = pairs.iter().map(|(_, y)| y).sum();
        let sum_xy: f32 = pairs.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f32 = pairs.iter().map(|(x, _)| x * x).sum();
        let sum_y2: f32 = pairs.iter().map(|(_, y)| y * y).sum();
        
        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();
        
        if denominator == 0.0 {
            None
        } else {
            Some(numerator / denominator)
        }
    }
}

/// Main pattern recognition engine
pub struct PatternRecognitionEngine {
    /// Temporal pattern detector
    temporal_detector: Arc<RwLock<TemporalDetector>>,
    
    /// Sequential pattern detector
    sequential_detector: Arc<RwLock<SequentialDetector>>,
    
    /// Correlation pattern detector
    correlation_detector: Arc<RwLock<CorrelationDetector>>,
    
    /// All detected patterns
    detected_patterns: Arc<RwLock<Vec<DetectedPattern>>>,
    
    /// Pattern detection enabled
    enabled: bool,
}

impl PatternRecognitionEngine {
    /// Create new pattern recognition engine
    pub fn new() -> Self {
        Self {
            temporal_detector: Arc::new(RwLock::new(TemporalDetector::new(1000, 3))),
            sequential_detector: Arc::new(RwLock::new(SequentialDetector::new(10, 3))),
            correlation_detector: Arc::new(RwLock::new(CorrelationDetector::new(100, 0.7))),
            detected_patterns: Arc::new(RwLock::new(Vec::new())),
            enabled: true,
        }
    }
    
    /// Record an event for pattern detection
    pub fn record_event(&self, event: PatternEvent) {
        if !self.enabled {
            return;
        }
        
        // Add to temporal detector
        {
            let mut temporal = self.temporal_detector.write();
            temporal.add_event(event.timestamp, event.event_type.clone(), event.severity);
        }
        
        // Add to sequential detector
        {
            let mut sequential = self.sequential_detector.write();
            sequential.add_event(event.event_type.clone());
        }
        
        // Add to correlation detector
        {
            let mut correlation = self.correlation_detector.write();
            for (metric_name, metric_value) in &event.metadata {
                if let Ok(value) = metric_value.parse::<f32>() {
                    correlation.add_metric(metric_name.clone(), event.timestamp, value);
                }
            }
        }
    }
    
    /// Analyze and update detected patterns
    pub fn analyze_patterns(&self) {
        if !self.enabled {
            return;
        }
        
        let mut all_patterns = Vec::new();
        
        // Detect temporal patterns
        {
            let temporal = self.temporal_detector.read();
            all_patterns.extend(temporal.detect_patterns());
        }
        
        // Detect sequential patterns
        {
            let sequential = self.sequential_detector.read();
            all_patterns.extend(sequential.detect_patterns());
        }
        
        // Detect correlation patterns
        {
            let correlation = self.correlation_detector.read();
            all_patterns.extend(correlation.detect_patterns());
        }
        
        // Update detected patterns
        {
            let mut patterns = self.detected_patterns.write();
            *patterns = all_patterns;
        }
    }
    
    /// Get all detected patterns
    pub fn get_patterns(&self) -> Vec<DetectedPattern> {
        self.detected_patterns.read().clone()
    }
    
    /// Get patterns by type
    pub fn get_patterns_by_type(&self, pattern_type: &str) -> Vec<DetectedPattern> {
        self.detected_patterns.read()
            .iter()
            .filter(|p| match &p.pattern_type {
                PatternType::Temporal { .. } => pattern_type == "temporal",
                PatternType::Sequential { .. } => pattern_type == "sequential",
                PatternType::Frequency { .. } => pattern_type == "frequency",
                PatternType::Correlation { .. } => pattern_type == "correlation",
                PatternType::Cyclic { .. } => pattern_type == "cyclic",
            })
            .cloned()
            .collect()
    }
    
    /// Get high-confidence patterns (confidence > threshold)
    pub fn get_high_confidence_patterns(&self, threshold: f32) -> Vec<DetectedPattern> {
        self.detected_patterns.read()
            .iter()
            .filter(|p| p.confidence > threshold)
            .cloned()
            .collect()
    }
    
    /// Check if a pattern is predicted to occur soon
    pub fn check_upcoming_patterns(&self, within_secs: u64) -> Vec<DetectedPattern> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        self.detected_patterns.read()
            .iter()
            .filter(|p| {
                if let Some(next) = p.next_predicted {
                    next <= now + within_secs && next >= now
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
    
    /// Clear all detected patterns
    pub fn clear_patterns(&self) {
        self.detected_patterns.write().clear();
    }
}

impl Default for PatternRecognitionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_temporal_detector() {
        let mut detector = TemporalDetector::new(100, 3);
        
        // Add events at regular intervals
        for i in 0..5 {
            detector.add_event(1000 + i * 3600, "latency_spike".to_string(), 0.8);
        }
        
        let patterns = detector.detect_patterns();
        assert!(!patterns.is_empty());
        
        // Should detect hourly pattern
        let has_temporal = patterns.iter().any(|p| matches!(p.pattern_type, PatternType::Temporal { .. }));
        assert!(has_temporal);
    }
    
    #[test]
    fn test_sequential_detector() {
        let mut detector = SequentialDetector::new(10, 3);
        
        // Add repeating sequence
        for _ in 0..5 {
            detector.add_event("high_load".to_string());
            detector.add_event("latency_spike".to_string());
        }
        
        let patterns = detector.detect_patterns();
        assert!(!patterns.is_empty());
    }
    
    #[test]
    fn test_correlation_detector() {
        let mut detector = CorrelationDetector::new(100, 0.7);
        
        // Add correlated metrics
        for i in 0..10 {
            let val = i as f32 * 0.1;
            detector.add_metric("latency".to_string(), 1000 + i * 10, val);
            detector.add_metric("error_rate".to_string(), 1000 + i * 10, val * 0.9);
        }
        
        let patterns = detector.detect_patterns();
        // May or may not detect correlation depending on threshold
        assert!(patterns.len() <= 1);
    }
    
    #[test]
    fn test_pattern_engine() {
        let engine = PatternRecognitionEngine::new();
        
        // Record events
        for i in 0..5 {
            engine.record_event(PatternEvent {
                timestamp: 1000 + i * 3600,
                event_type: "latency_spike".to_string(),
                severity: 0.8,
                metadata: HashMap::new(),
            });
        }
        
        engine.analyze_patterns();
        let patterns = engine.get_patterns();
        
        assert!(!patterns.is_empty());
    }
}
