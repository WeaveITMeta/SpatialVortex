# vortex

[![Crates.io](https://img.shields.io/crates/v/vortex.svg)](https://crates.io/crates/vortex)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Distilled SpatialVortex AGI/ASI seed — sacred geometry inference, continuous latent generation, and recursive self-improvement.

## Overview

`vortex` is a pure-Rust AI inference and training framework built on sacred geometry principles. It provides a complete pipeline from knowledge acquisition (HuggingFace datasets + web crawling) through multi-expert inference with runtime self-improvement.

**No external LLM required** — all inference is performed locally using learned embeddings, geometric world models, and multi-expert scoring.

## Quick Start

### CLI — Interactive Chat

```bash
# Interactive REPL
cargo run --bin vortex-cli

# Single prompt (plain text or JSON output)
cargo run --bin vortex-cli -- --prompt "What is sacred geometry?" --json

# With reasoning trace
cargo run --bin vortex-cli -- --prompt "Hello" --reasoning

# Piped input
echo "Hello" | cargo run --bin vortex-cli
```

Flags: `--prompt`, `--system`, `--temperature`, `--max-steps`, `--max-cycles`, `--json`, `--reasoning`, `--no-guard`, `--no-memory`

### REST API — OpenAI-Compatible Server

```bash
cargo run --bin vortex-api --features web
```

Endpoints:
- `POST /v1/chat/completions` — Chat completions (OpenAI-compatible)
- `GET  /v1/models` — List available models
- `GET  /health` — Health check

```bash
curl -X POST http://127.0.0.1:7000/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"vortex-0.1","messages":[{"role":"user","content":"Hello"}]}'
```

Responses include a `vortex` extension with confidence, energy, sacred alignment, full reasoning trace, and safety results.

Configure with env vars: `VORTEX_HOST` (default `127.0.0.1`), `VORTEX_PORT` (default `7000`).

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

### Run Benchmarks

```bash
cargo run --release --bin spatialvortex-eval --features "gpu,embeddings,web-learning" -- --tasks all
```

## Architecture

### Inference Pipeline

```
Question → Pipeline → Unified Inference → Multi-Expert → Answer
              ↓              ↓                  ↓
         Knowledge      3-pass MoE         21 experts
         Retrieval      + World Model      + CALM semantic
              ↓              ↓                  ↓
         Dynamic RSI learns which path works best per dataset
```

### Core Components

| Component | Description |
|-----------|-------------|
| `CALMEngine` | Continuous Autoregressive Language Model — semantic encoder/decoder |
| `UnifiedInferenceEngine` | 3-pass iterative refinement with MoE routing and reasoning layers |
| `UnifiedKnowledgePipeline` | RETRIEVE → EXTRACT → EMBED → REASON → SCORE |
| `DynamicRSI` | Runtime self-improving inference strategy per dataset |
| `SacredMoEModel` | Mixture of Experts with geometric (phi-based) routing |
| `TransitiveFluxReasoner` | Transitive reasoning via vortex flux matrix ladder index |
| `GenerativeVortexEngine` | Generative architecture with BPE tokenizer + CALM |
| `FluxMatrixEngine` | Vortex cycles (1→2→4→8→7→5→1), 3-6-9 sacred anchors |
| `ConsciousnessLearner` | Dynamic knowledge graph with web learning |
| `RealBenchmarkEvaluator` | Full eval harness (MMLU, GSM8K, ARC, HellaSwag, TruthfulQA, HumanEval) |

### Knowledge Sources

1. **HuggingFace Datasets** — 125 datasets across 9 categories (commonsense, science, math, code, etc.)
2. **Web Learning** — High-throughput crawler extracts facts from Wikipedia and educational sites
3. **EmbedVec Persistence** — One-time download, then loads from local cache (114K+ embeddings)
4. **Test-Time Training** — Learns from each question during inference

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `burn-cpu` | Yes | Burn ML framework with CPU backend |
| `web-learning` | Yes | Web crawler for knowledge acquisition |
| `burn-gpu` | No | Burn with PyTorch GPU backend |
| `gpu` | No | Burn with WebGPU backend |
| `onnx` | No | ONNX Runtime inference |
| `embeddings` | No | EmbedVec vector persistence |
| `web` | No | Actix-web chat API server |
| `storage` | No | RocksDB hot-path storage |
| `transport` | No | WebTransport/QUIC networking |
| `bevy_viz` | No | Bevy 3D visualization |

## Binaries

| Binary | Features Required | Description |
|--------|-------------------|-------------|
| `vortex-cli` | — | Interactive CLI chat with single-shot, piped, and REPL modes |
| `vortex-api` | `web` | OpenAI-compatible REST API server (`/v1/chat/completions`) |
| `spatialvortex-eval` | — | Full benchmark evaluation harness |
| `benchmark_crawler` | — | Web knowledge acquisition tool |

## Project Structure

```
src/
  lib.rs          — Public API surface
  error.rs        — Error types
  data/           — Models, attributes, HF datasets, benchmark loaders
  core/           — Sacred geometry engines
  ml/             — ML components (CALM, MoE, inference, RSI, reasoning)
  ai/             — AI orchestration (consensus, flux reasoning)
  engine.rs       — Unified VortexEngine (CLI + API entry point)
  cognition/      — Thinking, memory, RAG, constitution, tools
  serving/        — Batch scheduler, MoE gate, MCP server, chat API
  storage/        — Embeddings, RocksDB, unified store
  transport/      — WebTransport/QUIC
  bin/            — CLI binaries
rsi_macros/       — Proc-macro crate for compile-time RSI code generation
```

## License

MIT
