#!/usr/bin/env pwsh
# Quick start script for ParallelFusion v0.8.4 Academic Benchmarks

Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   ParallelFusion v0.8.4 Academic Benchmark Runner              ║" -ForegroundColor Cyan
Write-Host "╚══════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Check if dataset exists
if (-not (Test-Path "benchmarks/data/commonsenseqa/dev.jsonl")) {
    Write-Host "⚠️  Dataset not found: benchmarks/data/commonsenseqa/dev.jsonl" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Downloading datasets..." -ForegroundColor Yellow
    Push-Location benchmarks
    & .\scripts\download_datasets.ps1
    Pop-Location
    Write-Host ""
}

Write-Host "🚀 Running academic benchmarks..." -ForegroundColor Green
Write-Host "   This will test ParallelFusion on 50 CommonsenseQA questions" -ForegroundColor Gray
Write-Host "   Expected runtime: 5-10 minutes" -ForegroundColor Gray
Write-Host ""

# Run the benchmark
cargo run --release --bin academic_benchmark

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✅ Benchmark complete!" -ForegroundColor Green
    Write-Host ""
    
    # Find the most recent result file
    $resultFiles = Get-ChildItem "benchmarks/data/parallel_fusion_v0.8.4_academic_*.json" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending
    
    if ($resultFiles) {
        $latestResult = $resultFiles[0]
        Write-Host "📊 Results saved to: $($latestResult.FullName)" -ForegroundColor Cyan
        Write-Host ""
        
        # Try to parse and display summary
        try {
            $json = Get-Content $latestResult.FullName | ConvertFrom-Json
            Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray
            Write-Host " QUICK SUMMARY" -ForegroundColor White
            Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray
            Write-Host " Version: $($json.version)" -ForegroundColor Gray
            Write-Host " Model: $($json.model)" -ForegroundColor Gray
            Write-Host " Dataset: $($json.dataset)" -ForegroundColor Gray
            Write-Host " Questions: $($json.total_questions)" -ForegroundColor Gray
            Write-Host " Correct: $($json.correct)" -ForegroundColor Gray
            
            $accuracy = [math]::Round($json.accuracy_percent, 2)
            $color = if ($accuracy -ge 97) { "Green" } elseif ($accuracy -ge 90) { "Yellow" } else { "Red" }
            Write-Host " Accuracy: $accuracy%" -ForegroundColor $color
            Write-Host " Target: $($json.target_accuracy)" -ForegroundColor Gray
            Write-Host " Status: $($json.status)" -ForegroundColor $color
            Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray
        } catch {
            Write-Host "View full results in: $($latestResult.Name)" -ForegroundColor Gray
        }
    }
    
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "  1. Review the JSON file for detailed results" -ForegroundColor Gray
    Write-Host "  2. Check failed samples to identify improvement areas" -ForegroundColor Gray
    Write-Host "  3. Adjust fusion parameters if accuracy < 97%" -ForegroundColor Gray
    Write-Host ""
} else {
    Write-Host ""
    Write-Host "❌ Benchmark failed. Check the error messages above." -ForegroundColor Red
    Write-Host ""
}
