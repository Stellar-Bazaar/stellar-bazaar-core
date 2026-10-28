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

    fn record_reputation(&mut self, _reputation: SellerReputationRecord) -> Result<(), IndexerError> {
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

    fn get_seller_reputation(&self, _seller: &str) -> Result<Option<SellerReputationRecord>, IndexerError> {
        Ok(None)
    }

    fn get_last_ledger(&self) -> Result<u64, IndexerError> {
        Ok(0)
    }

    fn update_last_ledger(&mut self, _ledger: u64) -> Result<(), IndexerError> {
        Ok(())
    }
}
