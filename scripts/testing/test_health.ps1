try {
    $response = Invoke-WebRequest -Method GET -Uri "http://localhost:7000/api/v1/health"
    Write-Host "Server is running!"
    Write-Host "Status:" $response.StatusCode
    Write-Host "Response:" $response.Content
} catch {
    Write-Host "Server is NOT responding:"
    Write-Host $_.Exception.Message
}
