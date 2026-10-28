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
