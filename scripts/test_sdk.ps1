<#
.SYNOPSIS
  Automated test runner for @stellar-bazaar/sdk.
.DESCRIPTION
  Executes TypeScript compile-check and Vitest test suites for the SDK package.
#>

$ErrorActionPreference = "Stop"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host " Running @stellar-bazaar/sdk Validation Suite" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

$CoreDir = Split-Path -Parent $PSScriptRoot
$SdkDir = Join-Path $CoreDir "packages\sdk"
$FrontendDir = Join-Path (Split-Path -Parent $CoreDir) "stellar-bazaar-frontend"

if (-not (Test-Path $SdkDir)) {
    Write-Error "SDK directory not found at $SdkDir"
    exit 1
}

Write-Host "`n[1/2] Running SDK Vitest Unit Tests..." -ForegroundColor Yellow
$VitestCmd = "npx vitest run --root `"$SdkDir`""
Push-Location $FrontendDir
try {
    Invoke-Expression $VitestCmd
    if ($LASTEXITCODE -ne 0) {
        throw "Vitest suite failed with exit code $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

Write-Host "`n[2/2] Checking TypeScript Compilation..." -ForegroundColor Yellow
Push-Location $SdkDir
try {
    npx tsc --noEmit
    if ($LASTEXITCODE -ne 0) {
        throw "TypeScript compilation check failed with exit code $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

Write-Host "`n============================================================" -ForegroundColor Green
Write-Host " @stellar-bazaar/sdk Tests & Compilation Passed 100%!" -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green
exit 0
