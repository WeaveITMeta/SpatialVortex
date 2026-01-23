/// Vector Search Performance Benchmark
/// 
/// Benchmarks:
/// 1. Index construction (10K, 100K, 1M vectors)
/// 2. Search latency (k=1, k=10, k=100)
/// 3. Search with filters (position, ELP)
/// 4. Concurrent search throughput
/// 
/// Target: <10ms search time @ 1M vectors

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use spatial_vortex::vector_search::{VectorIndex, VectorMetadata, VECTOR_DIM};
use ndarray::Array1;
use std::sync::Arc;

fn random_vector() -> Array1<f32> {
    Array1::from_iter((0..VECTOR_DIM).map(|_| rand::random::<f32>()))
}

fn random_metadata(position: u8) -> VectorMetadata {
    VectorMetadata {
        position: Some(position),
        sacred: [3, 6, 9].contains(&position),
        ethos: rand::random::<f32>(),
        logos: rand::random::<f32>(),
        pathos: rand::random::<f32>(),
        created_at: std::time::SystemTime::now(),
    }
}

fn bench_index_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_construction");
    
    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let index = VectorIndex::new_default();
                for i in 0..size {
                    let vector = random_vector();
                    let metadata = random_metadata((i % 10) as u8);
                    index.add(format!("vec_{}", i), vector, metadata).unwrap();
                }
                index
            });
        });
    }
    
    group.finish();
}

fn bench_search_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_latency");
    
    // Build index with 10K vectors
    let index = VectorIndex::new_default();
    for i in 0..10000 {
        let vector = random_vector();
        let metadata = random_metadata((i % 10) as u8);
        index.add(format!("vec_{}", i), vector, metadata).unwrap();
    }
    
    for k in [1, 10, 100].iter() {
        group.bench_with_input(BenchmarkId::new("10k_vectors", k), k, |b, &k| {
            let query = random_vector();
            b.iter(|| {
                let results = index.search(black_box(&query), black_box(k)).unwrap();
                black_box(results)
            });
        });
    }
    
    group.finish();
}

fn bench_filtered_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtered_search");
    
    // Build index with 10K vectors
    let index = VectorIndex::new_default();
    for i in 0..10000 {
        let vector = random_vector();
        let metadata = random_metadata((i % 10) as u8);
        index.add(format!("vec_{}", i), vector, metadata).unwrap();
    }
    
    let query = random_vector();
    
    group.bench_function("position_filter", |b| {
        b.iter(|| {
            let results = index.search_by_position(black_box(&query), black_box(10), black_box(3)).unwrap();
            black_box(results)
        });
    });
    
    group.bench_function("elp_filter", |b| {
        b.iter(|| {
            let results = index.search_by_elp(black_box(&query), black_box(10), black_box(0.5)).unwrap();
            black_box(results)
        });
    });
    
    group.finish();
}

fn bench_concurrent_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_search");
    
    // Build index with 10K vectors
    let index = Arc::new(VectorIndex::new_default());
    for i in 0..10000 {
        let vector = random_vector();
        let metadata = random_metadata((i % 10) as u8);
        index.add(format!("vec_{}", i), vector, metadata).unwrap();
    }
    
    group.bench_function("4_threads_parallel", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            runtime.block_on(async {
                let mut handles = Vec::new();
                for _ in 0..4 {
                    let index_clone = Arc::clone(&index);
                    let handle = tokio::spawn(async move {
                        let query = random_vector();
                        index_clone.search(&query, 10).unwrap()
                    });
                    handles.push(handle);
                }
                
                for handle in handles {
                    handle.await.unwrap();
                }
            });
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_index_construction,
    bench_search_latency,
    bench_filtered_search,
    bench_concurrent_search
);
criterion_main!(benches);
