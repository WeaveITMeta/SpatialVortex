# Test SpatialVortex API endpoints

Write-Host "🧪 Testing SpatialVortex API Server" -ForegroundColor Cyan
Write-Host ""

# Test 1: Health check
Write-Host "1️⃣  Health Check..." -ForegroundColor Yellow
$health = Invoke-RestMethod -Uri "http://localhost:7000/health" -Method Get
Write-Host "   ✅ Status: $($health.status)" -ForegroundColor Green
Write-Host "   Version: $($health.version)"
Write-Host ""

# Test 2: Unified chat (text query with ThinkingAgent)
Write-Host "2️⃣  Testing ThinkingAgent (text query)..." -ForegroundColor Yellow
$chatRequest = @{
    message = "What is the meaning of consciousness?"
    user_id = "test_user"
    session_id = "test_session_001"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "http://localhost:7000/api/v1/chat/unified" -Method Post -Body $chatRequest -ContentType "application/json"
    Write-Host "   ✅ Response received" -ForegroundColor Green
    Write-Host "   Response: $($response.response.Substring(0, [Math]::Min(100, $response.response.Length)))..."
    Write-Host "   ELP - Ethos: $($response.elp_values.ethos), Logos: $($response.elp_values.logos), Pathos: $($response.elp_values.pathos)"
    Write-Host "   Flux Position: $($response.flux_position)"
    Write-Host "   Confidence: $($response.confidence)"
    Write-Host "   Generation Time: $($response.generation_time_ms)ms"
} catch {
    Write-Host "   ❌ Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 3: Safety check (PII detection)
Write-Host "3️⃣  Testing Safety Guardrails (should block PII)..." -ForegroundColor Yellow
$piiRequest = @{
    message = "My email is john@example.com"
    user_id = "test_user"
    session_id = "test_session_002"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "http://localhost:7000/api/v1/chat/unified" -Method Post -Body $piiRequest -ContentType "application/json"
    if ($response.response -like "*Safety Check Failed*") {
        Write-Host "   ✅ PII correctly blocked!" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️  PII not blocked" -ForegroundColor Yellow
    }
    Write-Host "   Response: $($response.response)"
} catch {
    Write-Host "   ❌ Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 4: Streaming chat
Write-Host "4️⃣  Testing Streaming Chat..." -ForegroundColor Yellow
Write-Host "   📡 SSE endpoint available at: http://localhost:7000/api/v1/chat/unified/stream"
Write-Host "   (Use a browser or SSE client to test streaming)"
Write-Host ""

Write-Host "✅ All tests complete!" -ForegroundColor Green
Write-Host ""
Write-Host "🌐 Frontend: Start the web server and navigate to http://localhost:3000" -ForegroundColor Cyan
Write-Host "📚 API Docs: http://localhost:7000/swagger-ui/" -ForegroundColor Cyan
