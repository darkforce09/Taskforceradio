//! Unit tests for `Compressor`: unity gain below the threshold, the steady-state reduction, the
//! measured attack and release times, the stereo link and the `i16` path.

#![allow(
    clippy::cast_possible_truncation,
    clippy::float_cmp,
    reason = "unity gain below the threshold is exact, so the tests compare bit for bit, and the \
              reference i16 conversion is the original's truncating cast"
)]

use super::*;
use crate::dynamics::error::Error;

/// 20 · log10 of a raw sample value, as the compressor measures it.
fn level_db(sample: f64) -> f64 {
    20.0 * (sample + DC_OFFSET).log10()
}

/// The gain reduction in decibels the compressor applied to a frame `(input, _)`.
fn reduction_db(compressor: &mut Compressor, input: f64) -> f64 {
    let (output, _) = compressor.process_frame(input, 0.0);
    20.0 * (output / input).log10()
}

/// The first frame index at which `reached` holds, feeding `input` frame after frame.
fn frames_until(
    compressor: &mut Compressor,
    input: f64,
    limit: usize,
    reached: impl Fn(&Compressor) -> bool,
) -> usize {
    (1..=limit)
        .find(|_| {
            compressor.process_frame(input, input);
            reached(compressor)
        })
        .unwrap()
}

fn stereo_block(samples: &[i16]) -> StereoBuffer {
    let mut buffer = StereoBuffer::with_frames(samples.len() / 2);
    buffer.as_mut_slice().copy_from_slice(samples);
    buffer
}

#[test]
fn quiet_input_passes_unchanged() {
    for settings in [
        CompressorSettings::VERSION_0_9,
        CompressorSettings::VERSION_1_0,
    ] {
        let mut compressor = Compressor::new(settings).unwrap();
        let below = 10.0_f64.powf(settings.threshold_db / 20.0) - 1.0;
        for (left, right) in [(0.0, 0.0), (below, -below), (-123.0, 456.0), (1.0, -1.0)] {
            assert_eq!(compressor.process_frame(left, right), (left, right));
        }
        assert_eq!(compressor.over_threshold_db(), 0.0);
    }
}

#[test]
fn quiet_i16_blocks_pass_unchanged() {
    let mut compressor = Compressor::new(CompressorSettings::VERSION_1_0).unwrap();
    let samples: Vec<i16> = (-9_999..=9_999).step_by(7).collect();
    let samples = [samples.as_slice(), &[0]].concat();
    let mut buffer = stereo_block(&samples);
    compressor.process_stereo_i16(&mut buffer);
    assert_eq!(buffer.as_slice(), samples.as_slice());
}

#[test]
fn loud_input_settles_at_the_threshold_plus_ratio_times_the_excess() {
    for settings in [
        CompressorSettings::VERSION_0_9,
        CompressorSettings::VERSION_1_0,
    ] {
        let mut compressor = Compressor::new(settings).unwrap();
        let input = 32_767.0;
        let mut output = 0.0;
        for _ in 0..48_000 {
            (output, _) = compressor.process_frame(input, input);
        }
        let over_db = level_db(input) - settings.threshold_db;
        let expected_db = settings.threshold_db + settings.ratio * over_db;
        assert!(
            (level_db(output) - expected_db).abs() < 1e-9,
            "{settings:?}"
        );
        assert!((compressor.over_threshold_db() - over_db).abs() < 1e-9);
    }
}

#[test]
fn the_steady_state_reduction_of_full_scale_is_nine_tenths_of_the_excess() {
    // 1.0: 20·log10(32767) = 90.309 dB, 10.309 dB over 80 dB, reduced by 9.278 dB to 11 260.
    let mut compressor = Compressor::new(CompressorSettings::VERSION_1_0).unwrap();
    let mut reduction = 0.0;
    for _ in 0..48_000 {
        reduction = reduction_db(&mut compressor, 32_767.0);
    }
    assert!((reduction + 0.9 * (level_db(32_767.0) - 80.0)).abs() < 1e-9);
    let (output, _) = compressor.process_frame(32_767.0, 0.0);
    assert!((output - 11_260.1).abs() < 0.1, "{output}");
}

#[test]
fn the_measured_attack_time_is_one_millisecond() {
    let settings = CompressorSettings::VERSION_1_0;
    let mut compressor = Compressor::new(settings).unwrap();
    let input = 32_767.0;
    let target_db = level_db(input) - settings.threshold_db;
    let threshold = (1.0 - (-1.0_f64).exp()) * target_db;
    let frames = frames_until(&mut compressor, input, 48_000, |compressor| {
        compressor.over_threshold_db() >= threshold - 1e-9
    });
    // 1 ms at 48 kHz is 48 frames to 1 − 1/e of the step.
    assert_eq!(frames, 48);
}

#[test]
fn the_measured_release_time_is_three_hundred_milliseconds() {
    let settings = CompressorSettings::VERSION_1_0;
    let mut compressor = Compressor::new(settings).unwrap();
    for _ in 0..48_000 {
        compressor.process_frame(32_767.0, 32_767.0);
    }
    let start_db = compressor.over_threshold_db();
    let threshold = (-1.0_f64).exp() * start_db;
    let frames = frames_until(&mut compressor, 0.0, 96_000, |compressor| {
        compressor.over_threshold_db() <= threshold + 1e-9
    });
    // 300 ms at 48 kHz is 14 400 frames to 1/e of the step.
    assert_eq!(frames, 14_400);
}

#[test]
fn the_gain_is_stereo_linked_on_the_louder_channel() {
    let mut linked = Compressor::new(CompressorSettings::VERSION_0_9).unwrap();
    let mut swapped = Compressor::new(CompressorSettings::VERSION_0_9).unwrap();
    for _ in 0..4_800 {
        let (loud, quiet) = linked.process_frame(30_000.0, 100.0);
        let (quiet_swapped, loud_swapped) = swapped.process_frame(-100.0, -30_000.0);
        assert!((loud / 30_000.0 - quiet / 100.0).abs() < 1e-15);
        assert_eq!(loud, -loud_swapped);
        assert_eq!(quiet, -quiet_swapped);
    }
    let (_, quiet) = linked.process_frame(30_000.0, 100.0);
    assert!(
        quiet < 50.0,
        "the quiet channel shares the loud one's reduction: {quiet}"
    );
}

#[test]
fn the_0_9_threshold_compresses_a_level_the_1_0_threshold_leaves_alone() {
    let level = 3_162.0; // about 70 dB
    let mut version_0_9 = Compressor::new(CompressorSettings::VERSION_0_9).unwrap();
    let mut version_1_0 = Compressor::new(CompressorSettings::VERSION_1_0).unwrap();
    for _ in 0..4_800 {
        version_0_9.process_frame(level, level);
        version_1_0.process_frame(level, level);
    }
    // 5 dB over 65 dB, reduced by 4.5 dB.
    let expected = level * 10.0_f64.powf(-0.9 * (level_db(level) - 65.0) / 20.0);
    let (compressed, _) = version_0_9.process_frame(level, level);
    assert!(
        (compressed - expected).abs() < 1e-9 * level,
        "{compressed} vs {expected}"
    );
    assert!(compressed < 0.6 * level);
    assert_eq!(version_1_0.process_frame(level, level), (level, level));
}

#[test]
fn the_i16_path_truncates_each_compressed_frame_toward_zero() {
    let mut block_compressor = Compressor::new(CompressorSettings::VERSION_0_9).unwrap();
    let mut frame_compressor = Compressor::new(CompressorSettings::VERSION_0_9).unwrap();
    let samples: Vec<i16> = (0..4_800)
        .flat_map(|index: i16| [index.wrapping_mul(97), -(index % 300) * 109])
        .collect();
    let mut buffer = stereo_block(&samples);
    block_compressor.process_stereo_i16(&mut buffer);
    for (frame, output) in samples
        .as_chunks::<2>()
        .0
        .iter()
        .zip(buffer.as_slice().as_chunks::<2>().0)
    {
        let (left, right) =
            frame_compressor.process_frame(f64::from(frame[0]), f64::from(frame[1]));
        assert_eq!(output, &[left.trunc() as i16, right.trunc() as i16]);
    }
}

#[test]
fn the_i16_path_saturates_instead_of_wrapping_under_expansion() {
    let expander = CompressorSettings {
        ratio: 2.0,
        ..CompressorSettings::VERSION_1_0
    };
    let mut compressor = Compressor::new(expander).unwrap();
    let mut buffer = stereo_block(&[30_000, i16::MIN].repeat(4_800));
    compressor.process_stereo_i16(&mut buffer);
    let last = &buffer.as_slice()[buffer.as_slice().len() - 2..];
    assert_eq!(last, &[i16::MAX, i16::MIN]);
}

#[test]
fn reset_puts_the_envelope_back_at_rest() {
    let mut compressor = Compressor::new(CompressorSettings::VERSION_1_0).unwrap();
    for _ in 0..480 {
        compressor.process_frame(32_767.0, 32_767.0);
    }
    assert!(compressor.over_threshold_db() > 1.0);
    compressor.reset();
    assert_eq!(compressor.over_threshold_db(), 0.0);
    assert_eq!(
        compressor.process_frame(5_000.0, -5_000.0),
        (5_000.0, -5_000.0)
    );
}

#[test]
fn invalid_settings_are_refused_and_valid_ones_kept() {
    let bad = CompressorSettings {
        attack_ms: 0.0,
        ..CompressorSettings::VERSION_1_0
    };
    assert!(matches!(
        Compressor::new(bad),
        Err(Error::InvalidSetting {
            setting: "attack_ms",
            ..
        })
    ));
    let compressor = Compressor::new(CompressorSettings::VERSION_0_9).unwrap();
    assert_eq!(compressor.settings(), &CompressorSettings::VERSION_0_9);
}
