# Coding Agent Benchmark Test Suite
# Consolidated test script for measuring coding agent performance
# Tests with REAL execution, not expectations

param(
    [switch]$Verbose,
    [switch]$Quick,  # Run 5 quick tests
    [switch]$Full    # Run full 16 test suite
)

Write-Host ""
Write-Host "=== Coding Agent Benchmark ===" -ForegroundColor Cyan
Write-Host ""

# Check if CLI exists
$cliPath = "../../target/release/coding_agent_cli.exe"
if (-not (Test-Path $cliPath)) {
    Write-Host "ERROR: CLI not found. Building..." -ForegroundColor Red
    Set-Location ../..
    cargo build --release --bin coding_agent_cli --features agents
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed. Exiting." -ForegroundColor Red
        exit 1
    }
    Set-Location scripts/testing
}

# Define test suites
$quickTests = @(
    @{Name="Two Sum"; Prompt="Write a Python function called two_sum that takes a list of integers and a target integer. Return indices of two numbers that sum to target. Use hash table for O(n) time."; ExpectedFunc="def two_sum"; Category="Easy"},
    @{Name="Valid Parentheses"; Prompt="Write a Python function called is_valid that checks if a string of parentheses is valid using a stack."; ExpectedFunc="def is_valid"; Category="Easy"},
    @{Name="Binary Search"; Prompt="Write a Python function called binary_search that performs binary search on a sorted array."; ExpectedFunc="def binary_search"; Category="Easy"},
    @{Name="3Sum"; Prompt="Write a Python function called three_sum that finds all unique triplets in array that sum to zero."; ExpectedFunc="def three_sum"; Category="Medium"},
    @{Name="Edit Distance"; Prompt="Write a Python function called min_distance that finds minimum edit distance using dynamic programming."; ExpectedFunc="def min_distance"; Category="Hard"}
)

$fullTests = @(
    @{Name="Two Sum"; Prompt="Write a Python function called two_sum that takes a list of integers and a target integer. Return indices of two numbers that sum to target."; ExpectedFunc="def two_sum"; Category="Easy"},
    @{Name="Valid Parentheses"; Prompt="Write a Python function called is_valid that checks if a string of parentheses is valid using a stack."; ExpectedFunc="def is_valid"; Category="Easy"},
    @{Name="Remove Duplicates"; Prompt="Write a Python function called remove_duplicates that removes duplicates from sorted array in-place."; ExpectedFunc="def remove_duplicates"; Category="Easy"},
    @{Name="Merge Sorted Arrays"; Prompt="Write a Python function called merge that merges two sorted arrays."; ExpectedFunc="def merge"; Category="Easy"},
    @{Name="Maximum Subarray"; Prompt="Write a Python function called max_subarray using Kadane's algorithm."; ExpectedFunc="def max_subarray"; Category="Easy"},
    @{Name="3Sum"; Prompt="Write a Python function called three_sum that finds all unique triplets that sum to zero."; ExpectedFunc="def three_sum"; Category="Medium"},
    @{Name="Longest Substring"; Prompt="Write a Python function called length_of_longest_substring without repeating characters."; ExpectedFunc="def length_of_longest_substring"; Category="Medium"},
    @{Name="Container With Water"; Prompt="Write a Python function called max_area for container with most water problem."; ExpectedFunc="def max_area"; Category="Medium"},
    @{Name="Group Anagrams"; Prompt="Write a Python function called group_anagrams that groups anagrams together."; ExpectedFunc="def group_anagrams"; Category="Medium"},
    @{Name="Binary Tree Level Order"; Prompt="Write a Python function called level_order for binary tree level order traversal."; ExpectedFunc="def level_order"; Category="Medium"},
    @{Name="Trapping Rain Water"; Prompt="Write a Python function called trap that calculates trapped rain water."; ExpectedFunc="def trap"; Category="Hard"},
    @{Name="Edit Distance"; Prompt="Write a Python function called min_distance for edit distance using DP."; ExpectedFunc="def min_distance"; Category="Hard"},
    @{Name="Regular Expression"; Prompt="Write a Python function called is_match for regex matching with . and *."; ExpectedFunc="def is_match"; Category="Hard"},
    @{Name="Rust Quicksort"; Prompt="Write a Rust function called quicksort that sorts a mutable vector in-place."; ExpectedFunc="fn quicksort"; Category="Multi-Lang"},
    @{Name="JavaScript Async"; Prompt="Write a JavaScript async function called fetch_users that fetches users from API."; ExpectedFunc="async function fetch_users"; Category="Multi-Lang"},
    @{Name="TypeScript Interface"; Prompt="Write a TypeScript interface called User with name, email, and age fields."; ExpectedFunc="interface User"; Category="Multi-Lang"}
)

# Select test suite
if ($Quick) {
    $tests = $quickTests
    Write-Host "Running QUICK test suite (5 tests)" -ForegroundColor Yellow
} elseif ($Full) {
    $tests = $fullTests
    Write-Host "Running FULL test suite (16 tests)" -ForegroundColor Yellow
} else {
    $tests = $quickTests
    Write-Host "Running default QUICK test suite (5 tests)" -ForegroundColor Yellow
    Write-Host "Use -Full for complete test suite" -ForegroundColor Gray
}

Write-Host ""

$results = @()
$totalTime = 0

foreach ($test in $tests) {
    $testNum = $results.Count + 1
    Write-Host "[$testNum/$($tests.Count)] $($test.Name) ($($test.Category))" -ForegroundColor White
    
    $startTime = Get-Date
    
    try {
        $output = & $cliPath $test.Prompt 2>&1 | Out-String
        $endTime = Get-Date
        $duration = ($endTime - $startTime).TotalSeconds
        $totalTime += $duration
        
        # Parse results
        $hasFunction = $output -match [regex]::Escape($test.ExpectedFunc)
        $exitCode = if ($output -match "Exit Code: (\d+)") { [int]$matches[1] } else { -1 }
        $success = ($exitCode -eq 0 -and $hasFunction)
        
        Write-Host "  Result: $(if ($success) {'PASS ✓'} else {'FAIL ✗'})" -ForegroundColor $(if ($success) {"Green"} else {"Red"})
        Write-Host "  Time: $([math]::Round($duration, 2))s" -ForegroundColor Gray
        
        if ($Verbose) {
            Write-Host "  Exit Code: $exitCode" -ForegroundColor Gray
            Write-Host "  Has Function: $hasFunction" -ForegroundColor Gray
        }
        
        $results += [PSCustomObject]@{
            Test = $test.Name
            Category = $test.Category
            Success = $success
            Duration = [math]::Round($duration, 2)
            ExitCode = $exitCode
            HasFunction = $hasFunction
        }
        
    } catch {
        Write-Host "  ERROR: $_" -ForegroundColor Red
        $results += [PSCustomObject]@{
            Test = $test.Name
            Category = $test.Category
            Success = $false
            Duration = 0
            ExitCode = -1
            HasFunction = $false
        }
    }
    
    Write-Host ""
}

# Calculate statistics
Write-Host "=" * 70 -ForegroundColor Cyan
Write-Host "RESULTS" -ForegroundColor Cyan
Write-Host "=" * 70 -ForegroundColor Cyan
Write-Host ""

$totalTests = $results.Count
$passed = ($results | Where-Object {$_.Success}).Count
$failed = $totalTests - $passed
$successRate = [math]::Round(($passed / $totalTests) * 100, 1)
$avgTime = [math]::Round($totalTime / $totalTests, 2)

Write-Host "Total Tests: $totalTests" -ForegroundColor White
Write-Host "Passed: " -NoNewline
Write-Host "$passed " -NoNewline -ForegroundColor Green
Write-Host "($successRate%)"
Write-Host "Failed: " -NoNewline
Write-Host "$failed" -ForegroundColor $(if ($failed -eq 0) {"Green"} else {"Red"})
Write-Host "Average Time: ${avgTime}s" -ForegroundColor Gray
Write-Host ""

# By category
Write-Host "By Category:" -ForegroundColor Cyan
$categories = $results | Group-Object -Property Category
foreach ($cat in $categories) {
    $catPassed = ($cat.Group | Where-Object {$_.Success}).Count
    $catTotal = $cat.Count
    $catRate = [math]::Round(($catPassed / $catTotal) * 100, 1)
    
    Write-Host "  $($cat.Name): $catPassed/$catTotal " -NoNewline
    Write-Host "($catRate%)" -ForegroundColor $(if ($catRate -ge 80) {"Green"} elseif ($catRate -ge 60) {"Yellow"} else {"Red"})
}
Write-Host ""

# Verdict
Write-Host "VERDICT: " -NoNewline
if ($successRate -ge 90) {
    Write-Host "EXCELLENT ($successRate%)" -ForegroundColor Green
} elseif ($successRate -ge 80) {
    Write-Host "GOOD ($successRate%)" -ForegroundColor Green
} elseif ($successRate -ge 70) {
    Write-Host "ACCEPTABLE ($successRate%)" -ForegroundColor Yellow
} elseif ($successRate -ge 60) {
    Write-Host "NEEDS WORK ($successRate%)" -ForegroundColor Yellow
} else {
    Write-Host "FAILING ($successRate%)" -ForegroundColor Red
}

Write-Host ""

# Save results
$timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$resultFile = "../../benchmark_results_$timestamp.json"

$report = @{
    timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    test_suite = if ($Quick -or -not $Full) {"quick"} else {"full"}
    total_tests = $totalTests
    passed = $passed
    failed = $failed
    success_rate = $successRate
    average_time = $avgTime
    total_time = $totalTime
    results = $results
} | ConvertTo-Json -Depth 4

$report | Out-File -FilePath $resultFile -Encoding UTF8

Write-Host "Results saved to: $resultFile" -ForegroundColor Green
Write-Host ""
