# TPT Relay (Go)

Part of TPT AstroLink — Project Cosmos. Licensed MIT OR Apache-2.0.

The Relay is the crowdsourcing scheduling engine. It assigns observation
targets to Edge Nodes by local night-time availability, resolves conflicts, and
stitches multi-node observations in the cloud.

## Layout (`internal/`)
- `scheduler` — target assignment + conflict resolution
- `stitching` — multi-node data stitching (TODO)

## Build / Run
```
go build ./...
go run ./cmd/relay
```
