use std::time::Instant;

/// Runtime operational state and telemetry counters for the indexer daemon
#[derive(Debug)]
pub struct IndexerDaemonState {
    pub started_at: Instant,
    pub latest_ledger: u64,
    pub total_events_indexed: u64,
    pub total_circles_indexed: u64,
    pub total_disputes_indexed: u64,
    pub last_poll_timestamp_epoch: u64,
    pub is_syncing: bool,
    pub last_error: Option<String>,
}

impl Default for IndexerDaemonState {
    fn default() -> Self {
        Self {
            started_at: Instant::now(),
            latest_ledger: 0,
            total_events_indexed: 0,
            total_circles_indexed: 0,
            total_disputes_indexed: 0,
            last_poll_timestamp_epoch: 0,
            is_syncing: false,
            last_error: None,
        }
    }
}

impl IndexerDaemonState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Elapsed time since daemon process startup in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// Updates metrics upon successful ledger batch ingestion
    pub fn record_batch_success(
        &mut self,
        ledger: u64,
        new_events: u64,
        new_circles: u64,
        new_disputes: u64,
    ) {
        if ledger > self.latest_ledger {
            self.latest_ledger = ledger;
        }
        self.total_events_indexed += new_events;
        self.total_circles_indexed += new_circles;
        self.total_disputes_indexed += new_disputes;
        self.last_error = None;
    }

    /// Records daemon exception without clearing prior high water mark
    pub fn record_error(&mut self, err: String) {
        self.last_error = Some(err);
    }

    /// Returns true if daemon has no unrecovered fatal errors
    pub fn is_healthy(&self) -> bool {
        self.last_error.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_state_lifecycle() {
        let mut state = IndexerDaemonState::new();
        assert!(state.is_healthy());
        assert_eq!(state.latest_ledger, 0);

        state.record_batch_success(124500, 5, 2, 1);
        assert_eq!(state.latest_ledger, 124500);
        assert_eq!(state.total_events_indexed, 5);
        assert_eq!(state.total_circles_indexed, 2);
        assert_eq!(state.total_disputes_indexed, 1);
        assert!(state.is_healthy());

        state.record_error("RPC connection timeout".to_string());
        assert!(!state.is_healthy());
        assert_eq!(state.last_error.as_deref(), Some("RPC connection timeout"));

        // Success clears transient error
        state.record_batch_success(124501, 1, 0, 0);
        assert!(state.is_healthy());
    }
}
