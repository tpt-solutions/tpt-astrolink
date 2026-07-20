// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub node_id: String,
    pub mqtt_broker: String,
    pub mqtt_port: u16,
    pub s3_bucket: String,
    pub s3_region: String,
    pub update_manifest_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            node_id: env::var("TPT_NODE_ID")?,
            mqtt_broker: env::var("TPT_MQTT_BROKER").unwrap_or_else(|_| "localhost".into()),
            mqtt_port: env::var("TPT_MQTT_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1883),
            s3_bucket: env::var("TPT_S3_BUCKET").unwrap_or_else(|_| "tpt-astrolink-fits".into()),
            s3_region: env::var("TPT_S3_REGION").unwrap_or_else(|_| "us-east-1".into()),
            update_manifest_url: env::var("TPT_UPDATE_MANIFEST_URL").ok(),
        })
    }
}
