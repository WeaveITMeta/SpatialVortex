# Quick Fixes for SpatialVortex Warnings (PowerShell)
# Run this script to automatically fix code warnings

Write-Host "🔧 Applying automatic fixes..." -ForegroundColor Cyan

# Fix 1: Automatically fix unused imports and variables
Write-Host "Fixing unused code..." -ForegroundColor Yellow
cargo fix --lib --allow-dirty --allow-staged
cargo fix --bin geometric_reasoning_benchmark --allow-dirty --allow-staged

# Fix 2: Kill locked processes
Write-Host "Checking for locked processes..." -ForegroundColor Yellow
Get-Process -Name "spatial-vortex" -ErrorAction SilentlyContinue | Stop-Process -Force

# Fix 3: Run tests after fixes
Write-Host "Running tests..." -ForegroundColor Yellow
cargo test --lib

# Fix 4: Re-check for warnings
Write-Host "Checking for remaining warnings..." -ForegroundColor Yellow
cargo check --benches

Write-Host "✅ Fixes applied! Check output above for results." -ForegroundColor Green
