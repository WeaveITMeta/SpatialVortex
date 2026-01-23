# FEAUTRE

A concise map of all implemented and planned features across this repository.

## Table of Contents

- **[Project Structure](#project-structure)**
- **[Core Crate: spatial-vortex](#core-crate-spatial-vortex)**
- **[Viewer/NNA Crate: windsurf](#viewernna-crate-windsurf)**
- **[Feature Flags](#feature-flags)**
- **[Run Guides](#run-guides)**
- **[Tests](#tests)**
- **[Planned Next](#planned-next)**

---

## Project Structure

- **Root crate**: `spatial-vortex` (library + optional Bevy viewer)
- **Secondary crate**: `viewer/` (experimentation: NN models, seeds, self-loop, viewer)

Key paths:
- **Core**: `src/flux_matrix.rs`, `src/models.rs`, `src/change_dot.rs`
- **Viewer (root)**: `src/bin/vortex_view.rs`
- **Viewer**: `viewer/src/main.rs`, `viewer/src/model.rs`, `viewer/src/transformer.rs`, `viewer/src/seeds.rs`

---

## Core Crate: spatial-vortex

- **Flux Matrix Engine** (`src/flux_matrix.rs`)
  - **Base pattern**: `1, 2, 4, 8, 7, 5, 1` describes the doubling cycle behavior
  - **Digit reduction**: `reduce_digits()` implements digital root (e.g., `13→4`, `15→6`)
  - **Coverage**: 10 positions `0–9` (positions manifest their own values; sacred `3,6,9` included)
  - **Nodes/Guides**: create regular nodes and sacred guides (`create_flux_node()`, `create_sacred_guide()`)
  - **Connections**: sequential, geometric, and sacred connection topologies
  - **Seed mapping**: `seed_to_flux_sequence()` generates a 9-step flux sequence via doubling + digit reduction (values in `0–9`)
  - **Validation & RL**: `validate_matrix()`, `update_matrix_with_rl()`

- **Data Models** (`src/models.rs`)
  - `FluxMatrix`, `FluxNode`, `SacredGuide`, `NodeConnection`
  - `SemanticIndex`, `LearningAdjustment`, `AssociationSource`

- **Change Dot Parser & Iterator** (`src/change_dot.rs`)
  - `ChangeDotIter` yields events:
    - **Step** `{ from, to, to_position, cycle_index }`
    - **SacredHit** `{ position: 3 }` cadence (every 3rd yield)
    - **Loop** `{ length: 6 }` on cycle closure back to `1`
  - `parse_change_dot("888.872") -> Vec<ChangeDotIter>`

- **Viewer (optional)** (`src/bin/vortex_view.rs`)
  - Spawns a 3D tetra-like mesh in Bevy
  - Consumes `ChangeDotEvent` to drive rotation/scale ("breathing")
  - Text input to swap change-dot iterator at runtime

- **Utilities**
  - Spin lerp: `src/angle.rs` → `compute_next(angle, digit, alpha)`

---

## Viewer/NNA Crate: viewer

- **Bevy viewer** (`viewer/src/main.rs`)
  - Spawns torus and animates from a live intent digit
  - **Self-loop**: 5s timer generates internal self-talk, updates state
  - **Short-term memory (STM)**: last 10 tokens and last 10 9-vectors (ring buffers)
  - Keyboard input updates intent; future: whisper/audio hook

- **Neural Nets (LibTorch via tch)**
  - **VortexNet** (`viewer/src/model.rs`): MLP 2×Linear + ReLU → 9 logits
    - Text and token-ID featurizers (`featurize_row`, `featurize_ids`)
  - **TransformerNet** (`viewer/src/transformer.rs`):
    - 1-layer MHA decoder with token+pos embeddings → pooled → 9 logits
    - **Sacred clamp** for digits `3,6,9` (freeze planned: strict 6+3 head)

- **Seeds & Persistence** (`viewer/src/seeds.rs`)
  - Save/load compact 9×`f32` seed vectors per subject (binary), `memmap2` for fast load
  - Planned: `.pt` tensor save/load to preserve device (GPU) residency

- **Mapping & Guards** (`viewer/src/mapping.rs`)
  - `normalize_text()` (e.g., "wendsurf" → "windsurf")
  - `map_text_to_digit()` hashing+scaling path (fast), `center_digit()` to [-4..+4]

- **Tokenizer & STT hooks** (`viewer/src/lib.rs`)
  - `#[cfg(feature = "nlp")] tokenize_to_seeds(text, &mut VortexCore)` via `tokenizers::Tokenizer` and local `tokenizer.json`
  - `#[cfg(feature = "stt")] stream_audio_to_seeds(audio_bytes, &mut VortexCore)` via `whisper-rs` (expects model `base.en`)

---

## Feature Flags

- **Root crate** `spatial-vortex` (`Cargo.toml`)
  - **`bevy_support`**: pulls Bevy 0.7 viewer (`src/bin/vortex_view.rs`)

- **`viewer/` crate** (`viewer/Cargo.toml`)
  - **`nlp`**: enable `tokenizers` and `tokenize_to_seeds()`
  - **`stt`**: enable `whisper-rs` and `stream_audio_to_seeds()`

---

## Run Guides

- **Root Bevy viewer** (eventful change-dot loop):
```bash
# from repo root
cargo run --features bevy_support --bin vortex_view
```

- **Viewer app** (NN-driven intent):
```bash
# from viewer/
cargo run

# with tokenizer demo (requires viewer/tokenizer.json)
cargo run --features nlp

# with tokenizer + STT stub (requires whisper model file "base.en")
cargo run --features "nlp stt"
```

Assets expected:
- `viewer/tokenizer.json` (BPE or similar)
- `viewer/base.en` (Whisper model; size varies)

---

## Tests

- **Root crate**:
  - `tests/flux_reducer_tests.rs`: validates `reduce_digits()` known cases
  - `tests/change_dot_events_tests.rs`: 24-step iterator survival, `Loop` and `SacredHit` cadence
  - `tests/angle_tests.rs`: rotation delta proportional to digit magnitude

- **Viewer**:
  - `viewer/tests/mapping_tests.rs`: numeric "128" → digit `8`, normalization, centered range bounds

Run:
```bash
# root crate tests
cargo test

# viewer crate tests
cargo test --manifest-path viewer/Cargo.toml
```

---

## Planned Next

- **Sacred freeze (strict)**: 6-trainable outputs + constants for 3/6/9
- **Tensor seeds (.pt)**: GPU-resident, encrypted-at-rest seeds; preload 20 common subjects
- **Cosine flash**: green flash when live vector matches a preloaded seed (cos > 0.8)
- **Whisper streaming**: mic → whisper → token IDs → transformer → viewer (no JSON)
- **Intent engine**: nearest-seed cosine + k-means voting for synonyms/misspells
- **Physics orbits**: node masses, forces from semantic weights; sacred anchors
- **Zero-bias loss**: center-out target mapping or consistency penalty
- **Federated skeleton**: local training, later parameter averaging hooks
- **Multimodal fusion**: CLIP image/text + audio embeddings, late fusion before 9-way
- **Constrained beam search**: rule-guided autocomplete for next-intent tokens
- **E2E harness (ignored)**: audio fixture → whisper → transformer → headless Bevy step/assert
