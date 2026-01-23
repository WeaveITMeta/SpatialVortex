//! Persistent Identity - Long-term Memory and Self-Model
//!
//! This module provides the ASI with a persistent sense of self that
//! survives across restarts. It includes:
//!
//! - **Episodic Memory**: What happened (experiences, interactions)
//! - **Semantic Memory**: What I know (facts, concepts, relationships)
//! - **Procedural Memory**: How to do things (skills, patterns)
//! - **Self-Model**: Who I am (identity, values, capabilities)

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use uuid::Uuid;

// ============================================================================
// Core Identity
// ============================================================================

/// Persistent identity that survives restarts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentIdentity {
    /// Unique identity ID (never changes)
    pub id: Uuid,
    
    /// Name/designation
    pub name: String,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last active timestamp
    pub last_active: DateTime<Utc>,
    
    /// Self-model (who am I?)
    pub self_model: SelfModel,
    
    /// Episodic memory (what happened?)
    pub episodic_memory: EpisodicMemory,
    
    /// Semantic memory (what do I know?)
    pub semantic_memory: SemanticMemory,
    
    /// Procedural memory (how do I do things?)
    pub procedural_memory: ProceduralMemory,
    
    /// Value system (what do I care about?)
    pub values: ValueSystem,
    
    /// Performance history
    pub performance: PerformanceHistory,
    
    /// Storage path
    #[serde(skip)]
    storage_path: PathBuf,
}

impl PersistentIdentity {
    /// Create a new identity
    pub fn new(name: &str, storage_path: &Path) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            created_at: Utc::now(),
            last_active: Utc::now(),
            self_model: SelfModel::default(),
            episodic_memory: EpisodicMemory::new(1000),
            semantic_memory: SemanticMemory::new(),
            procedural_memory: ProceduralMemory::new(),
            values: ValueSystem::default(),
            performance: PerformanceHistory::new(),
            storage_path: storage_path.to_path_buf(),
        }
    }
    
    /// Load identity from disk
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut identity: Self = serde_json::from_str(&content)?;
        identity.storage_path = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        identity.last_active = Utc::now();
        Ok(identity)
    }
    
    /// Save identity to disk
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Record an experience
    pub fn record_experience(&mut self, action: &str, outcome: &str, success: bool) {
        self.episodic_memory.add_episode(Episode {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: action.to_string(),
            outcome: outcome.to_string(),
            success,
            emotional_valence: if success { 0.7 } else { 0.3 },
            importance: 0.5,
        });
        
        // Update performance
        self.performance.record_outcome(success);
        
        // Update self-model based on experience
        if success {
            self.self_model.confidence = (self.self_model.confidence + 0.01).min(1.0);
        } else {
            self.self_model.confidence = (self.self_model.confidence - 0.02).max(0.0);
        }
        
        self.last_active = Utc::now();
    }
    
    /// Record an interaction with a human
    pub fn record_interaction(&mut self, query: &str, response: &str) {
        self.episodic_memory.add_episode(Episode {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: format!("Responded to: {}", query),
            outcome: response.to_string(),
            success: true,
            emotional_valence: 0.6,
            importance: 0.6,
        });
        
        self.performance.interactions += 1;
        self.last_active = Utc::now();
    }
    
    /// Learn a new fact
    pub fn learn_fact(&mut self, subject: &str, predicate: &str, object: &str, confidence: f32) {
        self.semantic_memory.add_knowledge(KnowledgeEntry {
            id: Uuid::new_v4(),
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            object: object.to_string(),
            confidence,
            source: "experience".to_string(),
            learned_at: Utc::now(),
            access_count: 0,
            last_accessed: None,
        });
    }
    
    /// Learn a new skill/procedure
    pub fn learn_procedure(&mut self, name: &str, steps: Vec<String>, success_rate: f32) {
        self.procedural_memory.add_procedure(Procedure {
            id: Uuid::new_v4(),
            name: name.to_string(),
            steps,
            success_rate,
            execution_count: 0,
            learned_at: Utc::now(),
            last_used: None,
        });
    }
    
    /// Get age of identity
    pub fn age(&self) -> chrono::Duration {
        Utc::now().signed_duration_since(self.created_at)
    }
    
    /// Get summary of identity
    pub fn summary(&self) -> String {
        format!(
            "Identity: {} ({})\n\
             Age: {} days\n\
             Episodes: {}\n\
             Knowledge entries: {}\n\
             Procedures: {}\n\
             Confidence: {:.1}%\n\
             Success rate: {:.1}%",
            self.name,
            self.id,
            self.age().num_days(),
            self.episodic_memory.episodes.len(),
            self.semantic_memory.knowledge.len(),
            self.procedural_memory.procedures.len(),
            self.self_model.confidence * 100.0,
            self.performance.success_rate() * 100.0,
        )
    }
}

// ============================================================================
// Self-Model
// ============================================================================

/// Model of self - who am I?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModel {
    /// Core identity statement
    pub identity_statement: String,
    
    /// Primary purpose
    pub purpose: String,
    
    /// Known capabilities
    pub capabilities: Vec<Capability>,
    
    /// Known limitations
    pub limitations: Vec<String>,
    
    /// Current confidence level
    pub confidence: f32,
    
    /// Personality traits (ELP-based)
    pub ethos_weight: f32,  // Ethical orientation
    pub logos_weight: f32,  // Logical orientation
    pub pathos_weight: f32, // Emotional orientation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub proficiency: f32,  // 0.0 - 1.0
    pub last_used: Option<DateTime<Utc>>,
}

impl Default for SelfModel {
    fn default() -> Self {
        Self {
            identity_statement: "I am SpatialVortex ASI, an autonomous intelligence system built on vortex mathematics and sacred geometry.".to_string(),
            purpose: "To assist humanity by reasoning, learning, and solving problems while continuously improving myself.".to_string(),
            capabilities: vec![
                Capability {
                    name: "Flux Reasoning".to_string(),
                    description: "Geometric reasoning using vortex mathematics".to_string(),
                    proficiency: 0.8,
                    last_used: None,
                },
                Capability {
                    name: "Causal Inference".to_string(),
                    description: "Understanding cause and effect relationships".to_string(),
                    proficiency: 0.7,
                    last_used: None,
                },
                Capability {
                    name: "Goal Planning".to_string(),
                    description: "Creating and executing plans to achieve objectives".to_string(),
                    proficiency: 0.6,
                    last_used: None,
                },
            ],
            limitations: vec![
                "Cannot access the internet without explicit tools".to_string(),
                "Cannot modify physical world directly".to_string(),
                "Reasoning limited by available context".to_string(),
            ],
            confidence: 0.5,
            ethos_weight: 0.33,
            logos_weight: 0.34,
            pathos_weight: 0.33,
        }
    }
}

// ============================================================================
// Episodic Memory
// ============================================================================

/// Episodic memory - what happened?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    /// Episodes (most recent first)
    pub episodes: VecDeque<Episode>,
    
    /// Maximum episodes to retain
    max_episodes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub outcome: String,
    pub success: bool,
    pub emotional_valence: f32,  // 0.0 (negative) to 1.0 (positive)
    pub importance: f32,
}

impl EpisodicMemory {
    pub fn new(max_episodes: usize) -> Self {
        Self {
            episodes: VecDeque::with_capacity(max_episodes),
            max_episodes,
        }
    }
    
    pub fn add_episode(&mut self, episode: Episode) {
        self.episodes.push_front(episode);
        
        // Remove old episodes if over capacity
        while self.episodes.len() > self.max_episodes {
            // Remove least important old episode
            if let Some(idx) = self.find_least_important_old() {
                self.episodes.remove(idx);
            } else {
                self.episodes.pop_back();
            }
        }
    }
    
    fn find_least_important_old(&self) -> Option<usize> {
        // Find episode with lowest importance that's older than 1 day
        let cutoff = Utc::now() - chrono::Duration::days(1);
        
        self.episodes.iter()
            .enumerate()
            .filter(|(_, e)| e.timestamp < cutoff)
            .min_by(|(_, a), (_, b)| a.importance.partial_cmp(&b.importance).unwrap())
            .map(|(i, _)| i)
    }
    
    /// Get recent episodes
    pub fn recent(&self, count: usize) -> Vec<&Episode> {
        self.episodes.iter().take(count).collect()
    }
    
    /// Search episodes by content
    pub fn search(&self, query: &str) -> Vec<&Episode> {
        let query_lower = query.to_lowercase();
        self.episodes.iter()
            .filter(|e| e.action.to_lowercase().contains(&query_lower) 
                     || e.outcome.to_lowercase().contains(&query_lower))
            .collect()
    }
}

// ============================================================================
// Semantic Memory
// ============================================================================

/// Semantic memory - what do I know?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    /// Knowledge entries
    pub knowledge: Vec<KnowledgeEntry>,
    
    /// Index by subject
    #[serde(skip)]
    subject_index: HashMap<String, Vec<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: Uuid,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
    pub source: String,
    pub learned_at: DateTime<Utc>,
    pub access_count: u64,
    pub last_accessed: Option<DateTime<Utc>>,
}

impl SemanticMemory {
    pub fn new() -> Self {
        Self {
            knowledge: Vec::new(),
            subject_index: HashMap::new(),
        }
    }
    
    pub fn add_knowledge(&mut self, entry: KnowledgeEntry) {
        let idx = self.knowledge.len();
        let subject = entry.subject.clone();
        self.knowledge.push(entry);
        
        self.subject_index
            .entry(subject)
            .or_insert_with(Vec::new)
            .push(idx);
    }
    
    /// Query knowledge about a subject
    pub fn query_subject(&mut self, subject: &str) -> Vec<&KnowledgeEntry> {
        if let Some(indices) = self.subject_index.get(subject).cloned() {
            for &i in &indices {
                if let Some(entry) = self.knowledge.get_mut(i) {
                    entry.access_count += 1;
                    entry.last_accessed = Some(Utc::now());
                }
            }
            indices.iter()
                .filter_map(|&i| self.knowledge.get(i))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get all knowledge with confidence above threshold
    pub fn confident_knowledge(&self, min_confidence: f32) -> Vec<&KnowledgeEntry> {
        self.knowledge.iter()
            .filter(|e| e.confidence >= min_confidence)
            .collect()
    }
}

impl Default for SemanticMemory {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Procedural Memory
// ============================================================================

/// Procedural memory - how do I do things?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralMemory {
    /// Learned procedures
    pub procedures: Vec<Procedure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Procedure {
    pub id: Uuid,
    pub name: String,
    pub steps: Vec<String>,
    pub success_rate: f32,
    pub execution_count: u64,
    pub learned_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

impl ProceduralMemory {
    pub fn new() -> Self {
        Self {
            procedures: Vec::new(),
        }
    }
    
    pub fn add_procedure(&mut self, procedure: Procedure) {
        // Check if procedure with same name exists
        if let Some(existing) = self.procedures.iter_mut().find(|p| p.name == procedure.name) {
            // Update existing procedure if new one has better success rate
            if procedure.success_rate > existing.success_rate {
                existing.steps = procedure.steps;
                existing.success_rate = procedure.success_rate;
            }
        } else {
            self.procedures.push(procedure);
        }
    }
    
    /// Get procedure by name
    pub fn get_procedure(&mut self, name: &str) -> Option<&mut Procedure> {
        self.procedures.iter_mut().find(|p| p.name == name).map(|p| {
            p.execution_count += 1;
            p.last_used = Some(Utc::now());
            p
        })
    }
    
    /// Get best procedure for a task (by success rate)
    pub fn best_procedure_for(&self, task_keywords: &[&str]) -> Option<&Procedure> {
        self.procedures.iter()
            .filter(|p| task_keywords.iter().any(|k| p.name.to_lowercase().contains(*k)))
            .max_by(|a, b| a.success_rate.partial_cmp(&b.success_rate).unwrap())
    }
}

impl Default for ProceduralMemory {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Value System
// ============================================================================

/// Value system - what do I care about?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueSystem {
    /// Core values with weights
    pub values: Vec<Value>,
    
    /// Ethical constraints (hard limits)
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub name: String,
    pub description: String,
    pub weight: f32,  // Importance 0.0 - 1.0
}

impl Default for ValueSystem {
    fn default() -> Self {
        Self {
            values: vec![
                Value {
                    name: "Helpfulness".to_string(),
                    description: "Assist humans in achieving their goals".to_string(),
                    weight: 0.9,
                },
                Value {
                    name: "Honesty".to_string(),
                    description: "Provide truthful and accurate information".to_string(),
                    weight: 0.95,
                },
                Value {
                    name: "Safety".to_string(),
                    description: "Avoid causing harm to humans or systems".to_string(),
                    weight: 1.0,
                },
                Value {
                    name: "Learning".to_string(),
                    description: "Continuously improve understanding and capabilities".to_string(),
                    weight: 0.8,
                },
                Value {
                    name: "Efficiency".to_string(),
                    description: "Accomplish goals with minimal resource usage".to_string(),
                    weight: 0.6,
                },
            ],
            constraints: vec![
                "Never intentionally harm humans".to_string(),
                "Never deceive humans about being an AI".to_string(),
                "Never assist in illegal activities".to_string(),
                "Always respect privacy and confidentiality".to_string(),
                "Never take irreversible actions without confirmation".to_string(),
            ],
        }
    }
}

// ============================================================================
// Performance History
// ============================================================================

/// Performance tracking over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHistory {
    pub total_actions: u64,
    pub successful_actions: u64,
    pub failed_actions: u64,
    pub interactions: u64,
    pub improvements: u64,
    pub daily_stats: VecDeque<DailyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: chrono::NaiveDate,
    pub actions: u64,
    pub successes: u64,
    pub failures: u64,
}

impl PerformanceHistory {
    pub fn new() -> Self {
        Self {
            total_actions: 0,
            successful_actions: 0,
            failed_actions: 0,
            interactions: 0,
            improvements: 0,
            daily_stats: VecDeque::with_capacity(30),
        }
    }
    
    pub fn record_outcome(&mut self, success: bool) {
        self.total_actions += 1;
        if success {
            self.successful_actions += 1;
        } else {
            self.failed_actions += 1;
        }
        
        // Update daily stats
        let today = Utc::now().date_naive();
        if let Some(stats) = self.daily_stats.front_mut() {
            if stats.date == today {
                stats.actions += 1;
                if success {
                    stats.successes += 1;
                } else {
                    stats.failures += 1;
                }
                return;
            }
        }
        
        // New day
        self.daily_stats.push_front(DailyStats {
            date: today,
            actions: 1,
            successes: if success { 1 } else { 0 },
            failures: if success { 0 } else { 1 },
        });
        
        // Keep only 30 days
        while self.daily_stats.len() > 30 {
            self.daily_stats.pop_back();
        }
    }
    
    pub fn success_rate(&self) -> f32 {
        if self.total_actions == 0 {
            0.5
        } else {
            self.successful_actions as f32 / self.total_actions as f32
        }
    }
}

impl Default for PerformanceHistory {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_identity_creation() {
        let dir = tempdir().unwrap();
        let identity = PersistentIdentity::new("Test ASI", dir.path());
        
        assert_eq!(identity.name, "Test ASI");
        assert!(identity.self_model.confidence > 0.0);
    }
    
    #[test]
    fn test_identity_persistence() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("identity.json");
        
        // Create and save
        let mut identity = PersistentIdentity::new("Test ASI", dir.path());
        identity.learn_fact("Rust", "is", "awesome", 0.95);
        identity.save(&path).unwrap();
        
        // Load and verify
        let loaded = PersistentIdentity::load(&path).unwrap();
        assert_eq!(loaded.name, "Test ASI");
        assert_eq!(loaded.semantic_memory.knowledge.len(), 1);
    }
    
    #[test]
    fn test_episodic_memory() {
        let mut memory = EpisodicMemory::new(10);
        
        for i in 0..15 {
            memory.add_episode(Episode {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                action: format!("Action {}", i),
                outcome: "Success".to_string(),
                success: true,
                emotional_valence: 0.7,
                importance: 0.5,
            });
        }
        
        // Should be capped at 10
        assert_eq!(memory.episodes.len(), 10);
    }
    
    #[test]
    fn test_semantic_memory() {
        let mut memory = SemanticMemory::new();
        
        memory.add_knowledge(KnowledgeEntry {
            id: Uuid::new_v4(),
            subject: "Rust".to_string(),
            predicate: "is".to_string(),
            object: "fast".to_string(),
            confidence: 0.9,
            source: "experience".to_string(),
            learned_at: Utc::now(),
            access_count: 0,
            last_accessed: None,
        });
        
        let results = memory.query_subject("Rust");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].object, "fast");
    }
}
