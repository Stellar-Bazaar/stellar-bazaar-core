## Summary of Changes

<!-- Briefly describe the feature, fix, or optimization introduced by this pull request. -->

## Motivation & Context

<!-- Why is this change required? What problem or hackathon/grant requirement does it solve? -->

## Type of Change

- [ ] 🌟 Smart Contract Upgrade / New Soroban Feature
- [ ] ⚡ Indexer / Relational Storage Enhancement
- [ ] 📦 SDK / API Client Improvement
- [ ] 🐛 Bug Fix
- [ ] 🛡️ Security / Auth Hardening
- [ ] 📚 Documentation / Developer Guide

---

## Stellar & Soroban Safety Checklist

Before submitting this PR, please confirm that the following safety criteria are met:

- [ ] **Lossless Stroop Arithmetic**: All token amounts and calculations preserve 7-decimal integer stroop precision (no floating-point rounding errors).
- [ ] **Authorization Verification**: Every mutating invocation explicitly checks `caller.require_auth()`.
- [ ] **Check-Effects-Interactions (CEI)**: Internal state updates occur strictly prior to token transfers or cross-contract invocations.
- [ ] **State Storage TTL**: Instance and persistent entries extend TTL appropriately (`INSTANCE_TTL_EXTEND_TO`).
- [ ] **Fee Bound Constraints**: Protocol fees strictly adhere to maximum limit $\le 10\%$ (1,000 bps).
- [ ] **Event Emission**: All lifecycle transitions emit structured Soroban topics (`circle_created`, `deal_settled`, `dispute_opened`, etc.).
- [ ] **WASM Size Budget**: Compiled contracts stay comfortably below the 256KB threshold.
- [ ] **Unit & E2E Tests**: All existing tests pass (`cargo test --workspace`) and new tests have been added.
- [ ] **SDK & Indexer Sync**: Any contract interface changes have been reflected in `@stellar-bazaar/sdk` and `bazaar-indexer`.
