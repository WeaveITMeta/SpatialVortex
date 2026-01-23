# Quick Start - SpatialVortex Web UI

## ✅ Fixed! Run This Now

### Option 1: Frontend Only (Works Immediately)

```powershell
# Just start the frontend
cd e:\Libraries\SpatialVortex\web
bun run dev

# Open browser
# http://localhost:28082
```

**Status**: The UI will work but show "Backend Offline" since we haven't created the Rust backend yet.

### Option 2: With Mock Backend (Future)

The `backend-rs` directory doesn't exist yet. That's the actual Rust API server we planned but haven't implemented.

**To create it later**:
1. Create `backend-rs/` directory
2. Set up Actix-Web server on port 28080
3. Implement endpoints from `docs/OPENWEBUI_RUST_FORK.md`

---

## What You'll See Now

### ✅ Working
- Chat interface loads
- Model selector appears
- 3D panel shows (empty canvas)
- Input field works
- Settings panel works
- Beautiful dark theme

### ⚠️ Not Working (Expected)
- Backend shows "offline" (no backend yet)
- Can't send messages (needs backend)
- 3D visualization empty (needs WASM binary)

---

## Next Steps to Make It Fully Functional

### Step 1: Create Simple Mock Backend (Optional - 15 mins)

Create `backend-rs/Cargo.toml`:
```toml
[package]
name = "spatialvortex-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.9"
actix-cors = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
```

Create `backend-rs/src/main.rs`:
```rust
use actix_web::{web, App, HttpServer, HttpResponse};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ChatRequest {
    prompt: String,
    model: Option<String>,
    compress: Option<bool>,
}

#[derive(Serialize)]
struct ChatResponse {
    response: String,
    model: String,
    thinking_time: f32,
    compressed_hash: Option<String>,
    beam_position: Option<u8>,
    elp_channels: Option<ELPChannels>,
}

#[derive(Serialize)]
struct ELPChannels {
    ethos: f32,
    logos: f32,
    pathos: f32,
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "backend": "rust-actix",
        "version": "0.1.0"
    }))
}

async fn chat(req: web::Json<ChatRequest>) -> HttpResponse {
    // Mock response for testing
    HttpResponse::Ok().json(ChatResponse {
        response: format!("Echo: {}", req.prompt),
        model: req.model.clone().unwrap_or("llama2".to_string()),
        thinking_time: 0.1,
        compressed_hash: Some("a3f7c29e8b4d1506f2a8".to_string()),
        beam_position: Some(9),
        elp_channels: Some(ELPChannels {
            ethos: 9.0,
            logos: 8.5,
            pathos: 7.0,
        }),
    })
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 SpatialVortex Backend starting on http://localhost:28080");
    
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        
        App::new()
            .wrap(cors)
            .route("/health", web::get().to(health))
            .route("/api/chat", web::post().to(chat))
    })
    .bind(("127.0.0.1", 28080))?
    .run()
    .await
}
```

Then run:
```powershell
cd e:\Libraries\SpatialVortex\backend-rs
cargo run
```

### Step 2: Build WASM (Future - 15 mins)

```powershell
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release --bin flux_matrix --features bevy_support
wasm-bindgen target/wasm32-unknown-unknown/release/flux_matrix.wasm --out-dir web/static/bevy --target web
```

---

## Current Status

**You have**: Beautiful fully-typed chat interface  
**You need**: Backend API + WASM binary  
**Time to complete**: 30 minutes for mock backend + WASM

---

## Test It Now!

```powershell
cd e:\Libraries\SpatialVortex\web
bun run dev
```

Then open: http://localhost:28082

You'll see the interface working, just without backend connectivity! 🎉
