/// Integration test: OLD prompts vs ENHANCED prompts
/// Measures ACTUAL performance difference, not expectations

use spatial_vortex::agents::{CodingAgent, AgentConfig, Language, LLMConfig, LLMBackend};

/// Test configuration for baseline (OLD prompts)
fn config_baseline() -> AgentConfig {
    AgentConfig {
        use_enhanced_prompts: false,  // OLD prompts
        ..Default::default()
    }
}

/// Test configuration for enhanced (NEW prompts)
fn config_enhanced() -> AgentConfig {
    AgentConfig {
        use_enhanced_prompts: true,  // NEW prompts
        ..Default::default()
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test --test prompt_enhancement_integration_test -- --ignored
async fn test_two_sum_baseline_vs_enhanced() {
    let task = "Write a Python function called two_sum that takes a list of integers and a target integer. \
                Return indices of two numbers that sum to target.";
    
    // OLD prompts
    let agent_old = CodingAgent::with_config(config_baseline());
    let result_old = agent_old.execute_task(task).await;
    
    // NEW prompts
    let agent_new = CodingAgent::with_config(config_enhanced());
    let result_new = agent_new.execute_task(task).await;
    
    // Compare
    println!("\n=== TWO SUM COMPARISON ===");
    println!("OLD prompts: {:?}", result_old.is_ok());
    println!("NEW prompts: {:?}", result_new.is_ok());
    
    if let Ok(res) = &result_new {
        assert!(res.code.contains("def two_sum"), "Missing function definition");
        if let Some(exec) = &res.execution {
            assert_eq!(exec.exit_code, 0, "Execution failed");
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_edit_distance_baseline_vs_enhanced() {
    let task = "Write a Python function called min_distance that finds minimum edit distance \
                between two strings using dynamic programming.";
    
    let agent_old = CodingAgent::with_config(config_baseline());
    let result_old = agent_old.execute_task(task).await;
    
    let agent_new = CodingAgent::with_config(config_enhanced());
    let result_new = agent_new.execute_task(task).await;
    
    println!("\n=== EDIT DISTANCE COMPARISON ===");
    println!("OLD prompts: {:?}", result_old.is_ok());
    println!("NEW prompts: {:?}", result_new.is_ok());
    
    // NEW should succeed where OLD failed
    if result_old.is_err() {
        assert!(result_new.is_ok(), "Enhanced prompts should fix this problem");
    }
}

#[tokio::test]
#[ignore]
async fn test_three_sum_baseline_vs_enhanced() {
    let task = "Write a Python function called three_sum that finds all unique triplets \
                in array that sum to zero. Use two pointers approach.";
    
    let agent_old = CodingAgent::with_config(config_baseline());
    let result_old = agent_old.execute_task(task).await;
    
    let agent_new = CodingAgent::with_config(config_enhanced());
    let result_new = agent_new.execute_task(task).await;
    
    println!("\n=== THREE SUM COMPARISON ===");
    println!("OLD prompts: {:?}", result_old.is_ok());
    println!("NEW prompts: {:?}", result_new.is_ok());
    
    // Both should succeed (this was already 100%)
    assert!(result_new.is_ok(), "Enhanced should maintain performance");
}

/// Full benchmark suite
#[tokio::test]
#[ignore]
async fn full_benchmark_suite() {
    let tests = vec![
        ("Two Sum (Easy)", 
         "Write a Python function called two_sum that takes a list and target, returns indices that sum to target"),
        
        ("Valid Parentheses (Easy)",
         "Write a Python function called is_valid that checks if a string of parentheses is valid using a stack"),
        
        ("3Sum (Medium)",
         "Write a Python function called three_sum that finds all unique triplets that sum to zero using two pointers"),
        
        ("Edit Distance (Hard)",
         "Write a Python function called min_distance that finds minimum edit distance using dynamic programming"),
        
        ("Binary Search (Easy)",
         "Write a Python function called binary_search that performs binary search on sorted array"),
    ];
    
    let mut old_passed = 0;
    let mut new_passed = 0;
    
    println!("\n=== FULL BENCHMARK SUITE ===\n");
    
    for (name, task) in &tests {
        println!("Testing: {}", name);
        
        // OLD
        let agent_old = CodingAgent::with_config(config_baseline());
        let result_old = agent_old.execute_task(task).await;
        if result_old.is_ok() {
            old_passed += 1;
            println!("  OLD: PASS");
        } else {
            println!("  OLD: FAIL");
        }
        
        // NEW
        let agent_new = CodingAgent::with_config(config_enhanced());
        let result_new = agent_new.execute_task(task).await;
        if result_new.is_ok() {
            new_passed += 1;
            println!("  NEW: PASS");
        } else {
            println!("  NEW: FAIL");
        }
        
        println!();
    }
    
    let total = tests.len();
    let old_rate = (old_passed as f64 / total as f64) * 100.0;
    let new_rate = (new_passed as f64 / total as f64) * 100.0;
    let improvement = new_rate - old_rate;
    
    println!("=== RESULTS ===");
    println!("OLD: {} / {} ({:.1}%)", old_passed, total, old_rate);
    println!("NEW: {} / {} ({:.1}%)", new_passed, total, new_rate);
    println!("IMPROVEMENT: {:+.1}%", improvement);
    
    // Assert improvement
    assert!(new_passed >= old_passed, "Enhanced prompts should not regress");
    assert!(improvement >= 0.0, "Should show improvement or maintain performance");
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use spatial_vortex::agents::{Difficulty, PromptTemplate};
    
    #[test]
    fn test_difficulty_detection() {
        let agent = CodingAgent::with_config(config_enhanced());
        
        // Easy
        let easy = agent.detect_difficulty("Write a function to reverse a string");
        assert_eq!(easy, Difficulty::Easy);
        
        // Medium
        let medium = agent.detect_difficulty("Use two pointers to solve this problem");
        assert_eq!(medium, Difficulty::Medium);
        
        // Hard
        let hard = agent.detect_difficulty("Use dynamic programming to find edit distance");
        assert_eq!(hard, Difficulty::Hard);
    }
    
    #[test]
    fn test_algorithm_hint_detection() {
        let agent = CodingAgent::with_config(config_enhanced());
        
        // Hash table
        let hint = agent.detect_algorithm_hint("find two numbers that sum to target");
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("hash table"));
        
        // Two pointers
        let hint = agent.detect_algorithm_hint("find three numbers that sum to zero triplet");
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("two pointers"));
        
        // DP
        let hint = agent.detect_algorithm_hint("calculate edit distance between strings");
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("DP"));
    }
    
    #[test]
    fn test_enhanced_prompt_generation() {
        let agent = CodingAgent::with_config(config_enhanced());
        
        let prompt = agent.build_enhanced_prompt(
            "Write a function to find two sum",
            Language::Python,
            None,
        );
        
        // Should contain structured components
        assert!(prompt.contains("# Task"));
        assert!(prompt.contains("# Language"));
        assert!(prompt.contains("# Recommended Approach") || prompt.contains("hash table"));
        assert!(prompt.contains("# Language-Specific Guidelines"));
    }
}
