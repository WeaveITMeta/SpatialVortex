# Testing Scripts

Consolidated location for all coding agent test scripts.

## Available Scripts

### `coding_agent_benchmark.ps1` (RECOMMENDED)
Comprehensive benchmark suite for measuring coding agent performance.

**Usage:**
```powershell
# Quick test (5 tests)
.\coding_agent_benchmark.ps1 -Quick

# Full test suite (16 tests)
.\coding_agent_benchmark.ps1 -Full

# Verbose output
.\coding_agent_benchmark.ps1 -Quick -Verbose
```

**Features:**
- Real execution with actual results
- Multiple test suites (quick/full)
- Category-based analysis (Easy/Medium/Hard/Multi-Lang)
- JSON result export
- Performance metrics

### `quick_test.ps1` (LEGACY)
Simple 5-test suite for quick validation.

**Usage:**
```powershell
.\quick_test.ps1
```

### `measure_coverage.ps1`
Code coverage measurement tool.

## Test Results

All test results are automatically saved to `../../test_results/` with timestamps.

## Test Categories

- **Easy**: Basic algorithms (Two Sum, Binary Search, etc.)
- **Medium**: Intermediate problems (3Sum, Longest Substring, etc.)
- **Hard**: Advanced algorithms (Edit Distance, Regular Expression, etc.)
- **Multi-Lang**: Cross-language tests (Rust, JavaScript, TypeScript)

## Requirements

- Coding agent CLI must be built: `cargo build --release --bin coding_agent_cli --features agents`
- LLM backend must be configured (Ollama recommended)

## Success Criteria

- **Excellent**: ≥90% success rate
- **Good**: ≥80% success rate
- **Acceptable**: ≥70% success rate
- **Needs Work**: ≥60% success rate
- **Failing**: <60% success rate
