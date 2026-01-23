# Component Testing & Benchmarking Guide

**Purpose**: Map each benchmark to its component and provide targeted testing commands

---

## 📊 **Complete Benchmark-to-Component Map**

### **Benchmark Files (3 total)**

| Benchmark File | Components Tested | Location |
|----------------|-------------------|----------|
| `runtime_performance.rs` | Runtime engine components | `benches/` |
| `lock_free_performance.rs` | Lock-free data structures | `benches/` |
| `vector_search_benchmark.rs` | HNSW vector search | `benches/` |
| `geometric_reasoning_benchmark.rs` | Flux matrix reasoning | `benchmarks/custom/` |

---

## 🎯 **Component #1: ELP Tensor Operations**

### **Source Component**
```
src/models.rs
  └─ struct ELPTensor
      ├─ distance()     [HOT PATH]
      ├─ magnitude()    [HOT PATH]
      └─ normalize()
```

### **Benchmark**
```rust
// benches/runtime_performance.rs
fn bench_elp_distance()    // Line 21
fn bench_elp_magnitude()   // Line 33
```

### **Run Benchmark**
```bash
# All ELP benchmarks
cargo bench --bench runtime_performance -- elp

# Specific benchmark
cargo bench --bench runtime_performance -- elp_distance
cargo bench --bench runtime_performance -- elp_magnitude
```

### **Run Unit Tests**
```bash
# Test ELP tensor functionality
cargo test --lib elp

# With output
cargo test --lib elp -- --nocapture
```

### **Expected Performance**
- **distance()**: <100ns per operation
- **magnitude()**: <100ns per operation

---

## 🎯 **Component #2: Vortex Cycle Engine**

### **Source Component**
```
src/runtime/vortex_cycle.rs
  └─ struct VortexCycleEngine
      ├─ add_object()
      ├─ start()
      ├─ stop()
      ├─ get_objects()
      └─ tick() [CORE LOOP]
```

### **Benchmark**
```rust
// benches/runtime_performance.rs
fn bench_vortex_cycle()    // Line 44
  └─ Tests: 10, 100, 1K, 5K objects
```

### **Run Benchmark**
```bash
# All vortex benchmarks
cargo bench --bench runtime_performance -- vortex

# Specific size
cargo bench --bench runtime_performance -- vortex_1000
```

### **Run Unit Tests**
```bash
# Test vortex cycle
cargo test --lib vortex_cycle

# Integration test
cargo test --test '*' vortex
```

### **Expected Performance**
- **Throughput**: >10,000 objects/second
- **Latency**: <100μs per tick

---

## 🎯 **Component #3: Ladder Index**

### **Source Component**
```
src/runtime/ladder_index.rs
  └─ struct LadderIndex
      ├─ add_entry()
      ├─ get_ranked_entries()  [HOT PATH]
      ├─ update_rankings()
      └─ test_similarity()
```

### **Benchmark**
```rust
// benches/runtime_performance.rs
fn bench_ladder_ranking()  // Line 84
  └─ Tests: 100, 500, 1K, 5K entries
```

### **Run Benchmark**
```bash
# All ladder benchmarks
cargo bench --bench runtime_performance -- ladder

# Specific size
cargo bench --bench runtime_performance -- ladder_1000
```

### **Run Unit Tests**
```bash
# Test ladder index
cargo test --lib ladder_index

# Test similarity detection
cargo test --lib ladder -- --nocapture
```

### **Expected Performance**
- **Re-ranking**: <1ms for 1,000 entries
- **Similarity test**: <50ns per comparison

---

## 🎯 **Component #4: Intersection Analyzer**

### **Source Component**
```
src/runtime/intersection_analysis.rs
  └─ struct IntersectionAnalyzer
      ├─ detect_intersections()  [COMPUTE HEAVY]
      ├─ calculate_strength()
      └─ stats()
```

### **Benchmark**
```rust
// benches/runtime_performance.rs
fn bench_intersection_detection()  // Line 119
  └─ Tests: 10, 50, 100 nodes
```

### **Run Benchmark**
```bash
# All intersection benchmarks
cargo bench --bench runtime_performance -- intersection

# Specific size
cargo bench --bench runtime_performance -- intersection_100
```

### **Run Unit Tests**
```bash
# Test intersection detection
cargo test --lib intersection_analysis

# Integration test
cargo test --test '*' intersection
```

### **Expected Performance**
- **Detection**: <10ms for 100 nodes
- **Scaling**: O(n²) worst case

---

## 🎯 **Component #5: Pattern Traversal**

### **Source Component**
```
src/runtime/pattern_engine.rs
  └─ struct VortexPattern
      ├─ sacred_doubling_sequence()
      ├─ linear_sequence()
      └─ calculate_anchor_proximity()
```

### **Benchmark**
```rust
// benches/runtime_performance.rs
fn bench_pattern_traversal()        // Sacred doubling
fn bench_sacred_anchor_proximity()  // Distance calc
```

### **Run Benchmark**
```bash
# All pattern benchmarks
cargo bench --bench runtime_performance -- pattern
cargo bench --bench runtime_performance -- anchor
```

### **Run Unit Tests**
```bash
# Test pattern engine
cargo test --lib pattern_engine

# Test sacred sequences
cargo test --lib vortex_pattern -- --nocapture
```

### **Expected Performance**
- **Sacred doubling**: Faster than linear
- **Anchor proximity**: <50ns per calculation

---

## 🎯 **Component #6: Lock-Free Structures**

### **Source Component**
```
src/lock_free_flux.rs
  └─ struct LockFreeFluxMatrix
      ├─ add_node()
      ├─ get_node()
      ├─ update_node()
      └─ concurrent access [THREAD SAFE]
```

### **Benchmark**
```rust
// benches/lock_free_performance.rs
fn bench_concurrent_reads()
fn bench_concurrent_writes()
fn bench_mixed_operations()
```

### **Run Benchmark**
```bash
# All lock-free benchmarks
cargo bench --bench lock_free_performance

# Specific operation type
cargo bench --bench lock_free_performance -- concurrent_reads
```

### **Run Unit Tests**
```bash
# Test lock-free operations
cargo test --lib lock_free_flux

# Concurrency tests
cargo test --lib lock_free -- --test-threads=4
```

### **Expected Performance**
- **Concurrent reads**: Linear scaling with cores
- **Concurrent writes**: 10-100× faster than RwLock

---

## 🎯 **Component #7: Vector Search (HNSW)**

### **Source Component**
```
src/vector_search/mod.rs
  └─ struct VectorIndex
      ├─ insert()
      ├─ search()  [HOT PATH]
      └─ build_graph()
```

### **Benchmark**
```rust
// benches/vector_search_benchmark.rs
fn bench_hnsw_insert()
fn bench_hnsw_search()
fn bench_hnsw_build()
```

### **Run Benchmark**
```bash
# All vector search benchmarks
cargo bench --bench vector_search_benchmark

# Specific operation
cargo bench --bench vector_search_benchmark -- search
```

### **Run Unit Tests**
```bash
# Test HNSW implementation
cargo test --lib vector_search

# Test search accuracy
cargo test --lib hnsw -- --nocapture
```

### **Expected Performance**
- **Search**: <1ms for 10K vectors
- **Insert**: <100μs per vector
- **Recall**: >95% @ k=10

---

## 🎯 **Component #8: Flux Matrix Engine**

### **Source Component**
```
src/flux_matrix.rs
  └─ struct FluxMatrixEngine
      ├─ reduce_digits()     [CORE MATH]
      ├─ seed_to_flux_sequence()
      ├─ create_matrix()
      └─ validate_matrix()
```

### **Benchmark**
```rust
// benchmarks/custom/geometric_reasoning_benchmark.rs
// (Add FluxMatrixEngine usage per IMPLEMENTATION_GUIDE)
```

### **Run Benchmark**
```bash
# Build geometric benchmark
cargo build --release --bin geometric_reasoning_benchmark

# Run it
.\target\release\geometric_reasoning_benchmark.exe
```

### **Run Unit Tests**
```bash
# Test flux matrix operations
cargo test --lib flux_matrix

# Test digit reduction
cargo test --lib reduce_digits -- --nocapture
```

### **Expected Performance**
- **reduce_digits()**: <10ns
- **seed_to_flux()**: <100ns for 9-step sequence

---

## 📋 **Quick Command Reference**

### **Run All Benchmarks**
```bash
# Complete benchmark suite
cargo bench

# Specific benchmark file
cargo bench --bench runtime_performance
cargo bench --bench lock_free_performance
cargo bench --bench vector_search_benchmark
```

### **Run All Tests**
```bash
# All unit tests
cargo test --lib

# All integration tests
cargo test --test '*'

# All tests with output
cargo test -- --nocapture
```

### **Test Specific Component**
```bash
# By module name
cargo test --lib <module_name>

# Examples:
cargo test --lib vortex_cycle
cargo test --lib ladder_index
cargo test --lib intersection_analysis
cargo test --lib pattern_engine
cargo test --lib lock_free_flux
cargo test --lib vector_search
cargo test --lib flux_matrix
```

### **Benchmark Specific Component**
```bash
# By benchmark function pattern
cargo bench --bench <file> -- <pattern>

# Examples:
cargo bench --bench runtime_performance -- elp
cargo bench --bench runtime_performance -- vortex
cargo bench --bench runtime_performance -- ladder
cargo bench --bench runtime_performance -- intersection
cargo bench --bench lock_free_performance -- concurrent
cargo bench --bench vector_search_benchmark -- search
```

---

## 🧪 **Integration Testing Strategy**

### **Level 1: Unit Tests** (Component-specific)
```bash
# Test individual components in isolation
cargo test --lib <component_name>
```

### **Level 2: Integration Tests** (Component interaction)
```bash
# Test how components work together
cargo test --test '*'
```

### **Level 3: Benchmarks** (Performance validation)
```bash
# Measure component performance under load
cargo bench --bench <benchmark_file>
```

### **Level 4: End-to-End Tests** (Full pipeline)
```bash
# Test complete system workflows
cargo test --test integration_full_pipeline
```

---

## 🎯 **Component Testing Matrix**

| Component | Unit Test | Bench | Integration | E2E |
|-----------|-----------|-------|-------------|-----|
| **ELP Tensor** | ✅ models.rs | ✅ runtime_perf | ✅ tensor_ops | ✅ inference |
| **Vortex Cycle** | ✅ vortex_cycle.rs | ✅ runtime_perf | ✅ propagation | ✅ realtime |
| **Ladder Index** | ✅ ladder_index.rs | ✅ runtime_perf | ✅ ranking | ✅ search |
| **Intersection** | ✅ intersection_analysis.rs | ✅ runtime_perf | ✅ detection | ✅ visualization |
| **Pattern Engine** | ✅ pattern_engine.rs | ✅ runtime_perf | ✅ traversal | ✅ flow |
| **Lock-Free Flux** | ✅ lock_free_flux.rs | ✅ lock_free_perf | ✅ concurrency | ✅ high_load |
| **Vector Search** | ✅ vector_search.rs | ✅ vector_search | ✅ similarity | ✅ retrieval |
| **Flux Matrix** | ✅ flux_matrix.rs | ⚠️ TBD | ✅ reasoning | ✅ inference |

**Legend**:
- ✅ Implemented and tested
- ⚠️ Needs implementation
- ❌ Not applicable

---

## 🔍 **Finding Component Tests**

### **Method 1: By Module Name**
```bash
# List all test functions in a module
cargo test --lib <module> -- --list

# Example:
cargo test --lib vortex_cycle -- --list
```

### **Method 2: By Pattern Matching**
```bash
# Find tests matching pattern
cargo test --lib <pattern> -- --list

# Examples:
cargo test --lib elp -- --list
cargo test --lib sacred -- --list
```

### **Method 3: Search Source Code**
```bash
# Find test modules
rg "#\[cfg\(test\)\]" src/

# Find specific test
rg "fn test_" src/
```

---

## 📊 **Component Performance Dashboard**

Create this script to test all components:

```bash
#!/bin/bash
# test_all_components.sh

echo "=== SpatialVortex Component Testing Dashboard ==="
echo ""

echo "1. ELP Tensor Operations..."
cargo test --lib elp --quiet
cargo bench --bench runtime_performance -- elp --quiet

echo "2. Vortex Cycle Engine..."
cargo test --lib vortex_cycle --quiet
cargo bench --bench runtime_performance -- vortex_100 --quiet

echo "3. Ladder Index..."
cargo test --lib ladder_index --quiet
cargo bench --bench runtime_performance -- ladder_100 --quiet

echo "4. Intersection Analyzer..."
cargo test --lib intersection_analysis --quiet
cargo bench --bench runtime_performance -- intersection_10 --quiet

echo "5. Pattern Engine..."
cargo test --lib pattern_engine --quiet
cargo bench --bench runtime_performance -- pattern --quiet

echo "6. Lock-Free Structures..."
cargo test --lib lock_free_flux --quiet
cargo bench --bench lock_free_performance --quiet

echo "7. Vector Search..."
cargo test --lib vector_search --quiet
cargo bench --bench vector_search_benchmark --quiet

echo "8. Flux Matrix Engine..."
cargo test --lib flux_matrix --quiet

echo ""
echo "=== Testing Complete ==="
```

---

## 🚀 **Quick Start Examples**

### **Test & Benchmark ELP Tensor**
```bash
# 1. Run unit tests
cargo test --lib elp

# 2. Run benchmarks
cargo bench --bench runtime_performance -- elp

# 3. View detailed results
start target/criterion/elp_distance/report/index.html
```

### **Test & Benchmark Vortex Cycle**
```bash
# 1. Run unit tests
cargo test --lib vortex_cycle -- --nocapture

# 2. Run benchmarks with different sizes
cargo bench --bench runtime_performance -- vortex

# 3. Profile specific size
cargo flamegraph --bench runtime_performance -- vortex_1000
```

### **Test & Benchmark Lock-Free**
```bash
# 1. Run concurrent tests
cargo test --lib lock_free_flux -- --test-threads=8

# 2. Benchmark concurrent operations
cargo bench --bench lock_free_performance

# 3. Compare to regular locks
cargo bench --bench lock_free_performance -- --baseline rwlock
```

---

## ✅ **Component Health Check**

Run this to verify all components:

```bash
# Quick health check
cargo test --lib --quiet && \
cargo bench --no-run && \
echo "✅ All components healthy!"
```

---

## 📝 **Summary**

### **Test Individual Component**
```bash
cargo test --lib <component_name>
```

### **Benchmark Individual Component**
```bash
cargo bench --bench <file> -- <component_pattern>
```

### **Full Component Validation**
```bash
# Test + Benchmark + Profile
cargo test --lib <component>
cargo bench --bench <file> -- <component>
cargo flamegraph --bench <file> -- <component>_test
```

---

**Now you can test each component independently!** 🎯✨
