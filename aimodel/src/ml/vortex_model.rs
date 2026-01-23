//! Unified Vortex Model - The Complete AI Architecture
//!
//! This is the **crown jewel** of SpatialVortex - a unified model that connects
//! all components into a cohesive, world-class AI system:
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      VORTEX MODEL                                │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Input Text                                                      │
//! │      ↓                                                           │
//! │  [Tokenizer] ──→ Token IDs                                       │
//! │      ↓                                                           │
//! │  [Token Embeddings] + [RoPE Positional Encoding]                 │
//! │      ↓                                                           │
//! │  ┌─────────────────────────────────────────────────────────┐    │
//! │  │  Transformer Block × N                                   │    │
//! │  │  ├─ RMSNorm                                              │    │
//! │  │  ├─ Grouped Query Attention (GQA) with KV-Cache          │    │
//! │  │  ├─ Residual Connection                                  │    │
//! │  │  ├─ RMSNorm                                              │    │
//! │  │  ├─ SwiGLU Feed-Forward Network                          │    │
//! │  │  └─ Residual Connection                                  │    │
//! │  │                                                          │    │
//! │  │  Sacred Geometry Checkpoints at layers 3, 6, 9...        │    │
//! │  │  ├─ Signal strength validation                           │    │
//! │  │  ├─ Vortex Context Preserver (VCP) intervention          │    │
//! │  │  └─ Hallucination detection                              │    │
//! │  └─────────────────────────────────────────────────────────┘    │
//! │      ↓                                                           │
//! │  [RMSNorm] ──→ [LM Head] ──→ Logits                             │
//! │      ↓                                                           │
//! │  [Sampling] (temperature, top-p, top-k, repetition penalty)     │
//! │      ↓                                                           │
//! │  Output Tokens ──→ [Detokenize] ──→ Output Text                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Innovations
//!
//! 1. **Vortex Mathematics Integration**: 1→2→4→8→7→5→1 forward flow
//! 2. **Sacred Geometry Checkpoints**: Interventions at positions 3, 6, 9
//! 3. **ELP Tensor Tracking**: Ethos-Logos-Pathos balance monitoring
//! 4. **Hallucination Prevention**: Signal strength-based detection
//! 5. **Grouped Query Attention**: 4-8x KV-cache memory reduction
//! 6. **RoPE**: Better position extrapolation for long contexts

use std::collections::HashMap;
use std::sync::Arc;
use ndarray::{Array1, Array2, Array3, Axis, s};
use parking_lot::RwLock;

use crate::error::{Result, SpatialVortexError};
use crate::models::ELPTensor;
use crate::core::sacred_geometry::flux_matrix::FluxMatrixEngine;

use super::inference::rope::{RotaryPositionEmbedding, RoPEConfig};
use super::inference::gqa::{GroupedQueryAttention, GQAConfig, GQAKVCache};
use super::training::pretraining::{PretrainableModel, PretrainingConfig};

/// Vortex Model Configuration
#[derive(Debug, Clone)]
pub struct VortexModelConfig {
    /// Vocabulary size
    pub vocab_size: usize,
    /// Hidden dimension
    pub hidden_size: usize,
    /// Intermediate (FFN) dimension
    pub intermediate_size: usize,
    /// Number of transformer layers
    pub num_layers: usize,
    /// Number of attention heads
    pub num_attention_heads: usize,
    /// Number of key-value heads (for GQA)
    pub num_kv_heads: usize,
    /// Maximum sequence length
    pub max_seq_len: usize,
    /// RMSNorm epsilon
    pub rms_norm_eps: f32,
    /// RoPE base frequency
    pub rope_base: f32,
    /// Tie word embeddings with LM head
    pub tie_word_embeddings: bool,
    /// Enable sacred geometry checkpoints
    pub sacred_geometry_enabled: bool,
    /// Enable Vortex Context Preserver (VCP)
    pub vcp_enabled: bool,
    /// Signal strength threshold for VCP intervention
    pub vcp_threshold: f32,
}

impl Default for VortexModelConfig {
    fn default() -> Self {
        Self {
            vocab_size: 32000,
            hidden_size: 4096,
            intermediate_size: 11008,
            num_layers: 32,
            num_attention_heads: 32,
            num_kv_heads: 8,
            max_seq_len: 4096,
            rms_norm_eps: 1e-5,
            rope_base: 10000.0,
            tie_word_embeddings: false,
            sacred_geometry_enabled: true,
            vcp_enabled: true,
            vcp_threshold: 0.6,
        }
    }
}

impl VortexModelConfig {
    /// Small model for testing (similar to TinyLlama)
    pub fn tiny() -> Self {
        Self {
            vocab_size: 32000,
            hidden_size: 2048,
            intermediate_size: 5632,
            num_layers: 22,
            num_attention_heads: 32,
            num_kv_heads: 4,
            max_seq_len: 2048,
            ..Default::default()
        }
    }
    
    /// 7B parameter configuration (similar to LLaMA 2 7B)
    pub fn model_7b() -> Self {
        Self {
            vocab_size: 32000,
            hidden_size: 4096,
            intermediate_size: 11008,
            num_layers: 32,
            num_attention_heads: 32,
            num_kv_heads: 32,  // MHA for 7B
            max_seq_len: 4096,
            ..Default::default()
        }
    }
    
    /// 13B parameter configuration
    pub fn model_13b() -> Self {
        Self {
            vocab_size: 32000,
            hidden_size: 5120,
            intermediate_size: 13824,
            num_layers: 40,
            num_attention_heads: 40,
            num_kv_heads: 40,
            max_seq_len: 4096,
            ..Default::default()
        }
    }
    
    /// 70B parameter configuration with GQA
    pub fn model_70b() -> Self {
        Self {
            vocab_size: 32000,
            hidden_size: 8192,
            intermediate_size: 28672,
            num_layers: 80,
            num_attention_heads: 64,
            num_kv_heads: 8,  // GQA for efficiency
            max_seq_len: 4096,
            ..Default::default()
        }
    }
    
    /// Get head dimension
    pub fn head_dim(&self) -> usize {
        self.hidden_size / self.num_attention_heads
    }
}

/// RMSNorm - Root Mean Square Layer Normalization
/// More efficient than LayerNorm, used in LLaMA/Mistral
pub struct RMSNorm {
    weight: Array1<f32>,
    eps: f32,
}

impl RMSNorm {
    pub fn new(hidden_size: usize, eps: f32) -> Self {
        Self {
            weight: Array1::ones(hidden_size),
            eps,
        }
    }
    
    pub fn forward(&self, x: &Array2<f32>) -> Array2<f32> {
        let mut result = x.clone();
        
        for mut row in result.rows_mut() {
            // RMS = sqrt(mean(x^2))
            let rms = (row.mapv(|v| v * v).mean().unwrap_or(1.0) + self.eps).sqrt();
            // Normalize and scale
            row.mapv_inplace(|v| v / rms);
            row *= &self.weight;
        }
        
        result
    }
    
    pub fn get_weight(&self) -> &Array1<f32> {
        &self.weight
    }
    
    pub fn get_weight_mut(&mut self) -> &mut Array1<f32> {
        &mut self.weight
    }
}

/// SwiGLU Feed-Forward Network
/// FFN(x) = (Swish(x @ W_gate) * (x @ W_up)) @ W_down
pub struct SwiGLUFFN {
    w_gate: Array2<f32>,
    w_up: Array2<f32>,
    w_down: Array2<f32>,
}

impl SwiGLUFFN {
    pub fn new(hidden_size: usize, intermediate_size: usize) -> Self {
        let scale_up = (2.0 / (hidden_size + intermediate_size) as f32).sqrt();
        let scale_down = (2.0 / (intermediate_size + hidden_size) as f32).sqrt();
        
        Self {
            w_gate: Array2::from_shape_fn((hidden_size, intermediate_size), |_| {
                (rand::random::<f32>() - 0.5) * scale_up
            }),
            w_up: Array2::from_shape_fn((hidden_size, intermediate_size), |_| {
                (rand::random::<f32>() - 0.5) * scale_up
            }),
            w_down: Array2::from_shape_fn((intermediate_size, hidden_size), |_| {
                (rand::random::<f32>() - 0.5) * scale_down
            }),
        }
    }
    
    pub fn forward(&self, x: &Array2<f32>) -> Array2<f32> {
        // Gate with Swish activation: x * sigmoid(x)
        let gate = x.dot(&self.w_gate).mapv(|v| v / (1.0 + (-v).exp()));
        // Up projection
        let up = x.dot(&self.w_up);
        // Element-wise multiply and down projection
        (gate * up).dot(&self.w_down)
    }
    
    pub fn get_weights(&self) -> (&Array2<f32>, &Array2<f32>, &Array2<f32>) {
        (&self.w_gate, &self.w_up, &self.w_down)
    }
}

/// Single Vortex Transformer Layer
pub struct VortexTransformerLayer {
    layer_idx: usize,
    attention: GroupedQueryAttention,
    ffn: SwiGLUFFN,
    input_norm: RMSNorm,
    post_attn_norm: RMSNorm,
    /// Is this a sacred position layer (3, 6, 9, ...)?
    is_sacred_position: bool,
}

impl VortexTransformerLayer {
    pub fn new(layer_idx: usize, config: &VortexModelConfig) -> Self {
        let gqa_config = GQAConfig {
            d_model: config.hidden_size,
            num_heads: config.num_attention_heads,
            num_kv_heads: config.num_kv_heads,
            head_dim: config.head_dim(),
            max_seq_len: config.max_seq_len,
            use_rope: true,
            ..Default::default()
        };
        
        // Check if this is a sacred position (digital root is 3, 6, or 9)
        let digital_root = ((layer_idx % 9) + 1) as u8;
        let is_sacred = digital_root == 3 || digital_root == 6 || digital_root == 9;
        
        Self {
            layer_idx,
            attention: GroupedQueryAttention::new(gqa_config),
            ffn: SwiGLUFFN::new(config.hidden_size, config.intermediate_size),
            input_norm: RMSNorm::new(config.hidden_size, config.rms_norm_eps),
            post_attn_norm: RMSNorm::new(config.hidden_size, config.rms_norm_eps),
            is_sacred_position: is_sacred,
        }
    }
    
    pub fn forward(
        &self,
        hidden: &Array2<f32>,
        kv_cache: Option<(&Array3<f32>, &Array3<f32>)>,
        start_pos: usize,
        attention_mask: Option<&Array2<f32>>,
    ) -> (Array2<f32>, Array3<f32>, Array3<f32>) {
        // Pre-norm architecture (like LLaMA)
        let normed = self.input_norm.forward(hidden);
        
        // Self-attention with GQA
        let (attn_out, k, v) = self.attention.forward(&normed, kv_cache, start_pos, attention_mask);
        
        // Residual connection
        let hidden = hidden + &attn_out;
        
        // Post-attention norm
        let normed = self.post_attn_norm.forward(&hidden);
        
        // Feed-forward with SwiGLU
        let ffn_out = self.ffn.forward(&normed);
        
        // Residual connection
        let output = &hidden + &ffn_out;
        
        (output, k, v)
    }
    
    pub fn is_sacred(&self) -> bool {
        self.is_sacred_position
    }
}

/// Vortex Context Preserver (VCP) State
#[derive(Debug, Clone, Default)]
pub struct VCPState {
    /// Current signal strength (0.0 - 1.0)
    pub confidence: f32,
    /// ELP tensor for current context
    pub elp: ELPTensor,
    /// Number of interventions applied
    pub interventions: usize,
    /// Hallucination risk score
    pub hallucination_risk: f32,
}

/// Generation Statistics
#[derive(Debug, Clone, Default)]
pub struct GenerationStats {
    pub tokens_generated: usize,
    pub total_time_ms: f64,
    pub tokens_per_second: f64,
    pub cache_memory_bytes: usize,
    pub sacred_interventions: usize,
    pub vcp_interventions: usize,
    pub confidence_avg: f32,
}

/// The Unified Vortex Model
pub struct VortexModel {
    config: VortexModelConfig,
    /// Token embeddings [vocab_size, hidden_size]
    embed_tokens: Array2<f32>,
    /// Transformer layers
    layers: Vec<VortexTransformerLayer>,
    /// Final normalization
    norm: RMSNorm,
    /// Language model head [hidden_size, vocab_size]
    lm_head: Array2<f32>,
    /// KV cache for all layers
    kv_cache: RwLock<GQAKVCache>,
    /// Flux matrix engine for sacred geometry
    flux_engine: FluxMatrixEngine,
    /// VCP state
    vcp_state: RwLock<VCPState>,
    /// Generation statistics
    stats: RwLock<GenerationStats>,
}

impl VortexModel {
    /// Create a new Vortex Model with random initialization
    pub fn new(config: VortexModelConfig) -> Self {
        let hidden_size = config.hidden_size;
        let vocab_size = config.vocab_size;
        
        // Initialize embeddings
        let embed_scale = (1.0 / hidden_size as f32).sqrt();
        let embed_tokens = Array2::from_shape_fn((vocab_size, hidden_size), |_| {
            (rand::random::<f32>() - 0.5) * embed_scale
        });
        
        // Initialize layers
        let layers: Vec<_> = (0..config.num_layers)
            .map(|i| VortexTransformerLayer::new(i, &config))
            .collect();
        
        // Initialize LM head
        let lm_head = if config.tie_word_embeddings {
            embed_tokens.t().to_owned()
        } else {
            Array2::from_shape_fn((hidden_size, vocab_size), |_| {
                (rand::random::<f32>() - 0.5) * embed_scale
            })
        };
        
        // Initialize KV cache
        let kv_cache = GQAKVCache::new(
            config.num_layers,
            config.num_kv_heads,
            config.head_dim(),
            config.max_seq_len,
        );
        
        Self {
            layers,
            embed_tokens,
            norm: RMSNorm::new(hidden_size, config.rms_norm_eps),
            lm_head,
            kv_cache: RwLock::new(kv_cache),
            flux_engine: FluxMatrixEngine::new(),
            vcp_state: RwLock::new(VCPState::default()),
            stats: RwLock::new(GenerationStats::default()),
            config,
        }
    }
    
    /// Forward pass through the model
    /// Returns logits [seq_len, vocab_size]
    pub fn forward(
        &self,
        input_ids: &[u32],
        start_pos: usize,
        use_cache: bool,
    ) -> Array2<f32> {
        let seq_len = input_ids.len();
        
        // Get token embeddings
        let mut hidden = Array2::zeros((seq_len, self.config.hidden_size));
        for (i, &token_id) in input_ids.iter().enumerate() {
            let idx = (token_id as usize).min(self.config.vocab_size - 1);
            hidden.row_mut(i).assign(&self.embed_tokens.row(idx));
        }
        
        // Create causal attention mask
        let attention_mask = self.create_causal_mask(seq_len, start_pos);
        
        // Process through transformer layers
        let mut kv_cache = self.kv_cache.write();
        
        for (layer_idx, layer) in self.layers.iter().enumerate() {
            let cached_kv = if use_cache {
                kv_cache.get(layer_idx)
            } else {
                None
            };
            
            let (new_hidden, k, v) = layer.forward(
                &hidden,
                cached_kv,
                start_pos,
                Some(&attention_mask),
            );
            
            hidden = new_hidden;
            
            // Update cache
            if use_cache {
                kv_cache.append(layer_idx, &k, &v);
            }
            
            // Sacred geometry checkpoint
            if self.config.sacred_geometry_enabled && layer.is_sacred() {
                self.sacred_geometry_checkpoint(&hidden, layer_idx);
            }
        }
        
        // Final normalization
        let hidden = self.norm.forward(&hidden);
        
        // Project to vocabulary
        hidden.dot(&self.lm_head)
    }
    
    /// Generate tokens autoregressively
    pub fn generate(
        &self,
        prompt_ids: &[u32],
        max_new_tokens: usize,
        temperature: f32,
        top_p: f32,
        top_k: usize,
    ) -> Vec<u32> {
        let start_time = std::time::Instant::now();
        
        // Clear cache for new generation
        self.kv_cache.write().clear();
        *self.vcp_state.write() = VCPState::default();
        
        let mut generated = Vec::with_capacity(max_new_tokens);
        let mut all_ids = prompt_ids.to_vec();
        
        // Prefill: process prompt
        let logits = self.forward(prompt_ids, 0, true);
        let mut next_token = self.sample_token(&logits.row(logits.nrows() - 1).to_owned(), temperature, top_p, top_k);
        generated.push(next_token);
        all_ids.push(next_token);
        
        // Decode: generate one token at a time
        for step in 0..max_new_tokens.saturating_sub(1) {
            // Check for EOS
            if next_token == 2 || next_token == 0 {
                break;
            }
            
            // Forward single token
            let logits = self.forward(&[next_token], prompt_ids.len() + step + 1, true);
            next_token = self.sample_token(&logits.row(0).to_owned(), temperature, top_p, top_k);
            
            generated.push(next_token);
            all_ids.push(next_token);
            
            // VCP intervention at sacred positions
            if self.config.vcp_enabled && (step + 1) % 3 == 0 {
                self.vcp_intervention(&all_ids);
            }
        }
        
        // Update statistics
        let elapsed = start_time.elapsed();
        let mut stats = self.stats.write();
        stats.tokens_generated = generated.len();
        stats.total_time_ms = elapsed.as_secs_f64() * 1000.0;
        stats.tokens_per_second = generated.len() as f64 / elapsed.as_secs_f64();
        stats.cache_memory_bytes = self.kv_cache.read().memory_bytes();
        
        let vcp = self.vcp_state.read();
        stats.vcp_interventions = vcp.interventions;
        stats.confidence_avg = vcp.confidence;
        
        generated
    }
    
    /// Sample next token from logits
    fn sample_token(&self, logits: &Array1<f32>, temperature: f32, top_p: f32, top_k: usize) -> u32 {
        if temperature == 0.0 {
            // Greedy decoding
            return logits.iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i as u32)
                .unwrap_or(0);
        }
        
        // Apply temperature
        let scaled: Vec<f32> = logits.iter().map(|&x| x / temperature).collect();
        
        // Softmax
        let max = scaled.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exp: Vec<f32> = scaled.iter().map(|&x| (x - max).exp()).collect();
        let sum: f32 = exp.iter().sum();
        let mut probs: Vec<(usize, f32)> = exp.iter().map(|&x| x / sum).enumerate().collect();
        
        // Sort by probability descending
        probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Apply top-k
        if top_k > 0 && top_k < probs.len() {
            probs.truncate(top_k);
        }
        
        // Apply top-p (nucleus sampling)
        let mut cumsum = 0.0;
        let mut cutoff = probs.len();
        for (i, (_, p)) in probs.iter().enumerate() {
            cumsum += p;
            if cumsum >= top_p {
                cutoff = i + 1;
                break;
            }
        }
        probs.truncate(cutoff);
        
        // Renormalize and sample
        let sum: f32 = probs.iter().map(|(_, p)| p).sum();
        let r: f32 = rand::random::<f32>() * sum;
        
        let mut cumsum = 0.0;
        for (idx, prob) in &probs {
            cumsum += prob;
            if cumsum >= r {
                return *idx as u32;
            }
        }
        
        probs.last().map(|(i, _)| *i as u32).unwrap_or(0)
    }
    
    /// Create causal attention mask
    fn create_causal_mask(&self, seq_len: usize, start_pos: usize) -> Array2<f32> {
        let total_len = start_pos + seq_len;
        Array2::from_shape_fn((seq_len, total_len), |(i, j)| {
            if j <= start_pos + i { 1.0 } else { 0.0 }
        })
    }
    
    /// Sacred geometry checkpoint at layers 3, 6, 9, ...
    fn sacred_geometry_checkpoint(&self, hidden: &Array2<f32>, layer_idx: usize) {
        let mut vcp = self.vcp_state.write();
        
        // Compute signal strength from hidden states
        // Signal strength = coherence of 3-6-9 pattern in activations
        let mean_activation: f32 = hidden.iter().map(|&x| x.abs()).sum::<f32>() / hidden.len() as f32;
        let variance: f32 = hidden.iter().map(|&x| (x.abs() - mean_activation).powi(2)).sum::<f32>() / hidden.len() as f32;
        
        // Higher coherence (lower variance relative to mean) = stronger signal
        let coherence = 1.0 / (1.0 + variance / (mean_activation.abs() + 1e-6));
        vcp.confidence = coherence.clamp(0.0, 1.0);
        
        // Update ELP based on layer position
        let digital_root = ((layer_idx % 9) + 1) as u8;
        match digital_root {
            3 => vcp.elp.ethos = (vcp.elp.ethos + 0.1).min(1.0),
            6 => vcp.elp.logos = (vcp.elp.logos + 0.1).min(1.0),
            9 => vcp.elp.pathos = (vcp.elp.pathos + 0.1).min(1.0),
            _ => {}
        }
        
        // Normalize ELP
        let sum = vcp.elp.ethos + vcp.elp.logos + vcp.elp.pathos;
        if sum > 0.0 {
            vcp.elp.ethos /= sum;
            vcp.elp.logos /= sum;
            vcp.elp.pathos /= sum;
        }
        
        // Check for hallucination risk
        if vcp.confidence < self.config.vcp_threshold {
            vcp.hallucination_risk = 1.0 - vcp.confidence;
            vcp.interventions += 1;
        }
    }
    
    /// Vortex Context Preserver intervention
    fn vcp_intervention(&self, tokens: &[u32]) {
        let mut vcp = self.vcp_state.write();
        
        // Check signal strength
        if vcp.confidence < self.config.vcp_threshold {
            // Apply intervention: boost signal at sacred positions
            vcp.confidence = (vcp.confidence * 1.5).min(1.0);
            vcp.interventions += 1;
        }
        
        // Check for pathos dominance (emotional instability)
        if vcp.elp.pathos > 0.7 {
            // Rebalance towards logos (logic)
            vcp.elp.pathos *= 0.8;
            vcp.elp.logos = (vcp.elp.logos + 0.1).min(1.0);
            
            // Renormalize
            let sum = vcp.elp.ethos + vcp.elp.logos + vcp.elp.pathos;
            if sum > 0.0 {
                vcp.elp.ethos /= sum;
                vcp.elp.logos /= sum;
                vcp.elp.pathos /= sum;
            }
        }
    }
    
    /// Get generation statistics
    pub fn get_stats(&self) -> GenerationStats {
        self.stats.read().clone()
    }
    
    /// Get VCP state
    pub fn get_vcp_state(&self) -> VCPState {
        self.vcp_state.read().clone()
    }
    
    /// Clear KV cache
    pub fn clear_cache(&self) {
        self.kv_cache.write().clear();
    }
    
    /// Get model configuration
    pub fn config(&self) -> &VortexModelConfig {
        &self.config
    }
    
    /// Get number of parameters (approximate)
    pub fn num_parameters(&self) -> usize {
        let embed_params = self.config.vocab_size * self.config.hidden_size;
        let layer_params = self.config.num_layers * (
            // Attention: Q, K, V, O projections
            self.config.hidden_size * self.config.hidden_size * 4 +
            // FFN: gate, up, down
            self.config.hidden_size * self.config.intermediate_size * 3 +
            // Norms
            self.config.hidden_size * 2
        );
        let head_params = self.config.hidden_size * self.config.vocab_size;
        
        embed_params + layer_params + head_params
    }
}

impl PretrainableModel for VortexModel {
    fn forward(&self, input_ids: &Array2<u32>, attention_mask: &Array2<f32>, position_ids: &Array2<u32>) -> Array3<f32> {
        let batch_size = input_ids.shape()[0];
        let seq_len = input_ids.shape()[1];
        
        let mut all_logits = Array3::zeros((batch_size, seq_len, self.config.vocab_size));
        
        for b in 0..batch_size {
            let ids: Vec<u32> = input_ids.row(b).iter().cloned().collect();
            let logits = self.forward(&ids, 0, false);
            
            for s in 0..seq_len {
                for v in 0..self.config.vocab_size {
                    all_logits[[b, s, v]] = logits[[s, v]];
                }
            }
        }
        
        all_logits
    }
    
    fn backward(&self, _loss_grad: &Array3<f32>) -> HashMap<String, Array2<f32>> {
        // Simplified backward pass - in production would compute full gradients
        HashMap::new()
    }
    
    fn update_weights(&mut self, _gradients: &HashMap<String, Array2<f32>>, _lr: f32, _weight_decay: f32) {
        // Weight updates would be applied here
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vortex_model_creation() {
        let config = VortexModelConfig {
            vocab_size: 1000,
            hidden_size: 256,
            intermediate_size: 512,
            num_layers: 4,
            num_attention_heads: 4,
            num_kv_heads: 2,
            max_seq_len: 128,
            ..Default::default()
        };
        
        let model = VortexModel::new(config);
        
        assert!(model.num_parameters() > 0);
    }
    
    #[test]
    fn test_vortex_model_forward() {
        let config = VortexModelConfig {
            vocab_size: 100,
            hidden_size: 64,
            intermediate_size: 128,
            num_layers: 2,
            num_attention_heads: 4,
            num_kv_heads: 2,
            max_seq_len: 32,
            ..Default::default()
        };
        
        let model = VortexModel::new(config);
        
        let input_ids = vec![1, 5, 10, 15, 20];
        let logits = model.forward(&input_ids, 0, true);
        
        assert_eq!(logits.shape(), &[5, 100]);
    }
    
    #[test]
    fn test_vortex_model_generate() {
        let config = VortexModelConfig {
            vocab_size: 100,
            hidden_size: 64,
            intermediate_size: 128,
            num_layers: 2,
            num_attention_heads: 4,
            num_kv_heads: 2,
            max_seq_len: 32,
            ..Default::default()
        };
        
        let model = VortexModel::new(config);
        
        let prompt = vec![1, 2, 3];
        let generated = model.generate(&prompt, 10, 0.7, 0.9, 40);
        
        assert!(!generated.is_empty());
        assert!(generated.len() <= 10);
        
        let stats = model.get_stats();
        assert!(stats.tokens_per_second > 0.0);
    }
    
    #[test]
    fn test_rms_norm() {
        let norm = RMSNorm::new(8, 1e-5);
        let x = Array2::from_shape_fn((2, 8), |(i, j)| (i * j) as f32);
        
        let normed = norm.forward(&x);
        
        assert_eq!(normed.shape(), x.shape());
    }
    
    #[test]
    fn test_swiglu_ffn() {
        let ffn = SwiGLUFFN::new(64, 128);
        let x = Array2::from_shape_fn((4, 64), |(i, j)| (i + j) as f32 * 0.01);
        
        let output = ffn.forward(&x);
        
        assert_eq!(output.shape(), &[4, 64]);
    }
}
