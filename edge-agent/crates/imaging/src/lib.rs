// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! FITS capture + compression pipeline. Captures frames, gzips them, and
//! hands bytes to the S3 uploader. Hardware capture is stubbed for now
//! (Phase 2 integrates the camera via FFI).

use anyhow::Result;
use tpt_edge_s3::S3Uploader;
use tracing::info;

pub struct CapturePipeline {
    uploader: S3Uploader,
    active: bool,
}

impl CapturePipeline {
    pub async fn new(bucket: &str, region: &str) -> Result<Self> {
        Ok(Self {
            uploader: S3Uploader::new(bucket, region).await?,
            active: false,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        self.active = true;
        info!("imaging sequence started");
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.active = false;
        info!("imaging sequence stopped");
        Ok(())
    }

    /// Capture one frame and upload it as a gzipped FITS object.
    pub async fn capture_frame(
        &self,
        node_id: &str,
        obs_id: &str,
        frame: u32,
    ) -> Result<String> {
        let raw = Self::capture_raw()?;
        let compressed = Self::gzip(&raw)?;
        let key = Self::object_key(node_id, obs_id, frame);
        self.uploader.upload_fits(&key, compressed).await?;
        Ok(key)
    }

    fn capture_raw() -> Result<Vec<u8>> {
        // TODO(Phase 2): real camera capture via FFI.
        Ok(vec![0u8; 0])
    }

    fn gzip(data: &[u8]) -> Result<Vec<u8>> {
        use std::io::Write;
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        e.write_all(data)?;
        Ok(e.finish()?)
    }

    fn object_key(node_id: &str, obs_id: &str, frame: u32) -> String {
        format!(
            "fits/raw/{}/{}/{}.fits.gz",
            node_id,
            obs_id,
            frame
        )
    }
}
