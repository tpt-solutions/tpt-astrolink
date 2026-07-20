# TPT AstroLink — Project Cosmos — Task Checklist

License: Dual MIT / Apache-2.0 · TPT Solutions

## Phase 0 — Project Setup & Legal
- [x] Monorepo scaffolding (`/edge-agent`, `/cloud-core`, `/client-ui`, `/relay`, `/docs`, `/infra`)
- [x] Root README with project overview, architecture diagram placeholder, TPT Solutions attribution
- [x] LICENSE-MIT and LICENSE-APACHE files + dual-license note in README and each subpackage manifest
- [x] `.gitignore`, `.editorconfig`, base git repo init
- [x] Define shared API/protocol contracts (WebSocket message schema, MQTT topic schema) in `/docs`

## Phase 1 — Architecture & Protocol Design
- [x] Finalize command/telemetry/data flow diagrams from spec section 5 (`docs/architecture.md`)
- [x] Define WebSocket message contract (Cloud Core <-> Client UI) (`docs/protocols/websocket-contract.md`)
- [x] Define MQTT topic/payload contract (Cloud Core <-> Edge Agent) (`docs/protocols/mqtt-contract.md`)
- [x] Define FFI interface boundary for INDI/ASCOM C-library bindings (`docs/ffi-boundary.md`)
- [x] Define Postgres schema (users, nodes, observations, targets, metadata) (`docs/storage/postgres-schema.md`)
- [x] Define S3 bucket/object layout for FITS storage (`docs/storage/s3-layout.md`)

## Phase 2 — TPT Edge Agent (Rust)
- [x] Project scaffold (Cargo workspace) for Raspberry Pi 5 / Intel NUC targets (`edge-agent/`)
- [x] FFI bindings to INDI and ASCOM C/C++ drivers (safe boundary in `crates/ffi`)
- [x] Mount control (slew) command handling
- [x] Focuser control
- [x] Weather sensor monitoring/ingestion
- [x] MQTT client integration (publish telemetry, subscribe commands)
- [x] FITS image capture + compression pipeline (`crates/imaging`)
- [x] S3 upload client (`crates/s3`)
- [ ] Edge AI transient alert: ONNX runtime integration, brightness anomaly model inference, "Target of Opportunity" trigger
  - [x] Detector boundary + ToO alert type scaffolded (`crates/edge-ai`); ONNX runtime wired in Phase 2 follow-up

## Phase 3 — TPT Cloud Core (Go)
- [x] Project scaffold, module layout for microservices (`cloud-core/`)
- [x] WebSocket gateway for real-time Client UI telemetry/commands (`internal/gateway`)
- [x] MQTT broker integration/bridge to Edge Agents (`internal/mqttbridge`)
- [x] Postgres data layer (metadata, users, nodes, observations) (`internal/postgres`)
- [x] S3 integration for FITS storage (`internal/s3`)
- [x] Astrometry.net C-backend integration (API/CLI wrapper) for plate-solving (`internal/astrometry`)
- [x] Cloud worker: triggers astrometry job on S3 upload, writes RA/Dec metadata to Postgres (`internal/worker`)
- [x] Auth/session service for multi-user access (`internal/auth` — JWT TODO)

## Phase 4 — TPT Client UI (TypeScript / React / Next.js)
- [x] Next.js app scaffold (`client-ui/`)
- [x] Universal Control Dashboard: mount slew controls, focuser controls, weather sensor display
- [x] Real-time telemetry via WebSocket client (`src/lib/useTelemetry.ts`)
- [x] Imaging sequence trigger UI
- [x] Three.js 3D sky visualization component (`src/components/SkyView.tsx`)
- [x] Target of Opportunity alert UI/notifications (`src/components/ToONotifications.tsx`)
- [x] Mobile-responsive layout pass (`src/app/globals.css` + `.tpt-grid` responsive)
- [x] Multi-timezone display support for researchers (`components/MultiTimezoneClocks.tsx`, `.env.example`)

## Phase 5 — Relay Protocol (Crowdsourcing Engine, Go)
- [x] Scheduling engine: assign observation targets to Edge Nodes by local night-time availability (`internal/scheduler`)
- [x] Node availability/registration tracking (`internal/registry`, in-memory; Postgres persistence TODO)
- [x] Target assignment algorithm + conflict resolution (`internal/scheduler.Schedule`)
- [x] Data stitching pipeline (combine multi-node observations in cloud) (`internal/stitching`: circular-mean co-add)

## Phase 6 — Integration & End-to-End Data Flow
- [x] Command flow: Web UI -> WebSocket -> Cloud (Go) -> MQTT -> Edge Agent (Rust) -> FFI -> Mount (`client-ui` dashboard, `cloud-core` gateway+mqttbridge+transport, `edge-agent` commands+ffi)
- [x] Telemetry flow: Edge Agent -> MQTT -> Cloud -> WebSocket -> Web UI (`edge-agent` publish_telemetry, `cloud-core` transport subscribe -> hub.Broadcast)
- [x] Data flow: Edge capture -> compress -> S3 -> Cloud worker -> Astrometry -> Postgres (Edge `imaging`/`s3`, Cloud `worker`/`astrometry`/`postgres`)
- [ ] End-to-end latency validation (sub-second command target)

## Phase 7 — Infra/DevOps & Deployment
- [x] CI/CD pipelines per component (Rust, Go, TS) (`.github/workflows/`)
- [x] Dockerize Edge Agent, Cloud Core, Client UI (Dockerfiles)
- [x] Kubernetes/orchestration setup for Cloud Core (`infra/k8s/cloud-core.yaml`)
- [ ] AWS provisioning (S3 buckets, Postgres/RDS, networking, secrets management)
- [x] Observability: logging, metrics, alerting for Cloud Core and Edge fleet (`cloud-core/internal/metrics`, `/metrics`; alerting TODO)
- [ ] Edge Agent deployment/update mechanism for Raspberry Pi 5 / Intel NUC hardware

## Phase 8 — Testing & QA
- [x] Unit tests per component (Rust, Go, TS) (`edge-agent` imaging+edge-ai; `cloud-core` protocol+mqttbridge; `relay` scheduler+stitching; `client-ui` vitest)
- [ ] Integration tests for WebSocket/MQTT contracts
- [ ] Hardware-in-the-loop test plan for INDI/ASCOM FFI layer
- [ ] Load testing for concurrent WebSocket connections (Cloud Core)
- [ ] Edge AI model accuracy validation (transient detection)

## Phase 9 — MVP Launch
- [ ] Security review (auth, secrets, FFI boundary safety)
- [ ] Documentation pass (setup guides for pro-amateur node operators)
- [ ] Beta onboarding for pilot observatories
- [ ] MVP release checklist sign-off
