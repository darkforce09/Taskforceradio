//! Unit tests for `compressor_oracle`: settings checks and SimpleComp's plain properties.

use super::*;

const VERSION_1_0: OracleCompressorSettings = OracleCompressorSettings {
    sample_rate_hz: 48_000.0,
    threshold_db: 80.0,
    ratio: 0.1,
    attack_ms: 1.0,
    release_ms: 300.0,
};

#[test]
fn a_ratio_of_zero_and_an_infinite_threshold_are_refused() {
    let zero_ratio = OracleCompressorSettings {
        ratio: 0.0,
        ..VERSION_1_0
    };
    let infinite_threshold = OracleCompressorSettings {
        threshold_db: f64::INFINITY,
        ..VERSION_1_0
    };
    assert!(OracleCompressor::new(zero_ratio).is_err());
    assert!(OracleCompressor::new(infinite_threshold).is_err());
}

#[test]
fn input_below_the_threshold_passes_with_unity_gain() {
    let compressor = OracleCompressor::new(VERSION_1_0).unwrap();
    let frames = vec![[1_000.0, -2_000.0]; 480];
    let run = compressor.process_frames(&frames).unwrap();
    assert_eq!(run.frames, frames);
    assert!(run.gains.iter().all(|gain| (*gain - 1.0).abs() < 1.0e-12));
}

#[test]
fn loud_input_is_reduced_by_the_same_gain_on_both_channels() {
    let compressor = OracleCompressor::new(VERSION_1_0).unwrap();
    let frames = vec![[30_000.0, 15_000.0]; 4_800];
    let run = compressor.process_frames(&frames).unwrap();
    let [left, right] = run.frames[4_799];
    let gain = run.gains[4_799];
    assert!(gain < 0.5);
    assert!((left - 30_000.0 * gain).abs() < 1.0e-9);
    assert!((right - 15_000.0 * gain).abs() < 1.0e-9);
}

#[test]
fn the_i16_path_truncates_the_compressed_samples() {
    let compressor = OracleCompressor::new(VERSION_1_0).unwrap();
    let mut samples = vec![1_000_i16, -1_000];
    compressor.process_stereo_i16(&mut samples).unwrap();
    assert_eq!(samples, [1_000, -1_000]);
}
