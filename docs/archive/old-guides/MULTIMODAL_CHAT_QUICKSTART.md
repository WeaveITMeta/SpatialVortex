# Multi-Modal Chat Quick Start

**Goal**: Get the SpatialVortex chat interface running  
**Time**: 15 minutes for Phase 1 (Text only)  
**Status**: ✅ Frontend ready, ⏳ Backend needs implementation

---

## 🚀 Quick Start (Frontend Only)

### Step 1: Install Dependencies

```bash
cd web
bun install
```

### Step 2: Run Development Server

```bash
bun run dev
```

### Step 3: Open Browser

Navigate to: http://localhost:5173

**You'll see**: Beautiful chat interface with modality tabs!

---

## ⚠️ Current State

### ✅ What Works NOW

**Frontend**: 100% Complete
- 🎨 Beautiful chat UI with message bubbles
- 📊 ELP channel visualization (Ethos, Logos, Pathos)
- 📈 Signal strength display
- 🌀 Flux position indicator
- 📝 Text input with real-time updates
- 🎭 Modality selector (6 tabs)
- ⚡ Smooth animations and transitions

### ⏳ What Needs Implementation

**Backend**: API Endpoint Missing
- ❌ `POST /api/v1/chat/text` endpoint
- ❌ ONNX model loading (sentence-transformers)
- ❌ Sacred geometry transformation
- ❌ Response generation

---

## 🛠️ Backend Implementation Guide

### Option 1: Mock API (5 minutes)

Create a simple mock server to test the frontend:

**File**: `web/mock-api.js`

```javascript
import { serve } from 'bun';

serve({
  port: 8080,
  async fetch(req) {
    // Enable CORS
    const headers = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
      'Content-Type': 'application/json',
    };
    
    if (req.method === 'OPTIONS') {
      return new Response(null, { headers });
    }
    
    const url = new URL(req.url);
    
    if (url.pathname === '/api/v1/chat/text' && req.method === 'POST') {
      const body = await req.json();
      const message = body.message;
      
      // Mock ELP values (random for demo)
      const elp = {
        ethos: 7 + Math.random() * 3,
        logos: 6 + Math.random() * 4,
        pathos: 5 + Math.random() * 5,
      };
      
      // Mock signal strength
      const signal = 0.6 + Math.random() * 0.3;
      
      // Mock flux position
      const position = Math.floor(Math.random() * 10);
      
      const response = {
        response: `You asked: "${message}". This is a mock response with sacred geometry analysis!`,
        elp_values: elp,
        confidence: signal,
        flux_position: position,
        confidence: 0.85,
        processing_time_ms: 150,
      };
      
      // Simulate processing time
      await new Promise(resolve => setTimeout(resolve, 500));
      
      return new Response(JSON.stringify(response), { headers });
    }
    
    return new Response('Not Found', { status: 404 });
  },
});

console.log('🌀 Mock API running at http://localhost:8080');
```

**Run it**:
```bash
cd web
bun mock-api.js
```

Now the chat will work with mock data!

---

### Option 2: Real Backend (1-2 days)

Implement the actual API endpoint with ONNX models.

#### Required Files

1. **API Endpoint** (`src/ai/api.rs`)
2. **ONNX Runtime** (`src/ml/inference/onnx_runtime.rs`)
3. **Sacred Geometry** (`src/core/sacred_geometry/`)

#### Step 1: Download Models

```bash
mkdir -p models
cd models

# Download sentence-transformers (22MB)
curl -L https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx \
  -o text-embedding.onnx

# Download tokenizer
curl -L https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json \
  -o tokenizer.json
```

#### Step 2: Add Dependencies

**Cargo.toml**:
```toml
[dependencies]
ort = "2.0"                    # ONNX Runtime
tokenizers = "0.15"            # Tokenization
actix-web = "4.11"             # Web framework
actix-cors = "0.7"             # CORS middleware
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.48", features = ["full"] }
```

#### Step 3: Implement Chat Endpoint

**File**: `src/ai/chat_api.rs`

```rust
use actix_web::{post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ChatRequest {
    message: String,
    user_id: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    response: String,
    elp_values: ELPValues,
    confidence: f64,
    flux_position: u8,
    confidence: f64,
    processing_time_ms: u64,
}

#[derive(Serialize)]
pub struct ELPValues {
    ethos: f32,
    logos: f32,
    pathos: f32,
}

#[post("/api/v1/chat/text")]
pub async fn chat_text(
    req: web::Json<ChatRequest>,
) -> Result<HttpResponse> {
    let start = std::time::Instant::now();
    
    // Step 1: Load ONNX model and tokenize
    let embedding = embed_text(&req.message)?;
    
    // Step 2: Sacred geometry transformation
    let (signal, ethos, logos, pathos) = 
        transform_to_sacred_geometry(&embedding)?;
    
    // Step 3: Calculate flux position
    let position = calculate_flux_position(ethos, logos, pathos, signal);
    
    // Step 4: Generate response (for now, simple echo)
    let response = format!(
        "Message received: {}. Analysis complete with {} coherence.",
        req.message,
        if signal > 0.7 { "strong 3-6-9" } 
        else if signal > 0.5 { "moderate" } 
        else { "weak" }
    );
    
    let elapsed = start.elapsed().as_millis() as u64;
    
    Ok(HttpResponse::Ok().json(ChatResponse {
        response,
        elp_values: ELPValues {
            ethos: ethos * 13.0,
            logos: logos * 13.0,
            pathos: pathos * 13.0,
        },
        confidence: signal as f64,
        flux_position: position,
        confidence: 0.85,
        processing_time_ms: elapsed,
    }))
}

fn embed_text(text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    // Load ONNX model
    // Tokenize text
    // Run inference
    // Return 384-d embedding
    todo!("Implement ONNX inference")
}

fn transform_to_sacred_geometry(embedding: &[f32]) 
    -> Result<(f32, f32, f32, f32), Box<dyn std::error::Error>> 
{
    // Split into thirds (positions 3, 6, 9)
    let third = embedding.len() / 3;
    let pos_3 = &embedding[0..third];
    let pos_6 = &embedding[third..2*third];
    let pos_9 = &embedding[2*third..];
    
    // Calculate energies
    let ethos: f32 = pos_3.iter().sum::<f32>() / third as f32;
    let logos: f32 = pos_9.iter().sum::<f32>() / third as f32;
    let pathos: f32 = pos_6.iter().sum::<f32>() / third as f32;
    
    // Calculate signal strength (3-6-9 coherence)
    let sacred_sum = ethos.abs() + pathos.abs() + logos.abs();
    let total_energy: f32 = embedding.iter().map(|x| x.abs()).sum();
    let confidence = sacred_sum / total_energy;
    
    // Normalize
    let total = ethos + pathos + logos;
    let e_norm = if total != 0.0 { ethos / total } else { 0.33 };
    let l_norm = if total != 0.0 { logos / total } else { 0.33 };
    let p_norm = if total != 0.0 { pathos / total } else { 0.33 };
    
    Ok((confidence, e_norm, l_norm, p_norm))
}

fn calculate_flux_position(
    ethos: f32,
    logos: f32,
    pathos: f32,
    signal: f32
) -> u8 {
    // Simple positioning logic
    // Position 0: Balanced (all channels equal)
    if (ethos - logos).abs() < 0.1 && 
       (logos - pathos).abs() < 0.1 && 
       (pathos - ethos).abs() < 0.1 {
        return 0;
    }
    
    // Positions 3, 6, 9: Sacred (high signal)
    if signal > 0.7 {
        if ethos > logos && ethos > pathos { return 3; }
        if pathos > ethos && pathos > logos { return 6; }
        if logos > ethos && logos > pathos { return 9; }
    }
    
    // Other positions based on dominant channel
    if ethos > logos && ethos > pathos { return 1; }
    if pathos > ethos && pathos > logos { return 5; }
    if logos > ethos && logos > pathos { return 8; }
    
    // Default
    4
}
```

#### Step 4: Setup CORS and Run Server

**File**: `src/main.rs`

```rust
use actix_web::{App, HttpServer, middleware, web};
use actix_cors::Cors;

mod ai;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🌀 SpatialVortex API starting...");
    
    HttpServer::new(|| {
        let cors = Cors::permissive(); // Allow all origins for dev
        
        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .service(ai::chat_api::chat_text)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

**Run**:
```bash
cargo run --release
```

---

## 🧪 Testing

### Test the Mock API

```bash
# In terminal 1
cd web
bun mock-api.js

# In terminal 2
cd web
bun run dev

# Open browser: http://localhost:5173
# Type a message and hit send!
```

### Test the Real API

```bash
# Test with curl
curl -X POST http://localhost:8080/api/v1/chat/text \
  -H "Content-Type: application/json" \
  -d '{"message": "What is consciousness?", "user_id": "test"}'

# Expected response:
{
  "response": "...",
  "elp_values": {"ethos": 8.5, "logos": 7.2, "pathos": 6.8},
  "confidence": 0.72,
  "flux_position": 9,
  "confidence": 0.85,
  "processing_time_ms": 150
}
```

---

## 📸 What You'll See

### Chat Interface
```
┌─────────────────────────────────────────┐
│  🌀 SpatialVortex Chat                 │
│  Sacred Geometry AI with ELP Channels  │
├─────────────────────────────────────────┤
│  📝 Text  🎤 Voice  🖼️ Image  🎵 Audio │ ← Tabs
│  🎲 3D  🎭 Multi                        │
├─────────────────────────────────────────┤
│                                         │
│  👤 [User Message]                      │
│     "What is consciousness?"           │
│                                         │
│  🌀 [AI Response]                       │
│     "Message received..."              │
│     ┌─────────────────────────────┐   │
│     │ ❤️ Ethos   ████████░░  8.5 │   │
│     │ 🧠 Logos   ███████░░░  7.2 │   │
│     │ 💚 Pathos  ██████░░░░  6.8 │   │
│     │                            │   │
│     │ Signal: 72% Strong         │   │
│     │ Position: 9 (Sacred)       │   │
│     └─────────────────────────────┘   │
│                                         │
├─────────────────────────────────────────┤
│  Type your message...                  │
│  🚀 Send                                │
└─────────────────────────────────────────┘
```

---

## 🎯 Next Steps

1. **✅ Done**: Beautiful frontend with ELP visualization
2. **⏳ Next**: Implement backend API endpoint
3. **🔜 After**: Add voice, image, audio, 3D, multimodal

### Phase 1 Complete When:
- [ ] Frontend running at localhost:5173
- [ ] Backend running at localhost:8080
- [ ] Chat messages get ELP analysis
- [ ] Signal strength displayed
- [ ] Flux position calculated

### Start Here:
```bash
# Terminal 1: Mock API (for testing)
cd web && bun mock-api.js

# Terminal 2: Frontend
cd web && bun run dev

# Terminal 3: Real backend (when ready)
cargo run --release
```

---

## 📚 Related Documentation

- **[MULTIMODAL_CHAT_ROADMAP.md](../implementation/MULTIMODAL_CHAT_ROADMAP.md)** - Complete 10-week plan
- **[MODALITIES.md](../architecture/MODALITIES.md)** - Multi-modal specifications
- **[ROOT_DIRECTORY_GUIDE.md](../../ROOT_DIRECTORY_GUIDE.md)** - Project structure

---

**Status**: Frontend ✅ Complete | Backend ⏳ Needs Implementation  
**Estimated Time**: 1-2 days for backend, then Phase 1 complete!

Let's build it! 🚀
