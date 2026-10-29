# Soroban Protocol 22 Compliance & Storage Architecture

This technical specification documents **Stellar Bazaar's** adherence to **Soroban Protocol 22** standards and storage rent lifecycles.

---

## 1. Storage Tier Stratification

Soroban Protocol 22 distinguishes between three storage tiers:

| Tier | Storage Type | Used in Stellar Bazaar | Rationale |
| :--- | :--- | :--- | :--- |
| **Instance** | `env.storage().instance()` | Contract configuration, Admin address, Treasury address, Circle count, Fee basis points | Shared globally across all invocations; kept active via `extend_ttl`. |
| **Persistent** | `env.storage().persistent()` | Demand Circles, Buyer Commitments, Seller Offers, Reputation Scores | Long-term commercial records requiring persistent historical custody until settlement. |
| **Temporary** | `env.storage().temporary()` | Transient dispute evidence, ephemeral simulation nonces | Discarded after ledger TTL to optimize costs. |

---

## 2. Dynamic TTL Extension Parameters

```rust
pub const INSTANCE_TTL_THRESHOLD: u32 = 17_280; // ~1 day (5s ledger time)
pub const INSTANCE_TTL_EXTEND_TO: u32 = 518_400; // ~30 days

pub fn extend_instance_ttl(env: &Env) {
    env.storage().instance().extend_ttl(
        INSTANCE_TTL_THRESHOLD,
        INSTANCE_TTL_EXTEND_TO
    );
}
```

Every state-mutating entry point (`create_deal_from_registry`, `commit_demand`, `submit_seller_offer`, `settle_deal`, `claim_refund`, `claim_surplus_rebate`, `dispute_deal`) triggers this routine, ensuring active circles never risk archival.

---

## 3. Protocol 22 Error Codes & Symbol Encoding

- Custom error codes `1..27` derive `#[contracterror]` and map to standard host contract errors.
- Structured event topics use short symbols:
  ```rust
  env.events().publish(
      (symbol_short!("deal_eng"), symbol_short!("rebate")),
      (circle_id, buyer, rebate_amount)
  );
  ```
- Guaranteed forward compatibility with future Stellar protocol upgrades.

