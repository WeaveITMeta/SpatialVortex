# SpatialVortex Test Runner
# Runs all test categories and reports results

Write-Host "🧪 SpatialVortex Test Suite Runner" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Continue"
$totalPassed = 0
$totalFailed = 0

# Function to run test category
function Run-TestCategory {
    param(
        [string]$Category,
        [string]$Pattern
    )
    
    Write-Host "📋 Running $Category Tests..." -ForegroundColor Yellow
    Write-Host "-----------------------------------" -ForegroundColor Gray
    
    $result = cargo test --test $Pattern 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ $Category tests PASSED" -ForegroundColor Green
        $script:totalPassed++
    } else {
        Write-Host "❌ $Category tests FAILED" -ForegroundColor Red
        $script:totalFailed++
        Write-Host $result -ForegroundColor Red
    }
    Write-Host ""
}

# Run all test categories
Run-TestCategory "Unit" "unit/*"
Run-TestCategory "Integration" "integration/*"
Run-TestCategory "API" "api/*"
Run-TestCategory "Performance" "performance/*"

# Summary
Write-Host "===================================" -ForegroundColor Cyan
Write-Host "📊 Test Summary" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan
Write-Host "✅ Passed Categories: $totalPassed" -ForegroundColor Green
Write-Host "❌ Failed Categories: $totalFailed" -ForegroundColor Red
Write-Host ""

if ($totalFailed -eq 0) {
    Write-Host "🎉 All test categories passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "⚠️  Some test categories failed. Review output above." -ForegroundColor Yellow
    exit 1
}
