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
}
