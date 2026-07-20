// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! TPT Edge Agent — main binary.
//!
//! Wires together device control (FFI), MQTT transport, S3 upload,
//! imaging pipeline and the edge-AI transient detector.

mod config;
mod devices;
mod commands;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

use crate::commands::CommandBus;
use crate::config::Config;
use crate::devices::DeviceHub;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let config = Config::from_env()?;
    tracing::info!(node_id = %config.node_id, "starting TPT Edge Agent");

    let hub = DeviceHub::connect(&config).await?;
    let mut bus = CommandBus::new(hub, &config).await?;

    // Optional OTA self-update loop, supervised by tpt-edge-watchdog.
    if let Some(manifest_url) = &config.update_manifest_url {
        let target = std::env::consts::ARCH.to_string() + "-" + std::env::consts::OS;
        let version = env!("CARGO_PKG_VERSION").to_string();
        let manifest_url = manifest_url.clone();
        tokio::spawn(async move {
            let updater = tpt_edge_update::Updater::new(manifest_url, version, target);
            if let Err(e) = updater.run().await {
                tracing::error!(error = %e, "ota updater exited");
            }
        });
    }

    bus.run().await?;
    Ok(())
}
