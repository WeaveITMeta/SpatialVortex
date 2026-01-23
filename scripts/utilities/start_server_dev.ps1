# Development Server Startup Script
# Handles missing dependencies gracefully

Write-Host "🚀 Starting SpatialVortex API Server (Development Mode)" -ForegroundColor Cyan
Write-Host ""

# Set environment variables
$env:API_PORT = "7000"
$env:API_HOST = "127.0.0.1"
$env:API_CORS = "true"
$env:RUST_LOG = "info"

# Optional: Set database to in-memory for development
$env:DATABASE_URL = "sqlite::memory:"

# Optional: Disable features that might cause issues
# $env:SKIP_DB_INIT = "true"

Write-Host "📋 Configuration:" -ForegroundColor Green
Write-Host "   Port: 7000"
Write-Host "   Host: 127.0.0.1"
Write-Host "   CORS: Enabled"
Write-Host "   Log Level: info"
Write-Host ""

Write-Host "🔨 Building and starting server..." -ForegroundColor Yellow
Write-Host ""

# Run the server
cargo run --bin api_server

# If it fails, show helpful message
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "❌ Server failed to start!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Common issues:" -ForegroundColor Yellow
    Write-Host "  1. Database connection failed - check config.toml"
    Write-Host "  2. Port 7000 already in use - kill existing process"
    Write-Host "  3. Missing dependencies - run: cargo build"
    Write-Host ""
}
