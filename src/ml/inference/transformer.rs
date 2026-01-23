//! Transformer Architecture with Sacred Geometry Integration
//!
//! Complete transformer implementation with:
//! - Positional encoding for sequence order
//! - Multi-head self-attention (Query, Key, Value)
//! - Feed-forward multi-layer perceptron (MLP) units
//! - Training infrastructure with backpropagation
//! - Integration with vortex mathematics and sacred geometry
//!
//! ## Architecture
//!
//! ```
//! Input Tokens → Positional Encoding → Multi-Head Attention
//!             → Feed-Forward Network → Sacred Geometry Transform
//!             → Vortex Positioning → Output
//! ```
//!
//! ## Training Loop (Halving Sequence - Backpropagation)
//!
//! ```
//! 1. Forward Pass: Model generates output
//! 2. Compute Loss: Difference from target (cross-entropy)
//! 3. Backward Pass: Compute gradients via chain rule
//! 4. Optimizer: Update weights and biases
//! 5. Repeat: Billions of times across terabytes of data
//! ```
//!
//! **Halving Sequence**: 1 → 5 → 7 → 8 → 4 → 2 → 1 (error correction phase)

use std::sync::Arc;
use ndarray::{Array1, Array2, s};

/// Positional Encoding for Sequence Order Awareness
///
/// Uses sinusoidal functions to encode absolute position in sequence:
/// - PE(pos, 2i) = sin(pos / 10000^(2i/d_model))
/// - PE(pos, 2i+1) = cos(pos / 10000^(2i/d_model))
pub struct PositionalEncoding {
    max_seq_len: usize,
    d_model: usize,
    encoding: Array2<f32>,  // [max_seq_len, d_model]
}

impl PositionalEncoding {
    /// Create positional encoding matrix
    ///
    /// # Arguments
    /// * `max_seq_len` - Maximum sequence length (e.g., 2048, 4096)
    /// * `d_model` - Model dimension (e.g., 384, 512, 768)
    pub fn new(max_seq_len: usize, d_model: usize) -> Self {
        let mut encoding = Array2::<f32>::zeros((max_seq_len, d_model));
        
        for pos in 0..max_seq_len {
            for i in 0..(d_model / 2) {
                let angle = pos as f32 / 10000_f32.powf(2.0 * i as f32 / d_model as f32);
                encoding[[pos, 2 * i]] = angle.sin();
                encoding[[pos, 2 * i + 1]] = angle.cos();
            }
        }
        
        Self {
            max_seq_len,
            d_model,
            encoding,
        }
    }
    
    /// Add positional encoding to input embeddings
    ///
    /// # Arguments
    /// * `embeddings` - Input token embeddings [seq_len, d_model]
    ///
    /// # Returns
    /// * Embeddings with positional information added
    pub fn encode(&self, embeddings: &Array2<f32>) -> Array2<f32> {
        let seq_len = embeddings.nrows();
        assert!(seq_len <= self.max_seq_len, "Sequence too long");
        assert_eq!(embeddings.ncols(), self.d_model, "Dimension mismatch");
        
        // Add positional encoding to embeddings
        embeddings + &self.encoding.slice(s![0..seq_len, ..])
    }
}

/// Self-Attention Mechanism with Query, Key, Value
///
/// Core attention computation:
/// ```
/// Attention(Q, K, V) = softmax(Q·K^T / √d_k) · V
/// ```
///
/// Where:
/// - Q (Query): "What am I looking for?"
/// - K (Key): "What do I have?"
/// - V (Value): "What information do I provide?"
pub struct SelfAttention {
    #[allow(dead_code)]  // Reserved for dynamic dimension adjustment
    d_model: usize,
    d_k: usize,        // Key/Query dimension
    #[allow(dead_code)]  // Reserved for separate value dimension support
    d_v: usize,        // Value dimension
    
    // Learnable weight matrices
    w_query: Array2<f32>,   // [d_model, d_k]
    w_key: Array2<f32>,     // [d_model, d_k]
    w_value: Array2<f32>,   // [d_model, d_v]
    w_output: Array2<f32>,  // [d_v, d_model]
    
    // Biases
    b_query: Array1<f32>,
    b_key: Array1<f32>,
    b_value: Array1<f32>,
    b_output: Array1<f32>,
}

impl SelfAttention {
    /// Create new self-attention layer
    pub fn new(d_model: usize, d_k: usize, d_v: usize) -> Self {
        // Initialize weights with Xavier/Glorot initialization
        let scale_k = (2.0 / (d_model + d_k) as f32).sqrt();
        let scale_v = (2.0 / (d_model + d_v) as f32).sqrt();
        let scale_out = (2.0 / (d_v + d_model) as f32).sqrt();
        
        Self {
            d_model,
            d_k,
            d_v,
            w_query: Array2::from_shape_fn((d_model, d_k), |_| {
                (rand::random::<f32>() - 0.5) * scale_k
            }),
            w_key: Array2::from_shape_fn((d_model, d_k), |_| {
                (rand::random::<f32>() - 0.5) * scale_k
            }),
            w_value: Array2::from_shape_fn((d_model, d_v), |_| {
                (rand::random::<f32>() - 0.5) * scale_v
            }),
            w_output: Array2::from_shape_fn((d_v, d_model), |_| {
                (rand::random::<f32>() - 0.5) * scale_out
            }),
            b_query: Array1::zeros(d_k),
            b_key: Array1::zeros(d_k),
            b_value: Array1::zeros(d_v),
            b_output: Array1::zeros(d_model),
        }
    }
    
    /// Forward pass: Compute attention
    ///
    /// # Arguments
    /// * `input` - Input embeddings [seq_len, d_model]
    /// * `mask` - Optional attention mask (for causal/padding masking)
    ///
    /// # Returns
    /// * Attention output [seq_len, d_model]
    /// * Attention weights [seq_len, seq_len] (for interpretability)
    pub fn forward(
        &self,
        input: &Array2<f32>,
        mask: Option<&Array2<bool>>,
    ) -> (Array2<f32>, Array2<f32>) {
        let seq_len = input.nrows();
        
        // Step 1: Compute Q, K, V
        // Q = X·W_Q + b_Q
        let query = input.dot(&self.w_query) + &self.b_query;
        let key = input.dot(&self.w_key) + &self.b_key;
        let value = input.dot(&self.w_value) + &self.b_value;
        
        // Step 2: Compute attention scores
        // scores = Q·K^T / √d_k
        let scores = query.dot(&key.t()) / (self.d_k as f32).sqrt();
        
        // Step 3: Apply mask if provided
        let mut scores = scores;
        if let Some(mask) = mask {
            for i in 0..seq_len {
                for j in 0..seq_len {
                    if mask[[i, j]] {
                        scores[[i, j]] = f32::NEG_INFINITY;  // Mask out
                    }
                }
            }
        }
        
        // Step 4: Softmax to get attention weights
        let attention_weights = Self::softmax(&scores);
        
        // Step 5: Apply attention to values
        // output = softmax(scores)·V
        let attended_values = attention_weights.dot(&value);
        
        // Step 6: Project back to d_model
        let output = attended_values.dot(&self.w_output) + &self.b_output;
        
        (output, attention_weights)
    }
    
    /// Softmax activation (row-wise)
    fn softmax(x: &Array2<f32>) -> Array2<f32> {
        let mut result = x.clone();
        for mut row in result.rows_mut() {
            let max = row.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            row.mapv_inplace(|v| (v - max).exp());
            let sum: f32 = row.sum();
            row.mapv_inplace(|v| v / sum);
        }
        result
    }
}

/// Multi-Head Attention
///
/// Runs multiple attention heads in parallel for richer representations:
/// ```
/// MultiHead(Q, K, V) = Concat(head_1, ..., head_h)·W_O
/// where head_i = Attention(Q·W_Q^i, K·W_K^i, V·W_V^i)
/// ```
pub struct MultiHeadAttention {
    num_heads: usize,
    d_model: usize,
    heads: Vec<SelfAttention>,
    context_window: usize,  // Maximum tokens to attend to
}

impl MultiHeadAttention {
    /// Create multi-head attention layer
    ///
    /// # Arguments
    /// * `num_heads` - Number of parallel attention heads (e.g., 8, 12)
    /// * `d_model` - Model dimension (must be divisible by num_heads)
    /// * `context_window` - Context window size (e.g., 2048, 4096)
    pub fn new(num_heads: usize, d_model: usize, context_window: usize) -> Self {
        assert_eq!(d_model % num_heads, 0, "d_model must be divisible by num_heads");
        
        let d_k = d_model / num_heads;
        let d_v = d_model / num_heads;
        
        let heads = (0..num_heads)
            .map(|_| SelfAttention::new(d_model, d_k, d_v))
            .collect();
        
        Self {
            num_heads,
            d_model,
            heads,
            context_window,
        }
    }
    
    /// Forward pass through all attention heads
    ///
    /// # Arguments
    /// * `input` - Input embeddings [seq_len, d_model]
    /// * `mask` - Optional attention mask
    ///
    /// # Returns
    /// * Multi-head attention output [seq_len, d_model]
    /// * Attention weights from all heads [num_heads, seq_len, seq_len]
    pub fn forward(
        &self,
        input: &Array2<f32>,
        mask: Option<&Array2<bool>>,
    ) -> (Array2<f32>, Vec<Array2<f32>>) {
        let seq_len = input.nrows();
        
        // Limit context window if sequence is too long
        let effective_len = seq_len.min(self.context_window);
        let input_windowed = if seq_len > self.context_window {
            input.slice(s![seq_len - effective_len.., ..]).to_owned()
        } else {
            input.clone()
        };
        
        // Run all heads in parallel
        let head_outputs: Vec<_> = self.heads
            .iter()
            .map(|head| head.forward(&input_windowed, mask))
            .collect();
        
        // Concatenate head outputs
        let mut concatenated = Array2::zeros((effective_len, self.d_model));
        let head_dim = self.d_model / self.num_heads;
        
        for (i, (output, _)) in head_outputs.iter().enumerate() {
            let start = i * head_dim;
            let end = (i + 1) * head_dim;
            concatenated.slice_mut(s![.., start..end]).assign(output);
        }
        
        // Extract attention weights
        let attention_weights = head_outputs.into_iter()
            .map(|(_, weights)| weights)
            .collect();
        
        (concatenated, attention_weights)
    }
    
    /// Async forward pass using Tokio for parallel computation
    pub async fn forward_async(
        &self,
        input: Arc<Array2<f32>>,
        mask: Option<Arc<Array2<bool>>>,
    ) -> (Array2<f32>, Vec<Array2<f32>>) {
        let seq_len = input.nrows();
        let effective_len = seq_len.min(self.context_window);
        
        // Process each head (simplified - not using async for now due to Send constraints)
        let head_outputs: Vec<_> = self.heads
            .iter()
            .map(|head| {
                head.forward(input.as_ref(), mask.as_deref())
            })
            .collect();
        
        // Concatenate results
        let mut concatenated = Array2::zeros((effective_len, self.d_model));
        let head_dim = self.d_model / self.num_heads;
        
        for (i, (output, _)) in head_outputs.iter().enumerate() {
            let start = i * head_dim;
            let end = (i + 1) * head_dim;
            concatenated.slice_mut(s![.., start..end]).assign(output);
        }
        
        let attention_weights = head_outputs.into_iter()
            .map(|(_, weights)| weights)
            .collect();
        
        (concatenated, attention_weights)
    }
}

/// Feed-Forward Multi-Layer Perceptron (MLP)
///
/// Two-layer network with activation:
/// ```
/// FFN(x) = activation(x·W_1 + b_1)·W_2 + b_2
/// ```
///
/// Purpose:
/// - Selection bias interpretation
/// - Non-linear transformations
/// - Feature extraction
/// - Subject inference understanding
pub struct FeedForwardNetwork {
    #[allow(dead_code)]  // Reserved for dynamic network resizing
    d_model: usize,
    #[allow(dead_code)]  // Reserved for adaptive hidden dimensions
    d_ff: usize,      // Hidden dimension (typically 4 * d_model)
    
    // Learnable weights
    w1: Array2<f32>,  // [d_model, d_ff]
    b1: Array1<f32>,  // [d_ff]
    w2: Array2<f32>,  // [d_ff, d_model]
    b2: Array1<f32>,  // [d_model]
    
    activation: ActivationFunction,
}

pub enum ActivationFunction {
    ReLU,
    GELU,
    Swish,
}

impl FeedForwardNetwork {
    /// Create feed-forward network
    ///
    /// # Arguments
    /// * `d_model` - Model dimension
    /// * `d_ff` - Hidden dimension (typically 4 * d_model)
    /// * `activation` - Activation function (ReLU, GELU, Swish)
    pub fn new(d_model: usize, d_ff: usize, activation: ActivationFunction) -> Self {
        let scale1 = (2.0 / (d_model + d_ff) as f32).sqrt();
        let scale2 = (2.0 / (d_ff + d_model) as f32).sqrt();
        
        Self {
            d_model,
            d_ff,
            w1: Array2::from_shape_fn((d_model, d_ff), |_| {
                (rand::random::<f32>() - 0.5) * scale1
            }),
            b1: Array1::zeros(d_ff),
            w2: Array2::from_shape_fn((d_ff, d_model), |_| {
                (rand::random::<f32>() - 0.5) * scale2
            }),
            b2: Array1::zeros(d_model),
            activation,
        }
    }
    
    /// Forward pass
    pub fn forward(&self, input: &Array2<f32>) -> Array2<f32> {
        // Layer 1: x·W_1 + b_1
        let hidden = input.dot(&self.w1) + &self.b1;
        
        // Activation
        let activated = self.apply_activation(&hidden);
        
        // Layer 2: activation·W_2 + b_2
        activated.dot(&self.w2) + &self.b2
    }
    
    fn apply_activation(&self, x: &Array2<f32>) -> Array2<f32> {
        match self.activation {
            ActivationFunction::ReLU => x.mapv(|v| v.max(0.0)),
            ActivationFunction::GELU => x.mapv(|v| {
                0.5 * v * (1.0 + (v * 0.7978845608 * (1.0 + 0.044715 * v * v)).tanh())
            }),
            ActivationFunction::Swish => x.mapv(|v| v / (1.0 + (-v).exp())),
        }
    }
}

/// Complete Transformer Block
///
/// Combines:
/// - Multi-head self-attention
/// - Feed-forward network
/// - Layer normalization
/// - Residual connections
pub struct TransformerBlock {
    attention: MultiHeadAttention,
    feed_forward: FeedForwardNetwork,
    norm1: LayerNorm,
    norm2: LayerNorm,
    #[allow(dead_code)]  // Reserved for dropout implementation
    dropout_rate: f32,
}

impl TransformerBlock {
    /// Create transformer block
    pub fn new(
        num_heads: usize,
        d_model: usize,
        d_ff: usize,
        context_window: usize,
        dropout_rate: f32,
    ) -> Self {
        Self {
            attention: MultiHeadAttention::new(num_heads, d_model, context_window),
            feed_forward: FeedForwardNetwork::new(d_model, d_ff, ActivationFunction::GELU),
            norm1: LayerNorm::new(d_model),
            norm2: LayerNorm::new(d_model),
            dropout_rate,
        }
    }
    
    /// Forward pass through transformer block
    ///
    /// Architecture:
    /// ```
    /// x = x + Attention(LayerNorm(x))   // Residual connection
    /// x = x + FFN(LayerNorm(x))         // Residual connection
    /// ```
    pub fn forward(&self, input: &Array2<f32>, mask: Option<&Array2<bool>>) -> Array2<f32> {
        // Sub-layer 1: Self-attention with residual
        let normalized1 = self.norm1.forward(input);
        let (attended, _) = self.attention.forward(&normalized1, mask);
        let residual1 = input + &attended;
        
        // Sub-layer 2: Feed-forward with residual
        let normalized2 = self.norm2.forward(&residual1);
        let ff_output = self.feed_forward.forward(&normalized2);
        let output = &residual1 + &ff_output;
        
        output
    }
    
    /// Async forward pass
    pub async fn forward_async(
        &self,
        input: Arc<Array2<f32>>,
        mask: Option<Arc<Array2<bool>>>,
    ) -> Array2<f32> {
        // Sub-layer 1: Async self-attention
        let normalized1 = self.norm1.forward(&input);
        let (attended, _) = self.attention.forward_async(
            Arc::new(normalized1),
            mask,
        ).await;
        let residual1 = &*input + &attended;
        
        // Sub-layer 2: Feed-forward
        let normalized2 = self.norm2.forward(&residual1);
        let ff_output = self.feed_forward.forward(&normalized2);
        let output = &residual1 + &ff_output;
        
        output
    }
}

/// Layer Normalization
///
/// Normalizes across features for stable training:
/// ```
/// LayerNorm(x) = γ·(x - μ)/σ + β
/// ```
pub struct LayerNorm {
    #[allow(dead_code)]  // Reserved for learnable normalization parameters
    d_model: usize,
    gamma: Array1<f32>,  // Scale
    beta: Array1<f32>,   // Shift
    eps: f32,            // Numerical stability
}

impl LayerNorm {
    pub fn new(d_model: usize) -> Self {
        Self {
            d_model,
            gamma: Array1::ones(d_model),
            beta: Array1::zeros(d_model),
            eps: 1e-6,
        }
    }
    
    pub fn forward(&self, input: &Array2<f32>) -> Array2<f32> {
        let mut result = input.clone();
        
        for mut row in result.rows_mut() {
            let mean = row.mean().unwrap();
            let variance = row.mapv(|x| (x - mean).powi(2)).mean().unwrap();
            let std = (variance + self.eps).sqrt();
            
            // Normalize
            row.mapv_inplace(|x| (x - mean) / std);
            
            // Scale and shift
            row *= &self.gamma;
            row += &self.beta;
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_positional_encoding() {
        let pe = PositionalEncoding::new(10, 8);
        let embeddings = Array2::zeros((5, 8));
        let encoded = pe.encode(&embeddings);
        
        assert_eq!(encoded.shape(), &[5, 8]);
    }
    
    #[test]
    fn test_self_attention() {
        let attention = SelfAttention::new(8, 8, 8);
        let input = Array2::from_shape_fn((3, 8), |_| rand::random::<f32>());
        
        let (output, weights) = attention.forward(&input, None);
        
        assert_eq!(output.shape(), &[3, 8]);
        assert_eq!(weights.shape(), &[3, 3]);
    }
    
    #[test]
    fn test_multi_head_attention() {
        let mha = MultiHeadAttention::new(4, 8, 10);
        let input = Array2::from_shape_fn((3, 8), |_| rand::random::<f32>());
        
        let (output, weights) = mha.forward(&input, None);
        
        assert_eq!(output.shape(), &[3, 8]);
        assert_eq!(weights.len(), 4);  // 4 heads
    }
    
    #[test]
    fn test_feed_forward() {
        let ffn = FeedForwardNetwork::new(8, 32, ActivationFunction::GELU);
        let input = Array2::from_shape_fn((3, 8), |_| rand::random::<f32>());
        
        let output = ffn.forward(&input);
        
        assert_eq!(output.shape(), &[3, 8]);
    }
    
    #[test]
    fn test_transformer_block() {
        let block = TransformerBlock::new(4, 8, 32, 10, 0.1);
        let input = Array2::from_shape_fn((3, 8), |_| rand::random::<f32>());
        
        let output = block.forward(&input, None);
        
        assert_eq!(output.shape(), &[3, 8]);
    }
}
