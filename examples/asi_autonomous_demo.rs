//! ASI Autonomous Demo
//!
//! Demonstrates the autonomous intelligence loop running continuously,
//! perceiving the environment, reasoning, and taking actions.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example asi_autonomous_demo
//! ```

use spatial_vortex::asi::bootstrap::ASIBuilder;
use spatial_vortex::asi::goal_manager::GoalPriority;
use spatial_vortex::asi::core::ASIMode;

use std::path::PathBuf;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();
    
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║           SpatialVortex ASI - Autonomous Demo                 ║");
    println!("║                                                               ║");
    println!("║  This demo shows the ASI running autonomously:                ║");
    println!("║  • Perceiving environment through sensors                     ║");
    println!("║  • Reasoning using flux mathematics                           ║");
    println!("║  • Taking actions through actuators                           ║");
    println!("║  • Learning from outcomes                                     ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    
    // Get current directory for project path
    let project_path = std::env::current_dir()?;
    
    // Build the ASI with development configuration
    println!("🔧 Initializing ASI Core...");
    
    let asi = ASIBuilder::new()
        .with_storage_path(project_path.join(".asi_demo_data"))
        .with_loop_interval(2000)  // 2 second loop for demo visibility
        .with_time_sensor(true)
        .with_file_watching(vec![project_path.join("src")])
        .with_process_monitoring(vec![
            "cargo".to_string(),
            "rust-analyzer".to_string(),
        ])
        .with_shell_access(project_path.to_str().unwrap())
        .with_shell_timeout(10)
        .with_filesystem_access(vec![project_path.clone()])
        .with_autonomous_goals(true)
        .with_default_goals()
        .with_goal("Explore the codebase and understand its structure", GoalPriority::Medium)
        .with_goal("Identify potential improvements", GoalPriority::Low)
        .build()
        .await?;
    
    println!("✅ ASI Core initialized");
    println!();
    
    // Display initial state
    let state = asi.get_state().await;
    println!("📊 Initial State:");
    println!("   Mode: {:?}", state.mode);
    println!("   Cycle: {}", state.cycle_count);
    println!("   Confidence: {:.1}%", state.confidence * 100.0);
    println!();
    
    // Start the autonomous loop in a background task
    println!("🚀 Starting autonomous loop...");
    println!("   Press Ctrl+C to stop");
    println!();
    
    let asi_clone = std::sync::Arc::new(asi);
    let asi_for_loop = asi_clone.clone();
    
    // Spawn the autonomous loop
    let loop_handle = tokio::spawn(async move {
        if let Err(e) = asi_for_loop.start().await {
            tracing::error!("ASI loop error: {}", e);
        }
    });
    
    // Monitor and display status
    let asi_for_monitor = asi_clone.clone();
    let monitor_handle = tokio::spawn(async move {
        let mut last_cycle = 0u64;
        
        loop {
            sleep(Duration::from_secs(5)).await;
            
            let state = asi_for_monitor.get_state().await;
            let stats = asi_for_monitor.get_stats().await;
            
            if state.cycle_count > last_cycle {
                println!("─────────────────────────────────────────────");
                println!("📈 Status Update (Cycle {})", state.cycle_count);
                println!("   Mode: {:?}", state.mode);
                println!("   Observations: {}", stats.observations_processed);
                println!("   Thoughts: {}", stats.thoughts_generated);
                println!("   Actions: {}", stats.actions_taken);
                println!("   Goals Completed: {}", stats.goals_completed);
                println!("   Confidence: {:.1}%", state.confidence * 100.0);
                
                if let Some(goal_id) = state.current_goal {
                    println!("   Active Goal: {}", goal_id);
                }
                
                last_cycle = state.cycle_count;
            }
        }
    });
    
    // Handle Ctrl+C gracefully
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!();
            println!("🛑 Stopping ASI...");
            asi_clone.stop();
        }
        _ = loop_handle => {
            println!("ASI loop ended");
        }
    }
    
    // Final statistics
    let final_stats = asi_clone.get_stats().await;
    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("📊 Final Statistics:");
    println!("   Total Cycles: {}", final_stats.total_cycles);
    println!("   Observations Processed: {}", final_stats.observations_processed);
    println!("   Thoughts Generated: {}", final_stats.thoughts_generated);
    println!("   Actions Taken: {}", final_stats.actions_taken);
    println!("   Goals Completed: {}", final_stats.goals_completed);
    println!("   Errors: {}", final_stats.errors_encountered);
    println!("═══════════════════════════════════════════════════════════════");
    
    Ok(())
}
