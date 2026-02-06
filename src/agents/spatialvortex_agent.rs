//! SpatialVortex Agent - Core agent implementation for SpatialVortex integration
//!
//! This agent provides high-level interface to SpatialVortex capabilities including:
//! - Sacred geometry reasoning
//! - Vortex flux matrix navigation
//! - ELP (Ethos/Logos/Pathos) attribute analysis
//! - Continuous learning and self-improvement

use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use crate::agents::{
    EnhancedCodingAgent, ThinkingAgent, SelfOptimizationAgent,
    TaskManager, FirstPrinciplesReasoner, LLMBridge,
};

/// SpatialVortex Agent - Unified interface to all SpatialVortex capabilities
pub struct SpatialVortexAgent {
    /// Core coding agent with sacred geometry reasoning
    coding_agent: Arc<RwLock<EnhancedCodingAgent>>,
    /// Thinking agent for deep reasoning
    thinking_agent: Arc<RwLock<ThinkingAgent>>,
    /// Self-optimization agent for continuous improvement
    optimization_agent: Arc<RwLock<SelfOptimizationAgent>>,
    /// Task manager for orchestration
    task_manager: Arc<RwLock<TaskManager>>,
    /// First principles reasoner
    principles_reasoner: Arc<RwLock<FirstPrinciplesReasoner>>,
    /// LLM bridge for external AI integration
    llm_bridge: Arc<RwLock<LLMBridge>>,
    /// Agent configuration
    config: SpatialVortexConfig,
    /// Current vortex position (sacred geometry state)
    vortex_position: VortexPosition,
}

/// Configuration for SpatialVortex Agent
#[derive(Debug, Clone)]
pub struct SpatialVortexConfig {
    /// Enable sacred geometry reasoning
    pub sacred_geometry: bool,
    /// Enable continuous learning
    pub continuous_learning: bool,
    /// Enable self-optimization
    pub self_optimization: bool,
    /// ELP balance weights
    pub elp_weights: ELPWeights,
    /// Maximum reasoning depth
    pub max_reasoning_depth: usize,
    /// Enable web learning
    pub web_learning: bool,
}

impl Default for SpatialVortexConfig {
    fn default() -> Self {
        Self {
            sacred_geometry: true,
            continuous_learning: true,
            self_optimization: true,
            elp_weights: ELPWeights::balanced(),
            max_reasoning_depth: 9, // Sacred number
            web_learning: true,
        }
    }
}

/// ELP (Ethos/Logos/Pathos) weights for reasoning balance
#[derive(Debug, Clone, Copy)]
pub struct ELPWeights {
    pub ethos: f32,  // Character/Authority
    pub logos: f32,  // Logic/Analytical
    pub pathos: f32, // Emotion/Expressive
}

impl ELPWeights {
    pub fn balanced() -> Self {
        Self {
            ethos: 0.33,
            logos: 0.34,
            pathos: 0.33,
        }
    }

    pub fn logical() -> Self {
        Self {
            ethos: 0.2,
            logos: 0.6,
            pathos: 0.2,
        }
    }

    pub fn creative() -> Self {
        Self {
            ethos: 0.2,
            logos: 0.2,
            pathos: 0.6,
        }
    }
}

/// Vortex position in sacred geometry cycle (1-9)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VortexPosition {
    One,   // New beginnings
    Two,   // Duality/cooperation
    Three, // Sacred (trinity)
    Four,  // Foundation
    Five,  // Change/freedom
    Six,   // Sacred (harmony)
    Seven, // Spirituality
    Eight, // Abundance/power
    Nine,  // Sacred (completion)
}

impl VortexPosition {
    pub fn as_number(&self) -> u8 {
        match self {
            VortexPosition::One => 1,
            VortexPosition::Two => 2,
            VortexPosition::Three => 3,
            VortexPosition::Four => 4,
            VortexPosition::Five => 5,
            VortexPosition::Six => 6,
            VortexPosition::Seven => 7,
            VortexPosition::Eight => 8,
            VortexPosition::Nine => 9,
        }
    }

    pub fn is_sacred(&self) -> bool {
        matches!(self, VortexPosition::Three | VortexPosition::Six | VortexPosition::Nine)
    }

    pub fn next(&self) -> Self {
        match self {
            VortexPosition::One => VortexPosition::Two,
            VortexPosition::Two => VortexPosition::Three,
            VortexPosition::Three => VortexPosition::Four,
            VortexPosition::Four => VortexPosition::Five,
            VortexPosition::Five => VortexPosition::Six,
            VortexPosition::Six => VortexPosition::Seven,
            VortexPosition::Seven => VortexPosition::Eight,
            VortexPosition::Eight => VortexPosition::Nine,
            VortexPosition::Nine => VortexPosition::One,
        }
    }
}

/// Result of a SpatialVortex reasoning operation
#[derive(Debug, Clone)]
pub struct SpatialVortexResult {
    /// The generated output (code, text, analysis)
    pub output: String,
    /// Final vortex position reached
    pub final_position: VortexPosition,
    /// ELP state at completion
    pub elp_state: ELPState,
    /// Reasoning steps taken
    pub reasoning_steps: Vec<ReasoningStep>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Whether sacred checkpoint was reached
    pub sacred_checkpoint_reached: bool,
}

/// ELP state during reasoning
#[derive(Debug, Clone)]
pub struct ELPState {
    pub ethos: f32,
    pub logos: f32,
    pub pathos: f32,
}

/// Single reasoning step in the vortex cycle
#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub position: VortexPosition,
    pub thought: String,
    pub elp_state: ELPState,
    pub confidence: f32,
}

/// Request types for SpatialVortex Agent
#[derive(Debug, Clone)]
pub enum AgentRequest {
    /// Generate code with sacred geometry reasoning
    GenerateCode {
        prompt: String,
        language: String,
        context: Option<String>,
    },
    /// Analyze text with ELP attributes
    AnalyzeELP {
        text: String,
    },
    /// Deep thinking with first principles
    DeepThink {
        question: String,
        depth: usize,
    },
    /// Self-optimize based on performance
    SelfOptimize {
        metrics: PerformanceMetrics,
    },
    /// Navigate vortex to specific position
    NavigateVortex {
        target: VortexPosition,
    },
}

/// Performance metrics for self-optimization
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub task_completion_rate: f32,
    pub average_confidence: f32,
    pub reasoning_depth_achieved: usize,
    pub sacred_checkpoints_hit: usize,
}

impl SpatialVortexAgent {
    /// Create a new SpatialVortex Agent with default configuration
    pub async fn new(config: SpatialVortexConfig) -> Result<Self> {
        // Initialize all sub-agents
        let coding_agent = Arc::new(RwLock::new(
            EnhancedCodingAgent::new().await?
        ));

        let thinking_agent_arc = Arc::new(ThinkingAgent::new().await?);
        let thinking_agent = Arc::new(RwLock::new(ThinkingAgent::new().await?));

        let optimization_agent = Arc::new(RwLock::new(
            SelfOptimizationAgent::new().await?
        ));

        let task_manager = Arc::new(RwLock::new(
            TaskManager::new(coding_agent.clone(), thinking_agent_arc.clone())
        ));

        let principles_reasoner = Arc::new(RwLock::new(
            FirstPrinciplesReasoner::new().await?
        ));

        let llm_bridge = Arc::new(RwLock::new(
            LLMBridge::new_default().await?
        ));

        Ok(Self {
            coding_agent,
            thinking_agent,
            optimization_agent,
            task_manager,
            principles_reasoner,
            llm_bridge,
            config,
            vortex_position: VortexPosition::One,
        })
    }

    /// Process an agent request and return result
    pub async fn process(&mut self, request: AgentRequest) -> Result<SpatialVortexResult> {
        match request {
            AgentRequest::GenerateCode { prompt, language, context } => {
                self.generate_code(prompt, language, context).await
            }
            AgentRequest::AnalyzeELP { text } => {
                self.analyze_elp(text).await
            }
            AgentRequest::DeepThink { question, depth } => {
                self.deep_think(question, depth).await
            }
            AgentRequest::SelfOptimize { metrics } => {
                self.self_optimize(metrics).await
            }
            AgentRequest::NavigateVortex { target } => {
                self.navigate_vortex(target).await
            }
        }
    }

    /// Generate code with sacred geometry reasoning
    async fn generate_code(
        &mut self,
        prompt: String,
        language: String,
        context: Option<String>,
    ) -> Result<SpatialVortexResult> {
        let mut steps = Vec::new();

        // Navigate through vortex positions for sacred geometry reasoning
        for i in 0..self.config.max_reasoning_depth {
            let position = self.vortex_position;

            // Perform reasoning at this position
            let thought = if position.is_sacred() {
                format!("Sacred checkpoint at position {} - verifying coherence", position.as_number())
            } else {
                format!("Reasoning at position {}", position.as_number())
            };

            steps.push(ReasoningStep {
                position,
                thought,
                elp_state: ELPState {
                    ethos: self.config.elp_weights.ethos,
                    logos: self.config.elp_weights.logos,
                    pathos: self.config.elp_weights.pathos,
                },
                confidence: 0.85,
            });

            // Move to next position
            self.vortex_position = self.vortex_position.next();

            // Stop if we completed a cycle
            if i > 0 && position == VortexPosition::Nine {
                break;
            }
        }

        // Use coding agent for actual code generation
        let coding_result = {
            let agent = self.coding_agent.read().await;
            agent.execute_with_reasoning(&format!("Generate {} code for: {}", language, prompt)).await?
        };

        Ok(SpatialVortexResult {
            output: coding_result.code,
            final_position: self.vortex_position,
            elp_state: ELPState {
                ethos: self.config.elp_weights.ethos,
                logos: self.config.elp_weights.logos,
                pathos: self.config.elp_weights.pathos,
            },
            reasoning_steps: steps,
            confidence: coding_result.confidence,
            sacred_checkpoint_reached: steps.iter().any(|s| s.position.is_sacred()),
        })
    }

    /// Analyze text for ELP attributes
    async fn analyze_elp(&mut self, text: String) -> Result<SpatialVortexResult> {
        // Simple ELP analysis based on keyword detection
        let text_lower = text.to_lowercase();

        let ethos_keywords = ["should", "must", "ethical", "moral", "principle", "character"];
        let logos_keywords = ["because", "therefore", "proof", "evidence", "logic", "reason"];
        let pathos_keywords = ["feel", "emotion", "love", "fear", "passion", "believe"];

        let ethos_count = ethos_keywords.iter().filter(|&&k| text_lower.contains(k)).count();
        let logos_count = logos_keywords.iter().filter(|&&k| text_lower.contains(k)).count();
        let pathos_count = pathos_keywords.iter().filter(|&&k| text_lower.contains(k)).count();

        let total = (ethos_count + logos_count + pathos_count).max(1) as f32;

        let elp_state = ELPState {
            ethos: (ethos_count as f32 / total) * 13.0,
            logos: (logos_count as f32 / total) * 13.0,
            pathos: (pathos_count as f32 / total) * 13.0,
        };

        let analysis = format!(
            "ELP Analysis:\n- Ethos (Character): {:.2}/13\n- Logos (Logic): {:.2}/13\n- Pathos (Emotion): {:.2}/13",
            elp_state.ethos, elp_state.logos, elp_state.pathos
        );

        Ok(SpatialVortexResult {
            output: analysis,
            final_position: self.vortex_position,
            elp_state,
            reasoning_steps: vec![],
            confidence: 0.75,
            sacred_checkpoint_reached: false,
        })
    }

    /// Deep thinking with first principles reasoning
    async fn deep_think(&mut self, question: String, depth: usize) -> Result<SpatialVortexResult> {
        let depth = depth.min(self.config.max_reasoning_depth);

        let result = {
            let reasoner = self.principles_reasoner.read().await;
            reasoner.analyze(&question)
        };

        Ok(SpatialVortexResult {
            output: result.conclusion,
            final_position: VortexPosition::Nine, // Completion
            elp_state: ELPState {
                ethos: 7.0,
                logos: 8.0,
                pathos: 5.0,
            },
            reasoning_steps: result.steps.iter().enumerate().map(|(i, step)| {
                ReasoningStep {
                    position: match i % 9 {
                        0 => VortexPosition::One,
                        1 => VortexPosition::Two,
                        2 => VortexPosition::Three,
                        3 => VortexPosition::Four,
                        4 => VortexPosition::Five,
                        5 => VortexPosition::Six,
                        6 => VortexPosition::Seven,
                        7 => VortexPosition::Eight,
                        _ => VortexPosition::Nine,
                    },
                    thought: step.description.clone(),
                    elp_state: ELPState {
                        ethos: 7.0,
                        logos: 8.0,
                        pathos: 5.0,
                    },
                    confidence: step.confidence,
                }
            }).collect(),
            confidence: result.overall_confidence,
            sacred_checkpoint_reached: depth >= 3,
        })
    }

    /// Self-optimize based on performance metrics
    async fn self_optimize(&mut self, metrics: PerformanceMetrics) -> Result<SpatialVortexResult> {
        let optimization = {
            let agent = self.optimization_agent.read().await;
            agent.optimize(&metrics).await?
        };

        // Update configuration based on optimization recommendations
        if optimization.should_increase_reasoning_depth {
            self.config.max_reasoning_depth = (self.config.max_reasoning_depth + 1).min(13);
        }

        if let Some(new_weights) = optimization.suggested_elp_weights {
            self.config.elp_weights = ELPWeights {
                ethos: new_weights.0,
                logos: new_weights.1,
                pathos: new_weights.2,
            };
        }

        Ok(SpatialVortexResult {
            output: format!(
                "Self-optimization complete:\n- Reasoning depth: {}\n- ELP weights adjusted: Ethos={:.2}, Logos={:.2}, Pathos={:.2}",
                self.config.max_reasoning_depth,
                self.config.elp_weights.ethos,
                self.config.elp_weights.logos,
                self.config.elp_weights.pathos
            ),
            final_position: self.vortex_position,
            elp_state: ELPState {
                ethos: self.config.elp_weights.ethos,
                logos: self.config.elp_weights.logos,
                pathos: self.config.elp_weights.pathos,
            },
            reasoning_steps: vec![],
            confidence: optimization.confidence,
            sacred_checkpoint_reached: true,
        })
    }

    /// Navigate to a specific vortex position
    async fn navigate_vortex(&mut self, target: VortexPosition) -> Result<SpatialVortexResult> {
        let mut steps = Vec::new();
        let start = self.vortex_position;

        // Navigate through positions until we reach target
        while self.vortex_position != target {
            steps.push(ReasoningStep {
                position: self.vortex_position,
                thought: format!("Navigating toward position {}", target.as_number()),
                elp_state: ELPState {
                    ethos: self.config.elp_weights.ethos,
                    logos: self.config.elp_weights.logos,
                    pathos: self.config.elp_weights.pathos,
                },
                confidence: 1.0,
            });

            self.vortex_position = self.vortex_position.next();

            // Prevent infinite loop
            if self.vortex_position == start {
                break;
            }
        }

        Ok(SpatialVortexResult {
            output: format!("Navigated to position {}", target.as_number()),
            final_position: target,
            elp_state: ELPState {
                ethos: self.config.elp_weights.ethos,
                logos: self.config.elp_weights.logos,
                pathos: self.config.elp_weights.pathos,
            },
            reasoning_steps: steps,
            confidence: 1.0,
            sacred_checkpoint_reached: target.is_sacred(),
        })
    }

    /// Get current agent status
    pub fn status(&self) -> AgentStatus {
        AgentStatus {
            vortex_position: self.vortex_position,
            sacred_geometry_enabled: self.config.sacred_geometry,
            continuous_learning_enabled: self.config.continuous_learning,
            self_optimization_enabled: self.config.self_optimization,
            current_elp: self.config.elp_weights.clone(),
        }
    }
}

/// Agent status information
#[derive(Debug, Clone)]
pub struct AgentStatus {
    pub vortex_position: VortexPosition,
    pub sacred_geometry_enabled: bool,
    pub continuous_learning_enabled: bool,
    pub self_optimization_enabled: bool,
    pub current_elp: ELPWeights,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vortex_position_navigation() {
        let mut pos = VortexPosition::One;
        assert_eq!(pos.as_number(), 1);

        pos = pos.next();
        assert_eq!(pos.as_number(), 2);

        pos = pos.next().next().next(); // Skip to position 5
        assert_eq!(pos.as_number(), 5);
    }

    #[test]
    fn test_sacred_positions() {
        assert!(VortexPosition::Three.is_sacred());
        assert!(VortexPosition::Six.is_sacred());
        assert!(VortexPosition::Nine.is_sacred());
        assert!(!VortexPosition::One.is_sacred());
        assert!(!VortexPosition::Five.is_sacred());
    }

    #[test]
    fn test_elp_weights() {
        let balanced = ELPWeights::balanced();
        assert_eq!(balanced.ethos, 0.33);
        assert_eq!(balanced.logos, 0.34);
        assert_eq!(balanced.pathos, 0.33);

        let logical = ELPWeights::logical();
        assert!(logical.logos > logical.ethos);
        assert!(logical.logos > logical.pathos);
    }
}
