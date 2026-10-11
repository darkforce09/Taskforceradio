//! `tfar_audio::dynamics` against SimpleComp: the 0.9 and 1.0 presets over speech-like signals (an
//! amplitude-modulated noise burst and level steps), comparing the gain-reduction curve, the
//! compressed frames and the stereo `i16` path.

#![allow(
    clippy::cast_possible_truncation,
    reason = "the test signals are turned into i16 samples on purpose, within the i16 range"
)]
#![allow(
    clippy::unwrap_used,
    reason = "helpers outside the test functions stop the test on a failed call, as the tests do"
)]

use tfar_audio::dynamics::{Compressor, CompressorSettings};
use tfar_audio::sample_buffer::StereoBuffer;
use tfar_golden_vectors::{
    OracleCompressor, OracleCompressorSettings, SAMPLE_RATE_HZ, level_steps, modulated_noise_burst,
};

/// How far our gain may lie from SimpleComp's, frame by frame.
const GAIN_TOLERANCE: f64 = 1.0e-6;

fn oracle_settings(settings: &CompressorSettings) -> OracleCompressorSettings {
    OracleCompressorSettings {
        sample_rate_hz: settings.sample_rate_hz,
        threshold_db: settings.threshold_db,
        ratio: settings.ratio,
        attack_ms: settings.attack_ms,
        release_ms: settings.release_ms,
    }
}

fn presets() -> [(&'static str, CompressorSettings); 2] {
    [
        ("0.9", CompressorSettings::VERSION_0_9),
        ("1.0", CompressorSettings::VERSION_1_0),
    ]
}

/// Two seconds of a stereo speech-like burst on the raw `i16` scale: 4 Hz syllables of noise,
/// the right channel quieter and independent.
fn speech_burst() -> Vec<[f64; 2]> {
    let left = modulated_noise_burst(96_000, 30_000.0, 4.0, SAMPLE_RATE_HZ, 41);
    let right = modulated_noise_burst(96_000, 12_000.0, 4.0, SAMPLE_RATE_HZ, 42);
    left.into_iter()
        .zip(right)
        .map(|(left, right)| [left, right])
        .collect()
}

/// Level steps across the thresholds, a quarter second each, the right channel at half level.
fn steps() -> Vec<[f64; 2]> {
    let levels = [
        100.0, 30_000.0, 1_000.0, 20_000.0, 3_000.0, 32_000.0, 10.0, 8_000.0,
    ];
    level_steps(&levels, 12_000)
        .into_iter()
        .map(|sample| [sample, sample * 0.5])
        .collect()
}

/// Our gain of every frame, read off the louder channel, and our compressed frames.
fn our_run(settings: CompressorSettings, frames: &[[f64; 2]]) -> (Vec<f64>, Vec<[f64; 2]>) {
    let mut compressor = Compressor::new(settings).unwrap();
    let mut gains = Vec::with_capacity(frames.len());
    let mut compressed = Vec::with_capacity(frames.len());
    for [left, right] in frames {
        let (out_left, out_right) = compressor.process_frame(*left, *right);
        let gain = if *left == 0.0 && *right == 0.0 {
            // A silent frame shows no gain; the comparison skips it.
            f64::NAN
        } else if left.abs() >= right.abs() {
            out_left / left
        } else {
            out_right / right
        };
        gains.push(gain);
        compressed.push([out_left, out_right]);
    }
    (gains, compressed)
}

/// Compares our gain curve and frames with SimpleComp's over `frames` for both presets.
fn check_gain_curve(signal_name: &str, frames: &[[f64; 2]]) {
    let mut failures = Vec::new();
    for (preset, settings) in presets() {
        let original = OracleCompressor::new(oracle_settings(&settings))
            .unwrap()
            .process_frames(frames)
            .unwrap();
        let (our_gains, our_frames) = our_run(settings, frames);
        let worst_gain = original
            .gains
            .iter()
            .zip(&our_gains)
            .zip(frames)
            .filter(|(_, [left, right])| *left != 0.0 || *right != 0.0)
            .map(|((expected, actual), _)| (expected - actual).abs())
            .fold(0.0_f64, f64::max);
        let worst_sample = original
            .frames
            .iter()
            .zip(&our_frames)
            .flat_map(|(expected, actual)| {
                [
                    (expected[0] - actual[0]).abs(),
                    (expected[1] - actual[1]).abs(),
                ]
            })
            .fold(0.0_f64, f64::max);
        let least_gain = original.gains.iter().copied().fold(1.0_f64, f64::min);
        println!(
            "{signal_name}, preset {preset}: worst gain difference {worst_gain:e}, worst sample \
             difference {worst_sample:e}, deepest SimpleComp gain {least_gain:.4}"
        );
        if worst_gain.is_nan() || worst_gain > GAIN_TOLERANCE {
            failures.push(format!("preset {preset}: gain difference {worst_gain:e}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{signal_name}:\n{}",
        failures.join("\n")
    );
}

#[test]
fn both_presets_follow_simplecomps_gain_curve_on_a_speech_like_burst() {
    check_gain_curve("speech burst", &speech_burst());
}

#[test]
fn both_presets_follow_simplecomps_gain_curve_on_level_steps() {
    check_gain_curve("level steps", &steps());
}

#[test]
fn the_signals_drive_both_presets_into_gain_reduction() {
    // Without reduction the curve comparison would hold trivially.
    for (preset, settings) in presets() {
        for frames in [speech_burst(), steps()] {
            let (gains, _) = our_run(settings, &frames);
            let least = gains.iter().copied().fold(1.0_f64, f64::min);
            assert!(least < 0.9, "preset {preset}: deepest gain {least}");
        }
    }
}

#[test]
fn both_presets_match_process_compressor_on_stereo_i16_blocks() {
    let mut failures = Vec::new();
    for (preset, settings) in presets() {
        for (name, frames) in [("speech burst", speech_burst()), ("level steps", steps())] {
            let input: Vec<i16> = frames
                .iter()
                .flat_map(|[left, right]| [*left as i16, *right as i16])
                .collect();
            let mut original = input.clone();
            OracleCompressor::new(oracle_settings(&settings))
                .unwrap()
                .process_stereo_i16(&mut original)
                .unwrap();
            let mut buffer = StereoBuffer::with_frames(frames.len());
            buffer.as_mut_slice().copy_from_slice(&input);
            Compressor::new(settings)
                .unwrap()
                .process_stereo_i16(&mut buffer);
            let differing = original
                .iter()
                .zip(buffer.as_slice())
                .filter(|(expected, actual)| expected != actual)
                .count();
            println!(
                "{name}, preset {preset}: {differing} of {} i16 samples differ",
                input.len()
            );
            if differing > 0 {
                failures.push(format!(
                    "{name}, preset {preset}: {differing} samples differ"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
