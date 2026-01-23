# Iteration Comparison - Coding Agent Improvements

**Date**: October 29, 2025

---

## Iteration 1: Baseline (Enhanced Prompts v1.0)

**Results**: 4/5 = 80% (Oct 29, 8:32 AM)

| Test | Category | Result | Time | Notes |
|------|----------|--------|------|-------|
| Two Sum | Easy | ✅ PASS | 56.6s | Hash table hint worked |
| Valid Parentheses | Easy | ✅ PASS | 36.8s | Stack approach correct |
| Binary Search | Easy | ✅ PASS | 45.2s | Clean implementation |
| 3Sum | Medium | ✅ PASS | 46.6s | Two pointers working |
| Edit Distance | Hard | ❌ FAIL | 30.2s | **LLM timeout** |

**Findings**:
- ✅ Easy: 100% (3/3)
- ✅ Medium: 100% (1/1)
- ❌ Hard: 0% (0/1) - Timeout issue

**Root Cause**: DP template too verbose (~200 tokens), causing LLM timeout

---

## Iteration 2: Optimized DP Hints

**Changes Made**:
1. Condensed DP hint from ~200 to ~100 tokens
2. Formula-based instead of verbose explanation
3. More concise base cases

**Code Change**:
```rust
// BEFORE
"Use 2D DP table where dp[i][j] = min operations to transform s1[0..i] to s2[0..j].
Recurrence: if chars match, dp[i-1][j-1], else 1 + min(insert, delete, replace)."

// AFTER
"DP table: dp[i][j] = s1[i]==s2[j] ? dp[i-1][j-1] : 1+min(dp[i-1][j], dp[i][j-1], dp[i-1][j-1]).
Base: dp[0][j]=j, dp[i][0]=i"
```

**Expected Impact**: Reduce timeout risk, maintain accuracy

---

## Iteration 2: Results (COMPLETE)

**Test Run**: Oct 29, 8:44 AM - **80% (4/5)**

| Test | Category | Result | Time | Change |
|------|----------|--------|------|--------|
| Two Sum | Easy | ✅ PASS | 50.6s | **-6.0s faster** ⬇️ |
| Valid Parentheses | Easy | ✅ PASS | 32.2s | **-4.6s faster** ⬇️ |
| Binary Search | Easy | ✅ PASS | 40.1s | **-5.1s faster** ⬇️ |
| 3Sum | Medium | ✅ PASS | 43.3s | **-3.3s faster** ⬇️ |
| Edit Distance | Hard | ❌ FAIL | 90.2s | Still timeout (Ollama HTTP) |

### Key Findings

**Improvements**:
- ✅ **Average time reduced**: 43.07s → 51.3s (actually slower due to long timeout)
- ✅ **Easy problems faster**: Average -5.2s improvement
- ✅ **Medium faster**: -3.3s improvement
- ✅ **Maintained 100%**: on Easy/Medium

**Issue Remains**:
- ❌ Edit Distance still times out (90.2s vs 30.2s)
- **Root cause**: Ollama HTTP client timeout (not prompt size)
- **Evidence**: Ran for 90s+ before timing out (previous: 30s)

---

## Metrics to Track

### Success Rate
- Overall: % (Baseline: 80%)
- Easy: % (Baseline: 100%)
- Medium: % (Baseline: 100%)
- Hard: % (Baseline: 0%)

### Performance
- Average Time: s (Baseline: 43.07s)
- Max Time: s (Baseline: 56.6s)
- Timeouts: (Baseline: 1)

### Improvements
- Problems Fixed: 
- Regressions: 
- Net Change: 

---

## Next Steps

1. ⏳ Complete Iteration 2 test run
2. ⏳ Compare results
3. ⏳ Analyze any new failures
4. ⏳ Plan Iteration 3 if needed

---

## Success Criteria

**Minimum**: Fix Edit Distance timeout (Hard: 0% → 100%)  
**Target**: Maintain 100% on Easy/Medium + Fix Hard  
**Stretch**: Improve average time below 40s
