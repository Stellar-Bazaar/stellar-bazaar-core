# Stellar Bazaar Protocol: Architectural Sequence Diagrams

This document details the on-chain interaction sequences between buyers, sellers, the **DemandCircleRegistry**, the **BazaarDealEngine**, and the **Stellar Asset Contract (SAC)**.

---

## 1. Complete Deal Settlement & Escrow Release Sequence

```mermaid
sequenceDiagram
    autonumber
    actor Creator as Buyer / Circle Creator
    actor Buyer2 as Co-Buyer
    actor Seller as Merchant / Seller
    participant Registry as DemandCircleRegistry
    participant Engine as BazaarDealEngine
    participant SAC as Stellar Asset Contract (XLM / USDC)

    Note over Creator,Registry: Phase 1: Commercial Registration
    Creator->>Registry: create_circle(title, qty=10, max_price=10 stroops)
    Registry-->>Creator: circle_id = 1 (State: Open)

    Note over Creator,Engine: Phase 2: Deal Engine Initialization
    Creator->>Engine: create_deal_from_registry(registry_addr, circle_id=1, min_vol=5)
    Engine->>Registry: get_circle(1)
    Registry-->>Engine: ExternalCircleRecord (verified constraints)
    Engine-->>Creator: deal_id = 1 (State: Open)

    Note over Creator,SAC: Phase 3: Collective Escrow Commitments
    Creator->>SAC: approve(Engine, 30 stroops)
    Creator->>Engine: commit_demand(deal_id=1, qty=3)
    Engine->>SAC: transfer(Creator -> Engine, 30 stroops)
    Engine-->>Creator: Commitment confirmed (Current Vol: 3)

    Buyer2->>SAC: approve(Engine, 30 stroops)
    Buyer2->>Engine: commit_demand(deal_id=1, qty=3)
    Engine->>SAC: transfer(Buyer2 -> Engine, 30 stroops)
    Engine-->>Buyer2: Commitment confirmed (Current Vol: 6 >= Min 5 -> QuorumReached!)

    Note over Seller,Engine: Phase 4: Competitive Quote Submission
    Seller->>Engine: submit_seller_offer(deal_id=1, price=9.5 stroops, vol=10, lead_time=7d)
    Engine-->>Seller: offer_id = 1 (State: Submitted)

    Note over Creator,Engine: Phase 5: Offer Acceptance
    Creator->>Engine: accept_seller_offer(deal_id=1, offer_id=1)
    Engine-->>Creator: Offer accepted (accepted_offer_id = 1)

    Note over Engine,SAC: Phase 6: Deal Settlement (Checks-Effects-Interactions)
    Engine->>Engine: 1. CHECKS: Quorum verify, volume match, escrow balance
    Engine->>Engine: 2. EFFECTS: Set Status = Settled, update SellerReputation on-chain
    Engine->>SAC: 3. INTERACTIONS: transfer(Engine -> Seller, payout = 6 * 9.5 = 57 stroops)
    Engine-->>Seller: Escrow funds disbursed & reputation incremented!
```

---

## 2. Unsettled / Cancelled Deal Refund Sequence

```mermaid
sequenceDiagram
    autonumber
    actor Buyer as Committed Buyer
    actor Creator as Circle Creator
    participant Engine as BazaarDealEngine
    participant SAC as Stellar Asset Contract

    alt Scenario A: Deadline Elapses Without Quorum
        Note over Engine: Ledger Timestamp > Circle Deadline
        Buyer->>Engine: claim_refund(deal_id=1)
        Engine->>Engine: Verify is_expired_unsettled = true & refunded = false
        Engine->>Engine: Set commitment.refunded = true (EFFECTS)
        Engine->>SAC: transfer(Engine -> Buyer, original_amount) (INTERACTIONS)
        Engine-->>Buyer: 100% Escrow Refund Received
    else Scenario B: Explicit Creator Cancellation
        Creator->>Engine: cancel_circle(deal_id=1)
        Engine->>Engine: Set CircleStatus = Cancelled
        Buyer->>Engine: claim_refund(deal_id=1)
        Engine->>Engine: Set commitment.refunded = true (EFFECTS)
        Engine->>SAC: transfer(Engine -> Buyer, original_amount) (INTERACTIONS)
        Engine-->>Buyer: 100% Escrow Refund Received
    end
```
