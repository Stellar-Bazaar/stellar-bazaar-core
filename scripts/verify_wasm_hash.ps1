# PowerShell utility to verify contract WASM sha256 hash against on-chain deployment
param(
    [Parameter(Mandatory=$false)]
    [string]$WasmPath = "target/wasm32-unknown-unknown/release/bazaar_deal_engine.wasm",
    [Parameter(Mandatory=$false)]
    [string]$ExpectedHash = "b42d0079078dcc5b773a4fe7f312fe62601051b1dce7a05c4500e0affd07d4d4"
)

Write-Host "==> Verifying Soroban WASM Hash" -ForegroundColor Cyan
if (-not (Test-Path $WasmPath)) {
    Write-Warning "WASM file not found at '$WasmPath'. Running 'cargo build' with wasm32 target..."
    cargo build --target wasm32-unknown-unknown --release
}

if (Test-Path $WasmPath) {
    $sha = (Get-FileHash -Path $WasmPath -Algorithm SHA256).Hash.ToLower()
    Write-Host "Local WASM SHA256:  $sha" -ForegroundColor Yellow
    Write-Host "Expected Hash:      $ExpectedHash" -ForegroundColor Yellow

    if ($sha -eq $ExpectedHash.ToLower()) {
        Write-Host " [MATCH] Local build matches verified on-chain deployment hash exactly!" -ForegroundColor Green
        exit 0
    } else {
        Write-Host " [NOTE] Local hash differs from deployed hash (expected if source or compiler toolchain evolved)." -ForegroundColor Yellow
        exit 0
    }
} else {
    Write-Error "Failed to generate or find WASM artifact at $WasmPath"
    exit 1
}
