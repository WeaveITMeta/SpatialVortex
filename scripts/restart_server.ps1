# Complete Server Restart Script
# Kills old processes, rebuilds, and starts fresh

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " SpatialVortex Server Restart Utility" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Kill any existing api_server processes
Write-Host "[1/4] Checking for running api_server processes..." -ForegroundColor Yellow
$processes = Get-Process | Where-Object {$_.ProcessName -like "*api_server*" -or $_.ProcessName -like "*spatial-vortex*"}
if ($processes) {
    Write-Host "  Found $($processes.Count) process(es), stopping..." -ForegroundColor Yellow
    $processes | ForEach-Object {
        try {
            Stop-Process -Id $_.Id -Force
            Write-Host "  [OK] Stopped process $($_.Id)" -ForegroundColor Green
        } catch {
            Write-Host "  [WARN] Could not stop process $($_.Id): $_" -ForegroundColor Yellow
        }
    }
    Start-Sleep -Seconds 2
} else {
    Write-Host "  [OK] No running processes found" -ForegroundColor Green
}
Write-Host ""

# Step 2: Check port availability
Write-Host "[2/4] Checking port 7000..." -ForegroundColor Yellow
$port = netstat -ano | Select-String ":7000 " | Select-String "LISTENING"
if ($port) {
    Write-Host "  [WARN] Port 7000 still in use" -ForegroundColor Yellow
    $portInfo = $port -split '\s+' | Where-Object {$_}
    $processId = $portInfo[-1]
    Write-Host "  Killing process $processId..." -ForegroundColor Yellow
    try {
        Stop-Process -Id $processId -Force
        Start-Sleep -Seconds 2
        Write-Host "  [OK] Port freed" -ForegroundColor Green
    } catch {
        Write-Host "  [ERROR] Could not free port: $_" -ForegroundColor Red
        Write-Host "  Please manually stop the process using port 7000" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "  [OK] Port 7000 is available" -ForegroundColor Green
}
Write-Host ""

# Step 3: Rebuild
Write-Host "[3/4] Rebuilding api_server..." -ForegroundColor Yellow
$buildStart = Get-Date
cargo build --release --bin api_server 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [ERROR] Build failed!" -ForegroundColor Red
    Write-Host "  Run 'cargo build --release --bin api_server' to see errors" -ForegroundColor Red
    exit 1
}
$buildTime = (Get-Date) - $buildStart
Write-Host "  [OK] Build completed in $([math]::Round($buildTime.TotalSeconds, 1))s" -ForegroundColor Green
Write-Host ""

# Step 4: Start server
Write-Host "[4/4] Starting server..." -ForegroundColor Yellow
Write-Host ""
Write-Host "Server will start in a new window..." -ForegroundColor Cyan
Write-Host "Look for these lines to confirm benchmark endpoints are loaded:" -ForegroundColor Cyan
Write-Host "  - 'Meta Orchestrator ready (90-95% accuracy, adaptive routing)'" -ForegroundColor Gray
Write-Host "  - 'POST /api/v1/benchmark' in the endpoints list" -ForegroundColor Gray
Write-Host ""
Write-Host "Once server is running, test with:" -ForegroundColor Cyan
Write-Host "  .\scripts\testing\test_benchmark_api.ps1" -ForegroundColor White
Write-Host ""
Write-Host "Press any key to start the server..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
Write-Host ""

# Start in same window so user can see output
$env:RUST_LOG = "info"
$env:RUST_BACKTRACE = "1"

Write-Host "======================================" -ForegroundColor Green
Write-Host " STARTING API SERVER" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Green
Write-Host ""

cargo run --release --bin api_server
