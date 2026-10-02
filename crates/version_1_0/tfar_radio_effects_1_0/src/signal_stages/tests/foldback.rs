//! Unit tests for `foldback` and `foldback_threshold` in `tfar_radio_effects_1_0`.

#![allow(
    clippy::cast_precision_loss,
    reason = "the test steps are small integers, exact as f32"
)]

use super::*;

#[test]
fn a_sample_inside_the_threshold_passes_unchanged() {
    assert_eq!(foldback(0.2, 0.25), 0.2);
    assert_eq!(foldback(-0.25, 0.25), -0.25);
}

#[test]
fn a_sample_outside_the_threshold_folds_back_inside() {
    assert!((foldback(0.4, 0.25) - 0.1).abs() < 1e-6);
    assert!((foldback(-0.4, 0.25) + 0.1).abs() < 1e-6);
    for step in -400..=400 {
        let input = step as f32 / 100.0;
        assert!(foldback(input, 0.25).abs() <= 0.25 + 1e-6, "{input}");
    }
}

#[test]
fn a_threshold_below_the_floor_gives_silence() {
    assert_eq!(foldback(0.5, 0.000_009), 0.0);
    assert_eq!(foldback(0.0, 0.0), 0.0);
}

#[test]
fn the_threshold_is_point_three_of_the_level_scaled_by_the_error() {
    let block = [0.005_f32, -0.005, 0.005, -0.005];
    assert!((foldback_threshold(&block, 0.0) - 0.3).abs() < 1e-6);
    assert!((foldback_threshold(&block, 0.5) - 0.15).abs() < 1e-6);
    assert_eq!(foldback_threshold(&[], 0.0), 0.0);
    assert_eq!(foldback_threshold(&[0.0; 8], 0.0), 0.0);
}
