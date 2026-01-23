# SpatialVortex - Start All Servers (PowerShell)
# Run each command in a separate terminal window

Write-Host "=== SpatialVortex Server Startup Guide ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Open 3 separate PowerShell windows and run these commands:" -ForegroundColor Yellow
Write-Host ""
Write-Host "Terminal 1 - Ollama Server:" -ForegroundColor Green
Write-Host "  ollama serve" -ForegroundColor White
Write-Host ""
Write-Host "Terminal 2 - API Server:" -ForegroundColor Green
Write-Host "  cargo run --bin api_server --features agents,persistence,postgres,lake,burn-cuda-backend" -ForegroundColor White
Write-Host ""
Write-Host "Terminal 3 - Frontend:" -ForegroundColor Green
Write-Host "  cd web" -ForegroundColor White
Write-Host "  pnpm run dev" -ForegroundColor White
Write-Host ""
Write-Host "Then visit: http://localhost:28083" -ForegroundColor Cyan
Write-Host ""
Write-Host "Watch for consensus logs in API server terminal:" -ForegroundColor Yellow
Write-Host "  🌀 Vector Consensus: 4 vectors, ELP=(6.2,7.8,5.5), conf=0.82, div=0.75, sacred=0.68" -ForegroundColor Gray
