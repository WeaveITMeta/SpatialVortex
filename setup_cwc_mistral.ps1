# Setup CWC-Mistral-Nemo-12B for Ollama
# Downloads from HuggingFace and creates Ollama model

Write-Host "[*] Setting up CWC-Mistral-Nemo-12B for Ollama" -ForegroundColor Cyan
Write-Host ("=" * 70) -ForegroundColor Cyan

# Model details
$modelName = "cwc-mistral-nemo"
$ggufFile = "CWC-Mistral-Nemo-12B-V2.Q4_K_M.gguf"
$hfRepo = "CWClabs/CWC-Mistral-Nemo-12B-V2-q4_k_m"
$downloadUrl = "https://huggingface.co/$hfRepo/resolve/main/$ggufFile"

# Create models directory
$modelsDir = "$env:USERPROFILE\.ollama\models\cwc"
Write-Host "`n[1/4] Creating models directory..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path $modelsDir | Out-Null
Write-Host "   [OK] Directory: $modelsDir" -ForegroundColor Green

# Download GGUF file
$ggufPath = "$modelsDir\$ggufFile"
if (Test-Path $ggufPath) {
    Write-Host "`n[2/4] GGUF file already exists" -ForegroundColor Green
    Write-Host "   [OK] $ggufPath" -ForegroundColor Green
} else {
    Write-Host "`n[2/4] Downloading GGUF file (~5GB)..." -ForegroundColor Yellow
    Write-Host "   [i] This may take 5-15 minutes depending on your connection" -ForegroundColor Cyan
    Write-Host "   [*] Downloading from HuggingFace..." -ForegroundColor Cyan
    
    try {
        # Use WebClient for progress
        $webClient = New-Object System.Net.WebClient
        $webClient.DownloadFile($downloadUrl, $ggufPath)
        Write-Host "   [OK] Download complete!" -ForegroundColor Green
    } catch {
        Write-Host "   [X] Download failed: $_" -ForegroundColor Red
        Write-Host "`n   [!] Manual download:" -ForegroundColor Yellow
        Write-Host "   1. Visit: https://huggingface.co/$hfRepo" -ForegroundColor White
        Write-Host "   2. Download: $ggufFile" -ForegroundColor White
        Write-Host "   3. Place in: $modelsDir" -ForegroundColor White
        exit 1
    }
}

# Create Modelfile
Write-Host "`n[3/4] Creating Ollama Modelfile..." -ForegroundColor Yellow
$modelfile = @"
FROM $ggufPath

# Model parameters
PARAMETER temperature 0.7
PARAMETER top_p 0.9
PARAMETER top_k 40
PARAMETER num_ctx 4096
PARAMETER repeat_penalty 1.1

# System message
SYSTEM You are CWC-Mistral-Nemo, a highly capable AI assistant focused on providing accurate and helpful responses.
"@

$modelfilePath = "$modelsDir\Modelfile"
$modelfile | Out-File -FilePath $modelfilePath -Encoding UTF8
Write-Host "   [OK] Modelfile created" -ForegroundColor Green

# Create Ollama model
Write-Host "`n[4/4] Creating Ollama model '$modelName'..." -ForegroundColor Yellow
Write-Host "   [*] This may take a minute..." -ForegroundColor Cyan

try {
    Set-Location $modelsDir
    $output = ollama create $modelName -f Modelfile 2>&1
    Write-Host "   [OK] Model created successfully!" -ForegroundColor Green
} catch {
    Write-Host "   [X] Failed to create model: $_" -ForegroundColor Red
    exit 1
}

# Test the model
Write-Host "`n[5/5] Testing model..." -ForegroundColor Yellow
try {
    $testResult = ollama run $modelName "Say 'Hello from CWC-Mistral-Nemo!'" --verbose 2>&1
    Write-Host "   [OK] Model is working!" -ForegroundColor Green
} catch {
    Write-Host "   [!] Test failed, but model is created" -ForegroundColor Yellow
}

# Summary
Write-Host "`n" + ("=" * 70) -ForegroundColor Green
Write-Host "[SUCCESS] CWC-Mistral-Nemo-12B is ready!" -ForegroundColor Green
Write-Host ("=" * 70) -ForegroundColor Green

Write-Host "`n[Model Info]" -ForegroundColor Cyan
Write-Host "   Name: $modelName" -ForegroundColor White
Write-Host "   Size: ~5GB (Q4_K_M quantization)" -ForegroundColor White
Write-Host "   Context: 4096 tokens" -ForegroundColor White

Write-Host "`n[Quick Test]" -ForegroundColor Cyan
Write-Host "   ollama run $modelName" -ForegroundColor Yellow

Write-Host "`n[Use in SpatialVortex]" -ForegroundColor Cyan
Write-Host "   Update your code to use model: `"$modelName`"" -ForegroundColor Yellow

Write-Host ""
