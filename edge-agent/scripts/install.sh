#!/usr/bin/env bash
# Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.
#
# Provision a TPT Edge Agent node on Raspberry Pi 5 (aarch64) or
# Intel NUC (x86_64). Installs the watchdog + agent binaries, drops a
# systemd unit, writes the environment file, and enables the service.
#
# Usage:
#   sudo ./install.sh --manifest-url https://updates.tpt.example/manifest.json \
#                       [--pubkey-hex <64-hex>] [--target auto]
#
# The agent performs OTA self-updates via tpt-edge-watchdog (see
# crates/update). This script is the *initial* provisioning only; after
# that, updates are delivered over the air and verified (sha256 + Ed25519).

set -euo pipefail

MANIFEST_URL=""
PUBKEY_HEX=""
TARGET="auto"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --manifest-url) MANIFEST_URL="$2"; shift 2 ;;
    --pubkey-hex)   PUBKEY_HEX="$2";   shift 2 ;;
    --target)        TARGET="$2";        shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [[ -z "$MANIFEST_URL" ]]; then
  echo "error: --manifest-url is required" >&2
  exit 1
fi

# --- detect target -------------------------------------------------------
if [[ "$TARGET" == "auto" ]]; then
  case "$(uname -m)" in
    aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
    x86_64|amd64) TARGET="x86_64-unknown-linux-gnu" ;;
    *) echo "error: unsupported arch $(uname -m); pass --target" >&2; exit 1 ;;
  esac
fi
echo "provisioning for target: $TARGET"

# --- service user --------------------------------------------------------
if ! id -u tptedge >/dev/null 2>&1; then
  useradd --system --no-create-home --shell /usr/sbin/nologin tptedge
fi
mkdir -p /var/lib/tpt-edge-agent/staging
chown -R tptedge:tptedge /var/lib/tpt-edge-agent

# --- install binaries ---------------------------------------------------
INSTALL_BIN=/usr/local/bin
install -m 0755 "target/$TARGET/release/tpt-edge-agent" "$INSTALL_BIN/"
install -m 0755 "target/$TARGET/release/tpt-edge-watchdog" "$INSTALL_BIN/"

# --- environment file ----------------------------------------------------
ENV_FILE=/etc/tpt-edge-agent/env
mkdir -p "$(dirname "$ENV_FILE")"
{
  echo "TPT_UPDATE_MANIFEST_URL=$MANIFEST_URL"
  [[ -n "$PUBKEY_HEX" ]] && echo "TPT_UPDATE_PUBKEY=$PUBKEY_HEX"
  # Operator overrides (MQTT broker, S3 bucket, node id) live here too.
  # TPT_MQTT_BROKER=...
  # TPT_S3_BUCKET=...
  # TPT_NODE_ID=...
} > "$ENV_FILE"
chmod 0600 "$ENV_FILE"
chown root:root "$ENV_FILE"

# --- systemd -------------------------------------------------------------
install -m 0644 infra/systemd/tpt-edge-watchdog.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now tpt-edge-watchdog

echo "done. status:"
systemctl status tpt-edge-watchdog --no-pager || true
