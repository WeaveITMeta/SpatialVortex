# Swagger UI - Interactive API Documentation

**SpatialVortex** includes a beautiful, interactive API documentation interface powered by Swagger UI!

---

## 🌐 Access Swagger UI

### Local Development
```
http://localhost:8080/swagger-ui/
```

### Production
```
https://api.spatialvortex.ai/swagger-ui/
```

---

## ✨ Features

### 1. **Interactive Documentation**
- View all API endpoints
- See request/response schemas
- Try endpoints directly from the browser
- No Postman needed!

### 2. **Live Testing**
- Execute API calls right from the UI
- See real responses
- Test with different parameters
- Validate request/response formats

### 3. **Schema Exploration**
- Browse all data models
- See example values
- Understand field types
- Required vs optional fields

---

## 📖 Available Sections

### Health & Status
```
GET /api/v1/health
```
- Check system status
- Database connectivity
- Cache connectivity
- Inference engine stats

### Inference Operations
```
POST /api/v1/inference/reverse
POST /api/v1/inference/forward
```
- Reverse: Seeds → Meanings (3, 6, 9 → "Ethos", "Pathos", "Logos")
- Forward: Meanings → Seeds ("love", "truth" → [3, 9])

### Subject Management
```
GET  /api/v1/subjects
POST /api/v1/subjects/generate
```
- List available subjects
- Generate new subjects

---

## 🎯 How to Use

### 1. Start the Server
```powershell
.\target\release\api_server.exe
```

### 2. Open Swagger UI
Navigate to: **http://localhost:8080/swagger-ui/**

### 3. Try an Endpoint

#### Example: Reverse Inference
1. Click **`POST /api/v1/inference/reverse`**
2. Click **"Try it out"**
3. Modify the request body:
   ```json
   {
     "seed_numbers": [3, 6, 9],
     "subject_filter": "all",
     "include_synonyms": true
   }
   ```
4. Click **"Execute"**
5. See the response!

---

## 📊 Request Examples

### Health Check
```bash
# Click "Try it out" on GET /api/v1/health
# No parameters needed!
```

**Response**:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "database_status": "healthy",
  "cache_status": "healthy",
  "inference_engine_stats": {
    "total_matrices": 0,
    "cached_inferences": 0,
    "available_subjects": []
  }
}
```

### Reverse Inference
```json
{
  "seed_numbers": [3, 6, 9],
  "subject_filter": "all",
  "include_synonyms": true
}
```

**Response**:
```json
{
  "inference_id": "b8ae13f2-b9e1-459f-8aa8-3d2440093421",
  "inferred_meanings": [],
  "confidence_score": 0.0,
  "processing_time_ms": 1,
  "moral_alignment_summary": "Constructive: 0, Destructive: 0, Neutral: 0"
}
```

### Forward Inference
```json
{
  "target_meanings": ["love", "truth", "wisdom"],
  "subject_filter": "all",
  "max_results": 10
}
```

---

## 🔧 Advanced Features

### 1. **Download OpenAPI Spec**
```
http://localhost:8080/api-docs/openapi.json
```

Use this with:
- Postman (import)
- Code generators
- CI/CD validation
- Client library generation

### 2. **Schema Validation**
All requests are automatically validated against the OpenAPI schema:
- ✅ Required fields
- ✅ Data types
- ✅ Format validation
- ✅ Enum values

### 3. **Example Values**
Every schema includes example values for easy testing!

---

## 📝 Available Tags

### **health**
System health and status endpoints

### **inference**
AI inference operations (reverse & forward)

### **subjects**
Subject management and listing

### **flux**
Flux matrix operations (coming soon)

### **sacred-geometry**
Sacred geometry analysis (coming soon)

---

## 🚀 Production Deployment

### Enable in Production
Swagger UI is automatically available in production builds.

### Disable Swagger UI (Optional)
If you want to disable Swagger UI in production:

```rust
// In server.rs
#[cfg(debug_assertions)]  // Only in debug builds
.service(
    SwaggerUi::new("/swagger-ui/{_:.*}")
        .url("/api-docs/openapi.json", openapi.clone())
)
```

---

## 🎨 Customization

### Theme
Swagger UI uses the default theme. To customize:

1. Edit colors in the OpenAPI spec
2. Add custom CSS
3. Use Swagger UI configuration options

### Branding
Update the OpenAPI info section in `src/ai/swagger.rs`:

```rust
info(
    title = "Your API Name",
    description = "Your description",
    contact(name = "Your Team")
)
```

---

## 📖 OpenAPI Specification

### Current Version
**OpenAPI 3.0**

### Schemas Documented
- ✅ HealthResponse
- ✅ InferenceEngineStats
- ✅ ReverseInferenceRequest
- ✅ ReverseInferenceResponse
- ✅ ForwardInferenceRequest
- ✅ ForwardInferenceResponse
- ✅ SubjectInfo

### More Coming Soon
- FluxMatrixRequest
- FluxMatrixResponse
- SacredGeometryAnalysis
- OnnxInferenceRequest
- ConfidenceLakeStatus

---

## 🔍 Troubleshooting

### Swagger UI Not Loading

**Issue**: Page shows 404 or blank

**Fix**:
1. Verify server is running
2. Check URL: `http://localhost:8080/swagger-ui/` (note trailing slash)
3. Check console for errors

### API Calls Failing

**Issue**: "CORS error" or "Network error"

**Fix**:
1. Verify CORS is enabled in `config.toml`
2. Check server logs
3. Verify request body matches schema

### Schema Not Updating

**Issue**: Changes to schema not reflected

**Fix**:
1. Rebuild the server: `cargo build --release`
2. Restart the server
3. Hard refresh browser (Ctrl+Shift+R)

---

## 💡 Tips & Tricks

### 1. **Copy-Paste Requests**
Use the "Copy" button to copy cURL commands!

### 2. **Response Headers**
Check the "Response headers" section for debugging

### 3. **Schema Examples**
Click on schema names to see full object structure

### 4. **Try Different Values**
Experiment with different seed numbers:
- Sacred positions: 3, 6, 9
- Vortex sequence: 1, 2, 4, 8, 7, 5

---

## 📚 Related Documentation

- **API Endpoints**: `docs/API_ENDPOINTS.md`
- **Setup Guide**: `SETUP.md`
- **Testing**: `tests/README.md`

---

## 🎯 Quick Links

| Resource | URL |
|----------|-----|
| **Swagger UI** | http://localhost:8080/swagger-ui/ |
| **OpenAPI JSON** | http://localhost:8080/api-docs/openapi.json |
| **Health Check** | http://localhost:8080/api/v1/health |
| **GitHub** | https://github.com/WeaveSolutions/SpatialVortex |

---

## 🎉 Benefits

### For Developers
- ✅ No need for Postman
- ✅ Test endpoints instantly
- ✅ See real examples
- ✅ Validate requests easily

### For API Consumers
- ✅ Self-documenting API
- ✅ Interactive exploration
- ✅ Copy-paste ready code
- ✅ Always up-to-date

### For Teams
- ✅ Shared understanding
- ✅ Faster onboarding
- ✅ Reduced support tickets
- ✅ Better collaboration

---

**Enjoy exploring your API with Swagger UI!** 🚀✨

Access it now: **http://localhost:8080/swagger-ui/**
