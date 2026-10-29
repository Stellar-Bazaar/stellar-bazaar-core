# Cross-Contract Architecture: Registry & Deal Engine

This specification details the cross-contract invocation flow and authorization boundaries between `demand_circle_registry` and `bazaar_deal_engine`.

---

## 1. Architectural Separation of Concerns

```
                                  +---------------------------------+
                                  |         Circle Creator          |
                                  +----------------+----------------+
                                                   |
                        1. create_circle(...)      | 2. create_deal_from_registry(...)
                                                   v
+-------------------------------+      Cross-Contract Call      +-------------------------------+
|     demand_circle_registry    |<------------------------------|       bazaar_deal_engine      |
|  - Metadata URI Storage       |  3. client.get_circle(id)     |  - Non-Custodial SAC Escrow   |
|  - Circle Status (Open/Closed)|                               |  - Quorum Qualification Engine|
|  - Target Units & Constraints |                               |  - Winning Offer Settlement   |
+-------------------------------+                               |  - Surplus Rebate Arithmetic  |
                                                                +---------------+---------------+
                                                                                |
                                                                                v
                                                                +-------------------------------+
                                                                |    Stellar Asset Contract     |
                                                                |        (XLM / USDC SAC)       |
                                                                +-------------------------------+
```

---

## 2. Cross-Contract Invocation Pattern

The deal engine uses a generated Soroban client to query the registry:

```rust
pub fn create_deal_from_registry(
    env: Env,
    creator: Address,
    registry_contract: Address,
    registry_circle_id: u64,
    token: Address,
    min_volume: u32,
    max_volume: u32,
) -> Result<u64, Error> {
    creator.require_auth();

    // 1. Cross-contract call to DemandCircleRegistry
    let registry_client = RegistryClient::new(&env, &registry_contract);
    let circle = registry_client.get_circle(&registry_circle_id);

    // 2. Validate circle properties
    if circle.creator != creator {
        return Err(Error::Unauthorized);
    }
    if circle.status != CircleStatus::Open {
        return Err(Error::CircleNotActive);
    }

    // 3. Register deal with verified parameters
    // ...
}
```

---

## 3. Security Benefits
1. **Isolated Risk**: If metadata schemas evolve in the registry, financial escrow contracts remain unchanged and secure.
2. **Deterministic Data Propagation**: Deal terms (ceiling price, deadline) are fetched directly from immutable on-chain storage, preventing creator parameter spoofing.
3. **Composability**: External dApps can build custom registry interfaces while relying on the audited `bazaar_deal_engine` for settlement.
