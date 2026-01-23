# AIModel Implementation Checklist

## Distilled SpatialVortex Stack (Jan 2026)

Sacred-geometry-centric AGI/ASI seed with absolute best solvers per slot.

---

## ✅ MIGRATED (from SpatialVortex)

### Core Sacred Geometry
- [x] `FluxMatrixEngine` - Vortex cycles (1→2→4→8→7→5→1), 3-6-9 anchors, 833:1 compression
- [x] `GeometricInferenceEngine` - Bi-directional rule-based + ML enhancement
- [x] `flux_matrix.rs` - Core flux matrix operations
- [x] `change_dot.rs` - Change detection
- [x] `angle.rs` - Angular calculations

### ML Core
- [x] `VCP (VortexContextPreserver)` - Subspace hallucination detection + sacred interventions
- [x] `EBRM (EnergyBasedReasoningModel)` - Global energy for path refinement/scoring
- [x] `VortexModel` - Transformer/GQA/RoPE with VCP integration
- [x] `ProductionEngine` - High-throughput autoregressive core
- [x] `autoregressive.rs` - Autoregressive decoding
- [x] `rope.rs` - Rotary Position Embeddings
- [x] `gqa.rs` - Grouped Query Attention
- [x] `optimized_ops.rs` - SIMD/BLAS operations
- [x] `tokenizer.rs` - Tokenization

### AI Orchestration
- [x] `AIConsensusEngine` - Multi-LLM fusion with weighted confidence
- [x] `ASIOrchestrator` - Unified intelligence coordinator
- [x] `FluxReasoning` - Sacred geometry reasoning chains

### Data Models
- [x] `BeamTensor`, `ELPTensor`, `FluxMatrix` - Core data structures

### Visualization
- [x] `bevy_3d.rs` - 3D rendering foundation

---

## 🔲 TO IMPLEMENT (New Crates/Modules)

### SpectralSphereOptimizer (SSO) - HIGH PRIORITY
- [ ] Port from arXiv 2601.08393 / Unakar/Spectral-Sphere-Optimizer
- [ ] Module-wise spectral-norm sphere constraints on weights + updates
- [ ] Steepest descent on spectral sphere → μP-aligned stability
- [ ] Implement as Burn optimizer trait
- [ ] Benefits: bounded activations, outlier suppression, rapid convergence
- [ ] Location: `src/ml/training/spectral_sphere_optimizer.rs`

### CALM (Continuous Autoregressive Language Models) - HIGH PRIORITY
- [ ] Build high-fidelity autoencoder in Burn (compress K semantic chunks → continuous latent)
- [ ] Autoregress in latent space (energy-based prediction)
- [ ] Decode back → K× fewer steps, smoother vortex orbits
- [ ] Integrate with ProductionEngine
- [ ] Add speculative decoding + batching
- [ ] Location: `src/ml/calm.rs`

### VortexDiscovery (Test-Time Adaptation) - MEDIUM PRIORITY
- [ ] Lightweight Burn LoRA-style adapter per hard query
- [ ] Self-generate candidates → score via EBRM + vortex consistency + sacred alignment
- [ ] Refine adapter weights iteratively (test-time gradient steps)
- [ ] Bevy visualizes orbit tightening in real time
- [ ] Trigger on high-entropy / novel flux paths
- [ ] Location: `src/ml/vortex_discovery.rs`

### embedvec Integration - MEDIUM PRIORITY
- [ ] Port FluxMatrixEngine to use embedvec for vector ops
- [ ] SacredEmbedding layer: HNSW indexing + SIMD distances
- [ ] Geometric priors for flux position lookups
- [ ] Location: `src/storage/embeddings.rs`

### WebTransport Server - MEDIUM PRIORITY
- [ ] Upgrade from WebSockets to wtransport
- [ ] Stream Bevy scene deltas + flux updates
- [ ] Real-time consensus/voice telemetry
- [ ] Location: `src/transport/mod.rs`

### RocksDB Persistence - LOW PRIORITY
- [ ] Hot-path storage for flux states / embeddings
- [ ] Replace Redis caching layer
- [ ] Location: `src/storage/rocksdb_store.rs`

---

## 📦 Dependencies Status

| Crate | Version | Status | Purpose |
|-------|---------|--------|---------|
| `ort` | 2.0 | ✅ Added | ONNX Runtime inference |
| `burn` | 0.16 | ✅ Added | ML training framework |
| `burn-tch` | 0.16 | ✅ Added | GPU backend (libtorch) |
| `wtransport` | 0.2 | ✅ Added | WebTransport/QUIC |
| `rocksdb` | 0.22 | ✅ Added | Embedded KV store |
| `embedvec` | 0.1 | ✅ Added | Vector embeddings |
| `bevy` | 0.15 | ✅ Added | 3D visualization |

---

## 🎯 Integration Order (Recommended)

1. **Setup module structure** - Create mod.rs files, wire up imports
2. **Port FluxMatrixEngine + VCP** to use embedvec for vector ops
3. **Create ProductionEngine crate** → implement CALM latent path in Burn
4. **Implement SpectralSphereOptimizer** as Burn optimizer
5. **Wire VortexDiscovery** → hook into ASIOrchestrator for hard queries
6. **Upgrade server to wtransport** → stream Bevy scene deltas
7. **RocksDB for persistence** of flux states / embeddings

---

## 🔧 IMMEDIATE TODO (Import Path Fixes)

The copied files still reference `crate::` paths from SpatialVortex. These need to be updated:

### Files Requiring Import Fixes
- `src/core/sacred_geometry/*.rs` - References to `crate::data::models`, `crate::error`, etc.
- `src/ml/*.rs` - References to `crate::data::models`, `crate::core::sacred_geometry`, etc.
- `src/ai/*.rs` - References to `crate::ml`, `crate::core`, `crate::data`, etc.
- `src/data/models.rs` - References to `crate::data::attributes` (not copied)

### Missing Dependencies (Not Copied)
- `src/data/attributes.rs` - Attributes system
- `src/data/elp_attributes.rs` - ELP attributes
- `src/data/compression/` - Compression utilities
- `src/processing/` - Processing modules
- `src/consciousness/` - Consciousness simulation
- Various utility modules

### Recommended Approach
1. Start fresh with minimal core types in `data/models.rs`
2. Implement FluxMatrix and sacred geometry from scratch with clean imports
3. Build up VCP and EBRM on top of clean foundation
4. Add ProductionEngine with CALM integration

---

## 📝 Notes

- Sacred geometry soul (3-6-9 vortex math, flux matrix, ELP judgment) is preserved
- All other pieces upgraded to current best solver
- No redundancies - single best option per slot
- Rust-native, portable (CPU/GPU/WASM)
