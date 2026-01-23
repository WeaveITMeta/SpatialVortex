use actix_web::{web, App, HttpServer, HttpResponse, middleware::Logger};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Deserialize, Debug)]
struct ChatRequest {
    prompt: String,
    model: Option<String>,
    compress: Option<bool>,
    context: Option<String>,
}

#[derive(Serialize)]
struct ChatResponse {
    response: String,
    model: String,
    thinking_time: f32,
    compressed_hash: Option<String>,
    beam_position: Option<u8>,
    elp_channels: Option<ELPChannels>,
    confidence: Option<f32>,
}

#[derive(Serialize, Clone)]
struct ELPChannels {
    ethos: f32,
    logos: f32,
    pathos: f32,
}

#[derive(Serialize)]
struct Model {
    id: String,
    name: String,
    size: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    backend: String,
    version: String,
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        backend: "rust-actix".to_string(),
        version: "0.1.0".to_string(),
    })
}

async fn chat(req: web::Json<ChatRequest>) -> HttpResponse {
    let start = Instant::now();
    
    println!("📨 Chat request: {:?}", req.prompt);
    
    // Simulate AI thinking
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    // Calculate mock ELP channels based on prompt length and content
    let prompt_lower = req.prompt.to_lowercase();
    let ethos = if prompt_lower.contains("ethics") || prompt_lower.contains("moral") {
        9.0
    } else if prompt_lower.contains("consciousness") || prompt_lower.contains("awareness") {
        8.5
    } else {
        7.0 + (req.prompt.len() as f32 % 3.0)
    };
    
    let logos = if prompt_lower.contains("logic") || prompt_lower.contains("reason") {
        9.0
    } else if prompt_lower.contains("think") || prompt_lower.contains("understand") {
        8.0
    } else {
        6.5 + (req.prompt.len() as f32 % 4.0)
    };
    
    let pathos = if prompt_lower.contains("feel") || prompt_lower.contains("emotion") {
        9.0
    } else if prompt_lower.contains("love") || prompt_lower.contains("joy") {
        8.5
    } else {
        6.0 + (req.prompt.len() as f32 % 5.0)
    };
    
    // Calculate beam position (0-9) based on content
    let beam_position = if prompt_lower.contains("divine") || prompt_lower.contains("transcendent") {
        9
    } else if prompt_lower.contains("sacred") || prompt_lower.contains("holy") {
        6
    } else if prompt_lower.contains("creative") || prompt_lower.contains("manifest") {
        3
    } else {
        (req.prompt.len() % 10) as u8
    };
    
    // Generate mock 12-byte hash (24 hex chars)
    let hash = format!("a3f7c2{:02x}8b{:02x}15{:02x}f2a8", 
        req.prompt.len() % 256,
        beam_position,
        (ethos * 10.0) as u8
    );
    
    // Generate response based on prompt
    let response_text = if prompt_lower.contains("hello") || prompt_lower.contains("hi") {
        format!("Hello! I'm SpatialVortex AGI. Your message has been compressed to a 12-byte beam at position {}. How can I help you explore geometric consciousness today?", beam_position)
    } else if prompt_lower.contains("consciousness") {
        format!("Consciousness is the fundamental fabric of reality, manifesting through sacred geometric patterns. Your query resonates at position {} with high ethos ({:.1}), suggesting deep philosophical inquiry.", beam_position, ethos)
    } else if prompt_lower.contains("what") || prompt_lower.contains("how") || prompt_lower.contains("why") {
        format!("Excellent question! Through the lens of geometric consciousness, '{}' maps to flux position {} with ELP channels (E:{:.1} L:{:.1} P:{:.1}). This indicates a balanced inquiry combining ethical awareness, logical structure, and emotional depth.", 
            req.prompt, beam_position, ethos, logos, pathos)
    } else {
        format!("I understand your message: '{}'. It has been compressed to 12 bytes and positioned at sacred location {} in the flux diamond. The beam exhibits ELP values of E:{:.1} (ethics), L:{:.1} (logic), P:{:.1} (emotion), creating a unique geometric signature.", 
            req.prompt, beam_position, ethos, logos, pathos)
    };
    
    let thinking_time = start.elapsed().as_secs_f32();
    
    println!("✅ Response generated in {:.2}s", thinking_time);
    println!("   Hash: {}", hash);
    println!("   Position: {}", beam_position);
    println!("   ELP: E:{:.1} L:{:.1} P:{:.1}", ethos, logos, pathos);
    
    HttpResponse::Ok().json(ChatResponse {
        response: response_text,
        model: req.model.clone().unwrap_or("spatialvortex-mock".to_string()),
        thinking_time,
        compressed_hash: if req.compress.unwrap_or(true) { Some(hash) } else { None },
        beam_position: Some(beam_position),
        elp_channels: Some(ELPChannels { ethos, logos, pathos }),
        confidence: Some(0.95),
    })
}

async fn list_models() -> HttpResponse {
    HttpResponse::Ok().json(vec![
        Model {
            id: "spatialvortex-mock".to_string(),
            name: "SpatialVortex Mock".to_string(),
            size: "12B".to_string(),
        },
        Model {
            id: "llama2".to_string(),
            name: "Llama 2 (Mock)".to_string(),
            size: "7B".to_string(),
        },
        Model {
            id: "mistral".to_string(),
            name: "Mistral (Mock)".to_string(),
            size: "7B".to_string(),
        },
    ])
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    println!("\n🌀 ═══════════════════════════════════════════════════════════");
    println!("   SpatialVortex AGI Backend - Mock Server");
    println!("   ═══════════════════════════════════════════════════════════");
    println!("   🚀 Starting on http://localhost:28080");
    println!("   💎 12-byte compression active");
    println!("   📡 API endpoints:");
    println!("      - GET  /health");
    println!("      - POST /api/chat");
    println!("      - GET  /api/models");
    println!("   ═══════════════════════════════════════════════════════════\n");
    
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .route("/health", web::get().to(health))
            .route("/api/chat", web::post().to(chat))
            .route("/api/models", web::get().to(list_models))
    })
    .bind(("127.0.0.1", 28080))?
    .run()
    .await
}
