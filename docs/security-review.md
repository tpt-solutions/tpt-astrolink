# Security Review — TPT AstroLink

**Scope:** auth, secrets management, and the INDI/ASCOM FFI boundary.
**Status:** pre-MVP review · **Owner:** TPT Solutions

## 1. Authentication & authorization (Cloud Core)

- **Current state:** `internal/auth` issues JWTs but the `TODO(JWT)` marker
  remains — token **signing/verification is not yet enforced** on the
  WebSocket gateway or MQTT bridge. This is the top pre-launch gap.
- **Action:** Implement RS256/Ed25519 JWT verification at the gateway
  `ServeWS` entry point; reject connections without a valid, unexpired
  token. Bind `nodeId` claims to MQTT topic ACLs (no node may
  publish to another node's command topic).
- **Transport:** enforce `CheckOrigin` in the gateway upgrade (today it
  returns `true` for all origins) — allowlist the Client UI origin(s).

## 2. Secrets management (AWS)

- RDS master password and S3 bucket metadata are stored in **AWS
  Secrets Manager** (`infra/aws/main.tf`), referenced by Cloud Core
  at runtime via IAM role — never in repo, env, or logs.
- `db_master_password` is supplied only at `terraform apply` via
  `example.tfvars` (git-ignored) or CI secret; it is **never**
  committed (see `infra/aws/.gitignore`).
- Prefer IAM Roles Anywhere / task roles over long-lived keys.

## 3. FFI boundary safety (Edge Agent)

- All INDI/ASCOM calls cross a single crate (`crates/ffi`) with a
  documented `unsafe` contract. Each `extern "C"` binding:
  - copies inputs into owned Rust buffers (no borrowing of C memory),
  - bounds-checks string/payload lengths **before** passing to C,
  - returns a sentinel error code rather than panicking on null/garbage.
- **HIL validation** of the boundary is mandated before fleet rollout
  (see `docs/testing/hil-ffi-test-plan.md`); the unit suite mocks
  the C library so CI stays hermetic.
- No `unsafe` outside `crates/ffi`; `cargo clippy -D warnings` is run
  in CI.

## 4. Over-the-air (OTA) update chain

- `crates/update` downloads a release, verifies **SHA-256** against the
  manifest, and (when a key is configured) verifies an **Ed25519**
  release signature over the bytes. Unsigned artifacts are rejected when a
  key is present.
- The public key is provisioned to the node via
  `TPT_UPDATE_PUBKEY` (hex) in `/etc/tpt-edge-agent/env`; the
  **private** key lives only in the release pipeline
  (`scripts/release.sh --sign-with`).
- The watchdog (`tpt-edge-watchdog`) atomically swaps the live
  binary and restarts; a failed apply rolls back to the `.bak` copy.

## 5. Network & data

- S3 FITS bucket enforces **TLS-only** access and SSE-KMS
  (`infra/aws/main.tf` bucket policy + `aws:kms`).
- RDS is in private subnets, reachable only from the Cloud Core SG on
  5432; no public endpoint (`publicly_accessible = false`).
- Telemetry is scoped to the node's VPC CIDR.

## 6. Outstanding items (gating MVP)

1. [ ] Enforce JWT verification at gateway + MQTT ACLs.
2. [ ] Restrict WebSocket `CheckOrigin`.
3. [ ] FFI HIL pass recorded in the release checklist.
4. [ ] OTA signing key generated and distributed to pilot nodes.

## 7. Tooling

- `cargo audit` (Rust) and `govulncheck` (Go) in CI.
- Dependabot/Renovate for the three language ecosystems.
- Terraform via `tfsec` / `checkov` with the supplied `.tfvars`.
