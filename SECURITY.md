# Stellar Bazaar Security & Threat Model

This document outlines the security architecture, threat model, trust boundaries, and operational failure mitigations for the **Stellar Bazaar** collective purchasing protocol and decentralized marketplace.

---

## 1. Architectural Trust Boundaries & Overview

Stellar Bazaar operates across three distinct security domains:
1. **On-Chain Soroban Execution Layer**: Authoritative state, escrow custody, quorum qualification, and deterministic state transitions.
2. **Indexing & Application Service Layer**: Off-chain ledger ingestion, projection caching, query acceleration, and notification indexing.
3. **Client / Browser Domain**: User interface, wallet key management (Freighter, xBull, Hana, Albedo), and transaction assembly.

```
+-------------------------------------------------------------+
|                     Client / Wallet Domain                  |
|  - Freighter / xBull / Albedo / Companion Signers           |
|  - Local Secret Storage (Hardware/Enclave - zero server key)|
+------------------------------+------------------------------+
                               | Signs Transaction XDR
                               v
+-------------------------------------------------------------+
|               Stellar Network & Soroban Host                |
|  DemandCircleRegistry   <--->   BazaarDealEngine Contract   |
|  - Circle Constraints           - Quorum Qualification      |
|  - Expiration Bounds            - Native SAC Escrow Custody |
|  - Cross-Contract Call          - Settlement Payout Release |
|                                 - Buyer Refund Guarantees   |
+------------------------------+------------------------------+
                               | Event Stream Ingestion
                               v
+-------------------------------------------------------------+
|             Backend & Event Indexer Domain                  |
|  - Idempotent Checkpointing (Ledger Sequences)              |
|  - Postgres Projection Database (Read-Only Cache)           |
+-------------------------------------------------------------+
```

---

## 2. Comprehensive Threat Matrix & Mitigations

### 2.1 Unauthorized Access and Contract Calls
- **Threat**: Malicious actors invoking privileged endpoints (e.g. canceling circles, claiming others' refunds, settling deals without authorization).
- **Mitigation**:
  - Strict Soroban `require_auth()` checks on every mutating function.
  - Creator authentication: Only the recorded `circle.creator` can cancel a circle or accept an offer.
  - Buyer authentication: In `commit_demand` and `claim_refund`, `buyer.require_auth()` ensures only the legitimate account holder commits funds or receives refunds.
  - Seller authentication: `seller.require_auth()` prevents unauthorized quotes on behalf of arbitrary addresses.

### 2.2 Duplicate Transactions & Replay-Like Behavior
- **Threat**: Replaying signed commitment transactions or attempting duplicate participation to manipulate volume.
- **Mitigation**:
  - Stellar accounts employ incrementing sequence numbers enforced at the consensus layer, rendering transaction replays invalid.
  - Soroban state storage uses explicit composite keys: `DataKey::Commitment(circle_id, buyer)`.
  - The contract strictly asserts that the buyer does not already have an active commitment in the circle (`Error::AlreadyCommitted`).

### 2.3 Incorrect Deadline or Threshold Evaluation
- **Threat**: Late commitments accepted after expiry; premature settlement before quorum or before expiration.
- **Mitigation**:
  - All time checks evaluate against authoritative Soroban host time: `env.ledger().timestamp()`.
  - `commit_demand` and `submit_seller_offer` assert `timestamp <= circle.deadline` (`Error::DeadlinePassed`).
  - `claim_refund` asserts `timestamp > circle.deadline || status == Cancelled` (`Error::CircleAlreadyClosed`).
  - Quorum threshold evaluation: `settle_deal` strictly verifies `circle.current_volume >= circle.min_volume` (`Error::QuorumNotReached`).

### 2.4 Double Spending, Double Release, and Double Refund
- **Threat**: Withdrawing escrowed capital multiple times; claiming a refund after settlement.
- **Mitigation**:
  - Atomic state transitions: `Commitment.refunded` is set to `true` prior to or atomically with the token transfer.
  - Double refund check: `if commitment.refunded { return Err(Error::AlreadyRefunded); }`.
  - Deal settlement transitions `circle.status` to `CircleStatus::Settled`. Once settled, `claim_refund` unconditionally rejects withdrawal attempts.

### 2.5 Malicious or Expired Seller Offers
- **Threat**: Sellers submitting offers exceeding buyer budget constraints or failing to deliver.
- **Mitigation**:
  - `submit_seller_offer` validates `unit_price <= circle.target_unit_price` (`Error::PriceExceedsTarget`).
  - Volume matching: `settle_deal` asserts `offer.volume >= circle.current_volume` (`Error::InvalidVolume`).
  - Immutable accepted terms: Once accepted by the creator, commercial terms cannot be mutated unilaterally.

### 2.6 Invalid Asset Configuration
- **Threat**: Callers supplying spoofed token contract addresses or unbacked mock tokens.
- **Mitigation**:
  - The protocol uses the Stellar Asset Contract (SAC) for native XLM (`CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` on Testnet).
  - Cross-contract deal registration (`create_deal_from_registry`) binds asset addresses to verified on-chain deployments.

### 2.7 Reentrancy & Cross-Contract Risk
- **Threat**: External contract callbacks attempting reentrant state manipulation.
- **Mitigation**:
  - The Soroban runtime execution model prohibits arbitrary reentrancy.
  - Follows Checks-Effects-Interactions: Storage mutations and status transitions precede external token client transfers.

### 2.8 Administrative & Dispute Powers
- **Threat**: Centralized administrators arbitrarily seizing or redirecting buyer escrow funds.
- **Mitigation**:
  - Minimal trusted authority: Contract administrator cannot withdraw escrow capital or modify winning payout parameters.
  - All fund flows are programmatic: escrow is released strictly to the winning seller or refunded 100% to buyers.

### 2.9 Indexer Outages and Stale Backend State
- **Threat**: Frontend showing stale or out-of-sync state due to indexer lag.
- **Mitigation**:
  - The frontend reads critical financial state directly from the authoritative Soroban contract RPC (`get_circle`, `get_offer`).
  - Event indexers maintain explicit checkpoints (`indexer_checkpoints` table with `last_processed_ledger`) and process events idempotently.

### 2.10 Frontend Transaction Spoofing
- **Threat**: Malicious client-side tampering attempting to broadcast unsigned transactions.
- **Mitigation**:
  - Transactions are simulated against the Soroban RPC before presentation to the user's wallet for signature.
  - The wallet displays full transaction details (destination address, function arguments, resource fees) before user confirmation.

### 2.11 Private Key, Wallet, and Secret Hygiene
- **Threat**: Secret leakage in logs, repository commits, or client-side bundles.
- **Mitigation**:
  - No private keys or secrets are committed or included in client code.
  - Deployment keys reside in ignored `.deployer.json` or environment variables (`STELLAR_DEPLOYER_SECRET`).
  - CI workflows utilize ephemeral test credentials without production access.

---

## 3. Incident Response & Responsible Disclosure

To report a security vulnerability or anomalous contract execution:
- Contact: `security@stellarbazaar.io`
- Include: Detailed reproduction steps, transaction hashes, and affected contract addresses.
