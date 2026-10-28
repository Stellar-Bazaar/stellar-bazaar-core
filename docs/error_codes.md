# Stellar Bazaar: Smart Contract Error Code Reference

This document provides a comprehensive lookup table of all on-chain error codes emitted by **Demand Circle Registry** and **Bazaar Deal Engine** Soroban smart contracts.

---

## 1. DemandCircleRegistry Errors

Contract: `contracts/demand_circle_registry`

| Code (`u32`) | Identifier | Description | Remediation |
| :--- | :--- | :--- | :--- |
| **1** | `AlreadyInitialized` | The registry contract has already been initialized with an administrative address. | Do not invoke `initialize` on an already active contract instance. |
| **2** | `NotInitialized` | Attempted operation requires contract initialization, but no admin is stored. | Run `initialize(admin)` before executing administrative operations. |
| **3** | `Unauthorized` | Caller address does not match the circle creator or authorized admin. | Sign the transaction using the creator account key. |
| **4** | `CircleNotFound` | The requested circle ID does not exist in instance storage. | Verify the circle ID against `get_circle_count()`. |
| **5** | `InvalidQuantity` | Target unit quantity must be greater than zero (`> 0`). | Provide a non-zero unit volume target. |
| **6** | `InvalidPrice` | Target unit price in Stroops must be strictly positive (`> 0`). | Provide a positive price in Stroops (1 XLM = 10,000,000 Stroops). |
| **7** | `InvalidDeadline` | Circle duration is outside the permitted bounds [60s, 31,536,000s]. | Set duration between 1 minute and 1 year. |
| **8** | `InvalidTitle` | Circle title is empty or exceeds the 64-byte limit. | Provide a concise title between 1 and 64 characters. |
| **9** | `InvalidState` | Circle is not in the required state (`Open`) to execute this transition. | Check circle status before attempting close or expiration. |
| **10** | `DeadlineNotPassed` | Expiration cannot be triggered before block timestamp exceeds deadline. | Wait until ledger timestamp is strictly greater than deadline. |
| **11** | `DeadlinePassed` | Circle duration has already elapsed. | Create a new circle or participate before deadline. |
| **12** | `InvalidMetadataUri` | Metadata URI is empty or exceeds the 256-byte limit. | Provide a standard IPFS content URI (e.g. `ipfs://bafy...`). |

---

## 2. BazaarDealEngine Errors

Contract: `contracts/bazaar_deal_engine`

| Code (`u32`) | Identifier | Description | Remediation |
| :--- | :--- | :--- | :--- |
| **1** | `AlreadyInitialized` | Engine contract has already been initialized. | Contract initialization is one-time only. |
| **2** | `NotInitialized` | Deal engine state has not been initialized. | Execute `initialize` with admin address. |
| **3** | `Unauthorized` | Caller lacks authority for the requested action. | Ensure transaction signer matches the required role. |
| **4** | `CircleNotFound` | Circle ID does not exist in Deal Engine storage. | Verify circle ID using `get_circle_count()`. |
| **5** | `CircleAlreadyClosed` | Circle is in `Settled` or `Cancelled` state. | No further commitments or offers can be accepted. |
| **6** | `CircleNotSettled` | Action requires a settled deal state. | Settle the winning offer first. |
| **7** | `InvalidAmount` | Escrow amount or payout is zero or exceeds available balance. | Check unit price and total escrow funding. |
| **8** | `InvalidVolume` | Commitment or offer volume is zero or exceeds allowed bounds. | Specify non-zero volume within min/max thresholds. |
| **9** | `DeadlinePassed` | Ledger timestamp has exceeded the circle deadline. | Transactions must be submitted before deadline timestamp. |
| **10** | `DeadlineNotPassed` | Refund or expiration attempted prematurely. | Allow circle deadline to lapse before claiming expiry refund. |
| **11** | `QuorumNotReached` | Deal settlement attempted before committed volume reached minimum threshold. | Wait for additional buyer commitments to reach quorum. |
| **12** | `OfferNotFound` | The requested seller offer ID does not exist. | Query `get_circle_offers` for valid offer IDs. |
| **13** | `OfferMismatch` | Offer does not belong to the referenced circle. | Submit offer specifically for the target circle ID. |
| **14** | `AlreadyCommitted` | Buyer has already committed capital to this circle. | Each buyer is permitted one active commitment per circle. |
| **15** | `CommitmentNotFound` | Buyer address has no recorded commitment in this circle. | Verify buyer address and circle ID. |
| **16** | `AlreadyRefunded` | Buyer has already withdrawn their escrow refund. | Refunds are one-time only. |
| **17** | `VolumeExceeded` | Commitment would push current volume past `max_volume`. | Commit a smaller quantity or wait for remaining allocation. |
| **18** | `RegistryError` | Cross-contract call to `DemandCircleRegistry` failed. | Ensure target registry contract address is valid and initialized. |
| **19** | `PriceExceedsTarget` | Seller offer unit price exceeds the circle's maximum acceptable price. | Submit a quote at or below `target_unit_price`. |
| **20** | `DuplicateOffer` | Seller already has an active offer for this circle. | Sellers may only have one submitted quote per circle. |
| **21** | `NotUnderInspection` | Settlement attempted via inspection confirmation on a circle not in Inspection state. | Circle must enter inspection phase before delivery confirmation settlement. |
