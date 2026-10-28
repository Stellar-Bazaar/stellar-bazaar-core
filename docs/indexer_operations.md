# Stellar Bazaar Indexer Daemon Operations Guide

The `bazaar-indexer` daemon provides continuous, low-latency indexing of Soroban smart contract events emitted by the Demand Circle Registry and Bazaar Deal Engine contracts. It projects decentralized ledger events into an embedded relational SQLite store for high-performance frontend queries, buyer analytics, and seller reputation lookups.

---

## 1. Architecture Overview

```
                      +-----------------------------+
                      |   Stellar Soroban Network   |
                      |   (Testnet / Standalone)    |
                      +--------------+--------------+
                                     |
                         RPC getEvents (Polling)
                                     v
                      +-----------------------------+
                      |     Soroban RPC Client      |
                      +--------------+--------------+
                                     |
                           Raw Diagnostic Events
                                     v
                      +-----------------------------+
                      |        Event Decoder        |
                      +--------------+--------------+
                                     |
                          Typed BazaarContractEvent
                                     v
                      +-----------------------------+
                      |       Event Processor       |
                      +--------------+--------------+
                                     |
                      +--------------+--------------+
                      |                             |
                      v                             v
           +--------------------+         +-------------------+
           | Checkpoint Manager |         | SQLite Relational |
           | (Monotonic Ledgers)|         |    Repository     |
           +--------------------+         +-------------------+
```

### Components
1. **RPC Polling Loop**: Repeatedly queries `getEvents` for designated contract addresses starting from the last ingested ledger sequence checkpoint.
2. **Exponential Backoff**: If the Soroban RPC endpoint experiences rate-limiting, network partitions, or 503 timeouts, backoff doubles with each consecutive failure up to a strict cap of 30 seconds (`2^min(errors, 5) * base_interval`), preventing RPC spamming while ensuring prompt recovery.
3. **Event Decoder**: Decodes Soroban ScVal event topics into typed Rust events (`DemandCircleCreated`, `CommitmentAdded`, `OfferProposed`, `OfferAccepted`, `InspectionInitiated`, `SettlementCompleted`, `DisputeOpened`, `DisputeResolved`).
4. **Relational Repository**: Atomic SQLite transactions maintain normalized tables (`demand_circles`, `commitments`, `seller_offers`, `seller_reputation`, `disputes`, `protocol_fees`, `indexer_checkpoints`).
5. **Monotonic Checkpoint Manager**: Guarantees that ledger replay is idempotent and prevents checkpoint rollbacks.

---

## 2. Configuration Presets

Configuration can be specified via command-line flags or the `.env.indexer` template:

| Parameter | CLI Flag | Default | Description |
|---|---|---|---|
| Database Path | `--db <path>` | `bazaar_indexer.db` | Path to SQLite file (`:memory:` for ephemeral mode) |
| Soroban RPC URL | `--rpc <url>` | `https://soroban-testnet.stellar.org` | Soroban JSON-RPC 2.0 endpoint |
| Contract ID | `--contract <id>` | Default Testnet IDs | Contract addresses to filter events for |
| Batch Size | `--batch-size <num>` | `100` | Max events per RPC request |
| Start Ledger | `--start-ledger <seq>` | `None` (Checkpoint) | Ledger sequence to initiate replay from |
| Health Probe | `--health` | N/A | Evaluates daemon state and outputs JSON |
| Metrics Exporter | `--metrics` | N/A | Renders Prometheus text exposition format |
| Dry Run | `--dry-run` | N/A | Validates database connectivity and config without starting polling loop |

---

## 3. Operational Runbook

### Running Dry-Run Verification
To verify database schemas and configuration without connecting to live nodes:
```bash
cargo run -p bazaar-indexer -- --dry-run --db :memory:
```

### Running with PowerShell Helper
```powershell
.\scripts\run_indexer.ps1 -DryRun -DatabasePath :memory:
```

### Running Test Suite
```bash
make indexer-test
# or
cargo test -p bazaar-indexer
```

### Running Verification Script
```powershell
.\scripts\verify_indexer.ps1
```

---

## 4. Observability & Monitoring

### Health Probe
Running the daemon with `--health` evaluates operational vitality:
```json
{
  "status": "healthy",
  "uptime_seconds": 1284,
  "latest_ledger": 142050,
  "total_events_indexed": 84,
  "total_circles_indexed": 12,
  "total_disputes_indexed": 1,
  "consecutive_errors": 0,
  "last_error": null
}
```

Health status transitions:
- `syncing`: Fresh daemon startup before initial ledger synchronization.
- `healthy`: Active ledger ingestion with fewer than 5 consecutive transient errors.
- `degraded`: 5 or more consecutive RPC errors requiring alert escalation.

### Prometheus Metrics
The daemon exposes standard Prometheus format metrics via `--metrics`:
- `bazaar_indexer_latest_ledger`: Gauge of latest processed ledger.
- `bazaar_indexer_events_total`: Counter of all contract events ingested.
- `bazaar_indexer_circles_total`: Counter of demand circles indexed.
- `bazaar_indexer_disputes_total`: Counter of disputes logged.
- `bazaar_indexer_uptime_seconds`: Gauge of daemon process uptime.
- `bazaar_indexer_health`: Gauge (`1` for healthy, `0` for unhealthy/degraded).
