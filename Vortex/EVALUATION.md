# Evaluation of the existing Vortex AI system (2026-10-02)

This audit covers what existed before `Vortex/`: the root crate `spatial-vortex` (`src/`), the `aimodel/` crate (package name `vortex`), the Eustress-side crates in `eustress/crates/`, and the EustressEngine submodule. Four independent read-only audits ran in parallel; their findings are summarised below. The verdict on each part decides whether it transfers.

## Headline

About 190K lines of Rust exist across the two legacy crates. Most of it is heuristics under ambitious names. The genuinely valuable parts are:

- the August 2026 autodiff work in `aimodel` (Burn attention, the SSO optimiser, graph attention, the physics loss);
- the integrity-hardened eval loaders;
- the Eustress `vortex-core` WorldState loop;
- the ARC-AGI-3 policy, which is the most mature code in the tree.

There was no pretrained or trained LLM, no working diffusion model, and no real RSI.

| Area | Where | Verdict |
|---|---|---|
| Root crate `spatial-vortex` | `src/` (~91K LOC) | **Does not compile** (17 errors, 771 warnings, last built 2026-03-19). |
| Generators (image, video, asset, doc) | `src/generators/` | Pure stubs that return `"default"`. |
| Self-modification | `src/asi/self_modification.rs` | **Unsafe.** A hardcoded "95/100 passed" is followed by real writes to its own source. Replaced by 012 RSI. |
| Consciousness | `src/consciousness/` | Prompt orchestration plus a heuristic Φ. Not carried over. |
| Multi-provider consensus | `src/ai/consensus.rs` | 5 of 6 providers are mocked. |
| Constitution | `aimodel/src/cognition/constitution.rs` | Four substring checks, plus a 1,200-line misconception list that mirrors TruthfulQA items. Replaced by 003 and 004. |
| LLM | `aimodel` | No transformer stack. The first real `.backward()` landed 2026-08-08. `mc_train` reaches 28.1% held-out (chance is 25%). |
| Diffusion | `aimodel/src/ml/vortex_diffusion.rs` | Hand-rolled, no autodiff, heuristic training. |
| RSI | `aimodel/src/ml/dynamic_rsi.rs`, `rsi_macros` | Threshold tuning driven by eval accuracy; the macros pass code through unchanged. |
| MCP | `aimodel/src/serving/mcp_server.rs` | No transport loop and not JSON-RPC 2.0. Replaced by the 006 MCP client. |
| Serving | `aimodel/src/bin/vortex_api.rs` | **Real.** OpenAI-compatible API, API keys, per-token billing. Transfer to 016. |
| Benchmarks | `aimodel` eval harness | Honest after the June 2026 fix, about **36.9% MC4** (chance 25%), MMLU ~26%. Two leaks remain: online test-time training on gold labels, and HellaSwag gold endings in CALM pretraining. 012 and 014 make both structurally impossible. |
| Eustress `vortex-core` | `eustress/crates/vortex-core` | **Clean and real.** Reused directly by 010. Note: the universal loop contributes 0% of live ARC-AGI-3 decisions (memory, 2026-07-28). |
| ARC-AGI-3 policy | `eustress/crates/arc-policy` | Most mature code (13K LOC, 131 tests). Official best score: 4/183 levels. Stays where it is; 010 will consume its world model. |

## Transfer plan, by subsystem

- **002 LLM:** from `aimodel`, port `burn_attention.rs`, the `training/sso_*` optimiser, `generative_arch.rs::SubwordTokenizer` (BPE with persistence), `context_ring.rs` and `graph_attention.rs`, in that order. Everything ports behind `LanguageModel` and must beat the n-gram floor on held-out perplexity and on the harness.
- **009 Memory:** from `src/`, `data/vector_search` (pure-Rust HNSW) and `asi/identity.rs` (episodic, semantic and procedural memory). Also consider `embedvec` (E:\Libraries\AI-ML-RL).
- **016 Serve:** from `aimodel`, `bin/vortex_api.rs`, the API-key store, `docs/API.md` and the Dockerfile.
- **018 Perception:** from `src/`, the `voice_pipeline` capture and FFT code (`cpal`, `rustfft`), plus `ai/whisper_api.rs`.
- **Never transfer:** sacred-geometry scoring bonuses, mocked consensus providers, fabricated "scholar" training articles, and the self-modification loop.

## Eustress Engine findings (submodule updated to upstream `main` 5db8f2ee)

- **AI mode exists upstream** as a Studio Mode (`engine/modes/ai.toml`). Its seven disciplines (Acquisition, Curation, Provenance, Training, Reinforcement, Evaluation, Publishing) became Vortex's lifecycle (005).
- **The Guardian Policy v1.2** became Vortex's safety taxonomy (000, 004).
- **`eustress-data` provenance** gated Vortex training (002, through A7) until 2026-11-19, when `vortex_api::provenance` replaced it and Vortex stopped depending on engine crates.
- The engine **Workshop** is an agentic tool loop with per-message cost tracking. The **Soul Kernel** gates Rune rewrites. Vortex's 012 gate plays the same role for Vortex itself.
- Scripting runs Luau via `mlua` 0.10 and Rune 0.14. The engine exposes both through `execute_luau` and `execute_rune` (MCP and Engine Bridge). Vortex reaches them as tools through 006 and 017. Emitting Rune, Kernel-gated, is 013's job.
- **forge-orchestration** 0.6 (`E:\Libraries\Forge`) has job, MoE-routing and sim-cell scheduling, but **no cost metering**. 015 supplies that.
- The licence is **PolyForm Shield 1.0.0**: Vortex must not become a product that competes with Eustress itself. Building inside Eustress, as here, is the intended use.

## The local checkout was stale

When this audit began, the submodule sat on a feature branch 181 commits behind upstream. It held one unmerged local commit (`66cb4e4`, the billboard cull fix), which is still on branch `fix/billboard-cull-uses-primary-camera`. The submodule is now fast-forwarded to `origin/main`. The parent repository's recorded submodule pointer has not been committed.
