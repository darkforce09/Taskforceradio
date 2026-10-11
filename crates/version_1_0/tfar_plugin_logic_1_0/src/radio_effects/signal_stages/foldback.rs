//! Foldback distortion and its level-driven threshold (`foldback` and the `x` term of
//! `SimpleRadioEffect::process`, `reference/tfar_1_0/ts/src/RadioEffect.hpp:93`).

#[cfg(test)]
#[path = "tests/foldback.rs"]
mod tests;

/// Below this threshold the original's foldback returns silence.
const SILENT_THRESHOLD: f64 = 0.000_01;

/// The mean absolute level that makes `x` equal 1.
const LEVEL_BASE: f64 = 0.005;

/// The share of `x` that becomes the threshold at error level 0.
const THRESHOLD_SCALE: f32 = 0.3;

/// The foldback threshold for a block at `error_level`: `0.3 · (1 − error) · x`, with
/// `x = mean(|sample|) / 0.005` over `block`. An empty block gives 0.
pub(crate) fn foldback_threshold(block: &[f32], error_level: f32) -> f32 {
    let Some(count) = sample_count(block) else {
        return 0.0;
    };
    let sum: f64 = block.iter().map(|sample| f64::from(sample.abs())).sum();
    let level = sum / count / LEVEL_BASE;
    let scale = THRESHOLD_SCALE * (1.0 - error_level);
    super::narrow_to_f32(f64::from(scale) * level)
}

/// Folds `input` back inside `±threshold`: `||fmod(in − t, 4t)| − 2t| − t` when it lies outside,
/// unchanged inside; a threshold below 0.00001 gives silence.
pub(crate) fn foldback(input: f32, threshold: f32) -> f32 {
    if f64::from(threshold) < SILENT_THRESHOLD {
        return 0.0;
    }
    if input > threshold || input < -threshold {
        (((input - threshold) % (threshold * 4.0)).abs() - threshold * 2.0).abs() - threshold
    } else {
        input
    }
}

/// The number of samples in `block` as `f64`, `None` when it is empty.
#[expect(
    clippy::cast_precision_loss,
    reason = "a voice block holds far fewer than 2^52 samples, so the count is exact"
)]
fn sample_count(block: &[f32]) -> Option<f64> {
    (!block.is_empty()).then_some(block.len() as f64)
}
