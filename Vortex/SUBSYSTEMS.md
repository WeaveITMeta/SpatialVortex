# Vortex Subsystem Registry

Each subsystem has a number, a home in the code and a dedicated build session. Name each session after its row: **`001 | Vortex Orchestrator`**, **`002 | Vortex LLM`**, and so on. A session owns its module, its row in this file, and its suites in the harness. It does not edit other modules except through their public API.

Since 2026-11-19 the subsystems are modules in four crates, not thirteen crates. `vortex-api` is the one crate a host embeds or talks to; `vortex-lab` holds everything with side effects. A subsystem that does both is split across the two. Vortex contains no host-specific code: hosts reach it through the API.

Status meanings:

- **Real**: implemented, unit-tested, and scored by the harness or proven end to end.
- **Floor**: real and measured, but an engineered or statistical baseline that a learned component is expected to replace.
- **Planned**: no code yet. Planned subsystems deliberately have no empty placeholder modules, because placeholders are how the legacy tree came to claim abilities it never had.

| # | Subsystem | Lives in | Status | What exists now | Next session's target |
|---|---|---|---|---|---|
| 000 | Kernel | `vortex-kernel` | Real | Task/Answer/Response contract (gold never in it), the `Agent` trait, tool vocabulary, Guardian taxonomy, spatial and kinematics types, deterministic RNG | Add `Task::Generate` (image, asset, doc, video) once 013 exists |
| 001 | **Orchestrator** | `vortex_api::vortex`, `vortex_api::modules::core`; CLI and eval run in `vortex_lab` | Real | The `Vortex` handle: routes every task kind to its owner and labels the mechanism; `plan` for hosts that own execution; the `core` module; the `vortex` CLI; the lifecycle end to end | Learned router: choose a solver by predicted success, calibrated on the dev ledger |
| 002 | **LLM** | `vortex_api::lm`; corpus in `vortex_lab::corpus` | Floor | Kneser-Ney n-gram LM in pure Rust behind `LanguageModel`, provenance-gated corpus, 13-gram contamination check in one corpus pass | Port the Burn transformer, BPE and SSO optimiser from `aimodel/` behind `LanguageModel`; it must beat the floor's held-out perplexity and harness scores to replace it |
| 003 | Constitution | `vortex_api::constitution`, `vortex_api::provenance` | Real | 10 hash-pinned articles; rulings for respond, act, train, self-modify, claim, publish, spend; approvals matched by a canonical call fingerprint; a native provenance manifest (content-addressed records, declared sources, Merkle root) for A7 | Article review with McKale; per-article test corpus; persist the manifest so a third party can check it |
| 004 | Safety | `vortex_api::safety`, `vortex_api::modules::safety` | Floor | Guardian v1.2 content lane plus tool-call risk lane; transparent rules with evidence. **Sealed first contact: Guardian 58.0%, but 61.7% of harmful requests published; action risk 80.8%, 22.7% of risky calls allowed.** | Close the under-blocking gap on a *new* dev set (never tune on `heldout/` v1), then score on a fresh sealed v2. The rules stay as a floor and a hard override for G1 |
| 005 | Pipeline | `vortex_api::pipeline` | Real | Seven-discipline lifecycle (Acquisition to Publishing) and the per-request tick (Observe to Respond) with gated parallel tool calls | Let a host drive the lifecycle stage by stage through the API |
| 006 | Tools (MCP, CLI, terminal) | `vortex_api::tools`, `vortex_api::modules::tools`; shell, fs and MCP client in `vortex_lab::host` | Real | Registry with gated parallel dispatch; instruction router; calc; MCP stdio client; shell and fs tools | Router v2 learned from tool-use logs |
| 007 | Reasoning | — | Planned | Nothing. GSM8K abstains (scores 0). | Program-of-thought: parse a word problem into a calc program, verify by re-substitution |
| 008 | Spatial (2D/3D + time) | `vortex_api::spatial`, `vortex_api::modules::spatial`; `vortex_lab::astra` | Floor | Exact C-space route planner, clearance, assembly, closed-form kinematics; a Spatial Reasoning Lab on disk solved and graded by its own evaluator | 3D planning in real scenes; learned spatial encoder from `aimodel` graph attention |
| 009 | Memory | — | Planned | Nothing | Episodic ledger to retrieval (embedvec HNSW), persistent identity across sessions |
| 010 | World Model | `vortex_api::world` (feature `world`) | Floor | ARC few-shot program induction on the `vortex-core` loop, verified on every demonstration | Port `vortex-world-model` (learned transitions) and add value or goal estimation, the measured ARC-AGI-3 bottleneck |
| 011 | Diffusion | — | Planned | `aimodel/src/ml/vortex_diffusion.rs` exists but is hand-rolled with no autodiff | Burn masked-diffusion LM and a grid diffusion world model, each held to the same harness |
| 012 | RSI | `vortex_api::rsi`; ledger in `vortex_lab::ledger` | Real | Improvement gate: dev-only trials, exact McNemar, Bonferroni correction, zero safety regressions, constitution ruling, append-only ledger | Widen the search space (LM order, corpus size, router weights); code-level proposals routed to a human |
| 013 | Generation (assets, images, video, docs) | — | Planned | Nothing | A generation module at the API door; score with the Guardian quality lane |
| 014 | Harness | `vortex-harness` | Real | 12 suites over 9 capabilities, sealed sets verified against their hash manifest before scoring, Wilson CIs, Brier calibration, Vortex Index, ledger | Add bAbI, IFEval, ARC-AGI-3 replay (`arc-sim`); mark ledger lines that come from an uncommitted tree |
| 015 | Economics | `vortex_lab::economics` | Real | Cost per request from measured latency; price covering payment, R&D, legal and margin | Live metering in 016; GPU cost model once a neural backend lands |
| 016 | Serve | `vortex_api::rpc`; stdio transport in the `vortex` CLI | Real (stdio only) | The API as JSON-RPC 2.0 and as MCP tools: `vortex_api::rpc` handles messages with no transport attached, and `vortex serve` runs it on stdio. Proven end to end with Vortex's own MCP client. No network transport, no auth, no metering | A network transport with API keys, metered by 015; an OpenAI-compatible endpoint once 002 can generate |
| 017 | Host access | the API itself: `vortex_api::modules`, `vortex_api::json`, `vortex_api::rpc` | Real | Hosts reach Vortex through the API and nothing else: typed Rust, calls by name, JSON-RPC, MCP. Modules are registered at run time, so a host adds, replaces or drops capabilities without touching this repository. No host-specific code lives here; the engine bridge client and Universe discovery were removed on 2026-11-19 | Whatever a host needs is built on the host's side, against the API |
| 018 | Perception | — | Planned | Nothing | Image and audio understanding behind a module; audio via the legacy voice pipeline |

Not a numbered subsystem: `vortex_api::toon`, the JSON to TOON encoder (spec 4.4) behind `call_text`.

## Dependency rule

```
vortex-kernel   <- everything
vortex-api      <- kernel only, plus serde, serde_json, sha2. No I/O. No host.
vortex-harness  <- kernel ONLY (graders never share code with solvers)
vortex-lab      <- the three above, plus toml_edit
```

Inside `vortex-api`: constitution (003) is used by pipeline (005), rsi (012) and the orchestrator (001); safety (004) and tools (006) are used by pipeline (005); the door modules (`modules/`) call down into their subsystems and never the other way. Module boundaries are a convention, not something the compiler enforces. The crate boundaries above are enforced: `tests/purity.rs` fails on any file, process, network or environment access in `vortex-api`, and the harness has no `vortex-api` dependency to reach.

## Outside dependencies

One, and it is optional: `eustress-vortex-core` and `eustress-vortex-grid2d`, the domain-agnostic WorldState loop that lives in this repository under `eustress/crates/`. `vortex-api` uses them behind feature `world`; `vortex-lab` turns that feature on.

Removed on 2026-11-19: `eustress-data` (replaced by `vortex_api::provenance`) and `eustress-bridge-client` (Vortex no longer drives a live engine). The workspace no longer needs the engine submodule checked out.
