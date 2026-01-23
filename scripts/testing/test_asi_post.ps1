$body = @{
    text = "Test"
    mode = "fast"
} | ConvertTo-Json

Write-Host "Testing ASI Inference Endpoint..."
Write-Host "URL: http://localhost:7000/api/v1/ml/asi/infer"
Write-Host "Body: $body"
Write-Host ""

try {
    $response = Invoke-WebRequest -Method POST `
        -Uri "http://localhost:7000/api/v1/ml/asi/infer" `
        -Headers @{"Content-Type"="application/json"} `
        -Body $body `
        -UseBasicParsing

    Write-Host "SUCCESS! Status: $($response.StatusCode)"
    Write-Host "Response:"
    $response.Content | ConvertFrom-Json | ConvertTo-Json -Depth 10
} catch {
    Write-Host "ERROR: $($_.Exception.Message)"
    if ($_.Exception.Response) {
        Write-Host "Status Code: $($_.Exception.Response.StatusCode.value__)"
    }
}
