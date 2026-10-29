# Dispute Arbitration & Inspection Protocol Specification

This document formalizes the commercial dispute lifecycle, inspection window deadlines, and arbitrator refund partitioning implemented in `bazaar_deal_engine`.

---

## 1. Lifecycle State Machine

```
              +------------------------------------------+
              |           OPEN / QUORUM REACHED          |
              +--------------------+---------------------+
                                   |
                         accept_seller_offer()
                                   v
              +------------------------------------------+
              |           INSPECTION PERIOD              |
              | (Timer: delivery_deadline + inspection)  |
              +---------+----------------------+---------+
                        |                      |
      Inspection lapses |                      | Buyer invokes
      with zero dispute |                      | dispute_deal()
                        v                      v
              +------------------+    +------------------+
              |   settle_deal()  |    |     DISPUTED     |
              | (Payout Seller)  |    +--------+---------+
              +------------------+             |
                                               | Arbitrator invokes
                                               | resolve_dispute(bps)
                                               v
                                      +------------------+
                                      | BINDING DIVISION |
                                      | (Split by bps)   |
                                      +------------------+
```

---

## 2. Invariant Rules

1. **Inspection Window Enforcement**:
   - Disputes can **only** be filed during the active inspection window.
   - Once the inspection window passes, the seller offer is irrevocably eligible for `settle_deal()`.
2. **Buyer Authorization**:
   - Only accounts with active token commitments in the circle may file a dispute (`dispute_deal`).
3. **Arbitrator Restriction**:
   - Only the designated contract arbitrator or administrative multi-sig may invoke `resolve_dispute()`.
4. **Partition Math**:
   - `refund_buyer_bps` must be between $0$ and $10,000$ (0% to 100%).
   - Protocol fees are assessed only against the seller's received portion. If 100% is refunded to buyers, zero protocol fee is assessed.

