# Flux Matrix: Voice-to-Space Summary

Captured: Last 15 Minutes of Real Thought

## Table of Contents

1. **Core Vision: My Voice, My Data, My 3D Space**
2. **Confidence Lake (#confidence-lake)**
3. **Ray Sphere — 3-6-9 as Geometric Law, Not Chaos**
4. **Tensor (#tensor)**
5. **Rust Pipeline (#rust-pipeline)**
6. **Import Flow: Svelte → Axum → Bevy**
7. **Noise ≠ Chaos — Frequency + Amplitude = Curvature**
8. **Federated (#federated)**
9. **Workflow as Matrix — Apps = Nodes, Flow = Rays**
10. **Next: Transpile Whisper → Rust with Py2many**

---

## 1. Core Vision: My Voice, My Data, My 3D Space

- **On-device first**: capture, infer, and visualize locally. Diamond moments are encrypted, zero egress by default.
- **Live semantics**: voice/text becomes 3D motion. Numbers are the skeleton; meaning bends the path.
- **Personal model**: small, fast, fine-tuned on your voice/logs; not a generic internet blob.

---

## 2. Confidence Lake (#confidence-lake)

- **What**: encrypted, append-only memory of high-confidence tensors (diamond moments), keyed by UUID.
- **When**: save if `ethos ≥ 8.5` AND `logos ≥ 7` AND down-tone (falling pitch).
- **Payload**: full ELP (`ethos[9], logos[9], pathos[9]`), pitch curve, text, metadata.
- **Storage**: `confidence_lake.dat` (AES-GCM-SIV encrypted), `confidence_lake.idx` (mmapped index), device-bound master key.
- **API**: `open/append/get/latest_n`. Replay beacons the exact state back to 3D (triple tori + curve).

---

## 3. Ray Sphere — 3-6-9 as Geometric Law, Not Chaos

- **Engine**: `src/flux_matrix.rs`
  - Positions `0–9` manifest their own values; sacred anchors at `3,6,9` with special geometry.
  - Digital root reduction and 9-step doubling sequence provide the straight skeleton.
- **Bending**: channel masses (Ethos/Logos/Pathos) curve routing away from straight edges, toward sacred anchors when meaning is heavy.
- **Visual**: the scene is a ray sphere; rays follow the skeleton unless bent by meaning (pathos wobble, logos straightness, ethos stability).

---

## 4. Tensor (#tensor)

- **BeadTensor (runtime, 13 floats / 52 bytes)**
  - `digits[9]`: fused per-position distribution (1–9)
  - `ethos, logos, pathos` (0–9 scalars)
  - `curviness_signed` (amplitude × sign(pitch slope))
- **Diamond (rich memory)**
  - `ethos[9], logos[9], pathos[9]`, pitch curve, text, model version; encrypted at rest.
- **3D mapping**
  - Color = Pathos (hue/saturation); Brightness = average(E,L,P) with down-tone boost; Wobble = |curviness|; Curve bend = neighbor scoring using channels.

---

## 5. Rust Pipeline (#rust-pipeline)

- **Tokio tasks**
  - Mic capture (cpal) → ring buffer
  - STT (whisper-rs) → text
  - ONNX inference (`ethos, logos, pathos`) → pooling
  - DSP (rustfft) → pitch curve & slope
  - Fusion → BeadTensor
  - Threshold → ConfidenceLake append
  - Routing → FluxMatrixEngine next node (curvature scoring)
- **Bevy systems**
  - Triple tori (Ethos/Logos/Pathos), inter-layer threads, wobble and bend per BeadTensor.

---

## 6. Import Flow: Svelte → Axum → Bevy

- See: `docs/import/IMPORT_PIPELINE.md`
- **SvelteKit** uploads (`multipart/form-data`) → **Axum** `/api/import`→ parse & map → Node Bus → **Bevy** spawns 3D nodes + labels.
- SSE for progress. RBAC/SSO gating (OIDC + JWT) controls format access, size limits, and redaction.

---

## 7. Noise ≠ Chaos — Frequency + Amplitude = Curvature

- **Pitch**: extract via FFT; compute slope over a sliding window.
- **Curvature amplitude**: function of Pathos mass and digit entropy.
- **CurvinessSigned**: `curviness = amp * sign(pitch_slope)`; bends the curve and drives wobble.

---

## 8. Federated (#federated)

- **Local-first**: fine-tune on-device with private logs.
- **Sharing (opt-in)**: encrypted envelopes of gradients or tri-channel seeds. FedAvg / secure aggregation if collaborating.
- **Policy**: Never send raw audio/text; only share if explicitly initiated by the user.

---

## 9. Workflow as Matrix — Apps = Nodes, Flow = Rays

- **Nodes**: OS apps/services as positions in a context graph.
- **Rays**: events/intent routes between nodes (speech → text → inference → visualization).
- **RBAC scope**: import/visualization/actions constrained by user roles and tenant claims.

---

## 10. Next: Transpile Whisper → Rust with Py2many

- **Practical**: `whisper-rs` already provides fast, local STT. Prefer it as the backbone.
- **If migrating Python models**
  - Export to ONNX and load with ONNX Runtime in Rust (`ort`).
  - Py2many can prototype simple dataflow, but ML models translate better via ONNX than code transpilation.
- **Acceptance**: latency < 20 ms/utterance on CPU, stable ELP scores, diamond replay fidelity 1:1.

---

## Appendix: Curvature Scoring (sketch)

```
score[i→j] = W[i,j]
             + αe * mass(E) * E[j]
             + αl * mass(L) * L[j]
             + αp * mass(P) * P[j]
             + β  * sacred_bias(j)
```

- `mass(C) = 1 - entropy(softmax(C)) / log 9` (decisiveness)
- Next node = `argmax_j score`; curvature = `1 - cos(baseline_dir, chosen_dir)`
