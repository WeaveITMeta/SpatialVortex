# Next Steps

This document tracks the recommended next actions across the project. Update as we go.

## Viewer (Bevy)

- **[raycasting-choice]** Decide between Rapier (`bevy_rapier3d`) vs `bevy_mod_raycast` for real ray hits (current math-based markers work). 
- **[ray-markers]** Add rotation/pulse to markers and stack mini-hierarchies along +Z per index.
- **[semantic-pulls]** Add `SemanticTag` and a demo input: typing a term pulls similar-tagged markers; antonyms invert to negative indices.

## ELP Inference (ONNX Runtime)

- **[onnx-loader]** Implement ONNX `infer_triple(text) -> (E[9], L[9], P[9])` in future `viewer/` crate.
- **[pooling]** Mean/attention pooling over tokens → per-utterance triple.
- **[fusion]** Compute BeadTensor (13 floats) from E/L/P + pitch/coherence.

## Audio (whisper-rs + DSP)

- **[stt]** Integrate `whisper-rs` (feature `stt`) for mic → text.
- **[pitch]** Extract pitch via `rustfft`; compute slope; feed `curviness_signed`.

## Curvature Routing

- **[scoring]** Implement neighbor scoring overlay in `FluxMatrixEngine` using channel masses and sacred bias.
- **[visuals]** Use curvature magnitude to drive wobble; color edges by channel dominance.

## Confidence Lake (Encrypted)

- **[design]** Append-only encrypted format with DPAPI-protected master key.
- **[api]** `open/append/get/latest_n`; write on ethos ≥ 8.5 & logos ≥ 7 & down-tone.
- **[replay]** Load-by-UUID to beam exact tri-channel state back to 3D (“diamond mode”).

## Import Pipeline (Svelte → Axum → Bevy)

- **[server]** Implement `POST /api/import` (multipart), `GET /status`, `GET /events` SSE.
- **[parsers]** JSON/CSV/RBXML → unified node schema; redaction by RBAC entitlements.
- **[auth]** OIDC/JWT validation; roles: viewer, importer.basic, importer.rbxml, admin.

## Mnemonic k-NN Engine

- **[featurizer]** Implement char 3–5-gram + phonetic hashing → 128D vectors.
- **[index]** Use `hnsw_rs` for approximate k-NN; cosine distance.
- **[adapter]** Aggregate k neighbors into `digits[9]` (and ELP if stored); emit BeadTensor.
- **[tests]** Recall@k, latency, misspell robustness.

## Docs & Wiring

- **[linking]** Link `docs/VOICE_TO_SPACE_SUMMARY.md` and `docs/import/IMPORT_PIPELINE.md` from `README_PURPOSED.md`.
- **[bevy-version]** Decide whether to keep Bevy 0.7 or upgrade; align physics/raycast crates.

---

## Updated Plan (October 20, 2025)

- **Scope**: Incorporates current crate versions and best practices. High-impact marked with ⭐. Track in GitHub Issues.

### Viewer (Bevy)

- **[raycasting-choice]** Decide Rapier vs `bevy_mod_raycast`.
  - ⭐ Research: `bevy_rapier3d` (~0.27) for physics-integrated raycasts and collisions; `bevy_mod_raycast` (~0.14) for lightweight scene queries.
  - Next: Prototype both in a branch; if tensor impulses/forces are planned, prefer Rapier. Deadline: EOW.
  - Deps: Align with chosen Bevy version (see Bevy decision below).

- **[ray-markers]** Rotation/pulse + Z-stacks per index.
  - Sub-steps: Use `bevy_tweening` for pulse/rotate. Parent children; offset along Z by `index * 0.5`.
  - Next: Add `MarkerComponent { rotation_speed: f32 }` and update via system.
  - Visuals: Color by digital root; emphasize 3/6/9 in gold.

- **[semantic-pulls]** Tags and similarity-driven pulls.
  - Sub-steps: `SemanticTag(String)`; use tiny embeddings (`rust-bert`) or hashed n-grams. Thresholds: >0.7 pull positive; < -0.7 flip negative.
  - Next: bevy_ui input; on enter, filter and lerp markers; tie to anchor rays.

### ELP Inference (ONNX Runtime)

- **[onnx-loader]** Implement `infer_triple(text) -> (E[9], L[9], P[9])`.
  - ⭐ Next: Use `ort` crate (~0.2). Define `ElpTriple { ethos:[f32;9], logos:[f32;9], pathos:[f32;9] }`.
  - Sub-steps: Mock model if needed; `session.run()` → arrays. Test shapes and ranges.

- **[pooling]** Mean/attention pooling to utterance-level triple.
  - Next: Implement mean now; attention later. Use `ndarray`.
  - Deps: After `onnx-loader`.

- **[fusion]** Build BeadTensor (13 floats) from E/L/P + pitch/coherence.
  - Sub-steps: Define weights/normalization; map to 13D. Add pitch slope and a coherence scalar.

### Audio (whisper-rs + DSP)

- **[stt]** Mic → Whisper (feature `stt`).
  - Next: `cpal` for mic; process chunks with tiny.en; stream text to ELP.

- **[pitch]** Pitch & slope → `curviness_signed`.
  - ⭐ Next: Prefer YIN if available; else FFT peak. Slope = diff over window; sign from trend.
  - Integration: Parallel to STT; fused in BeadTensor.

### Curvature Routing

- **[scoring]** Neighbor scoring overlay in `FluxMatrixEngine`.
  - Next: Define 10×10 adjacency; `score = W[i,j] + αe·E[j] + αl·L[j] + αp·P[j] + β·sacred_bias(j)`.
  - Sub-steps: Expose scores to Bevy for debugging.

- **[visuals]** Wobble by curvature; edge colors by dominance.
  - Next: Draw edges with Gizmos/lines; thickness ∝ score; wobble via `sin(time * mag)`.

### Confidence Lake (Encrypted)

- **[design]** Append-only encrypted log + mmap index.
  - Next: AES-GCM (SIV if available); device-bound key via DPAPI; compact binary payload.

- **[api]** `open/append/get/latest_n`; threshold: `ethos ≥ 8.5`, `logos ≥ 7`, down-tone.
  - ⭐ Next: Implement as a crate/module; conditional writes post-fusion.
  - Deps: Fusion producing stable scalars.

- **[replay]** UUID load → exact tri-channel replay (“diamond mode”).
  - Next: Spawn materials/curves matching saved state; prismatic highlight.

### Import Pipeline (Svelte → Axum → Bevy)

- **[server]** `POST /api/import`, `GET /status`, `GET /events` (SSE).
  - Next: Axum v0.7; `axum-extra` multipart; SSE for progress.
  - Sub-steps: Job store; Node Bus to Bevy.

- **[parsers]** JSON/CSV/RBXML → unified node schema; RBAC redaction.
  - Next: `NodeSchema`; adapters; clamp colors; mask sensitive keys per role.

- **[auth]** OIDC/JWT; roles: viewer, importer.basic, importer.rbxml, admin.
  - ⭐ Next: `tower-http` middleware; `jsonwebtoken`/`openidconnect` with JWKS cache.

### Mnemonic k-NN Engine

- **[featurizer]** Char 3–5-gram + phonetic hashing → 128D.
  - Next: `ngrams`; `rust-phonetics` (Soundex/Metaphone); L2-normalize.

- **[index]** `hnsw_rs` approximate k-NN; cosine.
  - Next: Build/persist index; add/query APIs.

- **[adapter]** k-NN → `digits[9]` (and ELP if stored); emit BeadTensor.
  - Next: Weighted average by similarity; softmax stabilize.

- **[tests]** Recall@k, latency, misspell robustness.
  - Next: Unit tests + `criterion` benches; fuzz misspellings.

### Docs & Wiring

- **[linking]** Link `docs/VOICE_TO_SPACE_SUMMARY.md` and `docs/import/IMPORT_PIPELINE.md` from `README_PURPOSED.md`; add ToC.
- **[bevy-version]** Decide Bevy 0.7 vs upgrade.
  - ⭐ Research: Bevy v0.15 (Oct 2025) has better ECS perf and plugin ecosystem.
  - Next: Migrate in feature branch; align Rapier and others; run compile/runtime tests.
