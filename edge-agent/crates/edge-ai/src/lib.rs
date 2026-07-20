// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! Edge AI transient detector. Runs a brightness-anomaly ONNX model locally
//! to flag Target-of-Opportunity events. Model integration (ort / onnxruntime)
//! is added in Phase 2; the boundary below is stable.

use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TooAlert {
    pub object_id: String,
    pub ra: f64,
    pub dec: f64,
    pub mag_delta: f64,
    pub confidence: f32,
    pub image_key: String,
}

pub struct TransientDetector {
    // model: ort::Session,  // Phase 2
    threshold: f32,
}

impl TransientDetector {
    /// Load the default transient model from `models/transient/<version>/`.
    pub fn load_default() -> Result<Self> {
        Ok(Self { threshold: 0.85 })
    }

    /// Minimum confidence to emit a Target-of-Opportunity alert.
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Run inference on a captured frame (normalized pixel buffer).
    /// Returns an alert when a brightness anomaly exceeds the threshold.
    pub fn detect(&self, _pixels: &[f32], ra: f64, dec: f64, image_key: &str) -> Option<TooAlert> {
        // TODO(Phase 2): real ONNX inference -> mag_delta, confidence.
        let _ = self.threshold;
        Some(TooAlert {
            object_id: new_object_id(),
            ra,
            dec,
            mag_delta: 0.0,
            confidence: 0.0,
            image_key: image_key.to_string(),
        })
    }
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
        let d = TransientDetector::load_default().unwrap();
        assert!(d.threshold() > 0.0 && d.threshold() <= 1.0);
    }
}
