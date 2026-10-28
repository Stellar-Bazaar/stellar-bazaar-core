# PowerShell runner script for Stellar Bazaar Soroban Event Indexer
param (
    [string]$DatabasePath = "bazaar_indexer.db",
    [string]$RpcUrl = "https://soroban-testnet.stellar.org",
    [switch]$DryRun,
    [switch]$Health,
    [switch]$Metrics
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "     Stellar Bazaar Indexer Daemon Launch Script           " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "Repo Root     : $RepoRoot" -ForegroundColor White
Write-Host "Database Path : $DatabasePath" -ForegroundColor White
Write-Host "Soroban RPC   : $RpcUrl" -ForegroundColor White

$cmdArgs = @("run", "-p", "bazaar-indexer", "--", "--db", $DatabasePath, "--rpc", $RpcUrl)

if ($DryRun) {
    $cmdArgs += "--dry-run"
}
if ($Health) {
    $cmdArgs += "--health"
}
if ($Metrics) {
    $cmdArgs += "--metrics"
}

Write-Host "`n[Launcher] Invoking: cargo $($cmdArgs -join ' ')" -ForegroundColor Yellow
& cargo @cmdArgs

if ($LASTEXITCODE -ne 0) {
    Write-Host "`n[Launcher] Daemon exited with status code: $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
} else {
    Write-Host "`n[Launcher] Process completed successfully." -ForegroundColor Green
}
