# Quick Coding Agent Test
# Tests 5 standard programming challenges

Write-Host ""
Write-Host "=== Coding Agent Challenge Test ===" -ForegroundColor Cyan
Write-Host ""

$tests = @(
    "Write a Python function called two_sum that takes a list and target, returns indices that sum to target",
    "Write a Python function called fibonacci that returns the nth Fibonacci number using memoization",
    "Write a Python function called binary_search on a sorted list, returns index or -1",
    "Write a Python function called reverse_string that reverses a string",
    "Write a Python function called is_palindrome that checks if a string is a palindrome"
)

$passed = 0
$failed = 0

for ($i = 0; $i -lt $tests.Count; $i++) {
    $testNum = $i + 1
    $prompt = $tests[$i]
    
    Write-Host "[$testNum/$($tests.Count)] Running: $prompt" -ForegroundColor Yellow
    
    $output = & ./target/release/coding_agent_cli $prompt 2>&1 | Out-String
    
    if ($output -match "Exit Code: 0") {
        Write-Host "  [PASS] Success" -ForegroundColor Green
        $passed++
    } else {
        Write-Host "  [FAIL] Failed" -ForegroundColor Red
        $failed++
    }
    
    if ($output -match '```(?:py|python)?\s*\n(.*?)```') {
        $code = $matches[1].Trim()
        $lines = ($code -split "`n") | Select-Object -First 5
        Write-Host "  Code preview:" -ForegroundColor Cyan
        foreach ($line in $lines) {
            Write-Host "    $line" -ForegroundColor Gray
        }
    }
    
    Write-Host ""
}

Write-Host "=== Results ===" -ForegroundColor Cyan
Write-Host "Passed: $passed / $($tests.Count)" -ForegroundColor Green
Write-Host "Failed: $failed / $($tests.Count)" -ForegroundColor Red
$rate = [math]::Round(($passed / $tests.Count) * 100, 1)
Write-Host "Success Rate: $rate%" -ForegroundColor White
Write-Host ""
