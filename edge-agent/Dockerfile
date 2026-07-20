# syntax=docker/dockerfile:1
# TPT Edge Agent (Rust) — multi-arch build for Raspberry Pi 5 / Intel NUC.
FROM rust:1.81-slim AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --workspace

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /src/target/release/tpt-edge-agent /app/tpt-edge-agent
ENTRYPOINT ["/app/tpt-edge-agent"]
