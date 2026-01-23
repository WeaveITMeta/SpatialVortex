# Rename Script: Windsurf Cascade to Vortex Context Preserver (VCP)
# Date: October 26, 2025

Write-Host "Starting Windsurf Cascade to VCP Rename..." -ForegroundColor Cyan

$repoRoot = "e:\Libraries\SpatialVortex"
Set-Location $repoRoot

$filesRenamed = 0
$filesModified = 0
$totalReplacements = 0

# PHASE 1: Rename Files
Write-Host "`nPHASE 1: Renaming files..." -ForegroundColor Yellow

$filesToRename = @(
    @{Old = "README_WINDSURF_CASCADE.md"; New = "README_VCP.md"},
    @{Old = "WINDSURF_CASCADE_COMPLETE.md"; New = "VCP_COMPLETE.md"},
    @{Old = "WINDSURF_CASCADE_FINAL_SUMMARY.md"; New = "VCP_FINAL_SUMMARY.md"},
    @{Old = "docs\research\WINDSURF_CASCADE_IMPLEMENTATION.md"; New = "docs\research\VCP_IMPLEMENTATION.md"},
    @{Old = "docs\architecture\WINDSURF_CASCADE_ARCHITECTURE.md"; New = "docs\architecture\VCP_ARCHITECTURE.md"},
    @{Old = "docs\milestones\WINDSURF_CASCADE_SESSION.md"; New = "docs\milestones\VCP_SESSION.md"}
)

foreach ($file in $filesToRename) {
    $oldPath = Join-Path $repoRoot $file.Old
    $newPath = Join-Path $repoRoot $file.New
    
    if (Test-Path $oldPath) {
        Move-Item -Path $oldPath -Destination $newPath -Force
        Write-Host "  Renamed: $($file.Old) to $($file.New)" -ForegroundColor Green
        $filesRenamed++
    } else {
        Write-Host "  Not found: $($file.Old)" -ForegroundColor Yellow
    }
}

Write-Host "`nTotal files renamed: $filesRenamed" -ForegroundColor Cyan

# PHASE 2: Text Replacements in Markdown Files
Write-Host "`nPHASE 2: Replacing text in markdown files..." -ForegroundColor Yellow

$replacements = @(
    @{Pattern = "Windsurf Cascade Framework"; Replacement = "Vortex Context Preserver (VCP) Framework"},
    @{Pattern = "Windsurf Cascade System"; Replacement = "Vortex Context Preserver System"},
    @{Pattern = "Windsurf Cascade"; Replacement = "Vortex Context Preserver (VCP)"},
    @{Pattern = "WINDSURF_CASCADE"; Replacement = "VCP"},
    @{Pattern = "WINDSURF CASCADE"; Replacement = "VORTEX CONTEXT PRESERVER"}
)

$mdFiles = Get-ChildItem -Path $repoRoot -Recurse -Include *.md -File | 
    Where-Object { 
        $_.FullName -notmatch "node_modules|target|\.git|\.svelte-kit" 
    }

Write-Host "Found $($mdFiles.Count) markdown files to process..."

foreach ($file in $mdFiles) {
    $content = Get-Content -Path $file.FullName -Raw -ErrorAction SilentlyContinue
    
    if (-not $content) { continue }
    
    $originalContent = $content
    $fileModified = $false
    $fileReplacements = 0
    
    foreach ($rep in $replacements) {
        $matches = ([regex]::Matches($content, [regex]::Escape($rep.Pattern))).Count
        if ($matches -gt 0) {
            $content = $content -replace [regex]::Escape($rep.Pattern), $rep.Replacement
            $fileReplacements += $matches
            $fileModified = $true
        }
    }
    
    if ($fileModified) {
        Set-Content -Path $file.FullName -Value $content -NoNewline
        $relativePath = $file.FullName.Replace($repoRoot + "\", "")
        Write-Host "  Modified: $relativePath - $fileReplacements replacements" -ForegroundColor Green
        $filesModified++
        $totalReplacements += $fileReplacements
    }
}

Write-Host "`nTotal files modified: $filesModified" -ForegroundColor Cyan
Write-Host "Total text replacements: $totalReplacements" -ForegroundColor Cyan

# SUMMARY
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "RENAME COMPLETE!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan

Write-Host "`nSummary:" -ForegroundColor Yellow
Write-Host "  Files renamed: $filesRenamed"
Write-Host "  Files modified: $filesModified"
Write-Host "  Total replacements: $totalReplacements"

Write-Host "`nNext Steps:" -ForegroundColor Yellow
Write-Host "  1. Review changes: git diff"
Write-Host "  2. Test compilation: cargo build"
Write-Host "  3. Stage changes: git add -A"
Write-Host "  4. Commit changes"

Write-Host "`nDone! Framework renamed to Vortex Context Preserver (VCP)" -ForegroundColor Cyan
