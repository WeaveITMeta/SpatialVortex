# Start API Server with detailed logging
# Ensures proper startup and shows all endpoints

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "Starting SpatialVortex API Server" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Check if port 7000 is already in use
$port7000 = netstat -ano | Select-String ":7000"
if ($port7000) {
    Write-Host "[WARNING] Port 7000 appears to be in use:" -ForegroundColor Yellow
    Write-Host $port7000
    Write-Host ""
    Write-Host "Attempting to continue anyway..." -ForegroundColor Yellow
    Write-Host ""
}

# Set environment for verbose logging
$env:RUST_LOG = "info,spatial_vortex=debug"
$env:RUST_BACKTRACE = "1"

Write-Host "[INFO] Building release binary..." -ForegroundColor Gray
$buildOutput = cargo build --release --bin api_server 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Build failed!" -ForegroundColor Red
    Write-Host $buildOutput
    exit 1
}
Write-Host "[OK] Build completed successfully" -ForegroundColor Green
Write-Host ""

Write-Host "[INFO] Starting server..." -ForegroundColor Gray
Write-Host "[INFO] Press Ctrl+C to stop the server" -ForegroundColor Gray
Write-Host ""

# Start the server and capture output
cargo run --release --bin api_server
