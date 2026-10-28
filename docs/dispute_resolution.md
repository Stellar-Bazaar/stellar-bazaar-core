# Commercial Inspection & Dispute Resolution Architecture

In B2B bulk commerce and decentralized demand aggregation, escrow cannot immediately be released without proof of fulfillment and inspection of delivered bulk freight. Stellar Bazaar introduces an on-chain **Two-Step Inspection & Dispute Arbitration Framework** natively built on Soroban.

## 1. Lifecycle Overview

```mermaid
stateDiagram-v2
    [*] --> Open: create_circle
    Open --> QuorumReached: commit_demand >= min_volume
    QuorumReached --> Inspection: initiate_inspection(duration)
    Inspection --> Settled: confirm_inspection_and_settle
    Inspection --> Disputed: dispute_deal(reason)
    Disputed --> Cancelled: resolve_dispute(refund_to_buyers: true)
    Disputed --> Settled: resolve_dispute(refund_to_buyers: false)
    Cancelled --> [*]: claim_refund
    Settled --> [*]: surplus rebate / withdrawal
```

---

## 2. States & Roles

| State | Allowed Actors | Permitted Operations |
|---|---|---|
| **Inspection** | Circle Creator | `confirm_inspection_and_settle` releases escrow payout to seller. |
| **Inspection** | Creator & Committed Buyers | `dispute_deal(circle_id, reason)` opens formal arbitration. |
| **Disputed** | Protocol Admin | `resolve_dispute(circle_id, refund_to_buyers)` arbitrates. |
| **Cancelled** | Committed Buyers | `claim_refund(circle_id)` reclaims 100% of committed capital. |

---

## 3. Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor Creator as Circle Creator
    actor Buyer as Buyer
    actor Seller as Seller
    actor Admin as Protocol Arbiter
    participant Engine as BazaarDealEngine
    participant Token as SAC Token Contract

    Creator->>Engine: initiate_inspection(circle_id, 7 days)
    Engine-->>Creator: Status -> Inspection (Deadline extended)

    alt Inspection Passed
        Creator->>Engine: confirm_inspection_and_settle(circle_id)
        Engine->>Token: transfer(net_payout to Seller)
        Engine->>Token: transfer(fee to Treasury)
        Engine-->>Seller: Escrow Released
    else Defect / Delivery Failure
        Buyer->>Engine: dispute_deal(circle_id, "DAMAGED_GOODS")
        Engine-->>Buyer: Status -> Disputed

        Admin->>Engine: resolve_dispute(circle_id, refund_to_buyers: true)
        Engine-->>Admin: Status -> Cancelled, Seller Penalty Recorded

        Buyer->>Engine: claim_refund(circle_id)
        Engine->>Token: transfer(100% Escrow to Buyer)
        Engine-->>Buyer: Refund Complete
    end
```

---

## 4. Protocol Fee Treasury

The protocol fee is governed by the contract admin and capped at a maximum of **10.00% (1,000 bps)**:

- `set_protocol_fee(caller, fee_bps)`: Configures fee basis points (`bps <= 1000`).
- `set_treasury(caller, treasury_address)`: Configures recipient treasury account.
- `get_protocol_fee_bps()`: Queries active fee bps.
- `get_treasury()`: Queries active treasury address.

During deal settlement, `bazaar_math::calculate_net_payout(gross_payout, fee_bps)` computes the split, and the tokens are atomically transferred to the treasury and seller following Checks-Effects-Interactions (CEI).
