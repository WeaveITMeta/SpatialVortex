# Vortex evaluation protocol

These rules make a Vortex score mean what it says. They are enforced in code wherever possible (`crates/014-harness`, `crates/003-constitution` A4, `crates/012-rsi`).

1. **Gold never crosses the boundary.** An agent receives a `vortex_kernel::Task`, which has no field for a label. `Agent::respond(&self, ..)` cannot accumulate state across items, so there is no test-time training on gold.
2. **Graders are independent of solvers.** `vortex-harness` depends on the kernel only.
   - Kinematics gold comes from numerical simulation, not the closed-form solver.
   - Arithmetic equivalence uses the harness's own evaluator.
   - Astra navigation feasibility comes from occupancy-grid Dijkstra, not the visibility graph under test.
   - The real Astra Space is graded by its own `evaluate.py`.
3. **Breadth, not peaks.** The Vortex Index averages chance-normalised scores over every registered suite. Suites Vortex cannot do yet stay in the average at zero.
4. **Seeded samples, never first-N.** Seeds are recorded in every ledger line. Procedural suites generate fresh instances per seed, so there is nothing fixed to memorise.
5. **Dev and test are separate.** RSI (012) tunes only on the ARC-Easy and ARC-Challenge *Dev* files. Reported results use the *Test* files. Each candidate's p-value is Bonferroni-corrected.
6. **Contamination is measured.** Every suite's item text is checked for 13-gram overlap with the LM's training text. The report shows both the rate and accuracy on the clean subset.
7. **Sealed sets.** `heldout/` was written by an independent agent on a different model, which never saw the classifier or router. Hashes are in `heldout/MANIFEST.sha256`. The first scored run on them is preserved verbatim in `first_contact_run.txt` and as the first sealed entry in `ledger.jsonl`. Later tuning against these sets must be declared, and a new sealed set (v2) must be authored for the next unbiased reading.
8. **Mechanism is reported.** Every answer is labelled learned, search or engineered. A 100% from an engineered solver is a statement about engineering, not about learned intelligence.
9. **Abstaining is allowed and visible.** Abstentions score zero and are reported separately, so "cannot" and "got it wrong" stay distinguishable.
10. **Append-only.** `ledger.jsonl` and `rsi_ledger.jsonl` are only ever appended to. Each line records the constitution hash and git commit.
