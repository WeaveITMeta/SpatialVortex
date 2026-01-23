//! Matrix-Guided Inference
//!
//! Phase 3: Extract knowledge from FluxMatrix positions to enhance LLM prompts
//! Phase 3.5: Response quality enhancement and adaptive modes

use crate::data::models::*;
use crate::error::{Result, SpatialVortexError};
use crate::core::sacred_geometry::FluxMatrixEngine;
use crate::ai::response_quality::{ResponseMode, ResponseQualityAnalyzer};

/// Context extracted from FluxMatrix for inference enhancement
#[derive(Debug, Clone)]
pub struct MatrixInferenceContext {
    /// Primary subject being queried
    pub subject: String,
    
    /// Position in flux matrix (0-9)
    pub position: u8,
    
    /// Whether this is a sacred position (3, 6, 9)
    pub is_sacred: bool,
    
    /// Positive semantic associations at this position
    pub positive_associations: Vec<SemanticAssociation>,
    
    /// Negative semantic associations at this position
    pub negative_associations: Vec<SemanticAssociation>,
    
    /// Neutral base concept name
    pub neutral_base: String,
    
    /// ELP tensor for this position
    pub elp_context: ELPTensor,
    
    /// Related subjects (cross-references)
    pub related_subjects: Vec<String>,
    
    /// Divine properties (if sacred position)
    pub divine_properties: Option<Vec<String>>,
}

impl MatrixInferenceContext {
    /// Create context from a flux node
    pub fn from_node(node: &FluxNode, subject: &str, sacred_guide: Option<&SacredGuide>) -> Self {
        let is_sacred = matches!(node.position, 3 | 6 | 9);
        
        // Extract related subjects from cross-references
        let related_subjects = node.semantic_index.positive_associations
            .iter()
            .filter(|assoc| assoc.word.contains('-') || assoc.word.len() > 10)
            .map(|assoc| assoc.word.clone())
            .collect();
        
        // Get divine properties if sacred
        let divine_properties = sacred_guide.map(|guide| guide.divine_properties.clone());
        
        Self {
            subject: subject.to_string(),
            position: node.position,
            is_sacred,
            positive_associations: node.semantic_index.positive_associations.clone(),
            negative_associations: node.semantic_index.negative_associations.clone(),
            neutral_base: node.semantic_index.neutral_base.clone(),
            elp_context: ELPTensor {
                ethos: node.attributes.properties.get("ethos")
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0),
                logos: node.attributes.properties.get("logos")
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0),
                pathos: node.attributes.properties.get("pathos")
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0),
            },
            related_subjects,
            divine_properties,
        }
    }
    
    /// Build enhanced prompt with matrix context
    pub fn build_enhanced_prompt(&self, user_query: &str) -> String {
        let mut prompt = String::new();
        
        // Add base query
        prompt.push_str(&format!("Query: {}\n\n", user_query));
        
        // Add matrix position context
        prompt.push_str(&format!("--- Matrix Context (Position {}) ---\n", self.position));
        prompt.push_str(&format!("Subject: {}\n", self.subject));
        prompt.push_str(&format!("Core Concept: {}\n", self.neutral_base));
        
        // Add ELP context
        if self.is_sacred {
            prompt.push_str(&format!("🌟 SACRED POSITION {} 🌟\n", self.position));
            if let Some(ref divine_props) = self.divine_properties {
                prompt.push_str(&format!("Divine Properties: {}\n", divine_props.join(", ")));
            }
        }
        
        prompt.push_str(&format!(
            "ELP Balance → Ethos: {:.1}, Logos: {:.1}, Pathos: {:.1}\n",
            self.elp_context.ethos, self.elp_context.logos, self.elp_context.pathos
        ));
        
        // Add top positive associations
        if !self.positive_associations.is_empty() {
            prompt.push_str("\nPositive Associations:\n");
            for (i, assoc) in self.positive_associations.iter().take(10).enumerate() {
                prompt.push_str(&format!(
                    "  {}. {} (confidence: {:.2}, strength: {})\n",
                    i + 1, assoc.word, assoc.confidence, assoc.index
                ));
            }
        }
        
        // Add negative associations (what to avoid)
        if !self.negative_associations.is_empty() {
            prompt.push_str("\nAvoid (Negative Associations):\n");
            for (i, assoc) in self.negative_associations.iter().take(5).enumerate() {
                prompt.push_str(&format!(
                    "  {}. {} (confidence: {:.2})\n",
                    i + 1, assoc.word, assoc.confidence
                ));
            }
        }
        
        // Add related subjects
        if !self.related_subjects.is_empty() {
            prompt.push_str(&format!(
                "\nRelated Topics: {}\n",
                self.related_subjects.join(", ")
            ));
        }
        
        prompt.push_str("\n--- End Matrix Context ---\n\n");
        prompt.push_str("Respond considering the above semantic context and associations.\n");
        
        prompt
    }
    
    /// Build a concise context summary for injection
    pub fn build_context_summary(&self) -> String {
        let mut summary = format!(
            "[Matrix Context: {} at position {} ({})]",
            self.neutral_base,
            self.position,
            if self.is_sacred { "SACRED" } else { "regular" }
        );
        
        // Add top 3 positive associations
        if !self.positive_associations.is_empty() {
            let top_assocs: Vec<_> = self.positive_associations
                .iter()
                .take(3)
                .map(|a| a.word.as_str())
                .collect();
            summary.push_str(&format!(" Related: {}", top_assocs.join(", ")));
        }
        
        summary
    }
}

/// Matrix-guided inference engine
pub struct MatrixGuidedInference {
    flux_engine: FluxMatrixEngine,
    quality_analyzer: ResponseQualityAnalyzer,
}

impl MatrixGuidedInference {
    /// Create new matrix-guided inference engine
    pub fn new(flux_engine: FluxMatrixEngine) -> Self {
        Self {
            flux_engine,
            quality_analyzer: ResponseQualityAnalyzer::new(),
        }
    }
    
    /// Build adaptive prompt with quality awareness
    pub fn build_adaptive_prompt(
        &self,
        user_query: &str,
        subject: &str,
    ) -> Result<(String, ResponseMode)> {
        // Extract matrix context
        let context = self.extract_context(user_query, subject)?;
        
        // Determine appropriate response mode
        let mode = self.quality_analyzer.determine_mode(user_query, Some(&context));
        
        // Build context-aware prompt
        let prompt = self.quality_analyzer.build_prompt(user_query, &context, mode);
        
        Ok((prompt, mode))
    }
    
    /// Extract inference context from matrix for a query
    pub fn extract_context(
        &self,
        user_query: &str,
        subject: &str,
    ) -> Result<MatrixInferenceContext> {
        // Create matrix for subject
        let matrix = self.flux_engine.create_matrix(subject.to_string())?;
        
        // Find best position for query
        let (position, _confidence) = self.flux_engine.find_best_position(user_query, subject)?;
        
        // Get node at position
        let node = matrix.nodes.get(&position)
            .ok_or_else(|| SpatialVortexError::InvalidFluxMatrix(
                format!("No node at position {}", position)
            ))?;
        
        // Get sacred guide if applicable
        let sacred_guide = matrix.sacred_guides.get(&position);
        
        // Build context
        Ok(MatrixInferenceContext::from_node(node, subject, sacred_guide))
    }
    
    /// Enhance a query with matrix context
    pub fn enhance_query(
        &self,
        user_query: &str,
        subject: &str,
        enhancement_mode: EnhancementMode,
    ) -> Result<String> {
        let context = self.extract_context(user_query, subject)?;
        
        match enhancement_mode {
            EnhancementMode::Full => Ok(context.build_enhanced_prompt(user_query)),
            EnhancementMode::Summary => {
                let summary = context.build_context_summary();
                Ok(format!("{}\n\n{}", summary, user_query))
            }
            EnhancementMode::Minimal => {
                let top_assocs: Vec<_> = context.positive_associations
                    .iter()
                    .take(3)
                    .map(|a| a.word.as_str())
                    .collect();
                Ok(format!(
                    "[Context: {}]\n{}",
                    top_assocs.join(", "),
                    user_query
                ))
            }
        }
    }
    
    /// Analyze response quality using matrix context
    pub fn analyze_response_quality(
        &self,
        user_query: &str,
        response: &str,
        subject: &str,
    ) -> Result<ResponseQualityAnalysis> {
        let context = self.extract_context(user_query, subject)?;
        
        // Count positive association mentions
        let mut positive_matches = 0;
        let mut positive_matched = Vec::new();
        for assoc in &context.positive_associations {
            if response.to_lowercase().contains(&assoc.word.to_lowercase()) {
                positive_matches += 1;
                positive_matched.push(assoc.word.clone());
            }
        }
        
        // Count negative association mentions (should be low)
        let mut negative_matches = 0;
        let mut negative_matched = Vec::new();
        for assoc in &context.negative_associations {
            if response.to_lowercase().contains(&assoc.word.to_lowercase()) {
                negative_matches += 1;
                negative_matched.push(assoc.word.clone());
            }
        }
        
        // Calculate quality score
        let positive_score = if !context.positive_associations.is_empty() {
            positive_matches as f32 / context.positive_associations.len() as f32
        } else {
            0.5
        };
        
        let negative_penalty = if !context.negative_associations.is_empty() {
            negative_matches as f32 / context.negative_associations.len() as f32
        } else {
            0.0
        };
        
        let quality_score = (positive_score - negative_penalty * 0.5).max(0.0).min(1.0);
        
        // Determine quality level
        let quality_level = if quality_score >= 0.8 {
            QualityLevel::Excellent
        } else if quality_score >= 0.6 {
            QualityLevel::Good
        } else if quality_score >= 0.4 {
            QualityLevel::Fair
        } else {
            QualityLevel::Poor
        };
        
        Ok(ResponseQualityAnalysis {
            quality_score,
            quality_level,
            positive_matches,
            negative_matches,
            positive_matched,
            negative_matched,
            position: context.position,
            is_sacred: context.is_sacred,
        })
    }
}

/// Enhancement mode for matrix-guided prompts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnhancementMode {
    /// Full detailed context with all associations
    Full,
    /// Summary with top associations
    Summary,
    /// Minimal context (top 3 associations only)
    Minimal,
}

/// Response quality analysis result
#[derive(Debug, Clone)]
pub struct ResponseQualityAnalysis {
    /// Overall quality score (0.0-1.0)
    pub quality_score: f32,
    
    /// Quality level classification
    pub quality_level: QualityLevel,
    
    /// Number of positive associations found
    pub positive_matches: usize,
    
    /// Number of negative associations found (should be low)
    pub negative_matches: usize,
    
    /// Which positive associations were matched
    pub positive_matched: Vec<String>,
    
    /// Which negative associations were matched
    pub negative_matched: Vec<String>,
    
    /// Position used for context
    pub position: u8,
    
    /// Whether response came from sacred position
    pub is_sacred: bool,
}

/// Quality level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityLevel {
    Excellent,  // >= 0.8
    Good,       // >= 0.6
    Fair,       // >= 0.4
    Poor,       // < 0.4
}

impl std::fmt::Display for QualityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QualityLevel::Excellent => write!(f, "⭐ Excellent"),
            QualityLevel::Good => write!(f, "✓ Good"),
            QualityLevel::Fair => write!(f, "• Fair"),
            QualityLevel::Poor => write!(f, "⚠ Poor"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_summary() {
        let context = MatrixInferenceContext {
            subject: "cognition".to_string(),
            position: 4,
            is_sacred: false,
            positive_associations: vec![
                SemanticAssociation::new("reasoning".to_string(), 2, 0.95),
                SemanticAssociation::new("logic".to_string(), 2, 0.9),
            ],
            negative_associations: vec![],
            neutral_base: "Logical Cognition".to_string(),
            elp_context: ELPTensor::new(4.0, 6.0, 3.0),
            related_subjects: vec![],
            divine_properties: None,
        };
        
        let summary = context.build_context_summary();
        assert!(summary.contains("Logical Cognition"));
        assert!(summary.contains("position 4"));
        assert!(summary.contains("reasoning"));
    }
    
    #[test]
    fn test_sacred_context() {
        let context = MatrixInferenceContext {
            subject: "cognition".to_string(),
            position: 9,
            is_sacred: true,
            positive_associations: vec![],
            negative_associations: vec![],
            neutral_base: "Meta-Cognition".to_string(),
            elp_context: ELPTensor::new(6.0, 9.0, 6.0),
            related_subjects: vec![],
            divine_properties: Some(vec!["universal-reasoning".to_string()]),
        };
        
        let summary = context.build_context_summary();
        assert!(summary.contains("SACRED"));
    }
}
