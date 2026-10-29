# Developer SDK Integration Guide: Bots, Marketplaces & Wholesale Daemons

This guide walks through building automated integrations, bots, and marketplace extensions using the official `@stellar-bazaar/sdk` package.

---

## 1. Architecture Overview

```
                      +-----------------------------+
                      |   Third-Party Integration   |
                      |  (Telegram Bot, Quoting AI) |
                      +--------------+--------------+
                                     |
                                     v
                       +---------------------------+
                       |    @stellar-bazaar/sdk    |
                       +-------------+-------------+
                                     |
                +--------------------+--------------------+
                |                                         |
                v                                         v
    +-----------------------+                 +-----------------------+
    |   Stellar RPC Node    |                 | Embedded REST Indexer |
    |   (Soroban Engine)    |                 |   (Relational Cache)  |
    +-----------+-----------+                 +-----------+-----------+
                |                                         |
                v                                         v
   +-------------------------+               +-------------------------+
   |   bazaar_deal_engine    |               |    PostgreSQL/SQLite    |
   |   Soroban WASM Engine   |               |    Historical Ledger    |
   +-------------------------+               +-------------------------+
```

---

## 2. Recipe: Telegram Group Buying Bot

A common pattern is allowing a DAO or group chat to aggregate demand and auto-commit to a buying circle.

```typescript
import { StellarBazaarClient, decimalToStroops, TransactionBuilder } from '@stellar-bazaar/sdk';
import { Keypair } from '@stellar/stellar-sdk';

const client = new StellarBazaarClient({ network: 'testnet' });

export async function joinCircleFromTelegram(
  circleId: number,
  buyerSecretKey: string,
  quantityUnits: number,
  unitCeilingPriceXlm: string
) {
  const buyerKeypair = Keypair.fromSecret(buyerSecretKey);
  const totalStroops = decimalToStroops(unitCeilingPriceXlm) * BigInt(quantityUnits);

  // 1. Prepare Soroban contract call invocation
  const invocation = client.commitToCircleInvocation(
    circleId,
    buyerKeypair.publicKey(),
    totalStroops,
    BigInt(quantityUnits)
  );

  // 2. Budget fees and assemble envelope
  const tx = new TransactionBuilder(buyerKeypair.publicKey(), invocation)
    .setBaseFee(250n)
    .setTimeout(300)
    .build();

  console.log(`[Bot] Assembled commit transaction for circle #${circleId}`, tx);
  return tx;
}
```

---

## 3. Recipe: Merchant Automated Quoting Daemon

Wholesale suppliers can run background daemons that monitor new circles in real time and automatically tender bids.

```typescript
import { EventWatcher, StellarBazaarClient, decimalToStroops } from '@stellar-bazaar/sdk';

const client = new StellarBazaarClient({ network: 'testnet' });

const watcher = new EventWatcher({
  rpcUrl: 'https://soroban-testnet.stellar.org',
  contractId: 'CBLJ2Y7PVR64F7V4GZJ5332YVDFO44V4V5P2O7C6X7NXXS4E77LNZD2E',
  onEvent: async (ev) => {
    if (ev.topics.includes('circle_created')) {
      const circleId = Number(ev.topics[1]);
      console.log(`[Daemon] New circle detected: #${circleId}`);

      // Query circle parameters
      const circle = await client.getCircle(circleId);
      if (circle && circle.minVolume <= 500) {
        // Auto-quote 10% below ceiling price with 5-day lead time
        const discountedStroops = (circle.ceilingPriceStroops * 90n) / 100n;
        const offerInvocation = client.submitSellerOfferInvocation(
          circleId,
          'GSELLER_PUBLIC_KEY...',
          discountedStroops,
          circle.minVolume,
          5
        );
        console.log(`[Daemon] Generated competitive bid:`, offerInvocation);
      }
    }
  },
});

watcher.start();
```

---

## 4. Recipe: Automated Surplus Rebate Claimer

When a deal settles with a wholesale discount, buyers earn surplus rebates. The SDK makes it trivial to calculate and claim.

```typescript
import {
  StellarBazaarClient,
  calculateMaxBuyerSavingsStroops,
  stroopsToDecimal,
} from '@stellar-bazaar/sdk';

async function checkAndClaimRebate(circleId: number, buyerAddress: string) {
  const client = new StellarBazaarClient({ network: 'testnet' });
  const commitment = await client.getBuyerCommitment(circleId, buyerAddress);

  if (commitment && !commitment.rebateClaimed) {
    const winningOfferPrice = 45_000_000n; // 4.5 XLM
    const ceilingPrice = 50_000_000n;     // 5.0 XLM

    const rebateStroops = calculateMaxBuyerSavingsStroops(
      ceilingPrice,
      winningOfferPrice,
      commitment.units
    );

    console.log(`Eligible Rebate: ${stroopsToDecimal(rebateStroops)} XLM`);

    const claimTx = client.claimSurplusRebateInvocation(circleId, buyerAddress);
    return claimTx;
  }
}
```

---

## 5. Security & Best Practices

1. **Never hardcode private keys**: Always use secure key management (KMS, encrypted keystores, hardware wallets).
2. **Handle Soroban RPC rate limits**: Set `pollIntervalMs` to $\ge 2000$ ms for public endpoints.
3. **Simulate before signing**: Always parse simulation responses to check CPU/RAM footprints and ensure authorization entries match the sender.
4. **Inspect contract error codes**: Catch `SorobanContractError` to display specific remediation actions to end users.

