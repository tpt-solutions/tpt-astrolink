# TPT Cloud Core (Go)

Part of TPT AstroLink — Project Cosmos. Licensed MIT OR Apache-2.0.

Cloud Core is the Go backend: a WebSocket gateway for the Client UI, an MQTT
bridge to Edge Agents, a Postgres data layer, S3 integration, an Astrometry.net
worker, and session auth.

## Layout (`internal/`)
- `protocol` — shared message envelope
- `gateway` — WebSocket gateway (hub + client)
- `mqttbridge` — MQTT bridge (Cloud <-> Edge)
- `postgres` — data layer (schema: `docs/storage/postgres-schema.md`)
- `s3` — FITS storage client
- `astrometry` — Astrometry.net wrapper
- `worker` — cloud worker (S3 upload -> astrometry -> Postgres)
- `auth` — session service

## Build / Run
```
go build ./...
go run ./cmd/cloud-core   # listens on :8080 (/ws, /healthz)
```
