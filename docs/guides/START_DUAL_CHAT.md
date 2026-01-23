# 🚀 Starting the Dual Chat System

## What You'll Get

- **First Message**: Consensus from Ollama models (llama3.2, mixtral, codellama) - Blue bubble
- **Second Message**: Native Rust AI from SpatialVortex - **Orange bubble with white text**

---

## 🎯 Quick Start

### 1. Start Ollama (Terminal 1)
```bash
ollama serve
```

Keep this running!

### 2. Start API Server (Terminal 2)
```bash
cargo run --bin api_server --features agents
```

API will be on **http://localhost:7000**

### 3. Start Frontend (Terminal 3)
```bash
cd web
bun run dev
```

Frontend will be on **http://localhost:28082**

### 4. Open Browser
```
http://localhost:28082
```

---

## ✅ What Changed

### Backend
- ✅ New endpoint: `/api/v1/chat/dual-response`
- ✅ Calls multi-model consensus (Ollama)
- ✅ Calls native Rust AI (ASIOrchestrator)
- ✅ Returns both responses

### Frontend
- ✅ Modified `Chat.svelte` to show TWO messages per query
- ✅ First message: Consensus AI (Blue, 🌀 avatar)
- ✅ Second message: Native AI (**Orange with white text**, ⚡ avatar)
- ✅ Updated ChatMessage type to support `'native'` role

---

## 🎨 Visual Differences

| Type | Avatar | Color | Text Color |
|------|--------|-------|------------|
| User | 👤 | Red gradient | White |
| Consensus AI | 🌀 | Gray/transparent | White |
| **Native Rust AI** | **⚡** | **Orange gradient** | **White** |

---

## 📡 API Endpoint

**POST** `/api/v1/chat/dual-response`

Request:
```json
{
  "message": "What is consciousness?"
}
```

Response:
```json
{
  "consensus": {
    "text": "Consciousness is...",
    "confidence": 92.5
  },
  "native": {
    "text": "From a sacred geometry perspective...",
    "confidence": 89.2,
    "flux_position": 6
  }
}
```

---

## 🔧 Files Modified

### Backend
- `src/ai/dual_response_api.rs` - New dual response endpoint
- `src/ai/mod.rs` - Register module
- `src/ai/api.rs` - Register route

### Frontend
- `web/src/lib/components/Chat.svelte` - Two-message display
- `web/src/lib/types/chat.ts` - Added 'native' role

---

## ✅ Expected Behavior

1. **Type a message** in the chat
2. **Wait ~30-40 seconds** (models are loading/processing)
3. **See TWO responses**:
   - First: **Blue bubble** from Consensus AI
   - Second: **Orange bubble with white text** from Native Rust AI

---

## 🎉 Success!

You now have dual AI responses with visual distinction - consensus in blue, native in orange with white text!
