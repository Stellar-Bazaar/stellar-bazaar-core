# Developer FAQ & Troubleshooting Guide

Frequently asked questions and operational troubleshooting tips for building on **Stellar Bazaar**.

---

## 1. Frequently Asked Questions

### Q: How do I obtain Testnet XLM or USDC for testing?
**A**:
1. **XLM**: Use the built-in "Fund with Friendbot" button in the dApp header or visit the [Stellar Laboratory Friendbot](https://laboratory.stellar.org/#account-creator?network=test).
2. **USDC**: Visit the [Stellar Testnet USDC Faucet](https://laboratory.stellar.org) and establish a trustline to Circle's Testnet issuing address, or interact directly with the Testnet SAC (`CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`).

### Q: Why did my contract invocation fail with code 8?
**A**: Error code `8` corresponds to `CeilingPriceExceeded`. The seller offer unit price was higher than the maximum ceiling price specified by the circle creator. Lower the quoted unit price to $\le \text{ceiling\_price}$.

### Q: What happens if a buying circle expires before meeting quorum?
**A**: Buyers are 100% protected. Any participant can invoke `claim_refund` (or click "Claim Refund" in the UI) to immediately withdraw all escrowed funds from the smart contract back to their wallet.

### Q: How does the surplus rebate mechanism work?
**A**: When a seller quotes a unit price lower than the circle's ceiling price (e.g. 40 XLM vs 50 XLM ceiling), the difference (10 XLM per unit) forms a surplus pool. Upon settlement, buyers claim their proportional share (`10 XLM * units`), recovering their discount without leaving funds locked in escrow.

---

## 2. Common Troubleshooting Steps

### 1. Freighter Connection Not Detected
- **Issue**: "Freighter wallet is not installed or extension is locked."
- **Remedy**:
  - Open the Freighter browser extension and unlock it.
  - Verify network is set to **Testnet** (Settings -> Network -> Testnet).
  - Refresh the dApp page.

### 2. Transaction Simulation Reverted
- **Issue**: `SimulationFailed: HostError`
- **Remedy**:
  - Check account balance to ensure sufficient funds exist for both the escrow commitment and base transaction fee (~100 stroops).
  - Verify that the calling account has signed the required authorization entry (`caller.require_auth()`).

### 3. Indexer Lag or Database Locked Error
- **Issue**: `bazaar-indexer: SQLite busy / database locked`
- **Remedy**:
  - Ensure the SQLite database operates in **WAL (Write-Ahead Logging)** mode. Our schema automatically issues `PRAGMA journal_mode=WAL;` on startup.
