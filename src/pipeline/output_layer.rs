//! Output Layer
//!
//! The fifth layer - generates the best possible result for any modality.
//!
//! ## Responsibilities
//!
//! - **Multi-modal Generation**: Text, images, 3D, structured data, actions
//! - **Format Conversion**: Convert to requested output format
//! - **Visualization**: Generate visual representations
//! - **Export**: Export to various formats (JSON, CSV, Parquet, etc.)
//! - **Knowledge Feedback**: Feed insights back to knowledge base

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;

use super::data_types::{DataEnvelope, UniversalData, ProcessingStage};
use super::intelligence_layer::{ReasoningResult, Insight, Hypothesis, LearningUpdate};
use super::PipelineError;

/// Output layer configuration
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Default output format
    pub default_format: OutputFormat,
    /// Include reasoning chain in output
    pub include_reasoning: bool,
    /// Include hypotheses in output
    pub include_hypotheses: bool,
    /// Include raw features in output
    pub include_features: bool,
    /// Maximum output size in bytes
    pub max_output_size: usize,
    /// Enable streaming output
    pub enable_streaming: bool,
    /// Pretty print JSON output
    pub pretty_print: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_format: OutputFormat::Json,
            include_reasoning: true,
            include_hypotheses: true,
            include_features: false,
            max_output_size: 10 * 1024 * 1024, // 10MB
            enable_streaming: false,
            pretty_print: true,
        }
    }
}

/// Output format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    #[default]
    Json,
    Text,
    Markdown,
    Html,
    Csv,
    Xml,
    Binary,
    Structured,
    Visualization,
    Action,
}

impl OutputFormat {
    pub fn mime_type(&self) -> &'static str {
        match self {
            OutputFormat::Json => "application/json",
            OutputFormat::Text => "text/plain",
            OutputFormat::Markdown => "text/markdown",
            OutputFormat::Html => "text/html",
            OutputFormat::Csv => "text/csv",
            OutputFormat::Xml => "application/xml",
            OutputFormat::Binary => "application/octet-stream",
            OutputFormat::Structured => "application/json",
            OutputFormat::Visualization => "image/svg+xml",
            OutputFormat::Action => "application/json",
        }
    }
}

/// Output result from the pipeline
#[derive(Debug, Clone)]
pub struct OutputResult {
    /// Unique output ID
    pub id: String,
    /// Output format
    pub format: OutputFormat,
    /// Primary output content
    pub content: OutputContent,
    /// Reasoning result (for reference)
    pub reasoning: ReasoningResult,
    /// Summary of insights
    pub summary: String,
    /// Confidence in output (includes signal strength)
    pub confidence: f32,
    /// ELP tensor
    pub elp: [f32; 3],
    /// Output metadata
    pub metadata: OutputMetadata,
}

/// Output content variants
#[derive(Debug, Clone)]
pub enum OutputContent {
    /// Text output
    Text(String),
    /// JSON output
    Json(serde_json::Value),
    /// Binary output
    Binary(Vec<u8>),
    /// Structured response
    Structured(StructuredOutput),
    /// Visualization data
    Visualization(VisualizationData),
    /// Action to perform
    Action(ActionOutput),
}

/// Structured output with sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredOutput {
    /// Title/heading
    pub title: String,
    /// Summary paragraph
    pub summary: String,
    /// Key insights
    pub insights: Vec<InsightOutput>,
    /// Hypotheses (if any)
    pub hypotheses: Vec<HypothesisOutput>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Data tables
    pub tables: Vec<TableOutput>,
    /// Metrics
    pub metrics: HashMap<String, f64>,
}

/// Insight in output format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightOutput {
    pub insight_type: String,
    pub description: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

/// Hypothesis in output format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisOutput {
    pub statement: String,
    pub confidence: f32,
    pub predictions: Vec<String>,
}

/// Table output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOutput {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Visualization data
#[derive(Debug, Clone)]
pub struct VisualizationData {
    /// Visualization type
    pub viz_type: VisualizationType,
    /// Data points
    pub data: Vec<DataPoint>,
    /// Chart configuration
    pub config: HashMap<String, String>,
    /// SVG or other format
    pub rendered: Option<String>,
}

/// Visualization types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizationType {
    LineChart,
    BarChart,
    PieChart,
    ScatterPlot,
    Heatmap,
    Network,
    TreeMap,
    Spatial3D,
}

/// Data point for visualization
#[derive(Debug, Clone)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
    pub z: Option<f64>,
    pub label: Option<String>,
    pub value: Option<f64>,
    pub color: Option<String>,
}

/// Action output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionOutput {
    /// Action type
    pub action_type: String,
    /// Target
    pub target: String,
    /// Parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Confidence in action
    pub confidence: f32,
    /// Requires confirmation
    pub requires_confirmation: bool,
}

/// Output metadata
#[derive(Debug, Clone, Default)]
pub struct OutputMetadata {
    /// Generation time in milliseconds
    pub generation_time_ms: u64,
    /// Output size in bytes
    pub size_bytes: usize,
    /// Source ID
    pub source_id: String,
    /// Pipeline stages completed
    pub stages_completed: Vec<String>,
    /// Total pipeline time
    pub total_pipeline_time_ms: u64,
}

/// The Output Layer
pub struct OutputLayer {
    config: OutputConfig,
    /// Output statistics
    stats: RwLock<OutputStats>,
}

/// Output statistics
#[derive(Debug, Clone, Default)]
pub struct OutputStats {
    pub total_outputs: u64,
    pub by_format: HashMap<String, u64>,
    pub avg_generation_time_ms: f64,
    pub total_bytes_generated: u64,
}

impl OutputLayer {
    /// Create new output layer
    pub fn new(config: OutputConfig) -> Self {
        Self {
            config,
            stats: RwLock::new(OutputStats::default()),
        }
    }
    
    /// Generate output from reasoning result
    pub async fn generate(&self, reasoning: ReasoningResult) -> Result<OutputResult, PipelineError> {
        let start = std::time::Instant::now();
        
        // Determine output format
        let format = self.determine_format(&reasoning);
        
        // Generate content based on format
        let content = self.generate_content(&reasoning, format)?;
        
        // Generate summary
        let summary = self.generate_summary(&reasoning);
        
        // Calculate total pipeline time
        let total_time = reasoning.processed.envelope.total_processing_time().as_millis() as u64
            + reasoning.metadata.reasoning_time_ms
            + start.elapsed().as_millis() as u64;
        
        // Create output
        let output_id = format!("out_{}", reasoning.processed.envelope.id);
        let size_bytes = self.estimate_size(&content);
        
        let metadata = OutputMetadata {
            generation_time_ms: start.elapsed().as_millis() as u64,
            size_bytes,
            source_id: reasoning.processed.envelope.source.resource_id.clone(),
            stages_completed: vec![
                "input".to_string(),
                "inference".to_string(),
                "processing".to_string(),
                "intelligence".to_string(),
                "output".to_string(),
            ],
            total_pipeline_time_ms: total_time,
        };
        
        // Update stats
        {
            let mut stats = self.stats.write();
            stats.total_outputs += 1;
            *stats.by_format.entry(format!("{:?}", format)).or_insert(0) += 1;
            stats.avg_generation_time_ms = (stats.avg_generation_time_ms * (stats.total_outputs - 1) as f64 
                + metadata.generation_time_ms as f64) / stats.total_outputs as f64;
            stats.total_bytes_generated += size_bytes as u64;
        }
        
        // Extract values before moving reasoning
        let confidence = reasoning.confidence;
        let elp = reasoning.elp;
        let confidence = reasoning.confidence;
        
        Ok(OutputResult {
            id: output_id,
            format,
            content,
            reasoning,
            summary,
            confidence,
            elp,
            metadata,
        })
    }
    
    /// Determine best output format
    fn determine_format(&self, reasoning: &ReasoningResult) -> OutputFormat {
        // Check context for format hints
        if let Some(format_hint) = reasoning.processed.envelope.context.attributes.get("output_format") {
            match format_hint.to_lowercase().as_str() {
                "json" => return OutputFormat::Json,
                "text" => return OutputFormat::Text,
                "markdown" | "md" => return OutputFormat::Markdown,
                "html" => return OutputFormat::Html,
                "csv" => return OutputFormat::Csv,
                _ => {}
            }
        }
        
        // Default based on modality
        let modality = &reasoning.processed.envelope.context.modality;
        if modality.contains("text") || modality.contains("markdown") {
            OutputFormat::Markdown
        } else if modality.contains("json") || modality.contains("structured") {
            OutputFormat::Json
        } else {
            self.config.default_format
        }
    }
    
    /// Generate content based on format
    fn generate_content(&self, reasoning: &ReasoningResult, format: OutputFormat) -> Result<OutputContent, PipelineError> {
        match format {
            OutputFormat::Json => self.generate_json(reasoning),
            OutputFormat::Text => self.generate_text(reasoning),
            OutputFormat::Markdown => self.generate_markdown(reasoning),
            OutputFormat::Structured => self.generate_structured(reasoning),
            _ => self.generate_json(reasoning), // Fallback to JSON
        }
    }
    
    /// Generate JSON output
    fn generate_json(&self, reasoning: &ReasoningResult) -> Result<OutputContent, PipelineError> {
        let mut output = serde_json::json!({
            "id": reasoning.processed.envelope.id,
            "confidence": reasoning.confidence,
            "confidence": reasoning.confidence,
            "elp": {
                "ethos": reasoning.elp[0],
                "logos": reasoning.elp[1],
                "pathos": reasoning.elp[2]
            },
            "insights": reasoning.insights.iter().map(|i| serde_json::json!({
                "type": format!("{:?}", i.insight_type),
                "description": i.description,
                "confidence": i.confidence,
                "evidence": i.evidence
            })).collect::<Vec<_>>()
        });
        
        if self.config.include_hypotheses && !reasoning.hypotheses.is_empty() {
            output["hypotheses"] = serde_json::json!(
                reasoning.hypotheses.iter().map(|h| serde_json::json!({
                    "statement": h.statement,
                    "confidence": h.confidence,
                    "predictions": h.predictions
                })).collect::<Vec<_>>()
            );
        }
        
        if self.config.include_reasoning {
            output["reasoning_steps"] = serde_json::json!(reasoning.reasoning_chain.len());
            output["sacred_checkpoints"] = serde_json::json!(reasoning.metadata.sacred_checkpoints);
        }
        
        Ok(OutputContent::Json(output))
    }
    
    /// Generate text output
    fn generate_text(&self, reasoning: &ReasoningResult) -> Result<OutputContent, PipelineError> {
        let mut text = String::new();
        
        text.push_str(&format!("Analysis Results (Confidence: {:.1}%)\n", reasoning.confidence * 100.0));
        text.push_str(&format!("Confidence: {:.2}\n\n", reasoning.confidence));
        
        if !reasoning.insights.is_empty() {
            text.push_str("Insights:\n");
            for insight in &reasoning.insights {
                text.push_str(&format!("  - [{}] {} (confidence: {:.1}%)\n", 
                    format!("{:?}", insight.insight_type),
                    insight.description,
                    insight.confidence * 100.0
                ));
            }
            text.push('\n');
        }
        
        if self.config.include_hypotheses && !reasoning.hypotheses.is_empty() {
            text.push_str("Hypotheses:\n");
            for hyp in &reasoning.hypotheses {
                text.push_str(&format!("  - {} (confidence: {:.1}%)\n", hyp.statement, hyp.confidence * 100.0));
            }
        }
        
        Ok(OutputContent::Text(text))
    }
    
    /// Generate Markdown output
    fn generate_markdown(&self, reasoning: &ReasoningResult) -> Result<OutputContent, PipelineError> {
        let mut md = String::new();
        
        md.push_str("# Analysis Results\n\n");
        md.push_str(&format!("**Confidence:** {:.1}%  \n", reasoning.confidence * 100.0));
        md.push_str(&format!("**Confidence:** {:.2}  \n", reasoning.confidence));
        md.push_str(&format!("**ELP:** E={:.2} L={:.2} P={:.2}\n\n", 
            reasoning.elp[0], reasoning.elp[1], reasoning.elp[2]));
        
        if !reasoning.insights.is_empty() {
            md.push_str("## Insights\n\n");
            for insight in &reasoning.insights {
                md.push_str(&format!("### {} ({:.0}% confidence)\n\n", 
                    format!("{:?}", insight.insight_type),
                    insight.confidence * 100.0
                ));
                md.push_str(&format!("{}\n\n", insight.description));
                if !insight.evidence.is_empty() {
                    md.push_str("**Evidence:**\n");
                    for ev in &insight.evidence {
                        md.push_str(&format!("- {}\n", ev));
                    }
                    md.push('\n');
                }
            }
        }
        
        if self.config.include_hypotheses && !reasoning.hypotheses.is_empty() {
            md.push_str("## Hypotheses\n\n");
            for hyp in &reasoning.hypotheses {
                md.push_str(&format!("### {} ({:.0}% confidence)\n\n", hyp.statement, hyp.confidence * 100.0));
                if !hyp.predictions.is_empty() {
                    md.push_str("**Predictions:**\n");
                    for pred in &hyp.predictions {
                        md.push_str(&format!("- {}\n", pred));
                    }
                    md.push('\n');
                }
            }
        }
        
        if self.config.include_reasoning {
            md.push_str("## Processing Summary\n\n");
            md.push_str(&format!("- **Reasoning Steps:** {}\n", reasoning.reasoning_chain.len()));
            md.push_str(&format!("- **Sacred Checkpoints:** {}\n", reasoning.metadata.sacred_checkpoints));
            md.push_str(&format!("- **VCP Interventions:** {}\n", reasoning.metadata.vcp_interventions));
            md.push_str(&format!("- **Processing Time:** {}ms\n", reasoning.metadata.reasoning_time_ms));
        }
        
        Ok(OutputContent::Text(md))
    }
    
    /// Generate structured output
    fn generate_structured(&self, reasoning: &ReasoningResult) -> Result<OutputContent, PipelineError> {
        let structured = StructuredOutput {
            title: format!("Analysis of {}", reasoning.processed.envelope.source.resource_id),
            summary: self.generate_summary(reasoning),
            insights: reasoning.insights.iter().map(|i| InsightOutput {
                insight_type: format!("{:?}", i.insight_type),
                description: i.description.clone(),
                confidence: i.confidence,
                evidence: i.evidence.clone(),
            }).collect(),
            hypotheses: reasoning.hypotheses.iter().map(|h| HypothesisOutput {
                statement: h.statement.clone(),
                confidence: h.confidence,
                predictions: h.predictions.clone(),
            }).collect(),
            recommendations: self.generate_recommendations(reasoning),
            tables: Vec::new(),
            metrics: self.extract_metrics(reasoning),
        };
        
        Ok(OutputContent::Structured(structured))
    }
    
    /// Generate summary
    fn generate_summary(&self, reasoning: &ReasoningResult) -> String {
        let insight_count = reasoning.insights.len();
        let hyp_count = reasoning.hypotheses.len();
        
        format!(
            "Analyzed data with {:.0}% confidence. Found {} insight(s) and generated {} hypothesis(es). \
            Signal strength: {:.2}. ELP balance: E={:.2}/L={:.2}/P={:.2}.",
            reasoning.confidence * 100.0,
            insight_count,
            hyp_count,
            reasoning.confidence,
            reasoning.elp[0], reasoning.elp[1], reasoning.elp[2]
        )
    }
    
    /// Generate recommendations
    fn generate_recommendations(&self, reasoning: &ReasoningResult) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if reasoning.confidence < 0.6 {
            recommendations.push("Consider providing more context data to improve signal strength.".to_string());
        }
        
        if reasoning.elp[2] > 0.5 {
            recommendations.push("High pathos detected - consider balancing with more logical/factual content.".to_string());
        }
        
        for insight in &reasoning.insights {
            if insight.insight_type == super::intelligence_layer::InsightType::Anomaly {
                recommendations.push(format!("Investigate anomaly: {}", insight.description));
            }
        }
        
        recommendations
    }
    
    /// Extract metrics
    fn extract_metrics(&self, reasoning: &ReasoningResult) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        metrics.insert("confidence".to_string(), reasoning.confidence as f64);
        metrics.insert("confidence".to_string(), reasoning.confidence as f64);
        metrics.insert("ethos".to_string(), reasoning.elp[0] as f64);
        metrics.insert("logos".to_string(), reasoning.elp[1] as f64);
        metrics.insert("pathos".to_string(), reasoning.elp[2] as f64);
        metrics.insert("insight_count".to_string(), reasoning.insights.len() as f64);
        metrics.insert("hypothesis_count".to_string(), reasoning.hypotheses.len() as f64);
        metrics.insert("reasoning_steps".to_string(), reasoning.reasoning_chain.len() as f64);
        
        metrics
    }
    
    /// Estimate output size
    fn estimate_size(&self, content: &OutputContent) -> usize {
        match content {
            OutputContent::Text(t) => t.len(),
            OutputContent::Json(j) => j.to_string().len(),
            OutputContent::Binary(b) => b.len(),
            OutputContent::Structured(s) => serde_json::to_string(s).map(|s| s.len()).unwrap_or(0),
            OutputContent::Visualization(v) => v.rendered.as_ref().map(|r| r.len()).unwrap_or(0),
            OutputContent::Action(a) => serde_json::to_string(a).map(|s| s.len()).unwrap_or(0),
        }
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> OutputStats {
        self.stats.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::intelligence_layer::{ReasoningResult, Insight, InsightType, ReasoningStep, ReasoningMetadata};
    use super::super::processing_layer::{ProcessedData, Features, ProcessingMetadata};
    use super::super::data_types::DataEnvelope;
    
    fn create_test_reasoning() -> ReasoningResult {
        let envelope = DataEnvelope::new("test", super::super::data_types::UniversalData::Text("test".to_string()));
        let processed = ProcessedData {
            envelope,
            features: Features::default(),
            embeddings: None,
            tokens: None,
            metadata: ProcessingMetadata::default(),
        };
        
        ReasoningResult {
            processed,
            insights: vec![Insight {
                insight_type: InsightType::Summary,
                description: "Test insight".to_string(),
                confidence: 0.8,
                evidence: vec!["test".to_string()],
                related_features: Vec::new(),
            }],
            hypotheses: Vec::new(),
            reasoning_chain: vec![ReasoningStep {
                step: 1,
                operation: "test".to_string(),
                input: "test".to_string(),
                output: "test".to_string(),
                confidence: 0.8,
                is_sacred_position: false,
            }],
            elp: [0.33, 0.34, 0.33],
            confidence: 0.8,
            knowledge_updates: Vec::new(),
            metadata: ReasoningMetadata::default(),
        }
    }
    
    #[tokio::test]
    async fn test_output_layer_generate() {
        let layer = OutputLayer::new(OutputConfig::default());
        let reasoning = create_test_reasoning();
        
        let result = layer.generate(reasoning).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.confidence > 0.0);
        assert!(!output.summary.is_empty());
    }
    
    #[test]
    fn test_generate_markdown() {
        let layer = OutputLayer::new(OutputConfig::default());
        let reasoning = create_test_reasoning();
        
        let result = layer.generate_markdown(&reasoning);
        assert!(result.is_ok());
        
        if let OutputContent::Text(md) = result.unwrap() {
            assert!(md.contains("# Analysis Results"));
            assert!(md.contains("Insights"));
        }
    }
}
