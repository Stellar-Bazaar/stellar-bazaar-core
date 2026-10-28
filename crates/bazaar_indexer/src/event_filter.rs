use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BazaarEventType {
    CircleCreated,
    CircleClosed,
    CircleExpired,
    DemandCommitted,
    OfferSubmitted,
    OfferAccepted,
    DealSettled,
    RefundClaimed,
    Unknown(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SorobanRpcEventFilter {
    pub event_type: String,
    pub contract_ids: Vec<String>,
    pub topics: Vec<Vec<String>>,
}

impl SorobanRpcEventFilter {
    /// Builds a filter for a specific contract address and event topics
    pub fn new(contract_id: &str, topics: Vec<Vec<String>>) -> Self {
        Self {
            event_type: "contract".to_string(),
            contract_ids: vec![contract_id.to_string()],
            topics,
        }
    }

    /// Builds the standard Stellar Bazaar filter covering both Registry and DealEngine contracts
    pub fn for_bazaar(registry_id: &str, deal_engine_id: &str) -> Vec<Self> {
        vec![
            Self::new(
                registry_id,
                vec![vec!["circle".to_string()]],
            ),
            Self::new(
                deal_engine_id,
                vec![
                    vec!["demand".to_string(), "commit".to_string()],
                    vec!["offer".to_string()],
                    vec!["deal".to_string(), "settled".to_string()],
                    vec!["refund".to_string(), "claimed".to_string()],
                ],
            ),
        ]
    }
}

/// Identifies event type from two-part Soroban topic strings
pub fn parse_event_topic(topic_0: &str, topic_1: Option<&str>) -> BazaarEventType {
    match (topic_0, topic_1) {
        ("circle", Some("created")) => BazaarEventType::CircleCreated,
        ("circle", Some("closed")) => BazaarEventType::CircleClosed,
        ("circle", Some("expired")) => BazaarEventType::CircleExpired,
        ("demand", Some("commit")) => BazaarEventType::DemandCommitted,
        ("offer", Some("submit")) => BazaarEventType::OfferSubmitted,
        ("offer", Some("accept")) => BazaarEventType::OfferAccepted,
        ("deal", Some("settled")) => BazaarEventType::DealSettled,
        ("refund", Some("claimed")) => BazaarEventType::RefundClaimed,
        (other, _) => BazaarEventType::Unknown(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_event_topics() {
        assert_eq!(
            parse_event_topic("circle", Some("created")),
            BazaarEventType::CircleCreated
        );
        assert_eq!(
            parse_event_topic("demand", Some("commit")),
            BazaarEventType::DemandCommitted
        );
        assert_eq!(
            parse_event_topic("deal", Some("settled")),
            BazaarEventType::DealSettled
        );
        assert_eq!(
            parse_event_topic("refund", Some("claimed")),
            BazaarEventType::RefundClaimed
        );
        assert_eq!(
            parse_event_topic("custom", Some("action")),
            BazaarEventType::Unknown("custom".to_string())
        );
    }

    #[test]
    fn test_filter_generation() {
        let filters = SorobanRpcEventFilter::for_bazaar("CCBY...", "CDCK...");
        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0].contract_ids[0], "CCBY...");
        assert_eq!(filters[1].contract_ids[0], "CDCK...");
    }
}
