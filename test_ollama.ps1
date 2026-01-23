#!/usr/bin/env pwsh
# Ollama Setup Verification and Auto-Fix Script
# Tests Ollama installation, starts server if needed, and pulls required models

Write-Host "[*] SpatialVortex Ollama Setup Verification" -ForegroundColor Cyan
Write-Host ("=" * 70) -ForegroundColor Cyan

# 1. Check if Ollama is installed
Write-Host "`n[1/5] Checking Ollama installation..." -ForegroundColor Yellow
$ollamaPath = Get-Command ollama -ErrorAction SilentlyContinue
if ($null -eq $ollamaPath) {
    Write-Host "   [X] Ollama not found in PATH" -ForegroundColor Red
    Write-Host "   [!] Install from: https://ollama.ai/download" -ForegroundColor Yellow
    Write-Host "   Or use: winget install Ollama.Ollama" -ForegroundColor Yellow
    exit 1
}
Write-Host "   [OK] Ollama installed at: $($ollamaPath.Source)" -ForegroundColor Green

# 2. Check if Ollama is running
Write-Host "`n[2/5] Checking if Ollama server is running..." -ForegroundColor Yellow
$ollamaRunning = $false
try {
    $response = Invoke-WebRequest -Uri "http://localhost:11434/api/tags" -Method GET -TimeoutSec 2 -ErrorAction Stop
    Write-Host "   [OK] Ollama server is running on port 11434" -ForegroundColor Green
    $ollamaRunning = $true
} catch {
    Write-Host "   [!] Ollama server not responding" -ForegroundColor Yellow
    
    # Check if process exists but not responding
    $ollamaProcess = Get-Process ollama -ErrorAction SilentlyContinue
    if ($ollamaProcess) {
        Write-Host "   [i] Ollama process found but not responding, restarting..." -ForegroundColor Cyan
        Stop-Process -Name ollama -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
    }
    
    Write-Host "   [*] Starting Ollama server..." -ForegroundColor Cyan
    
    # Start Ollama in background
    $startInfo = New-Object System.Diagnostics.ProcessStartInfo
    $startInfo.FileName = "ollama"
    $startInfo.Arguments = "serve"
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $startInfo
    $process.Start() | Out-Null
    
    Write-Host "   [*] Waiting for server to start (10 seconds)..." -ForegroundColor Cyan
    Start-Sleep -Seconds 10
    
    # Verify it started
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:11434/api/tags" -Method GET -TimeoutSec 2 -ErrorAction Stop
        Write-Host "   [OK] Ollama server started successfully" -ForegroundColor Green
        $ollamaRunning = $true
    } catch {
        Write-Host "   [X] Failed to start Ollama server" -ForegroundColor Red
        Write-Host "   [!] Try manually: ollama serve" -ForegroundColor Yellow
        exit 1
    }
}

# 3. List installed models
Write-Host "`n[3/5] Checking installed models..." -ForegroundColor Yellow
try {
    $modelList = ollama list 2>&1 | Out-String
    Write-Host $modelList
    
    $hasMistral = $modelList -match "mistral"
    
    if ($hasMistral) {
        Write-Host "   [OK] mistral:latest is already installed" -ForegroundColor Green
    } else {
        Write-Host "   [!] mistral:latest not found" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   [!] Could not list models: $_" -ForegroundColor Yellow
    $hasMistral = $false
}

# 4. Pull model if needed
if (-not $hasMistral) {
    Write-Host "`n[4/5] Pulling mistral:latest model..." -ForegroundColor Yellow
    Write-Host "   [i] This may take several minutes (4GB download)..." -ForegroundColor Cyan
    
    try {
        ollama pull mistral:latest
        Write-Host "   [OK] Model downloaded successfully" -ForegroundColor Green
    } catch {
        Write-Host "   [X] Failed to pull model: $_" -ForegroundColor Red
        Write-Host "   [!] Try manually: ollama pull mistral:latest" -ForegroundColor Yellow
        exit 1
    }
} else {
    Write-Host "`n[4/5] Skipping model download (already installed)" -ForegroundColor Green
}

# 5. Test model generation
Write-Host "`n[5/5] Testing model generation..." -ForegroundColor Yellow
try {
    $testPayload = @{
        model = "mistral:latest"
        prompt = "Say only 'Hello from Ollama!'"
        stream = $false
        options = @{
            num_predict = 10
        }
    } | ConvertTo-Json -Depth 3
    
    Write-Host "   [*] Sending test request..." -ForegroundColor Cyan
    $result = Invoke-RestMethod -Uri "http://localhost:11434/api/generate" `
        -Method POST `
        -ContentType "application/json" `
        -Body $testPayload `
        -TimeoutSec 30 `
        -ErrorAction Stop
    
    if ($result.response) {
        Write-Host "   [OK] Model generation works!" -ForegroundColor Green
        Write-Host "   [Response] $($result.response.Trim())" -ForegroundColor Cyan
    } else {
        Write-Host "   [!] Received empty response" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   [X] Model generation failed: $_" -ForegroundColor Red
    Write-Host "   [!] Check Ollama logs for details" -ForegroundColor Yellow
    exit 1
}

# Summary
Write-Host "`n" + ("=" * 70) -ForegroundColor Green
Write-Host "[SUCCESS] Ollama Setup Complete!" -ForegroundColor Green
Write-Host ("=" * 70) -ForegroundColor Green

Write-Host "`n[Summary]" -ForegroundColor Cyan
Write-Host "   * Ollama Server: Running on http://localhost:11434" -ForegroundColor White
Write-Host "   * Model: mistral:latest (installed)" -ForegroundColor White
Write-Host "   * API: Responding correctly" -ForegroundColor White

Write-Host "`n[Next Steps]" -ForegroundColor Cyan
Write-Host "   Run the demo:" -ForegroundColor White
Write-Host "   cargo run --example asi_ollama_demo --features agents" -ForegroundColor Yellow

Write-Host "`n[Useful Commands]" -ForegroundColor Cyan
Write-Host "   ollama list                    # List installed models" -ForegroundColor White
Write-Host "   ollama pull <model>           # Download a model" -ForegroundColor White
Write-Host "   ollama run mistral Hello      # Quick test" -ForegroundColor White
Write-Host "   curl http://localhost:11434/api/tags # Check server" -ForegroundColor White

Write-Host ""
