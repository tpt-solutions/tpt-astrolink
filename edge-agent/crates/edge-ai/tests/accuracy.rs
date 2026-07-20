// Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

//! Model-accuracy validation harness for the edge transient detector.
//!
//! Builds a baseline on clean synthetic sky, then measures the detector's
//! confusion matrix over a labeled dataset of normal vs. transient frames.
//! Run with:
//!
//! ```text
//! cargo test -p tpt-edge-ai --test accuracy -- --nocapture
//! ```

use tpt_edge_ai::{inject_transient, synthetic_frame, TransientDetector};

const W: usize = 64;
const H: usize = 64;

fn main() {
    let d = TransientDetector::load_statistical().expect("statistical detector");

    // Warm up the rolling baseline on clean sky.
    for i in 0..40 {
        let _ = d.detect(&synthetic_frame(W, H, i as u64 + 1), 0.0, 0.0, "warmup");
    }

    let mut tp = 0u32;
    let mut fn_ = 0u32;
    let mut fp = 0u32;
    let mut tn = 0u32;

    // 400 normal frames (label = no transient).
    for i in 0..400u32 {
        let frame = synthetic_frame(W, H, 1000 + i as u64);
        if d.detect(&frame, 0.0, 0.0, "normal").is_some() {
            fp += 1;
        } else {
            tn += 1;
        }
    }

    // 400 transient frames (label = transient), varied position/strength.
    for i in 0..400u32 {
        let clean = synthetic_frame(W, H, 5000 + i as u64);
        let cx = 8 + (i as usize % (W - 16));
        let cy = 8 + ((i as usize / 7) % (H - 16));
        let amp = 0.45 + (i as f32 % 10.0) / 20.0; // 0.45..0.9
        let transient = inject_transient(&clean, W, cx, cy, 3, amp);
        if d.detect(&transient, 0.0, 0.0, "transient").is_some() {
            tp += 1;
        } else {
            fn_ += 1;
        }
    }

    let precision = tp as f32 / (tp + fp).max(1) as f32;
    let recall = tp as f32 / (tp + fn_) as f32;
    let fpr = fp as f32 / (fp + tn).max(1) as f32;

    println!("transient detection validation:");
    println!("  TP={tp} FN={fn_} FP={fp} TN={tn}");
    println!("  precision={precision:.3} recall={recall:.3} false-positive-rate={fpr:.3}");

    assert!(recall >= 0.9, "recall below 0.9: {recall:.3}");
    assert!(fpr < 0.05, "false-positive rate above 0.05: {fpr:.3}");
    println!("  PASS: recall >= 0.9 and FPR < 0.05");
}

#[test]
fn accuracy_validation() {
    main();
}
