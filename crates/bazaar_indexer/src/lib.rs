pub mod schema;
pub mod domain;
pub mod event_filter;
pub mod sqlite_repository;
pub mod event_decoder;
pub mod config;
pub mod state;
pub mod rpc_client;
pub mod event_processor;
pub mod checkpoint;
pub mod worker;
pub mod health;
pub mod metrics;
pub mod http;
pub mod router;
pub mod controller;
pub mod server;

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

    fn record_dispute(&mut self, _dispute: DisputeRecord) -> Result<(), IndexerError> {
        Ok(())
    }
    fn get_disputes_for_circle(&self, _circle_id: &str) -> Result<Vec<DisputeRecord>, IndexerError> {
        Ok(Vec::new())
    }
    fn record_protocol_fee(&mut self, _fee: ProtocolFeeRecord) -> Result<(), IndexerError> {
        Ok(())
    }
    fn get_protocol_fees_for_circle(&self, _circle_id: &str) -> Result<Vec<ProtocolFeeRecord>, IndexerError> {
        Ok(Vec::new())
    }

    fn list_active_circles(&self) -> Result<Vec<DemandCircleRecord>, IndexerError> {
        Ok(Vec::new())
    }
    fn list_top_sellers(&self, _limit: usize) -> Result<Vec<SellerReputationRecord>, IndexerError> {
        Ok(Vec::new())
    }
}

/// In-memory implementation of the repository boundary for testing and local operation
#[derive(Default)]
pub struct InMemoryBazaarRepository {
    pub circles: Vec<DemandCircleRecord>,
    pub commitments: Vec<CommitmentRecord>,
    pub offers: Vec<SellerOfferRecord>,
    pub settlements: Vec<SettlementRecord>,
    pub reputations: Vec<SellerReputationRecord>,
    pub disputes: Vec<DisputeRecord>,
    pub protocol_fees: Vec<ProtocolFeeRecord>,
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

    fn record_dispute(&mut self, dispute: DisputeRecord) -> Result<(), IndexerError> {
        self.disputes.retain(|d| d.id != dispute.id);
        self.disputes.push(dispute);
        Ok(())
    }

    fn get_disputes_for_circle(&self, circle_id: &str) -> Result<Vec<DisputeRecord>, IndexerError> {
        Ok(self.disputes.iter().filter(|d| d.circle_id == circle_id).cloned().collect())
    }

    fn record_protocol_fee(&mut self, fee: ProtocolFeeRecord) -> Result<(), IndexerError> {
        self.protocol_fees.retain(|f| f.id != fee.id);
        self.protocol_fees.push(fee);
        Ok(())
    }

    fn get_protocol_fees_for_circle(&self, circle_id: &str) -> Result<Vec<ProtocolFeeRecord>, IndexerError> {
        Ok(self.protocol_fees.iter().filter(|f| f.circle_id == circle_id).cloned().collect())
    }

    fn list_active_circles(&self) -> Result<Vec<DemandCircleRecord>, IndexerError> {
        Ok(self.circles.iter().filter(|c| c.status == "OPEN" || c.status == "QUORUM_REACHED" || c.status == "INSPECTION").cloned().collect())
    }

    fn list_top_sellers(&self, limit: usize) -> Result<Vec<SellerReputationRecord>, IndexerError> {
        let mut sorted = self.reputations.clone();
        sorted.sort_by_key(|a| std::cmp::Reverse(a.successful_deals));
        sorted.truncate(limit);
        Ok(sorted)
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

    #[test]
    fn test_dispute_and_protocol_fee_storage_lifecycle() {
        let mut repo = InMemoryBazaarRepository::new();

        let dispute = DisputeRecord {
            id: "disp_1".to_string(),
            circle_id: "circle_42".to_string(),
            initiator_address: "GBUYER123".to_string(),
            dispute_reason: "DAMAGED_CRATE".to_string(),
            status: "OPEN".to_string(),
            resolution_refund_to_buyers: None,
            opened_at_timestamp: 1700000000,
            resolved_at_timestamp: None,
        };

        repo.record_dispute(dispute.clone()).unwrap();
        let disputes = repo.get_disputes_for_circle("circle_42").unwrap();
        assert_eq!(disputes.len(), 1);
        assert_eq!(disputes[0].dispute_reason, "DAMAGED_CRATE");
        assert_eq!(disputes[0].status, "OPEN");

        // Protocol fee record
        let fee = ProtocolFeeRecord {
            id: "fee_1".to_string(),
            circle_id: "circle_42".to_string(),
            gross_payout_stroops: "500000000".to_string(),
            fee_bps: 150,
            fee_stroops: "7500000".to_string(),
            net_payout_stroops: "492500000".to_string(),
            treasury_address: "GTREASURY99".to_string(),
            settled_at_timestamp: 1700003600,
        };

        repo.record_protocol_fee(fee).unwrap();
        let fees = repo.get_protocol_fees_for_circle("circle_42").unwrap();
        assert_eq!(fees.len(), 1);
        assert_eq!(fees[0].fee_bps, 150);
        assert_eq!(fees[0].fee_stroops, "7500000");
    }
}
