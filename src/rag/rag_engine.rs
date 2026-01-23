//! High-Performance 3-Stage RAG Engine
//!
//! Architecture:
//! 1. HNSW Retrieval → topK candidates (sublinear graph navigation)
//! 2. embedvec Dot Rerank → topR reranked (SIMD-optimized scoring)
//! 3. Autoregressive Decode → coherent text generation
//!
//! Key optimizations:
//! - SV8 blocked-transpose layout for AVX2 (8 candidates per SIMD lane)
//! - Zero-allocation hot path with preallocated buffers
//! - L2-normalized vectors (cosine = dot product)
//! - MMR diversity for answer quality

use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

// ============================================================================
// Core Traits
// ============================================================================

/// Query embedding interface
pub trait Embedder: Send + Sync {
    /// Embed query text to normalized vector (length d)
    fn embed_query(&self, text: &str) -> Vec<f32>;
    
    /// Embedding dimension
    fn dimension(&self) -> usize;
}

/// HNSW retrieval interface
pub trait Retriever: Send + Sync {
    /// Search for topK candidate document IDs
    fn search(&self, q: &[f32], top_k: usize) -> Vec<u32>;
    
    /// Get raw document vector by ID
    fn get_vector(&self, doc_id: u32) -> Option<&[f32]>;
}

/// Fast dot-product reranking interface
pub trait Reranker: Send + Sync {
    /// Rerank candidates, return topR (doc_id, score) pairs
    fn rerank(&self, q: &[f32], doc_ids: &[u32], top_r: usize) -> Vec<(u32, f32)>;
}

/// Text generation interface
pub trait Generator: Send + Sync {
    /// Generate response given prompt with context
    fn generate(&self, prompt: &str) -> String;
    
    /// Generate with token limit
    fn generate_with_limit(&self, prompt: &str, max_tokens: usize) -> String;
}

// ============================================================================
// Aligned Memory Allocation (64-byte for AVX2/cacheline)
// ============================================================================

/// 64-byte aligned f32 buffer for SIMD operations
pub struct AlignedF32 {
    ptr: NonNull<f32>,
    len: usize,
    layout: Layout,
}

impl AlignedF32 {
    /// Allocate aligned buffer
    pub fn new(len: usize) -> Self {
        Self::with_alignment(len, 64)
    }
    
    /// Allocate with specific alignment
    pub fn with_alignment(len: usize, align: usize) -> Self {
        let bytes = len * std::mem::size_of::<f32>();
        let layout = Layout::from_size_align(bytes.max(align), align).unwrap();
        let raw = unsafe { alloc(layout) } as *mut f32;
        let ptr = NonNull::new(raw).expect("allocation failed");
        
        // Zero-initialize
        unsafe {
            std::ptr::write_bytes(ptr.as_ptr(), 0, len);
        }
        
        Self { ptr, len, layout }
    }
    
    #[inline]
    pub fn as_ptr(&self) -> *const f32 {
        self.ptr.as_ptr()
    }
    
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut f32 {
        self.ptr.as_ptr()
    }
    
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    #[inline]
    pub fn as_slice(&self) -> &[f32] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
    
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
    
    /// Fill with zeros
    pub fn clear(&mut self) {
        unsafe {
            std::ptr::write_bytes(self.ptr.as_ptr(), 0, self.len);
        }
    }
}

impl Drop for AlignedF32 {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr() as *mut u8, self.layout);
        }
    }
}

unsafe impl Send for AlignedF32 {}
unsafe impl Sync for AlignedF32 {}

// ============================================================================
// SV8 Blocked-Transpose Layout for AVX2 SIMD Reranking
// ============================================================================

/// SV8 rerank worker with preallocated buffers
/// 
/// Memory layout (blocked transpose):
/// - Block size B=8 (AVX2 = 8×f32)
/// - K candidates → K_pad = (K+7) & !7 padded
/// - num_blocks = K_pad / 8
/// 
/// Indexing: scratch_sv8[(block*d + dim)*8 + lane]
/// This makes AVX2 lanes represent different candidates, not dimensions.
pub struct RerankWorker {
    /// Maximum candidates supported
    max_k: usize,
    /// Embedding dimension
    dimension: usize,
    /// SV8 scratch buffer: num_blocks * d * 8 floats
    scratch_sv8: AlignedF32,
    /// Score output buffer
    scores: AlignedF32,
    /// Candidate ID buffer
    cand_ids: Vec<u32>,
    /// Index buffer for selection
    indices: Vec<usize>,
}

impl RerankWorker {
    /// Create worker with capacity for max_k candidates
    pub fn new(max_k: usize, dimension: usize) -> Self {
        let max_k_pad = (max_k + 7) & !7;
        let num_blocks = max_k_pad / 8;
        let scratch_len = num_blocks * dimension * 8;
        
        Self {
            max_k,
            dimension,
            scratch_sv8: AlignedF32::new(scratch_len),
            scores: AlignedF32::new(max_k_pad),
            cand_ids: vec![0u32; max_k],
            indices: vec![0usize; max_k],
        }
    }
    
    /// Fill SV8 scratch buffer from row-major document vectors
    /// 
    /// docs_rm: flat row-major buffer, doc i at offset i*d
    #[inline]
    pub fn fill_scratch_sv8(&mut self, docs_rm: &[f32], doc_ids: &[u32]) {
        let k = doc_ids.len().min(self.max_k);
        let d = self.dimension;
        
        // Clear scratch
        self.scratch_sv8.clear();
        
        // Copy candidate IDs
        self.cand_ids[..k].copy_from_slice(&doc_ids[..k]);
        
        // Blocked transpose into SV8 layout
        let scratch = self.scratch_sv8.as_mut_slice();
        for (i, &doc_id) in doc_ids[..k].iter().enumerate() {
            let b = i >> 3;      // block = i / 8
            let l = i & 7;       // lane = i % 8
            let src_offset = (doc_id as usize) * d;
            
            if src_offset + d <= docs_rm.len() {
                for j in 0..d {
                    scratch[(b * d + j) * 8 + l] = docs_rm[src_offset + j];
                }
            }
        }
    }
    
    /// Score all candidates using dot product
    /// 
    /// Returns slice of scores (length = k_pad)
    #[inline]
    pub fn score_candidates(&mut self, query: &[f32], k: usize) -> &[f32] {
        let k_pad = (k + 7) & !7;
        let num_blocks = k_pad / 8;
        let d = self.dimension;
        
        // Use SIMD scoring if available
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "fma"))]
        unsafe {
            self.score_blocks_avx2(query, num_blocks);
        }
        
        #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "fma")))]
        {
            self.score_blocks_scalar(query, num_blocks);
        }
        
        &self.scores.as_slice()[..k_pad]
    }
    
    /// AVX2 SIMD scoring kernel (8 candidates per iteration)
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2", target_feature = "fma"))]
    #[inline]
    unsafe fn score_blocks_avx2(&mut self, query: &[f32], num_blocks: usize) {
        use std::arch::x86_64::*;
        
        let d = self.dimension;
        let scratch = self.scratch_sv8.as_ptr();
        let scores_out = self.scores.as_mut_ptr();
        let q = query.as_ptr();
        
        for b in 0..num_blocks {
            let mut acc = _mm256_setzero_ps();
            let base = scratch.add(b * d * 8);
            
            for j in 0..d {
                // Load 8 floats = dim j across 8 candidates (contiguous in SV8)
                let x = _mm256_loadu_ps(base.add(j * 8));
                // Broadcast query[j]
                let qj = _mm256_set1_ps(*q.add(j));
                // FMA: acc += x * qj
                acc = _mm256_fmadd_ps(x, qj, acc);
            }
            
            _mm256_storeu_ps(scores_out.add(b * 8), acc);
        }
    }
    
    /// Scalar fallback scoring
    #[inline]
    fn score_blocks_scalar(&mut self, query: &[f32], num_blocks: usize) {
        let d = self.dimension;
        let scratch = self.scratch_sv8.as_slice();
        let scores_out = self.scores.as_mut_slice();
        
        for b in 0..num_blocks {
            for l in 0..8 {
                let mut acc = 0.0f32;
                for j in 0..d {
                    acc += scratch[(b * d + j) * 8 + l] * query[j];
                }
                scores_out[b * 8 + l] = acc;
            }
        }
    }
    
    /// Select top-R candidates without full sort
    pub fn select_top_r(&mut self, k: usize, top_r: usize) -> Vec<(u32, f32)> {
        let top_r = top_r.min(k);
        let scores = self.scores.as_slice();
        
        // Initialize indices
        self.indices.clear();
        self.indices.extend(0..k);
        
        // Partial sort to get top_r
        self.indices[..k].select_nth_unstable_by(top_r.saturating_sub(1), |&a, &b| {
            scores[b].partial_cmp(&scores[a]).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Collect results
        self.indices[..top_r]
            .iter()
            .map(|&i| (self.cand_ids[i], scores[i]))
            .collect()
    }
    
    /// Rerank with MMR diversity (Max Marginal Relevance)
    /// 
    /// Balances relevance and diversity to avoid near-duplicate results
    pub fn rerank_with_mmr(
        &mut self,
        docs_rm: &[f32],
        doc_ids: &[u32],
        query: &[f32],
        top_r: usize,
        lambda: f32,  // 0.0 = max diversity, 1.0 = max relevance
    ) -> Vec<(u32, f32)> {
        let k = doc_ids.len().min(self.max_k);
        let d = self.dimension;
        
        // First pass: compute query-doc scores
        self.fill_scratch_sv8(docs_rm, doc_ids);
        let _ = self.score_candidates(query, k);
        let scores = self.scores.as_slice();
        
        // MMR selection
        let mut selected: Vec<(u32, f32)> = Vec::with_capacity(top_r);
        let mut used = vec![false; k];
        
        for _ in 0..top_r {
            let mut best_idx = None;
            let mut best_mmr = f32::NEG_INFINITY;
            
            for i in 0..k {
                if used[i] {
                    continue;
                }
                
                let relevance = scores[i];
                
                // Compute max similarity to already selected docs
                let mut max_sim = 0.0f32;
                for &(sel_id, _) in &selected {
                    let sim = self.doc_similarity(docs_rm, doc_ids[i], sel_id, d);
                    max_sim = max_sim.max(sim);
                }
                
                // MMR score
                let mmr = lambda * relevance - (1.0 - lambda) * max_sim;
                
                if mmr > best_mmr {
                    best_mmr = mmr;
                    best_idx = Some(i);
                }
            }
            
            if let Some(idx) = best_idx {
                used[idx] = true;
                selected.push((doc_ids[idx], scores[idx]));
            } else {
                break;
            }
        }
        
        selected
    }
    
    /// Compute dot product similarity between two docs
    #[inline]
    fn doc_similarity(&self, docs_rm: &[f32], id_a: u32, id_b: u32, d: usize) -> f32 {
        let offset_a = (id_a as usize) * d;
        let offset_b = (id_b as usize) * d;
        
        if offset_a + d > docs_rm.len() || offset_b + d > docs_rm.len() {
            return 0.0;
        }
        
        let a = &docs_rm[offset_a..offset_a + d];
        let b = &docs_rm[offset_b..offset_b + d];
        
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }
}

// ============================================================================
// Document Store (Contiguous Row-Major)
// ============================================================================

/// Flat document store optimized for SIMD access
pub struct DocumentStore {
    /// Flat row-major vectors: doc i at offset i*d
    vectors: AlignedF32,
    /// Document texts
    texts: Vec<String>,
    /// Embedding dimension
    dimension: usize,
    /// Number of documents
    count: usize,
}

impl DocumentStore {
    /// Create store with capacity
    pub fn with_capacity(capacity: usize, dimension: usize) -> Self {
        Self {
            vectors: AlignedF32::new(capacity * dimension),
            texts: Vec::with_capacity(capacity),
            dimension,
            count: 0,
        }
    }
    
    /// Add document (vector must be L2-normalized)
    pub fn add(&mut self, vector: &[f32], text: String) -> u32 {
        assert_eq!(vector.len(), self.dimension);
        
        let id = self.count as u32;
        let offset = self.count * self.dimension;
        
        // Copy vector
        let dest = &mut self.vectors.as_mut_slice()[offset..offset + self.dimension];
        dest.copy_from_slice(vector);
        
        self.texts.push(text);
        self.count += 1;
        
        id
    }
    
    /// Get vector by ID
    pub fn get_vector(&self, id: u32) -> Option<&[f32]> {
        let offset = (id as usize) * self.dimension;
        if offset + self.dimension <= self.vectors.len() {
            Some(&self.vectors.as_slice()[offset..offset + self.dimension])
        } else {
            None
        }
    }
    
    /// Get text by ID
    pub fn get_text(&self, id: u32) -> Option<&str> {
        self.texts.get(id as usize).map(|s| s.as_str())
    }
    
    /// Get raw vector buffer for SIMD operations
    pub fn vectors_raw(&self) -> &[f32] {
        self.vectors.as_slice()
    }
    
    /// Number of documents
    pub fn len(&self) -> usize {
        self.count
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    
    /// Embedding dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

// ============================================================================
// SpatialVortex RAG Engine
// ============================================================================

/// Configuration for RAG engine
#[derive(Debug, Clone)]
pub struct RagConfig {
    /// HNSW candidates to retrieve
    pub top_k: usize,
    /// Reranked results to keep
    pub top_r: usize,
    /// MMR lambda (0=diversity, 1=relevance)
    pub mmr_lambda: f32,
    /// Max tokens for generation
    pub max_tokens: usize,
    /// Context template
    pub context_template: String,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            top_k: 200,
            top_r: 20,
            mmr_lambda: 0.7,
            max_tokens: 512,
            context_template: "Answer using the context.\n\nCONTEXT:\n{context}\n\nQUESTION:\n{question}\n\nANSWER:".to_string(),
        }
    }
}

/// High-performance 3-stage RAG engine
pub struct SpatialVortexRag<E, R, G>
where
    E: Embedder,
    R: Retriever,
    G: Generator,
{
    embedder: E,
    retriever: R,
    generator: G,
    doc_store: DocumentStore,
    rerank_worker: RerankWorker,
    config: RagConfig,
}

impl<E, R, G> SpatialVortexRag<E, R, G>
where
    E: Embedder,
    R: Retriever,
    G: Generator,
{
    /// Create new RAG engine
    pub fn new(
        embedder: E,
        retriever: R,
        generator: G,
        doc_store: DocumentStore,
        config: RagConfig,
    ) -> Self {
        let dimension = embedder.dimension();
        let rerank_worker = RerankWorker::new(config.top_k, dimension);
        
        Self {
            embedder,
            retriever,
            generator,
            doc_store,
            rerank_worker,
            config,
        }
    }
    
    /// Answer a question using 3-stage RAG
    /// 
    /// 1. Embed query
    /// 2. HNSW retrieve topK candidates
    /// 3. Rerank with MMR to topR
    /// 4. Build context and generate
    pub fn answer(&mut self, question: &str) -> String {
        // Stage 1: Embed query (normalized)
        let query = self.embedder.embed_query(question);
        
        // Stage 2: HNSW retrieval
        let candidates = self.retriever.search(&query, self.config.top_k);
        
        if candidates.is_empty() {
            return self.generator.generate(&format!(
                "No relevant context found. Question: {}", question
            ));
        }
        
        // Stage 3: Rerank with MMR
        let reranked = self.rerank_worker.rerank_with_mmr(
            self.doc_store.vectors_raw(),
            &candidates,
            &query,
            self.config.top_r,
            self.config.mmr_lambda,
        );
        
        // Build context
        let mut context = String::new();
        for (doc_id, score) in &reranked {
            if let Some(text) = self.doc_store.get_text(*doc_id) {
                context.push_str(&format!(
                    "\n[doc {} | score {:.4}]\n{}\n",
                    doc_id, score, text
                ));
            }
        }
        
        // Format prompt
        let prompt = self.config.context_template
            .replace("{context}", &context)
            .replace("{question}", question);
        
        // Stage 4: Generate
        self.generator.generate_with_limit(&prompt, self.config.max_tokens)
    }
    
    /// Answer with custom config overrides
    pub fn answer_with_config(
        &mut self,
        question: &str,
        top_k: usize,
        top_r: usize,
        max_tokens: usize,
    ) -> String {
        let query = self.embedder.embed_query(question);
        let candidates = self.retriever.search(&query, top_k);
        
        if candidates.is_empty() {
            return self.generator.generate(&format!(
                "No relevant context found. Question: {}", question
            ));
        }
        
        let reranked = self.rerank_worker.rerank_with_mmr(
            self.doc_store.vectors_raw(),
            &candidates,
            &query,
            top_r,
            self.config.mmr_lambda,
        );
        
        let mut context = String::new();
        for (doc_id, score) in &reranked {
            if let Some(text) = self.doc_store.get_text(*doc_id) {
                context.push_str(&format!("\n[doc {} | score {:.4}]\n{}\n", doc_id, score, text));
            }
        }
        
        let prompt = self.config.context_template
            .replace("{context}", &context)
            .replace("{question}", question);
        
        self.generator.generate_with_limit(&prompt, max_tokens)
    }
    
    /// Get retrieval-only results (for benchmarking)
    pub fn retrieve_only(&mut self, question: &str) -> Vec<(u32, f32)> {
        let query = self.embedder.embed_query(question);
        let candidates = self.retriever.search(&query, self.config.top_k);
        
        self.rerank_worker.rerank_with_mmr(
            self.doc_store.vectors_raw(),
            &candidates,
            &query,
            self.config.top_r,
            self.config.mmr_lambda,
        )
    }
}

// ============================================================================
// Utility: L2 Normalization
// ============================================================================

/// Normalize vector to unit length (in-place)
#[inline]
pub fn normalize_l2(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-10 {
        let inv_norm = 1.0 / norm;
        for x in v.iter_mut() {
            *x *= inv_norm;
        }
    }
}

/// Normalize vector to unit length (returns new vec)
#[inline]
pub fn normalized_l2(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-10 {
        let inv_norm = 1.0 / norm;
        v.iter().map(|x| x * inv_norm).collect()
    } else {
        v.to_vec()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aligned_allocation() {
        let buf = AlignedF32::new(1024);
        assert_eq!(buf.len(), 1024);
        
        // Check alignment (64-byte)
        let ptr = buf.as_ptr() as usize;
        assert_eq!(ptr % 64, 0);
    }
    
    #[test]
    fn test_rerank_worker_sv8_layout() {
        let mut worker = RerankWorker::new(256, 384);
        
        // Create mock doc vectors (row-major)
        let num_docs = 100;
        let d = 384;
        let docs_rm: Vec<f32> = (0..num_docs * d)
            .map(|i| ((i % 100) as f32) / 100.0)
            .collect();
        
        // Mock candidate IDs
        let doc_ids: Vec<u32> = (0..50).collect();
        
        // Fill SV8 scratch
        worker.fill_scratch_sv8(&docs_rm, &doc_ids);
        
        // Verify layout
        let scratch = worker.scratch_sv8.as_slice();
        
        // Check doc 0, dim 0 is at block 0, lane 0
        assert!((scratch[0] - docs_rm[0]).abs() < 1e-6);
        
        // Check doc 8, dim 0 is at block 1, lane 0
        assert!((scratch[d * 8] - docs_rm[8 * d]).abs() < 1e-6);
    }
    
    #[test]
    fn test_rerank_worker_scoring() {
        let mut worker = RerankWorker::new(64, 8);
        
        // Simple test: 16 docs, 8 dims
        let d = 8;
        let num_docs = 16;
        
        // Create normalized vectors
        let mut docs_rm = vec![0.0f32; num_docs * d];
        for i in 0..num_docs {
            for j in 0..d {
                docs_rm[i * d + j] = if j == i % d { 1.0 } else { 0.0 };
            }
        }
        
        let doc_ids: Vec<u32> = (0..num_docs as u32).collect();
        
        // Query = [1, 0, 0, 0, 0, 0, 0, 0]
        let query = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        
        worker.fill_scratch_sv8(&docs_rm, &doc_ids);
        let scores = worker.score_candidates(&query, num_docs);
        
        // Doc 0 should have score 1.0 (exact match)
        assert!((scores[0] - 1.0).abs() < 1e-5);
        
        // Doc 8 should also have score 1.0 (same pattern)
        assert!((scores[8] - 1.0).abs() < 1e-5);
    }
    
    #[test]
    fn test_document_store() {
        let mut store = DocumentStore::with_capacity(100, 4);
        
        let v1 = vec![1.0, 0.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0, 0.0];
        
        let id1 = store.add(&v1, "doc one".to_string());
        let id2 = store.add(&v2, "doc two".to_string());
        
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(store.len(), 2);
        
        assert_eq!(store.get_vector(0), Some(v1.as_slice()));
        assert_eq!(store.get_text(1), Some("doc two"));
    }
    
    #[test]
    fn test_normalize_l2() {
        let mut v = vec![3.0, 4.0];
        normalize_l2(&mut v);
        
        assert!((v[0] - 0.6).abs() < 1e-6);
        assert!((v[1] - 0.8).abs() < 1e-6);
        
        // Check unit length
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_top_r_selection() {
        let mut worker = RerankWorker::new(64, 4);
        
        // Create docs with known scores
        let d = 4;
        let num_docs = 20;
        let mut docs_rm = vec![0.0f32; num_docs * d];
        
        // Doc i has score proportional to i
        for i in 0..num_docs {
            docs_rm[i * d] = i as f32 / 10.0;
        }
        
        let doc_ids: Vec<u32> = (0..num_docs as u32).collect();
        let query = vec![1.0, 0.0, 0.0, 0.0];
        
        worker.fill_scratch_sv8(&docs_rm, &doc_ids);
        let _ = worker.score_candidates(&query, num_docs);
        let top5 = worker.select_top_r(num_docs, 5);
        
        // Top 5 should be docs 19, 18, 17, 16, 15 (highest scores)
        assert_eq!(top5.len(), 5);
        assert_eq!(top5[0].0, 19);
    }
}
