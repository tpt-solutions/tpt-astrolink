# Beta Onboarding — Pilot Observatories

**Program:** TPT AstroLink "Project Cosmos" beta · **Owner:** TPT Solutions

This is the runbook for bringing the first pilot nodes (pro-am
observatories) onto the AstroLink virtual-observatory network.

## 1. Who this is for

- A pro-am or small research observatory with a **computerized mount**
  (INDI or ASCOM), a **focuser**, and (ideally) a **weather
  station**. OS: Raspberry Pi 5 (Pi OS 64-bit) or Intel NUC
  (Ubuntu 64-bit).
- Sites willing to share idle night-time for crowdsourced
  transient monitoring in exchange for network access + dashboards.

## 2. Onboarding steps

1. **Apply** via the TPT beta portal; provide site coords,
   timezone, and equipment list (mount / focuser / camera / weather).
2. **Provision** the edge node using the operator guide
   (`docs/node-operator-guide.md`) — flash OS, run
   `scripts/install.sh`, set `/etc/tpt-edge-agent/env`.
3. **Assign ID + creds:** TPT issues `TPT_NODE_ID` and the MQTT
   broker TLS endpoint + client cert; record them in the env file.
4. **Connect hardware:** INDI server or ASCOM drivers per the
   operator guide; confirm the node shows **online** in the Client UI.
5. **Smoke test:** from the UI, slew 3° to a bright star, focus in,
   start a 4-frame imaging sequence; confirm telemetry + FITS in S3.
6. **Sign the beta agreement** (data usage + uptime expectation).

## 3. Pilot cohorts (wave plan)

| Wave | Sites | Focus |
|------|-------|-------|
| W1 | 3 | Hardware/FFI bring-up, HIL checks |
| W2 | 8 | Relay scheduling + stitching dry-run |
| W3 | 15 | Edge-AI ToO alerts in real night sky |

## 4. Success criteria per site

- Node online ≥ 80% of local night-time over a 14-day window.
- Command round-trip (UI→mount) observed **sub-second** end to end.
- ≥ 1 successful imaging sequence + astrometry write-back to Postgres.
- (W3) ≥ 1 edge-AI Target-of-Opportunity alert, validated
  against the catalog.

## 5. Comms & support

- Pilot Slack/Matrix channel (invite sent at step 2).
- Triage SLA: 1 business day for "node offline" tickets.
- Monthly beta call; changelog shared before each OTA release.

## 6. Offboarding / rollback

- To leave the beta: `systemctl stop tpt-edge-watchdog`, remove
  the env file, or simply disconnect hardware. No data is held
  locally on the node beyond the staging dir.
- OTA rollback is automatic (`.bak` binary) if an apply fails.
