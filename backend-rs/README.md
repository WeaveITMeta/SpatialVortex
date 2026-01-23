# SpatialVortex Backend - Mock Server

A simple Rust/Actix-Web mock backend for testing the SpatialVortex web interface.

## Features

- ✅ **Mock AI responses** - Generates contextual responses based on your prompts
- ✅ **12-byte compression** - Simulates hash generation
- ✅ **ELP channel calculation** - Dynamic ethos/logos/pathos based on content
- ✅ **Beam positioning** - Maps messages to sacred geometry positions (0-9)
- ✅ **CORS enabled** - Works with frontend on different port
- ✅ **Fast responses** - 200ms simulated thinking time

## Quick Start

```bash
# Build and run
cargo run

# Server starts on http://localhost:28080
```

## API Endpoints

### GET /health
```json
{
  "status": "healthy",
  "backend": "rust-actix",
  "version": "0.1.0"
}
```

### POST /api/chat
Request:
```json
{
  "prompt": "What is consciousness?",
  "model": "spatialvortex-mock",
  "compress": true
}
```

Response:
```json
{
  "response": "AI generated response...",
  "model": "spatialvortex-mock",
  "thinking_time": 0.2,
  "compressed_hash": "a3f7c29e8b4d1506f2a8",
  "beam_position": 9,
  "elp_channels": {
    "ethos": 8.5,
    "logos": 8.0,
    "pathos": 7.0
  },
  "confidence": 0.95
}
```

### GET /api/models
```json
[
  {
    "id": "spatialvortex-mock",
    "name": "SpatialVortex Mock",
    "size": "12B"
  },
  {
    "id": "llama2",
    "name": "Llama 2 (Mock)",
    "size": "7B"
  }
]
```

## How It Works

1. **Content Analysis**: Analyzes prompt for keywords
2. **ELP Calculation**: Computes ethics/logic/emotion scores
3. **Position Mapping**: Maps to sacred geometry (0-9)
4. **Hash Generation**: Creates 12-byte compression hash
5. **Response**: Returns contextual AI response with metadata

## Test Commands

```bash
# Health check
curl http://localhost:28080/health

# Chat request
curl -X POST http://localhost:28080/api/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt": "What is consciousness?", "compress": true}'

# List models
curl http://localhost:28080/api/models
```

## Next Steps

Replace this mock server with:
1. Real Ollama/LLM integration
2. Actual 12-byte compression algorithm
3. Real ELP tensor calculation
4. Confidence Lake storage
5. ONNX inference engine

## Dependencies

- `actix-web` - Web framework
- `actix-cors` - CORS middleware
- `serde` - Serialization
- `tokio` - Async runtime
