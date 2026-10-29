# Treasury Governance & Protocol Tokenomics

This specification outlines the economic dynamics, fee distribution mechanics, and mathematical formulation of buyer surplus rebates in **Stellar Bazaar**.

---

## 1. Wholesale Group-Buying Tokenomics

```
+--------------------------------------------------------------------------+
| Total Escrow Capital = Target Volume x Ceiling Price                     |
+--------------------------------------------------------------------------+
                                     |
                                     v
+------------------------------------+-------------------------------------+
|                                    |                                     |
v                                    v                                     v
+--------------------------+  +--------------------------+  +--------------------------+
| Winning Seller Payout    |  | Protocol Fee (Treasury)  |  | Buyer Surplus Rebates    |
| (Offer Price x Volume)   |  | (e.g. 2% of settlement)  |  | (Ceiling Price - Offer)  |
| minus protocol fee       |  | Dedicated community pool |  | Returned 100% to buyers  |
+--------------------------+  +--------------------------+  +--------------------------+
```

---

## 2. Mathematical Formulation of Surplus Rebates

Let:
- $P_{\text{ceil}}$ = Maximum ceiling unit price committed by buyers (in stroops)
- $P_{\text{win}}$ = Winning wholesale merchant unit price ($P_{\text{win}} \le P_{\text{ceil}}$)
- $U_i$ = Units committed by buyer $i$
- $F_{\text{bps}}$ = Protocol fee basis points ($0 \le F_{\text{bps}} \le 1,000$)

### Buyer Surplus Rebate:
$$\text{Rebate}_i = (P_{\text{ceil}} - P_{\text{win}}) \times U_i$$

### Total Deal Settlement:
$$\text{Gross Settlement} = P_{\text{win}} \times \sum U_i$$
$$\text{Protocol Fee} = \frac{\text{Gross Settlement} \times F_{\text{bps}}}{10,000}$$
$$\text{Net Merchant Payout} = \text{Gross Settlement} - \text{Protocol Fee}$$

### Solvency Proof:
$$\text{Total Escrow} = \text{Net Merchant Payout} + \text{Protocol Fee} + \sum \text{Rebate}_i$$
All terms are calculated in integer stroops, leaving **zero dust or unallocated funds**.

---

## 3. Protocol Fee Governance & Security Limits

1. **Immutable Fee Ceiling**:
   - The contract enforces `if fee_bps > 1000 { return Err(Error::InvalidFeeBps); }`. No admin can raise fees above 10%.
2. **Dedicated Community Treasury**:
   - Fees are transferred directly to a configurable multi-sig DAO treasury account.
3. **Dispute Protection**:
   - If a deal is cancelled or 100% refunded on dispute, protocol fees are waived to protect participants.
