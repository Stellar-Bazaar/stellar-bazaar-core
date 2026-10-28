use crate::domain::*;
use crate::event_decoder::BazaarContractEvent;
use crate::{BazaarRepository, IndexerError};

/// Projections engine applying decoded contract events into repository state
pub struct EventProcessor;

impl EventProcessor {
    /// Applies a decoded contract event into the target repository
    pub fn process_event<R: BazaarRepository>(
        repo: &mut R,
        event: BazaarContractEvent,
        ledger: u64,
        tx_hash: &str,
    ) -> Result<(), IndexerError> {
        match event {
            BazaarContractEvent::CircleCreated {
                circle_id,
                creator,
                target_unit_price,
                min_volume,
                max_volume,
                deadline,
            } => {
                let record = DemandCircleRecord {
                    id: circle_id.to_string(),
                    title: format!("Circle #{}", circle_id),
                    category: "General".to_string(),
                    target_unit_price_xlm: (target_unit_price as f64 / 10_000_000.0).to_string(),
                    min_volume,
                    max_volume,
                    current_volume: 0,
                    status: "OPEN".to_string(),
                    creator_address: creator,
                    deadline_timestamp: deadline as i64,
                };
                repo.record_circle(record)?;
            }
            BazaarContractEvent::DemandCommitted {
                circle_id,
                buyer,
                quantity,
                amount,
            } => {
                let id = format!("{}_{}", circle_id, buyer);
                let record = CommitmentRecord {
                    id,
                    circle_id: circle_id.to_string(),
                    buyer_address: buyer,
                    quantity,
                    committed_amount_xlm: (amount as f64 / 10_000_000.0).to_string(),
                    escrow_tx_hash: Some(tx_hash.to_string()),
                    status: "ESCROWED".to_string(),
                };
                repo.record_commitment(record)?;

                // Update circle volume and trigger quorum if threshold reached
                if let Some(mut circle) = repo.get_circle(&circle_id.to_string())? {
                    circle.current_volume += quantity;
                    if circle.current_volume >= circle.min_volume && circle.status == "OPEN" {
                        circle.status = "QUORUM_REACHED".to_string();
                    }
                    repo.record_circle(circle)?;
                }
            }
            BazaarContractEvent::OfferSubmitted {
                offer_id,
                circle_id,
                seller,
                unit_price,
                volume,
                lead_time_days,
            } => {
                let record = SellerOfferRecord {
                    id: offer_id.to_string(),
                    circle_id: circle_id.to_string(),
                    seller_address: seller,
                    offered_unit_price_xlm: (unit_price as f64 / 10_000_000.0).to_string(),
                    available_volume: volume,
                    lead_time_days,
                    status: "SUBMITTED".to_string(),
                };
                repo.record_offer(record)?;
            }
            BazaarContractEvent::InspectionStarted { circle_id, .. } => {
                if let Some(mut circle) = repo.get_circle(&circle_id.to_string())? {
                    circle.status = "INSPECTION".to_string();
                    repo.record_circle(circle)?;
                }
            }
            BazaarContractEvent::DisputeOpened {
                circle_id,
                caller,
                reason,
            } => {
                let disp_id = format!("disp_{}_{}", circle_id, caller);
                let record = DisputeRecord {
                    id: disp_id,
                    circle_id: circle_id.to_string(),
                    initiator_address: caller,
                    dispute_reason: reason,
                    status: "OPEN".to_string(),
                    resolution_refund_to_buyers: None,
                    opened_at_timestamp: ledger as i64,
                    resolved_at_timestamp: None,
                };
                repo.record_dispute(record)?;

                if let Some(mut circle) = repo.get_circle(&circle_id.to_string())? {
                    circle.status = "DISPUTED".to_string();
                    repo.record_circle(circle)?;
                }
            }
            BazaarContractEvent::DisputeResolved {
                circle_id,
                refund_to_buyers,
                ..
            } => {
                if let Some(mut circle) = repo.get_circle(&circle_id.to_string())? {
                    circle.status = if refund_to_buyers { "CANCELLED" } else { "SETTLED" }.to_string();
                    repo.record_circle(circle)?;
                }
            }
            BazaarContractEvent::DealSettled {
                circle_id,
                winning_offer_id,
                seller_payout,
            } => {
                if let Some(mut circle) = repo.get_circle(&circle_id.to_string())? {
                    circle.status = "SETTLED".to_string();
                    repo.record_circle(circle)?;
                }
                let settlement = SettlementRecord {
                    id: format!("settle_{}", circle_id),
                    circle_id: circle_id.to_string(),
                    winning_offer_id: winning_offer_id.to_string(),
                    total_volume_cleared: 0,
                    total_payout_xlm: (seller_payout as f64 / 10_000_000.0).to_string(),
                    contract_call_tx_hash: tx_hash.to_string(),
                    ledger_sequence: ledger,
                };
                repo.record_settlement(settlement)?;
            }
            BazaarContractEvent::SurplusRebateClaimed { .. } => {
                // Verified in dispute and rebate storage
            }
        }
        Ok(())
    }
}
