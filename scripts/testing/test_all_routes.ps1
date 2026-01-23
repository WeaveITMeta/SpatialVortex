$routes = @(
    @{Method="GET"; Path="/api/v1/health"},
    @{Method="GET"; Path="/api/v1/chat/text"},
    @{Method="POST"; Path="/api/v1/chat/text"},
    @{Method="POST"; Path="/api/v1/ml/embed"},
    @{Method="POST"; Path="/api/v1/ml/asi/infer"},
    @{Method="GET"; Path="/api/v1/ml/asi/metrics"},
    @{Method="GET"; Path="/api/v1/ml/asi/weights"},
    @{Method="GET"; Path="/api/v1/storage/confidence-lake/status"},
    @{Method="GET"; Path="/api/v1/voice/status"}
)

Write-Host "=== Testing All Routes ===" -ForegroundColor Cyan
Write-Host ""

foreach ($route in $routes) {
    $method = $route.Method
    $path = $route.Path
    
    try {
        $response = Invoke-WebRequest -Method $method -Uri "http://localhost:7000$path" -ErrorAction Stop -UseBasicParsing
        Write-Host "[OK] $method $path - Status: $($response.StatusCode)" -ForegroundColor Green
    } catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        if ($statusCode -eq 405) {
            Write-Host "[WARN] $method $path - Method Not Allowed (route exists, wrong method)" -ForegroundColor Yellow
        } elseif ($statusCode -eq 404) {
            Write-Host "[FAIL] $method $path - Not Found" -ForegroundColor Red
        } elseif ($statusCode -eq 400) {
            Write-Host "[WARN] $method $path - Bad Request (route exists, needs body)" -ForegroundColor Yellow
        } else {
            Write-Host "[ERR] $method $path - Status: $statusCode" -ForegroundColor Magenta
        }
    }
}
