// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! Watchdog for the TPT Edge Agent.
//!
//! Supervises `tpt-edge-agent`, restarting it on crash and after the OTA
//! updater writes a `restart-required` sentinel. Keeps the agent alive across
//! transient device/network failures on Raspberry Pi 5 / Intel NUC hardware.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const RESTART_SENTINEL: &str = "restart-required";
const STAGING_DIR: &str = "/var/lib/tpt-edge-agent/staging";
const MAX_BACKOFF: Duration = Duration::from_secs(30);

fn main() {
    tracing_subscriber::fmt::init();
    let staging = PathBuf::from(STAGING_DIR);
    let sentinel = staging.join(RESTART_SENTINEL);
    let mut backoff = Duration::from_secs(1);
    let mut child: Option<Child> = None;

    loop {
        if sentinel.exists() {
            let _ = std::fs::remove_file(&sentinel);
            tracing::info!("update sentinel seen; restarting agent with new binary");
            if let Some(mut c) = child.take() {
                let _ = c.kill();
                let _ = c.wait();
            }
            backoff = Duration::from_secs(1);
        }

        if child.is_none() {
            match spawn_agent() {
                Ok(c) => {
                    tracing::info!("agent started (pid {})", c.id());
                    child = Some(c);
                    backoff = Duration::from_secs(1);
                }
                Err(e) => {
                    tracing::error!(error = %e, "failed to start agent; retrying in {:?}", backoff);
                    std::thread::sleep(backoff);
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                    continue;
                }
            }
        }

        // Poll the child; if it exits, restart it.
        if let Some(c) = child.as_mut() {
            match c.try_wait() {
                Ok(Some(status)) => {
                    tracing::warn!(status = %status, "agent exited; restarting");
                    child = None;
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(500)),
                Err(e) => {
                    tracing::error!(error = %e, "agent wait error; restarting");
                    child = None;
                }
            }
        }
    }
}

fn spawn_agent() -> std::io::Result<Child> {
    Command::new("tpt-edge-agent")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
}
