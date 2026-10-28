.PHONY: all build test lint fmt audit clean doc deploy-testnet help

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
