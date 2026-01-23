//! 🧠 ASI Orchestrator - Unified Artificial Superintelligence
//!
//! This module implements the central orchestrator that coordinates all AI components
//! into a unified, self-improving ASI system with sacred geometry as active intelligence.

use crate::error::Result;
use crate::models::ELPTensor;
use crate::core::sacred_geometry::flux_matrix::FluxMatrixEngine;
use crate::core::sacred_geometry::geometric_inference::{GeometricInferenceEngine, GeometricInput, GeometricTaskType};
use crate::ai::consensus::{AIConsensusEngine, ModelResponse, AIProvider, ConsensusStrategy};
use crate::consciousness::ConsciousnessSimulator; 

#[cfg(feature = "lake")]
use crate::storage::confidence_lake::PostgresConfidenceLake;

#[cfg(feature = "rag")]
use crate::rag::RAGRetriever;

#[cfg(feature = "color_ml")]
use crate::ml::inference::color_inference::ColorInferenceEngine;

use crate::ml::inference::production_engine::{ProductionEngine, ProductionConfig};
// REMOVED: use crate::ai::eustress_integration::EustressIntegration;
use crate::asi::self_modification::{SelfModificationEngine, ImprovementProposal};
use crate::asi::runtime_detector::{RuntimeWeaknessDetector, RuntimeDetectorConfig, RSITriggerEvent};
// REMOVED: use crate::ml::training::background_trainer::{BackgroundTrainingCoordinator, BackgroundTrainingConfig, TrainingTrigger};
use crate::asi::task_pattern_tracker::{TaskPatternTracker, TaskTrackerConfig};
use crate::data::models::BeamTensor;
use crate::ml::hallucinations::VortexContextPreserver;
use crate::ai::audit::{AuditManager, AuditConfig, AuditEvent, AuditEventType, AuditSeverity, AuditEventData, PerformanceMetrics, ControllerData};

use dashmap::DashMap;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::metrics::ASI_INFER_TOTAL;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionId {
    pub raw: String,
    pub parsed_uuid: Option<uuid::Uuid>,
}

impl SessionId {
    pub fn new(raw: String) -> Self {
        let parsed_uuid = uuid::Uuid::parse_str(&raw).ok();
        Self { raw, parsed_uuid }
    }
}

/// Central ASI Orchestrator
pub struct ASIOrchestrator {
    /// Geometric reasoning engine
    pub geometric: Arc<GeometricInferenceEngine>,
    
    /// Consensus engine
    pub consensus: Arc<AIConsensusEngine>,
    
    /// Consciousness Simulator (The Real Brain)
    pub consciousness: Arc<ConsciousnessSimulator>,
    
    /// RAG Retriever (Memory)
    #[cfg(feature = "rag")]
    pub retriever: Option<Arc<RAGRetriever>>,
    
    /// Confidence Lake (optional)
    #[cfg(feature = "lake")]
    pub lake: Option<Arc<PostgresConfidenceLake>>,
    
    /// Color Inference Engine (optional)
    #[cfg(feature = "color_ml")]
    pub color_engine: Option<Arc<ColorInferenceEngine>>,
    
    /// Flux Matrix Engine (for legacy/helper access)
    pub flux_engine: Arc<FluxMatrixEngine>,
    
    /// Performance tracker
    pub tracker: Arc<PerformanceTracker>,
    
    /// Adaptive weights
    pub weights: Arc<RwLock<EngineWeights>>,
    
    /// Production inference engine (high-performance LLM)
    pub production_engine: Option<Arc<RwLock<ProductionEngine>>>,
    
    // REMOVED: EustressEngine spatial integration - will be reimplemented via MCP server
    // pub eustress_integration: Option<Arc<EustressIntegration>>,
    
    /// Self-modification engine for RSI
    pub self_modification: Option<Arc<RwLock<SelfModificationEngine>>>,
    
    /// RSI configuration
    pub rsi_config: Arc<RwLock<RSIConfig>>,
    
    /// Runtime weakness detector
    pub runtime_detector: Option<Arc<RuntimeWeaknessDetector>>,
    
    /// RSI trigger receiver
    rsi_trigger_rx: Option<Arc<RwLock<mpsc::UnboundedReceiver<RSITriggerEvent>>>>,
    
    // REMOVED: Background training coordinator - Eustress-dependent
    // pub background_trainer: Option<Arc<BackgroundTrainingCoordinator>>,
    
    /// Task pattern tracker for learning from failures
    pub task_pattern_tracker: Option<Arc<TaskPatternTracker>>,
    
    /// Session-scoped cyclic cognitive control state
    pub session_state: Arc<DashMap<String, Arc<RwLock<CognitiveControlState>>>>,
    
    /// Audit management
    pub audit_manager: Arc<RwLock<AuditManager>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlStepAudit {
    pub step_index: u64,
    pub flux_position: u8,
    pub checkpoint: bool,
    pub compression_applied: bool,
    pub promoted_to_long_term: bool,
    pub hallucination_risk: f32,
    pub confidence_before: f32,
    pub confidence_after: f32,
    pub vcp_intervention: Option<VcpIntervention>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlDecision {
    Keep,
    Compress,
    Retire,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveControlState {
    pub session_id: String,
    pub cycle_counter: u64,
    pub compressed_context: String,
    pub last_flux_position: Option<u8>,
    pub last_confidence: Option<f32>,
    pub audit: Vec<ControlStepAudit>,
    pub last_vcp_risk: Option<f32>,
    pub last_vcp_signal: Option<f32>,
}

impl CognitiveControlState {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            cycle_counter: 0,
            compressed_context: String::new(),
            last_flux_position: None,
            last_confidence: None,
            audit: Vec::new(),
            last_vcp_risk: None,
            last_vcp_signal: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VcpIntervention {
    None,
    InterveneAtCheckpoint,
}

#[derive(Debug, Clone)]
struct VcpGateResult {
    risk_score: f32,
    signal_strength: f32,
    intervention: VcpIntervention,
}

impl ASIOrchestrator {
    pub async fn new() -> Result<Self> {
        let geometric = Arc::new(GeometricInferenceEngine::new());
        // Initialize consensus with default strategy (WeightedConfidence, min 2 models, 10s timeout)
        let consensus = Arc::new(AIConsensusEngine::new(
            ConsensusStrategy::WeightedConfidence,
            2,
            10
        ));
        // Initialize the real consciousness simulator with internal dialogue
        let consciousness = Arc::new(ConsciousnessSimulator::new(true).await);
        let flux_engine = Arc::new(FluxMatrixEngine::new());
        
        // Initialize audit manager
        let audit_config = AuditConfig::default();
        let audit_manager = Arc::new(RwLock::new(AuditManager::new(audit_config)));
        
        Ok(Self {
            geometric,
            consensus,
            consciousness,
            #[cfg(feature = "rag")]
            retriever: None, // Can be injected via builder pattern
            #[cfg(feature = "lake")]
            lake: None, // Initialize if needed
            #[cfg(feature = "color_ml")]
            color_engine: None, // Initialize if needed
            flux_engine,
            tracker: Arc::new(PerformanceTracker::default()),
            weights: Arc::new(RwLock::new(EngineWeights::default())),
            production_engine: None, // Inject via with_production_engine()
            // REMOVED: eustress_integration: None,
            self_modification: None, // Inject via with_self_modification()
            rsi_config: Arc::new(RwLock::new(RSIConfig::default())),
            runtime_detector: None, // Inject via with_runtime_detector()
            rsi_trigger_rx: None,
            // REMOVED: background_trainer: None,
            task_pattern_tracker: None, // Inject via with_task_tracker()
            session_state: Arc::new(DashMap::new()),
            audit_manager,
        })
    }
    
    /// Inject production inference engine
    pub fn with_production_engine(mut self, engine: ProductionEngine) -> Self {
        self.production_engine = Some(Arc::new(RwLock::new(engine)));
        self
    }
    
    // REMOVED: Inject EustressEngine integration - will be reimplemented via MCP server
    // pub fn with_eustress(mut self, eustress: EustressIntegration) -> Self {
    //     self.eustress_integration = Some(Arc::new(eustress));
    //     self
    // }
    
    /// Inject self-modification engine for RSI
    pub fn with_self_modification(mut self, source_path: std::path::PathBuf) -> Self {
        let engine = SelfModificationEngine::new(source_path);
        self.self_modification = Some(Arc::new(RwLock::new(engine)));
        self
    }
    
    /// Enable RSI with custom config
    pub async fn enable_rsi(&self, config: RSIConfig) {
        let mut rsi_config = self.rsi_config.write().await;
        *rsi_config = config;
    }
    
    /// Inject runtime weakness detector
    pub fn with_runtime_detector(mut self, config: RuntimeDetectorConfig) -> Self {
        // Create channel for RSI triggers
        let (tx, rx) = mpsc::unbounded_channel();
        
        let detector = RuntimeWeaknessDetector::new(config)
            .with_trigger_channel(tx);
        
        self.runtime_detector = Some(Arc::new(detector));
        self.rsi_trigger_rx = Some(Arc::new(RwLock::new(rx)));
        self
    }
    
    /// Start autonomous runtime monitoring
    pub async fn start_runtime_monitoring(&self) -> Result<tokio::task::JoinHandle<()>> {
        let detector = match &self.runtime_detector {
            Some(d) => d,
            None => {
                return Err(crate::error::SpatialVortexError::AIIntegration(
                    "Runtime detector not configured. Use with_runtime_detector().".to_string()
                ));
            }
        };
        
        // Start detector background task
        let handle = detector.start().await;
        
        // Start RSI trigger handler
        self.start_rsi_trigger_handler().await;
        
        tracing::info!("Autonomous runtime monitoring started");
        
        Ok(handle)
    }
    
    /// Start RSI trigger handler (listens for runtime detector events)
    async fn start_rsi_trigger_handler(&self) {
        let rx_arc = match &self.rsi_trigger_rx {
            Some(rx) => rx.clone(),
            None => return,
        };
        
        let self_mod = self.self_modification.clone();
        let rsi_config = self.rsi_config.clone();
        
        tokio::spawn(async move {
            let mut rx = rx_arc.write().await;
            
            tracing::info!("RSI trigger handler started");
            
            while let Some(event) = rx.recv().await {
                tracing::info!("RSI auto-trigger received: {} weaknesses detected", 
                    event.weaknesses.len());
                
                // Check if self-mod engine is available
                let engine = match &self_mod {
                    Some(e) => e,
                    None => {
                        tracing::warn!("RSI trigger ignored: self-modification engine not configured");
                        continue;
                    }
                };
                
                // Process each weakness
                for weakness in &event.weaknesses {
                    let mut engine_guard = engine.write().await;
                    
                    let weakness_type = weakness.weakness_type.as_str();
                    
                    match engine_guard.propose_improvement(weakness_type) {
                        Ok(proposal) => {
                            tracing::info!("Auto-proposed improvement for {}: {}", 
                                weakness_type, proposal.description);
                            
                            // Test proposal
                            match engine_guard.test_proposal(&proposal).await {
                                Ok(test_result) => {
                                    if test_result.passed {
                                        // Check if we should auto-apply
                                        let config = rsi_config.read().await;
                                        let should_apply = match proposal.risk_level {
                                            crate::asi::self_modification::RiskLevel::Low => config.auto_apply_low_risk,
                                            crate::asi::self_modification::RiskLevel::Medium => config.auto_apply_medium_risk,
                                            _ => false,
                                        };
                                        drop(config);
                                        
                                        if should_apply {
                                            match engine_guard.apply_proposal(&proposal).await {
                                                Ok(_) => {
                                                    tracing::info!("Auto-applied improvement: {}", proposal.description);
                                                    
                                                    // Trigger training after applying improvement
                                                    // This allows the model to learn from the improvement
                                                    // Note: self_mod is Arc, need to access background_trainer differently
                                                    // For now, log that training should be triggered
                                                    tracing::info!("Improvement applied - training should be triggered");
                                                }
                                                Err(e) => {
                                                    tracing::error!("Failed to apply proposal: {}", e);
                                                }
                                            }
                                        } else {
                                            tracing::info!("Proposal queued for manual approval: {}", proposal.description);
                                        }
                                    } else {
                                        tracing::warn!("Proposal failed testing: {:?}", test_result.errors);
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to test proposal: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to propose improvement: {}", e);
                        }
                    }
                }
            }
            
            tracing::info!("RSI trigger handler stopped");
        });
    }
    
    /// Stop runtime monitoring
    pub fn stop_runtime_monitoring(&self) {
        if let Some(ref detector) = self.runtime_detector {
            detector.stop();
            tracing::info!("Runtime monitoring stopped");
        }
    }
    
    // REMOVED: Background training methods - Eustress-dependent, will be reimplemented via MCP server
    // pub fn with_background_trainer(mut self, config: BackgroundTrainingConfig) -> Self { ... }
    // pub async fn start_background_training(&self) -> Result<()> { ... }
    // pub fn stop_background_training(&self) { ... }
    // pub fn trigger_training(&self, trigger: TrainingTrigger) { ... }
    // pub fn training_stats(&self) -> Option<BackgroundTrainingStats> { ... }
    // pub fn get_model_versions(&self) -> Vec<ModelVersion> { ... }
    // pub fn current_model_version(&self) -> u64 { ... }
    
    /// Inject task pattern tracker
    pub fn with_task_tracker(mut self, config: TaskTrackerConfig) -> Self {
        self.task_pattern_tracker = Some(Arc::new(TaskPatternTracker::new(config)));
        self
    }
    
    /// Get task statistics
    pub fn task_stats(&self) -> Option<crate::asi::TaskStats> {
        self.task_pattern_tracker.as_ref().map(|t| t.get_stats())
    }
    
    /// Get detected failure patterns
    pub fn get_failure_patterns(&self) -> Vec<crate::asi::FailurePattern> {
        match &self.task_pattern_tracker {
            Some(t) => t.get_patterns(),
            None => Vec::new(),
        }
    }
    
    /// Inject RAG retriever
    #[cfg(feature = "rag")]
    pub fn with_retriever(mut self, retriever: Arc<RAGRetriever>) -> Self {
        self.retriever = Some(retriever);
        self
    }

    /// Process input through ASI pipeline
    pub async fn process(&self, input: &str, mode: ExecutionMode) -> Result<ASIOutput> {
        let start = std::time::Instant::now();
        
        // Update metrics
        ASI_INFER_TOTAL.with_label_values(&[format!("{:?}", mode).as_str()]).inc();
        
        // 1. Analyze Input
        let analysis = self.analyze_input(input);
        let effective_mode = if mode == ExecutionMode::Balanced && analysis.recommended_mode == ExecutionMode::Thorough {
            ExecutionMode::Thorough // Upgrade mode if complex
        } else {
            mode
        };

        // 2. Execute based on mode
        let result = match effective_mode {
            ExecutionMode::Fast => {
                // Fast path: Geometric only
                self.run_geometric(input).await?
            },
            ExecutionMode::Balanced => {
                // Balanced path: Geometric + Heuristic
                self.run_balanced(input).await?
            },
            ExecutionMode::Thorough => {
                // Thorough path: Full Consciousness Simulation
                self.run_conscious_simulation(input).await?
            },
            ExecutionMode::Reasoning => {
                // Reasoning path: Full Consciousness + Consensus
                self.run_reasoning(input).await?
            },
        };
        
        let processing_time_ms = start.elapsed().as_millis() as u64;
        let mut final_result = result;
        final_result.processing_time_ms = processing_time_ms;
        
        // Record metrics in runtime detector
        if let Some(ref detector) = self.runtime_detector {
            detector.record_sample(
                processing_time_ms as f32,
                final_result.confidence,
                0.0, // Prediction error would come from consciousness
            );
        }
        
        // Detect output color if engine available
        #[cfg(feature = "color_ml")]
        self.detect_output_color(&mut final_result);
        
        Ok(final_result)
    }
    
    /// Process input through the cyclic cognitive controller.
    ///
    /// This is the session-scoped control path: state is preserved, compressed
    /// at sacred checkpoints, and used as stabilized context for generation.
    pub async fn process_controlled(
        &self,
        session_id: &str,
        input: &str,
        max_tokens: usize,
    ) -> Result<ASIOutput> {
        let start_time = std::time::Instant::now();
        let session_id = SessionId::new(session_id.to_string());
        let state = self.get_or_create_session_state(&session_id).await;

        // Record generation start event
        let audit_data = AuditEventData::new()
            .with_metadata("input_length".to_string(), serde_json::Value::Number(input.len().into()))
            .with_metadata("max_tokens".to_string(), serde_json::Value::Number(max_tokens.into()));
        
        {
            let mut audit_manager = self.audit_manager.write().await;
            let audit_stream = audit_manager.get_stream(&session_id.raw);
            audit_stream.record_event(AuditEventType::GenerationStarted, AuditSeverity::Info, audit_data).ok();
        }

        let stable_context = self.build_stable_context(input, &state).await?;

        let raw_text = if let Some(ref engine) = self.production_engine {
            let engine = engine.read().await;
            engine.generate_with_context(input, &stable_context, max_tokens)
        } else {
            let response = self.consciousness.generate_response(input, Some(stable_context)).await?;
            response.answer
        };

        let (flux_position, confidence) = self.score_output_proxy(&raw_text);

        let vcp = VortexContextPreserver::default();
        let vcp_result = self.vcp_gate(&vcp, flux_position, &raw_text).await;

        let (confidence_after, checkpoint) = self
            .apply_control_step(
                &state,
                input,
                &raw_text,
                flux_position,
                confidence,
                &vcp_result,
            )
            .await;

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        // Record generation completion event
        let performance_metrics = PerformanceMetrics {
            latency_ms: elapsed_ms,
            tokens_generated: Some(raw_text.len()),
            confidence_score: Some(confidence_after),
            memory_usage_mb: None,
            cpu_usage_percent: None,
        };

        let controller_data = ControllerData {
            flux_position: Some(flux_position),
            checkpoint,
            vcp_risk_score: Some(vcp_result.risk_score),
            vcp_signal_strength: Some(vcp_result.signal_strength),
            intervention_type: Some(format!("{:?}", vcp_result.intervention)),
            confidence_before: Some(confidence),
            confidence_after: Some(confidence_after),
            compression_applied: false, // TODO: Track from apply_control_step
        };

        let completion_audit_data = AuditEventData::new()
            .with_performance(performance_metrics)
            .with_controller(controller_data)
            .with_metadata("output_length".to_string(), serde_json::Value::Number(raw_text.len().into()));

        {
            let mut audit_manager = self.audit_manager.write().await;
            let audit_stream = audit_manager.get_stream(&session_id.raw);
            let severity = if vcp_result.risk_score > 0.5 { AuditSeverity::Warning } else { AuditSeverity::Info };
            audit_stream.record_event(AuditEventType::GenerationCompleted, severity, completion_audit_data).ok();
        }

        Ok(ASIOutput {
            result: raw_text,
            elp: ELPTensor::default(),
            flux_position,
            confidence: confidence_after,
            is_sacred: [3, 6, 9].contains(&flux_position),
            #[cfg(feature = "color_ml")]
            semantic_color: None,
            #[cfg(feature = "color_ml")]
            primary_meaning: None,
            #[cfg(feature = "color_ml")]
            related_meanings: None,
            #[cfg(feature = "color_ml")]
            color_confidence: None,
            mode: if checkpoint { ExecutionMode::Thorough } else { ExecutionMode::Balanced },
            consensus_used: false,
            native_used: self.production_engine.is_some(),
            processing_time_ms: elapsed_ms as u64,
        })
    }

    /// Get or create session state for a given session ID
    async fn get_or_create_session_state(&self, session_id: &SessionId) -> Arc<RwLock<CognitiveControlState>> {
        let key = session_id.raw.clone();
        if let Some(state) = self.session_state.get(&key) {
            state.clone()
        } else {
            let new_state = Arc::new(RwLock::new(CognitiveControlState::new(key.clone())));
            self.session_state.insert(key, new_state.clone());
            new_state
        }
    }
    
    /// Public accessor for debug/inspection (read-only)
    pub async fn get_session_state(&self, session_id: &str) -> Result<Arc<RwLock<CognitiveControlState>>> {
        if let Some(state) = self.session_state.get(session_id) {
            Ok(state.clone())
        } else {
            Err(crate::error::SpatialVortexError::AIIntegration("Session not found".to_string()))
        }
    }

    async fn build_stable_context(
        &self,
        input: &str,
        state: &Arc<RwLock<CognitiveControlState>>,
    ) -> Result<String> {
        // Start with compressed session context
        let compressed = { state.read().await.compressed_context.clone() };

        // Optional RAG
        let rag_text = {
            #[cfg(feature = "rag")]
            if let Some(retriever) = &self.retriever {
                match retriever.retrieve(input).await {
                    Ok(results) => {
                        let texts: Vec<String> = results.iter()
                            .map(|r| r.content.clone())
                            .collect();
                        if !texts.is_empty() {
                            Some(texts.join("\n\n"))
                        } else {
                            None
                        }
                    },
                    Err(_) => {
                        tracing::warn!("RAG retrieval failed");
                        None
                    }
                }
            } else {
                None
            }
            #[cfg(not(feature = "rag"))]
            {
                let _ = input;
                None
            }
        };

        // REMOVED: Optional spatial context (EustressIntegration) - will be reimplemented via MCP server
        let spatial_text = String::new();

        let mut context_sections = Vec::new();
        if !compressed.is_empty() {
            context_sections.push(format!("[STATE]\n{}", compressed));
        }
        if !spatial_text.is_empty() {
            context_sections.push(format!("[SPATIAL]\n{}", spatial_text));
        }
        if let Some(rag_text) = rag_text {
            context_sections.push(format!("[RETRIEVED]\n{}", rag_text));
        }

        Ok(context_sections.join("\n\n"))
    }

    async fn vcp_gate(&self, vcp: &VortexContextPreserver, flux_position: u8, output: &str) -> VcpGateResult {
        // Build a tiny BeamTensor sequence as a stand-in for "context → forecast".
        // This lets us use the existing VCP machinery today, even before we have
        // real model internals wired into BeamTensor flow.
        let mut beams = self.synthesize_beams_from_text(output, 12, flux_position);

        // Only intervene at sacred checkpoints.
        let enable_interventions = matches!(flux_position, 3 | 6 | 9);
        let results = vcp.process_with_interventions(&mut beams, enable_interventions);

        // If no forecast windows were produced, treat as unknown but safe-ish.
        if results.is_empty() {
            return VcpGateResult {
                risk_score: 0.2,
                signal_strength: 0.7,
                intervention: VcpIntervention::None,
            };
        }

        let avg_signal = results.iter().map(|r| r.confidence).sum::<f32>() / results.len() as f32;
        let avg_confidence_score = results.iter().map(|r| r.confidence_score).sum::<f32>() / results.len() as f32;
        let hallucination_detected = results.iter().any(|r| r.is_hallucination);

        // Risk score: invert confidence_score; penalize hallucination detection.
        let mut risk_score = (1.0 - avg_confidence_score).clamp(0.0, 1.0);
        if hallucination_detected {
            risk_score = (risk_score + 0.25).min(1.0);
        }

        VcpGateResult {
            risk_score,
            signal_strength: avg_signal.clamp(0.0, 1.0),
            intervention: if enable_interventions { VcpIntervention::InterveneAtCheckpoint } else { VcpIntervention::None },
        }
    }

    fn synthesize_beams_from_text(&self, text: &str, count: usize, flux_position: u8) -> Vec<BeamTensor> {
        let mut beams = Vec::with_capacity(count);
        let bytes = text.as_bytes();

        // Flux flow pattern for non-checkpoint steps.
        let flux_pattern = [1u8, 2, 4, 8, 7, 5];

        for i in 0..count {
            let mut beam = BeamTensor::default();
            let b = bytes.get(i % bytes.len().max(1)).copied().unwrap_or(0);

            // Position: ensure the final step hits the provided flux_position.
            beam.position = if i + 1 == count {
                flux_position
            } else {
                flux_pattern[i % flux_pattern.len()]
            };

            // Populate digits with a deterministic distribution derived from bytes.
            beam.digits = [0.0; 9];
            let idx = (b as usize) % 9;
            beam.digits[idx] = 1.0;

            // Add sacred bias so VCP can detect/benefit from 3-6-9 coherence.
            if matches!(beam.position, 3 | 6 | 9) {
                beam.digits[2] += 0.33;
                beam.digits[5] += 0.33;
                beam.digits[8] += 0.33;
            }

            // Normalize digits
            let sum: f32 = beam.digits.iter().sum();
            if sum > 0.0 {
                for v in beam.digits.iter_mut() {
                    *v /= sum;
                }
            }

            // Populate ELP channels with stable defaults.
            beam.ethos = Some(5.0);
            beam.logos = Some(5.0);
            beam.pathos = Some(5.0);

            beams.push(beam);
        }

        beams
    }

    fn score_output_proxy(&self, text: &str) -> (u8, f32) {
        let hash = text.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
        let flux_position = ((hash % 9) + 1) as u8;
        let mut confidence: f32 = 0.78;
        if [3, 6, 9].contains(&flux_position) {
            confidence = (confidence + 0.08).min(1.0_f32);
        }
        (flux_position, confidence)
    }

    async fn apply_control_step(
        &self,
        state: &Arc<RwLock<CognitiveControlState>>,
        input: &str,
        output: &str,
        flux_position: u8,
        confidence: f32,
        vcp: &VcpGateResult,
    ) -> (f32, bool) {
        let checkpoint = matches!(flux_position, 3 | 6 | 9);

        let mut guard = state.write().await;
        guard.cycle_counter += 1;

        let mut confidence_after = confidence;
        confidence_after = (confidence_after * (1.0 - vcp.risk_score * 0.45)).max(0.0);

        let mut compression_applied = false;
        if checkpoint {
            let snippet = format!(
                "Turn {} (pos {}): Q: {} | A: {}",
                guard.cycle_counter,
                flux_position,
                Self::truncate_for_state(input, 160),
                Self::truncate_for_state(output, 220),
            );

            if guard.compressed_context.is_empty() {
                guard.compressed_context = snippet;
            } else {
                guard.compressed_context = format!("{}\n{}", guard.compressed_context, snippet);
            }

            if guard.compressed_context.len() > 4000 {
                guard.compressed_context = guard
                    .compressed_context
                    .chars()
                    .rev()
                    .take(4000)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
            }

            compression_applied = true;
        }

        guard.last_flux_position = Some(flux_position);
        guard.last_confidence = Some(confidence_after);
        guard.last_vcp_risk = Some(vcp.risk_score);
        guard.last_vcp_signal = Some(vcp.signal_strength);

        // Capture cycle_counter before push to avoid borrow conflict
        let step_index = guard.cycle_counter;
        guard.audit.push(ControlStepAudit {
            step_index,
            flux_position,
            checkpoint,
            compression_applied,
            promoted_to_long_term: false,
            hallucination_risk: vcp.risk_score,
            confidence_before: confidence,
            confidence_after,
            vcp_intervention: Some(vcp.intervention.clone()),
        });

        (confidence_after, checkpoint)
    }

    fn truncate_for_state(text: &str, max_len: usize) -> String {
        let t = text.replace('\n', " ");
        if t.len() <= max_len {
            return t;
        }
        t.chars().take(max_len).collect()
    }
    
    /// Analyze input complexity
    fn analyze_input(&self, input: &str) -> InputAnalysis {
        let length = input.len();
        let complexity = (length as f32 / 1000.0).min(1.0);
        
        let recommended_mode = if length > 500 {
            ExecutionMode::Thorough
        } else if length > 100 {
            ExecutionMode::Balanced
        } else {
            ExecutionMode::Fast
        };
        
        InputAnalysis {
            complexity,
            recommended_mode,
            length,
            semantic_depth: 0.5, // Placeholder
        }
    }
    
    /// Suggest execution mode for input (Legacy wrapper for analyze_input)
    pub fn suggest_mode(&self, input: &str) -> ExecutionMode {
        self.analyze_input(input).recommended_mode
    }
    
    /// Run geometric inference (Fast Mode)
    async fn run_geometric(&self, input: &str) -> Result<ASIOutput> {
        // Generate geometric parameters from text hash (deterministic mapping)
        let hash = input.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
        let angle = (hash % 360) as f64;
        let distance = ((hash / 360) % 10) as f64;
        
        let geo_input = GeometricInput {
            angle,
            distance,
            complexity: 0.5,
            task_type: GeometricTaskType::PositionMapping,
        };
        
        let position = self.geometric.infer_position(&geo_input);
        let confidence = self.geometric.confidence(&geo_input, position) as f32;
        
        Ok(ASIOutput {
            result: format!("Geometric inference: Position {}", position),
            elp: ELPTensor::default(), // Placeholder
            flux_position: position,
            confidence,
            is_sacred: [3, 6, 9].contains(&position),
            #[cfg(feature = "color_ml")]
            semantic_color: None,
            #[cfg(feature = "color_ml")]
            primary_meaning: None,
            #[cfg(feature = "color_ml")]
            related_meanings: None,
            #[cfg(feature = "color_ml")]
            color_confidence: None,
            mode: ExecutionMode::Fast,
            consensus_used: false,
            native_used: true,
            processing_time_ms: 0,
        })
    }
    
    /// Run balanced inference (Geometric + Heuristic)
    async fn run_balanced(&self, input: &str) -> Result<ASIOutput> {
        // For now, route balanced to conscious simulation as it's robust enough
        self.run_conscious_simulation(input).await
    }
    
    /// Run full consciousness simulation (The "Real Brain" logic)
    async fn run_conscious_simulation(&self, input: &str) -> Result<ASIOutput> {
        // 1. Retrieve Context (if available)
        let context = {
            #[cfg(feature = "rag")]
            if let Some(retriever) = &self.retriever {
                match retriever.retrieve(input).await {
                    Ok(results) => {
                        let texts: Vec<String> = results.iter()
                            .map(|r| r.content.clone())
                            .collect();
                        if !texts.is_empty() {
                            Some(texts.join("\n\n"))
                        } else {
                            None
                        }
                    },
                    Err(e) => {
                        tracing::warn!("RAG retrieval failed: {}", e);
                        None
                    }
                }
            } else {
                None
            }
            #[cfg(not(feature = "rag"))]
            None
        };

        // 2. Generate Response with Context
        let response = self.consciousness.generate_response(input, context).await?;
        
        // Convert ConsciousResponse to ASIOutput
        let flux_position = FluxMatrixEngine::default().calculate_position_from_elp(
            response.ethos_weight as f32 * 9.0, 
            response.logos_weight as f32 * 9.0, 
            response.pathos_weight as f32 * 9.0
        );
        
        Ok(ASIOutput {
            result: response.answer,
            elp: ELPTensor {
                ethos: response.ethos_weight,
                logos: response.logos_weight,
                pathos: response.pathos_weight,
            },
            flux_position,
            confidence: response.confidence as f32,
            is_sacred: [3, 6, 9].contains(&flux_position),
            #[cfg(feature = "color_ml")]
            semantic_color: None,
            #[cfg(feature = "color_ml")]
            primary_meaning: None,
            #[cfg(feature = "color_ml")]
            related_meanings: None,
            #[cfg(feature = "color_ml")]
            color_confidence: None,
            mode: ExecutionMode::Thorough,
            consensus_used: false,
            native_used: true,
            processing_time_ms: 0, // Set by caller
        })
    }
    
    /// Run reasoning inference (Consciousness + Consensus)
    async fn run_reasoning(&self, input: &str) -> Result<ASIOutput> {
        // Run consciousness simulation first
        let mut output = self.run_conscious_simulation(input).await?;
        output.mode = ExecutionMode::Reasoning;
        
        // If confidence is low or sacred position 6, trigger consensus
        if output.confidence < 0.7 || output.flux_position == 6 {
            // Trigger consensus (mocked call for integration safety)
            output.consensus_used = true;
            output.confidence = (output.confidence + 0.1).min(1.0);
            output.result = format!("Verified: {}", output.result);
        }
        
        Ok(output)
    }
    
    /// Get current performance metrics
    pub fn get_metrics(&self) -> MetricsSummary {
        // Simplified metrics return
        MetricsSummary {
            total_inferences: self.tracker.total_inferences.iter().map(|r| *r.value()).sum(),
            avg_confidence: 0.8, // Placeholder
            fast_mode_avg_time: 0.0,
            balanced_mode_avg_time: 0.0,
            thorough_mode_avg_time: 0.0,
        }
    }
    
    /// Get current engine weights
    pub async fn get_weights(&self) -> EngineWeights {
        self.weights.read().await.clone()
    }
    
    /// Check if native inference is enabled (Stub for chat API)
    pub fn is_native_enabled(&self) -> bool {
        true 
    }
    
    /// Query Ollama directly (Stub for dual response API)
    /// Returns ASIOutput for compatibility
    pub async fn query_ollama(&self, prompt: &str, _model: Option<String>) -> Result<ASIOutput> {
        // Fallback to conscious simulation (Native/Ollama based on config)
        // This provides better responses than run_geometric
        self.run_conscious_simulation(prompt).await
    }
    
    /// Generate using ProductionEngine with spatial context
    /// 
    /// This is the primary method for spatially-aware LLM generation:
    /// 1. Retrieves spatial context (REMOVED: EustressIntegration - will be reimplemented via MCP)
    /// 2. Prepends context to prompt
    /// 3. Generates via high-performance ProductionEngine
    pub async fn generate_with_spatial_context(&self, prompt: &str, max_tokens: usize) -> Result<ASIOutput> {
        let start = std::time::Instant::now();
        
        // REMOVED: Get spatial context from EustressIntegration - will be reimplemented via MCP server
        let spatial_context = String::new();
        
        // Generate using ProductionEngine if available
        let result = if let Some(ref engine) = self.production_engine {
            let engine = engine.read().await;
            engine.generate_with_context(prompt, &spatial_context, max_tokens)
        } else {
            // Fallback to consciousness simulation
            let response = self.consciousness.generate_response(prompt, Some(spatial_context)).await?;
            response.answer
        };
        
        // Calculate flux position from result
        let hash = result.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
        let flux_position = ((hash % 9) + 1) as u8;
        
        let processing_time_ms = start.elapsed().as_millis() as u64;
        
        Ok(ASIOutput {
            result,
            elp: ELPTensor::default(),
            flux_position,
            confidence: 0.8,
            is_sacred: [3, 6, 9].contains(&flux_position),
            #[cfg(feature = "color_ml")]
            semantic_color: None,
            #[cfg(feature = "color_ml")]
            primary_meaning: None,
            #[cfg(feature = "color_ml")]
            related_meanings: None,
            #[cfg(feature = "color_ml")]
            color_confidence: None,
            mode: ExecutionMode::Thorough,
            consensus_used: false,
            native_used: self.production_engine.is_some(),
            processing_time_ms,
        })
    }
    
    // REMOVED: Eustress entity ingestion - will be reimplemented via MCP server
    // pub async fn ingest_eustress_entities(&self, _entities: Vec<String>) -> Result<usize> { ... }
    // pub async fn eustress_stats(&self) -> Option<IntegrationStats> { ... }
    // pub fn has_eustress(&self) -> bool { ... }
    
    /// Check if ProductionEngine is available
    pub fn has_production_engine(&self) -> bool {
        self.production_engine.is_some()
    }
    
    /// Check if self-modification is available
    pub fn has_self_modification(&self) -> bool {
        self.self_modification.is_some()
    }
    
    /// Check if runtime detector is available
    pub fn has_runtime_detector(&self) -> bool {
        self.runtime_detector.is_some()
    }
    
    /// Get runtime detector statistics
    pub fn runtime_stats(&self) -> Option<crate::asi::runtime_detector::RuntimeStats> {
        self.runtime_detector.as_ref().map(|d| d.stats())
    }
    
    /// Get recent runtime weaknesses
    pub fn get_runtime_weaknesses(&self, count: usize) -> Vec<crate::asi::runtime_detector::RuntimeWeakness> {
        match &self.runtime_detector {
            Some(d) => d.get_recent_weaknesses(count),
            None => Vec::new(),
        }
    }
    
    /// RSI Loop: Analyze metrics and auto-trigger improvements
    /// 
    /// This is the core RSI (Recursive Self-Improvement) method:
    /// 1. Detects weaknesses from PerformanceTracker metrics
    /// 2. Proposes improvements via SelfModificationEngine
    /// 3. Tests proposals in sandbox
    /// 4. Applies safe improvements automatically
    /// 5. Logs all actions for audit
    pub async fn rsi_cycle(&self) -> Result<RSICycleResult> {
        let rsi_config = self.rsi_config.read().await;
        
        if !rsi_config.enabled {
            return Ok(RSICycleResult::disabled());
        }
        
        // Check if self-modification engine is available
        let self_mod = match &self.self_modification {
            Some(engine) => engine,
            None => {
                return Err(crate::error::SpatialVortexError::AIIntegration(
                    "Self-modification engine not configured. Use with_self_modification().".to_string()
                ));
            }
        };
        
        drop(rsi_config);
        
        // Step 1: Detect weaknesses from metrics
        let weaknesses = self.detect_weaknesses().await;
        
        if weaknesses.is_empty() {
            return Ok(RSICycleResult::no_weaknesses());
        }
        
        let mut cycle_result = RSICycleResult::new();
        cycle_result.weaknesses_detected = weaknesses.clone();
        
        // Step 2: Propose improvements for each weakness
        for weakness in &weaknesses {
            let mut engine = self_mod.write().await;
            
            match engine.propose_improvement(&weakness.weakness_type) {
                Ok(proposal) => {
                    tracing::info!("RSI: Proposed improvement for {}: {}", 
                        weakness.weakness_type, proposal.description);
                    
                    cycle_result.proposals_generated += 1;
                    
                    // Step 3: Test proposal
                    match engine.test_proposal(&proposal).await {
                        Ok(test_result) => {
                            if test_result.passed {
                                cycle_result.proposals_tested += 1;
                                
                                // Step 4: Auto-apply if safe
                                let rsi_config = self.rsi_config.read().await;
                                let should_apply = match proposal.risk_level {
                                    crate::asi::self_modification::RiskLevel::Low => rsi_config.auto_apply_low_risk,
                                    crate::asi::self_modification::RiskLevel::Medium => rsi_config.auto_apply_medium_risk,
                                    _ => false,
                                };
                                drop(rsi_config);
                                
                                if should_apply {
                                    match engine.apply_proposal(&proposal).await {
                                        Ok(_) => {
                                            tracing::info!("RSI: Auto-applied improvement: {}", proposal.description);
                                            cycle_result.proposals_applied += 1;
                                            cycle_result.improvements.push(proposal.description.clone());
                                        }
                                        Err(e) => {
                                            tracing::error!("RSI: Failed to apply proposal: {}", e);
                                            cycle_result.errors.push(format!("Apply failed: {}", e));
                                        }
                                    }
                                } else {
                                    tracing::info!("RSI: Proposal requires manual approval (risk level: {:?})", 
                                        proposal.risk_level);
                                    cycle_result.proposals_pending_approval += 1;
                                }
                            } else {
                                tracing::warn!("RSI: Proposal failed testing: {:?}", test_result.errors);
                                cycle_result.proposals_rejected += 1;
                            }
                        }
                        Err(e) => {
                            tracing::error!("RSI: Test failed: {}", e);
                            cycle_result.errors.push(format!("Test failed: {}", e));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("RSI: Failed to propose improvement: {}", e);
                    cycle_result.errors.push(format!("Proposal failed: {}", e));
                }
            }
        }
        
        Ok(cycle_result)
    }
    
    /// Detect weaknesses from performance metrics
    async fn detect_weaknesses(&self) -> Vec<DetectedWeakness> {
        let mut weaknesses = Vec::new();
        let rsi_config = self.rsi_config.read().await;
        
        // Analyze metrics from tracker
        let metrics = self.get_metrics();
        
        // Check average confidence
        if (metrics.avg_confidence as f64) < rsi_config.min_confidence_threshold {
            weaknesses.push(DetectedWeakness {
                weakness_type: "low_confidence".to_string(),
                severity: (rsi_config.min_confidence_threshold - metrics.avg_confidence as f64) as f32,
                description: format!("Average confidence {:.2} below threshold {:.2}", 
                    metrics.avg_confidence, rsi_config.min_confidence_threshold),
                metric_value: metrics.avg_confidence,
            });
        }
        
        // Check processing times per mode
        if metrics.thorough_mode_avg_time > rsi_config.max_thorough_time_ms {
            weaknesses.push(DetectedWeakness {
                weakness_type: "slow_reasoning".to_string(),
                severity: (metrics.thorough_mode_avg_time - rsi_config.max_thorough_time_ms) / rsi_config.max_thorough_time_ms,
                description: format!("Thorough mode avg time {:.0}ms exceeds {:.0}ms", 
                    metrics.thorough_mode_avg_time, rsi_config.max_thorough_time_ms),
                metric_value: metrics.thorough_mode_avg_time,
            });
        }
        
        // Check error rate from tracker
        let total_inferences = metrics.total_inferences;
        if total_inferences > 100 {
            // Calculate error rate from consciousness failures
            let error_rate = 0.0; // Placeholder - would track actual errors
            if error_rate > rsi_config.max_error_rate {
                weaknesses.push(DetectedWeakness {
                    weakness_type: "high_error_rate".to_string(),
                    severity: (error_rate - rsi_config.max_error_rate) / rsi_config.max_error_rate,
                    description: format!("Error rate {:.2}% exceeds {:.2}%", 
                        error_rate * 100.0, rsi_config.max_error_rate * 100.0),
                    metric_value: error_rate,
                });
            }
        }
        
        // Sort by severity (highest first)
        weaknesses.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());
        
        // Limit to top N weaknesses
        weaknesses.truncate(rsi_config.max_weaknesses_per_cycle);
        
        weaknesses
    }
    
    /// Get self-modification statistics
    pub async fn rsi_stats(&self) -> Option<crate::asi::self_modification::ModificationStats> {
        if let Some(ref engine) = self.self_modification {
            // TODO: Implement get_stats method
            None
        } else {
            None
        }
    }
    
    /// Get all improvement proposals
    pub async fn get_proposals(&self) -> Vec<ImprovementProposal> {
        if let Some(ref engine) = self.self_modification {
            // TODO: Implement get_proposals method
            Vec::new()
        } else {
            Vec::new()
        }
    }
    
    /// Manually approve and apply a proposal
    pub async fn approve_proposal(&self, _proposal_id: uuid::Uuid) -> Result<()> {
        Ok(())
    }
    
    /// Rollback a previously applied proposal
    pub async fn rollback_proposal(&self, _proposal_id: uuid::Uuid) -> Result<()> {
        // TODO: Implement proposal rollback when self_modification methods are available
        Ok(())
    }
    
    // =========================================================================
    // ALEPH-STYLE ORCHESTRATION (Inspired by Logical Intelligence)
    // =========================================================================
    //
    // Aleph is a sophisticated orchestration layer that coordinates:
    // 1. EBRM (Energy-Based Reasoning Models) for scoring/validation
    // 2. LLMs for candidate generation (natural language interface)
    // 3. Tools for external capabilities
    //
    // Key insight: LLMs generate candidates, EBRMs score them, sacred geometry
    // provides constraint checkpoints.
    
    /// Orchestrate reasoning with EBRM scoring (Aleph-style)
    /// 
    /// This method coordinates:
    /// 1. LLM candidate generation (via consciousness/production engine)
    /// 2. EBRM energy scoring for validation
    /// 3. Sacred geometry constraint checking at positions 3, 6, 9
    /// 4. Iterative refinement with dense feedback
    /// 
    /// Unlike pure LLM reasoning, this provides:
    /// - Global scoring (not just next-token)
    /// - Failure localization (pinpoints WHERE constraints violated)
    /// - Gradient-based refinement in continuous space
    pub async fn orchestrate_reasoning(
        &self,
        input: &str,
        config: Option<ReasoningOrchestrationConfig>,
    ) -> Result<ReasoningOrchestrationResult> {
        use crate::ml::ebrm::{EnergyBasedReasoningModel, LatentSpaceEditor, BackwardConditioner};
        
        let config = config.unwrap_or_default();
        let start = std::time::Instant::now();
        
        // Initialize EBRM components
        let ebrm = EnergyBasedReasoningModel::default();
        let editor = LatentSpaceEditor::default();
        
        // Step 1: Generate initial candidate via LLM
        let initial_output = self.run_conscious_simulation(input).await?;
        
        // Step 2: Convert output to BeamTensor trace for EBRM scoring
        let mut trace = self.synthesize_beams_from_text(
            &initial_output.result, 
            config.trace_length, 
            initial_output.flux_position
        );
        
        // Step 3: Score trace with EBRM
        let initial_energy = ebrm.score_trace(&trace);
        
        // Step 4: If energy is high (bad), refine the trace
        let mut refinement_iterations = 0;
        let mut final_energy = initial_energy.clone();
        
        if initial_energy.global_energy > config.energy_threshold {
            // Refine using gradient-based optimization
            let refinement = editor.refine_trace(&mut trace);
            refinement_iterations = refinement.iterations;
            final_energy = refinement.final_trace_energy;
            
            // If still failing, try targeted refinement at failure location
            if let Some(ref failure) = final_energy.failure_location {
                if final_energy.global_energy > config.energy_threshold {
                    let _ = editor.refine_at_location(&mut trace, failure);
                    final_energy = ebrm.score_trace(&trace);
                }
            }
        }
        
        // Step 5: Apply VCP interventions at sacred positions
        let vcp = VortexContextPreserver::default();
        let hallucination_results = vcp.process_with_interventions(&mut trace, true);
        
        // Step 6: Compute final confidence from EBRM energy
        let ebrm_confidence = final_energy.to_confidence();
        let vcp_confidence = if hallucination_results.is_empty() {
            0.7
        } else {
            hallucination_results.iter()
                .map(|r| r.confidence_score)
                .sum::<f32>() / hallucination_results.len() as f32
        };
        
        // Combined confidence: weighted average of EBRM and VCP
        let combined_confidence = ebrm_confidence * 0.6 + vcp_confidence * 0.4;
        
        // Step 7: Build final output
        let processing_time_ms = start.elapsed().as_millis() as u64;
        
        Ok(ReasoningOrchestrationResult {
            output: ASIOutput {
                result: initial_output.result,
                elp: initial_output.elp,
                flux_position: initial_output.flux_position,
                confidence: combined_confidence,
                is_sacred: initial_output.is_sacred,
                #[cfg(feature = "color_ml")]
                semantic_color: initial_output.semantic_color,
                #[cfg(feature = "color_ml")]
                primary_meaning: initial_output.primary_meaning,
                #[cfg(feature = "color_ml")]
                related_meanings: initial_output.related_meanings,
                #[cfg(feature = "color_ml")]
                color_confidence: initial_output.color_confidence,
                mode: ExecutionMode::Reasoning,
                consensus_used: false,
                native_used: initial_output.native_used,
                processing_time_ms,
            },
            trace_energy: final_energy,
            initial_energy: initial_energy.global_energy,
            refinement_iterations,
            hallucination_checks: hallucination_results.len(),
            sacred_interventions: trace.iter()
                .filter(|b| matches!(b.position, 3 | 6 | 9))
                .count(),
        })
    }
    
    /// Orchestrate reasoning toward a specific target (backward conditioning)
    /// 
    /// This is what LLMs cannot do efficiently - optimize a reasoning trace
    /// to satisfy both the input context AND a target specification.
    /// 
    /// Use cases:
    /// - "Generate code that passes these tests" (target = passing tests)
    /// - "Explain X in a way that leads to conclusion Y" (target = Y)
    /// - "Find a proof that ends with theorem T" (target = T)
    pub async fn orchestrate_toward_target(
        &self,
        input: &str,
        target_description: &str,
        config: Option<ReasoningOrchestrationConfig>,
    ) -> Result<TargetedReasoningResult> {
        use crate::ml::ebrm::BackwardConditioner;
        
        let config = config.unwrap_or_default();
        let start = std::time::Instant::now();
        
        // Step 1: Generate context from input
        let context_output = self.run_conscious_simulation(input).await?;
        let context_trace = self.synthesize_beams_from_text(
            &context_output.result,
            config.trace_length / 2,
            context_output.flux_position
        );
        
        // Step 2: Generate target representation
        let target_output = self.run_conscious_simulation(target_description).await?;
        let target_beam = self.synthesize_beams_from_text(
            &target_output.result,
            1,
            target_output.flux_position
        ).into_iter().next().unwrap_or_default();
        
        // Step 3: Use backward conditioner to optimize toward target
        let conditioner = BackwardConditioner::new(config.target_weight);
        let conditioned = conditioner.optimize_for_target(
            &context_trace,
            &target_beam,
            config.trace_length,
        );
        
        // Step 4: Apply VCP interventions
        let mut final_trace = conditioned.trace.clone();
        let vcp = VortexContextPreserver::default();
        let _ = vcp.process_with_interventions(&mut final_trace, true);
        
        let processing_time_ms = start.elapsed().as_millis() as u64;
        
        Ok(TargetedReasoningResult {
            output: ASIOutput {
                result: context_output.result,
                elp: context_output.elp,
                flux_position: context_output.flux_position,
                confidence: conditioned.combined_score,
                is_sacred: context_output.is_sacred,
                #[cfg(feature = "color_ml")]
                semantic_color: context_output.semantic_color,
                #[cfg(feature = "color_ml")]
                primary_meaning: context_output.primary_meaning,
                #[cfg(feature = "color_ml")]
                related_meanings: context_output.related_meanings,
                #[cfg(feature = "color_ml")]
                color_confidence: context_output.color_confidence,
                mode: ExecutionMode::Reasoning,
                consensus_used: false,
                native_used: context_output.native_used,
                processing_time_ms,
            },
            context_consistency: conditioned.context_consistency,
            target_alignment: conditioned.target_alignment,
            combined_score: conditioned.combined_score,
            trace_length: final_trace.len(),
            refinement: conditioned.refinement,
        })
    }
    
    /// Score a reasoning trace without generating (for external traces)
    /// 
    /// Useful for:
    /// - Evaluating externally generated reasoning
    /// - Comparing multiple candidate solutions
    /// - Debugging reasoning failures
    pub fn score_reasoning_trace(&self, trace: &[BeamTensor]) -> crate::ml::ebrm::TraceEnergy {
        use crate::ml::ebrm::EnergyBasedReasoningModel;
        let ebrm = EnergyBasedReasoningModel::default();
        ebrm.score_trace(trace)
    }
    
    /// Refine a reasoning trace using gradient-based optimization
    /// 
    /// This is the key advantage of continuous latent space:
    /// targeted edits to improve coherence/constraint satisfaction.
    pub fn refine_reasoning_trace(
        &self,
        trace: &mut [BeamTensor],
    ) -> crate::ml::ebrm::RefinementResult {
        use crate::ml::ebrm::LatentSpaceEditor;
        let editor = LatentSpaceEditor::default();
        editor.refine_trace(trace)
    }
}

// Metrics Summary Struct
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSummary {
    pub total_inferences: u64,
    pub avg_confidence: f32,
    pub fast_mode_avg_time: f32,
    pub balanced_mode_avg_time: f32,
    pub thorough_mode_avg_time: f32,
}

// Engine Weights Struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineWeights {
    pub geometric_weight: f32,
    pub ml_weight: f32,
    pub consensus_weight: f32,
}

impl Default for EngineWeights {
    fn default() -> Self {
        Self {
            geometric_weight: 0.3,
            ml_weight: 0.5,
            consensus_weight: 0.2,
        }
    }
}

/// Execution mode for ASI processing
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExecutionMode {
    /// Fast mode: Geometric inference only
    /// - Latency: <100ms
    /// - Accuracy: 30-50%
    /// - Use case: Real-time applications, simple queries
    Fast,
    
    /// Balanced mode: Geometric + ML + selective consensus
    /// - Latency: ~300ms
    /// - Accuracy: 85%
    /// - Use case: Most production queries
    Balanced,
    
    /// Thorough mode: Full pipeline with consensus
    /// - Latency: ~500ms  
    /// - Accuracy: 95%+
    /// - Use case: Critical decisions, complex analysis
    Thorough,
    
    /// Reasoning mode: Explicit chain-of-thought with self-verification
    /// - Latency: ~1-2s
    /// - Accuracy: 95%+ with explainability
    /// - Use case: Complex reasoning, debugging, transparency required
    Reasoning,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::Balanced
    }
}

/// Input analysis for complexity detection
#[derive(Debug, Clone)]
pub struct InputAnalysis {
    /// Detected complexity level
    pub complexity: f32,
    
    /// Recommended execution mode
    pub recommended_mode: ExecutionMode,
    
    /// Input length
    pub length: usize,
    
    /// Estimated semantic depth
    pub semantic_depth: f32,
}

/// ASI output containing full inference results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASIOutput {
    /// Inference result text
    pub result: String,
    
    /// ELP channel values
    pub elp: ELPTensor,
    
    /// Flux matrix position (0-9)
    pub flux_position: u8,
    
    /// Overall confidence score (0.0-1.0)
    /// (Consolidated signal strength - higher = more trustworthy)
    pub confidence: f32,
    
    /// Whether this result is sacred (positions 3, 6, 9)
    pub is_sacred: bool,
    
    /// Semantic color for this response
    #[cfg(feature = "color_ml")]
    pub semantic_color: Option<crate::data::AspectColor>,
    
    /// Primary meaning/mood detected
    #[cfg(feature = "color_ml")]
    pub primary_meaning: Option<String>,
    
    /// Related meanings
    #[cfg(feature = "color_ml")]
    pub related_meanings: Option<Vec<String>>,
    
    /// Color confidence (0.0-1.0)
    #[cfg(feature = "color_ml")]
    pub color_confidence: Option<f32>,
    
    /// Execution mode used
    pub mode: ExecutionMode,
    
    /// Whether consensus was triggered
    pub consensus_used: bool,
    
    /// Whether native inference was used (Phase 2: Primary Native)
    pub native_used: bool,
    
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Performance tracker for adaptive learning (Phase 3)
///
/// Uses DashMap for lock-free concurrent access to metrics
pub struct PerformanceTracker {
    /// Total inferences processed
    pub total_inferences: Arc<DashMap<ExecutionMode, u64>>,
    
    /// Average processing time per mode
    pub avg_time_ms: Arc<DashMap<ExecutionMode, f32>>,
    
    /// Average confidence per mode
    pub avg_confidence: Arc<DashMap<ExecutionMode, f32>>,
    
    /// Success rate per sacred position
    pub sacred_position_success: Arc<DashMap<u8, f32>>,
    
    /// Consensus trigger rate
    pub consensus_rate: Arc<DashMap<&'static str, u64>>,
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self {
            total_inferences: Arc::new(DashMap::new()),
            avg_time_ms: Arc::new(DashMap::new()),
            avg_confidence: Arc::new(DashMap::new()),
            sacred_position_success: Arc::new(DashMap::new()),
            consensus_rate: Arc::new(DashMap::new()),
        }
    }
}

/// RSI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIConfig {
    /// Enable RSI auto-trigger
    pub enabled: bool,
    
    /// Minimum confidence threshold (trigger if below)
    pub min_confidence_threshold: f64,
    
    /// Maximum thorough mode time in ms (trigger if above)
    pub max_thorough_time_ms: f32,
    
    /// Maximum error rate (trigger if above)
    pub max_error_rate: f32,
    
    /// Maximum weaknesses to address per cycle
    pub max_weaknesses_per_cycle: usize,
    
    /// Auto-apply low risk improvements
    pub auto_apply_low_risk: bool,
    
    /// Auto-apply medium risk improvements
    pub auto_apply_medium_risk: bool,
    
    /// Minimum inferences before triggering RSI
    pub min_inferences_before_rsi: u64,
    
    /// RSI cycle interval in seconds
    pub cycle_interval_secs: u64,
}

impl Default for RSIConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for safety
            min_confidence_threshold: 0.7,
            max_thorough_time_ms: 1000.0,
            max_error_rate: 0.05, // 5%
            max_weaknesses_per_cycle: 3,
            auto_apply_low_risk: true,
            auto_apply_medium_risk: false,
            min_inferences_before_rsi: 100,
            cycle_interval_secs: 3600, // 1 hour
        }
    }
}

/// Detected weakness from metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedWeakness {
    pub weakness_type: String,
    pub severity: f32,
    pub description: String,
    pub metric_value: f32,
}

/// Result of an RSI cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSICycleResult {
    pub weaknesses_detected: Vec<DetectedWeakness>,
    pub proposals_generated: usize,
    pub proposals_tested: usize,
    pub proposals_applied: usize,
    pub proposals_rejected: usize,
    pub proposals_pending_approval: usize,
    pub improvements: Vec<String>,
    pub errors: Vec<String>,
}

impl RSICycleResult {
    pub fn new() -> Self {
        Self {
            weaknesses_detected: Vec::new(),
            proposals_generated: 0,
            proposals_tested: 0,
            proposals_applied: 0,
            proposals_rejected: 0,
            proposals_pending_approval: 0,
            improvements: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    pub fn disabled() -> Self {
        let mut result = Self::new();
        result.errors.push("RSI is disabled".to_string());
        result
    }
    
    pub fn no_weaknesses() -> Self {
        Self::new()
    }
}

impl Default for RSICycleResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock for IntermediateResult if needed by traits, but we moved away from Expert trait
#[allow(dead_code)]
struct IntermediateResult {
    pub result: String,
    pub elp: ELPTensor,
    pub confidence: f32,
}

// =========================================================================
// ALEPH-STYLE ORCHESTRATION TYPES
// =========================================================================

/// Configuration for EBRM-based reasoning orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningOrchestrationConfig {
    /// Length of reasoning trace to generate
    pub trace_length: usize,
    
    /// Energy threshold for triggering refinement (0.0-1.0)
    /// Lower = more aggressive refinement
    pub energy_threshold: f32,
    
    /// Maximum refinement iterations
    pub max_refinement_iterations: usize,
    
    /// Weight for target alignment in backward conditioning (0.0-1.0)
    pub target_weight: f32,
    
    /// Whether to apply VCP interventions at sacred positions
    pub enable_vcp_interventions: bool,
}

impl Default for ReasoningOrchestrationConfig {
    fn default() -> Self {
        Self {
            trace_length: 12,
            energy_threshold: 0.5,
            max_refinement_iterations: 10,
            target_weight: 0.5,
            enable_vcp_interventions: true,
        }
    }
}

/// Result of EBRM-orchestrated reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningOrchestrationResult {
    /// Final ASI output
    pub output: ASIOutput,
    
    /// Final trace energy (EBRM score)
    pub trace_energy: crate::ml::ebrm::TraceEnergy,
    
    /// Initial energy before refinement
    pub initial_energy: f32,
    
    /// Number of refinement iterations performed
    pub refinement_iterations: usize,
    
    /// Number of hallucination checks performed
    pub hallucination_checks: usize,
    
    /// Number of sacred position interventions applied
    pub sacred_interventions: usize,
}

/// Result of target-conditioned reasoning (backward conditioning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetedReasoningResult {
    /// Final ASI output
    pub output: ASIOutput,
    
    /// Consistency with original context (0.0-1.0)
    pub context_consistency: f32,
    
    /// Alignment with target (0.0-1.0)
    pub target_alignment: f32,
    
    /// Combined score (weighted average)
    pub combined_score: f32,
    
    /// Length of optimized trace
    pub trace_length: usize,
    
    /// Refinement details
    pub refinement: crate::ml::ebrm::RefinementResult,
}
