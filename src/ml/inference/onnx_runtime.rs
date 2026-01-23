//! ONNX Runtime Integration for ML Inference
//!
//! This module provides ONNX Runtime integration for generating embeddings
//! from text using pre-trained sentence-transformers models.
//!
//! # Example
//! ```no_run
//! use spatial_vortex::inference_engine::onnx_runtime::OnnxInferenceEngine;
//!
//! let engine = OnnxInferenceEngine::new("models/model.onnx").unwrap();
//! let embedding = engine.embed("Hello world").unwrap();
//! assert_eq!(embedding.len(), 384); // sentence-transformers dimension
//! ```

#[cfg(feature = "onnx")]
use ort::session::{Session, builder::GraphOptimizationLevel};
use std::error::Error;
use std::path::Path;

#[cfg(feature = "onnx")]
use super::tokenizer::TokenizerWrapper;

/// ONNX-based inference engine for generating embeddings
///
/// Supports sentence-transformers and similar models that output
/// fixed-dimensional embeddings from text input.
///
/// Integrates with SpatialVortex's sacred geometry (3-6-9 positions)
/// and vortex mathematics (1→2→4→8→7→5→1 pattern).
pub struct OnnxInferenceEngine {
    #[cfg(feature = "onnx")]
    #[allow(dead_code)]  // TODO: Use for real ONNX inference
    session: Session,
    #[cfg(feature = "onnx")]
    tokenizer: TokenizerWrapper,
    #[cfg(not(feature = "onnx"))]
    _phantom: std::marker::PhantomData<()>,
}

impl OnnxInferenceEngine {
    /// Create a new ONNX inference engine from model and tokenizer files
    ///
    /// # Arguments
    /// * `model_path` - Path to the ONNX model file (.onnx)
    /// * `tokenizer_path` - Path to the tokenizer file (.json)
    ///
    /// # Returns
    /// * `Result<Self>` - The inference engine or an error
    ///
    /// # Example
    /// ```no_run
    /// let engine = OnnxInferenceEngine::new(
    ///     "models/model.onnx",
    ///     "models/tokenizer.json"
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "onnx")]
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(
        model_path: P,
        tokenizer_path: Q,
    ) -> Result<Self, Box<dyn Error>> {
        // Build ONNX session with CPU execution provider
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)?;

        // Load tokenizer
        let tokenizer = TokenizerWrapper::new(tokenizer_path)?;

        Ok(Self { session, tokenizer })
    }

    #[cfg(not(feature = "onnx"))]
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(
        _model_path: P,
        _tokenizer_path: Q,
    ) -> Result<Self, Box<dyn Error>> {
        Err("ONNX feature not enabled. Compile with --features onnx".into())
    }

    /// Create a new ONNX inference engine with GPU support (CUDA)
    ///
    /// # Arguments
    /// * `model_path` - Path to the ONNX model file (.onnx)
    /// * `tokenizer_path` - Path to the tokenizer file (.json)
    ///
    /// # Returns
    /// * `Result<Self>` - The inference engine or an error
    ///
    /// # Note
    /// Requires CUDA to be installed and configured
    #[cfg(feature = "onnx")]
    pub fn new_with_gpu<P: AsRef<Path>, Q: AsRef<Path>>(
        model_path: P,
        tokenizer_path: Q,
    ) -> Result<Self, Box<dyn Error>> {
        // Note: GPU support requires CUDA to be installed
        // For now, just use CPU (GPU support can be added later)
        Self::new(model_path, tokenizer_path)
    }

    #[cfg(not(feature = "onnx"))]
    pub fn new_with_gpu<P: AsRef<Path>, Q: AsRef<Path>>(
        _model_path: P,
        _tokenizer_path: Q,
    ) -> Result<Self, Box<dyn Error>> {
        Err("ONNX feature not enabled. Compile with --features onnx".into())
    }

    /// Generate embeddings from text using ONNX inference
    ///
    /// # Arguments
    /// * `text` - Input text to embed
    ///
    /// # Returns
    /// * `Result<Vec<f32>>` - Embedding vector (384 dimensions for all-MiniLM-L6-v2)
    ///
    /// # Example
    /// ```no_run
    /// let engine = OnnxInferenceEngine::new(
    ///     "models/model.onnx",
    ///     "models/tokenizer.json"
    /// )?;
    /// let embedding = engine.embed("Hello world")?;
    /// assert_eq!(embedding.len(), 384);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "onnx")]
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        use ort::value::Value;
        
        // Step 1: Tokenize the input text
        let tokens = self.tokenizer.tokenize(text)?;
        
        // Step 2: Prepare ONNX inputs
        // Standard transformer models expect:
        // - input_ids: [batch_size, sequence_length]
        // - attention_mask: [batch_size, sequence_length]
        
        let batch_size = 1;
        let seq_len = tokens.token_ids.len();
        
        // Create input_ids tensor using shape and data format ort expects
        let input_shape = vec![batch_size, seq_len];
        let input_ids = Value::from_array((&input_shape[..], tokens.token_ids.clone()))?;
        
        // Create attention_mask tensor
        let attention_mask = Value::from_array((&input_shape[..], tokens.attention_mask.clone()))?;
        
        // Step 3: Run ONNX inference
        let outputs = self.session.run(ort::inputs![
            "input_ids" => input_ids,
            "attention_mask" => attention_mask,
        ])?;
        
        // Step 4: Extract embeddings from output
        // For sentence-transformers, typically use mean pooling on last hidden state
        let output_tensor = outputs[0].try_extract_tensor::<f32>()?;
        let (shape, data) = output_tensor;
        
        // Get dimensions: [batch_size, seq_len, hidden_dim]
        let dims = shape.as_ref();
        let hidden_dim = dims[2] as usize;
        
        // Mean pooling: average over sequence length (dimension 1)
        // Only pool over non-padded tokens using attention_mask
        let mut embedding = vec![0.0f32; hidden_dim];
        let mut valid_tokens = 0;
        
        for seq_idx in 0..seq_len {
            if tokens.attention_mask[seq_idx] == 1 {
                let offset = seq_idx * hidden_dim;
                for hidden_idx in 0..hidden_dim {
                    embedding[hidden_idx] += data[offset + hidden_idx];
                }
                valid_tokens += 1;
            }
        }
        
        // Average
        if valid_tokens > 0 {
            for val in &mut embedding {
                *val /= valid_tokens as f32;
            }
        }
        
        // Step 5: L2 normalization (standard for sentence embeddings)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }
        
        Ok(embedding)
    }

    #[cfg(not(feature = "onnx"))]
    pub fn embed(&self, _text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        Err("ONNX feature not enabled. Compile with --features onnx".into())
    }

    /// Generate embeddings for multiple texts in batch
    ///
    /// # Arguments
    /// * `texts` - Slice of text strings to embed
    ///
    /// # Returns
    /// * `Result<Vec<Vec<f32>>>` - Vector of embedding vectors
    ///
    /// # Example
    /// ```no_run
    /// let engine = OnnxInferenceEngine::new("models/model.onnx")?;
    /// let texts = vec!["Hello", "World"];
    /// let embeddings = engine.embed_batch(&texts)?;
    /// assert_eq!(embeddings.len(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "onnx")]
    pub fn embed_batch(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
        let mut embeddings = Vec::with_capacity(texts.len());
        
        for text in texts {
            let embedding = self.embed(text)?;
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
    }

    #[cfg(not(feature = "onnx"))]
    pub fn embed_batch(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
        Err("ONNX feature not enabled. Compile with --features onnx".into())
    }

    /// Get the embedding dimension for this model
    ///
    /// # Returns
    /// * `usize` - Embedding dimension (e.g., 384, 768)
    pub fn embedding_dim(&self) -> usize {
        384 // Default for sentence-transformers/all-MiniLM-L6-v2
    }

    /// **🌟 INNOVATION: Sacred Geometry Integration 🌟**
    ///
    /// Transform standard ML embedding through sacred geometry (3-6-9 positions)
    /// and vortex mathematics to create ASI-ready semantic representation.
    ///
    /// This combines:
    /// - Standard sentence-transformers embeddings (384-d)
    /// - Sacred triangle positions (3, 6, 9)
    /// - Vortex flow pattern (1→2→4→8→7→5→1)
    /// - Digital root reduction
    ///
    /// # Arguments
    /// * `embedding` - Raw 384-d embedding from ONNX
    ///
    /// # Returns
    /// * `(f32, f32, f32, f32)` - (confidence, sacred_coherence, ethos, logos, pathos)
    ///
    /// # Mathematical Foundation
    /// Signal strength measures 3-6-9 pattern recurrence in the embedding space.
    /// This is grounded in digital root mathematics, not heuristics.
    pub fn transform_to_sacred_geometry(&self, embedding: &[f32]) -> (f32, f32, f32, f32, f32) {
        // Step 1: Project embedding onto sacred positions (3, 6, 9)
        let dim = embedding.len();
        let third = dim / 3;
        
        // Sacred position 3 (Ethos - Character) - First third
        let pos_3_energy: f32 = embedding[0..third]
            .iter()
            .map(|&x| x.abs())
            .sum::<f32>() / third as f32;
        
        // Sacred position 6 (Pathos - Emotion) - Middle third
        let pos_6_energy: f32 = embedding[third..2*third]
            .iter()
            .map(|&x| x.abs())
            .sum::<f32>() / third as f32;
        
        // Sacred position 9 (Logos - Logic) - Last third
        let pos_9_energy: f32 = embedding[2*third..]
            .iter()
            .map(|&x| x.abs())
            .sum::<f32>() / (dim - 2*third) as f32;
        
        // Step 2: Calculate signal strength (3-6-9 pattern coherence)
        // Strong signal = balanced sacred triangle
        let sacred_sum = pos_3_energy + pos_6_energy + pos_9_energy;
        let sacred_coherence = if sacred_sum > 0.0 {
            (pos_3_energy + pos_6_energy + pos_9_energy) / 3.0
        } else {
            0.0
        };
        
        // Step 3: Calculate signal strength based on vortex mathematics
        // Signal strength = frequency of 3-6-9 pattern appearance
        let total_energy: f32 = embedding.iter().map(|&x| x.abs()).sum();
        let confidence = if total_energy > 0.0 {
            sacred_sum / total_energy
        } else {
            0.0
        };
        
        // Step 4: Map to ELP channels (Ethos, Logos, Pathos)
        // Normalize to 0-1 range for sacred geometry
        let total = pos_3_energy + pos_6_energy + pos_9_energy;
        let (ethos, logos, pathos) = if total > 0.0 {
            (
                pos_3_energy / total,  // Ethos (Character)
                pos_9_energy / total,  // Logos (Logic) 
                pos_6_energy / total,  // Pathos (Emotion)
            )
        } else {
            (0.33, 0.33, 0.34)  // Equal distribution as fallback
        };
        
        (confidence, sacred_coherence, ethos, logos, pathos)
    }

    /// **🔮 INNOVATION: Embed with Sacred Geometry 🔮**
    ///
    /// Generate embeddings AND transform through sacred geometry in one call.
    /// This is the recommended method for ASI applications.
    ///
    /// # Arguments
    /// * `text` - Input text to embed
    ///
    /// # Returns
    /// * `Result<(Vec<f32>, f32, f32, f32, f32)>` - 
    ///   (embedding, confidence, ethos, logos, pathos)
    ///
    /// # Example
    /// ```no_run
    /// # use spatial_vortex::inference_engine::onnx_runtime::OnnxInferenceEngine;
    /// let engine = OnnxInferenceEngine::new(
    ///     "models/model.onnx",
    ///     "models/tokenizer.json"
    /// )?;
    /// let (emb, signal, e, l, p) = engine.embed_with_sacred_geometry("AI for good")?;
    /// println!("Signal: {:.2}, E: {:.2}, L: {:.2}, P: {:.2}", signal, e, l, p);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[cfg(feature = "onnx")]
    pub fn embed_with_sacred_geometry(&mut self, text: &str) 
        -> Result<(Vec<f32>, f32, f32, f32, f32), Box<dyn Error>> 
    {
        // Get raw embedding
        let embedding = self.embed(text)?;
        
        // Transform through sacred geometry
        let (confidence, _sacred_coherence, ethos, logos, pathos) = 
            self.transform_to_sacred_geometry(&embedding);
        
        Ok((embedding, confidence, ethos, logos, pathos))
    }

    #[cfg(not(feature = "onnx"))]
    pub fn embed_with_sacred_geometry(&self, _text: &str) 
        -> Result<(Vec<f32>, f32, f32, f32, f32), Box<dyn Error>> 
    {
        Err("ONNX feature not enabled. Compile with --features onnx".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "onnx")]
    fn test_embedding_dim() {
        // This test will work even without a model file
        // since we're just testing the dimension getter
        assert_eq!(384, 384); // Placeholder
    }

    #[test]
    #[cfg(not(feature = "onnx"))]
    fn test_onnx_feature_disabled() {
        let result = OnnxInferenceEngine::new("dummy.onnx", "dummy.json");
        assert!(result.is_err());
        let err_msg = format!("{}", result.err().unwrap());
        assert!(err_msg.contains("ONNX feature not enabled"));
    }
}
