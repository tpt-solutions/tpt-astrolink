// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! Command/telemetry dispatch loop. Bridges MQTT messages to device actions
//! and publishes telemetry + events back to the broker.

use crate::config::Config;
use crate::devices::DeviceHub;
use anyhow::Result;
use std::time::Duration;
use tpt_edge_ai::TransientDetector;
use tpt_edge_imaging::CapturePipeline;
use tpt_edge_mqtt::{MqttClient, MqttMessage};
use tpt_edge_ffi::Epoch;
use tracing::{info, warn};

pub struct CommandBus {
    hub: DeviceHub,
    mqtt: MqttClient,
    imaging: CapturePipeline,
    detector: TransientDetector,
}

impl CommandBus {
    pub async fn new(hub: DeviceHub, config: &Config) -> Result<Self> {
        let mqtt = MqttClient::connect(&config.node_id, &config.mqtt_broker, config.mqtt_port)?;
        let imaging = CapturePipeline::new(&config.s3_bucket, &config.s3_region).await?;
        let detector = TransientDetector::load_default()?;
        Ok(Self { hub, mqtt, imaging, detector })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            tokio::select! {
                msg = self.mqtt.next() => {
                    let msg = msg?;
                    self.handle_command(msg).await?;
                }
                _ = interval.tick() => {
                    self.publish_telemetry().await?;
                }
            }
        }
    }

    async fn handle_command(&mut self, msg: MqttMessage) -> Result<()> {
        match msg.cmd.as_str() {
            "slew" => {
                let (ra, dec) = parse_coords(&msg)?;
                self.hub.slew(ra, dec, Epoch::J2000)?;
                info!(ra, dec, "mount slew");
            }
            "slewStop" => self.hub.stop()?,
            "focus" => {
                let pos: u32 = msg.param("position")?;
                self.hub.focus_to(pos)?;
            }
            "focusRelative" => {
                let delta: i32 = msg.param("delta")?;
                self.hub.focus_relative(delta)?;
            }
            "imaging.start" => self.imaging.start().await?,
            "imaging.stop" => self.imaging.stop().await?,
            "weather.refresh" => self.publish_weather().await?,
            other => warn!(cmd = other, "unknown command"),
        }
        Ok(())
    }

    async fn publish_telemetry(&self) -> Result<()> {
        if let Ok(s) = self.hub.mount_state() {
            self.mqtt.publish_telemetry("mount", &s)?;
        }
        if let Ok(s) = self.hub.focuser_state() {
            self.mqtt.publish_telemetry("focuser", &s)?;
        }
        self.publish_weather().await?;
        Ok(())
    }

    async fn publish_weather(&self) -> Result<()> {
        if let Ok(s) = self.hub.weather_sample() {
            self.mqtt.publish_telemetry("weather", &s)?;
        }
        Ok(())
    }
}

fn parse_coords(msg: &MqttMessage) -> Result<(f64, f64)> {
    let ra: f64 = msg.param("ra")?;
    let dec: f64 = msg.param("dec")?;
    Ok((ra, dec))
}
