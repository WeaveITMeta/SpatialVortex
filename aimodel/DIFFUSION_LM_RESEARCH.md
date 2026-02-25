# Diffusion Language Models — Research Notes for SpatialVortex

## The Core Idea

**Diffusion LMs generate text by iteratively unmasking tokens, not predicting them left-to-right.**

Instead of autoregressive generation (predict next token given previous tokens), diffusion LMs:
1. Start with a sequence of all `[MASK]` tokens
2. Run a transformer to predict what ALL masked tokens should be simultaneously
3. Unmask the most confident predictions, re-mask the rest
4. Repeat for T steps until fully unmasked

This is fundamentally different from AR models and directly solves the problem SpatialVortex has:
**hash-based BeamTensor embeddings can't decode back to words** because the mapping isn't invertible.
With diffusion, you don't decode latent states → you predict token IDs directly from vocabulary logits.

---

## Paper 1: MDLM — Masked Discrete Diffusion Language Models
**arXiv: 2406.07524 | NeurIPS 2024 | Simplest approach**

### Key Insight
Masked diffusion loss simplifies to a **mixture of classical masked language modeling (MLM) losses**.
This means: if you can train BERT, you can train a diffusion LM. Same architecture, same loss, different sampling.

### Forward Process (Corruption)
```
q(z_t | x) = Cat(z_t; α_t * x + (1 - α_t) * m)
```
- `x` = clean token (one-hot)
- `m` = mask token (one-hot for [MASK])
- `α_t` = noise schedule, decreasing from 1→0 as t goes 0→1
- At t=0: mostly clean. At t=1: fully masked.

### Reverse Process (Generation)
```
p_θ(z_s | z_t) = {
    Cat(z_s; z_t)                                    if z_t ≠ [MASK]  (carry over)
    Cat(z_s; ((1-α_s)*m + (α_s-α_t)*x_θ) / (1-α_t))  if z_t = [MASK]  (predict)
}
```
Two critical properties:
1. **Zero masking probability**: Model never predicts [MASK] as output (set logit to -∞)
2. **Carry-over unmasking**: Already-unmasked tokens are copied, not re-predicted

### Loss Function
```
L = E_q ∫₀¹ (α'_t / (1 - α_t)) Σ_ℓ log ⟨x_θ^ℓ(z_t), x^ℓ⟩ dt
```
This is just **weighted cross-entropy on masked positions** — identical to BERT's MLM loss
with a time-dependent reweighting factor `α'_t / (1 - α_t)`.

### Training Recipe (Critical for Performance)
1. **Tokenizer matters**: Use 32K+ vocabulary (not 8K). Longer sequences = harder dependencies.
2. **Modern engineering**: Flash attention, bf16, proper learning rate schedules
3. **Noise schedule**: Log-linear `α_t = 1 - t` works well (simplest possible)

### Sampling Algorithm
```python
def sample(model, length, T=64):
    x = [MASK] * length
    for i in range(T):
        t = 1.0 - i / T
        s = 1.0 - (i + 1) / T
        logits = model(x, t)
        for pos in range(length):
            if x[pos] == MASK:
                probs = softmax(logits[pos])
                probs[MASK_ID] = 0  # never predict mask
                token = sample(probs)
                # Unmask with probability (α_s - α_t) / (1 - α_t)
                p_unmask = (alpha(s) - alpha(t)) / (1 - alpha(t))
                if random() < p_unmask:
                    x[pos] = token
    return x
```

### Results
- SOTA among diffusion models on LM1B and OpenWebText
- Approaches AR perplexity (not quite matching, but close)
- 6-8× better generative perplexity than GPT-2 without temperature scaling

---

## Paper 2: SEDD — Score Entropy Discrete Diffusion
**arXiv: 2310.16834 | ICML 2024 Best Paper | Theoretical foundation**

### Key Insight
Instead of modeling `p(x)` directly, model **ratios** between distributions:
```
s_θ(x)_y ≈ p_data(y) / p_data(x)
```
This eliminates intractable normalization constants. "Concrete scores" for discrete spaces.

### Score Entropy Loss
Extends score matching (continuous) to discrete spaces. The loss naturally:
- Compares predicted ratios against true data distribution ratios
- Is consistent (converges to true scores given enough data)
- 25-75% perplexity improvement over previous discrete diffusion

### Two Noise Types
1. **SEDD-Absorb**: Forward process masks tokens (like MDLM) — best for text
2. **SEDD-Uniform**: Forward process randomly replaces tokens — more general

### Results
- Outperforms GPT-2 at comparable model sizes
- 32× fewer network evaluations for similar quality
- 6-8× better generative perplexity than un-annealed GPT-2
- Enables controllable infilling naturally (not just left-to-right)

### Relevance to SpatialVortex
SEDD's ratio-based approach maps directly to your **sacred geometry energy signals**.
The "concrete score" `s(x)_y = p(y)/p(x)` is conceptually similar to your
`energy` and `alignment` signals — both measure relative confidence between states.

---

## Paper 3: DiffuLLaMA — Convert AR → Diffusion (Zero-Architecture-Change)
**arXiv: 2410.17891 | ICLR 2025 | Apple Research | THE KEY PAPER FOR YOU**

### Core Contribution
**You can convert ANY existing autoregressive model into a diffusion model** with:
1. Change causal attention mask → bidirectional attention
2. Continual pre-training with masking objective
3. That's it. Same architecture. Same tokenizer. Same embedding matrix.

### Three Adaptation Tricks

#### 1. Attention Mask Annealing
- Don't immediately switch causal → bidirectional
- Gradually anneal: start with causal, progressively reveal right-side context
- 10K steps of annealing for GPT-2 scale
- For DiffuLLaMA 7B: they went straight to bidirectional (it works!)

#### 2. Shift Operation
- AR models predict token n+1 from position n (shifted)
- Keep this shift during diffusion training
- At sampling time, shift back and prepend start token

#### 3. Time-Embedding-Free Architecture
- No need for timestep embeddings!
- Model implicitly learns the noise level from the number of [MASK] tokens
- This means: **zero new parameters needed**

### Tokenizer Trick
- Don't add a new [MASK] token — reuse a rare existing token
- DiffuGPT: tokenid=50257 (unused)
- DiffuLLaMA: tokenid=811

### Training Budget
| Model | Base | Training Tokens | GPUs |
|-------|------|-----------------|------|
| DiffuGPT-S | GPT2-small (127M) | 30B effective | 8× A100 |
| DiffuGPT-M | GPT2-medium (355M) | 30B effective | 8× A100 |
| DiffuLLaMA | LLaMA2-7B | 65B tokens | 16× 4xGH200 |

### Sampling
```
Initialize: x_T = [MASK, MASK, ..., MASK]
For t = T down to 0:
    logits = model(x_t)
    shift logits back (undo AR shift)
    For each masked position:
        sample token from softmax(logits)
        unmask with probability (α_s - α_t) / (1 - α_t)
        only unmask HIGH CONFIDENCE predictions first
Return x_0
```

### Key Results
- DiffuGPT outperforms all previous DLMs
- DiffuLLaMA 7B: competitive with LLaMA2 on most benchmarks
- **Naturally supports fill-in-the-middle** (infilling) — AR models can't do this
- Better at math reasoning than expected (bidirectional context helps)

---

## Paper 4: LLaDA — Large Language Diffusion with mAsking
**arXiv: 2502.09992 | Feb 2025 | Trained from scratch at 8B scale**

### Key Insight
> "The intelligence of LLMs stems not from the autoregressive mechanism per se,
> but from the core principle of generative modeling: approximating the true
> language distribution through maximum likelihood estimation."

### Method
- **Pretraining**: Mask tokens at ratio t ~ U[0,1], predict all masked tokens
- **SFT**: Only mask response tokens (prompt stays visible)
- **Sampling**: Full masking → iterative unmasking with remasking
- **Architecture**: Vanilla transformer. No modifications. No time embeddings.

### Reversal Curse Solution
AR models trained on "A is B" can't generate "B is A" (the reversal curse).
LLaDA naturally handles reversals because it sees bidirectional context.
**Beats GPT-4o on reversal poem completion.**

### Scalability
- 8B parameter model trained from scratch
- Competitive with LLaMA3 8B on in-context learning
- Shows clear scaling laws (bigger = better, just like AR)

---

## Mapping to SpatialVortex Architecture

### Current Problem
```
ThinkingEngine.think() → latent states → beam_to_word() → FAILS
Because: hash-based BeamTensor embeddings are not invertible
```

### Diffusion Solution
```
DiffusionEngine.generate() → [MASK] sequence → iterative unmasking → tokens
Because: predicts token IDs directly from vocabulary, no decoding needed
```

### Architecture Mapping

| SpatialVortex Component | Diffusion Equivalent | Status |
|------------------------|---------------------|--------|
| Vortex cycles (1→2→4→8→7→5→1) | Diffusion timesteps (T→0) | Natural fit! |
| Sacred positions (3,6,9) | Verification checkpoints during unmasking | Natural fit! |
| CALM LatentState | Noised token sequence z_t | Replace |
| BeamTensor hash embeddings | Real vocabulary embeddings (borrowed) | Replace |
| beam_to_word() | Not needed — predict token IDs directly | Delete |
| ThinkingEngine.think() | Iterative denoise loop | Adapt |
| energy / alignment signals | Unmasking confidence threshold | Map directly |
| SacredMoE experts | Could route different experts per denoising step | Future |

### The "No Training" Path — What's Actually Possible

**Honest assessment**: You CANNOT get a useful text generator with literally zero training.
But here's what you CAN do with minimal effort:

#### Level 0: Zero Training (Inference Only)
- Download pre-trained embedding matrix from any open model (e.g., GPT-2's `wte` weight)
- Use your existing transformer as a very rough denoiser
- Result: **Garbage text** — the transformer hasn't learned the masking objective

#### Level 1: Borrow Complete Model (~10 minutes setup)
- Download DiffuGPT-S (127M params) weights from HuggingFace
- Load into Rust via safetensors/GGUF format
- Use their weights for the full diffusion sampling loop
- Result: **Fluent text generation** with borrowed intelligence
- This is the DiffuLLaMA approach: borrow weights, run inference in Rust

#### Level 2: Lightweight Adaptation (~1-2 hours on 1 GPU)
- Take GPT-2 small weights
- Fine-tune with masking objective on a small corpus
- Only need ~1B tokens (a few hours on a single A100/4090)
- Result: **Your own diffusion LM** with decent quality

#### Level 3: Full Integration (Days)
- Wire diffusion sampling into ThinkingEngine
- Map vortex cycles to denoising steps
- Use sacred positions as verification checkpoints
- Integrate with existing RAG/knowledge systems
- Result: **SpatialVortex with real text generation**

### Recommended Rust Implementation Strategy

#### Phase 1: Embedding + Vocabulary Layer
```rust
// Load pre-trained embeddings (safetensors format)
struct VocabEmbeddings {
    embeddings: Array2<f32>,  // [vocab_size × embed_dim]
    vocab: Vec<String>,        // token_id → string
    mask_token_id: u32,        // special [MASK] token
}
```

#### Phase 2: Diffusion Sampler (Pure Algorithm, No Weights)
```rust
struct MaskedDiffusionSampler {
    num_steps: usize,     // T, typically 32-256
    alpha_schedule: Vec<f32>, // noise schedule
}

impl MaskedDiffusionSampler {
    fn sample(&self, model: &dyn Denoiser, prompt: &[u32], gen_len: usize) -> Vec<u32> {
        let mut seq = prompt.to_vec();
        seq.extend(vec![MASK_ID; gen_len]);
        
        for step in (0..self.num_steps).rev() {
            let t = step as f32 / self.num_steps as f32;
            let s = (step - 1).max(0) as f32 / self.num_steps as f32;
            let alpha_t = self.alpha_schedule[step];
            let alpha_s = if step > 0 { self.alpha_schedule[step - 1] } else { 1.0 };
            
            let logits = model.forward(&seq, t);
            
            for i in prompt.len()..seq.len() {
                if seq[i] == MASK_ID {
                    let mut probs = softmax(&logits[i]);
                    probs[MASK_ID as usize] = 0.0; // never predict mask
                    normalize(&mut probs);
                    
                    let (token, conf) = sample_with_confidence(&probs);
                    let p_unmask = (alpha_s - alpha_t) / (1.0 - alpha_t);
                    
                    if rand() < p_unmask {
                        seq[i] = token;
                    }
                }
            }
        }
        seq
    }
}
```

#### Phase 3: Wire Into ThinkingEngine
Replace `generate_response()` → `beam_to_word()` path with diffusion sampling.
The `Denoiser` trait can be implemented by:
- Borrowed model weights (Level 1)
- Your existing CALM transformer adapted (Level 2+)

### Noise Schedule Options
```
Log-linear (simplest):  α(t) = 1 - t
Cosine:                 α(t) = cos(πt/2)²  
Geometric (SEDD):       α(t) = exp(-σ(t))  where σ is geometric
```
Log-linear works fine for a first implementation (MDLM uses it).

---

## Key Takeaways

1. **The algorithm is simple**: It's iterative BERT-style unmasking with a schedule.
2. **Architecture is vanilla transformer**: No special layers, no time embeddings needed.
3. **You can borrow weights**: DiffuGPT/DiffuLLaMA weights are open-source.
4. **Tokenizer trick**: Reuse a rare token as [MASK], no vocab expansion needed.
5. **Bidirectional context**: Diffusion naturally uses full context (not just left).
6. **Confidence-based unmasking**: Maps directly to your energy/alignment signals.
7. **Vortex cycles = denoising steps**: The iteration already exists in your architecture.
8. **Sacred positions = verification**: 3,6,9 can checkpoint the denoising quality.

## References

- **MDLM**: Sahoo et al. "Simple and Effective Masked Diffusion Language Models" NeurIPS 2024. arXiv:2406.07524
- **SEDD**: Lou et al. "Discrete Diffusion Modeling by Estimating the Ratios of the Data Distribution" ICML 2024 Best Paper. arXiv:2310.16834
- **DiffuLLaMA**: Ye et al. "Scaling Diffusion Language Models via Adaptation from Autoregressive Models" ICLR 2025. arXiv:2410.17891
- **LLaDA**: Nie et al. "Large Language Diffusion Models" Feb 2025. arXiv:2502.09992
- **MDLM Code**: https://github.com/kuleshov-group/mdlm
- **SEDD Code**: https://github.com/louaaron/Score-Entropy-Discrete-Diffusion
- **DiffuLLaMA Code**: https://github.com/HKUNLP/DiffuLLaMA
- **LLaDA Code**: https://github.com/ML-GSAI/LLaDA
- **Gemini Diffusion**: Google I/O 2025, 1479 tokens/sec, 5× faster than comparable AR models
