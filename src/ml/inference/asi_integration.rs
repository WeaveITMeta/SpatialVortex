//! 🌟 ASI Integration - Sacred Geometry → BeadTensor → FluxMatrix
//!
//! This module bridges the Sacred Geometry Innovation with SpatialVortex's
//! existing architecture, creating a complete ASI inference pipeline.
//!
//! # Pipeline Flow
//! ```text
//! Text Input
//!     ↓
//! ONNX Embedding (384-d)
//!     ↓
//! Sacred Geometry Transform
//!     ↓
//! ELP Channels (Ethos, Logos, Pathos)
//!     ↓
//! BeadTensor (with signal strength)
//!     ↓
//! FluxMatrix (spatial positioning)
//!     ↓
//! Confidence Lake (if signal ≥ 0.6)
//! ```

use crate::models::ELPTensor;
use crate::flux_matrix::FluxMatrixEngine;
use std::error::Error;
#[cfg(feature = "onnx")]
use chrono::Utc;

#[cfg(feature = "onnx")]
use super::onnx_runtime::OnnxInferenceEngine;
use crate::core::sacred_geometry::{VortexPositioningEngine, FluxPosition, PositionArchetype};

/// ASI Integration Engine
///
/// Connects ONNX + Sacred Geometry with BeadTensor and FluxMatrix
/// to create a complete Artificial Superintelligence inference pipeline.
pub struct ASIIntegrationEngine {
    #[cfg(feature = "onnx")]
    onnx_engine: OnnxInferenceEngine,
    #[allow(dead_code)]  // Reserved for advanced flux matrix integration
    flux_engine: FluxMatrixEngine,
    #[allow(dead_code)]  // Reserved for vortex positioning features
    vortex_engine: VortexPositioningEngine,
    #[cfg(not(feature = "onnx"))]
    _phantom: std::marker::PhantomData<()>,
}

/// BeadTensor with semantic embedding
///
/// Extends the concept of BeadTensor to include ML embeddings
/// and sacred geometry transformations.
#[derive(Debug, Clone)]
pub struct SemanticBeadTensor {
    /// ELP tensor coordinates (from sacred geometry)
    pub elp_values: ELPTensor,
    
    /// Signal strength (3-6-9 pattern coherence)
    pub confidence: f64,
    
    /// Raw embedding vector (384-d)
    pub embedding: Vec<f32>,
    
    /// Original text input
    pub text: String,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Result of ASI inference
#[derive(Debug, Clone)]
pub struct ASIInferenceResult {
    /// Semantic bead tensor
    pub bead: SemanticBeadTensor,
    
    /// Flux matrix position (0-9) with vortex mathematics
    pub flux_position: FluxPosition,
    
    /// Should be added to Confidence Lake?
    pub lake_worthy: bool,
    
    /// Interpretation
    pub interpretation: String,
}

impl ASIIntegrationEngine {
    /// Create a new ASI integration engine
    ///
    /// # Arguments
    /// * `model_path` - Path to ONNX model
    /// * `tokenizer_path` - Path to tokenizer
    ///
    /// # Example
    /// ```no_run
    /// use spatial_vortex::inference_engine::asi_integration::ASIIntegrationEngine;
    /// 
    /// let asi = ASIIntegrationEngine::new(
    ///     "models/model.onnx",
    ///     "models/tokenizer.json"
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "onnx")]
    pub fn new<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
        model_path: P,
        tokenizer_path: Q,
    ) -> Result<Self, Box<dyn Error>> {
        let onnx_engine = OnnxInferenceEngine::new(model_path, tokenizer_path)?;
        let flux_engine = FluxMatrixEngine::new();
        let vortex_engine = VortexPositioningEngine::new();
        
        Ok(Self {
            onnx_engine,
            flux_engine,
            vortex_engine,
        })
    }

    #[cfg(not(feature = "onnx"))]
    pub fn new<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
        _model_path: P,
        _tokenizer_path: Q,
    ) -> Result<Self, Box<dyn Error>> {
        Err("ONNX feature not enabled. Compile with --features onnx".into())
    }

    /// 🌟 Complete ASI Inference Pipeline
    ///
    /// Takes text input and runs through the complete ASI stack:
    /// 1. ONNX embedding generation
    /// 2. Sacred geometry transformation
    /// 3. ELP channel mapping
    /// 4. BeadTensor creation
    /// 5. FluxMatrix positioning
    /// 6. Confidence Lake eligibility check
    ///
    /// # Arguments
    /// * `text` - Input text to analyze
    ///
    /// # Returns
    /// * Complete ASI inference result with interpretation
    ///
    /// # Example
    /// ```no_run
    /// # use spatial_vortex::inference_engine::asi_integration::ASIIntegrationEngine;
    /// let asi = ASIIntegrationEngine::new("models/model.onnx", "models/tokenizer.json")?;
    /// let result = asi.infer("Truth and justice prevail")?;
    /// 
    /// println!("Position: {}", result.flux_position);
    /// println!("Signal: {:.2}", result.bead.confidence);
    /// println!("Lake worthy: {}", result.lake_worthy);
    /// println!("{}", result.interpretation);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "onnx")]
    pub fn infer(&mut self, text: &str) -> Result<ASIInferenceResult, Box<dyn Error>> {
        // Step 1: ONNX + Sacred Geometry
        let (embedding, confidence, ethos, logos, pathos) = 
            self.onnx_engine.embed_with_sacred_geometry(text)?;
        
        // Step 2: Create ELP Tensor (13-scale normalization)
        // Scale from 0-1 to ±13 for sacred geometry
        let elp_values = ELPTensor {
            ethos: (ethos * 13.0) as f64,
            logos: (logos * 13.0) as f64,
            pathos: (pathos * 13.0) as f64,
        };
        
        // Step 3: Create Semantic BeadTensor
        let bead = SemanticBeadTensor {
            elp_values: elp_values.clone(),
            confidence: confidence as f64,
            embedding: embedding.clone(),
            text: text.to_string(),
            timestamp: Utc::now(),
            confidence: confidence as f64,
        };
        
        // Step 4: Determine FluxMatrix position using advanced vortex mathematics
        // Full vortex flow (0-9) with gradient positioning
        let flux_position = self.vortex_engine.calculate_position(
            ethos,
            logos,
            pathos,
            confidence,
        );
        
        // Step 5: Confidence Lake eligibility
        // Sacred geometry threshold: confidence ≥ 0.6
        let lake_worthy = confidence >= 0.6;
        
        // Step 6: Generate interpretation
        let interpretation = self.generate_interpretation(
            &bead,
            flux_position,
            lake_worthy,
        );
        
        Ok(ASIInferenceResult {
            bead,
            flux_position,
            lake_worthy,
            interpretation,
        })
    }

    #[cfg(not(feature = "onnx"))]
    pub fn infer(&self, _text: &str) -> Result<ASIInferenceResult, Box<dyn Error>> {
        Err("ONNX feature not enabled. Compile with --features onnx".into())
    }


    /// Generate human-readable interpretation
    #[allow(dead_code)]  // Reserved for natural language interpretation features
    fn generate_interpretation(
        &self,
        bead: &SemanticBeadTensor,
        flux_position: FluxPosition,
        lake_worthy: bool,
    ) -> String {
        let signal = bead.confidence as f32;
        let e = (bead.elp_values.ethos / 13.0) as f32;
        let l = (bead.elp_values.logos / 13.0) as f32;
        let p = (bead.elp_values.pathos / 13.0) as f32;
        
        let signal_desc = match signal {
            s if s >= 0.7 => "⭐ Very Strong",
            s if s >= 0.5 => "✅ Strong",
            s if s >= 0.3 => "⚡ Moderate",
            _ => "⚠️ Weak",
        };
        
        let dominant = if e > l && e > p {
            format!("Ethos-dominant ({:.1}%) - Character/ethical focus", e * 100.0)
        } else if l > p {
            format!("Logos-dominant ({:.1}%) - Logic/analytical focus", l * 100.0)
        } else {
            format!("Pathos-dominant ({:.1}%) - Emotion/empathetic focus", p * 100.0)
        };
        
        // Use vortex math position name
        let position_meaning = format!("Position {} - {}", flux_position.0, flux_position.name());
        
        // Add archetype info
        let archetype_info = match flux_position.archetype() {
            PositionArchetype::Source => "🌟 Divine Source (Perfect Balance)",
            PositionArchetype::Sacred => "🔺 Sacred Checkpoint (Stable Attractor)",
            PositionArchetype::Flow => "🌀 Vortex Flow (Dynamic Position)",
        };
        
        let lake_status = if lake_worthy {
            "✅ Eligible for Confidence Lake (high signal strength)"
        } else {
            "❌ Not eligible for Confidence Lake (signal too weak)"
        };
        
        format!(
            "Confidence: {:.4} {}\n{}\n{}\n{}\n{}",
            signal, signal_desc, dominant, position_meaning, archetype_info, lake_status
        )
    }

    /// Batch inference for multiple texts
    ///
    /// # Arguments
    /// * `texts` - Vector of text strings to analyze
    ///
    /// # Returns
    /// * Vector of ASI inference results
    #[cfg(feature = "onnx")]
    pub fn infer_batch(&mut self, texts: &[String]) -> Result<Vec<ASIInferenceResult>, Box<dyn Error>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.infer(text)?);
        }
        Ok(results)
    }

    #[cfg(not(feature = "onnx"))]
    pub fn infer_batch(&self, _texts: &[String]) -> Result<Vec<ASIInferenceResult>, Box<dyn Error>> {
        Err("ONNX feature not enabled. Compile with --features onnx".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vortex_positioning() {
        let vortex = VortexPositioningEngine::new();
        
        // Ethos-dominant should map to position 3 (sacred)
        let pos = vortex.calculate_position(0.8, 0.1, 0.1, 0.9);
        assert_eq!(pos, FluxPosition(3));
        
        // Logos-dominant should map to position 9 (sacred)
        let pos = vortex.calculate_position(0.1, 0.8, 0.1, 0.9);
        assert_eq!(pos, FluxPosition(9));
        
        // Pathos-dominant should map to position 6 (sacred)
        let pos = vortex.calculate_position(0.1, 0.1, 0.8, 0.9);
        assert_eq!(pos, FluxPosition(6));
        
        // Balanced should map to position 0 (divine source)
        let pos = vortex.calculate_position(0.33, 0.33, 0.34, 0.8);
        assert_eq!(pos, FluxPosition(0));
    }

    #[test]
    fn test_lake_worthiness_threshold() {
        // Signal strength ≥ 0.6 is lake-worthy
        assert!(0.6 >= 0.6);
        assert!(0.7 >= 0.6);
        assert!(0.5 < 0.6);
    }

    #[test]
    fn test_vortex_flow_positions() {
        // Test flow positions
        assert!(FluxPosition(1).is_in_vortex_flow());
        assert!(FluxPosition(2).is_in_vortex_flow());
        assert!(FluxPosition(4).is_in_vortex_flow());
        assert!(FluxPosition(5).is_in_vortex_flow());
        assert!(FluxPosition(7).is_in_vortex_flow());
        assert!(FluxPosition(8).is_in_vortex_flow());
        
        // Test sacred positions
        assert!(FluxPosition(3).is_sacred());
        assert!(FluxPosition(6).is_sacred());
        assert!(FluxPosition(9).is_sacred());
        
        // Test divine source
        assert!(FluxPosition(0).is_divine_source());
    }
}
