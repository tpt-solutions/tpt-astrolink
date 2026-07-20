# MQTT Topic & Payload Contract — Cloud Core ↔ Edge Agent

**Owner:** TPT Edge Agent (Rust) ↔ TPT Cloud Core (Go)
**Spec ref:** Data Flow, "Command" & "Telemetry" & "Data" paths.

MQTT 5.0 broker. Each Edge Node connects with a unique client id
`node/<nodeId>` and uses topic prefixes namespaced by node.

## Topic Layout

```
tpt/v1/<nodeId>/cmd/<device>      # Cloud Core → Edge Agent (commands)
tpt/v1/<nodeId>/tele/<device>     # Edge Agent → Cloud Core (telemetry)
tpt/v1/<nodeId>/evt/<event>       # Edge Agent → Cloud Core (events)
tpt/v1/<nodeId>/status            # Edge Agent → Cloud Core (LWT + health)
```

- `<device>` ∈ { `mount`, `focuser`, `weather`, `camera`, `all` }
- `<event>` ∈ { `imaging.progress`, `too`, `astrometry.done` }

## Commands — Cloud Core → Edge Agent

Topic: `tpt/v1/<nodeId>/cmd/<device>`

QoS 1, retained false. Payload JSON:

```json
{
  "cmd": "slew",
  "id": "<uuid-v4>",
  "ts": "<rfc3339-utc>",
  "params": { "ra": 12.5, "dec": -30.0, "epoch": "J2000" }
}
```

| cmd | device | params |
|-----|--------|--------|
| `slew` | mount | `{ "ra": <deg>, "dec": <deg>, "epoch": "J2000" }` |
| `slewStop` | mount | `{}` |
| `focus` | focuser | `{ "position": <steps> }` |
| `focusRelative` | focuser | `{ "delta": <steps> }` |
| `imaging.start` | camera | `{ "sequence": [...], "exposure": <s>, "gain": <int>, "bin": <int> }` |
| `imaging.stop` | camera | `{}` |
| `weather.refresh` | weather | `{}` |

## Telemetry — Edge Agent → Cloud Core

Topic: `tpt/v1/<nodeId>/tele/<device>`

QoS 0 (high-frequency), retained true. Payload JSON (same shape as the
WebSocket `telemetry.*` payloads, minus the envelope `type`/`id`).

## Events — Edge Agent → Cloud Core

Topic: `tpt/v1/<nodeId>/evt/<event>`

QoS 1, retained false.

- `imaging.progress`: `{ "frame": <int>, "total": <int>, "objectKey": "<s3-key>" }`
- `too` (Target of Opportunity): `{ "objectId": "<id>", "ra": <deg>, "dec": <deg>, "magDelta": <float>, "confidence": <0..1>, "imageKey": "<s3-key>" }`

## Status / LWT

Topic: `tpt/v1/<nodeId>/status`

- Last-Will topic with payload `{ "online": false, "ts": "..." }`.
- Published periodically (heartbeat, e.g. 15s) with `{ "online": true, "fw": "<version>", "uptime": <s> }`.

## S3 Data Flow

After an `imaging.progress` event with an `objectKey`, the Cloud worker
triggers Astrometry.net against the object in S3 and publishes
`tpt/v1/<nodeId>/evt/astrometry.done`.

## Open Questions

- Per-node ACL / topic-scoped auth on the broker.
- Telemetry batching to reduce publish volume.
