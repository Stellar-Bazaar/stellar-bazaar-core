# Bazaar Event Indexer (`bazaar-indexer`)

The **Bazaar Indexer** crate provides event consumption, decoding, and persistence abstractions for the Stellar Bazaar protocol. It processes contract events emitted by `DemandCircleRegistry` and `BazaarDealEngine` on Soroban and projects them into relational representations.

---

## 1. Supported Contract Events

Contracts emit structured two-part topics with typed values:

| Event Topic | Emitting Contract | Topic Filter | Data Payload (`ScVal`) |
| :--- | :--- | :--- | :--- |
| `circle:created` | `DemandCircleRegistry` | `("circle", "created")` | `(id: u64, creator: Address, qty: u32, price_stroops: i128)` |
| `circle:closed` | `DemandCircleRegistry` | `("circle", "closed")` | `(circle_id: u64, caller: Address)` |
| `circle:expired` | `DemandCircleRegistry` | `("circle", "expired")` | `(circle_id: u64)` |
| `demand:commit` | `BazaarDealEngine` | `("demand", "commit")` | `(circle_id: u64, buyer: Address, qty: u32, amount: i128)` |
| `offer:submit` | `BazaarDealEngine` | `("offer", "submit")` | `(circle_id: u64, offer_id: u64, seller: Address)` |
| `offer:accept` | `BazaarDealEngine` | `("offer", "accept")` | `(circle_id: u64, offer_id: u64, caller: Address)` |
| `deal:settled` | `BazaarDealEngine` | `("deal", "settled")` | `(circle_id: u64, winning_offer_id: u64, payout: i128)` |
| `refund:claimed` | `BazaarDealEngine` | `("refund", "claimed")` | `(circle_id: u64, buyer: Address, amount: i128)` |

---

## 2. Event Decoding Example in Rust

```rust
use soroban_sdk::xdr::{ScVal, ScVec};

pub enum DecodedBazaarEvent {
    CircleCreated {
        circle_id: u64,
        creator: String,
        target_quantity: u32,
        target_price_stroops: i128,
    },
    DemandCommitted {
        circle_id: u64,
        buyer: String,
        quantity: u32,
        amount_stroops: i128,
    },
    DealSettled {
        circle_id: u64,
        offer_id: u64,
        payout_stroops: i128,
    },
}

pub fn decode_event(topic: &str, data: &ScVal) -> Result<DecodedBazaarEvent, &'static str> {
    match topic {
        "circle:created" => {
            // Unpack 4-tuple ScVal: (id, creator, qty, price)
            Ok(DecodedBazaarEvent::CircleCreated {
                circle_id: 1,
                creator: "GDJ4...".to_string(),
                target_quantity: 50,
                target_price_stroops: 120_000_000,
            })
        }
        "demand:commit" => {
            Ok(DecodedBazaarEvent::DemandCommitted {
                circle_id: 1,
                buyer: "GB3X...".to_string(),
                quantity: 10,
                amount_stroops: 120_000_000,
            })
        }
        "deal:settled" => {
            Ok(DecodedBazaarEvent::DealSettled {
                circle_id: 1,
                offer_id: 1,
                payout_stroops: 114_000_000,
            })
        }
        _ => Err("Unknown event topic"),
    }
}
```

---

## 3. Idempotent Ledger Checkpointing

The indexer records the latest confirmed ledger sequence:

```sql
CREATE TABLE IF NOT EXISTS indexer_checkpoint (
    network VARCHAR(32) PRIMARY KEY,
    latest_ledger_sequence BIGINT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

When querying Soroban RPC `getEvents`:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getEvents",
  "params": {
    "startLedger": 1245000,
    "filters": [
      {
        "type": "contract",
        "contractIds": ["CCBY...", "CDCK..."]
      }
    ]
  }
}
```
Transactions within a ledger are committed atomically with the checkpoint sequence to ensure zero duplicate projections during restarts.

---

## 4. SQLite Storage Engine (`SqliteBazaarRepository`)

The indexer provides an embedded, zero-configuration SQLite backend with WAL mode enabled:

```rust
use bazaar_indexer::sqlite_repository::SqliteBazaarRepository;
use bazaar_indexer::BazaarRepository;

// Initialize in-memory or on-disk database
let mut repo = SqliteBazaarRepository::new("./bazaar_indexer.db")?;

// Query active demand circles
let active_circles = repo.list_active_circles()?;

// Query top-ranked sellers by verified on-chain score
let top_sellers = repo.list_top_sellers(10)?;
```

---

## 5. Soroban Event Decoding (`EventDecoder`)

The `event_decoder` module converts raw JSON-RPC event topics into strongly-typed domain events:

| Contract Event | Topic Symbol Pair | Decoded Rust Variant |
|---|---|---|
| Demand Circle Created | `("circle", "create")` | `BazaarContractEvent::CircleCreated` |
| Buyer Demand Committed | `("demand", "commit")` | `BazaarContractEvent::DemandCommitted` |
| Seller Offer Submitted | `("offer", "submit")` | `BazaarContractEvent::OfferSubmitted` |
| Inspection Window Started | `("inspect", "start")` | `BazaarContractEvent::InspectionStarted` |
| Commercial Dispute Raised | `("dispute", "opened")` | `BazaarContractEvent::DisputeOpened` |
| Dispute Arbitrated | `("dispute", "resolve")` | `BazaarContractEvent::DisputeResolved` |
| Bulk Deal Settled | `("deal", "settled")` | `BazaarContractEvent::DealSettled` |
| Surplus Rebate Claimed | `("rebate", "claimed")` | `BazaarContractEvent::SurplusRebateClaimed` |

