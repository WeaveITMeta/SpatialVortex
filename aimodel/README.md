# Vortex

[![Crates.io](https://img.shields.io/crates/v/vortex.svg)](https://crates.io/crates/vortex)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**A pure-Rust reasoning engine with zero pretrained weights, zero GPU, and an
honest benchmark harness — served as an OpenAI-compatible API with API keys
and per-token billing.**

---

## What Is Vortex?

Vortex is an experimental AI built entirely in Rust. It scores multiple-choice
reasoning problems on a single CPU thread with no neural network training and
no pretrained weights, using an ensemble of specialized reasoning experts
(symbolic arithmetic chains, retrieval, entity tracking, misconception
checking) coordinated by a geometric cycle framework.

It is a research artifact about how far *architecture without scale* can go —
and an honest measurement of where it can't.

### Benchmark Results (June 2026, integrity-hardened harness)

All tasks are evaluated as **4-choice multiple-choice (MC4) variants** with
shuffled answer positions and dataset-derived distractors. 4-choice chance is
25%. These numbers are NOT comparable to free-form leaderboard scores (GPT-4
GSM8K is free-form generation; ours is selection among 4 candidates).

| Task (MC4 variant) | Vortex | 4-choice chance | Free-form reference* |
|--------------------|:------:|:---------------:|:--------------------:|
| **HumanEval-MC4** (match solution to spec) | 55.3% | 25% | 67.0% (GPT-4, pass@1) |
| **TruthfulQA-MC4** (1 correct + 3 real incorrect) | 39.0% | 25% | 59.0% (GPT-4, MC1) |
| **GSM8K-MC4** (numeric answer + 3 derived distractors) | 35.6% | 25% | 92.0% (GPT-4, free-form) |
| **HellaSwag** (original 4-way) | 36.8% | 25% | 95.3% (GPT-4) |
| **MMLU** (original 4-way) | 26.7% | 25% | 86.4% (GPT-4) |
| **Overall** | **36.0%** | 25% | — |

\* Reference scores use different (harder) protocols and pretrained models —
context, not competition.

- **2,139 questions** evaluated in **2,079 s** (~1 s/question), 500/task cap
- **Zero GPU hours. Zero pretrained weights. Zero gradient updates.**
- Reproduce: see [Run Benchmarks](#run-benchmarks); answer positions are
  shuffled with a per-question seed, so runs are deterministic.

**Where the signal is real.** HumanEval-MC4 (identifier overlap between a
spec and candidate implementations) and GSM8K-MC4 (sequential arithmetic
chain solving over the question's own numbers) are scored by genuinely
discriminative experts and sit well above chance. TruthfulQA benefits from a
general misconception database.

**Where there is no signal yet.** MMLU and HellaSwag sit near chance. They
are dominated by world knowledge and fluency ranking — capabilities that, on
the evidence of this codebase, do not emerge from heuristic experts without
learned representations.

### Benchmark Integrity (June 2026 hardening)

Earlier published numbers (100% HumanEval, 93.7% GSM8K, 55.2% overall — May
2026) were inflated by harness defects, found and fixed in June 2026:

1. **Answer-position leakage**: synthetic-MCQ loaders (GSM8K, HumanEval,
   TruthfulQA, bAbI) always placed the correct answer at index 0 — and one
   GSM8K code path explicitly fell back to "choose index 0". All synthetic
   loaders now shuffle positions with a deterministic per-question seed, and
   the positional fallback was removed.
2. **Trivial distractors**: HumanEval's wrong answers were `return None` /
   `pass` stubs (and the scorer penalized exactly those strings). Distractors
   are now canonical solutions from *other* HumanEval problems — real code
   solving the wrong problem. TruthfulQA distractors now come from the
   dataset's own "Incorrect Answers" column instead of generic fillers. bAbI
   distractors are sampled from the task's actual answer vocabulary.
3. **Honest labeling**: results are reported as MC4 variants, never as
   free-form equivalents.

The harness prints free-form reference scores as context with an explicit
non-comparability note.

---

## Serving: OpenAI-Compatible API with Keys & Billing

Vortex serves as a drop-in OpenAI-style endpoint with API-key auth,
per-token usage metering, and configurable input/output pricing:

```bash
# Self-host (Docker, from repo root)
docker compose -f aimodel/docker-compose.yml up -d --build

# Or bare metal
VORTEX_HOST=0.0.0.0 cargo run --release --bin vortex-api --features web
```

- `POST /v1/chat/completions` — OpenAI-compatible (point any OpenAI SDK at it)
- `POST /admin/keys` / `DELETE /admin/keys/{key}` — issue/revoke developer keys
- `GET /v1/usage` — tokens + accumulated cost per key
- Pricing defaults: **$0.10/M input, $0.40/M output** (CPU-only inference —
  set to cover your hosting cost via `VORTEX_PRICE_*_PER_M`)
- `VORTEX_LIGHTWEIGHT=true` boots in seconds (skips dataset downloads)

Full guide: [docs/API.md](docs/API.md)

---

## How It Works

Vortex orchestrates **specialized reasoning experts** through a geometric
coordination framework.

### The Active Inference Pipeline

```
                         Question
                            |
                    [Dynamic RSI Routing]
                     Per-dataset strategy
                            |
              +-------------+-------------+
              |             |             |
         [Pipeline]    [Unified]    [Multi-Expert]
         Knowledge     3-pass MoE   12 specialists
         retrieval     reasoning    scoring
              |             |             |
              +------+------+------+------+
                     |             |
              [JEPA Pathway]  [Diffusion LM]
              LTR re-ranking  Token generation
              RL evidence     Sacred verification
                     |             |
                     +------+------+
                            |
                    [Vortex Cycle Refinement]
                    1 -> 2 -> 4 -> 8 -> 7 -> 5 -> 1
                    Sacred observers: 3, 6, 9
                            |
                    [Temperature-Scaled Softmax]
                            |
                         Answer
```

### Key Experts

| Expert | What It Does | Where It Helps |
|--------|-------------|----------------|
| **Symbolic Math** | Sequential narrative-chain solver: beams over in-order op chains consuming the question's numbers (plus text-gated constants like "twice"→2, week→7), exact-matches candidates against choices. | GSM8K |
| **Code Match** | Identifier overlap between the prompt's `def` signature/docstring and each candidate implementation; penalizes identifiers the prompt never mentions. | HumanEval |
| **Truth Checker** | General-knowledge misconception database via constitutional principles. | TruthfulQA |
| **Entity-Attribute** | Tracks entities and properties across context. | bAbI |
| **RAG Retrieval** | 3-stage retrieval with MMR rerank. | MMLU |
| **Semantic / Attention / CALM** | Hash-based embeddings, cross-attention, generative log-likelihood (no pretrained vectors). | General |

Plus: transitive spatial reasoning, web-knowledge patterns, comprehensive
multi-hop reasoning, HellaSwag fluency heuristics, and a discrete diffusion
LM (feature-gated) for generative tasks.

### Sacred Geometry: The Coordination Framework

The vortex cycle `1 -> 2 -> 4 -> 8 -> 7 -> 5 -> 1` is the scheduling clock of
the system; positions 3, 6, 9 act as verification checkpoints that never
mutate the data stream, and the golden ratio is used for threshold weighting.
These constants are an organizing aesthetic for the pipeline's control flow —
the measurable work happens in the experts above.

### Dynamic Recursive Self-Improvement (RSI)

Vortex adapts its inference strategy per dataset at runtime without gradient
updates: each dataset gets a tuned `InferenceStrategy` controlling pipeline
thresholds, multi-expert toggles, and pass counts, adjusted every 5 questions
from observed accuracy.

---

## Quick Start

### Run Benchmarks

```bash
cd aimodel

# Full honest suite (500/task, ~30 min single CPU core)
cargo run --bin spatialvortex-eval --release --features diffusion-expert -- \
  --tasks mmlu,gsm8k,hellaswag,truthfulqa,humaneval --eval-only --skip-hf --audit

# Quick smoke (50/task — note: small limits skew easy, not representative)
cargo run --bin spatialvortex-eval --release --features diffusion-expert -- \
  --tasks mmlu,gsm8k,hellaswag,truthfulqa,humaneval --limit 50 --eval-only --skip-hf

# Debug a single task with per-expert breakdowns
cargo run --bin spatialvortex-eval --release --features diffusion-expert -- \
  --tasks gsm8k --limit 10 --eval-only --debug-reasoning --skip-hf
```

### Interactive Chat

```bash
cargo run --bin vortex-cli                       # REPL (lightweight)
cargo run --bin vortex-cli -- --prompt "What is 48 / 2 + 12?"
cargo run --bin vortex-cli -- --prompt "Hello" --reasoning
```

### Library Usage

```toml
[dependencies]
vortex = "0.1"
```

```rust
use vortex::{VortexEngine, VortexEngineConfig};

let mut engine = VortexEngine::new();
let response = engine.chat("What is sacred geometry?");
println!("{} (confidence: {:.0}%)", response.content, response.confidence * 100.0);
```

---

## Performance Characteristics

| Metric | Value |
|--------|-------|
| **Average per question** | ~1 second (single CPU core) |
| **GPU memory required** | 0 bytes |
| **Pretrained weights loaded** | 0 bytes |
| **Training time** | 0 seconds |
| **Binary size** | ~15 MB (release, stripped) |
| **API boot (lightweight)** | seconds; full-knowledge mode downloads datasets at startup |

---

## Feature Flags

| Feature | Default | Description |
|---------|:-------:|-------------|
| `burn-cpu` | Yes | Burn ML framework with CPU backend |
| `web-learning` | Yes | Web crawler for knowledge acquisition |
| `diffusion-expert` | No | Diffusion LM with pathway-guided unmasking |
| `web` | No | Actix-web REST API server (vortex-api) |
| `burn-gpu` / `gpu` | No | GPU backends (PyTorch / WebGPU) |
| `onnx` | No | ONNX Runtime inference |
| `embeddings` | No | EmbedVec vector persistence (HNSW) |
| `storage` | No | RocksDB hot-path storage |

## Binaries

| Binary | Features | Description |
|--------|----------|-------------|
| `vortex-api` | `web` | OpenAI-compatible API server with keys + billing |
| `vortex-cli` | — | Interactive CLI chat |
| `spatialvortex-eval` | — | Benchmark evaluation harness with integrity-hardened loaders |
| `benchmark_crawler` | — | Web knowledge acquisition tool |

---

## Honest Positioning

Traditional AI gets capability from learned parameters. Vortex tests how much
structure alone buys: genuinely useful on tasks with exploitable formal
structure (arithmetic chains, identifier matching, misconception lookup), and
honestly near chance where world knowledge is required. It is a research
vehicle and a cheap, auditable serving stack — not a GPT-4 replacement.

## License

MIT
