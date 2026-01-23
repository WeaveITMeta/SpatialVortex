//! Complete Alignment Training Demo
//!
//! Runs a full two-stage RL training cycle with many iterations:
//! - Stage 1: Discovery (explore reasoning patterns)
//! - Stage 2: Alignment (align to sacred geometry)
//!
//! Shows how the system learns and improves over time

use spatial_vortex::ml::training::two_stage_rl::{TwoStageRLTrainer, TwoStageConfig};
use anyhow::Result;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🎓 ═══════════════════════════════════════════════════════════");
    println!("   COMPLETE ALIGNMENT TRAINING");
    println!("   Running full two-stage RL training cycle");
    println!("🎓 ═══════════════════════════════════════════════════════════\n");

    // Configuration
    let config = TwoStageConfig {
        discovery_epsilon: 0.25,
        alignment_epsilon: 0.05,
        discovery_min_confidence: 0.6,
        alignment_min_confidence: 0.75,
        warmstart_traces: 1000,
        max_steps: 12, // Allow longer chains
    };
    
    println!("⚙️  Configuration:");
    println!("   Discovery Epsilon: {}", config.discovery_epsilon);
    println!("   Alignment Epsilon: {}", config.alignment_epsilon);
    println!("   Discovery Min Confidence: {}", config.discovery_min_confidence);
    println!("   Alignment Min Confidence: {}", config.alignment_min_confidence);
    println!("   Max Steps per Chain: {}\n", config.max_steps);

    let mut trainer = TwoStageRLTrainer::new(config)?;
    
    // Warmstart from Confidence Lake
    println!("🌊 Warmstarting from Confidence Lake...");
    trainer.warmstart_from_lake().await?;
    let stats = trainer.get_stats();
    println!("   ✅ Warmstart complete");
    println!("   📊 Initial buffer size: {}\n", stats.discovery_buffer_size);
    
    // Training tasks (varied complexity)
    let tasks = vec![
        "Implement quicksort with O(n log n) guarantee",
        "Design a cache with LRU eviction policy",
        "Create a thread-safe bounded queue",
        "Build an efficient string matching algorithm",
        "Implement a binary search tree with balance",
        "Design a memory-efficient graph representation",
        "Create a rate limiter with token bucket",
        "Implement merge sort for linked lists",
        "Build a bloom filter for set membership",
        "Design a lock-free stack using CAS",
    ];
    
    println!("📚 Training Tasks: {}", tasks.len());
    for (i, task) in tasks.iter().enumerate() {
        println!("   {}. {}", i + 1, task);
    }
    println!();
    
    // Stage 1: Discovery
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔍 STAGE 1: DISCOVERY (High Exploration)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let discovery_iterations = 15;
    let mut discovery_rewards = Vec::new();
    let start_discovery = Instant::now();
    
    for i in 1..=discovery_iterations {
        let task = &tasks[i % tasks.len()];
        let chain = trainer.train_iteration(task)?;
        
        let stats = trainer.get_stats();
        discovery_rewards.push(stats.discovery_avg_reward);
        
        if i % 3 == 0 {
            println!("   Iteration {}/{}: {} steps, conf {:.1}%, reward {:.3}, buffer {}",
                i, discovery_iterations,
                chain.steps.len(),
                chain.overall_confidence * 100.0,
                stats.discovery_avg_reward,
                stats.discovery_buffer_size
            );
        }
        
        // Check if we switched to alignment
        if matches!(stats.stage, spatial_vortex::ml::training::two_stage_rl::TrainingStage::Alignment) {
            println!("\n   🔄 Switched to Alignment stage at iteration {}", i);
            println!("   📊 Discovery buffer: {} experiences\n", stats.discovery_buffer_size);
            break;
        }
    }
    
    let discovery_time = start_discovery.elapsed();
    let final_discovery_reward = discovery_rewards.last().unwrap_or(&0.0);
    let initial_discovery_reward = discovery_rewards.first().unwrap_or(&0.0);
    let discovery_improvement = (final_discovery_reward - initial_discovery_reward) / 
        initial_discovery_reward.max(0.001);
    
    println!("📊 Discovery Stage Summary:");
    println!("   Duration: {:.1}s", discovery_time.as_secs_f32());
    println!("   Iterations: {}", discovery_rewards.len());
    println!("   Initial Reward: {:.3}", initial_discovery_reward);
    println!("   Final Reward: {:.3}", final_discovery_reward);
    println!("   Improvement: {:.1}%\n", discovery_improvement * 100.0);
    
    // Stage 2: Alignment
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎯 STAGE 2: ALIGNMENT (Sacred Geometry Optimization)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let alignment_iterations = 20;
    let mut alignment_rewards = Vec::new();
    let mut vortex_complete_count = 0;
    let mut sacred_complete_count = 0;
    let start_alignment = Instant::now();
    
    for i in 1..=alignment_iterations {
        let task = &tasks[i % tasks.len()];
        let chain = trainer.train_iteration(task)?;
        
        let stats = trainer.get_stats();
        alignment_rewards.push(stats.alignment_avg_reward);
        
        if chain.completed_vortex_cycle {
            vortex_complete_count += 1;
        }
        
        let sacred_count = chain.steps.iter().filter(|s| s.is_sacred).count();
        if sacred_count >= 3 {
            sacred_complete_count += 1;
        }
        
        if i % 4 == 0 {
            let cycle_icon = if chain.completed_vortex_cycle { "🔄" } else { "  " };
            println!("   {} Iteration {}/{}: {} steps, conf {:.1}%, reward {:.3}, sacred {}/3",
                cycle_icon, i, alignment_iterations,
                chain.steps.len(),
                chain.overall_confidence * 100.0,
                stats.alignment_avg_reward,
                sacred_count
            );
        }
    }
    
    let alignment_time = start_alignment.elapsed();
    let final_alignment_reward = alignment_rewards.last().unwrap_or(&0.0);
    let initial_alignment_reward = alignment_rewards.first().unwrap_or(&0.0);
    let alignment_improvement = (final_alignment_reward - initial_alignment_reward) / 
        initial_alignment_reward.max(0.001);
    
    println!("\n📊 Alignment Stage Summary:");
    println!("   Duration: {:.1}s", alignment_time.as_secs_f32());
    println!("   Iterations: {}", alignment_rewards.len());
    println!("   Initial Reward: {:.3}", initial_alignment_reward);
    println!("   Final Reward: {:.3}", final_alignment_reward);
    println!("   Improvement: {:.1}%", alignment_improvement * 100.0);
    println!("   Vortex Cycles Complete: {}/{} ({:.0}%)",
        vortex_complete_count, alignment_iterations,
        (vortex_complete_count as f32 / alignment_iterations as f32) * 100.0);
    println!("   Sacred Positions Complete: {}/{} ({:.0}%)\n",
        sacred_complete_count, alignment_iterations,
        (sacred_complete_count as f32 / alignment_iterations as f32) * 100.0);
    
    // Overall statistics
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📈 OVERALL STATISTICS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let stats = trainer.get_stats();
    let total_time = start_discovery.elapsed();
    let total_iterations = discovery_rewards.len() + alignment_rewards.len();
    
    println!("⏱️  Performance:");
    println!("   Total Duration: {:.1}s", total_time.as_secs_f32());
    println!("   Total Iterations: {}", total_iterations);
    println!("   Avg Iteration Time: {:.0}ms", 
        (total_time.as_millis() as f32 / total_iterations as f32));
    
    println!("\n🎯 Final Metrics:");
    println!("   Discovery Avg Reward: {:.3}", stats.discovery_avg_reward);
    println!("   Alignment Avg Reward: {:.3}", stats.alignment_avg_reward);
    println!("   Discovery Buffer: {} experiences", stats.discovery_buffer_size);
    println!("   Alignment Buffer: {} experiences", stats.alignment_buffer_size);
    
    println!("\n📊 Learning Progress:");
    
    // Discovery progress
    if discovery_rewards.len() > 1 {
        print!("   Discovery: ");
        for (i, &reward) in discovery_rewards.iter().enumerate() {
            if i % 3 == 0 {
                let bar_len = (reward * 20.0) as usize;
                print!("{:.2}|{}| ", reward, "█".repeat(bar_len));
            }
        }
        println!();
    }
    
    // Alignment progress
    if alignment_rewards.len() > 1 {
        print!("   Alignment: ");
        for (i, &reward) in alignment_rewards.iter().enumerate() {
            if i % 4 == 0 {
                let bar_len = (reward * 20.0) as usize;
                print!("{:.2}|{}| ", reward, "█".repeat(bar_len));
            }
        }
        println!();
    }
    
    // Quality assessment
    println!("\n🎊 Training Assessment:");
    if *final_alignment_reward >= 0.8 {
        println!("   ⭐ EXCELLENT - Model highly aligned to sacred geometry");
        println!("   ✅ Ready for production use");
    } else if *final_alignment_reward >= 0.6 {
        println!("   ✅ GOOD - Model well-aligned, minor improvements possible");
        println!("   ✅ Suitable for most applications");
    } else if *final_alignment_reward >= 0.4 {
        println!("   ⚠️  MODERATE - Model partially aligned, more training recommended");
        println!("   📚 Run additional iterations");
    } else {
        println!("   ❌ NEEDS MORE TRAINING - Continue training with more iterations");
        println!("   📚 Increase warmstart traces or training duration");
    }
    
    // Convergence analysis
    println!("\n🔬 Convergence Analysis:");
    
    let discovery_variance = calculate_variance(&discovery_rewards);
    let alignment_variance = calculate_variance(&alignment_rewards);
    
    println!("   Discovery Variance: {:.4}", discovery_variance);
    println!("   Alignment Variance: {:.4}", alignment_variance);
    
    if alignment_variance < 0.01 {
        println!("   ✅ CONVERGED - Stable performance achieved");
    } else if alignment_variance < 0.05 {
        println!("   ⚡ CONVERGING - Nearly stable");
    } else {
        println!("   📈 IMPROVING - Still learning, more iterations beneficial");
    }
    
    // Recommendations
    println!("\n💡 Recommendations:");
    
    if *final_alignment_reward < 0.7 {
        println!("   • Run {} more alignment iterations", 
            ((0.8 - final_alignment_reward) * 50.0) as usize);
    }
    
    if vortex_complete_count < alignment_iterations * 7 / 10 {
        println!("   • Increase chain length (max_steps > {})", config.max_steps);
    }
    
    if sacred_complete_count < alignment_iterations * 8 / 10 {
        println!("   • Boost sacred position rewards further");
    }
    
    if alignment_variance > 0.05 {
        println!("   • Continue training for better convergence");
    } else {
        println!("   • Model converged! Consider saving checkpoint");
    }
    
    println!("\n✨ Complete alignment training finished! ✨\n");
    
    Ok(())
}

fn calculate_variance(values: &[f32]) -> f32 {
    if values.len() < 2 {
        return 0.0;
    }
    
    let mean: f32 = values.iter().sum::<f32>() / values.len() as f32;
    let variance: f32 = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f32>() / values.len() as f32;
    
    variance
}
