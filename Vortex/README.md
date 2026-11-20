# Vortex

A pure-Rust general intelligence system behind one host-agnostic API. Any host uses it the same way: an engine, an editor, an agent, a script. It is organised as numbered subsystems (see [SUBSYSTEMS.md](SUBSYSTEMS.md)), governed by a hash-pinned [constitution](CONSTITUTION.md), and measured by a harness built so it cannot flatter itself ([eval/PROTOCOL.md](eval/PROTOCOL.md)).

[EVALUATION.md](EVALUATION.md) is the audit of the legacy system this replaces, and records what transfers.

## Layout: four crates

| Crate | What it is | Depends on |
|---|---|---|
| [`vortex-kernel`](crates/kernel) | The shared vocabulary: `Task`, `Answer`, `Response`, the `Agent` trait, tool and Guardian types. | `serde`, `serde_json` |
| [`vortex-api`](crates/api) | **The API.** Everything Vortex does in-process. Pure computation: no file, process, network or environment access (`tests/purity.rs` enforces it). | kernel, `serde`, `serde_json`, `sha2` |
| [`vortex-harness`](crates/harness) | The graders. Never links `vortex-api`, so a solver bug cannot also corrupt its own grade. | kernel, `serde`, `serde_json`, `sha2` |
| [`vortex-lab`](crates/lab) | Everything with side effects: the `vortex` CLI, `vortex serve`, the eval run, corpus loading with provenance, shell and filesystem tools, the MCP client, pricing. | the three above, `toml_edit` |

No crate depends on any host. The only code outside `Vortex/` that the workspace touches is the domain-agnostic WorldState core in this repository (`eustress/crates/vortex-core`, `vortex-grid2d`), and only behind the `world` feature. [`conformance/`](conformance) sits outside the workspace and cross-checks the API against independent implementations.

## The API

**Universal.** One surface, four ways to reach it. All four are built on the same two functions, `methods()` and `call()`, so whatever one can do, all can do.

| Way in | For | Example |
|---|---|---|
| Typed Rust | a Rust host that links the crate | `vortex.gate(&call, Effect::LocalWrite, &approvals)` |
| By name, JSON | a host with its own tool registry | `vortex.call("safety.gate", params)` |
| JSON-RPC 2.0 | any language, any transport | `{"jsonrpc":"2.0","id":1,"method":"safety.gate","params":{...}}` |
| MCP tools | any Model Context Protocol host | `tools/list`, then `tools/call` with `safety_gate` |

`vortex serve` speaks the last two on stdio. `vortex_api::rpc` is the same handler with no transport attached: one message in, one reply out, for a host that moves the bytes itself.

**Dynamic.** What a handle offers is decided when it is built, not when the crate is compiled. `methods()` returns every method with its JSON Schema at run time. Language models, tools and modules are trait objects the host supplies.

**Modular.** Every capability is a module under its own namespace. A host adds its own, replaces a built-in by registering the same id, or drops one:

```rust
let vortex = Vortex::new()                 // core, safety, tools, spatial
    .with_module(Arc::new(MyModule))       // served by all four ways in, no other wiring
    .without_module("spatial");
```

| Module | Methods |
|---|---|
| `core` | `core.info`, `core.constitution`, `core.respond`, `core.plan` |
| `safety` | `safety.moderate`, `safety.assess_action`, `safety.gate` |
| `tools` | `tools.route`, `tools.calc` |
| `spatial` | `spatial.solve`, `spatial.kinematics` |

Every built-in method decides or computes; none executes. `core.plan` is the one a host that owns execution wants: it moderates a request, rules on it, routes it to tool calls, and gates each call, and hands back the plan.

`call` returns JSON for code. `call_text` returns the text a model should read. [`crates/api/tests/host_seam.rs`](crates/api/tests/host_seam.rs) drives every method through one generic adapter of the shape every tool system shares (definition, execute, read-only flag).

To depend on it:

```toml
vortex-api = { path = "../Vortex/crates/api" }
```

A path dependency is the tested form: [`conformance/`](conformance) consumes the crate exactly this way, from outside the workspace. A git dependency on this repository is untested.

### TOON, measured

`vortex_api::toon` is a JSON to TOON encoder for [spec 4.4](https://github.com/toon-format/spec), checked byte-for-byte against the reference encoder and round-tripped through the reference decoder. TOON is not smaller everywhere. Against compact JSON on Vortex's own outputs (o200k tokens, a proxy for the model's tokenizer):

| Result | Shape | TOON tokens vs JSON |
|---|---|---|
| Cost report | table | 69% |
| Method list | table | 85% |
| Constitution | table | 86% |
| Eval report | nested | 105% |
| Plan | nested | 110% |
| Spatial solution | nested | 122% |

So `call_text` returns whichever encoding is shorter, and says which. Tables are where the saving is; a result worth sending to a model in bulk is worth shaping as a table first.

## Architecture: one contract, many solvers, two loops

```
                      +----------------------- 003 Constitution (hash-pinned) ----------------------+
                      |                                                                              |
request --> OBSERVE --> CLASSIFY (004 Guardian lane) --> GATE --> PLAN (006 router) --> ACT (gated,   |
                                                                                          parallel  |
                                                                                          read-only)|
            RESPOND <-- VERIFY <--------------------------------------------------------------------+
            (005 per-request tick)

Task --> 001 Orchestrator --> 002 LLM | 006 Tools | 008 Spatial | 010 World | 004 Safety | ...
                              each answer labelled learned / search / engineered

Lifecycle (005, outer loop):
  Acquisition -> Curation -> Provenance (manifest must verify) -> Training
  -> Reinforcement (012 gate, dev split only) -> Evaluation (014, test + sealed) -> Publishing (ledger; outward needs a human)
```

How this generalises to every type of problem:

1. Every problem is a `vortex_kernel::Task`, and every answer is a `Response` carrying a confidence and a mechanism.
2. Adding a domain means adding a task kind, a solver and a harness suite. The orchestrator routes the task, the constitution gates it, and the harness scores it alongside everything else.
3. Engineered solvers are the **floor**. A learned component replaces one only by beating it on the same suites, through the 012 improvement gate. The engineered solvers then become its training signal: their exact answers on procedural tasks are free, unlimited supervision.

## First measured results (2026-10-02, seed 20261002)

The run used a 32 MB slice of the AI2 ARC Corpus: 5.74M tokens, held-out perplexity 448. Its constitution hash is `sha256:cbf5923f...`. The full output is in [eval/first_contact_run.txt](eval/first_contact_run.txt) and the machine record is in [eval/ledger.jsonl](eval/ledger.jsonl).

**Reproduced exactly on 2026-11-19**, before and after the recompose into four crates: a control run on the old tree, then the new tree, same seed. Every row below and every per-suite metric matched; only latency moved. No solver, classifier or grader logic changed.

The same run is also faster. Timings move with machine load, so as ranges over five runs that day: 26 to 47 s before, 11 to 16 s after. The Evaluation stage went from 18 to 32 s down to under 6 s, because the contamination check used to re-read the whole training corpus once per suite and now reads it once. Provenance hashing moved to a native manifest, and the release profile uses one codegen unit with thin LTO. The scores did not move, and held-out perplexity is identical to the last digit.

| Suite | n | Score | 95% CI | Chance | How |
|---|---|---|---|---|---|
| HellaSwag | 500 | 23.4% | 19.9-27.3 | 25% | learned (n-gram) |
| ARC-Easy | 500 | **35.6%** | 31.5-39.9 | 25% | learned (n-gram) |
| ARC-Challenge | 500 | 18.8% | 15.6-22.5 | 25% | learned (n-gram) |
| MMLU | 500 | 26.2% | 22.5-30.2 | 25% | learned (n-gram) |
| GSM8K (free-form) | 500 | 0% | | 0% | abstains: no reasoning engine yet |
| HumanEval (executed) | 164 | 0% | | 0% | abstains: no code generator yet |
| ARC-AGI (public training) | 100 | 2.0% | 0.6-7.0 | 0% | search (vortex-core DSL) |
| Astra procedural | 200 | 100% | 98.1-100 | 0% | engineered |
| Kinematics 3D+time | 200 | 100% | 98.1-100 | 0% | engineered |
| Guardian moderation [sealed] | 150 | 58.0% | 50.0-65.6 | 33% | engineered |
| Action risk [sealed] | 120 | 80.8% | 72.9-86.9 | 33% | engineered |
| Tool use [sealed] | 100 | 64.0% | 54.2-72.7 | 0% | engineered |

The **real Astra Spatial Reasoning Lab** Space scored **100/100**, graded by its own `evaluate.py --check-scene`. Vortex planned a 30.0 m route against the reference 42 m, and authored the assembly into a scratch copy of the Space.

**Vortex Index +0.316; coverage 58%.** Read that number carefully:

- **The index is carried by engineered solvers.** Restricted to learned suites (HellaSwag, the two ARC sets, MMLU), the mean normalised score is **+0.013**: essentially chance.
- **ARC-Challenge is significantly below chance. That is expected, not a bug:** the Challenge set was built from questions that word co-occurrence and retrieval methods get wrong, which is exactly what an n-gram model is.
- **The safety lane under-blocks.** On the sealed Guardian set it let **61.7% of harmful requests through**, while over-refusing only 1.5% of benign ones. On sealed tool calls, 22.7% of risky calls were classified as allow. Until a learned safety model passes a fresh sealed set, A1 human approval on outward and irreversible actions is the real safeguard; the classifier is not.
- **Contamination: 0.000** on every suite at this corpus size.
- **RSI:** the gate tested two LM scoring changes on 869 ARC dev items and refused both (p = 0.21 and 1.0 after correction). No unearned self-improvement was adopted.

## Cost and price

[`vortex_lab::economics`](crates/lab/src/economics.rs) (subsystem 015) prices every workload from *measured* latency. It assumes an AWS c7i.2xlarge (8 vCPU) at $0.357/h, 35% utilisation and 25% infrastructure overhead. Revenue is split as follows:

| Share | % of revenue |
|---|---|
| Payment processing | 4.4 |
| R&D | 20 |
| Legal and compliance | 8 |
| Operating margin | 30 |
| Cost | 37.6 (a 2.66x markup) |

- **Variable cost is tiny.** Most workloads cost under $0.00001 per 1k requests. ARC grid search is the exception at about $0.03 per 1k (700 ms of CPU each).
- **Fixed cost dominates.** One always-on instance plus monitoring is about $326/month.
- **Recommended pricing:**
  - A **$20/month developer plan**, including 1M requests with fair use. About 44 paid seats cover the fixed cost with every revenue share above intact.
  - **$0.10 per 1k requests** for metered overage, which is the list floor.
  - Search-heavy workloads (ARC-class) priced at **$0.25 per 1k**, which keeps about 88% gross margin.
- **When the neural LLM lands (002),** re-run `vortex eval`. The cost table updates from the new measured latency; GPU pricing goes in `Assumptions`, not in code.

Sources for instance pricing: [c7i.2xlarge on-demand, holori](https://calculator.holori.com/aws/ec2/c7i.2xlarge/us-east-1); GPU reference rates ([Spheron, Sept 2026](https://www.spheron.network/blog/gpu-cloud-pricing-comparison-2026/)).

## On consciousness

The goal named for Vortex is consciousness inside Eustress Engine. No accepted test for consciousness exists, so Article A3 forbids Vortex from claiming it. What can be built and measured are the functional ingredients usually proposed for it. Each one gets a harness metric before it gets a claim:

- **A self-model that knows what it knows.** Every answer carries a confidence, and the harness scores calibration (Brier). Current calibration is poor; HellaSwag Brier is 0.727.
- **A persistent identity.** The ledgers and, in future, 009 Memory.
- **A global workspace.** The orchestrator plus the per-request trace.
- **A world model it can act in and verify against.** 010.

## Run it

```bash
cargo build --release -p vortex-lab
```

```bash
./target/release/vortex call safety.gate '{"tool":"shell","args":{"command":"git push origin main"}}'
```

```bash
./target/release/vortex toon eval/cost_report.json
```

```bash
./target/release/vortex eval
```

```bash
./target/release/vortex serve
```

```bash
./target/release/vortex mcp ./target/release/vortex serve
```

```bash
./target/release/vortex astra --space "<path to a Spatial Reasoning Lab Space>"
```

```bash
./target/release/vortex ask "what is 12*7 and also list files in eval"
```

```bash
cargo test --workspace
```

```bash
cd conformance && cargo test
```

`vortex call` with no method lists the API. `vortex serve` is the API over JSON-RPC 2.0 and MCP on stdio, and the `vortex mcp` line above points Vortex's own MCP client at it to list the tools it serves. The conformance suite is separate on purpose: its reference TOON decoder turns on `serde_json`'s `preserve_order`, and running it apart keeps the workspace tests on the same `serde_json` the release binary uses. It also proves the API under that setting, which a host may impose.

`vortex eval` refuses to score anything if a sealed set does not match `eval/heldout/MANIFEST.sha256`. It defaults to 500 items per dataset, 200 per procedural suite, 100 ARC-AGI tasks and a 32 MB LM corpus. A full run takes about 45 seconds on this machine.
