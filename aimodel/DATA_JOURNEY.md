# Data Journey: Input Question → Output Answer

## PHASE 0: Startup (Constructor) — ~25 minutes total

### Step 0.1: EmbedVec Init (~1s)
```
EmbedVec::with_persistence() → Load cached embeddings from Sled DB
IF >100 cached embeddings → SKIP HF download (Step 0.2)
```

### Step 0.2: HuggingFace Dataset Loading (~665s = 11 min) ⚠️ BOTTLENECK
```
9 categories × 5 datasets × 500 samples = ~45 datasets
Each dataset: 5 API pages × 100 rows + 1s rate-limit delay
429 errors: exponential backoff (3s, 6s, 12s, 24s, 48s)
Many datasets fail (401/404) → wasted retry time
```
**SPEED FIX**: Skip if EmbedVec cache exists (already implemented but not always triggered)

### Step 0.3: Knowledge Extraction (~30s)
```
19K examples → extract entity-attrs, causal patterns, word embeddings
Early-stop if no growth in 10K examples
Result: ~122K embeddings, ~25K entity-attrs, ~4.5K causal patterns
Sync to CALM engine + RAG engine
```

### Step 0.4: Vortex Pre-seeding (~2s)
```
Seed consciousness vortex from HF entity-attrs
Gap-fill: +2K attrs, +42K relations
Health score: ~97%
```

### Step 0.5: Web Learning (~12s)
```
33 queries → 66 seed URLs → ~74 pages crawled
~5K facts extracted → 25K subjects created
Sync to RAG engine
```

### Step 0.6: CALM Pretraining (~770s = 13 min) ⚠️ BOTTLENECK
```
2000 texts × 25 epochs = 50,000 training steps
Pre-compute 2000 embeddings
gen_loss: 66.7 → 55.0 → 56.2 (U-shaped, overfits after epoch 20)
```
**SPEED FIX**: Reduce epochs 25→10 (loss plateaus at epoch 15, then rises)

### Step 0.7: Pipeline Build (~72s = 1.2 min) ⚠️ BOTTLENECK
```
120K documents → build knowledge base
Result: ~20K subjects, ~173K facts, ~20K embeddings
```

### Step 0.8: GPU Training (~1s)
```
Simplified for eval harness — loads 0 samples, completes immediately
```

---

## PHASE 1: Per-Question Inference (~0.5-2s per question)

### Step 1.0: Few-Shot Context Prepend
```
First 5 questions → exemplars (skipped during inference)
Questions 6+: prepend exemplar Q&A pairs to question text
```

### Step 1.1: Route Decision
```
INPUT: RealBenchmarkQuestion { question, choices, source, correct_answer }

IF source starts with "bAbI" or "GSM8K":
  → SKIP pipeline, go to Step 1.3 (Unified Inference)
IF question contains "def " or ">>> " or source == "HumanEval":
  → SKIP pipeline + unified, go to Step 1.4 (Multi-Expert)
ELSE:
  → Step 1.2 (Knowledge Pipeline)
```

### Step 1.2: Knowledge Pipeline (MMLU, SQuAD, ARC, etc.)
```
pipeline.infer(question, choices):
  1. RETRIEVE: query knowledge base for relevant facts
  2. SCORE each choice:
     a. Fact matching: word overlap between choice and fact objects (×10)
     b. Context matching: choice appears in fact context (×5)
     c. Semantic similarity: cosine(query_embed, choice_embed) × 2.0
     d. Truth check: score_truthfulness() → 0.0 (empty DB)
  3. RANK by score, compute margin-based confidence

IF confidence > 0.6 → RETURN (committed answer)  ← threshold raised from 0.3
ELSE → fall through to Step 1.3
```
**PROBLEM**: Word-matching is noisy. Semantic similarity rewards word overlap, not correctness.

### Step 1.3: Unified Inference Engine
```
Split question into context + question text
unified_engine.infer(context, question, choices):
  - Single forward pass through reasoning layer
  - Entity tracking, location history, path finding (bAbI)
  - Symbolic math evaluation (GSM8K)

IF confidence >= 0.70 → RETURN (committed answer)
ELSE → fall through to Step 1.4
```

### Step 1.4: Multi-Expert Scoring (18+ experts) ⚠️ SLOW PER-QUESTION
```
For EACH choice (4 choices typically):
  Expert 1:  Entity-Attribute match (learned_entity_attrs)
  Expert 2:  Causal pattern match (learned_causal)
  Expert 3:  N-gram frequency scoring
  Expert 4:  RAG retrieval scoring
  Expert 5:  Semantic embedding similarity
  Expert 6:  Theorem prover scoring
  Expert 7:  Logic Tensor Network
  Expert 8:  Deduction engine
  Expert 9:  Pattern template matching
  Expert 10: Attribute-focused attention
  Expert 11: Chain-of-Thought decomposition
  Expert 12: Grounded context extraction
  Expert 13: Sacred attention (369 implication extraction)
  Expert 14: Exhaustive pathway search
  Expert 15: MoE routing
  Expert 16: Vortex cycle refinement
  Expert 17: JEPA target prediction
  Expert 18: Energy-based selection
  Expert 19: Transitive flux reasoning
  Expert 20: CALM web scoring
  Expert 21: Truth checker

→ 21 experts × 4 choices = 84 expert evaluations per question
```

### Step 1.5: Quantum JEPA + Energy Scoring
```
quantum_jepa.quantum_search(question_embed, choice_embeds)
Predict target embedding, compute MSE energy per choice
Combine: 70% expert scores + 30% quantum energy × 10
```

### Step 1.6: Vortex Cycle Refinement
```
iterative_refinement(combined_logits, question_embed, question)
Cycle: 1→2→4→8→7→5→1
```

### Step 1.7: Temperature-Scaled Softmax
```
Temperature = 0.1 (sharp distribution)
Margin-based confidence: best_prob - second_best_prob
```

### Step 1.8: Final Decision
```
IF quantum_confidence > 0.7 → use quantum answer
ELIF expert_best_prob > 0.4 → use expert answer
ELIF quantum == expert → average confidence + 0.1
ELSE → expert answer × 0.8 confidence
```

### Step 1.9: Test-Time Training
```
IF correct: update qa_patterns, entity_attrs, word_embeddings, CALM
IF correct + high confidence: train CALM encoder/decoder
```

→ OUTPUT: (predicted_answer_index, confidence)

---

## BOTTLENECK SUMMARY

| Phase | Time | % of Total | Fix |
|-------|------|-----------|-----|
| HF Loading | 665s | 44% | Skip if cache exists (already impl) |
| CALM Pretrain | 770s | 51% | Reduce 25→10 epochs |
| Pipeline Build | 72s | 5% | Keep (one-time) |
| Web Learning | 12s | <1% | Keep (fast) |
| **Per-question** | **~1s** | - | Pipeline threshold 0.3→0.6 |

**Total startup: ~25 min → Target: ~5 min with fixes**
