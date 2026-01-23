# Verify benchmark datasets are properly downloaded - PowerShell version
# Run with: .\benchmarks\scripts\verify_datasets.ps1

$ErrorActionPreference = "Continue"

$BENCHMARK_DIR = "benchmarks\data"

Write-Host "Verifying benchmark datasets...`n" -ForegroundColor Cyan

function Test-DatasetFile {
    param(
        [string]$FilePath,
        [long]$MinSize
    )
    
    if (Test-Path $FilePath) {
        $size = (Get-Item $FilePath).Length
        if ($size -gt $MinSize) {
            $sizeStr = if ($size -gt 1MB) { "{0:N2} MB" -f ($size / 1MB) } else { "{0:N2} KB" -f ($size / 1KB) }
            Write-Host "OK $FilePath ($sizeStr)" -ForegroundColor Green
            return $true
        } else {
            $sizeStr = if ($size -gt 1MB) { "{0:N2} MB" -f ($size / 1MB) } else { "{0:N2} KB" -f ($size / 1KB) }
            Write-Host "FAIL $FilePath (too small: $sizeStr)" -ForegroundColor Red
            return $false
        }
    } else {
        Write-Host "FAIL $FilePath (missing)" -ForegroundColor Red
        return $false
    }
}

# FB15k-237
Write-Host "Checking FB15k-237..." -ForegroundColor Yellow
Test-DatasetFile "$BENCHMARK_DIR\fb15k237\train.txt" 1000000 | Out-Null
Test-DatasetFile "$BENCHMARK_DIR\fb15k237\valid.txt" 100000 | Out-Null
Test-DatasetFile "$BENCHMARK_DIR\fb15k237\test.txt" 100000 | Out-Null
Write-Host ""

# WN18RR
Write-Host "Checking WN18RR..." -ForegroundColor Yellow
Test-DatasetFile "$BENCHMARK_DIR\wn18rr\train.txt" 500000 | Out-Null
Test-DatasetFile "$BENCHMARK_DIR\wn18rr\test.txt" 100000 | Out-Null
Write-Host ""

# STS
Write-Host "Checking STS Benchmark..." -ForegroundColor Yellow
Test-DatasetFile "$BENCHMARK_DIR\sts\stsbenchmark\sts-test.csv" 100000 | Out-Null
Write-Host ""

# SICK
Write-Host "Checking SICK..." -ForegroundColor Yellow
Test-DatasetFile "$BENCHMARK_DIR\sick\SICK_test.txt" 50000 | Out-Null
Write-Host ""

# SQuAD
Write-Host "Checking SQuAD 2.0..." -ForegroundColor Yellow
Test-DatasetFile "$BENCHMARK_DIR\squad\dev-v2.0.json" 1000000 | Out-Null
Write-Host ""

# CommonsenseQA
Write-Host "Checking CommonsenseQA..." -ForegroundColor Yellow
Test-DatasetFile "$BENCHMARK_DIR\commonsenseqa\dev.jsonl" 100000 | Out-Null
Write-Host ""

# bAbI
Write-Host "Checking bAbI..." -ForegroundColor Yellow
Test-DatasetFile "$BENCHMARK_DIR\babi\tasks_1-20_v1-2\en-10k\qa1_single-supporting-fact_test.txt" 1000 | Out-Null
Write-Host ""

# CLUTRR
Write-Host "Checking CLUTRR..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\clutrr\test.csv") {
    Test-DatasetFile "$BENCHMARK_DIR\clutrr\test.csv" 10000 | Out-Null
} else {
    Write-Host "SKIP CLUTRR (optional dataset)" -ForegroundColor Yellow
}
Write-Host ""

# Silesia
Write-Host "Checking Silesia Corpus..." -ForegroundColor Yellow
Test-DatasetFile "$BENCHMARK_DIR\silesia\dickens" 1000000 | Out-Null
Write-Host ""

Write-Host "==================================" -ForegroundColor Cyan
Write-Host "Verification complete!" -ForegroundColor Green
Write-Host "==================================" -ForegroundColor Cyan
Write-Host "`nReady to run benchmarks:" -ForegroundColor Yellow
Write-Host "  cargo test --release --package benchmarks"
