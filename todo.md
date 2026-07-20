# TPT AstroLink — Project Cosmos — Task Checklist

License: Dual MIT / Apache-2.0 · TPT Solutions

## Phase 0 — Project Setup & Legal
- [x] Monorepo scaffolding (`/edge-agent`, `/cloud-core`, `/client-ui`, `/relay`, `/docs`, `/infra`)
- [x] Root README with project overview, architecture diagram placeholder, TPT Solutions attribution
- [x] LICENSE-MIT and LICENSE-APACHE files + dual-license note in README and each subpackage manifest
- [x] `.gitignore`, `.editorconfig`, base git repo init
- [x] Define shared API/protocol contracts (WebSocket message schema, MQTT topic schema) in `/docs`

## Phase 1 — Architecture & Protocol Design
- [ ] Finalize command/telemetry/data flow diagrams from spec section 5
- [ ] Define WebSocket message contract (Cloud Core <-> Client UI)
- [ ] Define MQTT topic/payload contract (Cloud Core <-> Edge Agent)
- [ ] Define FFI interface boundary for INDI/ASCOM C-library bindings
- [ ] Define Postgres schema (users, nodes, observations, targets, metadata)
- [ ] Define S3 bucket/object layout for FITS storage

## Phase 2 — TPT Edge Agent (Rust)
- [ ] Project scaffold (Cargo workspace) for Raspberry Pi 5 / Intel NUC targets
- [ ] FFI bindings to INDI and ASCOM C/C++ drivers
- [ ] Mount control (slew) command handling
- [ ] Focuser control
- [ ] Weather sensor monitoring/ingestion
- [ ] MQTT client integration (publish telemetry, subscribe commands)
- [ ] FITS image capture + compression pipeline
- [ ] S3 upload client
- [ ] Edge AI transient alert: ONNX runtime integration, brightness anomaly model inference, "Target of Opportunity" trigger

## Phase 3 — TPT Cloud Core (Go)
- [ ] Project scaffold, module layout for microservices
- [ ] WebSocket gateway for real-time Client UI telemetry/commands
- [ ] MQTT broker integration/bridge to Edge Agents
- [ ] Postgres data layer (metadata, users, nodes, observations)
- [ ] S3 integration for FITS storage
- [ ] Astrometry.net C-backend integration (API/CLI wrapper) for plate-solving
- [ ] Cloud worker: triggers astrometry job on S3 upload, writes RA/Dec metadata to Postgres
- [ ] Auth/session service for multi-user access

## Phase 4 — TPT Client UI (TypeScript / React / Next.js)
- [ ] Next.js app scaffold
- [ ] Universal Control Dashboard: mount slew controls, focuser controls, weather sensor display
- [ ] Real-time telemetry via WebSocket client
- [ ] Imaging sequence trigger UI
- [ ] Three.js 3D sky visualization component
- [ ] Target of Opportunity alert UI/notifications
- [ ] Mobile-responsive layout pass
- [ ] Multi-timezone display support for researchers

## Phase 5 — Relay Protocol (Crowdsourcing Engine, Go)
- [ ] Scheduling engine: assign observation targets to Edge Nodes by local night-time availability
- [ ] Node availability/registration tracking
- [ ] Target assignment algorithm + conflict resolution
- [ ] Data stitching pipeline (combine multi-node observations in cloud)

## Phase 6 — Integration & End-to-End Data Flow
- [ ] Command flow: Web UI -> WebSocket -> Cloud (Go) -> MQTT -> Edge Agent (Rust) -> FFI -> Mount
- [ ] Telemetry flow: Edge Agent -> MQTT -> Cloud -> WebSocket -> Web UI
- [ ] Data flow: Edge capture -> compress -> S3 -> Cloud worker -> Astrometry -> Postgres
- [ ] End-to-end latency validation (sub-second command target)

## Phase 7 — Infra/DevOps & Deployment
- [ ] CI/CD pipelines per component (Rust, Go, TS)
- [ ] Dockerize Edge Agent, Cloud Core, Client UI
- [ ] Kubernetes/orchestration setup for Cloud Core
- [ ] AWS provisioning (S3 buckets, Postgres/RDS, networking, secrets management)
- [ ] Observability: logging, metrics, alerting for Cloud Core and Edge fleet
- [ ] Edge Agent deployment/update mechanism for Raspberry Pi 5 / Intel NUC hardware

## Phase 8 — Testing & QA
- [ ] Unit tests per component (Rust, Go, TS)
- [ ] Integration tests for WebSocket/MQTT contracts
- [ ] Hardware-in-the-loop test plan for INDI/ASCOM FFI layer
- [ ] Load testing for concurrent WebSocket connections (Cloud Core)
- [ ] Edge AI model accuracy validation (transient detection)

## Phase 9 — MVP Launch
- [ ] Security review (auth, secrets, FFI boundary safety)
- [ ] Documentation pass (setup guides for pro-amateur node operators)
- [ ] Beta onboarding for pilot observatories
- [ ] MVP release checklist sign-off
