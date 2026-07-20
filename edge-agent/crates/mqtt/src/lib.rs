// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! MQTT transport for the Edge Agent. Topic layout per docs/protocols/mqtt-contract.md.

use anyhow::Result;
use rumqttc::{Client, Event, Incoming, MqttOptions, QoS, Transport};
use serde::Serialize;
use std::time::Duration;
use tracing::debug;

#[derive(Debug)]
pub struct MqttMessage {
    pub cmd: String,
    pub id: String,
    pub params: serde_json::Value,
}

impl MqttMessage {
    pub fn param<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T> {
        let v = self
            .params
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("missing param '{}'", key))?;
        Ok(serde_json::from_value(v.clone())?)
    }
}

pub struct MqttClient {
    node_id: String,
    client: Client,
}

impl MqttClient {
    pub fn connect(node_id: &str, broker: &str, port: u16) -> Result<Self> {
        let mut opts = MqttOptions::new(format!("node/{}", node_id), broker, port);
        opts.set_keep_alive(Duration::from_secs(30));
        opts.set_last_will(rumqttc::LastWill::new(
            format!("tpt/v1/{}/status", node_id),
            r#"{"online":false}"#,
            QoS::AtLeastOnce,
            true,
        ));
        let (client, mut eventloop) = Client::new(opts, 10);
        let cmd_topic = format!("tpt/v1/{}/cmd/+", node_id);
        client.subscribe(cmd_topic, QoS::AtLeastOnce)?;
        // Drain initial connection events in a background task.
        let node = node_id.to_string();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let _ = &eventloop;
            });
        });
        Ok(Self {
            node_id: node_id.into(),
            client,
        })
    }

    pub fn next(&mut self) -> Result<MqttMessage> {
        // Polled by the command loop via the eventloop; simplified here.
        // In the full impl this borrows the eventloop. See commands.rs integration.
        Err(anyhow::anyhow!("use run() eventloop; next() placeholder"))
    }

    pub fn publish_telemetry<T: Serialize>(&self, device: &str, payload: &T) -> Result<()> {
        let topic = format!("tpt/v1/{}/tele/{}", self.node_id, device);
        let body = serde_json::to_string(payload)?;
        self.client.publish(topic, QoS::AtMostOnce, true, body)?;
        debug!(device, "telemetry published");
        Ok(())
    }

    pub fn publish_event(&self, event: &str, payload: &str) -> Result<()> {
        let topic = format!("tpt/v1/{}/evt/{}", self.node_id, event);
        self.client.publish(topic, QoS::AtLeastOnce, false, payload)?;
        Ok(())
    }

    pub fn publish_status(&self, online: bool) -> Result<()> {
        let topic = format!("tpt/v1/{}/status", self.node_id);
        let body = format!(r#"{{"online":{}}}"#, online);
        self.client.publish(topic, QoS::AtLeastOnce, true, body)?;
        Ok(())
    }
}

// Helper to extract a MqttMessage from an Incoming Publish (used by run loop).
pub fn parse_incoming(incoming: &Incoming) -> Option<MqttMessage> {
    match incoming {
        Incoming::Publish(p) => {
            let v: serde_json::Value = serde_json::from_slice(&p.payload).ok()?;
            Some(MqttMessage {
                cmd: v.get("cmd")?.as_str()?.to_string(),
                id: v.get("id")?.as_str()?.to_string(),
                params: v.get("params").cloned().unwrap_or(serde_json::Value::Null),
            })
        }
        _ => None,
    }
}

#[allow(dead_code)]
fn _event_marker(_e: Event) {}
