# Start SpatialVortex API Server
# Run with: .\start_server.ps1

Write-Host "🚀 Starting SpatialVortex API Server..." -ForegroundColor Cyan
Write-Host ""

# Check if build exists
if (-Not (Test-Path "target\release\api_server.exe")) {
    Write-Host "⚠️  Binary not found. Building first..." -ForegroundColor Yellow
    cargo build --bin api_server --release
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Build failed!" -ForegroundColor Red
        exit 1
    }
}

# Check for .env file
if (-Not (Test-Path ".env")) {
    Write-Host "⚠️  No .env file found. Creating from .env.example..." -ForegroundColor Yellow
    Copy-Item ".env.example" ".env"
}

Write-Host "✅ Starting server on http://localhost:7000" -ForegroundColor Green
Write-Host ""
Write-Host "📊 Features enabled:" -ForegroundColor Cyan
Write-Host "  ✨ Streaming Chat (SSE)"
Write-Host "  🔧 Tool Calling (Calculator, Search, Time)"
Write-Host "  🧠 ThinkingAgent (9-step reasoning)"
Write-Host "  📚 RAG Integration (fact-grounded)"
Write-Host "  🛡️  Safety Guardrails (PII detection)"
Write-Host "  🎨 Markdown Rendering"
Write-Host ""
Write-Host "Press Ctrl+C to stop the server" -ForegroundColor Yellow
Write-Host ""

# Run the server
& "target\release\api_server.exe"
