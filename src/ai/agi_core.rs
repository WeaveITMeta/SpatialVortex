//! AGI Core - Unified Artificial General Intelligence System
//!
//! Integrates all AGI components into a cohesive system:
//! - Flux Reasoning (geometric thought substrate)
//! - Goal Planning (HTN planning, goal arbitration)
//! - Causal Reasoning (causal graphs, counterfactuals)
//! - Self-Improvement (meta-learning, architecture search)
//! - Curiosity Engine (exploration, hypothesis testing)
//! - Meta-Learning (pattern extraction, query acceleration)

use crate::ai::flux_reasoning::{FluxReasoningChain, FluxThought};
use crate::ai::goal_planner::GoalPlanner;
use crate::ai::causal_reasoning::{CausalWorldModel, CausalValue};
use crate::ai::self_improvement::{MetaLearner, PerformanceMetrics};
use crate::ai::curiosity_engine::CuriosityEngine;
use crate::ai::meta_learning::{PatternExtractor, InMemoryPatternStorage, PatternStorage};
use crate::ai::meta_learning_matcher::{PatternMatcher, QueryAccelerator};
use crate::data::models::ELPTensor;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use uuid::Uuid;

/// The unified AGI system
pub struct AGICore {
    /// Flux reasoning engine (geometric thought)
    pub flux_engine: FluxEngine,
    
    /// Goal planning system
    pub goal_planner: GoalPlanner,
    
    /// Causal world model
    pub causal_model: CausalWorldModel,
    
    /// Self-improvement system
    pub meta_learner: MetaLearner,
    
    /// Curiosity-driven exploration
    pub curiosity: CuriosityEngine,
    
    /// Pattern matching for acceleration
    pub pattern_matcher: Arc<PatternMatcher>,
    
    /// Query accelerator
    pub accelerator: QueryAccelerator,
    
    /// Pattern extractor
    pub pattern_extractor: PatternExtractor,
    
    /// AGI state
    pub state: AGIState,
    
    /// Statistics
    pub stats: AGIStats,
}

/// Wrapper for flux reasoning with AGI integration
pub struct FluxEngine {
    pub current_chain: Option<FluxReasoningChain>,
    pub completed_chains: Vec<FluxReasoningChain>,
}

impl Default for FluxEngine {
    fn default() -> Self {
        Self { current_chain: None, completed_chains: Vec::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AGIState {
    pub id: Uuid,
    pub mode: AGIMode,
    pub current_goal: Option<Uuid>,
    pub active_hypotheses: Vec<Uuid>,
    pub elp_state: ELPTensor,
    pub confidence: f32,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AGIMode {
    /// Idle, waiting for input
    Idle,
    /// Reasoning about a query
    Reasoning,
    /// Planning to achieve a goal
    Planning,
    /// Executing a plan
    Executing,
    /// Exploring out of curiosity
    Exploring,
    /// Self-improving
    Improving,
    /// Learning from experience
    Learning,
}

#[derive(Debug, Clone, Default)]
pub struct AGIStats {
    pub queries_processed: u64,
    pub goals_achieved: u64,
    pub patterns_learned: u64,
    pub causal_relations_discovered: u64,
    pub hypotheses_confirmed: u64,
    pub self_improvements: u64,
    pub total_reasoning_steps: u64,
    pub avg_confidence: f32,
}

/// Result of AGI processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AGIResponse {
    pub answer: String,
    pub confidence: f32,
    pub reasoning_steps: usize,
    pub mode_used: AGIMode,
    pub goals_created: Vec<Uuid>,
    pub patterns_matched: usize,
    pub causal_insights: Vec<String>,
    pub knowledge_gaps: Vec<String>,
    pub processing_time_ms: u64,
}

impl Default for AGICore {
    fn default() -> Self { Self::new() }
}

impl AGICore {
    pub fn new() -> Self {
        let storage: Arc<dyn PatternStorage> = Arc::new(InMemoryPatternStorage::new());
        let pattern_matcher = Arc::new(PatternMatcher::new(storage.clone()));
        let accelerator = QueryAccelerator::new(pattern_matcher.clone());
        
        Self {
            flux_engine: FluxEngine::default(),
            goal_planner: GoalPlanner::new(),
            causal_model: CausalWorldModel::new(),
            meta_learner: MetaLearner::new(),
            curiosity: CuriosityEngine::new(),
            pattern_matcher,
            accelerator,
            pattern_extractor: PatternExtractor::new(),
            state: AGIState {
                id: Uuid::new_v4(),
                mode: AGIMode::Idle,
                current_goal: None,
                active_hypotheses: Vec::new(),
                elp_state: ELPTensor { ethos: 5.0, logos: 5.0, pathos: 5.0 },
                confidence: 0.5,
                last_activity: Utc::now(),
            },
            stats: AGIStats::default(),
        }
    }
    
    /// Main AGI processing entry point
    pub async fn process(&mut self, query: &str) -> Result<AGIResponse> {
        let start = std::time::Instant::now();
        self.state.mode = AGIMode::Reasoning;
        self.state.last_activity = Utc::now();
        
        tracing::info!("AGI processing: {}", query);
        
        // Step 1: Analyze query and create initial ELP state
        let initial_elp = self.analyze_query(query);
        self.state.elp_state = initial_elp.clone();
        
        // Step 2: Check for pattern match (acceleration)
        let mut chain = FluxReasoningChain::new(query);
        let patterns_matched = if let Ok(Some(accel)) = self.accelerator.try_accelerate(&chain).await {
            chain = accel.accelerated_chain;
            tracing::info!("Pattern acceleration: saved {} steps", accel.steps_saved);
            1
        } else {
            0
        };
        
        // Step 3: Check curiosity - should we explore?
        let mut knowledge_gaps = Vec::new();
        if self.curiosity.should_explore() {
            if let Some(gap) = self.curiosity.get_most_curious() {
                knowledge_gaps.push(gap.description.clone());
                self.state.mode = AGIMode::Exploring;
            }
        }
        
        // Step 4: Main reasoning loop
        self.state.mode = AGIMode::Reasoning;
        let final_thought = chain.reason(20).await?;
        self.stats.total_reasoning_steps += chain.thoughts.len() as u64;
        
        // Step 5: Extract causal insights
        let causal_insights = self.extract_causal_insights(&chain);
        
        // Step 6: Create goal if needed
        let mut goals_created = Vec::new();
        if self.should_create_goal(&final_thought) {
            let goal = self.goal_planner.create_goal(query, &final_thought.elp_state);
            goals_created.push(goal.id);
            self.goal_planner.add_goal(goal);
        }
        
        // Step 7: Synthesize answer
        let answer = chain.synthesize_final_answer().await?;
        
        // Step 8: Learn from this interaction
        self.state.mode = AGIMode::Learning;
        if self.pattern_extractor.extract(&chain).is_some() {
            self.stats.patterns_learned += 1;
        }
        
        // Step 9: Check for self-improvement opportunity
        if self.stats.queries_processed % 10 == 0 {
            self.state.mode = AGIMode::Improving;
            if let Ok(config) = self.meta_learner.propose_improvement() {
                if let Ok(exp) = self.meta_learner.run_experiment("Periodic improvement", config) {
                    if exp.improvement.unwrap_or(0.0) > 0.0 {
                        self.stats.self_improvements += 1;
                    }
                }
            }
        }
        
        // Update stats
        self.stats.queries_processed += 1;
        self.state.confidence = final_thought.certainty;
        self.state.mode = AGIMode::Idle;
        
        // Store completed chain
        self.flux_engine.completed_chains.push(chain.clone());
        
        Ok(AGIResponse {
            answer,
            confidence: final_thought.certainty,
            reasoning_steps: chain.thoughts.len(),
            mode_used: AGIMode::Reasoning,
            goals_created,
            patterns_matched,
            causal_insights,
            knowledge_gaps,
            processing_time_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    /// Analyze query to determine initial ELP state
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
    
    /// Extract causal insights from reasoning chain
    fn extract_causal_insights(&mut self, chain: &FluxReasoningChain) -> Vec<String> {
        let mut insights = Vec::new();
        
        for thought in &chain.thoughts {
            for oracle in &thought.oracle_contributions {
                // Look for causal language
                let response = oracle.response.to_lowercase();
                if response.contains("because") || response.contains("causes") || response.contains("leads to") {
                    // Extract and store causal relation
                    let insight = format!("Causal insight from {}: {}", oracle.model, 
                        oracle.response.chars().take(100).collect::<String>());
                    insights.push(insight);
                    
                    // Learn causal relation
                    self.causal_model.learn_from_observation(
                        &oracle.question,
                        &oracle.response.chars().take(50).collect::<String>(),
                        oracle.entropy_reduction,
                        &thought.elp_state,
                    );
                    self.stats.causal_relations_discovered += 1;
                }
            }
        }
        
        insights
    }
    
    /// Determine if we should create a goal from this query
    fn should_create_goal(&self, thought: &FluxThought) -> bool {
        // Create goal if high importance (high ELP magnitude) and low certainty
        let elp_magnitude = thought.elp_state.ethos + thought.elp_state.logos + thought.elp_state.pathos;
        elp_magnitude > 18.0 && thought.certainty < 0.8
    }
    
    /// Process a goal-directed task
    pub async fn pursue_goal(&mut self, objective: &str) -> Result<AGIResponse> {
        let start = std::time::Instant::now();
        self.state.mode = AGIMode::Planning;
        
        // Create goal
        let elp = self.analyze_query(objective);
        let goal = self.goal_planner.create_goal(objective, &elp);
        let goal_id = goal.id;
        self.goal_planner.add_goal(goal);
        self.state.current_goal = Some(goal_id);
        
        // Plan
        let plan = self.goal_planner.plan_next_goal()?;
        
        if let Some(plan) = plan {
            // Execute
            self.state.mode = AGIMode::Executing;
            let success = self.goal_planner.execute_plan().await?;
            
            if success {
                self.goal_planner.mark_goal_achieved(goal_id);
                self.stats.goals_achieved += 1;
            }
            
            self.state.mode = AGIMode::Idle;
            self.state.current_goal = None;
            
            Ok(AGIResponse {
                answer: format!("Goal '{}' {}", objective, if success { "achieved" } else { "failed" }),
                confidence: if success { 0.9 } else { 0.3 },
                reasoning_steps: plan.tasks.len(),
                mode_used: AGIMode::Executing,
                goals_created: vec![goal_id],
                patterns_matched: 0,
                causal_insights: vec![],
                knowledge_gaps: vec![],
                processing_time_ms: start.elapsed().as_millis() as u64,
            })
        } else {
            Ok(AGIResponse {
                answer: "No plan could be created".to_string(),
                confidence: 0.1,
                reasoning_steps: 0,
                mode_used: AGIMode::Planning,
                goals_created: vec![goal_id],
                patterns_matched: 0,
                causal_insights: vec![],
                knowledge_gaps: vec![],
                processing_time_ms: start.elapsed().as_millis() as u64,
            })
        }
    }
    
    /// Ask a counterfactual question
    pub fn ask_counterfactual(&mut self, description: &str, target: &str, value: f64, query: &str) -> Result<String> {
        let cf = self.causal_model.ask_counterfactual(
            description,
            target,
            CausalValue::Numeric(value),
            query,
        )?;
        
        let answer = match cf.counterfactual_value {
            Some(CausalValue::Numeric(v)) => format!("If {} were {}, then {} would be approximately {:.2} (confidence: {:.0}%)", 
                target, value, query, v, cf.confidence * 100.0),
            Some(CausalValue::Boolean(b)) => format!("If {} were {}, then {} would be {} (confidence: {:.0}%)",
                target, value, query, b, cf.confidence * 100.0),
            _ => format!("Could not determine counterfactual outcome (confidence: {:.0}%)", cf.confidence * 100.0),
        };
        
        Ok(answer)
    }
    
    /// Explore a topic out of curiosity
    pub fn explore(&mut self, topic: &str) -> Vec<String> {
        let elp = ELPTensor { ethos: 5.0, logos: 7.0, pathos: 4.0 };
        self.curiosity.identify_gap(topic, topic, 0.8, &elp);
        
        let mut explorations = Vec::new();
        
        if let Some(gap) = self.curiosity.get_most_curious() {
            let action = self.curiosity.generate_exploration(&gap);
            explorations.push(format!("Exploring: {:?}", action.action_type));
        }
        
        explorations
    }
    
    /// Get current AGI state
    pub fn get_state(&self) -> &AGIState { &self.state }
    
    /// Get statistics
    pub fn get_stats(&self) -> &AGIStats { &self.stats }
    
    /// Get performance metrics for self-improvement
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.meta_learner.performance_tracker.get_current_metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_agi_creation() {
        let agi = AGICore::new();
        assert_eq!(agi.state.mode, AGIMode::Idle);
    }
    
    #[test]
    fn test_query_analysis() {
        let agi = AGICore::new();
        
        let elp = agi.analyze_query("Why should we be ethical?");
        assert!(elp.ethos > 5.0); // Ethical question
        assert!(elp.logos > 5.0); // "Why" question
        
        let elp2 = agi.analyze_query("How do I calculate this?");
        assert!(elp2.logos > 5.0); // Logical question
    }
    
    #[test]
    fn test_counterfactual() {
        let mut agi = AGICore::new();
        let elp = ELPTensor { ethos: 5.0, logos: 7.0, pathos: 3.0 };
        
        agi.causal_model.learn_from_observation("Exercise", "Health", 0.8, &elp);
        agi.causal_model.learn_from_observation("Health", "Happiness", 0.7, &elp);
        
        let result = agi.ask_counterfactual(
            "What if I exercised more?",
            "Exercise",
            1.0,
            "Happiness"
        );
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_exploration() {
        let mut agi = AGICore::new();
        let explorations = agi.explore("quantum computing");
        assert!(!explorations.is_empty());
    }
}
