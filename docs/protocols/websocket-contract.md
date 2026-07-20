# WebSocket Message Contract — Cloud Core ↔ Client UI

**Owner:** TPT Cloud Core (Go)
**Consumers:** TPT Client UI (Next.js)
**Spec ref:** Data Flow, "Command" & "Telemetry" paths.

All messages are JSON over a single WebSocket connection, per-connection
multiplexed by `type`. The connection is authenticated via the auth/session
service; see `docs/protocols/auth.md` (TBD).

## Envelope

```json
{
  "type": "<MessageType>",
  "id": "<uuid-v4>",
  "ts": "<rfc3339-utc>",
  "nodeId": "<edge-node-id>",
  "payload": { }
}
```

- `id` — client-or-server generated correlation id for request/ack pairing.
- `ts` — RFC3339 UTC timestamp (e.g. `2026-07-20T02:15:00Z`).
- `nodeId` — the target/source Edge Node (omitted for global client messages).

## Client → Server (Commands)

| type | payload | description |
|------|---------|-------------|
| `cmd.slew` | `{ "ra": <deg>, "dec": <deg>, "epoch": "J2000" }` | Slew mount to equatorial coordinates. |
| `cmd.slewStop` | `{}` | Halt mount motion. |
| `cmd.focus` | `{ "position": <steps> }` | Move focuser to absolute position. |
| `cmd.focusRelative` | `{ "delta": <steps> }` | Move focuser relatively. |
| `cmd.weather.refresh` | `{}` | Request fresh weather sensor sample. |
| `cmd.imaging.start` | `{ "sequence": [...], "exposure": <s>, "gain": <int>, "bin": <int> }` | Trigger an imaging sequence. |
| `cmd.imaging.stop` | `{}` | Abort active sequence. |
| `cmd.subscribe` | `{ "nodeId": "<id>" }` | Subscribe to a node's telemetry stream. |

## Server → Client (Telemetry / Events)

| type | payload | description |
|------|---------|-------------|
| `ack` | `{ "ok": <bool>, "error": "<msg>" }` | Command acknowledgement, paired by `id`. |
| `telemetry.mount` | `{ "ra": <deg>, "dec": <deg>, "alt": <deg>, "az": <deg>, "tracking": <bool>, "status": "idle|slewing|tracking|error" }` | Mount state. |
| `telemetry.focuser` | `{ "position": <steps>, "temperature": <C> }` | Focuser state. |
| `telemetry.weather` | `{ "temp": <C>, "humidity": <%>, "pressure": <hPa>, "windSpeed": <m/s>, "dewPoint": <C>, "cloudCover": <%> }` | Weather sensor sample. |
| `event.imaging.progress` | `{ "frame": <int>, "total": <int>, "objectKey": "<s3-key>" }` | Imaging sequence progress. |
| `alert.too` | `{ "objectId": "<id>", "ra": <deg>, "dec": <deg>, "magDelta": <float>, "confidence": <0..1>, "imageKey": "<s3-key>" }` | Target of Opportunity transient alert (Edge AI). |
| `event.astrometry` | `{ "observationId": "<id>", "ra": <deg>, "dec": <deg>, "fov": <deg>, "orientation": <deg> }` | Plate-solve result from cloud worker. |

## Sequences

1. Client sends `cmd.slew` with `id=X`.
2. Server forwards to Edge Agent over MQTT, awaits device ack, and replies
   `ack` (`id=X`) to the client.
3. Edge Agent streams `telemetry.mount` which Cloud Core relays to the client.

## Open Questions

- Backpressure / rate limiting for high-frequency telemetry.
- Reconnect + resubscribe semantics on dropped connections.
