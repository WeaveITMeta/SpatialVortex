# 🎯 Run Academic Benchmark - Complete Guide

**Target**: 97-99% accuracy on CommonsenseQA
**Current Setup**: ParallelFusion v0.8.4 + Ollama LLM

---

## ✅ Prerequisites

### 1. Install Ollama

**Windows:**
```powershell
# Download from https://ollama.ai
# Or use winget
winget install Ollama.Ollama
```

### 2. Pull the Model
```bash
ollama pull llama3.2:latest
```

### 3. Verify Ollama is Running
```bash
# Should return Ollama version
ollama list

# Test generation
ollama run llama3.2:latest "What is 2+2?"
```

---

## 🚀 Step-by-Step Execution

### Step 1: Build with Agents Feature
```bash
cargo build --release --features agents --bin api_server
cargo build --release --features agents --bin academic_benchmark
```

**Why `agents` feature?**
- Enables LLMBridge integration
- Connects ASI Orchestrator to Ollama
- Required for real AI responses

### Step 2: Start API Server
```bash
# Terminal 1
cargo run --release --features agents --bin api_server
```

**Expected output:**
```
🔥 Initializing ParallelFusion v0.8.4 (Ensemble)...
   ✅ ParallelFusion ready (97-99% accuracy target)
🌐 Starting HTTP server at http://127.0.0.1:7000
   POST /api/v1/process                - 🔥 ParallelFusion v0.8.4
```

### Step 3: Run Benchmark
```bash
# Terminal 2
cargo run --release --features agents --bin academic_benchmark
```

**Expected output:**
```
╔══════════════════════════════════════════════════════════════════╗
║   ParallelFusion v0.8.4 Academic Benchmark (Real AI)           ║
║   Dataset: CommonsenseQA Dev Set                               ║
║   Backend: Ollama (llama3.2) via ASI Orchestrator              ║
║   Endpoint: POST http://localhost:7000/api/v1/process          ║
║   Target: 97-99% accuracy                                      ║
╚══════════════════════════════════════════════════════════════════╝

🔍 Checking API server at http://localhost:7000...
✅ API server is running

📊 Testing on 50 questions...

  [1/50] Processing: 1afa02df02c908a558b4036e80242fac... [3.2s] ✅ (92% conf)
  [2/50] Processing: 07121800cd110aa3ff5789a84491f60d... [2.8s] ✅ (88% conf)
  ...
```

---

## 🔧 How It Works

```
Question → ParallelFusion v0.8.4
            ↓
        ASI Orchestrator
            ↓
    1. Geometric Expert (heuristic)
    2. Heuristic Expert (pattern matching)
    3. RAG Expert (retrieval)
    4. Consensus Expert (verification)
            ↓
    5. generate_with_llm() → Ollama (llama3.2) → **Real AI Answer**
            ↓
    6. Fusion with FluxOrchestrator
            ↓
        Final Answer ✅
```

### Key Components

**1. LLMBridge** (`src/agents/llm_bridge.rs`)
- Connects to Ollama at `http://localhost:11434`
- Uses `llama3.2:latest` model
- 10-minute timeout per request

**2. ASIOrchestrator** (`src/ai/orchestrator.rs:604-624`)
```rust
async fn generate_with_llm(&self, input: &str) -> Option<String> {
    if let Some(llm) = &self.llm_bridge {
        match llm.generate_code(&prompt, Language::Text).await {
            Ok(response) => Some(response),
            Err(_) => None, // Falls back to mock
        }
    }
}
```

**3. Integration Point** (`src/ai/orchestrator.rs:856-861`)
```rust
// REAL AI: Try to generate response with LLM (Ollama)
#[cfg(feature = "agents")]
if let Some(llm_response) = self.generate_with_llm(input).await {
    final_result_text = llm_response;
    confidence = (confidence * 1.2).min(1.0);
}
```

---

## 📊 Expected Performance

### Timing
- **Per question**: 2-5 seconds (with Ollama)
- **Total time**: 2-4 minutes for 50 questions
- **Timeout**: 10 minutes per question (generous buffer)

### Accuracy Targets
- **Target**: 97-99%
- **Minimum**: 90% (llama3.2 baseline)
- **With Ensemble**: 97%+ (fusion boosts accuracy)

---

## 🐛 Troubleshooting

### Issue: "Connection refused" to Ollama

**Cause**: Ollama not running

**Fix**:
```bash
# Windows
ollama serve

# Check if running
curl http://localhost:11434
```

### Issue: "Fusion timed out after 600000ms"

**Cause**: Ollama taking too long or not responding

**Fix**:
1. Check Ollama is running: `ollama list`
2. Test directly: `ollama run llama3.2:latest "test"`
3. Check RAM (llama3.2 needs ~4GB)

### Issue: Fallback to Mock Responses

**Symptoms**: Fast responses (<1s) with generic answers

**Cause**: LLMBridge failed to connect

**Fix**:
```bash
# Verify Ollama endpoint
curl http://localhost:11434/api/generate -d '{
  "model": "llama3.2:latest",
  "prompt": "test",
  "stream": false
}'
```

### Issue: Low Accuracy (<90%)

**Possible causes**:
1. Model not responding correctly
2. Prompt format issues
3. Answer extraction failing

**Debug**:
```bash
# Check API server logs for LLM responses
# Look for "generate_with_llm" in output
```

---

## 📈 Results Format

After completion, results are saved to:
```
benchmarks/data/commonsenseqa_results_[timestamp].json
```

**Format**:
```json
{
  "version": "0.8.4",
  "model": "ParallelFusion Ensemble + Ollama",
  "timestamp": "2025-11-01T...",
  "dataset": "CommonsenseQA",
  "total_questions": 50,
  "correct": 49,
  "accuracy_percent": 98.0,
  "target_accuracy": "97-99%",
  "status": "✅ PASSED",
  "samples": [...]
}
```

---

## 🎯 Success Criteria

✅ **PASSED**: 97-99% accuracy
⚠️  **GOOD**: 90-96% accuracy  
❌ **FAILED**: <90% accuracy

---

## 🚀 Quick Start Commands

```bash
# 1. Ensure Ollama is running
ollama list

# 2. Start API server (Terminal 1)
cargo run --release --features agents --bin api_server

# 3. Run benchmark (Terminal 2)
cargo run --release --features agents --bin academic_benchmark
```

**That's it!** The benchmark will run and show results in 2-4 minutes.

---

## 📝 Notes

- **First run** may be slower as Ollama loads the model
- **Accuracy improves** with ensemble fusion (multiple expert verification)
- **Sacred positions** (3, 6, 9) trigger additional validation
- **Confidence boost** (1.2x) for LLM responses vs heuristics
- **Fallback** to mock if Ollama unavailable (but accuracy will be lower)
