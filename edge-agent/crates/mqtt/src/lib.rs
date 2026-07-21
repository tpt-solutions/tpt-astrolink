// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! MQTT transport for the Edge Agent. Topic layout per docs/protocols/mqtt-contract.md.

use anyhow::Result;
use rumqttc::{Client, Connection, Event, Incoming, MqttOptions, QoS};
use serde::Serialize;
use std::sync::{Arc, Mutex};
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

/// Publisher abstracts outgoing MQTT publishes so the contract can be tested
/// without a live broker. Implemented for rumqttc::Client in production and
/// for a capture double in tests.
pub trait Publisher: Send + Sync {
    fn publish(&self, topic: &str, payload: &[u8]) -> Result<()>;
}

impl Publisher for Client {
    fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        self.publish(topic, QoS::AtLeastOnce, false, payload.to_vec())?;
        Ok(())
    }
}

pub struct MqttClient {
    node_id: String,
    publisher: Box<dyn Publisher>,
    eventloop: Connection,
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
        let (client, eventloop) = Client::new(opts, 10);
        let cmd_topic = format!("tpt/v1/{}/cmd/+", node_id);
        client.subscribe(cmd_topic, QoS::AtLeastOnce)?;
        Ok(Self {
            node_id: node_id.into(),
            publisher: Box::new(client),
            eventloop,
        })
    }

    /// Test-only constructor with an injected publisher.
    #[cfg(test)]
    pub fn with_publisher(node_id: &str, publisher: Box<dyn Publisher>) -> Self {
        let opts = MqttOptions::new(format!("node/{}", node_id), "localhost", 1883);
        let (_client, eventloop) = Client::new(opts, 10);
        Self {
            node_id: node_id.into(),
            publisher,
            eventloop,
        }
    }

    /// Poll the eventloop for the next command. Returns `Ok(None)` for
    /// keep-alive/ack events that should be ignored.
    pub async fn next(&mut self) -> Result<Option<MqttMessage>> {
        loop {
            match self.eventloop.eventloop.poll().await? {
                Event::Incoming(Incoming::Publish(p)) => {
                    if let Some(msg) = parse_incoming(&Incoming::Publish(p)) {
                        return Ok(Some(msg));
                    }
                }
                _ => continue,
            }
        }
    }

    pub fn publish_telemetry<T: Serialize>(&self, device: &str, payload: &T) -> Result<()> {
        let topic = format!("tpt/v1/{}/tele/{}", self.node_id, device);
        let body = serde_json::to_string(payload)?;
        self.publisher.publish(&topic, body.as_bytes())?;
        debug!(device, "telemetry published");
        Ok(())
    }

    pub fn publish_event(&self, event: &str, payload: &str) -> Result<()> {
        let topic = format!("tpt/v1/{}/evt/{}", self.node_id, event);
        self.publisher.publish(&topic, payload.as_bytes())?;
        Ok(())
    }

    pub fn publish_status(&self, online: bool) -> Result<()> {
        let topic = format!("tpt/v1/{}/status", self.node_id);
        let body = format!(r#"{{"online":{}}}"#, online);
        self.publisher.publish(&topic, body.as_bytes())?;
        Ok(())
    }
}

/// Extract a MqttMessage from an Incoming Publish (used by the run loop and
/// the integration tests). Mirrors docs/protocols/mqtt-contract.md.
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

#[cfg(test)]
mod contract_tests {
    use super::*;

    /// Capture records every (topic, payload) for contract assertions. The
    /// recorded vector is shared via Arc so tests can read it after publishing.
    struct Capture {
        recorded: Arc<Mutex<Vec<(String, String)>>>,
    }
    impl Publisher for Capture {
        fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
            self.recorded.lock().unwrap().push((
                topic.to_string(),
                String::from_utf8_lossy(payload).into_owned(),
            ));
            Ok(())
        }
    }

    fn capture() -> (Box<dyn Publisher>, Arc<Mutex<Vec<(String, String)>>>) {
        let recorded: Arc<Mutex<Vec<(String, String)>>> = Default::default();
        let cap = Capture {
            recorded: recorded.clone(),
        };
        (Box::new(cap), recorded)
    }

    #[test]
    fn telemetry_topic_and_payload() {
        let (pub_box, recorded) = capture();
        let client = MqttClient::with_publisher("n1", pub_box);
        let payload = serde_json::json!({"ra":1.0,"status":"idle"});
        client.publish_telemetry("mount", &payload).unwrap();

        let pubs = recorded.lock().unwrap();
        assert_eq!(pubs.len(), 1);
        assert_eq!(pubs[0].0, "tpt/v1/n1/tele/mount");
        assert_eq!(pubs[0].1, r#"{"ra":1.0,"status":"idle"}"#);
    }

    #[test]
    fn event_and_status_topics() {
        let (pub_box, recorded) = capture();
        let client = MqttClient::with_publisher("n2", pub_box);
        client.publish_event("too", r#"{"objectId":"x"}"#).unwrap();
        client.publish_status(true).unwrap();

        let pubs = recorded.lock().unwrap();
        assert_eq!(pubs[0].0, "tpt/v1/n2/evt/too");
        assert_eq!(pubs[1].0, "tpt/v1/n2/status");
        assert_eq!(pubs[1].1, r#"{"online":true}"#);
    }

    #[test]
    fn parse_incoming_extracts_contract() {
        let raw = br#"{"cmd":"slew","id":"c1","params":{"ra":12.5,"dec":-30.0}}"#;
        let incoming = rumqttc::Incoming::Publish(rumqttc::Publish::new(
            "tpt/v1/n/cmd/all",
            rumqttc::QoS::AtLeastOnce,
            raw.to_vec(),
        ));
        let msg = parse_incoming(&incoming).expect("should parse");
        assert_eq!(msg.cmd, "slew");
        assert_eq!(msg.id, "c1");
        let ra: f64 = msg.param("ra").unwrap();
        assert!((ra - 12.5).abs() < 1e-9);
    }
}
