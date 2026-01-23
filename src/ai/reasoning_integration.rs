//! Reasoning Integration - Unified AGI Reasoning Pipeline
//!
//! Connects all AGI subsystems with feedback loops:
//! - Flux Reasoning ↔ Working Memory
//! - Goal Planning ↔ Causal Reasoning
//! - Meta-Learning ↔ Transfer Learning
//! - Curiosity ↔ All Systems
//!
//! ## Architecture
//!
//! ```text
//!                    ┌─────────────────┐
//!                    │  User Query     │
//!                    └────────┬────────┘
//!                             ↓
//!              ┌──────────────────────────────┐
//!              │     Working Memory           │
//!              │  (Context & Short-term)      │
//!              └──────────────┬───────────────┘
//!                             ↓
//!     ┌───────────────────────┼───────────────────────┐
//!     ↓                       ↓                       ↓
//! ┌─────────┐          ┌─────────────┐         ┌──────────┐
//! │  Flux   │←────────→│   Causal    │←───────→│   Goal   │
//! │Reasoning│          │  Reasoning  │         │ Planning │
//! └────┬────┘          └──────┬──────┘         └────┬─────┘
//!      │                      │                     │
//!      └──────────────────────┼─────────────────────┘
//!                             ↓
//!              ┌──────────────────────────────┐
//!              │     Meta-Learning            │
//!              │  (Pattern Extraction)        │
//!              └──────────────┬───────────────┘
//!                             ↓
//!              ┌──────────────────────────────┐
//!              │     Transfer Learning        │
//!              │  (Cross-Domain Application)  │
//!              └──────────────┬───────────────┘
//!                             ↓
//!              ┌──────────────────────────────┐
//!              │     Curiosity Engine         │
//!              │  (Gap Detection & Explore)   │
//!              └──────────────────────────────┘
//! ```

use crate::ai::causal_reasoning::{CausalWorldModel, CausalValue};
use crate::ai::curiosity_engine::{CuriosityEngine, KnowledgeGap};
use crate::ai::flux_reasoning::{FluxReasoningChain, FluxThought, EntropyType};
use crate::ai::goal_planner::{GoalPlanner, Goal, Plan};
use crate::ai::meta_learning::{PatternExtractor, ReasoningPattern, InMemoryPatternStorage, PatternStorage};
use crate::ai::meta_learning_matcher::{PatternMatcher, QueryAccelerator};
use crate::ai::self_improvement::{MetaLearner, PerformanceMetrics};
use crate::ai::transfer_learning::{TransferLearningEngine, TransferResult};
use crate::ai::working_memory::{WorkingMemory, ContextWindow, MemoryContent, MemorySource};
use crate::data::models::ELPTensor;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Integrated Reasoning System
// ============================================================================

/// The unified reasoning integration system
pub struct ReasoningIntegration {
    /// Working memory for context
    pub working_memory: ContextWindow,
    
    /// Flux reasoning engine
    pub flux_engine: FluxReasoningEngine,
    
    /// Goal planning system
    pub goal_planner: GoalPlanner,
    
    /// Causal world model
    pub causal_model: CausalWorldModel,
    
    /// Meta-learning system
    pub meta_learner: MetaLearner,
    
    /// Pattern extraction
    pub pattern_extractor: PatternExtractor,
    
    /// Pattern matching for acceleration
    pub pattern_matcher: Arc<PatternMatcher>,
    
    /// Query accelerator
    pub accelerator: QueryAccelerator,
    
    /// Transfer learning engine
    pub transfer_engine: TransferLearningEngine,
    
    /// Curiosity engine
    pub curiosity: CuriosityEngine,
    
    /// Integration state
    pub state: IntegrationState,
    
    /// Statistics
    pub stats: IntegrationStats,
}

/// Wrapper for flux reasoning with memory integration
pub struct FluxReasoningEngine {
    pub current_chain: Option<FluxReasoningChain>,
    pub completed_chains: Vec<FluxReasoningChain>,
}

impl Default for FluxReasoningEngine {
    fn default() -> Self {
        Self {
            current_chain: None,
            completed_chains: Vec::new(),
        }
    }
}

/// Current state of the integration system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationState {
    pub id: Uuid,
    pub mode: ReasoningMode,
    pub current_query: Option<String>,
    pub active_goal: Option<Uuid>,
    pub elp_state: ELPTensor,
    pub confidence: f32,
    pub depth: usize,
    pub last_activity: chrono::DateTime<Utc>,
}

/// Current reasoning mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningMode {
    Idle,
    Analyzing,
    FluxReasoning,
    GoalPlanning,
    CausalInference,
    MetaLearning,
    TransferLearning,
    Exploring,
    Synthesizing,
}

/// Statistics for the integration system
#[derive(Debug, Clone, Default)]
pub struct IntegrationStats {
    pub queries_processed: u64,
    pub reasoning_chains_completed: u64,
    pub goals_achieved: u64,
    pub causal_insights: u64,
    pub patterns_learned: u64,
    pub transfers_successful: u64,
    pub knowledge_gaps_filled: u64,
    pub avg_confidence: f32,
    pub avg_reasoning_steps: f32,
}

/// Result of integrated reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedResponse {
    pub answer: String,
    pub confidence: f32,
    pub reasoning_steps: usize,
    pub mode_sequence: Vec<ReasoningMode>,
    pub causal_insights: Vec<String>,
    pub goals_created: Vec<Uuid>,
    pub patterns_matched: usize,
    pub knowledge_gaps: Vec<String>,
    pub transfer_applied: bool,
    pub processing_time_ms: u64,
    pub memory_items_used: usize,
}

impl Default for ReasoningIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl ReasoningIntegration {
    pub fn new() -> Self {
        let storage: Arc<dyn PatternStorage> = Arc::new(InMemoryPatternStorage::new());
        let pattern_matcher = Arc::new(PatternMatcher::new(storage.clone()));
        let accelerator = QueryAccelerator::new(pattern_matcher.clone());
        
        Self {
            working_memory: ContextWindow::new(9), // Sacred 9 slots
            flux_engine: FluxReasoningEngine::default(),
            goal_planner: GoalPlanner::new(),
            causal_model: CausalWorldModel::new(),
            meta_learner: MetaLearner::new(),
            pattern_extractor: PatternExtractor::new(),
            pattern_matcher,
            accelerator,
            transfer_engine: TransferLearningEngine::new(),
            curiosity: CuriosityEngine::new(),
            state: IntegrationState {
                id: Uuid::new_v4(),
                mode: ReasoningMode::Idle,
                current_query: None,
                active_goal: None,
                elp_state: ELPTensor { ethos: 5.0, logos: 5.0, pathos: 5.0 },
                confidence: 0.5,
                depth: 0,
                last_activity: Utc::now(),
            },
            stats: IntegrationStats::default(),
        }
    }
    
    /// Main entry point for integrated reasoning
    pub async fn reason(&mut self, query: &str) -> Result<IntegratedResponse> {
        let start = std::time::Instant::now();
        let mut mode_sequence = Vec::new();
        
        // Phase 1: Initialize context
        self.state.mode = ReasoningMode::Analyzing;
        self.state.current_query = Some(query.to_string());
        self.state.last_activity = Utc::now();
        mode_sequence.push(ReasoningMode::Analyzing);
        
        // Analyze query and set initial ELP
        let initial_elp = self.analyze_query(query);
        self.state.elp_state = initial_elp.clone();
        
        // Push context frame
        self.working_memory.push_context(query, &initial_elp);
        
        // Store query in working memory
        self.working_memory.store_in_context(
            MemoryContent::Text(query.to_string()),
            &initial_elp,
            MemorySource::UserInput,
        );
        
        // Phase 2: Check for pattern acceleration
        let mut chain = FluxReasoningChain::new(query);
        let patterns_matched = if let Ok(Some(accel)) = self.accelerator.try_accelerate(&chain).await {
            chain = accel.accelerated_chain;
            tracing::info!("Pattern acceleration: saved {} steps", accel.steps_saved);
            1
        } else {
            0
        };
        
        // Phase 3: Flux Reasoning
        self.state.mode = ReasoningMode::FluxReasoning;
        mode_sequence.push(ReasoningMode::FluxReasoning);
        
        let final_thought = chain.reason(20).await?;
        self.state.depth = chain.thoughts.len();
        
        // Store reasoning steps in working memory
        for (i, thought) in chain.thoughts.iter().enumerate() {
            self.working_memory.store_in_context(
                MemoryContent::ReasoningStep {
                    step_number: i,
                    vortex_position: thought.vortex_position,
                    entropy: thought.entropy,
                    insight: thought.reasoning_trace.clone(),
                },
                &thought.elp_state,
                MemorySource::InternalReasoning,
            );
        }
        
        // Phase 4: Causal Inference
        self.state.mode = ReasoningMode::CausalInference;
        mode_sequence.push(ReasoningMode::CausalInference);
        
        let causal_insights = self.extract_causal_insights(&chain);
        self.stats.causal_insights += causal_insights.len() as u64;
        
        // Store causal insights in working memory
        for insight in &causal_insights {
            self.working_memory.store_in_context(
                MemoryContent::Text(insight.clone()),
                &final_thought.elp_state,
                MemorySource::CausalInference,
            );
        }
        
        // Phase 5: Goal Creation (if needed)
        let mut goals_created = Vec::new();
        if self.should_create_goal(&final_thought) {
            self.state.mode = ReasoningMode::GoalPlanning;
            mode_sequence.push(ReasoningMode::GoalPlanning);
            
            let goal = self.goal_planner.create_goal(query, &final_thought.elp_state);
            goals_created.push(goal.id);
            self.goal_planner.add_goal(goal);
        }
        
        // Phase 6: Curiosity Check
        let mut knowledge_gaps = Vec::new();
        if self.curiosity.should_explore() {
            self.state.mode = ReasoningMode::Exploring;
            mode_sequence.push(ReasoningMode::Exploring);
            
            if let Some(gap) = self.curiosity.get_most_curious() {
                knowledge_gaps.push(gap.description.clone());
            }
        }
        
        // Phase 7: Transfer Learning Check
        let transfer_applied = self.check_transfer_opportunity(&chain);
        if transfer_applied {
            self.state.mode = ReasoningMode::TransferLearning;
            mode_sequence.push(ReasoningMode::TransferLearning);
            self.stats.transfers_successful += 1;
        }
        
        // Phase 8: Meta-Learning
        self.state.mode = ReasoningMode::MetaLearning;
        mode_sequence.push(ReasoningMode::MetaLearning);
        
        if let Some(_pattern) = self.pattern_extractor.extract(&chain) {
            self.stats.patterns_learned += 1;
        }
        
        // Phase 9: Synthesize Answer
        self.state.mode = ReasoningMode::Synthesizing;
        mode_sequence.push(ReasoningMode::Synthesizing);
        
        let answer = chain.synthesize_final_answer().await?;
        
        // Phase 10: Self-Improvement Check
        if self.stats.queries_processed % 10 == 0 {
            if let Ok(config) = self.meta_learner.propose_improvement() {
                if let Ok(exp) = self.meta_learner.run_experiment("Periodic improvement", config) {
                    if exp.improvement.unwrap_or(0.0) > 0.0 {
                        tracing::info!("Self-improvement: {:.2}% gain", exp.improvement.unwrap_or(0.0) * 100.0);
                    }
                }
            }
        }
        
        // Update state
        self.state.confidence = final_thought.certainty;
        self.state.mode = ReasoningMode::Idle;
        self.stats.queries_processed += 1;
        self.stats.reasoning_chains_completed += 1;
        
        // Update running averages
        let n = self.stats.queries_processed as f32;
        self.stats.avg_confidence = (self.stats.avg_confidence * (n - 1.0) + final_thought.certainty) / n;
        self.stats.avg_reasoning_steps = (self.stats.avg_reasoning_steps * (n - 1.0) + chain.thoughts.len() as f32) / n;
        
        // Store completed chain
        self.flux_engine.completed_chains.push(chain.clone());
        
        // Get memory usage
        let memory_summary = self.working_memory.memory.get_summary();
        
        // Pop context frame
        self.working_memory.pop_context();
        
        Ok(IntegratedResponse {
            answer,
            confidence: final_thought.certainty,
            reasoning_steps: chain.thoughts.len(),
            mode_sequence,
            causal_insights,
            goals_created,
            patterns_matched,
            knowledge_gaps,
            transfer_applied,
            processing_time_ms: start.elapsed().as_millis() as u64,
            memory_items_used: memory_summary.total_items,
        })
    }
    
    /// Pursue a goal with integrated reasoning
    pub async fn pursue_goal(&mut self, objective: &str) -> Result<IntegratedResponse> {
        let start = std::time::Instant::now();
        let mut mode_sequence = vec![ReasoningMode::GoalPlanning];
        
        self.state.mode = ReasoningMode::GoalPlanning;
        
        // Create and plan goal
        let elp = self.analyze_query(objective);
        let goal = self.goal_planner.create_goal(objective, &elp);
        let goal_id = goal.id;
        self.goal_planner.add_goal(goal);
        self.state.active_goal = Some(goal_id);
        
        // Store goal in working memory
        self.working_memory.push_context(objective, &elp);
        self.working_memory.store_in_context(
            MemoryContent::GoalContext {
                goal_id,
                objective: objective.to_string(),
                progress: 0.0,
            },
            &elp,
            MemorySource::GoalPlanning,
        );
        
        // Plan
        let plan = self.goal_planner.plan_next_goal()?;
        
        if let Some(plan) = plan {
            // Execute with causal monitoring
            self.state.mode = ReasoningMode::CausalInference;
            mode_sequence.push(ReasoningMode::CausalInference);
            
            let success = self.goal_planner.execute_plan().await?;
            
            if success {
                self.goal_planner.mark_goal_achieved(goal_id);
                self.stats.goals_achieved += 1;
            }
            
            self.state.mode = ReasoningMode::Idle;
            self.state.active_goal = None;
            self.working_memory.pop_context();
            
            Ok(IntegratedResponse {
                answer: format!("Goal '{}' {}", objective, if success { "achieved" } else { "failed" }),
                confidence: if success { 0.9 } else { 0.3 },
                reasoning_steps: plan.tasks.len(),
                mode_sequence,
                causal_insights: vec![],
                goals_created: vec![goal_id],
                patterns_matched: 0,
                knowledge_gaps: vec![],
                transfer_applied: false,
                processing_time_ms: start.elapsed().as_millis() as u64,
                memory_items_used: self.working_memory.memory.get_summary().total_items,
            })
        } else {
            self.working_memory.pop_context();
            
            Ok(IntegratedResponse {
                answer: "No plan could be created".to_string(),
                confidence: 0.1,
                reasoning_steps: 0,
                mode_sequence,
                causal_insights: vec![],
                goals_created: vec![goal_id],
                patterns_matched: 0,
                knowledge_gaps: vec![],
                transfer_applied: false,
                processing_time_ms: start.elapsed().as_millis() as u64,
                memory_items_used: 0,
            })
        }
    }
    
    /// Ask a counterfactual question with causal reasoning
    pub fn ask_counterfactual(
        &mut self,
        description: &str,
        target: &str,
        value: f64,
        query: &str,
    ) -> Result<String> {
        let cf = self.causal_model.ask_counterfactual(
            description,
            target,
            CausalValue::Numeric(value),
            query,
        )?;
        
        // Store in working memory
        let elp = ELPTensor { ethos: 5.0, logos: 8.0, pathos: 4.0 };
        self.working_memory.memory.store(
            MemoryContent::CausalLink {
                cause: target.to_string(),
                effect: query.to_string(),
                strength: cf.confidence,
            },
            &elp,
            MemorySource::CausalInference,
        );
        
        let answer = match cf.counterfactual_value {
            Some(CausalValue::Numeric(v)) => format!(
                "If {} were {}, then {} would be approximately {:.2} (confidence: {:.0}%)",
                target, value, query, v, cf.confidence * 100.0
            ),
            Some(CausalValue::Boolean(b)) => format!(
                "If {} were {}, then {} would be {} (confidence: {:.0}%)",
                target, value, query, b, cf.confidence * 100.0
            ),
            _ => format!(
                "Could not determine counterfactual outcome (confidence: {:.0}%)",
                cf.confidence * 100.0
            ),
        };
        
        Ok(answer)
    }
    
    /// Explore a topic driven by curiosity
    pub fn explore(&mut self, topic: &str) -> Vec<String> {
        let elp = ELPTensor { ethos: 5.0, logos: 7.0, pathos: 4.0 };
        self.curiosity.identify_gap(topic, topic, 0.8, &elp);
        
        let mut explorations = Vec::new();
        
        if let Some(gap) = self.curiosity.get_most_curious() {
            let action = self.curiosity.generate_exploration(&gap);
            explorations.push(format!("Exploring: {:?}", action.action_type));
            
            // Store exploration in working memory
            self.working_memory.memory.store(
                MemoryContent::Text(format!("Exploring: {}", gap.description)),
                &elp,
                MemorySource::InternalReasoning,
            );
        }
        
        explorations
    }
    
    /// Get current state
    pub fn get_state(&self) -> &IntegrationState {
        &self.state
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> &IntegrationStats {
        &self.stats
    }
    
    /// Get working memory summary
    pub fn get_memory_summary(&self) -> crate::ai::working_memory::MemorySummary {
        self.working_memory.memory.get_summary()
    }
    
    /// Apply time-based memory decay
    pub fn tick(&mut self, elapsed_seconds: f32) {
        self.working_memory.memory.apply_decay(elapsed_seconds);
    }
    
    // ========================================================================
    // Private helpers
    // ========================================================================
    
    fn analyze_query(&self, query: &str) -> ELPTensor {
        let lower = query.to_lowercase();
        
        let ethos = if lower.contains("should") || lower.contains("moral") || lower.contains("ethical") {
            8.0
        } else if lower.contains("right") || lower.contains("wrong") {
            7.0
        } else {
            5.0
        };
        
        let logos = if lower.contains("why") || lower.contains("how") || lower.contains("explain") {
            8.0
        } else if lower.contains("what") || lower.contains("calculate") {
            7.0
        } else {
            5.0
        };
        
        let pathos = if lower.contains("feel") || lower.contains("emotion") || lower.contains('!') {
            7.0
        } else {
            4.0
        };
        
        ELPTensor { ethos, logos, pathos }
    }
    
    fn extract_causal_insights(&mut self, chain: &FluxReasoningChain) -> Vec<String> {
        let mut insights = Vec::new();
        
        for thought in &chain.thoughts {
            for oracle in &thought.oracle_contributions {
                let response = oracle.response.to_lowercase();
                if response.contains("because") || response.contains("causes") || response.contains("leads to") {
                    let insight = format!(
                        "Causal insight from {}: {}",
                        oracle.model,
                        oracle.response.chars().take(100).collect::<String>()
                    );
                    insights.push(insight);
                    
                    // Learn causal relation
                    self.causal_model.learn_from_observation(
                        &oracle.question,
                        &oracle.response.chars().take(50).collect::<String>(),
                        oracle.entropy_reduction,
                        &thought.elp_state,
                    );
                }
            }
        }
        
        insights
    }
    
    fn should_create_goal(&self, thought: &FluxThought) -> bool {
        let elp_magnitude = thought.elp_state.ethos + thought.elp_state.logos + thought.elp_state.pathos;
        elp_magnitude > 18.0 && thought.certainty < 0.8
    }
    
    fn check_transfer_opportunity(&mut self, chain: &FluxReasoningChain) -> bool {
        // Check if we can apply transfer learning from a known domain
        let query_lower = chain.query.to_lowercase();
        
        // Simple domain detection
        let detected_domain = if query_lower.contains("health") || query_lower.contains("medical") {
            Some("health")
        } else if query_lower.contains("math") || query_lower.contains("calculate") {
            Some("mathematics")
        } else if query_lower.contains("code") || query_lower.contains("program") {
            Some("programming")
        } else {
            None
        };
        
        if let Some(domain) = detected_domain {
            // Check if we have applicable principles
            for principle in self.transfer_engine.principles.values() {
                if principle.name.to_lowercase().contains(domain) {
                    return true;
                }
            }
        }
        
        false
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_integration_creation() {
        let integration = ReasoningIntegration::new();
        assert_eq!(integration.state.mode, ReasoningMode::Idle);
    }
    
    #[test]
    fn test_query_analysis() {
        let integration = ReasoningIntegration::new();
        
        let elp = integration.analyze_query("Why should we be ethical?");
        assert!(elp.ethos > 5.0); // Ethical question
        assert!(elp.logos > 5.0); // "Why" question
    }
    
    #[test]
    fn test_working_memory_integration() {
        let mut integration = ReasoningIntegration::new();
        let elp = ELPTensor { ethos: 6.0, logos: 7.0, pathos: 5.0 };
        
        integration.working_memory.push_context("Test context", &elp);
        let id = integration.working_memory.store_in_context(
            MemoryContent::Text("Test memory".to_string()),
            &elp,
            MemorySource::UserInput,
        );
        
        let memories = integration.working_memory.get_context_memories();
        assert_eq!(memories.len(), 1);
    }
    
    #[test]
    fn test_exploration() {
        let mut integration = ReasoningIntegration::new();
        let explorations = integration.explore("quantum computing");
        assert!(!explorations.is_empty());
    }
}
