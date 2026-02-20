# SpatialVortex Benchmark Monitoring & Improvement Guide

## Phase 1: Live Monitoring (During Benchmark Run)

### What to Watch
1. **Per-task accuracy** — Record each task's accuracy as it completes
2. **Failure patterns** — Note the `[FAILURE ANALYSIS]` traces for wrong answers
3. **Confidence scores** — Low confidence (<0.5) on correct answers = fragile wins
4. **RAG knowledge growth** — Track `topics` and `facts` counts during test-time training
5. **Architecture traces** — Watch ELP balance, MoE routing, and Sacred checkpoint signals

### Red Flags
- **0% accuracy** on any task = broken loader or evaluation logic
- **Random-chance accuracy** (25% for 4-choice, 50% for binary) = model not learning
- **Confidence always 1.0 or 0.0** = scoring function degenerate
- **RAG facts not growing** during test-time training = knowledge pipeline stalled

## Phase 2: Post-Run Analysis (After eval_results.json Written)

### Step 1: Categorize Results by Performance Tier
- **Excellent (>80%>)**: Model of SOTA
- **Strong (>60%)**: No immediate action needed
- **Moderate (30-60%)**: Analyze failure patterns, prioritize fixes
- **Weak (<30%)**: Deep investigation — likely structural issues
- **Broken (0-5%)**: Check data loader, evaluation logic, answer format

### Step 2: Identify Root Causes per Weak Task
For each weak task, check:
1. **Data loading** — Did the benchmark data load correctly? Check file paths and parsing.
2. **Answer format** — Is the model's output format matching what the evaluator expects?
3. **Knowledge coverage** — Does the RAG engine have relevant facts for this domain?
4. **Reasoning type** — Does the task require a reasoning capability the model lacks?
5. **Few-shot exemplars** — Are the 5-shot examples representative and helpful?

### Step 3: Cross-Task Pattern Analysis
- Group failures by **reasoning type** (factual recall, logical deduction, math, code, spatial)
- Identify if a single subsystem (RAG, CALM, MoE, TransitiveFlux) is the bottleneck
- Check if contrastive loss stagnation (~0.90) correlates with semantic similarity failures

## Phase 3: Prioritized Improvements

### Priority Matrix
| Impact | Effort | Action |
|--------|--------|--------|
| High   | Low    | Fix answer format mismatches, data loader bugs |
| High   | Medium | Improve RAG retrieval for weak domains |
| High   | High   | Fix contrastive loss stagnation for better embeddings |
| Medium | Low    | Tune scoring weights and confidence thresholds |
| Medium | Medium | Add domain-specific few-shot exemplars |
| Low    | High   | Architectural changes (new reasoning modules) |

### Specific Subsystem Checks
- **CALM engine**: Is calm_loss < 0.01? If not, increase training epochs or adjust LR
- **Contrastive**: Is loss < 0.5? If stuck at 0.9, investigate negative sampling strategy
- **RAG engine**: Are retrieved facts relevant? Check topic coverage per benchmark domain
- **MoE routing**: Are experts specializing? Check router entropy
- **Sacred observers**: Are control signals (Verify/Verified/Continue) correlating with accuracy?
- **TransitiveFlux**: Only helps spatial/size tasks (bAbI 17, 18). Check relation extraction.

## Phase 4: Iteration Cycle
1. Pick the highest-impact, lowest-effort fix
2. Implement the fix
3. Re-run ONLY the affected benchmarks (not all 31)
4. Compare before/after accuracy
5. If improved, commit. If not, revert and try next fix.

## Notes While Testing
- **HumanEval** Speed struggles during the HumanEval benchmark. Find ways to speed it up, and increase final score.