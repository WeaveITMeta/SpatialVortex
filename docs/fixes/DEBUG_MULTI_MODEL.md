# 🐛 Debugging Multi-Model Chat

## Current Issue

Only seeing 1 response instead of 4 responses.

## Steps to Debug

### 1. Restart Everything

```bash
# Terminal 1
ollama serve

# Terminal 2
cargo run --release --bin api_server --features agents,persistence,postgres,lake,burn-cuda-backend

# Terminal 3
cd web
bun run dev
```

### 2. Open Browser DevTools

Press **F12** to open browser console

### 3. Send a Test Message

Type: "Hello" or "Test"

### 4. Check Console Logs

You should see:
```
🔍 Received data: { model_responses: [...], vortex_consensus: {...} }
📊 Model responses: [...]
🌀 Vortex consensus: {...}
```

---

## What to Look For

### ✅ Success Case
```json
{
  "model_responses": [
    { "model_name": "llama3.2:latest", "text": "..." },
    { "model_name": "mixtral:8x7b", "text": "..." },
    { "model_name": "codellama:13b", "text": "..." }
  ],
  "vortex_consensus": {
    "text": "...",
    "confidence": 85.5,
    "flux_position": 6
  }
}
```

### ❌ Problem Case 1: Wrong Endpoint
If console shows:
```
POST http://localhost:7000/api/v1/chat/unified/stream
```

**Fix**: Frontend is using old endpoint. Hard refresh: **Ctrl+Shift+R**

### ❌ Problem Case 2: API Error
If console shows:
```
API error: 500
```

**Fix**: Check Terminal 2 (API server) for Rust errors

### ❌ Problem Case 3: Ollama Not Running
If console shows:
```
Failed to query Ollama models: connection refused
```

**Fix**: Start Ollama in Terminal 1: `ollama serve`

### ❌ Problem Case 4: Models Not Installed
If API logs show:
```
model 'mixtral:8x7b' not found
```

**Fix**:
```bash
ollama pull llama3.2:latest
ollama pull mixtral:8x7b
ollama pull codellama:13b
```

---

## Network Tab Check

1. Open **Network tab** in DevTools (F12)
2. Send a message
3. Look for: `dual-response`
4. Click on it
5. Check **Response** tab

Should see JSON with `model_responses` array and `vortex_consensus` object.

---

## If Still Seeing System Prompts

The system prompt text like "You are Vortex, an advanced AI assistant. CRITICAL FORMATTING RULES:" suggests Ollama models have custom system prompts configured.

**To fix**:
```bash
# Check Ollama modelfiles
ollama show llama3.2:latest

# If they have SYSTEM prompts, pull fresh versions
ollama rm llama3.2:latest
ollama pull llama3.2:latest
```

---

## Expected Timeline

- **First query**: 40-60 seconds (models loading)
- **After that**: 10-30 seconds

---

## Success Criteria

✅ See 4 separate message bubbles  
✅ 3 labeled with model names (gray)  
✅ 1 labeled "🌀 Vortex" (orange)  
✅ No system prompt text visible  

---

## Share Console Output

If still having issues, share the console output from browser DevTools!
