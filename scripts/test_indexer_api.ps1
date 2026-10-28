# PowerShell validation script for Stellar Bazaar Indexer REST API
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "     Stellar Bazaar Indexer REST API Test Suite            " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

Write-Host "`n[1/2] Running indexer test suite (HTTP server, router, controller, SQLite)..." -ForegroundColor Yellow
cargo test -p bazaar-indexer -- --nocapture

if ($LASTEXITCODE -ne 0) {
    Write-Host "`n[FAIL] Indexer test suite failed with status code $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "`n[2/2] Verifying indexer CLI dry-run execution..." -ForegroundColor Yellow
cargo run -p bazaar-indexer -- --dry-run --db :memory:

if ($LASTEXITCODE -ne 0) {
    Write-Host "`n[FAIL] Indexer dry-run execution failed with status code $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "`n============================================================" -ForegroundColor Green
Write-Host "All Indexer REST API tests and routing verifications passed!" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
