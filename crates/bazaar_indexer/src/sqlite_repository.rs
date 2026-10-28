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

    fn record_commitment(&mut self, _commitment: CommitmentRecord) -> Result<(), IndexerError> {
        Ok(())
    }

    fn record_offer(&mut self, _offer: SellerOfferRecord) -> Result<(), IndexerError> {
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

    fn get_offers_for_circle(&self, _circle_id: &str) -> Result<Vec<SellerOfferRecord>, IndexerError> {
        Ok(Vec::new())
    }

    fn get_commitments_for_circle(&self, _circle_id: &str) -> Result<Vec<CommitmentRecord>, IndexerError> {
        Ok(Vec::new())
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
