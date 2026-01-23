# SpatialVortex API Migration Guide - v0.7.0

**Date**: October 31, 2025  
**Breaking Changes**: Yes - `confidence` field removed

---

## 🚨 Breaking Changes

### API Field Consolidation: `confidence` → `confidence`

We've consolidated the duplicate metrics into a single `confidence` field for clarity and simplicity.

---

## 📊 What Changed

### Before (v0.6.x)

API responses had **two** similar fields:

```json
{
  "confidence": 0.85,  // ❌ REMOVED
  "confidence": 0.90
}
```

This was confusing because:
- Both measured trustworthiness (0.0-1.0)
- Users didn't know which to use
- Maintained for "backward compatibility" that nobody needed

### After (v0.7.0)

API responses now have **one** clear field:

```json
{
  "confidence": 0.90  // ✅ Single source of truth
}
```

---

## 🔄 Migration Instructions

### 1. ONNX Embedding Endpoint

**Endpoint**: `POST /api/v1/ml/embed`

**Before**:
```json
{
  "text": "Hello world",
  "embedding": [0.1, 0.2, ...],
  "embedding_dim": 384,
  "confidence": 0.75,  // ❌ REMOVED
  "elp_channels": {
    "ethos": 6.5,
    "logos": 7.0,
    "pathos": 5.5
  }
}
```

**After**:
```json
{
  "text": "Hello world",
  "embedding": [0.1, 0.2, ...],
  "embedding_dim": 384,
  "confidence": 0.75,  // ✅ Use this
  "elp_channels": {
    "ethos": 6.5,
    "logos": 7.0,
    "pathos": 5.5
  }
}
```

**Migration Code**:
```python
# Before
response = requests.post("/api/v1/ml/embed", json=request_data)
quality = response.json()["confidence"]  # ❌ No longer exists

# After
response = requests.post("/api/v1/ml/embed", json=request_data)
quality = response.json()["confidence"]  # ✅ Use this
```

```rust
// Before
let quality = response.confidence;  // ❌ Compile error

// After
let quality = response.confidence;  // ✅ Works
```

```javascript
// Before
const quality = response.confidence;  // ❌ undefined

// After
const quality = response.confidence;  // ✅ Works
```

---

### 2. ASI Inference Endpoint

**Endpoint**: `POST /api/v1/ml/asi/infer`

**Before**:
```json
{
  "text": "Truth and justice prevail",
  "flux_position": 9,
  "position_archetype": "Divine/Righteous",
  "elp_values": {"ethos": 8.5, "logos": 7.5, "pathos": 5.0},
  "confidence": 0.85,  // ❌ REMOVED
  "confidence": 0.90,
  "lake_worthy": true,
  "interpretation": "High moral alignment..."
}
```

**After**:
```json
{
  "text": "Truth and justice prevail",
  "flux_position": 9,
  "position_archetype": "Divine/Righteous",
  "elp_values": {"ethos": 8.5, "logos": 7.5, "pathos": 5.0},
  "confidence": 0.90,  // ✅ Single field
  "lake_worthy": true,
  "interpretation": "High moral alignment..."
}
```

**Migration Code**:
```python
# Before
result = asi_client.infer("Truth and justice")
if result["confidence"] > 0.8:  # ❌ KeyError
    store_in_lake(result)

# After
result = asi_client.infer("Truth and justice")
if result["confidence"] > 0.8:  # ✅ Works
    store_in_lake(result)
```

---

### 3. RAG Search Endpoint

**Endpoint**: `POST /api/v1/rag/search`

**Before**:
```json
{
  "query": "sacred geometry",
  "filters": {
    "confidence_min": 0.7  // ❌ REMOVED
  }
}
```

**After**:
```json
{
  "query": "sacred geometry",
  "filters": {
    "confidence_min": 0.7  // ✅ Use this
  }
}
```

**Migration Code**:
```python
# Before
response = rag_client.search(
    "sacred geometry",
    filters={"confidence_min": 0.7}  # ❌ Ignored
)

# After
response = rag_client.search(
    "sacred geometry",
    filters={"confidence_min": 0.7}  # ✅ Works
)
```

---

### 4. RAG Statistics Endpoint

**Endpoint**: `GET /api/v1/rag/embeddings/stats`

**Before**:
```json
{
  "total_embeddings": 1247,
  "dimensions": 384,
  "sacred_position_distribution": {
    "3": 156,
    "6": 187,
    "9": 203
  },
  "average_confidence": 0.76,  // ❌ REMOVED
  "storage_mb": 45.3
}
```

**After**:
```json
{
  "total_embeddings": 1247,
  "dimensions": 384,
  "sacred_position_distribution": {
    "3": 156,
    "6": 187,
    "9": 203
  },
  "average_confidence": 0.76,  // ✅ Renamed
  "storage_mb": 45.3
}
```

---

## 🔍 Quick Migration Checklist

### For API Consumers

- [ ] **Find all references** to `confidence` in your code
  ```bash
  # Unix/Linux/Mac
  grep -r "confidence" .
  
  # Windows PowerShell
  Select-String -Path . -Pattern "confidence" -Recurse
  ```

- [ ] **Replace with `confidence`**:
  - Python: `response["confidence"]` → `response["confidence"]`
  - JavaScript: `response.confidence` → `response.confidence`
  - Rust: `response.confidence` → `response.confidence`
  - TypeScript: Update interface definitions

- [ ] **Update filter parameters**:
  - `confidence_min` → `confidence_min`
  - `average_confidence` → `average_confidence`

- [ ] **Test your integration**:
  ```python
  # Test script
  response = api_client.infer("test")
  assert "confidence" in response
  assert "confidence" not in response  # Should pass
  ```

---

## 💡 Why This Change?

### Problem
Having both `confidence` and `confidence` was confusing:
- Users didn't know which to use
- Documentation was inconsistent
- APIs looked redundant
- Internal code had to maintain both

### Solution
- **Public APIs**: Use `confidence` (user-facing metric)
- **Internal VCP code**: Still uses `confidence` (mathematical term for 3-6-9 pattern coherence)
- **Result**: Clear separation of concerns

### Benefits
✅ **Clearer API**: One metric to rule them all  
✅ **Better docs**: No confusion about which field to use  
✅ **Simpler code**: No duplicate field handling  
✅ **Consistent**: All endpoints use the same field name

---

## 🛠️ Language-Specific Migration Examples

### Python Client

```python
# Before (v0.6.x)
class SpatialVortexClient:
    def get_quality(self, response):
        # Which one to use? 🤔
        return response.get("confidence") or response.get("confidence")

# After (v0.7.0)
class SpatialVortexClient:
    def get_quality(self, response):
        return response["confidence"]  # Clear! ✅
```

### JavaScript/TypeScript Client

```typescript
// Before (v0.6.x)
interface ASIResponse {
  confidence?: number;  // ❌ Remove
  confidence: number;
}

function getQuality(response: ASIResponse): number {
  return response.confidence ?? response.confidence;  // Confusing
}

// After (v0.7.0)
interface ASIResponse {
  confidence: number;  // ✅ Single field
}

function getQuality(response: ASIResponse): number {
  return response.confidence;  // Clear!
}
```

### Rust Client

```rust
// Before (v0.6.x)
#[derive(Deserialize)]
struct ASIResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    confidence: Option<f64>,  // ❌ Remove
    confidence: f64,
}

fn get_quality(response: &ASIResponse) -> f64 {
    response.confidence.unwrap_or(response.confidence)  // Confusing
}

// After (v0.7.0)
#[derive(Deserialize)]
struct ASIResponse {
    confidence: f64,  // ✅ Single field
}

fn get_quality(response: &ASIResponse) -> f64 {
    response.confidence  // Clear!
}
```

---

## 📚 Updated Documentation

All documentation has been updated:

- ✅ `docs/api/API_ENDPOINTS.md` - API reference updated
- ✅ `docs/getting-started/README.md` - Examples updated (if exists)
- ✅ `src/ai/endpoints.rs` - Code comments updated
- ✅ API structs - Field removed

---

## 🆘 Need Help?

### If you see errors:

**Error**: `KeyError: 'confidence'` (Python)  
**Solution**: Replace with `response["confidence"]`

**Error**: `undefined` (JavaScript)  
**Solution**: Replace with `response.confidence`

**Error**: `field not found: confidence` (Rust)  
**Solution**: Update your struct definition to use `confidence`

### Migration Support

If you need help migrating:
1. Check this guide first
2. Review updated examples in `examples/` directory
3. Check API documentation: `docs/api/API_ENDPOINTS.md`
4. Run tests to verify your integration still works

---

## 📝 Summary

| Old Field | New Field | Applies To |
|-----------|-----------|------------|
| `confidence` | `confidence` | All API responses |
| `confidence_min` | `confidence_min` | Filter parameters |
| `average_confidence` | `average_confidence` | Statistics |

**Migration Time**: ~5 minutes (simple find-and-replace)  
**Breaking**: Yes, but easy to fix  
**Benefits**: Much clearer API, better developer experience

---

**Version**: 0.7.0  
**Release Date**: October 31, 2025  
**Migration Deadline**: None (update at your convenience, but old field won't work)
