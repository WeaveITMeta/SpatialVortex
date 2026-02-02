#!/usr/bin/env pwsh
# Download all benchmark datasets for SpatialVortex
# Run with: .\download.ps1

$ErrorActionPreference = "Stop"

Write-Host "╔══════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   SpatialVortex Benchmark Dataset Downloader                   ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Check if benchmarks directory exists
if (-not (Test-Path "benchmarks")) {
    Write-Host "❌ Error: benchmarks directory not found" -ForegroundColor Red
    Write-Host "   Please run this script from the project root" -ForegroundColor Gray
    exit 1
}

# Run the download script from benchmarks/scripts
Push-Location benchmarks
& .\scripts\download_datasets.ps1
Pop-Location

Write-Host ""
Write-Host "✅ Download complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Run benchmarks: cargo run --bin spatialvortex-eval --release --features embeddings,gpu,web-learning -- --tasks all --limit 100" -ForegroundColor Gray
Write-Host "  2. View results in eval_results.json" -ForegroundColor Gray
Write-Host ""
