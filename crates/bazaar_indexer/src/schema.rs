/// Embedded SQLite schema definitions for the Stellar Bazaar indexer
pub const SQLITE_INITIAL_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS indexer_checkpoint (
    network TEXT PRIMARY KEY,
    latest_ledger_sequence INTEGER NOT NULL,
    updated_at_timestamp INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS demand_circles (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    category TEXT NOT NULL,
    target_unit_price_xlm TEXT NOT NULL,
    min_volume INTEGER NOT NULL,
    max_volume INTEGER NOT NULL,
    current_volume INTEGER DEFAULT 0 NOT NULL,
    status TEXT NOT NULL,
    creator_address TEXT NOT NULL,
    deadline_timestamp INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS buyer_commitments (
    id TEXT PRIMARY KEY,
    circle_id TEXT NOT NULL REFERENCES demand_circles(id),
    buyer_address TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    committed_amount_xlm TEXT NOT NULL,
    escrow_tx_hash TEXT,
    status TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS seller_offers (
    id TEXT PRIMARY KEY,
    circle_id TEXT NOT NULL REFERENCES demand_circles(id),
    seller_address TEXT NOT NULL,
    offered_unit_price_xlm TEXT NOT NULL,
    available_volume INTEGER NOT NULL,
    lead_time_days INTEGER NOT NULL,
    status TEXT NOT NULL,
    submitted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS seller_reputations (
    seller_address TEXT PRIMARY KEY,
    successful_deals INTEGER DEFAULT 0 NOT NULL,
    total_volume_settled INTEGER DEFAULT 0 NOT NULL,
    total_amount_settled_xlm TEXT DEFAULT '0' NOT NULL,
    disputed_or_refunded_deals INTEGER DEFAULT 0 NOT NULL,
    score_percentage INTEGER DEFAULT 100 NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS disputes (
    id TEXT PRIMARY KEY,
    circle_id TEXT NOT NULL REFERENCES demand_circles(id),
    initiator_address TEXT NOT NULL,
    dispute_reason TEXT NOT NULL,
    status TEXT NOT NULL,
    resolution_refund_to_buyers INTEGER,
    opened_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS protocol_fees (
    id TEXT PRIMARY KEY,
    circle_id TEXT NOT NULL REFERENCES demand_circles(id),
    gross_payout_stroops TEXT NOT NULL,
    fee_bps INTEGER NOT NULL,
    fee_stroops TEXT NOT NULL,
    net_payout_stroops TEXT NOT NULL,
    treasury_address TEXT NOT NULL,
    settled_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_string_validity() {
        assert!(SQLITE_INITIAL_SCHEMA.contains("CREATE TABLE IF NOT EXISTS demand_circles"));
        assert!(SQLITE_INITIAL_SCHEMA.contains("CREATE TABLE IF NOT EXISTS indexer_checkpoint"));
        assert!(SQLITE_INITIAL_SCHEMA.contains("CREATE TABLE IF NOT EXISTS seller_reputations"));
        assert!(SQLITE_INITIAL_SCHEMA.contains("CREATE TABLE IF NOT EXISTS disputes"));
        assert!(SQLITE_INITIAL_SCHEMA.contains("CREATE TABLE IF NOT EXISTS protocol_fees"));
    }

    #[test]
    fn test_schema_contains_relational_foreign_keys() {
        assert!(SQLITE_INITIAL_SCHEMA.contains("REFERENCES demand_circles(id)"));
        assert!(SQLITE_INITIAL_SCHEMA.contains("gross_payout_stroops TEXT NOT NULL"));
        assert!(SQLITE_INITIAL_SCHEMA.contains("treasury_address TEXT NOT NULL"));
        assert!(SQLITE_INITIAL_SCHEMA.contains("resolution_refund_to_buyers INTEGER"));
    }
}

