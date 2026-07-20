# TPT AstroLink — Project Cosmos

> Cloud-native remote observatory control and a distributed "virtual observatory" network for crowdsourced sky monitoring.

**License:** Dual MIT / Apache-2.0 — Copyright © TPT Solutions
**Codename:** Project Cosmos · **Spec Version:** 1.1 (2026-07-20)

---

## Overview

TPT AstroLink democratizes astronomical observation by bridging amateur hardware
and professional-grade data collection. It provides:

- A SaaS **remote observatory control dashboard** (mobile-friendly, cross-platform).
- A distributed **"virtual observatory"** network for crowdsourced sky monitoring
  via the *Relay* scheduling protocol.
- **Edge AI transient alerts** that automatically trigger "Target of Opportunity"
  events when a brightness anomaly is detected.

## Architecture (Zero-Python)

| Component | Tech | Role |
|-----------|------|------|
| **TPT Edge Agent** ("Node") | Rust | Lightweight binary on observatory hardware (Raspberry Pi 5, Intel NUC). FFI to INDI/ASCOM C/C++ drivers, FITS capture, edge AI inference, MQTT/S3. |
| **TPT Cloud Core** | Go | WebSocket gateway, MQTT bridge, Postgres (metadata), S3 (FITS), Astrometry worker, auth. |
| **TPT Client UI** | TypeScript / React / Next.js + Three.js | Universal control dashboard, 3D sky visualization, real-time telemetry. |
| **The "Relay" Protocol** | Go | Crowdsourcing scheduling engine: assign targets to nodes by local night-time availability, stitch multi-node data. |

### Repository Layout

```
tpt-astrolink/
├── edge-agent/   # Rust workspace — TPT Edge Agent
├── cloud-core/   # Go modules — TPT Cloud Core
├── client-ui/    # Next.js app — TPT Client UI
├── relay/        # Go — Relay scheduling engine
├── docs/         # Spec, protocol & API contracts
├── infra/        # IaC, CI/CD, Docker, K8s
└── LICENSE-MIT / LICENSE-APACHE
```

## Data Flow

1. **Command:** Web UI → WebSocket → Cloud Core (Go) → MQTT → Edge Agent (Rust) → FFI → INDI/ASCOM → Mount.
2. **Telemetry:** Edge Agent → MQTT → Cloud Core → WebSocket → Web UI.
3. **Data:** Edge Agent captures FITS → compresses → S3 → Cloud worker → Astrometry.net → Postgres.

```
                 ┌────────────┐
   Command ─────▶│  Client UI │──── WebSocket ────┐
                 │ (Next.js) │◀─── WebSocket ────┤
                 └────────────┘     Telemetry     │
                                                  ▼
                                          ┌────────────────┐
                                          │  Cloud Core    │
                                          │  (Go / Go)     │◀──┐
                                          └────────────────┘   │
                                               │  MQTT          │ S3
                                               ▼                │
                                        ┌─────────────┐        │
                                        │ Edge Agent  │────────┘
                                        │   (Rust)    │── FFI ──▶ INDI/ASCOM
                                        └─────────────┘
```

> Architecture diagram is a placeholder; refine in `docs/architecture.md`.

## Documentation

- `docs/protocols/websocket-contract.md` — Cloud Core ↔ Client UI message schema.
- `docs/protocols/mqtt-contract.md` — Cloud Core ↔ Edge Agent topic/payload schema.
- `docs/architecture.md` — Full command/telemetry/data flow diagrams.

## License

This project is dual-licensed under the [MIT License](./LICENSE-MIT) and the
[Apache License 2.0](./LICENSE-APACHE). You may choose either license at your option.

See `LICENSE-MIT` and `LICENSE-APACHE` for the full text.

---

© TPT Solutions. All rights reserved.
