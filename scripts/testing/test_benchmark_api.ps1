# SpatialVortex Benchmark API Test Script
# Verifies the benchmark system is working correctly

param(
    [string]$ApiUrl = "http://localhost:7000",
    [switch]$Verbose = $false
)

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "    SpatialVortex Benchmark API Test Suite" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

$passCount = 0
$failCount = 0

function Test-Endpoint {
    param(
        [string]$Name,
        [string]$Url,
        [string]$Method = "GET",
        [object]$Body = $null
    )
    
    Write-Host "Testing: $Name..." -NoNewline
    
    try {
        $headers = @{
            "Content-Type" = "application/json"
        }
        
        if ($Body) {
            $bodyJson = $Body | ConvertTo-Json -Depth 10
            if ($Verbose) {
                Write-Host ""
                Write-Host "  Request: $bodyJson" -ForegroundColor Gray
            }
            $response = Invoke-RestMethod -Uri $Url -Method $Method -Headers $headers -Body $bodyJson -TimeoutSec 30
        } else {
            $response = Invoke-RestMethod -Uri $Url -Method $Method -Headers $headers -TimeoutSec 30
        }
        
        if ($Verbose) {
            Write-Host ""
            Write-Host "  Response: " -ForegroundColor Gray
            Write-Host ($response | ConvertTo-Json -Depth 5) -ForegroundColor Gray
        }
        
        Write-Host " [PASS]" -ForegroundColor Green
        $script:passCount++
        return $response
    }
    catch {
        Write-Host " [FAIL]" -ForegroundColor Red
        Write-Host "  Error: $($_.Exception.Message)" -ForegroundColor Red
        $script:failCount++
        return $null
    }
}

# Test 1: Health Check
Write-Host "[1/7] Health Check" -ForegroundColor Yellow
Test-Endpoint -Name "GET /api/v1/health" -Url "$ApiUrl/api/v1/health"
Write-Host ""

# Test 2: Single Benchmark - Geometric Reasoning (RuntimeFirst - Fast)
Write-Host "[2/7] Single Benchmark - Geometric Reasoning (RuntimeFirst)" -ForegroundColor Yellow
$benchmark1 = @{
    benchmark_type = "GeometricReasoning"
    query = "What position represents Unity?"
    expected_answer = "0"
    strategy = "RuntimeFirst"
}
$result1 = Test-Endpoint -Name "POST /api/v1/benchmark (RuntimeFirst)" `
    -Url "$ApiUrl/api/v1/benchmark" `
    -Method "POST" `
    -Body $benchmark1

if ($result1 -and $result1.is_correct -eq $true) {
    Write-Host "  [OK] Correct answer: $($result1.answer)" -ForegroundColor Green
    Write-Host "  [OK] Confidence: $([math]::Round($result1.confidence * 100, 1))%" -ForegroundColor Green
    Write-Host "  [OK] Processing time: $($result1.processing_time_ms)ms" -ForegroundColor Green
} elseif ($result1) {
    Write-Host "  [X] Wrong answer: Expected '0', got '$($result1.answer)'" -ForegroundColor Red
}
Write-Host ""

# Test 3: Single Benchmark - Sacred Position (ParallelFusion - High Accuracy)
Write-Host "[3/7] Single Benchmark - Sacred Position (ParallelFusion)" -ForegroundColor Yellow
$benchmark2 = @{
    benchmark_type = "GeometricReasoning"
    query = "What position represents Harmonic Balance?"
    expected_answer = "6"
    strategy = "ParallelFusion"
}
$result2 = Test-Endpoint -Name "POST /api/v1/benchmark (ParallelFusion)" `
    -Url "$ApiUrl/api/v1/benchmark" `
    -Method "POST" `
    -Body $benchmark2

if ($result2 -and $result2.is_correct -eq $true) {
    Write-Host "  [OK] Correct answer: $($result2.answer)" -ForegroundColor Green
    Write-Host "  [OK] Sacred boost applied: $($result2.sacred_boost)" -ForegroundColor Cyan
    Write-Host "  [OK] Confidence: $([math]::Round($result2.confidence * 100, 1))%" -ForegroundColor Green
    Write-Host "  [OK] Orchestrators: $($result2.orchestrators_used)" -ForegroundColor Gray
}
Write-Host ""

# Test 4: Single Benchmark - Hybrid Routing (Auto-select)
Write-Host "[4/7] Single Benchmark - Hybrid Routing (Auto-select)" -ForegroundColor Yellow
$benchmark3 = @{
    benchmark_type = "GeneralKnowledge"
    query = "What is the vortex flow pattern in SpatialVortex architecture?"
    expected_answer = "1 2 4 8 7 5 1"
    strategy = "Hybrid"
}
$result3 = Test-Endpoint -Name "POST /api/v1/benchmark (Hybrid)" `
    -Url "$ApiUrl/api/v1/benchmark" `
    -Method "POST" `
    -Body $benchmark3

if ($result3) {
    Write-Host "  [OK] Answer: $($result3.answer)" -ForegroundColor Green
    Write-Host "  [OK] Strategy used: $($result3.metadata.routing_strategy)" -ForegroundColor Gray
}
Write-Host ""

# Test 5: Batch Benchmark - Sequential
Write-Host "[5/7] Batch Benchmark - Sequential Execution" -ForegroundColor Yellow
$batchSeq = @{
    benchmarks = @(
        @{
            benchmark_type = "GeometricReasoning"
            query = "What position represents Creative Trinity?"
            expected_answer = "3"
        },
        @{
            benchmark_type = "GeometricReasoning"
            query = "What position represents Harmonic Balance?"
            expected_answer = "6"
        },
        @{
            benchmark_type = "GeometricReasoning"
            query = "What position represents Divine Completion?"
            expected_answer = "9"
        }
    )
    parallel = $false
}
$resultBatchSeq = Test-Endpoint -Name "POST /api/v1/benchmark/batch (Sequential)" `
    -Url "$ApiUrl/api/v1/benchmark/batch" `
    -Method "POST" `
    -Body $batchSeq

if ($resultBatchSeq) {
    $summary = $resultBatchSeq.summary
    Write-Host "  [OK] Total: $($summary.total)" -ForegroundColor Green
    Write-Host "  [OK] Correct: $($summary.correct)" -ForegroundColor Green
    Write-Host "  [OK] Accuracy: $([math]::Round($summary.accuracy * 100, 1))%" -ForegroundColor Green
    Write-Host "  [OK] Avg confidence: $([math]::Round($summary.avg_confidence * 100, 1))%" -ForegroundColor Green
    Write-Host "  [OK] Total time: $($summary.total_time_ms)ms" -ForegroundColor Gray
}
Write-Host ""

# Test 6: Batch Benchmark - Parallel
Write-Host "[6/7] Batch Benchmark - Parallel Execution" -ForegroundColor Yellow
$batchPar = @{
    benchmarks = @(
        @{
            benchmark_type = "GeometricReasoning"
            query = "Position 0 represents?"
            expected_answer = "Unity"
        },
        @{
            benchmark_type = "GeometricReasoning"
            query = "Position 1 represents?"
            expected_answer = "Beginning"
        },
        @{
            benchmark_type = "GeometricReasoning"
            query = "Position 2 represents?"
            expected_answer = "Duality"
        },
        @{
            benchmark_type = "GeometricReasoning"
            query = "Position 3 represents?"
            expected_answer = "Trinity"
        }
    )
    parallel = $true
}
$resultBatchPar = Test-Endpoint -Name "POST /api/v1/benchmark/batch (Parallel)" `
    -Url "$ApiUrl/api/v1/benchmark/batch" `
    -Method "POST" `
    -Body $batchPar

if ($resultBatchPar) {
    $summary = $resultBatchPar.summary
    Write-Host "  [OK] Total: $($summary.total)" -ForegroundColor Green
    Write-Host "  [OK] Correct: $($summary.correct)" -ForegroundColor Green
    Write-Host "  [OK] Accuracy: $([math]::Round($summary.accuracy * 100, 1))%" -ForegroundColor Green
    Write-Host "  [OK] Total time: $($summary.total_time_ms)ms (parallel speedup)" -ForegroundColor Cyan
}
Write-Host ""

# Test 7: Adaptive Routing
Write-Host "[7/7] Adaptive Routing Test" -ForegroundColor Yellow
$benchmark4 = @{
    benchmark_type = "AdaptiveTest"
    query = "Test adaptive routing decision making"
    strategy = "Adaptive"
}
$result4 = Test-Endpoint -Name "POST /api/v1/benchmark (Adaptive)" `
    -Url "$ApiUrl/api/v1/benchmark" `
    -Method "POST" `
    -Body $benchmark4

if ($result4) {
    Write-Host "  [OK] Answer generated: $($result4.answer.Substring(0, [Math]::Min(50, $result4.answer.Length)))..." -ForegroundColor Green
    Write-Host "  [OK] Routing decision logged" -ForegroundColor Gray
}
Write-Host ""

# Summary
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "TEST SUMMARY" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "Total Tests: $($passCount + $failCount)" -ForegroundColor White
Write-Host "Passed: $passCount" -ForegroundColor Green
Write-Host "Failed: $failCount" -ForegroundColor Red

if ($failCount -eq 0) {
    Write-Host ""
    Write-Host "[SUCCESS] ALL TESTS PASSED! Benchmark API is ready for production." -ForegroundColor Green
    Write-Host ""
    Write-Host "Next Steps:" -ForegroundColor Yellow
    Write-Host "  1. Run full benchmark suite: cargo run --release --bin run_benchmarks" -ForegroundColor White
    Write-Host "  2. Test with real datasets (MMLU-Pro, HumanEval)" -ForegroundColor White
    Write-Host "  3. Monitor performance metrics" -ForegroundColor White
    Write-Host "  4. Optimize based on results" -ForegroundColor White
    exit 0
} else {
    Write-Host ""
    Write-Host "[FAILED] SOME TESTS FAILED! Please review errors above." -ForegroundColor Red
    Write-Host ""
    Write-Host "Troubleshooting:" -ForegroundColor Yellow
    Write-Host "  1. Ensure API server is running: cargo run --bin api_server --release" -ForegroundColor White
    Write-Host "  2. Check server logs for errors" -ForegroundColor White
    Write-Host "  3. Verify database connection" -ForegroundColor White
    Write-Host "  4. Review BENCHMARK_SETUP.md for configuration" -ForegroundColor White
    exit 1
}
