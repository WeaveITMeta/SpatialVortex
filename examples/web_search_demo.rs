//! Multi-Source Web Search Demo
//!
//! Demonstrates DuckDuckGo (free) + optional Brave API web search
//! with credibility scoring and source aggregation.
//!
//! Run with:
//! ```bash
//! cargo run --example web_search_demo --features agents
//! ```

use spatial_vortex::ai::multi_source_search::{
    MultiSourceSearcher, SearchConfig, SearchEngine,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 Multi-Source Web Search Demo\n");
    
    // Configure search engines
    // DuckDuckGo is always available (no API key needed)
    // Brave will be added automatically if BRAVE_API_KEY is set
    let mut engines = vec![SearchEngine::DuckDuckGo];
    
    if std::env::var("BRAVE_API_KEY").is_ok() {
        println!("✅ Brave API key detected - adding Brave Search");
        engines.insert(0, SearchEngine::Brave);
    } else {
        println!("ℹ️  No Brave API key - using DuckDuckGo only (FREE!)");
        println!("   Get Brave key at: https://brave.com/search/api/\n");
    }
    
    let config = SearchConfig {
        max_sources: 10,
        engines,
        timeout_secs: 10,
        min_credibility: 0.4,
    };
    
    // Create searcher
    let searcher = MultiSourceSearcher::new(config)?;
    
    // Example queries
    let queries = vec![
        "What is Rust programming language?",
        "Sacred geometry in mathematics",
        "Artificial intelligence ethics",
    ];
    
    for (i, query) in queries.iter().enumerate() {
        println!("─────────────────────────────────────────────────");
        println!("Query {}: {}", i + 1, query);
        println!("─────────────────────────────────────────────────\n");
        
        match searcher.search(query).await {
            Ok(result) => {
                println!("✅ Found {} sources in {}ms", 
                    result.total_results, 
                    result.search_time_ms
                );
                println!("🎯 Overall confidence: {:.1}%\n", result.confidence * 100.0);
                
                // Show top 3 sources
                println!("Top Sources:");
                for (idx, source) in result.sources.iter().take(3).enumerate() {
                    println!("\n{}. 📊 {} ({:.0}% credible)", 
                        idx + 1,
                        source.title,
                        source.credibility_score * 100.0
                    );
                    println!("   🔗 {}", source.url);
                    println!("   📝 {}", 
                        if source.snippet.len() > 100 {
                            format!("{}...", &source.snippet[..100])
                        } else {
                            source.snippet.clone()
                        }
                    );
                    println!("   🏷️  Type: {:?} | Engine: {}", 
                        source.source_type,
                        source.search_engine
                    );
                }
                
                println!("\n📋 Aggregated Answer:");
                let answer_preview = if result.aggregated_answer.len() > 300 {
                    format!("{}...", &result.aggregated_answer[..300])
                } else {
                    result.aggregated_answer.clone()
                };
                println!("{}\n", answer_preview);
            }
            Err(e) => {
                println!("❌ Search failed: {}\n", e);
            }
        }
    }
    
    println!("─────────────────────────────────────────────────");
    println!("✨ Demo complete!\n");
    println!("💡 Tips:");
    println!("   • DuckDuckGo works immediately (no setup)");
    println!("   • Add BRAVE_API_KEY to .env for better results");
    println!("   • API endpoint: POST /api/v1/rag/web-search");
    println!("─────────────────────────────────────────────────");
    
    Ok(())
}
