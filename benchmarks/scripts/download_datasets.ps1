# Download benchmark datasets - PowerShell version for Windows
# Run with: .\benchmarks\scripts\download_datasets.ps1
# Re-running will skip already downloaded datasets

$ErrorActionPreference = "Stop"

$BENCHMARK_DIR = "benchmarks\data"
New-Item -ItemType Directory -Force -Path $BENCHMARK_DIR | Out-Null

Write-Host "Downloading benchmark datasets..." -ForegroundColor Cyan
Write-Host "(Existing datasets will be skipped)" -ForegroundColor Gray

$downloaded = 0
$skipped = 0

# FB15k-237 Knowledge Graph
Write-Host "`n[1/16] FB15k-237..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\fb15k237\Release") {
    Write-Host "SKIP FB15k-237 (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\fb15k237" | Out-Null
    Invoke-WebRequest -Uri "https://download.microsoft.com/download/8/7/0/8700516A-AB3D-4850-B4BB-805C515AECE1/FB15K-237.2.zip" -OutFile "$BENCHMARK_DIR\fb15k237.zip"
    Expand-Archive -Path "$BENCHMARK_DIR\fb15k237.zip" -DestinationPath "$BENCHMARK_DIR\fb15k237" -Force
    Remove-Item "$BENCHMARK_DIR\fb15k237.zip"
    Write-Host "OK FB15k-237 downloaded" -ForegroundColor Green
    $downloaded++
}

# WN18RR Knowledge Graph
Write-Host "`n[2/16] WN18RR..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\wn18rr\WN18RR") {
    Write-Host "SKIP WN18RR (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\wn18rr" | Out-Null
    Invoke-WebRequest -Uri "https://github.com/TimDettmers/ConvE/raw/master/WN18RR.tar.gz" -OutFile "$BENCHMARK_DIR\wn18rr.tar.gz"
    tar -xzf "$BENCHMARK_DIR\wn18rr.tar.gz" -C "$BENCHMARK_DIR\wn18rr"
    Remove-Item "$BENCHMARK_DIR\wn18rr.tar.gz"
    Write-Host "OK WN18RR downloaded" -ForegroundColor Green
    $downloaded++
}

# STS Benchmark
Write-Host "`n[3/16] STS Benchmark..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\sts\stsbenchmark") {
    Write-Host "SKIP STS Benchmark (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\sts" | Out-Null
    Invoke-WebRequest -Uri "http://ixa2.si.ehu.es/stswiki/images/4/48/Stsbenchmark.tar.gz" -OutFile "$BENCHMARK_DIR\stsbenchmark.tar.gz"
    tar -xzf "$BENCHMARK_DIR\stsbenchmark.tar.gz" -C "$BENCHMARK_DIR\sts"
    Remove-Item "$BENCHMARK_DIR\stsbenchmark.tar.gz"
    Write-Host "OK STS Benchmark downloaded" -ForegroundColor Green
    $downloaded++
}

# SICK Dataset
Write-Host "`n[4/16] SICK..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\sick\SICK_train.txt") {
    Write-Host "SKIP SICK (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\sick" | Out-Null
    try {
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/brmson/dataset-sts/master/data/sts/sick2014/SICK_train.txt" -OutFile "$BENCHMARK_DIR\sick\SICK_train.txt" -UseBasicParsing
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/brmson/dataset-sts/master/data/sts/sick2014/SICK_trial.txt" -OutFile "$BENCHMARK_DIR\sick\SICK_trial.txt" -UseBasicParsing
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/brmson/dataset-sts/master/data/sts/sick2014/SICK_test_annotated.txt" -OutFile "$BENCHMARK_DIR\sick\SICK_test.txt" -UseBasicParsing
        Write-Host "OK SICK downloaded" -ForegroundColor Green
        $downloaded++
    } catch {
        Write-Host "WARN: SICK download failed, skipping..." -ForegroundColor Yellow
        "# SICK dataset unavailable" | Out-File "$BENCHMARK_DIR\sick\README.txt"
    }
}

# SQuAD 2.0
Write-Host "`n[5/16] SQuAD 2.0..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\squad\dev-v2.0.json") {
    Write-Host "SKIP SQuAD 2.0 (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\squad" | Out-Null
    Invoke-WebRequest -Uri "https://rajpurkar.github.io/SQuAD-explorer/dataset/dev-v2.0.json" -OutFile "$BENCHMARK_DIR\squad\dev-v2.0.json"
    Write-Host "OK SQuAD 2.0 downloaded" -ForegroundColor Green
    $downloaded++
}

# CommonsenseQA
Write-Host "`n[6/16] CommonsenseQA..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\commonsenseqa\dev.jsonl") {
    Write-Host "SKIP CommonsenseQA (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\commonsenseqa" | Out-Null
    Invoke-WebRequest -Uri "https://s3.amazonaws.com/commensenseqa/dev_rand_split.jsonl" -OutFile "$BENCHMARK_DIR\commonsenseqa\dev.jsonl"
    Write-Host "OK CommonsenseQA downloaded" -ForegroundColor Green
    $downloaded++
}

# bAbI Tasks
Write-Host "`n[7/16] bAbI..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\babi\tasks_1-20_v1-2") {
    Write-Host "SKIP bAbI (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\babi" | Out-Null
    Invoke-WebRequest -Uri "http://www.thespermwhale.com/jaseweston/babi/tasks_1-20_v1-2.tar.gz" -OutFile "$BENCHMARK_DIR\babi.tar.gz"
    tar -xzf "$BENCHMARK_DIR\babi.tar.gz" -C "$BENCHMARK_DIR\babi"
    Remove-Item "$BENCHMARK_DIR\babi.tar.gz"
    Write-Host "OK bAbI downloaded" -ForegroundColor Green
    $downloaded++
}

# CLUTRR
Write-Host "`n[8/16] CLUTRR..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\clutrr\test.csv") {
    Write-Host "SKIP CLUTRR (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\clutrr" | Out-Null
    try {
        Invoke-WebRequest -Uri "https://huggingface.co/datasets/CLUTRR/v1/raw/main/data_089907f8/test.csv" -OutFile "$BENCHMARK_DIR\clutrr\test.csv" -UseBasicParsing -ErrorAction Stop
        Write-Host "OK CLUTRR downloaded" -ForegroundColor Green
        $downloaded++
    } catch {
        Write-Host "SKIP CLUTRR (requires generation - see https://github.com/facebookresearch/clutrr)" -ForegroundColor Yellow
        "# CLUTRR requires generation, not a static file" | Out-File "$BENCHMARK_DIR\clutrr\README.txt"
    }
}

# Silesia Corpus
Write-Host "`n[9/16] Silesia Corpus..." -ForegroundColor Yellow
if ((Get-ChildItem "$BENCHMARK_DIR\silesia" -ErrorAction SilentlyContinue | Measure-Object).Count -gt 2) {
    Write-Host "SKIP Silesia Corpus (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\silesia" | Out-Null
    Invoke-WebRequest -Uri "http://sun.aei.polsl.pl/~sdeor/corpus/silesia.zip" -OutFile "$BENCHMARK_DIR\silesia.zip"
    Expand-Archive -Path "$BENCHMARK_DIR\silesia.zip" -DestinationPath "$BENCHMARK_DIR\silesia" -Force
    Remove-Item "$BENCHMARK_DIR\silesia.zip"
    Write-Host "OK Silesia Corpus downloaded" -ForegroundColor Green
    $downloaded++
}

# MMLU (Massive Multitask Language Understanding)
Write-Host "`n[10/16] MMLU..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\mmlu\data") {
    Write-Host "SKIP MMLU (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\mmlu" | Out-Null
    try {
        Invoke-WebRequest -Uri "https://people.eecs.berkeley.edu/~hendrycks/data.tar" -OutFile "$BENCHMARK_DIR\mmlu.tar" -UseBasicParsing
        tar -xf "$BENCHMARK_DIR\mmlu.tar" -C "$BENCHMARK_DIR\mmlu"
        Remove-Item "$BENCHMARK_DIR\mmlu.tar"
        Write-Host "OK MMLU downloaded" -ForegroundColor Green
        $downloaded++
    } catch {
        Write-Host "WARN: MMLU download failed - try manual download from HuggingFace" -ForegroundColor Yellow
    }
}

# GSM8K (Grade School Math)
Write-Host "`n[11/16] GSM8K..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\gsm8k\test.jsonl") {
    Write-Host "SKIP GSM8K (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\gsm8k" | Out-Null
    try {
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/openai/grade-school-math/master/grade_school_math/data/test.jsonl" -OutFile "$BENCHMARK_DIR\gsm8k\test.jsonl" -UseBasicParsing
        Write-Host "OK GSM8K downloaded" -ForegroundColor Green
        $downloaded++
    } catch {
        Write-Host "WARN: GSM8K download failed" -ForegroundColor Yellow
    }
}

# ARC (AI2 Reasoning Challenge)
Write-Host "`n[12/16] ARC..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\arc\ARC-V1-Feb2018-2") {
    Write-Host "SKIP ARC (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\arc" | Out-Null
    try {
        Invoke-WebRequest -Uri "https://ai2-public-datasets.s3.amazonaws.com/arc/ARC-V1-Feb2018.zip" -OutFile "$BENCHMARK_DIR\arc.zip" -UseBasicParsing
        Expand-Archive -Path "$BENCHMARK_DIR\arc.zip" -DestinationPath "$BENCHMARK_DIR\arc" -Force
        Remove-Item "$BENCHMARK_DIR\arc.zip"
        Write-Host "OK ARC downloaded" -ForegroundColor Green
        $downloaded++
    } catch {
        Write-Host "WARN: ARC download failed" -ForegroundColor Yellow
    }
}

# HellaSwag
Write-Host "`n[13/16] HellaSwag..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\hellaswag\validation.jsonl") {
    Write-Host "SKIP HellaSwag (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\hellaswag" | Out-Null
    try {
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/rowanz/hellaswag/master/data/hellaswag_val.jsonl" -OutFile "$BENCHMARK_DIR\hellaswag\validation.jsonl" -UseBasicParsing
        Write-Host "OK HellaSwag downloaded" -ForegroundColor Green
        $downloaded++
    } catch {
        Write-Host "WARN: HellaSwag download failed" -ForegroundColor Yellow
    }
}

# TruthfulQA
Write-Host "`n[14/16] TruthfulQA..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\truthfulqa\TruthfulQA.csv") {
    Write-Host "SKIP TruthfulQA (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\truthfulqa" | Out-Null
    try {
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/sylinrl/TruthfulQA/main/TruthfulQA.csv" -OutFile "$BENCHMARK_DIR\truthfulqa\TruthfulQA.csv" -UseBasicParsing
        Write-Host "OK TruthfulQA downloaded" -ForegroundColor Green
        $downloaded++
    } catch {
        Write-Host "WARN: TruthfulQA download failed" -ForegroundColor Yellow
    }
}

# HumanEval
Write-Host "`n[15/16] HumanEval..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\humaneval\HumanEval.jsonl") {
    Write-Host "SKIP HumanEval (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\humaneval" | Out-Null
    try {
        Invoke-WebRequest -Uri "https://raw.githubusercontent.com/openai/human-eval/master/data/HumanEval.jsonl.gz" -OutFile "$BENCHMARK_DIR\humaneval\HumanEval.jsonl.gz" -UseBasicParsing
        # Decompress gzip
        $gzipPath = "$BENCHMARK_DIR\humaneval\HumanEval.jsonl.gz"
        $outputPath = "$BENCHMARK_DIR\humaneval\HumanEval.jsonl"
        $input = New-Object System.IO.FileStream $gzipPath, ([IO.FileMode]::Open), ([IO.FileAccess]::Read)
        $output = New-Object System.IO.FileStream $outputPath, ([IO.FileMode]::Create), ([IO.FileAccess]::Write)
        $gzipStream = New-Object System.IO.Compression.GzipStream $input, ([IO.Compression.CompressionMode]::Decompress)
        $gzipStream.CopyTo($output)
        $gzipStream.Close(); $input.Close(); $output.Close()
        Remove-Item $gzipPath
        Write-Host "OK HumanEval downloaded" -ForegroundColor Green
        $downloaded++
    } catch {
        Write-Host "WARN: HumanEval download failed" -ForegroundColor Yellow
    }
}

# SWE-Bench Lite (Software Engineering)
Write-Host "`n[16/16] SWE-Bench Lite..." -ForegroundColor Yellow
if (Test-Path "$BENCHMARK_DIR\swe-bench-lite\test.parquet") {
    Write-Host "SKIP SWE-Bench Lite (already exists)" -ForegroundColor Gray
    $skipped++
} else {
    New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\swe-bench-lite" | Out-Null
    try {
        Invoke-WebRequest -Uri "https://huggingface.co/datasets/princeton-nlp/SWE-bench_Lite/resolve/main/data/test-00000-of-00001.parquet" -OutFile "$BENCHMARK_DIR\swe-bench-lite\test.parquet" -UseBasicParsing
        Write-Host "OK SWE-Bench Lite downloaded (parquet format)" -ForegroundColor Green
        Write-Host "   Note: Convert parquet to jsonl with Python: pandas.read_parquet().to_json()" -ForegroundColor Yellow
        $downloaded++
    } catch {
        Write-Host "WARN: SWE-Bench Lite download failed - try manual download from HuggingFace" -ForegroundColor Yellow
        Write-Host "   URL: https://huggingface.co/datasets/princeton-nlp/SWE-bench_Lite" -ForegroundColor Yellow
    }
}

Write-Host "`n==================================" -ForegroundColor Cyan
Write-Host "COMPLETE!" -ForegroundColor Green
Write-Host "==================================" -ForegroundColor Cyan
Write-Host "Downloaded: $downloaded | Skipped: $skipped" -ForegroundColor White
Write-Host "Data location: $BENCHMARK_DIR" -ForegroundColor White
Write-Host "`nBenchmarks available:" -ForegroundColor Yellow
Write-Host "  - MMLU (57 subjects)"
Write-Host "  - GSM8K (math)"
Write-Host "  - ARC-Challenge (science)"
Write-Host "  - HellaSwag (commonsense)"
Write-Host "  - TruthfulQA (factual)"
Write-Host "  - HumanEval (code)"
Write-Host "  - SWE-Bench Lite (software engineering)"
Write-Host "  - CommonsenseQA, SQuAD, bAbI"
Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "  1. Run: .\benchmarks\scripts\verify_datasets.ps1"
Write-Host "  2. Run: cargo run --bin spatialvortex-eval --release --features gpu -- --tasks all"
