/// Grok API + Agglomerated Vortex Demo
/// 
/// Real end-to-end example using actual Grok API calls
/// 
/// Setup:
/// 1. Set XAI_API_KEY environment variable
/// 2. cargo run --example grok_vortex_demo
/// 
/// Order of Operations:
/// 1. Load test subjects
/// 2. Create FluxMatrix vortices
/// 3. Generate embeddings (mock for now, can integrate sentence-transformers)
/// 4. Index vectors
/// 5. Query for similar concepts
/// 6. Call Grok API with context
/// 7. Display results with sacred position analysis

use spatial_vortex::{
    lock_free_flux::LockFreeFluxMatrix,
    vector_search::{VectorIndex, VectorMetadata, VECTOR_DIM},
    runtime::ParallelRuntime,
    models::{FluxNode, NodeAttributes, NodeState, NodeDynamics, SemanticIndex},
};
use std::sync::Arc;
use std::collections::HashMap;
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use anyhow::Result;

const GROK_API_URL: &str = "https://api.x.ai/v1/chat/completions";

#[derive(Debug, Serialize)]
struct GrokRequest {
    model: String,
    messages: Vec<GrokMessage>,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GrokMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct GrokResponse {
    choices: Vec<GrokChoice>,
}

#[derive(Debug, Deserialize)]
struct GrokChoice {
    message: GrokMessage,
}

/// Subject data with geometric properties
struct SubjectData {
    name: String,
    position: u8,
    category: String,
    description: String,
}

impl SubjectData {
    fn new(name: &str, position: u8, category: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            position,
            category: category.to_string(),
            description: description.to_string(),
        }
    }
    
    fn is_sacred(&self) -> bool {
        [3, 6, 9].contains(&self.position)
    }
    
    fn extract_elp(&self) -> (f32, f32, f32) {
        // Extract Ethos, Logos, Pathos from category and description
        let ethos = match self.category.as_str() {
            "Virtue" => 0.95,
            "Philosophy" => 0.85,
            "Divine" => 0.90,
            _ => 0.60,
        };
        
        let logos = match self.category.as_str() {
            "Philosophy" | "Science" => 0.95,
            "Concept" => 0.85,
            _ => 0.60,
        };
        
        let pathos = match self.category.as_str() {
            "Emotion" => 0.95,
            "Art" => 0.85,
            _ => 0.50,
        };
        
        (ethos, logos, pathos)
    }
}

/// Generate mock embedding (in production, use sentence-transformers)
fn generate_embedding(text: &str) -> Array1<f32> {
    let seed = text.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
    let mut rng_state = seed;
    
    let mut vec = vec![0.0f32; VECTOR_DIM];
    for i in 0..VECTOR_DIM {
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        vec[i] = ((rng_state >> 16) as f32) / 32768.0 - 1.0;
    }
    
    // Normalize
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut vec {
            *v /= norm;
        }
    }
    
    Array1::from_vec(vec)
}

/// Call Grok API for inference
async fn call_grok_api(
    client: &Client,
    api_key: &str,
    subject: &SubjectData,
    context: &[String],
) -> Result<String> {
    let system_prompt = format!(
        "You are analyzing the concept '{}' within a geometric-semantic framework. \
        Position: {} ({}). Category: {}. \
        Analyze this concept's philosophical, emotional, and logical dimensions.",
        subject.name,
        subject.position,
        if subject.is_sacred() { "SACRED" } else { "standard" },
        subject.category
    );
    
    let user_prompt = if context.is_empty() {
        format!("Analyze '{}': {}", subject.name, subject.description)
    } else {
        format!(
            "Analyze '{}' in relation to these concepts: {}. Description: {}",
            subject.name,
            context.join(", "),
            subject.description
        )
    };
    
    let request = GrokRequest {
        model: "grok-beta".to_string(),
        messages: vec![
            GrokMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            GrokMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        temperature: 0.7,
    };
    
    let response = client
        .post(GROK_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Grok API error: {}", error_text);
    }
    
    let grok_response: GrokResponse = response.json().await?;
    
    Ok(grok_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response".to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🌀 GROK API + AGGLOMERATED VORTEX DEMO");
    println!("======================================\n");
    
    // Check for API key
    let api_key = std::env::var("XAI_API_KEY")
        .expect("❌ XAI_API_KEY environment variable not set!\n   Get your key from: https://console.x.ai/");
    
    println!("✅ Grok API key loaded\n");
    
    // Initialize components
    println!("📦 Initializing components...");
    let flux_matrix = Arc::new(LockFreeFluxMatrix::new("grok_demo".to_string()));
    let vector_index = Arc::new(VectorIndex::new_default());
    let runtime = Arc::new(ParallelRuntime::new_default()?);
    let client = Client::new();
    
    println!("   ✅ Lock-free FluxMatrix");
    println!("   ✅ Vector index ({}D)", VECTOR_DIM);
    println!("   ✅ Parallel runtime ({} threads)", runtime.config().worker_threads);
    println!("   ✅ HTTP client\n");
    
    // Define test subjects
    let subjects = vec![
        SubjectData::new(
            "Love",
            3,
            "Emotion",
            "The profound affection and care between beings, a universal force that binds"
        ),
        SubjectData::new(
            "Truth",
            6,
            "Philosophy",
            "The alignment with reality, honesty, and the fundamental nature of existence"
        ),
        SubjectData::new(
            "Creation",
            9,
            "Divine",
            "The act of bringing forth new existence, the fundamental generative force"
        ),
        SubjectData::new(
            "Joy",
            1,
            "Emotion",
            "A state of profound happiness and contentment arising from alignment"
        ),
        SubjectData::new(
            "Wisdom",
            8,
            "Philosophy",
            "The integration of knowledge with experience and understanding"
        ),
    ];
    
    // Step 1: Create vortices and index
    println!("🎯 Creating {} vortices...", subjects.len());
    for subject in &subjects {
        // Create FluxNode
        let (ethos, logos, pathos) = subject.extract_elp();
        
        let mut parameters = HashMap::new();
        parameters.insert("ethos".to_string(), ethos as f64);
        parameters.insert("logos".to_string(), logos as f64);
        parameters.insert("pathos".to_string(), pathos as f64);
        
        let mut properties = HashMap::new();
        properties.insert("subject".to_string(), subject.name.clone());
        properties.insert("category".to_string(), subject.category.clone());
        
        let node = FluxNode {
            position: subject.position,
            base_value: subject.position,
            semantic_index: SemanticIndex {
                positive_associations: vec![],
                negative_associations: vec![],
                neutral_base: subject.name.clone(),
                predicates: vec![],
                relations: vec![],
            },
            attributes: NodeAttributes {
                properties,
                parameters,
                state: NodeState {
                    active: true,
                    last_accessed: chrono::Utc::now(),
                    usage_count: 0,
                    context_stack: vec![],
                },
                dynamics: NodeDynamics {
                    evolution_rate: 1.0,
                    stability_index: 1.0,
                    interaction_patterns: vec![],
                    learning_adjustments: vec![],
                },
            },
            connections: vec![],
        };
        
        flux_matrix.insert(node);
        
        // Index vector
        let embedding = generate_embedding(&subject.name);
        let metadata = VectorMetadata {
            position: Some(subject.position),
            sacred: subject.is_sacred(),
            ethos,
            logos,
            pathos,
            created_at: std::time::SystemTime::now(),
        };
        
        vector_index.add(subject.name.clone(), embedding, metadata)?;
        
        let sacred_mark = if subject.is_sacred() { "⭐" } else { " " };
        println!("   {} Position {}: {} (E:{:.2} L:{:.2} P:{:.2})",
            sacred_mark, subject.position, subject.name, ethos, logos, pathos);
    }
    
    println!("\n✅ {} vortices created and indexed\n", subjects.len());
    
    // Step 2: Process each subject with Grok API
    println!("🤖 Processing with Grok API...\n");
    
    for subject in &subjects {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📝 Subject: {} (Position {})", subject.name, subject.position);
        
        if subject.is_sacred() {
            println!("⭐ SACRED POSITION - Orbital anchor point");
        }
        
        println!("   Category: {}", subject.category);
        let (e, l, p) = subject.extract_elp();
        println!("   ELP: Ethos={:.2}, Logos={:.2}, Pathos={:.2}", e, l, p);
        
        // Find similar concepts via vector search
        let query_embedding = generate_embedding(&subject.name);
        let similar_results = vector_index.search(&query_embedding, 4)?;
        
        let context: Vec<String> = similar_results
            .iter()
            .filter(|r| r.id != subject.name)
            .take(3)
            .map(|r| r.id.clone())
            .collect();
        
        if !context.is_empty() {
            println!("   Related: {}", context.join(", "));
        }
        
        println!("\n🔮 Calling Grok API...");
        
        match call_grok_api(&client, &api_key, subject, &context).await {
            Ok(response) => {
                println!("\n💬 Grok Response:");
                println!("   {}\n", response.replace('\n', "\n   "));
            }
            Err(e) => {
                println!("   ❌ Error: {}\n", e);
            }
        }
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Final stats
    println!("📊 Final Statistics:");
    println!("   FluxMatrix: {} nodes", flux_matrix.stats().total_nodes);
    println!("   VectorIndex: {} vectors", vector_index.stats().total_vectors);
    println!("   Sacred positions: {:?}", flux_matrix.stats().sacred_positions);
    println!("   Runtime tasks: {}", runtime.metrics().total_tasks);
    
    println!("\n✅ DEMO COMPLETE!\n");
    
    Ok(())
}
