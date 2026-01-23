//! Pattern Matching and Query Acceleration
//!
//! High-performance pattern matching with caching and query acceleration

use super::meta_learning::{
    ReasoningPattern, QuerySignature, AccelerationResult, PatternStorage,
};
use crate::ai::flux_reasoning::{FluxReasoningChain, OracleQuery};
use crate::ai::consensus::query_ollama;
use crate::data::models::ELPTensor;
use anyhow::Result;
use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;

// ============================================================================
// Pattern Matcher
// ============================================================================

/// Fast pattern matching with caching for <10ms retrieval
pub struct PatternMatcher {
    storage: Arc<dyn PatternStorage>,
    cache: Arc<DashMap<String, Vec<ReasoningPattern>>>,  // Lock-free cache
    cache_size_limit: usize,
}

impl PatternMatcher {
    pub fn new(storage: Arc<dyn PatternStorage>) -> Self {
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
            cache_size_limit: 1000,  // Cache up to 1000 query patterns
        }
    }
    
    /// Find best matching pattern for a query (<10ms target)
    pub async fn find_best_match(
        &self,
        query: &str,
        elp_state: &ELPTensor,
    ) -> Result<Option<ReasoningPattern>> {
        // Step 1: Create query signature
        let signature = self.create_signature(query, elp_state);
        
        // Step 2: Check cache first (<1ms)
        let cache_key = format!("{}:{}", signature.domain, signature.elp_dominant);
        if let Some(cached) = self.cache.get(&cache_key) {
            if let Some(best) = self.rank_patterns(&cached, &signature, elp_state).first() {
                tracing::debug!("🎯 Cache hit for pattern match");
                return Ok(Some(best.clone()));
            }
        }
        
        // Step 3: Retrieve from storage (5-10ms target)
        let candidates = self.storage.find_similar(&signature, 50).await?;
        
        if candidates.is_empty() {
            tracing::debug!("No patterns found for domain: {}", signature.domain);
            return Ok(None);
        }
        
        // Step 4: Rank by multiple criteria
        let ranked = self.rank_patterns(&candidates, &signature, elp_state);
        
        // Step 5: Cache top results (manage cache size)
        if !ranked.is_empty() {
            if self.cache.len() >= self.cache_size_limit {
                // Simple LRU: remove random entry
                if let Some(entry) = self.cache.iter().next() {
                    let key = entry.key().clone();
                    drop(entry);
                    self.cache.remove(&key);
                }
            }
            self.cache.insert(cache_key, ranked.clone());
        }
        
        Ok(ranked.first().cloned())
    }
    
    /// Create query signature from query text and ELP state
    fn create_signature(&self, query: &str, elp_state: &ELPTensor) -> QuerySignature {
        let query_lower = query.to_lowercase();
        
        // Domain detection
        let domain = if query_lower.contains("health") || query_lower.contains("diabetes") {
            "health"
        } else if query_lower.contains("math") || query_lower.contains("calculate") {
            "math"
        } else if query_lower.contains("ethics") || query_lower.contains("moral") {
            "ethics"
        } else if query_lower.contains("code") || query_lower.contains("program") {
            "technology"
        } else {
            "general"
        };
        
        // Extract keywords
        let keywords: Vec<String> = query.split_whitespace()
            .filter(|w| w.len() > 3)
            .take(10)  // Limit to 10 keywords
            .map(|w| w.to_lowercase())
            .collect();
        
        // Complexity estimation
        let complexity = if query.len() > 100 { 0.7 } else { 0.5 };
        
        // ELP dominance
        let elp_dominant = if elp_state.ethos > elp_state.logos && elp_state.ethos > elp_state.pathos {
            'E'
        } else if elp_state.logos > elp_state.pathos {
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
    
    /// Rank patterns by relevance to query
    fn rank_patterns(
        &self,
        patterns: &[ReasoningPattern],
        signature: &QuerySignature,
        elp_state: &ELPTensor,
    ) -> Vec<ReasoningPattern> {
        let mut scored: Vec<_> = patterns.iter()
            .map(|p| {
                let score = self.compute_match_score(p, signature, elp_state);
                (p.clone(), score)
            })
            .collect();
        
        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        scored.into_iter().map(|(p, _)| p).collect()
    }
    
    /// Compute match score between pattern and query
    fn compute_match_score(
        &self,
        pattern: &ReasoningPattern,
        signature: &QuerySignature,
        elp_state: &ELPTensor,
    ) -> f32 {
        // Domain match (40% weight)
        let domain_score = if pattern.query_signature.domain == signature.domain {
            0.4
        } else {
            0.0
        };
        
        // Success rate (30% weight)
        let success_score = pattern.success_rate * 0.3;
        
        // ELP compatibility (20% weight)
        let elp_score = self.elp_similarity(&pattern.elp_profile, elp_state) * 0.2;
        
        // Efficiency (10% weight)
        let efficiency_score = pattern.efficiency_score * 0.1;
        
        domain_score + success_score + elp_score + efficiency_score
    }
    
    /// Compute similarity between two ELP states (cosine similarity)
    fn elp_similarity(&self, p1: &ELPTensor, p2: &ELPTensor) -> f32 {
        // Cosine similarity in 3D ELP space
        let dot = p1.ethos * p2.ethos + p1.logos * p2.logos + p1.pathos * p2.pathos;
        let mag1 = (p1.ethos.powi(2) + p1.logos.powi(2) + p1.pathos.powi(2)).sqrt();
        let mag2 = (p2.ethos.powi(2) + p2.logos.powi(2) + p2.pathos.powi(2)).sqrt();
        
        if mag1 == 0.0 || mag2 == 0.0 {
            return 0.0;
        }
        
        ((dot / (mag1 * mag2)) as f32).clamp(0.0, 1.0)
    }
    
    /// Clear cache (for testing or memory management)
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

// ============================================================================
// Query Accelerator
// ============================================================================

/// Accelerates queries by applying matched patterns
pub struct QueryAccelerator {
    matcher: Arc<PatternMatcher>,
    confidence_threshold: f32,  // Minimum confidence to use pattern
}

impl QueryAccelerator {
    pub fn new(matcher: Arc<PatternMatcher>) -> Self {
        Self {
            matcher,
            confidence_threshold: 0.8,  // High confidence required
        }
    }
    
    /// Try to accelerate reasoning using a matched pattern
    pub async fn try_accelerate(
        &self,
        chain: &FluxReasoningChain,
    ) -> Result<Option<AccelerationResult>> {
        let query = &chain.query;
        let elp_state = &chain.current_thought().elp_state;
        
        // Find best matching pattern
        let pattern = match self.matcher.find_best_match(query, elp_state).await? {
            Some(p) if p.success_rate >= self.confidence_threshold => p,
            Some(p) => {
                tracing::debug!("Pattern found but success rate too low: {:.2}", p.success_rate);
                return Ok(None);
            },
            None => {
                tracing::debug!("No matching pattern found");
                return Ok(None);
            },
        };
        
        tracing::info!("🚀 Pattern match: {} (success: {:.1}%, reuse: {})", 
            pattern.pattern_id, pattern.success_rate * 100.0, pattern.reuse_count);
        
        // Clone chain for acceleration
        let mut accelerated = chain.clone();
        let initial_steps = accelerated.thoughts.len();
        
        // Apply pattern's oracle questions (skip regular reasoning)
        for (idx, question) in pattern.oracle_questions.iter().enumerate() {
            tracing::debug!("  ↳ Applying oracle question {}/{}: {}", 
                idx + 1, pattern.oracle_questions.len(), question);
            
            // Query oracle with pattern's proven questions
            let response = query_ollama(question, None).await?;
            
            // Integrate response
            let flux_update = accelerated.integrate_oracle_response(&response.response_text);
            let entropy_reduction = flux_update.entropy_reduction;
            
            accelerated.apply_flux_update(flux_update, Some(OracleQuery {
                model: response.model_name.clone(),
                question: question.clone(),
                response: response.response_text.clone(),
                entropy_reduction,
                timestamp: Utc::now(),
            }));
            
            // Check if converged early
            if accelerated.has_converged() {
                tracing::info!("✨ Early convergence via pattern!");
                break;
            }
        }
        
        let steps_saved = initial_steps.saturating_sub(accelerated.thoughts.len());
        let confidence_boost = accelerated.chain_confidence - chain.chain_confidence;
        
        // Update pattern metrics (will be confirmed after full reasoning)
        self.matcher.storage.update_metrics(
            pattern.pattern_id,
            true,  // Optimistic - will update later with actual result
            accelerated.thoughts.len(),
        ).await?;
        
        Ok(Some(AccelerationResult {
            pattern_id: pattern.pattern_id,
            steps_saved,
            confidence_boost,
            accelerated_chain: accelerated,
        }))
    }
    
    /// Set minimum confidence threshold
    pub fn set_confidence_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
    }
}

// ============================================================================
// Feedback Collector
// ============================================================================

/// Collects feedback on pattern effectiveness for continuous improvement
pub struct FeedbackCollector {
    storage: Arc<dyn PatternStorage>,
}

impl FeedbackCollector {
    pub fn new(storage: Arc<dyn PatternStorage>) -> Self {
        Self { storage }
    }
    
    /// Record outcome of pattern application
    pub async fn record_outcome(
        &self,
        pattern_id: uuid::Uuid,
        actual_success: bool,
        actual_steps: usize,
        final_confidence: f32,
    ) -> Result<()> {
        // Update pattern with actual performance
        self.storage.update_metrics(pattern_id, actual_success, actual_steps).await?;
        
        if !actual_success {
            tracing::warn!("⚠️ Pattern {} failed (final confidence: {:.2})", 
                pattern_id, final_confidence);
        } else {
            tracing::debug!("✅ Pattern {} succeeded (confidence: {:.2}, steps: {})", 
                pattern_id, final_confidence, actual_steps);
        }
        
        Ok(())
    }
    
    /// Evolve patterns by pruning ineffective ones
    pub async fn evolve_patterns(&self, min_success_rate: f32) -> Result<usize> {
        let pruned = self.storage.prune_ineffective(min_success_rate).await?;
        
        if pruned > 0 {
            tracing::info!("🧹 Pruned {} ineffective patterns (success < {:.1}%)", 
                pruned, min_success_rate * 100.0);
        }
        
        Ok(pruned)
    }
    
    /// Get current learning metrics
    pub async fn get_metrics(&self) -> Result<super::meta_learning::LearningMetrics> {
        self.storage.get_metrics().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::meta_learning::InMemoryPatternStorage;
    
    #[tokio::test]
    async fn test_pattern_matcher_creation() {
        let storage = Arc::new(InMemoryPatternStorage::new());
        let matcher = PatternMatcher::new(storage);
        
        let elp = ELPTensor {
            ethos: 5.0,
            logos: 7.0,
            pathos: 4.0,
        };
        
        let result = matcher.find_best_match("test query", &elp).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());  // No patterns stored yet
    }
    
    #[test]
    fn test_elp_similarity() {
        let storage = Arc::new(InMemoryPatternStorage::new());
        let matcher = PatternMatcher::new(storage);
        
        let elp1 = ELPTensor { ethos: 5.0, logos: 7.0, pathos: 4.0 };
        let elp2 = ELPTensor { ethos: 5.0, logos: 7.0, pathos: 4.0 };
        let elp3 = ELPTensor { ethos: 1.0, logos: 1.0, pathos: 1.0 };
        
        // Identical ELP should have similarity ~1.0
        let sim1 = matcher.elp_similarity(&elp1, &elp2);
        assert!(sim1 > 0.99);
        
        // Different ELP should have lower similarity
        let sim2 = matcher.elp_similarity(&elp1, &elp3);
        assert!(sim2 < 1.0);
    }
}
