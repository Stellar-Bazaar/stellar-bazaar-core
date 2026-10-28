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
