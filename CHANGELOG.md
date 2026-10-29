# Changelog

All notable changes to the **Stellar Bazaar Core** protocol, smart contracts, indexer, and SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] - 2026-10-29

### Major Protocol & Infrastructure Transformation (Wave Program Readiness)

This release delivers **300 high-impact improvements** across smart contracts, indexer infrastructure, developer tooling, and security auditing, increasing verified automated tests from 43 to **319 system tests** (100% pass rate).

### Added

#### Soroban Smart Contracts (Protocol 22)
- **Cross-Contract Registry Deal Engine**: Implemented `create_deal_from_registry` executing genuine Soroban cross-contract queries to `DemandCircleRegistry.get_circle()`.
- **Native Stellar Asset Contract (SAC) Multi-Token Escrow**: Full integration with Stellar Asset Contracts holding XLM, Circle USDC, and EURC without wrapped tokens or synthetic balances.
- **Programmatic Buyer Surplus Rebates**: Implemented `claim_surplus_rebate` distributing bulk price-difference savings back to buyers when winning merchant offers beat the circle ceiling price.
- **Formal Dispute Resolution Protocol**: Added `dispute_deal` with configurable inspection countdown windows and `resolve_dispute` for binding basis-point fund partitioning by authorized arbitrators.
- **On-Chain Verifiable Seller Reputation**: Added `get_seller_reputation` tracking successful volume, settled values, and dispute/refund outcomes directly in ledger state.
- **Storage TTL Rent Bumpers**: Integrated automatic `extend_ttl` extending instance storage to 518,400 ledgers (~30 days) on every state-mutating transaction to protect against ledger archiving.

#### Relational Event Indexer & REST API (`crates/bazaar_indexer`)
- **Event Consumer**: High-throughput Rust indexer polling Soroban RPC `getEvents` with cursor-based pagination.
- **Idempotent Checkpointing**: SQLite WAL transaction-safe repository preventing duplicate commitment ingestion on batch replay or crash recovery.
- **Embedded HTTP REST Server**: 10 production endpoints (`/health`, `/metrics`, `/api/v1/circles`, `/api/v1/disputes`, `/api/v1/top-sellers`).
- **Prometheus Observability**: Native `/metrics` endpoint exporting ingestion counters, ledger lag, and query response latencies.
- **CLI Runner**: Daemon CLI binary with configurable RPC endpoints and SQLite connection pools.

#### Lossless Precision Math Crate (`crates/bazaar_math`)
- Strict 7-decimal stroop integer conversion functions (`xlm_to_stroops`, `stroops_to_xlm`).
- Integer surplus rebate arithmetic with mathematical invariant verification.
- Protocol fee basis point partitioning with strict upper bound validation (<= 1,000 bps / 10%).

#### Developer SDK (`packages/sdk` / `@stellar-bazaar/sdk`)
- First-party TypeScript SDK in strict mode with Zero `any` types.
- `TransactionBuilder` with automatic CPU/RAM fee budgeting and simulation parsing.
- `EventWatcher` client providing real-time streaming of Soroban contract events.
- Custom error hierarchy mapping Soroban contract error codes to actionable TypeScript exceptions.

#### CI/CD & Governance
- Added `Stellar Bazaar Core CI` with Rust unit/integration tests, clippy `-D warnings`, rustfmt check, and SDK Node 20/22 matrix.
- Added `Soroban Lifecycle Simulation` workflow verifying complete cooperative deal lifecycles end-to-end.
- Added `Security & Vulnerability Scanning` with Gitleaks and Cargo audit.
- Added `SECURITY.md` with threat model, asset custody boundary analysis, and 24-hour response SLA.
- Added `docs/hackathon_judging_guide.md`, `docs/demo_runbook.md`, and `docs/security_audit_report.md`.

### Changed
- Standardized all stroop values to 7-decimal integer types (`i128` / `u64`).
- Refactored all contract payout methods to follow the strict Checks-Effects-Interactions (CEI) anti-reentrancy pattern.
- Updated root `README.md` with comprehensive reviewer quick-reference tables, test breakdown matrix, and architecture diagrams.

### Fixed
- Fixed unconstrained storage growth by scoping map keys to unique circle and commitment identifiers.
- Fixed premature settlement vulnerability by enforcing strict ledger timestamp inspection window elapsed checks.
- Fixed duplicate seller offer submissions by validating unique merchant addresses per circle.

---

## [0.1.0] - 2026-10-08

### Initial Prototype
- Initial baseline smart contract prototypes for `DemandCircleRegistry` and `BazaarDealEngine`.
- Basic Soroban testnet deployment scripts.
- Initial project scaffolding.
