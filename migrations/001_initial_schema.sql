-- Stellar Bazaar: Initial Core Database Schema
-- Version: 001
-- Description: Core tables for Demand Circles, Buyer Commitments, Seller Offers, and Settlement Audit

-- 1. Demand Circles (Buyer Demand Aggregation Pools)
CREATE TABLE IF NOT EXISTS demand_circles (
    id VARCHAR(64) PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    category VARCHAR(64) NOT NULL,
    target_unit_price NUMERIC(18, 7) NOT NULL, -- XLM or Token price per unit
    min_volume INTEGER NOT NULL,
    max_volume INTEGER NOT NULL,
    current_volume INTEGER DEFAULT 0 NOT NULL,
    min_participants INTEGER DEFAULT 1 NOT NULL,
    current_participants INTEGER DEFAULT 0 NOT NULL,
    creator_address VARCHAR(56) NOT NULL,
    contract_circle_id BIGINT,
    status VARCHAR(32) NOT NULL DEFAULT 'OPEN', -- OPEN, QUORUM_REACHED, SETTLING, SETTLED, CANCELLED
    deadline_timestamp BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_demand_circles_status ON demand_circles(status);
CREATE INDEX IF NOT EXISTS idx_demand_circles_creator ON demand_circles(creator_address);
CREATE INDEX IF NOT EXISTS idx_demand_circles_deadline ON demand_circles(deadline_timestamp);

-- 2. Buyer Commitments (Escrowed Capital Backing Demand)
CREATE TABLE IF NOT EXISTS buyer_commitments (
    id VARCHAR(64) PRIMARY KEY,
    circle_id VARCHAR(64) NOT NULL REFERENCES demand_circles(id) ON DELETE CASCADE,
    buyer_address VARCHAR(56) NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    committed_amount_xlm NUMERIC(18, 7) NOT NULL CHECK (committed_amount_xlm > 0),
    escrow_tx_hash VARCHAR(64),
    status VARCHAR(32) NOT NULL DEFAULT 'COMMITTED', -- COMMITTED, REFUNDED, SETTLED
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    settled_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX IF NOT EXISTS idx_buyer_commitments_circle ON buyer_commitments(circle_id);
CREATE INDEX IF NOT EXISTS idx_buyer_commitments_buyer ON buyer_commitments(buyer_address);

-- 3. Seller Quotes & Offers (Suppliers Competing for Aggregate Demand)
CREATE TABLE IF NOT EXISTS seller_offers (
    id VARCHAR(64) PRIMARY KEY,
    circle_id VARCHAR(64) NOT NULL REFERENCES demand_circles(id) ON DELETE CASCADE,
    seller_address VARCHAR(56) NOT NULL,
    offered_unit_price NUMERIC(18, 7) NOT NULL CHECK (offered_unit_price > 0),
    available_volume INTEGER NOT NULL CHECK (available_volume > 0),
    lead_time_days INTEGER NOT NULL CHECK (lead_time_days >= 0),
    reputation_score NUMERIC(5, 2) DEFAULT 100.0,
    status VARCHAR(32) NOT NULL DEFAULT 'ACTIVE', -- ACTIVE, ACCEPTED, REJECTED, EXPIRED
    terms_hash VARCHAR(64) NOT NULL,
    submitted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_seller_offers_circle ON seller_offers(circle_id);
CREATE INDEX IF NOT EXISTS idx_seller_offers_seller ON seller_offers(seller_address);

-- 4. Settlements & Deal Executions (Soroban Contract Executed Settlements)
CREATE TABLE IF NOT EXISTS settlements (
    id VARCHAR(64) PRIMARY KEY,
    circle_id VARCHAR(64) NOT NULL REFERENCES demand_circles(id),
    winning_offer_id VARCHAR(64) NOT NULL REFERENCES seller_offers(id),
    total_volume_cleared INTEGER NOT NULL,
    total_payout_xlm NUMERIC(18, 7) NOT NULL,
    contract_call_tx_hash VARCHAR(64) NOT NULL,
    ledger_sequence BIGINT NOT NULL,
    executed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_settlements_circle ON settlements(circle_id);
CREATE INDEX IF NOT EXISTS idx_settlements_tx ON settlements(contract_call_tx_hash);

-- 5. Indexer Checkpoints (Ledger Sync Tracking)
CREATE TABLE IF NOT EXISTS indexer_checkpoints (
    network VARCHAR(32) PRIMARY KEY,
    last_processed_ledger BIGINT NOT NULL,
    last_block_hash VARCHAR(64),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
