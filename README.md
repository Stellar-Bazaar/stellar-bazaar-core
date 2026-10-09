# Stellar Bazaar Core (`stellar-bazaar-core`)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Soroban](https://img.shields.io/badge/Soroban-v22-blue.svg)](https://soroban.stellar.org)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org)

The core architecture, Soroban smart contracts, indexing boundaries, and database infrastructure for **Stellar Bazaar** — a demand-driven marketplace on the Stellar network.

---

## Architecture Overview

Stellar Bazaar flips traditional e-commerce by enabling buyers to aggregate demand into **Demand Circles**. When sufficient volume commitment is reached, verified sellers compete on price and fulfillment terms. Soroban smart contracts enforce custody, milestone escrow, and automated settlement.

```
                      +-----------------------------+
                      |   stellar-bazaar-frontend   |
                      |   (Freighter Wallet / UI)   |
                      +--------------+--------------+
                                     |
                         RPC / Horizon Transactions
                                     |
                                     v
                      +-----------------------------+
                      |     Stellar Testnet /       |
                      |   Soroban Smart Contracts   |
                      +--------------+--------------+
                                     |
                       Contract Events & Ledgers
                                     |
                                     v
                      +-----------------------------+
                      |      bazaar-indexer         |
                      | (Event Sync & Data Ingestion)|
                      +--------------+--------------+
                                     |
                                     v
                      +-----------------------------+
                      |     PostgreSQL / SQLite     |
                      |      Relational Schema      |
                      +-----------------------------+
```

### Components

1. **`contracts/bazaar_deal_engine`**:
   - Soroban smart contract managing Demand Circles, escrowed buyer deposits, seller quote submissions, quorum detection, deal execution, and deadline refund claims.
   - Built with `soroban-sdk` v22.0.11.
2. **`crates/bazaar_indexer`**:
   - Backend event listener and indexer abstraction (`BazaarRepository` boundary, ledger gap detection, and domain records).
3. **`migrations/`**:
   - Relational database schema (`001_initial_schema.sql`) for PostgreSQL/SQLite storing demand circles, commitments, seller bids, and settlement audit logs.

---

## Prerequisites

- **Rust**: `1.80.0` or higher (`rustup update stable`)
- **WebAssembly Target**: `wasm32-unknown-unknown` (`rustup target add wasm32-unknown-unknown`)
- **Soroban CLI** (optional for testnet deployments): `cargo install --locked stellar-cli`

---

## Environment Variables

Copy `.env.example` to `.env`:

```bash
cp .env.example .env
```

Key variables:
- `STELLAR_NETWORK`: Target network (`TESTNET` or `PUBLIC`).
- `STELLAR_RPC_URL`: Soroban RPC endpoint (`https://soroban-testnet.stellar.org`).
- `STELLAR_HORIZON_URL`: Horizon API endpoint (`https://horizon-testnet.stellar.org`).
- `STELLAR_NETWORK_PASSPHRASE`: Network passphrase (`"Test SDF Network ; September 2015"`).
- `DATABASE_URL`: PostgreSQL connection string.

---

## Development & Test Commands

### 1. Run Workspace Unit Tests
```bash
cargo test
```

### 2. Format Code & Verify Lints
```bash
cargo fmt --check
cargo check
```

### 3. Build Contract for WASM Deployment
```bash
cargo build --target wasm32-unknown-unknown --release
```

Compiled WASM artifacts will be generated at:
`target/wasm32-unknown-unknown/release/bazaar_deal_engine.wasm`

---

## Database Schema & Migrations

The database migrations are located in `migrations/`:
- `001_initial_schema.sql`: Contains tables for `demand_circles`, `buyer_commitments`, `seller_offers`, `settlements`, and `indexer_checkpoints`.

To apply to PostgreSQL:
```bash
psql $DATABASE_URL -f migrations/001_initial_schema.sql
```

---

## Security Considerations

1. **Escrow Safety**: All buyer funds are held in trust by the Soroban contract until quorum is verified and the winning seller offer is executed.
2. **Refund Guarantee**: If the reservation deadline passes without deal settlement, buyers are cryptographically guaranteed refunds through `claim_refund`.
3. **No Key Exposure**: No private keys or seed phrases are stored or logged in the repository.

---

## License

MIT License. Copyright (c) 2026 KingTaiwoDev.
