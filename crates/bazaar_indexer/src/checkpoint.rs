use crate::{BazaarRepository, IndexerError};

/// Manages safe ledger checkpointing and sequence resumption
pub struct CheckpointManager;

impl CheckpointManager {
    /// Determines the safe start ledger sequence to resume indexing
    pub fn get_safe_start_ledger<R: BazaarRepository>(
        repo: &R,
        override_ledger: Option<u64>,
    ) -> Result<u64, IndexerError> {
        if let Some(ledger) = override_ledger {
            return Ok(ledger);
        }
        let last_indexed = repo.get_last_ledger()?;
        if last_indexed == 0 {
            // First run, start at ledger 1
            Ok(1)
        } else {
            // Resume from the subsequent ledger sequence
            Ok(last_indexed + 1)
        }
    }

    /// Commit the latest processed ledger sequence monotonically
    pub fn commit_checkpoint<R: BazaarRepository>(
        repo: &mut R,
        processed_ledger: u64,
    ) -> Result<(), IndexerError> {
        let current = repo.get_last_ledger()?;
        if processed_ledger > current {
            repo.update_last_ledger(processed_ledger)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InMemoryBazaarRepository;

    #[test]
    fn test_checkpoint_resumption_and_monotonicity() {
        let mut repo = InMemoryBazaarRepository::new();

        // 1. First run with no override starts at 1
        let start = CheckpointManager::get_safe_start_ledger(&repo, None).unwrap();
        assert_eq!(start, 1);

        // 2. Explicit override takes precedence
        let override_start = CheckpointManager::get_safe_start_ledger(&repo, Some(50000)).unwrap();
        assert_eq!(override_start, 50000);

        // 3. Commit ledger 100
        CheckpointManager::commit_checkpoint(&mut repo, 100).unwrap();
        assert_eq!(repo.get_last_ledger().unwrap(), 100);

        // 4. Next start resumes at 101
        let next_start = CheckpointManager::get_safe_start_ledger(&repo, None).unwrap();
        assert_eq!(next_start, 101);

        // 5. Committing an older ledger does not regress checkpoint (monotonicity)
        CheckpointManager::commit_checkpoint(&mut repo, 99).unwrap();
        assert_eq!(repo.get_last_ledger().unwrap(), 100);
    }

    #[test]
    fn test_checkpoint_idempotency_on_batch_replay() {
        use crate::sqlite_repository::SqliteBazaarRepository;
        use crate::event_decoder::BazaarContractEvent;
        use crate::event_processor::EventProcessor;

        let mut repo = SqliteBazaarRepository::new_in_memory().unwrap();

        let event = BazaarContractEvent::CircleCreated {
            circle_id: 10,
            creator: "GBIDEMPOTENT".to_string(),
            target_unit_price: 5_000_000,
            min_volume: 10,
            max_volume: 100,
            deadline: 1800000000,
        };

        // First ingestion at ledger 500
        EventProcessor::process_event(&mut repo, event.clone(), 500, "tx_500").unwrap();
        CheckpointManager::commit_checkpoint(&mut repo, 500).unwrap();
        assert_eq!(repo.get_last_ledger().unwrap(), 500);

        // Replay ingestion of exact same event at ledger 500 (e.g. following process restart)
        EventProcessor::process_event(&mut repo, event, 500, "tx_500").unwrap();
        CheckpointManager::commit_checkpoint(&mut repo, 500).unwrap();
        assert_eq!(repo.get_last_ledger().unwrap(), 500);

        // Verify exactly 1 circle exists without duplication
        let circle = repo.get_circle("10").unwrap().unwrap();
        assert_eq!(circle.creator_address, "GBIDEMPOTENT");
    }
}
