/// Performance benchmarks for Lock-Free Flux Matrix
/// Target: <100 nanosecond access time

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use spatial_vortex::lock_free_flux::LockFreeFluxMatrix;
use spatial_vortex::models::*;
use std::collections::HashMap;

fn create_test_node(position: u8) -> FluxNode {
    let mut parameters = HashMap::new();
    parameters.insert("ethos".to_string(), 0.8);
    
    FluxNode {
        position,
        base_value: position,
        semantic_index: SemanticIndex {
            positive_associations: vec![],
            negative_associations: vec![],
            neutral_base: format!("Position {}", position),
            predicates: vec![],
            relations: vec![],
        },
        attributes: NodeAttributes {
            properties: HashMap::new(),
            parameters,
            state: NodeState {
                active: true,
                last_accessed: chrono::Utc::now(),
                usage_count: 0,
                context_stack: vec![],
            },
            dynamics: NodeDynamics {
                evolution_rate: 1.0,
                stability_index: 1.0,
                interaction_patterns: vec![],
                learning_adjustments: vec![],
            },
        },
        connections: vec![],
    }
}

fn bench_insert(c: &mut Criterion) {
    let matrix = LockFreeFluxMatrix::new("benchmark".to_string());
    
    c.bench_function("lock_free_insert", |b| {
        let mut position = 0u8;
        b.iter(|| {
            let node = create_test_node(position);
            matrix.insert(black_box(node));
            position = (position + 1) % 9;
        });
    });
}

fn bench_get(c: &mut Criterion) {
    let matrix = LockFreeFluxMatrix::new("benchmark".to_string());
    
    // Pre-populate
    for pos in 0..9 {
        matrix.insert(create_test_node(pos));
    }
    
    c.bench_function("lock_free_get", |b| {
        let mut position = 0u8;
        b.iter(|| {
            let result = matrix.get(black_box(position));
            position = (position + 1) % 9;
            result
        });
    });
}

fn bench_attribute_query(c: &mut Criterion) {
    let matrix = LockFreeFluxMatrix::new("benchmark".to_string());
    
    // Pre-populate with 100 nodes
    for i in 0..100 {
        let mut node = create_test_node((i % 9) as u8);
        node.attributes.parameters.insert("ethos".to_string(), (i as f64) / 100.0);
        matrix.insert(node);
    }
    
    c.bench_function("attribute_query", |b| {
        b.iter(|| {
            matrix.get_by_attribute(black_box("ethos"), black_box(0.8), black_box(1.0))
        });
    });
}

fn bench_sacred_anchor(c: &mut Criterion) {
    let matrix = LockFreeFluxMatrix::new("benchmark".to_string());
    
    c.bench_function("sacred_anchor_access", |b| {
        b.iter(|| {
            matrix.get_sacred_anchor(black_box(3))
        });
    });
}

fn bench_snapshot(c: &mut Criterion) {
    let matrix = LockFreeFluxMatrix::new("benchmark".to_string());
    
    // Pre-populate
    for pos in 0..9 {
        matrix.insert(create_test_node(pos));
    }
    
    c.bench_function("snapshot_creation", |b| {
        b.iter(|| {
            matrix.snapshot()
        });
    });
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let matrix = LockFreeFluxMatrix::new("throughput".to_string());
            
            b.iter(|| {
                for i in 0..size {
                    let node = create_test_node((i % 9) as u8);
                    matrix.insert(black_box(node));
                }
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_insert,
    bench_get,
    bench_attribute_query,
    bench_sacred_anchor,
    bench_snapshot,
    bench_throughput
);
criterion_main!(benches);
