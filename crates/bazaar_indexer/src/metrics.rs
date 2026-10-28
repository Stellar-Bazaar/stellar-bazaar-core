use crate::health::{HealthChecker, HealthStatus};
use crate::state::IndexerDaemonState;

/// Prometheus text format metrics generator
pub struct PrometheusExporter;

impl PrometheusExporter {
    pub fn render(state: &IndexerDaemonState, consecutive_errors: u32) -> String {
        let health = HealthChecker::evaluate(state, consecutive_errors);
        let health_val = if health.status == HealthStatus::Healthy { 1 } else { 0 };

        format!(
            "# HELP bazaar_indexer_latest_ledger Latest processed Soroban ledger sequence\n\
             # TYPE bazaar_indexer_latest_ledger gauge\n\
             bazaar_indexer_latest_ledger {}\n\
             \n\
             # HELP bazaar_indexer_events_total Total contract events ingested and decoded\n\
             # TYPE bazaar_indexer_events_total counter\n\
             bazaar_indexer_events_total {}\n\
             \n\
             # HELP bazaar_indexer_circles_total Total demand circles indexed\n\
             # TYPE bazaar_indexer_circles_total counter\n\
             bazaar_indexer_circles_total {}\n\
             \n\
             # HELP bazaar_indexer_disputes_total Total disputes recorded\n\
             # TYPE bazaar_indexer_disputes_total counter\n\
             bazaar_indexer_disputes_total {}\n\
             \n\
             # HELP bazaar_indexer_uptime_seconds Total daemon process uptime in seconds\n\
             # TYPE bazaar_indexer_uptime_seconds gauge\n\
             bazaar_indexer_uptime_seconds {}\n\
             \n\
             # HELP bazaar_indexer_health Health status of the daemon (1 = healthy, 0 = unhealthy)\n\
             # TYPE bazaar_indexer_health gauge\n\
             bazaar_indexer_health {}\n",
            state.latest_ledger,
            state.total_events_indexed,
            state.total_circles_indexed,
            state.total_disputes_indexed,
            state.uptime_seconds(),
            health_val
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_metrics_rendering() {
        let mut state = IndexerDaemonState::new();
        state.record_batch_success(999_120, 150, 20, 3);

        let output = PrometheusExporter::render(&state, 0);
        assert!(output.contains("# TYPE bazaar_indexer_latest_ledger gauge"));
        assert!(output.contains("bazaar_indexer_latest_ledger 999120"));
        assert!(output.contains("# TYPE bazaar_indexer_events_total counter"));
        assert!(output.contains("bazaar_indexer_events_total 150"));
        assert!(output.contains("# TYPE bazaar_indexer_circles_total counter"));
        assert!(output.contains("bazaar_indexer_circles_total 20"));
        assert!(output.contains("# TYPE bazaar_indexer_disputes_total counter"));
        assert!(output.contains("bazaar_indexer_disputes_total 3"));
        assert!(output.contains("bazaar_indexer_health 1"));

        // When degraded, health should be 0
        let degraded_output = PrometheusExporter::render(&state, 6);
        assert!(degraded_output.contains("bazaar_indexer_health 0"));
    }
}
