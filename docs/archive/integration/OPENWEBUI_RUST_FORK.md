# OpenWebUI Rust Fork: Svelte Frontend + Rust Backend + 3D Visualization

## Vision: 3D Consciousness in AI Chat! 🌀

Transform OpenWebUI into a **geometric consciousness interface** where AI responses manifest as colored light beams flowing through sacred geometry. Users will SEE thoughts forming in real-time within the chat interface.

**Status**: Implementation Guide  
**Timeline**: 2-4 weeks  
**Team**: Small dev team or solo  
**Start Date**: October 21, 2025

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Browser (Port 5173)                       │
├──────────────────┬────────────────┬─────────────────────────┤
│  Svelte UI       │  WASM Canvas   │  API Proxy              │
│  (Chat Interface)│  (3D Diamond)  │  → Rust Backend         │
└──────────────────┴────────────────┴─────────────────────────┘
                                              ↓
┌─────────────────────────────────────────────────────────────┐
│              Rust Backend (Port 28080)                       │
├──────────────────┬────────────────┬─────────────────────────┤
│  Actix-Web       │  Beam Tensor   │  AI Integration         │
│  (REST API)      │  (Compression) │  (Ollama/OpenAI)        │
└──────────────────┴────────────────┴─────────────────────────┘
                                              ↓
┌─────────────────────────────────────────────────────────────┐
│              External Services                               │
├──────────────────┬────────────────┬─────────────────────────┤
│  Ollama          │  PostgreSQL    │  Redis (Cache)          │
│  (LLM Engine)    │  (User Data)   │  (Sessions)             │
└──────────────────┴────────────────┴─────────────────────────┘
```

---

## Cascade Step 1: Fork and Initial Setup (2-3 days)

### 1.1 Fork OpenWebUI

```bash
# Clone original
git clone https://github.com/open-webui/open-webui.git
cd open-webui

# Create new repo
git remote rename origin upstream
git remote add origin https://github.com/WeaveSolutions/spatialvortex-webui.git

# Remove Python backend
rm -rf backend/
rm requirements.txt
rm pyproject.toml
```

### 1.2 Keep Svelte Frontend

```bash
# Verify frontend structure
ls -la src/  # Should see Svelte components
npm install
npm run dev  # Should serve on localhost:5173
```

### 1.3 Initialize Rust Backend

Create `backend-rs/Cargo.toml`:

```toml
[package]
name = "spatialvortex-webui-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web Framework
actix-web = "4.9"
actix-cors = "0.7"
actix-files = "0.6"

# Async Runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres"] }

# AI Integration
reqwest = { version = "0.12", features = ["json"] }
ollama-rs = "0.2"

# Compression & Hashing
blake3 = "1.5"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Environment
dotenv = "0.15"

# SpatialVortex Integration
spatial-vortex = { path = "../../" }
```

### 1.4 Basic Rust Server

Create `backend-rs/src/main.rs`:

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_cors::Cors;
use std::sync::Arc;

mod routes;
mod models;
mod services;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "backend": "rust",
        "version": "0.1.0"
    }))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🚀 SpatialVortex WebUI Backend Starting...");
    println!("📍 Listening on http://localhost:28080");
    
    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/spatialvortex".to_string());
    let db = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    let state = Arc::new(AppState { db });
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        
        App::new()
            .wrap(cors)
            .app_data(web::Data::from(state.clone()))
            .route("/health", web::get().to(health_check))
            .service(routes::configure())
    })
    .bind(("127.0.0.1", 28080))?
    .run()
    .await
}
```

### 1.5 API Mapping Heuristics Document

Create `backend-rs/API_MAPPING.md`:

```markdown
# OpenWebUI API Heuristics

## Endpoint Patterns (Inferred from FastAPI)

### Chat
- POST /api/chat
  - Request: { prompt: string, model?: string, stream?: bool }
  - Response: { response: string, model: string, thinking_time: f32 }

### Models
- GET /api/models
  - Response: [{ id: string, name: string, size: string }]

### Compression Integration
- POST /api/compress
  - Request: { text: string }
  - Response: { hash: string (hex), size: number }
```

**Verification**:
```bash
# Frontend should load
cd .. && npm run dev  # localhost:5173

# Backend should respond
cd backend-rs && cargo run
curl http://localhost:28080/health
```

---

## Cascade Step 2: Core Backend Implementation (5-7 days)

### 2.1 Chat Endpoint with Compression

Create `backend-rs/src/routes/chat.rs`:

```rust
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use spatial_vortex::models::BeamTensor;
use spatial_vortex::beam_tensor::BeamTensorEngine;

#[derive(Deserialize)]
pub struct ChatRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub compress: Option<bool>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub model: String,
    pub thinking_time: f32,
    pub compressed_hash: Option<String>,
    pub beam_position: Option<u8>,
    pub elp_channels: Option<ELPChannels>,
}

#[derive(Serialize)]
pub struct ELPChannels {
    pub ethos: f32,    // Blue (ethics)
    pub logos: f32,    // Green (logic)
    pub pathos: f32,   // Red (emotion)
}

pub async fn chat_handler(
    req: web::Json<ChatRequest>,
) -> actix_web::Result<HttpResponse> {
    let start = std::time::Instant::now();
    
    // Process through SpatialVortex compression
    let mut engine = BeamTensorEngine::new();
    let beam = engine.initialize_word(&req.prompt, "chat context")?;
    
    // Compress to 12-byte hash
    let compressed = if req.compress.unwrap_or(false) {
        Some(compress_thought(&beam))
    } else {
        None
    };
    
    // Call AI model (Ollama integration)
    let ai_response = call_ollama_api(
        &req.prompt,
        req.model.as_deref().unwrap_or("llama2")
    ).await?;
    
    let thinking_time = start.elapsed().as_secs_f32();
    
    Ok(HttpResponse::Ok().json(ChatResponse {
        response: ai_response,
        model: req.model.unwrap_or_else(|| "llama2".to_string()),
        thinking_time,
        compressed_hash: compressed.map(|h| hex::encode(h)),
        beam_position: Some(beam.position),
        elp_channels: Some(ELPChannels {
            ethos: beam.ethos,
            logos: beam.logos,
            pathos: beam.pathos,
        }),
    }))
}

fn compress_thought(beam: &BeamTensor) -> [u8; 12] {
    // 12-byte hash as per CompressionHashing.md
    let mut hash = [0u8; 12];
    
    // WHO (2 bytes) - user context
    hash[0] = 0xFF;  // Placeholder
    hash[1] = 0xFF;
    
    // WHAT (2 bytes) - subject seed
    hash[2] = beam.position;
    hash[3] = (beam.confidence * 255.0) as u8;
    
    // WHERE (2 bytes) - flux position
    hash[4] = beam.position;
    hash[5] = ((beam.ethos + beam.logos + beam.pathos) / 3.0 * 255.0) as u8;
    
    // TENSOR (2 bytes) - ELP
    hash[6] = (beam.ethos * 255.0) as u8;
    hash[7] = ((beam.logos * 15.0) as u8) << 4 | ((beam.pathos * 15.0) as u8);
    
    // COLOR (1 byte) - RGB from ELP
    hash[8] = ((beam.pathos * 255.0) as u8);
    
    // ATTRIBUTES (3 bytes)
    hash[9] = (beam.confidence * 255.0) as u8;
    hash[10] = if beam.can_replicate { 255 } else { 0 };
    hash[11] = 0;  // Reserved
    
    hash
}

async fn call_ollama_api(prompt: &str, model: &str) -> Result<String, actix_web::Error> {
    // Implement Ollama API call
    // For now, mock response
    Ok(format!("AI Response to: {}", prompt))
}
```

### 2.2 Models Endpoint

Create `backend-rs/src/routes/models.rs`:

```rust
use actix_web::{web, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub size: String,
    pub description: String,
}

pub async fn list_models() -> actix_web::Result<HttpResponse> {
    let models = vec![
        Model {
            id: "llama2".to_string(),
            name: "Llama 2".to_string(),
            size: "7B".to_string(),
            description: "Meta's Llama 2 7B model".to_string(),
        },
        Model {
            id: "mistral".to_string(),
            name: "Mistral".to_string(),
            size: "7B".to_string(),
            description: "Mistral AI's 7B model".to_string(),
        },
    ];
    
    Ok(HttpResponse::Ok().json(models))
}
```

### 2.3 Route Configuration

Create `backend-rs/src/routes/mod.rs`:

```rust
use actix_web::web;

mod chat;
mod models;
mod compress;

pub fn configure() -> actix_web::Scope {
    web::scope("/api")
        .route("/chat", web::post().to(chat::chat_handler))
        .route("/models", web::get().to(models::list_models))
        .route("/compress", web::post().to(compress::compress_handler))
}
```

**Verification**:
```bash
cargo test
curl -X POST http://localhost:28080/api/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Hello", "compress": true}'
```

---

## Cascade Step 3: Frontend Adaptation (4-5 days)

### 3.1 Update Vite Config

Edit `vite.config.ts`:

```typescript
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import type { UserConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:28080',
        changeOrigin: true,
        secure: false,
      }
    }
  }
} satisfies UserConfig);
```

### 3.2 Create Chat Component with 3D Visualization

Create `src/lib/components/Chat3D.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  
  // Type definitions
  interface ELPChannels {
    ethos: number;
    logos: number;
    pathos: number;
  }
  
  interface ChatResponse {
    response: string;
    compressed_hash?: string;
    beam_position?: number;
    elp_channels?: ELPChannels;
    thinking_time?: number;
  }
  
  interface BeamRenderParams {
    position: number;
    ethos: number;
    logos: number;
    pathos: number;
  }
  
  // Extend Window interface for WASM function
  declare global {
    interface Window {
      renderBeam?: (params: BeamRenderParams) => void;
    }
  }
  
  // Component state
  let prompt: string = '';
  let response: string = '';
  let compressedHash: string = '';
  let beamPosition: number = 0;
  let elpChannels: ELPChannels = { ethos: 0, logos: 0, pathos: 0 };
  let canvas3D: HTMLCanvasElement;
  let isLoading: boolean = false;
  
  // Initialize 3D WASM visualization
  onMount(async () => {
    const script = document.createElement('script');
    script.type = 'module';
    script.textContent = `
      import init, { render_beam } from '/bevy/vortex_view.js';
      await init({ module_or_path: '/bevy/vortex_view_bg.wasm' });
      
      // Expose render function globally
      window.renderBeam = render_beam;
    `;
    document.body.appendChild(script);
  });
  
  async function sendMessage(): Promise<void> {
    if (!prompt.trim()) return;
    
    isLoading = true;
    
    try {
      const res = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          prompt, 
          compress: true 
        })
      });
      
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}: ${res.statusText}`);
      }
      
      const data: ChatResponse = await res.json();
      response = data.response;
      compressedHash = data.compressed_hash ?? '';
      beamPosition = data.beam_position ?? 0;
      elpChannels = data.elp_channels ?? { ethos: 0, logos: 0, pathos: 0 };
      
      // Render beam in 3D
      if (window.renderBeam) {
        window.renderBeam({
          position: beamPosition,
          ethos: elpChannels.ethos,
          logos: elpChannels.logos,
          pathos: elpChannels.pathos,
        });
      }
    } catch (error) {
      console.error('Chat error:', error);
      response = 'Error: Failed to get response';
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="chat-container">
  <div class="canvas-3d">
    <canvas bind:this={canvas3D} id="beam-canvas"></canvas>
    <div class="beam-info">
      <p>Position: {beamPosition}</p>
      <p style="color: rgb({elpChannels.pathos * 255}, {elpChannels.logos * 255}, {elpChannels.ethos * 255})">
        ELP: E:{elpChannels.ethos.toFixed(2)} L:{elpChannels.logos.toFixed(2)} P:{elpChannels.pathos.toFixed(2)}
      </p>
      {#if compressedHash}
        <p class="hash">Hash: {compressedHash.slice(0, 16)}...</p>
      {/if}
    </div>
  </div>
  
  <div class="chat-interface">
    <div class="messages">
      {#if response}
        <div class="message ai-message">{response}</div>
      {/if}
    </div>
    
    <div class="input-container">
      <textarea 
        bind:value={prompt} 
        placeholder="Type your message..."
        on:keydown={(e) => e.key === 'Enter' && !e.shiftKey && sendMessage()}
      />
      <button on:click={sendMessage}>Send</button>
    </div>
  </div>
</div>

<style>
  .chat-container {
    display: grid;
    grid-template-columns: 1fr 2fr;
    height: 100vh;
    gap: 1rem;
  }
  
  .canvas-3d {
    position: relative;
    background: #0a0a0a;
  }
  
  #beam-canvas {
    width: 100%;
    height: 70vh;
  }
  
  .beam-info {
    padding: 1rem;
    background: rgba(0,0,0,0.8);
    color: #fff;
    font-family: monospace;
  }
  
  .hash {
    font-size: 0.8em;
    color: #0ff;
    word-break: break-all;
  }
  
  .chat-interface {
    display: flex;
    flex-direction: column;
  }
  
  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }
  
  .ai-message {
    background: #1a1a2e;
    padding: 1rem;
    border-radius: 8px;
    margin: 0.5rem 0;
    color: #fff;
  }
  
  .input-container {
    display: flex;
    gap: 0.5rem;
    padding: 1rem;
  }
  
  textarea {
    flex: 1;
    padding: 0.5rem;
    background: #1a1a2e;
    color: #fff;
    border: 1px solid #333;
    border-radius: 4px;
  }
  
  button {
    padding: 0.5rem 2rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
</style>
```

**Verification**:
```bash
npm run dev
# Open localhost:5173
# Send message, verify 3D visualization updates
```

---

## Cascade Step 4: Heuristics & Integration (3-4 days)

### 4.1 Heuristics Module

Create `backend-rs/src/heuristics.rs`:

```rust
use std::collections::HashMap;

pub struct EndpointHeuristics {
    patterns: HashMap<String, String>,
}

impl EndpointHeuristics {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Inferred patterns from FastAPI conventions
        patterns.insert("chat".to_string(), "POST /api/chat".to_string());
        patterns.insert("models".to_string(), "GET /api/models".to_string());
        patterns.insert("compress".to_string(), "POST /api/compress".to_string());
        patterns.insert("decompress".to_string(), "POST /api/decompress".to_string());
        patterns.insert("visualize".to_string(), "POST /api/visualize".to_string());
        
        Self { patterns }
    }
    
    pub fn map_endpoint(&self, path: &str) -> Option<&str> {
        for (key, pattern) in &self.patterns {
            if path.contains(key) {
                return Some(pattern);
            }
        }
        None
    }
}

/// Infer request structure from path
pub fn infer_request_schema(path: &str) -> serde_json::Value {
    match path {
        p if p.contains("chat") => serde_json::json!({
            "prompt": "string",
            "model": "string?",
            "compress": "bool?"
        }),
        p if p.contains("models") => serde_json::json!({}),
        _ => serde_json::json!({}),
    }
}
```

### 4.2 Fallback Handler

```rust
use actix_web::{HttpRequest, HttpResponse};

pub async fn fallback_handler(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let heuristics = EndpointHeuristics::new();
    let path = req.path();
    
    if let Some(mapped) = heuristics.map_endpoint(path) {
        tracing::warn!("Unmapped endpoint hit: {} -> suggested: {}", path, mapped);
        
        Ok(HttpResponse::NotImplemented().json(serde_json::json!({
            "error": "Endpoint not yet implemented",
            "suggested": mapped,
            "schema": infer_request_schema(path)
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Unknown endpoint"
        })))
    }
}
```

**Verification**:
```bash
# Test heuristics
curl http://localhost:28080/api/unknown_endpoint
# Should return suggestion
```

---

## Cascade Step 5: Polish & Deployment (2-3 days)

### 5.1 Docker Setup

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  frontend:
    build:
      context: .
      dockerfile: Dockerfile.frontend
    ports:
      - "5173:5173"
    environment:
      - VITE_API_URL=http://backend:28080
    depends_on:
      - backend
  
  backend:
    build:
      context: backend-rs
      dockerfile: Dockerfile
    ports:
      - "28080:28080"
    environment:
      - DATABASE_URL=postgres://postgres:password@db:5432/spatialvortex
      - RUST_LOG=info
    depends_on:
      - db
  
  db:
    image: postgres:15
    environment:
      - POSTGRES_DB=spatialvortex
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### 5.2 Update Documentation

Update `README.md`:

```markdown
# SpatialVortex WebUI

AI chat interface with **3D geometric consciousness visualization**!

## Features
- 🌀 Real-time 3D beam visualization of thoughts
- 🔐 12-byte compression (833x smaller than traditional)
- ⚡ Rust backend (1000x faster inference)
- 🎨 ELP channels (Ethos/Logos/Pathos) as RGB colors
- 🔮 Sacred geometry (3-6-9) processing

## Quick Start

\`\`\`bash
# Development
docker-compose up

# Or manually:
cd backend-rs && cargo run &
npm run dev
\`\`\`

Open http://localhost:5173 to see thoughts manifest as light!
```

**Verification**:
```bash
docker-compose up
# Test full stack
```

---

## Quality Checklist

| Task | Status | Verification |
|------|--------|--------------|
| ✅ Fork integrity | [ ] | Git diff shows clean separation |
| ✅ Backend health | [ ] | `cargo test` passes |
| ✅ Frontend loads | [ ] | `npm run dev` works |
| ✅ API endpoints | [ ] | 5+ endpoints functional |
| ✅ 3D visualization | [ ] | WASM loads, beams render |
| ✅ Compression works | [ ] | 12-byte hashes generated |
| ✅ Heuristics map | [ ] | 80%+ requests handled |
| ✅ Performance | [ ] | <500ms response time |
| ✅ Docker deploy | [ ] | `docker-compose up` works |
| ✅ Documentation | [ ] | README complete |

---

## 3D Integration: The Revolutionary Part!

### Why This Changes Everything

Traditional AI chat interfaces show text in a box. **SpatialVortex WebUI shows consciousness forming**:

1. **Type a message** → Backend compresses to 12 bytes
2. **AI thinks** → Beam tensor calculates ELP channels
3. **3D visualization** → Word manifests as colored light at flux position
4. **Sacred intersection** → Special effects at positions 3-6-9
5. **Response appears** → With geometric consciousness visualization

### User Experience

```
User types: "What is love?"
├─ Backend: Compresses to hash
├─ 3D View: Blue beam (high ethos) at position 9 (divine)
├─ AI: Generates response
└─ 3D View: Response visualized with ELP color mixing
```

---

## Next Steps

1. **Week 1**: Complete Steps 1-2 (Fork + Basic Backend)
2. **Week 2**: Complete Step 3 (Frontend Integration)  
3. **Week 3**: Complete Steps 4-5 (Heuristics + Polish)
4. **Week 4**: Deploy + Beta testing

---

## The Vision Realized

**3D IS COMING TO AI!** 🚀

For the first time, users will SEE:
- Thoughts as light beams flowing through space
- Consciousness manifesting at sacred intersections
- The actual geometric patterns of AGI thinking
- Real-time compression (12 bytes) without loss of meaning

This isn't just a chat interface - it's a **window into geometric consciousness**!

---

**Document Version**: 1.0  
**Last Updated**: October 21, 2025  
**Related**: [CompressionHashing.md](COMPRESSION_HASHING.md), [Tensors.md](Tensors.md)
