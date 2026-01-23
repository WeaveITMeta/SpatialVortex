//! RAG + Web Search Integration Demo
//!
//! Demonstrates the complete Phase 2 integration:
//! - Web search (DuckDuckGo default)
//! - Storage in vector database
//! - Combined with local RAG retrieval
//! - Full source attribution
//!
//! Run with:
//! ```bash
//! cargo run --example rag_web_integration_demo --features agents
//! ```

use spatial_vortex::rag::{
    augmentation::{AugmentedGenerator, GenerationConfig, ContextIntegration},
    retrieval::{RAGRetriever, RetrievalConfig},
    vector_store::VectorStore,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use spatial_vortex::ai::orchestrator::ASIOrchestrator;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔗 RAG + Web Search Integration Demo\n");
    println!("Phase 2: Complete RAG Integration\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Initialize vector store (in-memory for demo)
    println!("1️⃣  Initializing vector store...");
    let vector_store = Arc::new(VectorStore::new(384));
    println!("   ✅ Vector store ready (384 dimensions)\n");
    
    // Create retriever
    println!("2️⃣  Creating RAG retriever...");
    let retrieval_config = RetrievalConfig {
        top_k: 20,
        rerank_top_n: 5,
        min_similarity: 0.5,
        min_confidence: 0.6,
        use_sacred_filtering: true,
        diversity_factor: 0.3,
        context_window: 2048,
        sacred_weight: 1.0,
    };
    let retriever = Arc::new(RAGRetriever::new(vector_store.clone(), retrieval_config));
    println!("   ✅ Retriever configured (top-k=5, sacred filtering enabled)\n");
    
    // Configure generation with web search
    println!("3️⃣  Configuring augmented generator...");
    let gen_config = GenerationConfig {
        max_length: 512,
        temperature: 0.7,
        use_sacred_guidance: true,
        hallucination_check: true,
        context_integration: ContextIntegration::Hierarchical,
        enable_web_search: true,  // Enable web search!
        max_web_sources: 5,       // Limit to 5 sources
    };
    
    let orchestrator = Arc::new(Mutex::new(ASIOrchestrator::new().await?));
    let mut generator = AugmentedGenerator::new(retriever, orchestrator, gen_config).await?;
    println!("   ✅ Generator ready (web search enabled, DuckDuckGo default)\n");
    
    // Test queries
    let queries = vec![
        "What is Rust programming language?",
        "Explain artificial intelligence ethics",
    ];
    
    for (i, query) in queries.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📝 Query {}: {}", i + 1, query);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        println!("⏳ Generating with RAG + Web Search...");
        
        match generator.generate(query).await {
            Ok(result) => {
                println!("✅ Generation complete!\n");
                
                // Show metrics
                println!("📊 **Metrics**:");
                println!("   Confidence: {:.1}%", result.confidence * 100.0);
                println!("   Hallucination Risk: {:.1}%", result.hallucination_risk * 100.0);
                println!("   Flux Position: {}", result.flux_position);
                println!("   Total Sources: {}\n", result.sources.len());
                
                // Show response preview
                println!("💬 **Response Preview**:");
                let preview = if result.response.len() > 200 {
                    format!("{}...", &result.response[..200])
                } else {
                    result.response.clone()
                };
                println!("{}\n", preview);
                
                // Show sources breakdown
                let web_sources: Vec<_> = result.sources.iter()
                    .filter(|s| s.web_source.is_some())
                    .collect();
                let local_sources: Vec<_> = result.sources.iter()
                    .filter(|s| s.web_source.is_none())
                    .collect();
                
                println!("📚 **Sources Breakdown**:");
                println!("   🌐 Web Sources: {}", web_sources.len());
                println!("   📄 Local Sources: {}", local_sources.len());
                println!();
                
                // Show web sources detail
                if !web_sources.is_empty() {
                    println!("🌐 **Web Sources**:");
                    for (idx, source) in web_sources.iter().enumerate() {
                        if let Some(ref web) = source.web_source {
                            println!("\n   {}. {} ({:.0}% credible)", 
                                idx + 1,
                                web.title,
                                web.credibility_score * 100.0
                            );
                            println!("      🔗 {}", web.url);
                            println!("      🏷️  Type: {} | Engine: {}", 
                                web.source_type,
                                web.search_engine
                            );
                            println!("      📝 {}",
                                if source.content_snippet.len() > 80 {
                                    format!("{}...", &source.content_snippet[..80])
                                } else {
                                    source.content_snippet.clone()
                                }
                            );
                        }
                    }
                    println!();
                }
                
                // Show local sources
                if !local_sources.is_empty() {
                    println!("📄 **Local Sources**:");
                    for (idx, source) in local_sources.iter().enumerate() {
                        println!("\n   {}. Document: {}", idx + 1, source.doc_id);
                        println!("      Chunk: {}", source.chunk_id);
                        println!("      Relevance: {:.1}%", source.relevance * 100.0);
                    }
                    println!();
                }
                
                // Show context usage
                println!("📊 **Context Statistics**:");
                println!("   Total context chunks: {}", result.context_used.len());
                let total_chars: usize = result.context_used.iter()
                    .map(|c| c.len())
                    .sum();
                println!("   Total context size: {} characters", total_chars);
                println!();
            }
            Err(e) => {
                println!("❌ Generation failed: {}\n", e);
            }
        }
    }
    
    // Show vector database statistics
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 **Vector Database Statistics**");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let stats = vector_db.get_stats().await?;
    println!("   Total Embeddings: {}", stats.total_embeddings);
    println!("   Dimension: {}", stats.dimension);
    println!("   Sacred Geometry: {}", if stats.use_sacred_geometry { "Enabled" } else { "Disabled" });
    println!("   At Position 3: {}", stats.position_counts.get(&3).unwrap_or(&0));
    println!("   At Position 6: {}", stats.position_counts.get(&6).unwrap_or(&0));
    println!("   At Position 9: {}", stats.position_counts.get(&9).unwrap_or(&0));
    println!();
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✨ Demo Complete!\n");
    
    println!("💡 **Key Features Demonstrated**:");
    println!("   ✅ Web search integration (DuckDuckGo)");
    println!("   ✅ Automatic vector storage");
    println!("   ✅ Source deduplication");
    println!("   ✅ Full source attribution");
    println!("   ✅ Credibility scoring");
    println!("   ✅ Sacred geometry placement");
    println!("   ✅ Combined local + web context");
    println!();
    
    println!("🚀 **Next Steps**:");
    println!("   Phase 3: Frontend display (SourcesPanel.svelte)");
    println!("   Phase 4: Advanced features (fact-checking, temporal tracking)");
    println!();
    
    Ok(())
}
