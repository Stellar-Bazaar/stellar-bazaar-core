# Smart Contract Formal Verification & Mathematical Invariants

This document outlines the formal mathematical invariants and solvency guarantees enforced by `bazaar_deal_engine` and verified across our test suites.

---

## 1. Primary Protocol Invariants

### Invariant 1: Escrow Solvency Invariant
$$\text{Contract SAC Balance} \ge \sum_{c \in \text{Circles}} \text{Escrow}(c)$$
- **Guarantee**: At any ledger height $H$, the contract's actual token balance on the Stellar Asset Contract is greater than or equal to the sum of all unrefunded buyer commitments and unclaimed surplus rebates.
- **Proof Mechanism**: In `commit_demand`, the contract transfers tokens from the buyer before recording the commitment. In `claim_refund` and `claim_surplus_rebate`, internal state transitions (`refunded = true`, `rebate_claimed = true`) precede token transfers, guaranteeing strict solvency.

### Invariant 2: Ceiling Price Upper Bound Invariant
$$\forall o \in \text{Offers}(c), \quad o.\text{unit\_price} \le c.\text{target\_unit\_price}$$
- **Guarantee**: No seller offer can ever commit buyers to pay more than their explicitly agreed ceiling price.
- **Enforcement**:
  ```rust
  if unit_price > circle.target_unit_price {
      return Err(Error::CeilingPriceExceeded);
  }
  ```

### Invariant 3: Single Winning Offer Invariant
$$|\{ o \in \text{Offers}(c) \mid o.\text{status} = \text{Accepted} \}| \le 1$$
- **Guarantee**: A buying circle can bind to at most one wholesale deal. Conflicting simultaneous settlements are impossible.
- **Enforcement**: Once `accepted_offer_id` is set, subsequent calls to `accept_seller_offer` fail immediately with `Error::OfferAlreadyAccepted`.

### Invariant 4: Non-Negative Surplus Rebate Invariant
$$\forall b \in \text{Buyers}(c), \quad \text{Rebate}(b) \ge 0$$
- **Proof**: Since $P_{\text{win}} \le P_{\text{ceil}}$ (by Invariant 2) and committed units $U_i > 0$, the quantity $(P_{\text{ceil}} - P_{\text{win}}) \times U_i \ge 0$ monotonically.

### Invariant 5: Idempotent Single-Claim Invariant
$$\forall b \in \text{Buyers}(c), \quad \text{ClaimCount}(b) \le 1$$
- **Guarantee**: Double claims of refunds or rebates are rejected by the Soroban host.
- **Enforcement**: Explicit boolean guard `if commitment.rebate_claimed { return Err(Error::RebateAlreadyClaimed); }`.

