//! Background Training Demo
//!
//! Demonstrates continuous model evolution:
//! 1. Background training coordinator monitors sample buffer
//! 2. Trains models periodically using DistributedTrainer
//! 3. Versions and hot-swaps improved models
//! 4. Integrates with RSI for training triggers

use spatial_vortex::ai::orchestrator::{ASIOrchestrator, ExecutionMode, RSIConfig};
use spatial_vortex::asi::runtime_detector::RuntimeDetectorConfig;
use spatial_vortex::ml::training::{BackgroundTrainingCoordinator, BackgroundTrainingConfig, TrainingTrigger};
use spatial_vortex::eustress_bridge::training_pipeline::EustressTrainingPipeline;
use spatial_vortex::eustress_bridge::flux_dynamics::FluxDynamics;
use spatial_vortex::eustress_bridge::entity_embedding::EustressEntity;
use spatial_vortex::error::Result;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("=== Background Training Demo ===\n");
    println!("Continuous Model Evolution with Distributed Training\n");
    
    // Step 1: Create training pipeline and coordinator
    println!("📦 Step 1: Initializing Background Training System...");
    
    let flux = FluxDynamics::new();
    let training_pipeline = EustressTrainingPipeline::new(flux);
    
    let mut training_config = BackgroundTrainingConfig::default();
    training_config.enabled = true;
    training_config.training_interval_secs = 10; // Train every 10 seconds for demo
    training_config.min_samples_for_training = 5; // Low threshold for demo
    training_config.max_samples_per_batch = 100;
    training_config.num_epochs = 2;
    training_config.auto_swap_models = true;
    training_config.min_improvement_threshold = 1.0; // 1% improvement
    
    let coordinator = BackgroundTrainingCoordinator::new(
        training_config.clone(),
        training_pipeline,
    );
    
    println!("✓ Training pipeline created");
    println!("✓ Background coordinator initialized");
    println!("✓ Training interval: {}s", training_config.training_interval_secs);
    println!("✓ Min samples: {}", training_config.min_samples_for_training);
    println!();
    
    // Step 2: Create orchestrator with background training
    println!("🧠 Step 2: Creating ASI Orchestrator with Background Training...");
    
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
    let orchestrator = ASIOrchestrator::new()
        .await?
        .with_self_modification(source_path)
        .with_runtime_detector(RuntimeDetectorConfig::default())
        .with_background_trainer(coordinator);
    
    // Enable RSI
    let mut rsi_config = RSIConfig::default();
    rsi_config.enabled = true;
    rsi_config.auto_apply_low_risk = true;
    orchestrator.enable_rsi(rsi_config).await;
    
    println!("✓ Orchestrator created");
    println!("✓ Background trainer: configured");
    println!("✓ RSI: enabled");
    println!();
    
    // Step 3: Start background training
    println!("🔄 Step 3: Starting Background Training...");
    let _training_handle = orchestrator.start_background_training().await?;
    
    println!("✓ Background training started");
    println!("✓ Monitoring sample buffer");
    println!("✓ Auto-training when thresholds met");
    println!();
    
    // Step 4: Simulate workload and generate training samples
    println!("📊 Step 4: Generating Training Samples (30 seconds)...");
    println!("   Simulating entity processing and sample collection\n");
    
    for i in 0..30 {
        // Create test entities
        let entities: Vec<EustressEntity> = (0..2)
            .map(|_| EustressEntity::default())
            .collect();
        
        // Add entities to training buffer
        if let Some(ref trainer) = orchestrator.background_trainer {
            trainer.add_entities(entities).await;
        }
        
        // Show progress
        if i % 5 == 0 {
            if let Some(stats) = orchestrator.training_stats() {
                print!("  [{}s] Buffer: {} samples", i + 1, 
                    orchestrator.background_trainer.as_ref().map(|t| t.sample_buffer_size()).unwrap_or(0));
                print!(" | Runs: {}", stats.total_training_runs);
                print!(" | Version: {}", stats.current_model_version);
                print!(" | Best acc: {:.1}%", stats.best_validation_accuracy * 100.0);
                println!();
            }
        }
        
        sleep(Duration::from_secs(1)).await;
    }
    println!();
    
    // Step 5: Manually trigger training
    println!("🎯 Step 5: Manually Triggering Training...");
    orchestrator.trigger_training(TrainingTrigger::Manual);
    
    println!("✓ Manual training triggered");
    
    // Wait for training to complete
    sleep(Duration::from_secs(3)).await;
    println!();
    
    // Step 6: Trigger from RSI (simulated)
    println!("🤖 Step 6: RSI-Triggered Training...");
    orchestrator.trigger_training(TrainingTrigger::RSITriggered {
        reason: "Performance improvement applied".to_string(),
    });
    
    println!("✓ RSI training triggered");
    println!("✓ Model will learn from applied improvements");
    
    // Wait for training
    sleep(Duration::from_secs(3)).await;
    println!();
    
    // Step 7: Show training statistics
    println!("📈 Step 7: Training Statistics");
    if let Some(stats) = orchestrator.training_stats() {
        println!("  Total training runs: {}", stats.total_training_runs);
        println!("  Total samples trained: {}", stats.total_samples_trained);
        println!("  Current model version: {}", stats.current_model_version);
        println!("  Model swaps: {}", stats.model_swaps);
        println!("  Best validation accuracy: {:.2}%", stats.best_validation_accuracy * 100.0);
        println!("  Avg training duration: {:.1}s", stats.average_training_duration_secs);
        
        if let Some(last_time) = stats.last_training_time {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            println!("  Last training: {}s ago", now - last_time);
        }
    }
    println!();
    
    // Step 8: Show model versions
    println!("📦 Step 8: Model Version History");
    let versions = orchestrator.get_model_versions();
    
    if versions.is_empty() {
        println!("  No model versions yet");
    } else {
        for version in &versions {
            println!("  Version {}: loss={:.4}, acc={:.2}%, improvement={:.2}%, samples={}",
                version.version,
                version.training_loss,
                version.validation_accuracy * 100.0,
                version.improvement_over_previous,
                version.training_samples);
            println!("    Trigger: {}", version.trigger);
            println!("    Path: {}", version.checkpoint_path.display());
            println!();
        }
    }
    
    // Step 9: Show current model
    println!("🎯 Step 9: Current Active Model");
    let current_version = orchestrator.current_model_version();
    println!("  Active model version: {}", current_version);
    
    if let Some(version) = versions.iter().find(|v| v.version == current_version) {
        println!("  Accuracy: {:.2}%", version.validation_accuracy * 100.0);
        println!("  Training loss: {:.4}", version.training_loss);
        println!("  Trained on {} samples", version.training_samples);
    }
    println!();
    
    // Stop training
    orchestrator.stop_background_training();
    println!("🛑 Background training stopped");
    println!();
    
    // Summary
    println!("=== Demo Summary ===");
    if let Some(stats) = orchestrator.training_stats() {
        println!("✓ Completed {} training runs", stats.total_training_runs);
        println!("✓ Trained on {} total samples", stats.total_samples_trained);
        println!("✓ Evolved to model version {}", stats.current_model_version);
        println!("✓ Performed {} model swaps", stats.model_swaps);
        println!("✓ Achieved {:.2}% best accuracy", stats.best_validation_accuracy * 100.0);
    }
    println!();
    
    println!("🎯 Background Training System: OPERATIONAL");
    println!("   The system continuously:");
    println!("   1. Collects training samples from entities");
    println!("   2. Trains models periodically (scheduled)");
    println!("   3. Trains on-demand (manual/RSI triggers)");
    println!("   4. Versions all trained models");
    println!("   5. Auto-swaps to improved models");
    println!("   6. Integrates with RSI for continuous evolution");
    println!();
    
    println!("🚀 Continuous Model Evolution: COMPLETE");
    
    Ok(())
}
