# Chat Interface Native Integration Guide

**Goal**: Enable native inference in the SpatialVortex chat interface

---

## Current Architecture

```
User → /chat/text → AIRouter → Grok4/Consensus → External LLM
                                                      ↓
                                                 Response
```

**Problem**: ASIOrchestrator (native inference) is initialized but not used

---

## Proposed Architecture

```
User → /chat/text → AIRouter → ASIOrchestrator (native) → Response
                          ↓          ↓ (if confidence low)
                          └─→ Grok4/LLM → Response
```

---

## Implementation Options

### Option 1: Integrate into AIRouter (Recommended)

**File**: `src/ai/router.rs`

**Changes**:
1. Add `ASIOrchestrator` to `AIRouter` struct
2. Modify `generate_response()` to try native first
3. Fallback to external LLM if needed

**Code**:
```rust
// In AIRouter struct
pub struct AIRouter {
    // ... existing fields ...
    asi_orchestrator: Option<Arc<Mutex<ASIOrchestrator>>>,
    use_native_primary: bool,
}

impl AIRouter {
    pub fn with_asi(mut self, asi: Arc<Mutex<ASIOrchestrator>>) -> Self {
        self.asi_orchestrator = Some(asi);
        self
    }
    
    pub async fn generate_response(
        &mut self,
        message: &str,
        user_id: &str,
        subject: Option<&str>,
        confidence: f32,
        flux_position: u8,
    ) -> Result<String> {
        // Try native inference first
        if self.use_native_primary {
            if let Some(asi) = &self.asi_orchestrator {
                let mut asi_lock = asi.lock().await;
                
                if asi_lock.is_native_enabled() {
                    match asi_lock.process(message, ExecutionMode::Fast).await {
                        Ok(result) if result.native_used => {
                            tracing::info!("✅ Native inference used: {:.1}% confidence", 
                                result.confidence * 100.0);
                            return Ok(result.result);
                        }
                        Ok(_) => {
                            tracing::info!("🔄 Native confidence low, using LLM fallback");
                        }
                        Err(e) => {
                            tracing::warn!("⚠️ Native inference failed: {}, using LLM", e);
                        }
                    }
                }
            }
        }
        
        // Fallback to existing logic
        let context = self.build_context(message, confidence, flux_position, 
            &subject.unwrap_or("General"));
        
        if self.use_consensus {
            self.generate_consensus_response(&context).await
        } else {
            self.generate_grok4_response(&context).await
        }
    }
}
```

**Server Setup** (`src/ai/server.rs`):
```rust
let ai_router = Arc::new(RwLock::new(
    crate::ai::router::AIRouter::new(app_config.ai.api_key.clone(), false)
        .with_asi(asi_orchestrator.clone())  // Link ASI
));
```

**Environment Config**:
```bash
USE_NATIVE_INFERENCE=true
FALLBACK_TO_LLM=true
NATIVE_MIN_CONFIDENCE=0.6
NATIVE_PRIMARY=true  # Use native as primary for chat
```

---

### Option 2: Create Dedicated Native Endpoint

**File**: `src/ai/chat_native.rs` (new)

**Endpoint**: `POST /api/v1/chat/native`

**Code**:
```rust
use actix_web::{post, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ai::orchestrator::{ASIOrchestrator, ExecutionMode};
use crate::ai::api::AppState;

#[derive(Debug, Deserialize)]
pub struct NativeChatRequest {
    pub message: String,
    pub mode: Option<String>, // "fast", "balanced", "comprehensive"
}

#[derive(Debug, Serialize)]
pub struct NativeChatResponse {
    pub response: String,
    pub elp_values: ELPValues,
    pub confidence: f32,
    pub flux_position: u8,
    pub is_sacred: bool,
    pub native_used: bool,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct ELPValues {
    pub ethos: f32,
    pub logos: f32,
    pub pathos: f32,
}

#[post("/chat/native")]
pub async fn chat_native(
    req: web::Json<NativeChatRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let start = std::time::Instant::now();
    
    // Determine execution mode
    let mode = match req.mode.as_deref() {
        Some("fast") => ExecutionMode::Fast,
        Some("comprehensive") => ExecutionMode::Comprehensive,
        _ => ExecutionMode::Balanced,
    };
    
    // Process with ASI Orchestrator
    let mut asi = state.asi_orchestrator.lock().await;
    let result = asi.process(&req.message, mode).await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let elapsed = start.elapsed().as_millis() as u64;
    
    Ok(HttpResponse::Ok().json(NativeChatResponse {
        response: result.result,
        elp_values: ELPValues {
            ethos: result.elp.ethos,
            logos: result.elp.logos,
            pathos: result.elp.pathos,
        },
        confidence: result.confidence,
        flux_position: result.flux_position,
        is_sacred: result.is_sacred,
        native_used: result.native_used,
        processing_time_ms: elapsed,
    }))
}
```

**Register Endpoint** (`src/ai/api.rs`):
```rust
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Existing endpoints
            .service(chat_api::chat_text)
            // NEW: Native chat endpoint
            .service(chat_native::chat_native)
            // ... other endpoints
    );
}
```

---

## Configuration

### Environment Variables

```bash
# Native Inference Control
USE_NATIVE_INFERENCE=true       # Enable native as primary
FALLBACK_TO_LLM=true           # Allow LLM fallback
NATIVE_MIN_CONFIDENCE=0.6      # 60% confidence threshold

# Chat Router
CHAT_USE_NATIVE=true           # Use native for /chat/text
NATIVE_PRIMARY=true            # Try native before external LLM

# External LLM (Optional)
GROK_API_KEY=your_key_here
OLLAMA_MODEL=llama3.2:latest
```

### config.toml

```toml
[ai]
use_native = true
fallback_to_llm = true
native_min_confidence = 0.6

[chat]
use_native_primary = true
mode = "balanced"  # fast, balanced, comprehensive
```

---

## Testing

### Test Native Endpoint

```bash
# POST /api/v1/chat/native
curl -X POST http://localhost:7000/api/v1/chat/native \
  -H "Content-Type: application/json" \
  -d '{
    "message": "What is consciousness?",
    "mode": "balanced"
  }'
```

**Expected Response**:
```json
{
  "response": "Logically speaking, 'What is consciousness?' can be understood...",
  "elp_values": {
    "ethos": 5.2,
    "logos": 7.8,
    "pathos": 4.1
  },
  "confidence": 0.73,
  "flux_position": 6,
  "is_sacred": true,
  "native_used": true,
  "processing_time_ms": 42
}
```

### Test Fallback

```bash
# Set low confidence threshold to trigger fallback
export NATIVE_MIN_CONFIDENCE=0.95

# Same request should fallback to LLM
curl -X POST http://localhost:7000/api/v1/chat/native ...
```

**Expected**:
```json
{
  ...
  "native_used": false,  // Fell back to LLM
  "confidence": 0.88     // Below 0.95 threshold
}
```

---

## Performance Comparison

| Metric | Native | LLM (Ollama) | LLM (Grok4) |
|--------|--------|--------------|-------------|
| Latency | 20-50ms | 200-500ms | 500-2000ms |
| Memory | 500MB | 6GB | N/A (external) |
| Offline | ✅ Yes | ✅ Yes | ❌ No |
| Accuracy | 85%+ | 90%+ | 95%+ |
| Cost | $0 | $0 | $$$ |

---

## Migration Path

### Phase 1: Add Native Endpoint (Parallel)
- ✅ Keep existing `/chat/text` unchanged
- ✅ Add `/chat/native` for testing
- ✅ Compare responses

### Phase 2: Hybrid Mode
- ✅ Update AIRouter to try native first
- ✅ Fallback to LLM if needed
- ✅ Monitor confidence metrics

### Phase 3: Native Primary
- ✅ Make native default for `/chat/text`
- ✅ LLM as fallback only
- ✅ Reduce external API costs

---

## Recommendation

**Use Option 1 (Integrate into AIRouter)**

**Why**:
- ✅ Seamless user experience (same endpoint)
- ✅ Automatic fallback logic
- ✅ Configurable via environment
- ✅ No frontend changes needed

**Next Steps**:
1. Modify `AIRouter` to include `ASIOrchestrator`
2. Add configuration flags
3. Test with existing chat frontend
4. Monitor native vs fallback ratio
5. Tune confidence threshold

**Timeline**: ~2-4 hours implementation + testing

---

## Example Implementation

See: `examples/chat_native_integration.rs` (to be created)

```bash
cargo run --example chat_native_integration --features agents
```
