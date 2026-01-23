$endpoints = @(
    "/api/v1/health",
    "/api/v1/ml/embed",
    "/api/v1/ml/asi/infer",
    "/api/v1/ml/asi/metrics",
    "/api/v1/ml/asi/weights",
    "/api/v1/chat/text"
)

foreach ($endpoint in $endpoints) {
    try {
        $response = Invoke-WebRequest -Method GET -Uri "http://localhost:7000$endpoint" -ErrorAction SilentlyContinue
        Write-Host "OK $endpoint - Status: $($response.StatusCode)"
    } catch {
        if ($_.Exception.Response.StatusCode.value__ -eq 405) {
            Write-Host "WARN $endpoint - Method Not Allowed (try POST)"
        } elseif ($_.Exception.Response.StatusCode.value__ -eq 404) {
            Write-Host "FAIL $endpoint - Not Found"
        } else {
            Write-Host "ERR $endpoint - $($_.Exception.Message)"
        }
    }
}
