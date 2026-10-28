use crate::IndexerError;
use serde::{Deserialize, Serialize};

/// High-level strongly-typed representations of decoded Soroban contract events
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BazaarContractEvent {
    CircleCreated {
        circle_id: u64,
        creator: String,
        target_unit_price: i128,
        min_volume: u32,
        max_volume: u32,
        deadline: u64,
    },
    DemandCommitted {
        circle_id: u64,
        buyer: String,
        quantity: u32,
        amount: i128,
    },
    OfferSubmitted {
        offer_id: u64,
        circle_id: u64,
        seller: String,
        unit_price: i128,
        volume: u32,
        lead_time_days: u32,
    },
    InspectionStarted {
        circle_id: u64,
        caller: String,
        inspection_deadline: u64,
    },
    DisputeOpened {
        circle_id: u64,
        caller: String,
        reason: String,
    },
    DisputeResolved {
        circle_id: u64,
        caller: String,
        refund_to_buyers: bool,
    },
    DealSettled {
        circle_id: u64,
        winning_offer_id: u64,
        seller_payout: i128,
    },
    SurplusRebateClaimed {
        circle_id: u64,
        buyer: String,
        rebate_amount: i128,
    },
}

/// Decodes topic symbols and JSON payload into a structured BazaarContractEvent
pub fn decode_event(
    topic0: &str,
    topic1: &str,
    payload: &serde_json::Value,
) -> Result<Option<BazaarContractEvent>, IndexerError> {
    match (topic0, topic1) {
        ("circle", "create") => {
            let circle_id = payload.get("circle_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let creator = payload.get("creator").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let target_unit_price = payload.get("target_unit_price").and_then(|v| v.as_i64()).unwrap_or(0) as i128;
            let min_volume = payload.get("min_volume").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let max_volume = payload.get("max_volume").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let deadline = payload.get("deadline").and_then(|v| v.as_u64()).unwrap_or(0);

            Ok(Some(BazaarContractEvent::CircleCreated {
                circle_id,
                creator,
                target_unit_price,
                min_volume,
                max_volume,
                deadline,
            }))
        }
        ("demand", "commit") => {
            let circle_id = payload.get("circle_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let buyer = payload.get("buyer").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let quantity = payload.get("quantity").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let amount = payload.get("amount").and_then(|v| v.as_i64()).unwrap_or(0) as i128;

            Ok(Some(BazaarContractEvent::DemandCommitted {
                circle_id,
                buyer,
                quantity,
                amount,
            }))
        }
        ("offer", "submit") => {
            let offer_id = payload.get("offer_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let circle_id = payload.get("circle_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let seller = payload.get("seller").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let unit_price = payload.get("unit_price").and_then(|v| v.as_i64()).unwrap_or(0) as i128;
            let volume = payload.get("volume").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let lead_time_days = payload.get("lead_time_days").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

            Ok(Some(BazaarContractEvent::OfferSubmitted {
                offer_id,
                circle_id,
                seller,
                unit_price,
                volume,
                lead_time_days,
            }))
        }
        ("inspect", "start") => {
            let circle_id = payload.get("circle_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let caller = payload.get("caller").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let inspection_deadline = payload.get("inspection_deadline").and_then(|v| v.as_u64()).unwrap_or(0);

            Ok(Some(BazaarContractEvent::InspectionStarted {
                circle_id,
                caller,
                inspection_deadline,
            }))
        }
        ("dispute", "opened") => {
            let circle_id = payload.get("circle_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let caller = payload.get("caller").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let reason = payload.get("reason").and_then(|v| v.as_str()).unwrap_or("").to_string();

            Ok(Some(BazaarContractEvent::DisputeOpened {
                circle_id,
                caller,
                reason,
            }))
        }
        ("dispute", "resolve") => {
            let circle_id = payload.get("circle_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let caller = payload.get("caller").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let refund_to_buyers = payload.get("refund_to_buyers").and_then(|v| v.as_bool()).unwrap_or(false);

            Ok(Some(BazaarContractEvent::DisputeResolved {
                circle_id,
                caller,
                refund_to_buyers,
            }))
        }
        ("deal", "settled") => {
            let circle_id = payload.get("circle_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let winning_offer_id = payload.get("winning_offer_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let seller_payout = payload.get("seller_payout").and_then(|v| v.as_i64()).unwrap_or(0) as i128;

            Ok(Some(BazaarContractEvent::DealSettled {
                circle_id,
                winning_offer_id,
                seller_payout,
            }))
        }
        ("rebate", "claimed") => {
            let circle_id = payload.get("circle_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let buyer = payload.get("buyer").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let rebate_amount = payload.get("rebate_amount").and_then(|v| v.as_i64()).unwrap_or(0) as i128;

            Ok(Some(BazaarContractEvent::SurplusRebateClaimed {
                circle_id,
                buyer,
                rebate_amount,
            }))
        }
        _ => Ok(None),
    }
}
