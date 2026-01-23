use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use spatial_vortex::ai::orchestrator::{ASIOrchestrator, ExecutionMode};
use tokio::runtime::Runtime;

fn bench_asi_modes(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("ASI Execution Modes");
    
    let inputs = vec![
        "Short input",
        "Medium length input with some context",
        "Long input with extensive context and detail that requires thorough analysis and processing to extract meaningful insights",
    ];
    
    for input in inputs.iter() {
        // Fast mode
        group.benchmark_with_input(
            BenchmarkId::new("Fast", input.len()),
            input,
            |b, i| {
                b.to_async(&rt).iter(|| async {
                    let mut asi = ASIOrchestrator::new().unwrap();
                    asi.process(black_box(i), ExecutionMode::Fast).await.unwrap()
                });
            },
        );
        
        // Balanced mode
        group.benchmark_with_input(
            BenchmarkId::new("Balanced", input.len()),
            input,
            |b, i| {
                b.to_async(&rt).iter(|| async {
                    let mut asi = ASIOrchestrator::new().unwrap();
                    asi.process(black_box(i), ExecutionMode::Balanced).await.unwrap()
                });
            },
        );
        
        // Thorough mode
        group.benchmark_with_input(
            BenchmarkId::new("Thorough", input.len()),
            input,
            |b, i| {
                b.to_async(&rt).iter(|| async {
                    let mut asi = ASIOrchestrator::new().unwrap();
                    asi.process(black_box(i), ExecutionMode::Thorough).await.unwrap()
                });
            },
        );
    }
    
    group.finish();
}

fn bench_sacred_positions(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("Sacred Position Detection");
    
    let inputs = vec![
        ("Position 3", "Creative question?"),
        ("Position 6", "Logical systematic reasoning with balance"),
        ("Position 9", "Perfect divine completion with emotion!"),
    ];
    
    for (name, input) in inputs.iter() {
        group.benchmark_with_input(
            BenchmarkId::new("Sacred", name),
            input,
            |b, i| {
                b.to_async(&rt).iter(|| async {
                    let mut asi = ASIOrchestrator::new().unwrap();
                    asi.process(black_box(i), ExecutionMode::Balanced).await.unwrap()
                });
            },
        );
    }
    
    group.finish();
}

fn bench_parallel_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("parallel_geometric_ml", |b| {
        b.to_async(&rt).iter(|| async {
            let mut asi = ASIOrchestrator::new().unwrap();
            asi.process(
                black_box("Test input for parallel execution benchmark"),
                ExecutionMode::Balanced
            ).await.unwrap()
        });
    });
}

fn bench_adaptive_learning(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("adaptive_weight_update", |b| {
        b.to_async(&rt).iter(|| async {
            let mut asi = ASIOrchestrator::new().unwrap();
            // Trigger weight update with high-confidence result
            asi.process(
                black_box("High quality input for adaptive learning benchmark"),
                ExecutionMode::Thorough
            ).await.unwrap()
        });
    });
}

criterion_group!(
    benches,
    bench_asi_modes,
    bench_sacred_positions,
    bench_parallel_execution,
    bench_adaptive_learning
);
criterion_main!(benches);
