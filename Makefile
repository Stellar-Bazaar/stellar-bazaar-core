.PHONY: all build test lint fmt audit clean doc deploy-testnet simulate-dispute simulate-e2e indexer-test indexer-run sdk-test docker-build docker-compose-up help

# Default target
all: test

help:
	@echo "Stellar Bazaar Core - Development Commands"
	@echo "=========================================="
	@echo "  make build           - Build all contracts and workspace crates"
	@echo "  make test            - Execute all unit and integration tests"
	@echo "  make lint            - Run cargo clippy with strict warnings"
	@echo "  make fmt             - Format all Rust codebases"
	@echo "  make fmt-check       - Verify formatting without modifying"
	@echo "  make audit           - Run cargo audit for known vulnerabilities"
	@echo "  make doc             - Generate offline Rust documentation"
	@echo "  make clean           - Remove build artifacts and caches"
	@echo "  make deploy-testnet  - Deploy contracts to Stellar Testnet"
	@echo "  make simulate-dispute - Execute dispute resolution lifecycle simulation"
	@echo "  make simulate-e2e    - Execute full end-to-end cooperative lifecycle simulation"
	@echo "  make indexer-test    - Run Soroban event indexer test suite"
	@echo "  make indexer-run     - Run indexer daemon dry run validation"
	@echo "  make sdk-test        - Run @stellar-bazaar/sdk unit tests and type checks"
	@echo "  make docker-build    - Build indexer Docker image"
	@echo "  make docker-compose-up - Launch indexer container via docker-compose"

build:
	cargo build --workspace --all-targets

build-contracts:
	stellar contract build

test:
	cargo test --workspace -- --nocapture

lint:
	cargo clippy --workspace --all-targets -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

audit:
	cargo audit

doc:
	cargo doc --workspace --no-deps

clean:
	cargo clean

deploy-testnet:
	powershell -ExecutionPolicy Bypass -File ./scripts/deploy_testnet.ps1

simulate-dispute:
	powershell -ExecutionPolicy Bypass -File ./scripts/simulate_dispute_flow.ps1

simulate-e2e:
	powershell -ExecutionPolicy Bypass -File ./scripts/simulate_e2e_cooperative_lifecycle.ps1

indexer-test:
	cargo test -p bazaar-indexer -- --nocapture

indexer-run:
	cargo run -p bazaar-indexer -- --dry-run

sdk-test:
	powershell -ExecutionPolicy Bypass -File ./scripts/test_sdk.ps1

docker-build:
	docker build -t stellar-bazaar-indexer:latest .

docker-compose-up:
	docker compose up -d
