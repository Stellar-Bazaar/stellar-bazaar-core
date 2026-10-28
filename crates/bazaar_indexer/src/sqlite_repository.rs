use std::sync::{Arc, Mutex};
use rusqlite::{params, Connection};
use crate::domain::*;
use crate::schema::SQLITE_INITIAL_SCHEMA;
use crate::{BazaarRepository, IndexerError};

/// Persistent SQLite implementation of the BazaarRepository storage trait
pub struct SqliteBazaarRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteBazaarRepository {
    /// Initialize an in-memory SQLite database instance with schema migrations
    pub fn new_in_memory() -> Result<Self, IndexerError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| IndexerError::Database(e.to_string()))?;
        conn.execute_batch(SQLITE_INITIAL_SCHEMA)
            .map_err(|e| IndexerError::Database(e.to_string()))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open or create a persistent on-disk SQLite database file with schema migrations
    pub fn new(path: &str) -> Result<Self, IndexerError> {
        let conn = Connection::open(path)
            .map_err(|e| IndexerError::Database(e.to_string()))?;
        conn.execute_batch(SQLITE_INITIAL_SCHEMA)
            .map_err(|e| IndexerError::Database(e.to_string()))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

impl BazaarRepository for SqliteBazaarRepository {
    fn record_circle(&mut self, circle: DemandCircleRecord) -> Result<(), IndexerError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO demand_circles (
                id, title, category, target_unit_price_xlm, min_volume, max_volume, current_volume, status, creator_address, deadline_timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                current_volume = excluded.current_volume,
                status = excluded.status,
                deadline_timestamp = excluded.deadline_timestamp"#,
            params![
                circle.id,
                circle.title,
                circle.category,
                circle.target_unit_price_xlm,
                circle.min_volume,
                circle.max_volume,
                circle.current_volume,
                circle.status,
                circle.creator_address,
                circle.deadline_timestamp,
            ],
        ).map_err(|e| IndexerError::Database(e.to_string()))?;
        Ok(())
    }

    fn record_commitment(&mut self, commitment: CommitmentRecord) -> Result<(), IndexerError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO buyer_commitments (
                id, circle_id, buyer_address, quantity, committed_amount_xlm, escrow_tx_hash, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                escrow_tx_hash = excluded.escrow_tx_hash"#,
            params![
                commitment.id,
                commitment.circle_id,
                commitment.buyer_address,
                commitment.quantity,
                commitment.committed_amount_xlm,
                commitment.escrow_tx_hash,
                commitment.status,
            ],
        ).map_err(|e| IndexerError::Database(e.to_string()))?;
        Ok(())
    }

    fn record_offer(&mut self, offer: SellerOfferRecord) -> Result<(), IndexerError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO seller_offers (
                id, circle_id, seller_address, offered_unit_price_xlm, available_volume, lead_time_days, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status"#,
            params![
                offer.id,
                offer.circle_id,
                offer.seller_address,
                offer.offered_unit_price_xlm,
                offer.available_volume,
                offer.lead_time_days,
                offer.status,
            ],
        ).map_err(|e| IndexerError::Database(e.to_string()))?;
        Ok(())
    }

    fn record_settlement(&mut self, _settlement: SettlementRecord) -> Result<(), IndexerError> {
        Ok(())
    }

    fn record_reputation(&mut self, reputation: SellerReputationRecord) -> Result<(), IndexerError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO seller_reputations (
                seller_address, successful_deals, total_volume_settled, total_amount_settled_xlm, disputed_or_refunded_deals, score_percentage
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(seller_address) DO UPDATE SET
                successful_deals = excluded.successful_deals,
                total_volume_settled = excluded.total_volume_settled,
                total_amount_settled_xlm = excluded.total_amount_settled_xlm,
                disputed_or_refunded_deals = excluded.disputed_or_refunded_deals,
                score_percentage = excluded.score_percentage"#,
            params![
                reputation.seller_address,
                reputation.successful_deals,
                reputation.total_volume_settled as i64,
                reputation.total_amount_settled_xlm,
                reputation.disputed_or_refunded_deals,
                reputation.score_percentage,
            ],
        ).map_err(|e| IndexerError::Database(e.to_string()))?;
        Ok(())
    }

    fn get_circle(&self, id: &str) -> Result<Option<DemandCircleRecord>, IndexerError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, title, category, target_unit_price_xlm, min_volume, max_volume, current_volume, status, creator_address, deadline_timestamp
               FROM demand_circles WHERE id = ?1"#
        ).map_err(|e| IndexerError::Database(e.to_string()))?;

        let mut rows = stmt.query(params![id]).map_err(|e| IndexerError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| IndexerError::Database(e.to_string()))? {
            Ok(Some(DemandCircleRecord {
                id: row.get(0).map_err(|e| IndexerError::Database(e.to_string()))?,
                title: row.get(1).map_err(|e| IndexerError::Database(e.to_string()))?,
                category: row.get(2).map_err(|e| IndexerError::Database(e.to_string()))?,
                target_unit_price_xlm: row.get(3).map_err(|e| IndexerError::Database(e.to_string()))?,
                min_volume: row.get(4).map_err(|e| IndexerError::Database(e.to_string()))?,
                max_volume: row.get(5).map_err(|e| IndexerError::Database(e.to_string()))?,
                current_volume: row.get(6).map_err(|e| IndexerError::Database(e.to_string()))?,
                status: row.get(7).map_err(|e| IndexerError::Database(e.to_string()))?,
                creator_address: row.get(8).map_err(|e| IndexerError::Database(e.to_string()))?,
                deadline_timestamp: row.get(9).map_err(|e| IndexerError::Database(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_offers_for_circle(&self, circle_id: &str) -> Result<Vec<SellerOfferRecord>, IndexerError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, circle_id, seller_address, offered_unit_price_xlm, available_volume, lead_time_days, status
               FROM seller_offers WHERE circle_id = ?1 ORDER BY submitted_at ASC"#
        ).map_err(|e| IndexerError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![circle_id], |row| {
            Ok(SellerOfferRecord {
                id: row.get(0)?,
                circle_id: row.get(1)?,
                seller_address: row.get(2)?,
                offered_unit_price_xlm: row.get(3)?,
                available_volume: row.get(4)?,
                lead_time_days: row.get(5)?,
                status: row.get(6)?,
            })
        }).map_err(|e| IndexerError::Database(e.to_string()))?;

        let mut offers = Vec::new();
        for offer in rows {
            offers.push(offer.map_err(|e| IndexerError::Database(e.to_string()))?);
        }
        Ok(offers)
    }

    fn get_commitments_for_circle(&self, circle_id: &str) -> Result<Vec<CommitmentRecord>, IndexerError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, circle_id, buyer_address, quantity, committed_amount_xlm, escrow_tx_hash, status
               FROM buyer_commitments WHERE circle_id = ?1 ORDER BY created_at ASC"#
        ).map_err(|e| IndexerError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![circle_id], |row| {
            Ok(CommitmentRecord {
                id: row.get(0)?,
                circle_id: row.get(1)?,
                buyer_address: row.get(2)?,
                quantity: row.get(3)?,
                committed_amount_xlm: row.get(4)?,
                escrow_tx_hash: row.get(5)?,
                status: row.get(6)?,
            })
        }).map_err(|e| IndexerError::Database(e.to_string()))?;

        let mut commitments = Vec::new();
        for commitment in rows {
            commitments.push(commitment.map_err(|e| IndexerError::Database(e.to_string()))?);
        }
        Ok(commitments)
    }

    fn get_seller_reputation(&self, seller: &str) -> Result<Option<SellerReputationRecord>, IndexerError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT seller_address, successful_deals, total_volume_settled, total_amount_settled_xlm, disputed_or_refunded_deals, score_percentage
               FROM seller_reputations WHERE seller_address = ?1"#
        ).map_err(|e| IndexerError::Database(e.to_string()))?;

        let mut rows = stmt.query(params![seller]).map_err(|e| IndexerError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| IndexerError::Database(e.to_string()))? {
            let volume: i64 = row.get(2).map_err(|e| IndexerError::Database(e.to_string()))?;
            Ok(Some(SellerReputationRecord {
                seller_address: row.get(0).map_err(|e| IndexerError::Database(e.to_string()))?,
                successful_deals: row.get(1).map_err(|e| IndexerError::Database(e.to_string()))?,
                total_volume_settled: volume as u64,
                total_amount_settled_xlm: row.get(3).map_err(|e| IndexerError::Database(e.to_string()))?,
                disputed_or_refunded_deals: row.get(4).map_err(|e| IndexerError::Database(e.to_string()))?,
                score_percentage: row.get(5).map_err(|e| IndexerError::Database(e.to_string()))?,
            }))
        } else {
            Ok(None)
        }
    }

    fn get_last_ledger(&self) -> Result<u64, IndexerError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT latest_ledger_sequence FROM indexer_checkpoint WHERE network = 'testnet'"#
        ).map_err(|e| IndexerError::Database(e.to_string()))?;

        let mut rows = stmt.query([]).map_err(|e| IndexerError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| IndexerError::Database(e.to_string()))? {
            let seq: i64 = row.get(0).map_err(|e| IndexerError::Database(e.to_string()))?;
            Ok(seq as u64)
        } else {
            Ok(0)
        }
    }

    fn record_dispute(&mut self, dispute: DisputeRecord) -> Result<(), IndexerError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO disputes (
                id, circle_id, initiator_address, dispute_reason, status, resolution_refund_to_buyers
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                resolution_refund_to_buyers = excluded.resolution_refund_to_buyers"#,
            params![
                dispute.id,
                dispute.circle_id,
                dispute.initiator_address,
                dispute.dispute_reason,
                dispute.status,
                dispute.resolution_refund_to_buyers.map(|b| if b { 1 } else { 0 }),
            ],
        ).map_err(|e| IndexerError::Database(e.to_string()))?;
        Ok(())
    }

    fn get_disputes_for_circle(&self, circle_id: &str) -> Result<Vec<DisputeRecord>, IndexerError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, circle_id, initiator_address, dispute_reason, status, resolution_refund_to_buyers
               FROM disputes WHERE circle_id = ?1 ORDER BY opened_at ASC"#
        ).map_err(|e| IndexerError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![circle_id], |row| {
            let res_int: Option<i32> = row.get(5)?;
            Ok(DisputeRecord {
                id: row.get(0)?,
                circle_id: row.get(1)?,
                initiator_address: row.get(2)?,
                dispute_reason: row.get(3)?,
                status: row.get(4)?,
                resolution_refund_to_buyers: res_int.map(|i| i == 1),
                opened_at_timestamp: 0,
                resolved_at_timestamp: None,
            })
        }).map_err(|e| IndexerError::Database(e.to_string()))?;

        let mut disputes = Vec::new();
        for d in rows {
            disputes.push(d.map_err(|e| IndexerError::Database(e.to_string()))?);
        }
        Ok(disputes)
    }

    fn record_protocol_fee(&mut self, fee: ProtocolFeeRecord) -> Result<(), IndexerError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO protocol_fees (
                id, circle_id, gross_payout_stroops, fee_bps, fee_stroops, net_payout_stroops, treasury_address
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                fee.id,
                fee.circle_id,
                fee.gross_payout_stroops,
                fee.fee_bps,
                fee.fee_stroops,
                fee.net_payout_stroops,
                fee.treasury_address,
            ],
        ).map_err(|e| IndexerError::Database(e.to_string()))?;
        Ok(())
    }

    fn get_protocol_fees_for_circle(&self, circle_id: &str) -> Result<Vec<ProtocolFeeRecord>, IndexerError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, circle_id, gross_payout_stroops, fee_bps, fee_stroops, net_payout_stroops, treasury_address
               FROM protocol_fees WHERE circle_id = ?1 ORDER BY settled_at ASC"#
        ).map_err(|e| IndexerError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![circle_id], |row| {
            Ok(ProtocolFeeRecord {
                id: row.get(0)?,
                circle_id: row.get(1)?,
                gross_payout_stroops: row.get(2)?,
                fee_bps: row.get(3)?,
                fee_stroops: row.get(4)?,
                net_payout_stroops: row.get(5)?,
                treasury_address: row.get(6)?,
                settled_at_timestamp: 0,
            })
        }).map_err(|e| IndexerError::Database(e.to_string()))?;

        let mut fees = Vec::new();
        for f in rows {
            fees.push(f.map_err(|e| IndexerError::Database(e.to_string()))?);
        }
        Ok(fees)
    }

    fn update_last_ledger(&mut self, ledger: u64) -> Result<(), IndexerError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO indexer_checkpoint (network, latest_ledger_sequence, updated_at_timestamp)
               VALUES ('testnet', ?1, strftime('%s', 'now'))
               ON CONFLICT(network) DO UPDATE SET
                   latest_ledger_sequence = excluded.latest_ledger_sequence,
                   updated_at_timestamp = excluded.updated_at_timestamp"#,
            params![ledger as i64],
        ).map_err(|e| IndexerError::Database(e.to_string()))?;
        Ok(())
    }

    fn list_active_circles(&self) -> Result<Vec<DemandCircleRecord>, IndexerError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, title, category, target_unit_price_xlm, min_volume, max_volume, current_volume, status, creator_address, deadline_timestamp
               FROM demand_circles WHERE status IN ('OPEN', 'QUORUM_REACHED', 'INSPECTION')
               ORDER BY created_at DESC"#
        ).map_err(|e| IndexerError::Database(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            Ok(DemandCircleRecord {
                id: row.get(0)?,
                title: row.get(1)?,
                category: row.get(2)?,
                target_unit_price_xlm: row.get(3)?,
                min_volume: row.get(4)?,
                max_volume: row.get(5)?,
                current_volume: row.get(6)?,
                status: row.get(7)?,
                creator_address: row.get(8)?,
                deadline_timestamp: row.get(9)?,
            })
        }).map_err(|e| IndexerError::Database(e.to_string()))?;

        let mut circles = Vec::new();
        for c in rows {
            circles.push(c.map_err(|e| IndexerError::Database(e.to_string()))?);
        }
        Ok(circles)
    }

    fn list_top_sellers(&self, limit: usize) -> Result<Vec<SellerReputationRecord>, IndexerError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT seller_address, successful_deals, total_volume_settled, total_amount_settled_xlm, disputed_or_refunded_deals, score_percentage
               FROM seller_reputations ORDER BY successful_deals DESC, score_percentage DESC LIMIT ?1"#
        ).map_err(|e| IndexerError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let volume: i64 = row.get(2)?;
            Ok(SellerReputationRecord {
                seller_address: row.get(0)?,
                successful_deals: row.get(1)?,
                total_volume_settled: volume as u64,
                total_amount_settled_xlm: row.get(3)?,
                disputed_or_refunded_deals: row.get(4)?,
                score_percentage: row.get(5)?,
            })
        }).map_err(|e| IndexerError::Database(e.to_string()))?;

        let mut reps = Vec::new();
        for r in rows {
            reps.push(r.map_err(|e| IndexerError::Database(e.to_string()))?);
        }
        Ok(reps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_repository_circle_and_commitment_lifecycle() {
        let mut repo = SqliteBazaarRepository::new_in_memory().unwrap();

        let circle = DemandCircleRecord {
            id: "circle_100".to_string(),
            title: "Agricultural Fertilizer Bulk Demand".to_string(),
            category: "Agriculture".to_string(),
            target_unit_price_xlm: "50.0".to_string(),
            min_volume: 50,
            max_volume: 200,
            current_volume: 0,
            status: "OPEN".to_string(),
            creator_address: "GBCREATOR123".to_string(),
            deadline_timestamp: 1899999999,
        };

        repo.record_circle(circle.clone()).unwrap();
        let fetched = repo.get_circle("circle_100").unwrap().unwrap();
        assert_eq!(fetched.title, "Agricultural Fertilizer Bulk Demand");
        assert_eq!(fetched.min_volume, 50);

        // Buyer commitment
        let commitment = CommitmentRecord {
            id: "comm_1".to_string(),
            circle_id: "circle_100".to_string(),
            buyer_address: "GBUYER1".to_string(),
            quantity: 25,
            committed_amount_xlm: "1250.0".to_string(),
            escrow_tx_hash: Some("tx_abc123".to_string()),
            status: "ESCROWED".to_string(),
        };
        repo.record_commitment(commitment).unwrap();

        let commitments = repo.get_commitments_for_circle("circle_100").unwrap();
        assert_eq!(commitments.len(), 1);
        assert_eq!(commitments[0].buyer_address, "GBUYER1");
        assert_eq!(commitments[0].quantity, 25);
    }

    #[test]
    fn test_sqlite_repository_dispute_and_checkpoint_lifecycle() {
        let mut repo = SqliteBazaarRepository::new_in_memory().unwrap();

        // Checkpoint starts at 0
        assert_eq!(repo.get_last_ledger().unwrap(), 0);

        repo.update_last_ledger(987654).unwrap();
        assert_eq!(repo.get_last_ledger().unwrap(), 987654);

        // Record a circle first to satisfy foreign key constraints
        let circle = DemandCircleRecord {
            id: "circle_200".to_string(),
            title: "Industrial Machinery Parts".to_string(),
            category: "Industrial".to_string(),
            target_unit_price_xlm: "500.0".to_string(),
            min_volume: 10,
            max_volume: 50,
            current_volume: 10,
            status: "INSPECTION".to_string(),
            creator_address: "GBINDUS1".to_string(),
            deadline_timestamp: 1899999999,
        };
        repo.record_circle(circle).unwrap();

        // Record dispute
        let dispute = DisputeRecord {
            id: "disp_99".to_string(),
            circle_id: "circle_200".to_string(),
            initiator_address: "GBUYER1".to_string(),
            dispute_reason: "WRONG_SPECIFICATION".to_string(),
            status: "OPEN".to_string(),
            resolution_refund_to_buyers: None,
            opened_at_timestamp: 1700000000,
            resolved_at_timestamp: None,
        };
        repo.record_dispute(dispute).unwrap();

        let disputes = repo.get_disputes_for_circle("circle_200").unwrap();
        assert_eq!(disputes.len(), 1);
        assert_eq!(disputes[0].dispute_reason, "WRONG_SPECIFICATION");

        // Record protocol fee
        let fee = ProtocolFeeRecord {
            id: "fee_99".to_string(),
            circle_id: "circle_200".to_string(),
            gross_payout_stroops: "5000000000".to_string(),
            fee_bps: 200,
            fee_stroops: "100000000".to_string(),
            net_payout_stroops: "4900000000".to_string(),
            treasury_address: "GTREASURY".to_string(),
            settled_at_timestamp: 1700003600,
        };
        repo.record_protocol_fee(fee).unwrap();

        let fees = repo.get_protocol_fees_for_circle("circle_200").unwrap();
        assert_eq!(fees.len(), 1);
        assert_eq!(fees[0].fee_bps, 200);
    }

    #[test]
    fn test_sqlite_repository_queries_active_circles_and_top_sellers() {
        let mut repo = SqliteBazaarRepository::new_in_memory().unwrap();

        let c1 = DemandCircleRecord {
            id: "c1".to_string(),
            title: "Circle 1".to_string(),
            category: "Cat1".to_string(),
            target_unit_price_xlm: "10.0".to_string(),
            min_volume: 5,
            max_volume: 20,
            current_volume: 5,
            status: "OPEN".to_string(),
            creator_address: "G1".to_string(),
            deadline_timestamp: 1800000000,
        };
        let c2 = DemandCircleRecord {
            id: "c2".to_string(),
            title: "Circle 2".to_string(),
            category: "Cat2".to_string(),
            target_unit_price_xlm: "20.0".to_string(),
            min_volume: 10,
            max_volume: 50,
            current_volume: 10,
            status: "INSPECTION".to_string(),
            creator_address: "G2".to_string(),
            deadline_timestamp: 1800000000,
        };
        let c3 = DemandCircleRecord {
            id: "c3".to_string(),
            title: "Circle 3".to_string(),
            category: "Cat3".to_string(),
            target_unit_price_xlm: "30.0".to_string(),
            min_volume: 10,
            max_volume: 50,
            current_volume: 0,
            status: "CANCELLED".to_string(),
            creator_address: "G3".to_string(),
            deadline_timestamp: 1800000000,
        };

        repo.record_circle(c1).unwrap();
        repo.record_circle(c2).unwrap();
        repo.record_circle(c3).unwrap();

        let active = repo.list_active_circles().unwrap();
        assert_eq!(active.len(), 2);
        assert!(active.iter().any(|c| c.id == "c1"));
        assert!(active.iter().any(|c| c.id == "c2"));
        assert!(!active.iter().any(|c| c.id == "c3"));

        // Sellers
        let s1 = SellerReputationRecord {
            seller_address: "SELLER_A".to_string(),
            successful_deals: 5,
            total_volume_settled: 100,
            total_amount_settled_xlm: "5000.0".to_string(),
            disputed_or_refunded_deals: 0,
            score_percentage: 95,
        };
        let s2 = SellerReputationRecord {
            seller_address: "SELLER_B".to_string(),
            successful_deals: 12,
            total_volume_settled: 300,
            total_amount_settled_xlm: "15000.0".to_string(),
            disputed_or_refunded_deals: 0,
            score_percentage: 99,
        };

        repo.record_reputation(s1).unwrap();
        repo.record_reputation(s2).unwrap();

        let top_sellers = repo.list_top_sellers(5).unwrap();
        assert_eq!(top_sellers.len(), 2);
        assert_eq!(top_sellers[0].seller_address, "SELLER_B");
        assert_eq!(top_sellers[1].seller_address, "SELLER_A");
    }
}
