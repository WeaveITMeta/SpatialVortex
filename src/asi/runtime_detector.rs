//! Runtime Weakness Detector
//!
//! Continuously monitors system metrics in real-time and detects:
//! - High latency spikes
//! - Low confidence trends
//! - Prediction error spikes
//! - Memory pressure
//! - Throughput degradation
//!
//! Auto-triggers RSI proposals when thresholds are exceeded.

use std::collections::{VecDeque, HashMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, interval};
use tokio::sync::mpsc;
use crate::asi::pattern_recognition::{PatternRecognitionEngine, PatternEvent, DetectedPattern};

/// Runtime weakness detector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDetectorConfig {
    /// Enable runtime detection
    pub enabled: bool,
    
    /// Monitoring interval in milliseconds
    pub monitor_interval_ms: u64,
    
    /// Window size for rolling statistics (number of samples)
    pub window_size: usize,
    
    /// Latency spike threshold (ms above baseline)
    pub latency_spike_threshold_ms: f32,
    
    /// Latency spike count before trigger
    pub latency_spike_count: usize,
    
    /// Confidence drop threshold (absolute drop)
    pub confidence_drop_threshold: f32,
    
    /// Confidence low threshold (absolute value)
    pub confidence_low_threshold: f32,
    
    /// Prediction error spike threshold (% increase)
    pub prediction_error_spike_threshold: f32,
    
    /// Memory pressure threshold (% of max)
    pub memory_pressure_threshold: f32,
    
    /// Throughput drop threshold (% decrease)
    pub throughput_drop_threshold: f32,
    
    /// Auto-trigger RSI on detection
    pub auto_trigger_rsi: bool,
    
    /// Cooldown period after trigger (seconds)
    pub trigger_cooldown_secs: u64,
    
    /// Enable pattern recognition
    pub enable_pattern_recognition: bool,
    
    /// Minimum pattern confidence to trigger (0-1)
    pub pattern_confidence_threshold: f32,
}

impl Default for RuntimeDetectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            monitor_interval_ms: 1000, // Check every second
            window_size: 60, // 60-second rolling window
            latency_spike_threshold_ms: 500.0, // 500ms above baseline
            latency_spike_count: 3, // 3 spikes in window
            confidence_drop_threshold: 0.15, // 15% drop
            confidence_low_threshold: 0.6, // Below 60%
            prediction_error_spike_threshold: 0.5, // 50% increase
            memory_pressure_threshold: 0.85, // 85% memory usage
            throughput_drop_threshold: 0.3, // 30% throughput drop
            auto_trigger_rsi: true,
            trigger_cooldown_secs: 300, // 5 minutes
            enable_pattern_recognition: true,
            pattern_confidence_threshold: 0.75,
        }
    }
}

/// Metric sample for rolling window
#[derive(Debug, Clone, Copy)]
struct MetricSample {
    timestamp: u64,
    latency_ms: f32,
    confidence: f32,
    prediction_error: f32,
    memory_usage_pct: f32,
    throughput: f32,
}

/// Detected runtime weakness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeWeakness {
    pub weakness_type: RuntimeWeaknessType,
    pub severity: f32,
    pub description: String,
    pub detected_at: u64,
    pub metric_value: f32,
    pub baseline_value: f32,
    pub samples_analyzed: usize,
}

/// Types of runtime weaknesses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeWeaknessType {
    LatencySpike,
    ConfidenceDrop,
    ConfidenceLow,
    PredictionErrorSpike,
    MemoryPressure,
    ThroughputDrop,
    RecurringPattern,
}

impl RuntimeWeaknessType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LatencySpike => "slow_reasoning",
            Self::ConfidenceDrop => "confidence_drop",
            Self::ConfidenceLow => "low_confidence",
            Self::PredictionErrorSpike => "high_error_rate",
            Self::MemoryPressure => "memory_leak",
            Self::ThroughputDrop => "slow_reasoning",
            Self::RecurringPattern => "recurring_pattern",
        }
    }
}

/// Runtime statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    pub samples_collected: u64,
    pub weaknesses_detected: u64,
    pub rsi_triggers: u64,
    pub avg_latency_ms: f32,
    pub avg_confidence: f32,
    pub avg_prediction_error: f32,
    pub current_memory_pct: f32,
    pub current_throughput: f32,
    pub last_trigger_time: Option<u64>,
}

/// Trigger event sent to orchestrator
#[derive(Debug, Clone)]
pub struct RSITriggerEvent {
    pub weaknesses: Vec<RuntimeWeakness>,
    pub triggered_at: u64,
}

/// Runtime weakness detector
pub struct RuntimeWeaknessDetector {
    config: Arc<RwLock<RuntimeDetectorConfig>>,
    
    /// Rolling window of metric samples
    samples: Arc<RwLock<VecDeque<MetricSample>>>,
    
    /// Detected weaknesses history
    weaknesses: Arc<RwLock<Vec<RuntimeWeakness>>>,
    
    /// Pattern recognition engine
    pattern_engine: Arc<PatternRecognitionEngine>,
    
    /// Statistics
    samples_collected: AtomicU64,
    weaknesses_detected: AtomicU64,
    rsi_triggers: AtomicU64,
    last_trigger_time: AtomicU64,
    
    /// Running flag
    running: AtomicBool,
    
    /// Channel for RSI trigger events
    trigger_tx: Option<mpsc::UnboundedSender<RSITriggerEvent>>,
}

impl RuntimeWeaknessDetector {
    /// Create new runtime detector
    pub fn new(config: RuntimeDetectorConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            samples: Arc::new(RwLock::new(VecDeque::new())),
            weaknesses: Arc::new(RwLock::new(Vec::new())),
            pattern_engine: Arc::new(PatternRecognitionEngine::new()),
            samples_collected: AtomicU64::new(0),
            weaknesses_detected: AtomicU64::new(0),
            rsi_triggers: AtomicU64::new(0),
            last_trigger_time: AtomicU64::new(0),
            running: AtomicBool::new(false),
            trigger_tx: None,
        }
    }
    
    /// Create with trigger channel
    pub fn with_trigger_channel(mut self, tx: mpsc::UnboundedSender<RSITriggerEvent>) -> Self {
        self.trigger_tx = Some(tx);
        self
    }
    
    /// Record a metric sample manually
    pub fn record_sample(&self, latency_ms: f32, confidence: f32, prediction_error: f32) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let sample = MetricSample {
            timestamp: now,
            latency_ms,
            confidence,
            prediction_error,
            memory_usage_pct: Self::get_memory_usage(),
            throughput: 0.0, // Would be calculated from request rate
        };
        
        let mut samples = self.samples.write();
        samples.push_back(sample);
        
        let config = self.config.read();
        if samples.len() > config.window_size {
            samples.pop_front();
        }
        
        self.samples_collected.fetch_add(1, Ordering::Relaxed);
        
        // Feed pattern recognition engine
        if config.enable_pattern_recognition {
            let mut metadata = HashMap::new();
            metadata.insert("latency_ms".to_string(), latency_ms.to_string());
            metadata.insert("confidence".to_string(), confidence.to_string());
            metadata.insert("prediction_error".to_string(), prediction_error.to_string());
            
            // Determine event type based on metrics
            let event_type = if latency_ms > 500.0 {
                "latency_spike"
            } else if confidence < 0.6 {
                "low_confidence"
            } else if prediction_error > 0.3 {
                "prediction_error"
            } else {
                "normal"
            };
            
            let severity = (latency_ms / 1000.0).max(1.0 - confidence).max(prediction_error);
            
            self.pattern_engine.record_event(PatternEvent {
                timestamp: now / 1000, // Convert to seconds
                event_type: event_type.to_string(),
                severity,
                metadata,
            });
        }
    }
    
    /// Start runtime monitoring
    pub async fn start(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.read().clone();
        
        if !config.enabled {
            tracing::warn!("Runtime detector is disabled");
            return tokio::spawn(async {});
        }
        
        self.running.store(true, Ordering::SeqCst);
        
        let samples = self.samples.clone();
        let weaknesses = self.weaknesses.clone();
        let config_arc = self.config.clone();
        let samples_collected = Arc::new(AtomicU64::new(self.samples_collected.load(Ordering::Relaxed)));
        let weaknesses_detected = Arc::new(AtomicU64::new(self.weaknesses_detected.load(Ordering::Relaxed)));
        let rsi_triggers = Arc::new(AtomicU64::new(self.rsi_triggers.load(Ordering::Relaxed)));
        let last_trigger_time = Arc::new(AtomicU64::new(self.last_trigger_time.load(Ordering::Relaxed)));
        let running = Arc::new(AtomicBool::new(self.running.load(Ordering::Relaxed)));
        let trigger_tx = self.trigger_tx.clone();
        let pattern_engine = self.pattern_engine.clone();
        
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(config.monitor_interval_ms));
            
            tracing::info!("Runtime weakness detector started (interval: {}ms)", 
                config.monitor_interval_ms);
            
            while running.load(Ordering::SeqCst) {
                ticker.tick().await;
                
                // Analyze for weaknesses
                let config = config_arc.read();
                let mut detected = Self::analyze_weaknesses(&samples, &config);
                
                // Analyze patterns if enabled
                if config.enable_pattern_recognition {
                    pattern_engine.analyze_patterns();
                    let patterns = pattern_engine.get_high_confidence_patterns(config.pattern_confidence_threshold);
                    
                    // Convert patterns to weaknesses
                    for pattern in patterns {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        
                        detected.push(RuntimeWeakness {
                            weakness_type: RuntimeWeaknessType::RecurringPattern,
                            severity: pattern.confidence * pattern.avg_severity,
                            description: pattern.description.clone(),
                            detected_at: now,
                            metric_value: pattern.avg_severity,
                            baseline_value: 0.5,
                            samples_analyzed: pattern.occurrences,
                        });
                    }
                }
                
                drop(config);
                
                if !detected.is_empty() {
                    weaknesses_detected.fetch_add(detected.len() as u64, Ordering::Relaxed);
                    
                    // Store weaknesses
                    {
                        let mut weaknesses_guard = weaknesses.write();
                        weaknesses_guard.extend(detected.clone());
                        
                        // Keep only recent weaknesses (last 1000)
                        let len = weaknesses_guard.len();
                        if len > 1000 {
                            weaknesses_guard.drain(0..len - 1000);
                        }
                    }
                    
                    // Check if we should trigger RSI
                    let config = config_arc.read();
                    if config.auto_trigger_rsi {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        
                        let last_trigger = last_trigger_time.load(Ordering::Relaxed);
                        let cooldown_elapsed = now.saturating_sub(last_trigger) > config.trigger_cooldown_secs;
                        
                        if cooldown_elapsed {
                            tracing::warn!("Runtime detector triggering RSI for {} weaknesses", detected.len());
                            
                            for weakness in &detected {
                                tracing::warn!("  - {:?}: {} (severity: {:.2})", 
                                    weakness.weakness_type, 
                                    weakness.description,
                                    weakness.severity);
                            }
                            
                            last_trigger_time.store(now, Ordering::Relaxed);
                            rsi_triggers.fetch_add(1, Ordering::Relaxed);
                            
                            // Send trigger event if channel available
                            if let Some(ref tx) = trigger_tx {
                                let event = RSITriggerEvent {
                                    weaknesses: detected.clone(),
                                    triggered_at: now,
                                };
                                let _ = tx.send(event);
                            }
                        }
                    }
                }
            }
            
            tracing::info!("Runtime weakness detector stopped");
        })
    }
    
    /// Stop runtime monitoring
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
    
    /// Check if detector is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// Get current memory usage percentage
    fn get_memory_usage() -> f32 {
        // In production, would use system APIs
        // For now, return simulated value
        0.5
    }
    
    /// Analyze samples for weaknesses
    fn analyze_weaknesses(
        samples: &Arc<RwLock<VecDeque<MetricSample>>>,
        config: &RuntimeDetectorConfig,
    ) -> Vec<RuntimeWeakness> {
        let samples_guard = samples.read();
        
        if samples_guard.len() < 10 {
            return Vec::new(); // Need minimum samples
        }
        
        let mut weaknesses = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Calculate baselines
        let baseline_latency = Self::calculate_baseline_latency(&samples_guard);
        let baseline_confidence = Self::calculate_baseline_confidence(&samples_guard);
        let baseline_error = Self::calculate_baseline_error(&samples_guard);
        let baseline_throughput = Self::calculate_baseline_throughput(&samples_guard);
        
        // Get recent samples (last 10)
        let recent_count = 10.min(samples_guard.len());
        let recent: Vec<_> = samples_guard.iter()
            .rev()
            .take(recent_count)
            .cloned()
            .collect();
        
        // 1. Detect latency spikes
        let spike_count = recent.iter()
            .filter(|s| s.latency_ms > baseline_latency + config.latency_spike_threshold_ms)
            .count();
        
        if spike_count >= config.latency_spike_count {
            let max_latency = recent.iter()
                .map(|s| s.latency_ms)
                .fold(0.0f32, f32::max);
            
            weaknesses.push(RuntimeWeakness {
                weakness_type: RuntimeWeaknessType::LatencySpike,
                severity: (max_latency - baseline_latency) / baseline_latency,
                description: format!("{} latency spikes detected (max: {:.0}ms, baseline: {:.0}ms)", 
                    spike_count, max_latency, baseline_latency),
                detected_at: now,
                metric_value: max_latency,
                baseline_value: baseline_latency,
                samples_analyzed: recent.len(),
            });
        }
        
        // 2. Detect confidence drop
        let recent_confidence = recent.iter()
            .map(|s| s.confidence)
            .sum::<f32>() / recent.len() as f32;
        
        let confidence_drop = baseline_confidence - recent_confidence;
        if confidence_drop > config.confidence_drop_threshold {
            weaknesses.push(RuntimeWeakness {
                weakness_type: RuntimeWeaknessType::ConfidenceDrop,
                severity: confidence_drop / baseline_confidence,
                description: format!("Confidence dropped {:.1}% (from {:.2} to {:.2})", 
                    confidence_drop * 100.0, baseline_confidence, recent_confidence),
                detected_at: now,
                metric_value: recent_confidence,
                baseline_value: baseline_confidence,
                samples_analyzed: recent.len(),
            });
        }
        
        // 3. Detect low confidence
        if recent_confidence < config.confidence_low_threshold {
            weaknesses.push(RuntimeWeakness {
                weakness_type: RuntimeWeaknessType::ConfidenceLow,
                severity: (config.confidence_low_threshold - recent_confidence) / config.confidence_low_threshold,
                description: format!("Confidence below threshold ({:.2} < {:.2})", 
                    recent_confidence, config.confidence_low_threshold),
                detected_at: now,
                metric_value: recent_confidence,
                baseline_value: config.confidence_low_threshold,
                samples_analyzed: recent.len(),
            });
        }
        
        // 4. Detect prediction error spike
        let recent_error = recent.iter()
            .map(|s| s.prediction_error)
            .sum::<f32>() / recent.len() as f32;
        
        let error_increase = (recent_error - baseline_error) / baseline_error.max(0.001);
        if error_increase > config.prediction_error_spike_threshold {
            weaknesses.push(RuntimeWeakness {
                weakness_type: RuntimeWeaknessType::PredictionErrorSpike,
                severity: error_increase,
                description: format!("Prediction error spiked {:.1}% (from {:.3} to {:.3})", 
                    error_increase * 100.0, baseline_error, recent_error),
                detected_at: now,
                metric_value: recent_error,
                baseline_value: baseline_error,
                samples_analyzed: recent.len(),
            });
        }
        
        // 5. Detect memory pressure
        let recent_memory = recent.iter()
            .map(|s| s.memory_usage_pct)
            .sum::<f32>() / recent.len() as f32;
        
        if recent_memory > config.memory_pressure_threshold {
            weaknesses.push(RuntimeWeakness {
                weakness_type: RuntimeWeaknessType::MemoryPressure,
                severity: (recent_memory - config.memory_pressure_threshold) / (1.0 - config.memory_pressure_threshold),
                description: format!("Memory usage at {:.1}% (threshold: {:.1}%)", 
                    recent_memory * 100.0, config.memory_pressure_threshold * 100.0),
                detected_at: now,
                metric_value: recent_memory,
                baseline_value: config.memory_pressure_threshold,
                samples_analyzed: recent.len(),
            });
        }
        
        // 6. Detect throughput drop
        if baseline_throughput > 0.0 {
            let recent_throughput = recent.iter()
                .map(|s| s.throughput)
                .sum::<f32>() / recent.len() as f32;
            
            let throughput_drop = (baseline_throughput - recent_throughput) / baseline_throughput;
            if throughput_drop > config.throughput_drop_threshold {
                weaknesses.push(RuntimeWeakness {
                    weakness_type: RuntimeWeaknessType::ThroughputDrop,
                    severity: throughput_drop,
                    description: format!("Throughput dropped {:.1}% (from {:.0} to {:.0} req/s)", 
                        throughput_drop * 100.0, baseline_throughput, recent_throughput),
                    detected_at: now,
                    metric_value: recent_throughput,
                    baseline_value: baseline_throughput,
                    samples_analyzed: recent.len(),
                });
            }
        }
        
        weaknesses
    }
    
    /// Calculate baseline latency (median of older samples)
    fn calculate_baseline_latency(samples: &VecDeque<MetricSample>) -> f32 {
        if samples.len() < 20 {
            return samples.iter().map(|s| s.latency_ms).sum::<f32>() / samples.len() as f32;
        }
        
        // Use first half as baseline
        let baseline_count = samples.len() / 2;
        samples.iter()
            .take(baseline_count)
            .map(|s| s.latency_ms)
            .sum::<f32>() / baseline_count as f32
    }
    
    /// Calculate baseline confidence
    fn calculate_baseline_confidence(samples: &VecDeque<MetricSample>) -> f32 {
        if samples.len() < 20 {
            return samples.iter().map(|s| s.confidence).sum::<f32>() / samples.len() as f32;
        }
        
        let baseline_count = samples.len() / 2;
        samples.iter()
            .take(baseline_count)
            .map(|s| s.confidence)
            .sum::<f32>() / baseline_count as f32
    }
    
    /// Calculate baseline prediction error
    fn calculate_baseline_error(samples: &VecDeque<MetricSample>) -> f32 {
        if samples.len() < 20 {
            return samples.iter().map(|s| s.prediction_error).sum::<f32>() / samples.len() as f32;
        }
        
        let baseline_count = samples.len() / 2;
        samples.iter()
            .take(baseline_count)
            .map(|s| s.prediction_error)
            .sum::<f32>() / baseline_count as f32
    }
    
    /// Calculate baseline throughput
    fn calculate_baseline_throughput(samples: &VecDeque<MetricSample>) -> f32 {
        if samples.len() < 20 {
            return samples.iter().map(|s| s.throughput).sum::<f32>() / samples.len() as f32;
        }
        
        let baseline_count = samples.len() / 2;
        samples.iter()
            .take(baseline_count)
            .map(|s| s.throughput)
            .sum::<f32>() / baseline_count as f32
    }
    
    /// Get runtime statistics
    pub fn stats(&self) -> RuntimeStats {
        let samples = self.samples.read();
        
        let (avg_latency, avg_confidence, avg_error, current_memory, current_throughput) = 
            if samples.is_empty() {
                (0.0, 0.0, 0.0, 0.0, 0.0)
            } else {
                let count = samples.len() as f32;
                (
                    samples.iter().map(|s| s.latency_ms).sum::<f32>() / count,
                    samples.iter().map(|s| s.confidence).sum::<f32>() / count,
                    samples.iter().map(|s| s.prediction_error).sum::<f32>() / count,
                    samples.back().map(|s| s.memory_usage_pct).unwrap_or(0.0),
                    samples.back().map(|s| s.throughput).unwrap_or(0.0),
                )
            };
        
        let last_trigger = self.last_trigger_time.load(Ordering::Relaxed);
        
        RuntimeStats {
            samples_collected: self.samples_collected.load(Ordering::Relaxed),
            weaknesses_detected: self.weaknesses_detected.load(Ordering::Relaxed),
            rsi_triggers: self.rsi_triggers.load(Ordering::Relaxed),
            avg_latency_ms: avg_latency,
            avg_confidence,
            avg_prediction_error: avg_error,
            current_memory_pct: current_memory,
            current_throughput,
            last_trigger_time: if last_trigger > 0 { Some(last_trigger) } else { None },
        }
    }
    
    /// Get recent weaknesses
    pub fn get_recent_weaknesses(&self, count: usize) -> Vec<RuntimeWeakness> {
        let weaknesses = self.weaknesses.read();
        weaknesses.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
    
    /// Update configuration
    pub fn update_config(&self, config: RuntimeDetectorConfig) {
        let mut cfg = self.config.write();
        *cfg = config;
    }
    
    /// Get all detected patterns
    pub fn get_detected_patterns(&self) -> Vec<DetectedPattern> {
        self.pattern_engine.get_patterns()
    }
    
    /// Get patterns by type
    pub fn get_patterns_by_type(&self, pattern_type: &str) -> Vec<DetectedPattern> {
        self.pattern_engine.get_patterns_by_type(pattern_type)
    }
    
    /// Check for upcoming predicted patterns
    pub fn check_upcoming_patterns(&self, within_secs: u64) -> Vec<DetectedPattern> {
        self.pattern_engine.check_upcoming_patterns(within_secs)
    }
    
    /// Clear pattern history
    pub fn clear_patterns(&self) {
        self.pattern_engine.clear_patterns();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_detector_creation() {
        let config = RuntimeDetectorConfig::default();
        let detector = RuntimeWeaknessDetector::new(config);
        
        assert!(!detector.is_running());
        assert_eq!(detector.samples_collected.load(Ordering::Relaxed), 0);
    }
    
    #[tokio::test]
    async fn test_record_sample() {
        let config = RuntimeDetectorConfig::default();
        let detector = RuntimeWeaknessDetector::new(config);
        
        detector.record_sample(250.0, 0.8, 0.1);
        detector.record_sample(300.0, 0.75, 0.12);
        
        assert_eq!(detector.samples_collected.load(Ordering::Relaxed), 2);
        
        let stats = detector.stats();
        assert!(stats.avg_latency_ms > 0.0);
        assert!(stats.avg_confidence > 0.0);
    }
    
    #[tokio::test]
    async fn test_weakness_detection() {
        let mut config = RuntimeDetectorConfig::default();
        config.confidence_low_threshold = 0.9; // High threshold to trigger
        
        let detector = RuntimeWeaknessDetector::new(config);
        
        // Record samples with low confidence
        for _ in 0..20 {
            detector.record_sample(200.0, 0.5, 0.1); // Low confidence
        }
        
        // Analyze
        let samples = detector.samples.clone();
        let config = detector.config.read();
        let weaknesses = RuntimeWeaknessDetector::analyze_weaknesses(&samples, &config);
        
        // Should detect low confidence
        assert!(!weaknesses.is_empty());
        assert!(weaknesses.iter().any(|w| matches!(w.weakness_type, RuntimeWeaknessType::ConfidenceLow)));
    }
}
