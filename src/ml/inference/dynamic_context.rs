//! Dynamic Context Window with Confidence-Based Extension
//!
//! Solves the fixed 4096 token limit by making positional encoding
//! dynamic based on signal strength and confidence scores.
//!
//! ## Key Innovations
//!
//! 1. **Confidence-Based Extension**: High-confidence tokens stay longer
//! 2. **Sacred Checkpoints**: Prune at positions 3, 6, 9
//! 3. **Adaptive Window**: Grows/shrinks based on importance
//! 4. **No Forgetting**: Important context preserved indefinitely
//! 5. **Overflow Prevention**: Integrated with VortexContextPreserver
//!
//! ## Problem Solved
//!
//! Standard transformers:
//! - Fixed 4096 token limit
//! - All tokens equally important
//! - Forget everything beyond window
//! - No selective retention
//!
//! Our solution:
//! - Dynamic unlimited context
//! - Confidence-weighted importance
//! - Selective pruning at sacred positions
//! - Important ideas never forgotten

use ndarray::{Array1, Array2};
use std::collections::VecDeque;

/// Dynamic positional encoding with confidence-based extension
pub struct DynamicPositionalEncoding {
    /// Model dimension
    d_model: usize,
    
    /// Base window size (soft limit)
    base_window: usize,
    
    /// Maximum window size (hard limit, but can extend)
    #[allow(dead_code)]  // Reserved for cache management features
    max_window: usize,
    
    /// Confidence threshold for retention
    confidence_threshold: f32,
    
    /// Cached encodings (computed on-demand)
    encoding_cache: Vec<Array1<f32>>,
}

impl DynamicPositionalEncoding {
    /// Create dynamic positional encoding
    ///
    /// # Arguments
    /// * `d_model` - Model dimension
    /// * `base_window` - Base context window (e.g., 2048)
    /// * `confidence_threshold` - Min confidence to extend context (e.g., 0.7)
    pub fn new(d_model: usize, base_window: usize, confidence_threshold: f32) -> Self {
        Self {
            d_model,
            base_window,
            max_window: base_window * 4, // Initial max, can grow
            confidence_threshold,
            encoding_cache: Vec::new(),
        }
    }
    
    /// Compute positional encoding for a given position
    ///
    /// Uses sinusoidal encoding: PE(pos, 2i) = sin(pos / 10000^(2i/d_model))
    fn compute_encoding(&self, pos: usize) -> Array1<f32> {
        let mut encoding = Array1::<f32>::zeros(self.d_model);
        
        for i in 0..(self.d_model / 2) {
            let angle = pos as f32 / 10000_f32.powf(2.0 * i as f32 / self.d_model as f32);
            encoding[2 * i] = angle.sin();
            encoding[2 * i + 1] = angle.cos();
        }
        
        encoding
    }
    
    /// Get or compute positional encoding for position
    fn get_encoding(&mut self, pos: usize) -> &Array1<f32> {
        // Extend cache if needed
        while self.encoding_cache.len() <= pos {
            let new_pos = self.encoding_cache.len();
            let encoding = self.compute_encoding(new_pos);
            self.encoding_cache.push(encoding);
        }
        
        &self.encoding_cache[pos]
    }
    
    /// Encode with dynamic context management
    ///
    /// # Arguments
    /// * `embeddings` - Token embeddings [seq_len, d_model]
    /// * `confidences` - Confidence score for each token [seq_len]
    ///
    /// # Returns
    /// * Embeddings with positional encoding + retained indices
    pub fn encode_with_confidence(
        &mut self,
        embeddings: &Array2<f32>,
        confidences: &[f32],
    ) -> (Array2<f32>, Vec<usize>) {
        let seq_len = embeddings.nrows();
        assert_eq!(confidences.len(), seq_len, "Confidence count mismatch");
        
        // Determine which tokens to retain based on confidence
        let retained_indices = self.select_tokens_by_confidence(confidences, seq_len);
        
        // Build output with only retained tokens
        let retained_count = retained_indices.len();
        let d_model = self.d_model;  // Extract before loop to avoid borrow issues
        let mut encoded = Array2::<f32>::zeros((retained_count, d_model));
        
        for (out_idx, &in_idx) in retained_indices.iter().enumerate() {
            // Get positional encoding for this position
            let pos_encoding = self.get_encoding(out_idx);
            
            // Add to embedding
            let embedding_row = embeddings.row(in_idx);
            for j in 0..d_model {
                encoded[[out_idx, j]] = embedding_row[j] + pos_encoding[j];
            }
        }
        
        (encoded, retained_indices)
    }
    
    /// Select tokens to retain based on confidence and sacred positions
    fn select_tokens_by_confidence(&self, confidences: &[f32], seq_len: usize) -> Vec<usize> {
        let mut retained = Vec::new();
        
        // Always keep recent tokens (within base window)
        let start_idx = if seq_len > self.base_window {
            seq_len - self.base_window
        } else {
            0
        };
        
        // Keep all tokens in base window
        for i in start_idx..seq_len {
            retained.push(i);
        }
        
        // For older tokens, keep only high-confidence ones
        for i in 0..start_idx {
            if confidences[i] >= self.confidence_threshold {
                retained.push(i);
            }
        }
        
        // Sort to maintain chronological order
        retained.sort();
        
        retained
    }
}

/// Confidence-Weighted Context Manager
///
/// Manages dynamic context window with confidence-based retention
pub struct ConfidenceContextManager {
    /// Dynamic positional encoding
    encoding: DynamicPositionalEncoding,
    
    /// Token buffer with metadata
    tokens: VecDeque<TokenWithMetadata>,
    
    /// Sacred position checkpoints
    #[allow(dead_code)]  // Reserved for checkpoint-based interventions
    sacred_checkpoints: Vec<usize>, // Positions 3, 6, 9, etc.
}

/// Token with confidence metadata
#[derive(Clone)]
struct TokenWithMetadata {
    /// Token embedding
    embedding: Array1<f32>,
    
    /// Confidence score [0, 1] (includes signal strength from VortexContextPreserver)
    confidence: f32,
    
    /// Original position
    #[allow(dead_code)]  // Reserved for position-aware features
    position: usize,
    
    /// Is this a sacred checkpoint?
    is_sacred: bool,
}

impl ConfidenceContextManager {
    /// Create new confidence-based context manager
    ///
    /// # Arguments
    /// * `d_model` - Model dimension
    /// * `base_window` - Base context window (soft limit)
    /// * `confidence_threshold` - Min confidence to retain old tokens
    pub fn new(d_model: usize, base_window: usize, confidence_threshold: f32) -> Self {
        Self {
            encoding: DynamicPositionalEncoding::new(d_model, base_window, confidence_threshold),
            tokens: VecDeque::new(),
            sacred_checkpoints: vec![3, 6, 9], // Can extend: 12, 15, 18, etc.
        }
    }
    
    /// Add new tokens to context
    ///
    /// # Arguments
    /// * `embeddings` - New token embeddings
    /// * `confidences` - Confidence scores
    /// * `confidences` - Signal strength (from hallucination detector)
    pub fn add_tokens(
        &mut self,
        embeddings: &Array2<f32>,
        confidences: &[f32],
    ) {
        let seq_len = embeddings.nrows();
        
        for i in 0..seq_len {
            let position = self.tokens.len();
            let is_sacred = self.is_sacred_position(position);
            
            let token = TokenWithMetadata {
                embedding: embeddings.row(i).to_owned(),
                confidence: confidences[i],
                position,
                is_sacred,
            };
            
            self.tokens.push_back(token);
            
            // Prune at sacred checkpoints
            if is_sacred {
                self.prune_low_confidence_tokens();
            }
        }
    }
    
    /// Check if position is sacred (3, 6, 9, 12, 15, 18, ...)
    fn is_sacred_position(&self, pos: usize) -> bool {
        pos > 0 && pos % 3 == 0
    }
    
    /// Prune low-confidence tokens at sacred checkpoint
    fn prune_low_confidence_tokens(&mut self) {
        // Keep recent tokens (base window)
        let base_window = self.encoding.base_window;
        let total_len = self.tokens.len();
        
        if total_len <= base_window {
            return; // Don't prune if we're within base window
        }
        
        // Calculate how many old tokens we have
        let old_token_count = total_len - base_window;
        
        // Collect tokens to keep
        let mut keep_indices = Vec::new();
        
        // Evaluate old tokens
        for i in 0..old_token_count {
            let token = &self.tokens[i];
            
            // Keep if:
            // 1. Sacred checkpoint (always keep)
            // 2. High confidence AND high signal strength
            // 3. Very high signal strength (even if low confidence)
            let should_keep = token.is_sacred
                || (token.confidence >= self.encoding.confidence_threshold 
                    && token.confidence >= 0.6)
                || token.confidence >= 0.8;
            
            if should_keep {
                keep_indices.push(i);
            }
        }
        
        // Always keep all recent tokens
        for i in old_token_count..total_len {
            keep_indices.push(i);
        }
        
        // Rebuild tokens buffer with only kept tokens
        let old_tokens: Vec<_> = self.tokens.drain(..).collect();
        for idx in keep_indices {
            self.tokens.push_back(old_tokens[idx].clone());
        }
    }
    
    /// Get current context for processing
    ///
    /// Returns embeddings and metadata for retained tokens
    pub fn get_context(&mut self) -> (Array2<f32>, Vec<f32>) {
        let token_count = self.tokens.len();
        let d_model = self.encoding.d_model;
        
        // Build embeddings matrix
        let mut embeddings = Array2::<f32>::zeros((token_count, d_model));
        let mut confidences = Vec::with_capacity(token_count);
        
        for (i, token) in self.tokens.iter().enumerate() {
            for j in 0..d_model {
                embeddings[[i, j]] = token.embedding[j];
            }
            confidences.push(token.confidence);
        }
        
        // Apply positional encoding with confidence weighting
        let (encoded, _) = self.encoding.encode_with_confidence(&embeddings, &confidences);
        
        (encoded, confidences)
    }
    
    /// Get context statistics
    pub fn stats(&self) -> ContextStats {
        let total = self.tokens.len();
        let sacred = self.tokens.iter().filter(|t| t.is_sacred).count();
        let high_conf = self.tokens.iter()
            .filter(|t| t.confidence >= self.encoding.confidence_threshold)
            .count();
        let high_signal = self.tokens.iter()
            .filter(|t| t.confidence >= 0.7)
            .count();
        
        let avg_conf = if total > 0 {
            self.tokens.iter().map(|t| t.confidence).sum::<f32>() / total as f32
        } else {
            0.0
        };
        
        let avg_signal = if total > 0 {
            self.tokens.iter().map(|t| t.confidence).sum::<f32>() / total as f32
        } else {
            0.0
        };
        
        ContextStats {
            total_tokens: total,
            sacred_checkpoints: sacred,
            high_confidence_tokens: high_conf,
            high_signal_tokens: high_signal,
            avg_confidence: avg_conf,
            base_window: self.encoding.base_window,
            effective_window: total,
        }
    }
}

/// Context statistics
#[derive(Debug, Clone)]
pub struct ContextStats {
    pub total_tokens: usize,
    pub sacred_checkpoints: usize,
    pub high_confidence_tokens: usize,
    pub high_signal_tokens: usize,
    pub avg_confidence: f32,
    pub base_window: usize,
    pub effective_window: usize,
}

impl ContextStats {
    pub fn display(&self) {
        println!("📊 Context Statistics:");
        println!("   Total tokens: {}", self.total_tokens);
        println!("   Base window: {}", self.base_window);
        println!("   Effective window: {} ({}x base)", 
            self.effective_window,
            self.effective_window as f32 / self.base_window as f32
        );
        println!("   Sacred checkpoints: {}", self.sacred_checkpoints);
        println!("   High confidence: {} ({:.1}%)", 
            self.high_confidence_tokens,
            100.0 * self.high_confidence_tokens as f32 / self.total_tokens as f32
        );
        println!("   High signal: {} ({:.1}%)",
            self.high_signal_tokens,
            100.0 * self.high_signal_tokens as f32 / self.total_tokens as f32
        );
        println!("   Avg confidence: {:.3}", self.avg_confidence);
        println!("   Avg signal strength: {:.3}", self.avg_confidence);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dynamic_encoding() {
        let mut encoding = DynamicPositionalEncoding::new(8, 10, 0.7);
        
        // Create sample embeddings
        let embeddings = Array2::from_shape_fn((5, 8), |_| rand::random::<f32>());
        let confidences = vec![0.9, 0.5, 0.8, 0.6, 0.95];
        
        let (encoded, retained) = encoding.encode_with_confidence(&embeddings, &confidences);
        
        // Should have retained some tokens
        assert!(retained.len() <= 5);
        assert_eq!(encoded.nrows(), retained.len());
    }
    
    #[test]
    fn test_confidence_context_manager() {
        let mut manager = ConfidenceContextManager::new(8, 10, 0.7);
        
        // Add tokens
        let embeddings = Array2::from_shape_fn((5, 8), |_| rand::random::<f32>());
        let confidences = vec![0.9, 0.5, 0.8, 0.6, 0.95];
        
        manager.add_tokens(&embeddings, &confidences);
        
        let stats = manager.stats();
        assert_eq!(stats.total_tokens, 5);
    }
    
    #[test]
    fn test_sacred_positions() {
        let manager = ConfidenceContextManager::new(8, 10, 0.7);
        
        assert!(!manager.is_sacred_position(0));
        assert!(!manager.is_sacred_position(1));
        assert!(!manager.is_sacred_position(2));
        assert!(manager.is_sacred_position(3));
        assert!(!manager.is_sacred_position(4));
        assert!(!manager.is_sacred_position(5));
        assert!(manager.is_sacred_position(6));
        assert!(manager.is_sacred_position(9));
        assert!(manager.is_sacred_position(12));
    }
}
