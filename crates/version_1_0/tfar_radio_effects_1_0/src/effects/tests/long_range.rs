//! Unit tests for `LongRangeRadioEffect` in `tfar_radio_effects_1_0`.

use tfar_dsp_filters::MonoFilter;

use super::*;
use crate::error_level::error_level_from_distance;
use crate::signal_measurements::{
    BLOCK_FRAMES, SINE_AMPLITUDE, assert_band_shape, assert_error_one_spreads_the_tone,
    assert_silence_stays_silent, band_edge_attenuations, first_sound, process_in_blocks, sine,
};
use crate::signal_stages::{DelayLine, RingModulator, foldback, foldback_threshold, narrow_to_f32};

/// The band centre: the geometric mean of the speaker corners.
const CENTRE_HZ: f64 = 822.0;

/// The block size of the impulse test: a size at which the folded impulse is not 0.
const IMPULSE_BLOCK_FRAMES: usize = 500;

/// The original's speaker high-pass: RBJ `setup(48000, 520, 0.97)`.
const ORIGINAL_SPEAKER_HIGH_PASS: FilterDesign = FilterDesign::RbjHighPass {
    cutoff_hz: 520.0,
    quality: 0.97,
};

/// The original's speaker low-pass: RBJ `setup(48000, 1300, 1.0)`.
const ORIGINAL_SPEAKER_LOW_PASS: FilterDesign = FilterDesign::RbjLowPass {
    cutoff_hz: 1300.0,
    quality: 1.0,
};

#[test]
fn silence_in_gives_silence_out() {
    for distance_error in [0.0, 0.5, 1.0] {
        assert_silence_stays_silent(&mut LongRangeRadioEffect::new().unwrap(), distance_error);
    }
}

#[test]
fn an_impulse_comes_out_2399_samples_late_through_the_speaker_filters() {
    let mut effect = LongRangeRadioEffect::new().unwrap();
    effect.set_error_level(0.0);
    let mut signal = vec![0.0_f32; IMPULSE_BLOCK_FRAMES * 8];
    signal[0] = 0.5;
    let folded = foldback(
        0.5,
        foldback_threshold(&signal[..IMPULSE_BLOCK_FRAMES], 0.0),
    ) * 30.0;
    assert!(folded.abs() > 1e-3, "the folded impulse is {folded}");
    for block in signal.chunks_mut(IMPULSE_BLOCK_FRAMES) {
        effect.process(block);
    }

    let mut high_pass = MonoFilter::new(ORIGINAL_SPEAKER_HIGH_PASS, 48_000.0).unwrap();
    let mut low_pass = MonoFilter::new(ORIGINAL_SPEAKER_LOW_PASS, 48_000.0).unwrap();
    let expected: Vec<f32> = (0..signal.len() - 2399)
        .map(|index| {
            let input = if index == 0 { folded } else { 0.0 };
            let high = narrow_to_f32(high_pass.process_sample(f64::from(input)));
            narrow_to_f32(low_pass.process_sample(f64::from(high)))
        })
        .collect();
    assert_eq!(first_sound(&signal), Some(2399));
    for (index, (actual, wanted)) in signal[2399..].iter().zip(&expected).enumerate() {
        assert!(
            (actual - wanted).abs() < 1e-6,
            "sample {index}: {actual} against {wanted}"
        );
    }
}

#[test]
fn error_one_sounds_measurably_different_from_error_zero() {
    assert_error_one_spreads_the_tone(|| Box::new(LongRangeRadioEffect::new().unwrap()), 1000.0);
}

#[test]
fn the_band_runs_from_520_hz_to_1300_hz() {
    let attenuations = band_edge_attenuations(
        || Box::new(LongRangeRadioEffect::new().unwrap()),
        &[ORIGINAL_SPEAKER_HIGH_PASS, ORIGINAL_SPEAKER_LOW_PASS],
        CENTRE_HZ,
    );
    assert_band_shape(attenuations);
}

#[test]
fn an_empty_block_is_left_alone() {
    let mut effect = LongRangeRadioEffect::new().unwrap();
    effect.process(&mut []);
    let mut block = vec![0.0_f32; BLOCK_FRAMES];
    effect.process(&mut block);
    assert_eq!(first_sound(&block), None);
}

#[test]
fn a_sound_delayed_into_a_silent_block_still_plays() {
    let mut effect = LongRangeRadioEffect::new().unwrap();
    let mut signal = vec![0.0_f32; 4000];
    signal[0] = 0.5;
    process_in_blocks(&mut effect, &mut signal, 1000);
    assert_eq!(first_sound(&signal), Some(2399));
}

#[test]
fn the_stages_run_in_the_original_order() {
    let mut effect = LongRangeRadioEffect::new().unwrap();
    effect.set_error_level(1.0);
    let error_level = narrow_to_f32(error_level_from_distance(1.0));
    let mut delay_line = DelayLine::new();
    let mut ring_modulator = RingModulator::default();
    let mut high_pass = MonoFilter::new(ORIGINAL_SPEAKER_HIGH_PASS, 48_000.0).unwrap();
    let mut low_pass = MonoFilter::new(ORIGINAL_SPEAKER_LOW_PASS, 48_000.0).unwrap();
    let mut signal = sine(700.0, SINE_AMPLITUDE, 6000);
    let mut expected = signal.clone();
    for block in expected.chunks_mut(1000) {
        let threshold = foldback_threshold(block, error_level);
        for sample in block.iter_mut() {
            let folded = foldback(*sample, threshold) * 30.0;
            let modulated = ring_modulator.modulate(delay_line.delay(folded), error_level);
            let high = narrow_to_f32(high_pass.process_sample(f64::from(modulated)));
            *sample = narrow_to_f32(low_pass.process_sample(f64::from(high)));
        }
    }
    process_in_blocks(&mut effect, &mut signal, 1000);
    assert!(signal[2399..].iter().any(|sample| sample.abs() > 1e-4));
    for (index, (actual, wanted)) in signal.iter().zip(&expected).enumerate() {
        assert!(
            (actual - wanted).abs() < 1e-6,
            "sample {index}: {actual} against {wanted}"
        );
    }
}
