# 🌀 Build Epic Flux 3D - Consolidated WASM Visualization
# Combines ALL Bevy features into one amazing web experience

Write-Host "🌀 Building Epic Flux 3D Visualization" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Check for wasm-pack
Write-Host "Checking wasm-pack..." -NoNewline
if (Get-Command wasm-pack -ErrorAction SilentlyContinue) {
    Write-Host "   [OK] wasm-pack installed`n" -ForegroundColor Green
} else {
    Write-Host "   [ERROR] wasm-pack not found`n" -ForegroundColor Red
    Write-Host "Install with: cargo install wasm-pack" -ForegroundColor Yellow
    exit 1
}

# Check WASM target
Write-Host "Checking WASM target..." -NoNewline
$targets = rustup target list --installed
if ($targets -match "wasm32-unknown-unknown") {
    Write-Host "   [OK] WASM target installed`n" -ForegroundColor Green
} else {
    Write-Host "   [WARNING] Installing WASM target...`n" -ForegroundColor Yellow
    rustup target add wasm32-unknown-unknown
}

# Build WASM module
Write-Host "Building WASM module..." -ForegroundColor Yellow
Write-Host "   Source: wasm/epic_flux_3d.rs`n"

# Create temporary Cargo.toml for wasm-pack build
$tempCargoToml = @"
[package]
name = "epic-flux-3d"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
path = "wasm/epic_flux_3d.rs"

[dependencies]
spatial-vortex = { path = ".", features = ["bevy_support"] }
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"

[dependencies.bevy]
git = "https://github.com/bevyengine/bevy"
default-features = false
features = [
    "bevy_core_pipeline",
    "bevy_render",
    "bevy_asset",
    "bevy_winit",
    "bevy_pbr",
    "webgl2"
]
"@

# Save temp Cargo.toml
$tempCargoToml | Out-File -FilePath "Cargo.toml.epic" -Encoding UTF8

try {
    # Build with wasm-pack
    wasm-pack build `
        --target web `
        --out-dir web/src/lib/wasm `
        --out-name epic_flux_3d `
        --no-typescript `
        --release `
        -- --config "Cargo.toml.epic"
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "`n   [OK] WASM built successfully`n" -ForegroundColor Green
        
        # List generated files
        Write-Host "Generated files:" -ForegroundColor Cyan
        Get-ChildItem "web/src/lib/wasm/epic_flux_3d*" | ForEach-Object {
            $size = [math]::Round($_.Length / 1MB, 2)
            Write-Host "   OK $($_.Name) ($size MB)" -ForegroundColor Green
        }
    } else {
        Write-Host "`n   [ERROR] Build failed`n" -ForegroundColor Red
        exit 1
    }
} finally {
    # Cleanup temp file
    if (Test-Path "Cargo.toml.epic") {
        Remove-Item "Cargo.toml.epic" -Force
    }
}

# Check web server
Write-Host "`nChecking web server..." -NoNewline
$webProcess = Get-Process -Name "node","bun" -ErrorAction SilentlyContinue | Where-Object {
    $_.MainWindowTitle -match "28082"
}

if ($webProcess) {
    Write-Host "   [OK] Web server running`n" -ForegroundColor Green
} else {
    Write-Host "   [INFO] Starting web server...`n" -ForegroundColor Yellow
    
    # Start web server in background
    Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd web; bun run dev" -WindowStyle Normal
    
    Write-Host "   Waiting for server to start..." -NoNewline
    Start-Sleep -Seconds 5
    Write-Host " Done`n" -ForegroundColor Green
}

# Update Svelte page
Write-Host "Creating Svelte page..." -NoNewline
$sveltePage = @"
<script lang="ts">
  import { onMount } from 'svelte';
  
  let isLoading = true;
  let error: string | null = null;
  
  onMount(async () => {
    try {
      const module = await import('`$lib/wasm/epic_flux_3d.js');
      await module.default();
      isLoading = false;
      console.log('✅ Epic Flux 3D loaded!');
    } catch (err) {
      error = err.toString();
      isLoading = false;
      console.error('❌ Failed to load:', err);
    }
  });
</script>

<svelte:head>
  <title>🌀 Epic Flux 3D - SpatialVortex</title>
</svelte:head>

<div class="epic-page">
  <header>
    <h1>🌀 Epic Flux Matrix 3D</h1>
    <p>Consolidated Sacred Geometry Visualization</p>
  </header>
  
  <div class="canvas-container">
    {#if isLoading}
      <div class="loading">Loading Epic Visualization...</div>
    {:else if error}
      <div class="error">Error: {error}</div>
    {:else}
      <canvas id="bevy-canvas"></canvas>
    {/if}
  </div>
  
  <footer>
    <div class="legend">
      <h3>Features</h3>
      <ul>
        <li>✨ Sacred Triangle (3-6-9) in Cyan</li>
        <li>🔄 Flow Pattern (1→2→4→8→7→5→1)</li>
        <li>🎨 ELP Color Channels (Red/Green/Blue)</li>
        <li>📦 Shape Architecture (Box/Cylinder/Sphere)</li>
        <li>💫 Sacred Intersection Effects</li>
        <li>🎥 Auto-rotating Camera</li>
      </ul>
    </div>
  </footer>
</div>

<style>
  .epic-page {
    width: 100vw;
    height: 100vh;
    background: linear-gradient(135deg, #0a0a1a 0%, #1a1a2e 100%);
    color: white;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  
  header {
    padding: 1.5rem;
    text-align: center;
    background: rgba(0, 0, 0, 0.3);
    border-bottom: 2px solid rgba(0, 191, 255, 0.3);
  }
  
  h1 {
    margin: 0;
    font-size: 2.5em;
    background: linear-gradient(90deg, #00bfff, #ff00ff);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  .canvas-container {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  #bevy-canvas {
    width: 100%;
    height: 100%;
  }
  
  .loading, .error {
    padding: 2rem;
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.5);
  }
  
  .error {
    color: #ff4444;
    border: 2px solid #ff4444;
  }
  
  footer {
    padding: 1rem;
    background: rgba(0, 0, 0, 0.3);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .legend {
    max-width: 800px;
    margin: 0 auto;
  }
  
  .legend h3 {
    color: #00bfff;
    margin: 0 0 0.5rem 0;
  }
  
  .legend ul {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 0.5rem;
    list-style: none;
    padding: 0;
    margin: 0;
  }
  
  .legend li {
    font-size: 0.9em;
  }
</style>
"@

$sveltePage | Out-File -FilePath "web/src/routes/epic-flux-3d/+page.svelte" -Encoding UTF8 -Force
New-Item -Path "web/src/routes/epic-flux-3d" -ItemType Directory -Force | Out-Null

Write-Host "   [OK]`n" -ForegroundColor Green

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "✅ Build Complete!" -ForegroundColor Green
Write-Host "========================================`n" -ForegroundColor Cyan

Write-Host "Access the visualization at:" -ForegroundColor Yellow
Write-Host "   >> http://localhost:28082/epic-flux-3d`n" -ForegroundColor Cyan

Write-Host "Features included:" -ForegroundColor Yellow
Write-Host "   * Sacred Geometry (3-6-9 triangle)" -ForegroundColor White
Write-Host "   * Flow Pattern visualization" -ForegroundColor White
Write-Host "   * ELP color channels" -ForegroundColor White
Write-Host "   * Box/Cylinder/Sphere shapes" -ForegroundColor White
Write-Host "   * Sacred intersection effects" -ForegroundColor White
Write-Host "   * Auto-rotating orbit camera" -ForegroundColor White
Write-Host "   * Word beams with trails" -ForegroundColor White
Write-Host "   * ML enhancement visualization`n" -ForegroundColor White

Write-Host "Press any key to open in browser..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
Start-Process "http://localhost:28082/epic-flux-3d"
