# PowerShell utility to export deployment contract addresses to JSON and frontend .env format
param(
    [Parameter(Mandatory=$false)]
    [string]$DeploymentsPath = "deployments/testnet.json",
    [Parameter(Mandatory=$false)]
    [string]$TargetEnvPath = "../stellar-bazaar-frontend/.env.local",
    [Parameter(Mandatory=$false)]
    [string]$TargetJsonPath = "../stellar-bazaar-frontend/src/config/contract.json"
)

Write-Host "==> Exporting Stellar Bazaar Deployment Addresses" -ForegroundColor Cyan

if (-not (Test-Path $DeploymentsPath)) {
    Write-Error "Deployment file not found at $DeploymentsPath"
    exit 1
}

$deployments = Get-Content -Path $DeploymentsPath | ConvertFrom-Json
$registryId = $deployments.contracts.demand_circle_registry.contract_id
$dealEngineId = $deployments.contracts.bazaar_deal_engine.contract_id
$networkPassphrase = $deployments.network.network_passphrase
$rpcUrl = $deployments.network.rpc_url

Write-Host "Demand Circle Registry: $registryId" -ForegroundColor Yellow
Write-Host "Bazaar Deal Engine:     $dealEngineId" -ForegroundColor Yellow

# 1. Export to Frontend .env.local
$envContent = @"
VITE_STELLAR_NETWORK=TESTNET
VITE_STELLAR_RPC_URL=$rpcUrl
VITE_STELLAR_NETWORK_PASSPHRASE=$networkPassphrase
VITE_REGISTRY_CONTRACT_ID=$registryId
VITE_DEAL_ENGINE_CONTRACT_ID=$dealEngineId
"@

$envDir = Split-Path $TargetEnvPath -Parent
if (Test-Path $envDir) {
    Set-Content -Path $TargetEnvPath -Value $envContent -Force
    Write-Host " [OK] Successfully updated $TargetEnvPath" -ForegroundColor Green
}

# 2. Export to Frontend contract.json
$jsonDir = Split-Path $TargetJsonPath -Parent
if (Test-Path $jsonDir) {
    Copy-Item -Path $DeploymentsPath -Destination $TargetJsonPath -Force
    Write-Host " [OK] Successfully synced $TargetJsonPath" -ForegroundColor Green
}

Write-Host "Deployment export complete!" -ForegroundColor Green
