//! Unit tests for `AirborneRadioEffect` in `tfar_radio_effects_1_0`.

use tfar_dsp_filters::MonoFilter;

use super::*;
use crate::signal_measurements::{
    BLOCK_FRAMES, assert_band_shape, assert_error_one_spreads_the_tone,
    assert_silence_stays_silent, band_edge_attenuations, first_sound,
};
use crate::signal_stages::{foldback, foldback_threshold, narrow_to_f32};

/// The band centre: the geometric mean of the speaker corners, 1000 Hz and 4000 Hz.
const CENTRE_HZ: f64 = 2000.0;

/// The block size of the impulse test: a size at which the folded impulse is not 0.
const IMPULSE_BLOCK_FRAMES: usize = 500;

/// The original's speaker high-pass: RBJ `setup(48000, 1000, 1.0)`.
const ORIGINAL_SPEAKER_HIGH_PASS: FilterDesign = FilterDesign::RbjHighPass {
    cutoff_hz: 1000.0,
    quality: 1.0,
};

/// The original's speaker low-pass: RBJ `setup(48000, 4000, 1.0)`.
const ORIGINAL_SPEAKER_LOW_PASS: FilterDesign = FilterDesign::RbjLowPass {
    cutoff_hz: 4000.0,
    quality: 1.0,
};

#[test]
fn silence_in_gives_silence_out() {
    for distance_error in [0.0, 0.5, 1.0] {
        assert_silence_stays_silent(&mut AirborneRadioEffect::new().unwrap(), distance_error);
    }
}

#[test]
fn an_impulse_comes_out_2399_samples_late_through_the_speaker_filters() {
    let mut effect = AirborneRadioEffect::new().unwrap();
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
    assert_error_one_spreads_the_tone(|| Box::new(AirborneRadioEffect::new().unwrap()), 2000.0);
}

#[test]
fn the_band_runs_from_1000_hz_to_4000_hz() {
    let attenuations = band_edge_attenuations(
        || Box::new(AirborneRadioEffect::new().unwrap()),
        &[ORIGINAL_SPEAKER_HIGH_PASS, ORIGINAL_SPEAKER_LOW_PASS],
        CENTRE_HZ,
    );
    assert_band_shape(attenuations);
}

#[test]
fn an_empty_block_is_left_alone() {
    let mut effect = AirborneRadioEffect::new().unwrap();
    effect.process(&mut []);
    let mut block = vec![0.0_f32; BLOCK_FRAMES];
    effect.process(&mut block);
    assert_eq!(first_sound(&block), None);
}
