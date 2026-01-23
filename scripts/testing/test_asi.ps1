$body = @{
    text = "What is consciousness?"
    mode = "balanced"
} | ConvertTo-Json

$response = Invoke-WebRequest -Method POST -Uri "http://localhost:7000/api/v1/ml/asi/infer" -Headers @{"Content-Type"="application/json"} -Body $body

Write-Host "Response Status:" $response.StatusCode
Write-Host "Response Body:"
$response.Content | ConvertFrom-Json | ConvertTo-Json -Depth 10
