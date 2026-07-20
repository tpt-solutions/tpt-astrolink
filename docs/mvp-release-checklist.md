# MVP Release Checklist — Sign-off

**Codename:** Project Cosmos · **Version target:** 0.1.0
**Owner:** TPT Solutions

This sheet is the gate for MVP sign-off. Every box is reviewed
by the named role before the tag is cut.

## Phase 6 — Data flow (verified)

- [x] Command flow: UI → WS → Cloud → MQTT → Edge → FFI → Mount
- [x] Telemetry flow: Edge → MQTT → Cloud → WS → UI
- [x] Data flow: Edge capture → S3 → Cloud worker → Astrometry → Postgres
- [x] **End-to-end command latency < 1 s** (p50 sub-ms, p99 < 1 s)
      validated by `cloud-core/internal/gateway/latency_test.go`
      (`TestCommandLatencyE2E`).

## Edge AI (Phase 2 follow-up)

- [x] ONNX Runtime integration wired (`ort`) in `crates/edge-ai`.
- [x] Brightness-anomaly inference + Target-of-Opportunity trigger.
- [x] Statistical fallback scores 400/400 transient vs normal with
      **recall 1.0 / FPR 0** on the synthetic suite
      (`crates/edge-ai/tests/accuracy.rs`).
- [ ] ONNX model trained & promoted to match the statistical
      baseline on real data (track post-MVP hardening).

## Deployment & OTA (Phase 7)

- [x] `crates/update` verifies SHA-256 **and** Ed25519 signature.
- [x] `tpt-edge-watchdog` supervises the agent + applies OTA.
- [x] systemd unit (`infra/systemd/tpt-edge-watchdog.service`).
- [x] `scripts/install.sh` (Pi 5 / Intel NUC) + `scripts/release.sh`.
- [ ] **HIL FFI pass recorded** for ≥ 3 pilot sites
      (`docs/testing/hil-ffi-test-plan.md`).

## Infrastructure

- [x] AWS: VPC + subnets, S3 (TLS-only, Glacier lifecycle),
      Aurora Postgres (private, KMS), Secrets Manager
      (`infra/aws/main.tf`).
- [x] K8s manifests for Cloud Core (`infra/k8s/cloud-core.yaml`).
- [x] Dockerfiles + compose for all three services.
- [x] CI per ecosystem (`.github/workflows`).
- [x] `/metrics` scrape endpoint + alerting scaffold.

## Load & resilience (Phase 8)

- [x] Concurrent WebSocket load test (`cmd/loadtest`) builds & runs.
- [x] Unit + integration tests per component green.
- [ ] Load test executed to the target concurrency against a
      staging Cloud Core (record p99 + error rate).
- [ ] Hardware-in-the-loop plan executed (cross-ref Phase 7).

## Security (pre-launch gate)

- [ ] JWT auth **enforced** at gateway + MQTT ACLs
      (`docs/security-review.md` §1).
- [ ] WebSocket `CheckOrigin` allowlist.
- [ ] Secrets sourced from AWS Secrets Manager only (no repo/env leaks).
- [ ] `cargo audit` / `govulncheck` / `tfsec` green in CI.

## Docs & onboarding

- [x] Architecture + protocol contracts (`docs/`).
- [x] Node operator setup guide (`docs/node-operator-guide.md`).
- [x] Beta onboarding runbook (`docs/beta-onboarding.md`).
- [x] HIL FFI test plan (`docs/testing/hil-ffi-test-plan.md`).
- [x] Security review (`docs/security-review.md`).
- [x] Edge-AI model contract + accuracy harness doc
      (`docs/edge-ai-model.md`).

## Sign-off

| Role | Name | Date | |
|------|------|------|---|
| Eng lead | | | |
| Security | | | |
| Product (TPT) | | | |

**Cut tag only when every unchecked box above is closed.**
