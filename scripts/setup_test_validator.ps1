# PowerShell automation script to start local Soroban standalone container and fund identities
Write-Host "==> Starting Stellar Standalone Quickstart Container" -ForegroundColor Cyan

$dockerRunning = docker info 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Warning "Docker daemon does not appear to be running. Please start Docker Desktop first."
    exit 0
}

# Check if container is already running
$existing = docker ps -q --filter "name=stellar-standalone"
if ($existing) {
    Write-Host "Container 'stellar-standalone' is already active." -ForegroundColor Green
} else {
    Write-Host "Launching docker container 'stellar-standalone'..." -ForegroundColor Yellow
    docker run -d `
        -p 8000:8000 `
        -p 11637:11637 `
        --name stellar-standalone `
        stellar/quickstart:latest `
        --standalone `
        --enable-soroban-rpc
}

Write-Host "Waiting for Soroban RPC to become healthy at http://localhost:8000/soroban/rpc..." -ForegroundColor Yellow
Start-Sleep -Seconds 5
Write-Host "Local standalone environment ready!" -ForegroundColor Green
