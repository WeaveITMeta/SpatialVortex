//! Consensus Validation Demo
//!
//! Demonstrates how consensus validation prevents low-quality training tasks:
//! 1. Generate tasks with multiple LLMs
//! 2. Validate quality via consensus
//! 3. Reject tasks that don't meet quality standards
//! 4. Prevent holes in training data

use spatial_vortex::asi::{
    AITaskGenerator, TaskGenerationStrategy, TaskCategory,
    ConsensusTaskValidator,
};
use spatial_vortex::ai::consensus::{AIConsensusEngine, ConsensusStrategy};
use spatial_vortex::error::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    println!("=== Consensus Validation Demo ===\n");
    println!("Preventing holes in training data with multi-LLM validation\n");
    
    // Step 1: Create consensus engine
    println!("🤝 Step 1: Creating Consensus Engine...");
    let consensus_engine = Arc::new(AIConsensusEngine::new(
        ConsensusStrategy::WeightedConfidence,
        2, // Minimum 2 models must agree
        30, // 30 second timeout
    ));
    println!("✓ Consensus engine created");
    println!("  Strategy: WeightedConfidence");
    println!("  Min models: 2");
    println!();
    
    // Step 2: Create AI task generator with consensus validation
    println!("🎯 Step 2: Creating AI Task Generator with Consensus...");
    let generator = AITaskGenerator::new(TaskGenerationStrategy::ConsensusValidation)
        .with_external_llm("http://localhost:11434".to_string())
        .with_consensus_engine(consensus_engine.clone())
        .with_consensus_models(vec![
            "llama3".to_string(),
            "mistral".to_string(),
            "codellama".to_string(),
        ])
        .with_min_consensus_score(0.7); // 70% agreement required
    
    println!("✓ Generator configured");
    println!("  Validation models: llama3, mistral, codellama");
    println!("  Min consensus score: 70%");
    println!();
    
    // Step 3: Generate and validate tasks
    println!("📋 Step 3: Generating Tasks with Consensus Validation\n");
    
    let mut validated_tasks = 0;
    let mut rejected_tasks = 0;
    
    for i in 0..5 {
        println!("  Task {}:", i + 1);
        
        match generator.generate_task(&TaskCategory::CodeGeneration, i).await {
            Ok(task) => {
                println!("    Description: {}", task.description);
                println!("    Difficulty: {}/10", task.difficulty);
                
                if let Some(consensus_score) = task.metadata.consensus_score {
                    println!("    Consensus score: {:.1}%", consensus_score * 100.0);
                }
                
                if let Some(models_agreed) = task.metadata.models_agreed {
                    println!("    Models agreed: {}/3", models_agreed);
                }
                
                if task.metadata.quality_validated {
                    println!("    ✅ VALIDATED - High quality, suitable for training");
                    validated_tasks += 1;
                } else {
                    println!("    ⚠️  NOT VALIDATED - Quality concerns");
                    rejected_tasks += 1;
                }
            }
            Err(e) => {
                println!("    ❌ REJECTED - {}", e);
                rejected_tasks += 1;
            }
        }
        println!();
    }
    
    println!("  Results:");
    println!("    Validated: {}", validated_tasks);
    println!("    Rejected: {}", rejected_tasks);
    println!();
    
    // Step 4: Show validation process
    println!("🔍 Step 4: Validation Process Breakdown\n");
    
    println!("  For each generated task:");
    println!("    1. Generate task with AI (our model or external LLM)");
    println!("    2. Send to 3 validator LLMs (llama3, mistral, codellama)");
    println!("    3. Each validator scores:");
    println!("       - Realism (0-10)");
    println!("       - Specificity (0-10)");
    println!("       - Difficulty match (0-10)");
    println!("       - Training value (0-10)");
    println!("    4. Consensus engine aggregates scores");
    println!("    5. Accept if consensus ≥ 70% and quality ≥ 7/10");
    println!("    6. Reject and regenerate if below threshold");
    println!();
    
    // Step 5: Compare with non-validated approach
    println!("⚖️  Step 5: Validation vs No Validation\n");
    
    println!("  WITHOUT Consensus Validation:");
    println!("    ❌ May accept low-quality tasks");
    println!("    ❌ Unrealistic scenarios slip through");
    println!("    ❌ Vague or generic tasks");
    println!("    ❌ Holes in training data");
    println!("    ❌ Poor AI performance after training");
    println!();
    
    println!("  WITH Consensus Validation:");
    println!("    ✅ Only high-quality tasks accepted");
    println!("    ✅ Multiple LLMs verify realism");
    println!("    ✅ Specific, detailed tasks");
    println!("    ✅ Comprehensive training coverage");
    println!("    ✅ Better AI performance");
    println!();
    
    // Step 6: Quality metrics
    println!("📊 Step 6: Quality Metrics\n");
    
    println!("  Validation Dimensions:");
    println!("    • Realism: Is this a real-world task?");
    println!("    • Specificity: Is it detailed enough?");
    println!("    • Difficulty Match: Does rating match complexity?");
    println!("    • Training Value: Will this improve the AI?");
    println!();
    
    println!("  Consensus Scoring:");
    println!("    • 90-100%: Strong agreement, high confidence");
    println!("    • 70-90%: Good agreement, acceptable");
    println!("    • 50-70%: Weak agreement, borderline");
    println!("    • <50%: No consensus, reject");
    println!();
    
    // Step 7: Benefits
    println!("🎯 Step 7: Benefits of Consensus Validation\n");
    
    println!("  1. **Prevents Low-Quality Tasks**");
    println!("     - Multiple LLMs must agree task is good");
    println!("     - Catches unrealistic or vague tasks");
    println!();
    
    println!("  2. **Ensures Training Data Quality**");
    println!("     - No holes from bad tasks");
    println!("     - Comprehensive coverage");
    println!("     - Realistic scenarios only");
    println!();
    
    println!("  3. **Redundant Validation**");
    println!("     - Our model + 3 external LLMs");
    println!("     - Different perspectives");
    println!("     - Catches edge cases");
    println!();
    
    println!("  4. **Automatic Quality Control**");
    println!("     - No manual review needed");
    println!("     - Consistent standards");
    println!("     - Scalable validation");
    println!();
    
    println!("  5. **Improved Training Outcomes**");
    println!("     - Better task quality → better AI");
    println!("     - Fewer failure modes");
    println!("     - Higher success rates");
    println!();
    
    // Summary
    println!("=== Summary ===\n");
    
    println!("🔒 Consensus Validation Architecture:");
    println!();
    println!("  Task Generation:");
    println!("    Our AI Model → Generate Task");
    println!("         ↓");
    println!("  Consensus Validation:");
    println!("    llama3 → Score & Validate");
    println!("    mistral → Score & Validate");
    println!("    codellama → Score & Validate");
    println!("         ↓");
    println!("  Consensus Engine:");
    println!("    Aggregate Scores → Weighted Confidence");
    println!("         ↓");
    println!("  Decision:");
    println!("    If consensus ≥ 70% → ACCEPT");
    println!("    If consensus < 70% → REJECT & REGENERATE");
    println!();
    
    println!("✅ Result:");
    println!("   Only high-quality, validated tasks used for training");
    println!("   No holes in training data");
    println!("   Better AI performance");
    println!();
    
    println!("🚀 Consensus Validation: COMPLETE");
    
    Ok(())
}
