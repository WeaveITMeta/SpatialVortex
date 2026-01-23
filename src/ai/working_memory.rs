//! Working Memory System for AGI
//!
//! Provides short-term context retention across reasoning steps:
//! - Attention-based memory slots with decay
//! - Semantic chunking for efficient storage
//! - Priority-based retention (sacred positions get priority)
//! - Integration with Flux Reasoning and Confidence Lake
//!
//! ## Architecture
//!
//! ```text
//! Sensory Input → Working Memory Buffer → Attention Filter → Active Slots
//!                        ↓                      ↓
//!                   Decay/Refresh          Priority Queue
//!                        ↓                      ↓
//!                 Confidence Lake ←── High-Value Consolidation
//! ```

use crate::data::models::ELPTensor;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

// ============================================================================
// Core Memory Structures
// ============================================================================

/// A single memory item in working memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    /// Unique identifier
    pub id: Uuid,
    
    /// Content of the memory (semantic representation)
    pub content: MemoryContent,
    
    /// ELP profile of this memory
    pub elp_profile: ELPTensor,
    
    /// Activation level (0.0-1.0) - decays over time
    pub activation: f32,
    
    /// Importance score (determines consolidation priority)
    pub importance: f32,
    
    /// Sacred position influence (3, 6, 9 boost retention)
    pub sacred_influence: Option<u8>,
    
    /// Source of this memory
    pub source: MemorySource,
    
    /// When this memory was created
    pub created_at: DateTime<Utc>,
    
    /// Last time this memory was accessed
    pub last_accessed: DateTime<Utc>,
    
    /// Number of times accessed (rehearsal strengthens memory)
    pub access_count: u32,
    
    /// Links to related memories
    pub associations: Vec<Uuid>,
}

/// Content types that can be stored in working memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryContent {
    /// Raw text/semantic content
    Text(String),
    
    /// Structured fact (subject, predicate, object)
    Fact {
        subject: String,
        predicate: String,
        object: String,
    },
    
    /// Reasoning step from flux chain
    ReasoningStep {
        step_number: usize,
        vortex_position: u8,
        entropy: f32,
        insight: String,
    },
    
    /// Goal-related information
    GoalContext {
        goal_id: Uuid,
        objective: String,
        progress: f32,
    },
    
    /// Causal relationship
    CausalLink {
        cause: String,
        effect: String,
        strength: f32,
    },
    
    /// Pattern from meta-learning
    LearnedPattern {
        pattern_id: Uuid,
        domain: String,
        description: String,
    },
}

/// Source of a memory item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemorySource {
    /// From user input
    UserInput,
    /// From oracle/LLM query
    OracleResponse,
    /// From internal reasoning
    InternalReasoning,
    /// From goal planning
    GoalPlanning,
    /// From causal inference
    CausalInference,
    /// From meta-learning
    MetaLearning,
    /// Retrieved from Confidence Lake
    ConsolidatedMemory,
}

// ============================================================================
// Working Memory Buffer
// ============================================================================

/// The main working memory system
pub struct WorkingMemory {
    /// Active memory slots (limited capacity)
    pub slots: HashMap<Uuid, MemoryItem>,
    
    /// Maximum number of active slots
    pub capacity: usize,
    
    /// Recent access queue for LRU-like behavior
    pub access_queue: VecDeque<Uuid>,
    
    /// Attention weights for different sources
    pub attention_weights: AttentionWeights,
    
    /// Decay rate per second
    pub decay_rate: f32,
    
    /// Minimum activation to stay in memory
    pub activation_threshold: f32,
    
    /// Items pending consolidation to long-term storage
    pub consolidation_queue: Vec<MemoryItem>,
    
    /// Statistics
    pub stats: WorkingMemoryStats,
}

/// Attention weights for prioritizing different memory sources
#[derive(Debug, Clone)]
pub struct AttentionWeights {
    pub user_input: f32,
    pub oracle_response: f32,
    pub internal_reasoning: f32,
    pub goal_planning: f32,
    pub causal_inference: f32,
    pub meta_learning: f32,
    pub sacred_position_boost: f32,
}

impl Default for AttentionWeights {
    fn default() -> Self {
        Self {
            user_input: 1.0,
            oracle_response: 0.9,
            internal_reasoning: 0.8,
            goal_planning: 0.85,
            causal_inference: 0.75,
            meta_learning: 0.7,
            sacred_position_boost: 1.5, // 50% boost for sacred positions
        }
    }
}

/// Statistics for working memory operations
#[derive(Debug, Clone, Default)]
pub struct WorkingMemoryStats {
    pub items_stored: u64,
    pub items_retrieved: u64,
    pub items_decayed: u64,
    pub items_consolidated: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_activation: f32,
    pub avg_retention_time_ms: f64,
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new(7) // Miller's magic number: 7 ± 2
    }
}

impl WorkingMemory {
    /// Create new working memory with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: HashMap::with_capacity(capacity),
            capacity,
            access_queue: VecDeque::with_capacity(capacity * 2),
            attention_weights: AttentionWeights::default(),
            decay_rate: 0.1, // 10% decay per second without rehearsal
            activation_threshold: 0.3,
            consolidation_queue: Vec::new(),
            stats: WorkingMemoryStats::default(),
        }
    }
    
    /// Store a new item in working memory
    pub fn store(&mut self, content: MemoryContent, elp: &ELPTensor, source: MemorySource) -> Uuid {
        let id = Uuid::new_v4();
        
        // Calculate initial activation based on source and attention
        let base_activation = match source {
            MemorySource::UserInput => self.attention_weights.user_input,
            MemorySource::OracleResponse => self.attention_weights.oracle_response,
            MemorySource::InternalReasoning => self.attention_weights.internal_reasoning,
            MemorySource::GoalPlanning => self.attention_weights.goal_planning,
            MemorySource::CausalInference => self.attention_weights.causal_inference,
            MemorySource::MetaLearning => self.attention_weights.meta_learning,
            MemorySource::ConsolidatedMemory => 0.8,
        };
        
        // Calculate importance from ELP magnitude
        let importance = ((elp.ethos + elp.logos + elp.pathos) / 39.0) as f32;
        
        // Determine sacred influence
        let sacred_influence = self.determine_sacred_influence(elp);
        
        // Apply sacred boost if applicable
        let activation = if sacred_influence.is_some() {
            (base_activation * self.attention_weights.sacred_position_boost).min(1.0)
        } else {
            base_activation
        };
        
        let now = Utc::now();
        let item = MemoryItem {
            id,
            content,
            elp_profile: elp.clone(),
            activation,
            importance,
            sacred_influence,
            source,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            associations: Vec::new(),
        };
        
        // If at capacity, evict lowest activation item
        if self.slots.len() >= self.capacity {
            self.evict_lowest_activation();
        }
        
        self.slots.insert(id, item);
        self.access_queue.push_back(id);
        self.stats.items_stored += 1;
        
        tracing::debug!("Working memory stored: {:?} (activation: {:.2})", id, activation);
        
        id
    }
    
    /// Store a text memory (convenience method)
    pub fn store_text(&mut self, text: &str, elp: &ELPTensor, source: MemorySource) -> Uuid {
        self.store(MemoryContent::Text(text.to_string()), elp, source)
    }
    
    /// Store a reasoning step
    pub fn store_reasoning_step(
        &mut self,
        step: usize,
        position: u8,
        entropy: f32,
        insight: &str,
        elp: &ELPTensor,
    ) -> Uuid {
        self.store(
            MemoryContent::ReasoningStep {
                step_number: step,
                vortex_position: position,
                entropy,
                insight: insight.to_string(),
            },
            elp,
            MemorySource::InternalReasoning,
        )
    }
    
    /// Store a causal link
    pub fn store_causal_link(&mut self, cause: &str, effect: &str, strength: f32, elp: &ELPTensor) -> Uuid {
        self.store(
            MemoryContent::CausalLink {
                cause: cause.to_string(),
                effect: effect.to_string(),
                strength,
            },
            elp,
            MemorySource::CausalInference,
        )
    }
    
    /// Retrieve a memory by ID (increases activation)
    pub fn retrieve(&mut self, id: Uuid) -> Option<&MemoryItem> {
        if let Some(item) = self.slots.get_mut(&id) {
            // Rehearsal: boost activation
            item.activation = (item.activation + 0.2).min(1.0);
            item.last_accessed = Utc::now();
            item.access_count += 1;
            
            self.access_queue.push_back(id);
            self.stats.items_retrieved += 1;
            self.stats.cache_hits += 1;
            
            Some(item)
        } else {
            self.stats.cache_misses += 1;
            None
        }
    }
    
    /// Search for memories by content similarity
    pub fn search(&mut self, query: &str, limit: usize) -> Vec<Uuid> {
        let query_lower = query.to_lowercase();
        
        let mut matches: Vec<_> = self.slots.iter()
            .filter(|(_, item)| {
                match &item.content {
                    MemoryContent::Text(t) => t.to_lowercase().contains(&query_lower),
                    MemoryContent::Fact { subject, predicate, object } => {
                        subject.to_lowercase().contains(&query_lower)
                            || predicate.to_lowercase().contains(&query_lower)
                            || object.to_lowercase().contains(&query_lower)
                    },
                    MemoryContent::ReasoningStep { insight, .. } => {
                        insight.to_lowercase().contains(&query_lower)
                    },
                    MemoryContent::GoalContext { objective, .. } => {
                        objective.to_lowercase().contains(&query_lower)
                    },
                    MemoryContent::CausalLink { cause, effect, .. } => {
                        cause.to_lowercase().contains(&query_lower)
                            || effect.to_lowercase().contains(&query_lower)
                    },
                    MemoryContent::LearnedPattern { domain, description, .. } => {
                        domain.to_lowercase().contains(&query_lower)
                            || description.to_lowercase().contains(&query_lower)
                    },
                }
            })
            .map(|(id, item)| (*id, item.activation))
            .collect();
        
        // Sort by activation (most active first)
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        matches.truncate(limit);
        
        // Collect IDs to update
        let ids: Vec<Uuid> = matches.iter().map(|(id, _)| *id).collect();
        
        // Update access for retrieved items
        for id in &ids {
            if let Some(m) = self.slots.get_mut(id) {
                m.activation = (m.activation + 0.1).min(1.0);
                m.last_accessed = Utc::now();
            }
        }
        
        ids
    }
    
    /// Get memories by source type
    pub fn get_by_source(&self, source: MemorySource) -> Vec<&MemoryItem> {
        self.slots.values()
            .filter(|item| item.source == source)
            .collect()
    }
    
    /// Get memories with sacred influence
    pub fn get_sacred_memories(&self) -> Vec<&MemoryItem> {
        self.slots.values()
            .filter(|item| item.sacred_influence.is_some())
            .collect()
    }
    
    /// Apply time-based decay to all memories
    pub fn apply_decay(&mut self, elapsed_seconds: f32) {
        let decay_amount = self.decay_rate * elapsed_seconds;
        let mut to_remove = Vec::new();
        
        for (id, item) in self.slots.iter_mut() {
            // Sacred positions decay slower
            let effective_decay = if item.sacred_influence.is_some() {
                decay_amount * 0.5 // 50% slower decay
            } else {
                decay_amount
            };
            
            item.activation = (item.activation - effective_decay).max(0.0);
            
            if item.activation < self.activation_threshold {
                // Check if should consolidate before removing
                if item.importance > 0.6 || item.sacred_influence.is_some() {
                    self.consolidation_queue.push(item.clone());
                    self.stats.items_consolidated += 1;
                }
                to_remove.push(*id);
            }
        }
        
        for id in to_remove {
            self.slots.remove(&id);
            self.stats.items_decayed += 1;
        }
    }
    
    /// Refresh a memory (prevent decay)
    pub fn refresh(&mut self, id: Uuid) {
        if let Some(item) = self.slots.get_mut(&id) {
            item.activation = 1.0;
            item.last_accessed = Utc::now();
        }
    }
    
    /// Create association between two memories
    pub fn associate(&mut self, id1: Uuid, id2: Uuid) {
        if let Some(item1) = self.slots.get_mut(&id1) {
            if !item1.associations.contains(&id2) {
                item1.associations.push(id2);
            }
        }
        if let Some(item2) = self.slots.get_mut(&id2) {
            if !item2.associations.contains(&id1) {
                item2.associations.push(id1);
            }
        }
    }
    
    /// Get associated memories
    pub fn get_associations(&self, id: Uuid) -> Vec<&MemoryItem> {
        self.slots.get(&id)
            .map(|item| {
                item.associations.iter()
                    .filter_map(|assoc_id| self.slots.get(assoc_id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get items ready for consolidation to long-term storage
    pub fn get_consolidation_candidates(&mut self) -> Vec<MemoryItem> {
        std::mem::take(&mut self.consolidation_queue)
    }
    
    /// Get current memory state summary
    pub fn get_summary(&self) -> MemorySummary {
        let total = self.slots.len();
        let avg_activation = if total > 0 {
            self.slots.values().map(|i| i.activation).sum::<f32>() / total as f32
        } else {
            0.0
        };
        
        let sacred_count = self.slots.values()
            .filter(|i| i.sacred_influence.is_some())
            .count();
        
        MemorySummary {
            total_items: total,
            capacity: self.capacity,
            utilization: total as f32 / self.capacity as f32,
            avg_activation,
            sacred_items: sacred_count,
            pending_consolidation: self.consolidation_queue.len(),
        }
    }
    
    /// Clear all working memory
    pub fn clear(&mut self) {
        // Move important items to consolidation queue
        for item in self.slots.values() {
            if item.importance > 0.7 || item.sacred_influence.is_some() {
                self.consolidation_queue.push(item.clone());
            }
        }
        
        self.slots.clear();
        self.access_queue.clear();
    }
    
    // ========================================================================
    // Private helpers
    // ========================================================================
    
    fn determine_sacred_influence(&self, elp: &ELPTensor) -> Option<u8> {
        // Determine dominant dimension and map to sacred position
        if elp.ethos > elp.logos && elp.ethos > elp.pathos && elp.ethos > 8.0 {
            Some(3) // Ethos → Position 3
        } else if elp.logos > elp.pathos && elp.logos > 8.0 {
            Some(6) // Logos → Position 6
        } else if elp.pathos > 8.0 {
            Some(9) // Pathos → Position 9
        } else {
            None
        }
    }
    
    fn evict_lowest_activation(&mut self) {
        if let Some((id, item)) = self.slots.iter()
            .min_by(|a, b| a.1.activation.partial_cmp(&b.1.activation).unwrap())
            .map(|(id, item)| (*id, item.clone()))
        {
            let activation = item.activation;
            
            // Check if should consolidate
            if item.importance > 0.5 {
                self.consolidation_queue.push(item);
                self.stats.items_consolidated += 1;
            }
            
            self.slots.remove(&id);
            self.stats.items_decayed += 1;
            
            tracing::debug!("Evicted memory {:?} (activation: {:.2})", id, activation);
        }
    }
}

/// Summary of working memory state
#[derive(Debug, Clone)]
pub struct MemorySummary {
    pub total_items: usize,
    pub capacity: usize,
    pub utilization: f32,
    pub avg_activation: f32,
    pub sacred_items: usize,
    pub pending_consolidation: usize,
}

// ============================================================================
// Context Window Manager
// ============================================================================

/// Manages context window for reasoning chains
pub struct ContextWindow {
    /// Working memory instance
    pub memory: WorkingMemory,
    
    /// Current reasoning context
    pub context_stack: Vec<ContextFrame>,
    
    /// Maximum context depth
    pub max_depth: usize,
}

/// A frame in the context stack
#[derive(Debug, Clone)]
pub struct ContextFrame {
    pub id: Uuid,
    pub name: String,
    pub memory_ids: Vec<Uuid>,
    pub elp_state: ELPTensor,
    pub created_at: DateTime<Utc>,
}

impl Default for ContextWindow {
    fn default() -> Self {
        Self::new(9) // 9 slots for sacred geometry alignment
    }
}

impl ContextWindow {
    pub fn new(memory_capacity: usize) -> Self {
        Self {
            memory: WorkingMemory::new(memory_capacity),
            context_stack: Vec::new(),
            max_depth: 10,
        }
    }
    
    /// Push a new context frame
    pub fn push_context(&mut self, name: &str, elp: &ELPTensor) -> Uuid {
        let frame = ContextFrame {
            id: Uuid::new_v4(),
            name: name.to_string(),
            memory_ids: Vec::new(),
            elp_state: elp.clone(),
            created_at: Utc::now(),
        };
        
        let id = frame.id;
        
        if self.context_stack.len() >= self.max_depth {
            // Pop oldest context
            let old_frame = self.context_stack.remove(0);
            // Decay memories from old context
            for mem_id in old_frame.memory_ids {
                if let Some(item) = self.memory.slots.get_mut(&mem_id) {
                    item.activation *= 0.5;
                }
            }
        }
        
        self.context_stack.push(frame);
        id
    }
    
    /// Pop current context frame
    pub fn pop_context(&mut self) -> Option<ContextFrame> {
        self.context_stack.pop()
    }
    
    /// Get current context
    pub fn current_context(&self) -> Option<&ContextFrame> {
        self.context_stack.last()
    }
    
    /// Add memory to current context
    pub fn add_to_context(&mut self, memory_id: Uuid) {
        if let Some(frame) = self.context_stack.last_mut() {
            frame.memory_ids.push(memory_id);
        }
    }
    
    /// Get all memories in current context
    pub fn get_context_memories(&self) -> Vec<&MemoryItem> {
        self.current_context()
            .map(|frame| {
                frame.memory_ids.iter()
                    .filter_map(|id| self.memory.slots.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Store and add to context in one operation
    pub fn store_in_context(
        &mut self,
        content: MemoryContent,
        elp: &ELPTensor,
        source: MemorySource,
    ) -> Uuid {
        let id = self.memory.store(content, elp, source);
        self.add_to_context(id);
        id
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_working_memory_store_retrieve() {
        let mut wm = WorkingMemory::new(5);
        let elp = ELPTensor { ethos: 7.0, logos: 8.0, pathos: 5.0 };
        
        let id = wm.store_text("Test memory", &elp, MemorySource::UserInput);
        
        let retrieved = wm.retrieve(id);
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().activation > 0.9);
    }
    
    #[test]
    fn test_sacred_influence() {
        let mut wm = WorkingMemory::new(5);
        
        // High ethos should get position 3
        let elp_ethos = ELPTensor { ethos: 10.0, logos: 5.0, pathos: 3.0 };
        let id = wm.store_text("Ethical decision", &elp_ethos, MemorySource::InternalReasoning);
        
        let item = wm.slots.get(&id).unwrap();
        assert_eq!(item.sacred_influence, Some(3));
    }
    
    #[test]
    fn test_decay() {
        let mut wm = WorkingMemory::new(5);
        let elp = ELPTensor { ethos: 5.0, logos: 5.0, pathos: 5.0 };
        
        let id = wm.store_text("Decaying memory", &elp, MemorySource::InternalReasoning);
        let initial_activation = wm.slots.get(&id).unwrap().activation;
        
        wm.apply_decay(5.0); // 5 seconds of decay
        
        let final_activation = wm.slots.get(&id).unwrap().activation;
        assert!(final_activation < initial_activation);
    }
    
    #[test]
    fn test_capacity_eviction() {
        let mut wm = WorkingMemory::new(3);
        let elp = ELPTensor { ethos: 5.0, logos: 5.0, pathos: 5.0 };
        
        // Store 4 items (exceeds capacity of 3)
        for i in 0..4 {
            wm.store_text(&format!("Memory {}", i), &elp, MemorySource::UserInput);
        }
        
        assert_eq!(wm.slots.len(), 3);
    }
    
    #[test]
    fn test_context_window() {
        let mut ctx = ContextWindow::new(5);
        let elp = ELPTensor { ethos: 6.0, logos: 7.0, pathos: 5.0 };
        
        ctx.push_context("Reasoning about health", &elp);
        
        let mem_id = ctx.store_in_context(
            MemoryContent::Text("Exercise is important".to_string()),
            &elp,
            MemorySource::OracleResponse,
        );
        
        let memories = ctx.get_context_memories();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].id, mem_id);
    }
    
    #[test]
    fn test_associations() {
        let mut wm = WorkingMemory::new(5);
        let elp = ELPTensor { ethos: 5.0, logos: 5.0, pathos: 5.0 };
        
        let id1 = wm.store_text("Cause", &elp, MemorySource::CausalInference);
        let id2 = wm.store_text("Effect", &elp, MemorySource::CausalInference);
        
        wm.associate(id1, id2);
        
        let associations = wm.get_associations(id1);
        assert_eq!(associations.len(), 1);
        assert_eq!(associations[0].id, id2);
    }
}
