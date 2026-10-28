use serde::{Deserialize, Serialize};

/// Soroban JSON-RPC filter parameter for getEvents
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RpcEventFilter {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(rename = "contractIds")]
    pub contract_ids: Vec<String>,
}

/// Pagination parameter for getEvents
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RpcPagination {
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Parameters for Soroban getEvents JSON-RPC query
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetEventsParams {
    #[serde(rename = "startLedger")]
    pub start_ledger: u64,
    pub filters: Vec<RpcEventFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<RpcPagination>,
}

/// Standard JSON-RPC 2.0 request container
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: T,
}

/// Raw event item returned in getEvents result
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SorobanRawEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub ledger: u64,
    #[serde(rename = "ledgerClosedAt")]
    pub ledger_closed_at: String,
    #[serde(rename = "contractId")]
    pub contract_id: String,
    pub id: String,
    #[serde(rename = "pagingToken")]
    pub paging_token: String,
    pub topic: Vec<String>,
    pub value: serde_json::Value,
    #[serde(rename = "inSuccessfulContractCall")]
    pub in_successful_contract_call: bool,
}

/// Result payload of getEvents call
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetEventsResult {
    #[serde(rename = "latestLedger")]
    pub latest_ledger: u64,
    pub events: Vec<SorobanRawEvent>,
    pub cursor: Option<String>,
}

/// Complete JSON-RPC 2.0 response wrapper for getEvents
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetEventsResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<GetEventsResult>,
    pub error: Option<serde_json::Value>,
}

/// Constructs a typed getEvents JSON-RPC request
pub fn build_get_events_request(
    id: u64,
    start_ledger: u64,
    contract_ids: Vec<String>,
    limit: u32,
) -> JsonRpcRequest<GetEventsParams> {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id,
        method: "getEvents".to_string(),
        params: GetEventsParams {
            start_ledger,
            filters: vec![RpcEventFilter {
                event_type: "contract".to_string(),
                contract_ids,
            }],
            pagination: Some(RpcPagination {
                limit,
                cursor: None,
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_get_events_request_serialization() {
        let req = build_get_events_request(
            1,
            150000,
            vec!["CDCK6A2QB5QTILDWILUQGAUWXI543TK63WN2OKG7WEWHBAMGZ6ESKY4G".to_string()],
            50,
        );

        let json_str = serde_json::to_string(&req).unwrap();
        assert!(json_str.contains("\"method\":\"getEvents\""));
        assert!(json_str.contains("\"startLedger\":150000"));
        assert!(json_str.contains("\"contractIds\":[\"CDCK6A2QB5QTILDWILUQGAUWXI543TK63WN2OKG7WEWHBAMGZ6ESKY4G\"]"));
        assert!(json_str.contains("\"limit\":50"));
    }

    #[test]
    fn test_get_events_response_deserialization() {
        let raw_json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "latestLedger": 150005,
                "cursor": "150000-0000000001",
                "events": [
                    {
                        "type": "contract",
                        "ledger": 150001,
                        "ledgerClosedAt": "2024-02-15T12:00:00Z",
                        "contractId": "CDCK6A2QB5QTILDWILUQGAUWXI543TK63WN2OKG7WEWHBAMGZ6ESKY4G",
                        "id": "0000150001-0000000001",
                        "pagingToken": "150001-0000000001",
                        "topic": ["AAAADwAAAANkaXIAAA==", "AAAADwAAAANzZXR0bGU="],
                        "value": {"circle_id": 10, "winning_offer_id": 2, "seller_payout": 50000000},
                        "inSuccessfulContractCall": true
                    }
                ]
            }
        }"#;

        let resp: GetEventsResponse = serde_json::from_str(raw_json).unwrap();
        assert!(resp.error.is_none());
        let res = resp.result.unwrap();
        assert_eq!(res.latest_ledger, 150005);
        assert_eq!(res.events.len(), 1);
        let event = &res.events[0];
        assert_eq!(event.ledger, 150001);
        assert_eq!(event.paging_token, "150001-0000000001");
        assert!(event.in_successful_contract_call);
    }
}
