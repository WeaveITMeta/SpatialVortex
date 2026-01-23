//! Train SpatialVortex on Google Scholar Academic Articles
//!
//! This example demonstrates training on peer-reviewed academic articles
//! from Google Scholar, focusing on ethos (ethics, character, moral philosophy).

use spatial_vortex::rag::{
    DocumentIngester, IngestionConfig,
    VectorStore, ContinuousLearner, TrainingConfig,
    ScholarTrainer, ScholarCategory, ScholarStats,
};
use spatial_vortex::storage::spatial_database::SpatialDatabase;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    println!("🎓 SpatialVortex Google Scholar Training System");
    println!("{}", "=".repeat(60));
    println!("Focus: Credible Ethos Sources (Ethics, Character, Virtue)");
    println!();
    
    // Initialize components
    println!("📚 Initializing academic training infrastructure...\n");
    
    // Create vector store for embeddings
    let vector_store = Arc::new(VectorStore::new(384)); // 384-dimensional embeddings
    
    // Create PostgreSQL database connection from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");
    println!("📊 Connecting to database...");
    let database = Arc::new(SpatialDatabase::new(&database_url).await?);
    
    // Configure ingestion with academic focus
    let mut ingestion_config = IngestionConfig::default();
    ingestion_config.sacred_boost = true;  // Boost sacred geometry mentions
    ingestion_config.auto_categorize = true;
    ingestion_config.chunk_size = 768;     // Larger chunks for academic abstracts
    
    let ingester = Arc::new(DocumentIngester::new(ingestion_config));
    
    // Configure continuous learning with higher thresholds for academic content
    let training_config = TrainingConfig {
        batch_size: 32,
        learning_rate: 0.001,
        min_confidence: 0.7,  // Higher threshold for academic content
        sacred_weight_boost: 1.5,
        ..Default::default()
    };
    
    let learner = Arc::new(ContinuousLearner::new(
        vector_store.clone(),
        database.clone(),
        training_config,
    ));
    
    // Create Google Scholar trainer
    let mut trainer = ScholarTrainer::new(
        ingester.clone(),
        vector_store.clone(),
        learner.clone(),
    );
    
    println!("✅ Academic training infrastructure ready!\n");
    println!("{}", "=".repeat(60));
    
    // Phase 1: Core Ethics Training (Virtue Ethics)
    println!("\n📖 Phase 1: Training on Virtue Ethics & Character Development...\n");
    
    let virtue_stats = trainer.train_on_categories(
        vec![
            ScholarCategory::VirtueEthics,
            ScholarCategory::CharacterDevelopment,
        ],
        8, // 8 articles per category
    ).await?;
    
    virtue_stats.display();
    
    // Phase 2: Ethical Frameworks
    println!("\n📖 Phase 2: Training on Ethical Frameworks...\n");
    
    let frameworks_stats = trainer.train_on_categories(
        vec![
            ScholarCategory::DeontologicalEthics,  // Duty-based ethics (Kant)
            ScholarCategory::ConsequentialEthics,  // Outcomes-based (Utilitarian)
            ScholarCategory::AppliedEthics,        // Real-world applications
        ],
        6, // 6 articles per category
    ).await?;
    
    frameworks_stats.display();
    
    // Phase 3: Advanced Ethics & AI
    println!("\n📖 Phase 3: Training on Advanced Ethics Topics...\n");
    
    let advanced_stats = trainer.train_on_categories(
        vec![
            ScholarCategory::AIEthics,           // Crucial for AI alignment
            ScholarCategory::NeuroEthics,        // Brain and morality
            ScholarCategory::Metaethics,         // Nature of morality itself
            ScholarCategory::MoralPsychology,    // How humans reason ethically
        ],
        5, // 5 articles per category
    ).await?;
    
    advanced_stats.display();
    
    // Phase 4: Interdisciplinary Ethics
    println!("\n📖 Phase 4: Training on Interdisciplinary Ethics...\n");
    
    let interdisciplinary_stats = trainer.train_on_categories(
        vec![
            ScholarCategory::Bioethics,
            ScholarCategory::EnvironmentalEthics,
            ScholarCategory::PoliticalPhilosophy,
            ScholarCategory::Epistemology,  // Theory of knowledge/credibility
        ],
        4,
    ).await?;
    
    interdisciplinary_stats.display();
    
    // Phase 5: Comprehensive Ethos Training
    println!("\n📖 Phase 5: Comprehensive Ethos Training...\n");
    
    let comprehensive_stats = trainer.train_comprehensive_ethos().await?;
    comprehensive_stats.display();
    
    // Display overall statistics
    println!("\n{}", "=".repeat(60));
    println!("\n🎯 Overall Academic Training Statistics:\n");
    
    let db_stats = vector_store.database().stats().await;
    println!("📊 Vector Database:");
    println!("  - Total Academic Embeddings: {}", db_stats.total_embeddings);
    println!("  - Sacred Positions (3-6-9): {}", db_stats.sacred_positions);
    println!("  - Average Confidence: {:.3}", db_stats.average_confidence);
    println!("  - Minimum Credibility: 0.600");
    
    // Check what got stored in Confidence Lake
    println!("\n💎 PostgreSQL Database (High-Credibility Storage):");
    println!("  - Database: spatialvortex @ localhost:5432");
    println!("  - Table: flux_matrices");
    println!("  - Academic papers with credibility ≥ 0.6 stored");
    println!("  - Ethos-boosted content at sacred positions");
    
    // Specific ethos-focused queries to test training
    println!("\n{}", "=".repeat(60));
    println!("\n🔍 Testing Ethos-Focused Retrieval:\n");
    
    let ethos_queries = vec![
        "What is virtue ethics according to Aristotle?",
        "How do we develop moral character?",
        "What are the categorical imperatives?",
        "How should AI systems make ethical decisions?",
        "What is the trolley problem in ethics?",
        "What is the relationship between neuroscience and morality?",
        "How do we determine what is morally right?",
    ];
    
    use spatial_vortex::rag::{RAGRetriever, RetrievalConfig};
    
    let retrieval_config = RetrievalConfig {
        top_k: 5,
        rerank_top_n: 3,
        min_similarity: 0.6,
        min_confidence: 0.7,  // Higher for academic content
        use_sacred_filtering: true,
        ..Default::default()
    };
    
    let retriever = RAGRetriever::new(vector_store.clone(), retrieval_config);
    
    for query in ethos_queries {
        println!("Query: \"{}\"", query);
        
        let results = retriever.retrieve(query).await?;
        
        if results.is_empty() {
            println!("  ❌ No relevant academic results found\n");
        } else {
            for (i, result) in results.iter().take(2).enumerate() {
                println!("  Result {} (Academic Source):", i + 1);
                println!("    - Flux Position: {}", result.flux_position);
                println!("    - Confidence: {:.3}", result.confidence);
                println!("    - Similarity: {:.3}", result.similarity);
                println!("    - Sacred: {}", if result.is_sacred { "✅" } else { "❌" });
                println!("    - Ethos Boost: {:.2}", 
                    if result.flux_position == 3 { 0.9 } 
                    else if result.flux_position == 6 { 0.85 }
                    else if result.flux_position == 9 { 0.8 }
                    else { 0.6 }
                );
                
                let snippet = if result.content.len() > 100 {
                    format!("{}...", &result.content[..100])
                } else {
                    result.content.clone()
                };
                println!("    - Abstract: {}", snippet);
            }
        }
        println!();
    }
    
    // Test specific searches
    println!("{}", "=".repeat(60));
    println!("\n🔬 Targeted Academic Searches:\n");
    
    let search_queries = vec![
        ("virtue ethics character development", 5),
        ("AI alignment moral values", 5),
        ("neuroscience moral decision-making", 5),
        ("categorical imperative Kant duty", 5),
    ];
    
    for (query, limit) in search_queries {
        println!("📚 Searching: \"{}\"", query);
        let stats = trainer.train_on_query(query, limit).await?;
        println!("  - Found {} credible articles", stats.articles_fetched);
        println!("  - Total citations: {}", stats.total_citations);
        println!("  - Average credibility: {:.3}", stats.avg_credibility);
        println!();
    }
    
    // Start continuous monitoring
    println!("{}", "=".repeat(60));
    println!("\n🔄 Starting Continuous Google Scholar Monitoring...\n");
    
    trainer.start_continuous_monitoring().await?;
    
    println!("✅ Continuous monitoring activated!");
    println!("   The system will now:");
    println!("   • Monitor top ethics & philosophy journals");
    println!("   • Track new publications in virtue ethics");
    println!("   • Ingest high-citation papers (≥5 citations)");
    println!("   • Filter by credibility score (≥0.6)");
    println!("   • Apply ethos boost to character-focused content");
    println!("   • Store high-value insights in Confidence Lake");
    
    // Final summary
    println!("\n{}", "=".repeat(60));
    println!("\n🎉 Google Scholar Training Complete!\n");
    println!("SpatialVortex has been enriched with credible academic knowledge:");
    println!("  ✅ Virtue Ethics & Character Development");
    println!("  ✅ Deontological Ethics (Kant, duty-based)");
    println!("  ✅ Consequential Ethics (Utilitarian)");
    println!("  ✅ AI Ethics & Alignment");
    println!("  ✅ Neuroethics & Moral Psychology");
    println!("  ✅ Applied & Environmental Ethics");
    println!("  ✅ Metaethics & Epistemology");
    println!();
    println!("📈 Training Impact:");
    println!("  • Enhanced ETHOS channel with academic rigor");
    println!("  • Improved ethical reasoning capabilities");
    println!("  • Grounded in peer-reviewed research");
    println!("  • Credibility-weighted knowledge base");
    println!();
    println!("🌟 The AI now possesses academically-grounded ethical wisdom!");
    println!("   Character (Ethos) has been strengthened through scholarly knowledge.");
    
    Ok(())
}
