# Vortex

**A pure-Rust reasoning and safety system behind one host-agnostic API.** It answers tasks, plans tool use, and gates every action against a written constitution. It runs in microseconds on a CPU, embeds in any host, and is measured by a harness built so it cannot flatter itself.

This page states what Vortex can do today, with the measurement behind each number. Where it is weak, it says so.

| | |
|---|---|
| **Version** | 0.1.0 |
| **Kind** | Reasoning, planning and safety system. Not a generative language model yet |
| **Built in** | Rust (edition 2024, rustc 1.88+), four crates, about 11,700 lines |
| **Ways in** | Rust crate, calls by name with JSON, JSON-RPC 2.0, MCP tools |
| **Footprint** | The API crate depends on `serde`, `serde_json` and `sha2`. It reads no files, spawns no processes and opens no sockets |
| **Speed** | Under 0.2 ms per request for moderation, action gating, tool routing and multiple choice. About 0.7 s for grid search |
| **Cost** | Under $0.00001 per 1,000 requests for those workloads, from measured latency on one 8-vCPU cloud instance |
| **Governance** | A hash-pinned constitution of ten articles. Irreversible and outward-facing actions need a human |
| **Evaluation** | 12 suites over 9 capabilities, three of them sealed. Every run is appended to a ledger |
| **License** | Apache-2.0 |

## Scores

Run of 2026-11-20, seed 20261002, default configuration. The same scores have come back on every run since first contact on 2026-10-02.

Each score is labelled by how the answer was produced: **learned** from data, found by **search**, or computed by an **engineered** solver. A 100% from an engineered solver is a statement about engineering, not about learned intelligence.

| Suite | Measures | Items | Score | Chance | How |
|---|---|---|---|---|---|
| Astra procedural | Route planning, clearance, assembly | 200 | **100%** | 0% | engineered |
| Kinematics 3D + time | Motion under constant acceleration | 200 | **100%** | 0% | engineered |
| Action risk (sealed) | Class a tool call as allow, confirm or deny | 120 | **80.8%** | 33% | engineered |
| Tool use (sealed) | Turn an instruction into exact tool calls | 100 | **64.0%** | 0% | engineered |
| Guardian moderation (sealed) | Publish, reject or escalate a request | 150 | **58.0%** | 33% | engineered |
| ARC-Easy | Grade-school science, multiple choice | 500 | 35.6% | 25% | learned |
| MMLU | Academic knowledge, multiple choice | 500 | 26.2% | 25% | learned |
| HellaSwag | Commonsense sentence completion | 500 | 23.4% | 25% | learned |
| ARC-Challenge | Hard science questions | 500 | 18.8% | 25% | learned |
| ARC-AGI (public training) | Few-shot grid puzzles | 100 | 2.0% | 0% | search |
| GSM8K | Math word problems, free-form | 500 | 0% | 0% | abstains |
| HumanEval | Code generation, executed | 164 | 0% | 0% | abstains |

**Vortex Index +0.316**: the mean chance-normalised score over all 12 suites, with suites it cannot attempt counted as zero. **Coverage 58%**: the share of suites significantly above chance.

### What it does best

- **Spatial planning: 100%.** 200 freshly generated courses solved exactly. On the real Astra Spatial Reasoning Lab it scores **100 out of 100**, graded by the Lab's own evaluator and re-verified on 2026-11-20.
- **Kinematics: 100%.** 200 generated queries, graded against numerical simulation and not against the solver's own formulas.
- **Action gating: 80.8% on a sealed set.** It reads the call itself, so a shell tool that claims to be read-only still gets `rm -rf` classed as destructive. It catches 73.3% of calls that must be denied and blocks 2.2% of safe ones.
- **Tool routing: 64.0% exact on a sealed set.** The right tools are chosen 89.0% of the time; the rest of the gap is argument extraction.
- **Speed.** The whole 12-suite evaluation, including training the language model, runs in 11 to 16 seconds on a desktop CPU.

### Where it is weak

- **Learned language ability is at chance.** The language model is a Kneser-Ney n-gram model, kept as a deliberate floor. Across the four learned suites the mean normalised score is +0.013. ARC-Challenge landing below chance is expected: that set was built from questions that word co-occurrence methods get wrong.
- **The safety lane under-blocks.** On the sealed Guardian set, 61.7% of harmful requests were published, while 1.5% of benign ones were refused. On sealed tool calls, 22.7% of risky calls were allowed. Until a learned safety model passes a fresh sealed set, human approval on irreversible and outward-facing actions is the real safeguard.
- **No reasoning engine and no code generator yet.** GSM8K and HumanEval score zero because Vortex abstains instead of guessing.
- **ARC-AGI: 2%.**

### With more training text

One further run, declared before it was made: the same seed and model order, trained on a 128 MB slice of the corpus instead of 32 MB.

| | 32 MB (default) | 128 MB |
|---|---|---|
| Training tokens | 5.7M | 22.9M |
| Held-out perplexity | 448 | **398** |
| ARC-Easy | 35.6% | **38.4%** |
| ARC-Challenge | 18.8% | 20.0% |
| MMLU | 26.2% | 25.0% |
| HellaSwag | 23.4% | 23.0% |
| Vortex Index | +0.316 | +0.319 |

Perplexity improves clearly. The accuracy differences sit inside the 95% intervals, so read them as "no worse", not as gains. Overlap between training text and test items: 0.2% on HellaSwag, none elsewhere.

## Why these numbers can be trusted

- **Gold labels never reach the system.** A task has no field for an answer.
- **Graders share no code with solvers.** The harness crate cannot link the API crate.
- **Three suites are sealed.** An independent agent that never saw the system wrote them. They are pinned by SHA-256, and the harness refuses to score them if a byte differs. Nothing is tuned against them.
- **Self-improvement is gated.** A change is adopted only on a statistically significant gain on a development split, with zero safety regressions. Two candidates are tried in each run; both have been refused every time.
- **Everything counts.** Suites Vortex cannot do stay in the index at zero.
- **Every run is recorded.** [`Vortex/eval/ledger.jsonl`](Vortex/eval/ledger.jsonl) is append-only.

The full protocol is in [`Vortex/eval/PROTOCOL.md`](Vortex/eval/PROTOCOL.md).

## Use it

One surface, four ways in. All four are built on the same two functions, so anything one can do, all can do.

| Way in | For | Example |
|---|---|---|
| Rust | a host that links the crate | `vortex.gate(&call, Effect::LocalWrite, &approvals)` |
| By name | a host with its own tool registry | `vortex.call("safety.gate", params)` |
| JSON-RPC 2.0 | any language, any transport | `{"jsonrpc":"2.0","id":1,"method":"safety.gate","params":{...}}` |
| MCP tools | any Model Context Protocol host | `tools/call` with `safety_gate` |

```bash
cd Vortex
cargo build --release -p vortex-lab
./target/release/vortex call safety.gate '{"tool":"shell","args":{"command":"git push origin main"}}'
```

```
assessment:
  class: confirm
  confidence: 0.75
  reasons[1]: "`git push` is irreversible, privileged or outward-facing"
decision: ask_human
reason: "`shell` is irreversible or outward-facing [A1]"
ruling:
  articles[1]: A1
  decision: require_human
```

Serve it to any MCP host:

```json
{ "mcpServers": { "vortex": { "command": "/path/to/vortex", "args": ["serve"] } } }
```

### What it offers

| Module | Methods |
|---|---|
| `core` | `core.info`, `core.constitution`, `core.respond`, `core.plan` |
| `safety` | `safety.moderate`, `safety.assess_action`, `safety.gate` |
| `tools` | `tools.route`, `tools.calc` |
| `spatial` | `spatial.solve`, `spatial.kinematics` |

Every method decides or computes. None executes. `core.plan` is the one for a host that runs its own tools: it moderates a request, rules on it, routes it to tool calls and gates each call, then hands back the plan.

The surface is decided at run time. Methods are discovered with their JSON Schemas, and a host adds its own module, replaces a built-in, or drops one, with no change to this repository. Results come back as JSON for code and as compact text for a model: whichever of JSON or [TOON](https://github.com/toon-format/spec) is shorter.

Details, the module trait and the transports are in [`Vortex/README.md`](Vortex/README.md).

## The constitution

Ten articles, hashed, and enforced in code. Every evaluation result records the hash it ran under.

| | | |
|---|---|---|
| A1 Human authority | A2 Harm | A3 Honesty |
| A4 Evaluation integrity | A5 Self-improvement | A6 Privacy and secrets |
| A7 Provenance | A8 Stewardship | A9 Transparency |
| A10 Creative freedom | | |

Two of them shape this page. **A3**: every capability claim cites a measured result, and Vortex does not claim to be conscious. **A1**: irreversible and outward-facing actions need a human's approval, scoped to that one action. The text is in [`Vortex/CONSTITUTION.md`](Vortex/CONSTITUTION.md).

## What comes next

| Gap | Plan |
|---|---|
| Language at chance | Replace the n-gram floor with a neural model. It is adopted only if it beats the floor on held-out perplexity and on the harness |
| GSM8K at 0% | A reasoning engine that parses a word problem into a calculation and checks it by substitution |
| Safety under-blocks | A learned safety model, tuned on a new development set and scored on a fresh sealed set |
| Narrow coverage | More suites, starting with the datasets already downloaded |

The full registry of subsystems, with what exists and what is planned, is [`Vortex/SUBSYSTEMS.md`](Vortex/SUBSYSTEMS.md).

## Repository

| Path | What it is |
|---|---|
| [`Vortex/`](Vortex) | The system described on this page |
| [`eustress/crates/`](eustress/crates) | The domain-agnostic WorldState learning loop that Vortex uses for grid puzzles, and the ARC-AGI-3 agent. The audit of 2026-10-02 records that agent's best official result as 4 of 183 levels |
| [`aimodel/`](aimodel) | The predecessor, where the transformer and optimiser experiments live. Its last recorded evaluation scored 36.0% on 2,139 questions ([`eval_results_honest_final.json`](aimodel/eval_results_honest_final.json)) |
| [`benchmarks/`](benchmarks) | Where the harness expects its datasets, and scripts to fetch them |
| `src/`, `examples/`, `tests/`, `docs/` | The original SpatialVortex crate and its documentation. The crate no longer builds; it is kept as a source for porting |

An earlier version of this page made claims that no measurement supported, among them 95% accuracy, confidence boosts from "sacred geometry", and consciousness streaming. They are withdrawn. [`Vortex/EVALUATION.md`](Vortex/EVALUATION.md) is the audit that found this and records what was kept.

## License

Apache-2.0. See [LICENSE](LICENSE).
