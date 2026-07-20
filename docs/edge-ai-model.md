# Edge AI Transient Model — Contract & Accuracy Validation

**Component:** `edge-agent/crates/edge-ai` · **Owner:** TPT Solutions

TPT AstroLink runs a brightness-anomaly detector on every captured frame at
the edge (Raspberry Pi 5 / Intel NUC). When an anomaly exceeds the
configured confidence threshold it emits a **Target-of-Opportunity (ToO)**
alert (`too` MQTT event) that is forwarded Cloud Core → Client UI.

## Two execution paths

1. **ONNX inference (preferred in production).** When a model file exists
   at `$TPT_TRANSIENT_MODEL` (default `models/transient/default/transient.onnx`),
   it is loaded with ONNX Runtime via the `ort` crate and run on the
   feature vector below.
2. **Statistical fallback (default out of the box).** With no model
   deployed, a robust rolling-baseline brightness-anomaly scorer
   (`StatisticalDetector`) provides ToO triggering without any trained
   weights. It is also the reference scorer for the accuracy harness.

Both paths expose the same `TransientDetector::detect` entry point, so the
command bus and Client UI are agnostic to which is active.

## Model I/O contract

| Item | Spec |
|------|------|
| Input name | `input` (or first model input) |
| Input shape | `[1, 8]` float32 |
| Output name | `output` (or first model output) |
| Output shape | `[1, 2]` float32 |
| Output `[0]` | `confidence` ∈ [0, 1] |
| Output `[1]` | `mag_delta` (apparent-magnitude change vs. baseline; negative = brightened) |

### Input features (length 8, order-sensitive)

`[ mean, median, std, max, min, p95, peak_count, snr ]`

- All derived from the normalized (0.0..=1.0) frame pixel buffer.
- `peak_count`: pixels exceeding `median + 5σ` (resolved-source proxy).
- `snr`: `max / std`.
- Computed by `extract_features` in `crates/edge-ai/src/lib.rs`.

## Training guidance

Train a small regressor/classifier (e.g. a 2-hidden-layer MLP, < 50 kB)
on labeled frame feature vectors:

- **Positives:** frames containing injected/vetted transients
  (supernovae, flaring stars, unknown moving objects) with the
  `mag_delta` ground truth from the reference catalog.
- **Negatives:** clean sky, clouds, airplane/satellite trails, hot pixels.
- Target the 0.9 recall / < 5% false-positive operating point used by
  the validation harness.
- Export to ONNX (opset ≥ 17) and ship at
  `models/transient/<version>/transient.onnx`; sign the artifact and
  distribute via the OTA update channel (`crates/update`).

## Accuracy validation

`cargo test -p tpt-edge-ai --test accuracy -- --nocapture`

Builds a baseline on clean synthetic sky, then measures the confusion
matrix over 400 normal + 400 transient frames (varied position and
0.45–0.9 brightness). Asserts **recall ≥ 0.9** and
**false-positive rate < 0.05**. The statistical fallback currently
passes both on the synthetic suite; the ONNX model must match or exceed
this before promotion to a node fleet.

### Robustness notes (statistical scorer)

- Uses **median + MAD** (not mean/std) so a single extreme transient
  cannot poison the baseline.
- Detected anomalies are **not** ingested into the baseline, so a run of
  consecutive transients remains detectable.
- Baseline warms up on the first `window/2` frames before scoring.
