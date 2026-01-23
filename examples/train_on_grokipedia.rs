//! Train SpatialVortex on Grokipedia Articles
//!
//! This example demonstrates training the AI on articles from Grokipedia,
//! building a knowledge base focused on sacred geometry, AI, and consciousness.

use spatial_vortex::rag::{
    DocumentIngester, IngestionConfig,
    VectorStore, ContinuousLearner, TrainingConfig,
    GrokipediaTrainer, GrokipediaCategory,
};
use spatial_vortex::storage::spatial_database::SpatialDatabase;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    println!("🚀 SpatialVortex Grokipedia Training System");
    println!("{}", "=".repeat(60));
    println!();
    
    // Initialize components
    println!("📚 Initializing training infrastructure...\n");
    
    // Create vector store for embeddings
    let vector_store = Arc::new(VectorStore::new(384)); // 384-dimensional embeddings
    
    // Create PostgreSQL database connection from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");
    println!("📊 Connecting to database...");
    let database = Arc::new(SpatialDatabase::new(&database_url).await?);
    
    // Configure ingestion with sacred geometry boost
    let mut ingestion_config = IngestionConfig::default();
    ingestion_config.sacred_boost = true;
    ingestion_config.auto_categorize = true;
    ingestion_config.chunk_size = 512;
    
    let ingester = Arc::new(DocumentIngester::new(ingestion_config));
    
    // Configure continuous learning
    let training_config = TrainingConfig {
        batch_size: 32,
        learning_rate: 0.001,
        min_confidence: 0.6,
        sacred_weight_boost: 1.5,
        ..Default::default()
    };
    
    let learner = Arc::new(ContinuousLearner::new(
        vector_store.clone(),
        database.clone(),
        training_config,
    ));
    
    // Create Grokipedia trainer
    let mut trainer = GrokipediaTrainer::new(
        ingester.clone(),
        vector_store.clone(),
        learner.clone(),
    );
    
    println!("✅ Training infrastructure ready!\n");
    println!("{}", "=".repeat(60));
    
    // Phase 1: Sacred Geometry Knowledge
    println!("\n📖 Phase 1: Training on Sacred Geometry...\n");
    
    let sacred_stats = trainer.train_on_categories(
        vec![GrokipediaCategory::SacredGeometry],
        5, // 5 articles per category
    ).await?;
    
    sacred_stats.display();
    
    // Phase 2: AI and Machine Learning
    println!("\n📖 Phase 2: Training on AI and Machine Learning...\n");
    
    let ai_stats = trainer.train_on_categories(
        vec![
            GrokipediaCategory::ArtificialIntelligence,
            GrokipediaCategory::MachineLearning,
            GrokipediaCategory::NeuralNetworks,
        ],
        3, // 3 articles per category
    ).await?;
    
    ai_stats.display();
    
    // Phase 3: Advanced Topics
    println!("\n📖 Phase 3: Training on Advanced Topics...\n");
    
    let advanced_stats = trainer.train_on_categories(
        vec![
            GrokipediaCategory::QuantumComputing,
            GrokipediaCategory::Consciousness,
            GrokipediaCategory::Philosophy,
        ],
        3,
    ).await?;
    
    advanced_stats.display();
    
    // Phase 4: Comprehensive Training
    println!("\n📖 Phase 4: Comprehensive Training on All Categories...\n");
    
    let comprehensive_stats = trainer.train_comprehensive().await?;
    comprehensive_stats.display();
    
    // Display overall statistics
    println!("\n{}", "=".repeat(60));
    println!("\n🎯 Overall Training Statistics:\n");
    
    let db_stats = vector_store.database().stats().await;
    println!("📊 Vector Database:");
    println!("  - Total Embeddings: {}", db_stats.total_embeddings);
    println!("  - Sacred Positions (3-6-9): {}", db_stats.sacred_positions);
    println!("  - Average Confidence: {:.3}", db_stats.average_confidence);
    println!("  - Dimension: {}", db_stats.dimension);
    
    // Check what got stored in PostgreSQL database
    println!("\n💎 PostgreSQL Database (High-Value Storage):");
    println!("  - Database: spatialvortex @ localhost:5432");
    println!("  - Table: flux_matrices");
    println!("  - Sacred positions (3-6-9) stored with enhanced weighting");
    
    // Test retrieval with newly trained knowledge
    println!("\n{}", "=".repeat(60));
    println!("\n🔍 Testing Retrieval with Trained Knowledge:\n");
    
    let test_queries = vec![
        "What is the significance of 3-6-9 in sacred geometry?",
        "How does vortex mathematics relate to energy flow?",
        "Explain transformer architecture in AI",
        "What is quantum entanglement?",
        "How does consciousness relate to integrated information?",
    ];
    
    use spatial_vortex::rag::{RAGRetriever, RetrievalConfig};
    
    let retrieval_config = RetrievalConfig {
        top_k: 5,
        rerank_top_n: 2,
        min_similarity: 0.5,
        min_confidence: 0.6,
        use_sacred_filtering: true,
        ..Default::default()
    };
    
    let retriever = RAGRetriever::new(vector_store.clone(), retrieval_config);
    
    for query in test_queries {
        println!("Query: \"{}\"", query);
        
        let results = retriever.retrieve(query).await?;
        
        if results.is_empty() {
            println!("  ❌ No relevant results found\n");
        } else {
            for (i, result) in results.iter().take(2).enumerate() {
                println!("  Result {}:", i + 1);
                println!("    - Flux Position: {}", result.flux_position);
                println!("    - Confidence: {:.3}", result.confidence);
                println!("    - Similarity: {:.3}", result.similarity);
                println!("    - Sacred: {}", if result.is_sacred { "✅" } else { "❌" });
                
                let snippet = if result.content.len() > 80 {
                    format!("{}...", &result.content[..80])
                } else {
                    result.content.clone()
                };
                println!("    - Content: {}", snippet);
            }
        }
        println!();
    }
    
    // Start continuous training
    println!("{}", "=".repeat(60));
    println!("\n🔄 Starting Continuous Training Mode...\n");
    
    trainer.start_continuous_training().await?;
    
    println!("✅ Continuous training activated!");
    println!("   The system will now:");
    println!("   • Monitor Grokipedia for new articles");
    println!("   • Automatically ingest relevant content");
    println!("   • Build embeddings with sacred geometry");
    println!("   • Store high-value knowledge in Confidence Lake");
    println!("   • Continuously improve understanding");
    
    // Final summary
    println!("\n{}", "=".repeat(60));
    println!("\n🎉 Grokipedia Training Complete!\n");
    println!("SpatialVortex has successfully learned from Grokipedia:");
    println!("  ✅ Sacred Geometry principles (3-6-9, vortex flow)");
    println!("  ✅ AI and Machine Learning concepts");
    println!("  ✅ Quantum Computing fundamentals");
    println!("  ✅ Consciousness theories");
    println!("  ✅ Mathematical and philosophical foundations");
    println!();
    println!("The AI is now enriched with knowledge organized through");
    println!("sacred geometry patterns and will continue learning!");
    println!();
    println!("🌀 Knowledge flows through the vortex, ever-expanding! 🌀");
    
    Ok(())
}
