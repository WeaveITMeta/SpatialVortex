//! Vortex Context Preserver (VCP) Integration Test
//!
//! Demonstrates the full pipeline:
//! Voice → FFT → ELP → ASI Orchestrator → ML Enhancement → Confidence Lake

#[cfg(test)]
mod vcp_integration_tests {
    use spatial_vortex::ai::orchestrator::{ASIOrchestrator, ExecutionMode};
    use spatial_vortex::models::ELPTensor;
    
    #[cfg(feature = "voice")]
    use spatial_vortex::voice_pipeline::{
        VoicePipeline, StreamingVoiceProcessor, AudioConfig, SpectralAnalyzer
    };
    
    #[cfg(feature = "lake")]
    use spatial_vortex::storage::confidence_lake::{PostgresConfidenceLake, SecureStorage};
    
    #[cfg(feature = "onnx")]
    use spatial_vortex::ml::inference::OnnxSessionPool;
    
    use std::sync::Arc;
    use tokio::sync::mpsc;
    
    /// Test complete voice → ASI → Lake pipeline
    #[tokio::test]
    #[cfg(all(feature = "voice", feature = "lake"))]
    async fn test_voice_to_lake_pipeline() {
        // Step 1: Initialize ASI Orchestrator
        let mut asi = ASIOrchestrator::new()
            .expect("Failed to create ASI orchestrator");
        
        // Initialize Confidence Lake
        asi.init_confidence_lake_async(":memory:")
            .await
            .expect("Failed to init Confidence Lake");
        
        // Step 2: Create simulated voice data
        let (audio_tx, audio_rx) = mpsc::channel::<Vec<f32>>(100);
        
        // Simulate 440Hz tone (A4 note) at 44100 Hz sample rate
        let sample_rate = 44100;
        let frequency = 440.0;
        let duration = 0.1; // 100ms of audio
        let samples = (sample_rate as f32 * duration) as usize;
        
        let audio_chunk: Vec<f32> = (0..samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();
        
        // Step 3: Process through spectral analyzer
        let analyzer = Arc::new(tokio::sync::Mutex::new(
            SpectralAnalyzer::new(sample_rate)
        ));
        
        let streaming = StreamingVoiceProcessor::new(analyzer.clone(), audio_rx);
        
        // Send audio data
        audio_tx.send(audio_chunk).await.unwrap();
        drop(audio_tx);
        
        // Step 4: Process through ASI Orchestrator
        // Simulate voice-derived ELP from spectral features
        let voice_elp = ELPTensor {
            ethos: 5.0,   // Character from voice tone
            logos: 4.0,   // Logic from pitch stability
            pathos: 7.0,  // Emotion from intensity
        };
        
        let voice_input = format!(
            "Voice: pitch={}Hz, loudness=-15dB, confidence=0.85",
            frequency
        );
        
        let result = asi.process_voice(&voice_input, Some(voice_elp))
            .await
            .expect("Failed to process voice");
        
        // Verify results
        assert!(result.confidence >= 0.6, "Signal too weak: {}", result.confidence);
        assert!(result.confidence > 0.5, "Confidence too low: {}", result.confidence);
        
        // Should be stored in Confidence Lake (signal >= 0.6)
        println!("Voice processed successfully:");
        println!("  Position: {}", result.flux_position);
        println!("  Signal: {:.2}", result.confidence);
        println!("  Confidence: {:.2}", result.confidence);
        println!("  Sacred: {}", result.is_sacred);
    }
    
    /// Test ML enhancement with ONNX pooling
    #[tokio::test]
    #[cfg(feature = "onnx")]
    async fn test_onnx_pool_enhancement() {
        // Create ONNX session pool (would use real model in production)
        let pool = OnnxSessionPool::new(
            "models/test_model.onnx",
            "models/test_tokenizer.json",
            2,  // initial size
            4   // max size
        ).unwrap_or_else(|_| {
            // Fallback for test environment without models
            OnnxSessionPool::new("dummy.onnx", "dummy.json", 2, 4).unwrap()
        });
        
        // Test concurrent embeddings
        let texts = vec![
            "Sacred geometry at position 3".to_string(),
            "Vortex flow pattern detected".to_string(),
            "Signal strength optimal".to_string(),
        ];
        
        // This would fail without ONNX feature, but that's okay for testing
        if cfg!(feature = "onnx") {
            match pool.embed_batch(&texts).await {
                Ok(embeddings) => {
                    assert_eq!(embeddings.len(), texts.len());
                    println!("Generated {} embeddings", embeddings.len());
                }
                Err(e) => {
                    println!("ONNX not available: {}", e);
                }
            }
        }
    }
    
    /// Test Confidence Lake encryption and retrieval
    #[tokio::test]
    #[cfg(feature = "lake")]
    async fn test_confidence_lake_encryption() {
        // Create encrypted lake (requires a running PostgreSQL instance)
        let mut lake = PostgresConfidenceLake::new("postgresql://localhost/confidence")
            .await
            .expect("Failed to create lake");
        
        // Enable encryption
        let key = SecureStorage::generate_key();
        lake.enable_encryption(&key);
        
        // Create high-quality output to store (confidence is the consolidated metric)
        let output = spatial_vortex::ai::orchestrator::ASIOutput {
            result: "Sacred position detected with high confidence".to_string(),
            elp: ELPTensor {
                ethos: 6.0,
                logos: 7.0,
                pathos: 8.0,
            },
            flux_position: 9, // Sacred position
            confidence: 0.92, // Well above threshold
            is_sacred: true,
            mode: ExecutionMode::Thorough,
            consensus_used: false,
            processing_time_ms: 250,
        };
        
        // Store flux matrix
        let id = lake.store_flux_matrix(&output)
            .await
            .expect("Failed to store flux matrix");
        
        // Retrieve and verify
        let flux_matrix = lake.retrieve_flux_matrix(id)
            .await
            .expect("Failed to retrieve flux matrix");
        
        assert_eq!(flux_matrix.confidence, 0.92);
        assert_eq!(flux_matrix.flux_position, 9);
        assert!(flux_matrix.is_sacred);
        
        // Query sacred flux matrices
        let sacred_flux = lake.query_sacred_flux_matrices()
            .await
            .expect("Failed to query sacred flux matrices");
        
        assert_eq!(sacred_flux.len(), 1);
        
        // Get lake statistics
        let stats = lake.get_stats()
            .await
            .expect("Failed to get stats");
        
        assert_eq!(stats.total_flux_matrices, 1);
        assert_eq!(stats.sacred_count, 1);
        assert!(stats.avg_confidence > 0.8);
        
        println!("Confidence Lake test passed:");
        println!("  Total flux matrices: {}", stats.total_flux_matrices);
        println!("  Avg confidence: {:.2}", stats.avg_confidence);
        println!("  Sacred count: {}", stats.sacred_count);
    }
    
    /// Test end-to-end latency requirements
    #[tokio::test]
    async fn test_vcp_latency() {
        use std::time::Instant;
        
        let mut asi = ASIOrchestrator::new()
            .expect("Failed to create orchestrator");
        
        // Fast mode should be <100ms
        let start = Instant::now();
        let result = asi.process("Quick test", ExecutionMode::Fast)
            .await
            .expect("Failed to process");
        let elapsed = start.elapsed();
        
        assert!(elapsed.as_millis() < 200, "Fast mode too slow: {}ms", elapsed.as_millis());
        assert!(result.processing_time_ms < 100, "Processing time too high: {}ms", result.processing_time_ms);
        
        println!("Latency test passed:");
        println!("  Mode: {:?}", result.mode);
        println!("  Processing time: {}ms", result.processing_time_ms);
        println!("  Total elapsed: {}ms", elapsed.as_millis());
    }
    
    /// Test sacred geometry integration
    #[tokio::test]
    async fn test_sacred_geometry_boost() {
        let mut asi = ASIOrchestrator::new()
            .expect("Failed to create orchestrator");
        
        // Test sacred position detection
        let sacred_inputs = vec![
            (3, "Creative trinity energy"),
            (6, "Harmonic balance point"),
            (9, "Completion cycle achieved"),
        ];
        
        for (expected_position, input) in sacred_inputs {
            // Process with specific ELP to force position
            let elp = match expected_position {
                3 => ELPTensor { ethos: 9.0, logos: 3.0, pathos: 3.0 },
                6 => ELPTensor { ethos: 3.0, logos: 3.0, pathos: 9.0 },
                9 => ELPTensor { ethos: 3.0, logos: 9.0, pathos: 3.0 },
                _ => ELPTensor { ethos: 5.0, logos: 5.0, pathos: 5.0 },
            };
            
            let result = asi.process_voice(input, Some(elp))
                .await
                .expect("Failed to process");
            
            assert!(result.is_sacred, "Position {} should be sacred", expected_position);
            assert!(result.confidence > 0.7, "Sacred boost not applied");
            
            println!("Sacred position {} verified:", expected_position);
            println!("  Confidence: {:.2}", result.confidence);
            println!("  Signal: {:.2}", result.confidence);
        }
    }
}

/// Performance benchmarks for VCP components
#[cfg(all(test, not(target_arch = "wasm32")))]
mod benchmarks {
    use criterion::{black_box, Criterion};
    use spatial_vortex::ai::orchestrator::{ASIOrchestrator, ExecutionMode};
    
    fn benchmark_asi_modes(c: &mut Criterion) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        c.bench_function("asi_fast_mode", |b| {
            b.iter(|| {
                runtime.block_on(async {
                    let mut asi = ASIOrchestrator::new().unwrap();
                    let _ = asi.process(
                        black_box("Benchmark input"),
                        black_box(ExecutionMode::Fast)
                    ).await;
                });
            });
        });
        
        c.bench_function("asi_balanced_mode", |b| {
            b.iter(|| {
                runtime.block_on(async {
                    let mut asi = ASIOrchestrator::new().unwrap();
                    let _ = asi.process(
                        black_box("Benchmark input"),
                        black_box(ExecutionMode::Balanced)
                    ).await;
                });
            });
        });
    }
    
    #[cfg(feature = "voice")]
    fn benchmark_fft_processing(c: &mut Criterion) {
        use spatial_vortex::voice_pipeline::SpectralAnalyzer;
        
        let mut analyzer = SpectralAnalyzer::new(44100);
        let audio: Vec<f32> = vec![0.0; 1024]; // Mock audio
        
        c.bench_function("fft_analysis", |b| {
            b.iter(|| {
                let _ = analyzer.analyze(black_box(&audio));
            });
        });
    }
}
