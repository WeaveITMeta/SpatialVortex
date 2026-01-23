#!/usr/bin/env pwsh
# Quick Coding Agent Challenge Demo
# Runs 5 standard programming challenges and reports results

Write-Host ""
Write-Host "🎯 Coding Agent Challenge Test Suite" -ForegroundColor Cyan
Write-Host "Based on LeetCode and HackerRank problems" -ForegroundColor Gray
Write-Host "=" * 70 -ForegroundColor Cyan
Write-Host ""

$challenges = @(
    @{
        Name = "Two Sum (LeetCode #1 - Easy)"
        Prompt = "Write a Python function called two_sum that takes a list of integers and a target. Return indices of two numbers that add up to target. Example: two_sum([2,7,11,15], 9) returns [0,1]"
        Expected = "def two_sum"
    },
    @{
        Name = "Palindrome Number (LeetCode #9 - Easy)"
        Prompt = "Write a Python function called is_palindrome that checks if a number is a palindrome without converting to string. Example: is_palindrome(121) returns True"
        Expected = "def is_palindrome"
    },
    @{
        Name = "Fibonacci (Classic - Easy)"
        Prompt = "Write a Python function called fibonacci that returns the nth Fibonacci number using dynamic programming. Example: fibonacci(10) returns 55"
        Expected = "def fibonacci"
    },
    @{
        Name = "Binary Search (Classic - Medium)"
        Prompt = "Write a Python function called binary_search that implements binary search on a sorted list. Return the index of target or -1. Example: binary_search([1,2,3,4,5], 3) returns 2"
        Expected = "def binary_search"
    },
    @{
        Name = "Merge Sorted Arrays (LeetCode #88 - Medium)"
        Prompt = "Write a Python function called merge that merges two sorted lists into one sorted list. Example: merge([1,3,5], [2,4,6]) returns [1,2,3,4,5,6]"
        Expected = "def merge"
    }
)

$results = @()
$passed = 0
$failed = 0
$totalTime = 0

foreach ($challenge in $challenges) {
    Write-Host "📝 Challenge: " -ForegroundColor Yellow -NoNewline
    Write-Host $challenge.Name -ForegroundColor White
    Write-Host "   Prompt: $($challenge.Prompt.Substring(0, [Math]::Min(80, $challenge.Prompt.Length)))..." -ForegroundColor Gray
    Write-Host ""
    
    $startTime = Get-Date
    
    # Run the coding agent
    $output = & ./target/release/coding_agent_cli $challenge.Prompt 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    
    $endTime = Get-Date
    $duration = ($endTime - $startTime).TotalSeconds
    $totalTime += $duration
    
    # Parse results
    $success = $false
    $language = "Unknown"
    $execStatus = "Unknown"
    $code = ""
    
    if ($output -match "Language: (\w+)") {
        $language = $matches[1]
    }
    
    if ($output -match "Status: (.*?)Exit Code") {
        $execStatus = $matches[1].Trim()
    }
    
    if ($output -match "Exit Code: (\d+)") {
        $execCode = $matches[1]
        if ($execCode -eq "0") {
            $success = $true
        }
    }
    
    # Extract generated code
    if ($output -match '```(?:py|python)?\s*\n(.*?)```' -or $output -match 'Generated Code:\s*```(?:py|python)?\s*\n(.*?)```') {
        $code = $matches[1].Trim()
    }
    
    # Check if expected function is present
    $hasExpected = $code -match $challenge.Expected
    
    # Display results
    Write-Host "   📊 Results:" -ForegroundColor Cyan
    Write-Host "      Language: $language" -ForegroundColor Gray
    Write-Host "      Execution: " -NoNewline -ForegroundColor Gray
    
    if ($success) {
        Write-Host "✅ Success" -ForegroundColor Green
        $passed++
    } else {
        Write-Host "❌ Failed" -ForegroundColor Red
        $failed++
    }
    
    Write-Host "      Contains '$($challenge.Expected)': " -NoNewline -ForegroundColor Gray
    if ($hasExpected) {
        Write-Host "✅ Yes" -ForegroundColor Green
    } else {
        Write-Host "❌ No" -ForegroundColor Red
    }
    
    Write-Host "      Time: ${duration}s" -ForegroundColor Gray
    Write-Host ""
    
    Write-Host "   📝 Generated Code:" -ForegroundColor Cyan
    if ($code.Length -gt 0) {
        $codeLines = $code -split "`n"
        foreach ($line in $codeLines | Select-Object -First 10) {
            Write-Host "      $line" -ForegroundColor White
        }
        if ($codeLines.Count -gt 10) {
            $moreLines = $codeLines.Count - 10
            Write-Host "      ... $moreLines more lines" -ForegroundColor DarkGray
        }
    } else {
        Write-Host "      (code not extracted)" -ForegroundColor DarkRed
    }
    
    Write-Host ""
    Write-Host ("   " + ("-" * 66)) -ForegroundColor DarkGray
    Write-Host ""
    
    $results += [PSCustomObject]@{
        Challenge = $challenge.Name
        Language = $language
        Success = $success
        HasExpectedFunction = $hasExpected
        Duration = [math]::Round($duration, 2)
        CodeLines = ($code -split "`n").Count
    }
}

# Final Summary
Write-Host ""
Write-Host "=" * 70 -ForegroundColor Cyan
Write-Host "🏆 Final Results" -ForegroundColor Cyan
Write-Host "=" * 70 -ForegroundColor Cyan
Write-Host ""

Write-Host "Total Challenges:  $($challenges.Count)" -ForegroundColor White
Write-Host "Passed:            " -NoNewline -ForegroundColor White
Write-Host "$passed " -ForegroundColor Green -NoNewline
Write-Host "✅" -ForegroundColor Green
Write-Host "Failed:            " -NoNewline -ForegroundColor White
Write-Host "$failed " -ForegroundColor Red -NoNewline
Write-Host "❌" -ForegroundColor Red

$successRate = [math]::Round(($passed / $challenges.Count) * 100, 1)
Write-Host "Success Rate:      " -NoNewline -ForegroundColor White
if ($successRate -ge 80) {
    Write-Host "$successRate% 🎉" -ForegroundColor Green
} elseif ($successRate -ge 60) {
    Write-Host "$successRate% 👍" -ForegroundColor Yellow
} else {
    Write-Host "$successRate% 😞" -ForegroundColor Red
}

$avgTime = [math]::Round($totalTime / $challenges.Count, 2)
Write-Host "Average Time:      ${avgTime}s" -ForegroundColor White
Write-Host "Total Time:        ${totalTime}s" -ForegroundColor White
Write-Host ""

# Detailed table
Write-Host "📋 Detailed Results:" -ForegroundColor Cyan
$results | Format-Table -AutoSize

Write-Host ""
Write-Host "💡 Notes:" -ForegroundColor Yellow
Write-Host "   • These challenges are from LeetCode and HackerRank" -ForegroundColor Gray
Write-Host "   • All problems are commonly asked in technical interviews" -ForegroundColor Gray
Write-Host "   • Success means code compiled and executed without errors" -ForegroundColor Gray
Write-Host ""

# Save results
$timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$resultsFile = "coding_challenge_results_$timestamp.json"

$resultsJson = @{
    timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    total_challenges = $challenges.Count
    passed = $passed
    failed = $failed
    success_rate = $successRate
    avg_time = $avgTime
    total_time = $totalTime
    challenges = $results
} | ConvertTo-Json -Depth 3

$resultsJson | Out-File -FilePath $resultsFile -Encoding UTF8

Write-Host "[SAVED] Results saved to: $resultsFile" -ForegroundColor Green
Write-Host ""
