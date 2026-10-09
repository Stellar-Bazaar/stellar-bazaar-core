pub mod domain;

use domain::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Horizon RPC error: {0}")]
    Rpc(String),
    #[error("Deserialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Ledger gap detected: expected {expected}, found {found}")]
    LedgerGap { expected: u64, found: u64 },
}

/// Abstract storage boundary for the indexer
pub trait BazaarRepository: Send + Sync {
    fn record_circle(&mut self, circle: DemandCircleRecord) -> Result<(), IndexerError>;
    fn record_commitment(&mut self, commitment: CommitmentRecord) -> Result<(), IndexerError>;
    fn record_offer(&mut self, offer: SellerOfferRecord) -> Result<(), IndexerError>;
    fn record_settlement(&mut self, settlement: SettlementRecord) -> Result<(), IndexerError>;
    fn get_last_ledger(&self) -> Result<u64, IndexerError>;
    fn update_last_ledger(&mut self, ledger: u64) -> Result<(), IndexerError>;
}

/// In-memory implementation of the repository boundary for testing and local operation
#[derive(Default)]
pub struct InMemoryBazaarRepository {
    pub circles: Vec<DemandCircleRecord>,
    pub commitments: Vec<CommitmentRecord>,
    pub offers: Vec<SellerOfferRecord>,
    pub settlements: Vec<SettlementRecord>,
    pub last_ledger: u64,
}

impl InMemoryBazaarRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BazaarRepository for InMemoryBazaarRepository {
    fn record_circle(&mut self, circle: DemandCircleRecord) -> Result<(), IndexerError> {
        self.circles.push(circle);
        Ok(())
    }

    fn record_commitment(&mut self, commitment: CommitmentRecord) -> Result<(), IndexerError> {
        self.commitments.push(commitment);
        Ok(())
    }

    fn record_offer(&mut self, offer: SellerOfferRecord) -> Result<(), IndexerError> {
        self.offers.push(offer);
        Ok(())
    }

    fn record_settlement(&mut self, settlement: SettlementRecord) -> Result<(), IndexerError> {
        self.settlements.push(settlement);
        Ok(())
    }

    fn get_last_ledger(&self) -> Result<u64, IndexerError> {
        Ok(self.last_ledger)
    }

    fn update_last_ledger(&mut self, ledger: u64) -> Result<(), IndexerError> {
        self.last_ledger = ledger;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_repository_lifecycle() {
        let mut repo = InMemoryBazaarRepository::new();
        assert_eq!(repo.get_last_ledger().unwrap(), 0);

        repo.update_last_ledger(12345).unwrap();
        assert_eq!(repo.get_last_ledger().unwrap(), 12345);

        let circle = DemandCircleRecord {
            id: "circle_1".to_string(),
            title: "Solar Panels Bulk Buy".to_string(),
            category: "Clean Energy".to_string(),
            target_unit_price_xlm: "150.0".to_string(),
            min_volume: 20,
            max_volume: 100,
            current_volume: 0,
            status: "OPEN".to_string(),
            creator_address: "GBBAZAAR...1".to_string(),
            deadline_timestamp: 1893456000,
        };

        repo.record_circle(circle.clone()).unwrap();
        assert_eq!(repo.circles.len(), 1);
        assert_eq!(repo.circles[0].title, "Solar Panels Bulk Buy");
    }
}
