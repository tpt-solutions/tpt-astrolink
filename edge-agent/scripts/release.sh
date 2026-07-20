#!/usr/bin/env bash
# Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.
#
# Build, checksum and (optionally) sign a TPT Edge Agent release for the
# Raspberry Pi 5 (aarch64) and Intel NUC (x86_64) targets, then
# emit a signed release manifest consumed by crates/update at the
# node (see docs/security-review.md). Signing uses Ed25519; the
# agent verifies the signature with the public key in TPT_UPDATE_PUBKEY.
#
# Usage:
#   ./release.sh 1.2.3 [--sign-with /path/to/ed25519-priv.pem]
#
# The private key is never shipped. Generate one with:
#   openssl pkey -genpkey -algorithm ed25519 -out ed25519-priv.pem
#   openssl pkey -pubout -in ed25519-priv.pem -out ed25519-pub.pem
# The hex public key the agent needs:
#   openssl pkey -pubin -in ed25519-pub.pem -outform DER \
#     | tail -c 32 | xxd -p | tr -d '\n'

set -euo pipefail

VERSION="${1:?version required, e.g. 1.2.3}"
SIGN_PEM=""
if [[ "${2:-}" == "--sign-with" ]]; then
  SIGN_PEM="$3"
fi

TARGETS=(aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu)
OUT="release-artifacts"
mkdir -p "$OUT"

for t in "${TARGETS[@]}"; do
  echo "==> building $t"
  cargo build --release --target "$t"

  stage="$OUT/$t"
  mkdir -p "$stage"
  tar -C "target/$t/release" -czf "$stage/tpt-edge-agent-$VERSION.tar.gz" \
      tpt-edge-agent tpt-edge-watchdog

  url="https://updates.tpt.example/$VERSION/$t/tpt-edge-agent-$VERSION.tar.gz"
  sha=$(sha256sum "$stage/tpt-edge-agent-$VERSION.tar.gz" | cut -d' ' -f1)

  sig=""
  if [[ -n "$SIGN_PEM" ]]; then
    # Ed25519 sign (raw, 64-byte sig) over the artifact bytes.
    sig=$(openssl pkeyutl -sign -inkey "$SIGN_PEM" -rawin \
            -in "$stage/tpt-edge-agent-$VERSION.tar.gz" \
            | xxd -p | tr -d '\n')
  fi

  cat > "$stage/manifest.json" <<JSON
{
  "version": "$VERSION",
  "target": "$t",
  "url": "$url",
  "sha256": "$sha",
  "signature": "$sig",
  "notes": "TPT Edge Agent release $VERSION"
}
JSON
  echo "    wrote $stage/manifest.json (sha256=$sha)"
done

echo "release $VERSION staged under $OUT/"
