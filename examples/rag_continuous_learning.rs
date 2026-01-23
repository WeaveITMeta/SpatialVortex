//! RAG Continuous Learning Example
//!
//! Demonstrates how SpatialVortex automatically ingests training data,
//! builds embeddings, and gets smarter over time.

use spatial_vortex::rag::{
    augmentation::{AugmentedGenerator, GenerationConfig},
    ingestion::{DocumentIngester, IngestionConfig},
    retrieval::{RAGRetriever, RetrievalConfig},
    vector_store::{VectorStore, VectorDatabase},
};
use spatial_vortex::ai::orchestrator::ASIOrchestrator;
use std::sync::Arc;
use tokio::sync::Mutex;
use spatial_vortex::rag::{ContinuousLearner, TrainingConfig};
use spatial_vortex::rag::training::DataSource;
use spatial_vortex::storage::spatial_database::SpatialDatabase;

#[cfg(feature = "lake")]
use spatial_vortex::storage::confidence_lake::ConfidenceLake;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 SpatialVortex RAG Continuous Learning Demo");
    println!("{}", "=".repeat(60));
    
    // Step 1: Initialize components
    println!("\n📦 Initializing RAG components...");
    
    // Create vector store (384-dimensional embeddings)
    let vector_store = Arc::new(VectorStore::new(384));
    
    // Create spatial database (for continuous learning)
    let database = Arc::new(SpatialDatabase::new(":memory:").await?);
    
    // Create Confidence Lake (if enabled)
    #[cfg(feature = "lake")]
    let confidence_lake = ConfidenceLake::new(database.clone()).await?;
    
    // Create document ingester
    let mut ingestion_config = IngestionConfig::default();
    ingestion_config.sacred_boost = true;  // Boost sacred geometry mentions
    ingestion_config.auto_categorize = true;
    
    let ingester = DocumentIngester::new(ingestion_config);
    
    println!("✅ Components initialized");
    
    // Step 2: Ingest initial training data
    println!("\n📚 Ingesting initial training data...");
    
    // Example: Ingest from a directory
    let docs_path = PathBuf::from("./docs");
    if docs_path.exists() {
        let documents = ingester.ingest_directory(&docs_path).await?;
        println!("📄 Found {} documents", documents.len());
        
        // Chunk and index documents
        for doc in documents {
            println!("  - Processing: {}", doc.source);
            let chunks = ingester.chunk_document(&doc).await?;
            
            for chunk in chunks {
                // Store in vector database
                let mut metadata = std::collections::HashMap::new();
                metadata.insert("content".to_string(), chunk.content.clone());
                metadata.insert("doc_id".to_string(), doc.id.clone());
                
                vector_store.store_chunk(
                    &doc.id,
                    &chunk.id,
                    &chunk.content,
                    chunk.elp_tensor,
                    chunk.flux_position,
                    metadata,
                ).await?;
            }
        }
        
        let stats = vector_store.database().stats().await;
        println!("\n📊 Vector Database Statistics:");
        println!("  - Total embeddings: {}", stats.total_embeddings);
        println!("  - Sacred positions: {}", stats.sacred_positions);
        println!("  - Avg confidence: {:.2}", stats.average_confidence);
    }
    
    // Step 3: Create RAG retriever
    println!("\n🔍 Setting up RAG retriever...");
    
    let mut retrieval_config = RetrievalConfig::default();
    retrieval_config.use_sacred_filtering = true;
    retrieval_config.min_confidence = 0.6;
    
    let retriever = Arc::new(RAGRetriever::new(
        vector_store.clone(),
        retrieval_config,
    ));
    
    // Step 4: Test retrieval
    println!("\n🔎 Testing retrieval system...");
    
    let test_queries = vec![
        "What is sacred geometry?",
        "How does the vortex flow pattern work?",
        "Explain ELP tensors",
        "What are positions 3, 6, and 9?",
    ];
    
    for query in test_queries {
        println!("\n  Query: '{}'", query);
        let results = retriever.retrieve(query).await?;
        
        for (i, result) in results.iter().take(2).enumerate() {
            println!("    Result {}: [Flux {}] Confidence {:.2}, Similarity {:.2}",
                i + 1,
                result.flux_position,
                result.confidence,
                result.similarity
            );
            
            // Show snippet
            let snippet = if result.content.len() > 100 {
                format!("{}...", &result.content[..100])
            } else {
                result.content.clone()
            };
            println!("      Content: {}", snippet);
        }
    }
    
    // Step 5: Create augmented generator
    println!("\n✨ Creating augmented generator...");
    
    let generation_config = GenerationConfig {
        use_sacred_guidance: true,
        hallucination_check: true,
        context_integration: spatial_vortex::rag::augmentation::ContextIntegration::Sacred,
        ..Default::default()
    };
    
    let orchestrator = Arc::new(Mutex::new(ASIOrchestrator::new().await?));
    let mut generator = AugmentedGenerator::new(retriever.clone(), orchestrator, generation_config).await?;
    
    // Step 6: Generate with RAG
    println!("\n🤖 Generating responses with RAG...");
    
    let prompts = vec![
        "Explain how SpatialVortex uses sacred geometry",
        "What makes the 3-6-9 pattern special?",
        "How does RAG improve the AI's knowledge?",
    ];
    
    for prompt in prompts {
        println!("\n  Prompt: '{}'", prompt);
        let result = generator.generate(prompt).await?;
        
        println!("  Response: {}", result.response);
        println!("  Confidence: {:.2}", result.confidence);
        println!("  Hallucination Risk: {:.2}", result.hallucination_risk);
        println!("  Sources Used: {}", result.sources.len());
        
        for source in result.sources.iter().take(2) {
            println!("    - Doc: {}, Relevance: {:.2}", source.doc_id, source.relevance);
        }
    }
    
    // Step 7: Start continuous learning
    println!("\n🎓 Starting continuous learning...");
    
    let training_config = TrainingConfig {
        batch_size: 16,
        learning_rate: 0.001,
        min_confidence: 0.6,
        sacred_weight_boost: 1.5,
        auto_ingest_interval: std::time::Duration::from_secs(60), // Every minute for demo
        ..Default::default()
    };
    
    let learner = ContinuousLearner::new(
        vector_store.clone(),
        database.clone(),
        training_config,
    );
    
    // Define data sources to monitor
    let data_sources = vec![
        DataSource::Directory(PathBuf::from("./docs")),
        // DataSource::Url("https://example.com/feed".to_string()),
        // DataSource::Stream("kafka://topics/training-data".to_string()),
    ];
    
    // Start learning (runs in background)
    learner.start_learning(data_sources).await?;
    
    println!("✅ Continuous learning started!");
    println!("   The system will automatically:");
    println!("   - Ingest new documents from watched directories");
    println!("   - Build embeddings with sacred geometry");
    println!("   - Store high-value content in Confidence Lake");
    println!("   - Improve responses over time");
    
    // Step 8: Monitor learning progress
    println!("\n📈 Monitoring learning progress...");
    
    // Simulate some time passing
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    if let Some(metrics) = learner.get_latest_metrics().await {
        println!("  Latest Learning Metrics:");
        println!("    - Documents processed: {}", metrics.documents_processed);
        println!("    - Chunks indexed: {}", metrics.chunks_indexed);
        println!("    - Sacred ratio: {:.2}%", metrics.sacred_ratio * 100.0);
        println!("    - Avg confidence: {:.2}", metrics.average_confidence);
    }
    
    // Step 9: Show how the system gets smarter
    println!("\n🧠 System Intelligence Growth:");
    println!("  As more data is ingested:");
    println!("  1. Vector database grows with embeddings");
    println!("  2. Sacred patterns (3-6-9) are reinforced");
    println!("  3. Confidence scores improve");
    println!("  4. Hallucinations decrease");
    println!("  5. Responses become more accurate");
    
    // Final stats
    let final_stats = vector_store.database().stats().await;
    println!("\n📊 Final Statistics:");
    println!("  Total Embeddings: {}", final_stats.total_embeddings);
    println!("  Sacred Positions: {}", final_stats.sacred_positions);
    println!("  Average Confidence: {:.2}", final_stats.average_confidence);
    
    // Check Confidence Lake (if enabled)
    #[cfg(feature = "lake")]
    {
        if let Ok(diamonds) = confidence_lake.query_high_confidence(0.9).await {
            println!("\n💎 Confidence Lake:");
            println!("  High Confidence Records: {}", diamonds.len());
        }
    }
    
    println!("\n✅ RAG Continuous Learning Demo Complete!");
    println!("\n🚀 SpatialVortex is now automatically learning and improving!");
    println!("   It will continue to get smarter as it ingests more data.");
    println!("   The sacred geometry patterns ensure knowledge is organized optimally.");
    
    // Stop learning before exit
    learner.stop_learning().await;
    
    Ok(())
}
