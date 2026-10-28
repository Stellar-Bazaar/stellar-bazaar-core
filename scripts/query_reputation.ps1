# PowerShell script to query on-chain seller reputation from BazaarDealEngine contract
param(
    [Parameter(Mandatory=$false)]
    [string]$SellerAddress = "GDJ4HFRD4N2Q34XG5O4T3HQMUD2Z54K54K2D6WGLP24WCVXJ67VFXV6L",
    [Parameter(Mandatory=$false)]
    [string]$ContractId = "CDCK6A2QB5QTILDWILUQGAUWXI543TK63WN2OKG7WEWHBAMGZ6ESKY4G",
    [Parameter(Mandatory=$false)]
    [string]$Network = "testnet"
)

Write-Host "==> Querying On-Chain Seller Reputation" -ForegroundColor Cyan
Write-Host "Contract ID:    $ContractId" -ForegroundColor Yellow
Write-Host "Seller Address: $SellerAddress" -ForegroundColor Yellow
Write-Host "Network:        $Network" -ForegroundColor Yellow

$rpcUrl = if ($Network -eq "testnet") { "https://soroban-testnet.stellar.org" } else { "https://soroban-rpc.stellar.org" }

try {
    stellar contract invoke `
        --id $ContractId `
        --source-account testnet_admin `
        --network $Network `
        -- `
        get_seller_reputation `
        --seller $SellerAddress
} catch {
    Write-Warning "Stellar CLI invoke completed or returned simulation output. Inspect details above."
}
