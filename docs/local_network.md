# Local Standalone Soroban Network Setup Guide

This guide details how to spin up, configure, and interact with a local standalone Stellar and Soroban network for rapid offline smart contract development and integration testing.

---

## 1. Prerequisites

- **Docker Desktop** installed and running
- **Stellar CLI** (`stellar --version >= 21.0.0`)
- **Rust & Cargo** (`cargo >= 1.80.0`) with `wasm32-unknown-unknown` target

---

## 2. Running Local Soroban Container

Run the official Stellar Quickstart container configured for standalone Soroban development:

```bash
docker run --rm -it \
  -p 8000:8000 \
  -p 11637:11637 \
  --name stellar-standalone \
  stellar/quickstart:latest \
  --standalone \
  --enable-soroban-rpc
```

### Key Endpoints
- **Horizon API**: `http://localhost:8000`
- **Soroban RPC**: `http://localhost:8000/soroban/rpc`
- **Network Passphrase**: `Standalone Network ; February 2017`

---

## 3. Configuring Stellar CLI for Localhost

Configure a network profile in Stellar CLI:

```bash
# Add network
stellar network add \
  --global local \
  --rpc-url "http://localhost:8000/soroban/rpc" \
  --network-passphrase "Standalone Network ; February 2017"

# Generate local test identities
stellar keys generate --global alice --network local
stellar keys generate --global bob --network local
stellar keys generate --global admin --network local

# Fund accounts via local friendbot
curl "http://localhost:8000/friendbot?addr=$(stellar keys address admin)"
curl "http://localhost:8000/friendbot?addr=$(stellar keys address alice)"
curl "http://localhost:8000/friendbot?addr=$(stellar keys address bob)"
```

---

## 4. Building and Deploying Contracts Locally

```bash
# Build contracts to wasm
stellar contract build

# Deploy DemandCircleRegistry
REGISTRY_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/demand_circle_registry.wasm \
  --source admin \
  --network local)

echo "Registry deployed at: $REGISTRY_ID"

# Initialize Registry
stellar contract invoke \
  --id $REGISTRY_ID \
  --source admin \
  --network local \
  -- \
  initialize \
  --admin $(stellar keys address admin)

# Deploy BazaarDealEngine
ENGINE_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/bazaar_deal_engine.wasm \
  --source admin \
  --network local)

echo "Deal Engine deployed at: $ENGINE_ID"

# Initialize Engine
stellar contract invoke \
  --id $ENGINE_ID \
  --source admin \
  --network local \
  -- \
  initialize \
  --admin $(stellar keys address admin)
```

---

## 5. Local Automated Testing Script

For one-command test execution against local node, run:
```powershell
powershell -ExecutionPolicy Bypass -File ./scripts/setup_test_validator.ps1
```
