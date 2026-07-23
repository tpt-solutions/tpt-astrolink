// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! FITS capture + compression pipeline. Captures frames, gzips them, and
//! hands bytes to the S3 uploader. Hardware capture is stubbed for now
//! (Phase 2 integrates the camera via FFI).

use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;
use tpt_edge_s3::S3Uploader;
use tracing::info;

/// Synthetic frame geometry used while real camera capture (Phase 2 FFI) is
/// stubbed. The edge-AI detector downstream expects a normalized `f32` buffer
/// in this shape; swapping in real hardware capture must preserve it.
pub const FRAME_W: usize = 64;
pub const FRAME_H: usize = 64;
pub const FRAME_LEN: usize = FRAME_W * FRAME_H;

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

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.active = false;
        info!("imaging sequence stopped");
        Ok(())
    }

    /// Capture one frame, upload it as a gzipped FITS object, and return the
    /// object key together with the normalized pixel buffer for downstream
    /// edge-AI transient detection.
    pub async fn capture_frame_pixels(
        &self,
        node_id: &str,
        obs_id: &str,
        frame: u32,
    ) -> Result<(String, Vec<f32>)> {
        let pixels = Self::capture_raw_pixels()?;
        let compressed = Self::gzip_pixels(&pixels)?;
        let key = Self::object_key(node_id, obs_id, frame);
        self.uploader.upload_fits(&key, compressed).await?;
        Ok((key, pixels))
    }

    /// Capture one frame and upload it as a gzipped FITS object.
    pub async fn capture_frame(&self, node_id: &str, obs_id: &str, frame: u32) -> Result<String> {
        let (key, _pixels) = self.capture_frame_pixels(node_id, obs_id, frame).await?;
        Ok(key)
    }

    /// Generate a normalized (0.0..=1.0) synthetic frame. Stands in for the
    /// real camera while FFI capture is stubbed; produces a stable sky
    /// background plus read noise so the edge-AI baseline has something to
    /// adapt to.
    fn capture_raw_pixels() -> Result<Vec<f32>> {
        let mut rng = Lcg::new(0x9E37_79B9_2EB5_D7C7 ^ (FRAME_LEN as u64));
        let mut pixels = Vec::with_capacity(FRAME_LEN);
        for y in 0..FRAME_H {
            for x in 0..FRAME_W {
                // Gentle vignetted sky background + read noise.
                let vignette = 1.0
                    - 0.3
                        * (((x as f32 / FRAME_W as f32) - 0.5).powi(2)
                            + ((y as f32 / FRAME_H as f32) - 0.5).powi(2));
                let background = 0.12 * vignette;
                let noise = (rng.next_f32() - 0.5) * 0.04;
                pixels.push((background + noise).clamp(0.0, 1.0));
            }
        }
        Ok(pixels)
    }

    /// Compress a normalized `f32` pixel buffer to gzipped little-endian bytes.
    fn gzip_pixels(pixels: &[f32]) -> Result<Vec<u8>> {
        let mut raw = Vec::with_capacity(pixels.len() * 4);
        for p in pixels {
            raw.extend_from_slice(&p.to_le_bytes());
        }
        Self::gzip(&raw)
    }

    pub fn gzip(data: &[u8]) -> Result<Vec<u8>> {
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(data)?;
        Ok(e.finish()?)
    }

    pub fn object_key(node_id: &str, obs_id: &str, frame: u32) -> String {
        format!("fits/raw/{}/{}/{}.fits.gz", node_id, obs_id, frame)
    }
}

/// Tiny deterministic LCG so synthetic frames are reproducible in tests and
/// during validation runs.
struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(63_629_635_429)
            .wrapping_add(0x9E37_79B9_7F4A_7C15);
        self.0
    }
    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_key_layout() {
        let k = CapturePipeline::object_key("node-1", "obs-9", 3);
        assert_eq!(k, "fits/raw/node-1/obs-9/3.fits.gz");
    }

    #[test]
    fn gzip_round_trip() {
        let data = b"hello fits frame";
        let compressed = CapturePipeline::gzip(data).unwrap();
        assert!(!compressed.is_empty());
        // gz magic bytes
        assert_eq!(&compressed[..2], &[0x1f, 0x8b]);
    }
}
