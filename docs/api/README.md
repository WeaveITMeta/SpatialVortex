# SpatialVortex API Documentation

RESTful API reference and integration guides for SpatialVortex.

---

## 📚 Available Documentation

### Core API Reference

**[API_ENDPOINTS.md](API_ENDPOINTS.md)** - Complete API Reference
- All REST endpoints
- Request/response formats
- Authentication & authorization
- Rate limiting
- Error handling

### Specialized APIs

**[VISUAL_SUBJECT_GENERATION.md](VISUAL_SUBJECT_GENERATION.md)** - Subject Generation API
- Visual subject generation endpoints
- Subject domain APIs (physics, ethics, logic, emotion)
- Semantic mapping
- ELP tensor responses

**[SWAGGER_UI.md](SWAGGER_UI.md)** - Interactive Documentation
- Swagger/OpenAPI specification
- Interactive API testing
- Code generation
- Schema definitions

---

## 🚀 Quick Start

### Basic API Usage

```bash
# Start the API server
cargo run --bin api_server

# Server runs on http://localhost:7000
```

### Example Requests

**Generate a Beam**:
```bash
curl -X POST http://localhost:7000/api/beam \
  -H "Content-Type: application/json" \
  -d '{"seed": 42}'
```

**Subject Generation**:
```bash
curl -X POST http://localhost:7000/api/subject/generate \
  -H "Content-Type: application/json" \
  -d '{"domain": "physics", "seed": 123}'
```

**Health Check**:
```bash
curl http://localhost:7000/health
```

---

## 🔑 Authentication

**Current Status**: ⚠️ Basic implementation

### API Key Authentication (Planned)

```bash
curl -X GET http://localhost:7000/api/endpoint \
  -H "Authorization: Bearer YOUR_API_KEY"
```

See [API_ENDPOINTS.md](API_ENDPOINTS.md) for authentication details.

---

## 📊 Response Formats

### Standard Response Structure

**Success Response**:
```json
{
  "success": true,
  "data": {
    // Response data
  },
  "meta": {
    "timestamp": "2025-10-27T09:00:00Z",
    "version": "2.0.0"
  }
}
```

**Error Response**:
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid seed value",
    "details": {}
  }
}
```

---

## 🎯 Common Endpoints

### Beam Operations

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/beam` | POST | Generate new beam |
| `/api/beam/{id}` | GET | Get beam by ID |
| `/api/beam/{id}/signal` | GET | Get signal strength |

### Subject Generation

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/subject/generate` | POST | Generate subject |
| `/api/subject/{domain}` | GET | List domain subjects |
| `/api/subject/{id}` | GET | Get subject details |

### Inference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/inference` | POST | Run inference |
| `/api/inference/batch` | POST | Batch inference |
| `/api/inference/{id}/status` | GET | Check status |

### Hallucination Detection

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/hallucination/detect` | POST | Detect hallucinations |
| `/api/hallucination/metrics` | GET | Get metrics |

See [API_ENDPOINTS.md](API_ENDPOINTS.md) for complete endpoint list.

---

## 🔧 Client Libraries

### Rust Client

```rust
use spatial_vortex::client::ApiClient;

let client = ApiClient::new("http://localhost:7000");
let beam = client.generate_beam(42).await?;
```

### Python Client (Planned)

```python
from spatial_vortex import ApiClient

client = ApiClient("http://localhost:7000")
beam = client.generate_beam(42)
```

### TypeScript Client (Planned)

```typescript
import { ApiClient } from 'spatial-vortex';

const client = new ApiClient('http://localhost:7000');
const beam = await client.generateBeam(42);
```

---

## 📈 Rate Limiting

**Current Limits**:
- 100 requests/minute per API key
- 1000 requests/hour per API key

**Headers**:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1698412800
```

---

## 🐛 Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Invalid request data |
| `UNAUTHORIZED` | 401 | Missing/invalid auth |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `RATE_LIMIT` | 429 | Rate limit exceeded |
| `SERVER_ERROR` | 500 | Internal server error |

---

## 🔗 Integration Guides

For integrating with external systems:
- [../integration/](../integration/) - Integration documentation
- [../guides/](../guides/) - Tutorial guides

---

## 🆘 Troubleshooting

### Connection Issues

**Problem**: Cannot connect to API server

```bash
# Check if server is running
curl http://localhost:7000/health

# Check logs
tail -f logs/api_server.log
```

### Authentication Errors

**Problem**: 401 Unauthorized

- Verify API key is valid
- Check Authorization header format
- Ensure API key has required permissions

### Performance Issues

**Problem**: Slow responses

- Check server logs for bottlenecks
- Monitor Redis connection (if enabled)
- Review [../guides/PROCESSING_SPEED.md](../guides/PROCESSING_SPEED.md)

---

## 📊 API Status

| Feature | Status | Notes |
|---------|--------|-------|
| Basic endpoints | ⚠️ Partial | Core endpoints working |
| Authentication | ❌ Planned | Basic implementation |
| Rate limiting | ❌ Planned | Not yet implemented |
| Swagger UI | ⚠️ Partial | Basic spec exists |
| Client libraries | ❌ Planned | Rust only (internal) |
| Webhooks | ❌ Planned | Not yet implemented |

---

## 🚧 Roadmap

See [../planning/NEXT_STEPS_FOR_YOU.md](../planning/NEXT_STEPS_FOR_YOU.md) for API development priorities.

**Upcoming Features**:
1. Complete authentication system
2. Rate limiting implementation
3. Webhook support
4. Client library releases
5. GraphQL endpoint (planned)

---

## 📖 Additional Resources

- **[../guides/BUILD_COMMANDS.md](../guides/BUILD_COMMANDS.md)** - Build API server
- **[../architecture/AI_ROUTER.md](../architecture/AI_ROUTER.md)** - AI routing architecture
- **[../status/PROJECT_STATUS.md](../status/PROJECT_STATUS.md)** - Current development status

---

**API Version**: 2.0 (Alpha)  
**Last Updated**: October 27, 2025  
**Maintainer**: SpatialVortex API Team
