//! Runtime Performance Benchmarks
//! 
//! Comprehensive benchmarking suite for critical paths:
//! - Vortex cycle propagation
//! - Ladder index ranking
//! - Intersection detection
//! - ELP tensor operations
//! - Pattern traversal

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use spatial_vortex::models::ELPTensor;
use spatial_vortex::runtime::{
    VortexCycleEngine, CycleObject, CycleDirection,
    LadderIndex,
    IntersectionAnalyzer,
    VortexPattern,
};
use std::time::Duration;

/// Benchmark ELP tensor distance calculations (HOT PATH)
fn bench_elp_distance(c: &mut Criterion) {
    let tensor_a = ELPTensor::new(7.5, 3.2, 9.1);
    let tensor_b = ELPTensor::new(4.1, 8.7, 2.3);
    
    c.bench_function("elp_distance", |b| {
        b.iter(|| {
            black_box(tensor_a.distance(black_box(&tensor_b)))
        });
    });
}

/// Benchmark ELP tensor magnitude (HOT PATH)
fn bench_elp_magnitude(c: &mut Criterion) {
    let tensor = ELPTensor::new(7.5, 3.2, 9.1);
    
    c.bench_function("elp_magnitude", |b| {
        b.iter(|| {
            black_box(tensor.magnitude())
        });
    });
}

/// Benchmark vortex cycle with varying object counts
fn bench_vortex_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("vortex_cycle");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [10, 100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            b.iter(|| {
                rt.block_on(async {
                    let engine = VortexCycleEngine::new(60.0);
                    
                    // Add objects
                    for i in 0..size {
                        let obj = CycleObject::new(
                            format!("obj_{}", i),
                            ELPTensor::new(
                                (i % 13) as f64,
                                ((i * 2) % 13) as f64,
                                ((i * 3) % 13) as f64,
                            ),
                            CycleDirection::Forward,
                        );
                        engine.add_object(obj).await;
                    }
                    
                    // Start and run for one tick
                    engine.start().await.unwrap();
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    engine.stop().await;
                    
                    black_box(engine.get_objects().await.len())
                })
            });
        });
    }
    
    group.finish();
}

/// Benchmark ladder ranking with varying entry counts
fn bench_ladder_ranking(c: &mut Criterion) {
    let mut group = c.benchmark_group("ladder_ranking");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            b.iter(|| {
                rt.block_on(async {
                    let ladder = LadderIndex::new(0.1, 10);
                    
                    // Add entries
                    for i in 0..size {
                        ladder.add_entry(
                            (i % 10) as u8,
                            ELPTensor::new(
                                (i % 13) as f64,
                                ((i * 2) % 13) as f64,
                                ((i * 3) % 13) as f64,
                            ),
                        ).await;
                    }
                    
                    // Benchmark get_ranked_entries
                    black_box(ladder.get_ranked_entries().await.len())
                })
            });
        });
    }
    
    group.finish();
}

/// Benchmark intersection detection
fn bench_intersection_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("intersection_detection");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            b.iter(|| {
                rt.block_on(async {
                use spatial_vortex::models::{FluxNode, NodeAttributes, NodeState, NodeDynamics, SemanticIndex};
                use std::collections::HashMap;
                
                let analyzer = IntersectionAnalyzer::new(0.5);
                
                // Create nodes
                let mut nodes = HashMap::new();
                for i in 0..size {
                    let mut params = HashMap::new();
                    params.insert("ethos".to_string(), (i % 13) as f64);
                    params.insert("logos".to_string(), ((i * 2) % 13) as f64);
                    params.insert("pathos".to_string(), ((i * 3) % 13) as f64);
                    
                    let node = FluxNode {
                        position: (i % 10) as u8,
                        base_value: (i % 10) as u8,
                        semantic_index: SemanticIndex {
                            positive_associations: vec![],
                            negative_associations: vec![],
                            neutral_base: format!("Node_{}", i),
                            predicates: vec![],
                            relations: vec![],
                        },
                        attributes: NodeAttributes {
                            properties: HashMap::new(),
                            parameters: params,
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
                    };
                    
                    nodes.insert(format!("node_{}", i), node);
                }
                
                // Benchmark intersection detection
                analyzer.detect_intersections(&nodes).await;
                black_box(analyzer.stats().await.total_intersections)
                })
            });
        });
    }
    
    group.finish();
}

/// Benchmark pattern traversal
fn bench_pattern_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_traversal");
    
    let patterns = vec![
        VortexPattern::sacred_doubling(),
        VortexPattern::linear_ascending(),
    ];
    
    for pattern in patterns {
        group.bench_with_input(
            BenchmarkId::new("traverse", pattern.name.clone()),
            &pattern,
            |b, pattern| {
                b.iter(|| {
                    let mut pos_idx = 0;
                    for _ in 0..1000 {
                        let (next_pos, next_idx) = pattern.next_position(pos_idx);
                        pos_idx = next_idx;
                        black_box((next_pos, next_idx));
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark sacred anchor proximity calculations
fn bench_anchor_proximity(c: &mut Criterion) {
    c.bench_function("anchor_proximity", |b| {
        b.iter(|| {
            for pos in 0..10 {
                let proximity = [3u8, 6, 9].iter()
                    .map(|&anchor| {
                        let diff = (anchor as i32 - pos as i32).abs();
                        diff.min(10 - diff) as f64
                    })
                    .fold(f64::INFINITY, f64::min);
                black_box(1.0 - (proximity / 5.0));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_elp_distance,
    bench_elp_magnitude,
    bench_vortex_cycle,
    bench_ladder_ranking,
    bench_intersection_detection,
    bench_pattern_traversal,
    bench_anchor_proximity,
);

criterion_main!(benches);
