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
        self.state.record_batch_success(latest_ledger, events_count, 0, 0);
    }

    /// Handles failure and increments backoff counter
    pub fn on_poll_failure(&mut self, error_msg: String) {
        self.consecutive_errors = self.consecutive_errors.saturating_add(1);
        self.state.record_error(error_msg);
    }
}
