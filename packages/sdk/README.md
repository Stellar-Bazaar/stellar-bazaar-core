# `@stellar-bazaar/sdk`

Official TypeScript/JavaScript developer SDK for interacting with the **Stellar Bazaar** Soroban Smart Contract deal engine, indexed relational feeds, and real-time ledger events.

## Features

- ⚡ **Type-Safe Contract Invocations**: Seamlessly prepare and build invocations for circle creation, buyer commitments, merchant offers, surplus rebates, and dispute arbitrations.
- 💰 **Lossless BigInt Stroop Converters**: High-precision arithmetic converting human-readable token decimals to Stellar stroops (7 decimals) and vice versa without floating-point truncation.
- 📡 **EventWatcher with Cursor Tracking**: Continuous polling of Soroban RPC `getEvents` with automatic cursor and ledger progression.
- 🛠️ **TransactionBuilder**: Fine-tune Stellar base fees, CPU instruction limits, RAM budgets, and transaction timeout windows.
- 🔍 **Descriptive Error Hierarchy**: Native mapping of contract exit codes 1 through 27 to human-friendly diagnostic explanations and remediation advice.

---

## Installation

```bash
npm install @stellar-bazaar/sdk
# or via pnpm / yarn
pnpm add @stellar-bazaar/sdk
```

---

## Quickstart

### 1. Initialize the Client

```typescript
import { StellarBazaarClient, TESTNET_CONFIG } from '@stellar-bazaar/sdk';

const client = new StellarBazaarClient(TESTNET_CONFIG);
```

### 2. Inspect an Active Buying Circle

```typescript
const circle = await client.getCircle(1);
if (circle) {
  console.log(`Circle Status: ${circle.status}`);
  console.log(`Committed Units: ${circle.committedUnits} / ${circle.targetUnits}`);
  console.log(`Ceiling Price: ${circle.ceilingPriceStroops} stroops`);
}
```

### 3. Prepare a Commitment Invocation

```typescript
import { TransactionBuilder, decimalToStroops } from '@stellar-bazaar/sdk';

// Convert 50 XLM to 500,000,000 stroops
const commitAmount = decimalToStroops('50.0');

const invocation = client.commitToCircleInvocation(
  1,
  'GBUYER1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ',
  commitAmount,
  10n
);

const tx = new TransactionBuilder('GBUYER1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ', invocation)
  .setBaseFee(200n)
  .setTimeout(600)
  .build();

console.log('Prepared Transaction Envelope:', tx);
```

### 4. Listen to Real-Time Soroban Events

```typescript
import { EventWatcher, TESTNET_CONFIG } from '@stellar-bazaar/sdk';

const watcher = new EventWatcher({
  rpcUrl: TESTNET_CONFIG.rpcUrl,
  contractId: TESTNET_CONFIG.dealEngineContractId,
  startLedger: 120000,
  pollIntervalMs: 2500,
  onEvent: (event) => {
    console.log(`[Ledger ${event.ledger}] Event:`, event.topics, event.value);
  },
  onError: (err) => {
    console.error('Event stream error:', err);
  },
});

watcher.start();
```

---

## Error Handling

```typescript
import { resolveContractError, SorobanContractError } from '@stellar-bazaar/sdk';

try {
  // Execute transaction...
} catch (err) {
  if (err instanceof SorobanContractError) {
    console.error(`Error #${err.code} (${err.errorName}): ${err.message}`);
    console.info(`Recommended Action: ${err.remediation}`);
  }
}
```

---

## License

MIT © KingTaiwoDev & Stellar Bazaar contributors

