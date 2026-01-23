# Benchmark Dataset Download Scripts

This folder contains scripts to download official benchmark datasets.

## For Windows (PowerShell)

```powershell
# Download all datasets
.\benchmarks\scripts\download_datasets.ps1

# Verify downloads
.\benchmarks\scripts\verify_datasets.ps1
```

## For Linux/Mac (Bash)

```bash
# Make executable
chmod +x benchmarks/scripts/download_datasets.sh
chmod +x benchmarks/scripts/verify_datasets.sh

# Download all datasets
./benchmarks/scripts/download_datasets.sh

# Verify downloads
./benchmarks/scripts/verify_datasets.sh
```

## What Gets Downloaded

1. **FB15k-237** (~310K triples) - Knowledge graph link prediction
2. **WN18RR** (~93K triples) - Lexical knowledge graph
3. **STS Benchmark** (~8.6K pairs) - Semantic similarity
4. **SICK** (~10K pairs) - Compositional semantics
5. **SQuAD 2.0** (~150K questions) - Reading comprehension
6. **CommonsenseQA** (~12K questions) - Common sense reasoning
7. **bAbI** (20 tasks) - Toy reasoning tasks
8. **CLUTRR** - Kinship reasoning
9. **Silesia Corpus** (211 MB) - Text compression benchmark

## Total Size

Approximately **500 MB** to **1 GB** of data.

## Requirements

### Windows
- PowerShell 5.1+ (included with Windows 10+)
- `tar.exe` (included with Windows 10+ build 17063+)

### Linux/Mac
- `curl`
- `unzip`
- `tar`

## Troubleshooting

### Windows: "Execution of scripts is disabled"

Run this in PowerShell as Administrator:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Windows: tar.exe not found

Update Windows to the latest version (Windows 10 build 17063+) or install Git Bash.

### Download fails

Some datasets may be temporarily unavailable. Wait a few minutes and try again, or check the BENCHMARK.md file for alternative download links.
