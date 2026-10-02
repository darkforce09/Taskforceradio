//! Unit tests for `PersonalRadioEffect` in `tfar_radio_effects_0_9`.

use super::*;
use crate::signal_measurements::{
    assert_band_shape, assert_error_one_spreads_the_tone, assert_silence_stays_silent,
    band_edge_attenuations, first_sound,
};

/// The band centre: the geometric mean of the microphone corners, the narrower band.
const CENTRE_HZ: f64 = 1643.0;

/// The original's microphone high-pass: RBJ `setup(48000, 900, 0.85)`.
const ORIGINAL_MICROPHONE_HIGH_PASS: FilterDesign = FilterDesign::RbjHighPass {
    cutoff_hz: 900.0,
    quality: 0.85,
};

/// The original's microphone low-pass: RBJ `setup(48000, 3000, 2.0)`.
const ORIGINAL_MICROPHONE_LOW_PASS: FilterDesign = FilterDesign::RbjLowPass {
    cutoff_hz: 3000.0,
    quality: 2.0,
};

/// The original's speaker high-pass: RBJ `setup(48000, 520, 0.97)`.
const ORIGINAL_SPEAKER_HIGH_PASS: FilterDesign = FilterDesign::RbjHighPass {
    cutoff_hz: 520.0,
    quality: 0.97,
};

/// The original's speaker low-pass: RBJ `setup(48000, 4300, 2.0)`.
const ORIGINAL_SPEAKER_LOW_PASS: FilterDesign = FilterDesign::RbjLowPass {
    cutoff_hz: 4300.0,
    quality: 2.0,
};

#[test]
fn silence_in_gives_silence_out() {
    for distance_error in [0.0, 0.5, 1.0] {
        assert_silence_stays_silent(&mut PersonalRadioEffect::new().unwrap(), distance_error);
    }
}

#[test]
fn an_impulse_first_sounds_2399_samples_late() {
    let mut effect = PersonalRadioEffect::new().unwrap();
    effect.set_error_level(0.0);
    let mut block = vec![0.0_f32; 4000];
    block[0] = 1.0;
    effect.process(&mut block);
    assert_eq!(first_sound(&block), Some(2399));
}

#[test]
fn error_one_sounds_measurably_different_from_error_zero() {
    assert_error_one_spreads_the_tone(|| Box::new(PersonalRadioEffect::new().unwrap()), 1500.0);
}

#[test]
fn the_band_is_the_microphone_band_inside_the_speaker_band() {
    let attenuations = band_edge_attenuations(
        || Box::new(PersonalRadioEffect::new().unwrap()),
        &[
            ORIGINAL_MICROPHONE_HIGH_PASS,
            ORIGINAL_MICROPHONE_LOW_PASS,
            ORIGINAL_SPEAKER_HIGH_PASS,
            ORIGINAL_SPEAKER_LOW_PASS,
        ],
        CENTRE_HZ,
    );
    assert_band_shape(attenuations);
}
