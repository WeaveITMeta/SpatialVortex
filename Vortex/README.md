# Vortex

A pure-Rust general intelligence system, built to run inside Eustress Engine. It is organised as numbered subsystems (see [SUBSYSTEMS.md](SUBSYSTEMS.md)), governed by a hash-pinned [constitution](CONSTITUTION.md), and measured by a harness built so it cannot flatter itself ([eval/PROTOCOL.md](eval/PROTOCOL.md)).

[EVALUATION.md](EVALUATION.md) is the audit of the legacy system this replaces, and records what transfers.

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

Eustress AI Mode lifecycle (005, outer loop):
  Acquisition -> Curation -> Provenance (eustress-data Manifest must verify) -> Training
  -> Reinforcement (012 gate, dev split only) -> Evaluation (014, test + sealed) -> Publishing (ledger; outward needs a human)
```

How this generalises to every type of problem:

1. Every problem is a `vortex_kernel::Task`, and every answer is a `Response` carrying a confidence and a mechanism.
2. Adding a domain means adding a task kind, a solver and a harness suite. The orchestrator routes the task, the constitution gates it, and the harness scores it alongside everything else.
3. Engineered solvers are the **floor**. A learned component replaces one only by beating it on the same suites, through the 012 improvement gate. The engineered solvers then become its training signal: their exact answers on procedural tasks are free, unlimited supervision.

## First measured results (2026-10-02, seed 20261002)

The run used a 32 MB slice of the AI2 ARC Corpus: 5.74M tokens, held-out perplexity 448. Its constitution hash is `sha256:cbf5923f...`. The full output is in [eval/first_contact_run.txt](eval/first_contact_run.txt) and the machine record is in [eval/ledger.jsonl](eval/ledger.jsonl).

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

[`015-economics`](crates/015-economics) prices every workload from *measured* latency. It assumes an AWS c7i.2xlarge (8 vCPU) at $0.357/h, 35% utilisation and 25% infrastructure overhead. Revenue is split as follows:

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
- **A world model it can act in and verify against.** 010 and 017.

## Run it

```bash
cargo build --release -p vortex-orchestrator
```

```bash
./target/release/vortex eval
```

```bash
./target/release/vortex astra
```

```bash
./target/release/vortex ask "what is 12*7 and also list files in eval"
```

```bash
./target/release/vortex spaces
```

```bash
cargo test --workspace
```

`vortex eval` defaults to 500 items per dataset, 200 per procedural suite, 100 ARC-AGI tasks and a 32 MB LM corpus. A full run takes about 45 seconds on this machine.
