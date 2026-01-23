# PowerShell script to build Flux Matrix 3D for web
# Run: .\BUILD_WEB_POWERSHELL.ps1

Write-Host "`n🌐 Building Flux Matrix 3D for Web (WASM + Svelte)" -ForegroundColor Cyan
Write-Host "================================================`n" -ForegroundColor Cyan

# Step 1: Install wasm-pack (if not already installed)
Write-Host "📦 Step 1: Checking wasm-pack..." -ForegroundColor Yellow
if (!(Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "   Installing wasm-pack..." -ForegroundColor White
    cargo install wasm-pack
} else {
    Write-Host "   ✅ wasm-pack already installed" -ForegroundColor Green
}

# Step 2: Build WASM module
Write-Host "`n🔨 Step 2: Building WASM module..." -ForegroundColor Yellow
wasm-pack build --target web --out-dir web/wasm --features bevy_support

if ($LASTEXITCODE -ne 0) {
    Write-Host "   ❌ WASM build failed!" -ForegroundColor Red
    exit 1
}
Write-Host "   ✅ WASM built successfully" -ForegroundColor Green

# Step 3: Setup Svelte (if directory doesn't exist)
Write-Host "`n📝 Step 3: Setting up Svelte..." -ForegroundColor Yellow
if (!(Test-Path "web/svelte-app")) {
    Write-Host "   Creating Svelte app..." -ForegroundColor White
    Set-Location web
    npm create vite@latest svelte-app -- --template svelte
    Set-Location ..
    Write-Host "   ✅ Svelte app created" -ForegroundColor Green
} else {
    Write-Host "   ✅ Svelte app already exists" -ForegroundColor Green
}

# Step 4: Install dependencies
Write-Host "`n📦 Step 4: Installing npm dependencies..." -ForegroundColor Yellow
Set-Location web/svelte-app
npm install
npm install -D vite-plugin-wasm vite-plugin-top-level-await

if ($LASTEXITCODE -ne 0) {
    Write-Host "   ❌ npm install failed!" -ForegroundColor Red
    Set-Location ../..
    exit 1
}
Write-Host "   ✅ Dependencies installed" -ForegroundColor Green

# Step 5: Copy components
Write-Host "`n📋 Step 5: Checking components..." -ForegroundColor Yellow
if (Test-Path "../../web/svelte-app/src/FluxVisualization.svelte") {
    Write-Host "   ✅ FluxVisualization.svelte already in place" -ForegroundColor Green
} else {
    Write-Host "   ⚠️  FluxVisualization.svelte not found - copy manually" -ForegroundColor Yellow
}

# Return to root
Set-Location ../..

Write-Host "`n✅ Build complete!" -ForegroundColor Green
Write-Host "`nTo run the dev server:" -ForegroundColor Cyan
Write-Host "   cd web\svelte-app" -ForegroundColor White
Write-Host "   npm run dev" -ForegroundColor White
Write-Host "`nThen visit: http://localhost:5173`n" -ForegroundColor Cyan
