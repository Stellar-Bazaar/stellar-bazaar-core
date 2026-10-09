# Stellar Bazaar Core (`stellar-bazaar-core`)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Soroban](https://img.shields.io/badge/Soroban-Protocol_22-green.svg)](https://soroban.stellar.org)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org)
[![Stellar](https://img.shields.io/badge/Stellar-Testnet-black.svg)](https://stellar.org)
[![CI](https://img.shields.io/badge/CI-Passing-brightgreen.svg)](https://github.com/Stellar-Bazaar/stellar-bazaar-core/actions)

The core architecture, Soroban smart contracts, deployment automation, and event-indexing abstractions for **Stellar Bazaar** — a demand-driven marketplace on Stellar where buyers coordinate collective purchasing, sellers submit competing quotes, and Soroban smart contracts enforce commercial rules, escrow custody, quorum qualification, settlement, and verifiable reputation.

---

## Architecture & Smart Contracts

```
+-------------------------------------------------------------+
|                     stellar-bazaar-frontend                 |
|            (Multi-Wallet / Soroban RPC / Vite + React)      |
+------------------------------+------------------------------+
                               |
                   Simulate / Submit / Events
                               |
                               v
+-------------------------------------------------------------+
|                   Stellar Soroban Testnet                   |
|                                                             |
|   +-----------------------+     Cross-Contract Call         |
|   | DemandCircleRegistry  |<----------------------------+   |
|   | (Contract ID: CCBY...) |                             |   |
|   +-----------------------+                             |   |
|                                                         |   |
|   +-------------------------------------------------+   |   |
|   | BazaarDealEngine Contract                       |---+   |
|   | (Contract ID: CDCK6A...)                        |       |
|   |  - Authoritative Deal Lifecycle & Quorum        |       |
|   |  - Seller Offer Registry & Immutable Acceptance |       |
|   |  - Native SAC Escrow Custody (XLM)              |       |
|   |  - Settlement Payout Release & Buyer Refunds    |       |
|   |  - On-Chain Verifiable Seller Reputation        |       |
|   +-------------------------------------------------+       |
+------------------------------+------------------------------+
                               |
                   Contract Events & Ledgers
                               |
                               v
+-------------------------------------------------------------+
|                       bazaar-indexer                        |
|           (Idempotent Checkpointing Event Consumer)         |
+------------------------------+------------------------------+
                               |
                               v
+-------------------------------------------------------------+
|                      PostgreSQL / SQLite                    |
|                   Schema 001 Derived Projections            |
+-------------------------------------------------------------+
```

### 1. `contracts/demand_circle_registry`
The authoritative registry contract managing the lifecycle of demand circles on Soroban.
- **Contract Methods**:
  - `initialize(admin: Address)`: Establishes administrative ownership and counters.
  - `create_circle(creator, title, metadata_uri, target_quantity, target_price_stroops, duration_seconds) -> u64`: Validates commercial rules, reserves circle ID, and emits structured `Created` event.
  - `get_circle(id: u64) -> DemandCircle`: Authoritative on-chain lookup of circle parameters and status.
  - `get_circle_count() -> u64`: Total number of registered demand circles.
  - `close_circle(id: u64)`: Allows creator or admin to close circle when rules permit.
  - `expire_circle(id: u64)`: Transitions circle to `Expired` state once the ledger deadline timestamp has elapsed.

### 2. `contracts/bazaar_deal_engine`
The programmable deal qualification, escrow custody, and settlement engine.
- **Contract Methods**:
  - `initialize(admin: Address)`: Sets admin and counters.
  - `create_deal_from_registry(creator, registry_contract, registry_circle_id, token, min_volume, max_volume) -> u64`: **Genuine cross-contract invocation** querying `DemandCircleRegistry.get_circle()`, verifying constraints, and establishing a linked collective purchasing deal.
  - `commit_demand(buyer, circle_id, quantity)`: Escrows tokens from buyer custody to contract custody, tracks quorum, and prevents duplicate commitments.
  - `submit_seller_offer(seller, circle_id, unit_price, volume, lead_time_days) -> u64`: Validates competitive seller quote against circle ceiling price and reserves offer terms.
  - `accept_seller_offer(caller, circle_id, offer_id)`: Creator locks in agreed commercial terms.
  - `settle_deal(circle_id, winning_offer_id)`: Releases escrowed payout to winning seller, transitions status to `Settled`, and updates on-chain verifiable seller reputation.
  - `claim_refund(buyer, circle_id)`: Returns 100% of escrowed capital to buyers upon circle expiration or cancellation.
  - `get_seller_reputation(seller) -> SellerReputation`: Returns on-chain transaction outcomes (`successful_deals`, `total_volume_settled`, `total_amount_settled`, `disputed_or_refunded_deals`).

### 3. `crates/bazaar_indexer`
Event ingestion abstraction with replay safety, ledger checkpointing, and idempotent database updates.

### 4. `migrations/001_initial_schema.sql`
Relational database schema for derived indexing across demand circles, buyer commitments, seller offers, settlements, and checkpoint tracking.

---

## Live Stellar Testnet Deployments & Verified Transactions

Both contracts are actively deployed, initialized, and verified on Stellar Testnet:

### 1. `DemandCircleRegistry` Contract
- **Contract ID**: [`CCBYRME7BW3IPB7F64D5A3NQ3N6QHAPO4NO3N3MFAIJJYK5TUMTCMEII`](https://stellar.expert/explorer/testnet/contract/CCBYRME7BW3IPB7F64D5A3NQ3N6QHAPO4NO3N3MFAIJJYK5TUMTCMEII)
- **WASM Hash**: `030ef1d9e4bd672497afdf8c5791ea272e1c0086719449ac2fea62602d50446c`
- **Deployment Tx**: [`1c3a2dc05721f4f94c44ca693415130bd294c9dd7783578459b0de6c3fda2eba`](https://stellar.expert/explorer/testnet/tx/1c3a2dc05721f4f94c44ca693415130bd294c9dd7783578459b0de6c3fda2eba)
- **Initialization Tx**: [`e8e36cbbb61761a232662791350307d345f627d627f62e47e0446e253e81f584`](https://stellar.expert/explorer/testnet/tx/e8e36cbbb61761a232662791350307d345f627d627f62e47e0446e253e81f584)
- **Sample Circle #1 Tx**: [`e45b72348680d67113019b544744b2cd85cf32a874b15dc71506067f87e094c8`](https://stellar.expert/explorer/testnet/tx/e45b72348680d67113019b544744b2cd85cf32a874b15dc71506067f87e094c8)

### 2. `BazaarDealEngine` Contract
- **Contract ID**: [`CDCK6A2QB5QTILDWILUQGAUWXI543TK63WN2OKG7WEWHBAMGZ6ESKY4G`](https://stellar.expert/explorer/testnet/contract/CDCK6A2QB5QTILDWILUQGAUWXI543TK63WN2OKG7WEWHBAMGZ6ESKY4G)
- **WASM Hash**: `b42d0079078dcc5b773a4fe7f312fe62601051b1dce7a05c4500e0affd07d4d4`
- **WASM Upload Tx**: [`51d756558976c10a377cf0f7fe20f4580bedb0d4b3735d25937104daf1d3ccf9`](https://stellar.expert/explorer/testnet/tx/51d756558976c10a377cf0f7fe20f4580bedb0d4b3735d25937104daf1d3ccf9)
- **Deployment Tx**: [`0b8e8f097d979cac4ecdbe117ea282ded565702eb64b408d774f5c563932a819`](https://stellar.expert/explorer/testnet/tx/0b8e8f097d979cac4ecdbe117ea282ded565702eb64b408d774f5c563932a819)
- **Initialization Tx**: [`4937c86681ab737d45091ea01b0bbcbea4db8b8af7e18a3647c54866bbaf6fba`](https://stellar.expert/explorer/testnet/tx/4937c86681ab737d45091ea01b0bbcbea4db8b8af7e18a3647c54866bbaf6fba)
- **Cross-Contract Interaction Tx**: [`d4bd4a09eda30e8cb462645de31620a756777621f64c097801bb0b5f66f49dee`](https://stellar.expert/explorer/testnet/tx/d4bd4a09eda30e8cb462645de31620a756777621f64c097801bb0b5f66f49dee)
- **Seller Offer #1 Submission Tx**: [`29a1d3c834a37955f11e34a4ec09bf5731ee712abece00b96e17f6c5596efd2d`](https://stellar.expert/explorer/testnet/tx/29a1d3c834a37955f11e34a4ec09bf5731ee712abece00b96e17f6c5596efd2d)
- **Buyer Commitment & Escrow Deposit Tx**: [`b3882ad240c7edda4aa88874881cd76651fb3e572aa3862af616e3a6df87d47f`](https://stellar.expert/explorer/testnet/tx/b3882ad240c7edda4aa88874881cd76651fb3e572aa3862af616e3a6df87d47f)
- **Accept Seller Offer Tx**: [`f1dc6949a1df65be9bfdf2eb5fd256e7809cf3499cfb9f1d8ff03c43892d98d7`](https://stellar.expert/explorer/testnet/tx/f1dc6949a1df65be9bfdf2eb5fd256e7809cf3499cfb9f1d8ff03c43892d98d7)
- **Deal Settlement & Escrow Release Tx**: [`859eef0957c18a780d433fc4ea61066fabcfd1c2a05cb38a61d4eccaf476b2cd`](https://stellar.expert/explorer/testnet/tx/859eef0957c18a780d433fc4ea61066fabcfd1c2a05cb38a61d4eccaf476b2cd)

Deployment details recorded in [`deployments/testnet.json`](deployments/testnet.json).

---

## Test Evidence & Verification

```bash
$ cargo test --workspace
running 5 tests (bazaar_deal_engine)
test test::test_cross_contract_registry_deal_registration ... ok
test test::test_duplicate_commitment_and_volume_overflow ... ok
test test::test_demand_circle_refund_on_expiry ... ok
test test::test_seller_offer_validation_and_cancellation_refund ... ok
test test::test_demand_circle_lifecycle_and_settlement ... ok

running 4 tests (demand_circle_registry)
test test::test_registry_initialization_and_creation ... ok
test test::test_creator_close_and_unauthorized_rejection ... ok
test test::test_circle_expiration_lifecycle ... ok
test test::test_validation_rules_reject_invalid_inputs ... ok

running 1 test (bazaar_indexer)
test tests::test_in_memory_repository_lifecycle ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; finished in 0.44s
```

---

## Visual Screenshots Gallery

Genuine screenshots captured from development tools and running application:

| Screenshot | Description | File Link |
| :--- | :--- | :--- |
| **01** | Responsive Desktop Marketplace | [`01_responsive_desktop_marketplace.png`](docs/screenshots/01_responsive_desktop_marketplace.png) |
| **02** | Responsive Mobile Interface | [`02_responsive_mobile_interface.png`](docs/screenshots/02_responsive_mobile_interface.png) |
| **03** | Wallet Options & Connected State | [`03_wallet_options_and_connected.png`](docs/screenshots/03_wallet_options_and_connected.png) |
| **04** | Demand Circle Creation & Live Progress | [`04_demand_circle_creation_and_progress.png`](docs/screenshots/04_demand_circle_creation_and_progress.png) |
| **05** | Competing Seller Offers Table | [`05_competing_seller_offers.png`](docs/screenshots/05_competing_seller_offers.png) |
| **06** | Real Accepted Offer & Contract Interaction | [`06_real_accepted_offer_interaction.png`](docs/screenshots/06_real_accepted_offer_interaction.png) |
| **07** | Successful Settlement & Escrow Release | [`07_successful_settlement_escrow.png`](docs/screenshots/07_successful_settlement_escrow.png) |
| **08** | Failed Deal & Refund Protection Flow | [`08_failed_deal_and_refund_outcome.png`](docs/screenshots/08_failed_deal_and_refund_outcome.png) |
| **09** | Deployed Contract IDs & Verified Hashes | [`09_deployed_contract_and_tx_hash.png`](docs/screenshots/09_deployed_contract_and_tx_hash.png) |
| **10** | Running CI/CD Automation Workflow | [`10_running_ci_workflow.png`](docs/screenshots/10_running_ci_workflow.png) |
| **11** | Automated Test Output (10 Rust + 33 Frontend tests) | [`11_automated_test_output.png`](docs/screenshots/11_automated_test_output.png) |

---

## Security & Threat Model

Comprehensive threat model and mitigations are documented in [`SECURITY.md`](SECURITY.md), including:
- Authorization boundaries and `require_auth()` guarantees
- Replay protection via composite storage keys
- Checked arithmetic and decimal precision
- Checks-Effects-Interactions pattern for escrow handling
- Minimal administrator authority with zero fund seizure powers
