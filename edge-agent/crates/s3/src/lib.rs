// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! S3 client for FITS upload. Key layout per docs/storage/s3-layout.md.

use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client;
use tracing::info;

pub struct S3Uploader {
    client: Client,
    bucket: String,
}

impl S3Uploader {
    pub async fn new(bucket: &str, region: &str) -> Result<Self> {
        let region_provider =
            RegionProviderChain::first_try(Some(aws_config::Region::new(region.to_string())));
        let cfg = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        Ok(Self {
            client: Client::new(&cfg),
            bucket: bucket.to_string(),
        })
    }

    /// Upload a gzipped FITS frame. `object_key` follows
    /// `fits/raw/<nodeId>/<yyyy>/<mm>/<dd>/<obsId>/<frame>.fits.gz`.
    pub async fn upload_fits(&self, object_key: &str, data: Vec<u8>) -> Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(object_key)
            .body(data.into())
            .content_type("application/x-gzip-fits")
            .send()
            .await?;
        info!(key = object_key, "FITS uploaded");
        Ok(())
    }
}
