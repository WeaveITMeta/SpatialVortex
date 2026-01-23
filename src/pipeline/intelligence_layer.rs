//! Intelligence Layer
//!
//! The fourth layer - reasons, learns, and generates insights using VortexModel.
//!
//! ## Responsibilities
//!
//! - **Reasoning**: Apply VortexModel for contextual understanding
//! - **Learning**: Update knowledge from processed data
//! - **Hypothesis Generation**: Generate hypotheses and deductions
//! - **Multi-modal Fusion**: Combine insights across modalities
//! - **Sacred Geometry Integration**: Apply 3-6-9 checkpoints for coherence

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;

use super::data_types::{DataEnvelope, ProcessingStage};
use super::processing_layer::{ProcessedData, Features, EmbeddingResult};
use super::PipelineError;

/// Intelligence layer configuration
#[derive(Debug, Clone)]
pub struct IntelligenceConfig {
    /// Enable VortexModel reasoning
    pub enable_reasoning: bool,
    /// Enable continuous learning
    pub enable_learning: bool,
    /// Enable hypothesis generation
    pub enable_hypothesis: bool,
    /// Minimum confidence for hypothesis
    pub min_hypothesis_confidence: f32,
    /// Maximum hypotheses per input
    pub max_hypotheses: usize,
    /// Signal strength threshold for learning
    pub learning_threshold: f32,
    /// Enable sacred geometry checkpoints
    pub sacred_geometry_enabled: bool,
    /// VCP (Vortex Context Preserver) threshold
    pub vcp_threshold: f32,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            enable_reasoning: true,
            enable_learning: true,
            enable_hypothesis: true,
            min_hypothesis_confidence: 0.5,
            max_hypotheses: 5,
            learning_threshold: 0.6,
            sacred_geometry_enabled: true,
            vcp_threshold: 0.6,
        }
    }
}

/// Reasoning result from intelligence layer
#[derive(Debug, Clone)]
pub struct ReasoningResult {
    /// Original processed data
    pub processed: ProcessedData,
    /// Generated insights
    pub insights: Vec<Insight>,
    /// Generated hypotheses
    pub hypotheses: Vec<Hypothesis>,
    /// Reasoning chain
    pub reasoning_chain: Vec<ReasoningStep>,
    /// ELP tensor after reasoning
    pub elp: [f32; 3],
    /// Signal strength and confidence after reasoning
    pub confidence: f32,
    /// Knowledge updates to apply
    pub knowledge_updates: Vec<LearningUpdate>,
    /// Metadata
    pub metadata: ReasoningMetadata,
}

/// An insight derived from data
#[derive(Debug, Clone)]
pub struct Insight {
    /// Insight type
    pub insight_type: InsightType,
    /// Description
    pub description: String,
    /// Confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Supporting evidence
    pub evidence: Vec<String>,
    /// Related features
    pub related_features: Vec<String>,
}

/// Types of insights
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightType {
    Pattern,
    Anomaly,
    Trend,
    Correlation,
    Classification,
    Prediction,
    Summary,
    Recommendation,
}

/// A hypothesis generated from reasoning
#[derive(Debug, Clone)]
pub struct Hypothesis {
    /// Hypothesis ID
    pub id: String,
    /// Statement
    pub statement: String,
    /// Confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Supporting evidence
    pub evidence: Vec<String>,
    /// Counter-evidence
    pub counter_evidence: Vec<String>,
    /// Testable predictions
    pub predictions: Vec<String>,
    /// Domain relevance
    pub domain: String,
}

/// A step in the reasoning chain
#[derive(Debug, Clone)]
pub struct ReasoningStep {
    /// Step number
    pub step: usize,
    /// Operation performed
    pub operation: String,
    /// Input summary
    pub input: String,
    /// Output summary
    pub output: String,
    /// Confidence after this step
    pub confidence: f32,
    /// Is this a sacred position (3, 6, 9)?
    pub is_sacred_position: bool,
}

/// Knowledge update for learning
#[derive(Debug, Clone)]
pub struct LearningUpdate {
    /// Update type
    pub update_type: LearningUpdateType,
    /// Key/identifier
    pub key: String,
    /// Value/content
    pub value: String,
    /// Confidence
    pub confidence: f32,
    /// Source
    pub source: String,
    /// Timestamp
    pub timestamp: std::time::SystemTime,
}

/// Types of learning updates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningUpdateType {
    /// New knowledge
    NewKnowledge,
    /// Reinforcement of existing knowledge
    Reinforcement,
    /// Correction of existing knowledge
    Correction,
    /// Pattern learned
    Pattern,
    /// Association learned
    Association,
}

/// Reasoning metadata
#[derive(Debug, Clone, Default)]
pub struct ReasoningMetadata {
    /// Total reasoning time in milliseconds
    pub reasoning_time_ms: u64,
    /// Number of reasoning steps
    pub steps: usize,
    /// Sacred checkpoints triggered
    pub sacred_checkpoints: usize,
    /// VCP interventions
    pub vcp_interventions: usize,
    /// Model used
    pub model: String,
}

/// Knowledge base for continuous learning
#[derive(Debug, Clone, Default)]
pub struct KnowledgeBase {
    /// Facts (key -> value with confidence)
    pub facts: HashMap<String, (String, f32)>,
    /// Patterns (pattern_id -> description)
    pub patterns: HashMap<String, String>,
    /// Associations (from -> to with strength)
    pub associations: HashMap<String, Vec<(String, f32)>>,
    /// Domain knowledge
    pub domains: HashMap<String, DomainKnowledge>,
    /// Total updates
    pub total_updates: u64,
}

/// Domain-specific knowledge
#[derive(Debug, Clone, Default)]
pub struct DomainKnowledge {
    /// Domain name
    pub name: String,
    /// Key concepts
    pub concepts: Vec<String>,
    /// Common patterns
    pub patterns: Vec<String>,
    /// ELP bias for this domain
    pub elp_bias: [f32; 3],
}

/// The Intelligence Layer
pub struct IntelligenceLayer {
    config: IntelligenceConfig,
    /// Knowledge base
    knowledge: RwLock<KnowledgeBase>,
    /// Reasoning statistics
    stats: RwLock<IntelligenceStats>,
}

/// Intelligence layer statistics
#[derive(Debug, Clone, Default)]
pub struct IntelligenceStats {
    pub total_reasoned: u64,
    pub total_hypotheses: u64,
    pub total_insights: u64,
    pub total_learning_updates: u64,
    pub avg_confidence: f32,
    pub sacred_interventions: u64,
}

impl IntelligenceLayer {
    /// Create new intelligence layer
    pub fn new(config: IntelligenceConfig) -> Self {
        Self {
            config,
            knowledge: RwLock::new(KnowledgeBase::default()),
            stats: RwLock::new(IntelligenceStats::default()),
        }
    }
    
    /// Reason over processed data
    pub async fn reason(&self, processed: ProcessedData) -> Result<ReasoningResult, PipelineError> {
        let start = std::time::Instant::now();
        
        let mut reasoning_chain = Vec::new();
        let mut insights = Vec::new();
        let mut hypotheses = Vec::new();
        let mut knowledge_updates = Vec::new();
        let mut sacred_checkpoints = 0;
        let mut vcp_interventions = 0;
        
        // Step 1: Analyze features
        let (feature_insights, step1_confidence) = self.analyze_features(&processed.features);
        insights.extend(feature_insights);
        reasoning_chain.push(ReasoningStep {
            step: 1,
            operation: "feature_analysis".to_string(),
            input: format!("{} features", processed.features.numeric.len() + processed.features.categorical.len()),
            output: format!("{} insights", insights.len()),
            confidence: step1_confidence,
            is_sacred_position: false,
        });
        
        // Step 2: Embedding analysis (if available)
        let step2_confidence = if let Some(ref embedding) = processed.embeddings {
            let (embed_insights, conf) = self.analyze_embedding(embedding);
            insights.extend(embed_insights);
            conf
        } else {
            step1_confidence
        };
        reasoning_chain.push(ReasoningStep {
            step: 2,
            operation: "embedding_analysis".to_string(),
            input: "embedding vector".to_string(),
            output: format!("confidence: {:.2}", step2_confidence),
            confidence: step2_confidence,
            is_sacred_position: false,
        });
        
        // Step 3: Sacred checkpoint (position 3)
        if self.config.sacred_geometry_enabled {
            let (adjusted_confidence, intervention) = self.sacred_checkpoint(step2_confidence, 3);
            if intervention {
                sacred_checkpoints += 1;
                vcp_interventions += 1;
            }
            reasoning_chain.push(ReasoningStep {
                step: 3,
                operation: "sacred_checkpoint_3".to_string(),
                input: format!("confidence: {:.2}", step2_confidence),
                output: format!("adjusted: {:.2}", adjusted_confidence),
                confidence: adjusted_confidence,
                is_sacred_position: true,
            });
        }
        
        // Step 4: Domain reasoning
        let domain = &processed.envelope.context.domain;
        let domain_insights = self.domain_reasoning(domain, &processed.features);
        insights.extend(domain_insights);
        reasoning_chain.push(ReasoningStep {
            step: 4,
            operation: "domain_reasoning".to_string(),
            input: domain.clone(),
            output: format!("{} domain insights", insights.len()),
            confidence: step2_confidence,
            is_sacred_position: false,
        });
        
        // Step 5: Pattern matching
        let patterns = self.match_patterns(&processed.features);
        for pattern in &patterns {
            insights.push(Insight {
                insight_type: InsightType::Pattern,
                description: pattern.clone(),
                confidence: 0.7,
                evidence: vec!["pattern_match".to_string()],
                related_features: Vec::new(),
            });
        }
        reasoning_chain.push(ReasoningStep {
            step: 5,
            operation: "pattern_matching".to_string(),
            input: "features".to_string(),
            output: format!("{} patterns", patterns.len()),
            confidence: step2_confidence,
            is_sacred_position: false,
        });
        
        // Step 6: Sacred checkpoint (position 6)
        let step6_confidence = if self.config.sacred_geometry_enabled {
            let (adjusted, intervention) = self.sacred_checkpoint(step2_confidence, 6);
            if intervention {
                sacred_checkpoints += 1;
                vcp_interventions += 1;
            }
            reasoning_chain.push(ReasoningStep {
                step: 6,
                operation: "sacred_checkpoint_6".to_string(),
                input: format!("confidence: {:.2}", step2_confidence),
                output: format!("adjusted: {:.2}", adjusted),
                confidence: adjusted,
                is_sacred_position: true,
            });
            adjusted
        } else {
            step2_confidence
        };
        
        // Step 7: Hypothesis generation
        if self.config.enable_hypothesis {
            let generated = self.generate_hypotheses(&insights, domain, step6_confidence);
            hypotheses.extend(generated);
        }
        reasoning_chain.push(ReasoningStep {
            step: 7,
            operation: "hypothesis_generation".to_string(),
            input: format!("{} insights", insights.len()),
            output: format!("{} hypotheses", hypotheses.len()),
            confidence: step6_confidence,
            is_sacred_position: false,
        });
        
        // Step 8: Knowledge extraction
        if self.config.enable_learning && step6_confidence >= self.config.learning_threshold {
            let updates = self.extract_knowledge(&processed, &insights);
            knowledge_updates.extend(updates);
        }
        reasoning_chain.push(ReasoningStep {
            step: 8,
            operation: "knowledge_extraction".to_string(),
            input: "insights".to_string(),
            output: format!("{} updates", knowledge_updates.len()),
            confidence: step6_confidence,
            is_sacred_position: false,
        });
        
        // Step 9: Sacred checkpoint (position 9) - final validation
        let final_confidence = if self.config.sacred_geometry_enabled {
            let (adjusted, intervention) = self.sacred_checkpoint(step6_confidence, 9);
            if intervention {
                sacred_checkpoints += 1;
                vcp_interventions += 1;
            }
            reasoning_chain.push(ReasoningStep {
                step: 9,
                operation: "sacred_checkpoint_9".to_string(),
                input: format!("confidence: {:.2}", step6_confidence),
                output: format!("final: {:.2}", adjusted),
                confidence: adjusted,
                is_sacred_position: true,
            });
            adjusted
        } else {
            step6_confidence
        };
        
        // Calculate ELP
        let elp = processed.envelope.elp.unwrap_or([0.33, 0.34, 0.33]);
        
        // Update statistics
        {
            let mut stats = self.stats.write();
            stats.total_reasoned += 1;
            stats.total_hypotheses += hypotheses.len() as u64;
            stats.total_insights += insights.len() as u64;
            stats.total_learning_updates += knowledge_updates.len() as u64;
            stats.sacred_interventions += sacred_checkpoints as u64;
            stats.avg_confidence = (stats.avg_confidence * (stats.total_reasoned - 1) as f32 + final_confidence) / stats.total_reasoned as f32;
        }
        
        let metadata = ReasoningMetadata {
            reasoning_time_ms: start.elapsed().as_millis() as u64,
            steps: reasoning_chain.len(),
            sacred_checkpoints,
            vcp_interventions,
            model: "vortex_intelligence".to_string(),
        };
        
        // Update envelope stage
        let mut envelope = processed.envelope.clone();
        envelope.advance_stage(ProcessingStage::Intelligence);
        
        Ok(ReasoningResult {
            processed: ProcessedData { envelope, ..processed },
            insights,
            hypotheses,
            reasoning_chain,
            elp,
            confidence: final_confidence,
            knowledge_updates,
            metadata,
        })
    }
    
    /// Analyze features and generate insights
    fn analyze_features(&self, features: &Features) -> (Vec<Insight>, f32) {
        let mut insights = Vec::new();
        let mut confidence = 0.7;
        
        // Analyze numeric features
        for (key, value) in &features.numeric {
            if *value > 100.0 {
                insights.push(Insight {
                    insight_type: InsightType::Anomaly,
                    description: format!("High value detected for {}: {:.2}", key, value),
                    confidence: 0.6,
                    evidence: vec![format!("{}={:.2}", key, value)],
                    related_features: vec![key.clone()],
                });
            }
        }
        
        // Check for patterns in categorical features
        if features.categorical.len() > 3 {
            insights.push(Insight {
                insight_type: InsightType::Summary,
                description: format!("Data contains {} categorical attributes", features.categorical.len()),
                confidence: 0.9,
                evidence: features.categorical.keys().cloned().collect(),
                related_features: Vec::new(),
            });
        }
        
        if !insights.is_empty() {
            confidence = 0.8;
        }
        
        (insights, confidence)
    }
    
    /// Analyze embedding
    fn analyze_embedding(&self, embedding: &EmbeddingResult) -> (Vec<Insight>, f32) {
        let mut insights = Vec::new();
        
        // Check embedding quality
        if embedding.quality > 0.8 {
            insights.push(Insight {
                insight_type: InsightType::Summary,
                description: format!("High quality {} embedding generated", embedding.model),
                confidence: embedding.quality,
                evidence: vec![format!("dim={}", embedding.dim)],
                related_features: Vec::new(),
            });
        }
        
        // Check for sparse embedding
        let non_zero = embedding.embedding.iter().filter(|&&x| x.abs() > 0.01).count();
        let sparsity = 1.0 - (non_zero as f32 / embedding.dim as f32);
        if sparsity > 0.8 {
            insights.push(Insight {
                insight_type: InsightType::Pattern,
                description: format!("Sparse embedding detected (sparsity: {:.2})", sparsity),
                confidence: 0.7,
                evidence: vec![format!("non_zero={}", non_zero)],
                related_features: Vec::new(),
            });
        }
        
        (insights, embedding.quality)
    }
    
    /// Sacred geometry checkpoint
    fn sacred_checkpoint(&self, confidence: f32, position: u8) -> (f32, bool) {
        // Check if intervention needed
        if confidence < self.config.vcp_threshold {
            // Apply sacred boost
            let boost = match position {
                3 => 1.15,  // Ethos boost
                6 => 1.10,  // Logos boost
                9 => 1.05,  // Pathos boost (final validation)
                _ => 1.0,
            };
            let adjusted = (confidence * boost).min(1.0);
            (adjusted, true)
        } else {
            (confidence, false)
        }
    }
    
    /// Domain-specific reasoning
    fn domain_reasoning(&self, domain: &str, features: &Features) -> Vec<Insight> {
        let mut insights = Vec::new();
        
        match domain {
            "healthcare" => {
                if features.categorical.contains_key("patient") || features.categorical.contains_key("diagnosis") {
                    insights.push(Insight {
                        insight_type: InsightType::Classification,
                        description: "Healthcare data detected with patient information".to_string(),
                        confidence: 0.85,
                        evidence: vec!["domain=healthcare".to_string()],
                        related_features: vec!["patient".to_string()],
                    });
                }
            }
            "iot" => {
                if features.numeric.contains_key("temperature") || features.numeric.contains_key("humidity") {
                    insights.push(Insight {
                        insight_type: InsightType::Classification,
                        description: "Environmental sensor data detected".to_string(),
                        confidence: 0.8,
                        evidence: vec!["domain=iot".to_string()],
                        related_features: vec!["temperature".to_string(), "humidity".to_string()],
                    });
                }
            }
            _ => {}
        }
        
        insights
    }
    
    /// Match known patterns
    fn match_patterns(&self, features: &Features) -> Vec<String> {
        let mut patterns = Vec::new();
        let knowledge = self.knowledge.read();
        
        for (pattern_id, description) in &knowledge.patterns {
            // Simple pattern matching based on feature presence
            if features.numeric.len() > 5 {
                patterns.push(format!("Complex numeric pattern: {}", description));
            }
        }
        
        // Built-in patterns
        if features.numeric.contains_key("mean") && features.numeric.contains_key("std_dev") {
            patterns.push("Statistical distribution pattern detected".to_string());
        }
        
        patterns
    }
    
    /// Generate hypotheses from insights
    fn generate_hypotheses(&self, insights: &[Insight], domain: &str, confidence: f32) -> Vec<Hypothesis> {
        let mut hypotheses = Vec::new();
        
        if confidence < self.config.min_hypothesis_confidence {
            return hypotheses;
        }
        
        // Generate hypotheses based on insights
        for (i, insight) in insights.iter().take(self.config.max_hypotheses).enumerate() {
            if insight.confidence > 0.6 {
                hypotheses.push(Hypothesis {
                    id: format!("hyp_{}", i),
                    statement: format!("Based on {}: {}", insight.insight_type.as_str(), insight.description),
                    confidence: insight.confidence * 0.8,
                    evidence: insight.evidence.clone(),
                    counter_evidence: Vec::new(),
                    predictions: vec![format!("Further {} data will show similar patterns", domain)],
                    domain: domain.to_string(),
                });
            }
        }
        
        hypotheses
    }
    
    /// Extract knowledge for learning
    fn extract_knowledge(&self, processed: &ProcessedData, insights: &[Insight]) -> Vec<LearningUpdate> {
        let mut updates = Vec::new();
        
        // Extract facts from features
        for (key, value) in &processed.features.numeric {
            if *value > 0.0 {
                updates.push(LearningUpdate {
                    update_type: LearningUpdateType::NewKnowledge,
                    key: format!("feature:{}", key),
                    value: format!("{:.4}", value),
                    confidence: 0.8,
                    source: processed.envelope.source.resource_id.clone(),
                    timestamp: std::time::SystemTime::now(),
                });
            }
        }
        
        // Extract patterns from insights
        for insight in insights {
            if insight.insight_type == InsightType::Pattern && insight.confidence > 0.7 {
                updates.push(LearningUpdate {
                    update_type: LearningUpdateType::Pattern,
                    key: format!("pattern:{}", insight.description.chars().take(50).collect::<String>()),
                    value: insight.description.clone(),
                    confidence: insight.confidence,
                    source: processed.envelope.source.resource_id.clone(),
                    timestamp: std::time::SystemTime::now(),
                });
            }
        }
        
        updates
    }
    
    /// Update knowledge base from output
    pub async fn update_knowledge(&self, result: &super::output_layer::OutputResult) {
        let mut knowledge = self.knowledge.write();
        
        // Apply learning updates
        for update in &result.reasoning.knowledge_updates {
            match update.update_type {
                LearningUpdateType::NewKnowledge | LearningUpdateType::Reinforcement => {
                    knowledge.facts.insert(update.key.clone(), (update.value.clone(), update.confidence));
                }
                LearningUpdateType::Pattern => {
                    knowledge.patterns.insert(update.key.clone(), update.value.clone());
                }
                LearningUpdateType::Association => {
                    knowledge.associations
                        .entry(update.key.clone())
                        .or_insert_with(Vec::new)
                        .push((update.value.clone(), update.confidence));
                }
                _ => {}
            }
        }
        
        knowledge.total_updates += result.reasoning.knowledge_updates.len() as u64;
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> IntelligenceStats {
        self.stats.read().clone()
    }
    
    /// Get knowledge base size
    pub fn knowledge_size(&self) -> usize {
        let kb = self.knowledge.read();
        kb.facts.len() + kb.patterns.len() + kb.associations.len()
    }
}

impl InsightType {
    fn as_str(&self) -> &'static str {
        match self {
            InsightType::Pattern => "pattern",
            InsightType::Anomaly => "anomaly",
            InsightType::Trend => "trend",
            InsightType::Correlation => "correlation",
            InsightType::Classification => "classification",
            InsightType::Prediction => "prediction",
            InsightType::Summary => "summary",
            InsightType::Recommendation => "recommendation",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::data_types::DataEnvelope;
    use super::super::processing_layer::{ProcessedData, Features, ProcessingMetadata};
    
    fn create_test_processed() -> ProcessedData {
        let mut features = Features::default();
        features.add_numeric("test_value", 42.0);
        features.add_categorical("category", "test");
        
        ProcessedData {
            envelope: DataEnvelope::new("test", super::super::data_types::UniversalData::Text("test".to_string())),
            features,
            embeddings: None,
            tokens: None,
            metadata: ProcessingMetadata::default(),
        }
    }
    
    #[tokio::test]
    async fn test_intelligence_layer_reason() {
        let layer = IntelligenceLayer::new(IntelligenceConfig::default());
        let processed = create_test_processed();
        
        let result = layer.reason(processed).await;
        assert!(result.is_ok());
        
        let reasoning = result.unwrap();
        assert!(!reasoning.reasoning_chain.is_empty());
        assert!(reasoning.confidence > 0.0);
    }
    
    #[test]
    fn test_sacred_checkpoint() {
        let layer = IntelligenceLayer::new(IntelligenceConfig::default());
        
        // Low confidence should trigger intervention
        let (adjusted, intervention) = layer.sacred_checkpoint(0.4, 3);
        assert!(intervention);
        assert!(adjusted > 0.4);
        
        // High confidence should not trigger
        let (adjusted, intervention) = layer.sacred_checkpoint(0.9, 3);
        assert!(!intervention);
        assert!((adjusted - 0.9).abs() < 0.01);
    }
    
    #[test]
    fn test_feature_analysis() {
        let layer = IntelligenceLayer::new(IntelligenceConfig::default());
        
        let mut features = Features::default();
        features.add_numeric("high_value", 150.0);
        
        let (insights, confidence) = layer.analyze_features(&features);
        assert!(!insights.is_empty());
        assert!(confidence > 0.5);
    }
}
