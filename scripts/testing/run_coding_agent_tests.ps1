#!/usr/bin/env pwsh
# Coding Agent Benchmark Test Runner
# Runs comprehensive tests based on LeetCode, HackerRank, and Project Euler problems

Write-Host "🧪 SpatialVortex Coding Agent Benchmark Tests" -ForegroundColor Cyan
Write-Host "=" * 60 -ForegroundColor Cyan
Write-Host ""

# Test categories
$tests = @(
    @{Name="Two Sum (Easy)"; Test="test_two_sum"; Category="Easy"},
    @{Name="Palindrome Check (Easy)"; Test="test_palindrome_check"; Category="Easy"},
    @{Name="Reverse String (Easy)"; Test="test_reverse_string"; Category="Easy"},
    @{Name="Fibonacci (Easy)"; Test="test_fibonacci"; Category="Easy"},
    @{Name="Merge Intervals (Medium)"; Test="test_merge_intervals"; Category="Medium"},
    @{Name="Longest Substring (Medium)"; Test="test_longest_substring"; Category="Medium"},
    @{Name="Binary Search Tree (Medium)"; Test="test_binary_search_tree"; Category="Medium"},
    @{Name="Median of Two Arrays (Hard)"; Test="test_median_two_sorted_arrays"; Category="Hard"},
    @{Name="Word Ladder (Hard)"; Test="test_word_ladder"; Category="Hard"},
    @{Name="Rust Quicksort (Multi-Lang)"; Test="test_rust_quicksort"; Category="Multi-Language"},
    @{Name="JavaScript Debounce (Multi-Lang)"; Test="test_javascript_debounce"; Category="Multi-Language"},
    @{Name="Coin Change DP (Optimization)"; Test="test_dynamic_programming"; Category="Optimization"},
    @{Name="LRU Cache (Complex)"; Test="test_complex_data_structure"; Category="Complex"}
)

$results = @()
$totalTests = $tests.Count
$passed = 0
$failed = 0
$totalTime = 0

Write-Host "📋 Running $totalTests tests..." -ForegroundColor Yellow
Write-Host ""

foreach ($test in $tests) {
    $testName = $test.Name
    $testFunc = $test.Test
    $category = $test.Category
    
    Write-Host "▶️  Running: $testName [$category]" -ForegroundColor White
    
    $startTime = Get-Date
    
    # Run the test
    $output = cargo test --test coding_agent_benchmark $testFunc --release -- --nocapture 2>&1
    $exitCode = $LASTEXITCODE
    
    $endTime = Get-Date
    $duration = ($endTime - $startTime).TotalSeconds
    $totalTime += $duration
    
    if ($exitCode -eq 0) {
        Write-Host "   ✅ PASSED" -ForegroundColor Green -NoNewline
        Write-Host " (${duration}s)" -ForegroundColor Gray
        $passed++
        $status = "PASSED"
    } else {
        Write-Host "   ❌ FAILED" -ForegroundColor Red -NoNewline
        Write-Host " (${duration}s)" -ForegroundColor Gray
        $failed++
        $status = "FAILED"
        
        # Show error details
        Write-Host "   Error: " -ForegroundColor Red -NoNewline
        $errorLines = $output | Select-String -Pattern "Error|assertion|panicked" | Select-Object -First 3
        foreach ($line in $errorLines) {
            Write-Host "   $line" -ForegroundColor DarkRed
        }
    }
    
    $results += [PSCustomObject]@{
        Test = $testName
        Category = $category
        Status = $status
        Duration = [math]::Round($duration, 2)
    }
    
    Write-Host ""
}

# Summary
Write-Host "=" * 60 -ForegroundColor Cyan
Write-Host "📊 Test Summary" -ForegroundColor Cyan
Write-Host "=" * 60 -ForegroundColor Cyan
Write-Host ""

Write-Host "Total Tests:  $totalTests" -ForegroundColor White
Write-Host "Passed:       " -NoNewline -ForegroundColor White
Write-Host "$passed" -ForegroundColor Green
Write-Host "Failed:       " -NoNewline -ForegroundColor White
Write-Host "$failed" -ForegroundColor Red
Write-Host "Success Rate: " -NoNewline -ForegroundColor White
$successRate = [math]::Round(($passed / $totalTests) * 100, 1)
if ($successRate -ge 80) {
    Write-Host "$successRate%" -ForegroundColor Green
} elseif ($successRate -ge 50) {
    Write-Host "$successRate%" -ForegroundColor Yellow
} else {
    Write-Host "$successRate%" -ForegroundColor Red
}
Write-Host "Total Time:   ${totalTime}s" -ForegroundColor White
$avgTime = [math]::Round($totalTime / $totalTests, 2)
Write-Host "Avg Time:     ${avgTime}s" -ForegroundColor White
Write-Host ""

# Category breakdown
Write-Host "📈 Results by Category:" -ForegroundColor Cyan
Write-Host ""

$categories = $results | Group-Object Category
foreach ($cat in $categories) {
    $catName = $cat.Name
    $catPassed = ($cat.Group | Where-Object {$_.Status -eq "PASSED"}).Count
    $catTotal = $cat.Count
    $catRate = [math]::Round(($catPassed / $catTotal) * 100, 1)
    
    Write-Host "  $catName : " -NoNewline -ForegroundColor White
    Write-Host "$catPassed/$catTotal " -NoNewline
    Write-Host "($catRate%)" -ForegroundColor $(if ($catRate -ge 80) {"Green"} elseif ($catRate -ge 50) {"Yellow"} else {"Red"})
}

Write-Host ""

# Detailed results table
Write-Host "📋 Detailed Results:" -ForegroundColor Cyan
Write-Host ""
$results | Format-Table -AutoSize

# Performance insights
Write-Host "⚡ Performance Insights:" -ForegroundColor Cyan
Write-Host ""

$slowest = $results | Sort-Object Duration -Descending | Select-Object -First 3
Write-Host "  Slowest Tests:" -ForegroundColor Yellow
foreach ($slow in $slowest) {
    Write-Host "    • $($slow.Test): $($slow.Duration)s" -ForegroundColor Gray
}

$fastest = $results | Sort-Object Duration | Select-Object -First 3
Write-Host ""
Write-Host "  Fastest Tests:" -ForegroundColor Green
foreach ($fast in $fastest) {
    Write-Host "    • $($fast.Test): $($fast.Duration)s" -ForegroundColor Gray
}

Write-Host ""
Write-Host "=" * 60 -ForegroundColor Cyan

# Export results to JSON
$resultsJson = @{
    timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    total_tests = $totalTests
    passed = $passed
    failed = $failed
    success_rate = $successRate
    total_time = $totalTime
    avg_time = $avgTime
    results = $results
} | ConvertTo-Json -Depth 3

$resultsJson | Out-File -FilePath "coding_agent_benchmark_results.json" -Encoding UTF8

Write-Host "📁 Results saved to: coding_agent_benchmark_results.json" -ForegroundColor Green
Write-Host ""
