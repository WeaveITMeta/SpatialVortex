# Testing SpatialVortex AGI

## ✅ What We Built

### **Core AGI System** (`src/ai/flux_reasoning.rs`)
- **500+ lines** of geometric reasoning substrate
- Thinks in **flux matrices**, not language
- **Entropy-based oracle queries** - only asks LLMs when uncertain
- **Sacred geometry checkpoints** (3, 6, 9) for consolidation
- **Vortex flow** (1→2→4→8→7→5→1) through reasoning space

### **API Endpoint** (`src/ai/agi_api.rs`)
- RESTful AGI interface at `/api/v1/agi`
- Returns full reasoning chain (optional)
- Shows oracle queries made
- Tracks confidence and entropy

---

## 🧪 Testing

### **1. Check Compilation**

```powershell
cargo check --lib --features "agents,persistence,postgres,lake"
```

**Expected**: ✅ Clean compile (warnings OK)

---

### **2. Run Example Demo**

```powershell
cargo run --example agi_demo --features "agents,persistence"
```

**Expected Output**:
```
🧠 SpatialVortex AGI Demo
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Query: What is quantum entanglement?
Type: Factual query - should query physics oracle

📊 Initial Flux State:
   Ethos:  5.00
   Logos:  8.00
   Pathos: 6.00
   Entropy: 0.90 (MissingFacts)
   Vortex Position: 1

🔄 Reasoning Process:
   🔮 Oracle query: What are the key facts about...
   ✨ Flux update applied → position 2, entropy 0.60
   ...

✅ Reasoning Complete!
   Steps: 5
   Sacred Milestones: [3, 6]
   Final Confidence: 75%
   Oracle Queries: 2

💬 Final Answer:
   [Synthesized answer from oracle responses]
```

---

### **3. Test API Endpoint**

**Start API Server:**
```powershell
cargo run --bin api_server --features "agents,persistence,postgres,lake"
```

**Send Test Query:**
```powershell
# Using curl (if installed)
curl -X POST http://localhost:7000/api/v1/agi `
  -H "Content-Type: application/json" `
  -d '{"query": "How do I reverse type 2 diabetes?", "max_steps": 20, "include_chain": true}'

# OR using Invoke-WebRequest
$body = @{
    query = "How do I reverse type 2 diabetes?"
    max_steps = 20
    include_chain = $true
} | ConvertTo-Json

Invoke-WebRequest -Uri "http://localhost:7000/api/v1/agi" `
  -Method POST `
  -ContentType "application/json" `
  -Body $body
```

**Expected Response:**
```json
{
  "answer": "Based on my reasoning (confidence: 85%, 8 steps):\n\n...",
  "confidence": 85.0,
  "final_entropy": 0.25,
  "steps_taken": 8,
  "oracle_queries": 3,
  "sacred_milestones": [3, 6, 9],
  "converged": true,
  "reasoning_chain": [
    {
      "step": 1,
      "vortex_position": 1,
      "ethos": 5.0,
      "logos": 8.0,
      "pathos": 6.0,
      "certainty": 0.3,
      "entropy": 0.9,
      "oracle_query": null,
      "trace": "Initial query analysis..."
    },
    ...
  ],
  "processing_time_ms": 2345
}
```

---

### **4. Health Check**

```powershell
curl http://localhost:7000/api/v1/agi/health

# OR
Invoke-WebRequest -Uri "http://localhost:7000/api/v1/agi/health"
```

**Expected:**
```json
{
  "status": "healthy",
  "system": "SpatialVortex AGI",
  "capabilities": [
    "flux_reasoning",
    "geometric_thought",
    "oracle_queries",
    "sacred_geometry",
    "self_consolidation"
  ],
  "version": "1.0.0-alpha"
}
```

---

## 📊 What to Observe

### **Key Metrics**

| Metric | Meaning | Good Value |
|--------|---------|------------|
| **Entropy** | Uncertainty level | Start high (0.9), end low (<0.3) |
| **Confidence** | Reasoning certainty | Should increase each step |
| **Oracle Queries** | LLM calls made | Fewer = more efficient |
| **Sacred Milestones** | Checkpoints hit | Should hit 3, 6, 9 |
| **Convergence** | Reasoning complete | `true` if successful |

### **Reasoning Patterns**

**High Entropy Queries** (0.7-1.0):
- Multiple oracle queries
- More steps needed
- Lower initial confidence

**Medium Entropy Queries** (0.4-0.7):
- 1-2 oracle queries
- Mix of internal + external reasoning
- Moderate confidence

**Low Entropy Queries** (<0.4):
- Mostly internal transformation
- Minimal oracle queries
- High confidence quickly

---

## 🎯 Success Criteria

✅ **Core Functionality**
- [ ] Compiles without errors
- [ ] Example runs without crashes
- [ ] API endpoint responds
- [ ] Health check passes

✅ **AGI Behavior**
- [ ] Starts with high entropy
- [ ] Queries oracles when uncertain
- [ ] Reduces entropy over time
- [ ] Hits sacred checkpoints
- [ ] Converges with high confidence

✅ **Efficiency**
- [ ] Factual queries: 2-3 oracle calls
- [ ] Complex queries: 3-5 oracle calls
- [ ] Internal reasoning: <1 second/step
- [ ] Total time: <5 seconds for most queries

---

## 🐛 Debugging

### **If Oracle Queries Fail**

Check that Ollama is running:
```powershell
ollama serve
```

Verify models are available:
```powershell
ollama list
```

### **If API Doesn't Start**

Check port 7000 is free:
```powershell
netstat -ano | findstr :7000
```

Check database connection (if using):
```powershell
# PostgreSQL should be running
# Check connection string in .env
```

### **If Reasoning Doesn't Converge**

- Increase `max_steps` (default: 20)
- Check oracle responses are valid
- Review reasoning chain for stuck patterns

---

## 🚀 Next Steps

Once basic testing passes:

1. **Test with varied queries**
   - Factual (What is X?)
   - Causal (Why does X happen?)
   - Multi-path (How to achieve X?)
   - Ethical (Should we X?)

2. **Measure efficiency**
   - Track oracle queries vs internal steps
   - Measure time per reasoning step
   - Analyze convergence rates

3. **Build meta-learning**
   - Extract successful reasoning patterns
   - Store in Confidence Lake
   - Reuse patterns for similar queries

4. **Integrate with frontend**
   - Visualize reasoning chain
   - Show ELP state evolution
   - Display sacred checkpoints

---

## 📝 Notes

- **This is alpha** - Expect rough edges
- **LLM quality matters** - Better models = better oracles
- **Entropy thresholds tunable** - Currently 0.7 for oracle queries
- **Sacred positions hardcoded** - Future: Learn optimal checkpoints

---

**Let's test AGI!** 🧠🚀
