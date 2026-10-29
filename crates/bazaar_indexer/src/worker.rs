use crate::config::IndexerConfig;
use crate::state::IndexerDaemonState;
use std::time::Duration;

/// Polling worker orchestrating Soroban RPC event loops and resilience logic
pub struct IndexerWorker {
    pub config: IndexerConfig,
    pub state: IndexerDaemonState,
    pub consecutive_errors: u32,
}

impl IndexerWorker {
    pub fn new(config: IndexerConfig) -> Self {
        Self {
            config,
            state: IndexerDaemonState::new(),
            consecutive_errors: 0,
        }
    }

    /// Computes exponential backoff sleep duration capped at 30 seconds
    pub fn calculate_backoff(&self) -> Duration {
        if self.consecutive_errors == 0 {
            Duration::from_millis(self.config.poll_interval_ms)
        } else {
            let mult = 2u64.pow(self.consecutive_errors.min(5));
            let delay_ms = (self.config.poll_interval_ms * mult).min(30_000);
            Duration::from_millis(delay_ms)
        }
    }

    /// Handles successful polling iteration
    pub fn on_poll_success(&mut self, latest_ledger: u64, events_count: u64) {
        self.consecutive_errors = 0;
        self.state
            .record_batch_success(latest_ledger, events_count, 0, 0);
    }

    /// Handles failure and increments backoff counter
    pub fn on_poll_failure(&mut self, error_msg: String) {
        self.consecutive_errors = self.consecutive_errors.saturating_add(1);
        self.state.record_error(error_msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_exponential_backoff_and_recovery() {
        let mut config = IndexerConfig::local_testnet();
        config.poll_interval_ms = 1_000;
        let mut worker = IndexerWorker::new(config);

        // 0 errors -> base interval 1000ms
        assert_eq!(worker.calculate_backoff(), Duration::from_millis(1_000));

        // 1 error -> 1000 * 2^1 = 2000ms
        worker.on_poll_failure("RPC timeout".to_string());
        assert_eq!(worker.consecutive_errors, 1);
        assert_eq!(worker.calculate_backoff(), Duration::from_millis(2_000));

        // 2 errors -> 1000 * 2^2 = 4000ms
        worker.on_poll_failure("RPC connection reset".to_string());
        assert_eq!(worker.consecutive_errors, 2);
        assert_eq!(worker.calculate_backoff(), Duration::from_millis(4_000));

        // 5 errors -> 1000 * 2^5 = 32000ms capped at 30000ms
        worker.consecutive_errors = 5;
        assert_eq!(worker.calculate_backoff(), Duration::from_millis(30_000));

        // 10 errors -> still capped at 30000ms
        worker.consecutive_errors = 10;
        assert_eq!(worker.calculate_backoff(), Duration::from_millis(30_000));

        // Success resets backoff
        worker.on_poll_success(500, 10);
        assert_eq!(worker.consecutive_errors, 0);
        assert_eq!(worker.calculate_backoff(), Duration::from_millis(1_000));
        assert_eq!(worker.state.latest_ledger, 500);
        assert_eq!(worker.state.total_events_indexed, 10);
    }
}
