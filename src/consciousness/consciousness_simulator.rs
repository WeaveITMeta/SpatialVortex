//! Consciousness Simulator - High-level API for conscious dialogue
//!
//! Provides a unified interface to simulate consciousness through:
//! - Global workspace architecture
//! - Multi-perspective thinking (Ethos, Logos, Pathos)
//! - Sacred geometry checkpoints
//! - Internal dialogue between agents

use super::{
    GlobalWorkspace, 
    MetaCognitiveMonitor, 
    PredictiveProcessor, 
    IntegratedInformationCalculator, 
    Thought, 
    ThoughtPriority,
    DreamModule, // Import DreamModule
    SubjectGraph, // Import SubjectGraph
    FluxSubjectDefinition, // Import SubjectDefinition
    cognitive_module::CognitiveModule, // Import CognitiveModule trait
};
use super::streaming::{ConsciousnessStreamingServer, create_word_insights};
use super::analytics::{AnalyticsSnapshot, ConsciousnessMetrics, MetaCognitiveMetrics, PredictiveMetrics, ELPMetrics, SacredGeometryMetrics, SessionStats, ThoughtMetrics};
use super::background_learner::BackgroundLearner;
use crate::agents::thinking_agent::ThinkingAgent;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Complete consciousness simulation system
pub struct ConsciousnessSimulator {
    /// Global workspace (theater of consciousness)
    #[allow(dead_code)]
    workspace: Arc<RwLock<GlobalWorkspace>>,
    
    /// Specialized thinking agents for perspectives
    ethos_agent: Arc<ThinkingAgent>,   // Moral perspective
    logos_agent: Arc<ThinkingAgent>,   // Logical perspective
    pathos_agent: Arc<ThinkingAgent>,  // Emotional perspective
    
    /// Meta-cognitive monitor (self-awareness)
    meta_monitor: Arc<RwLock<MetaCognitiveMonitor>>,
    
    /// Predictive processor (learning from errors)
    predictor: Arc<RwLock<PredictiveProcessor>>,
    
    /// Integrated information calculator (Φ)
    phi_calculator: Arc<RwLock<IntegratedInformationCalculator>>,
    
    /// WebTransport streaming server (v1.5.0)
    streaming_server: Option<Arc<ConsciousnessStreamingServer>>,
    
    /// Background learner (v1.5.1)
    background_learner: Option<Arc<BackgroundLearner>>,
    
    /// Dream module (v1.6.0) - Generative dreaming
    dream_module: Option<Arc<RwLock<DreamModule>>>,
    
    /// Subject Graph (v1.7.0) - Feed-forward subject chaining
    subject_graph: Arc<RwLock<SubjectGraph>>,
    
    /// Unique session ID
    session_id: String,
    
    /// Enable internal dialogue display
    show_internal_dialogue: bool,
}

/// Response from consciousness simulation
#[derive(Debug, Clone)]
pub struct ConsciousResponse {
    /// Final synthesized answer
    pub answer: String,
    
    /// Internal dialogue/thought process
    pub internal_dialogue: Vec<InternalThought>,
    
    /// Dominant ELP dimensions
    pub ethos_weight: f64,
    pub logos_weight: f64,
    pub pathos_weight: f64,
    
    /// Confidence in response
    pub confidence: f64,
    
    /// Sacred checkpoint insights
    pub checkpoint_insights: Vec<String>,
    
    /// Meta-cognitive insights (v1.4.0)
    pub mental_state: String,
    pub awareness_level: f64,
    pub detected_patterns: Vec<String>,
    
    /// Predictive processing metrics (v1.4.0)
    pub prediction_accuracy: f64,
    pub current_surprise: f64,
    pub learning_progress: f64,
    
    /// Integrated information (v1.4.0)
    pub phi: f64,
    pub consciousness_level: f64,
}

/// A single step in the internal dialogue
#[derive(Debug, Clone)]
pub struct InternalThought {
    pub agent: String,
    pub thought: String,
    pub elp_profile: (f64, f64, f64),
}

impl InternalThought {
    /// Create new internal thought
    pub fn new(agent: String, thought: String, elp_profile: (f64, f64, f64)) -> Self {
        Self {
            agent,
            thought,
            elp_profile,
        }
    }
}

impl ConsciousnessSimulator {
    /// Create new consciousness simulator
    pub async fn new(show_internal_dialogue: bool) -> Self {
        let session_id = Uuid::new_v4().to_string();
        
        // Try to initialize dream module
        let dream_module = if let Ok(dm) = DreamModule::new().await {
            let dm_arc = Arc::new(RwLock::new(dm));
            Some(dm_arc)
        } else {
            None
        };

        // Initialize Subject Graph
        let mut subject_graph = SubjectGraph::new(64); // 64-dim workspace
        // Add core subjects (simplified)
        subject_graph.add_subject(FluxSubjectDefinition::new("Ethics", 64, 64));
        subject_graph.add_subject(FluxSubjectDefinition::new("Logic", 64, 64));
        subject_graph.add_subject(FluxSubjectDefinition::new("Emotion", 64, 64));
        
        Self {
            workspace: Arc::new(RwLock::new(GlobalWorkspace::new())),
            ethos_agent: Arc::new(ThinkingAgent::new()),
            logos_agent: Arc::new(ThinkingAgent::new()),
            pathos_agent: Arc::new(ThinkingAgent::new()),
            meta_monitor: Arc::new(RwLock::new(MetaCognitiveMonitor::new())),
            predictor: Arc::new(RwLock::new(PredictiveProcessor::new())),
            phi_calculator: Arc::new(RwLock::new(IntegratedInformationCalculator::new())),
            streaming_server: None, // Disabled by default
            background_learner: None, // Disabled by default
            dream_module,
            subject_graph: Arc::new(RwLock::new(subject_graph)),
            session_id,
            show_internal_dialogue,
        }
    }
    
    /// Create new consciousness simulator with streaming enabled
    pub async fn with_streaming(show_internal_dialogue: bool) -> Self {
        let session_id = Uuid::new_v4().to_string();
        let streaming_server = Arc::new(ConsciousnessStreamingServer::new(session_id.clone()));
        
        let dream_module = if let Ok(dm) = DreamModule::new().await {
            let dm_arc = Arc::new(RwLock::new(dm));
            Some(dm_arc)
        } else {
            None
        };

        // Initialize Subject Graph
        let mut subject_graph = SubjectGraph::new(64); 
        subject_graph.add_subject(FluxSubjectDefinition::new("Ethics", 64, 64));
        subject_graph.add_subject(FluxSubjectDefinition::new("Logic", 64, 64));
        subject_graph.add_subject(FluxSubjectDefinition::new("Emotion", 64, 64));

        Self {
            workspace: Arc::new(RwLock::new(GlobalWorkspace::new())),
            ethos_agent: Arc::new(ThinkingAgent::new()),
            logos_agent: Arc::new(ThinkingAgent::new()),
            pathos_agent: Arc::new(ThinkingAgent::new()),
            meta_monitor: Arc::new(RwLock::new(MetaCognitiveMonitor::new())),
            predictor: Arc::new(RwLock::new(PredictiveProcessor::new())),
            phi_calculator: Arc::new(RwLock::new(IntegratedInformationCalculator::new())),
            streaming_server: Some(streaming_server),
            background_learner: None, // Disabled by default
            dream_module,
            subject_graph: Arc::new(RwLock::new(subject_graph)),
            session_id,
            show_internal_dialogue,
        }
    }
    
    /// Suggest next subject based on current activation (Feed-Forward Chaining)
    pub async fn suggest_next_subject(&self) -> Option<String> {
        let graph = self.subject_graph.read().await;
        
        // Get top active subjects (mocked activation for now)
        // In a real system, we'd pass the current workspace vector
        let top_k = graph.topk_subjects(1);
        
        if let Some(id) = top_k.first() {
            if let Some(subject) = graph.subjects.get(id) {
                return Some(subject.def.name.clone());
            }
        }
        None
    }
    
    /// Get streaming server (if enabled)
    pub fn streaming_server(&self) -> Option<Arc<ConsciousnessStreamingServer>> {
        self.streaming_server.clone()
    }
    
    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    
    /// Get meta-cognitive monitor reference (for Memory Palace)
    pub fn meta_monitor(&self) -> &Arc<RwLock<MetaCognitiveMonitor>> {
        &self.meta_monitor
    }
    
    /// Get predictive processor reference (for Memory Palace)
    pub fn predictor(&self) -> &Arc<RwLock<PredictiveProcessor>> {
        &self.predictor
    }
    
    /// Get Φ calculator reference (for Memory Palace)
    pub fn phi_calculator(&self) -> &Arc<RwLock<IntegratedInformationCalculator>> {
        &self.phi_calculator
    }
    
    /// Enable background learning (v1.5.1)
    pub async fn enable_background_learning(&mut self) -> Result<()> {
        if self.background_learner.is_some() {
            return Ok(()); // Already enabled
        }
        
        let learner = Arc::new(BackgroundLearner::new(
            self.meta_monitor.clone(),
            self.predictor.clone(),
            self.phi_calculator.clone(),
        ));
        
        // Start learning immediately
        learner.start().await?;
        
        self.background_learner = Some(learner);
        
        Ok(())
    }
    
    /// Start background learning (if enabled)
    pub async fn start_background_learning(&self) -> Result<()> {
        if let Some(ref learner) = self.background_learner {
            learner.start().await?;
        }
        Ok(())
    }
    
    /// Stop background learning
    pub async fn stop_background_learning(&self) {
        if let Some(ref learner) = self.background_learner {
            learner.stop().await;
        }
    }
    
    /// Check if background learning is active
    pub async fn is_learning_active(&self) -> bool {
        if let Some(ref learner) = self.background_learner {
            learner.is_active().await
        } else {
            false
        }
    }
    
    /// Get background learning statistics
    pub async fn learning_stats(&self) -> Option<super::background_learner::LearningStats> {
        if let Some(ref learner) = self.background_learner {
            Some(learner.stats().await)
        } else {
            None
        }
    }
    
    /// Get full analytics snapshot (v1.5.0)
    pub async fn get_analytics_snapshot(&self) -> AnalyticsSnapshot {
        let phi_calc = self.phi_calculator.read().await;
        let meta_monitor = self.meta_monitor.read().await;
        let predictor = self.predictor.read().await;
        
        // Build consciousness metrics
        let consciousness = ConsciousnessMetrics {
            phi: phi_calc.phi(),
            consciousness_level: phi_calc.consciousness_level(),
            peak_phi: phi_calc.peak_phi(),
            average_phi: phi_calc.average_phi(),
            network_size: phi_calc.network_size(),
            connection_count: phi_calc.connection_count(),
            integration_strength: phi_calc.phi() / phi_calc.network_size().max(1) as f64,
        };
        
        // Build meta-cognitive metrics
        let meta_cognition = MetaCognitiveMetrics {
            mental_state: meta_monitor.mental_state().to_string(),
            awareness_level: meta_monitor.metrics().awareness_level,
            introspection_depth: meta_monitor.metrics().introspection_depth,
            pattern_recognition: meta_monitor.metrics().pattern_recognition,
            self_correction_rate: meta_monitor.metrics().self_correction_rate,
            detected_patterns: vec![], // TODO: Convert patterns to PatternInfo
            state_duration_ms: 0, // TODO: Track state transitions
        };
        
        // Build predictive metrics
        let prediction = PredictiveMetrics {
            accuracy: predictor.prediction_accuracy(),
            current_surprise: predictor.current_surprise(),
            learning_progress: predictor.total_learning(),
            model_confidence: predictor.model_confidence(),
            prediction_history: vec![], // TODO: Expose history
            surprise_trend: vec![], // TODO: Track surprise over time
        };
        
        // Build ELP metrics (would need dialogue context)
        let elp_balance = ELPMetrics::default();
        
        // Build sacred geometry metrics (would need vortex tracker)
        let sacred_geometry = SacredGeometryMetrics::default();
        
        // Build session stats
        let session_stats = SessionStats::default();
        
        AnalyticsSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            session_id: self.session_id.clone(),
            consciousness,
            meta_cognition,
            prediction,
            elp_balance,
            sacred_geometry,
            session_stats,
        }
    }
    
    /// Simulate conscious thought process and generate response
    pub async fn generate_response(&self, input: &str, context: Option<String>) -> Result<ConsciousResponse> {
        let start_time = std::time::Instant::now();
        
        // Enhance input with context if available
        let enhanced_input = if let Some(ctx) = context {
            format!("Context:\n{}\n\nInput:\n{}", ctx, input)
        } else {
            input.to_string()
        };
        
        // 1. Predictive processing - Anticipate input
        let prediction = self.predictor.read().await.predict_next(); // No args
        
        // Create a thought object for the input to calculate surprise
        let input_thought = Thought::new(
            input.to_string(),
            "user".to_string(),
            ThoughtPriority::High,
        );
        
        let surprise_signal = self.predictor.write().await.observe_actual(&input_thought);
        let surprise = surprise_signal.magnitude;
        
        // 2. Dream Module Injection (Creative Spark) - v1.6.0
        if let Some(dm) = &self.dream_module {
            // If surprise is low (boring), inject a dream
            if surprise < 0.3 {
                // process() comes from CognitiveModule trait
                let _ = dm.read().await.process(input).await;
            }
        }
        
        // 3. Internal Dialogue Generation
        let mut internal_dialogue = Vec::new();
        
        // Ethos perspective
        let ethos_thought = self.get_ethos_perspective(&enhanced_input).await?;
        let internal_ethos = InternalThought::new(
            "Ethos".to_string(), 
            ethos_thought.clone(),
            (0.8, 0.1, 0.1)
        );
        internal_dialogue.push(internal_ethos.clone());
        self.process_thought(&internal_ethos).await;
        
        // Logos perspective
        let logos_thought = self.get_logos_perspective(&enhanced_input).await?;
        let internal_logos = InternalThought::new(
            "Logos".to_string(), 
            logos_thought.clone(),
            (0.1, 0.8, 0.1)
        );
        internal_dialogue.push(internal_logos.clone());
        self.process_thought(&internal_logos).await;
        
        // Pathos perspective
        let pathos_thought = self.get_pathos_perspective(&enhanced_input).await?;
        let internal_pathos = InternalThought::new(
            "Pathos".to_string(), 
            pathos_thought.clone(),
            (0.1, 0.1, 0.8)
        );
        internal_dialogue.push(internal_pathos.clone());
        self.process_thought(&internal_pathos).await;
        
        // Sacred Checkpoint 3: Moral Integration
        let checkpoint_3 = self.sacred_checkpoint_3(&ethos_thought, &logos_thought, &pathos_thought).await?;
        let internal_cp3 = InternalThought::new(
            "Sacred 3 (Moral)".to_string(), 
            checkpoint_3.clone(),
            (0.6, 0.2, 0.2)
        );
        internal_dialogue.push(internal_cp3.clone());
        self.process_thought(&internal_cp3).await;
        
        // Sacred Checkpoint 6: Logical Refinement
        let checkpoint_6 = self.sacred_checkpoint_6(&checkpoint_3, &logos_thought).await?;
        let internal_cp6 = InternalThought::new(
            "Sacred 6 (Logic)".to_string(), 
            checkpoint_6.clone(),
            (0.2, 0.6, 0.2)
        );
        internal_dialogue.push(internal_cp6.clone());
        self.process_thought(&internal_cp6).await;
        
        // Sacred Checkpoint 9: Divine Synthesis
        let final_answer = self.sacred_checkpoint_9(&checkpoint_6).await?;
        let internal_cp9 = InternalThought::new(
            "Sacred 9 (Divine)".to_string(), 
            final_answer.clone(),
            (0.33, 0.33, 0.34)
        );
        internal_dialogue.push(internal_cp9.clone());
        self.process_thought(&internal_cp9).await;
        
        let _processing_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Calculate metrics
        let (ethos_weight, logos_weight, pathos_weight) = self.calculate_elp_weights(&internal_dialogue);
        
        let checkpoint_insights = vec![
            format!("CP3: {}", checkpoint_3),
            format!("CP6: {}", checkpoint_6),
            format!("CP9: {}", final_answer),
        ];
        
        // Get meta-cognitive insights
        let meta_monitor = self.meta_monitor.read().await;
        let mental_state = meta_monitor.mental_state().to_string();
        let awareness_level = meta_monitor.metrics().awareness_level;
        let detected_patterns: Vec<String> = meta_monitor.patterns()
            .iter()
            .map(|p| format!("{:?}: {}", p.pattern_type, p.description))
            .collect();
        drop(meta_monitor);
        
        // Get predictive processing metrics
        let predictor = self.predictor.read().await;
        let prediction_accuracy = predictor.prediction_accuracy();
        let current_surprise = predictor.current_surprise();
        let learning_progress = predictor.total_learning();
        drop(predictor);
        
        // Get integrated information (Φ)
        let phi_calc = self.phi_calculator.read().await;
        let phi = phi_calc.phi();
        let consciousness_level = phi_calc.consciousness_level();
        drop(phi_calc);
        
        Ok(ConsciousResponse {
            answer: final_answer,
            internal_dialogue,
            ethos_weight,
            logos_weight,
            pathos_weight,
            confidence: 0.85, // TODO: Calculate from consensus
            checkpoint_insights,
            mental_state,
            awareness_level,
            detected_patterns,
            prediction_accuracy,
            current_surprise,
            learning_progress,
            phi,
            consciousness_level,
        })
    }
    
    /// Get moral/ethical perspective
    async fn get_ethos_perspective(&self, question: &str) -> Result<String> {
        let prompt = format!(
            "As an ethical advisor, evaluate this question from a MORAL perspective:\n\n\
            {}\n\n\
            Focus on: right vs wrong, fairness, justice, character, virtue.",
            question
        );
        
        let result = self.ethos_agent.think_and_respond(&prompt, None, None).await?;
        Ok(result.answer)
    }
    
    /// Get logical/rational perspective
    async fn get_logos_perspective(&self, question: &str) -> Result<String> {
        let prompt = format!(
            "As a logical analyst, evaluate this question from a RATIONAL perspective:\n\n\
            {}\n\n\
            Focus on: facts, evidence, logic, causality, coherence.",
            question
        );
        
        let result = self.logos_agent.think_and_respond(&prompt, None, None).await?;
        Ok(result.answer)
    }
    
    /// Get emotional/empathetic perspective
    async fn get_pathos_perspective(&self, question: &str) -> Result<String> {
        let prompt = format!(
            "As an empathetic counselor, evaluate this question from an EMOTIONAL perspective:\n\n\
            {}\n\n\
            Focus on: feelings, human impact, empathy, compassion, well-being.",
            question
        );
        
        let result = self.pathos_agent.think_and_respond(&prompt, None, None).await?;
        Ok(result.answer)
    }
    
    /// Agent responds to other perspectives
    async fn agent_responds_to_others(
        &self,
        agent_name: &str,
        own_view: &str,
        other1: &str,
        other2: &str,
    ) -> Result<String> {
        let prompt = format!(
            "You are the {} perspective. You said:\n{}\n\n\
            Others said:\n{}\n\n{}\n\n\
            Do you change your view? Respond briefly.",
            agent_name, own_view, other1, other2
        );
        
        let result = match agent_name {
            "Ethos" => self.ethos_agent.think_and_respond(&prompt, None, None).await?,
            "Logos" => self.logos_agent.think_and_respond(&prompt, None, None).await?,
            _ => self.pathos_agent.think_and_respond(&prompt, None, None).await?,
        };
        Ok(result.answer)
    }
    
    /// Sacred Checkpoint 3: Moral integration
    async fn sacred_checkpoint_3(
        &self,
        ethos: &str,
        logos: &str,
        pathos: &str,
    ) -> Result<String> {
        let prompt = format!(
            "SACRED CHECKPOINT 3 - Moral Integration:\n\n\
            Moral view: {}\n\
            Logical view: {}\n\
            Emotional view: {}\n\n\
            What is the ethical foundation we must preserve?",
            ethos, logos, pathos
        );
        
        let result = self.ethos_agent.think_and_respond(&prompt, None, None).await?;
        Ok(result.answer)
    }
    
    /// Sacred Checkpoint 6: Logical refinement
    async fn sacred_checkpoint_6(&self, ethos: &str, logos: &str) -> Result<String> {
        let prompt = format!(
            "SACRED CHECKPOINT 6 - Logical Refinement:\n\n\
            Refined moral view: {}\n\
            Refined logical view: {}\n\n\
            What is the logically sound path forward?",
            ethos, logos
        );
        
        let result = self.logos_agent.think_and_respond(&prompt, None, None).await?;
        Ok(result.answer)
    }
    
    /// Sacred Checkpoint 9: Divine synthesis
    async fn sacred_checkpoint_9(&self, synthesis: &str) -> Result<String> {
        let prompt = format!(
            "SACRED CHECKPOINT 9 - Divine Integration:\n\n\
            Synthesized understanding: {}\n\n\
            Provide the final, integrated answer that honors all perspectives.",
            synthesis
        );
        
        let result = self.ethos_agent.think_and_respond(&prompt, None, None).await?;
        Ok(result.answer)
    }
    
    /// Synthesize all perspectives
    async fn synthesize_perspectives(
        &self,
        ethos: &str,
        logos: &str,
        pathos: &str,
    ) -> Result<String> {
        let prompt = format!(
            "Synthesize these three perspectives into a unified understanding:\n\n\
            Moral: {}\n\n\
            Logical: {}\n\n\
            Emotional: {}\n\n\
            Create a balanced synthesis.",
            ethos, logos, pathos
        );
        
        let result = self.logos_agent.think_and_respond(&prompt, None, None).await?;
        Ok(result.answer)
    }
    
    /// Process thought through v1.4.0 consciousness components (with v1.5.0 streaming)
    async fn process_thought(&self, internal_thought: &InternalThought) {
        let start_time = std::time::Instant::now();
        
        // Convert InternalThought to Thought for v1.4.0 components
        let thought = Thought::new(
            internal_thought.thought.clone(),
            internal_thought.agent.clone(),
            ThoughtPriority::High, // Agent responses are high priority
        )
        .with_elp(
            internal_thought.elp_profile.0,
            internal_thought.elp_profile.1,
            internal_thought.elp_profile.2,
        )
        .with_confidence(0.8); // Default confidence for agent thoughts
        
        // Emit thought started event (v1.5.0)
        if let Some(ref streaming) = self.streaming_server {
            let broadcaster_arc = streaming.broadcaster();
            let broadcaster = broadcaster_arc.read().await;
            let preview = internal_thought.thought.chars().take(100).collect::<String>();
            let event = broadcaster.thought_started(internal_thought.agent.clone(), preview);
            drop(broadcaster); // Release lock before broadcasting
            let _ = streaming.broadcast(event).await;
        }
        
        // Meta-cognitive monitoring
        {
            let mut monitor = self.meta_monitor.write().await;
            monitor.observe_thought(&thought);
        }
        
        // Predictive processing
        {
            let mut predictor = self.predictor.write().await;
            let _surprise = predictor.observe_actual(&thought);
        }
        
        // Integrated information
        let phi_contribution = {
            let mut phi_calc = self.phi_calculator.write().await;
            let phi_before = phi_calc.phi();
            phi_calc.add_thought(thought.clone());
            phi_calc.phi() - phi_before
        };
        
        // Word-level tracking (v1.5.0)
        if let Some(ref streaming) = self.streaming_server {
            let words: Vec<&str> = internal_thought.thought.split_whitespace().collect();
            for word in words {
                let insights = create_word_insights(
                    word,
                    &internal_thought.agent,
                    internal_thought.elp_profile,
                    0.8,
                );
                let _ = streaming.add_word_insight(
                    word.to_string(),
                    internal_thought.agent.clone(),
                    insights,
                ).await;
            }
        }
        
        let processing_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Emit thought completed event (v1.5.0)
        if let Some(ref streaming) = self.streaming_server {
            let broadcaster_arc = streaming.broadcaster();
            let broadcaster = broadcaster_arc.read().await;
            let metrics = ThoughtMetrics {
                elp: internal_thought.elp_profile,
                confidence: 0.8,
                priority: "High".to_string(),
                source: internal_thought.agent.clone(),
                processing_time_ms,
                contribution_to_phi: phi_contribution,
            };
            let event = broadcaster.thought_completed(internal_thought.agent.clone(), metrics);
            drop(broadcaster); // Release lock before broadcasting
            let _ = streaming.broadcast(event).await;
        }
    }
    
    /// Calculate ELP weights from dialogue
    fn calculate_elp_weights(&self, dialogue: &[InternalThought]) -> (f64, f64, f64) {
        if dialogue.is_empty() {
            return (0.33, 0.33, 0.34);
        }
        
        let mut total_ethos = 0.0;
        let mut total_logos = 0.0;
        let mut total_pathos = 0.0;
        
        for thought in dialogue {
            total_ethos += thought.elp_profile.0;
            total_logos += thought.elp_profile.1;
            total_pathos += thought.elp_profile.2;
        }
        
        let sum = total_ethos + total_logos + total_pathos;
        (
            total_ethos / sum,
            total_logos / sum,
            total_pathos / sum,
        )
    }
}
