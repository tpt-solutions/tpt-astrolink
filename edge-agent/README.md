# TPT Edge Agent (Rust)

Part of TPT AstroLink — Project Cosmos. Licensed MIT OR Apache-2.0.

The Edge Agent is a memory-safe Rust binary that runs on observatory hardware
(Raspberry Pi 5, Intel NUC). It controls mounts, focusers and weather sensors
via INDI/ASCOM C-library FFI, captures/compresses FITS, uploads to S3, and runs
an edge-AI transient detector (ONNX).

## Layout
- `crates/agent` — binary + command/telemetry dispatch loop
- `crates/ffi` — safe FFI boundary for INDI/ASCOM (see `docs/ffi-boundary.md`)
- `crates/mqtt` — MQTT client (topics in `docs/protocols/mqtt-contract.md`)
- `crates/s3` — FITS upload (layout in `docs/storage/s3-layout.md`)
- `crates/imaging` — capture + gzip pipeline
- `crates/edge-ai` — transient detector (Target of Opportunity)

## Build
```
cargo build --release --workspace
```

## Run
```
TPT_NODE_ID=... TPT_MQTT_BROKER=... cargo run -p tpt-edge-agent
```
