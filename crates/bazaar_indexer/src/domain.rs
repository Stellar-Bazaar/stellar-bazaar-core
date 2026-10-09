use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DemandCircleRecord {
    pub id: String,
    pub title: String,
    pub category: String,
    pub target_unit_price_xlm: String,
    pub min_volume: u32,
    pub max_volume: u32,
    pub current_volume: u32,
    pub status: String,
    pub creator_address: String,
    pub deadline_timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitmentRecord {
    pub id: String,
    pub circle_id: String,
    pub buyer_address: String,
    pub quantity: u32,
    pub committed_amount_xlm: String,
    pub escrow_tx_hash: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SellerOfferRecord {
    pub id: String,
    pub circle_id: String,
    pub seller_address: String,
    pub offered_unit_price_xlm: String,
    pub available_volume: u32,
    pub lead_time_days: u32,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettlementRecord {
    pub id: String,
    pub circle_id: String,
    pub winning_offer_id: String,
    pub total_volume_cleared: u32,
    pub total_payout_xlm: String,
    pub contract_call_tx_hash: String,
    pub ledger_sequence: u64,
}
