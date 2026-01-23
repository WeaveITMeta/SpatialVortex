# ⚡ Quick Start: Multi-Model Chat

## What Changed

You now get **4 AI responses** instead of 1:
1. 🤖 **llama3.2:latest** (gray bubble)
2. 🤖 **mixtral:8x7b** (gray bubble)
3. 🤖 **codellama:13b** (gray bubble)
4. 🌀 **Vortex** (ORANGE bubble, synthesizes all)

---

## 🚀 Start Commands

### Terminal 1
```bash
ollama serve
```

### Terminal 2
```bash
cargo run --release --bin api_server --features agents,persistence,postgres,lake,burn-cuda-backend
```

### Terminal 3
```bash
cd web
bun run dev
```

### Browser
```
http://localhost:28082
```

---

## ✅ What You'll See

Type: **"What is consciousness?"**

You'll get:
- 3 gray bubbles (each Ollama model)
- 1 orange bubble (Vortex synthesis)

First query: ~40-60 seconds (models loading)  
After that: ~10-30 seconds per query

---

## 🎯 Success!

If you see 4 separate responses with the last one in **orange** labeled "🌀 Vortex", it's working! 🎉
