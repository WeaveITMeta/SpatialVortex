/// Enhanced Prompt Template Demo
/// Demonstrates how improved prompts increase success rates

use spatial_vortex::agents::{
    Language, PromptTemplate, PromptTemplateLibrary, Difficulty,
};

fn main() {
    println!("=== Enhanced Prompt Template System Demo ===\n");
    
    // Example 1: Basic vs Enhanced for Easy Problem
    println!("1. TWO SUM - Basic vs Enhanced Prompts\n");
    println!("--- BASIC PROMPT (Led to 80% success on Easy) ---");
    let basic_prompt = "Write a Python function called two_sum that takes a list and target, returns indices that sum to target";
    println!("{}\n", basic_prompt);
    
    println!("--- ENHANCED PROMPT (Expected: 95%+ success) ---");
    let enhanced = PromptTemplateLibrary::two_sum(Language::Python);
    println!("{}\n", enhanced.build());
    
    println!("\n{}\n", "=".repeat(80));
    
    // Example 2: Hard Problem with DP Pattern
    println!("2. EDIT DISTANCE - DP Pattern Injection\n");
    println!("--- BASIC PROMPT (Led to failure) ---");
    let basic_dp = "Write a Python function called min_distance that finds minimum edit distance between two strings";
    println!("{}\n", basic_dp);
    
    println!("--- ENHANCED PROMPT with DP Template (Expected: Success) ---");
    let enhanced_dp = PromptTemplateLibrary::edit_distance(Language::Python);
    println!("{}\n", enhanced_dp.build());
    
    println!("\n{}\n", "=".repeat(80));
    
    // Example 3: Custom Template with Sacred Geometry
    println!("3. CUSTOM TEMPLATE - Binary Search with Sacred Position 9\n");
    let custom = PromptTemplate::new(
        "Write a function that performs binary search on a sorted array. \
         Return the index of the target element, or -1 if not found.".to_string(),
        Language::Python,
    )
    .with_difficulty(Difficulty::Medium)
    .with_algorithm_hint(
        "Use two pointers (left, right) to divide search space in half each iteration.\n\
         Compare middle element with target:\n\
         - If equal: return middle index\n\
         - If middle < target: search right half\n\
         - If middle > target: search left half\n\
         Continue until left > right.".to_string()
    )
    .with_example(
        "[1, 2, 3, 4, 5], target = 3".to_string(),
        "2".to_string()
    )
    .with_example_explained(
        "[1, 2, 3, 4, 5], target = 6".to_string(),
        "-1".to_string(),
        "6 is not in the array".to_string(),
    )
    .with_complexity(Some("O(log n)".to_string()), Some("O(1)".to_string()))
    .with_signature("def binary_search(arr: List[int], target: int) -> int:".to_string())
    .with_edge_case("Empty array".to_string())
    .with_edge_case("Single element array".to_string())
    .with_edge_case("Target not in array".to_string())
    .with_sacred_position(9);  // Optimize for logic (Logos)
    
    println!("{}\n", custom.build());
    
    println!("\n{}\n", "=".repeat(80));
    
    // Example 4: Multi-Language Example - Rust with Language-Specific Guidance
    println!("4. RUST QUICKSORT - Language-Specific Guidelines\n");
    let rust_template = PromptTemplate::new(
        "Write a function that implements the quicksort algorithm to sort a vector of integers in-place.".to_string(),
        Language::Rust,
    )
    .with_difficulty(Difficulty::Medium)
    .with_algorithm_hint(
        "1. Choose a pivot element (e.g., last element)\n\
         2. Partition array: elements < pivot on left, >= pivot on right\n\
         3. Recursively sort left and right partitions\n\
         4. Use mutable references for in-place sorting".to_string()
    )
    .with_signature(
        "pub fn quicksort(arr: &mut Vec<i32>, low: usize, high: usize) -> ()".to_string()
    )
    .with_example(
        "arr = &mut vec![3, 1, 4, 1, 5, 9, 2, 6]".to_string(),
        "arr becomes [1, 1, 2, 3, 4, 5, 6, 9]".to_string()
    )
    .with_complexity(Some("O(n log n) average".to_string()), Some("O(log n)".to_string()))
    .with_edge_case("Empty vector".to_string())
    .with_edge_case("Single element".to_string())
    .with_edge_case("Already sorted".to_string())
    .with_sacred_position(6);  // Robustness (Pathos)
    
    println!("{}\n", rust_template.build());
    
    println!("\n{}\n", "=".repeat(80));
    
    // Summary
    println!("KEY IMPROVEMENTS:");
    println!("✓ Algorithm hints for Medium+ difficulty");
    println!("✓ Concrete examples with explanations");
    println!("✓ Complexity requirements specified");
    println!("✓ Edge cases explicitly listed");
    println!("✓ Language-specific guidelines");
    println!("✓ Sacred geometry position guidance");
    println!("✓ Function signatures when needed");
    println!("\nEXPECTED IMPACT:");
    println!("  Easy:   80% → 95-98% (+15-18%)");
    println!("  Medium: 100% → 100% (maintained)");
    println!("  Hard:   33% → 70-80% (+37-47%)");
}
