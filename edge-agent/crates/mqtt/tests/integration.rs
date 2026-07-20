// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! Integration test for the MQTT contract: a real in-process broker
//! (rumqttd) is started, the Edge Agent client subscribes to its command
//! topic, an external publisher sends a `cmd.slew` envelope, and the client
//! must receive and parse it.

use rumqttd::{Broker, Config, ConnectionSettings, ServerSettings};
use tpt_edge_mqtt::{MqttClient, MqttMessage};

fn start_broker() {
    let mut config = Config::default();
    config.id = 0;
    config.router.connect_retain_ms = 0;
    config.servers = vec![ServerSettings {
        name: "test".into(),
        listen: "127.0.0.1:1889".parse().unwrap(),
        tls: None,
        next_connection_delay_ms: 1,
        connections: ConnectionSettings {
            connection_timeout_ms: 1000,
            max_client_id_len: 256,
            throttle_delay_ms: 0,
            max_payload_size: 1024 * 1024,
            max_inflight_count: 100,
            max_inflight_size: 1024 * 1024,
            login_credentials: None,
        },
    }];
    let (mut broker, _shutdown) = Broker::new(config);
    std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let _ = broker.start().await;
            });
    });
}

#[tokio::test]
async fn command_received_and_parsed() {
    let _ = tracing_subscriber::fmt::try_init();
    start_broker();
    // Give the broker a moment to bind.
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    let mut client = MqttClient::connect("test-node", "localhost", 1889).unwrap();

    // External publisher sends a command on the node's command topic.
    let mut publisher = rumqttc::Client::new(
        rumqttc::MqttOptions::new("test-pub", "localhost", 1889),
        10,
    );
    let env = serde_json::json!({
        "cmd": "slew",
        "id": "cmd-1",
        "params": { "ra": 12.5, "dec": -30.0, "epoch": "J2000" }
    })
    .to_string();
    publisher
        .publish("tpt/v1/test-node/cmd/all", rumqttc::QoS::AtLeastOnce, false, env)
        .unwrap();

    // The edge client should receive and parse the command.
    let msg: MqttMessage = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if let Some(m) = client.next().await.unwrap() {
                return m;
            }
        }
    })
    .await
    .expect("timed out waiting for command");

    assert_eq!(msg.cmd, "slew");
    assert_eq!(msg.id, "cmd-1");
    let ra: f64 = msg.param("ra").unwrap();
    let dec: f64 = msg.param("dec").unwrap();
    assert!((ra - 12.5).abs() < 1e-9);
    assert!((dec + 30.0).abs() < 1e-9);
}

#[tokio::test]
async fn telemetry_publish_does_not_error() {
    let _ = tracing_subscriber::fmt::try_init();
    start_broker();
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    let client = MqttClient::connect("tele-node", "localhost", 1889).unwrap();
    let payload = serde_json::json!({"ra":1.0,"dec":2.0,"alt":0.0,"az":0.0,"tracking":false,"status":"idle"});
    client.publish_telemetry("mount", &payload).unwrap();
}
