# 🚀 SpatialVortex ASI Model - Quick Start Guide

**You're almost there!** You've set up the minimum config. Here's what to do next.

---

## ✅ What You've Done

- [x] Created `.env` file with minimum configuration
- [x] Set `DATABASE_URL`, `REDIS_URL`, `GROK_API_KEY`
- [x] Code compiled successfully

---

## 🎯 Next Steps (5 minutes)

### Step 1: Verify Services Running (1 min)

**Check PostgreSQL**:
```powershell
# Test connection
psql -U username -d spatial_vortex -c "SELECT 1;"
# Should return: 1
```

**Check Redis**:
```powershell
# Test connection
redis-cli ping
# Should return: PONG
```

**If not running**, start them:
```powershell
# PostgreSQL (Windows service)
net start postgresql-x64-14

# Redis (if installed)
redis-server

# OR use Docker (easier):
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=password postgres:15
docker run -d -p 6379:6379 redis:7-alpine
```

---

### Step 2: Initialize Database (30 sec)

```powershell
# Run with --init-db flag to create tables
cargo run --release -- --init-db
```

This creates:
- `flux_matrices` table
- Indexes for fast lookups
- Schema for inference storage

---

### Step 3: Start the Server (30 sec)

```powershell
# Start the ASI Model server
cargo run --release
```

You should see:
```
✅ Connected to Redis at redis://127.0.0.1:6379
✅ Components initialized
🌐 Starting Spatial Vortex server at 127.0.0.1:8080
✨ Chat API available at http://127.0.0.1:8080/api/v1/chat/text
```

---

### Step 4: Test the API (1 min)

**Option A: Use the test script** (Recommended):
```powershell
# Run automated tests
.\test_chat.ps1
```

**Option B: Manual curl test**:
```powershell
curl -X POST http://localhost:8080/api/v1/chat/text `
  -H "Content-Type: application/json" `
  -d '{
    "message": "What is consciousness?",
    "user_id": "test_user"
  }'
```

**Expected Response**:
```json
{
  "response": "Your message analyzed through sacred geometry...",
  "elp_values": {
    "ethos": 8.5,
    "logos": 7.2,
    "pathos": 6.8
  },
  "confidence": 0.72,
  "flux_position": 9,
  "confidence": 0.87,
  "processing_time_ms": 150,
  "subject": "Consciousness"
}
```

---

### Step 5: Connect Frontend (2 min)

```powershell
# In a new terminal
cd web
bun run dev
```

Open browser to: **http://localhost:5173**

You now have:
- ✅ Backend API running (port 8080)
- ✅ Frontend UI running (port 5173)
- ✅ Full multi-modal chat interface

---

## 🎊 Success Indicators

### ✅ Backend Working
- [ ] Server starts without errors
- [ ] Health check returns 200: `curl http://localhost:8080/api/v1/health`
- [ ] Chat endpoint responds
- [ ] Redis shows connections

### ✅ Frontend Working
- [ ] UI loads at localhost:5173
- [ ] "Text" modality is active
- [ ] Can type in chat input
- [ ] Messages send and receive responses
- [ ] ELP visualization shows values

### ✅ Integration Working
- [ ] Frontend messages reach backend
- [ ] Backend responses display in UI
- [ ] ELP channels update in real-time
- [ ] Flux position shown correctly
- [ ] Sacred positions (3, 6, 9) highlight

---

## 🔍 Troubleshooting

### "Connection refused"
```powershell
# Check if server is running
netstat -ano | findstr :8080

# Restart server
cargo run --release
```

### "Redis unavailable"
```powershell
# Check Redis
redis-cli ping

# Start Redis
redis-server
# OR
docker run -d -p 6379:6379 redis:7-alpine
```

### "Database error"
```powershell
# Reinitialize schema
cargo run --release -- --init-db

# Check connection
psql -U username -d spatial_vortex -c "\dt"
```

### "GROK_API_KEY not working"
```powershell
# Test with fallback response (works without API key)
curl -X POST http://localhost:8080/api/v1/chat/text `
  -H "Content-Type: application/json" `
  -d '{"message": "Test", "user_id": "test"}'

# Should still return a fallback response
```

---

## 📊 API Endpoints

### Health Check
```
GET http://localhost:8080/api/v1/health
```

### Chat (Multi-modal)
```
POST http://localhost:8080/api/v1/chat/text
Body: {
  "message": "Your message here",
  "user_id": "user123",
  "context": ["optional", "previous", "messages"]
}
```

### Flux Matrix Generation
```
POST http://localhost:8080/api/v1/flux/matrix/generate
Body: {
  "subject": "Subject Name",
  "use_ai_generation": true,
  "sacred_guides_enabled": true
}
```

### All Subjects
```
GET http://localhost:8080/api/v1/subjects
```

---

## 🌀 Understanding the Response

### ELP Values (Ethos, Logos, Pathos)
- **Ethos** (-13 to +13): Character/Ethics dimension
- **Logos** (-13 to +13): Logic/Reason dimension
- **Pathos** (-13 to +13): Emotion/Empathy dimension

### Confidence (0.0 to 1.0)
- **0.7-1.0**: Strong 3-6-9 coherence ✨
- **0.5-0.7**: Moderate coherence
- **0.0-0.5**: Weak coherence

### Flux Position (0-9)
- **Position 0**: Divine Source (balanced)
- **Position 3**: Sacred Trinity ✨ (Ethos anchor)
- **Position 6**: Sacred Balance ✨ (Pathos anchor)
- **Position 9**: Sacred Completion ✨ (Logos anchor)
- **Positions 1,2,4,5,7,8**: Vortex flow pattern

### Confidence (0.0 to 1.0)
- Boosted +15% for sacred positions (3, 6, 9)
- Boosted +10% for position 0 (balanced)
- Based on signal strength + position

---

## 🎯 What's Working

### ✅ Backend Features
- ONNX text embedding (placeholder ready for real model)
- Sacred geometry transformation
- ELP channel analysis
- Flux position calculation
- AI Router with Grok 4 integration
- Fallback responses (works without API key)
- Dynamic subject detection
- Consensus mode support

### ✅ Frontend Features
- Beautiful chat interface
- Real-time ELP visualization
- Signal strength display
- Flux position indicator
- Sacred position highlighting
- Modality selector (text active, others coming soon)

---

## 📈 Performance

### Current (Placeholder ONNX)
- **Embedding**: ~1ms (deterministic hash)
- **Sacred Transform**: <1ms
- **AI Response**: 500-2000ms (Grok 4 API)
- **Total**: ~500-2000ms per message

### With Real ONNX Model
- **Embedding**: ~10-50ms (sentence-transformers)
- **Sacred Transform**: <1ms
- **AI Response**: 500-2000ms (Grok 4 API)
- **Total**: ~510-2050ms per message

---

## 🚀 Next Features to Enable

### Short-term (This Week)
1. **Real ONNX Model**
   - Download sentence-transformers/all-MiniLM-L6-v2
   - Update `SPATIALVORTEX_ONNX_MODEL_PATH`
   - Restart server

2. **Additional AI Providers** (Consensus Mode)
   - Add `OPENAI_API_KEY` to .env
   - Add `ANTHROPIC_API_KEY` to .env
   - Enable consensus mode in code

3. **Rate Limiting**
   - Configure `RATE_LIMIT_PER_MINUTE`
   - Add rate limiter middleware

### Long-term (Next Month)
1. **Voice Input** (Phase 2)
2. **Image Input** (Phase 3)
3. **Audio Input** (Phase 4)
4. **3D Point Cloud** (Phase 5)
5. **Multi-modal Fusion** (Phase 6)

---

## 📚 Documentation

- **Backend API**: `docs/implementation/BACKEND_API_IMPLEMENTATION.md`
- **Multi-modal Roadmap**: `docs/implementation/MULTIMODAL_CHAT_ROADMAP.md`
- **Quick Start**: `docs/guides/MULTIMODAL_CHAT_QUICKSTART.md`
- **Config Reference**: `.env.example`

---

## ✅ Checklist

- [ ] Services running (PostgreSQL + Redis)
- [ ] Database initialized
- [ ] Server starts successfully
- [ ] Health check passes
- [ ] Chat endpoint responds
- [ ] Frontend connects
- [ ] Messages send/receive
- [ ] ELP values display
- [ ] Sacred positions highlight

---

## 🎊 You're Ready!

Your ASI Model is now:
- ✅ Running locally
- ✅ Connected to databases
- ✅ Processing messages
- ✅ Analyzing sacred geometry
- ✅ Generating AI responses
- ✅ Visualizing results

**Next**: Try sending different types of messages and watch the ELP channels change! 🌀✨

---

**Need Help?**
- Check `docs/` for detailed guides
- Run `.\test_chat.ps1` for automated testing
- See `TROUBLESHOOTING.md` for common issues
