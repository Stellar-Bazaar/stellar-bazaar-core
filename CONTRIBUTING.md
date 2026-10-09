# Contributing to Stellar Bazaar Core

Thank you for your interest in contributing to **Stellar Bazaar Core**! We welcome contributions to our Soroban smart contracts, event indexing services, database migrations, and testing tools.

---

## 1. Code of Conduct

All contributors are expected to uphold our [Code of Conduct](CODE_OF_CONDUCT.md). Please report unacceptable behavior to `conduct@stellarbazaar.io`.

---

## 2. Prerequisites & Development Environment

To build, test, and develop within this repository, install the following:

- **Rust toolchain** (1.80 or newer):
  ```bash
  rustup default stable
  rustup target add wasm32-unknown-unknown
  ```
- **Stellar CLI** (or Soroban CLI):
  ```bash
  cargo install --locked stellar-cli --features opt
  ```
- **Git** (for version control)

---

## 3. Getting Started

1. **Fork the Repository**: Create your own fork of `Stellar-Bazaar/stellar-bazaar-core`.
2. **Clone your fork**:
   ```bash
   git clone https://github.com/<your-username>/stellar-bazaar-core.git
   cd stellar-bazaar-core
   ```
3. **Build the workspace**:
   ```bash
   cargo build --workspace
   ```
4. **Run the test suite**:
   ```bash
   cargo test --workspace
   ```

---

## 4. Coding & Architecture Guidelines

### Smart Contract Development (`contracts/`)
- **Protocol Version**: We target Stellar Soroban **Protocol 22**.
- **Deterministic Execution**: Avoid non-deterministic logic, unvetted external crates, or floating-point calculations. Always use integer arithmetic with explicit stroop scaling (`1 XLM = 10,000,000 stroops`).
- **Authorization**: Mutating functions must enforce strict `require_auth()` verification on caller accounts.
- **Checks-Effects-Interactions**: Always update internal contract storage prior to invoking external token contract transfers.
- **Storage Strategy**: Group related deal and circle state cleanly using typed keys (`DataKey`).

### Indexer & Tooling (`crates/bazaar_indexer/`)
- Ensure ledger ingestion is replay-safe and idempotent.
- Checkpoints should be committed atomically alongside database transactions.

---

## 5. Formatting & Linting

Before creating a pull request, ensure your code compiles cleanly and adheres to standard formatting:

```bash
# Format code
cargo fmt --all -- --check

# Run linter
cargo clippy --workspace --all-targets -- -D warnings

# Execute all tests
cargo test --workspace
```

---

## 6. Commit Message Guidelines

We use conventional commit messages to maintain a clean git history:
- `feat: add escrow milestone releases`
- `fix: handle edge case in circle expiration timestamp`
- `docs: update testnet explorer verification links`
- `test: add unit test for seller dispute outcome`

---

## 7. Submitting a Pull Request

1. Create a feature branch from `main`:
   ```bash
   git checkout -b feat/your-feature-name
   ```
2. Commit your changes with concise, descriptive messages.
3. Push to your fork and submit a Pull Request to `Stellar-Bazaar/stellar-bazaar-core:main`.
4. Provide a clear summary of the changes, test results, and any linked issues.
5. All CI checks must pass before merging.

---

## 8. Reporting Security Vulnerabilities

Please **do not** open public GitHub issues for security vulnerabilities. Instead, refer to our [Security Policy](SECURITY.md) and report via `security@stellarbazaar.io`.
