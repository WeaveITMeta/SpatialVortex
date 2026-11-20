# Benchmark data

This folder holds the datasets the Vortex harness reads. The benchmark itself is the harness in [`Vortex/crates/harness`](../Vortex/crates/harness), run with `vortex eval`. Scores are on the [front page](../README.md).

An older runner used to live here. It printed scores that were constants in its source and measured nothing, so it was removed.

## Layout the harness expects

Everything goes under `benchmarks/data/`, which is not checked in.

| Suite | Path under `benchmarks/data/` |
|---|---|
| HellaSwag | `hellaswag/validation.jsonl` |
| ARC-Easy | `arc/ARC-V1-Feb2018-2/ARC-Easy/ARC-Easy-Test.jsonl` |
| ARC-Challenge | `arc/ARC-V1-Feb2018-2/ARC-Challenge/ARC-Challenge-Test.jsonl` |
| MMLU | `mmlu/data/test/*.csv` |
| GSM8K | `gsm8k/test.jsonl` |
| HumanEval | `humaneval/HumanEval.jsonl` |
| ARC-AGI | `arc-agi-2/arc_agi_training.json` |
| Language model training text | `arc/ARC-V1-Feb2018-2/ARC_Corpus.txt` |
| Tuning (development split only) | `arc/ARC-V1-Feb2018-2/ARC-Easy/ARC-Easy-Dev.jsonl`, `arc/ARC-V1-Feb2018-2/ARC-Challenge/ARC-Challenge-Dev.jsonl` |

The three sealed suites and the two procedural suites need nothing here: the sealed sets ship in `Vortex/eval/heldout/`, and the procedural ones are generated from the seed.

`vortex eval` prints a warning for each suite whose data is missing and scores the rest. HumanEval is graded by running Python, so it needs Python on the path.

## What else is here

| Path | What it is |
|---|---|
| `scripts/` | Download and verify scripts. They were written for the old runner, fetch several of the datasets above (MMLU and GSM8K among them), and may not cover every path |
| `src/bin/download_hf.rs` | A Rust downloader for datasets hosted on HuggingFace. It has no manifest at the moment, so it does not build as it stands |
| `benchmarks/data/` | A few small datasets that are checked in: CommonsenseQA, FB15k-237, SICK, SQuAD 2.0, STS and WN18RR. No suite reads them yet |
| `custom/`, `knowledge_graph/`, `semantic/` | Sources from the original SpatialVortex crate. They do not build |
