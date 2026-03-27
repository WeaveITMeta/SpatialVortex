# MSA: Memory Sparse Attention — Research & SpatialVortex Integration Roadmap

**Source:** EverMind-AI/MSA — Chen et al., Zenodo March 2026  
**DOI:** 10.5281/zenodo.19103670  
**Repo:** https://github.com/EverMind-AI/MSA

---

## 1. What MSA Is and Why It Matters

MSA is an **end-to-end trainable sparse latent-state memory architecture** that grows long-term memory *into* the attention mechanism itself rather than bolting retrieval on top.

### Why existing approaches fail at scale

| Approach | Fatal flaw |
|---|---|
| RAG / agents | Open-book exam: cross-document reasoning collapses, accuracy depends entirely on retrieval quality |
| Linear attention / RNNs | Compressed memory: precision degrades monotonically — the longer the context, the fuzzier the recall |
| Brute-force KV window expansion | Quadratic compute — 100M tokens is computationally unreachable |
| Hybrid (Mamba, RWKV) | Noticeably degrade at ≥128K tokens on RULER benchmarks |

### What MSA does instead

→ **No compression of content** — compresses routing keys only (K̄ᵣ), content K/V (K̄, V̄) stays lossless  
→ **Teaches the model to pick the right fragments** — top-k sparse selection is differentiable and trained end-to-end  
→ **Knows where and when each memory came from** — document-wise RoPE resets positions per document, so the model never confuses position 0 of document 47 with position 0 of the query  
→ **Chains scattered clues** — Memory Interleave alternates retrieval and generation, enabling multi-hop reasoning  

---

## 2. The Three Core Mechanisms (Technical Deep Dive)

### 2.1 Memory-Sparse Attention Layer

Each MSA layer maintains three projections per document chunk:
- **K̄ᵣ** — compressed routing key (mean-pooled over heads then over tokens within the chunk)
- **K̄** — full-rank content key (stored in host DRAM)
- **V̄** — full-rank content value (stored in host DRAM)

At inference time for a query Q:
1. Project Q → Qᵣ (router query, cheap, fits GPU)
2. Cosine similarity: Qᵣ · K̄ᵣᵀ → relevance scores over all chunks
3. Top-k selection: pick k most relevant chunks
4. Load selected K̄, V̄ on-demand from CPU DRAM → GPU
5. Concatenate with local K/V → standard sparse attention

**Complexity:** O(L) in both training and inference (vs O(L²) for full attention)  
**Key property:** Differentiable end-to-end through the top-k step (auxiliary routing loss)

### 2.2 Document-wise RoPE (Positional Encoding)

Standard RoPE assigns globally increasing positions. This causes "position drift" when training on 64K but inferring at 100M — the model has never seen position 50,000,000.

MSA uses **two RoPE modes simultaneously**:
- **Parallel (document-wise) RoPE:** Each document chunk resets to position 0. The model learns "what position X means within a document" uniformly regardless of where that document sits in the memory bank.
- **Global RoPE (active context):** The query's position is offset by k (the number of retrieved chunks), preserving causal order: `[retrieved_chunk_0 .. retrieved_chunk_k-1 | query_tokens | generation]`

This is why MSA can train on 64K and extrapolate to 100M with < 9% degradation.

### 2.3 Memory Interleave (Multi-hop Reasoning)

Single-pass retrieval fails on multi-hop questions because the first query often can not express the full chain of reasoning needed. Memory Interleave solves this:

```
Round 1: query → retrieve → expand context → generate partial answer
Round 2: partial_answer + query → retrieve new fragments → expand → generate fuller answer
...
Round N: final generation
```

This is an **adaptive alternating loop** of "generative retrieval → context expansion → generation." The number of rounds adapts per query. The paper reports this contributes substantially to multi-hop accuracy (removing it causes 5–37% drops on 2WikiMultiHopQA, HotpotQA, MuSiQue).

---

## 3. Results (What Was Beaten)

### Scale benchmark (MS MARCO, 16K → 100M tokens)
- MSA: **< 9% accuracy degradation** across the full range
- Backbone without MSA: collapses at 128K (down to 24.69% at 1M)
- Hybrid linear-attention models: noticeably degrade at ≥ 128K / 256K

### Long-context QA (9 datasets, 277K → 10M token memory banks)
- **vs same-backbone RAG (Qwen3-4B):** +16.0% over standard RAG, +11.5% over RAG+rerank, +14.8% over HippoRAG2
- **vs best-of-breed RAG (KaLMv2 + Qwen3-235B or Llama-3.3-70B):** MSA 4B beats 235B-scale RAG systems on 4/9 datasets, average score 3.760 (+5–11% relative)

### NIAH (Needle-in-a-Haystack, RULER, 32K → 1M)
- MSA: **94.84% at 1M tokens**
- RL-MemoryAgent-14B: stable but weaker absolute accuracy and steeper decay
- Unmodified backbone: 24.69% at 1M

**The key headline:** a **4B-parameter MSA model beats 235B-scale RAG stacks** on long-context reasoning. This is the efficiency unlocked by making memory native to attention.

---

## 4. How MSA Maps to SpatialVortex

SpatialVortex already has analogues of all three MSA mechanisms. The path forward is **replacement and refinement**, not greenfield build.

### 4.1 Current SpatialVortex Architecture vs MSA

| MSA Component | SpatialVortex Existing Analogue | Gap |
|---|---|---|
| Routing key K̄ᵣ (mean-pooled, GPU-resident) | `RAGSearchEngine.text_to_embedding()` (hash-based, 384-dim) | Hash embeddings ≠ learned routing keys. No top-k differentiable selection. |
| Content K̄/V̄ (CPU DRAM, on-demand fetch) | `RAGEngine.documents: Vec<Document>` (in-memory only) | No tiered GPU/CPU split. No on-demand fetch. |
| Document-wise RoPE | None — `SacredDynamicAttention` uses flat position indexing | Missing entirely. Causes position drift at long context. |
| Top-k sparse attention (forward pass) | `SacredDynamicAttention.forward()` (dense over all keys) | Dense, not sparse. No top-k document selection gate. |
| Memory Interleave (multi-hop) | None — single-pass RAG retrieval | Missing entirely. This is why multi-hop QA (HotpotQA, 2Wiki) is weak. |
| Auxiliary routing loss (end-to-end training) | No training loop in current inference engine | Requires training infrastructure to fully adopt. |
| KV cache compression (K̄ = chunk-mean-pool) | `MultiHeadLatentAttention.compress_kv()` — φ-ratio downsampling | Exists but uses naive downsampling, not mean-pooling over learned chunk boundaries |

### 4.2 What Can Be Implemented Now (No Training Required)

The MSA inference pipeline can be approximated in the current eval-harness framework by treating the existing `RAGSearchEngine` as the memory bank and implementing the routing and interleave patterns.

**Priority 1 — Document-wise position reset in `try_passage_span_commit`**

Currently the `PASSAGE\t<passage>\t<question>` sentinel is extracted and scanned verbatim. The sentence-proximity scorer already does something analogous to document-wise attention (finding the sentence with highest keyword density). This is already partially correct but uses raw char offsets rather than document-relative positions.

Concretely: when scoring choices in `try_passage_span_commit`, the proximity window calculation at line ~6580 uses absolute offsets into the passage. This should be replaced with a normalized position within the relevant sentence cluster — equivalent to document-wise RoPE at the scoring level.

**Priority 2 — Sparse top-k routing in `RAGSearchEngine.search()`**

Current `search()` returns all facts above a relevance threshold and truncates to `max_results=5`. This is a soft threshold, not a hard top-k selection. Replace with:
1. Compute cosine similarity for all entries
2. Hard top-k (k=5 by default, adaptive per query complexity)
3. Apply MMR only within the top-k, not as the selection mechanism

This matches MSA's routing: K̄ᵣ scoring → top-k → load K̄/V̄.

**Priority 3 — Memory Interleave in `generative_inference()`**

Multi-hop questions (HotpotQA, 2WikiMultiHopQA, MuSiQue) require iterative retrieval. The current pipeline does one RAG search then commits. A 2-round interleave would look like:

```
Round 1: question → SPAN-COMMIT or multi-expert → extract partial answer phrase
Round 2: question + partial_answer → re-run RAGSearchEngine → re-score choices
Commit: take best score across rounds
```

The bAbI multi-hop tasks (bAbI4 at 68.4%, bAbI10 at 83.2%) would directly benefit from this.

### 4.3 The Full MSA Integration Path (Requires Training Infrastructure)

This is a longer-term architectural change. The steps in order:

1. **Replace hash embeddings with learned routing keys**
   - File: `aimodel/src/ml/rag_search.rs` → `text_to_embedding()`
   - Replace `DefaultHasher`-based projection with a real embedding model (even a small 64-dim TF-IDF trained on the knowledge base would be strictly better)
   - The routing key K̄ᵣ in MSA is trained — in SpatialVortex the analogue would be training the `knowledge_base` projection weights using the benchmark QA pairs as supervision

2. **Implement document-wise RoPE in `SacredDynamicAttention`**
   - File: `aimodel/src/ml/generative_arch.rs`
   - Add a `doc_id: usize` parameter to `forward()`
   - Each document chunk resets `position_idx` to 0 when `doc_id` changes
   - The query's position starts at `top_k` (offset, same as MSA)

3. **Implement chunk-mean-pool K̄ for `MultiHeadLatentAttention`**
   - File: `aimodel/src/ml/sacred_moe.rs` → `compress_kv()`
   - Replace φ-ratio downsampling with `mean_pool_over_chunk(k, chunk_size=32)`
   - This matches MSA's compression: K̄ = mean(K[chunk_start..chunk_end])

4. **Tiered memory storage (GPU K̄ᵣ / CPU K̄V̄)**
   - File: `aimodel/src/cognition/memory.rs` → `MemoryStore`
   - Routing keys stay in a `Vec<Vec<f32>>` (equivalent to GPU-resident in CPU-only context)
   - Content K/V stored in a lazy `HashMap<chunk_id, (Vec<f32>, Vec<f32>)>` that is loaded only when selected by top-k
   - This mimics MSA's GPU/DRAM split at the CPU/cache level

5. **Memory Interleave loop in benchmark evaluation**
   - File: `aimodel/src/data/real_benchmarks.rs` → `generative_inference()`
   - Add `max_interleave_rounds: usize` parameter (default 1, set to 2 for multi-hop tasks)
   - Wrap the existing pipeline in a loop, feeding partial answers back into the query

---

## 5. Immediate Action Items (High ROI, No Training Required)

These can be implemented in the current codebase right now and are expected to improve multi-hop task accuracy:

### 5.1 Two-Round Memory Interleave for bAbI Multi-hop

The bAbI tasks 4, 10, 13 require chaining facts across multiple sentences. A simple 2-round interleave in `generative_inference()` would:
- Round 1: score as normal, extract the best partial answer
- Round 2: augment query with partial answer, re-score passage sentences, re-commit

Expected improvement: bAbI4 (68.4% → ~80%), bAbI10 (83.2% → ~90%)

### 5.2 Top-k Hard Selection in RAGSearchEngine

Replace soft threshold + truncation with hard top-k to reduce noise in the knowledge expert's signal. This directly mirrors MSA's routing design and should improve ARC and CommonsenseQA.

### 5.3 Document-Relative Proximity in SPAN-COMMIT

In `try_passage_span_commit`, the proximity bonus window (currently ±120 chars of absolute passage position) should be normalized by sentence position rather than character offset. A choice in the most relevant sentence should get the full bonus regardless of where that sentence sits in the passage.

---

## 6. SpatialVortex Term Mapping for MSA Concepts

| MSA Term | SpatialVortex Equivalent |
|---|---|
| Memory bank (encoded corpus) | `RAGEngine.documents` + `MemoryStore.memories` |
| Chunk-mean-pooled routing key K̄ᵣ | `RAGSearchEngine.text_to_embedding()` output |
| Top-k document selection | `RAGEngine.retrieve()` top_k |
| Document-wise RoPE | Sacred position reset per document (vortex cycle restart) |
| Memory Interleave | Multi-round reasoning in `generative_inference()` |
| Auxiliary routing loss | RSI (Runtime Self-Improving) accuracy feedback |
| Memory Parallel (multi-GPU) | SacredSwarm (multi-agent) |
| Active context (query + retrieved) | `fewshot_context` + `PASSAGE\t` sentinel |
| Global RoPE offset by k | Sacred position offset by number of retrieved chunks |

---

## 7. Training Notes (For Future Integration)

MSA was trained with:
- **158.95B tokens** of continuous pretraining with auxiliary routing loss
- **Two-stage SFT:** 8K → 64K curriculum (not 64K directly — curriculum matters)
- Ablations show curriculum, Memory Interleave, continuous pretraining, and injecting original text each contribute 5–37% independently

For SpatialVortex, the analogous training signal already exists: every benchmark run with `[OK]` / `[WRONG]` feedback is supervision data. The RSI mechanism already uses this for strategy tuning. The next step is using this feedback to train the routing key projection weights.

---

*Document generated: March 2026*  
*Benchmark baseline at time of writing: SQuAD 44.4%, ARC 66.7%, HellaSwag 55.6%*
