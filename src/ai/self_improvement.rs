//! Self-Improvement Loop for AGI
//!
//! Enables continuous self-improvement through:
//! - Architecture search (modify own neural architecture)
//! - Hyperparameter self-tuning
//! - Performance introspection
//! - Safe self-modification with rollback

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureConfig {
    pub id: Uuid,
    pub name: String,
    pub parameters: HashMap<String, ConfigValue>,
    pub performance_score: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    List(Vec<ConfigValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub accuracy: f32,
    pub latency_ms: f64,
    pub memory_mb: f64,
    pub throughput: f64,
    pub hallucination_rate: f32,
    pub confidence_avg: f32,
    pub sacred_coherence: f32,
    
    /// Sacred pattern coherence (3-6-9 recurrence tracking)
    pub sacred_pattern_coherence: f32,
    
    /// Sacred frequency (expected 0.333)
    pub sacred_frequency: f32,
    
    /// Digital root coherence
    pub digital_root_coherence: f32,
    
    /// Vortex cycle coherence
    pub vortex_cycle_coherence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: Uuid,
    pub hypothesis: String,
    pub config: ArchitectureConfig,
    pub baseline_metrics: PerformanceMetrics,
    pub experiment_metrics: Option<PerformanceMetrics>,
    pub improvement: Option<f32>,
    pub status: ExperimentStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentStatus { Pending, Running, Completed, Failed, RolledBack }

pub struct MetaLearner {
    pub current_config: ArchitectureConfig,
    pub config_history: Vec<ArchitectureConfig>,
    pub experiments: Vec<Experiment>,
    pub performance_tracker: PerformanceTracker,
    pub search_space: SearchSpace,
    pub stats: SelfImprovementStats,
}

#[derive(Debug, Clone, Default)]
pub struct SelfImprovementStats {
    pub experiments_run: u64,
    pub improvements_found: u64,
    pub rollbacks: u64,
    pub total_improvement: f32,
}

impl Default for MetaLearner {
    fn default() -> Self { Self::new() }
}

impl MetaLearner {
    pub fn new() -> Self {
        let default_config = ArchitectureConfig {
            id: Uuid::new_v4(),
            name: "default".to_string(),
            parameters: HashMap::from([
                ("entropy_threshold".to_string(), ConfigValue::Float(0.7)),
                ("sacred_weight_3".to_string(), ConfigValue::Float(1.5)),
                ("sacred_weight_6".to_string(), ConfigValue::Float(1.5)),
                ("sacred_weight_9".to_string(), ConfigValue::Float(1.5)),
                ("max_reasoning_steps".to_string(), ConfigValue::Int(20)),
                ("oracle_confidence_min".to_string(), ConfigValue::Float(0.8)),
            ]),
            performance_score: 0.5,
            created_at: Utc::now(),
        };
        
        Self {
            current_config: default_config,
            config_history: Vec::new(),
            experiments: Vec::new(),
            performance_tracker: PerformanceTracker::new(),
            search_space: SearchSpace::default(),
            stats: SelfImprovementStats::default(),
        }
    }
    
    /// Propose a new configuration based on performance analysis
    pub fn propose_improvement(&mut self) -> Result<ArchitectureConfig> {
        let metrics = self.performance_tracker.get_current_metrics();
        let mut new_config = self.current_config.clone();
        new_config.id = Uuid::new_v4();
        new_config.created_at = Utc::now();
        
        // Analyze weak points and propose changes
        if metrics.hallucination_rate > 0.1 {
            // High hallucination -> increase sacred weights
            if let Some(ConfigValue::Float(w)) = new_config.parameters.get_mut("sacred_weight_9") {
                *w = (*w * 1.1).min(2.0);
            }
        }
        
        if metrics.latency_ms > 100.0 {
            // High latency -> reduce max steps
            if let Some(ConfigValue::Int(s)) = new_config.parameters.get_mut("max_reasoning_steps") {
                *s = (*s - 2).max(5);
            }
        }
        
        if metrics.confidence_avg < 0.7 {
            // Low confidence -> lower entropy threshold
            if let Some(ConfigValue::Float(t)) = new_config.parameters.get_mut("entropy_threshold") {
                *t = (*t - 0.05).max(0.5);
            }
        }
        
        new_config.name = format!("config_v{}", self.config_history.len() + 1);
        Ok(new_config)
    }
    
    /// Run an experiment with a new configuration
    pub fn run_experiment(&mut self, hypothesis: &str, config: ArchitectureConfig) -> Result<Experiment> {
        let baseline = self.performance_tracker.get_current_metrics();
        
        let mut experiment = Experiment {
            id: Uuid::new_v4(),
            hypothesis: hypothesis.to_string(),
            config: config.clone(),
            baseline_metrics: baseline,
            experiment_metrics: None,
            improvement: None,
            status: ExperimentStatus::Running,
            created_at: Utc::now(),
        };
        
        // Simulate running with new config (in real system, would actually apply)
        let new_metrics = self.simulate_config(&config);
        let improvement = self.calculate_improvement(&experiment.baseline_metrics, &new_metrics);
        
        experiment.experiment_metrics = Some(new_metrics);
        experiment.improvement = Some(improvement);
        experiment.status = ExperimentStatus::Completed;
        
        self.stats.experiments_run += 1;
        
        if improvement > 0.0 {
            self.stats.improvements_found += 1;
            self.stats.total_improvement += improvement;
        }
        
        self.experiments.push(experiment.clone());
        Ok(experiment)
    }
    
    fn simulate_config(&self, config: &ArchitectureConfig) -> PerformanceMetrics {
        let base = self.performance_tracker.get_current_metrics();
        
        // Simulate effects of config changes
        let mut metrics = base.clone();
        
        if let Some(ConfigValue::Float(w)) = config.parameters.get("sacred_weight_9") {
            metrics.hallucination_rate *= (1.0 - (*w - 1.5) * 0.1) as f32;
            metrics.sacred_coherence *= (1.0 + (*w - 1.5) * 0.05) as f32;
        }
        
        if let Some(ConfigValue::Int(s)) = config.parameters.get("max_reasoning_steps") {
            metrics.latency_ms *= *s as f64 / 20.0;
        }
        
        metrics
    }
    
    fn calculate_improvement(&self, baseline: &PerformanceMetrics, new: &PerformanceMetrics) -> f32 {
        let accuracy_imp = (new.accuracy - baseline.accuracy) / baseline.accuracy.max(0.01);
        let latency_imp = (baseline.latency_ms - new.latency_ms) / baseline.latency_ms.max(1.0);
        let halluc_imp = (baseline.hallucination_rate - new.hallucination_rate) / baseline.hallucination_rate.max(0.01);
        
        accuracy_imp * 0.4 + latency_imp as f32 * 0.3 + halluc_imp * 0.3
    }
    
    /// Apply a successful configuration
    pub fn apply_config(&mut self, config: ArchitectureConfig) {
        self.config_history.push(self.current_config.clone());
        self.current_config = config;
        tracing::info!("Applied new config: {}", self.current_config.name);
    }
    
    /// Rollback to previous configuration
    pub fn rollback(&mut self) -> Result<()> {
        if let Some(prev) = self.config_history.pop() {
            self.current_config = prev;
            self.stats.rollbacks += 1;
            tracing::warn!("Rolled back to config: {}", self.current_config.name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No previous config to rollback to"))
        }
    }
    
    /// Get current hyperparameter value
    pub fn get_param<T: TryFrom<ConfigValue>>(&self, key: &str) -> Option<T> {
        self.current_config.parameters.get(key).and_then(|v| T::try_from(v.clone()).ok())
    }
}

impl TryFrom<ConfigValue> for f64 {
    type Error = ();
    fn try_from(v: ConfigValue) -> Result<Self, Self::Error> {
        match v { ConfigValue::Float(f) => Ok(f), ConfigValue::Int(i) => Ok(i as f64), _ => Err(()) }
    }
}

impl TryFrom<ConfigValue> for i64 {
    type Error = ();
    fn try_from(v: ConfigValue) -> Result<Self, Self::Error> {
        match v { ConfigValue::Int(i) => Ok(i), _ => Err(()) }
    }
}

pub struct PerformanceTracker {
    pub history: Vec<(DateTime<Utc>, PerformanceMetrics)>,
    pub current: PerformanceMetrics,
}

impl Default for PerformanceTracker {
    fn default() -> Self { Self::new() }
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            current: PerformanceMetrics {
                accuracy: 0.85, 
                latency_ms: 50.0, 
                memory_mb: 512.0,
                throughput: 100.0, 
                hallucination_rate: 0.05,
                confidence_avg: 0.75, 
                sacred_coherence: 0.8,
                sacred_pattern_coherence: 0.8,
                sacred_frequency: 0.33,
                digital_root_coherence: 0.8,
                vortex_cycle_coherence: 0.8,
            },
        }
    }
    
    pub fn record(&mut self, metrics: PerformanceMetrics) {
        self.history.push((Utc::now(), self.current.clone()));
        self.current = metrics;
    }
    
    pub fn get_current_metrics(&self) -> PerformanceMetrics { self.current.clone() }
    
    pub fn get_trend(&self, window: usize) -> f32 {
        if self.history.len() < 2 { return 0.0; }
        let recent: Vec<_> = self.history.iter().rev().take(window).collect();
        if recent.len() < 2 { return 0.0; }
        let first = &recent.last().unwrap().1;
        let last = &recent.first().unwrap().1;
        (last.accuracy - first.accuracy) / first.accuracy.max(0.01)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchSpace {
    pub parameters: HashMap<String, ParameterRange>,
}

#[derive(Debug, Clone)]
pub enum ParameterRange {
    IntRange { min: i64, max: i64, step: i64 },
    FloatRange { min: f64, max: f64, step: f64 },
    Categorical { options: Vec<String> },
}

impl SearchSpace {
    pub fn sample(&self, param: &str) -> Option<ConfigValue> {
        self.parameters.get(param).map(|range| match range {
            ParameterRange::IntRange { min, max, .. } => ConfigValue::Int((*min + *max) / 2),
            ParameterRange::FloatRange { min, max, .. } => ConfigValue::Float((*min + *max) / 2.0),
            ParameterRange::Categorical { options } => ConfigValue::String(options.first().cloned().unwrap_or_default()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_meta_learner() {
        let mut learner = MetaLearner::new();
        let new_config = learner.propose_improvement().unwrap();
        assert!(!new_config.parameters.is_empty());
    }
    
    #[test]
    fn test_experiment() {
        let mut learner = MetaLearner::new();
        let config = learner.propose_improvement().unwrap();
        let exp = learner.run_experiment("Test hypothesis", config).unwrap();
        assert_eq!(exp.status, ExperimentStatus::Completed);
    }
    
    #[test]
    fn test_rollback() {
        let mut learner = MetaLearner::new();
        let config = learner.propose_improvement().unwrap();
        learner.apply_config(config);
        assert!(learner.rollback().is_ok());
    }
}
