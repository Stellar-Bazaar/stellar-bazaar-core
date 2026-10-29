use bazaar_indexer::config::IndexerConfig;
use bazaar_indexer::health::HealthChecker;
use bazaar_indexer::metrics::PrometheusExporter;
use bazaar_indexer::server::ApiServer;
use bazaar_indexer::sqlite_repository::SqliteBazaarRepository;
use bazaar_indexer::worker::IndexerWorker;
use std::env;

fn print_usage() {
    println!(
        r#"Stellar Bazaar Soroban Event Indexer Daemon

USAGE:
    bazaar-indexer [OPTIONS]

OPTIONS:
    --db <PATH>                 SQLite database path [default: bazaar_indexer.db]
    --rpc <URL>                 Soroban RPC endpoint [default: https://soroban-testnet.stellar.org]
    --contract <ID>             Target Soroban smart contract ID to index
    --start-ledger <SEQ>        Starting ledger sequence for event catch-up
    --batch-size <NUM>          Maximum events to fetch per RPC call
    --port <PORT>               Port for embedded REST API server [default: 8088]
    --serve                     Launch live HTTP REST API server
    --health                    Evaluate daemon health check and print JSON
    --metrics                   Render Prometheus metrics output and exit
    --dry-run                   Validate database and configuration without starting loop
    --help                      Show this help message
"#
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        return Ok(());
    }

    let mut config = IndexerConfig::default();
    let mut dry_run = false;
    let mut print_health = false;
    let mut print_metrics = false;
    let mut serve = false;
    let mut port = 8088u16;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--db" => {
                if i + 1 < args.len() {
                    config.db_path = args[i + 1].clone();
                    i += 1;
                }
            }
            "--rpc" => {
                if i + 1 < args.len() {
                    config.rpc_url = args[i + 1].clone();
                    i += 1;
                }
            }
            "--contract" => {
                if i + 1 < args.len() {
                    config.contract_ids.push(args[i + 1].clone());
                    i += 1;
                }
            }
            "--batch-size" => {
                if i + 1 < args.len() {
                    if let Ok(size) = args[i + 1].parse::<u32>() {
                        config.batch_size = size;
                    }
                    i += 1;
                }
            }
            "--start-ledger" => {
                if i + 1 < args.len() {
                    if let Ok(seq) = args[i + 1].parse::<u64>() {
                        config.start_ledger = Some(seq);
                    }
                    i += 1;
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    if let Ok(p) = args[i + 1].parse::<u16>() {
                        port = p;
                        config.health_port = p;
                    }
                    i += 1;
                }
            }
            "--serve" => {
                serve = true;
            }
            "--dry-run" => {
                dry_run = true;
            }
            "--health" => {
                print_health = true;
            }
            "--metrics" => {
                print_metrics = true;
            }
            _ => {}
        }
        i += 1;
    }

    println!("============================================================");
    println!("        Stellar Bazaar Soroban Event Indexer Daemon         ");
    println!("============================================================");
    println!("Database Path    : {}", config.db_path);
    println!("Soroban RPC      : {}", config.rpc_url);
    println!("Network Phr      : {}", config.network_passphrase);
    println!("Contract IDs     : {:?}", config.contract_ids);
    println!("Poll Interval    : {} ms", config.poll_interval_ms);
    println!("Batch Size       : {} events", config.batch_size);
    println!("HTTP Port        : {}", port);
    println!("------------------------------------------------------------");

    // Initialize database
    let repo = if config.db_path == ":memory:" {
        SqliteBazaarRepository::new_in_memory()?
    } else {
        SqliteBazaarRepository::new(&config.db_path)?
    };
    println!("[DB] Relational schema and foreign keys initialized.");

    let worker = IndexerWorker::new(config);

    if print_health {
        let health = HealthChecker::evaluate(&worker.state, worker.consecutive_errors);
        println!(
            "\n[Health Check]:\n{}",
            serde_json::to_string_pretty(&health)?
        );
        return Ok(());
    }

    if print_metrics {
        let metrics = PrometheusExporter::render(&worker.state, worker.consecutive_errors);
        println!("\n[Prometheus Metrics]:\n{}", metrics);
        return Ok(());
    }

    if dry_run {
        println!("[Daemon] Dry run mode completed successfully. Config & storage valid.");
        return Ok(());
    }

    if serve {
        let addr = format!("127.0.0.1:{}", port);
        println!("[Daemon] Starting HTTP REST API server on http://{}", addr);
        let server = ApiServer::new(Box::new(repo), worker);
        server.serve(&addr)?;
        return Ok(());
    }

    println!("[Daemon] Worker initialized. Ready for ledger ingestion.");
    Ok(())
}
