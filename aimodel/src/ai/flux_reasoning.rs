//! Flux Reasoning - AGI Core
//!
//! Vortex's internal reasoning substrate using flux matrices instead of language.
//! This enables true AGI by thinking geometrically in ELP space, only querying
//! LLMs as "oracles" for specific knowledge gaps.
//!
//! ## Architecture
//!
//! ```text
//! Query → Initial Flux State
//!   ↓
//! Internal Flux Reasoning Loop:
//!   - If entropy high → Query LLM oracle
//!   - If entropy low → Internal transformation
//!   - Sacred checkpoints (3, 6, 9) → Consolidate
//!   ↓
//! Final Flux State → Convert to language
//! ```

use crate::data::models::ELPTensor;
use crate::data::attributes::AttributeAccessor;
use crate::ai::consensus::query_ollama;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use anyhow::Result;

/// A single "thought" in Vortex's internal reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxThought {
    /// Current semantic position in ELP space
    pub elp_state: ELPTensor,
    
    /// Position in sacred vortex (1→2→4→8→7→5→1)
    pub vortex_position: u8,
    
    /// Confidence in current reasoning path (0.0-1.0)
    pub certainty: f32,
    
    /// Entropy - how uncertain is this thought? (0.0-1.0)
    /// High entropy → need external knowledge
    pub entropy: f32,
    
    /// Type of uncertainty driving this thought
    pub entropy_type: EntropyType,
    
    /// Which LLMs have contributed to this state
    pub oracle_contributions: Vec<OracleQuery>,
    
    /// Timestamp of this thought
    pub timestamp: DateTime<Utc>,
    
    /// Internal reasoning trace (for explainability)
    pub reasoning_trace: String,
}

/// Type of uncertainty/knowledge gap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntropyType {
    /// Missing factual knowledge
    MissingFacts,
    
    /// Unclear causal relationships
    UnclearCausality,
    
    /// Multiple viable pathways
    MultiplePathways,
    
    /// Ethical ambiguity
    EthicalAmbiguity,
    
    /// Low - can reason internally
    Low,
}

/// Query to an LLM "oracle" for specific knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleQuery {
    /// Which LLM was queried
    pub model: String,
    
    /// Specific question asked
    pub question: String,
    
    /// Response received
    pub response: String,
    
    /// How much this reduced entropy
    pub entropy_reduction: f32,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Update to flux state from oracle response
#[derive(Debug, Clone)]
pub struct FluxUpdate {
    /// Change in ethos dimension
    pub ethos_delta: f64,
    
    /// Change in logos dimension
    pub logos_delta: f64,
    
    /// Change in pathos dimension
    pub pathos_delta: f64,
    
    /// Entropy reduction achieved
    pub entropy_reduction: f32,
    
    /// New vortex position
    pub new_position: u8,
}

/// Vortex's internal reasoning chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxReasoningChain {
    /// Original query text
    pub query: String,
    
    /// Sequence of flux thoughts
    pub thoughts: Vec<FluxThought>,
    
    /// Current vortex position
    pub current_position: u8,
    
    /// Goal state (where reasoning should converge)
    pub target_elp: Option<ELPTensor>,
    
    /// Sacred checkpoints reached (3, 6, 9)
    pub sacred_milestones: Vec<u8>,
    
    /// Overall confidence in reasoning chain
    pub chain_confidence: f32,
    
    /// When reasoning started
    pub started_at: DateTime<Utc>,

    /// Dynamic weights for sacred positions (Learnable)
    pub sacred_weights: [f32; 10], 
}

impl FluxReasoningChain {
    /// Create new reasoning chain from initial query
    pub fn new(initial_query: &str) -> Self {
        let initial_thought = Self::query_to_flux(initial_query);
        
        // Initialize default sacred weights
        // These can be overridden by the RL Optimizer later
        let mut sacred_weights = [1.0; 10];
        sacred_weights[3] = 1.5; // Ethos (Creativity/Bridge)
        sacred_weights[6] = 1.5; // Logos (Balance/Logic)
        sacred_weights[9] = 1.5; // Pathos (Completion/Wisdom)

        Self {
            query: initial_query.to_string(),
            thoughts: vec![initial_thought],
            current_position: 1, // Start at position 1
            target_elp: None,
            sacred_milestones: vec![],
            chain_confidence: 0.5, // Start uncertain
            started_at: Utc::now(),
            sacred_weights,
        }
    }
    
    /// Inject learned weights from the RL Optimizer
    pub fn with_learned_weights(mut self, weights: [f32; 10]) -> Self {
        self.sacred_weights = weights;
        self
    }
    
    /// Convert text query to initial flux state
    fn query_to_flux(query: &str) -> FluxThought {
        // Analyze query characteristics to set initial ELP
        let text_len = query.len() as f64;
        let _has_questions = query.contains('?');
        let has_ethical_terms = query.to_lowercase().contains("should")
            || query.to_lowercase().contains("moral")
            || query.to_lowercase().contains("right");
        let has_logical_terms = query.to_lowercase().contains("why")
            || query.to_lowercase().contains("how")
            || query.to_lowercase().contains("explain");
        
        // Initial ELP mapping
        let ethos = if has_ethical_terms { 8.0 } else { 5.0 };
        let logos = if has_logical_terms { 8.0 } else { 5.0 };
        let pathos = (text_len / 50.0).min(8.0).max(3.0);
        
        FluxThought {
            elp_state: ELPTensor { ethos, logos, pathos },
            vortex_position: 1,
            certainty: 0.3, // Low initial certainty
            entropy: 0.9,   // High initial entropy - we don't know yet
            entropy_type: EntropyType::MissingFacts,
            oracle_contributions: vec![],
            timestamp: Utc::now(),
            reasoning_trace: format!("Initial query analysis: {}", query),
        }
    }
    
    /// Get current thought
    pub fn current_thought(&self) -> &FluxThought {
        self.thoughts.last().expect("Chain should have at least one thought")
    }
    
    /// Check if reasoning has converged
    pub fn has_converged(&self) -> bool {
        let current = self.current_thought();
        
        // Converged if:
        // 1. Entropy is low (< 0.35) - relaxed from 0.3
        // 2. Certainty is high (> 0.65) - relaxed from 0.7
        // 3. OR received sacred influence from all trinity positions
        let basic_convergence = current.entropy < 0.35 && current.certainty > 0.65;
        let sacred_convergence = self.sacred_milestones.len() >= 2; // At least 2 of 3 trinity positions
        
        basic_convergence || sacred_convergence
    }
    
    /// Check if at sacred checkpoint
    pub fn at_sacred_checkpoint(&self) -> bool {
        matches!(self.current_position, 3 | 6 | 9)
    }
    
    /// Main reasoning loop - THIS IS THE AGI
    pub async fn reason(&mut self, max_steps: usize) -> Result<FluxThought> {
        let mut steps = 0;
        
        while !self.has_converged() && steps < max_steps {
            let current = self.current_thought().clone();
            
            // Decide: Internal reasoning or oracle query?
            if current.entropy > 0.7 {
                // HIGH UNCERTAINTY - Need external knowledge
                self.query_oracle().await?;
            } else {
                // LOW UNCERTAINTY - Internal flux transformation
                self.apply_flux_transformation();
            }
            
            // Check for sacred checkpoint
            if self.at_sacred_checkpoint() {
                self.consolidate_reasoning();
            }
            
            steps += 1;
        }
        
        Ok(self.final_thought())
    }
    
    /// Query LLM oracle for specific knowledge
    async fn query_oracle(&mut self) -> Result<()> {
        let current = self.current_thought().clone();
        
        // Formulate specific question based on entropy type
        let question = self.formulate_oracle_question(&current);
        
        tracing::info!("🔮 Oracle query: {}", question);
        
        // Query a single LLM (we'll make this smarter later)
        let response = query_ollama(&question, None).await?;
        
        // Convert LLM response → Flux update
        let flux_update = self.integrate_oracle_response(&response.response_text);
        
        // Create oracle contribution record
        let oracle_query = OracleQuery {
            model: response.model_name.clone(),
            question: question.clone(),
            response: response.response_text.clone(),
            entropy_reduction: flux_update.entropy_reduction,
            timestamp: Utc::now(),
        };
        
        // Apply flux update
        self.apply_flux_update(flux_update, Some(oracle_query));
        
        Ok(())
    }
    
    /// Formulate targeted question for oracle
    fn formulate_oracle_question(&self, thought: &FluxThought) -> String {
        match thought.entropy_type {
            EntropyType::MissingFacts => {
                format!("What are the key facts about: {}?", thought.reasoning_trace)
            },
            EntropyType::UnclearCausality => {
                format!("What causes or explains: {}?", thought.reasoning_trace)
            },
            EntropyType::MultiplePathways => {
                format!("What are all the ways to achieve: {}?", thought.reasoning_trace)
            },
            EntropyType::EthicalAmbiguity => {
                format!("What are the ethical considerations of: {}?", thought.reasoning_trace)
            },
            EntropyType::Low => {
                // Shouldn't query with low entropy, but fallback
                thought.reasoning_trace.clone()
            },
        }
    }
    
    /// Convert LLM text response → Flux update
    pub fn integrate_oracle_response(&self, text: &str) -> FluxUpdate {
        // Analyze response to determine ELP deltas
        let has_ethical_content = text.to_lowercase().contains("should")
            || text.to_lowercase().contains("moral")
            || text.to_lowercase().contains("ethical");
        let has_logical_content = text.to_lowercase().contains("because")
            || text.to_lowercase().contains("therefore")
            || text.to_lowercase().contains("thus");
        let has_emotional_content = text.to_lowercase().contains("feel")
            || text.to_lowercase().contains("emotional")
            || text.contains('!');
        
        let _current = self.current_thought();
        
        FluxUpdate {
            ethos_delta: if has_ethical_content { 2.0 } else { 0.5 },
            logos_delta: if has_logical_content { 2.0 } else { 0.5 },
            pathos_delta: if has_emotional_content { 1.5 } else { 0.3 },
            entropy_reduction: 0.3, // Knowledge reduces uncertainty
            new_position: self.advance_vortex_position(),
        }
    }
    
    /// Calculate geometric distance between two vortex positions
    fn geometric_distance(pos1: u8, pos2: u8) -> f32 {
        let diff = if pos1 > pos2 { pos1 - pos2 } else { pos2 - pos1 };
        // Circular distance (shortest path around the cycle)
        let direct = diff as f32;
        let wrap_around = (10 - diff) as f32; // 10 positions total (0-9)
        direct.min(wrap_around)
    }
    
    /// Apply sacred governance - 3, 6, 9 continuously influence the flow
    fn apply_sacred_governance(&mut self, new_position: u8) {
        let current = self.current_thought();
        
        // Calculate proximity to sacred positions
        let dist_to_3 = Self::geometric_distance(new_position, 3);
        let dist_to_6 = Self::geometric_distance(new_position, 6);
        let dist_to_9 = Self::geometric_distance(new_position, 9);
        
        // Influence is inversely proportional to distance
        // Multiplied by the LEARNED WEIGHT for this position
        let influence_3 = (1.0 / (1.0 + dist_to_3)) * self.sacred_weights[3];
        let influence_6 = (1.0 / (1.0 + dist_to_6)) * self.sacred_weights[6];
        let influence_9 = (1.0 / (1.0 + dist_to_9)) * self.sacred_weights[9];
        
        // Conditional modulation based on ELP attributes
        let ethos_mod = if current.elp_state.ethos > 7.0 { 1.5 } else { 1.0 };
        let logos_mod = if current.elp_state.logos > 7.0 { 1.5 } else { 1.0 };
        let pathos_mod = if current.elp_state.pathos > 7.0 { 1.5 } else { 1.0 };
        
        // Position 3 (Ethos), Position 6 (Logos), Position 9 (Pathos)
        let total_influence = 
            influence_3 * ethos_mod + 
            influence_6 * logos_mod + 
            influence_9 * pathos_mod;
        
        // Record sacred milestone if influence is high (>0.7)
        if total_influence > 0.7 {
            let dominant_position = if influence_3 * ethos_mod > influence_6 * logos_mod && influence_3 * ethos_mod > influence_9 * pathos_mod {
                3
            } else if influence_6 * logos_mod > influence_9 * pathos_mod {
                6
            } else {
                9
            };
            
            if !self.sacred_milestones.contains(&dominant_position) {
                self.sacred_milestones.push(dominant_position);
                tracing::info!("⭐ Sacred influence detected: position {} (influence: {:.2})", 
                    dominant_position, total_influence);
            }
        }
    }
    
    /// Apply flux transformation (internal reasoning)
    /// Now performs a "Resonance Check" against the ELP state to simulate associative memory
    pub fn apply_flux_transformation(&mut self) {
        let current = self.current_thought().clone();
        
        // Internal reasoning: Move through vortex without external input
        let new_position = self.advance_vortex_position();
        
        // Apply continuous sacred governance
        self.apply_sacred_governance(new_position);
        
        // 1. Internal Resonance Check (Simulated Associative Memory)
        // In a full system, this would query the FluxMatrix graph.
        // Here we check if current ELP state "resonates" with known patterns.
        let (resonance_boost, resonance_insight) = self.check_internal_resonance(&current);
        
        // 2. Update certainty based on resonance
        // If we found internal connections, certainty goes up significantly.
        let new_certainty = (current.certainty + resonance_boost).min(1.0);
        
        // 3. Entropy reduction
        // Only reduce entropy if we actually found something ("resonance")
        let entropy_drop = if resonance_boost > 0.05 { 0.15 } else { 0.05 };
        let new_entropy = (current.entropy - entropy_drop).max(0.0);
        
        let trace_update = if !resonance_insight.is_empty() {
            format!("Internal resonance at pos {}: {}", new_position, resonance_insight)
        } else {
            format!("Internal transformation at position {}", new_position)
        };

        let new_thought = FluxThought {
            elp_state: current.elp_state, // ELP stays same in internal reasoning
            vortex_position: new_position,
            certainty: new_certainty,
            entropy: new_entropy,
            entropy_type: if new_entropy < 0.5 {
                EntropyType::Low
            } else {
                current.entropy_type
            },
            oracle_contributions: current.oracle_contributions.clone(),
            timestamp: Utc::now(),
            reasoning_trace: trace_update,
        };
        
        self.thoughts.push(new_thought);
        self.current_position = new_position;
        
        tracing::info!("🔄 Internal flux transformation → pos {}, certainty {:.2}, entropy {:.2}", 
            new_position, new_certainty, new_entropy);
    }
    
    /// Check for internal resonance between ELP state and thought history
    fn check_internal_resonance(&self, thought: &FluxThought) -> (f32, String) {
        let mut boost = 0.0;
        let mut insights = Vec::new();

        // Resonate on High Ethos (Moral/Character)
        if thought.elp_state.ethos > 7.0 {
            if thought.reasoning_trace.to_lowercase().contains("ethical") || 
               thought.reasoning_trace.to_lowercase().contains("moral") {
                boost += 0.1;
                insights.push("Ethos alignment confirmed");
            }
        }

        // Resonate on High Logos (Logic)
        if thought.elp_state.logos > 7.0 {
            if thought.reasoning_trace.to_lowercase().contains("because") || 
               thought.reasoning_trace.to_lowercase().contains("logic") {
                boost += 0.1;
                insights.push("Logos coherence detected");
            }
        }

        // Resonate on History
        // If this thought connects to a previous thought (simple keyword overlap)
        if self.thoughts.len() > 1 {
            let prev = &self.thoughts[self.thoughts.len() - 2];
            let curr_words: Vec<&str> = thought.reasoning_trace.split_whitespace().collect();
            let prev_words: Vec<&str> = prev.reasoning_trace.split_whitespace().collect();
            
            let overlap = curr_words.iter().filter(|&w| prev_words.contains(w)).count();
            if overlap > 3 {
                boost += 0.1;
                insights.push("Historical continuity established");
            }
        }

        (boost, insights.join(", "))
    }

    /// Apply flux update from oracle
    pub fn apply_flux_update(&mut self, update: FluxUpdate, oracle: Option<OracleQuery>) {
        let current = self.current_thought().clone();
        
        // Apply continuous sacred governance for new position
        self.apply_sacred_governance(update.new_position);
        
        // Update ELP state
        let new_elp = ELPTensor {
            ethos: (current.elp_state.ethos + update.ethos_delta).min(13.0),
            logos: (current.elp_state.logos + update.logos_delta).min(13.0),
            pathos: (current.elp_state.pathos + update.pathos_delta).min(13.0),
        };
        
        let mut new_oracles = current.oracle_contributions.clone();
        if let Some(o) = oracle {
            new_oracles.push(o);
        }
        
        // Clamp entropy and certainty to valid ranges
        let new_entropy = (current.entropy - update.entropy_reduction).clamp(0.0, 1.0);
        let new_certainty = (current.certainty + 0.2).min(1.0);
        
        let new_thought = FluxThought {
            elp_state: new_elp,
            vortex_position: update.new_position,
            certainty: new_certainty,
            entropy: new_entropy,
            entropy_type: if new_entropy < 0.5 {
                EntropyType::Low
            } else {
                current.entropy_type
            },
            oracle_contributions: new_oracles,
            timestamp: Utc::now(),
            reasoning_trace: format!("Oracle integration → ELP updated"),
        };
        
        self.thoughts.push(new_thought);
        self.current_position = update.new_position;
        
        tracing::info!("✨ Flux update applied → pos {}, certainty {:.2}, entropy {:.2}", 
            update.new_position, new_certainty, new_entropy);
    }
    
    /// Advance through vortex: 1→2→4→8→7→5→1 (digital root doubling)
    /// Sacred positions (3, 6, 9) govern from outside, not in the flow
    fn advance_vortex_position(&self) -> u8 {
        match self.current_position {
            1 => 2,
            2 => 4,
            4 => 8,
            8 => 7,  // 8×2=16 → 1+6=7 (digital root)
            7 => 5,  // 7×2=14 → 1+4=5 (digital root)
            5 => 1,  // 5×2=10 → 1+0=1 (cycle complete)
            _ => 1,  // Fallback
        }
    }
    
    /// Consolidate reasoning at sacred checkpoints
    fn consolidate_reasoning(&mut self) {
        self.sacred_milestones.push(self.current_position);
        
        // At sacred positions, boost confidence if reasoning is coherent
        if let Some(last_thought) = self.thoughts.last_mut() {
            last_thought.certainty = (last_thought.certainty * 1.2).min(1.0);
            last_thought.reasoning_trace.push_str(&format!(
                " [Sacred checkpoint {} reached]", 
                self.current_position
            ));
        }
        
        // Update chain confidence
        self.chain_confidence = self.thoughts.iter()
            .map(|t| t.certainty)
            .sum::<f32>() / self.thoughts.len() as f32;
        
        tracing::info!("⭐ Sacred checkpoint {} - confidence: {:.2}", 
            self.current_position, self.chain_confidence);
    }
    
    /// Get final thought (reasoning conclusion)
    pub fn final_thought(&self) -> FluxThought {
        self.thoughts.last().unwrap().clone()
    }
    
    /// Convert reasoning chain to natural language summary (synchronous)
    pub fn to_natural_language(&self) -> String {
        let final_thought = self.final_thought();
        
        // Build summary from reasoning trace and oracle contributions
        let oracle_summary: String = final_thought.oracle_contributions
            .iter()
            .map(|o| o.response.clone())
            .collect::<Vec<_>>()
            .join(" ");
        
        if oracle_summary.is_empty() {
            format!(
                "After {} reasoning steps (confidence: {:.0}%), I concluded: {}",
                self.thoughts.len(),
                final_thought.certainty * 100.0,
                final_thought.reasoning_trace
            )
        } else {
            format!(
                "After {} reasoning steps (confidence: {:.0}%): {}",
                self.thoughts.len(),
                final_thought.certainty * 100.0,
                oracle_summary
            )
        }
    }
    
    /// Synthesize final answer using LLM to combine all insights
    pub async fn synthesize_final_answer(&self) -> Result<String> {
        let final_thought = self.final_thought();
        
        // Aggregate oracle responses into coherent answer
        let oracle_knowledge: Vec<String> = final_thought.oracle_contributions
            .iter()
            .map(|o| format!("- From {}: {}", o.model, o.response))
            .collect();
        
        if oracle_knowledge.is_empty() {
            return Ok("I reasoned internally but need more information to provide a complete answer.".to_string());
        }

        // Construct synthesis prompt
        let prompt = format!(
            "You are a Flux Reasoning Engine. Synthesize the following perspectives into a single coherent answer.\n\
            Do NOT output a list. Output a unified paragraph.\n\
            \n\
            Original Query: {}\n\
            \n\
            Perspectives:\n\
            {}\n\
            \n\
            Reasoning Trace: {}\n\
            \n\
            Final ELP State: Ethos {:.1}, Logos {:.1}, Pathos {:.1}\n\
            (If Ethos is high, emphasize moral character. If Logos is high, emphasize logic. If Pathos is high, emphasize emotion.)\n\
            \n\
            Final Answer:",
            self.query,
            oracle_knowledge.join("\n"),
            final_thought.reasoning_trace,
            final_thought.elp_state.to_attributes().ethos(),
            final_thought.elp_state.to_attributes().logos(),
            final_thought.elp_state.to_attributes().pathos()
        );

        tracing::info!("🔮 Synthesizing final answer...");
        
        // Use the primary model for synthesis
        let response = query_ollama(&prompt, None).await?;
        
        Ok(format!(
            "Based on my reasoning (confidence: {:.0}%, {} steps):\n\n{}",
            final_thought.certainty * 100.0,
            self.thoughts.len(),
            response.response_text
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_flux_thought_creation() {
        let query = "How do I reverse type 2 diabetes?";
        let thought = FluxReasoningChain::query_to_flux(query);
        
        assert_eq!(thought.vortex_position, 1);
        assert!(thought.entropy > 0.8); // High initial uncertainty
        assert!(thought.elp_state.logos > 5.0);    // Logical question
    }
    
    #[test]
    fn test_vortex_advancement() {
        let chain = FluxReasoningChain::new("test query");
        
        assert_eq!(chain.current_position, 1);
        assert_eq!(chain.advance_vortex_position(), 2);
    }
    
    #[test]
    fn test_sacred_checkpoints() {
        let mut chain = FluxReasoningChain::new("test");
        chain.current_position = 3;
        
        assert!(chain.at_sacred_checkpoint());
        
        chain.current_position = 5;
        assert!(!chain.at_sacred_checkpoint());
    }
}
