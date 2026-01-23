# Build Rust Documentation
# Generates rustdoc with all features enabled

Write-Host "Building Rust documentation..." -ForegroundColor Green
Write-Host "================================" -ForegroundColor Green
Write-Host ""

# Build documentation with all features
Write-Host "Running cargo doc..." -ForegroundColor Cyan
cargo doc --no-deps --all-features --document-private-items

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Documentation build failed!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Documentation built successfully!" -ForegroundColor Green
Write-Host ""

# Copy to docs directory
$sourceDir = "target/doc"
$destDir = "docs/rustdoc"

if (Test-Path $sourceDir) {
    Write-Host "Copying to docs/rustdoc..." -ForegroundColor Cyan
    
    # Create destination directory if it doesn't exist
    if (-not (Test-Path $destDir)) {
        New-Item -ItemType Directory -Path $destDir -Force | Out-Null
    }
    
    # Copy all documentation files
    Copy-Item -Path "$sourceDir/*" -Destination $destDir -Recurse -Force
    
    Write-Host "Documentation copied to: $destDir" -ForegroundColor Green
}

Write-Host ""
Write-Host "View documentation at:" -ForegroundColor Yellow
$indexPath = Resolve-Path "$sourceDir/spatial_vortex/index.html"
Write-Host "  file:///$($indexPath.Path.Replace('\', '/'))" -ForegroundColor White
Write-Host ""

# Open in browser
Write-Host "Opening in browser..." -ForegroundColor Cyan
Start-Process $indexPath

Write-Host ""
Write-Host "Done! Documentation is ready." -ForegroundColor Green
