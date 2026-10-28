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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_status_evaluation() {
        let mut state = IndexerDaemonState::new();

        // 1. Initial fresh state -> Syncing
        let initial_health = HealthChecker::evaluate(&state, 0);
        assert_eq!(initial_health.status, HealthStatus::Syncing);
        assert_eq!(initial_health.latest_ledger, 0);
        assert_eq!(initial_health.total_events_indexed, 0);

        // 2. Active indexing state -> Healthy
        state.record_batch_success(100_000, 25, 5, 2);
        let healthy = HealthChecker::evaluate(&state, 0);
        assert_eq!(healthy.status, HealthStatus::Healthy);
        assert_eq!(healthy.latest_ledger, 100_000);
        assert_eq!(healthy.total_events_indexed, 25);
        assert_eq!(healthy.total_circles_indexed, 5);
        assert_eq!(healthy.total_disputes_indexed, 2);

        // 3. Transient error with < 5 consecutive errors -> Healthy
        state.record_error("Temporary 503".to_string());
        let transient_error_health = HealthChecker::evaluate(&state, 2);
        assert_eq!(transient_error_health.status, HealthStatus::Healthy);
        assert_eq!(transient_error_health.consecutive_errors, 2);

        // 4. Repeated errors >= 5 -> Degraded
        let degraded = HealthChecker::evaluate(&state, 5);
        assert_eq!(degraded.status, HealthStatus::Degraded);
        assert_eq!(degraded.consecutive_errors, 5);
        assert_eq!(degraded.last_error.as_deref(), Some("Temporary 503"));
    }

    #[test]
    fn test_health_check_serialization() {
        let mut state = IndexerDaemonState::new();
        state.record_batch_success(42, 10, 3, 1);
        let health = HealthChecker::evaluate(&state, 0);

        let json = serde_json::to_string(&health).expect("Serialization failed");
        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"latest_ledger\":42"));
        assert!(json.contains("\"total_events_indexed\":10"));
    }
}
