# AIModel

Distilled SpatialVortex AGI/ASI seed - Sacred geometry + continuous latent generation.

## Overview

This crate contains the distilled, minimal, non-redundant stack from SpatialVortex (Jan 2026). Only the best solver per piece survives.

## Core Components

### Sacred Geometry (Preserved)
- **FluxMatrixEngine** - Vortex cycles (1→2→4→8→7→5→1), 3-6-9 anchors, 833:1 compression
- **VCP (VortexContextPreserver)** - Subspace hallucination detection + sacred interventions
- **GeometricInferenceEngine** - Bi-directional rule-based + ML enhancement (<500μs)
- **EBRM (EnergyBasedReasoningModel)** - Global energy for path refinement/scoring

### ML Stack
- **ProductionEngine** - High-throughput autoregressive with CALM integration
- **VortexModel** - Unified transformer with GQA/RoPE + VCP integration
- **Burn** - ML training framework with tch-rs backend

### AI Orchestration
- **AIConsensusEngine** - Multi-LLM fusion with weighted confidence
- **ASIOrchestrator** - Unified intelligence coordinator

## Dependencies (2026 Distilled)

| Crate | Purpose |
|-------|---------|
| `ort` | ONNX Runtime inference (primary) |
| `burn` | ML training framework |
| `wtransport` | WebTransport/QUIC networking |
| `rocksdb` | Hot-path storage |
| `embedvec` | Vector embeddings |
| `bevy` | 3D visualization |

## Features

```toml
[features]
default = ["onnx", "burn-cpu"]
onnx = ["ort", "tokenizers"]
burn-cpu = ["burn", "burn-ndarray", "burn-autodiff"]
burn-gpu = ["burn", "burn-tch", "burn-autodiff"]
bevy_viz = ["bevy"]
transport = ["wtransport"]
storage = ["rocksdb"]
embeddings = ["embedvec"]
```

## Status

**Work in Progress** - Files copied from SpatialVortex require import path fixes.

See `docs/IMPLEMENTATION_CHECKLIST.md` for:
- ✅ Migrated components
- 🔲 Components to implement (SSO, CALM, VortexDiscovery)
- 🔧 Import fixes needed

## License

MIT
