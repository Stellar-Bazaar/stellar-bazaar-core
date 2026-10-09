# Stellar Bazaar Core (`stellar-bazaar-core`)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Soroban](https://img.shields.io/badge/Soroban-Protocol_22-green.svg)](https://soroban.stellar.org)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org)
[![Stellar](https://img.shields.io/badge/Stellar-Testnet-black.svg)](https://stellar.org)

The core architecture, Soroban smart contracts, deployment automation, and event-indexing abstractions for **Stellar Bazaar** — a demand-driven marketplace on the Stellar network.

---

## Architecture & Smart Contracts

```
                      +-----------------------------+
                      |   stellar-bazaar-frontend   |
                      | (Multi-Wallet / Soroban RPC)|
                      +--------------+--------------+
                                     |
                         Simulate / Submit / Events
                                     |
                                     v
                      +-----------------------------+
                      |   Stellar Soroban Testnet   |
                      |   DemandCircleRegistry      |
                      |  (Contract ID: CCBYRM...)   |
                      +--------------+--------------+
                                     |
                        Contract Events & Ledgers
                                     |
                                     v
                      +-----------------------------+
                      |      bazaar-indexer         |
                      | (Idempotent Event Consumer) |
                      +--------------+--------------+
                                     |
                                     v
                      +-----------------------------+
                      |     PostgreSQL / SQLite     |
                      |      Derived Index Cache    |
                      +-----------------------------+
```

### 1. `contracts/demand_circle_registry`
The authoritative registry smart contract for Demand Circles on Soroban.
- **Contract Methods**:
  - `initialize(admin: Address)`: Establishes administrative ownership and counters.
  - `create_circle(creator, title, metadata_uri, target_quantity, target_price_stroops, duration_seconds) -> u64`: Validates commercial rules, reserves circle ID, and emits structured `Created` event.
  - `get_circle(id: u64) -> DemandCircle`: Authoritative on-chain lookup of circle parameters and status.
  - `get_circle_count() -> u64`: Total number of registered demand circles.
  - `close_circle(id: u64)`: Allows creator or admin to close circle when rules permit.
  - `expire_circle(id: u64)`: Transitions circle to `Expired` state once the ledger deadline timestamp has elapsed.
- **Security & Authorization**:
  - `creator.require_auth()` enforced on circle creation and closing.
  - Input boundary validation (1–64 char title, quantity > 0, price > 0, duration between 60s and 365 days).
  - Minimal state stored on-chain; descriptions and off-chain assets referenced by URI.

### 2. `contracts/bazaar_deal_engine`
Soroban smart contract managing milestone settlement, escrowed buyer deposits, and seller quote fulfillment.

### 3. `crates/bazaar_indexer`
Event ingestion abstraction with replay safety, cursor tracking, and idempotent database updates.

### 4. `migrations/`
Relational schema (`001_initial_schema.sql`) for PostgreSQL/SQLite derived indexing.

---

## Live Stellar Testnet Deployment

The `DemandCircleRegistry` contract has been compiled, optimized, deployed, and verified on Stellar Testnet:

| Property | Value |
| :--- | :--- |
| **Contract Name** | `DemandCircleRegistry` |
| **Network** | Stellar Testnet (`Test SDF Network ; September 2015`) |
| **Soroban RPC** | `https://soroban-testnet.stellar.org` |
| **Deployed Contract ID** | [`CCBYRME7BW3IPB7F64D5A3NQ3N6QHAPO4NO3N3MFAIJJYK5TUMTCMEII`](https://stellar.expert/explorer/testnet/contract/CCBYRME7BW3IPB7F64D5A3NQ3N6QHAPO4NO3N3MFAIJJYK5TUMTCMEII) |
| **WASM Hash** | `94eaa4d8f3b07aa50fd4362f64b3d614b27fc91ba4e69f143a02f7d5e5d8c3d4` |
| **Deployer Public Key** | `GDFY45PPNZP4RHRYX7F57ZUL6YXVA4D2RD6S4UV6IDK3UWH56VIDF6QP` |
| **Deployment Tx Hash** | [`1c3a2dc05721f4f94c44ca693415130bd294c9dd7783578459b0de6c3fda2eba`](https://stellar.expert/explorer/testnet/tx/1c3a2dc05721f4f94c44ca693415130bd294c9dd7783578459b0de6c3fda2eba) |
| **Initialization Tx Hash**| [`e8e36cbbb61761a232662791350307d345f627d627f62e47e0446e253e81f584`](https://stellar.expert/explorer/testnet/tx/e8e36cbbb61761a232662791350307d345f627d627f62e47e0446e253e81f584) |
| **Sample Circle #1 Tx** | [`e45b72348680d67113019b544744b2cd85cf32a874b15dc71506067f87e094c8`](https://stellar.expert/explorer/testnet/tx/e45b72348680d67113019b544744b2cd85cf32a874b15dc71506067f87e094c8) |

Deployment record saved in `deployments/testnet.json`.

---

## Reproducible Build & Deployment Workflow

### Prerequisites
- **Rust**: `1.80.0` or higher (`rustup target add wasm32-unknown-unknown`)
- **Node.js**: v18.0.0 or higher
- **binaryen**: For strict WebAssembly MVP canonicalization

### 1. Build WASM Artifact
```bash
cargo rustc -p demand-circle-registry --target wasm32-unknown-unknown --release --crate-type=cdylib
```

### 2. Optimize Bytecode to Strict MVP
```bash
node scripts/optimize-wasm.mjs
```
*Note: Sets `Features.MVP` and canonicalizes `call_indirect` table immediate bytes to ensure 100% compliance with Soroban VM host validator.*

### 3. Deploy and Verify on Testnet
```bash
node scripts/deploy-contract.cjs
```
This script:
1. Loads or generates a funded Testnet deployer account via Friendbot.
2. Uploads the WASM bytecode if not yet installed.
3. Deploys the custom contract instance and records the contract ID.
4. Initializes administrative ownership.
5. Verifies the on-chain interface (`get_circle_count`).
6. Creates an initial verified Demand Circle and reads back the persisted state.
7. Outputs `deployments/testnet.json` and syncs with the frontend.

---

## Testing

Run all unit tests across the workspace:

```bash
cargo test
```

### Test Coverage Summary:
- **`bazaar_deal_engine`**: 2 tests covering settlement and refund claims.
- **`bazaar_indexer`**: 1 test verifying repository lifecycles.
- **`demand_circle_registry`**: 4 unit tests:
  - `test_registry_initialization_and_creation`: Valid initialization and circle record fields.
  - `test_validation_rules_reject_invalid_inputs`: Rejection of zero quantities, negative prices, out-of-range durations, and excessive titles.
  - `test_creator_close_and_unauthorized_rejection`: Auth check preventing unauthorized callers from closing circles.
  - `test_circle_expiration_lifecycle`: Verifying deadline expiry logic.

---

## License

MIT © 2026 KingTaiwoDev
