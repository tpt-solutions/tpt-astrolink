// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! Edge AI transient detector. Runs a brightness-anomaly model locally to flag
//! Target-of-Opportunity (ToO) events.
//!
//! Two execution paths are supported:
//!
//! 1. **ONNX inference** — when a model file is present at
//!    `TPT_TRANSIENT_MODEL` (or `models/transient/default/transient.onnx`),
//!    it is loaded with ONNX Runtime (`ort`) and run on a fixed brightness
//!    feature vector. The model contract is documented in
//!    `docs/edge-ai-model.md`.
//! 2. **Statistical fallback** — when no model is deployed (the default on a
//!    freshly flashed node), a rolling-baseline brightness-anomaly detector
//!    provides ToO triggering without any trained weights. This keeps the
//!    "Target of Opportunity" trigger functional out of the box and also backs
//!    the model-accuracy validation harness.

use anyhow::{Context, Result};
use serde::Serialize;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::Mutex;
use tracing::{warn};

/// Number of brightness features fed to the model / statistical scorer.
/// Must match the ONNX model's input dimension (see `docs/edge-ai-model.md`).
pub const FEATURE_DIM: usize = 8;

#[derive(Debug, Clone, Serialize)]
pub struct TooAlert {
    pub object_id: String,
    pub ra: f64,
    pub dec: f64,
    /// Change in apparent magnitude relative to the node baseline. Negative
    /// means the source brightened (a new/erupting transient).
    pub mag_delta: f64,
    pub confidence: f32,
    pub image_key: String,
}

/// Fixed brightness features extracted from a normalized (0.0..=1.0) frame.
#[derive(Debug, Clone, Copy)]
pub struct BrightnessFeatures {
    pub mean: f32,
    pub median: f32,
    pub std: f32,
    pub max: f32,
    pub min: f32,
    pub p95: f32,
    pub peak_count: f32,
    pub snr: f32,
}

/// Extract the fixed brightness feature vector used for transient scoring.
pub fn extract_features(pixels: &[f32]) -> BrightnessFeatures {
    if pixels.is_empty() {
        return BrightnessFeatures {
            mean: 0.0, median: 0.0, std: 0.0, max: 0.0, min: 0.0,
            p95: 0.0, peak_count: 0.0, snr: 0.0,
        };
    }
    let mut sorted = pixels.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    let mean = pixels.iter().copied().sum::<f32>() / n as f32;
    let variance = pixels.iter().map(|p| (p - mean).powi(2)).sum::<f32>() / n as f32;
    let std = variance.sqrt();
    let median = sorted[n / 2];
    let max = sorted[n - 1];
    let min = sorted[0];
    let p95 = sorted[(0.95 * (n - 1) as f32).round() as usize];
    // Pixels brighter than baseline+5sigma, a proxy for resolved sources.
    let thresh = median + 5.0 * std.max(1e-4);
    let peak_count = pixels.iter().filter(|&&p| p > thresh).count() as f32;
    let snr = if std > 1e-4 { max / std } else { 0.0 };
    BrightnessFeatures { mean, median, std, max, min, p95, peak_count, snr }
}

/// Flatten features to the model input vector (length [`FEATURE_DIM`]).
fn features_to_vec(f: &BrightnessFeatures) -> Vec<f32> {
    vec![f.mean, f.median, f.std, f.max, f.min, f.p95, f.peak_count, f.snr]
}

/// ONNX Runtime session wrapper. Loaded lazily from disk on the node.
struct OnnxModel {
    session: Mutex<ort::session::Session>,
    input_name: Cow<'static, str>,
    output_name: Cow<'static, str>,
}

impl OnnxModel {
    fn load(path: &str) -> Result<Self> {
        // Initialize the global ONNX Runtime environment once.
        let _ = ort::init().with_name("tpt-edge-ai").commit();
        let session = ort::session::Session::builder()?
            .commit_from_file(path)
            .with_context(|| format!("load ONNX model from {path}"))?;
        let input_name = session
            .inputs()
            .first()
            .map(|o| Cow::Owned(o.name().to_string()))
            .unwrap_or(Cow::Borrowed("input"));
        let output_name = session
            .outputs()
            .first()
            .map(|o| Cow::Owned(o.name().to_string()))
            .unwrap_or(Cow::Borrowed("output"));
        Ok(Self { session: Mutex::new(session), input_name, output_name })
    }

    /// Run inference. Returns `(confidence, mag_delta)`.
    fn infer(&self, features: &[f32]) -> Result<(f32, f32)> {
        let mut session = self
            .session
            .lock()
            .map_err(|_| anyhow::anyhow!("onnx session lock poisoned"))?;
        let input = ort::value::Tensor::from_array((
            vec![1i64, features.len() as i64],
            features.to_vec(),
        ))?;
        let mut inputs: Vec<(Cow<'_, str>, ort::session::SessionInputValue<'_>)> =
            Vec::with_capacity(1);
        inputs.push((self.input_name.clone(), input.into()));
        let outputs = session.run(inputs)?;
        let out = outputs[self.output_name.as_ref()].try_extract_tensor::<f32>()?;
        let data = out.1;
        if data.len() < 2 {
            anyhow::bail!("model output too short: expected >=2 values, got {}", data.len());
        }
        Ok((data[0].clamp(0.0, 1.0), data[1]))
    }
}

/// Rolling baseline of sky background used by the statistical fallback.
///
/// Uses a robust median + median-absolute-deviation (MAD) reference so a
/// transient (an extreme outlier) does not poison the baseline. Detected
/// anomalies are deliberately *not* ingested (see [`StatisticalDetector::score`]).
struct Baseline {
    peaks: VecDeque<f32>,
    capacity: usize,
}

impl Baseline {
    fn new(capacity: usize) -> Self {
        Self { peaks: VecDeque::with_capacity(capacity), capacity }
    }

    fn push(&mut self, peak: f32) {
        if self.peaks.len() >= self.capacity {
            self.peaks.pop_front();
        }
        self.peaks.push_back(peak);
    }

    fn median(&self) -> f32 {
        if self.peaks.is_empty() {
            return 0.0;
        }
        let mut v: Vec<f32> = self.peaks.iter().copied().collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        v[v.len() / 2]
    }

    /// Robust spread: 1.4826 * median(|x - median|).
    fn mad(&self, median: f32) -> f32 {
        if self.peaks.len() < 2 {
            return 0.0;
        }
        let mut devs: Vec<f32> = self.peaks.iter().map(|p| (p - median).abs()).collect();
        devs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mad = devs[devs.len() / 2];
        1.4826 * mad
    }
}

/// Statistical brightness-anomaly scorer (no trained weights required).
pub struct StatisticalDetector {
    inner: Mutex<Baseline>,
    window: usize,
    /// z-score at which confidence saturates to 1.0.
    z_scale: f32,
    /// Frames scoring above this confidence are treated as anomalies and are
    /// excluded from the baseline so they cannot adapt it upward.
    ingest_threshold: f32,
}

impl StatisticalDetector {
    fn new() -> Self {
        Self {
            inner: Mutex::new(Baseline::new(64)),
            window: 64,
            z_scale: 6.0,
            ingest_threshold: 0.5,
        }
    }

    /// Observe a frame and return `(confidence, mag_delta)`. While the
    /// baseline is still warming up, returns `(0.0, 0.0)` (no alert) and
    /// ingests the frame. Anomalous frames are reported but not ingested.
    fn score(&self, peak: f32, _median: f32) -> (f32, f32) {
        let mut b = self.inner.lock().unwrap();
        if b.peaks.len() < self.window / 2 {
            b.push(peak);
            return (0.0, 0.0);
        }
        let med = b.median();
        let spread = b.mad(med).max(1e-4);
        let z = (peak - med) / spread;
        let confidence = (z / self.z_scale).clamp(0.0, 1.0);
        // Brightening relative to the baseline median peak -> negative mag_delta.
        let base_peak = med.max(1e-4);
        let mag_delta = -2.5 * (peak / base_peak).log10();
        // Only ingest frames that look normal, so transients never poison the
        // baseline and repeated transients stay detectable.
        if confidence < self.ingest_threshold {
            b.push(peak);
        }
        (confidence, mag_delta)
    }
}

/// Transient detector: prefers the ONNX model when deployed, otherwise the
/// statistical fallback. Exposes a single `detect` entry point used by the
/// command bus to emit Target-of-Opportunity alerts.
pub struct TransientDetector {
    threshold: f32,
    model: Option<OnnxModel>,
    stats: Option<StatisticalDetector>,
}

impl TransientDetector {
    /// Load the default transient detector. Uses the model at
    /// `TPT_TRANSIENT_MODEL` when present, else the statistical fallback.
    pub fn load_default() -> Result<Self> {
        let model_path = std::env::var("TPT_TRANSIENT_MODEL")
            .unwrap_or_else(|_| "models/transient/default/transient.onnx".to_string());
        let (model, stats) = if Path::new(&model_path).exists() {
            match OnnxModel::load(&model_path) {
                Ok(m) => {
                    tracing::info!(model = %model_path, "loaded ONNX transient model");
                    (Some(m), None)
                }
                Err(e) => {
                    warn!(error = %e, "ONNX model load failed; using statistical fallback");
                    (None, Some(StatisticalDetector::new()))
                }
            }
        } else {
            tracing::info!("no ONNX transient model found; using statistical fallback");
            (None, Some(StatisticalDetector::new()))
        };
        Ok(Self { threshold: 0.85, model, stats })
    }

    /// Force the statistical (weight-free) detector. Used by tests and the
    /// model-accuracy validation harness.
    pub fn load_statistical() -> Result<Self> {
        Ok(Self { threshold: 0.85, model: None, stats: Some(StatisticalDetector::new()) })
    }

    /// Minimum confidence to emit a Target-of-Opportunity alert.
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Score a frame without producing an alert (used by validation).
    pub fn score(&self, pixels: &[f32]) -> (f32, f32) {
        let f = extract_features(pixels);
        match &self.model {
            Some(m) => match m.infer(&features_to_vec(&f)) {
                Ok(v) => v,
                Err(e) => {
                    warn!(error = %e, "onnx infer failed; statistical fallback");
                    self.stats_score(&f)
                }
            },
            None => self.stats_score(&f),
        }
    }

    fn stats_score(&self, f: &BrightnessFeatures) -> (f32, f32) {
        match &self.stats {
            Some(s) => s.score(f.max, f.median),
            None => (0.0, 0.0),
        }
    }

    /// Run detection on a captured frame (normalized pixel buffer). Returns an
    /// alert when a brightness anomaly exceeds the threshold.
    pub fn detect(&self, pixels: &[f32], ra: f64, dec: f64, image_key: &str) -> Option<TooAlert> {
        let (confidence, mag_delta) = self.score(pixels);
        if confidence >= self.threshold {
            Some(TooAlert {
                object_id: new_object_id(),
                ra,
                dec,
                mag_delta: mag_delta as f64,
                confidence,
                image_key: image_key.to_string(),
            })
        } else {
            None
        }
    }
}

/// Inject a Gaussian brightness transient into a normalized frame. Used by the
/// accuracy-validation harness and tests to synthesize transients.
pub fn inject_transient(
    pixels: &[f32],
    width: usize,
    cx: usize,
    cy: usize,
    radius: usize,
    amplitude: f32,
) -> Vec<f32> {
    let mut out = pixels.to_vec();
    let r2 = (radius * radius) as f32;
    for y in 0..out.len() / width {
        for x in 0..width {
            let dx = x as f32 - cx as f32;
            let dy = y as f32 - cy as f32;
            let d2 = dx * dx + dy * dy;
            if d2 <= r2 {
                let g = (-d2 / r2.max(1.0) * 2.0).exp();
                let idx = y * width + x;
                out[idx] = (out[idx] + amplitude * g).clamp(0.0, 1.0);
            }
        }
    }
    out
}

/// Generate a reproducible synthetic sky frame for tests/validation.
pub fn synthetic_frame(width: usize, height: usize, seed: u64) -> Vec<f32> {
    let mut rng = Lcg::new(seed);
    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let vignette = 1.0
                - 0.3 * (((x as f32 / width as f32) - 0.5).powi(2)
                    + ((y as f32 / height as f32) - 0.5).powi(2));
            let background = 0.12 * vignette;
            let noise = (rng.next_f32() - 0.5) * 0.04;
            pixels.push((background + noise).clamp(0.0, 1.0));
        }
    }
    pixels
}

/// Generate a process-local unique id for an observation/object.
pub fn new_object_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:032x}", nanos)
}

/// Tiny deterministic LCG for reproducible synthetic frames.
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
    fn object_ids_unique() {
        let a = new_object_id();
        let b = new_object_id();
        assert_ne!(a, b, "object ids must be unique");
    }

    #[test]
    fn threshold_exposed() {
        let d = TransientDetector::load_statistical().unwrap();
        assert!(d.threshold() > 0.0 && d.threshold() <= 1.0);
    }

    #[test]
    fn no_false_trigger_on_clean_sky() {
        let d = TransientDetector::load_statistical().unwrap();
        let frame = synthetic_frame(64, 64, 42);
        // Warm up the baseline (must exceed window/2).
        for i in 0..40 {
            let _ = d.detect(&synthetic_frame(64, 64, i as u64 + 1), 0.0, 0.0, "k");
        }
        // A normal frame must not alert.
        assert!(d.detect(&frame, 0.0, 0.0, "k").is_none());
    }

    #[test]
    fn transient_triggers_alert() {
        let d = TransientDetector::load_statistical().unwrap();
        for i in 0..40 {
            let _ = d.detect(&synthetic_frame(64, 64, i as u64 + 1), 0.0, 0.0, "k");
        }
        let clean = synthetic_frame(64, 64, 99);
        let transient = inject_transient(&clean, 64, 32, 32, 4, 0.7);
        let alert = d.detect(&transient, 0.0, 0.0, "k");
        assert!(alert.is_some(), "bright transient must raise a ToO alert");
        let a = alert.unwrap();
        assert!(a.confidence >= d.threshold());
        assert!(a.mag_delta < 0.0, "brightening should be negative mag_delta");
    }
}
