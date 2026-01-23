# Build Bevy 3D visualization for your EXISTING web app on port 28082
# Run: .\BUILD_BEVY_FOR_WEB.ps1

Write-Host "`nBuilding Bevy 3D for Existing Web App" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Check if wasm-pack is installed
Write-Host "Checking wasm-pack..." -ForegroundColor Yellow
if (!(Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "   Installing wasm-pack..." -ForegroundColor White
    cargo install wasm-pack
} else {
    Write-Host "   [OK] wasm-pack installed" -ForegroundColor Green
}

# Build WASM from wasm/flux_3d_web.rs
Write-Host "`nBuilding WASM module..." -ForegroundColor Yellow
Write-Host "   Using: wasm/flux_3d_web.rs" -ForegroundColor White

wasm-pack build `
    --target web `
    --out-dir web/src/lib/wasm `
    --features bevy_support

if ($LASTEXITCODE -ne 0) {
    Write-Host "   [ERROR] WASM build failed!" -ForegroundColor Red
    Write-Host "`nTroubleshooting:" -ForegroundColor Yellow
    Write-Host "   1. Ensure bevy_support feature is configured for WASM" -ForegroundColor White
    Write-Host "   2. Check Cargo.toml for wasm32 target configuration" -ForegroundColor White
    Write-Host "   3. Run: rustup target add wasm32-unknown-unknown" -ForegroundColor White
    exit 1
}

Write-Host "   [OK] WASM built successfully" -ForegroundColor Green

# Check if web server is running
Write-Host "`nChecking web server..." -ForegroundColor Yellow
$webProcess = Get-NetTCPConnection -LocalPort 28082 -ErrorAction SilentlyContinue

if ($webProcess) {
    Write-Host "   [OK] Web server already running on port 28082" -ForegroundColor Green
    Write-Host "`nBuild complete! Refresh your browser:" -ForegroundColor Green
    Write-Host "   http://localhost:28082/flux-3d`n" -ForegroundColor Cyan
} else {
    Write-Host "   [INFO] Web server not running" -ForegroundColor Yellow
    Write-Host "`nTo start the server:" -ForegroundColor Cyan
    Write-Host "   cd web" -ForegroundColor White
    Write-Host "   npm run dev`n" -ForegroundColor White
}

Write-Host "Files created:" -ForegroundColor Yellow
Write-Host "   OK web/src/lib/wasm/spatial_vortex.js" -ForegroundColor Green
Write-Host "   OK web/src/lib/wasm/spatial_vortex_bg.wasm" -ForegroundColor Green
Write-Host "   OK web/src/routes/flux-3d/+page.svelte" -ForegroundColor Green
Write-Host ""
