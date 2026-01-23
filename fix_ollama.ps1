# Quick fix for Ollama port issues
Write-Host "[*] Fixing Ollama..." -ForegroundColor Cyan

# Step 1: Kill ALL Ollama processes
Write-Host "[1/4] Stopping all Ollama processes..." -ForegroundColor Yellow
Get-Process ollama* -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3
Write-Host "   [OK] Processes stopped" -ForegroundColor Green

# Step 2: Check if port is free
Write-Host "[2/4] Checking port 11434..." -ForegroundColor Yellow
$portInUse = Get-NetTCPConnection -LocalPort 11434 -ErrorAction SilentlyContinue
if ($portInUse) {
    Write-Host "   [!] Port 11434 still in use, waiting..." -ForegroundColor Yellow
    Start-Sleep -Seconds 5
}
Write-Host "   [OK] Port ready" -ForegroundColor Green

# Step 3: Start Ollama in new window
Write-Host "[3/4] Starting Ollama server..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "ollama serve" -WindowStyle Normal
Start-Sleep -Seconds 8

# Step 4: Test connection
Write-Host "[4/4] Testing connection..." -ForegroundColor Yellow
$maxAttempts = 5
$attempt = 0
$connected = $false

while ($attempt -lt $maxAttempts -and -not $connected) {
    $attempt++
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:11434/api/tags" -Method GET -TimeoutSec 2 -ErrorAction Stop
        Write-Host "   [OK] Ollama is responding!" -ForegroundColor Green
        $connected = $true
    } catch {
        Write-Host "   [*] Attempt $attempt/$maxAttempts..." -ForegroundColor Cyan
        Start-Sleep -Seconds 2
    }
}

if ($connected) {
    Write-Host "`n[SUCCESS] Ollama is running!" -ForegroundColor Green
    Write-Host "`nNow run: ollama pull mistral:latest" -ForegroundColor Yellow
} else {
    Write-Host "`n[X] Ollama failed to start" -ForegroundColor Red
    Write-Host "Check the Ollama window for errors" -ForegroundColor Yellow
}
