# Node Operator Setup Guide — TPT Edge Agent

Welcome, operator. This guide gets a **Raspberry Pi 5** or **Intel NUC**
running the TPT Edge Agent so it joins the AstroLink network and
(together with other nodes) crowdsources sky monitoring.

## What you need

| Item | Raspberry Pi 5 | Intel NUC |
|------|------------------|-------------|
| Board | Pi 5 (4/8 GB) | Any 64-bit NUC |
| OS | Raspberry Pi OS (64-bit) | Ubuntu 22.04/24.04 LTS |
| Mount/focuser | INDI- or ASCOM-driven | same |
| Network | Ethernet (prefered) | Ethernet |
| Power | Stable 5 V / 5 A | PoE or barrel |

You do **not** need to compile Rust — we ship signed images/binaries.

## 1. Flash & boot

- Flash Raspberry Pi OS (64-bit) to an SD card / NVMe.
- First boot: enable SSH, set hostname (e.g. `tpt-node-<yoursite>`),
  and join your observatory network.
- (Intel NUC) install Ubuntu Server 64-bit, create a user.

## 2. Install the agent

Copy the `tpt-astrolink` repo's `edge-agent/scripts/install.sh` to
the node, then run as root:

```bash
sudo ./install.sh \
  --manifest-url https://updates.tpt.example/manifest.json \
  --pubkey-hex <64-hex-char-ed25519-public-key>
```

This installs `tpt-edge-agent` + `tpt-edge-watchdog`, drops a
systemd unit, writes `/etc/tpt-edge-agent/env`, and starts the
**watchdog** (which keeps the agent alive and applies OTA updates).

## 3. Configure your site

Edit `/etc/tpt-edge-agent/env` and add your operator values:

```bash
# Your node identity (assigned by TPT during onboarding).
TPT_NODE_ID=tpt-node-<yoursite>

# MQTT broker the Cloud Core bridges to (provided at onboarding).
TPT_MQTT_BROKER=broker.astrolink.example
TPT_MQTT_PORT=8883            # TLS

# FITS upload target (S3 bucket/prefix from TPT).
TPT_S3_BUCKET=tpt-astrolink-prod-fits
TPT_S3_REGION=ap-southeast-2

# (Optional) local override of the OTA manifest.
# TPT_UPDATE_MANIFEST_URL=https://updates.tpt.example/manifest.json
```

Then: `sudo systemctl restart tpt-edge-watchdog`.

## 4. Connect your hardware

- **INDI:** start `indiserver` with your mount/focuser/weather
  drivers; the agent binds via the FFI layer.
- **ASCOM (Windows NUC):** register the ASCOM device drivers
  (Alpaca/USB); point the agent's FFI config at them.
- Verify in the Client UI: your node shows **online**, the mount
  slews from the dashboard, and the focuser moves.

## 5. Operate & update

- The agent **self-updates** over the air. Watchdog applies a new
  signed release, swaps the binary, and restarts — no manual
  action. Updates are SHA-256 + Ed25519 verified; unsigned builds
  are refused.
- To check health: `systemctl status tpt-edge-watchdog`.
- Logs: `journalctl -u tpt-edge-watchdog -f`.
- To pause updates, stop the watchdog and pin your version.

## 6. Troubleshooting

| Symptom | Fix |
|---------|-----|
| Node offline in UI | check MQTT TLS/creds in `/etc/tpt-edge-agent/env`; `journalctl` |
| Slew rejected | verify RA/Dec within driver limits; bad input is refused at the FFI boundary |
| No FITS in cloud | check `TPT_S3_*` env + node egress to S3 |
| Update stuck | `sudo systemctl restart tpt-edge-watchdog`; inspect staging dir `/var/lib/tpt-edge-agent/staging` |

## 7. Safety

- Keep clutches engaged and optics capped during first slew tests.
- The FFI layer rejects malformed commands before they reach hardware.
- See `docs/security-review.md` for the threat model.
