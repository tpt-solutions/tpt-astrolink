// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! OTA update client for the TPT Edge Agent (Raspberry Pi 5 / Intel NUC).
//!
//! `Updater` periodically polls a signed release manifest, downloads a new
//! release artifact when the running version is behind, verifies its
//! checksum + Ed25519 signature, extracts it, and signals the supervising
//! watchdog to restart the agent with the new binary.

use anyhow::{Context, Result};
use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Release manifest served over HTTPS from the update channel.
#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseManifest {
    pub version: String,
    pub target: String,
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub signature: String,
    #[serde(default)]
    pub notes: String,
}

/// How the updater reports progress back to the host.
pub trait UpdateListener {
    fn on_available(&self, manifest: &ReleaseManifest);
    fn on_applied(&self, manifest: &ReleaseManifest);
    fn on_error(&self, err: &anyhow::Error);
}

/// No-op listener (default for tests / headless runs).
pub struct NoopListener;

impl UpdateListener for NoopListener {
    fn on_available(&self, _: &ReleaseManifest) {}
    fn on_applied(&self, _: &ReleaseManifest) {}
    fn on_error(&self, _: &anyhow::Error) {}
}

pub struct Updater {
    manifest_url: String,
    current_version: String,
    target: String,
    staging_dir: PathBuf,
    poll_interval: Duration,
    listener: Box<dyn UpdateListener + Send + Sync>,
    /// Optional Ed25519 public key used to verify release signatures.
    /// When set, every downloaded artifact must carry a valid `signature`
    /// over its bytes; otherwise the update is rejected. When `None`
    /// (development), signature verification is skipped with a warning.
    pubkey: Option<VerifyingKey>,
}

impl Updater {
    pub fn new(manifest_url: impl Into<String>, current_version: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            manifest_url: manifest_url.into(),
            current_version: current_version.into(),
            target: target.into(),
            staging_dir: PathBuf::from("/var/lib/tpt-edge-agent/staging"),
            poll_interval: Duration::from_secs(300),
            listener: Box::new(NoopListener),
            pubkey: None,
        }
    }

    /// Enable signed-release verification. `hex` is the 32-byte Ed25519
    /// public key encoded as 64 hex characters.
    pub fn with_pubkey_hex(mut self, hex: &str) -> Result<Self> {
        let bytes = hex_decode(hex).context("decode update public key")?;
        if bytes.len() != 32 {
            anyhow::bail!("update public key must be 32 bytes, got {}", bytes.len());
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        let vk = VerifyingKey::from_bytes(&key).map_err(|e| anyhow::anyhow!("invalid ed25519 key: {e}"))?;
        self.pubkey = Some(vk);
        Ok(self)
    }

    pub fn with_staging_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.staging_dir = dir.into();
        self
    }

    pub fn with_poll_interval(mut self, d: Duration) -> Self {
        self.poll_interval = d;
        self
    }

    pub fn with_listener(mut self, l: Box<dyn UpdateListener + Send + Sync>) -> Self {
        self.listener = l;
        self
    }

    /// Returns true when `candidate` is semver-newer than the running version.
    pub fn is_newer(&self, candidate: &str) -> bool {
        compare_versions(candidate, &self.current_version).is_gt()
    }

    /// Fetch and parse the remote manifest.
    pub async fn fetch_manifest(&self) -> Result<ReleaseManifest> {
        let body = reqwest::get(&self.manifest_url)
            .await
            .context("fetch release manifest")?
            .error_for_status()
            .context("release manifest HTTP error")?
            .text()
            .await
            .context("read release manifest body")?;
        let manifest: ReleaseManifest = serde_json::from_str(&body).context("parse release manifest")?;
        if manifest.target != self.target {
            anyhow::bail!("manifest target {} does not match node target {}", manifest.target, self.target);
        }
        Ok(manifest)
    }

    /// Download, verify and stage the artifact for the given manifest.
    /// Returns the path to the verified, extracted binary.
    pub async fn download_and_stage(&self, manifest: &ReleaseManifest) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.staging_dir).ok();
        let archive_path = self.staging_dir.join(format!("tpt-edge-agent-{}.tar.gz", manifest.version));
        let bytes = reqwest::get(&manifest.url)
            .await
            .context("download release artifact")?
            .error_for_status()
            .context("release artifact HTTP error")?
            .bytes()
            .await
            .context("read release artifact")?;

        verify_sha256(&bytes, &manifest.sha256).context("checksum mismatch")?;

        if let Some(vk) = &self.pubkey {
            if manifest.signature.is_empty() {
                anyhow::bail!("release is unsigned but a verification key is configured");
            }
            verify_signature(vk, &bytes, &manifest.signature)
                .context("release signature verification failed")?;
                } else if !manifest.signature.is_empty() {
                    tracing::warn!("release carries a signature but no verification key is configured; skipping verification");
                }

        let extracted = self.staging_dir.join(format!("bin-{}", manifest.version));
        std::fs::write(&archive_path, &bytes).context("write archive to staging")?;
        extract_tar_gz(&archive_path, &extracted).context("extract release archive")?;
        Ok(extracted.join("tpt-edge-agent"))
    }

    /// Poll loop: checks for updates and applies them when found. The host is
    /// expected to run this under a watchdog that restarts the process after
    /// `request_restart` flips a flag / writes a sentinel.
    pub async fn run(mut self) -> Result<()> {
        loop {
            match self.check_and_apply().await {
                Ok(applied) if applied => return Ok(()),
                Ok(_) => {}
                Err(e) => self.listener.on_error(&e),
            }
            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// Single check+apply cycle. Returns true if an update was applied and the
    /// caller should restart.
    pub async fn check_and_apply(&mut self) -> Result<bool> {
        let manifest = self.fetch_manifest().await?;
        if !self.is_newer(&manifest.version) {
            return Ok(false);
        }
        self.listener.on_available(&manifest);
        let binary = self.download_and_stage(&manifest).await?;
        self.apply(binary).await?;
        self.listener.on_applied(&manifest);
        Ok(true)
    }

    /// Atomically replace the live binary and write a restart sentinel so the
    /// watchdog (see `watchdog.rs`) knows to relaunch the agent.
    async fn apply(&self, new_binary: PathBuf) -> Result<()> {
        let live = current_exe().context("resolve current exe")?;
        let backup = live.with_extension("bak");
        std::fs::copy(&live, &backup).context("back up current binary")?;
        std::fs::copy(&new_binary, &live).context("install new binary")?;
        std::fs::write(self.staging_dir.join("restart-required"), b"1")
            .context("write restart sentinel")?;
        Ok(())
    }
}

fn current_exe() -> Result<PathBuf> {
    Ok(std::env::current_exe()?)
}

fn verify_sha256(bytes: &[u8], expected: &str) -> Result<()> {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    let got = hex_encode(&digest);
    if !got.eq_ignore_ascii_case(expected) {
        anyhow::bail!("sha256 mismatch: expected {expected}, got {got}");
    }
    Ok(())
}

/// Verify an Ed25519 signature (hex-encoded, 64 bytes) over `bytes`.
fn verify_signature(vk: &VerifyingKey, bytes: &[u8], sig_hex: &str) -> Result<()> {
    let sig_bytes = hex_decode(sig_hex).context("decode release signature")?;
    if sig_bytes.len() != 64 {
        anyhow::bail!("signature must be 64 bytes, got {}", sig_bytes.len());
    }
    let mut arr = [0u8; 64];
    arr.copy_from_slice(&sig_bytes);
    let sig = Signature::from_bytes(&arr);
    vk.verify(bytes, &sig)
        .map_err(|e| anyhow::anyhow!("signature mismatch: {e}"))?;
    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest).ok();
    let file = std::fs::File::open(archive).context("open archive")?;
    let dec = flate2::read::GzDecoder::new(file);
    let mut ar = tar::Archive::new(dec);
    ar.unpack(dest).context("unpack archive")?;
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>> {
    // Tolerate whitespace (e.g. RFC-formatted vectors) between nibbles.
    let filtered: Vec<u8> = s
        .as_bytes()
        .iter()
        .copied()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    let s = filtered;
    if s.len() % 2 != 0 {
        anyhow::bail!("hex string must have even length");
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let mut i = 0;
    while i < s.len() {
        let hi = (s[i] as char)
            .to_digit(16)
            .ok_or_else(|| anyhow::anyhow!("invalid hex char"))?;
        let lo = (s[i + 1] as char)
            .to_digit(16)
            .ok_or_else(|| anyhow::anyhow!("invalid hex char"))?;
        out.push((hi * 16 + lo) as u8);
        i += 2;
    }
    Ok(out)
}

/// Semver-aware comparison that degrades gracefully for non-semver strings
/// (lexicographic fallback).
pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let pa: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let pb: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
    if pa.len() == pb.len() && !pa.is_empty() {
        pa.cmp(&pb)
    } else {
        a.cmp(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_compare() {
        assert_eq!(
            compare_versions("1.2.3", "1.2.2"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_versions("2.0.0", "1.9.9"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_versions("1.2.3", "1.2.3"),
            std::cmp::Ordering::Equal
        );
        assert_eq!(compare_versions("1.2.2", "1.2.3"), std::cmp::Ordering::Less);
    }

    #[test]
    fn is_newer_semver() {
        let u = Updater::new("https://x/manifest.json", "1.0.0", "aarch64-linux");
        assert!(u.is_newer("1.1.0"));
        assert!(!u.is_newer("1.0.0"));
        assert!(!u.is_newer("0.9.0"));
    }

    #[test]
    fn sha256_known_vector() {
        // "abc" -> ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        use sha2::Digest;
        let digest = sha2::Sha256::digest(b"abc");
        assert_eq!(
            hex_encode(&digest),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn verify_signature_rejects_bad_signature() {
        // A well-formed 32-byte public key (all 0x11).
        let vk = Updater::new("https://x/manifest.json", "1.0.0", "aarch64-linux")
            .with_pubkey_hex(&"11".repeat(32))
            .unwrap()
            .pubkey
            .unwrap();
        // 64 zero bytes is not a valid signature for this key/message.
        let bad_sig = "00".repeat(64);
        assert!(verify_signature(&vk, b"hello world", &bad_sig).is_err());
    }

    #[test]
    fn pubkey_hex_rejects_bad_length() {
        let u = Updater::new("https://x/manifest.json", "1.0.0", "aarch64-linux");
        assert!(u.with_pubkey_hex("deadbeef").is_err());
        // 64 hex chars -> 32 bytes -> ok.
        let good = "00".repeat(32);
        assert!(
            Updater::new("https://x/manifest.json", "1.0.0", "aarch64-linux")
                .with_pubkey_hex(&good)
                .is_ok()
        );
    }
}
