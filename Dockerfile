# syntax=docker/dockerfile:1
# Multi-stage Dockerfile for Stellar Bazaar Relational Indexer & REST API

# Stage 1: Build binary
FROM rust:1.80-bullseye AS builder

WORKDIR /usr/src/stellar-bazaar

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY contracts ./contracts
COPY crates ./crates

# Build bazaar-indexer binary in release mode
RUN cargo build --release -p bazaar-indexer

# Stage 2: Minimal runtime image
FROM debian:bullseye-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled indexer binary from builder
COPY --from=builder /usr/src/stellar-bazaar/target/release/bazaar-indexer /usr/local/bin/bazaar-indexer

# Create directory for SQLite persistent database
RUN mkdir -p /app/data && chown -R 1000:1000 /app

USER 1000:1000

ENV SQLITE_PATH=/app/data/bazaar_indexer.db \
    PORT=8080 \
    RUST_LOG=info

EXPOSE 8080

HEALTHCHECK --interval=15s --timeout=5s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["bazaar-indexer", "--serve", "--port", "8080"]

