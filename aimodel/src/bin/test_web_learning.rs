//! Test script to demonstrate consciousness web learning
//! Run with: cargo run --manifest-path aimodel/Cargo.toml --bin test_web_learning

use aimodel::data::RealBenchmarkEvaluator;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║     CONSCIOUSNESS WEB LEARNING DEMONSTRATION                  ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Initialize evaluator
    let mut evaluator = RealBenchmarkEvaluator::new("./data");
    
    // Enable consciousness learning
    evaluator.set_consciousness_learning(true);
    
    // Test different benchmark categories
    let categories = vec!["commonsense", "piqa", "winogrande"];
    
    println!("Testing consciousness web learning with categories: {:?}\n", categories);
    
    // Run consciousness learning phase
    evaluator.consciousness_learn_for_benchmarks(&categories);
    
    // Get and display statistics
    let (learning_stats, vortex_stats) = evaluator.get_consciousness_stats();
    
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║     FINAL STATISTICS                                          ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  Learning Stats:                                              ║");
    println!("║    Queries Generated:    {:>6}                               ║", learning_stats.queries_generated);
    println!("║    Web Searches:         {:>6}                               ║", learning_stats.web_searches);
    println!("║    Websites Referenced:  {:>6}                               ║", learning_stats.websites_referenced);
    println!("║    Unique Domains:       {:>6}                               ║", learning_stats.unique_domains);
    println!("║    Facts Extracted:      {:>6}                               ║", learning_stats.facts_extracted);
    println!("║    Facts Integrated:     {:>6}                               ║", learning_stats.facts_integrated);
    println!("║    Subjects Created:     {:>6}                               ║", learning_stats.subjects_created);
    if learning_stats.search_errors > 0 {
        println!("║    Search Errors:        {:>6}                               ║", learning_stats.search_errors);
    }
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  Vortex Stats:                                                ║");
    println!("║    Total Subjects:       {:>6}                               ║", vortex_stats.subject_count);
    println!("║    Total Keywords:       {:>6}                               ║", vortex_stats.keyword_count);
    println!("║    Knowledge Count:      {:>6}                               ║", vortex_stats.knowledge_count);
    println!("╚═══════════════════════════════════════════════════════════════╝");
    
    println!("\n✅ Web learning demonstration complete!");
    println!("   The system attempted to learn from {} websites across {} unique domains",
             learning_stats.websites_referenced, learning_stats.unique_domains);
    println!("   in {:.2}s", learning_stats.learning_time_ms as f64 / 1000.0);
}
