//! Meta-Learning System for SpatialVortex AGI
//!
//! Enables the AGI to:
//! - Extract patterns from successful reasoning chains
//! - Store patterns in Confidence Lake (PostgreSQL)
//! - Match new queries to similar patterns
//! - Accelerate reasoning by reusing proven pathways
//! - Learn and improve over time through feedback

use crate::ai::flux_reasoning::{FluxReasoningChain, EntropyType};
use crate::data::models::ELPTensor;
use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Core Data Structures
// ============================================================================

/// Reusable reasoning pattern extracted from successful chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPattern {
    // Identity
    pub pattern_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Query characteristics
    pub query_signature: QuerySignature,
    pub elp_profile: ELPTensor,
    pub entropy_type: EntropyType,
    
    // Solution pathway
    pub vortex_path: Vec<u8>,              // Positions visited (1→2→4→8→7→5→1)
    pub sacred_influences: Vec<u8>,        // Trinity positions that influenced (3, 6, 9)
    pub oracle_questions: Vec<String>,     // Effective questions asked
    pub key_transformations: Vec<TransformationSnapshot>,
    
    // Effectiveness metrics
    pub success_rate: f32,                 // 0.0-1.0
    pub avg_steps: usize,                  // Average steps to convergence
    pub confidence_achieved: f32,          // Typical final confidence
    pub reuse_count: u32,                  // Times pattern was successfully reused
    
    // Quality signals
    pub confidence: f32,              // Trinity coherence (≥0.6 required)
    pub efficiency_score: f32,             // Steps / baseline for domain
}

/// Signature of a query for pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySignature {
    pub domain: String,                    // "health", "math", "ethics", etc.
    pub complexity: f32,                   // 0.0 (simple) to 1.0 (complex)
    pub keywords: Vec<String>,             // Key terms from query
    pub elp_dominant: char,                // 'E', 'L', or 'P'
}

/// Snapshot of a key transformation in the reasoning pathway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationSnapshot {
    pub from_position: u8,
    pub to_position: u8,
    pub elp_delta: (f32, f32, f32),        // (ethos, logos, pathos) changes
    pub entropy_reduction: f32,
    pub description: String,
}

/// Result of attempting to accelerate a query with a pattern
#[derive(Debug, Clone)]
pub struct AccelerationResult {
    pub pattern_id: Uuid,
    pub steps_saved: usize,
    pub confidence_boost: f32,
    pub accelerated_chain: FluxReasoningChain,
}

/// Metrics tracking learning progress
#[derive(Debug, Clone, Default)]
pub struct LearningMetrics {
    pub patterns_extracted: u64,
    pub patterns_active: u64,
    pub patterns_pruned: u64,
    pub avg_reuse_count: f32,
    pub avg_success_rate: f32,
    pub acceleration_rate: f32,            // % of queries accelerated
    pub avg_speedup: f32,                  // Average steps saved
}

// ============================================================================
// Pattern Extractor
// ============================================================================

/// Extracts reusable patterns from successful reasoning chains
pub struct PatternExtractor {
    min_confidence: f32,
    min_sacred_milestones: usize,
    domain_baselines: DashMap<String, usize>,  // Average steps per domain
}

impl PatternExtractor {
    pub fn new() -> Self {
        Self {
            min_confidence: 0.7,
            min_sacred_milestones: 2,  // Must have 2+ trinity influences
            domain_baselines: DashMap::new(),
        }
    }
    
    /// Extract pattern from a completed reasoning chain
    pub fn extract(&self, chain: &FluxReasoningChain) -> Option<ReasoningPattern> {
        // Quality gates: Only extract from high-quality successful chains
        if !chain.has_converged() {
            tracing::debug!("Chain not converged - skipping extraction");
            return None;
        }
        
        if chain.chain_confidence < self.min_confidence {
            tracing::debug!("Chain confidence too low ({:.2}) - skipping", chain.chain_confidence);
            return None;
        }
        
        if chain.sacred_milestones.len() < self.min_sacred_milestones {
            tracing::debug!("Insufficient sacred milestones - skipping");
            return None;
        }
        
        // Analyze query to create signature
        let query_sig = self.analyze_query(&chain.query, &chain.thoughts[0].elp_state);
        
        // Extract solution pathway
        let vortex_path: Vec<u8> = chain.thoughts.iter()
            .map(|t| t.vortex_position)
            .collect();
        
        // Extract effective oracle questions
        let oracle_questions: Vec<String> = chain.thoughts.iter()
            .flat_map(|t| &t.oracle_contributions)
            .map(|o| o.question.clone())
            .collect();
        
        // Extract key transformations
        let key_transformations = self.extract_transformations(chain);
        
        // Calculate pattern quality metrics
        let confidence = self.compute_confidence(chain);
        let efficiency = self.compute_efficiency(&query_sig.domain, chain.thoughts.len());
        
        let pattern = ReasoningPattern {
            pattern_id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            query_signature: query_sig,
            elp_profile: chain.current_thought().elp_state.clone(),
            entropy_type: chain.thoughts[0].entropy_type,
            vortex_path,
            sacred_influences: chain.sacred_milestones.clone(),
            oracle_questions,
            key_transformations,
            success_rate: 1.0,  // Initial success rate
            avg_steps: chain.thoughts.len(),
            confidence_achieved: chain.chain_confidence,
            reuse_count: 0,
            confidence,
            efficiency_score: efficiency.min(1.0),
        };
        
        tracing::info!("📚 Pattern extracted: domain={}, steps={}, signal={:.2}", 
            pattern.query_signature.domain, pattern.avg_steps, pattern.confidence);
        
        Some(pattern)
    }
    
    /// Analyze query to create signature for matching
    fn analyze_query(&self, query: &str, initial_elp: &ELPTensor) -> QuerySignature {
        let query_lower = query.to_lowercase();
        
        // Determine domain from keywords
        let domain = if query_lower.contains("health") || query_lower.contains("diabetes") 
                        || query_lower.contains("medical") {
            "health"
        } else if query_lower.contains("math") || query_lower.contains("calculate") 
                    || query_lower.contains("equation") {
            "math"
        } else if query_lower.contains("ethics") || query_lower.contains("moral") 
                    || query_lower.contains("should") {
            "ethics"
        } else if query_lower.contains("code") || query_lower.contains("program") {
            "technology"
        } else {
            "general"
        };
        
        // Extract keywords (simple tokenization)
        let keywords: Vec<String> = query.split_whitespace()
            .filter(|w| w.len() > 3)  // Skip short words
            .map(|w| w.to_lowercase())
            .collect();
        
        // Estimate complexity based on question structure
        let complexity = if query.contains('?') && query.split('?').count() > 2 {
            0.8  // Multiple questions = complex
        } else if query.len() > 100 {
            0.7  // Long query = moderately complex
        } else {
            0.5  // Default
        };
        
        // Determine dominant ELP dimension
        let elp_dominant = if initial_elp.ethos > initial_elp.logos && initial_elp.ethos > initial_elp.pathos {
            'E'
        } else if initial_elp.logos > initial_elp.pathos {
            'L'
        } else {
            'P'
        };
        
        QuerySignature {
            domain: domain.to_string(),
            complexity,
            keywords,
            elp_dominant,
        }
    }
    
    /// Extract key transformations from chain
    fn extract_transformations(&self, chain: &FluxReasoningChain) -> Vec<TransformationSnapshot> {
        let mut transformations = Vec::new();
        
        for i in 1..chain.thoughts.len() {
            let prev = &chain.thoughts[i - 1];
            let curr = &chain.thoughts[i];
            
            // Only record significant transformations
            let entropy_reduction = prev.entropy - curr.entropy;
            if entropy_reduction > 0.1 {
                transformations.push(TransformationSnapshot {
                    from_position: prev.vortex_position,
                    to_position: curr.vortex_position,
                    elp_delta: (
                        (curr.elp_state.ethos - prev.elp_state.ethos) as f32,
                        (curr.elp_state.logos - prev.elp_state.logos) as f32,
                        (curr.elp_state.pathos - prev.elp_state.pathos) as f32,
                    ),
                    entropy_reduction,
                    description: curr.reasoning_trace.clone(),
                });
            }
        }
        
        transformations
    }
    
    /// Compute signal strength (3-6-9 pattern coherence)
    fn compute_confidence(&self, chain: &FluxReasoningChain) -> f32 {
        // Signal strength based on sacred milestone coverage
        let sacred_coverage = chain.sacred_milestones.len() as f32 / 3.0;  // 3 = all trinity
        
        // Bonus for final confidence
        let confidence_bonus = chain.chain_confidence * 0.3;
        
        (sacred_coverage * 0.7 + confidence_bonus).clamp(0.0, 1.0)
    }
    
    /// Compute efficiency score (steps vs baseline)
    fn compute_efficiency(&self, domain: &str, actual_steps: usize) -> f32 {
        // Get or estimate baseline for domain
        let baseline = self.domain_baselines.get(domain)
            .map(|b| *b)
            .unwrap_or(10);  // Default baseline
        
        // Update baseline with exponential moving average
        let new_baseline = (baseline * 9 + actual_steps) / 10;
        self.domain_baselines.insert(domain.to_string(), new_baseline);
        
        // Efficiency = baseline / actual (>1.0 means better than average)
        baseline as f32 / actual_steps as f32
    }
}

impl Default for PatternExtractor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Pattern Storage Interface
// ============================================================================

/// Storage interface for reasoning patterns
#[async_trait::async_trait]
pub trait PatternStorage: Send + Sync {
    /// Store a new pattern
    async fn store(&self, pattern: ReasoningPattern) -> Result<()>;
    
    /// Find patterns similar to a query signature
    async fn find_similar(
        &self,
        signature: &QuerySignature,
        limit: usize,
    ) -> Result<Vec<ReasoningPattern>>;
    
    /// Update pattern metrics after reuse
    async fn update_metrics(
        &self,
        pattern_id: Uuid,
        success: bool,
        actual_steps: usize,
    ) -> Result<()>;
    
    /// Prune patterns with low success rates
    async fn prune_ineffective(&self, min_success_rate: f32) -> Result<usize>;
    
    /// Get learning metrics
    async fn get_metrics(&self) -> Result<LearningMetrics>;
}

// ============================================================================
// In-Memory Pattern Storage (for testing/development)
// ============================================================================

/// Simple in-memory pattern storage for development
pub struct InMemoryPatternStorage {
    patterns: Arc<DashMap<Uuid, ReasoningPattern>>,
}

impl InMemoryPatternStorage {
    pub fn new() -> Self {
        Self {
            patterns: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl PatternStorage for InMemoryPatternStorage {
    async fn store(&self, pattern: ReasoningPattern) -> Result<()> {
        self.patterns.insert(pattern.pattern_id, pattern);
        Ok(())
    }
    
    async fn find_similar(
        &self,
        signature: &QuerySignature,
        limit: usize,
    ) -> Result<Vec<ReasoningPattern>> {
        let mut matches: Vec<_> = self.patterns.iter()
            .filter(|entry| {
                let p = entry.value();
                // Filter by domain and success rate
                p.query_signature.domain == signature.domain 
                    && p.success_rate >= 0.5
            })
            .map(|entry| entry.value().clone())
            .collect();
        
        // Sort by success rate descending
        matches.sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap());
        
        // Return top-k
        matches.truncate(limit);
        Ok(matches)
    }
    
    async fn update_metrics(
        &self,
        pattern_id: Uuid,
        success: bool,
        actual_steps: usize,
    ) -> Result<()> {
        if let Some(mut pattern) = self.patterns.get_mut(&pattern_id) {
            // Update with exponential moving average
            let alpha = 0.1;  // Learning rate
            
            pattern.reuse_count += 1;
            pattern.updated_at = Utc::now();
            
            if success {
                pattern.success_rate = pattern.success_rate * (1.0 - alpha) + alpha;
            } else {
                pattern.success_rate = pattern.success_rate * (1.0 - alpha);
            }
            
            pattern.avg_steps = ((pattern.avg_steps as f32 * (1.0 - alpha)) 
                + (actual_steps as f32 * alpha)) as usize;
        }
        
        Ok(())
    }
    
    async fn prune_ineffective(&self, min_success_rate: f32) -> Result<usize> {
        let to_remove: Vec<Uuid> = self.patterns.iter()
            .filter(|entry| entry.value().success_rate < min_success_rate)
            .map(|entry| *entry.key())
            .collect();
        
        let count = to_remove.len();
        for id in to_remove {
            self.patterns.remove(&id);
        }
        
        Ok(count)
    }
    
    async fn get_metrics(&self) -> Result<LearningMetrics> {
        let total = self.patterns.len() as u64;
        
        if total == 0 {
            return Ok(LearningMetrics::default());
        }
        
        let sum_reuse: u32 = self.patterns.iter()
            .map(|e| e.value().reuse_count)
            .sum();
        let sum_success: f32 = self.patterns.iter()
            .map(|e| e.value().success_rate)
            .sum();
        
        Ok(LearningMetrics {
            patterns_extracted: total,
            patterns_active: total,
            patterns_pruned: 0,
            avg_reuse_count: sum_reuse as f32 / total as f32,
            avg_success_rate: sum_success / total as f32,
            acceleration_rate: 0.0,  // Tracked externally
            avg_speedup: 0.0,        // Tracked externally
        })
    }
}

impl Default for InMemoryPatternStorage {
    fn default() -> Self {
        Self::new()
    }
}
