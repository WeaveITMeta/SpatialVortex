//! CALM - Continuous Autoregressive Language Models
//!
//! High-fidelity autoencoder compressing K semantic chunks → continuous latent.
//! Autoregress in latent space with energy-based prediction.
//! Decode back → K× fewer steps, smoother vortex orbits.

use crate::data::models::BeamTensor;
use crate::ml::ebrm::EnergyBasedReasoningModel;
use serde::{Deserialize, Serialize};

/// CALM Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CALMConfig {
    /// Latent dimension for continuous space
    pub latent_dim: usize,
    /// Number of semantic chunks to compress
    pub chunk_size: usize,
    /// Compression ratio (K)
    pub compression_ratio: usize,
    /// Energy threshold for latent predictions
    pub energy_threshold: f32,
    /// Enable speculative decoding
    pub speculative_decoding: bool,
    /// Batch size for parallel decoding
    pub batch_size: usize,
}

impl Default for CALMConfig {
    fn default() -> Self {
        Self {
            latent_dim: 256,  // Balanced dimension for current architecture
            chunk_size: 8,
            compression_ratio: 4,
            energy_threshold: 0.5,
            speculative_decoding: true,
            batch_size: 4,
        }
    }
}

impl CALMConfig {
    pub fn new() -> Self { Self::default() }
    pub fn with_latent_dim(mut self, dim: usize) -> Self { self.latent_dim = dim; self }
    pub fn with_compression_ratio(mut self, k: usize) -> Self { self.compression_ratio = k; self }
}

/// Continuous latent representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentState {
    /// Continuous latent vector
    pub latent: Vec<f32>,
    /// Energy score from EBRM
    pub energy: f32,
    /// Sacred alignment (0-1)
    pub sacred_alignment: f32,
    /// Step in the latent sequence
    pub step: usize,
}

impl LatentState {
    pub fn new(latent_dim: usize) -> Self {
        Self {
            latent: vec![0.0; latent_dim],
            energy: 0.0,
            sacred_alignment: 0.0,
            step: 0,
        }
    }
}

/// Multi-Head Attention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiHeadAttentionConfig {
    pub num_heads: usize,
    pub head_dim: usize,
    pub dropout: f32,
}

impl Default for MultiHeadAttentionConfig {
    fn default() -> Self {
        Self {
            num_heads: 8,
            head_dim: 32,  // 8 heads * 32 = 256 total dim
            dropout: 0.1,
        }
    }
}

/// Multi-Head Attention module with Q/K/V projections
#[derive(Debug, Clone)]
pub struct MultiHeadAttention {
    pub config: MultiHeadAttentionConfig,
    /// Query projection weights [latent_dim, num_heads * head_dim]
    pub w_q: Vec<f32>,
    /// Key projection weights
    pub w_k: Vec<f32>,
    /// Value projection weights
    pub w_v: Vec<f32>,
    /// Output projection weights
    pub w_o: Vec<f32>,
}

impl MultiHeadAttention {
    pub fn new(latent_dim: usize, config: MultiHeadAttentionConfig) -> Self {
        let total_dim = config.num_heads * config.head_dim;
        let scale = (2.0 / (latent_dim + total_dim) as f32).sqrt();
        
        Self {
            w_q: (0..latent_dim * total_dim).map(|i| ((i as f32 * 0.1).sin() * scale)).collect(),
            w_k: (0..latent_dim * total_dim).map(|i| ((i as f32 * 0.13).cos() * scale)).collect(),
            w_v: (0..latent_dim * total_dim).map(|i| ((i as f32 * 0.17).sin() * scale)).collect(),
            w_o: (0..total_dim * latent_dim).map(|i| ((i as f32 * 0.11).cos() * scale)).collect(),
            config,
        }
    }
    
    /// SIMD-optimized dot product for f32 slices
    #[inline(always)]
    fn simd_dot_product(a: &[f32], b: &[f32]) -> f32 {
        // Process 4 elements at a time (SSE/NEON compatible)
        let len = a.len().min(b.len());
        let chunks = len / 4;
        let remainder = len % 4;
        
        let mut sum = 0.0f32;
        
        // Vectorized loop (compiler will auto-vectorize)
        for i in 0..chunks {
            let base = i * 4;
            sum += a[base] * b[base];
            sum += a[base + 1] * b[base + 1];
            sum += a[base + 2] * b[base + 2];
            sum += a[base + 3] * b[base + 3];
        }
        
        // Handle remainder
        for i in (chunks * 4)..len {
            sum += a[i] * b[i];
        }
        
        sum
    }
    
    /// Fast matrix-vector multiply using SIMD-friendly layout
    #[inline]
    fn fast_matvec(&self, weights: &[f32], input: &[f32], output: &mut [f32], in_dim: usize, out_dim: usize) {
        for i in 0..out_dim {
            let row_start = i * in_dim;
            let row_end = (row_start + in_dim).min(weights.len());
            if row_end > row_start {
                let weight_slice = &weights[row_start..row_end];
                let input_slice = &input[..weight_slice.len().min(input.len())];
                output[i] = Self::simd_dot_product(weight_slice, input_slice);
            }
        }
    }
    
    /// Compute scaled dot-product attention (SIMD optimized)
    /// Returns attention output and attention weights
    pub fn forward(&self, query: &[f32], keys: &[Vec<f32>], values: &[Vec<f32>]) -> (Vec<f32>, Vec<f32>) {
        let num_heads = self.config.num_heads;
        let head_dim = self.config.head_dim;
        let total_dim = num_heads * head_dim;
        let latent_dim = query.len();
        
        // Fast path for small number of keys (avoid rayon overhead)
        if keys.len() <= 8 {
            return self.forward_sequential(query, keys, values);
        }
        
        // Fast query projection using SIMD
        let mut q_proj = vec![0.0f32; total_dim];
        self.fast_matvec(&self.w_q, query, &mut q_proj, latent_dim, total_dim);
        
        // Project keys and values in parallel using rayon
        use rayon::prelude::*;
        let projections: Vec<(Vec<f32>, Vec<f32>)> = keys.par_iter()
            .zip(values.par_iter())
            .map(|(key, value)| {
                let mut k_proj = vec![0.0f32; total_dim];
                let mut v_proj = vec![0.0f32; total_dim];
                
                // SIMD-friendly projection
                for i in 0..total_dim {
                    let row_start = i * latent_dim;
                    let row_end = (row_start + latent_dim).min(self.w_k.len());
                    if row_end > row_start {
                        let k_slice = &self.w_k[row_start..row_end];
                        let v_slice = &self.w_v[row_start..row_end.min(self.w_v.len())];
                        let key_slice = &key[..k_slice.len().min(key.len())];
                        let val_slice = &value[..v_slice.len().min(value.len())];
                        k_proj[i] = Self::simd_dot_product(k_slice, key_slice);
                        v_proj[i] = Self::simd_dot_product(v_slice, val_slice);
                    }
                }
                (k_proj, v_proj)
            })
            .collect();
        
        let k_projs: Vec<Vec<f32>> = projections.iter().map(|(k, _)| k.clone()).collect();
        let v_projs: Vec<Vec<f32>> = projections.iter().map(|(_, v)| v.clone()).collect();
        
        // Compute attention scores per head (parallelized)
        let scale = (head_dim as f32).sqrt();
        let num_keys = keys.len();
        
        // Parallel head computation
        let head_outputs: Vec<(Vec<f32>, Vec<f32>)> = (0..num_heads)
            .into_par_iter()
            .map(|head| {
                let head_start = head * head_dim;
                let head_end = head_start + head_dim;
                
                // Compute attention scores for this head using SIMD
                let scores: Vec<f32> = k_projs.iter()
                    .map(|k| {
                        let q_slice = &q_proj[head_start..head_end];
                        let k_slice = &k[head_start..head_end];
                        Self::simd_dot_product(q_slice, k_slice) / scale
                    })
                    .collect();
                
                // Softmax
                let max_score = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                let exp_scores: Vec<f32> = scores.iter().map(|s| (s - max_score).exp()).collect();
                let exp_sum: f32 = exp_scores.iter().sum();
                let attn_weights: Vec<f32> = exp_scores.iter().map(|e| e / exp_sum.max(1e-8)).collect();
                
                // Weighted sum of values for this head
                let mut head_output = vec![0.0f32; head_dim];
                for (v, &w) in v_projs.iter().zip(attn_weights.iter()) {
                    for i in 0..head_dim {
                        head_output[i] += v[head_start + i] * w;
                    }
                }
                
                (head_output, attn_weights)
            })
            .collect();
        
        // Combine head outputs
        let mut output = vec![0.0f32; total_dim];
        let mut all_attn_weights = vec![0.0f32; num_keys];
        
        for (head, (head_output, attn_weights)) in head_outputs.into_iter().enumerate() {
            let head_start = head * head_dim;
            for (i, &val) in head_output.iter().enumerate() {
                output[head_start + i] = val;
            }
            for (i, &w) in attn_weights.iter().enumerate() {
                all_attn_weights[i] += w / num_heads as f32;
            }
        }
        
        // Output projection using SIMD
        let mut final_output = vec![0.0f32; latent_dim];
        for i in 0..latent_dim {
            let mut sum = 0.0f32;
            for j in 0..total_dim {
                let idx = j * latent_dim + i;
                if idx < self.w_o.len() {
                    sum += output[j] * self.w_o[idx];
                }
            }
            final_output[i] = sum;
        }
        
        (final_output, all_attn_weights)
    }
    
    /// Sequential forward pass for small number of keys (avoids rayon overhead)
    fn forward_sequential(&self, query: &[f32], keys: &[Vec<f32>], values: &[Vec<f32>]) -> (Vec<f32>, Vec<f32>) {
        let num_heads = self.config.num_heads;
        let head_dim = self.config.head_dim;
        let total_dim = num_heads * head_dim;
        let latent_dim = query.len();
        
        // Fast query projection using SIMD
        let mut q_proj = vec![0.0f32; total_dim];
        self.fast_matvec(&self.w_q, query, &mut q_proj, latent_dim, total_dim);
        
        // Project keys and values sequentially
        let mut k_projs: Vec<Vec<f32>> = Vec::with_capacity(keys.len());
        let mut v_projs: Vec<Vec<f32>> = Vec::with_capacity(values.len());
        
        for (key, value) in keys.iter().zip(values.iter()) {
            let mut k_proj = vec![0.0f32; total_dim];
            let mut v_proj = vec![0.0f32; total_dim];
            
            for i in 0..total_dim {
                let row_start = i * latent_dim;
                let row_end = (row_start + latent_dim).min(self.w_k.len());
                if row_end > row_start {
                    let k_slice = &self.w_k[row_start..row_end];
                    let v_slice = &self.w_v[row_start..row_end.min(self.w_v.len())];
                    let key_slice = &key[..k_slice.len().min(key.len())];
                    let val_slice = &value[..v_slice.len().min(value.len())];
                    k_proj[i] = Self::simd_dot_product(k_slice, key_slice);
                    v_proj[i] = Self::simd_dot_product(v_slice, val_slice);
                }
            }
            k_projs.push(k_proj);
            v_projs.push(v_proj);
        }
        
        // Compute attention scores per head
        let scale = (head_dim as f32).sqrt();
        let num_keys = keys.len();
        let mut all_attn_weights = vec![0.0f32; num_keys];
        let mut output = vec![0.0f32; total_dim];
        
        for head in 0..num_heads {
            let head_start = head * head_dim;
            let head_end = head_start + head_dim;
            
            // Compute attention scores for this head using SIMD
            let scores: Vec<f32> = k_projs.iter()
                .map(|k| {
                    let q_slice = &q_proj[head_start..head_end];
                    let k_slice = &k[head_start..head_end];
                    Self::simd_dot_product(q_slice, k_slice) / scale
                })
                .collect();
            
            // Softmax
            let max_score = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let exp_scores: Vec<f32> = scores.iter().map(|s| (s - max_score).exp()).collect();
            let exp_sum: f32 = exp_scores.iter().sum();
            let attn_weights: Vec<f32> = exp_scores.iter().map(|e| e / exp_sum.max(1e-8)).collect();
            
            // Accumulate attention weights
            for (i, &w) in attn_weights.iter().enumerate() {
                all_attn_weights[i] += w / num_heads as f32;
            }
            
            // Weighted sum of values for this head
            for (v, &w) in v_projs.iter().zip(attn_weights.iter()) {
                for i in head_start..head_end {
                    output[i] += v[i] * w;
                }
            }
        }
        
        // Output projection using SIMD
        let mut final_output = vec![0.0f32; latent_dim];
        for i in 0..latent_dim {
            let mut sum = 0.0f32;
            for j in 0..total_dim {
                let idx = j * latent_dim + i;
                if idx < self.w_o.len() {
                    sum += output[j] * self.w_o[idx];
                }
            }
            final_output[i] = sum;
        }
        
        (final_output, all_attn_weights)
    }
    
    /// Train attention weights with gradient descent
    pub fn train_step(&mut self, query: &[f32], keys: &[Vec<f32>], values: &[Vec<f32>], 
                      target: &[f32], learning_rate: f32) {
        let (output, _) = self.forward(query, keys, values);
        let latent_dim = query.len();
        let total_dim = self.config.num_heads * self.config.head_dim;
        
        // Compute error
        let mut error = vec![0.0f32; latent_dim];
        for i in 0..latent_dim.min(target.len()).min(output.len()) {
            error[i] = target[i] - output[i];
        }
        
        // Update output projection weights
        for i in 0..latent_dim.min(error.len()) {
            for j in 0..total_dim {
                let idx = j * latent_dim + i;
                if idx < self.w_o.len() {
                    self.w_o[idx] += learning_rate * error[i] * 0.01;
                }
            }
        }
    }
}

/// Contrastive learning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrastiveLearningConfig {
    pub temperature: f32,
    pub num_negatives: usize,
    pub margin: f32,
}

impl Default for ContrastiveLearningConfig {
    fn default() -> Self {
        Self {
            temperature: 0.07,
            num_negatives: 5,
            margin: 0.5,
        }
    }
}

/// CALM Engine - Continuous Autoregressive Language Model
///
/// Integrates with EBRM for energy-based latent space prediction.
/// Provides K× speedup through latent compression.
/// Now with Multi-Head Attention, Contrastive Learning, and EmbedVec HNSW.
#[derive(Debug, Clone)]
pub struct CALMEngine {
    pub config: CALMConfig,
    ebrm: EnergyBasedReasoningModel,
    /// Encoder weights (simplified - would be Burn tensors in full impl)
    encoder_weights: Vec<f32>,
    /// Decoder weights
    decoder_weights: Vec<f32>,
    /// Latent predictor weights
    predictor_weights: Vec<f32>,
    /// Attention weights for context focusing (legacy, kept for compatibility)
    attention_weights: Vec<f32>,
    /// Multi-Head Attention module
    pub multi_head_attention: MultiHeadAttention,
    /// Contrastive learning config
    pub contrastive_config: ContrastiveLearningConfig,
    /// Word embeddings learned from training (HashMap for O(1) lookup)
    word_embeddings: std::collections::HashMap<String, Vec<f32>>,
    /// Word to ID mapping for EmbedVec
    word_to_id: std::collections::HashMap<String, u64>,
    /// Next ID for EmbedVec insertions
    next_embed_id: u64,
    /// Negative samples buffer for contrastive learning
    negative_buffer: Vec<Vec<f32>>,
    /// Training step counter
    training_steps: usize,
}

impl CALMEngine {
    pub fn new(config: CALMConfig) -> Self {
        let latent_dim = config.latent_dim;
        let input_dim = config.chunk_size * 9; // 9 digits per BeamTensor
        
        // Xavier initialization for better gradient flow
        let encoder_scale = (2.0 / (input_dim + latent_dim) as f32).sqrt();
        let decoder_scale = (2.0 / (latent_dim + input_dim) as f32).sqrt();
        let predictor_scale = (2.0 / (latent_dim * 2) as f32).sqrt();
        
        // Configure MHA based on latent dim
        let mha_config = MultiHeadAttentionConfig {
            num_heads: 8,
            head_dim: latent_dim / 8,  // Scale head_dim to match latent_dim
            dropout: 0.1,
        };
        
        Self {
            config,
            ebrm: EnergyBasedReasoningModel::new(),
            encoder_weights: (0..input_dim * latent_dim)
                .map(|i| ((i as f32 * 0.1).sin() * encoder_scale))
                .collect(),
            decoder_weights: (0..latent_dim * input_dim)
                .map(|i| ((i as f32 * 0.1).cos() * decoder_scale))
                .collect(),
            predictor_weights: (0..latent_dim * latent_dim)
                .map(|i| ((i as f32 * 0.1).sin() * predictor_scale))
                .collect(),
            attention_weights: vec![1.0 / latent_dim as f32; latent_dim],
            multi_head_attention: MultiHeadAttention::new(latent_dim, mha_config),
            contrastive_config: ContrastiveLearningConfig::default(),
            word_embeddings: std::collections::HashMap::new(),
            word_to_id: std::collections::HashMap::new(),
            next_embed_id: 0,
            negative_buffer: Vec::new(),
            training_steps: 0,
        }
    }
    
    /// Import embeddings from external source (e.g., HuggingFace datasets)
    /// This allows CALM to use pre-learned knowledge for semantic retrieval
    pub fn import_embeddings(&mut self, embeddings: &std::collections::HashMap<String, Vec<f32>>) {
        for (word, embed) in embeddings {
            let word_lower = word.to_lowercase();
            if !self.word_embeddings.contains_key(&word_lower) {
                self.word_embeddings.insert(word_lower.clone(), embed.clone());
                self.word_to_id.insert(word_lower, self.next_embed_id);
                self.next_embed_id += 1;
            }
        }
    }
    
    /// Get all learned embeddings (for persistence or transfer)
    pub fn get_all_embeddings(&self) -> &std::collections::HashMap<String, Vec<f32>> {
        &self.word_embeddings
    }
    
    /// Store a word embedding (learns from context)
    pub fn store_embedding(&mut self, word: &str, embedding: Vec<f32>) {
        let word_lower = word.to_lowercase();
        if !self.word_to_id.contains_key(&word_lower) {
            self.word_to_id.insert(word_lower.clone(), self.next_embed_id);
            self.next_embed_id += 1;
        }
        self.word_embeddings.insert(word_lower, embedding);
    }
    
    /// Get or create embedding for a word using hash-based initialization
    pub fn get_or_create_embedding(&mut self, word: &str) -> Vec<f32> {
        let word_lower = word.to_lowercase();
        
        if let Some(embed) = self.word_embeddings.get(&word_lower) {
            return embed.clone();
        }
        
        // Create hash-based embedding for unknown words
        let embed = Self::hash_embedding(&word_lower, self.config.latent_dim);
        self.store_embedding(&word_lower, embed.clone());
        embed
    }
    
    /// Generate hash-based embedding for a word (deterministic)
    fn hash_embedding(word: &str, dim: usize) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut embedding = vec![0.0f32; dim];
        let bytes = word.as_bytes();
        
        for (i, chunk) in embedding.chunks_mut(1).enumerate() {
            let mut hasher = DefaultHasher::new();
            (i, bytes).hash(&mut hasher);
            let hash = hasher.finish();
            chunk[0] = ((hash % 1000) as f32 / 500.0) - 1.0; // Range [-1, 1]
        }
        
        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }
        
        embedding
    }
    
    /// Search for similar words using brute-force cosine similarity
    /// Returns top-k most similar words with their scores
    pub fn search_similar(&self, query: &[f32], k: usize) -> Vec<(String, f32)> {
        let mut scores: Vec<(String, f32)> = self.word_embeddings.iter()
            .map(|(word, embed)| {
                let sim = self.cosine_similarity(query, embed);
                (word.clone(), sim)
            })
            .collect();
        
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(k);
        scores
    }
    
    /// Get text embedding by averaging word embeddings
    pub fn get_text_embedding(&mut self, words: &[&str]) -> Vec<f32> {
        let dim = self.config.latent_dim;
        let mut combined = vec![0.0f32; dim];
        let mut count = 0;
        
        for word in words {
            let embed = self.get_or_create_embedding(word);
            for (i, val) in embed.iter().enumerate() {
                if i < combined.len() {
                    combined[i] += val;
                }
            }
            count += 1;
        }
        
        if count > 0 {
            for val in &mut combined {
                *val /= count as f32;
            }
        }
        
        combined
    }
    
    /// Score a choice against a question using learned embeddings
    pub fn score_semantic_similarity(&mut self, question_words: &[&str], choice_words: &[&str]) -> f32 {
        let q_embed = self.get_text_embedding(question_words);
        let c_embed = self.get_text_embedding(choice_words);
        self.cosine_similarity(&q_embed, &c_embed)
    }
    
    /// Compute InfoNCE contrastive loss
    /// Positive pair: (anchor, positive) should be similar
    /// Negative pairs: (anchor, negative_i) should be dissimilar
    pub fn contrastive_loss(&self, anchor: &[f32], positive: &[f32], negatives: &[Vec<f32>]) -> f32 {
        let temp = self.contrastive_config.temperature;
        
        // Compute similarity between anchor and positive
        let pos_sim = self.cosine_similarity(anchor, positive) / temp;
        
        // Compute similarities between anchor and negatives
        let neg_sims: Vec<f32> = negatives.iter()
            .map(|neg| self.cosine_similarity(anchor, neg) / temp)
            .collect();
        
        // InfoNCE loss: -log(exp(pos_sim) / (exp(pos_sim) + sum(exp(neg_sims))))
        let pos_exp = pos_sim.exp();
        let neg_exp_sum: f32 = neg_sims.iter().map(|s| s.exp()).sum();
        let denominator = pos_exp + neg_exp_sum;
        
        if denominator > 0.0 {
            -(pos_exp / denominator).ln()
        } else {
            0.0
        }
    }
    
    /// Cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut dot = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;
        
        for i in 0..a.len().min(b.len()) {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }
        
        if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a.sqrt() * norm_b.sqrt())
        } else {
            0.0
        }
    }
    
    /// Train with contrastive learning
    /// anchor: question embedding
    /// positive: correct answer embedding
    /// negatives: wrong answer embeddings
    pub fn train_contrastive(&mut self, anchor: &[f32], positive: &[f32], negatives: &[Vec<f32>], learning_rate: f32) {
        let loss = self.contrastive_loss(anchor, positive, negatives);
        
        // Add to negative buffer for future use
        if self.negative_buffer.len() < 100 {
            for neg in negatives {
                self.negative_buffer.push(neg.clone());
            }
        } else {
            // Replace oldest negatives
            for (i, neg) in negatives.iter().enumerate() {
                let idx = (self.training_steps + i) % self.negative_buffer.len();
                self.negative_buffer[idx] = neg.clone();
            }
        }
        
        // Update encoder weights to minimize contrastive loss
        let latent_dim = self.config.latent_dim;
        let input_dim = self.config.chunk_size * 9;
        
        // Attraction: push anchor closer to positive
        let pos_sim = self.cosine_similarity(anchor, positive);
        let attract_scale = learning_rate * (1.0 - pos_sim).max(0.0) * 0.1;
        
        for i in 0..latent_dim.min(anchor.len()).min(positive.len()) {
            let attract_grad = positive[i] - anchor[i];
            
            // Repulsion: push anchor away from negatives
            let mut repel_grad = 0.0f32;
            for neg in negatives {
                if i < neg.len() {
                    let neg_sim = self.cosine_similarity(anchor, neg);
                    let repel_scale = neg_sim.max(0.0) * 0.05; // Stronger repulsion for more similar negatives
                    repel_grad -= (neg[i] - anchor[i]) * repel_scale;
                }
            }
            
            let total_grad = attract_grad * attract_scale + repel_grad * learning_rate;
            
            // Update encoder weights
            for j in 0..input_dim.min(self.encoder_weights.len() / latent_dim) {
                let idx = i * input_dim + j;
                if idx < self.encoder_weights.len() {
                    self.encoder_weights[idx] += total_grad;
                }
            }
        }
        
        // Also train MHA on this pair
        if !negatives.is_empty() {
            let keys: Vec<Vec<f32>> = std::iter::once(positive.to_vec())
                .chain(negatives.iter().cloned())
                .collect();
            let values = keys.clone();
            self.multi_head_attention.train_step(anchor, &keys, &values, positive, learning_rate);
        }
    }
    
    /// Apply multi-head attention to context
    pub fn attend_to_context(&self, query_latent: &LatentState, context_latents: &[LatentState]) -> (Vec<f32>, Vec<f32>) {
        if context_latents.is_empty() {
            return (query_latent.latent.clone(), vec![]);
        }
        
        let keys: Vec<Vec<f32>> = context_latents.iter().map(|l| l.latent.clone()).collect();
        let values = keys.clone();
        
        self.multi_head_attention.forward(&query_latent.latent, &keys, &values)
    }
    
    /// Train the CALM encoder/decoder on input-target reconstruction
    /// Encodes input to latent, decodes back, updates weights to minimize reconstruction error
    pub fn train_step(&mut self, input: &[BeamTensor], target: &[BeamTensor], learning_rate: f32) {
        if input.is_empty() || target.is_empty() {
            return;
        }
        
        self.training_steps += 1;
        
        let latent_dim = self.config.latent_dim;
        let input_dim = self.config.chunk_size * 9;
        let chunk_size = self.config.chunk_size.min(input.len());
        
        // Encode input to latent
        let latent = self.encode(input);
        
        // Decode latent back to output
        let decoded = self.decode(&latent);
        
        // Flatten target digits for comparison
        let mut target_flat = Vec::with_capacity(chunk_size * 9);
        for beam in target.iter().take(chunk_size) {
            target_flat.extend_from_slice(&beam.digits);
        }
        while target_flat.len() < self.config.chunk_size * 9 {
            target_flat.push(0.0);
        }
        
        // Flatten decoded digits
        let mut decoded_flat = Vec::with_capacity(chunk_size * 9);
        for beam in decoded.iter().take(chunk_size) {
            decoded_flat.extend_from_slice(&beam.digits);
        }
        while decoded_flat.len() < self.config.chunk_size * 9 {
            decoded_flat.push(0.0);
        }
        
        // Compute reconstruction error per output dimension
        let output_dim = self.config.chunk_size * 9;
        let lr = learning_rate / (1.0 + self.training_steps as f32 * 0.00001);
        
        // Update decoder weights: minimize (decoded - target)^2
        for i in 0..output_dim.min(decoded_flat.len()).min(target_flat.len()) {
            let error = decoded_flat[i] - target_flat[i];
            
            for j in 0..latent_dim.min(latent.latent.len()) {
                let idx = j * output_dim + i;
                if idx < self.decoder_weights.len() {
                    self.decoder_weights[idx] -= lr * error * latent.latent[j];
                }
            }
        }
        
        // Update encoder weights via chain rule: d(loss)/d(encoder) = d(loss)/d(latent) * d(latent)/d(encoder)
        // Compute d(loss)/d(latent) first
        let mut latent_grad = vec![0.0f32; latent_dim];
        for j in 0..latent_dim.min(latent.latent.len()) {
            for i in 0..output_dim.min(decoded_flat.len()).min(target_flat.len()) {
                let error = decoded_flat[i] - target_flat[i];
                let idx = j * output_dim + i;
                if idx < self.decoder_weights.len() {
                    latent_grad[j] += error * self.decoder_weights[idx];
                }
            }
            // Apply tanh derivative: d(tanh(x))/dx = 1 - tanh(x)^2
            let tanh_val = latent.latent[j];
            latent_grad[j] *= 1.0 - tanh_val * tanh_val;
        }
        
        // Flatten input for encoder gradient
        let mut input_flat = Vec::with_capacity(chunk_size * 9);
        for beam in input.iter().take(chunk_size) {
            input_flat.extend_from_slice(&beam.digits);
        }
        while input_flat.len() < self.config.chunk_size * 9 {
            input_flat.push(0.0);
        }
        
        // Update encoder weights
        for i in 0..latent_dim {
            for j in 0..input_dim.min(input_flat.len()) {
                let idx = i * input_dim + j;
                if idx < self.encoder_weights.len() {
                    self.encoder_weights[idx] -= lr * latent_grad[i] * input_flat[j];
                }
            }
        }
        
        // Learn word embeddings from input
        for beam in input.iter().chain(target.iter()) {
            if !beam.word.is_empty() {
                let word_lower = beam.word.to_lowercase();
                let embedding = self.word_embeddings
                    .entry(word_lower)
                    .or_insert_with(|| vec![0.0; latent_dim]);
                
                // Update embedding toward current latent
                for (i, e) in embedding.iter_mut().enumerate() {
                    if i < latent.latent.len() {
                        *e = *e * 0.95 + latent.latent[i] * 0.05;
                    }
                }
            }
        }
    }
    
    /// Compute semantic similarity between two latent states
    pub fn semantic_similarity(&self, a: &LatentState, b: &LatentState) -> f32 {
        if a.latent.is_empty() || b.latent.is_empty() {
            return 0.0;
        }
        
        // Cosine similarity with attention weighting
        let mut dot = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;
        
        for i in 0..a.latent.len().min(b.latent.len()) {
            let weight = self.attention_weights.get(i).copied().unwrap_or(1.0);
            dot += a.latent[i] * b.latent[i] * weight;
            norm_a += a.latent[i] * a.latent[i] * weight;
            norm_b += b.latent[i] * b.latent[i] * weight;
        }
        
        if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a.sqrt() * norm_b.sqrt())
        } else {
            0.0
        }
    }
    
    /// Get word embedding if learned
    pub fn get_word_embedding(&self, word: &str) -> Option<Vec<f32>> {
        self.word_embeddings.get(&word.to_lowercase()).cloned()
    }
    
    /// Compute attention scores over context words
    pub fn attention_over_context(&self, query: &LatentState, context_words: &[&str]) -> Vec<(String, f32)> {
        let mut scores = Vec::with_capacity(context_words.len());
        
        for word in context_words {
            let word_lower = word.to_lowercase();
            if let Some(embedding) = self.word_embeddings.get(&word_lower) {
                // Create a latent state from embedding
                let word_latent = LatentState {
                    latent: embedding.clone(),
                    energy: 1.0,
                    sacred_alignment: 0.5,
                    step: 0,
                };
                let sim = self.semantic_similarity(query, &word_latent);
                scores.push((word_lower, sim));
            } else {
                scores.push((word_lower, 0.1)); // Default low score for unknown words
            }
        }
        
        // Softmax normalization
        let max_score = scores.iter().map(|(_, s)| *s).fold(f32::NEG_INFINITY, f32::max);
        let exp_sum: f32 = scores.iter().map(|(_, s)| (s - max_score).exp()).sum();
        
        if exp_sum > 0.0 {
            for (_, score) in scores.iter_mut() {
                *score = (*score - max_score).exp() / exp_sum;
            }
        }
        
        scores
    }

    /// Encode a chunk of BeamTensors to continuous latent
    pub fn encode(&self, beams: &[BeamTensor]) -> LatentState {
        let chunk_size = self.config.chunk_size.min(beams.len());
        let latent_dim = self.config.latent_dim;
        
        // Flatten beam digits to input
        let mut input = Vec::with_capacity(chunk_size * 9);
        for beam in beams.iter().take(chunk_size) {
            input.extend_from_slice(&beam.digits);
        }
        
        // Pad if needed
        while input.len() < self.config.chunk_size * 9 {
            input.push(0.0);
        }

        // Simple linear encoding (would be neural network in full impl)
        let mut latent = vec![0.0f32; latent_dim];
        let input_dim = self.config.chunk_size * 9;
        for i in 0..latent_dim {
            for j in 0..input_dim.min(input.len()) {
                latent[i] += input[j] * self.encoder_weights[i * input_dim + j];
            }
            // Apply tanh activation
            latent[i] = latent[i].tanh();
        }

        // Score with EBRM
        let energy = self.ebrm.score_trace(beams);

        LatentState {
            latent,
            energy: energy.global_energy,
            sacred_alignment: energy.sacred_alignment,
            step: 0,
        }
    }

    /// Predict next latent state (autoregressive in latent space)
    pub fn predict_next(&self, state: &LatentState) -> LatentState {
        let latent_dim = self.config.latent_dim;
        let mut next_latent = vec![0.0f32; latent_dim];

        // Linear prediction + residual connection
        for i in 0..latent_dim {
            for j in 0..latent_dim {
                next_latent[i] += state.latent[j] * self.predictor_weights[i * latent_dim + j];
            }
            // Residual + tanh
            next_latent[i] = (next_latent[i] + state.latent[i] * 0.5).tanh();
        }

        // Energy decays slightly without grounding
        let energy_decay = 0.95;

        LatentState {
            latent: next_latent,
            energy: state.energy * energy_decay,
            sacred_alignment: state.sacred_alignment,
            step: state.step + 1,
        }
    }

    /// Decode latent state back to BeamTensors
    pub fn decode(&self, state: &LatentState) -> Vec<BeamTensor> {
        let chunk_size = self.config.chunk_size;
        let latent_dim = self.config.latent_dim;
        let output_dim = chunk_size * 9;

        // Linear decoding
        let mut output = vec![0.0f32; output_dim];
        for i in 0..output_dim {
            for j in 0..latent_dim {
                output[i] += state.latent[j] * self.decoder_weights[j * output_dim + i];
            }
            // Sigmoid to get probabilities
            output[i] = 1.0 / (1.0 + (-output[i]).exp());
        }

        // Convert to BeamTensors
        let mut beams = Vec::with_capacity(chunk_size);
        for c in 0..chunk_size {
            let mut digits = [0.0f32; 9];
            for d in 0..9 {
                digits[d] = output[c * 9 + d];
            }
            // Normalize to sum to 1
            let sum: f32 = digits.iter().sum();
            if sum > 0.0 {
                digits.iter_mut().for_each(|d| *d /= sum);
            }

            let mut beam = BeamTensor::default();
            beam.digits = digits;
            beam.confidence = state.energy;
            beam.position = (c as u8 % 9) + 1;
            beams.push(beam);
        }

        beams
    }

    /// Generate K steps in latent space, decode once
    /// This is the K× speedup: instead of K autoregressive token steps,
    /// we do K latent predictions then decode all at once.
    pub fn generate_compressed(&self, initial_beams: &[BeamTensor], steps: usize) -> Vec<BeamTensor> {
        let mut state = self.encode(initial_beams);
        
        // Autoregress in latent space (fast!)
        for _ in 0..steps {
            state = self.predict_next(&state);
            
            // Early exit if energy drops too low
            if state.energy < self.config.energy_threshold {
                break;
            }
        }

        // Decode back to BeamTensors (single decode for K steps)
        self.decode(&state)
    }

    /// Speculative decoding: generate multiple candidates, score with EBRM
    pub fn generate_speculative(&self, initial_beams: &[BeamTensor], steps: usize) -> Vec<BeamTensor> {
        if !self.config.speculative_decoding {
            return self.generate_compressed(initial_beams, steps);
        }

        let batch_size = self.config.batch_size;
        let mut candidates: Vec<Vec<BeamTensor>> = Vec::with_capacity(batch_size);

        // Generate multiple candidates with slight perturbations
        for b in 0..batch_size {
            let mut state = self.encode(initial_beams);
            
            // Add small noise for diversity
            let noise_scale = 0.1 * (b as f32 + 1.0) / batch_size as f32;
            for l in state.latent.iter_mut() {
                *l += noise_scale * ((*l * 1000.0) % 1.0 - 0.5);
            }

            for _ in 0..steps {
                state = self.predict_next(&state);
            }

            candidates.push(self.decode(&state));
        }

        // Score each candidate with EBRM, pick best
        let mut best_idx = 0;
        let mut best_energy = 0.0f32;
        for (i, candidate) in candidates.iter().enumerate() {
            let energy = self.ebrm.score_trace(candidate);
            if energy.global_energy > best_energy {
                best_energy = energy.global_energy;
                best_idx = i;
            }
        }

        candidates.swap_remove(best_idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calm_encode_decode() {
        let config = CALMConfig::new().with_latent_dim(64);
        let engine = CALMEngine::new(config);

        let beams: Vec<BeamTensor> = (0..4).map(|_| BeamTensor::default()).collect();
        let state = engine.encode(&beams);
        
        assert_eq!(state.latent.len(), 64);
        assert!(state.energy >= 0.0);

        let decoded = engine.decode(&state);
        assert!(!decoded.is_empty());
    }

    #[test]
    fn test_calm_generate() {
        let engine = CALMEngine::new(CALMConfig::default());
        let initial: Vec<BeamTensor> = (0..4).map(|_| BeamTensor::default()).collect();
        
        let result = engine.generate_compressed(&initial, 3);
        assert!(!result.is_empty());
    }
}
