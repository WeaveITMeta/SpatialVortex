# Test script for SpatialVortex Chat API
# Run this after starting the server

Write-Host "🧪 Testing SpatialVortex Chat API..." -ForegroundColor Cyan
Write-Host ""

# Test 1: Health check
Write-Host "1️⃣  Testing health endpoint..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "http://localhost:7000/api/v1/health" -Method Get
    Write-Host "✅ Health check passed!" -ForegroundColor Green
    Write-Host "   Status: $($health.status)" -ForegroundColor Gray
} catch {
    Write-Host "❌ Health check failed: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Test 2: Chat endpoint
Write-Host "2️⃣  Testing chat endpoint..." -ForegroundColor Yellow

$chatRequest = @{
    message = "What is consciousness?"
    user_id = "test_user_123"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "http://localhost:7000/api/v1/chat/text" `
        -Method Post `
        -ContentType "application/json" `
        -Body $chatRequest
    
    Write-Host "✅ Chat endpoint working!" -ForegroundColor Green
    Write-Host ""
    Write-Host "📊 Response Details:" -ForegroundColor Cyan
    Write-Host "   Response: $($response.response)" -ForegroundColor White
    Write-Host ""
    Write-Host "   ELP Values:" -ForegroundColor Cyan
    Write-Host "   - Ethos:  $($response.elp_values.ethos)" -ForegroundColor White
    Write-Host "   - Logos:  $($response.elp_values.logos)" -ForegroundColor White
    Write-Host "   - Pathos: $($response.elp_values.pathos)" -ForegroundColor White
    Write-Host ""
    Write-Host "   📈 Metrics:" -ForegroundColor Cyan
    Write-Host "   - Confidence: $($response.confidence * 100)%" -ForegroundColor White
    Write-Host "   - Flux Position:   $($response.flux_position)" -ForegroundColor White
    Write-Host "   - Confidence:      $($response.confidence * 100)%" -ForegroundColor White
    
    if ($response.subject) {
        Write-Host "   - Subject:         $($response.subject)" -ForegroundColor White
    }
    
    if ($response.processing_time_ms) {
        Write-Host "   - Processing Time: $($response.processing_time_ms)ms" -ForegroundColor White
    }
    
    # Check if sacred position
    $sacred = @(3, 6, 9)
    if ($sacred -contains $response.flux_position) {
        Write-Host ""
        Write-Host "   ✨ SACRED POSITION! Position $($response.flux_position) has geometric significance!" -ForegroundColor Magenta
    }
    
} catch {
    Write-Host "❌ Chat endpoint failed: $_" -ForegroundColor Red
    Write-Host $_.Exception.Response.StatusCode.value__ -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🎉 All tests passed! Your ASI Model is running!" -ForegroundColor Green
