# Download benchmark datasets - PowerShell version for Windows
# Run with: .\benchmarks\scripts\download_datasets.ps1

$ErrorActionPreference = "Stop"

$BENCHMARK_DIR = "benchmarks\data"
New-Item -ItemType Directory -Force -Path $BENCHMARK_DIR | Out-Null

Write-Host "Downloading benchmark datasets..." -ForegroundColor Cyan

# FB15k-237 Knowledge Graph
Write-Host "`n[1/9] Downloading FB15k-237..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\fb15k237" | Out-Null
Invoke-WebRequest -Uri "https://download.microsoft.com/download/8/7/0/8700516A-AB3D-4850-B4BB-805C515AECE1/FB15K-237.2.zip" -OutFile "$BENCHMARK_DIR\fb15k237.zip"
Expand-Archive -Path "$BENCHMARK_DIR\fb15k237.zip" -DestinationPath "$BENCHMARK_DIR\fb15k237" -Force
Remove-Item "$BENCHMARK_DIR\fb15k237.zip"
Write-Host "OK FB15k-237 downloaded" -ForegroundColor Green

# WN18RR Knowledge Graph
Write-Host "`n[2/9] Downloading WN18RR..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\wn18rr" | Out-Null
Invoke-WebRequest -Uri "https://github.com/TimDettmers/ConvE/raw/master/WN18RR.tar.gz" -OutFile "$BENCHMARK_DIR\wn18rr.tar.gz"
tar -xzf "$BENCHMARK_DIR\wn18rr.tar.gz" -C "$BENCHMARK_DIR\wn18rr"
Remove-Item "$BENCHMARK_DIR\wn18rr.tar.gz"
Write-Host "OK WN18RR downloaded" -ForegroundColor Green

# STS Benchmark
Write-Host "`n[3/9] Downloading STS Benchmark..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\sts" | Out-Null
Invoke-WebRequest -Uri "http://ixa2.si.ehu.es/stswiki/images/4/48/Stsbenchmark.tar.gz" -OutFile "$BENCHMARK_DIR\stsbenchmark.tar.gz"
tar -xzf "$BENCHMARK_DIR\stsbenchmark.tar.gz" -C "$BENCHMARK_DIR\sts"
Remove-Item "$BENCHMARK_DIR\stsbenchmark.tar.gz"
Write-Host "OK STS Benchmark downloaded" -ForegroundColor Green

# SICK Dataset
Write-Host "`n[4/9] Downloading SICK..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\sick" | Out-Null
try {
    # Download from brmson/dataset-sts GitHub repo (working source)
    Invoke-WebRequest -Uri "https://raw.githubusercontent.com/brmson/dataset-sts/master/data/sts/sick2014/SICK_train.txt" -OutFile "$BENCHMARK_DIR\sick\SICK_train.txt" -UseBasicParsing
    Invoke-WebRequest -Uri "https://raw.githubusercontent.com/brmson/dataset-sts/master/data/sts/sick2014/SICK_trial.txt" -OutFile "$BENCHMARK_DIR\sick\SICK_trial.txt" -UseBasicParsing
    Invoke-WebRequest -Uri "https://raw.githubusercontent.com/brmson/dataset-sts/master/data/sts/sick2014/SICK_test_annotated.txt" -OutFile "$BENCHMARK_DIR\sick\SICK_test.txt" -UseBasicParsing
    Write-Host "OK SICK downloaded" -ForegroundColor Green
} catch {
    Write-Host "WARN: SICK download failed, skipping..." -ForegroundColor Yellow
    "# SICK dataset unavailable" | Out-File "$BENCHMARK_DIR\sick\README.txt"
}

# SQuAD 2.0
Write-Host "`n[5/9] Downloading SQuAD 2.0..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\squad" | Out-Null
Invoke-WebRequest -Uri "https://rajpurkar.github.io/SQuAD-explorer/dataset/dev-v2.0.json" -OutFile "$BENCHMARK_DIR\squad\dev-v2.0.json"
Write-Host "OK SQuAD 2.0 downloaded" -ForegroundColor Green

# CommonsenseQA
Write-Host "`n[6/9] Downloading CommonsenseQA..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\commonsenseqa" | Out-Null
Invoke-WebRequest -Uri "https://s3.amazonaws.com/commensenseqa/dev_rand_split.jsonl" -OutFile "$BENCHMARK_DIR\commonsenseqa\dev.jsonl"
Write-Host "OK CommonsenseQA downloaded" -ForegroundColor Green

# bAbI Tasks
Write-Host "`n[7/9] Downloading bAbI..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\babi" | Out-Null
Invoke-WebRequest -Uri "http://www.thespermwhale.com/jaseweston/babi/tasks_1-20_v1-2.tar.gz" -OutFile "$BENCHMARK_DIR\babi.tar.gz"
tar -xzf "$BENCHMARK_DIR\babi.tar.gz" -C "$BENCHMARK_DIR\babi"
Remove-Item "$BENCHMARK_DIR\babi.tar.gz"
Write-Host "OK bAbI downloaded" -ForegroundColor Green

# CLUTRR
Write-Host "`n[8/9] Downloading CLUTRR..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\clutrr" | Out-Null
try {
    # Try downloading pre-generated data from HuggingFace
    Invoke-WebRequest -Uri "https://huggingface.co/datasets/CLUTRR/v1/raw/main/data_089907f8/test.csv" -OutFile "$BENCHMARK_DIR\clutrr\test.csv" -UseBasicParsing -ErrorAction Stop
    Write-Host "OK CLUTRR downloaded" -ForegroundColor Green
} catch {
    Write-Host "SKIP CLUTRR (requires generation - see https://github.com/facebookresearch/clutrr)" -ForegroundColor Yellow
    "# CLUTRR requires generation, not a static file" | Out-File "$BENCHMARK_DIR\clutrr\README.txt"
}

# Silesia Corpus
Write-Host "`n[9/9] Downloading Silesia Corpus..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "$BENCHMARK_DIR\silesia" | Out-Null
Invoke-WebRequest -Uri "http://sun.aei.polsl.pl/~sdeor/corpus/silesia.zip" -OutFile "$BENCHMARK_DIR\silesia.zip"
Expand-Archive -Path "$BENCHMARK_DIR\silesia.zip" -DestinationPath "$BENCHMARK_DIR\silesia" -Force
Remove-Item "$BENCHMARK_DIR\silesia.zip"
Write-Host "OK Silesia Corpus downloaded" -ForegroundColor Green

Write-Host "`n==================================" -ForegroundColor Cyan
Write-Host "SUCCESS! All datasets downloaded" -ForegroundColor Green
Write-Host "==================================" -ForegroundColor Cyan
Write-Host "Data location: $BENCHMARK_DIR" -ForegroundColor White
Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "  1. Run: .\benchmarks\scripts\verify_datasets.ps1"
Write-Host "  2. Run: cargo test --release --package benchmarks"
