#!/usr/bin/env pwsh
# Rename Script: Windsurf Cascade → Vortex Context Preserver (VCP)
# Date: October 26, 2025
# Purpose: Automated batch rename across all documentation and code

Write-Host "🔄 Starting Windsurf Cascade → Vortex Context Preserver (VCP) Rename" -ForegroundColor Cyan
Write-Host "=" * 80

$repoRoot = "e:\Libraries\SpatialVortex"
Set-Location $repoRoot

# Track changes
$filesRenamed = 0
$filesModified = 0
$totalReplacements = 0

# === PHASE 1: Rename Files ===
Write-Host "`n📁 PHASE 1: Renaming files containing 'WINDSURF_CASCADE'..." -ForegroundColor Yellow

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
        Write-Host "  ✅ Renamed: $($file.Old) → $($file.New)" -ForegroundColor Green
        $filesRenamed++
    } else {
        Write-Host "  ⚠️  Not found: $($file.Old)" -ForegroundColor Yellow
    }
}

Write-Host "`n  Total files renamed: $filesRenamed" -ForegroundColor Cyan

# === PHASE 2: Text Replacements ===
Write-Host "`n📝 PHASE 2: Replacing text in all markdown and documentation files..." -ForegroundColor Yellow

# Define replacement patterns
$replacements = @(
    @{Pattern = "Windsurf Cascade Framework"; Replacement = "Vortex Context Preserver (VCP) Framework"},
    @{Pattern = "Windsurf Cascade System"; Replacement = "Vortex Context Preserver System"},
    @{Pattern = "Windsurf Cascade"; Replacement = "Vortex Context Preserver (VCP)"},
    @{Pattern = "WindsurfCascade"; Replacement = "VortexContextPreserver"},
    @{Pattern = "windsurf_cascade"; Replacement = "vortex_context_preserver"},
    @{Pattern = "WINDSURF_CASCADE"; Replacement = "VCP"},
    @{Pattern = "WINDSURF CASCADE"; Replacement = "VORTEX CONTEXT PRESERVER"}
)

# Get all markdown files (excluding node_modules, target, .git)
$mdFiles = Get-ChildItem -Path $repoRoot -Recurse -Include *.md -File | 
    Where-Object { 
        $_.FullName -notmatch "node_modules|target|\.git|\.svelte-kit" 
    }

Write-Host "`n  Found $($mdFiles.Count) markdown files to process..."

foreach ($file in $mdFiles) {
    $content = Get-Content -Path $file.FullName -Raw -ErrorAction SilentlyContinue
    
    if (-not $content) { continue }
    
    $originalContent = $content
    $fileModified = $false
    $fileReplacements = 0
    
    # Apply all replacements
    foreach ($rep in $replacements) {
        $matches = ([regex]::Matches($content, [regex]::Escape($rep.Pattern))).Count
        if ($matches -gt 0) {
            $content = $content -replace [regex]::Escape($rep.Pattern), $rep.Replacement
            $fileReplacements += $matches
            $fileModified = $true
        }
    }
    
    # Write back if modified
    if ($fileModified) {
        Set-Content -Path $file.FullName -Value $content -NoNewline
        $relativePath = $file.FullName.Replace($repoRoot + "\", "")
        Write-Host "  ✅ Modified: $relativePath ($fileReplacements replacements)" -ForegroundColor Green
        $filesModified++
        $totalReplacements += $fileReplacements
    }
}

Write-Host "`n  Total files modified: $filesModified" -ForegroundColor Cyan
Write-Host "  Total text replacements: $totalReplacements" -ForegroundColor Cyan

# === PHASE 3: Update Code Comments ===
Write-Host "`n💻 PHASE 3: Updating Rust code comments..." -ForegroundColor Yellow

$rsFiles = Get-ChildItem -Path "$repoRoot\src" -Recurse -Include *.rs -File

$codeFilesModified = 0
$codeReplacements = 0

foreach ($file in $rsFiles) {
    $content = Get-Content -Path $file.FullName -Raw -ErrorAction SilentlyContinue
    
    if (-not $content) { continue }
    
    $originalContent = $content
    $fileModified = $false
    $fileReps = 0
    
    # Only replace in comments (lines starting with // or in /* */ blocks)
    foreach ($rep in $replacements) {
        $pattern = "(//.*)$([regex]::Escape($rep.Pattern))"
        $replacement = "`$1$($rep.Replacement)"
        
        if ($content -match $pattern) {
            $content = $content -replace $pattern, $replacement
            $fileModified = $true
            $fileReps++
        }
        
        # Also handle /* */ style comments
        $blockPattern = "(/\*.*?)$([regex]::Escape($rep.Pattern))(.*?\*/)"
        $blockReplacement = "`$1$($rep.Replacement)`$2"
        
        if ($content -match $blockPattern) {
            $content = $content -replace $blockPattern, $blockReplacement
            $fileModified = $true
            $fileReps++
        }
    }
    
    if ($fileModified) {
        Set-Content -Path $file.FullName -Value $content -NoNewline
        $relativePath = $file.FullName.Replace($repoRoot + "\", "")
        Write-Host "  ✅ Modified: $relativePath ($fileReps replacements)" -ForegroundColor Green
        $codeFilesModified++
        $codeReplacements += $fileReps
    }
}

Write-Host "`n  Total code files modified: $codeFilesModified" -ForegroundColor Cyan
Write-Host "  Total code comment replacements: $codeReplacements" -ForegroundColor Cyan

# === SUMMARY ===
Write-Host "`n" + ("=" * 80) -ForegroundColor Cyan
Write-Host "✅ RENAME COMPLETE!" -ForegroundColor Green
Write-Host ("=" * 80) -ForegroundColor Cyan

Write-Host "`n📊 Summary:" -ForegroundColor Yellow
Write-Host "  Files renamed: $filesRenamed"
Write-Host "  Documentation files modified: $filesModified"
Write-Host "  Code files modified: $codeFilesModified"
Write-Host "  Total text replacements: $($totalReplacements + $codeReplacements)"

Write-Host "`n🔍 Next Steps:" -ForegroundColor Yellow
Write-Host "  1. Review changes with: git diff"
Write-Host "  2. Test compilation: cargo build"
Write-Host "  3. Run tests: cargo test hallucinations"
Write-Host "  4. Stage changes: git add -A"
Write-Host "  5. Commit: git commit -m 'refactor: Rename Windsurf Cascade to Vortex Context Preserver (VCP)'"

Write-Host "`n✨ Done! The framework is now called 'Vortex Context Preserver (VCP)'" -ForegroundColor Cyan
