//! Distributed Training Demo
//!
//! Demonstrates the distributed training infrastructure:
//! - Model configuration (small to 7B)
//! - Parameter registration and management
//! - AdamW optimizer with weight decay
//! - Learning rate scheduling (warmup + cosine)
//! - Gradient synchronization
//! - Checkpoint saving/loading
//! - Data loading with batching
//!
//! ## Run
//!
//! ```bash
//! cargo run --example distributed_training_demo --release
//! ```

use spatial_vortex::ml::training::{
    // Distributed training
    DistributedTrainer,
    DistributedConfig,
    DistributedBackend,
    ParallelismStrategy,
    Parameter,
    ParameterStore,
    AdamW,
    OptimizerConfig,
    LRScheduler,
    LRSchedule,
    TrainingSample,
    InMemoryDataset,
    DataLoader,
    Dataset,
    // Model
    ModelConfig,
    SpatialVortexModel,
};
use std::sync::Arc;
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Distributed Training Infrastructure Demo               ║");
    println!("║      Data Parallel • AdamW • LR Schedule • Checkpoints       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ========== 1. Model Configuration ==========
    println!("📊 1. MODEL CONFIGURATIONS\n");
    
    let configs = [
        ("Small (testing)", ModelConfig::small()),
        ("Default (768d)", ModelConfig::default()),
        ("LLaMA 7B", ModelConfig::llama_7b()),
    ];
    
    for (name, config) in &configs {
        let params = config.total_params();
        let params_str = if params > 1_000_000_000 {
            format!("{:.2}B", params as f64 / 1e9)
        } else if params > 1_000_000 {
            format!("{:.2}M", params as f64 / 1e6)
        } else {
            format!("{:.2}K", params as f64 / 1e3)
        };
        
        println!("   {}: {} parameters", name, params_str);
        println!("      d_model={}, layers={}, heads={}, vocab={}",
            config.d_model, config.num_layers, config.num_heads, config.vocab_size);
    }
    println!();

    // ========== 2. Create Model ==========
    println!("🏗️  2. MODEL CREATION\n");
    
    let config = ModelConfig::small();
    let model = SpatialVortexModel::new(config.clone());
    
    println!("   Created SpatialVortexModel:");
    println!("   - d_model: {}", config.d_model);
    println!("   - layers: {}", config.num_layers);
    println!("   - heads: {}", config.num_heads);
    println!("   - vocab: {}", config.vocab_size);
    println!("   - total params: {}", model.num_params());
    println!();

    // ========== 3. Parameter Registration ==========
    println!("📝 3. PARAMETER REGISTRATION\n");
    
    let mut params = ParameterStore::new();
    model.register_params(&mut params);
    
    println!("   Registered {} total parameters", params.total_params());
    println!("   Parameter groups:");
    for param in params.iter().take(5) {
        println!("      - {}: {:?}", param.name, param.data.dim());
    }
    println!("      ... and more");
    println!();

    // ========== 4. Optimizer Setup ==========
    println!("⚙️  4. OPTIMIZER SETUP\n");
    
    let opt_config = OptimizerConfig {
        lr: 1e-4,
        weight_decay: 0.01,
        beta1: 0.9,
        beta2: 0.95,
        eps: 1e-8,
    };
    
    let mut optimizer = AdamW::new(opt_config.clone());
    
    println!("   AdamW Optimizer:");
    println!("   - learning_rate: {}", opt_config.lr);
    println!("   - weight_decay: {}", opt_config.weight_decay);
    println!("   - betas: ({}, {})", opt_config.beta1, opt_config.beta2);
    println!("   - eps: {}", opt_config.eps);
    println!();

    // ========== 5. Learning Rate Schedule ==========
    println!("📈 5. LEARNING RATE SCHEDULE\n");
    
    let total_steps = 10000;
    let warmup_steps = 500;
    
    let schedule = LRSchedule::WarmupCosine {
        warmup_steps,
        total_steps,
        min_lr: 1e-6,
    };
    let mut scheduler = LRScheduler::new(schedule, 1e-4);
    
    println!("   Warmup + Cosine Decay:");
    println!("   - warmup_steps: {}", warmup_steps);
    println!("   - total_steps: {}", total_steps);
    println!("   - base_lr: 1e-4");
    println!("   - min_lr: 1e-6");
    println!();
    
    // Show LR at different points
    println!("   LR at different steps:");
    for step in [0, 100, 500, 1000, 5000, 9000] {
        let mut temp_scheduler = LRScheduler::new(
            LRSchedule::WarmupCosine {
                warmup_steps,
                total_steps,
                min_lr: 1e-6,
            },
            1e-4,
        );
        for _ in 0..step {
            temp_scheduler.step();
        }
        println!("      Step {}: lr = {:.2e}", step, temp_scheduler.get_lr());
    }
    println!();

    // ========== 6. Data Loading ==========
    println!("📦 6. DATA LOADING\n");
    
    // Create synthetic dataset
    let num_samples = 1000;
    let seq_len = 64;
    let samples: Vec<Vec<u32>> = (0..num_samples)
        .map(|i| {
            (0..seq_len).map(|j| ((i * j) % config.vocab_size) as u32).collect()
        })
        .collect();
    
    let dataset = Arc::new(InMemoryDataset::from_tokens(samples, seq_len));
    
    println!("   Dataset: {} samples, seq_len={}", dataset.len(), seq_len);
    
    let batch_size = 8;
    let mut dataloader = DataLoader::new(
        dataset.clone(),
        batch_size,
        true,  // shuffle
        1,     // world_size
        0,     // rank
    );
    
    println!("   DataLoader: batch_size={}, num_batches={}", batch_size, dataloader.num_batches());
    
    // Get a batch
    dataloader.shuffle_indices();
    if let Some(batch) = dataloader.next_batch() {
        println!("   First batch: {} samples", batch.len());
        println!("   Sample input_ids len: {}", batch[0].input_ids.len());
    }
    println!();

    // ========== 7. Distributed Configuration ==========
    println!("🌐 7. DISTRIBUTED CONFIGURATION\n");
    
    let dist_configs = [
        ("Single GPU", DistributedConfig {
            world_size: 1,
            backend: DistributedBackend::Single,
            ..Default::default()
        }),
        ("4x GPU Data Parallel", DistributedConfig {
            world_size: 4,
            backend: DistributedBackend::Nccl,
            strategy: ParallelismStrategy::DataParallel,
            gradient_accumulation_steps: 4,
            ..Default::default()
        }),
        ("8x GPU with ZeRO-2", DistributedConfig {
            world_size: 8,
            backend: DistributedBackend::Nccl,
            strategy: ParallelismStrategy::ZeRO(spatial_vortex::ml::training::ZeROStage::Stage2),
            gradient_accumulation_steps: 8,
            mixed_precision: true,
            ..Default::default()
        }),
    ];
    
    for (name, config) in &dist_configs {
        println!("   {}:", name);
        println!("      world_size: {}", config.world_size);
        println!("      backend: {:?}", config.backend);
        println!("      strategy: {:?}", config.strategy);
        println!("      grad_accum: {}", config.gradient_accumulation_steps);
    }
    println!();

    // ========== 8. Training Loop Demo ==========
    println!("🔄 8. TRAINING LOOP DEMO\n");
    
    let dist_config = DistributedConfig::default();
    let trainer = DistributedTrainer::new(dist_config);
    trainer.init_process_group().expect("Failed to init process group");
    
    // Simple training loop
    let num_steps = 5;
    let mut total_loss = 0.0;
    
    println!("   Running {} training steps...", num_steps);
    
    let start = Instant::now();
    
    dataloader.reset();
    dataloader.shuffle_indices();
    
    for step in 0..num_steps {
        // Get batch
        let batch = match dataloader.next_batch() {
            Some(b) => b,
            None => {
                dataloader.reset();
                dataloader.shuffle_indices();
                dataloader.next_batch().unwrap()
            }
        };
        
        // Forward pass
        let sample = &batch[0];
        let logits = model.forward(&sample.input_ids);
        let loss = model.compute_loss(&logits, &sample.labels);
        
        // Simulate backward pass (in real training, this computes gradients)
        let grad_norm = trainer.training_step(
            &mut params,
            &mut optimizer,
            loss,
            |_params| {
                // Backward pass would compute gradients here
                // For demo, we skip actual gradient computation
            },
        );
        
        // Update LR
        scheduler.step();
        scheduler.set_lr(&mut optimizer);
        
        total_loss += loss;
        
        println!("      Step {}: loss={:.4}, grad_norm={:.4}, lr={:.2e}",
            step + 1, loss, grad_norm, scheduler.get_lr());
    }
    
    let elapsed = start.elapsed();
    let avg_loss = total_loss / num_steps as f32;
    
    println!("\n   Training complete:");
    println!("   - Total time: {:?}", elapsed);
    println!("   - Avg loss: {:.4}", avg_loss);
    println!("   - Steps/sec: {:.2}", num_steps as f64 / elapsed.as_secs_f64());
    println!();

    // ========== 9. Checkpoint Demo ==========
    println!("💾 9. CHECKPOINT DEMO\n");
    
    // Save checkpoint
    match trainer.save_checkpoint(&params, &optimizer, 0) {
        Ok(path) => println!("   Checkpoint saved to: {:?}", path),
        Err(e) => println!("   Checkpoint save skipped (rank != 0 or error: {})", e),
    }
    println!();

    // ========== 10. Scaling Estimates ==========
    println!("📊 10. SCALING ESTIMATES\n");
    
    let model_sizes = [
        ("125M", 125_000_000usize),
        ("350M", 350_000_000),
        ("1.3B", 1_300_000_000),
        ("7B", 7_000_000_000),
        ("13B", 13_000_000_000),
        ("70B", 70_000_000_000),
    ];
    
    println!("   Memory requirements (FP16 + AdamW states):");
    for (name, params) in model_sizes {
        // FP16 params + FP32 optimizer states (m, v) + gradients
        let param_mem = params * 2;  // FP16
        let opt_mem = params * 4 * 2;  // FP32 m and v
        let grad_mem = params * 2;  // FP16
        let total_gb = (param_mem + opt_mem + grad_mem) as f64 / 1e9;
        
        let gpus_needed = (total_gb / 80.0).ceil() as usize;  // A100 80GB
        
        println!("      {}: {:.1} GB ({} x A100 80GB)", name, total_gb, gpus_needed.max(1));
    }
    println!();

    // ========== Summary ==========
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     SUMMARY                                   ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ ✅ Model configs: Small (256d) to LLaMA 7B (4096d)           ║");
    println!("║ ✅ Parameter store: Registration and gradient tracking       ║");
    println!("║ ✅ AdamW optimizer: Decoupled weight decay                   ║");
    println!("║ ✅ LR scheduling: Warmup + cosine decay                      ║");
    println!("║ ✅ Data loading: Batching, shuffling, distributed            ║");
    println!("║ ✅ Gradient sync: All-reduce across processes                ║");
    println!("║ ✅ Checkpointing: Save/load model state                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("🚀 NEXT STEPS FOR PRODUCTION:");
    println!("   1. Enable burn-train feature for native Burn training");
    println!("   2. Add NCCL backend for multi-GPU (requires CUDA)");
    println!("   3. Implement gradient checkpointing for memory");
    println!("   4. Add mixed precision (FP16/BF16) support");
    println!("   5. Load real datasets (HuggingFace datasets)");
}
