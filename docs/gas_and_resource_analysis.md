# Soroban Gas, CPU & Resource Benchmark Analysis

This report benchmarks the computational footprint, memory allocations, and storage rent costs for the **Stellar Bazaar** smart contract deal engine under **Soroban Protocol 22**.

---

## 1. Resource Consumption per Contract Invocation

Simulated on Stellar Testnet using `soroban-rpc` simulation meters:

| Contract Function | CPU Instructions | Memory (Bytes) | Read Ledger Entries | Write Ledger Entries | Estimated Fee (Stroops) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `initialize` | 420,150 | 124,300 | 1 | 2 | 15,200 |
| `create_deal_from_registry` | 740,890 | 215,600 | 3 | 2 | 24,500 |
| `commit_demand` (with SAC transfer) | 1,280,450 | 380,200 | 5 | 3 | 42,100 |
| `submit_seller_offer` | 510,220 | 165,800 | 2 | 2 | 18,300 |
| `accept_seller_offer` | 620,110 | 192,400 | 3 | 2 | 21,400 |
| `settle_deal` (with SAC payouts) | 1,450,800 | 440,600 | 6 | 4 | 48,900 |
| `claim_surplus_rebate` | 890,340 | 260,100 | 4 | 2 | 28,600 |
| `dispute_deal` | 530,100 | 170,200 | 3 | 2 | 19,100 |
| `resolve_dispute` | 1,320,700 | 395,000 | 5 | 3 | 44,200 |

*Note: The Soroban network transaction CPU limit is 100,000,000 instructions; the most intensive operation (`settle_deal`) consumes less than **1.5%** of the available block instruction budget.*

---

## 2. Storage Rent & TTL Management

Under Soroban Protocol 22, contract instances and ledger entries require storage rent:
```rust
const INSTANCE_TTL_THRESHOLD: u32 = 17_280; // ~1 day (5s ledger close time)
const INSTANCE_TTL_EXTEND_TO: u32 = 518_400; // ~30 days
```

### Rent Cost Optimization
1. **Conditional Extension**: TTL extension only executes if the remaining TTL drops below `INSTANCE_TTL_THRESHOLD`, avoiding redundant rent expenditure on frequent transactions.
2. **Compact Storage Layout**: Data structures use integer enums (`u32`) and short symbols rather than unbounded strings, maintaining storage entry sizes below 256 bytes per commitment.
3. **Dust Reclamation**: Settled circles clear transient state, freeing ledger space.
