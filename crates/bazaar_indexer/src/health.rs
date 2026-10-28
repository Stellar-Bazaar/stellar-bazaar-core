use crate::state::IndexerDaemonState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Syncing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
    pub uptime_seconds: u64,
    pub latest_ledger: u64,
    pub total_events_indexed: u64,
    pub total_circles_indexed: u64,
    pub total_disputes_indexed: u64,
    pub consecutive_errors: u32,
    pub last_error: Option<String>,
}

pub struct HealthChecker;

impl HealthChecker {
    pub fn evaluate(state: &IndexerDaemonState, consecutive_errors: u32) -> HealthCheckResponse {
        let status = if consecutive_errors >= 5 {
            HealthStatus::Degraded
        } else if state.total_events_indexed == 0 && state.latest_ledger == 0 {
            HealthStatus::Syncing
        } else {
            HealthStatus::Healthy
        };

        HealthCheckResponse {
            status,
            uptime_seconds: state.uptime_seconds(),
            latest_ledger: state.latest_ledger,
            total_events_indexed: state.total_events_indexed,
            total_circles_indexed: state.total_circles_indexed,
            total_disputes_indexed: state.total_disputes_indexed,
            consecutive_errors,
            last_error: state.last_error.clone(),
        }
    }
}
