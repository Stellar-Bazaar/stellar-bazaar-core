use serde::{Deserialize, Serialize};

/// Operational configuration for the Stellar Bazaar indexer daemon
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexerConfig {
    /// Soroban RPC endpoint URL (e.g. Testnet or standalone local node)
    pub rpc_url: String,
    /// Network passphrase for cryptographic domain separation
    pub network_passphrase: String,
    /// Target Soroban smart contract addresses to index
    pub contract_ids: Vec<String>,
    /// Ledger polling interval in milliseconds
    pub poll_interval_ms: u64,
    /// Optional starting ledger sequence (defaults to latest checkpoint if omitted)
    pub start_ledger: Option<u64>,
    /// Maximum events to fetch per Soroban RPC request
    pub batch_size: u32,
    /// HTTP port for daemon health check and metrics probe
    pub health_port: u16,
    /// SQLite database file path (":memory:" for volatile development mode)
    pub db_path: String,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
            contract_ids: vec![
                "CDCK6A2QB5QTILDWILUQGAUWXI543TK63WN2OKG7WEWHBAMGZ6ESKY4G".to_string(),
                "CCBYK4O47SCLF6Q5F773R4U2W7Z26S4JGB27F5O6H2N72B3Z37Y2P6P7".to_string(),
            ],
            poll_interval_ms: 3000,
            start_ledger: None,
            batch_size: 100,
            health_port: 8088,
            db_path: "./bazaar_indexer.db".to_string(),
        }
    }
}

impl IndexerConfig {
    /// Create config for local standalone test validator
    pub fn local_testnet() -> Self {
        Self {
            rpc_url: "http://localhost:8000/soroban/rpc".to_string(),
            network_passphrase: "Standalone Network ; February 2024".to_string(),
            poll_interval_ms: 1000,
            db_path: ":memory:".to_string(),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_indexer_config() {
        let config = IndexerConfig::default();
        assert!(config.rpc_url.contains("testnet"));
        assert_eq!(config.poll_interval_ms, 3000);
        assert_eq!(config.contract_ids.len(), 2);
    }

    #[test]
    fn test_local_testnet_config() {
        let config = IndexerConfig::local_testnet();
        assert_eq!(config.rpc_url, "http://localhost:8000/soroban/rpc");
        assert_eq!(config.db_path, ":memory:");
        assert_eq!(config.poll_interval_ms, 1000);
    }
}
