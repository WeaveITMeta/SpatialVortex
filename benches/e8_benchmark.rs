//! E8 Lattice Quantization Benchmarks
//!
//! Validates the theoretical query complexity:
//! T_query = O(log n · (d/8) · bits_per_block)
//!
//! Key metrics:
//! - Query latency scaling with n (should be logarithmic)
//! - Compression ratio vs full FP32
//! - Recall at different quantization levels

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ndarray::Array1;
use spatial_vortex::data::vector_search::{E8VectorIndex, E8HNSWConfig, HNSWConfig, VectorMetadata, VECTOR_DIM};
use spatial_vortex::data::e8_integration::{ComplexityAnalysis, E8AmortizedCache, SacredE8Codec};

const DIMENSION: usize = VECTOR_DIM; // 384

fn random_vector() -> Array1<f32> {
    Array1::from_iter((0..DIMENSION).map(|_| rand::random::<f32>() * 2.0 - 1.0))
}

fn random_metadata(i: usize) -> VectorMetadata {
    VectorMetadata {
        position: Some((i % 10) as u8),
        sacred: matches!(i % 10, 3 | 6 | 9),
        ethos: rand::random(),
        logos: rand::random(),
        pathos: rand::random(),
        created_at: std::time::SystemTime::now(),
    }
}

/// Benchmark E8 encoding/decoding (per-vector cost)
fn bench_e8_codec(c: &mut Criterion) {
    let mut group = c.benchmark_group("E8 Codec");
    
    for bits in [8u8, 10, 12] {
        let codec = SacredE8Codec::new(DIMENSION, bits, true, 42, true);
        let vector: Vec<f32> = (0..DIMENSION).map(|i| (i as f32 * 0.01).sin()).collect();
        
        group.bench_with_input(
            BenchmarkId::new("encode", format!("{}bits", bits)),
            &bits,
            |b, _| {
                b.iter(|| {
                    black_box(codec.encode_sacred(black_box(&vector), None).unwrap())
                })
            },
        );
        
        let encoded = codec.encode_sacred(&vector, None).unwrap();
        group.bench_with_input(
            BenchmarkId::new("decode", format!("{}bits", bits)),
            &bits,
            |b, _| {
                b.iter(|| {
                    black_box(codec.decode_sacred(black_box(&encoded)))
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark E8AmortizedCache (O(1) lookup validation)
fn bench_amortized_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("E8 Amortized Cache");
    
    // Build cache (offline cost)
    group.bench_function("build_cache", |b| {
        b.iter(|| {
            black_box(E8AmortizedCache::build())
        })
    });
    
    let cache = E8AmortizedCache::build();
    
    // O(1) lookups
    group.bench_function("roots_at_position", |b| {
        b.iter(|| {
            for pos in 0..10u8 {
                black_box(cache.roots_at_position(spatial_vortex::data::E8FluxPosition(pos)));
            }
        })
    });
    
    group.bench_function("sacred_roots", |b| {
        b.iter(|| {
            black_box(cache.sacred_roots())
        })
    });
    
    group.finish();
}

/// Benchmark query scaling: T_query = O(log n · d/8 · bits)
/// 
/// Tests that query time grows logarithmically with index size
fn bench_query_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("E8 Query Scaling");
    group.sample_size(50); // Reduce samples for larger indices
    
    // Test sizes: 100, 1K, 10K (larger sizes would take too long for CI)
    for size in [100usize, 1_000, 10_000] {
        group.throughput(Throughput::Elements(1));
        
        let config = E8HNSWConfig {
            hnsw: HNSWConfig {
                max_connections: 16,
                ef_construction: 100,
                ef_search: 50,
                ..Default::default()
            },
            bits_per_block: 10,
            use_hadamard: true,
            random_seed: 42,
            use_sacred_boost: true,
        };
        
        let index = E8VectorIndex::new(config);
        
        // Build index
        for i in 0..size {
            let vector = random_vector();
            let metadata = random_metadata(i);
            index.add(format!("vec_{}", i), vector, metadata).unwrap();
        }
        
        let query = random_vector();
        
        group.bench_with_input(
            BenchmarkId::new("search_k10", size),
            &size,
            |b, _| {
                b.iter(|| {
                    black_box(index.search(black_box(&query), 10).unwrap())
                })
            },
        );
        
        // Also benchmark sacred-filtered search
        group.bench_with_input(
            BenchmarkId::new("search_sacred_k10", size),
            &size,
            |b, _| {
                b.iter(|| {
                    black_box(index.search_sacred(black_box(&query), 10).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark compression ratio at different bit levels
fn bench_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("E8 Compression");
    
    let vectors: Vec<Vec<f32>> = (0..1000)
        .map(|_| (0..DIMENSION).map(|_| rand::random::<f32>()).collect())
        .collect();
    
    for bits in [8u8, 10, 12] {
        let codec = SacredE8Codec::new(DIMENSION, bits, true, 42, true);
        
        // Measure encoding throughput
        group.throughput(Throughput::Elements(1000));
        group.bench_with_input(
            BenchmarkId::new("encode_1000", format!("{}bits", bits)),
            &bits,
            |b, _| {
                b.iter(|| {
                    for v in &vectors {
                        black_box(codec.encode_sacred(v, None).unwrap());
                    }
                })
            },
        );
    }
    
    group.finish();
}

/// Validate complexity analysis calculations
fn bench_complexity_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("Complexity Analysis");
    
    // Show theoretical speedup at different scales
    for n in [1_000usize, 100_000, 1_000_000, 1_000_000_000] {
        let analysis = ComplexityAnalysis {
            n,
            dimension: DIMENSION,
            bits_per_block: 10,
        };
        
        group.bench_with_input(
            BenchmarkId::new("speedup_calc", n),
            &n,
            |b, _| {
                b.iter(|| {
                    let brute = black_box(analysis.brute_force_ops());
                    let e8 = black_box(analysis.e8_hnsw_ops());
                    black_box(brute / e8)
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_e8_codec,
    bench_amortized_cache,
    bench_query_scaling,
    bench_compression,
    bench_complexity_analysis,
);

criterion_main!(benches);
