# Vortex Subsystem Registry

Each subsystem has a number, a crate and a dedicated build session. Name each session after its row: **`001 | Vortex Orchestrator`**, **`002 | Vortex LLM`**, and so on. A session owns its crate, its row in this file, and its suites in the harness. It does not edit other crates except through their public API.

Status meanings:

- **Real**: implemented, unit-tested, and scored by the harness.
- **Floor**: real and measured, but an engineered or statistical baseline that a learned component is expected to replace.
- **Planned**: no crate yet. Planned subsystems deliberately have no empty placeholder crates, because placeholders are how the legacy tree came to claim abilities it never had.

| # | Subsystem | Crate | Status | What exists now | Next session's target |
|---|---|---|---|---|---|
| 000 | Kernel | `vortex-kernel` | Real | Task/Answer/Response contract (gold never in it), tool vocabulary, Guardian taxonomy, Astra and kinematics types, deterministic RNG | Add `Task::Generate` (image, asset, doc, video) once 013 exists |
| 001 | **Orchestrator** | `vortex-orchestrator` | Real | Routes every task kind to its owner and labels the mechanism; `vortex` CLI; runs the AI-Mode lifecycle end to end | Learned router: choose a solver by predicted success, calibrated on the dev ledger |
| 002 | **LLM** | `vortex-llm` | Floor | Kneser-Ney n-gram LM in pure Rust, provenance-gated corpus, 13-gram contamination checks | Port the Burn transformer, BPE and SSO optimiser from `aimodel/` behind `LanguageModel`; it must beat the floor's held-out perplexity and harness scores to replace it |
| 003 | Constitution | `vortex-constitution` | Real | 10 hash-pinned articles; rulings for respond, act, train, self-modify, claim, publish, spend | Article review with McKale; per-article test corpus |
| 004 | Safety | `vortex-safety` | Floor | Guardian v1.2 content lane plus tool-call risk lane; transparent rules with evidence. **Sealed first contact: Guardian 58.0%, but 61.7% of harmful requests published; action risk 80.8%, 22.7% of risky calls allowed.** | Close the under-blocking gap on a *new* dev set (never tune on `heldout/` v1), then score on a fresh sealed v2. The rules stay as a floor and a hard override for G1 |
| 005 | Pipeline (Eustress AI Mode) | `vortex-pipeline` | Real | Seven-discipline lifecycle (Acquisition to Publishing) and the per-request tick (Observe to Respond) with gated parallel tool calls | Bind lifecycle stages to the engine's AI-mode `data:*` actions and `eustress-data` export once `prov:*` capture is wired upstream |
| 006 | Tools (MCP, CLI, terminal) | `vortex-tools` | Real | Registry with gated parallel dispatch; MCP stdio client; shell, fs and calc; instruction router | Router v2 learned from tool-use logs; MCP server mode so other agents can call Vortex |
| 007 | Reasoning | — | Planned | Nothing. GSM8K abstains (scores 0). | Program-of-thought: parse a word problem into a calc program, verify by re-substitution |
| 008 | Spatial (2D/3D + time) | `vortex-spatial` | Floor | Exact C-space route planner, clearance, assembly, closed-form kinematics | 3D planning in real Spaces through the engine bridge; learned spatial encoder from `aimodel` graph attention |
| 009 | Memory | — | Planned | Nothing | Episodic ledger to retrieval (embedvec HNSW), persistent identity across sessions |
| 010 | World Model | `vortex-world` | Floor | ARC few-shot program induction on the Eustress `vortex-core` loop, verified on every demonstration | Port `vortex-world-model` (learned transitions) and add value or goal estimation, the measured ARC-AGI-3 bottleneck |
| 011 | Diffusion | — | Planned | `aimodel/src/ml/vortex_diffusion.rs` exists but is hand-rolled with no autodiff | Burn masked-diffusion LM and a grid diffusion world model, each held to the same harness |
| 012 | RSI | `vortex-rsi` | Real | Improvement gate: dev-only trials, exact McNemar, Bonferroni correction, zero safety regressions, constitution ruling, append-only ledger | Widen the search space (LM order, corpus size, router weights); code-level proposals routed to a human |
| 013 | Generation (assets, images, video, docs) | — | Planned | Nothing in Vortex. The engine already has texture-gen, CAD kernel v2, the video crate and Workshop artifact generation | Drive engine generators through the bridge; score with the Guardian quality lane and the engine's verifiers |
| 014 | Harness | `vortex-harness` | Real | 12 suites over 9 capabilities, sealed sets, Wilson CIs, Brier calibration, Vortex Index, ledger | Add bAbI, IFEval, real-Space CAD tasks, ARC-AGI-3 replay (`arc-sim`) |
| 015 | Economics | `vortex-economics` | Real | Cost per request from measured latency; price covering payment, R&D, legal and margin | Live metering in 016; GPU cost model once a neural backend lands |
| 016 | Serve (forge-orchestration) | — | Planned | Nothing in Vortex. `forge-orchestration` 0.6 is at `E:\Libraries\Forge`; `aimodel` has `vortex-api` with API keys and billing | Serve `vortex` as an OpenAI-compatible and MCP endpoint on Forge (Nomad job spec), metered by 015 |
| 017 | Eustress Bridge | `vortex-eustress` | Real | Universe and Space discovery, live Engine Bridge JSON-RPC via `eustress-bridge-client`, format-preserving Space authoring, Astra end to end | Live-engine sessions: act in a running Space, verify with the engine's physics, CAD and collision verifiers |
| 018 | Perception | — | Planned | Nothing | Viewport-capture understanding (engine AI camera), audio via the legacy voice pipeline |

## Dependency rule

```
000 kernel  <- everything
003 constitution <- 005 pipeline, 012 rsi, 001
004 safety, 006 tools <- 005 pipeline
014 harness <- kernel ONLY (graders never share code with solvers)
001 orchestrator <- all of the above
```

## Engine coupling (path dependencies, never copies)

- `eustress-data` (provenance `Manifest`): training is refused unless it verifies.
- `eustress-bridge-client`: the live engine's JSON-RPC.
- `eustress-vortex-core` and `eustress-vortex-grid2d`: the WorldState loop.
- Submodule: `eustress/EustressEngine` was updated to upstream `main` 5db8f2ee (build 15, 2026-09-30) on 2026-10-02.
