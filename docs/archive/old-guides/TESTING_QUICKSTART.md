# Testing Quick Start

## Run Tests Now

```powershell
# Navigate to testing directory
cd scripts\testing

# Run quick test (5 tests, ~2-3 minutes)
.\coding_agent_benchmark.ps1 -Quick

# OR run full test (16 tests, ~8-10 minutes)
.\coding_agent_benchmark.ps1 -Full
```

## What Gets Tested

**Quick Mode** (5 tests):
- Two Sum (Easy)
- Valid Parentheses (Easy)  
- Binary Search (Easy)
- 3Sum (Medium)
- Edit Distance (Hard)

**Full Mode** (16 tests):
- 5 Easy problems
- 5 Medium problems
- 3 Hard problems
- 3 Multi-language problems

## Results Location

- Console: Real-time results during test
- Files: `test_results/benchmark_results_TIMESTAMP.json`

## Documentation

- **How to use**: `docs/agents/PROMPT_BENCHMARK_GUIDE.md`
- **What changed**: `docs/agents/WORKSPACE_REORGANIZATION.md`
- **Implementation**: `docs/agents/IMPLEMENTATION_STATUS.md`

## Expected Results

With enhanced prompts (now default):
- Easy: 95%+ success
- Medium: 100% success
- Hard: 70-80% success
- Overall: 85%+ success
