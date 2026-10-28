pub mod schema;
pub mod domain;
pub mod event_filter;

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
    fn record_reputation(&mut self, reputation: SellerReputationRecord) -> Result<(), IndexerError>;

    fn get_circle(&self, id: &str) -> Result<Option<DemandCircleRecord>, IndexerError>;
    fn get_offers_for_circle(&self, circle_id: &str) -> Result<Vec<SellerOfferRecord>, IndexerError>;
    fn get_commitments_for_circle(&self, circle_id: &str) -> Result<Vec<CommitmentRecord>, IndexerError>;
    fn get_seller_reputation(&self, seller: &str) -> Result<Option<SellerReputationRecord>, IndexerError>;

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
    pub reputations: Vec<SellerReputationRecord>,
    pub last_ledger: u64,
}

impl InMemoryBazaarRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BazaarRepository for InMemoryBazaarRepository {
    fn record_circle(&mut self, circle: DemandCircleRecord) -> Result<(), IndexerError> {
        self.circles.retain(|c| c.id != circle.id);
        self.circles.push(circle);
        Ok(())
    }

    fn record_commitment(&mut self, commitment: CommitmentRecord) -> Result<(), IndexerError> {
        self.commitments.retain(|c| c.id != commitment.id);
        self.commitments.push(commitment);
        Ok(())
    }

    fn record_offer(&mut self, offer: SellerOfferRecord) -> Result<(), IndexerError> {
        self.offers.retain(|o| o.id != offer.id);
        self.offers.push(offer);
        Ok(())
    }

    fn record_settlement(&mut self, settlement: SettlementRecord) -> Result<(), IndexerError> {
        self.settlements.push(settlement);
        Ok(())
    }

    fn record_reputation(&mut self, reputation: SellerReputationRecord) -> Result<(), IndexerError> {
        self.reputations.retain(|r| r.seller_address != reputation.seller_address);
        self.reputations.push(reputation);
        Ok(())
    }

    fn get_circle(&self, id: &str) -> Result<Option<DemandCircleRecord>, IndexerError> {
        Ok(self.circles.iter().find(|c| c.id == id).cloned())
    }

    fn get_offers_for_circle(&self, circle_id: &str) -> Result<Vec<SellerOfferRecord>, IndexerError> {
        Ok(self.offers.iter().filter(|o| o.circle_id == circle_id).cloned().collect())
    }

    fn get_commitments_for_circle(&self, circle_id: &str) -> Result<Vec<CommitmentRecord>, IndexerError> {
        Ok(self.commitments.iter().filter(|c| c.circle_id == circle_id).cloned().collect())
    }

    fn get_seller_reputation(&self, seller: &str) -> Result<Option<SellerReputationRecord>, IndexerError> {
        Ok(self.reputations.iter().find(|r| r.seller_address == seller).cloned())
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
        let found = repo.get_circle("circle_1").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Solar Panels Bulk Buy");

        // Record seller offer
        let offer = SellerOfferRecord {
            id: "offer_1".to_string(),
            circle_id: "circle_1".to_string(),
            seller_address: "GSELLER1".to_string(),
            offered_unit_price_xlm: "140.0".to_string(),
            available_volume: 50,
            lead_time_days: 7,
            status: "ACTIVE".to_string(),
        };
        repo.record_offer(offer).unwrap();

        let offers = repo.get_offers_for_circle("circle_1").unwrap();
        assert_eq!(offers.len(), 1);
        assert_eq!(offers[0].offered_unit_price_xlm, "140.0");

        // Record seller reputation
        let rep = SellerReputationRecord {
            seller_address: "GSELLER1".to_string(),
            successful_deals: 10,
            total_volume_settled: 500,
            total_amount_settled_xlm: "70000.0".to_string(),
            disputed_or_refunded_deals: 0,
            score_percentage: 100,
        };
        repo.record_reputation(rep).unwrap();
        let found_rep = repo.get_seller_reputation("GSELLER1").unwrap();
        assert_eq!(found_rep.unwrap().score_percentage, 100);
    }
}
