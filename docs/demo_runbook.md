# Stellar Bazaar - Interactive Reviewer Demo Runbook

This runbook guides evaluators and judges through an end-to-end interactive demonstration of **Stellar Bazaar** on the Stellar Testnet.

---

## Scenario: The Hardware Security Key Wholesale Pool

In this scenario:
- **Organizer Alice** creates a Buying Circle for 100 Hardware Security Keys with a maximum ceiling price of **50 XLM** each.
- **Buyer Bob** commits funds for 40 keys (**2,000 XLM** escrowed).
- **Buyer Charlie** commits funds for 60 keys (**3,000 XLM** escrowed). Quorum reaches 100%.
- **Merchant Nordic Supplies** submits a wholesale bid at **40 XLM** each (a **20% bulk discount**).
- Deal is accepted, clears the inspection window, and settles:
  - Seller receives **3,920 XLM** (4,000 XLM minus 2% protocol fee).
  - Treasury receives **80 XLM**.
  - **Surplus Rebates released**: Bob claims **400 XLM** rebate; Charlie claims **600 XLM** rebate!

---

## Walkthrough Option A: Automated CLI Lifecycle Simulation

If evaluating via terminal, execute the automated simulation script which runs the exact smart contract lifecycle with real Soroban host assertions:

```powershell
# Windows PowerShell
powershell -ExecutionPolicy Bypass -File ./scripts/simulate_e2e_cooperative_lifecycle.ps1
```

```bash
# POSIX Bash (Linux / macOS)
./scripts/simulate_e2e_cooperative_lifecycle.sh
```

### Expected Output:
```text
=== [Phase 1/6] Initializing Testnet Deal Engine & SAC Escrow ===
=== [Phase 2/6] Registering Buying Circle #1 ===
Target Volume: 100 units | Ceiling Price: 50.0000000 XLM (500,000,000 stroops)
=== [Phase 3/6] Depositing Multi-Buyer Commitments ===
Buyer 1 (Bob): Committed 40 units (2,000 XLM escrowed)
Buyer 2 (Charlie): Committed 60 units (3,000 XLM escrowed)
Quorum Reached: 100 / 100 units (100.0%)
=== [Phase 4/6] Submitting Merchant Wholesale Offer ===
Merchant: Nordic Supplies | Unit Price: 40.0000000 XLM | Discount: 20.0%
=== [Phase 5/6] Deal Acceptance & Inspection Window Elapse ===
Status transitioned to SETTLED
=== [Phase 6/6] Surplus Rebate Claim & Solvency Verification ===
Bob Rebate: 400.0000000 XLM (Claimed: OK)
Charlie Rebate: 600.0000000 XLM (Claimed: OK)
Escrow Solvency Verified: 0 stroops leakage. All invariants held!
```

---

## Walkthrough Option B: Interactive Web dApp Demo

### Step 1: Wallet Setup
1. Install the [Freighter Wallet](https://www.freighter.app/) extension.
2. In Freighter settings, switch network to **Testnet**.
3. Fund your account with test XLM using [Stellar Friendbot](https://laboratory.stellar.org/#account-creator?network=test).

### Step 2: Create a Buying Circle
1. Start the frontend: `cd ../stellar-bazaar-frontend && npm run dev`.
2. Connect Freighter using the header button.
3. Click **"Create Buying Circle"**:
   - Title: `Enterprise Security Keys Batch 1`
   - Target Units: `100`
   - Ceiling Price: `50` XLM
   - Asset: Select `XLM` (or `USDC`)
   - Inspection Period: `3` days
4. Sign the transaction in Freighter. Notice the transaction step indicator and simulation preview showing estimated CPU instructions and fee stroops.

### Step 3: Commit Capital as Buyer
1. Open the created circle card.
2. In the **"Join Wholesale Pool"** section, enter `40` units.
3. Observe the calculated commitment: `2,000 XLM`.
4. Click **"Deposit Escrow & Commit"** and approve in Freighter.
5. The progress bar updates to 40% quorum.

### Step 4: Submit Seller Wholesale Bid
1. Switch Freighter account to a secondary testnet address (acting as Merchant).
2. Click **"Submit Seller Bid"**:
   - Quoted Unit Price: `40` XLM
   - Delivery Lead Time: `5` days
3. Submit and verify offer appears in the **"Supplier Offers"** comparison table with the **"Best Price (-20%)"** badge highlighted.

### Step 5: Accept Offer & Settle
1. Switch back to the Circle Creator account.
2. Click **"Accept Winning Deal"**.
3. Notice status transitions to **INSPECTION** with the active countdown timer.
4. After inspection window closes, click **"Finalize Settlement"**.

### Step 6: Claim Surplus Rebate
1. Switch to Buyer account (Bob).
2. Click the **"Claim Surplus Rebate"** button in the banner.
3. Bob's wallet balance increases by **400 XLM**, recovering their wholesale discount directly to their account!
