# Stellar Bazaar Indexer REST API Reference

The `bazaar-indexer` includes a high-performance, embedded HTTP REST API server designed to serve normalized Soroban contract state to the frontend web application and external aggregators without requiring an intermediary database server.

---

## 1. Server Configuration & Invocation

Launch the embedded HTTP server:
```bash
cargo run -p bazaar-indexer -- --serve --port 8088 --db ./bazaar_indexer.db
```

Default Base URL: `http://localhost:8088`

---

## 2. API Endpoints

### 2.1 System & Telemetry

#### `GET /health`
Returns JSON health assessment, daemon uptime, and Soroban synchronization metrics.
- **Response 200 OK**:
```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "latest_ledger": 142050,
  "total_events_indexed": 84,
  "total_circles_indexed": 12,
  "total_disputes_indexed": 1,
  "consecutive_errors": 0,
  "last_error": null
}
```

#### `GET /metrics`
Exposes live Prometheus-compatible text format metrics for monitoring.
- **Response 200 OK** (`Content-Type: text/plain`):
```
# HELP bazaar_indexer_latest_ledger Latest processed Soroban ledger sequence
# TYPE bazaar_indexer_latest_ledger gauge
bazaar_indexer_latest_ledger 142050
```

---

### 2.2 Demand Circles

#### `GET /api/v1/circles`
Returns all active demand circles eligible for buyer commitments or seller proposals (`OPEN`, `QUORUM_REACHED`, `INSPECTION`).
- **Response 200 OK**:
```json
[
  {
    "id": "CDCK6A2Q...",
    "title": "Stellar Coffee Beans Bulk Order",
    "category": "Commodities",
    "target_unit_price_xlm": "10.0",
    "min_volume": 100,
    "max_volume": 5000,
    "current_volume": 2500,
    "status": "OPEN",
    "creator_address": "GCREATOR...",
    "deadline_timestamp": 1720000000
  }
]
```

#### `GET /api/v1/circles/{id}`
Returns granular metadata and volume progress for a single demand circle.
- **Response 200 OK**: Single circle object.
- **Response 404 Not Found**: `{"error": "Demand circle '{id}' not found", "code": 404}`

#### `GET /api/v1/circles/{id}/commitments`
Lists all buyer commitments pledged to a specific circle.
- **Response 200 OK**:
```json
[
  {
    "id": "COMMIT_1",
    "circle_id": "CDCK6A2Q...",
    "buyer_address": "GBUYER...",
    "quantity": 250,
    "committed_amount_xlm": "2500.0",
    "escrow_tx_hash": "a1b2c3...",
    "status": "COMMITTED"
  }
]
```

#### `GET /api/v1/circles/{id}/offers`
Lists all competitive proposals submitted by sellers for a circle.
- **Response 200 OK**:
```json
[
  {
    "id": "OFFER_1",
    "circle_id": "CDCK6A2Q...",
    "seller_address": "GSELLER...",
    "offered_unit_price_xlm": "9.5",
    "available_volume": 5000,
    "lead_time_days": 5,
    "status": "PROPOSED"
  }
]
```

#### `GET /api/v1/circles/{id}/disputes`
Lists arbitration and dispute escalation records opened against a circle.
- **Response 200 OK**:
```json
[
  {
    "id": "DISPUTE_1",
    "circle_id": "CDCK6A2Q...",
    "initiator_address": "GBUYER...",
    "dispute_reason": "Damaged items on arrival",
    "status": "OPEN",
    "resolution_refund_to_buyers": null,
    "opened_at_timestamp": 1720000100,
    "resolved_at_timestamp": null
  }
]
```

---

### 2.3 Sellers & Reputation

#### `GET /api/v1/sellers/top`
Returns seller leaderboard ranked by completed deals and fulfillment volume.
- **Response 200 OK**:
```json
[
  {
    "seller_address": "GSELLER...",
    "successful_deals": 42,
    "total_volume_settled": 125000,
    "total_amount_settled_xlm": "1250000.0",
    "disputed_or_refunded_deals": 1,
    "score_percentage": 98
  }
]
```

#### `GET /api/v1/sellers/{seller}/reputation`
Returns historical track record for a specific seller address.
- **Response 200 OK**: Single reputation object.
- **Response 404 Not Found**: `{"error": "Seller reputation for '{seller}' not found", "code": 404}`

---

### 2.4 Indexer State

#### `GET /api/v1/checkpoint`
Returns the highest ledger sequence indexed by the daemon.
- **Response 200 OK**:
```json
{
  "last_ledger": 142050
}
```

---

## 3. Cross-Origin Resource Sharing (CORS)

All responses include permissive CORS headers enabling seamless communication with web frontends:
```http
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, OPTIONS, HEAD
Access-Control-Allow-Headers: Content-Type, Authorization, X-Requested-With
Access-Control-Max-Age: 86400
```
Preflight `OPTIONS` requests automatically return `204 No Content`.
