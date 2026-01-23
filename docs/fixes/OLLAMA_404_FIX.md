# Fixing Ollama 404 Not Found Error

## 🔍 Diagnosis

You're getting: `Ollama returned error: 404 Not Found`

This means Ollama cannot find the requested model or endpoint.

---

## ✅ Solution Steps

### Step 1: Check if Ollama is Running

```powershell
# Check if Ollama process is running
Get-Process ollama -ErrorAction SilentlyContinue

# Or check the API endpoint
curl http://localhost:11434/api/tags
```

**Expected Response**: JSON list of models

**If you get an error**: Ollama is not running.

---

### Step 2: Start Ollama (if not running)

```powershell
# Start Ollama server
ollama serve
```

**Leave this terminal open** - Ollama needs to keep running.

---

### Step 3: Pull the Model

In a **NEW terminal** (keep Ollama running):

```bash
# Pull mistral model
ollama pull mistral:latest

# Verify it's installed
ollama list
```

**Expected Output**:
```
NAME              ID              SIZE      MODIFIED
mistral:latest    abc123def...    4.1 GB    2 minutes ago
```

---

### Step 4: Test Ollama Directly

```bash
# Quick test
ollama run mistral:latest "What is 2+2?"
```

**Expected**: Ollama should respond with an answer.

---

### Step 5: Verify API Endpoint

```powershell
# Test the generate endpoint
curl -X POST http://localhost:11434/api/generate -H "Content-Type: application/json" -d '{\"model\": \"mistral:latest\", \"prompt\": \"Hello\", \"stream\": false}'
```

**Expected**: JSON response with generated text.

---

## 🎯 Quick Fix (Most Common Issue)

The most common cause is **Ollama not running as a background service**.

### Windows Quick Start:

```powershell
# Option 1: Start as background service (recommended)
Start-Process ollama -ArgumentList "serve" -WindowStyle Hidden

# Option 2: Start in new window
Start-Process powershell -ArgumentList "ollama serve"

# Wait 3 seconds for startup
Start-Sleep -Seconds 3

# Pull the model
ollama pull mistral:latest

# Verify
ollama list
```

---

## 🔧 Alternative: Use a Different Model

If `mistral:latest` isn't working, try these smaller models:

```bash
# Tiny and fast (2GB)
ollama pull phi3:latest

# Or use llama
ollama pull llama3.2:latest
```

Then update your code:

```rust
let config = OllamaConfig {
    url: "http://localhost:11434".to_string(),
    model: "phi3:latest".to_string(),  // or llama3.2:latest
    temperature: 0.2,
    max_tokens: 500,
};
```

---

## 🚨 Common Issues

### Issue 1: Port Already in Use
**Error**: `bind: Only one usage of each socket address`

**Solution**: Another Ollama instance is already running (this is GOOD!). Just pull the model:
```bash
ollama pull mistral:latest
```

### Issue 2: Model Not Found
**Error**: `404 Not Found`

**Solution**: The model name doesn't exist. Check available models:
```bash
ollama list
```

### Issue 3: Connection Refused
**Error**: `Connection refused`

**Solution**: Ollama isn't running at all:
```bash
ollama serve
```

---

## ✅ Complete Verification Script

Save this as `test_ollama.ps1`:

```powershell
Write-Host "🔍 Testing Ollama Setup..." -ForegroundColor Cyan

# 1. Check if Ollama is installed
Write-Host "`n1. Checking Ollama installation..."
$ollamaPath = Get-Command ollama -ErrorAction SilentlyContinue
if ($null -eq $ollamaPath) {
    Write-Host "   ❌ Ollama not found. Install from: https://ollama.ai" -ForegroundColor Red
    exit 1
}
Write-Host "   ✅ Ollama installed at: $($ollamaPath.Source)" -ForegroundColor Green

# 2. Check if Ollama is running
Write-Host "`n2. Checking if Ollama is running..."
try {
    $response = Invoke-WebRequest -Uri "http://localhost:11434/api/tags" -Method GET -ErrorAction Stop
    Write-Host "   ✅ Ollama server is running" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Ollama server not responding" -ForegroundColor Yellow
    Write-Host "   🔄 Starting Ollama server..." -ForegroundColor Cyan
    Start-Process ollama -ArgumentList "serve" -WindowStyle Hidden
    Start-Sleep -Seconds 5
    Write-Host "   ✅ Ollama server started" -ForegroundColor Green
}

# 3. Check for models
Write-Host "`n3. Checking installed models..."
$models = ollama list
Write-Host $models

if ($models -match "mistral") {
    Write-Host "   ✅ mistral:latest is installed" -ForegroundColor Green
} else {
    Write-Host "   ⚠️  mistral:latest not found" -ForegroundColor Yellow
    Write-Host "   🔄 Pulling mistral:latest..." -ForegroundColor Cyan
    ollama pull mistral:latest
    Write-Host "   ✅ Model pulled successfully" -ForegroundColor Green
}

# 4. Test generation
Write-Host "`n4. Testing model generation..."
try {
    $testPrompt = @{
        model = "mistral:latest"
        prompt = "Say 'Hello from Ollama!'"
        stream = $false
    } | ConvertTo-Json
    
    $result = Invoke-WebRequest -Uri "http://localhost:11434/api/generate" `
        -Method POST `
        -ContentType "application/json" `
        -Body $testPrompt `
        -ErrorAction Stop
    
    Write-Host "   ✅ Model generation works!" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Model generation failed: $_" -ForegroundColor Red
}

Write-Host "`n✅ Ollama setup verification complete!" -ForegroundColor Green
Write-Host "`nYou can now run: cargo run --example asi_ollama_demo --features agents" -ForegroundColor Cyan
```

**Run it**:
```powershell
.\test_ollama.ps1
```

---

## 🎯 After Fixing

Once Ollama is running and the model is pulled, run:

```bash
cargo run --example asi_ollama_demo --features agents
```

You should see responses instead of 404 errors!

---

## 📞 Still Having Issues?

If you're still getting 404 errors after following these steps:

1. **Check Ollama logs**:
   ```bash
   # Windows: Check Event Viewer or Ollama console output
   ```

2. **Verify the exact error**:
   ```bash
   curl -v http://localhost:11434/api/tags
   ```

3. **Try a different port** (if 11434 is blocked):
   ```bash
   # Set custom port
   $env:OLLAMA_HOST="127.0.0.1:11435"
   ollama serve
   ```

   Then update your code:
   ```rust
   url: "http://localhost:11435".to_string(),
   ```

---

**Status**: Ready to fix! 🚀  
**Date**: November 9, 2025
