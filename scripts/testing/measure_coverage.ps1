# Measure Test Coverage with Tarpaulin
# Generates HTML coverage report

Write-Host "Measuring test coverage..." -ForegroundColor Green
Write-Host "=============================" -ForegroundColor Green
Write-Host ""

# Check if tarpaulin is installed
Write-Host "Checking for cargo-tarpaulin..." -ForegroundColor Cyan
$tarpaulinCheck = cargo tarpaulin --version 2>&1

if ($LASTEXITCODE -ne 0) {
    Write-Host "cargo-tarpaulin not found. Installing..." -ForegroundColor Yellow
    cargo install cargo-tarpaulin
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Failed to install cargo-tarpaulin" -ForegroundColor Red
        Write-Host "Note: Tarpaulin requires Linux or WSL on Windows" -ForegroundColor Yellow
        exit 1
    }
}

Write-Host "Running coverage analysis..." -ForegroundColor Cyan
Write-Host "(This may take a few minutes)" -ForegroundColor Gray
Write-Host ""

# Run tarpaulin with HTML output
cargo tarpaulin --out Html --output-dir coverage --exclude-files "src/bin/*" --exclude-files "examples/*"

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "Error: Coverage measurement failed!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Note: On Windows, you may need to use WSL:" -ForegroundColor Yellow
    Write-Host "  wsl cargo tarpaulin --out Html --output-dir coverage" -ForegroundColor White
    exit 1
}

Write-Host ""
Write-Host "Coverage report generated!" -ForegroundColor Green
Write-Host ""

# Open HTML report
$reportPath = "coverage/index.html"
if (Test-Path $reportPath) {
    Write-Host "Opening coverage report..." -ForegroundColor Cyan
    $fullPath = Resolve-Path $reportPath
    Write-Host "Report location: $fullPath" -ForegroundColor White
    Start-Process $fullPath
} else {
    Write-Host "Warning: HTML report not found at $reportPath" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Done! Coverage analysis complete." -ForegroundColor Green
