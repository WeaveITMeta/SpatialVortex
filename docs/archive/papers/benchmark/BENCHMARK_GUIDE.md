# SpatialVortex Benchmark Guide

**Last Updated**: 2025-01-25  
**Purpose**: Measure performance and optimize hot paths

---

## 🎯 **Quick Start**

```bash
# Build in release mode first
cargo build --release

# Run all benchmarks
cargo bench --bench runtime_performance

# View HTML report
start target/criterion/report/index.html
```

---

## 📊 **Available Benchmarks**

From `benches/runtime_performance.rs` (238 lines):

### **1. ELP Tensor Operations**
- `tensor_distance` - Calculate distance between two ELP tensors
- `tensor_magnitude` - Calculate tensor magnitude
- **Expected**: <100ns per operation

### **2. Vortex Cycle Throughput**
- `vortex_10_objects` - 10 objects through cycle
- `vortex_100_objects` - 100 objects
- `vortex_1000_objects` - 1,000 objects
- `vortex_5000_objects` - 5,000 objects
- **Target**: 10,000+ objects/second

### **3. Ladder Ranking Speed**
- `ladder_100_entries` - Rank 100 entries
- `ladder_1000_entries` - 1,000 entries
- `ladder_5000_entries` - 5,000 entries
- **Target**: <1ms for re-ranking

### **4. Intersection Detection**
- `intersection_10_nodes` - 10 nodes
- `intersection_50_nodes` - 50 nodes
- `intersection_100_nodes` - 100 nodes
- **Target**: <10ms

### **5. Pattern Traversal**
- `sacred_doubling_traversal` - Sacred doubling pattern
- `linear_traversal` - Linear pattern
- **Expected**: Doubling faster than linear

### **6. Sacred Anchor Proximity**
- `anchor_proximity` - Distance to sacred positions (3, 6, 9)
- **Expected**: <50ns per calculation

---

## 🚀 **Command Reference**

### **Basic Benchmarking**

```bash
# All benchmarks
cargo bench --bench runtime_performance

# Specific group
cargo bench --bench runtime_performance -- elp
cargo bench --bench runtime_performance -- vortex
cargo bench --bench runtime_performance -- ladder
cargo bench --bench runtime_performance -- intersection
cargo bench --bench runtime_performance -- pattern
cargo bench --bench runtime_performance -- anchor

# Single benchmark
cargo bench --bench runtime_performance -- tensor_distance
```

### **Baseline Comparison**

```bash
# Save current performance
cargo bench --bench runtime_performance -- --save-baseline current

# Make optimizations...

# Compare against baseline
cargo bench --bench runtime_performance -- --baseline current

# This will show % improvement/regression
```

### **Profiling with Flamegraph**

```bash
# Install (one-time)
cargo install flamegraph

# Profile benchmarks (generates flamegraph.svg)
cargo flamegraph --bench runtime_performance

# Profile specific benchmark
cargo flamegraph --bench runtime_performance -- vortex_1000_objects

# View flamegraph
start flamegraph.svg
```

### **Custom Measurement Time**

```bash
# Longer measurement for more accurate results
cargo bench --bench runtime_performance -- --measurement-time 10

# Shorter for quick checks
cargo bench --bench runtime_performance -- --measurement-time 3
```

---

## 📈 **Performance Targets**

Based on `docs/analysis/ITERATION_2_CRITICAL_IMPROVEMENTS.md`:

| Metric | Target | Stretch Goal |
|--------|--------|--------------|
| Vortex Throughput | 10,000 objects/s | 50,000 objects/s |
| Intersection Detection | <10ms | <5ms |
| Ladder Re-ranking | <1ms | <500μs |
| ELP Distance | <100ns | <50ns |
| Visualization FPS | 60 FPS @ 1,000 objects | 60 FPS @ 10,000 objects |

---

## 🔧 **Optimization Strategy**

### **Phase 1: Establish Baseline**
```bash
cargo bench --bench runtime_performance -- --save-baseline phase1_baseline
```

### **Phase 2: Implement Lock-Free Structures**

Replace in `src/runtime/`:
- `Arc<RwLock<Vec<T>>>` → `DashMap<K, T>`
- Regular queues → `crossbeam::SegQueue`

```bash
cargo bench --bench runtime_performance -- --baseline phase1_baseline
```

**Expected improvement**: 10-100× for concurrent operations

### **Phase 3: Add `#[inline]` to Hot Paths**

Already done for:
- ✅ `ELPTensor::distance()`
- ✅ `ELPTensor::magnitude()`
- ✅ `calculate_anchor_proximity()`
- ✅ `VortexCycleEngine::sequence()`

```bash
cargo bench --bench runtime_performance -- --baseline phase1_baseline
```

**Expected improvement**: 5-20% for small functions

### **Phase 4: SIMD Operations (if needed)**

If tensor operations are bottleneck:
```rust
use std::simd::f32x4;

#[inline]
pub fn distance_simd(&self, other: &ELPTensor) -> f64 {
    let a = f32x4::from_array([self.ethos, self.logos, self.pathos, 0.0]);
    let b = f32x4::from_array([other.ethos, other.logos, other.pathos, 0.0]);
    let diff = a - b;
    (diff * diff).reduce_sum().sqrt() as f64
}
```

---

## 📊 **Reading Criterion Output**

```
tensor_distance         time:   [45.123 ns 45.456 ns 45.789 ns]
                        change: [-5.2% -4.8% -4.4%] (p = 0.00 < 0.05)
                        Performance has improved.
```

**Meaning**:
- **time**: [lower_bound median upper_bound]
- **change**: % difference from baseline
- **p-value**: Statistical significance (< 0.05 = significant)

---

## 🐛 **Troubleshooting**

### **Build Errors**
```bash
# Clean and rebuild
cargo clean
cargo build --release
cargo bench
```

### **Benchmarks Too Slow**
```bash
# Reduce sample size
cargo bench -- --sample-size 10

# Or measurement time
cargo bench -- --measurement-time 1
```

### **Missing Criterion Reports**
```bash
# Ensure criterion is in Cargo.toml [dev-dependencies]
cargo add criterion --dev
```

---

## 💡 **Tips**

1. **Always run in release mode** - `cargo bench` does this automatically
2. **Close other programs** - Reduces system noise
3. **Run multiple times** - First run may be slower (cold cache)
4. **Use baselines** - Essential for tracking improvements
5. **Profile hot paths** - Flamegraph shows where time is spent
6. **Iterate quickly** - Short benchmarks for rapid testing

---

## 📁 **Output Files**

```
target/
├── criterion/
│   ├── report/
│   │   └── index.html          # Main benchmark report
│   ├── tensor_distance/
│   │   ├── report/index.html   # Individual benchmark
│   │   └── base/
│   │       └── estimates.json  # Baseline data
│   └── vortex_1000_objects/
│       └── ...
├── release/
│   └── deps/
│       └── runtime_performance-*  # Compiled benchmark binary
└── flamegraph.svg               # Profile visualization (if using flamegraph)
```

---

## 🎯 **Next Steps After Benchmarking**

1. **Identify bottlenecks** from flamegraph
2. **Optimize hot paths** (functions taking >10% time)
3. **Re-benchmark** to verify improvements
4. **Document results** in `docs/benchmarks/`
5. **Set performance regression tests** in CI/CD

---

## 📝 **Example Workflow**

```bash
# 1. Establish baseline
cargo bench --bench runtime_performance -- --save-baseline before_lockfree

# 2. Implement DashMap in vortex_cycle.rs
# ... make changes ...

# 3. Measure improvement
cargo bench --bench runtime_performance -- vortex --baseline before_lockfree

# 4. Profile if not meeting targets
cargo flamegraph --bench runtime_performance -- vortex_5000_objects

# 5. Optimize identified hot paths
# ... fix bottlenecks ...

# 6. Final verification
cargo bench --bench runtime_performance -- --baseline before_lockfree
```

---

## 🏆 **Success Criteria**

Benchmarks are successful when:
- ✅ All targets met or exceeded
- ✅ No performance regressions
- ✅ Flamegraph shows flat profile (no single hot path >20%)
- ✅ Scales linearly with input size
- ✅ Results are reproducible (low variance)

---

**Ready to optimize!** Run `cargo bench` to establish your baseline. 🚀
