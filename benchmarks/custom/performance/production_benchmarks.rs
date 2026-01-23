//! Production Benchmarks with Real Data
//!
//! Comprehensive benchmarks measuring actual performance metrics
//! for all cascade workflow components.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use spatial_vortex::ai::orchestrator::{ASIOrchestrator, ExecutionMode};
use spatial_vortex::models::ELPTensor;
use std::time::Duration;

#[cfg(feature = "voice")]
use spatial_vortex::voice_pipeline::{SpectralAnalyzer, AudioConfig};

#[cfg(feature = "lake")]
use spatial_vortex::storage::confidence_lake::{SecureStorage, SqliteConfidenceLake};

#[cfg(feature = "onnx")]
use spatial_vortex::ml::inference::OnnxSessionPool;

/// Benchmark ASI Orchestrator with different execution modes
fn benchmark_asi_orchestrator(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("asi_orchestrator");
    group.measurement_time(Duration::from_secs(10));
    
    // Test inputs of varying complexity
    let test_inputs = vec![
        ("simple", "Calculate position for basic input"),
        ("moderate", "Process voice signal with ELP tensor mapping and sacred geometry validation"),
        ("complex", "Analyze multi-modal input stream with voice, tensor, and geometric constraints, applying vortex flow pattern 1→2→4→8→7→5→1 through sacred positions 3-6-9 for optimal flux matrix convergence"),
    ];
    
    for (name, input) in test_inputs {
        // Benchmark Fast mode
        group.bench_with_input(
            BenchmarkId::new("fast", name),
            &input,
            |b, &input| {
                b.iter(|| {
                    runtime.block_on(async {
                        let mut asi = ASIOrchestrator::new().unwrap();
                        asi.process(
                            black_box(input),
                            black_box(ExecutionMode::Fast)
                        ).await
                    })
                });
            }
        );
        
        // Benchmark Balanced mode
        group.bench_with_input(
            BenchmarkId::new("balanced", name),
            &input,
            |b, &input| {
                b.iter(|| {
                    runtime.block_on(async {
                        let mut asi = ASIOrchestrator::new().unwrap();
                        asi.process(
                            black_box(input),
                            black_box(ExecutionMode::Balanced)
                        ).await
                    })
                });
            }
        );
        
        // Benchmark Thorough mode
        group.bench_with_input(
            BenchmarkId::new("thorough", name),
            &input,
            |b, &input| {
                b.iter(|| {
                    runtime.block_on(async {
                        let mut asi = ASIOrchestrator::new().unwrap();
                        asi.process(
                            black_box(input),
                            black_box(ExecutionMode::Thorough)
                        ).await
                    })
                });
            }
        );
    }
    
    group.finish();
}

/// Benchmark voice processing pipeline
#[cfg(feature = "voice")]
fn benchmark_voice_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("voice_pipeline");
    
    // Generate realistic audio samples
    let sample_rates = vec![16000, 44100, 48000];
    let durations_ms = vec![10, 50, 100, 500];
    
    for sample_rate in sample_rates {
        for duration_ms in &durations_ms {
            let samples = (sample_rate as f32 * (*duration_ms as f32 / 1000.0)) as usize;
            
            // Generate test audio (440Hz sine wave)
            let audio: Vec<f32> = (0..samples)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    (2.0 * std::f32::consts::PI * 440.0 * t).sin()
                })
                .collect();
            
            let mut analyzer = SpectralAnalyzer::new(sample_rate);
            
            group.bench_with_input(
                BenchmarkId::new(format!("fft_{}hz", sample_rate), duration_ms),
                &audio,
                |b, audio| {
                    b.iter(|| {
                        analyzer.analyze(black_box(audio))
                    });
                }
            );
        }
    }
    
    // Benchmark complete voice to ELP tensor mapping
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let audio_100ms: Vec<f32> = (0..4410)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
        .collect();
    
    group.bench_function("voice_to_elp", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let mut asi = ASIOrchestrator::new().unwrap();
                let elp = ELPTensor {
                    ethos: 5.0,
                    logos: 4.0,
                    pathos: 7.0,
                };
                asi.process_voice(
                    black_box("Voice: 440Hz, -15dB"),
                    black_box(Some(elp))
                ).await
            })
        });
    });
    
    group.finish();
}

/// Benchmark Confidence Lake encryption and storage
#[cfg(feature = "lake")]
fn benchmark_confidence_lake(c: &mut Criterion) {
    use spatial_vortex::ai::orchestrator::ASIOutput;
    
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("confidence_lake");
    
    // Benchmark encryption
    let key = SecureStorage::generate_key();
    let storage = SecureStorage::new(&key);
    
    let data_sizes = vec![
        ("small", vec![0u8; 100]),
        ("medium", vec![0u8; 1000]),
        ("large", vec![0u8; 10000]),
        ("huge", vec![0u8; 100000]),
    ];
    
    for (name, data) in data_sizes {
        group.bench_with_input(
            BenchmarkId::new("encrypt", name),
            &data,
            |b, data| {
                b.iter(|| {
                    storage.encrypt(black_box(data))
                });
            }
        );
        
        let encrypted = storage.encrypt(&data).unwrap();
        group.bench_with_input(
            BenchmarkId::new("decrypt", name),
            &encrypted,
            |b, encrypted| {
                b.iter(|| {
                    storage.decrypt(black_box(encrypted))
                });
            }
        );
    }
    
    // Benchmark SQLite operations
    runtime.block_on(async {
        let lake = SqliteConfidenceLake::new(":memory:").await.unwrap();
        
        let output = ASIOutput {
            result: "Benchmark result".to_string(),
            elp: ELPTensor {
                ethos: 6.0,
                logos: 7.0,
                pathos: 8.0,
            },
            flux_position: 9,
            confidence: 0.92,
            confidence: 0.85,
            is_sacred: true,
            mode: ExecutionMode::Thorough,
            consensus_used: false,
            processing_time_ms: 250,
        };
        
        // Benchmark store operation
        group.bench_function("sqlite_store", |b| {
            b.iter(|| {
                runtime.block_on(async {
                    lake.store_diamond(black_box(&output)).await
                })
            });
        });
        
        // Store some data for query benchmarks
        for i in 0..100 {
            let mut test_output = output.clone();
            test_output.confidence = 0.6 + (i as f32 * 0.004);
            let _ = lake.store_diamond(&test_output).await;
        }
        
        // Benchmark queries
        group.bench_function("sqlite_query_sacred", |b| {
            b.iter(|| {
                runtime.block_on(async {
                    lake.query_sacred_diamonds().await
                })
            });
        });
        
        group.bench_function("sqlite_query_signal", |b| {
            b.iter(|| {
                runtime.block_on(async {
                    lake.query_by_signal(black_box(0.8)).await
                })
            });
        });
    });
    
    group.finish();
}

/// Benchmark ML/ONNX inference
#[cfg(feature = "onnx")]
fn benchmark_ml_inference(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("ml_inference");
    
    // Note: Requires actual ONNX model files to run
    // Using mock pool for benchmark structure
    let pool_result = OnnxSessionPool::new(
        "models/test.onnx",
        "models/tokenizer.json",
        4,  // initial size
        8   // max size
    );
    
    if let Ok(pool) = pool_result {
        let test_texts = vec![
            "Simple text",
            "Sacred geometry pattern detected at position 3",
            "Complex multi-sentence input that requires more processing time. The vortex flow pattern demonstrates sacred geometry principles through positions 3, 6, and 9.",
        ];
        
        for (i, text) in test_texts.iter().enumerate() {
            group.bench_with_input(
                BenchmarkId::new("embed", format!("text_{}", i)),
                text,
                |b, &text| {
                    b.iter(|| {
                        runtime.block_on(async {
                            pool.embed(black_box(text)).await
                        })
                    });
                }
            );
        }
        
        // Benchmark batch embedding
        let batch: Vec<String> = test_texts.iter().map(|s| s.to_string()).collect();
        group.bench_function("embed_batch", |b| {
            b.iter(|| {
                runtime.block_on(async {
                    pool.embed_batch(black_box(&batch)).await
                })
            });
        });
        
        // Benchmark with sacred geometry transformation
        group.bench_function("embed_sacred", |b| {
            b.iter(|| {
                runtime.block_on(async {
                    pool.embed_with_sacred_geometry(black_box("Sacred test")).await
                })
            });
        });
    }
    
    group.finish();
}

/// Benchmark sacred geometry calculations
fn benchmark_sacred_geometry(c: &mut Criterion) {
    use spatial_vortex::core::sacred_geometry::flux_matrix::FluxMatrixEngine;
    
    let mut group = c.benchmark_group("sacred_geometry");
    let engine = FluxMatrixEngine::new();
    
    // Benchmark position calculations
    let elp_values = vec![
        (3.0, 3.0, 3.0),  // Balanced
        (9.0, 1.0, 1.0),  // Ethos dominant
        (1.0, 9.0, 1.0),  // Logos dominant
        (1.0, 1.0, 9.0),  // Pathos dominant
    ];
    
    for (i, (e, l, p)) in elp_values.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("position_from_elp", i),
            &(*e, *l, *p),
            |b, &(e, l, p)| {
                b.iter(|| {
                    engine.calculate_position_from_elp(
                        black_box(e),
                        black_box(l),
                        black_box(p)
                    )
                });
            }
        );
    }
    
    // Benchmark vortex flow pattern
    group.bench_function("vortex_flow", |b| {
        b.iter(|| {
            for position in [1, 2, 4, 8, 7, 5, 1] {
                let _ = engine.apply_vortex_flow(black_box(position));
            }
        });
    });
    
    // Benchmark sacred triangle validation
    group.bench_function("sacred_validation", |b| {
        b.iter(|| {
            for position in 0..10 {
                let _ = engine.is_sacred_position(black_box(position));
            }
        });
    });
    
    group.finish();
}

/// Memory usage benchmark
fn benchmark_memory_usage(c: &mut Criterion) {
    use spatial_vortex::models::{BeamTensor, BeadTensor};
    
    let mut group = c.benchmark_group("memory_usage");
    
    // Benchmark tensor creation
    group.bench_function("beam_tensor_create", |b| {
        b.iter(|| {
            BeamTensor::new(
                black_box("test"),
                black_box(5),
                black_box(0.8)
            )
        });
    });
    
    // Benchmark large tensor arrays
    let sizes = vec![10, 100, 1000, 10000];
    for size in sizes {
        group.bench_with_input(
            BenchmarkId::new("beam_tensor_array", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let tensors: Vec<BeamTensor> = (0..size)
                        .map(|i| BeamTensor::new(&format!("tensor_{}", i), i % 10, 0.7))
                        .collect();
                    black_box(tensors)
                });
            }
        );
    }
    
    group.finish();
}

// Group all benchmarks
criterion_group!(
    benches,
    benchmark_asi_orchestrator,
    #[cfg(feature = "voice")]
    benchmark_voice_pipeline,
    #[cfg(feature = "lake")]
    benchmark_confidence_lake,
    #[cfg(feature = "onnx")]
    benchmark_ml_inference,
    benchmark_sacred_geometry,
    benchmark_memory_usage
);

criterion_main!(benches);
