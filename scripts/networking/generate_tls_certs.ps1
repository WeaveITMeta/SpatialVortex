# Generate TLS certificates for WebTransport (QUIC) development
# 
# QUIC requires TLS 1.3, so we need certificates even for localhost
# This script generates self-signed certificates for development

Write-Host "🔐 Generating TLS certificates for WebTransport (QUIC)..." -ForegroundColor Cyan

# Create certs directory if it doesn't exist
$certsDir = "certs"
if (!(Test-Path $certsDir)) {
    New-Item -ItemType Directory -Path $certsDir
    Write-Host "✅ Created $certsDir directory" -ForegroundColor Green
}

# Check if openssl is available
$opensslPath = Get-Command openssl -ErrorAction SilentlyContinue

if (!$opensslPath) {
    Write-Host "❌ OpenSSL not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install OpenSSL:" -ForegroundColor Yellow
    Write-Host "  1. Download from: https://slproweb.com/products/Win32OpenSSL.html" -ForegroundColor Yellow
    Write-Host "  2. Or via Chocolatey: choco install openssl" -ForegroundColor Yellow
    Write-Host "  3. Or via Scoop: scoop install openssl" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ OpenSSL found at: $($opensslPath.Source)" -ForegroundColor Green

# Generate private key
Write-Host ""
Write-Host "📝 Generating private key..." -ForegroundColor Cyan
& openssl genrsa -out "$certsDir\key.pem" 4096 2>&1 | Out-Null

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Private key generated: $certsDir\key.pem" -ForegroundColor Green
} else {
    Write-Host "❌ Failed to generate private key" -ForegroundColor Red
    exit 1
}

# Generate certificate
Write-Host ""
Write-Host "📝 Generating self-signed certificate..." -ForegroundColor Cyan
& openssl req -x509 -new -nodes -key "$certsDir\key.pem" -sha256 -days 365 -out "$certsDir\cert.pem" `
    -subj "/C=US/ST=State/L=City/O=SpatialVortex/CN=localhost" 2>&1 | Out-Null

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Certificate generated: $certsDir\cert.pem" -ForegroundColor Green
} else {
    Write-Host "❌ Failed to generate certificate" -ForegroundColor Red
    exit 1
}

# Display certificate info
Write-Host ""
Write-Host "📋 Certificate Information:" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
& openssl x509 -in "$certsDir\cert.pem" -noout -subject -dates

Write-Host ""
Write-Host "✅ TLS certificates generated successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "📁 Files created:" -ForegroundColor Cyan
Write-Host "  • $certsDir\key.pem  (Private key - keep secure!)" -ForegroundColor White
Write-Host "  • $certsDir\cert.pem (Certificate)" -ForegroundColor White
Write-Host ""
Write-Host "⚠️  NOTE: These are self-signed certificates for DEVELOPMENT ONLY" -ForegroundColor Yellow
Write-Host "   For production, use certificates from a trusted CA (Let's Encrypt, etc.)" -ForegroundColor Yellow
Write-Host ""
Write-Host "🚀 You can now start the WebTransport server:" -ForegroundColor Cyan
Write-Host "   cargo run --features transport --bin webtransport_server" -ForegroundColor White
