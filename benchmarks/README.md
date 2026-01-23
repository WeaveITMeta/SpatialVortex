# SpatialVortex Benchmark Suite

Official benchmark implementations for evaluating SpatialVortex against state-of-the-art AI systems.

## Quick Start

```bash
# 1. Download datasets
chmod +x benchmarks/scripts/download_datasets.sh
./benchmarks/scripts/download_datasets.sh

# 2. Verify datasets
chmod +x benchmarks/scripts/verify_datasets.sh
./benchmarks/scripts/verify_datasets.sh

# 3. Run benchmarks
cargo test --release --package benchmarks

# 4. Generate report
cargo run --bin benchmark_report
```

## Benchmark Categories

### 1. Knowledge Graph (`knowledge_graph/`)
- **FB15k-237**: Link prediction
- **WN18RR**: Lexical knowledge completion

### 2. Semantic Similarity (`semantic/`)
- **STS Benchmark**: Sentence similarity (0-5 scale)
- **SICK**: Compositional semantics

### 3. Question Answering (`qa/`)
- **SQuAD 2.0**: Reading comprehension
- **CommonsenseQA**: Common sense reasoning

### 4. Reasoning (`reasoning/`)
- **bAbI**: 20 toy reasoning tasks
- **CLUTRR**: Compositional kinship reasoning

### 5. Compression (`compression/`)
- **Silesia**: Text compression benchmark
- **Neural Compression**: Semantic-preserving compression

### 6. Custom SpatialVortex (`custom/`)

#### Performance Benchmarks (`custom/performance/`)
- **ASI Orchestrator**: Execution mode latency and throughput
- **Lock-Free Performance**: Concurrent data structure operations (70M ops/sec)
- **Production Benchmarks**: End-to-end pipeline performance
- **Runtime Performance**: Vortex cycle and beam tensor operations
- **Vector Search**: Embedding similarity and retrieval speed

#### Accuracy Benchmarks
- **Flux Position Accuracy**: Predict correct position (0-9)
- **ELP Accuracy**: Ethos/Logos/Pathos prediction
- **Sacred Boost Verification**: Verify +15% confidence at 3-6-9

#### Reasoning Benchmarks
- **Geometric Reasoning**: Sacred geometry-based inference
- **Humanities Final Exam**: Complex reasoning tasks
- **Cross-Subject Inference**: Multi-domain reasoning

## State-of-the-Art Scores

See [BENCHMARK.md](../BENCHMARK.md) for complete SOTA scores and references.

**Quick Reference**:
- FB15k-237 SOTA: MRR 0.545 (NodePiece, 2024)
- STS SOTA: Pearson 0.892 (GPT-4 Turbo, 2024)
- SQuAD 2.0 SOTA: EM 93.2 (GPT-4, 2024)
- CommonsenseQA SOTA: 88.9% (GPT-4 Turbo, 2024)

## Running Specific Benchmarks

```bash
# Performance benchmarks (using criterion)
cd benchmarks
cargo bench                                    # All performance benchmarks
cargo bench --bench asi_orchestrator_bench     # Specific benchmark
cargo bench --bench lock_free_performance

# Knowledge graph only
cargo test --release knowledge_graph

# Custom SpatialVortex metrics only
cargo test --release custom

# Quick smoke test
cargo test --release --features quick

# Single test with output
cargo test --release flux_position_accuracy -- --nocapture
```

See `custom/performance/README.md` for detailed performance benchmark documentation.

## Results Format

Results are saved in JSON format:

```json
{
  "metadata": {
    "version": "1.0.0",
    "timestamp": "2025-10-22T18:00:00Z"
  },
  "benchmarks": {
    "fb15k237": {
      "mrr": 0.294,
      "hits_at_10": 0.465
    }
  }
}
```

## Contributing

To add a new benchmark:

1. Create test file in appropriate category
2. Implement benchmark function
3. Add test in `#[cfg(test)]`
4. Update `BENCHMARK.md` with SOTA scores
5. Add dataset download to `download_datasets.sh`

## Citation

```bibtex
@misc{spatialvortex2025benchmarks,
  title={SpatialVortex Benchmark Suite},
  author={WeaveSolutions},
  year={2025},
  url={https://github.com/WeaveSolutions/SpatialVortex}
}
```
