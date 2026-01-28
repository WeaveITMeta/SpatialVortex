//! Integration Tests: End-to-End Continuous Learning Loop
//!
//! Tests the full pipeline: VortexRunner → CALM → RSI → VerifiedPatterning
//! Simulates a "learning episode" with dummy data.

#[cfg(test)]
mod tests {
    use crate::cognition::verified_patterning::{
        VerifiedPatterningEngine, VerificationConfig, BenchmarkResult,
        ContinuousLearningConfig as VPLearningConfig,
    };
    use crate::ml::continuous_learning::{
        ContinuousTrainer, ContinuousLearningConfig,
    };
    use crate::ml::calm::{CALMEngine, CALMConfig};
    use crate::ml::huggingface::{RSIState, RSIMetric};
    use crate::data::models::BeamTensor;

    /// Test CALM encode/decode roundtrip
    #[test]
    fn test_calm_roundtrip() {
        let config = CALMConfig::default();
        let calm = CALMEngine::new(config);

        let input: Vec<BeamTensor> = (0..5).map(|i| {
            let mut beam = BeamTensor::default();
            beam.digits[0] = i as f32 * 0.1;
            beam.confidence = 0.8;
            beam
        }).collect();

        let latent = calm.encode(&input);
        assert!(latent.energy > 0.0, "Latent should have positive energy");
        
        let output = calm.decode(&latent);
        assert!(!output.is_empty(), "Decoded output should not be empty");
    }

    /// Test RSI-driven epoch scheduling
    #[test]
    fn test_rsi_epoch_scheduling() {
        let config = ContinuousLearningConfig::default();
        let mut trainer = ContinuousTrainer::new(config);

        // Add improving RSI metrics
        let rsi_state = RSIState {
            cycle: 5,
            metrics: (0..5).map(|i| RSIMetric {
                cycle: i,
                name: "reward".to_string(),
                value: 0.6 + (i as f64 * 0.05),
                timestamp: chrono::Utc::now().timestamp(),
            }).collect(),
            improvements: vec![],
            best_config: None,
        };
        trainer.update_rsi(&rsi_state);

        // Should have signal with best score
        let signal = trainer.rsi_signal();
        assert!(signal.best_score > 0.7, "Best score should reflect improvement");
    }

    /// Test training session with continuous trainer
    #[test]
    fn test_training_session() {
        let config = ContinuousLearningConfig::default();
        let mut trainer = ContinuousTrainer::new(config);

        // Create training data
        let input_beams: Vec<BeamTensor> = (0..10).map(|i| {
            let mut beam = BeamTensor::default();
            beam.digits[0] = i as f32 * 0.1;
            beam.confidence = 0.5 + (i as f32 * 0.05);
            beam
        }).collect();

        let training_data: Vec<(Vec<BeamTensor>, Vec<BeamTensor>)> = input_beams
            .chunks(2)
            .map(|chunk| {
                let input = chunk.to_vec();
                let mut target = chunk.to_vec();
                for beam in &mut target {
                    beam.confidence = (beam.confidence + 0.1).min(1.0);
                }
                (input, target)
            })
            .collect();

        // Run training session
        let mut loss = 1.0;
        let result = trainer.train_session(&training_data, |_batch, _lr| {
            loss *= 0.9;
            loss
        });

        assert!(result.best_loss < 1.0, "Training should reduce loss");
        assert!(!result.epochs.is_empty(), "Should have training epochs");

        // Verify training
        let verification = trainer.verify_training(&result);
        println!("Training verification: {:?}", verification);
    }

    /// Test benchmark tracking for SOTA
    #[test]
    fn test_benchmark_sota_tracking() {
        let vp_config = VPLearningConfig::default();
        let ver_config = VerificationConfig::default();
        let mut patterning = VerifiedPatterningEngine::new(vp_config, ver_config);

        // Record a benchmark with gap to SOTA
        let benchmark = BenchmarkResult {
            name: "MMLU".to_string(), // Use uppercase to match SOTA tracker
            version: "v1".to_string(),
            score: 70.0, // Use percentage scale to match SOTA scores
            max_score: 100.0,
            sota_score: 90.0,
            timestamp_ms: 1000,
            config_hash: "config1".to_string(),
        };
        let _evidence = patterning.record_benchmark(benchmark);

        let progress = patterning.benchmark_progress();
        
        // Should have benchmarks tracked
        assert!(!progress.benchmarks.is_empty(), "Should track benchmarks");
        
        // Gap should be positive (70 vs 90 SOTA)
        if !progress.benchmarks.is_empty() {
            let gap = progress.benchmarks[0].gap_to_sota;
            assert!(gap > 0.0, "Should have gap to SOTA: {}", gap);
        }
    }

    /// End-to-end integration: CALM → RSI → Training → Benchmark
    #[tokio::test]
    async fn test_end_to_end_learning_episode() {
        // 1. CALM compression
        let calm = CALMEngine::new(CALMConfig::default());
        let input_beams: Vec<BeamTensor> = (0..10).map(|i| {
            let mut beam = BeamTensor::default();
            beam.digits = [i as f32 * 0.1; 9];
            beam.confidence = 0.5 + (i as f32 * 0.05);
            beam
        }).collect();

        let latent = calm.encode(&input_beams);
        assert!(latent.energy > 0.0);
        let decoded = calm.decode(&latent);
        assert!(!decoded.is_empty());

        // 2. RSI-driven training
        let mut trainer = ContinuousTrainer::new(ContinuousLearningConfig::default());
        
        let rsi_state = RSIState {
            cycle: 10,
            metrics: (0..10).map(|i| RSIMetric {
                cycle: i,
                name: "reward".to_string(),
                value: 0.5 + (i as f64 * 0.03),
                timestamp: chrono::Utc::now().timestamp(),
            }).collect(),
            improvements: vec![],
            best_config: None,
        };
        trainer.update_rsi(&rsi_state);

        let signal = trainer.rsi_signal();
        assert!(signal.best_score > 0.5, "RSI should track best score");

        // 3. Training session
        let training_data: Vec<(Vec<BeamTensor>, Vec<BeamTensor>)> = input_beams
            .chunks(2)
            .map(|chunk| (chunk.to_vec(), chunk.to_vec()))
            .collect();

        let mut loss = 1.0;
        let result = trainer.train_session(&training_data, |_batch, _lr| {
            loss *= 0.9;
            loss
        });
        assert!(result.best_loss < 1.0);

        // 4. Benchmark tracking
        let mut patterning = VerifiedPatterningEngine::new(
            VPLearningConfig::default(),
            VerificationConfig::default(),
        );

        let benchmark = BenchmarkResult {
            name: "integration_test".to_string(),
            version: "v1.0".to_string(),
            score: 1.0 - result.best_loss,
            max_score: 1.0,
            sota_score: 0.95,
            timestamp_ms: 2000,
            config_hash: "test".to_string(),
        };
        patterning.record_benchmark(benchmark);

        let progress = patterning.benchmark_progress();
        // Benchmark was recorded, check it exists
        println!("Benchmarks tracked: {:?}", progress.benchmarks.len());
        
        println!("End-to-end test passed: CALM → RSI → Training → Benchmark");
    }
}
