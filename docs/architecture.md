# Architecture & Data Flow — TPT AstroLink (Project Cosmos)

**Spec ref:** Sections 3, 4, 5.

## Components

| Component | Tech | Responsibility |
|-----------|------|----------------|
| Edge Agent | Rust | FFI to INDI/ASCOM, device control, FITS capture, compression, S3 upload, edge AI (ONNX) transient detection, MQTT client. |
| Cloud Core | Go | WebSocket gateway, MQTT bridge, Postgres (metadata), S3 integration, Astrometry worker, auth/session. |
| Client UI | TypeScript / React / Next.js + Three.js | Control dashboard, real-time telemetry, 3D sky, ToO alerts, multi-timezone. |
| Relay | Go | Scheduling engine: assign targets to nodes by local night-time, conflict resolution, data stitching. |

## Data Flow Diagrams

### Command (sub-second target)

```
User clicks "Slew"
   │
   ▼
Client UI ──WebSocket cmd.slew──▶ Cloud Core ──MQTT cmd/mount──▶ Edge Agent
                                                       │
                                                       ▼
                                                  FFI ──▶ INDI/ASCOM ──▶ Mount
```

- Cloud Core replies `ack` to the UI (paired by message `id`).
- Edge Agent streams `telemetry.mount` back via MQTT → Cloud Core → WebSocket.

### Telemetry

```
Edge Agent ──MQTT tele/<device>──▶ Cloud Core ──WebSocket telemetry.*──▶ Client UI
   ▲
   │ FFI read
INDI/ASCOM (mount encoders, focuser, weather sensors)
```

### Data (capture → astrometry → metadata)

```
Edge Agent capture FITS
   │ compress
   ▼
S3 upload (objectKey) ──▶ Cloud worker (Go) triggers Astrometry.net (C backend)
                              │ plate-solve → RA/Dec, FOV, orientation
                              ▼
                       Postgres (observations.metadata)
                              │
                              ▼
                  Client UI event.astrometry + 3D sky update
```

### Relay (crowdsourcing)

```
Relay scheduler
   │ target + local night-time availability
   ▼
assign ──▶ Node A (MQTT cmd)      Node B (MQTT cmd)      Node C (MQTT cmd)
                                              │
                              multi-node observations ──▶ stitch pipeline ──▶ Postgres
```

## FFI Boundary (Edge Agent)

The Rust Edge Agent binds to INDI and ASCOM C/C++ libraries via `unsafe`
FFI wrappers. Safety contract:

- All pointer crossing the boundary is validated/owned on the Rust side.
- Device handles are wrapped in RAII guards.
- No untrusted input is passed directly into C calls.

## Storage Layout

### Postgres (Cloud Core)

- `users`, `nodes`, `observations`, `targets`, `metadata` (schema TBD — Phase 1).

### S3 (FITS)

- `s3://<bucket>/fits/<nodeId>/<yyyy>/<mm>/<dd>/<observationId>.fits.gz`
- Astrometry outputs stored as object metadata + Postgres row.

## Open Questions

- Schema for `users/nodes/observations/targets/metadata` (Phase 1).
- S3 lifecycle rules for raw vs. processed FITS.
